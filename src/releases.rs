// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use anyhow::Result;
use indexmap::IndexMap;
use serde::Deserialize;

const URI: &str = "https://chromiumdash.appspot.com/fetch_releases";

#[derive(Deserialize, Debug)]
struct Release {
    chromium_main_branch_position: u64,
    milestone: u16,
    channel: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ChannelInfo {
    pub(crate) milestone: u16,
    pub(crate) chromium_main_branch_position: u64,
}

pub(crate) fn get_channel_branch_positions() -> Result<IndexMap<String, ChannelInfo>> {
    let response = reqwest::blocking::get(URI)?;
    let releases: Vec<Release> = serde_json::from_reader(response)?;
    let mut channels: IndexMap<String, ChannelInfo> = IndexMap::new();
    for release in releases.into_iter() {
        let chromium_main_branch_position = release.chromium_main_branch_position;
        let milestone = release.milestone;
        channels
            .entry(release.channel)
            .and_modify(|existing| {
                if milestone > existing.milestone {
                    *existing = ChannelInfo {
                        milestone,
                        chromium_main_branch_position,
                    }
                }
            })
            .or_insert(ChannelInfo {
                milestone,
                chromium_main_branch_position,
            });
    }
    Ok(channels)
}
