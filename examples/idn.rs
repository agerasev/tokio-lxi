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
