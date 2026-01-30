# Capbit v3 Refactor Plan (Deferred Features)

**Prerequisite:** v2 complete (protected API, typed entities, bootstrap)

This document covers features deferred from v2 to keep the core lean.

---

## Overview

| Feature | LOC Est. | Priority | Dependency |
|---------|----------|----------|------------|
| Policies | ~200 | High | v2 core |
| Seeker Policies | ~80 | Medium | Policies |
| Audit Logging | ~150 | Medium | v2 core |
| NAPI Bindings | ~300 | Low | Stable API |

**Total v3 LOC:** ~730

---

## 1. Policy System

### 1.1 Purpose

Conditional access based on runtime context (time, IP, custom attributes).

**Use cases:**
- Time-based access (business hours only)
- Geo-fencing (office IP ranges)
- Device restrictions
- Temporary access windows

### 1.2 Database Additions

```
├── policies/            policy_id → policy_json
├── grant_policies/      scope/relation → policy_id
```

### 1.3 Data Structures

```rust
// src/policy.rs (~200 LOC)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub conditions: Vec<Condition>,
    pub combine: CombineMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    TimeRange { start_hour: u8, end_hour: u8 },
    DayOfWeek { days: Vec<u8> },  // 0=Sun, 6=Sat
    IpRange { cidrs: Vec<String> },
    DateRange { start: String, end: String },  // ISO dates
    Custom { key: String, op: Op, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op { Eq, Ne, Gt, Lt, Gte, Lte, Contains, StartsWith, In }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombineMode { All, Any }

#[derive(Debug, Clone, Default)]
pub struct EvalContext {
    pub time: Option<DateTime<Utc>>,
    pub ip: Option<IpAddr>,
    pub custom: HashMap<String, String>,
}

impl Policy {
    pub fn evaluate(&self, ctx: &EvalContext) -> bool {
        let results = self.conditions.iter().map(|c| c.evaluate(ctx));
        match self.combine {
            CombineMode::All => results.all(|r| r),
            CombineMode::Any => results.any(|r| r),
        }
    }
}
```

### 1.4 API Additions

```rust
// Policy CRUD
pub fn create_policy(requester: &str, policy: &Policy) -> Result<()>;
pub fn get_policy(policy_id: &str) -> Result<Option<Policy>>;
pub fn delete_policy(requester: &str, policy_id: &str) -> Result<bool>;
pub fn list_policies() -> Result<Vec<Policy>>;

// Attach policy to grant
pub fn set_grant_policy(requester: &str, scope: &str, relation: &str, policy_id: &str) -> Result<()>;
pub fn get_grant_policy(scope: &str, relation: &str) -> Result<Option<String>>;
pub fn delete_grant_policy(requester: &str, scope: &str, relation: &str) -> Result<bool>;

// Context-aware access check
pub fn check_access_with_context(
    seeker: &str,
    scope: &str,
    ctx: &EvalContext,
    max_depth: Option<usize>,
) -> Result<u64>;
```

### 1.5 Access Evaluation with Policies

```
check_access_with_context(seeker, scope, ctx):
  1. Gather relations: grants[seeker/*/scope]
  2. For each relation:
     a. Check grant_policies[scope/relation]
     b. If policy exists, evaluate with ctx
     c. If policy fails, skip this relation
  3. For passing relations, lookup capabilities[scope/relation]
  4. OR all capabilities together
  5. Return effective mask
```

### 1.6 Tests

```rust
// tests/policies.rs (~100 LOC)

#[test] fn policy_time_range_within() {}
#[test] fn policy_time_range_outside() {}
#[test] fn policy_ip_range_match() {}
#[test] fn policy_ip_range_no_match() {}
#[test] fn policy_combine_all_requires_all() {}
#[test] fn policy_combine_any_requires_one() {}
#[test] fn policy_custom_condition() {}
#[test] fn grant_policy_filters_access() {}
#[test] fn no_policy_means_always_allowed() {}
#[test] fn policy_crud_requires_permission() {}
```

---

## 2. Seeker Policies (Pre-flight Checks)

### 2.1 Purpose

