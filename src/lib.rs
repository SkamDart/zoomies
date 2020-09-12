//! An Asynchronous Rust Client for interacting with DogStatsD
//!
//! ## Usage
//!
//! Use a `ConfigBuilder` to configure an asynchronous `Client`.
//!
//! ```notest
//! use zoomies::{Client, ConfigBuilder};
//!
//! #[async_std::main]
//! async fn main() -> std::io::Result<()> {
//!   let config = ConfigBuilder::new()
//!                .from_addr("127.0.0.1:10001".into())
//!                .to_addr("MY_STATSD_HOST:PORT".into())
//!                .namespace("chungus".into())
//!                .finish();
//!
//!   let client = Client::with_config(config).await?;
//!   Ok(())
//! }
//! ```
use std::default;

use async_std::{io::Result, net::UdpSocket};

mod events;
pub use events::*;

mod metrics;
pub use metrics::*;

pub trait DatagramFormat {
    fn format(&self) -> String;
}

pub struct ConfigBuilder {
    from_addr: String,
    to_addr: String,
    namespace: String,
}

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn from_addr(&mut self, addr: String) -> &mut ConfigBuilder {
        self.from_addr = addr;
        self
    }

    pub fn to_addr(&mut self, addr: String) -> &mut ConfigBuilder {
        self.to_addr = addr;
        self
    }

    pub fn namespace(&mut self, ns: String) -> &mut ConfigBuilder {
        self.namespace = ns;
        self
    }

    pub fn finish(&self) -> ConfigBuilder {
        ConfigBuilder {
            from_addr: self.from_addr.clone(),
            to_addr: self.to_addr.clone(),
            namespace: self.namespace.clone(),
        }
    }
}

impl default::Default for ConfigBuilder {
    fn default() -> ConfigBuilder {
        ConfigBuilder {
            from_addr: "127.0.0.1:0".into(),
            to_addr: "127.0.0.1:8125".into(),
            namespace: String::new(),
        }
    }
}

/// `Client` handles sending metrics to the DogstatsD server.
pub struct Client {
    socket: UdpSocket,
    config: ConfigBuilder,
}

impl Client {
    /// Construct a client with a specific Client.
    pub async fn with_config(config: ConfigBuilder) -> Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(config.from_addr.clone()).await?,
            config: config,
        })
    }

    pub async fn send<M>(&self, df: &M) -> Result<()>
    where
        M: DatagramFormat,
    {
        self.socket
            .send_to(df.format().as_bytes(), &self.config.to_addr)
            .await?;
        Ok(())
    }
}
