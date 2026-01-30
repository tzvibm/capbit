//! Capbit REST API Server
//!
//! Run with: cargo run --features server --bin capbit-server
//!
//! Endpoints:
//!   POST /bootstrap          - Bootstrap system
//!   GET  /status             - Get system status
//!   POST /entity             - Create entity
//!   GET  /entities           - List entities
//!   POST /grant              - Create grant
//!   GET  /grants             - List grants
//!   POST /capability         - Define capability
//!   GET  /capabilities       - List capabilities
//!   POST /check              - Check access
//!   POST /reset              - Reset database (dev only)

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use capbit::{
    bootstrap, check_access, clear_all, get_meta, init, is_bootstrapped, protected, SystemCap,
    list_all_entities, list_all_grants, list_all_capabilities,
};

// ============================================================================
// State
// ============================================================================

struct AppState {
    _lock: Mutex<()>, // Serialize writes
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Deserialize)]
struct BootstrapReq {
    root_id: String,
}

#[derive(Deserialize)]
struct CreateEntityReq {
    actor: String,
    entity_type: String,
    id: String,
}

#[derive(Deserialize)]
struct CreateGrantReq {
    actor: String,
    seeker: String,
    relation: String,
    scope: String,
}

#[derive(Deserialize)]
struct CreateCapabilityReq {
    actor: String,
    scope: String,
    relation: String,
    cap_mask: u64,
}

#[derive(Deserialize)]
struct CheckAccessReq {
    subject: String,
    object: String,
    required: Option<u64>,
}

#[derive(Serialize)]
struct StatusRes {
    bootstrapped: bool,
    root_entity: Option<String>,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    fn err(msg: impl Into<String>) -> Self {
        Self { success: false, data: None, error: Some(msg.into()) }
    }
}

#[derive(Serialize)]
struct EntityInfo {
    id: String,
    entity_type: String,
}

#[derive(Serialize)]
struct GrantInfo {
    seeker: String,
    relation: String,
    scope: String,
}

#[derive(Serialize)]
struct CapabilityInfo {
    scope: String,
    relation: String,
    cap_mask: u64,
    cap_string: String,
}

#[derive(Serialize)]
struct CheckResult {
    allowed: bool,
    effective: u64,
    effective_string: String,
    required: u64,
    required_string: String,
}

// ============================================================================
// Helpers
// ============================================================================

/// Convert SystemCap bitmask to human-readable string (for _type:* entities only)
fn syscap_to_string(cap: u64) -> String {
    let mut parts = Vec::new();
    if cap & SystemCap::TYPE_CREATE != 0 { parts.push("TYPE_CREATE"); }
    if cap & SystemCap::TYPE_DELETE != 0 { parts.push("TYPE_DELETE"); }
    if cap & SystemCap::ENTITY_CREATE != 0 { parts.push("ENTITY_CREATE"); }
    if cap & SystemCap::ENTITY_DELETE != 0 { parts.push("ENTITY_DELETE"); }
    if cap & SystemCap::GRANT_READ != 0 { parts.push("GRANT_READ"); }
    if cap & SystemCap::GRANT_WRITE != 0 { parts.push("GRANT_WRITE"); }
    if cap & SystemCap::GRANT_DELETE != 0 { parts.push("GRANT_DELETE"); }
    if cap & SystemCap::CAP_READ != 0 { parts.push("CAP_READ"); }
    if cap & SystemCap::CAP_WRITE != 0 { parts.push("CAP_WRITE"); }
    if cap & SystemCap::CAP_DELETE != 0 { parts.push("CAP_DELETE"); }
    if cap & SystemCap::DELEGATE_READ != 0 { parts.push("DELEGATE_READ"); }
    if cap & SystemCap::DELEGATE_WRITE != 0 { parts.push("DELEGATE_WRITE"); }
    if cap & SystemCap::DELEGATE_DELETE != 0 { parts.push("DELEGATE_DELETE"); }
    if parts.is_empty() { "NONE".into() } else { parts.join(" | ") }
}

/// Convert cap to string - use SystemCap names for _type:* scopes, otherwise just hex
/// Org-defined capabilities have meanings defined by the org, not by Capbit
fn cap_to_string(cap: u64, scope: &str) -> String {
    if scope.starts_with("_type:") {
        syscap_to_string(cap)
    } else {
        // Org capabilities - just show hex, org defines the meaning
        format!("0x{:04x}", cap)
    }
}

