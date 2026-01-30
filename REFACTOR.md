# Capbit v2 Refactor Proposal

This document analyzes the proposed refactor, identifies issues, and provides a deployment-ready specification.

---

## 0. Critical Invariant: Entity Types as Mutation Control

### The Problem

Entities are user-defined logical words: "john", "sales", "slack". Without structure:
1. Anyone can use any string as an entity ID
2. No way to control WHO can create/modify WHICH entities
3. Attacker creates "john" and inherits john's permissions

### The Solution: Types Are Scopes

**Entity types themselves become the locus of mutation control.**

You don't ask "can I create john?" - you ask "can I create entities of type `user`?"

```
Types are entities:
  _type:user        ← the entity representing "all users"
  _type:team        ← the entity representing "all teams"
  _type:app         ← the entity representing "all apps"
  _type:resource    ← the entity representing "all resources"

Entities have typed IDs:
  user:john         ← john is a user
  team:sales        ← sales is a team
  app:slack         ← slack is an app
  resource:doc123   ← doc123 is a resource
```

### Mutation Control via Types

```
To create user:john:
  1. Extract type from ID → "user"
  2. Check: does requester have ENTITY_CREATE on _type:user?
  3. Check: does user:john already exist?
  4. Create user:john

To delete team:sales:
  1. Extract type from ID → "team"
  2. Check: does requester have ENTITY_DELETE on _type:team?
  3. Delete team:sales
```

**Delegation becomes natural:**
```rust
// Root delegates user management to HR
set_grant("root", "team:hr", "admin", "_type:user")?;

// Now HR team can create/delete users
// But HR cannot create teams, apps, or resources

// Root delegates app management to DevOps
set_grant("root", "team:devops", "admin", "_type:app")?;
```

### Database Structure

```
LMDB Environment
├── types/                 type_name → TypeMeta { creator, created_epoch, ... }
├── entities/              type:id → EntityMeta { creator, created_epoch, ... }
├── grants/                seeker/relation/scope → epoch
├── grants_rev/            scope/relation/seeker → epoch
├── capabilities/          scope/relation → capability (u64)
...
```

### Type Bootstrap

At genesis, system creates the meta-type and core types:

```rust
fn bootstrap(root_entity: &str) -> Result<()> {
    // 1. Create the meta-type (type of types)
    create_type_internal("_type")?;  // trusted, no permission check

    // 2. Create core entity types
    create_type_internal("user")?;
    create_type_internal("team")?;
    create_type_internal("app")?;
    create_type_internal("resource")?;

    // 3. Define capabilities on _type:_type (creating new types)
    set_capability_internal("_type:_type", "admin", SystemCap::ALL)?;

    // 4. Define capabilities on each type (managing entities of that type)
    for t in ["user", "team", "app", "resource"] {
        set_capability_internal(&format!("_type:{}", t), "admin",
            SystemCap::ENTITY_CREATE | SystemCap::ENTITY_DELETE)?;
    }

    // 5. Grant root full control over all types
    set_grant_internal(root_entity, "admin", "_type:_type")?;
    for t in ["user", "team", "app", "resource"] {
        set_grant_internal(root_entity, "admin", &format!("_type:{}", t))?;
    }

    // 6. Create root as a user entity
    create_entity_internal("user", root_entity.split(':').nth(1).unwrap())?;
}
```

### Entity ID Format

**Format**: `type:identifier`

```
user:john              # User named john
user:auth0|abc123      # User from auth0 (| instead of : for sub-namespacing)
team:engineering       # Team named engineering
app:slack              # App named slack
resource:doc-123       # Resource with ID doc-123
_type:user             # The user type itself (meta)
_system:capbit         # System-level entities
```

### Index Structure

The type prefix enables efficient queries:

```
entities/ database with prefix iteration:
  "user:" prefix → all users
  "team:" prefix → all teams
  "app:"  prefix → all apps

grants/ database:
  "user:john/" prefix → all grants where john is seeker

grants_rev/ database:
  "team:sales/" prefix → all grants where sales is scope
```

### Why This Works

