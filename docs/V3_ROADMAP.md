# Capbit v3 Roadmap

Features and improvements planned for Capbit v3.

---

## 1. Revocation Propagation

**Problem:** When a delegator loses access, delegatees retain inherited permissions until manually revoked.

**Solution:**

```rust
// v3: Automatic revocation propagation
pub struct RevocationEvent {
    entity: String,
    scope: String,
    timestamp: u64,
    cascade: bool,
}

// When alice loses access to doc:123, all who inherited from alice lose it too
protected::revoke_grant("user:root", "user:alice", "editor", "resource:doc")?;
// Automatically triggers: revoke_inherited("resource:doc", "user:alice")
```

**Implementation:**
- Add `revocation_log` database for tracking revocation events
- On grant deletion, scan `inheritance_by_source` for affected delegatees
- Option for soft revocation (mark invalid) vs hard revocation (delete records)
- Configurable cascade depth limit

**API:**
```rust
// Revoke with cascade (default)
protected::delete_grant(actor, seeker, relation, scope)?;

// Revoke without cascade (explicit)
protected::delete_grant_no_cascade(actor, seeker, relation, scope)?;

// Check if permission was revoked (for audit)
protected::is_revoked(seeker, scope) -> Result<Option<RevocationEvent>>
```

---

## 2. Rate Limiting

**Problem:** No protection against permission check floods or mutation spam.

**Solution:**

```rust
// v3: Built-in rate limiting
pub struct RateLimitConfig {
    // Permission checks
    check_limit_per_sec: u32,      // Default: 10000
    check_burst: u32,              // Default: 1000

    // Mutations
    mutation_limit_per_sec: u32,   // Default: 100
    mutation_burst: u32,           // Default: 10

    // Per-actor limits
    per_actor_check_limit: u32,    // Default: 1000/sec
    per_actor_mutation_limit: u32, // Default: 10/sec
}

// Initialize with rate limiting
init_with_config("./data", RateLimitConfig::default())?;
```

**Implementation:**
- Token bucket algorithm per operation type
- Optional per-actor tracking (requires actor registry)
- Configurable at init time
- Returns `CapbitError::RateLimited` when exceeded

**API:**
```rust
// Check with rate limit awareness
match check_access(subject, object, None) {
    Ok(caps) => { /* proceed */ },
    Err(e) if e.is_rate_limited() => { /* back off */ },
    Err(e) => { /* other error */ },
}

// Get current rate limit status
get_rate_limit_status() -> RateLimitStatus
```

---

## 3. Audit Logging

**Problem:** No built-in audit trail for compliance and forensics.

**Solution:**

```rust
// v3: Comprehensive audit logging
pub struct AuditEntry {
    id: u64,
    timestamp: u64,
    actor: String,
    action: AuditAction,
    target: String,
    scope: Option<String>,
    result: AuditResult,
    metadata: HashMap<String, String>,
}

pub enum AuditAction {
    // Reads
    CheckAccess,
    ListAccessible,
    ListSubjects,

    // Mutations
    CreateEntity,
    DeleteEntity,
    SetGrant,
    DeleteGrant,
    SetCapability,
    SetDelegation,
    DeleteDelegation,

    // Admin
    Bootstrap,
    CreateType,
}

pub enum AuditResult {
    Success,
    Denied { reason: String },
    Error { message: String },
}
```

**Implementation:**
- New `audit_log` database with append-only writes
- Configurable verbosity levels (mutations only, all ops, none)
- Optional async writing to avoid latency impact
- Log rotation / retention policies

**API:**
```rust
// Enable audit logging
init_with_audit("./data", AuditConfig {
    level: AuditLevel::Mutations,  // or ::All, ::None
    async_write: true,
    retention_days: 90,
})?;

// Query audit log
query_audit_log(AuditQuery {
    actor: Some("user:alice"),
    action: Some(AuditAction::SetGrant),
    start_time: Some(epoch - 86400000),
    end_time: None,
    limit: 100,
}) -> Result<Vec<AuditEntry>>

// Export for compliance
export_audit_log(format: ExportFormat::JSON, path: &str) -> Result<()>
```

---

## 4. Policy Engine

**Problem:** Complex authorization rules require code changes.

**Solution:**

