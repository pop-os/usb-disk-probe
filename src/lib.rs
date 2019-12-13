#[macro_use]
extern crate thiserror;

const DISK_DIR: &str = "/dev/disk/by-path/";

/// Contains an async stream-based version of the USB disk prober.
pub mod stream;
