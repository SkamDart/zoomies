use std::fmt;
use std::io;

use num_integer::Integer;

// Metric trait represents any datadog metric.
pub trait Metric {
    fn write(&self) -> String;
}

/// This trait represents anything that can be turned into a tag.
pub trait Tag {
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()>;
}

pub enum CountMetric<'a, T: Integer + fmt::Display> {
    // TODO
    // Why do I need to specify T here?
    Inc(&'a str, T),
    Dec(&'a str, T),
    Arb(&'a str, T),
}

impl<'a, T: Clone + fmt::Display + Integer> Metric for CountMetric<'a, T> {
    fn write(&self) -> String {
        match &*self {
            CountMetric::Inc(name, _) => write_count_metric_inc(name),
            CountMetric::Dec(name, _) => write_count_metric_dec(name),
            CountMetric::Arb(name, amt) => write_count_metric_arb(name, amt.clone()),
        }
    }
}

impl<T: AsRef<str>> Tag for T {
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(self.as_ref().as_bytes())
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

    for tag in tags.into_iter() {
        tag.write(&mut msg)?;
    }

    Ok(msg)
}

fn write_count_metric_inc(name: &str) -> String {
    let mut buf = String::with_capacity(3 + name.len() + 4);
    buf.push_str(name);
    buf.push_str(":1|c");
    buf
}

fn write_count_metric_dec(name: &str) -> String {
    let mut buf = String::with_capacity(3 + name.len() + 5);
    buf.push_str(name);
    buf.push_str(":-1|c");
    buf
}

fn write_count_metric_arb<T: Integer + fmt::Display>(name: &str, amt: T) -> String {
    let mut buf = String::with_capacity(3 + name.len() + 23);
    buf.push_str(name);
    buf.push_str(":");
    buf.push_str(&amt.to_string());
    buf.push_str("|c");
    buf
}
