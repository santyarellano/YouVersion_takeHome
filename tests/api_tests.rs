use std::sync::Arc;
use std::thread;

use chrono::{Datelike, Utc};
use serde_json::Value;

use votd_api::cache::VotdCache;
use votd_api::client::MockYouVersionClient;
use votd_api::error::AppError;
use votd_api::models::{
    BibleVersionSummary, ErrorResponse, HttpResponse, UpstreamPassageDto, UpstreamVotdDto,
    VotdResponse,
};
use votd_api::server::SimpleHttpServer;
use votd_api::service::VotdService;

const SAMPLE_APP_KEY: &str = "dummy_test_api_key";

/// Helper function that constructs an isolated test harness for `VotdService`.
///
/// Instantiates a fresh `MockYouVersionClient` and an empty `VotdCache`, returning:
/// - `service`: The initialized `VotdService` ready to handle test HTTP requests without hitting live external APIs.
/// - `mock`: The mock client handle used to pre-configure mock responses and inspect call counts.
/// - `cache`: The shared thread-safe in-memory cache instance for inspecting cached state.
fn setup_test_service() -> (VotdService, MockYouVersionClient, Arc<VotdCache>) {
    let mock = MockYouVersionClient::new();
    let cache = Arc::new(VotdCache::new());
    let service = VotdService::new(Arc::new(mock.clone()), cache.clone());
    (service, mock, cache)
}

/// Helper function that parses the JSON string body of an `HttpResponse` into a `serde_json::Value`.
///
/// Returns `Value::Null` if deserialization fails, allowing tests to make structural assertions
/// without panicking during JSON parsing.
fn parse_json(resp: &HttpResponse) -> Value {
    serde_json::from_str(&resp.body).unwrap_or(Value::Null)
}


// -------------------------------------------------------------
// 1. Happy Path Tests (Requirement 1)
// -------------------------------------------------------------

#[test]
fn test_happy_path_votd_explicit_params() {
    let (service, mock, _) = setup_test_service();
    mock.votd_responses.lock().unwrap().insert(
        195,
        Ok(UpstreamVotdDto {
            day: 195,
            passage_id: "REV.3.20".to_string(),
        }),
    );
    mock.passage_responses.lock().unwrap().insert(
        (206, "REV.3.20".to_string()),
        Ok(UpstreamPassageDto {
            id: "REV.3.20".to_string(),
            content: "Behold, I stand at the door and knock.".to_string(),
            reference: "Revelation 3:20".to_string(),
        }),
    );

    let res = service.handle_request("GET", "/votd?day=195&version=206");
    assert_eq!(res.status, 200, "Expected 200 OK for valid day and version");

    let json = parse_json(&res);
    let map = json.as_object().expect("Response must be a JSON object");
    assert_eq!(map.len(), 4, "Response must contain exactly four fields");
    assert!(map.contains_key("day"));
    assert!(map.contains_key("reference"));
    assert!(map.contains_key("text"));
    assert!(map.contains_key("version_id"));

    let votd: VotdResponse = serde_json::from_value(json).expect("Must deserialize into VotdResponse");
    assert_eq!(votd.day, 195);
    assert_eq!(votd.reference, "Revelation 3:20");
    assert_eq!(votd.text, "Behold, I stand at the door and knock.");
    assert_eq!(votd.version_id, 206);
}

#[test]
fn test_happy_path_votd_default_day() {
    let (service, mock, _) = setup_test_service();
    let today = Utc::now().ordinal();

    mock.votd_responses.lock().unwrap().insert(
        today,
        Ok(UpstreamVotdDto {
            day: today,
            passage_id: "JHN.3.16".to_string(),
        }),
    );
    mock.passage_responses.lock().unwrap().insert(
        (206, "JHN.3.16".to_string()),
        Ok(UpstreamPassageDto {
            id: "JHN.3.16".to_string(),
            content: "For God so loved the world...".to_string(),
            reference: "John 3:16".to_string(),
        }),
    );

    let res = service.handle_request("GET", "/votd?version=206");
    assert_eq!(res.status, 200);

    let votd: VotdResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(votd.day, today);
    assert_eq!(votd.version_id, 206);
}

