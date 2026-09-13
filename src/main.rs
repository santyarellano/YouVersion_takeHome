//! # Verse of the Day API Binary Entrypoint
//!
//! Initializes configuration from environment variables, instantiates the HTTP YouVersion
//! client and thread-safe in-memory cache, mounts the `VotdService`, and runs the standard
//! library TCP socket server.

use std::sync::Arc;

use votd_api::cache::VotdCache;
use votd_api::client::HttpYouVersionClient;
use votd_api::config::AppConfig;
use votd_api::server::SimpleHttpServer;
use votd_api::service::VotdService;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env();
    let addr = format!("{}:{}", config.host, config.port);
    println!("Starting Verse of the Day API on {}", addr);

    let client = Arc::new(HttpYouVersionClient::new(
        config.base_url.clone(),
        &config.app_key,
    ));
    let cache = Arc::new(VotdCache::new());
    let service = Arc::new(VotdService::new(client, cache));

    let server = SimpleHttpServer::bind(&addr, service)?;
    println!("Server listening on http://{}", addr);

    server.run()?;

    Ok(())
}
