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
    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_string();
        self
    }

    pub fn text(&mut self, text: &str) -> &mut Self {
        self.text = text.to_string();
        self
    }

    pub fn timestamp(&mut self, ts: SystemTime) -> &mut Self {
        self.timestamp = Some(ts);
        self
    }

    pub fn hostname(&mut self, host: &str) -> &mut Self {
        self.hostname = Some(Hostname {
            0: host.to_string(),
        });
        self
    }

    pub fn agg_key(&mut self, agg_key: &str) -> &mut Self {
        self.agg_key = Some(AggKey {
            0: agg_key.to_string(),
        });
        self
    }

    pub fn priority(&mut self, priority: Priority) -> &mut Self {
        self.priority = Some(priority);
        self
    }

    pub fn source_type_name(&mut self, name: &str) -> &mut Self {
        self.source_type_name = Some(SourceTypeName {
            0: name.to_string(),
        });
        self
    }

    pub fn alert_type(&mut self, alert_type: AlertType) -> &mut Self {
        self.alert_type = Some(alert_type);
        self
    }

    pub fn build(&mut self) -> Result<Event, &'static str> {
        Ok(Event {
            title: self.title.to_string(),
            text: self.text.to_string(),
            timestamp: self.timestamp,
            hostname: self.hostname.clone(),
            agg_key: self.agg_key.clone(),
            priority: self.priority.clone(),
            source_type_name: self.source_type_name.clone(),
            alert_type: self.alert_type.clone(),
        })
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
        msg.push_str(",");
        msg.push_str(&text_len.to_string());
        msg.push_str("}:");
        msg.push_str(&title);
        msg.push_str("|");
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
    fn test_event_creation() {
        let _event: Event = Event::new()
            .title("Chungus")
            .text("Big Chungus")
            .priority(Priority::Low)
            .timestamp(std::time::SystemTime::UNIX_EPOCH)
            .hostname("kevin")
            .agg_key("something_cool")
            .source_type_name("your_app")
            .alert_type(AlertType::Error)
            .build()
            .expect("Failed to build");
    }

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
