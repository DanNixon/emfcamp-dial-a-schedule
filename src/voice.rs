use crate::jambonz::{Say, SaySynthesizer, Verb};
use chrono::{DateTime, Datelike, Duration, FixedOffset};

pub(crate) fn speak(text: &str) -> Say {
    Say {
        text: text.to_string(),
        synthesizer: Some(SaySynthesizer {
            vendor: "aws".to_string(),
            language: "en-GB".to_string(),
            gender: None,
            voice: "Amy".to_string(),
        }),
    }
}

pub(crate) fn speak_verb(text: &str) -> Verb {
    Verb::Say(speak(text))
}

pub(crate) fn format_timestamp_relative_to(
    timestamp: DateTime<FixedOffset>,
    now: DateTime<FixedOffset>,
) -> String {
    if timestamp.day() == now.day() {
        timestamp.format("%H:%M")
    } else {
        timestamp.format("%A %H:%M")
    }
    .to_string()
}

pub(crate) fn format_duration(duration: Duration) -> String {
    let hours = if duration.num_hours() > 0 {
        format!("{} hours", duration.num_hours())
    } else {
        "".to_string()
    };

    let minutes = format!("{} minutes", duration.num_minutes() % 60);

    format!("{hours}{minutes}")
}