1. **Types are entities** → same permission model applies
2. **Type = scope for mutation** → natural delegation
3. **Flat key space** → efficient LMDB prefix scans
4. **Human readable** → `user:john` not `a]fk2-df92-...`
5. **Collision-free** → `user:alice` ≠ `team:alice`
6. **Audit friendly** → logs show `user:john created team:sales`

---

## 1. Naming Changes Analysis

### Proposed Changes

| Current | Proposed | Verdict |
|---------|----------|---------|
| subject/object | entity/entity | **ACCEPT** - Consistent with "everything is an entity" philosophy |
| rel_type | relation | **ACCEPT** - Neutral, accurate, no semantic confusion |
| cap_mask | capability | **ACCEPT** - Cleaner name |
| source | delegate | **ACCEPT** - Better mental model (active delegation vs passive sourcing) |

### Decision: Using "relation"

**Chosen**: `relation` - neutral, accurate, no semantic confusion.

The path `john/editor/sales` reads as "john has editor relation to sales".

---

## 2. Path Patterns Analysis

### Proposed Patterns

```
Scope Assignments:
  seeker/relation/scope          → epoch
  scope/relation/seeker          → epoch (reverse)

Capability Assignments:
  scope/relation/capability      → bitmask
  scope/relation/policy          → policy_id or inline
  scope/relation/bool            → 0|1
  scope/policy                   → default policy

Inheritance:
  seeker/scope/delegate          → epoch
  delegate/scope/seeker          → epoch (reverse)
  scope/delegate/seeker          → epoch (by-scope index)

Labels:
  scope/relation/capability/label → string
```

### Critique

#### Issue 1: `bool` is redundant

If you have capability bitmasks, a single bit (e.g., `0x01`) **is** a boolean. Adding a separate `bool` type creates:
- Extra code paths
- Confusion about when to use bool vs capability
- No functional benefit

**Recommendation**: Remove `bool`. Use capability with a single bit (e.g., `ACCESS = 0x01`).

#### ~~Issue 2: `entity/policy` (default policy) complicates evaluation~~ **ACCEPTED**

**Clarification**: This is a "double-check" mechanism—even if a seeker has a relation granting access, we always run an additional policy check on the seeker itself.

**Use cases**:
- User account status (active/suspended/locked)
- User location restrictions (geo-fencing)
- User session validity
- Global rate limiting
- Compliance holds

**Evaluation order**:
```
1. Check seeker/policy → if fails, DENY immediately
2. Check seeker/relation/scope → gather capabilities
3. Check scope/relation/policy → conditional relation rules
4. Return effective capabilities
```

This is NOT implicit behavior—it's explicit "always check this first" semantics. Similar to how firewalls check deny-lists before allow-lists.

**Implementation**: Add `seeker_policies` database storing `seeker → policy_id`.

#### Issue 3: Four-part label paths are over-engineered

Current: `entity/cap_bit → label` (simple)
Proposed: `scope/relation/capability/label` (complex)

Why would the same capability bit need different labels per relation? Labels are for human readability of the bitmask, not per-relation semantics.

**Recommendation**: Keep current `entity/cap_bit → label` pattern.

#### Issue 4: Policy storage needs definition

Where does policy logic live? Options:
1. **Inline expression**: `scope/relation/policy → "time >= 9 AND time <= 17"`
2. **Policy ID reference**: `scope/relation/policy → "work_hours_policy"`
3. **External function**: Policy is code registered with the system

**Recommendation**: Use policy ID reference. Store policies separately in a `policies` database with their logic. This allows policy reuse and centralized management.

---

## 3. Refined Path Patterns (Deployment-Ready)

```
DATABASES:
├── grants/              seeker/relation/scope → epoch
├── grants_rev/          scope/relation/seeker → epoch
├── capabilities/        scope/relation → bitmask
├── policies/            policy_id → policy_definition
├── grant_policies/      scope/relation → policy_id (optional conditional)
├── delegations/         seeker/scope/delegate → epoch
├── delegations_by_del/  delegate/scope/seeker → epoch
├── delegations_by_scope/ scope/delegate/seeker → epoch
├── cap_labels/          scope/cap_bit → label
└── audit_log/           epoch → delta (optional)
```

### Naming Conventions (Final)

