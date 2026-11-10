use peace_wave::{app, config::Config, telemetry};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // INFO: .env loads before telemetry so RUST_LOG inside .env is honored
    if let Err(err) = dotenvy::dotenv()
        && !err.not_found()
    {
        // INFO: subscriber not installed yet; a direct write is the only channel here
        eprintln!("failed to load .env: {err}");
        std::process::exit(1);
    }

    telemetry::init();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            error!(error = %err, "invalid application configuration");
            std::process::exit(1);
        }
    };

    info!(addr = %config.bind_addr(), "starting PeaceWave");

    if let Err(err) = app::serve(&config).await {
        error!(error = %err, "server startup or runtime failure");
        std::process::exit(1);
    }

    info!("server stopped");
}
