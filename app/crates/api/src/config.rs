use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_path: String,
    pub serpapi_key: Option<String>,
    /// Override the SerpAPI endpoint (tests, local mocks).
    pub serpapi_base_url: Option<String>,
    pub serpapi_daily_limit: u32,
    pub cors_origins: Vec<String>,
    /// Requests per client per minute. 0 disables the limiter.
    pub rate_limit_per_minute: u32,
    pub search_cache_ttl: Duration,
    pub reference_cache_ttl: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8080,
            database_path: "faredesk.sqlite".into(),
            serpapi_key: None,
            serpapi_base_url: None,
            serpapi_daily_limit: 100,
            rate_limit_per_minute: 120,
            cors_origins: vec![
                "http://localhost:5173".into(),
                "http://127.0.0.1:5173".into(),
                "http://localhost:4173".into(),
                "http://127.0.0.1:4173".into(),
            ],
            search_cache_ttl: Duration::from_secs(10 * 60),
            reference_cache_ttl: Duration::from_secs(24 * 60 * 60),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let defaults = Self::default();
        let serpapi_key = std::env::var("SERPAPI_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Self {
            port: std::env::var("PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(defaults.port),
            database_path: std::env::var("DATABASE_PATH").unwrap_or(defaults.database_path),
            serpapi_key,
            serpapi_base_url: std::env::var("SERPAPI_BASE_URL")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            serpapi_daily_limit: std::env::var("SERPAPI_DAILY_LIMIT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(defaults.serpapi_daily_limit),
            cors_origins: std::env::var("CORS_ORIGINS")
                .ok()
                .map(|value| {
                    value
                        .split(',')
                        .map(|origin| origin.trim().to_string())
                        .filter(|origin| !origin.is_empty())
                        .collect()
                })
                .unwrap_or(defaults.cors_origins),
            rate_limit_per_minute: std::env::var("RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(defaults.rate_limit_per_minute),
            search_cache_ttl: defaults.search_cache_ttl,
            reference_cache_ttl: defaults.reference_cache_ttl,
        }
    }
}
