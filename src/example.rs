use zoomies::{UdpClient, ConfigBuilder, Event, Metric};

use async_std::io;
use async_std::task;

fn main() -> io::Result<()> {
    task::block_on(async {
        let config = ConfigBuilder::new().finish();
        let client = UdpClient::with_config(config).await?;
        client.send(&Metric::Inc::<u32>("zoomies")).await?;
        let event = Event::new()
            .title("Chungus")
            .text("Big Chungus is back")
            .build()
            .expect("nice");
        client.send(&event).await?;
        Ok(())
    })
}
