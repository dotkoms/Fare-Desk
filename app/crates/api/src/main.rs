use tracing_subscriber::EnvFilter;

use faredesk_api::{config::Config, AppState};

#[tokio::main]
async fn main() {
    // Real environment variables win over .env.
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "faredesk_api=info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env();
    let state = match AppState::new(config) {
        Ok(state) => state,
        Err(error) => {
            tracing::error!(%error, "could not open the store");
            std::process::exit(1);
        }
    };

    if let Err(error) = faredesk_api::run(state).await {
        tracing::error!(%error, "server stopped");
        std::process::exit(1);
    }
}
