// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use anyhow::Result;
use indexmap::IndexSet;
use serde::Deserialize;

const BUCKET: &str = "https://commondatastorage.googleapis.com/chromium-browser-asan/";

#[derive(Deserialize, Debug)]
struct Content {
    #[serde(rename = "Key")]
    key: String,
}

#[derive(Deserialize, Debug)]
struct ListBucketResult {
    #[serde(rename = "Contents")]
    contents: Vec<Content>,
}

#[derive(Debug)]
pub(crate) struct BuildSpecification<'a> {
    pub(crate) build_type: &'a str,
    pub(crate) platform: &'a str,
    pub(crate) debugness: &'a str,
    pub(crate) bitness_suffix: Option<&'a str>,
}

pub(crate) fn format_prefix(specification: &BuildSpecification, version: &str) -> String {
    let build_type = specification.build_type;
    let platform = specification.platform;
    let debugness = specification.debugness;
    let bitness = specification.bitness_suffix.unwrap_or("");
    format!(
        "{platform}-{debugness}{bitness}/{build_type}-{platform}-{debugness}{bitness}-{version}"
    )
}

pub(crate) fn get_builds(
    specification: &BuildSpecification,
    version_prefix: &str,
) -> Result<IndexSet<u64>> {
    let prefix = format_prefix(specification, version_prefix);
    let uri = format!("{BUCKET}?prefix={prefix}");
    let response = reqwest::blocking::get(uri)?;
    let bucket_result: ListBucketResult = serde_xml_rs::from_reader(response)?;
    let prefix_to_remove = format_prefix(specification, "").len();
    Ok(bucket_result
        .contents
        .into_iter()
        .filter_map(|content| {
            let build_label = content.key;
            let version_slice = &build_label[prefix_to_remove..build_label.len() - 4];
            version_slice.parse::<u64>().ok()
        })
        .collect())
}

pub(crate) fn get_download_uri(specification: &BuildSpecification, version: u64) -> String {
    let prefix = format_prefix(specification, &format!("{version}"));
    let prefix = url_escape::encode_component(&prefix);
    format!("https://chromium-browser-asan.storage.googleapis.com/{prefix}.zip")
}
