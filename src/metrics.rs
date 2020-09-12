use std::fmt;
use std::io;

use crate::DatagramFormat;
use num_integer::Integer;

/// The module, `zoomies::metrics`, implements the following metric types that are accepted by DataDog.
///
/// Metrics
/// - Count (Inc, Dec, Arb)
/// - Gauge
/// - Set
/// - Histogram
/// - Distribution
pub enum Metric<'a, T> {
    /// The Rust representation of a Count Metric in StatsD
    /// The `Count` metric submission type represents the total number of event occurrences in one time interval.
    /// A `Count` can be used to track the total number of connections made to a database or the total number of requests to an endpoint.
    /// This number of events can accumulate or decrease over time—it is not monotonically increasing.
    Inc(&'a str),
    Dec(&'a str),
    Arb(&'a str, T),
    Gauge(&'a str, &'a str),
    Histogram(&'a str, &'a str),
    Distribution(&'a str, &'a str),
    Set(&'a str, &'a str),
}

impl<'a, T: fmt::Display + Integer> DatagramFormat for Metric<'a, T> {
    fn format(&self) -> String {
        let (metric_name, value, identifier) = match &*self {
            Metric::Set(metric_name, value) => (metric_name, value.to_string(), "|s"),
            Metric::Gauge(metric_name, value) => (metric_name, value.to_string(), "|g"),
            Metric::Histogram(metric_name, value) => (metric_name, value.to_string(), "|h"),
            Metric::Distribution(metric_name, value) => (metric_name, value.to_string(), "|d"),
            count => {
                let (name, val) = match count {
                    Metric::Inc(metric_name) => (metric_name, "1".to_string()),
                    Metric::Dec(metric_name) => (metric_name, "-1".to_string()),
                    Metric::Arb(metric_name, i) => (metric_name, i.to_string()),
                    _ => unreachable!(),
                };
                (name, val.to_string(), "|c".into())
            }
        };
        let mut msg = String::with_capacity(metric_name.len() + value.len() + identifier.len());
        msg.push_str(metric_name);
        msg.push_str(":");
        msg.push_str(&value);
        msg.push_str(identifier);
        msg
    }
}

/// This trait represents anything that can be turned into a tag.
pub trait Tag {
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()>;
}

impl<T: AsRef<str>> Tag for T {
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(self.as_ref().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    /// Metrics
    /// - Count
    /// - Gauge
    /// - Set
    /// - Histogram
    /// - Distribution
    use super::*;

    #[test]
    fn test_metrics_arb() {
        assert_eq!(
            Metric::Arb::<u32>("custom_metric", 5).format(),
            "custom_metric:5|c"
        );
    }

    #[test]
    fn test_metrics_inc() {
        assert_eq!(
            Metric::Inc::<u32>("custom_metric").format(),
            "custom_metric:1|c"
        );
    }

    #[test]
    fn test_metrics_dec() {
        assert_eq!(
            Metric::Dec::<u32>("custom_metric").format(),
            "custom_metric:-1|c"
        );
    }

    #[test]
    fn test_metrics_gauge() {
        assert_eq!(
            Metric::Gauge::<u32>("custom_metric", "3").format(),
            "custom_metric:3|g"
        );
    }

    #[test]
    fn test_metrics_set() {
        assert_eq!(
            Metric::Set::<u32>("custom_metric", "person").format(),
            "custom_metric:person|s"
        );
    }

    #[test]
    fn test_metrics_histogram() {
        assert_eq!(
            Metric::Histogram::<u32>("custom_metric", "240").format(),
            "custom_metric:240|h"
        );
    }

    #[test]
    fn test_metrics_distribution() {
        assert_eq!(
            Metric::Distribution::<u32>("custom_metric", "42").format(),
            "custom_metric:42|d"
        );
    }
}
