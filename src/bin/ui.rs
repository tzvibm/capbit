use axum::{extract::Json, response::Html, routing::{get, post}, Router};
use capbit::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)] struct GrantReq { actor: u64, sub: u64, obj: u64, role: u64 }
#[derive(Deserialize)] struct RevokeReq { actor: u64, sub: u64, obj: u64, role: u64 }
#[derive(Deserialize)] struct CreateReq { actor: u64, obj: u64, role: u64, mask: u64 }
#[derive(Deserialize)] struct UpdateReq { actor: u64, obj: u64, role: u64, mask: u64 }
#[derive(Deserialize)] struct DeleteReq { actor: u64, obj: u64, role: u64 }
#[derive(Deserialize)] struct CheckReq { sub: u64, obj: u64, req: u64 }
#[derive(Deserialize)] struct GetMaskReq { sub: u64, obj: u64 }
#[derive(Deserialize)] struct InheritReq { actor: u64, sub: u64, obj: u64, role: u64, parent: u64 }
#[derive(Deserialize)] struct RemoveInheritReq { actor: u64, sub: u64, obj: u64, role: u64 }
#[derive(Deserialize)] struct ListRolesReq { actor: u64, obj: u64 }
#[derive(Deserialize)] struct ListRolesForReq { actor: u64, sub: u64, obj: u64 }
#[derive(Deserialize)] struct ListGrantsReq { actor: u64, sub: u64 }
#[derive(Deserialize)] struct ListSubjectsReq { actor: u64, obj: u64 }
#[derive(Deserialize)] struct ListInheritsReq { actor: u64, sub: u64, obj: u64 }
#[derive(Deserialize)] struct ListInheritsOnObjReq { actor: u64, obj: u64 }
#[derive(Deserialize)] struct ListInheritsOnObjRoleReq { actor: u64, obj: u64, role: u64 }
#[derive(Deserialize)] struct ListInheritsFromParentReq { actor: u64, parent: u64 }
#[derive(Deserialize)] struct ListInheritsFromParentOnObjReq { actor: u64, parent: u64, obj: u64 }
#[derive(Serialize)] struct Resp { ok: bool, msg: String }

fn resp(r: Result<String>) -> Json<Resp> {
    Json(match r { Ok(m) => Resp { ok: true, msg: m }, Err(e) => Resp { ok: false, msg: e.0 } })
}

fn fmt2(v: &[(u64, u64)]) -> String { v.iter().map(|(a,b)| format!("({a},{b})")).collect::<Vec<_>>().join(", ") }
fn fmt3(v: &[(u64, u64, u64)]) -> String { v.iter().map(|(a,b,c)| format!("({a},{b},{c})")).collect::<Vec<_>>().join(", ") }

async fn do_bootstrap() -> Json<Resp> { resp(bootstrap().map(|(s,r)| format!("system={s}, root={r}"))) }
async fn do_clear() -> Json<Resp> { resp(clear().map(|_| "Cleared".into())) }
async fn do_grant(Json(r): Json<GrantReq>) -> Json<Resp> { resp(grant(r.actor, r.sub, r.obj, r.role).map(|_| "Granted".into())) }
async fn do_revoke(Json(r): Json<RevokeReq>) -> Json<Resp> { resp(revoke(r.actor, r.sub, r.obj, r.role).map(|_| "Revoked".into())) }
async fn do_create(Json(r): Json<CreateReq>) -> Json<Resp> { resp(create(r.actor, r.obj, r.role, r.mask).map(|_| "Created".into())) }
async fn do_update(Json(r): Json<UpdateReq>) -> Json<Resp> { resp(update(r.actor, r.obj, r.role, r.mask).map(|_| "Updated".into())) }
async fn do_delete(Json(r): Json<DeleteReq>) -> Json<Resp> { resp(delete(r.actor, r.obj, r.role).map(|_| "Deleted".into())) }
async fn do_check(Json(r): Json<CheckReq>) -> Json<Resp> { resp(check(r.sub, r.obj, r.req).map(|b| if b { "Allowed" } else { "Denied" }.into())) }
async fn do_get_mask(Json(r): Json<GetMaskReq>) -> Json<Resp> { resp(get_mask(r.sub, r.obj).map(|m| format!("0x{m:X} ({m})"))) }
async fn do_inherit(Json(r): Json<InheritReq>) -> Json<Resp> { resp(inherit(r.actor, r.sub, r.obj, r.role, r.parent).map(|_| "Inherited".into())) }
async fn do_remove_inherit(Json(r): Json<RemoveInheritReq>) -> Json<Resp> { resp(remove_inherit(r.actor, r.sub, r.obj, r.role).map(|_| "Removed".into())) }
async fn do_list_roles(Json(r): Json<ListRolesReq>) -> Json<Resp> { resp(list_roles(r.actor, r.obj).map(|v| fmt2(&v))) }
async fn do_list_roles_for(Json(r): Json<ListRolesForReq>) -> Json<Resp> { resp(list_roles_for(r.actor, r.sub, r.obj).map(|v| format!("{v:?}"))) }
async fn do_list_grants(Json(r): Json<ListGrantsReq>) -> Json<Resp> { resp(list_grants(r.actor, r.sub).map(|v| fmt2(&v))) }
async fn do_list_subjects(Json(r): Json<ListSubjectsReq>) -> Json<Resp> { resp(list_subjects(r.actor, r.obj).map(|v| fmt2(&v))) }
async fn do_list_inherits(Json(r): Json<ListInheritsReq>) -> Json<Resp> { resp(list_inherits(r.actor, r.sub, r.obj).map(|v| fmt2(&v))) }
async fn do_list_inherits_on_obj(Json(r): Json<ListInheritsOnObjReq>) -> Json<Resp> { resp(list_inherits_on_obj(r.actor, r.obj).map(|v| fmt3(&v))) }
async fn do_list_inherits_on_obj_role(Json(r): Json<ListInheritsOnObjRoleReq>) -> Json<Resp> { resp(list_inherits_on_obj_role(r.actor, r.obj, r.role).map(|v| fmt2(&v))) }
async fn do_list_inherits_from_parent(Json(r): Json<ListInheritsFromParentReq>) -> Json<Resp> { resp(list_inherits_from_parent(r.actor, r.parent).map(|v| fmt3(&v))) }
async fn do_list_inherits_from_parent_on_obj(Json(r): Json<ListInheritsFromParentOnObjReq>) -> Json<Resp> { resp(list_inherits_from_parent_on_obj(r.actor, r.parent, r.obj).map(|v| fmt2(&v))) }

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
        .route("/api/get_mask", post(do_get_mask))
        .route("/api/inherit", post(do_inherit))
        .route("/api/remove_inherit", post(do_remove_inherit))
        .route("/api/list_roles", post(do_list_roles))
        .route("/api/list_roles_for", post(do_list_roles_for))
        .route("/api/list_grants", post(do_list_grants))
        .route("/api/list_subjects", post(do_list_subjects))
        .route("/api/list_inherits", post(do_list_inherits))
        .route("/api/list_inherits_on_obj", post(do_list_inherits_on_obj))
        .route("/api/list_inherits_on_obj_role", post(do_list_inherits_on_obj_role))
        .route("/api/list_inherits_from_parent", post(do_list_inherits_from_parent))
        .route("/api/list_inherits_from_parent_on_obj", post(do_list_inherits_from_parent_on_obj));
    println!("UI running at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
