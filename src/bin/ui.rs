use axum::{extract::Json, response::Html, routing::{get, post}, Router};
use capbit::*;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static CB: OnceLock<Capbit> = OnceLock::new();
fn cb() -> &'static Capbit { CB.get().unwrap() }

fn pol(p: u64) -> Policy {
    match p { 0 => Policy::Not, 1 => Policy::Possible, _ => Policy::Necessary }
}

#[derive(Deserialize)] struct ActorObj { actor: u64, obj: u64 }
#[derive(Deserialize)] struct GrantReq { actor: u64, sub: u64, obj: u64, role: u64, policy: u64 }
#[derive(Deserialize)] struct RevokeReq { actor: u64, sub: u64, obj: u64, role: u64 }
#[derive(Deserialize)] struct RoleReq { actor: u64, obj: u64, role: u64, mask: u64 }
#[derive(Deserialize)] struct RoleDel { actor: u64, obj: u64, role: u64 }
#[derive(Deserialize)] struct CheckReq { sub: u64, obj: u64, req: u64 }
#[derive(Deserialize)] struct MaskReq { sub: u64, obj: u64 }
#[derive(Deserialize)] struct MemberReq { actor: u64, member: u64, group: u64, policy: u64 }
#[derive(Deserialize)] struct MemberDel { actor: u64, member: u64, group: u64 }
#[derive(Deserialize)] struct DelegateReq { actor: u64, sub: u64, obj: u64, role: u64, parent: u64, policy: u64 }
#[derive(Deserialize)] struct ParentReq { actor: u64, obj: u64, parent: u64 }
#[derive(Deserialize)] struct ActorSub { actor: u64, sub: u64 }
#[derive(Deserialize)] struct SubObj { actor: u64, sub: u64, obj: u64 }
#[derive(Deserialize)] struct PopReq { actor: u64, a: u64, b: u64 }
#[derive(Deserialize)] struct PageReq { actor: u64, obj: u64, after: Option<u64>, limit: Option<usize> }
#[derive(Deserialize)] struct AuditReq { actor: u64, after: Option<u64>, limit: Option<usize> }
#[derive(Serialize)] struct Resp { ok: bool, msg: String }

fn resp(r: Result<String>) -> Json<Resp> {
    Json(match r { Ok(m) => Resp { ok: true, msg: m }, Err(e) => Resp { ok: false, msg: e.to_string() } })
}

fn fmt<T: std::fmt::Debug>(v: &[T]) -> String {
    if v.is_empty() { "(empty)".into() } else { format!("{v:?}") }
}

