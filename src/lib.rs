//! # Verse of the Day API (`votd_api`)
//!
//! A simple REST API serving the Verse of the Day from the YouVersion Platform API.
//!
//! ### Core Components
//! - [`service::VotdService`]: Request routing, validation, caching, and upstream orchestration.
//! - [`cache::VotdCache`]: Thread-safe in-memory cache using `std::sync::RwLock`.
//! - [`client::YouVersionClient`]: Trait abstracting live HTTP and mock YouVersion clients.
//! - [`server::SimpleHttpServer`]: Pure standard library HTTP server (`std::net::TcpListener`).
//! - [`models`]: Strict JSON DTOs matching project specifications.
//! - [`error::AppError`]: Application errors mapped to HTTP status codes.
//! - [`config::AppConfig`]: Environment configuration with secret redaction.

pub mod cache;
pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod server;
pub mod service;

pub use cache::VotdCache;
pub use client::{HttpYouVersionClient, MockYouVersionClient, YouVersionClient};
pub use config::AppConfig;
pub use error::AppError;
pub use models::{HttpResponse, VotdResponse};
pub use server::SimpleHttpServer;
pub use service::VotdService;