| Concept | Name | Example |
|---------|------|---------|
| Entity seeking access | `seeker` | "john" |
| Entity providing scope | `scope` | "sales" |
| Relationship connector | `relation` | "editor" |
| Permission bits | `capability` | 0x03 |
| Inheritance source | `delegate` | "mary" |

---

## 4. Bootstrapping Analysis

### Proposed Bootstrap Model

```
SYSTEM entity
ROOT_USER entity
ROOT_AUTHORITY entity (all capabilities)
ROOT_AUTHORITY_* entities (granular)
ROOT_CAPABILITIES entity
```

### ~~Critical Issues~~ Design Decisions

#### ~~Issue 1: Scope creep~~ **ACCEPTED: Write-Side Access Control is Critical**

**The gap**: Current capbit has access control for **reads** (`check_access`, `has_capability`) but **none for writes** (`set_relationship`, `set_capability`, etc.). Anyone who can call the library can mutate permission data.

This is a security hole. Access control for mutations is just as critical as access control for queries.

**Solution**: Genesis bootstrapping creates an initial root user who can:
1. Assign authorities to other entities
2. Delegate subsets of their capabilities
3. Control who can mutate the permission graph

Without genesis, there's no "first mover" who legitimately has mutation rights.

#### Issue 2: Chicken-and-egg complexity

For the system to check permissions, it needs to read permission data. But reading permission data requires... permission?

Your proposed solution (genesis step) works but adds:
- Special "trusted mode" code paths
- Risk of misconfiguration leaving system in inconsistent state
- Complexity in upgrade/migration scenarios

#### Issue 3: Over-granular root authorities

Proposed: `ROOT_AUTHORITY_ASSIGN_CAPABILITIES`, `ROOT_AUTHORITY_ASSIGN_SCOPE`, etc.

This creates N+1 authorities to manage instead of 1. Consider:
- Who manages the ROOT_AUTHORITY_* assignments?
- What prevents accidental removal of critical grants?

**Recommendation**: Simpler model:

```
ROOT (all capabilities) → can delegate subsets
ADMIN (CRUD on grants/capabilities) → operational
AUDIT (read-only) → compliance
```

---

## 5. Refined Bootstrap Model (If Proceeding)

### System Capabilities (Bitmask)

```rust
pub mod SystemCap {
    // Type management (on _type:_type scope)
    pub const TYPE_CREATE: u64     = 0x0001;  // Create new entity types
    pub const TYPE_DELETE: u64     = 0x0002;  // Delete entity types

    // Entity management (on _type:{type} scopes)
    pub const ENTITY_CREATE: u64   = 0x0004;  // Create entities of this type
    pub const ENTITY_DELETE: u64   = 0x0008;  // Delete entities of this type

    // Grants (relations between entities)
    pub const GRANT_READ: u64      = 0x0010;  // Read grants
    pub const GRANT_WRITE: u64     = 0x0020;  // Create/update grants
    pub const GRANT_DELETE: u64    = 0x0040;  // Delete grants

    // Capability definitions
    pub const CAP_READ: u64        = 0x0080;  // Read capability definitions
    pub const CAP_WRITE: u64       = 0x0100;  // Create/update capabilities
    pub const CAP_DELETE: u64      = 0x0200;  // Delete capabilities

    // Delegations (inheritance)
    pub const DELEGATE_READ: u64   = 0x0400;  // Read delegations
    pub const DELEGATE_WRITE: u64  = 0x0800;  // Create/update delegations
    pub const DELEGATE_DELETE: u64 = 0x1000;  // Delete delegations

    // Policies
    pub const POLICY_READ: u64     = 0x2000;  // Read policies
    pub const POLICY_WRITE: u64    = 0x4000;  // Create/update policies
    pub const POLICY_DELETE: u64   = 0x8000;  // Delete policies

    // Audit & system (use next bits)
    pub const AUDIT_READ: u64      = 0x10000;  // Read audit logs
    pub const SYSTEM_ADMIN: u64    = 0x20000;  // Bootstrap/system ops

    // Composites
    pub const ALL: u64 = 0x3FFFF;
    pub const TYPE_ADMIN: u64 = TYPE_CREATE | TYPE_DELETE;
    pub const ENTITY_ADMIN: u64 = ENTITY_CREATE | ENTITY_DELETE;
    pub const GRANT_ADMIN: u64 = GRANT_WRITE | GRANT_DELETE;
    pub const CAP_ADMIN: u64 = CAP_WRITE | CAP_DELETE;
    pub const DELEGATE_ADMIN: u64 = DELEGATE_WRITE | DELEGATE_DELETE;
    pub const POLICY_ADMIN: u64 = POLICY_WRITE | POLICY_DELETE;
    pub const READ_ONLY: u64 = GRANT_READ | CAP_READ | DELEGATE_READ |
                               POLICY_READ | AUDIT_READ;
}
```

