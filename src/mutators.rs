use chrono::{DateTime, FixedOffset};
use emfcamp_schedule_api::schedule::{
    event::{Event, Kind, RelativeTime},
    mutation::Mutator,
};

pub(crate) struct EventsHappeningNow {
    timestamp: DateTime<FixedOffset>,
}

impl EventsHappeningNow {
    pub(crate) fn new(timestamp: DateTime<FixedOffset>) -> Self {
        Self { timestamp }
    }
}

impl Mutator for EventsHappeningNow {
    fn mutate(&self, events: &mut Vec<Event>) {
        events.retain(|e| e.relative_to(self.timestamp) == RelativeTime::Now);
    }
}

#[derive(Default)]
pub(crate) struct EventIsTalk {}

impl Mutator for EventIsTalk {
    fn mutate(&self, events: &mut Vec<Event>) {
        events.retain(|e| matches!(e.kind, Kind::Talk));
    }
}

#[derive(Default)]
pub(crate) struct EventIsWorkshop {}

impl Mutator for EventIsWorkshop {
    fn mutate(&self, events: &mut Vec<Event>) {
        events.retain(|e| matches!(e.kind, Kind::Workshop(_) | Kind::YouthWorkshop));
    }
}

#[derive(Default)]
pub(crate) struct EventIsPerformance {}

impl Mutator for EventIsPerformance {
    fn mutate(&self, events: &mut Vec<Event>) {
        events.retain(|e| matches!(e.kind, Kind::Performance));
    }
}
