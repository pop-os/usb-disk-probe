use super::DISK_DIR;

use async_std::{fs, path::Path};
use futures::stream::Stream;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

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
/// ```
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
            match unsafe { Pin::new_unchecked(&mut self.0) }.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Some(res)) => match filter_device(res) {
                    value @ Some(_) => return Poll::Ready(value),
                    None => continue,
                },
                Poll::Ready(None) => return Poll::Ready(None),
            }
        }
    }
}

/// Filter USB devices which are not USB devices
fn filter_device(entry: io::Result<fs::DirEntry>) -> Option<Result<Box<Path>, Error>> {
    transpose(entry.map_err(Error::Iteration), |entry| {
        let path = entry.path();

        let result = transpose(
            path.file_name().ok_or(Error::DeviceWithoutFileName),
            |filename_os| {
                transpose(
                    filename_os.to_str().ok_or(Error::DevicePathNotUtf8),
                    |filename_str| {
                        if is_usb(filename_str) {
                            Some(Ok(()))
                        } else {
                            None
                        }
                    },
                )
            },
        );

        match result {
            Some(Ok(())) => Some(Ok(path.into_boxed_path())),
            Some(Err(why)) => Some(Err(why)),
            None => None,
        }
    })
}

/// Checks if a device is a USB device
fn is_usb(filename: &str) -> bool {
    filename.starts_with("pci-") && filename.contains("-usb-") && filename.ends_with("-0:0:0:0")
}

/// Converts an `Err(E)` to a `Some(Err(E))`, and maps the `Ok(T)` to an `Option<Result<X, E>>`
#[inline]
fn transpose<T, E, X, F: FnOnce(T) -> Option<Result<X, E>>>(
    input: Result<T, E>,
    func: F,
) -> Option<Result<X, E>> {
    match input {
        Ok(value) => func(value),
        Err(why) => Some(Err(why)),
    }
}
