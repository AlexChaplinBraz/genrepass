use rustc_version::{version_meta, Channel};

fn main() {
    // This is a workaround for enabling a feature only under nightly
    // and still being able to run everything else as normal.
    let channel = match version_meta().unwrap().channel {
        Channel::Stable => "CHANNEL_STABLE",
        Channel::Beta => "CHANNEL_BETA",
        Channel::Nightly => "CHANNEL_NIGHTLY",
        Channel::Dev => "CHANNEL_DEV",
    };
    println!("cargo:rustc-cfg={channel}")
}
