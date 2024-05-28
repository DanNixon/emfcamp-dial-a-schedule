use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "verb")]
pub(crate) enum Verb {
    Redirect(Redirect),
    // Pause(Pause),
    Say(Say),
    Gather(Gather),
}

/// See https://www.jambonz.org/docs/webhooks/redirect/
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Redirect {
    pub action_hook: String,
}

/// See https://www.jambonz.org/docs/webhooks/pause/
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Pause {
    pub length: u64,
}

/// See https://www.jambonz.org/docs/webhooks/say/
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Say {
    pub text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub synthesizer: Option<SaySynthesizer>,
}

/// See https://www.jambonz.org/docs/webhooks/say/
#[derive(Debug, Clone, Serialize)]
pub(crate) struct SaySynthesizer {
    pub vendor: String,

    pub language: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,

    pub voice: String,
}

/// See https://www.jambonz.org/docs/webhooks/gather/
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Gather {
    pub action_hook: String,
    pub input: Vec<GatherInputs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_digits: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub recognizer: Option<GatherRecognizer>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub say: Option<Say>,
}

/// See https://www.jambonz.org/docs/webhooks/gather/
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum GatherInputs {
    Digits,
    // Speech,
}

/// See https://www.jambonz.org/docs/webhooks/gather/
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GatherRecognizer {
    pub vendor: String,

    pub language: String,

    pub hints: Vec<String>,

    pub hints_boost: i32,
}

/// See https://www.jambonz.org/docs/webhooks/gather/
#[derive(Debug, Deserialize)]
pub(crate) struct GatherResponse {
    pub digits: Option<String>,
}

/// See https://www.jambonz.org/docs/webhooks/overview/
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CallStatusDetails {
    #[allow(unused)]
    call_id: String,

    #[allow(unused)]
    call_sid: String,

    pub call_status: CallStatus,

    #[allow(unused)]
    call_termination_by: Option<String>,

    #[allow(unused)]
    duration: Option<i64>,

    pub from: String,
}

/// See https://www.jambonz.org/docs/webhooks/overview/
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CallStatus {
    Trying,
    Ringing,
    EarlyMedia,
    InProgress,
    Completed,
    Failed,
    Busy,
    NoAnswer,
}