// ============================================================================
// Handlers
// ============================================================================

async fn get_status() -> Json<ApiResponse<StatusRes>> {
    let bootstrapped = is_bootstrapped().unwrap_or(false);
    let root_entity = get_meta("root_entity").ok().flatten();
    Json(ApiResponse::ok(StatusRes { bootstrapped, root_entity }))
}

async fn post_bootstrap(
    Json(req): Json<BootstrapReq>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    match bootstrap(&req.root_id) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(format!("user:{}", req.root_id)))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn post_entity(
    Json(req): Json<CreateEntityReq>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    match protected::create_entity(&req.actor, &req.entity_type, &req.id) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(format!("{}:{}", req.entity_type, req.id)))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn get_entities() -> Json<ApiResponse<Vec<EntityInfo>>> {
    match list_all_entities() {
        Ok(entities) => {
            let infos: Vec<EntityInfo> = entities
                .into_iter()
                .filter_map(|id| {
                    // Parse "type:name" format
                    id.split_once(':').map(|(t, _)| EntityInfo {
                        id: id.clone(),
                        entity_type: t.to_string(),
                    })
                })
                .collect();
            Json(ApiResponse::ok(infos))
        }
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn post_grant(
    Json(req): Json<CreateGrantReq>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    match protected::set_grant(&req.actor, &req.seeker, &req.relation, &req.scope) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("created".into()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn get_grants() -> Json<ApiResponse<Vec<GrantInfo>>> {
    match list_all_grants() {
        Ok(grants) => {
            let infos: Vec<GrantInfo> = grants
                .into_iter()
                .map(|(seeker, relation, scope)| GrantInfo { seeker, relation, scope })
                .collect();
            Json(ApiResponse::ok(infos))
        }
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn post_capability(
    Json(req): Json<CreateCapabilityReq>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    match protected::set_capability(&req.actor, &req.scope, &req.relation, req.cap_mask) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("created".into()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn get_capabilities() -> Json<ApiResponse<Vec<CapabilityInfo>>> {
    match list_all_capabilities() {
        Ok(caps) => {
            let infos: Vec<CapabilityInfo> = caps
                .into_iter()
                .map(|(scope, relation, cap_mask)| {
                    let cap_string = cap_to_string(cap_mask, &scope);
                    CapabilityInfo {
                        scope,
                        relation,
                        cap_string,
                        cap_mask,
                    }
                })
                .collect();
            Json(ApiResponse::ok(infos))
        }
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn post_check(
    Json(req): Json<CheckAccessReq>,
) -> Json<ApiResponse<CheckResult>> {
    match check_access(&req.subject, &req.object, None) {
        Ok(effective) => {
            let required = req.required.unwrap_or(0);
            let allowed = (effective & required) == required;
            Json(ApiResponse::ok(CheckResult {
                allowed,
                effective,
                effective_string: cap_to_string(effective, &req.object),
                required,
                required_string: cap_to_string(required, &req.object),
            }))
        }
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn post_reset() -> (StatusCode, Json<ApiResponse<String>>) {
    match clear_all() {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("reset".into()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn serve_demo() -> impl IntoResponse {
    Html(include_str!("../../demo/index.html").replace(
        r#"value="http://localhost:3000""#,
        r#"value="""#
    ))
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize database
    let db_path = std::env::var("CAPBIT_DB").unwrap_or_else(|_| "./data/capbit.mdb".into());
    println!("Initializing database at: {}", db_path);
    init(&db_path).expect("Failed to initialize database");

    // CORS for demo
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // State
    let state = AppState { _lock: Mutex::new(()) };

    // Router
    let app = Router::new()
        .route("/", get(serve_demo))
        .route("/status", get(get_status))
        .route("/bootstrap", post(post_bootstrap))
        .route("/entity", post(post_entity))
        .route("/entities", get(get_entities))
        .route("/grant", post(post_grant))
        .route("/grants", get(get_grants))
        .route("/capability", post(post_capability))
        .route("/capabilities", get(get_capabilities))
        .route("/check", post(post_check))
        .route("/reset", post(post_reset))
        .layer(cors)
        .with_state(std::sync::Arc::new(state));

    // Bind
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("0.0.0.0:{}", port);
    println!("Capbit server running at http://localhost:{}", port);
    println!("Open http://localhost:{} in your browser", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
