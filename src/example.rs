use zoomies::{Client, ConfigBuilder, Event};

use async_std::io;
use async_std::task;

fn main() -> io::Result<()> {
    task::block_on(async {
        let config = ConfigBuilder::new().finish();
        let client = Client::with_config(config).await?;
        client.inc("zoomies", std::iter::empty::<&str>()).await?;
        let event = Event::new()
            .title("Chungus")
            .text("Big Chungus is back")
            .build()
            .expect("nice");
        client.log(event).await?;
        Ok(())
    })
}
