use std::cmp::max;

use anyhow::Result;
use indexmap::IndexMap;

const URI: &str = "https://chromiumdash.appspot.com/fetch_releases";
const BUCKET: &str = "https://commondatastorage.googleapis.com/chromium-browser-asan/";

// [{"channel":"Canary","chromium_main_branch_position":1075413,"hashes":{"angle":"541cdcbf094fd900f085d14cb896f11240393f0c","chromium":"b91adbb57872336e657aed3e4997b736a312246e","dawn":"f9c6633006e84f697996fb72e570114576cc32c3","devtools":"aeb2dc95cce6b337892a9315f2304d8862dc3cf3","pdfium":"76cec9aed8e2293f955f87b68bc7563bae5e6a8f","skia":"3744490336cf54f6a167643a8e40298cd6ce2756","v8":"acf8224030a79d87470640caced05e2a4abae976","webrtc":"46e2d103b4edc76b65a8e71a5de372c213cfb5c3"},"milestone":110,"platform":"Android","previous_version":"110.0.5436.0","time":1669280114778,"version":"110.0.5437.0"}

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Release {
    chromium_main_branch_position: u64,
    channel: String,
}

#[derive(Deserialize, Debug)]
struct Content {
    #[serde(rename = "Key")]
    key: String
}

#[derive(Deserialize, Debug)]
struct ListBucketResult {
    #[serde(rename = "Contents")]
    contents: Vec<Content>
}

fn main() -> Result<()> {
    println!("Fetching release information");
    // let response = reqwest::blocking::get(URI)?;
    // let releases: Vec<Release> = serde_json::from_reader(response)?;
    // let mut channels: IndexMap<String, u64> = IndexMap::new();
    // for release in releases.into_iter() {
    //     let branch_pos = release.chromium_main_branch_position;
    //     channels
    //         .entry(release.channel)
    //         .and_modify(|existing| *existing = max(*existing, branch_pos))
    //         .or_insert(branch_pos);
    // }
    // println!("Latest branch positions: {:?}", channels);
    let response = reqwest::blocking::get(BUCKET)?;
    let bucket_result: ListBucketResult = serde_xml_rs::from_reader(response).unwrap();
    println!("Builds found: {:?}", bucket_result.contents);
    Ok(())
}
