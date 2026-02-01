//! Capbit Minimal REST API Server

use axum::{extract::Query, response::Html, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use capbit::*;

// Generic response wrapper
#[derive(Serialize)]
struct R<T> { ok: bool, #[serde(skip_serializing_if = "Option::is_none")] data: Option<T>, #[serde(skip_serializing_if = "Option::is_none")] error: Option<String> }

fn ok<T: Serialize>(v: T) -> Json<R<T>> { Json(R { ok: true, data: Some(v), error: None }) }
fn err<T: Serialize>(e: impl ToString) -> Json<R<T>> { Json(R { ok: false, data: None, error: Some(e.to_string()) }) }
fn wrap<T: Serialize>(r: Result<T>) -> Json<R<T>> { match r { Ok(v) => ok(v), Err(e) => err(e) } }

// Requests
#[derive(Deserialize)] struct Grant { actor: u64, subject: u64, object: u64, mask: u64 }
#[derive(Deserialize)] struct Revoke { actor: u64, subject: u64, object: u64 }
#[derive(Deserialize)] struct Check { subject: u64, object: u64, required: Option<u64> }
#[derive(Deserialize)] struct List { actor: Option<u64>, subject: Option<u64>, object: Option<u64> }
#[derive(Deserialize)] struct Role { actor: u64, object: u64, role_id: u64, mask: u64 }
#[derive(Deserialize)] struct Inherit { actor: u64, object: u64, child: u64, parent: u64 }
#[derive(Deserialize)] struct Label { id: u64, name: String }
#[derive(Deserialize)] struct Entity { name: String }
#[derive(Deserialize)] struct EntityUpdate { id: u64, name: String }
#[derive(Deserialize)] struct EntityDelete { id: u64 }
#[derive(Deserialize)] struct Bootstrap { root_id: u64 }
#[derive(Deserialize)] struct Reset { actor: u64 }

// Responses
#[derive(Serialize)] struct Status { bootstrapped: bool, root: Option<u64> }
#[derive(Serialize)] struct CheckRes { allowed: bool, mask: u64 }
#[derive(Serialize)] struct Entry { id: u64, mask: u64, #[serde(skip_serializing_if = "Option::is_none")] label: Option<String> }

// Handlers
async fn h_status() -> Json<R<Status>> {
    ok(Status { bootstrapped: is_bootstrapped().unwrap_or(false), root: get_root().ok().flatten() })
}

async fn h_bootstrap(Json(r): Json<Bootstrap>) -> Json<R<u64>> { wrap(bootstrap(r.root_id).map(|_| r.root_id)) }
async fn h_grant(Json(r): Json<Grant>) -> Json<R<bool>> { wrap(protected_grant(r.actor, r.subject, r.object, r.mask).map(|_| true)) }
async fn h_revoke(Json(r): Json<Revoke>) -> Json<R<bool>> { wrap(protected_revoke(r.actor, r.subject, r.object)) }
async fn h_role(Json(r): Json<Role>) -> Json<R<bool>> { wrap(protected_set_role(r.actor, r.object, r.role_id, r.mask).map(|_| true)) }
async fn h_inherit(Json(r): Json<Inherit>) -> Json<R<bool>> { wrap(protected_set_inherit(r.actor, r.object, r.child, r.parent).map(|_| true)) }
async fn h_label(Json(r): Json<Label>) -> Json<R<bool>> { wrap(set_label(r.id, &r.name).map(|_| true)) }
async fn h_entity(Json(r): Json<Entity>) -> Json<R<u64>> { wrap(create_entity(&r.name)) }
async fn h_entity_update(Json(r): Json<EntityUpdate>) -> Json<R<bool>> { wrap(rename_entity(r.id, &r.name).map(|_| true)) }
async fn h_entity_delete(Json(r): Json<EntityDelete>) -> Json<R<bool>> { wrap(delete_entity(r.id)) }
async fn h_reset(Json(r): Json<Reset>) -> Json<R<bool>> {
    match get_root() {
        Ok(Some(root)) if root == r.actor => wrap(clear_all().map(|_| true)),
        _ => err("Only root can reset")
    }
}

async fn h_check(Query(q): Query<Check>) -> Json<R<CheckRes>> {
    wrap(get_mask(q.subject, q.object).map(|mask| {
        let req = q.required.unwrap_or(0);
        CheckRes { allowed: (mask & req) == req, mask }
    }))
}

async fn h_list(Query(q): Query<List>) -> Json<R<Vec<Entry>>> {
    let to_entries = |v: Vec<(u64, u64)>| v.into_iter().map(|(id, mask)| Entry { id, mask, label: get_label(id).ok().flatten() }).collect();
    if let Some(obj) = q.object {
        let actor = q.actor.unwrap_or(0);
        wrap(protected_list_for_object(actor, obj).map(to_entries))
    } else if let Some(subj) = q.subject {
        wrap(list_for_subject(subj).map(to_entries))
    } else {
        err("Provide subject= or object=")
    }
}

async fn h_labels() -> Json<R<Vec<Entry>>> {
    wrap(list_labels().map(|v| v.into_iter().map(|(id, _)| Entry { id, mask: 0, label: get_label(id).ok().flatten() }).collect()))
}

#[tokio::main]
async fn main() {
    let db = std::env::var("CAPBIT_DB").unwrap_or_else(|_| "./data/capbit.mdb".into());
    init(&db).expect("Failed to init db");

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../../demo/index.html")) }))
        .route("/status", get(h_status))
        .route("/bootstrap", post(h_bootstrap))
        .route("/grant", post(h_grant))
        .route("/revoke", post(h_revoke))
        .route("/role", post(h_role))
        .route("/inherit", post(h_inherit))
        .route("/check", get(h_check))
        .route("/list", get(h_list))
        .route("/label", post(h_label))
        .route("/entity", post(h_entity))
        .route("/entity/rename", post(h_entity_update))
        .route("/entity/delete", post(h_entity_delete))
        .route("/labels", get(h_labels))
        .route("/reset", post(h_reset))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap();
    println!("Capbit at http://localhost:{}", port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.ok();
}
