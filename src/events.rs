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

impl DatagramFormat for Priority {
    fn format(&self) -> String {
        let prefix = "p:";
        let p = match &*self {
            Priority::Low => "low",
            Priority::Normal => "normal",
        };
        let mut buf = String::with_capacity(prefix.len() + p.len());
        buf.push_str(prefix);
        buf.push_str(p);
        buf
    }
}

impl DatagramFormat for Option<Priority> {
    fn format(&self) -> String {
        match &*self {
            None => "".to_string(),
            Some(priority) => priority.format(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum AlertType {
    Error,
    Info,
    Success,
    Warning,
}

impl DatagramFormat for AlertType {
    fn format(&self) -> String {
        match &*self {
            AlertType::Error => "error".to_string(),
            AlertType::Info => "info".to_string(),
            AlertType::Success => "success".to_string(),
            AlertType::Warning => "warning".to_string(),
        }
    }
}

impl DatagramFormat for Option<AlertType> {
    fn format(&self) -> String {
        match &*self {
            None => "".to_string(),
            Some(alert_type) => alert_type.format(),
        }
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
    hostname: Option<String>,
    /// Add an aggregation key to group the event.
    /// There is no default.
    agg_key: Option<String>,
    /// Event priority.
    /// Default is Normal.
    priority: Option<Priority>,
    /// Event source.
    /// There is no default.
    source_type_name: Option<String>,
    /// Even alert type.
    /// Defaults to info.
    alert_type: Option<AlertType>,
    // Associated tags
    // tags: Option<T>,
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
        self.hostname = Some(host.to_string());
        self
    }

    pub fn agg_key(&mut self, agg_key: &str) -> &mut Self {
        self.agg_key = Some(agg_key.to_string());
        self
    }

    pub fn priority(&mut self, priority: Priority) -> &mut Self {
        self.priority = Some(priority);
        self
    }

    pub fn source_type_name(&mut self, name: &str) -> &mut Self {
        self.source_type_name = Some(name.to_string());
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

impl DatagramFormat for Event {
    fn format(&self) -> String {
        // Add this to Datadog format trait.
        let (title, title_size) = {
            let title = &self.title;
            (title, title.len())
        };
        let (text, text_size) = {
            let text = &self.text;
            (text, text.len())
        };
        /*
        let (timestamp, timestamp_size) = match self.timestamp {
            Some(ts) => {
                match ts.duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(elapsed) => {
                        let time = elapsed.to_string();
                        (time, time.len())
                    },
                    Err(_) => ("", 0),
                }
            }
            None => ("", 0),
        };:
        */
        let capacity = title_size + text_size;
        let mut msg = String::with_capacity(capacity);
        msg.push_str("_e{");
        msg.push_str(&title_size.to_string());
        msg.push_str(",");
        msg.push_str(&text_size.to_string());
        msg.push_str("}:");
        msg.push_str(title);
        msg.push_str("|");
        msg.push_str(text);
        msg
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_event_creation() {
        // let event: Event = Event::new()
        //     .title("Chungus")
        //     .text("Big Chungus")
        //     .priority(Priority::Low)
        //     .timestamp(std::time::SystemTime::UNIX_EPOCH)
        //     .hostname("kevin")
        //     .agg_key("something_cool")
        //     .source_type_name("your_app")
        //     .alert_type(AlertType::Error)
        //     .build()
        //     .expect("Failed to build");
    }

    #[test]
    fn test_simple_event() {
        assert_eq!(
            Event::new().title("Chungus").text("Big Chungus").format(),
            "_e{7,11}:Chungus|Big Chungus"
        );
    }

    #[test]
    #[ignore]
    fn test_event_with_timestamp() {
        assert_eq!(
            Event::new()
                .title("Chungus")
                .text("Big Chungus")
                .priority(Priority::Low)
                .format(),
            "_e{7,11}:Chungus|Big Chungus|p:low"
        );
    }

    #[test]
    #[ignore]
    fn test_event_with_all_stoppers() {
        assert_eq!(
            Event::new()
                .title("Chungus")
                .text("Big Chungus")
                .priority(Priority::Low)
                .timestamp(std::time::SystemTime::UNIX_EPOCH)
                .hostname("kevin")
                .agg_key("something_cool")
                .source_type_name("your_app")
                .alert_type(AlertType::Error)
                .format(),
            "_e{7,11}:Chungus|Big Chungus|d:0|h:kevin|a:something_cool|p:low|s:your_app|t:error"
        );
    }
}
