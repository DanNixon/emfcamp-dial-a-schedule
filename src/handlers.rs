use crate::{
    jambonz::{Gather, GatherInputs, GatherResponse, Redirect, Verb},
    mutators::{EventIsPerformance, EventIsTalk, EventIsWorkshop, EventsHappeningNow},
    AppState,
};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use emfcamp_schedule_api::schedule::{
    event::Event,
    mutation::{Mutators, SortedByStartTime, StartsAfter, StartsBefore},
    Schedule,
};
use metrics::counter;
use tracing::{error, info};

pub(super) fn build_router() -> Router<AppState> {
    Router::new()
        .route("/call_status", post(call_status))
        .route("/call/incoming", post(call_incoming))
        .route("/call/menu", post(call_menu))
        .route("/call/menu_selection", post(call_menu_selection))
        .route("/call/events_now", post(call_events_now))
        .route(
            "/call/events_starting_soon",
            post(call_events_starting_soon),
        )
        .route(
            "/call/next_events_everywhere",
            post(call_next_events_everywhere),
        )
        .route(
            "/call/upcoming_talks_summary",
            post(call_upcoming_talks_summary),
        )
        .route(
            "/call/upcoming_workshops_summary",
            post(call_upcoming_workshops_summary),
        )
        .route(
            "/call/upcoming_performances_summary",
            post(call_upcoming_performances_summary),
        )
}

#[axum::debug_handler]
async fn call_status(Json(status): Json<crate::jambonz::CallStatusDetails>) {
    info!("Call status: {:?}", status);

    let call_status = format!("{:?}", status.call_status);
    counter!(crate::METRIC_CALLS_NAME, "status" => call_status, "from" => status.from).increment(1);
}

#[axum::debug_handler]
async fn call_incoming() -> Response {
    info!("Incomming call");
    counter!(crate::METRIC_REQUESTS_NAME, "endpoint" => "incoming").increment(1);

    let verbs = vec![
        crate::voice::speak_verb("Hello, and welcome to Dial-a-Schedule."),
        Verb::Redirect(Redirect {
            action_hook: "/call/menu".into(),
        }),
    ];

    Json(verbs).into_response()
}

#[axum::debug_handler]
async fn call_menu() -> Response {
    info!("Menu");
    counter!(crate::METRIC_REQUESTS_NAME, "endpoint" => "menu").increment(1);

    let verbs = vec![Verb::Gather(Gather {
        action_hook: "/call/menu_selection".to_string(),
        input: vec![GatherInputs::Digits],
        num_digits: Some(1),
        recognizer: None,
        say: Some(crate::voice::speak(
            "Dial 1 to hear what's going on right now. Need something to do? Dial 2 to hear what events are starting soon. Dial 3 to hear what is happening next at each venue. Dial 4 to get a summary of upcoming talks, dial 5 to get a summary of upcoming workshops, or dial 6 to get a summary of performances.",
        )),
    })];

    Json(verbs).into_response()
}

#[axum::debug_handler]
async fn call_menu_selection(Json(payload): Json<GatherResponse>) -> Response {
    info!("Menu selection: {:?}", payload);
    counter!(crate::METRIC_REQUESTS_NAME, "endpoint" => "menu_selection").increment(1);

    let digits = payload.digits.unwrap();

    let redirect_to = match digits.as_str() {
        "1" => Some("/call/events_now"),
        "2" => Some("/call/events_starting_soon"),
        "3" => Some("/call/next_events_everywhere"),
        "4" => Some("/call/upcoming_talks_summary"),
        "5" => Some("/call/upcoming_workshops_summary"),
        "6" => Some("/call/upcoming_performances_summary"),
        _ => None,
    };

    let verbs = match redirect_to {
        Some(endpoint) => vec![Verb::Redirect(Redirect {
            action_hook: endpoint.to_string(),
        })],
        None => {
            info!("A user entered an obviously incorrect option");
            counter!(crate::METRIC_USER_ERROR_NAME).increment(1);
            vec![
                crate::voice::speak_verb(&format!("Yeah, so you know when I gave you those options? The intention is that you pick one of those. Not some nonsense number like {digits}. I am not angry, I am just disappointed. Try again.")),
                Verb::Redirect(Redirect{ action_hook: "/call/menu".to_string() })
            ]
        }
    };

    Json(verbs).into_response()
}

const API_ERROR_MESSAGE: &str = "Oh no, something has gone very wrong. If this keeps happening, please feel free to shout at Dan until it is fixed. Be aware, Dan may shout back, or indeed shout at others as appropriate.";

