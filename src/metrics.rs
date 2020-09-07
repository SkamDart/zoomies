use std::fmt;
use std::io;

use num_integer::Integer;

/// The module, `zoomies::metric`, implements the following metric types that are accepted by DataDog.
///
/// Metrics
/// - Count
/// - Rate
/// - Gauge
/// - Set
/// - Histogram
/// - Distribution

// The Metric trait describes converting any datadog metric into the following format.
/// <METRIC_NAME>:<VALUE>|<TYPE>|@<SAMPLE_RATE>|#<TAG_KEY_1>:<TAG_VALUE_1>,<TAG_2>
///
/// Note, The <SAMPLE_RATE> only works for Count, Histogram, and Timer metrics.
pub trait Metric {
    fn write(&self) -> String;
}

/// This trait represents anything that can be turned into a tag.
pub trait Tag {
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()>;
}

/// The Rust representation of a Count Metric in StatsD
/// The `Count` metric submission type represents the total number of event occurrences in one time interval.
/// A `Count` can be used to track the total number of connections made to a database or the total number of requests to an endpoint.
/// This number of events can accumulate or decrease over time—it is not monotonically increasing.
pub enum Count<'a, T: Integer + fmt::Display> {
    // TODO
    // Why do I need to specify T here?
    Inc(&'a str, T),
    Dec(&'a str, T),
    Arb(&'a str, T),
}

impl<'a, T: Clone + fmt::Display + Integer> Metric for Count<'a, T> {
    fn write(&self) -> String {
        match &*self {
            Count::Inc(name, _) => write_count_metric_arb(name, 1),
            Count::Dec(name, _) => write_count_metric_arb(name, -1),
            Count::Arb(name, amt) => write_count_metric_arb(name, amt.clone()),
        }
    }
}

impl<T: AsRef<str>> Tag for T {
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(self.as_ref().as_bytes())
    }
}

/// Gauge is a Rust representation of the StatsD Gauge.
pub struct Gauge<'a> {
    metric_name: &'a str,
    value: &'a str,
}

impl<'a> Gauge<'a> {
    pub fn new(metric_name: &'a str, value: &'a str) -> Self {
        Gauge {
            metric_name: metric_name,
            value: value,
        }
    }
}

/// Provides serialization from a Rust struct to StatsD supported format.
///
/// e.g. Gauge { metric_name: "chungus", value: "42" } <=> chungus:42|g
impl<'a> Metric for Gauge<'a> {
    fn write(&self) -> String {
        let mut msg = String::with_capacity(3 + self.metric_name.len() + self.value.len());
        msg.push_str(self.metric_name);
        msg.push_str(":");
        msg.push_str(self.value);
        msg.push_str("|g");
        msg
    }
}

pub struct Histogram<'a> {
    metric_name: &'a str,
    value: &'a str,
}

impl<'a> Histogram<'a> {
    pub fn new(metric_name: &'a str, value: &'a str) -> Self {
        Histogram {
            metric_name: metric_name,
            value: value,
        }
    }
}

impl<'a> Metric for Histogram<'a> {
    fn write(&self) -> String {
        let mut msg = String::with_capacity(self.metric_name.len() + self.value.len() + 3);
        msg.push_str(self.metric_name);
        msg.push_str(":");
        msg.push_str(self.value);
        msg.push_str("|h");
        msg
    }
}

pub struct Distribution<'a> {
    metric_name: &'a str,
    value: &'a str,
}

impl<'a> Distribution<'a> {
    pub fn new(metric_name: &'a str, value: &'a str) -> Self {
        Distribution {
            metric_name: metric_name,
            value: value,
        }
    }
}

impl<'a> Metric for Distribution<'a> {
    fn write(&self) -> String {
        let mut msg = String::with_capacity(self.metric_name.len() + self.value.len() + 3);
        msg.push_str(self.metric_name);
        msg.push_str(":");
        msg.push_str(self.value);
        msg.push_str("|d");
        msg
    }
}

pub struct Set<'a> {
    metric_name: &'a str,
    value: &'a str,
}

impl<'a> Set<'a> {
    pub fn new(metric_name: &'a str, value: &'a str) -> Self {
        Set {
            metric_name: metric_name,
            value: value,
        }
    }
}

impl<'a> Metric for Set<'a> {
    fn write(&self) -> String {
        let mut msg = String::with_capacity(self.metric_name.len() + self.value.len() + 3);
        msg.push_str(self.metric_name);
        msg.push_str(":");
        msg.push_str(self.value);
        msg.push_str("|s");
        msg
    }
}

pub fn format_metric<M, I, T>(metric: &M, namespace: &str, tags: I) -> io::Result<Vec<u8>>
where
    M: Metric,
    I: IntoIterator<Item = T>,
    T: Tag,
{
    let m = metric.write();
    let ns = namespace;
    let mut msg = Vec::with_capacity(m.len() + ns.len());

    if !ns.is_empty() {
        msg.extend_from_slice(ns.as_bytes());
        msg.extend_from_slice(b".");
    }

    msg.extend_from_slice(m.as_bytes());

    let mut tags_iter = tags.into_iter();
    let mut next_tag = tags_iter.next();

    while next_tag.is_some() {
        next_tag.unwrap().write(&mut msg)?;
        next_tag = tags_iter.next();
    }

    Ok(msg)
}

fn write_count_metric_arb<T: Integer + fmt::Display>(name: &str, amt: T) -> String {
    let (mut buf, num) = {
        let num = amt.to_string();
        (String::with_capacity(3 + name.len() + num.len()), num)
    };
    buf.push_str(name);
    buf.push_str(":");
    buf.push_str(&num);
    buf.push_str("|c");
    buf
}

mod tests {
    // use super::*;
}
