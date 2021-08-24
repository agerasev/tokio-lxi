# tokio-lxi

[![Crates.io][crates_badge]][crates]
[![Docs.rs][docs_badge]][docs]
[![Github Actions][github_badge]][github]
[![Appveyor][appveyor_badge]][appveyor]
[![Codecov.io][codecov_badge]][codecov]
[![License][license_badge]][license]

[crates_badge]: https://img.shields.io/crates/v/tokio-lxi.svg
[docs_badge]: https://docs.rs/tokio-lxi/badge.svg
[github_badge]: https://github.com/agerasev/tokio-lxi/actions/workflows/test.yml/badge.svg
[appveyor_badge]: https://ci.appveyor.com/api/projects/status/github/agerasev/tokio-lxi?branch=master&svg=true
[codecov_badge]: https://codecov.io/gh/agerasev/tokio-lxi/graphs/badge.svg
[license_badge]: https://img.shields.io/crates/l/tokio-lxi.svg

[crates]: https://crates.io/crates/tokio-lxi
[docs]: https://docs.rs/tokio-lxi
[github]: https://github.com/agerasev/tokio-lxi/actions/workflows/test.yml
[appveyor]: https://ci.appveyor.com/project/agerasev/tokio-lxi
[codecov]: https://codecov.io/gh/agerasev/tokio-lxi
[license]: #license

LXI protocol abstractions for Tokio with `async`/`.await` support.

## Example

```rust
use tokio;

use tokio_lxi::LxiDevice;

#[tokio::main]
async fn main() -> Result<(), tokio_lxi::Error> {
    let addr = "10.0.0.9:5025".parse().unwrap();
    let mut device = LxiDevice::connect(&addr).await?;
    device.send("*IDN?").await?;
    let reply = device.receive().await?;
    println!("{}", reply);

    Ok(())
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