Check seeker-level restrictions BEFORE evaluating grants. Fail-fast for suspended users, geo-blocked accounts, etc.

### 2.2 Database Additions

```
├── seeker_policies/     seeker → policy_id
```

### 2.3 API Additions

```rust
pub fn set_seeker_policy(requester: &str, seeker: &str, policy_id: &str) -> Result<()>;
pub fn get_seeker_policy(seeker: &str) -> Result<Option<String>>;
pub fn delete_seeker_policy(requester: &str, seeker: &str) -> Result<bool>;
```

### 2.4 Access Evaluation with Seeker Policy

```
check_access_with_context(seeker, scope, ctx):
  0. PRE-FLIGHT: Check seeker_policies[seeker]
     - If policy exists and fails → return 0 immediately
  1. (continue with normal evaluation)
```

### 2.5 Tests

```rust
// tests/seeker_policies.rs (~50 LOC)

#[test] fn seeker_policy_blocks_all_access() {}
#[test] fn seeker_policy_passes_continues_eval() {}
#[test] fn no_seeker_policy_allows_eval() {}
#[test] fn seeker_policy_checked_before_grants() {}
```

---

## 3. Audit Logging

### 3.1 Purpose

Record all mutations for compliance, debugging, and forensics.

### 3.2 Database Additions

```
├── audit_log/           epoch → audit_entry_json
├── meta/                audit_config → config_json
```

### 3.3 Data Structures

```rust
// src/audit.rs (~150 LOC)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub epoch: u64,
    pub timestamp: String,  // ISO 8601
    pub operation: AuditOp,
    pub requester: String,
    pub details: AuditDetails,
    pub result: AuditResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOp {
    // Entity lifecycle
    EntityCreated,
    EntityDeleted,
    // Grants
    GrantCreated,
    GrantDeleted,
    // Capabilities
    CapabilitySet,
    CapabilityDeleted,
    // Delegations
    DelegationCreated,
    DelegationDeleted,
    // Policies
    PolicyCreated,
    PolicyDeleted,
    GrantPolicySet,
    SeekerPolicySet,
    // Access (optional, high volume)
    AccessChecked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDetails {
    pub seeker: Option<String>,
    pub relation: Option<String>,
    pub scope: Option<String>,
    pub delegate: Option<String>,
    pub capability: Option<u64>,
    pub policy_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Denied { reason: String },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub operations: HashSet<AuditOp>,
    pub include_access_checks: bool,  // high volume, usually off
    pub retention_days: Option<u32>,
}
```

### 3.4 API Additions

```rust
// Configuration
pub fn set_audit_config(requester: &str, config: &AuditConfig) -> Result<()>;
pub fn get_audit_config() -> Result<AuditConfig>;

// Query
pub fn get_audit_log(
    start_epoch: Option<u64>,
    end_epoch: Option<u64>,
    operations: Option<&[AuditOp]>,
    requester: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<AuditEntry>>;

// Maintenance
pub fn prune_audit_log(before_epoch: u64) -> Result<u64>;  // returns count deleted
```

### 3.5 Implementation

Wrap all protected functions to emit audit entries:

```rust
pub fn set_grant(requester: &str, seeker: &str, relation: &str, scope: &str) -> Result<u64> {
    let result = set_grant_internal(requester, seeker, relation, scope);

    if audit_enabled(AuditOp::GrantCreated) {
        emit_audit(AuditEntry {
            operation: AuditOp::GrantCreated,
            requester: requester.into(),
            details: AuditDetails {
                seeker: Some(seeker.into()),
                relation: Some(relation.into()),
                scope: Some(scope.into()),
                ..Default::default()
            },
            result: match &result {
                Ok(_) => AuditResult::Success,
                Err(e) => AuditResult::Error { message: e.message.clone() },
            },
            ..Default::default()
        });
    }

    result
}
```

### 3.6 Tests

