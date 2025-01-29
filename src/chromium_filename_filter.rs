// Copyright 2024 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use glob_match::glob_match;
use ripunzip::FilenameFilter;

static PATTERNS: [&str; 11] = [
    "**/*.so",
    "**/chrome",
    "**/en-US.pak",
    "**/*.dat",
    "**/chrome_crash*",
    "**/v8_context_snapshot.bin",
    "**/resources.pak",
    "**/chrome_*_percent.pak",
    "**/nacl_helper",
    "**/Chromium.app/**",
    "**/d8",
];

/// Knows what files are typically necessary for running Chromium.
pub(crate) struct ChromiumFilenameFilter;

impl FilenameFilter for ChromiumFilenameFilter {
    fn should_unzip(&self, filename: &str) -> bool {
        PATTERNS.iter().any(|p| glob_match(p, filename))
    }
}
