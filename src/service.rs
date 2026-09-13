use std::sync::Arc;

use chrono::{Datelike, Utc};

use crate::cache::VotdCache;
use crate::client::YouVersionClient;
use crate::models::{HttpResponse, VersionsResponse, VotdResponse};

/// Core service encapsulating routing, parameter validation, caching, and upstream orchestration.
///
/// Decoupled from any network transport or framework so it can be tested directly in unit tests
/// without opening network sockets or starting servers.
#[derive(Clone)]
pub struct VotdService {
    /// Upstream client trait object (production HTTP or mock test double).
    pub client: Arc<dyn YouVersionClient>,
    /// Thread-safe in-memory cache container.
    pub cache: Arc<VotdCache>,
}

impl VotdService {
    /// Creates a new `VotdService` with the provided client and cache.
    pub fn new(client: Arc<dyn YouVersionClient>, cache: Arc<VotdCache>) -> Self {
        Self { client, cache }
    }

    /// Primary entrypoint: handles an HTTP method and URI (path + optional query string)
    /// and returns an [`HttpResponse`] (HTTP status code + JSON body).
    ///
    /// # Routing
    /// - `GET /votd` -> [`handle_votd`](Self::handle_votd)
    /// - `GET /versions` -> [`handle_versions`](Self::handle_versions)
    /// - Any other route or method -> 404 Not Found
    pub fn handle_request(&self, method: &str, uri: &str) -> HttpResponse {
        if method != "GET" {
            return HttpResponse::error(404, "NOT_FOUND", "Endpoint not found");
        }

        let mut parts = uri.splitn(2, '?');
        let path = parts.next().unwrap_or("");
        let query_str = parts.next().unwrap_or("");

        match path {
            "/votd" => self.handle_votd(query_str),
            "/versions" => self.handle_versions(),
            _ => HttpResponse::error(404, "NOT_FOUND", "Endpoint not found"),
        }
    }

    /// Handles `GET /votd`: parses parameters, validates inputs, coordinates caching,
    /// queries the upstream YouVersion API when necessary, and formats the unified 4-field response.
    fn handle_votd(&self, query_str: &str) -> HttpResponse {
        let mut day_param: Option<&str> = None;
        let mut version_param: Option<&str> = None;

        if !query_str.is_empty() {
            for param in query_str.split('&') {
                let mut kv = param.splitn(2, '=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    match key {
                        "day" => day_param = Some(value),
                        "version" => version_param = Some(value),
                        _ => {}
                    }
                }
            }
        }

        // Validate and resolve `day` parameter (default: current UTC day of the year)
        let day: u32 = match day_param {
            Some(val) => match val.parse::<i64>() {
                Ok(d) if (1..=366).contains(&d) => d as u32,
                _ => {
                    return HttpResponse::error(
                        400,
                        "INVALID_DAY",
                        "The day parameter must be an integer between 1 and 366.",
                    );
                }
            },
            None => Utc::now().ordinal(),
        };

        // Validate and resolve `version` parameter (default: 206 for WEB)
        let version: u64 = match version_param {
            Some(val) => match val.parse::<i64>() {
                Ok(v) if v > 0 => v as u64,
                _ => {
                    return HttpResponse::error(
                        400,
                        "INVALID_VERSION",
                        "The version parameter must be a positive integer.",
                    );
                }
            },
            None => 206,
        };

        // Step 1: Resolve passage identifier (check cache first, then upstream)
        let passage_id = match self.cache.get_passage_id(day) {
            Some(cached_id) => cached_id,
            None => match self.client.get_verse_of_the_day(day) {
                Ok(votd_dto) => {
                    self.cache.set_passage_id(day, votd_dto.passage_id.clone());
                    votd_dto.passage_id
                }
                Err(_) => {
                    return HttpResponse::error(
                        502,
                        "UPSTREAM_ERROR",
                        "Failed to retrieve verse of the day from upstream service.",
                    );
                }
            },
        };

        // Step 2: Check if passage text for this (version, passage_id) is already cached
        if let Some((reference, text)) = self.cache.get_passage_text(version, &passage_id) {
            return HttpResponse::json(
                200,
                &VotdResponse {
                    day,
                    reference,
                    text,
                    version_id: version,
                },
            );
        }

        // Step 3: Fetch passage text from upstream, cache the result, and return 200 OK
        match self.client.get_passage(version, &passage_id) {
            Ok(passage_dto) => {
                self.cache.set_passage_text(
                    version,
                    passage_id,
                    passage_dto.reference.clone(),
                    passage_dto.content.clone(),
                );
                HttpResponse::json(
                    200,
                    &VotdResponse {
                        day,
                        reference: passage_dto.reference,
                        text: passage_dto.content,
                        version_id: version,
                    },
                )
            }
            Err(_) => HttpResponse::error(
                502,
                "UPSTREAM_ERROR",
                "Failed to retrieve passage text from upstream service.",
            ),
        }
    }

    /// Handles `GET /versions`: returns available English Bible versions, caching results in memory.
    fn handle_versions(&self) -> HttpResponse {
        if let Some(versions) = self.cache.get_versions() {
            return HttpResponse::json(200, &VersionsResponse { versions });
        }

        match self.client.get_bibles() {
            Ok(versions) => {
                self.cache.set_versions(versions.clone());
                HttpResponse::json(200, &VersionsResponse { versions })
            }
            Err(_) => HttpResponse::error(
                502,
                "UPSTREAM_ERROR",
                "Failed to retrieve Bible versions from upstream service.",
            ),
        }
    }
}
