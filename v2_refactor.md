# Capbit v2 Refactor Plan

## Philosophy: Test-First, Stay Lean

**Current state:** ~550 LOC core, ~200 LOC tests, unprotected API
**Target:** Protected mutations, typed entities, same LOC order of magnitude

---

## Phase 1: Comprehensive Test Suite (Before Any Refactor)

### 1.1 Current Behavior Tests (Lock In)

These tests ensure we don't break existing functionality:

```rust
// tests/v1_compat.rs (~100 LOC)
mod current_api {
    // Existing tests already cover:
    // - relationships CRUD
    // - capabilities CRUD
    // - inheritance/delegation
    // - access checks
    // - cycle detection
    // - deep inheritance
    // - batch operations
    // - query operations (list_accessible, list_subjects)
}
```
**Status:** Already exists in `integration.rs` - keep as-is.

---

### 1.2 Attack Vector Tests (New)

```rust
// tests/attack_vectors.rs (~150 LOC)

/// ATTACK: Entity spoofing - attacker creates "john" before real john
#[test]
fn attack_entity_spoofing() {
    // Setup: alice has admin on _type:user
    // Attack: bob tries to create user:alice (impersonation)
    // Expected: DENIED - bob lacks ENTITY_CREATE on _type:user
}

/// ATTACK: Privilege escalation via self-grant
#[test]
fn attack_self_grant_escalation() {
    // Setup: alice has GRANT_WRITE on team:sales only
    // Attack: alice grants herself admin on _system
    // Expected: DENIED - alice lacks GRANT_WRITE on _system
}

/// ATTACK: Scope confusion - grant on wrong scope
#[test]
fn attack_scope_confusion() {
    // Setup: alice is admin on team:sales
    // Attack: alice grants bob admin on team:engineering
    // Expected: DENIED - alice lacks GRANT_WRITE on team:engineering
}

/// ATTACK: Type bypass - create entity without going through type
#[test]
fn attack_type_bypass() {
    // Setup: only root can create users
    // Attack: bob calls internal create_entity directly
    // Expected: Internal functions not exposed; protected API enforces
}

/// ATTACK: Delegation abuse - inherit more than delegator has
#[test]
fn attack_delegation_amplification() {
    // Setup: alice has READ on doc:x, delegates to bob
    // Attack: bob tries to get WRITE via delegation
    // Expected: bob gets only READ (can't exceed delegator)
}

/// ATTACK: Bootstrap replay - re-run bootstrap to become root
#[test]
fn attack_bootstrap_replay() {
    // Setup: system already bootstrapped
    // Attack: call bootstrap("attacker")
    // Expected: ERROR - already bootstrapped
}

/// ATTACK: Circular delegation DoS
#[test]
fn attack_circular_delegation_dos() {
    // Setup: A delegates to B, B delegates to C, C delegates to A
    // Attack: query access for A (infinite loop)
    // Expected: bounded traversal, no hang
}

/// ATTACK: Type mutation after bootstrap
#[test]
fn attack_mutate_system_types() {
    // Setup: bootstrapped system
    // Attack: bob tries to delete _type:user
    // Expected: DENIED - bob lacks TYPE_DELETE on _type:_type
}
```

---

### 1.3 Protected API Tests (New)

```rust
// tests/protected_api.rs (~200 LOC)

mod bootstrap {
    #[test] fn bootstrap_creates_root_with_all_caps() {}
    #[test] fn bootstrap_creates_core_types() {}
    #[test] fn bootstrap_only_runs_once() {}
    #[test] fn after_bootstrap_mutations_require_auth() {}
}

mod entity_lifecycle {
    #[test] fn create_entity_requires_entity_create_on_type() {}
    #[test] fn delete_entity_requires_entity_delete_on_type() {}
    #[test] fn entity_ids_must_be_typed_format() {}
    #[test] fn cannot_create_duplicate_entity() {}
}

mod grants {
    #[test] fn set_grant_requires_grant_write_on_scope() {}
    #[test] fn delete_grant_requires_grant_delete_on_scope() {}
    #[test] fn grant_validates_scope_exists() {}
}

mod capabilities {
    #[test] fn set_capability_requires_cap_write_on_scope() {}
    #[test] fn capabilities_are_per_scope_per_relation() {}
}

mod delegations {
    #[test] fn set_delegation_requires_delegate_write_on_scope() {}
    #[test] fn delegation_bounded_by_delegator_caps() {}
    #[test] fn delegation_is_scope_specific() {}
}

mod access_evaluation {
    #[test] fn check_direct_grant() {}
    #[test] fn check_via_delegation() {}
    #[test] fn check_multi_level_delegation() {}
    #[test] fn check_multiple_relations_or_together() {}
    #[test] fn check_no_access_returns_zero() {}
}
```

---

### 1.4 Simulation Tests (New - From SIMULATION.md)