#[test]
fn test_happy_path_votd_default_version() {
    let (service, mock, _) = setup_test_service();
    mock.votd_responses.lock().unwrap().insert(
        195,
        Ok(UpstreamVotdDto {
            day: 195,
            passage_id: "REV.3.20".to_string(),
        }),
    );
    mock.passage_responses.lock().unwrap().insert(
        (206, "REV.3.20".to_string()),
        Ok(UpstreamPassageDto {
            id: "REV.3.20".to_string(),
            content: "Behold, I stand at the door and knock.".to_string(),
            reference: "Revelation 3:20".to_string(),
        }),
    );

    let res = service.handle_request("GET", "/votd?day=195");
    assert_eq!(res.status, 200);

    let votd: VotdResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(votd.day, 195);
    assert_eq!(votd.version_id, 206);
}

#[test]
fn test_happy_path_votd_all_defaults() {
    let (service, mock, _) = setup_test_service();
    let today = Utc::now().ordinal();

    mock.votd_responses.lock().unwrap().insert(
        today,
        Ok(UpstreamVotdDto {
            day: today,
            passage_id: "GEN.1.1".to_string(),
        }),
    );
    mock.passage_responses.lock().unwrap().insert(
        (206, "GEN.1.1".to_string()),
        Ok(UpstreamPassageDto {
            id: "GEN.1.1".to_string(),
            content: "In the beginning...".to_string(),
            reference: "Genesis 1:1".to_string(),
        }),
    );

    let res = service.handle_request("GET", "/votd");
    assert_eq!(res.status, 200);

    let votd: VotdResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(votd.day, today);
    assert_eq!(votd.version_id, 206);
}

// -------------------------------------------------------------
// 2. Input Validation Tests (Requirement 2 & 3: 400 Bad Request)
// -------------------------------------------------------------

#[test]
fn test_invalid_day_zero() {
    let (service, _, _) = setup_test_service();
    let res = service.handle_request("GET", "/votd?day=0&version=206");
    assert_eq!(res.status, 400);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "INVALID_DAY");
}

#[test]
fn test_invalid_day_367() {
    let (service, _, _) = setup_test_service();
    let res = service.handle_request("GET", "/votd?day=367&version=206");
    assert_eq!(res.status, 400);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "INVALID_DAY");
}

#[test]
fn test_invalid_day_not_a_number() {
    let (service, _, _) = setup_test_service();
    let res = service.handle_request("GET", "/votd?day=abc&version=206");
    assert_eq!(res.status, 400);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "INVALID_DAY");
}

#[test]
fn test_invalid_day_negative() {
    let (service, _, _) = setup_test_service();
    let res = service.handle_request("GET", "/votd?day=-10&version=206");
    assert_eq!(res.status, 400);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "INVALID_DAY");
}

#[test]
fn test_invalid_version_not_a_number() {
    let (service, _, _) = setup_test_service();
    let res = service.handle_request("GET", "/votd?day=195&version=invalid");
    assert_eq!(res.status, 400);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "INVALID_VERSION");
}

#[test]
fn test_invalid_version_zero() {
    let (service, _, _) = setup_test_service();
    let res = service.handle_request("GET", "/votd?day=195&version=0");
    assert_eq!(res.status, 400);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "INVALID_VERSION");
}

// -------------------------------------------------------------
// 3. Upstream Failures (Requirement 3: 502 Bad Gateway)
// -------------------------------------------------------------

#[test]
fn test_upstream_day_not_found_returns_502() {
    let (service, mock, _) = setup_test_service();
    mock.votd_responses.lock().unwrap().insert(
        195,
        Err(AppError::UpstreamError("Upstream 404 Not Found".to_string())),
    );

    let res = service.handle_request("GET", "/votd?day=195&version=206");
    assert_eq!(res.status, 502);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "UPSTREAM_ERROR");
}

#[test]
fn test_upstream_passage_forbidden_returns_502() {
    let (service, mock, _) = setup_test_service();
    mock.votd_responses.lock().unwrap().insert(
        195,
        Ok(UpstreamVotdDto {
            day: 195,
            passage_id: "REV.3.20".to_string(),
        }),
    );
    mock.passage_responses.lock().unwrap().insert(
        (111, "REV.3.20".to_string()),
        Err(AppError::UpstreamError("Upstream 403 Access Denied".to_string())),
    );

    let res = service.handle_request("GET", "/votd?day=195&version=111");
    assert_eq!(res.status, 502);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "UPSTREAM_ERROR");
}

