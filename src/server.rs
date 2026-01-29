//! Capbit HTTP Server
//!
//! High-performance REST API for the capbit access control system.
//!
//! Run with: cargo run --release --features server --bin capbit-server

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod core;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct InitRequest {
    db_path: String,
}

#[derive(Debug, Deserialize)]
struct RelationshipRequest {
    subject: String,
    rel_type: String,
    object: String,
}

#[derive(Debug, Deserialize)]
struct BatchRelationshipRequest {
    entries: Vec<RelationshipRequest>,
}

#[derive(Debug, Deserialize)]
struct CapabilityRequest {
    entity: String,
    rel_type: String,
    cap_mask: u64,
}

#[derive(Debug, Deserialize)]
struct BatchCapabilityRequest {
    entries: Vec<CapabilityRequest>,
}

#[derive(Debug, Deserialize)]
struct InheritanceRequest {
    subject: String,
    object: String,
    source: String,
}

#[derive(Debug, Deserialize)]
struct BatchInheritanceRequest {
    entries: Vec<InheritanceRequest>,
}

#[derive(Debug, Deserialize)]
struct CapLabelRequest {
    entity: String,
    cap_bit: u64,
    label: String,
}

#[derive(Debug, Deserialize)]
struct AccessQuery {
    max_depth: Option<usize>,
}

#[derive(Debug, Serialize)]
struct EpochResponse {
    epoch: u64,
}

#[derive(Debug, Serialize)]
struct CountResponse {
    count: u64,
}

#[derive(Debug, Serialize)]
struct RelationshipsResponse {
    rel_types: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CapabilityResponse {
    cap_mask: Option<u64>,
}

#[derive(Debug, Serialize)]
struct InheritanceResponse {
    sources: Vec<String>,
}

#[derive(Debug, Serialize)]
struct InheritorsResponse {
    subjects: Vec<String>,
}

#[derive(Debug, Serialize)]
struct InheritanceRulesResponse {
    /// Array of {source, subject} pairs
    rules: Vec<InheritanceRule>,
}

#[derive(Debug, Serialize)]
struct InheritanceRule {
    source: String,
    subject: String,
}

#[derive(Debug, Serialize)]
struct AccessResponse {
    effective_cap: u64,
}

#[derive(Debug, Serialize)]
struct HasCapabilityResponse {
    has_capability: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

// ============================================================================
// App State
// ============================================================================

#[derive(Clone)]
struct AppState {
    initialized: Arc<std::sync::atomic::AtomicBool>,
}

impl AppState {
    fn new() -> Self {
        Self {
            initialized: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    fn is_initialized(&self) -> bool {
        self.initialized.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn set_initialized(&self) {
        self.initialized.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

// ============================================================================
// Handlers
// ============================================================================

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn init_db(
    State(state): State<AppState>,
    Json(req): Json<InitRequest>,
) -> Result<Json<EpochResponse>, (StatusCode, Json<ErrorResponse>)> {
    core::init(&req.db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    state.set_initialized();

    Ok(Json(EpochResponse {
        epoch: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    }))
}

// Relationships

async fn set_relationship(
    State(state): State<AppState>,
    Json(req): Json<RelationshipRequest>,
) -> Result<Json<EpochResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let epoch = core::set_relationship(&req.subject, &req.rel_type, &req.object).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(EpochResponse { epoch }))
}

async fn batch_set_relationships(
    State(state): State<AppState>,
    Json(req): Json<BatchRelationshipRequest>,
) -> Result<Json<CountResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let entries: Vec<_> = req
        .entries
        .into_iter()
        .map(|e| (e.subject, e.rel_type, e.object))
        .collect();

    let count = core::batch_set_relationships(&entries).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(CountResponse { count }))
}

async fn get_relationships(
    State(state): State<AppState>,
    Path((subject, object)): Path<(String, String)>,
) -> Result<Json<RelationshipsResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let rel_types = core::get_relationships(&subject, &object).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(RelationshipsResponse { rel_types }))
}

async fn delete_relationship(
    State(state): State<AppState>,
    Path((subject, rel_type, object)): Path<(String, String, String)>,
) -> Result<Json<bool>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let deleted = core::delete_relationship(&subject, &rel_type, &object).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(deleted))
}

// Capabilities

async fn set_capability(
    State(state): State<AppState>,
    Json(req): Json<CapabilityRequest>,
) -> Result<Json<EpochResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let epoch = core::set_capability(&req.entity, &req.rel_type, req.cap_mask).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(EpochResponse { epoch }))
}

async fn batch_set_capabilities(
    State(state): State<AppState>,
    Json(req): Json<BatchCapabilityRequest>,
) -> Result<Json<CountResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let entries: Vec<_> = req
        .entries
        .into_iter()
        .map(|e| (e.entity, e.rel_type, e.cap_mask))
        .collect();

    let count = core::batch_set_capabilities(&entries).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(CountResponse { count }))
}

async fn get_capability(
    State(state): State<AppState>,
    Path((entity, rel_type)): Path<(String, String)>,
) -> Result<Json<CapabilityResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let cap_mask = core::get_capability(&entity, &rel_type).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(CapabilityResponse { cap_mask }))
}

// Inheritance

async fn set_inheritance(
    State(state): State<AppState>,
    Json(req): Json<InheritanceRequest>,
) -> Result<Json<EpochResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let epoch = core::set_inheritance(&req.subject, &req.object, &req.source).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(EpochResponse { epoch }))
}

async fn batch_set_inheritance(
    State(state): State<AppState>,
    Json(req): Json<BatchInheritanceRequest>,
) -> Result<Json<CountResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let entries: Vec<_> = req
        .entries
        .into_iter()
        .map(|e| (e.subject, e.object, e.source))
        .collect();

    let count = core::batch_set_inheritance(&entries).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(CountResponse { count }))
}

