# usb-disk-probe

[![Crates.io](https://img.shields.io/crates/v/usb-disk-probe)](https://crates.io/crates/usb-disk-probe)

Provides a stream type which can be used to probe for USB storage devices in the system.

## Example

```rust
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
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
