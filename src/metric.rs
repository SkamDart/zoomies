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
            CountMetric::Inc(name, _) => write_count_metric_arb(name, 1),
            CountMetric::Dec(name, _) => write_count_metric_arb(name, -1),
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