```rust
// tests/audit.rs (~80 LOC)

#[test] fn audit_logs_grant_created() {}
#[test] fn audit_logs_grant_denied() {}
#[test] fn audit_disabled_no_entries() {}
#[test] fn audit_filter_by_operation() {}
#[test] fn audit_filter_by_requester() {}
#[test] fn audit_filter_by_time_range() {}
#[test] fn audit_prune_removes_old() {}
#[test] fn audit_access_checks_optional() {}
```

---

## 4. NAPI Bindings (Node.js)

### 4.1 Purpose

Use capbit from Node.js/TypeScript applications.

### 4.2 Package Structure

```
capbit-node/
├── Cargo.toml          # napi-rs dependency
├── src/
│   └── lib.rs          # NAPI exports (~300 LOC)
├── index.d.ts          # TypeScript definitions
├── package.json
└── tests/
    └── index.test.ts
```

### 4.3 API Surface

```typescript
// TypeScript interface

export interface Capbit {
  // Initialization
  init(dbPath: string): void;
  bootstrap(rootEntity: string): void;
  isBootstrapped(): boolean;

  // Entities
  createEntity(requester: string, entityType: string, id: string): number;
  deleteEntity(requester: string, entityId: string): boolean;
  entityExists(entityId: string): boolean;

  // Grants
  setGrant(requester: string, seeker: string, relation: string, scope: string): number;
  deleteGrant(requester: string, seeker: string, relation: string, scope: string): boolean;
  getGrants(seeker: string, scope: string): string[];

  // Capabilities
  setCapability(requester: string, scope: string, relation: string, capMask: bigint): number;
  getCapability(scope: string, relation: string): bigint | null;

  // Delegations
  setDelegation(requester: string, seeker: string, scope: string, delegate: string): number;
  deleteDelegation(requester: string, seeker: string, scope: string, delegate: string): boolean;

  // Access checks
  checkAccess(seeker: string, scope: string, maxDepth?: number): bigint;
  hasCapability(seeker: string, scope: string, required: bigint): boolean;

  // Queries
  listAccessible(seeker: string): Array<{ scope: string; relation: string }>;
  listSeekers(scope: string): Array<{ seeker: string; relation: string }>;
}

export const SystemCap: {
  TYPE_CREATE: bigint;
  TYPE_DELETE: bigint;
  ENTITY_CREATE: bigint;
  ENTITY_DELETE: bigint;
  GRANT_READ: bigint;
  GRANT_WRITE: bigint;
  GRANT_DELETE: bigint;
  CAP_READ: bigint;
  CAP_WRITE: bigint;
  CAP_DELETE: bigint;
  DELEGATE_READ: bigint;
  DELEGATE_WRITE: bigint;
  DELEGATE_DELETE: bigint;
  ALL: bigint;
};
```

### 4.4 Implementation Notes

- Use `napi-rs` for zero-copy bindings
- Capabilities as `BigInt` (u64 doesn't fit in JS number)
- Async variants for batch operations
- Error handling via napi Result types

### 4.5 Tests

```typescript
// tests/index.test.ts (~100 LOC)

describe('capbit-node', () => {
  test('bootstrap creates root', () => {});
  test('create entity requires permission', () => {});
  test('grant flow works', () => {});
  test('check access returns bigint', () => {});
  test('has capability returns boolean', () => {});
  test('SystemCap constants are bigints', () => {});
});
```

---

## Implementation Order

| Phase | Feature | Depends On | LOC |
|-------|---------|------------|-----|
| 3.1 | Policies core | v2 | +200 |
| 3.2 | Seeker policies | 3.1 | +80 |
| 3.3 | Audit logging | v2 | +150 |
| 3.4 | NAPI bindings | 3.1-3.3 stable | +300 |

**Recommended:** Ship 3.1-3.3 together, NAPI after API stabilizes.

---

## Summary

v3 adds operational features on top of v2's security foundation:

| Feature | What it enables |
|---------|-----------------|
| Policies | Conditional access (time, IP, custom) |
| Seeker policies | Account-level restrictions (suspended, geo-blocked) |
| Audit | Compliance, forensics, debugging |
| NAPI | Node.js/TypeScript integration |

**Total v3 LOC:** ~730
**Combined v2+v3:** ~1400 LOC (still lean)
