//! Capbit REST API Server with Authentication
//!
//! All mutation endpoints require Authorization: Bearer <token>

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, header::AUTHORIZATION, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use socket2::{Domain, Socket, Type};
use tower_http::cors::{Any, CorsLayer};

use capbit::{
    auth, check_access, clear_all, get_meta, init, is_bootstrapped, protected, SystemCap,
    list_accessible, list_subjects, set_cap_label, list_all_types, list_all_delegations,
    get_inheritance, get_inheritors_from_source, get_inheritance_for_object,
};

// ============================================================================
// Authentication Extractor
// ============================================================================

struct Auth(String); // Contains entity_id

#[async_trait]
impl<S> FromRequestParts<S> for Auth
where S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ApiResponse<()>>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts.headers.get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(ApiResponse::err("Missing Authorization header"))))?;

        auth::validate_session(header)
            .map(Auth)
            .map_err(|e| (StatusCode::UNAUTHORIZED, Json(ApiResponse::err(e.message))))
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Deserialize)]
struct BootstrapReq { root_id: String, password: Option<String> }

#[derive(Deserialize)]
struct LoginReq { entity_id: String, password: String }

#[derive(Deserialize)]
struct CreateTypeReq { type_name: String }

#[derive(Deserialize)]
struct CreateEntityReq { entity_type: String, id: String }

#[derive(Deserialize)]
struct CreateGrantReq { seeker: String, relation: String, scope: String }

#[derive(Deserialize)]
struct CreateCapabilityReq { scope: String, relation: String, cap_mask: u64 }

#[derive(Deserialize)]
struct CreateDelegationReq { seeker: String, scope: String, delegate: String }

#[derive(Deserialize)]
struct CheckAccessReq { subject: String, object: String, required: Option<u64> }

#[derive(Deserialize)]
struct QueryAccessibleReq { subject: String }

#[derive(Deserialize)]
struct QuerySubjectsReq { object: String }

#[derive(Deserialize)]
struct QueryDelegationReq { subject: String, object: String }

#[derive(Deserialize)]
struct QueryDelegateesReq { source: String, object: String }

#[derive(Deserialize)]
struct QueryDelegationsReq { object: String }

#[derive(Deserialize)]
struct CreateCapLabelReq { scope: String, bit: u64, label: String }

#[derive(Deserialize)]
struct CreateSessionReq { ttl_secs: Option<u64> }

#[derive(Deserialize)]
struct SetPasswordReq { entity_id: String, password: String }

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self { Self { success: true, data: Some(data), error: None } }
    fn err(msg: impl Into<String>) -> Self { Self { success: false, data: None, error: Some(msg.into()) } }
}

#[derive(Serialize)]
struct BootstrapRes { root_entity: String, token: String }

#[derive(Serialize)]
struct EntityInfo { id: String, entity_type: String }

#[derive(Serialize)]
struct GrantInfo { seeker: String, relation: String, scope: String }

#[derive(Serialize)]
struct CapabilityInfo { scope: String, relation: String, cap_mask: u64, cap_string: String }

#[derive(Serialize)]
struct CheckResult { allowed: bool, effective: u64, effective_string: String, required: u64 }

#[derive(Serialize)]
struct CapLabelInfo { scope: String, bit: u64, label: String }

#[derive(Serialize)]
struct AccessEntry { entity: String, relation: String, effective: u64, effective_string: String }

#[derive(Serialize)]
struct DelegationEntry { source: String, subject: String }

#[derive(Serialize)]
struct SessionInfo { created_at: u64, expires_at: u64 }

#[derive(Serialize)]
struct MeResponse { entity: String }

// ============================================================================
// Helpers
// ============================================================================

fn cap_to_string(cap: u64, scope: &str) -> String {
    if !scope.starts_with("_type:") { return format!("0x{:04x}", cap); }
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
    if cap & SystemCap::SYSTEM_READ != 0 { parts.push("SYSTEM_READ"); }
    if cap & SystemCap::PASSWORD_ADMIN != 0 { parts.push("PASSWORD_ADMIN"); }
    if parts.is_empty() { "NONE".into() } else { parts.join("|") }
}

