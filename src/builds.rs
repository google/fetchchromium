use std::fmt::Display;

use anyhow::Result;
use indexmap::IndexMap;
use serde::Deserialize;

const BUCKET: &str = "https://commondatastorage.googleapis.com/chromium-browser-asan/";

#[derive(Deserialize, Debug)]
struct Content {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "Generation")]
    generation: String,
}

#[derive(Debug)]
pub(crate) struct Generation(String);

impl Display for Generation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Deserialize, Debug)]
struct ListBucketResult {
    #[serde(rename = "Contents")]
    contents: Vec<Content>,
}

pub(crate) struct BuildSpecification<'a> {
    pub(crate) build_type: &'a str,
    pub(crate) platform: &'a str,
    pub(crate) debugness: &'a str,
}

pub(crate) fn format_prefix(specification: &BuildSpecification, version: &str) -> String {
    let build_type = specification.build_type;
    let platform = specification.platform;
    let debugness = specification.debugness;
    format!("{platform}-{debugness}/{build_type}-{platform}-{debugness}-{version}")
}

pub(crate) fn get_builds(
    specification: &BuildSpecification,
    version_prefix: &str,
) -> Result<IndexMap<u64, Generation>> {
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
            version_slice
                .parse::<u64>()
                .ok()
                .map(|ver| (ver, Generation(content.generation)))
        })
        .collect())
}

pub(crate) fn get_download_uri(
    specification: &BuildSpecification,
    version: &(u64, Generation),
) -> String {
    let prefix = format_prefix(specification, &format!("{}", version.0));
    let prefix = url_escape::encode_component(&prefix);
    let generation = &version.1;
    format!("https://www.googleapis.com/download/storage/v1/b/chromium-browser-asan/o/{prefix}.zip&alt=media&generation={generation}")
}
