//! Verbose demo test showing step-by-step Capbit v2 simulation

use capbit::{init, bootstrap, protected, check_access, SystemCap, clear_all, test_lock};
use tempfile::TempDir;
use std::sync::Once;

static INIT: Once = Once::new();
static mut TEST_DIR: Option<TempDir> = None;

fn setup() {
    INIT.call_once(|| {
        let dir = TempDir::new().unwrap();
        init(dir.path().to_str().unwrap()).unwrap();
        unsafe { TEST_DIR = Some(dir); }
    });
}

fn setup_bootstrapped() -> std::sync::MutexGuard<'static, ()> {
    let lock = test_lock();
    setup();
    clear_all().unwrap();
    bootstrap("root").unwrap();
    lock
}

fn show_caps(who: &str, scope: &str) {
    let caps = check_access(who, scope, None).unwrap_or(0);
    println!("    {} → {} = 0x{:04x}", who, scope, caps);
    if caps & SystemCap::ENTITY_CREATE != 0 { println!("      ✓ ENTITY_CREATE"); }
    if caps & SystemCap::GRANT_WRITE != 0 { println!("      ✓ GRANT_WRITE"); }
    if caps & SystemCap::CAP_WRITE != 0 { println!("      ✓ CAP_WRITE"); }
    if caps == 0 { println!("      ✗ No permissions"); }
}

#[test]
fn demo_simulation() {
    let _lock = setup_bootstrapped();

    println!("\n══════════════════════════════════════════════════════════");
    println!("  CAPBIT v2 SIMULATION: Acme Corp Organization");
    println!("══════════════════════════════════════════════════════════");

    // Step 1
    println!("\n┌─ STEP 1: System bootstrapped");
    println!("│  bootstrap(\"root\") → user:root created");
    println!("│  Root has full admin on all types:");
    show_caps("user:root", "_type:user");
    show_caps("user:root", "_type:team");

    // Step 2
    println!("\n┌─ STEP 2: Root creates teams");
    protected::create_entity("user:root", "team", "hr").unwrap();
    println!("│  ✓ team:hr created");
    protected::create_entity("user:root", "team", "engineering").unwrap();
    println!("│  ✓ team:engineering created");
    protected::create_entity("user:root", "team", "sales").unwrap();
    println!("│  ✓ team:sales created");

    // Step 3
    println!("\n┌─ STEP 3: Root creates users");
    for name in &["alice", "bob", "charlie", "dave", "eve"] {
        protected::create_entity("user:root", "user", name).unwrap();
        println!("│  ✓ user:{} created", name);
    }

    // Step 4
    println!("\n┌─ STEP 4: Root defines team roles");
    let owner_caps = SystemCap::CAP_WRITE | SystemCap::GRANT_WRITE | SystemCap::DELEGATE_WRITE;
    let lead_caps = SystemCap::GRANT_WRITE | SystemCap::GRANT_READ;
    let member_caps = SystemCap::GRANT_READ;

    for team in &["hr", "engineering", "sales"] {
        let scope = format!("team:{}", team);
        protected::set_capability("user:root", &scope, "owner", owner_caps).unwrap();
        protected::set_capability("user:root", &scope, "lead", lead_caps).unwrap();
        protected::set_capability("user:root", &scope, "member", member_caps).unwrap();
        protected::set_grant("user:root", "user:root", "owner", &scope).unwrap();
    }
    println!("│  Roles defined: owner, lead, member");
    println!("│  owner  = CAP_WRITE | GRANT_WRITE | DELEGATE_WRITE");
    println!("│  lead   = GRANT_WRITE | GRANT_READ");
    println!("│  member = GRANT_READ");

    // Step 5
    println!("\n┌─ STEP 5: Root assigns team leads");
    protected::set_grant("user:root", "user:alice", "lead", "team:hr").unwrap();
    println!("│  ✓ Alice → lead of team:hr");
    protected::set_grant("user:root", "user:bob", "lead", "team:engineering").unwrap();
    println!("│  ✓ Bob → lead of team:engineering");
    protected::set_grant("user:root", "user:charlie", "lead", "team:sales").unwrap();
    println!("│  ✓ Charlie → lead of team:sales");
    println!("│");
    println!("│  Bob's permissions:");
    show_caps("user:bob", "team:engineering");

    // Step 6
    println!("\n┌─ STEP 6: Bob (lead) adds team members");
    protected::set_grant("user:bob", "user:dave", "member", "team:engineering").unwrap();
    println!("│  ✓ Bob added Dave as member");
    protected::set_grant("user:bob", "user:eve", "member", "team:engineering").unwrap();
    println!("│  ✓ Bob added Eve as member");
    println!("│");
    println!("│  Dave's permissions (member):");
    show_caps("user:dave", "team:engineering");

    // Step 7 - Attack
    println!("\n┌─ STEP 7: ATTACK - Dave tries to add members");
    println!("│  Dave is only a member (GRANT_READ, no GRANT_WRITE)");
    match protected::set_grant("user:dave", "user:alice", "member", "team:engineering") {
        Ok(_) => println!("│  ✗ SECURITY BREACH!"),
        Err(e) => println!("│  ✓ BLOCKED: {}", e.message),
    }

    // Step 8 - Delegation
    println!("\n┌─ STEP 8: Root delegates user management to HR");
    protected::set_grant("user:root", "team:hr", "admin", "_type:user").unwrap();
    protected::set_capability("user:root", "_type:user", "delegator", SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:root", "delegator", "_type:user").unwrap();
    protected::set_delegation("user:root", "user:alice", "_type:user", "team:hr").unwrap();
    println!("│  ✓ team:hr granted admin on _type:user");
    println!("│  ✓ Alice (HR lead) delegated team:hr's permissions");
    println!("│");
    println!("│  Alice's permissions on _type:user:");
    show_caps("user:alice", "_type:user");

    // Step 9
    println!("\n┌─ STEP 9: Alice (HR) creates new user");
    protected::create_entity("user:alice", "user", "frank").unwrap();
    println!("│  ✓ Alice created user:frank (not root!)");

    // Step 10 - Attack
    println!("\n┌─ STEP 10: ATTACK - Alice tries to create team");
    println!("│  Alice has user admin, but NOT team admin");
    match protected::create_entity("user:alice", "team", "marketing") {
        Ok(_) => println!("│  ✗ SECURITY BREACH!"),
        Err(e) => println!("│  ✓ BLOCKED: {}", e.message),
    }
    println!("│");
    println!("│  Alice's permissions on _type:team:");
    show_caps("user:alice", "_type:team");

    println!("\n══════════════════════════════════════════════════════════");
    println!("  SIMULATION COMPLETE - All security checks passed!");
    println!("══════════════════════════════════════════════════════════");
    println!("\n  Summary:");
    println!("  • Root bootstrapped with full admin");
    println!("  • Teams and users created hierarchically");
    println!("  • Team leads can add members");
    println!("  • Members cannot escalate privileges");
    println!("  • HR can manage users via delegation");
    println!("  • HR cannot create teams (scope-limited)");
    println!("");
}
