mod handlers;
mod jambonz;
mod mutators;
mod voice;

use clap::Parser;
use emfcamp_schedule_api::Client as ScheduleClient;
use metrics::describe_counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
use url::Url;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(
        long,
        env,
        default_value = "https://schedule.emfcamp.dan-nixon.com/schedule"
    )]
    api_url: Url,

    #[arg(long, env, default_value = "0.0.0.0:8000")]
    webhook_address: SocketAddr,

    #[arg(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[derive(Clone)]
struct AppState {
    schedule_client: ScheduleClient,
}

const METRIC_API_ERRORS_NAME: &str = "dialaschedule_api_errors_total";
const METRIC_CALLS_NAME: &str = "dialaschedule_calls_total";
const METRIC_REQUESTS_NAME: &str = "dialaschedule_requests_total";
const METRIC_USER_ERROR_NAME: &str = "dialaschedule_user_error_total";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt::init();

    // Set up metrics server
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(cli.observability_address)
        .install()?;

    describe_counter!(
        METRIC_API_ERRORS_NAME,
        "Total number of times a call to the event API failed"
    );

    describe_counter!(METRIC_CALLS_NAME, "Total number of calls received");

    describe_counter!(
        METRIC_REQUESTS_NAME,
        "Total number of requests received to call endpoints"
    );

    describe_counter!(
        METRIC_USER_ERROR_NAME,
        "Total number of times a user entered an obviously wrong value"
    );

    // Setup schedule API client
    let schedule_client = ScheduleClient::new(cli.api_url);

    let state = AppState { schedule_client };

    let app = handlers::build_router().with_state(state);

    info!("Listening on {}", cli.webhook_address);
    let listener = TcpListener::bind(&cli.webhook_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