### Bootstrap Sequence

```rust
/// Called exactly once at system genesis. Creates root user with all capabilities.
/// After bootstrap, all mutations require authorization.
pub fn bootstrap(root_entity: &str) -> Result<()> {
    if is_bootstrapped()? {
        return Err(CapbitError { message: "Already bootstrapped".into() });
    }

    let mut batch = WriteBatch::new();

    // 1. Define system capabilities on _system scope
    batch.set_capability("_system", "root", SystemCap::ALL);
    batch.set_capability("_system", "admin", SystemCap::ADMIN);
    batch.set_capability("_system", "auditor", SystemCap::READ_ONLY);

    // 2. Grant root relation to genesis entity
    batch.set_grant(root_entity, "root", "_system");

    // 3. Label the system capabilities for human readability
    batch.set_cap_label("_system", SystemCap::GRANT_WRITE, "grant:write");
    batch.set_cap_label("_system", SystemCap::GRANT_DELETE, "grant:delete");
    batch.set_cap_label("_system", SystemCap::CAP_WRITE, "capability:write");
    // ... etc

    // 4. Mark as bootstrapped (prevents re-bootstrap)
    batch.set_meta("bootstrapped", "true");
    batch.set_meta("bootstrap_epoch", &current_epoch().to_string());
    batch.set_meta("root_entity", root_entity);

    batch.execute()
}

pub fn is_bootstrapped() -> Result<bool> {
    get_meta("bootstrapped").map(|v| v == Some("true".to_string()))
}
```

### Protected Operation Flow

```rust
/// All mutations go through protected functions after bootstrap
pub fn set_grant(
    requester: &str,      // Who is making this request
    seeker: &str,         // Who gets the relation
    relation: &str,       // What relation
    scope: &str,          // Over what scope
) -> Result<u64> {
    // 1. Check requester has GRANT_WRITE on _system
    let requester_caps = check_access(requester, "_system", None)?;
    if (requester_caps & SystemCap::GRANT_WRITE) == 0 {
        return Err(CapbitError {
            message: format!("{} lacks GRANT_WRITE on _system", requester)
        });
    }

    // 2. Authorized - execute the internal operation
    with_write_txn(|txn, dbs| {
        set_grant_in(txn, dbs, seeker, relation, scope)
    })
}
```

### Delegation Pattern

Root can delegate subsets:

```rust
// Root grants alice admin relation (CRUD but not system admin)
set_grant("root", "alice", "admin", "_system")?;

// Now alice can grant others, but only up to her own level
// Alice cannot grant "root" relation (she doesn't have it)
```
```

### Protected Operations

```rust
pub fn set_grant_protected(
    requester: &str,
    seeker: &str,
    relation: &str,
    scope: &str
) -> Result<u64> {
    // Check requester has GRANT_WRITE on _system
    if !has_capability(requester, "_system", SystemCap::GRANT_WRITE)? {
        return Err(CapbitError { message: "Unauthorized".into() });
    }
    set_grant(seeker, relation, scope)
}
```

---

## 6. Policy System Design

### Policy Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub conditions: Vec<Condition>,
    pub combine: CombineMode,  // All, Any
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    TimeRange { start_hour: u8, end_hour: u8 },
    DayOfWeek { days: Vec<u8> },  // 0=Sun, 6=Sat
    IpRange { cidrs: Vec<String> },
    Custom { key: String, op: Op, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op { Eq, Ne, Gt, Lt, Gte, Lte, Contains, StartsWith }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombineMode { All, Any }
```

