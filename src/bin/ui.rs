use axum::{extract::Json, response::Html, routing::{get, post}, Router};
use capbit::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)] struct GrantReq { actor: u64, sub: u64, obj: u64, role: u64 }
#[derive(Deserialize)] struct RevokeReq { actor: u64, sub: u64, obj: u64 }
#[derive(Deserialize)] struct CreateReq { actor: u64, obj: u64, role: u64, mask: u64 }
#[derive(Deserialize)] struct UpdateReq { actor: u64, obj: u64, role: u64, mask: u64 }
#[derive(Deserialize)] struct DeleteReq { actor: u64, obj: u64, role: u64 }
#[derive(Deserialize)] struct CheckReq { sub: u64, obj: u64, req: u64 }
#[derive(Deserialize)] struct GetMaskReq { sub: u64, obj: u64 }
#[derive(Serialize)] struct Resp { ok: bool, msg: String }

fn resp(r: Result<String>) -> Json<Resp> {
    Json(match r { Ok(m) => Resp { ok: true, msg: m }, Err(e) => Resp { ok: false, msg: e.0 } })
}

async fn do_bootstrap() -> Json<Resp> { resp(bootstrap().map(|(s,r)| format!("system={s}, root={r}"))) }
async fn do_clear() -> Json<Resp> { resp(clear().map(|_| "Cleared".into())) }
async fn do_grant(Json(r): Json<GrantReq>) -> Json<Resp> { resp(grant(r.actor, r.sub, r.obj, r.role).map(|_| "Granted".into())) }
async fn do_revoke(Json(r): Json<RevokeReq>) -> Json<Resp> { resp(revoke(r.actor, r.sub, r.obj).map(|_| "Revoked".into())) }
async fn do_create(Json(r): Json<CreateReq>) -> Json<Resp> { resp(create(r.actor, r.obj, r.role, r.mask).map(|_| "Created".into())) }
async fn do_update(Json(r): Json<UpdateReq>) -> Json<Resp> { resp(update(r.actor, r.obj, r.role, r.mask).map(|_| "Updated".into())) }
async fn do_delete(Json(r): Json<DeleteReq>) -> Json<Resp> { resp(delete(r.actor, r.obj, r.role).map(|_| "Deleted".into())) }
async fn do_check(Json(r): Json<CheckReq>) -> Json<Resp> { resp(check(r.sub, r.obj, r.req).map(|b| if b { "Allowed" } else { "Denied" }.into())) }
async fn do_get_mask(Json(r): Json<GetMaskReq>) -> Json<Resp> { resp(get_mask(r.sub, r.obj).map(|m| format!("0x{m:X} ({m})"))) }

async fn index() -> Html<&'static str> { Html(include_str!("ui.html")) }

#[tokio::main]
async fn main() {
    init("capbit_data").expect("init failed");
    let app = Router::new()
        .route("/", get(index))
        .route("/api/bootstrap", post(do_bootstrap))
        .route("/api/clear", post(do_clear))
        .route("/api/grant", post(do_grant))
        .route("/api/revoke", post(do_revoke))
        .route("/api/create", post(do_create))
        .route("/api/update", post(do_update))
        .route("/api/delete", post(do_delete))
        .route("/api/check", post(do_check))
        .route("/api/get_mask", post(do_get_mask));
    println!("UI running at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