#[test]
fn test_upstream_500_server_error_returns_502() {
    let (service, mock, _) = setup_test_service();
    mock.votd_responses.lock().unwrap().insert(
        195,
        Err(AppError::UpstreamError("Upstream 500 Internal Error".to_string())),
    );

    let res = service.handle_request("GET", "/votd?day=195&version=206");
    assert_eq!(res.status, 502);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "UPSTREAM_ERROR");
}

#[test]
fn test_upstream_timeout_returns_502() {
    let (service, mock, _) = setup_test_service();
    mock.votd_responses.lock().unwrap().insert(
        195,
        Err(AppError::UpstreamError("Network timeout connecting to upstream".to_string())),
    );

    let res = service.handle_request("GET", "/votd?day=195&version=206");
    assert_eq!(res.status, 502);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "UPSTREAM_ERROR");
}

// -------------------------------------------------------------
// 4. In-Memory Caching (Requirement 4: No duplicate upstream calls)
// -------------------------------------------------------------

#[test]
fn test_caching_prevents_duplicate_upstream_calls() {
    let (service, mock, _) = setup_test_service();
    mock.votd_responses.lock().unwrap().insert(
        195,
        Ok(UpstreamVotdDto {
            day: 195,
            passage_id: "REV.3.20".to_string(),
        }),
    );
    mock.passage_responses.lock().unwrap().insert(
        (206, "REV.3.20".to_string()),
        Ok(UpstreamPassageDto {
            id: "REV.3.20".to_string(),
            content: "Behold, I stand at the door and knock.".to_string(),
            reference: "Revelation 3:20".to_string(),
        }),
    );

    let res1 = service.handle_request("GET", "/votd?day=195&version=206");
    assert_eq!(res1.status, 200);
    assert_eq!(mock.get_votd_call_count(), 1);
    assert_eq!(mock.get_passage_call_count(), 1);

    let res2 = service.handle_request("GET", "/votd?day=195&version=206");
    assert_eq!(res2.status, 200);
    assert_eq!(
        mock.get_votd_call_count(),
        1,
        "VOTD upstream call count must remain 1"
    );
    assert_eq!(
        mock.get_passage_call_count(),
        1,
        "Passage upstream call count must remain 1"
    );
}

// -------------------------------------------------------------
// 5. App Key Secrecy (Requirement 7)
// -------------------------------------------------------------

#[test]
fn test_app_key_never_leaked_in_response() {
    let (service, mock, _) = setup_test_service();
    mock.votd_responses.lock().unwrap().insert(
        195,
        Ok(UpstreamVotdDto {
            day: 195,
            passage_id: "REV.3.20".to_string(),
        }),
    );
    mock.passage_responses.lock().unwrap().insert(
        (206, "REV.3.20".to_string()),
        Ok(UpstreamPassageDto {
            id: "REV.3.20".to_string(),
            content: "Behold, I stand at the door and knock.".to_string(),
            reference: "Revelation 3:20".to_string(),
        }),
    );

    let res_ok = service.handle_request("GET", "/votd?day=195&version=206");
    assert_eq!(res_ok.status, 200, "Expected 200 OK before validating key leakage");
    assert!(
        !res_ok.body.contains(SAMPLE_APP_KEY),
        "Success response must never leak the API app key"
    );

    let res_err = service.handle_request("GET", "/votd?day=invalid");
    assert!(
        !res_err.body.contains(SAMPLE_APP_KEY),
        "Error response must never leak the API app key"
    );
}

// -------------------------------------------------------------
// 6. Optional Stretch Endpoint: GET /versions
// -------------------------------------------------------------

