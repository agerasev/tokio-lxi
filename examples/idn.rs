extern crate tokio;
extern crate tokio_lxi;

use tokio::prelude::*;
use tokio_lxi::LxiDevice;


fn main() {
    let addr = "10.0.0.9:5025".parse().unwrap();
    let device = LxiDevice::connect(&addr)
    .and_then(|dev| dev.send(String::from("*IDN?")))
    .and_then(|(dev, _)| dev.receive(String::new()))
    .map(|(_, text)| println!("{}", text))
    .map_err(|e| panic!("{:?}", e));

    tokio::run(device);
}