async fn query_and_respond_with_a_list_of_events(
    state: &AppState,
    event_filter: impl Fn(Schedule) -> Vec<Event>,
    negative_response: &str,
    positive_response: &str,
    event_to_text: impl Fn(&Event) -> Verb,
) -> Response {
    let verbs = match state.schedule_client.get_schedule().await {
        Ok(schedule) => {
            let events = event_filter(schedule);
            info!("Got {} events for query", events.len());

            if events.is_empty() {
                vec![crate::voice::speak_verb(negative_response)]
            } else {
                let mut verbs = vec![crate::voice::speak_verb(positive_response)];

                for event in events {
                    verbs.push(event_to_text(&event));
                }

                verbs
            }
        }
        Err(e) => {
            error!("Schedule API error: {e}");
            counter!(crate::METRIC_API_ERRORS_NAME).increment(1);
            vec![crate::voice::speak_verb(API_ERROR_MESSAGE)]
        }
    };

    Json(verbs).into_response()
}

#[axum::debug_handler]
async fn call_events_now(State(state): State<AppState>) -> Response {
    info!("Events now");
    counter!(crate::METRIC_REQUESTS_NAME, "endpoint" => "events_now").increment(1);

    let now = Utc::now().into();

    let negative = "There are no events in progress. Sad, I know. Or maybe it is a silly time and you should be asleep.";
    let positive = "The following events are in progress.";

    query_and_respond_with_a_list_of_events(
        &state,
        |mut schedule| {
            let mutators = Mutators::new(vec![
                Box::<SortedByStartTime>::default(),
                Box::new(EventsHappeningNow::new(now)),
            ]);
            schedule.mutate(&mutators);
            schedule.events
        },
        negative,
        positive,
        |event| {
            let title = &event.title;
            let speaker = &event.speaker;
            let venue = &event.venue;
            let started = crate::voice::format_duration(now - event.start);
            let ending = crate::voice::format_duration(event.end - now);

            crate::voice::speak_verb(&format!(
                "Started {started} ago and ending in {ending} in {venue}: {title} by {speaker}."
            ))
        },
    )
    .await
}

#[axum::debug_handler]
async fn call_events_starting_soon(State(state): State<AppState>) -> Response {
    info!("Events starting soon");
    counter!(crate::METRIC_REQUESTS_NAME, "endpoint" => "events_now_starting_soon").increment(1);

    let now: DateTime<FixedOffset> = Utc::now().into();

    let negative = "There are no events starting soon. Sad, I know. Or maybe it is a silly time and you should be asleep.";
    let positive = "The following events may be of interest.";

    query_and_respond_with_a_list_of_events(
        &state,
        |mut schedule| {
            let range_start = now
                + Duration::try_minutes(-5)
                    .expect("hardcoded value for duration should be correct");
            let range_end = now
                + Duration::try_minutes(15)
                    .expect("hardcoded value for duration should be correct");

            let mutators = Mutators::new(vec![
                Box::new(StartsAfter::new(range_start)),
                Box::new(StartsBefore::new(range_end)),
            ]);

            schedule.mutate(&mutators);
            schedule.events
        },
        negative,
        positive,
        |event| {
            let title = &event.title;
            let speaker = &event.speaker;
            let venue = &event.venue;

            if event.start < now {
                let started = crate::voice::format_duration(now - event.start);
                crate::voice::speak_verb(&format!(
                    "Started {started} ago in {venue}: {title} by {speaker}."
                ))
            } else {
                let starting = crate::voice::format_duration(event.start - now);
                crate::voice::speak_verb(&format!(
                    "Starting in {starting} in {venue}: {title} by {speaker}."
                ))
            }
        },
    )
    .await
}

#[axum::debug_handler]
async fn call_next_events_everywhere(State(state): State<AppState>) -> Response {
    info!("Next events at all venues");
    counter!(crate::METRIC_REQUESTS_NAME, "endpoint" => "events_now").increment(1);

    let now = Utc::now().into();

    let negative = "There are no more events in the schedule. EMF 2024 is over. Everyone is sad, everyone apart from the spiders, and maybe the ducks.";
    let positive = "Here are the next events.";

    query_and_respond_with_a_list_of_events(
        &state,
        |schedule| {
            let epg = schedule.now_and_next(now);

            let mut events: Vec<Event> = epg.guide.values().fold(Vec::new(), |mut acc, x| {
                let mut next = x.next.clone();
                acc.append(&mut next);
                acc
            });

            // Ensure events are sorted by start time
            events.sort();

            events
        },
        negative,
        positive,
        |event| {
            let title = &event.title;
            let venue = &event.venue;
            let start = crate::voice::format_timestamp_relative_to(event.start, now);

            crate::voice::speak_verb(&format!("Starting {start} at {venue}: {title}."))
        },
    )
    .await
}

