use usb_disk_probe::stream::UsbDiskProbe;

use futures::stream::StreamExt;

fn main() {
    futures::executor::block_on(async move {
        let mut stream = UsbDiskProbe::new().await.unwrap();
        while let Some(device_result) = stream.next().await {
            let device = device_result.unwrap();
            println!("{}", device.display());
        }
    });
}
