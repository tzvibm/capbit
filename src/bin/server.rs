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
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use capbit::{
    bootstrap, check_access, clear_all, get_meta, init, is_bootstrapped, protected, SystemCap,
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

fn cap_to_string(cap: u64) -> String {
    let mut parts = Vec::new();
    if cap & SystemCap::GRANT_READ != 0 { parts.push("READ"); }
    if cap & SystemCap::GRANT_WRITE != 0 { parts.push("WRITE"); }
    if cap & SystemCap::GRANT_DELETE != 0 { parts.push("DELETE"); }
    if cap & SystemCap::ENTITY_CREATE != 0 { parts.push("CREATE"); }
    if cap & SystemCap::CAP_WRITE != 0 { parts.push("CAP_WRITE"); }
    if cap & SystemCap::DELEGATE_WRITE != 0 { parts.push("DELEGATE"); }
    if parts.is_empty() { "NONE".into() } else { parts.join("+") }
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
    // Note: This is a simplified implementation. In production, you'd have a proper list function.
    // For now, we return an empty list - the demo tracks entities client-side too.
    Json(ApiResponse::ok(vec![]))
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
    Json(ApiResponse::ok(vec![]))
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
    Json(ApiResponse::ok(vec![]))
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
                effective_string: cap_to_string(effective),
                required,
                required_string: cap_to_string(required),
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
    println!("Capbit server running at http://{}", addr);
    println!("Demo: Open demo/index.html and set API URL to http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
