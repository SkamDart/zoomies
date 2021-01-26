/// Format:
/// _e{<TITLE>.length,<TEXT>.length}:<TITLE>|<TEXT>|d:<TIMESTAMP>|h:<HOSTNAME>|p:<PRIORITY>|t:<ALERT_TYPE>|#<TAG_KEY_1>:<TAG_VALUE_1>,<TAG_2>
/// ex:
/// ## Send an exception
/// _e{21,36}:An exception occurred|Cannot parse CSV file from 10.0.0.17|t:warning|#err_type:bad_file
///
/// ## Send an event with a newline in the text
/// _e{21,42}:An exception occurred|Cannot parse JSON request:\\n{"foo: "bar"}|p:low|#err_type:bad_request
use std::time::SystemTime;

use crate::DatagramFormat;

#[derive(Clone, PartialEq)]
pub enum Priority {
    Low,
    Normal,
}

#[derive(Clone, PartialEq)]
pub enum AlertType {
    Error,
    Info,
    Success,
    Warning,
}

#[derive(Clone)]
pub struct Hostname(String);

#[derive(Clone)]
pub struct EventTime(SystemTime);

#[derive(Clone)]
pub struct AggKey(String);

#[derive(Clone)]
pub struct SourceTypeName(String);

impl DatagramFormat for &SourceTypeName {
    fn format(&self) -> String {
        "|s:".to_owned() + &self.0.to_owned()
    }
}

impl DatagramFormat for &Priority {
    fn format(&self) -> String {
        let p = match &*self {
            Priority::Low => "low",
            Priority::Normal => "normal",
        };
        "|p:".to_owned() + &p.to_owned()
    }
}

impl DatagramFormat for &AlertType {
    fn format(&self) -> String {
        let suffix = match &*self {
            AlertType::Error => "error".to_string(),
            AlertType::Info => "info".to_string(),
            AlertType::Success => "success".to_string(),
            AlertType::Warning => "warning".to_string(),
        };
        "|t:".to_owned() + &suffix
    }
}

impl DatagramFormat for &Hostname {
    fn format(&self) -> String {
        "|h:".to_owned() + &self.0.to_owned()
    }
}

impl DatagramFormat for &AggKey {
    fn format(&self) -> String {
        "|k:".to_owned() + &self.0.to_owned()
    }
}

impl DatagramFormat for &SystemTime {
    fn format(&self) -> String {
        let suffix = match self.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs(),
        };
        "|d:".to_owned() + &suffix.to_string()
    }
}

/// Rust representation of the DogStatsD Datagram Event.
#[derive(Default)]
pub struct Event {
    /// The event title.
    title: String,
    /// The text associated with an event.
    text: String,
    /// Add a timestamp to the event.
    /// The default is a the current Unix Epoch timestamp.
    timestamp: Option<SystemTime>,
    /// Add a hostname to the event.
    /// There is no default.
    hostname: Option<Hostname>,
    /// Add an aggregation key to group the event.
    /// There is no default.
    agg_key: Option<AggKey>,
    /// Event priority.
    /// Default is Normal.
    priority: Option<Priority>,
    /// Event source.
    /// There is no default.
    source_type_name: Option<SourceTypeName>,
    /// Even alert type.
    /// Defaults to info.
    alert_type: Option<AlertType>,
}

impl Event {
    /// Creates a new Event with default options..
    ///
    /// You probably don't want this by itself.
    pub fn new() -> Self {
        Event::default()
    }

    /// Set the event Title...
    pub fn title<S: Into<String>>(self, title: S) -> Self {
        Self {
            title: title.into(),
            ..self
        }
    }

    pub fn text<S: Into<String>>(self, text: S) -> Self {
        Self {
            text: text.into(),
            ..self
        }
    }

    pub fn timestamp<T: Into<SystemTime>>(self, ts: T) -> Self {
        Self {
            timestamp: Some(ts.into()),
            ..self
        }
    }

    pub fn hostname<S: Into<String>>(self, host: S) -> Self {
        Self {
            hostname: Some(Hostname(host.into())),
            ..self
        }
    }

    pub fn agg_key<S: Into<String>>(self, agg_key: S) -> Self {
        Self {
            agg_key: Some(AggKey(agg_key.into())),
            ..self
        }
    }

    pub fn priority(self, priority: Priority) -> Self {
        Self {
            priority: Some(priority),
            ..self
        }
    }

    pub fn source_type_name<S: Into<String>>(self, name: S) -> Self {
        Self {
            source_type_name: Some(SourceTypeName(name.into())),
            ..self
        }
    }

    pub fn alert_type(self, alert_type: AlertType) -> Self {
        Self {
            alert_type: Some(alert_type),
            ..self
        }
    }
}

// Very poorly named function that converts a &str to a String and gets then length.
//
// This is a helper for the DatagramFormat for Event.
fn convert_len(name: &str) -> (String, usize) {
    (name.to_string(), name.len())
}

fn convert_len_from_opt<T: DatagramFormat>(opt: Option<T>) -> (String, usize) {
    convert_len(&opt.format())
}

impl DatagramFormat for Event {
    fn format(&self) -> String {
        let (title, title_len) = convert_len(&self.title);
        let (text, text_len) = convert_len(&self.text);
        let (ts, ts_len) = convert_len_from_opt(self.timestamp.as_ref());
        let (hn, hn_len) = convert_len_from_opt(self.hostname.as_ref());
        let (ak, ak_len) = convert_len_from_opt(self.agg_key.as_ref());
        let (ap, ap_len) = convert_len_from_opt(self.priority.as_ref());
        let (at, at_len) = convert_len_from_opt(self.alert_type.as_ref());
        let (st, st_len) = convert_len_from_opt(self.source_type_name.as_ref());
        let capacity = title_len + text_len + ts_len + hn_len + ak_len + ap_len + at_len + st_len;
        let mut msg = String::with_capacity(capacity);
        msg.push_str("_e{");
        msg.push_str(&title_len.to_string());
        msg.push(',');
        msg.push_str(&text_len.to_string());
        msg.push_str("}:");
        msg.push_str(&title);
        msg.push('|');
        msg.push_str(&text);
        msg.push_str(&ts);
        msg.push_str(&hn);
        msg.push_str(&ak);
        msg.push_str(&ap);
        msg.push_str(&st);
        msg.push_str(&at);
        msg
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_event() {
        assert_eq!(
            Event::new().title("Chungus").text("Big Chungus").format(),
            "_e{7,11}:Chungus|Big Chungus"
        );
    }

    #[test]
    fn test_event_with_all_stoppers() {
        assert_eq!(
            Event::new()
                .title("Chungus")
                .text("Big Chungus")
                .priority(Priority::Low)
                .timestamp(SystemTime::UNIX_EPOCH)
                .hostname("kevin")
                .agg_key("something_cool")
                .source_type_name("your_app")
                .alert_type(AlertType::Error)
                .format(),
            "_e{7,11}:Chungus|Big Chungus|d:0|h:kevin|k:something_cool|p:low|s:your_app|t:error"
        );
    }
}