#[test]
fn test_get_versions_success() {
    let (service, mock, _) = setup_test_service();
    *mock.bibles_response.lock().unwrap() = Some(Ok(vec![
        BibleVersionSummary {
            id: 206,
            abbreviation: "WEB".to_string(),
            title: "World English Bible".to_string(),
        },
        BibleVersionSummary {
            id: 12,
            abbreviation: "ASV".to_string(),
            title: "American Standard Version".to_string(),
        },
    ]));

    let res = service.handle_request("GET", "/versions");
    assert_eq!(res.status, 200);

    let json = parse_json(&res);
    let versions = json
        .get("versions")
        .and_then(|v| v.as_array())
        .expect("Must have versions array");
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0]["id"], 206);
    assert_eq!(versions[0]["abbreviation"], "WEB");
}

#[test]
fn test_get_versions_upstream_failure() {
    let (service, mock, _) = setup_test_service();
    *mock.bibles_response.lock().unwrap() =
        Some(Err(AppError::UpstreamError("Upstream bibles error".to_string())));

    let res = service.handle_request("GET", "/versions");
    assert_eq!(res.status, 502);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "UPSTREAM_ERROR");
}

#[test]
fn test_get_versions_caching() {
    let (service, mock, _) = setup_test_service();
    *mock.bibles_response.lock().unwrap() = Some(Ok(vec![BibleVersionSummary {
        id: 206,
        abbreviation: "WEB".to_string(),
        title: "World English Bible".to_string(),
    }]));

    let res1 = service.handle_request("GET", "/versions");
    assert_eq!(res1.status, 200);
    assert_eq!(mock.get_bibles_call_count(), 1);

    let res2 = service.handle_request("GET", "/versions");
    assert_eq!(res2.status, 200);
    assert_eq!(
        mock.get_bibles_call_count(),
        1,
        "Subsequent /versions request must hit cache and not call upstream"
    );
}

// -------------------------------------------------------------
// 7. Route 404 Fallback
// -------------------------------------------------------------

#[test]
fn test_404_fallback_response_format() {
    let (service, _, _) = setup_test_service();
    let res = service.handle_request("GET", "/unknown-endpoint");
    assert_eq!(res.status, 404);

    let err: ErrorResponse = serde_json::from_value(parse_json(&res)).unwrap();
    assert_eq!(err.error.code, "NOT_FOUND");
}

// -------------------------------------------------------------
// 8. Standard Library TCP Socket Integration Test (No Web Frameworks)
// -------------------------------------------------------------

/// End-to-end integration test verifying standard library TCP socket communication (`std::net::TcpListener`).
///
/// This test confirms that:
/// 1. The server can dynamically bind to an OS-assigned ephemeral port (`127.0.0.1:0`).
/// 2. The server successfully accepts and reads raw HTTP requests across a physical TCP connection.
/// 3. Standard HTTP/1.1 response formatting (`Content-Length`, status line, headers) is transmitted
///    correctly across the network connection to an external HTTP client (`ureq`).
#[test]
fn test_std_net_http_server_wire_level() {
    let (service, mock, _) = setup_test_service();
    mock.votd_responses.lock().unwrap().insert(
        195,
        Ok(UpstreamVotdDto {
            day: 195,
            passage_id: "REV.3.20".to_string(),
        }),
    );
    mock.passage_responses.lock().unwrap().insert(
        (206, "REV.3.20".to_string()),
        Ok(UpstreamPassageDto {
            id: "REV.3.20".to_string(),
            content: "Behold, I stand at the door and knock.".to_string(),
            reference: "Revelation 3:20".to_string(),
        }),
    );

    let service = Arc::new(service);
    let server = SimpleHttpServer::bind("127.0.0.1:0", Arc::clone(&service)).unwrap();
    let addr = server.local_addr().unwrap();

    thread::spawn(move || {
        let _ = server.run();
    });

    // Make real HTTP call over raw TCP using ureq
    let url = format!("http://{}/votd?day=195&version=206", addr);
    let resp = ureq::get(&url).call().expect("HTTP network request must succeed");
    assert_eq!(resp.status(), 200);

    let body = resp.into_string().expect("Response body must be readable");
    let json: serde_json::Value = serde_json::from_str(&body).expect("Response must be valid JSON");
    assert_eq!(json["day"], 195);
    assert_eq!(json["version_id"], 206);
    assert_eq!(json["reference"], "Revelation 3:20");
    assert_eq!(json["text"], "Behold, I stand at the door and knock.");
}