### Policy Evaluation Context

```rust
pub struct EvalContext {
    pub time: DateTime<Utc>,
    pub ip: Option<IpAddr>,
    pub custom: HashMap<String, String>,
}

impl Policy {
    pub fn evaluate(&self, ctx: &EvalContext) -> bool {
        match self.combine {
            CombineMode::All => self.conditions.iter().all(|c| c.evaluate(ctx)),
            CombineMode::Any => self.conditions.iter().any(|c| c.evaluate(ctx)),
        }
    }
}
```

### Access Check with Policies

```rust
pub fn check_access_with_context(
    seeker: &str,
    scope: &str,
    ctx: &EvalContext,
    max_depth: Option<usize>,
) -> Result<u64> {
    // Same as check_access but:
    // 1. For each relation found, check if grant_policies has a policy
    // 2. If policy exists, evaluate it with ctx
    // 3. Only include capability if policy passes (or no policy exists)
}
```

---

## 7. Audit System Design

### Audit Log Entry

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub epoch: u64,
    pub operation: AuditOp,
    pub requester: Option<String>,  // Who initiated (if protected mode)
    pub details: AuditDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOp {
    GrantCreated,
    GrantDeleted,
    CapabilitySet,
    DelegationCreated,
    DelegationDeleted,
    PolicyCreated,
    PolicyDeleted,
    AccessChecked,  // Optional: log all access checks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDetails {
    pub seeker: Option<String>,
    pub relation: Option<String>,
    pub scope: Option<String>,
    pub delegate: Option<String>,
    pub capability: Option<u64>,
    pub result: Option<bool>,
}
```

### Audit Configuration

```rust
pub struct AuditConfig {
    pub enabled: bool,
    pub operations: HashSet<AuditOp>,  // Which ops to log
    pub scopes: Option<HashSet<String>>,  // Only these scopes (None = all)
    pub retention_epochs: Option<u64>,  // Auto-cleanup older than this
}
```

---

## 8. Architecture: Two-Layer Design

Write-side access control is critical. The architecture uses two layers:

```
┌─────────────────────────────────────────┐
│            Your Application             │
│  - Authenticates users (identity)       │
│  - Passes requester ID to capbit        │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         Capbit Protected API            │
├─────────────────────────────────────────┤
│  set_grant(requester, seeker, auth, scope)
│  - Checks requester has GRANT_WRITE     │
│  - If authorized, calls internal fn     │
│                                         │
│  check_access(seeker, scope)            │
│  - Read ops may or may not need auth    │
│  - Configurable per deployment          │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         Capbit Internal Layer           │
├─────────────────────────────────────────┤
│  _set_grant_in(txn, ...)                │
│  - No permission checks                 │
│  - Used by protected layer after auth   │
│  - Used by bootstrap (trusted context)  │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│               LMDB                      │
└─────────────────────────────────────────┘
```

**Key insight**: The internal `_in` functions already exist from our refactor. The protected layer wraps them with permission checks. Bootstrap uses them directly in trusted mode.

---

## 9. Migration Path

### Phase 1: Naming Refactor (Non-Breaking)

1. Add type aliases in lib.rs:
   ```rust
   pub type Authority = String;  // was rel_type
   pub type Capability = u64;    // was cap_mask
   ```

2. Add new function names as aliases:
   ```rust
   pub fn set_grant(...) -> Result<u64> { set_relationship(...) }
   pub fn set_delegation(...) -> Result<u64> { set_inheritance(...) }
   ```

3. Deprecate old names with `#[deprecated]`

### Phase 2: Policy Support

1. Add `policies` database
2. Add `grant_policies` database
3. Add `check_access_with_context()` function
4. Keep `check_access()` as policy-less fast path

### Phase 3: Audit Support

1. Add `audit_log` database
2. Add `AuditConfig` to init
3. Wrap write operations to emit audit entries

### Phase 4: Protected Mode (Optional)

1. Add `_protected` variants of all write functions
2. Add bootstrap function
3. Keep unprotected functions for library use

---

## 10. Final Database Schema

