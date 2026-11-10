use peace_wave::{app, config::Config};

#[tokio::main]
async fn main() {
    // INFO: missing .env is normal in production; malformed .env fails loudly
    if let Err(err) = dotenvy::dotenv()
        && !err.not_found()
    {
        eprintln!("failed to load .env: {err}");
        std::process::exit(1);
    }

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("configuration error: {err}");
            std::process::exit(1);
        }
    };

    if let Err(err) = app::serve(&config).await {
        eprintln!("server error: {err}");
        std::process::exit(1);
    }

    println!("server stopped");
}
