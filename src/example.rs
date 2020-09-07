use zoomies::{Client, ConfigBuilder};

use async_std::io;
use async_std::task;

fn main() -> io::Result<()> {
    task::block_on(async {
        let config = ConfigBuilder::new().finish();
        let client = Client::with_config(config).await?;
        client.inc("zoomies", std::iter::empty::<&str>()).await?;
        Ok(())
    })
}