```rust
// tests/simulation.rs (~100 LOC)

/// Full organization simulation from SIMULATION.md
#[test]
fn simulation_acme_corp() {
    // Phase 1: Bootstrap
    bootstrap("user:root")?;

    // Phase 2: Root creates org structure
    create_entity("user:root", "team", "hr")?;
    create_entity("user:root", "team", "engineering")?;
    create_entity("user:root", "user", "alice")?;  // HR lead
    create_entity("user:root", "user", "bob")?;    // Eng lead

    // Define team capabilities
    set_capability("user:root", "team:hr", "lead", GRANT_WRITE | GRANT_READ)?;
    set_grant("user:root", "user:alice", "lead", "team:hr")?;

    // HR manages users
    set_grant("user:root", "team:hr", "admin", "_type:user")?;
    set_delegation("user:root", "user:alice", "_type:user", "team:hr")?;

    // Phase 3: Delegated operations
    // Alice (HR) can now create users
    assert!(check_access("user:alice", "_type:user", None)? & ENTITY_CREATE != 0);
    create_entity("user:alice", "user", "frank")?;

    // Alice CANNOT create teams
    assert!(check_access("user:alice", "_type:team", None)? & ENTITY_CREATE == 0);

    // Bob can add members to his team
    assert!(check_access("user:bob", "team:engineering", None)? & GRANT_WRITE != 0);
}
```

---

## Phase 2: Implementation (Lean Approach)

### 2.1 File Structure (Minimal)

```
src/
├── lib.rs          # Re-exports (keep as-is)
├── core.rs         # Internal ops (rename funcs, add types/entities DBs)
├── protected.rs    # Protected API layer (~150 LOC) [NEW]
├── bootstrap.rs    # Genesis logic (~50 LOC) [NEW]
└── caps.rs         # SystemCap constants (~30 LOC) [NEW]
```

**Total new code:** ~230 LOC
**Modified code:** ~100 LOC changes to core.rs
**Net increase:** ~300 LOC (still under 1000 total)

---

### 2.2 Implementation Order

| Step | Change | LOC | Tests First |
|------|--------|-----|-------------|
| 1 | Add `SystemCap` constants | +30 | `caps.rs` |
| 2 | Add `types/` and `entities/` DBs to core | +50 | entity_lifecycle tests |
| 3 | Add internal `create_type_in`, `create_entity_in` | +40 | - |
| 4 | Add `bootstrap.rs` | +50 | bootstrap tests |
| 5 | Add `protected.rs` with `set_grant`, `set_capability`, etc. | +150 | protected_api tests |
| 6 | Rename: subject→seeker, object→scope, rel_type→relation | ~0 (aliases) | v1_compat tests pass |
| 7 | Wire up: all public write funcs go through protected | +20 | attack_vector tests |

---

### 2.3 What We're NOT Adding (Stay Lean)

| Feature | Status | Reason |
|---------|--------|--------|
| Policies | **DEFER to v3** | Adds ~200 LOC, not core to security |
| Audit logging | **DEFER to v3** | Adds ~150 LOC, can layer on later |
| seeker_policies | **DEFER to v3** | Pre-flight checks can be app-layer |
| NAPI bindings | **DEFER to v3** | Separate package |

---

### 2.4 Core Changes (Minimal Diff)

```rust
// core.rs additions

// New databases (add to Databases struct)
types: Database<Str, SerdeBincode<TypeMeta>>,
entities: Database<Str, SerdeBincode<EntityMeta>>,

// New internal functions
fn create_type_in(txn, dbs, type_name, creator) -> Result<u64>
fn create_entity_in(txn, dbs, entity_type, id, creator) -> Result<u64>
fn entity_exists(txn, dbs, entity_id) -> Result<bool>
fn parse_entity_id(id: &str) -> Result<(&str, &str)>  // "user:john" -> ("user", "john")

// Rename (via type alias for backwards compat)
pub type Subject = String;  // deprecated, use Seeker
pub type Object = String;   // deprecated, use Scope
pub type RelType = String;  // deprecated, use Relation
```

---

## Phase 3: Test Execution Matrix

| Test File | Runs Against | Purpose |
|-----------|--------------|---------|
| `v1_compat.rs` | Before & After | No regression |
| `attack_vectors.rs` | After only | Security validation |
| `protected_api.rs` | After only | New functionality |
| `simulation.rs` | After only | End-to-end scenario |

---

## Summary

**Test-first deliverables:**
1. `tests/attack_vectors.rs` (~150 LOC) - Security scenarios
2. `tests/protected_api.rs` (~200 LOC) - New API coverage
3. `tests/simulation.rs` (~100 LOC) - Full org scenario

**Implementation deliverables:**
1. `src/caps.rs` (~30 LOC) - System capability constants
2. `src/bootstrap.rs` (~50 LOC) - Genesis logic
3. `src/protected.rs` (~150 LOC) - Permission-checked API

**Total new LOC:** ~680 (tests: ~450, impl: ~230)
**Philosophy:** Tests are the spec. Implementation follows tests.
