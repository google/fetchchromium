// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod builds;
mod releases;

use std::collections::HashSet;
use std::fmt::Write;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;
use builds::BuildSpecification;
use clap::Parser;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use ripunzip::{UnzipOptions, UnzipProgressReporter};

use crate::builds::get_download_uri;

#[derive(Parser, Debug, Clone, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
enum Mode {
    /// Release mode.
    Release,
    /// Debug mode.
    Debug,
}

/// Fetch Chromium builds suitable for reproducing security bugs.
/// This tool will fetch builds for the oldest and newest release channel
/// by default.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The output directory into which to place the files. By default, the
    /// current working directory is used.
    #[arg(short, long, value_name = "DIRECTORY")]
    output_directory: Option<PathBuf>,

    /// The build type (asan, ubsan etc.)
    #[arg(short, long, default_value = "asan")]
    variant: String,

    /// The debug vs release variant
    #[arg(short, long, default_value_t=Mode::Release)]
    mode: Mode,

    /// Fetch all release channels (stable/extended/dev/canary/beta)
    /// instead of, as by default, just the oldest and newest.
    #[arg(short, long, conflicts_with = "channel", conflicts_with = "revision")]
    all_channels: bool,

    /// Fetch just some of the channels (stable/extended/dev/canary/beta)
    #[arg(short, long)]
    channel: Option<Vec<String>>,

    /// Fetch a specific revision instead of all of the main branches
    #[arg(short, long, conflicts_with = "channel")]
    revision: Option<u64>,
}

fn main() -> Result<()> {
    let os = std::env::consts::OS;
    let (platform, bitness_suffix) = match os {
        "macos" => ("mac", None),
        "windows" => ("win32", Some("_x64")),
        _ => (os, None),
    };

    env_logger::init();
    let args = Args::parse();
    let debugness = args.mode.to_string();

    let specification = BuildSpecification {
        build_type: &args.variant,
        platform,
        debugness: &debugness,
        bitness_suffix,
    };

    let mut downloads: IndexMap<u64, Vec<String>> = IndexMap::new();
    if let Some(revision) = args.revision {
        downloads.insert(revision, vec![format!("revision-{revision}")]);
    } else {
        println!("Fetching branch information");
        let mut channels = releases::get_channel_branch_positions()?;
        // Sometimes there are milestones in between, e.g. canary might be
        // 112 but dev might be 114. From the JSON endpoint we use, we have
        // no means to find out their branch position - this might be something
        // we want to look into in future.
        if let Some(channels_wanted) = args.channel {
            let channels_wanted: HashSet<String> = channels_wanted
                .into_iter()
                .map(|c| c.to_lowercase())
                .collect();
            channels.retain(|k, _| channels_wanted.contains(&k.to_lowercase()));
        } else if !args.all_channels {
            // By default, filter to the oldest and newest channel only.
            let oldest_branch_point = channels
                .values()
                .map(|ci| ci.chromium_main_branch_position)
                .min();
            let youngest_branch_point = channels
                .values()
                .map(|ci| ci.chromium_main_branch_position)
                .max();
            let branch_points_wanted: HashSet<_> = oldest_branch_point
                .into_iter()
                .chain(youngest_branch_point.into_iter())
                .collect();
            channels.retain(|_, v| branch_points_wanted.contains(&v.chromium_main_branch_position));
            println!(
                "Because -a was not specified, fetching only the oldest/newest branch points - {}",
                channels.iter().map(|(_, c)| c.milestone).join(", ")
            );
        }
        // Sometimes several channels can relate to the same branch, especially
        // for stable & extended stable. Aggregate them.
        for channel in channels.into_iter() {
            let desc = format!("{}-{}", channel.0, channel.1.milestone);
            downloads
                .entry(channel.1.chromium_main_branch_position)
                .and_modify(|v| v.push(desc.clone()))
                .or_insert(vec![desc]);
        }
    }
    println!(
        "Downloads we need to do: {downloads:?}. Investigating available builds."
    );

    let progress_bar = ProgressBar::new(0);
    progress_bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})\n{msg}")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#-"));

    let errors: Vec<_> = downloads
        .into_par_iter()
        .map(|(branch_point, channel_description)| {
            fetch_build(
                &args.output_directory,
                &specification,
                branch_point,
                &channel_description,
                BumpProgress(&progress_bar),
            )
        })
        .filter_map(Result::err)
        .collect();

    // Output any errors we found on any file
    for error in &errors {
        eprintln!("Error: {error}")
    }
    // Return the first error code, if any.
    errors.into_iter().next().map(Result::Err).unwrap_or(Ok(()))
}

struct BumpProgress<'a>(&'a ProgressBar);

impl<'a> UnzipProgressReporter for BumpProgress<'a> {
    fn total_bytes_expected(&self, expected: u64) {
        self.0.inc_length(expected)
    }

    fn bytes_extracted(&self, count: u64) {
        self.0.inc(count)
    }
}

fn fetch_build(
    output_directory: &Option<PathBuf>,
    specification: &BuildSpecification,
    branch_point: u64,
    channel_descriptions: &[String],
    progress: impl UnzipProgressReporter,
) -> Result<()> {
    // Find the build immediately before the branch point.
    let build = find_a_build_just_before(specification, branch_point)?;
    let uri = get_download_uri(specification, build);
    println!(
        "Channel {channel_descriptions:?}: branch point was {branch_point}, downloading build {build:?} from {uri}"
    );
    let concatenated_descriptions = channel_descriptions.join("_");
    let output_directory = Some(match output_directory.as_ref() {
        None => PathBuf::from(concatenated_descriptions),
        Some(root) => root.join(concatenated_descriptions),
    });
    let unzip_engine = ripunzip::UnzipEngine::for_uri(
        &uri,
        UnzipOptions {
            output_directory,
            single_threaded: false,
        },
        None,
        progress,
        || {},
    )?;
    unzip_engine.unzip()?;
    println!("Completed download from {uri}.");
    Ok(())
}

fn find_a_build_just_before(specification: &BuildSpecification, branch_point: u64) -> Result<u64> {
    // The build listing takes a version prefix, which we want to be as precise as possible,
    // to be quick and because there's a maximum result count. We'll take it digit by digit
    // and keep searching outwards until we find one which is at or below the intended branch
    // point.
    let branch_point_string = format!("{branch_point}");

    for prefix_length in (0..branch_point_string.len()).rev() {
        if let Ok(builds) =
            builds::get_builds(specification, &branch_point_string[0..prefix_length])
        {
            let the_build = builds
                .into_iter()
                .filter(|build| *build <= branch_point)
                .max();
            if let Some(the_build) = the_build {
                return Ok(the_build);
            }
        }
    }
    Err(anyhow!(
        "No matching builds found for {:?} at branch point {}",
        specification,
        branch_point_string
    ))
}
