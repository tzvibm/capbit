//! Full organization simulation test from SIMULATION.md

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

/// Full organization simulation from SIMULATION.md
/// Tests the complete Acme Corp scenario
#[test]
fn simulation_acme_corp() {
    let _lock = setup_bootstrapped();

    // =========================================================================
    // Phase 2: Root Sets Up Organization
    // =========================================================================

    // Create teams
    protected::create_entity("user:root", "team", "hr").unwrap();
    protected::create_entity("user:root", "team", "engineering").unwrap();
    protected::create_entity("user:root", "team", "sales").unwrap();

    // Create users
    protected::create_entity("user:root", "user", "alice").unwrap();   // HR lead
    protected::create_entity("user:root", "user", "bob").unwrap();     // Engineering lead
    protected::create_entity("user:root", "user", "charlie").unwrap(); // Sales lead
    protected::create_entity("user:root", "user", "dave").unwrap();    // Engineer
    protected::create_entity("user:root", "user", "eve").unwrap();     // Engineer

    // Define team ownership (who can manage team grants/caps)
    protected::set_capability("user:root", "team:hr", "owner",
        SystemCap::CAP_WRITE | SystemCap::GRANT_WRITE | SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_capability("user:root", "team:engineering", "owner",
        SystemCap::CAP_WRITE | SystemCap::GRANT_WRITE | SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_capability("user:root", "team:sales", "owner",
        SystemCap::CAP_WRITE | SystemCap::GRANT_WRITE | SystemCap::DELEGATE_WRITE).unwrap();

    // Root is owner of all teams
    protected::set_grant("user:root", "user:root", "owner", "team:hr").unwrap();
    protected::set_grant("user:root", "user:root", "owner", "team:engineering").unwrap();
    protected::set_grant("user:root", "user:root", "owner", "team:sales").unwrap();

    // Define what "lead" and "member" mean on teams
    protected::set_capability("user:root", "team:hr", "lead", SystemCap::GRANT_WRITE | SystemCap::GRANT_READ).unwrap();
    protected::set_capability("user:root", "team:hr", "member", SystemCap::GRANT_READ).unwrap();
    protected::set_capability("user:root", "team:engineering", "lead", SystemCap::GRANT_WRITE | SystemCap::GRANT_READ).unwrap();
    protected::set_capability("user:root", "team:engineering", "member", SystemCap::GRANT_READ).unwrap();
    protected::set_capability("user:root", "team:sales", "lead", SystemCap::GRANT_WRITE | SystemCap::GRANT_READ).unwrap();
    protected::set_capability("user:root", "team:sales", "member", SystemCap::GRANT_READ).unwrap();

    // Assign team leads
    protected::set_grant("user:root", "user:alice", "lead", "team:hr").unwrap();
    protected::set_grant("user:root", "user:bob", "lead", "team:engineering").unwrap();
    protected::set_grant("user:root", "user:charlie", "lead", "team:sales").unwrap();

    // =========================================================================
    // Key Step: Delegate User Management to HR
    // =========================================================================

    // HR team gets admin on _type:user
    protected::set_grant("user:root", "team:hr", "admin", "_type:user").unwrap();

    // Alice needs delegation to inherit HR's permissions on _type:user
    protected::set_capability("user:root", "_type:user", "delegator", SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:root", "delegator", "_type:user").unwrap();
    protected::set_delegation("user:root", "user:alice", "_type:user", "team:hr").unwrap();

    // =========================================================================
    // Phase 3: Verify Delegated Operations
    // =========================================================================

    // Alice (HR) can now create users
    let alice_caps = check_access("user:alice", "_type:user", None).unwrap();
    assert!((alice_caps & SystemCap::ENTITY_CREATE) != 0,
        "Alice should have ENTITY_CREATE on _type:user");

    // Alice creates frank
    protected::create_entity("user:alice", "user", "frank").unwrap();

    // Alice CANNOT create teams (no delegation for _type:team)
    let alice_team_caps = check_access("user:alice", "_type:team", None).unwrap();
    assert!((alice_team_caps & SystemCap::ENTITY_CREATE) == 0,
        "Alice should NOT have ENTITY_CREATE on _type:team");

    // Bob can add members to his team (he's lead with GRANT_WRITE)
    let bob_caps = check_access("user:bob", "team:engineering", None).unwrap();
    assert!((bob_caps & SystemCap::GRANT_WRITE) != 0,
        "Bob should have GRANT_WRITE on team:engineering");

    // Bob adds dave and eve as members
    protected::set_grant("user:bob", "user:dave", "member", "team:engineering").unwrap();
    protected::set_grant("user:bob", "user:eve", "member", "team:engineering").unwrap();

    // Dave (member) cannot add other members (only has GRANT_READ, not GRANT_WRITE)
    let dave_caps = check_access("user:dave", "team:engineering", None).unwrap();
    assert!((dave_caps & SystemCap::GRANT_WRITE) == 0,
        "Dave should NOT have GRANT_WRITE on team:engineering");

    // =========================================================================
    // Phase 4: Engineering Creates Apps
    // =========================================================================

    // Root delegates app management to engineering
    protected::set_grant("user:root", "team:engineering", "admin", "_type:app").unwrap();
    protected::set_capability("user:root", "_type:app", "delegator", SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:root", "delegator", "_type:app").unwrap();
    protected::set_delegation("user:root", "user:bob", "_type:app", "team:engineering").unwrap();

    // Bob can now create apps
    let bob_app_caps = check_access("user:bob", "_type:app", None).unwrap();
    assert!((bob_app_caps & SystemCap::ENTITY_CREATE) != 0,
        "Bob should have ENTITY_CREATE on _type:app");

    protected::create_entity("user:bob", "app", "backend-api").unwrap();
    protected::create_entity("user:bob", "app", "frontend-web").unwrap();

    // =========================================================================
    // Verification Summary
    // =========================================================================

    // Frank (new hire) has no permissions yet
    let frank_caps = check_access("user:frank", "team:hr", None).unwrap();
    assert_eq!(frank_caps, 0, "Frank should have no capabilities on team:hr");

    // Eve cannot access backend-api (she's on frontend)
    let eve_caps = check_access("user:eve", "app:backend-api", None).unwrap();
    assert_eq!(eve_caps, 0, "Eve should have no capabilities on app:backend-api");
}

/// Test that the simulation can be extended
#[test]
fn simulation_extend_with_app_access() {
    let _lock = setup_bootstrapped();

    // Quick setup
    protected::create_entity("user:root", "team", "engineering").unwrap();
    protected::create_entity("user:root", "user", "bob").unwrap();
    protected::create_entity("user:root", "user", "dave").unwrap();

    // Bob is lead
    protected::set_capability("user:root", "team:engineering", "lead",
        SystemCap::GRANT_WRITE | SystemCap::CAP_WRITE).unwrap();
    protected::set_grant("user:root", "user:bob", "lead", "team:engineering").unwrap();

    // Delegate app creation to bob
    protected::set_grant("user:root", "team:engineering", "admin", "_type:app").unwrap();
    protected::set_capability("user:root", "_type:app", "delegator", SystemCap::DELEGATE_WRITE).unwrap();
    protected::set_grant("user:root", "user:root", "delegator", "_type:app").unwrap();
    protected::set_delegation("user:root", "user:bob", "_type:app", "team:engineering").unwrap();

    // Bob creates app
    protected::create_entity("user:bob", "app", "myapp").unwrap();

    // Bob defines app permissions and becomes owner
    protected::set_capability("user:root", "app:myapp", "owner",
        SystemCap::CAP_WRITE | SystemCap::GRANT_WRITE).unwrap();
    protected::set_grant("user:root", "user:bob", "owner", "app:myapp").unwrap();

    // Bob can now define developer role and grant it
    protected::set_capability("user:bob", "app:myapp", "developer", 0x0F).unwrap();
    protected::set_grant("user:bob", "user:dave", "developer", "app:myapp").unwrap();

    // Dave has developer access
    let dave_caps = check_access("user:dave", "app:myapp", None).unwrap();
    assert_eq!(dave_caps, 0x0F, "Dave should have developer capabilities");
}
