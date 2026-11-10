use axum::{Json, Router, routing::get};
use serde_json::json;

// FIX: sync fn — no db/io yet; async unwrapping unnecessary
async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "name": "PeaceWave",
        "message": "PeaceWave API is running"
    }))
}

// NOTE: yields when first signal arrives; cancelling and second-signal paths both exit
async fn shutdown_signal() {
    let ctrl_c = async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => println!("received SIGINT, shutting down"),
            Err(err) => eprintln!("failed to listen for SIGINT: {err}"),
        }
    };

    #[cfg(unix)]
    let terminate = async {
        // INFO: SIGTERM only exists on Unix; cfg-gated per Rust API guidelines
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
                println!("received SIGTERM, shutting down");
            }
            Err(err) => eprintln!("failed to listen for SIGTERM: {err}"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(health));

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:5000").await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("failed to bind 127.0.0.1:5000: {err}");
            std::process::exit(1);
        }
    };

    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("server error: {err}");
        std::process::exit(1);
    }

    println!("server stopped");
}

