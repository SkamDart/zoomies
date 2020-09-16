//! An Asynchronous Rust Client for interacting with DogStatsD
//!
//! ## Usage
//!
//! Use a `ConfigBuilder` to configure an asynchronous `UdpClient`.
//!
//! ```notest
//! use zoomies::{UdpClient, ConfigBuilder};
//!
//! #[async_std::main]
//! async fn main() -> std::io::Result<()> {
//!   let config = ConfigBuilder::new()
//!                .from_addr("127.0.0.1:10001".into())
//!                .to_addr("MY_STATSD_HOST:PORT".into())
//!                .namespace("chungus".into())
//!                .finish();
//!
//!   let client = UdpClient::with_config(config).await?;
//!   Ok(())
//! }
//! ```
use std::collections::HashMap;
use std::default;
use std::fmt;

use async_std::{io::Result, net::UdpSocket};

mod events;
pub use events::*;

mod metrics;
pub use metrics::*;

// Trait that can serialize a type into the DogStatsD datagram format.
pub trait DatagramFormat {
    fn format(&self) -> String;
}

impl<T> DatagramFormat for Option<T>
where
    T: DatagramFormat,
{
    fn format(&self) -> String {
        match &*self {
            None => String::new(),
            Some(t) => t.format(),
        }
    }
}

// Convert rust HashMap to a -> #<TAG_KEY_1>:<TAG_VALUE_1>,<TAG_2> format.
impl<K, V> DatagramFormat for HashMap<K, V>
where
    K: fmt::Display,
    V: fmt::Display,
{
    fn format(&self) -> String {
        if self.len() == 0 {
            String::new()
        } else {
            let map_elem_size = self.iter().fold(0, |acc, (k, v)| {
                acc + k.to_string().len() + v.to_string().len() + 3
            });
            let capacity = map_elem_size + self.len() + 1;
            let mut buf = String::with_capacity(capacity);
            buf.push_str("|#");
            for (k, v) in self.into_iter() {
                let item = k.to_string() + ":" + &v.to_string() + ",";
                buf.push_str(&item);
            }
            buf.trim_end_matches(",").to_string()
        }
    }
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
pub struct UdpClient {
    socket: UdpSocket,
    config: ConfigBuilder,
}

impl UdpClient {
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

    pub async fn send_with_tags<M: DatagramFormat>(&self, df: &M, tags: M) -> Result<()> {
        let content = df.format() + &tags.format();
        self.socket
            .send_to(content.as_bytes(), &self.config.to_addr)
            .await?;
        Ok(())
    }
}

mod test {
    use super::DatagramFormat;
    use std::collections::HashMap;

    #[test]
    fn test_empty_tag() {
        let timber_resources: HashMap<&str, i32> = [].iter().cloned().collect();
        assert_eq!(timber_resources.format(), String::new());
    }

    #[test]
    fn test_single_tag() {
        let timber_resources: HashMap<&str, i32> = [("Norway", 100)].iter().cloned().collect();
        assert_eq!(timber_resources.format(), "|#Norway:100");
    }

    #[test]
    #[ignore]
    fn test_multiple_tags() {
        // TODO find better way to test this as iterator creation is not idempotent.
        let timber_resources: HashMap<&str, i32> =
            [("Norway", 100), ("Denmark", 50), ("Iceland", 10)]
                .iter()
                .cloned()
                .collect();
        assert_eq!(
            timber_resources.format(),
            "|#Norway:100,Denmark:50,Iceland:10"
        );
    }
}
