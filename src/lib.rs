//! An Asynchronous Rust Client for interacting with DogStatsD
//!
//! ## Usage
//!
//! Use a `ConfigBuilder` to configure an asynchronous `Client`.
//!
//! ```
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

use std::borrow::Cow;
use std::fmt;

use async_std::{io::Result, net::UdpSocket};
use num_integer::Integer;

mod metrics;
use metrics::*;

pub struct ConfigBuilder {
    from_addr: String,
    to_addr: String,
    namespace: String,
}

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        Self {
            from_addr: "127.0.0.1:0".into(),
            to_addr: "127.0.0.1:8125".into(),
            namespace: String::new(),
        }
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

/// `Client` handles sending metrics to the DogstatsD server.
pub struct Client {
    socket: UdpSocket,
    config: ConfigBuilder,
}

impl Client {
    pub async fn with_config(config: ConfigBuilder) -> Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(config.from_addr.clone()).await?,
            config: config,
        })
    }

    /// Increment a StatsD counter.
    pub async fn inc<'a, I, S, T>(&self, metric_name: S, tags: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        S: Into<Cow<'a, str>>,
        T: Tag,
    {
        self.send(&Count::Inc(metric_name.into().as_ref(), 0), tags)
            .await
    }

    /// Decrement a StatsD counter.
    pub async fn dec<'a, I, S, T>(&self, metric_name: S, tags: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        S: Into<Cow<'a, str>>,
        T: Tag,
    {
        self.send(&Count::Dec(metric_name.into().as_ref(), 0), tags)
            .await
    }

    /// Arbitrarily add to  a StatsD counter.
    pub async fn arb<'a, I, S, T, N>(&self, metric_name: S, n: N, tags: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        S: Into<Cow<'a, str>>,
        T: Tag,
        N: Copy + fmt::Display + Integer,
    {
        self.send(&Count::Arb(metric_name.into().as_ref(), n), tags)
            .await
    }

    /// Adds a value to histogram metric type.
    ///
    /// The HISTOGRAM metric submission type represents the statistical distribution of a set of values calculated Agent-side in one time interval.
    pub async fn histogram<'a, I, S, T, N>(&self, metric_name: S, n: N, tags: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        S: Into<Cow<'a, str>>,
        T: Tag,
        N: Copy + fmt::Display + Integer,
    {
        self.send(
            &Histogram::new(metric_name.into().as_ref(), n.to_string().as_ref()),
            tags,
        )
        .await
    }

    /// Adds a value to the distribution metric type.
    ///
    /// The DISTRIBUTION metric submission type represents the global statistical distribution of a set of values calculated across your entire distributed infrastructure in one time interval.
    pub async fn distribution<'a, I, S, T, N>(&self, metric_name: S, n: N, tags: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        S: Into<Cow<'a, str>>,
        T: Tag,
        N: Copy + fmt::Display + Integer,
    {
        self.send(
            &Distribution::new(metric_name.into().as_ref(), n.to_string().as_ref()),
            tags,
        )
        .await
    }

    pub async fn set<'a, I, S, T, N>(&self, metric_name: S, n: N, tags: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        S: Into<Cow<'a, str>>,
        T: Tag,
        N: Copy + fmt::Display + Integer,
    {
        self.send(
            &Set::new(metric_name.into().as_ref(), n.to_string().as_ref()),
            tags,
        )
        .await
    }

    /// Adds a gauge value.
    ///
    /// The GAUGE metric submission type represents a snapshot of events in one time interval.
    /// This representative snapshot value is the last value submitted to the Agent during a time interval.
    /// A GAUGE can be used to take a measure of something reporting continuously—like the available disk space or memory used.
    pub async fn gauge<'a, I, S, T, N>(&self, metric_name: S, n: N, tags: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        S: Into<Cow<'a, str>>,
        T: Tag,
        N: Copy + fmt::Display + Integer,
    {
        self.send(
            &Gauge::new(metric_name.into().as_ref(), n.to_string().as_ref()),
            tags,
        )
        .await
    }

    async fn send<M, I, T>(&self, metric: &M, tags: I) -> Result<()>
    where
        M: Metric,
        I: IntoIterator<Item = T>,
        T: Tag,
    {
        let formatted = format_metric(metric, &self.config.namespace, tags)?;
        self.socket
            .send_to(formatted.as_slice(), &self.config.to_addr)
            .await?;
        Ok(())
    }
}
