# fetchchromium

[![GitHub](https://img.shields.io/crates/l/fetchchromium)](https://github.com/google/fetchchromium)
[![crates.io](https://img.shields.io/crates/d/fetchchromium)](https://crates.io/crates/fetchchromium)
[![docs.rs](https://docs.rs/fetchchromium/badge.svg)](https://docs.rs/fetchchromium)

A tool to fetch Chromium builds. This is a thin wrapper above the [ripunzip library](https://github.com/google/ripunzip)
which aims to unzip files in parallel as efficiently as possible.

#### Installation and use

`cargo install fetchchromium` then `fetchchromium -h`. Alternatively,
a `.deb` file is available under the "releases" section on github.

#### Development

Release procedure:
1. Revise the version number
2. `cargo publish`
3. Retrieve the latest `.deb` file from the latest CI job
4. Declare a new release and tag on github
5. As you make that release, include the `.deb` file as an artifact.

#### License and usage notes

This is not an officially supported Google product.

<sup>
License

This software is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See LICENSE for details.
</sup>