```
LMDB Environment
│
│  # Type & Entity Registry
├── types/                 type_name → TypeMeta { creator, created_epoch }
├── entities/              type:id → EntityMeta { creator, created_epoch }
│
│  # Grants (who has what relation to whom)
├── grants/                seeker/relation/scope → epoch
├── grants_rev/            scope/relation/seeker → epoch
│
│  # Capabilities (what relations mean on each scope)
├── capabilities/          scope/relation → capability (u64)
│
│  # Policies
├── policies/              policy_id → policy_json
├── seeker_policies/       seeker → policy_id (pre-flight check)
├── grant_policies/        scope/relation → policy_id (conditional)
│
│  # Delegations (inheritance)
├── delegations/           seeker/scope/delegate → epoch
├── delegations_by_del/    delegate/scope/seeker → epoch
├── delegations_by_scope/  scope/delegate/seeker → epoch
│
│  # Labels & Audit
├── cap_labels/            scope/cap_bit → label
├── audit_log/             epoch → audit_entry_json
└── meta/                  key → value (bootstrapped, version, etc.)
```

### Key Relationships

```
_type:_type                          # Meta-type: controls creation of new types
  └── admin relation → TYPE_CREATE | TYPE_DELETE

_type:user                           # User type: controls creation of users
  └── admin relation → ENTITY_CREATE | ENTITY_DELETE

user:john                            # An actual user entity
team:sales                           # An actual team entity

# To create user:alice, requester needs ENTITY_CREATE on _type:user
# To create a new type "project", requester needs TYPE_CREATE on _type:_type
```

**Policy evaluation order**:
1. `seeker_policies[seeker]` → Pre-flight check (account active? location ok?)
2. `grants[seeker/*/scope]` → Get all relations
3. `grant_policies[scope/relation]` → Per-relation conditions (time-based access?)
4. `capabilities[scope/relation]` → Get capability bits
5. OR all passing capabilities → Return effective mask

---

## 11. Open Questions

1. ~~**Naming**: Confirm `authority` vs `grant` vs `relation`?~~ **DECIDED: `relation`**

2. **Policies**: Do you need complex expressions or just predefined conditions (time, IP, day)?

3. **Audit**: Log all operations or configurable subset?

4. **Seeker policy failure**: On seeker policy fail, return 0x00 (no capabilities) or explicit error?

5. **Read-side protection**: Should `check_access` also require a requester, or only mutations?

6. **Grant validation**: When creating a grant, must both seeker and scope exist in entities registry?
   - **Strict**: Yes, both must exist (secure, requires entity creation first)
   - **Scope-only**: Only scope must exist (seeker can be external identity)
   - **Lazy**: Neither required (convenient but less secure)

7. **Type creation control**: Who can create new entity types after bootstrap?
   - Only root (via _type:_type admin)?
   - Delegatable to others?

8. **NAPI bindings**: Add now or after core refactor stabilizes?

---

## 12. Recommended Implementation Order

1. **Naming refactor** - Add new names as aliases, deprecate old
2. **Database schema update** - Add new databases, migration script
3. **Bootstrap system** - Genesis, root user, system capabilities
4. **Protected API layer** - All mutations require requester + permission check
5. **Policy storage** - policies, seeker_policies, grant_policies databases
6. **Policy evaluation** - check_access_with_context, seeker pre-flight checks
7. **Audit logging** - Configurable delta recording
8. **NAPI bindings** - Once API is stable

---

## Summary

### Accept
- Entity-centric naming (seeker/scope vs subject/object)
- Delegate terminology for inheritance
- Capability as cleaner name for bitmask
- Policy support for conditional access
- Audit logging for compliance
- Triple delegation indexing

### Reject/Modify
- `bool` type → Use single-bit capability instead
- Four-part label paths → Keep simple `scope/cap_bit → label`

### Previously Questioned, Now Accepted
- `seeker/policy` (default policy) → Valid "double-check" pattern for seeker-level restrictions
- Bootstrap system → **Critical**: Write-side access control is as important as read-side; genesis required

### Clarify
- "Authority" naming—acceptable but consider alternatives
- Policy complexity level needed
- Protected mode timing