async fn get_inheritance(
    State(state): State<AppState>,
    Path((subject, object)): Path<(String, String)>,
) -> Result<Json<InheritanceResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let sources = core::get_inheritance(&subject, &object).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(InheritanceResponse { sources }))
}

async fn delete_inheritance(
    State(state): State<AppState>,
    Path((subject, object, source)): Path<(String, String, String)>,
) -> Result<Json<bool>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let deleted = core::delete_inheritance(&subject, &object, &source).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(deleted))
}

async fn get_inheritors_from_source(
    State(state): State<AppState>,
    Path((source, object)): Path<(String, String)>,
) -> Result<Json<InheritorsResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let subjects = core::get_inheritors_from_source(&source, &object).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(InheritorsResponse { subjects }))
}

async fn get_inheritance_for_object(
    State(state): State<AppState>,
    Path(object): Path<String>,
) -> Result<Json<InheritanceRulesResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let rules = core::get_inheritance_for_object(&object)
        .map(|v| {
            v.into_iter()
                .map(|(source, subject)| InheritanceRule { source, subject })
                .collect()
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e.message }),
            )
        })?;

    Ok(Json(InheritanceRulesResponse { rules }))
}

// Labels

async fn set_cap_label(
    State(state): State<AppState>,
    Json(req): Json<CapLabelRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    core::set_cap_label(&req.entity, req.cap_bit, &req.label).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

// Access checks

async fn check_access(
    State(state): State<AppState>,
    Path((subject, object)): Path<(String, String)>,
    Query(query): Query<AccessQuery>,
) -> Result<Json<AccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let effective_cap = core::check_access(&subject, &object, query.max_depth).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(AccessResponse { effective_cap }))
}

async fn has_capability(
    State(state): State<AppState>,
    Path((subject, object, required_cap)): Path<(String, String, u64)>,
) -> Result<Json<HasCapabilityResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !state.is_initialized() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Database not initialized".to_string() }),
        ));
    }

    let has = core::has_capability(&subject, &object, required_cap).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.message }),
        )
    })?;

    Ok(Json(HasCapabilityResponse { has_capability: has }))
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    let state = AppState::new();

    // Check for --db-path argument
    let args: Vec<String> = std::env::args().collect();
    let mut db_path: Option<String> = None;
    let mut port: u16 = 3000;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--db-path" | "-d" => {
                if i + 1 < args.len() {
                    db_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(3000);
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("capbit-server - High-performance access control server\n");
                println!("USAGE:");
                println!("    capbit-server [OPTIONS]\n");
                println!("OPTIONS:");
                println!("    -d, --db-path <PATH>  Initialize with database at PATH");
                println!("    -p, --port <PORT>     Listen on PORT (default: 3000)");
                println!("    -h, --help            Show this help message");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Auto-initialize if db_path provided
    if let Some(path) = db_path {
        match core::init(&path) {
            Ok(()) => {
                state.set_initialized();
                println!("Database initialized at: {}", path);
            }
            Err(e) => {
                eprintln!("Failed to initialize database: {}", e.message);
                std::process::exit(1);
            }
        }
    }

    let app = Router::new()
        // Health
        .route("/health", get(health))
        // Init
        .route("/init", post(init_db))
        // Relationships
        .route("/relationships", post(set_relationship))
        .route("/relationships/batch", post(batch_set_relationships))
        .route("/relationships/:subject/:object", get(get_relationships))
        .route(
            "/relationships/:subject/:rel_type/:object",
            delete(delete_relationship),
        )
        // Capabilities
        .route("/capabilities", post(set_capability))
        .route("/capabilities/batch", post(batch_set_capabilities))
        .route("/capabilities/:entity/:rel_type", get(get_capability))
        // Inheritance
        .route("/inheritance", post(set_inheritance))
        .route("/inheritance/batch", post(batch_set_inheritance))
        .route("/inheritance/:subject/:object", get(get_inheritance))
        .route("/inheritance/:subject/:object/:source", delete(delete_inheritance))
        .route("/inheritance/by-source/:source/:object", get(get_inheritors_from_source))
        .route("/inheritance/by-object/:object", get(get_inheritance_for_object))
        // Labels
        .route("/labels/cap", post(set_cap_label))
        // Access checks
        .route("/access/:subject/:object", get(check_access))
        .route("/access/:subject/:object/:required_cap", get(has_capability))
        // Middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("capbit-server v{} listening on {}", env!("CARGO_PKG_VERSION"), addr);
    println!("\nEndpoints:");
    println!("  POST   /init                                    Initialize database");
    println!("  GET    /health                                  Health check");
    println!("  POST   /relationships                           Set relationship");
    println!("  POST   /relationships/batch                     Batch set relationships");
    println!("  GET    /relationships/:subject/:object          Get relationship types");
    println!("  DELETE /relationships/:subject/:rel_type/:obj   Delete relationship");
    println!("  POST   /capabilities                            Set capability");
    println!("  POST   /capabilities/batch                      Batch set capabilities");
    println!("  GET    /capabilities/:entity/:rel_type          Get capability");
    println!("  POST   /inheritance                             Set inheritance");
    println!("  POST   /inheritance/batch                       Batch set inheritance");
    println!("  GET    /inheritance/:subject/:object            Get inheritance sources");
    println!("  DELETE /inheritance/:subject/:object/:source    Delete inheritance");
    println!("  GET    /inheritance/by-source/:source/:object   Get inheritors from source");
    println!("  GET    /inheritance/by-object/:object           Get all inheritance for object");
    println!("  POST   /labels/cap                              Set capability label");
    println!("  GET    /access/:subject/:object                 Check access");
    println!("  GET    /access/:subject/:object/:cap            Has capability");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
