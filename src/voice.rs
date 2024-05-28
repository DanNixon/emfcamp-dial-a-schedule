use crate::jambonz::{Say, SaySynthesizer, Verb};

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