```rust
// v3: Declarative policy layer on top of capabilities
pub struct Policy {
    id: String,
    name: String,
    conditions: Vec<Condition>,
    effect: Effect,
    priority: i32,
}

pub enum Condition {
    // Time-based
    TimeRange { start: u64, end: u64 },
    DayOfWeek { days: Vec<u8> },

    // Context-based
    ActorType { type_name: String },
    ScopeType { type_name: String },
    RelationType { relation: String },

    // Capability-based
    MinCapability { cap_mask: u64 },
    MaxCapability { cap_mask: u64 },

    // Custom
    Custom { evaluator: String },  // Named function reference
}

pub enum Effect {
    Allow,
    Deny,
    RequireMFA,  // Integration point for auth
}
```

**Implementation:**
- Policies stored in `policies` database
- Evaluated AFTER capability check (additional layer)
- Deny overrides Allow at same priority
- Higher priority evaluated first

**API:**
```rust
// Create policy
protected::create_policy("user:root", Policy {
    id: "no-weekend-access".into(),
    name: "Deny access on weekends".into(),
    conditions: vec![
        Condition::DayOfWeek { days: vec![0, 6] },  // Sun, Sat
        Condition::ScopeType { type_name: "resource".into() },
    ],
    effect: Effect::Deny,
    priority: 100,
})?;

// Check with policies
check_access_with_policies(subject, object, context) -> Result<AccessDecision>

pub struct AccessDecision {
    allowed: bool,
    capability: u64,
    applied_policies: Vec<String>,
    denied_by: Option<String>,
}
```

---

## 5. User Authentication Integration

**Problem:** Capbit handles authorization but not authentication.

**Solution:**

```rust
// v3: Authentication hooks (not a full auth system)
pub trait AuthProvider: Send + Sync {
    fn verify_token(&self, token: &str) -> Result<AuthenticatedActor>;
    fn refresh_token(&self, token: &str) -> Result<String>;
    fn revoke_token(&self, token: &str) -> Result<()>;
}

pub struct AuthenticatedActor {
    entity_id: String,           // e.g., "user:alice"
    session_id: String,
    issued_at: u64,
    expires_at: u64,
    metadata: HashMap<String, String>,
}

// Built-in providers
pub struct JWTProvider { /* JWT verification */ }
pub struct OIDCProvider { /* OpenID Connect */ }
pub struct APIKeyProvider { /* Simple API keys */ }
```

**Implementation:**
- Auth is optional - v2 API still works without it
- When enabled, all protected operations require valid token
- Token-to-entity mapping stored in `sessions` database
- Session revocation triggers permission cache invalidation

**API:**
```rust
// Initialize with auth
init_with_auth("./data", AuthConfig {
    provider: Box::new(JWTProvider::new(jwt_secret)),
    session_ttl: 3600,
    require_auth: true,
})?;

// Authenticated operations
let session = authenticate(token)?;
protected::set_grant(&session, "user:bob", "editor", "resource:doc")?;

// Or with middleware pattern
with_auth(token, |actor| {
    protected::set_grant(actor, "user:bob", "editor", "resource:doc")
})?;

// Session management
create_session(entity_id, ttl) -> Result<String>  // Returns token
validate_session(token) -> Result<AuthenticatedActor>
revoke_session(token) -> Result<()>
list_sessions(entity_id) -> Result<Vec<SessionInfo>>
```

---

## 6. Additional v3 Improvements

### 6.1 Capability Expiration
```rust
// Time-limited grants
protected::set_grant_with_expiry(
    actor, seeker, relation, scope,
    expires_at: u64,  // Unix timestamp
)?;

// Auto-cleanup of expired grants
cleanup_expired_grants() -> Result<u64>  // Returns count
```

### 6.2 Conditional Capabilities
```rust
// Capabilities that depend on context
protected::set_conditional_capability(
    actor, scope, relation,
    base_cap: u64,
    conditions: Vec<CapCondition>,
)?;

// e.g., "editor" gives write access only during business hours
```

### 6.3 Capability Inheritance (Type-Level)
```rust
// Define that "admin" always includes "editor" capabilities
protected::set_relation_inheritance(
    actor,
    scope_type: "resource",
    parent_relation: "admin",
    child_relation: "editor",
)?;
```

### 6.4 Batch Operations with Rollback
```rust
// Atomic batch with automatic rollback on failure
let batch = ProtectedBatch::new(actor)
    .create_entity("team", "new-team")
    .set_grant("user:alice", "lead", "team:new-team")
    .set_capability("team:new-team", "member", 0x10);

batch.execute()?;  // All or nothing
```

### 6.5 Word Masks (Unlimited Primitives)

**Problem:** Current 64-bit capability masks limit organizations to 64 primitives per entity. Complex systems may need hundreds of fine-grained permissions.