async fn do_bootstrap() -> Json<Resp> { resp(cb().bootstrap().map(|(s, r)| format!("system={s}, root={r}"))) }
async fn do_clear(Json(r): Json<ActorSub>) -> Json<Resp> { resp(cb().clear(r.actor).map(|_| "Cleared".into())) }
async fn do_create_object(Json(r): Json<ActorObj>) -> Json<Resp> { resp(cb().create_object(r.actor, r.obj).map(|_| "Created".into())) }
async fn do_delete_object(Json(r): Json<ActorObj>) -> Json<Resp> { resp(cb().delete_object(r.actor, r.obj).map(|_| "Deleted".into())) }
async fn do_grant(Json(r): Json<GrantReq>) -> Json<Resp> { resp(cb().grant(r.actor, r.sub, r.obj, r.role, pol(r.policy)).map(|_| "Granted".into())) }
async fn do_revoke(Json(r): Json<RevokeReq>) -> Json<Resp> { resp(cb().revoke(r.actor, r.sub, r.obj, r.role).map(|_| "Revoked".into())) }
async fn do_define_role(Json(r): Json<RoleReq>) -> Json<Resp> { resp(cb().define_role(r.actor, r.obj, r.role, r.mask).map(|_| "Defined".into())) }
async fn do_update_role(Json(r): Json<RoleReq>) -> Json<Resp> { resp(cb().update_role(r.actor, r.obj, r.role, r.mask).map(|_| "Updated".into())) }
async fn do_delete_role(Json(r): Json<RoleDel>) -> Json<Resp> { resp(cb().delete_role(r.actor, r.obj, r.role).map(|_| "Deleted".into())) }
async fn do_list_roles(Json(r): Json<ActorObj>) -> Json<Resp> { resp(cb().list_roles(r.actor, r.obj).map(|v| fmt(&v))) }
async fn do_check(Json(r): Json<CheckReq>) -> Json<Resp> { resp(cb().check(r.sub, r.obj, r.req).map(|b| if b { "Allowed" } else { "Denied" }.into())) }
async fn do_get_masks(Json(r): Json<MaskReq>) -> Json<Resp> {
    resp(cb().get_masks(r.sub, r.obj).map(|m| format!("necessary=0x{:X} possible=0x{:X} denied=0x{:X} allowed=0x{:X}", m.necessary, m.possible, m.denied, m.allowed())))
}
async fn do_add_member(Json(r): Json<MemberReq>) -> Json<Resp> { resp(cb().add_member(r.actor, r.member, r.group, pol(r.policy)).map(|_| "Added".into())) }
async fn do_remove_member(Json(r): Json<MemberDel>) -> Json<Resp> { resp(cb().remove_member(r.actor, r.member, r.group).map(|_| "Removed".into())) }
async fn do_groups_of(Json(r): Json<ActorSub>) -> Json<Resp> { resp(cb().groups_of(r.actor, r.sub).map(|v| fmt(&v))) }
async fn do_members_of(Json(r): Json<PageReq>) -> Json<Resp> { resp(cb().members_of(r.actor, r.obj, r.after, r.limit.unwrap_or(100)).map(|v| fmt(&v))) }
async fn do_pop_intersect(Json(r): Json<PopReq>) -> Json<Resp> { resp(cb().population_intersect(r.actor, r.a, r.b).map(|v| fmt(&v))) }
async fn do_pop_subtract(Json(r): Json<PopReq>) -> Json<Resp> { resp(cb().population_subtract(r.actor, r.a, r.b).map(|v| fmt(&v))) }
async fn do_delegate(Json(r): Json<DelegateReq>) -> Json<Resp> { resp(cb().delegate(r.actor, r.sub, r.obj, r.role, r.parent, pol(r.policy)).map(|_| "Delegated".into())) }
async fn do_undelegate(Json(r): Json<RevokeReq>) -> Json<Resp> { resp(cb().undelegate(r.actor, r.sub, r.obj, r.role).map(|_| "Removed".into())) }
async fn do_list_delegations(Json(r): Json<ActorObj>) -> Json<Resp> { resp(cb().list_delegations_on(r.actor, r.obj).map(|v| fmt(&v))) }
async fn do_set_parent(Json(r): Json<ParentReq>) -> Json<Resp> { resp(cb().set_parent(r.actor, r.obj, r.parent).map(|_| "Set".into())) }
async fn do_clear_parent(Json(r): Json<ActorObj>) -> Json<Resp> { resp(cb().clear_parent(r.actor, r.obj).map(|_| "Cleared".into())) }
async fn do_list_grants(Json(r): Json<ActorSub>) -> Json<Resp> { resp(cb().list_grants(r.actor, r.sub).map(|v| fmt(&v))) }
async fn do_list_subjects(Json(r): Json<PageReq>) -> Json<Resp> { resp(cb().list_subjects(r.actor, r.obj, r.after, r.limit.unwrap_or(100)).map(|v| fmt(&v))) }
async fn do_list_roles_for(Json(r): Json<SubObj>) -> Json<Resp> { resp(cb().list_roles_for(r.actor, r.sub, r.obj).map(|v| fmt(&v))) }
async fn do_audit(Json(r): Json<AuditReq>) -> Json<Resp> {
    resp(cb().audit_read(r.actor, r.after, r.limit.unwrap_or(100)).map(|v| {
        v.iter().map(|e| format!("#{} t={} actor={} op={} args={:?}", e.seq, e.ts_ms, e.actor, e.op, e.args))
            .collect::<Vec<_>>().join(" | ")
    }))
}

async fn index() -> Html<&'static str> { Html(include_str!("ui.html")) }

#[tokio::main]
async fn main() {
    CB.set(Capbit::open("capbit_data").expect("open failed")).ok();
    let app = Router::new()
        .route("/", get(index))
        .route("/api/bootstrap", post(do_bootstrap))
        .route("/api/clear", post(do_clear))
        .route("/api/create_object", post(do_create_object))
        .route("/api/delete_object", post(do_delete_object))
        .route("/api/grant", post(do_grant))
        .route("/api/revoke", post(do_revoke))
        .route("/api/define_role", post(do_define_role))
        .route("/api/update_role", post(do_update_role))
        .route("/api/delete_role", post(do_delete_role))
        .route("/api/list_roles", post(do_list_roles))
        .route("/api/check", post(do_check))
        .route("/api/get_masks", post(do_get_masks))
        .route("/api/add_member", post(do_add_member))
        .route("/api/remove_member", post(do_remove_member))
        .route("/api/groups_of", post(do_groups_of))
        .route("/api/members_of", post(do_members_of))
        .route("/api/pop_intersect", post(do_pop_intersect))
        .route("/api/pop_subtract", post(do_pop_subtract))
        .route("/api/delegate", post(do_delegate))
        .route("/api/undelegate", post(do_undelegate))
        .route("/api/list_delegations", post(do_list_delegations))
        .route("/api/set_parent", post(do_set_parent))
        .route("/api/clear_parent", post(do_clear_parent))
        .route("/api/list_grants", post(do_list_grants))
        .route("/api/list_subjects", post(do_list_subjects))
        .route("/api/list_roles_for", post(do_list_roles_for))
        .route("/api/audit", post(do_audit));
    println!("UI running at http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