#[axum::debug_handler]
async fn call_upcoming_talks_summary(State(state): State<AppState>) -> Response {
    info!("Upcoming talks summary");
    counter!(crate::METRIC_REQUESTS_NAME, "endpoint" => "upcoming_talks_summary").increment(1);

    let now = Utc::now().into();

    let hours = 3;

    let negative = format!("There are no talks starting in the next {hours} hours. Maybe it is late and you should have a beer and enjoy some music. Sadly I can't join you, I am stuck in the telephone.");
    let positive =
        format!("Here are the talks you can look forward to over the next {hours} hours.");

    query_and_respond_with_a_list_of_events(
        &state,
        |mut schedule| {
            let until = now
                + Duration::try_hours(hours)
                    .expect("hardcoded value for duration should be correct");

            let mutators = Mutators::new(vec![
                Box::new(StartsAfter::new(now)),
                Box::new(StartsBefore::new(until)),
                Box::<EventIsTalk>::default(),
            ]);

            schedule.mutate(&mutators);
            schedule.events
        },
        &negative,
        &positive,
        |event| {
            let title = &event.title;
            let venue = &event.venue;
            let start = crate::voice::format_timestamp_relative_to(event.start, now);

            crate::voice::speak_verb(&format!("Starting {start} at {venue}: {title}."))
        },
    )
    .await
}

#[axum::debug_handler]
async fn call_upcoming_workshops_summary(State(state): State<AppState>) -> Response {
    info!("Upcoming workshops summary");
    counter!(crate::METRIC_REQUESTS_NAME, "endpoint" => "upcoming_workshops_summary").increment(1);

    let now = Utc::now().into();

    let hours = 3;

    let negative = format!("There are no workshops starting in the next {hours} hours. Maybe it is late and you should have a beer and enjoy some music. Sadly I can't join you, I am stuck in the telephone.");
    let positive = format!("Here are the workshops you can look forward to over the next {hours} hours. Well, assuming you won the appropriate ticket lottery.");

    query_and_respond_with_a_list_of_events(
        &state,
        |mut schedule| {
            let until = now
                + Duration::try_hours(hours)
                    .expect("hardcoded value for duration should be correct");

            let mutators = Mutators::new(vec![
                Box::new(StartsAfter::new(now)),
                Box::new(StartsBefore::new(until)),
                Box::<EventIsWorkshop>::default(),
            ]);

            schedule.mutate(&mutators);
            schedule.events
        },
        &negative,
        &positive,
        |event| {
            let title = &event.title;
            let venue = &event.venue;
            let start = crate::voice::format_timestamp_relative_to(event.start, now);

            crate::voice::speak_verb(&format!("Starting {start} at {venue}: {title}."))
        },
    )
    .await
}

#[axum::debug_handler]
async fn call_upcoming_performances_summary(State(state): State<AppState>) -> Response {
    info!("Upcoming performances summary");
    counter!(crate::METRIC_REQUESTS_NAME, "endpoint" => "upcoming_performances_summary")
        .increment(1);

    let now = Utc::now().into();

    let hours = 3;

    let negative = format!("There are no performances starting in the next {hours} hours. Maybe you could find an interesting talk to pass the time?");
    let positive = format!("Here are the performances taking place over the next {hours} hours.");

    query_and_respond_with_a_list_of_events(
        &state,
        |mut schedule| {
            let until = now
                + Duration::try_hours(hours)
                    .expect("hardcoded value for duration should be correct");

            let mutators = Mutators::new(vec![
                Box::new(StartsAfter::new(now)),
                Box::new(StartsBefore::new(until)),
                Box::<EventIsPerformance>::default(),
            ]);

            schedule.mutate(&mutators);
            schedule.events
        },
        &negative,
        &positive,
        |event| {
            let title = &event.title;
            let venue = &event.venue;
            let start = crate::voice::format_timestamp_relative_to(event.start, now);

            crate::voice::speak_verb(&format!("Starting {start} at {venue}: {title}."))
        },
    )
    .await
}
