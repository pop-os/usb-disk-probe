use super::DISK_DIR;
use futures::stream::Stream;
use std::{
    io,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::fs;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to open /dev/disk/by-path/ directory for reading")]
    Open(#[source] io::Error),
    #[error("directory iteration error")]
    Iteration(#[source] io::Error),
    #[error("device path lacks a file name")]
    DeviceWithoutFileName,
    #[error("device path is not UTF-8 valid")]
    DevicePathNotUtf8,
}

/// A stream which iterates on USB device paths to find USB storage disks in the system.
///
/// # Example
///
/// ```no_run
/// use usb_disk_probe::stream::UsbDiskProbe;
///
/// use futures::stream::StreamExt;
///
/// fn main() {
///     futures::executor::block_on(async move {
///         let mut stream = UsbDiskProbe::new().await.unwrap();
///         while let Some(device_result) = stream.next().await {
///             let device = device_result.unwrap();
///             println!("{}", device.display());
///         }
///     });
/// }
/// ```
pub struct UsbDiskProbe(fs::ReadDir);

impl UsbDiskProbe {
    pub async fn new() -> Result<Self, Error> {
        fs::read_dir(DISK_DIR)
            .await
            .map(|dir| Self(dir))
            .map_err(Error::Open)
    }
}

impl Stream for UsbDiskProbe {
    type Item = Result<Box<Path>, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        loop {
            match self.0.poll_next_entry(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(Some(res))) => match filter_device(res) {
                    value @ Some(_) => return Poll::Ready(value),
                    None => continue,
                },
                Poll::Ready(Ok(None)) => return Poll::Ready(None),
                Poll::Ready(Err(why)) => return Poll::Ready(Some(Err(Error::Iteration(why)))),
            }
        }
    }
}

/// Filter USB devices which are not USB devices
fn filter_device(entry: fs::DirEntry) -> Option<Result<Box<Path>, Error>> {
    let path = entry.path();

    match path.file_name() {
        Some(filename) => match filename.to_str().ok_or(Error::DevicePathNotUtf8) {
            Ok(filename) => {
                if is_usb(filename) {
                    Some(Ok(path.into_boxed_path()))
                } else {
                    None
                }
            }
            Err(why) => return Some(Err(why)),
        },
        None => return Some(Err(Error::DeviceWithoutFileName)),
    }
}

/// Checks if a device is a USB device
fn is_usb(filename: &str) -> bool {
    filename.starts_with("pci-") && filename.contains("-usb-") && filename.ends_with("-0:0:0:0")
}