// ============================================================================
// Handlers - Public (no auth)
// ============================================================================

async fn get_health() -> &'static str { "ok" }

async fn get_status() -> Json<ApiResponse<serde_json::Value>> {
    let bootstrapped = is_bootstrapped().unwrap_or(false);
    let root = get_meta("root_entity").ok().flatten();
    Json(ApiResponse::ok(serde_json::json!({ "bootstrapped": bootstrapped, "root_entity": root })))
}

async fn post_bootstrap(Json(req): Json<BootstrapReq>) -> (StatusCode, Json<ApiResponse<BootstrapRes>>) {
    let result = match &req.password {
        Some(pwd) => auth::bootstrap_with_password(&req.root_id, pwd),
        None => auth::bootstrap_with_token(&req.root_id),
    };
    match result {
        Ok(r) => (StatusCode::OK, Json(ApiResponse::ok(BootstrapRes { root_entity: r.root_entity, token: r.token }))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn post_login(Json(req): Json<LoginReq>) -> (StatusCode, Json<ApiResponse<BootstrapRes>>) {
    match auth::login(&req.entity_id, &req.password) {
        Ok(token) => (StatusCode::OK, Json(ApiResponse::ok(BootstrapRes { root_entity: req.entity_id, token }))),
        Err(e) => (StatusCode::UNAUTHORIZED, Json(ApiResponse::err(e.message))),
    }
}

// ============================================================================
// Handlers - Authenticated
// ============================================================================

async fn get_me(Auth(entity): Auth) -> Json<ApiResponse<MeResponse>> {
    Json(ApiResponse::ok(MeResponse { entity }))
}

async fn post_session(Auth(entity): Auth, Json(req): Json<CreateSessionReq>) -> (StatusCode, Json<ApiResponse<String>>) {
    match auth::create_session(&entity, req.ttl_secs) {
        Ok(token) => (StatusCode::OK, Json(ApiResponse::ok(token))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn post_password(Auth(actor): Auth, Json(req): Json<SetPasswordReq>) -> (StatusCode, Json<ApiResponse<String>>) {
    // Can set own password, or need PASSWORD_ADMIN on the entity's type
    if actor != req.entity_id {
        let entity_type = req.entity_id.split(':').next().unwrap_or("");
        let type_scope = format!("_type:{}", entity_type);
        let caps = check_access(&actor, &type_scope, None).unwrap_or(0);
        if (caps & SystemCap::PASSWORD_ADMIN) == 0 {
            return (StatusCode::FORBIDDEN, Json(ApiResponse::err("Can only set your own password, or requires PASSWORD_ADMIN on entity type")));
        }
    }
    match auth::set_password(&req.entity_id, &req.password) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("password set".into()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn get_sessions(Auth(entity): Auth) -> Json<ApiResponse<Vec<SessionInfo>>> {
    match auth::list_sessions(&entity) {
        Ok(sessions) => Json(ApiResponse::ok(sessions.into_iter().map(|s| SessionInfo { created_at: s.created_at, expires_at: s.expires_at }).collect())),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn delete_sessions(Auth(entity): Auth) -> (StatusCode, Json<ApiResponse<u64>>) {
    match auth::revoke_all_sessions(&entity) {
        Ok(count) => (StatusCode::OK, Json(ApiResponse::ok(count))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn post_type(Auth(actor): Auth, Json(req): Json<CreateTypeReq>) -> (StatusCode, Json<ApiResponse<String>>) {
    match protected::create_type(&actor, &req.type_name) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(format!("_type:{}", req.type_name)))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn post_entity(Auth(actor): Auth, Json(req): Json<CreateEntityReq>) -> (StatusCode, Json<ApiResponse<String>>) {
    match protected::create_entity(&actor, &req.entity_type, &req.id) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok(format!("{}:{}", req.entity_type, req.id)))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn get_entities(Auth(actor): Auth) -> Json<ApiResponse<Vec<EntityInfo>>> {
    match protected::list_entities(&actor) {
        Ok(entities) => Json(ApiResponse::ok(entities.into_iter().filter_map(|id| {
            id.split_once(':').map(|(t, _)| EntityInfo { id: id.clone(), entity_type: t.into() })
        }).collect())),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn get_types(Auth(actor): Auth) -> Json<ApiResponse<Vec<String>>> {
    // Requires SYSTEM_READ on _type:_type to see types
    let caps = check_access(&actor, "_type:_type", None).unwrap_or(0);
    if (caps & SystemCap::SYSTEM_READ) == 0 {
        return Json(ApiResponse::err("Requires SYSTEM_READ on _type:_type"));
    }
    match list_all_types() {
        Ok(types) => Json(ApiResponse::ok(types)),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn post_grant(Auth(actor): Auth, Json(req): Json<CreateGrantReq>) -> (StatusCode, Json<ApiResponse<String>>) {
    match protected::set_grant(&actor, &req.seeker, &req.relation, &req.scope) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("created".into()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn get_grants(Auth(actor): Auth) -> Json<ApiResponse<Vec<GrantInfo>>> {
    match protected::list_grants(&actor) {
        Ok(grants) => Json(ApiResponse::ok(grants.into_iter().map(|(seeker, relation, scope)| GrantInfo { seeker, relation, scope }).collect())),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn post_capability(Auth(actor): Auth, Json(req): Json<CreateCapabilityReq>) -> (StatusCode, Json<ApiResponse<String>>) {
    match protected::set_capability(&actor, &req.scope, &req.relation, req.cap_mask) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("created".into()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn get_capabilities(Auth(actor): Auth) -> Json<ApiResponse<Vec<CapabilityInfo>>> {
    match protected::list_capabilities(&actor) {
        Ok(caps) => Json(ApiResponse::ok(caps.into_iter().map(|(scope, relation, cap_mask)| {
            CapabilityInfo { cap_string: cap_to_string(cap_mask, &scope), scope, relation, cap_mask }
        }).collect())),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn post_delegation(Auth(actor): Auth, Json(req): Json<CreateDelegationReq>) -> (StatusCode, Json<ApiResponse<String>>) {
    match protected::set_delegation(&actor, &req.seeker, &req.scope, &req.delegate) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("created".into()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn post_check(Auth(actor): Auth, Json(req): Json<CheckAccessReq>) -> Json<ApiResponse<CheckResult>> {
    // User can only check access for themselves, or if they have GRANT_READ on the object
    if actor != req.subject {
        let caps = check_access(&actor, &req.object, None).unwrap_or(0);
        if (caps & SystemCap::GRANT_READ) == 0 {
            return Json(ApiResponse::err("Can only check your own access, or requires GRANT_READ on object"));
        }
    }
    match check_access(&req.subject, &req.object, None) {
        Ok(effective) => {
            let required = req.required.unwrap_or(0);
            Json(ApiResponse::ok(CheckResult {
                allowed: (effective & required) == required,
                effective, effective_string: cap_to_string(effective, &req.object), required,
            }))
        }
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn post_query_accessible(Auth(actor): Auth, Json(req): Json<QueryAccessibleReq>) -> Json<ApiResponse<Vec<AccessEntry>>> {
    // User can query their own access, or anyone's if they have SYSTEM_READ
    if actor != req.subject {
        let caps = check_access(&actor, "_type:_type", None).unwrap_or(0);
        if (caps & SystemCap::SYSTEM_READ) == 0 {
            return Json(ApiResponse::err("Can only query your own access, or requires SYSTEM_READ"));
        }
    }
    match list_accessible(&req.subject) {
        Ok(results) => Json(ApiResponse::ok(results.into_iter().map(|(object, relation)| {
            let effective = check_access(&req.subject, &object, None).unwrap_or(0);
            AccessEntry { effective_string: cap_to_string(effective, &object), entity: object, relation, effective }
        }).collect())),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn post_query_subjects(Auth(actor): Auth, Json(req): Json<QuerySubjectsReq>) -> Json<ApiResponse<Vec<AccessEntry>>> {
    // Requires GRANT_READ on the object to see who has access
    let caps = check_access(&actor, &req.object, None).unwrap_or(0);
    if (caps & SystemCap::GRANT_READ) == 0 {
        return Json(ApiResponse::err("Requires GRANT_READ on object"));
    }
    match list_subjects(&req.object) {
        Ok(results) => Json(ApiResponse::ok(results.into_iter().map(|(subject, relation)| {
            let effective = check_access(&subject, &req.object, None).unwrap_or(0);
            AccessEntry { effective_string: cap_to_string(effective, &req.object), entity: subject, relation, effective }
        }).collect())),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

// Delegation query: Who delegated to this subject on this object?
async fn post_query_delegators(Auth(actor): Auth, Json(req): Json<QueryDelegationReq>) -> Json<ApiResponse<Vec<String>>> {
    // Requires DELEGATE_READ on the object
    let caps = check_access(&actor, &req.object, None).unwrap_or(0);
    if (caps & SystemCap::DELEGATE_READ) == 0 {
        return Json(ApiResponse::err("Requires DELEGATE_READ on object"));
    }
    match get_inheritance(&req.subject, &req.object) {
        Ok(sources) => Json(ApiResponse::ok(sources)),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

// Delegation query: Who did this source delegate to on this object?
async fn post_query_delegatees(Auth(actor): Auth, Json(req): Json<QueryDelegateesReq>) -> Json<ApiResponse<Vec<String>>> {
    // Requires DELEGATE_READ on the object
    let caps = check_access(&actor, &req.object, None).unwrap_or(0);
    if (caps & SystemCap::DELEGATE_READ) == 0 {
        return Json(ApiResponse::err("Requires DELEGATE_READ on object"));
    }
    match get_inheritors_from_source(&req.source, &req.object) {
        Ok(subjects) => Json(ApiResponse::ok(subjects)),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

// Delegation query: All delegations on this object
async fn post_query_delegations(Auth(actor): Auth, Json(req): Json<QueryDelegationsReq>) -> Json<ApiResponse<Vec<DelegationEntry>>> {
    // Requires DELEGATE_READ on the object
    let caps = check_access(&actor, &req.object, None).unwrap_or(0);
    if (caps & SystemCap::DELEGATE_READ) == 0 {
        return Json(ApiResponse::err("Requires DELEGATE_READ on object"));
    }
    match get_inheritance_for_object(&req.object) {
        Ok(pairs) => Json(ApiResponse::ok(pairs.into_iter().map(|(source, subject)| DelegationEntry { source, subject }).collect())),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn post_reset(Auth(actor): Auth) -> (StatusCode, Json<ApiResponse<String>>) {
    // Allow reset if:
    // 1. Actor has TYPE_ADMIN on _type:_type, OR
    // 2. Actor IS the root entity (handles corrupted DB where grants are missing)
    let required = SystemCap::TYPE_CREATE | SystemCap::TYPE_DELETE | SystemCap::SYSTEM_READ;
    let caps = check_access(&actor, "_type:_type", None).unwrap_or(0);
    let is_root = get_meta("root_entity").ok().flatten().as_deref() == Some(&actor);
    if (caps & required) != required && !is_root {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::err("Requires type admin on _type:_type")));
    }
    match clear_all() {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("reset".into()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn post_cap_label(Auth(actor): Auth, Json(req): Json<CreateCapLabelReq>) -> (StatusCode, Json<ApiResponse<String>>) {
    // Require CAP_WRITE on the scope
    let caps = check_access(&actor, &req.scope, None).unwrap_or(0);
    if (caps & SystemCap::CAP_WRITE) == 0 {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::err("Requires CAP_WRITE on scope")));
    }
    match set_cap_label(&req.scope, req.bit, &req.label) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("created".into()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::err(e.message))),
    }
}

async fn get_cap_labels(Auth(actor): Auth) -> Json<ApiResponse<Vec<CapLabelInfo>>> {
    match protected::list_cap_labels(&actor) {
        Ok(labels) => Json(ApiResponse::ok(labels.into_iter().map(|(scope, bit, label)| CapLabelInfo { scope, bit, label }).collect())),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

#[derive(Serialize)]
struct DelegationInfo { seeker: String, scope: String, source: String }

async fn get_delegations(Auth(_actor): Auth) -> Json<ApiResponse<Vec<DelegationInfo>>> {
    match list_all_delegations() {
        Ok(delegs) => Json(ApiResponse::ok(delegs.into_iter().map(|(seeker, scope, source)| DelegationInfo { seeker, scope, source }).collect())),
        Err(e) => Json(ApiResponse::err(e.message)),
    }
}

async fn serve_demo() -> impl IntoResponse {
    Html(include_str!("../../demo/index.html"))
}

async fn serve_styles() -> impl IntoResponse {
    (
        [("content-type", "text/css")],
        include_str!("../../demo/styles.css"),
    )
}

async fn serve_app_js() -> impl IntoResponse {
    (
        [("content-type", "application/javascript")],
        include_str!("../../demo/app.js"),
    )
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    let db_path = std::env::var("CAPBIT_DB").unwrap_or_else(|_| "./data/capbit.mdb".into());
    println!("Initializing database at: {}", db_path);
    init(&db_path).expect("Failed to initialize database");

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let app = Router::new()
        // Public
        .route("/", get(serve_demo))
        .route("/styles.css", get(serve_styles))
        .route("/app.js", get(serve_app_js))
        .route("/health", get(get_health))
        .route("/status", get(get_status))
        .route("/bootstrap", post(post_bootstrap))
        .route("/login", post(post_login))
        .route("/reset", post(post_reset))
        // Read (no auth for now - add if needed)
        .route("/entities", get(get_entities))
        .route("/types", get(get_types))
        .route("/grants", get(get_grants))
        .route("/capabilities", get(get_capabilities))
        .route("/cap-labels", get(get_cap_labels))
        .route("/delegations", get(get_delegations))
        .route("/check", post(post_check))
        .route("/query/accessible", post(post_query_accessible))
        .route("/query/subjects", post(post_query_subjects))
        .route("/query/delegators", post(post_query_delegators))
        .route("/query/delegatees", post(post_query_delegatees))
        .route("/query/delegations", post(post_query_delegations))
        // Authenticated
        .route("/me", get(get_me))
        .route("/session", post(post_session))
        .route("/password", post(post_password))
        .route("/sessions", get(get_sessions))
        .route("/sessions", delete(delete_sessions))
        .route("/type", post(post_type))
        .route("/entity", post(post_entity))
        .route("/grant", post(post_grant))
        .route("/capability", post(post_capability))
        .route("/delegation", post(post_delegation))
        .route("/cap-label", post(post_cap_label))
        .layer(cors);

    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().expect("Invalid PORT");
    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();

    let socket = Socket::new(Domain::IPV4, Type::STREAM, None).expect("Failed to create socket");
    socket.set_reuse_address(true).ok();
    socket.set_nonblocking(true).ok();
    if let Err(e) = socket.bind(&addr.into()) {
        eprintln!("Failed to bind to port {}: {}", port, e);
        std::process::exit(1);
    }
    socket.listen(128).expect("Failed to listen");

    let listener = tokio::net::TcpListener::from_std(socket.into()).expect("Failed to create listener");
    println!("Capbit server at http://localhost:{}", port);

    axum::serve(listener, app).await.ok();
}