**Reality Check:** In practice, 64 bits is almost always sufficient because:
- **Capabilities are scoped per entity** - Each entity (resource:office, app:api, team:sales) defines its own primitive bits independently
- **Grants are per entity** - You don't need a global namespace of thousands of actions
- **Roles bundle primitives** - An entity typically has 5-15 meaningful roles, not thousands

Even a large corporation with thousands of resources only needs 64 primitives *per resource*. The bits for `resource:nyc-office` are completely separate from `resource:london-office`. This scoping makes word masks a "nice to have" rather than essential.

**Solution (for rare edge cases):**

```rust
// v3: Variable-width capability masks using multiple words
pub struct CapabilityMask {
    words: Vec<u64>,  // Dynamically sized
    labels: Option<HashMap<u32, String>>,  // Optional bit labels
}

impl CapabilityMask {
    // Create from single word (backwards compatible)
    pub fn from_u64(val: u64) -> Self { /* ... */ }

    // Create multi-word mask
    pub fn new(num_words: usize) -> Self { /* ... */ }

    // Set/get individual bits
    pub fn set_bit(&mut self, bit: u32) { /* ... */ }
    pub fn get_bit(&self, bit: u32) -> bool { /* ... */ }

    // Label bits for human readability
    pub fn set_label(&mut self, bit: u32, label: &str) { /* ... */ }
}
```

**Example Usage:**

```rust
// Organization with 200+ primitives
let mut mask = CapabilityMask::new(4);  // 256 bits = 4 words

// Define primitives across the full range
mask.set_bit(0);   mask.set_label(0, "can_enter");
mask.set_bit(1);   mask.set_label(1, "can_print");
mask.set_bit(64);  mask.set_label(64, "can_access_floor_2");
mask.set_bit(128); mask.set_label(128, "can_access_building_b");
mask.set_bit(192); mask.set_label(192, "can_access_classified");

// Store capability with word mask
protected::set_capability_wide(
    "user:root",
    "resource:mega-corp-hq",
    "full-access",
    mask,
)?;

// Backwards compatible: u64 automatically converts to CapabilityMask
protected::set_capability("user:root", "resource:doc", "editor", 0x03)?;  // Still works!
```

**Implementation:**
- Storage: Serialize `Vec<u64>` to bytes in capabilities database
- Zero-copy: Only deserialize needed words during checks
- Migration: Existing u64 capabilities auto-upgrade to 1-word CapabilityMask
- Default: 1 word (64 bits) for performance; organizations opt-in to more

**API:**
```rust
// Check with word mask
check_access_wide(subject, object, None) -> Result<CapabilityMask>

// Check specific bit
has_capability_bit(subject, object, bit: u32) -> Result<bool>

// Get capability with labels (for UI display)
get_capability_with_labels(entity, relation) -> Result<(CapabilityMask, HashMap<u32, String>)>

// Define primitive labels for an entity (stored separately)
set_primitive_labels(actor, scope, labels: HashMap<u32, String>) -> Result<()>
```

**Use Cases (rare edge cases):**
- Single entities with genuinely 100+ distinct actions (e.g., complex manufacturing equipment)
- Legacy systems migrating from flat permission lists
- Regulatory environments requiring bit-level audit of all possible actions
- Most organizations will never need this - 64 primitives per entity is plenty

---

## Migration Path

### v2 → v3
```rust
// v2 code continues to work
protected::set_grant("user:root", "user:alice", "editor", "resource:doc")?;

// v3 features are opt-in
init_v3("./data", V3Config {
    audit: Some(AuditConfig::default()),
    rate_limit: Some(RateLimitConfig::default()),
    auth: None,  // Not required
    policies: false,
})?;
```

---

## Implementation Priority

| Feature | Priority | Complexity | Dependencies |
|---------|----------|------------|--------------|
| Audit Logging | HIGH | Medium | None |
| Revocation Propagation | HIGH | Medium | None |
| Rate Limiting | MEDIUM | Low | None |
| Policy Engine | MEDIUM | High | Audit (optional) |
| User Authentication | LOW | High | None |
| Capability Expiration | LOW | Low | Audit |
| Word Masks (Unlimited Primitives) | LOW | Medium | None (rarely needed - 64 bits/entity is plenty) |

---

## Non-Goals for v3

- **Full identity provider**: Use external IdP (Keycloak, Auth0, etc.)
- **Global distribution**: Remains single-node; add replication externally
- **Admin UI**: API-first; build UI separately if needed
- **Graph queries**: Not a graph database; use specialized tools

---

*Capbit v3 - Secure, Auditable, Policy-Driven Access Control*
