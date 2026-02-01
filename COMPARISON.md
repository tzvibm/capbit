# Capbit vs Zanzibar: A Technical Comparison

## Executive Summary

Capbit and Google's Zanzibar are both authorization systems, but with fundamentally different designs. This document argues that Capbit's model is **more expressive, faster, and simpler** for the majority of authorization use cases.

| Dimension | Zanzibar | Capbit |
|-----------|----------|--------|
| Permission model | Boolean relations | 64-bit masks (2^64 combinations) |
| Roles per object | Fixed by schema | Unlimited (any u64) |
| Schema requirement | Yes | None |
| Entity types | Separate namespaces | Unified u64 IDs (with labels) |
| Local check latency | Graph traversal + lookups | ~2-3µs (bitmask AND) |
| Update propagation | Cache invalidation | Instant |
| Graph restructuring | Update many tuples | Single inheritance change |
| Embedding | Bundled with distribution | ~280 lines, embeddable |

---

## 1. Permission Expressiveness

### Zanzibar: Boolean Relations

Zanzibar uses relation tuples with boolean semantics:

```
doc:readme#viewer@user:alice    // alice IS a viewer (true/false)
doc:readme#editor@user:bob      // bob IS an editor (true/false)
```

Relations must be predefined in namespace configuration:

```
namespace doc {
  relation viewer: user
  relation editor: user
  relation owner: user
}
```

**Limitations:**
- Adding a new permission type requires schema migration
- Each relation is binary (has or doesn't have)
- Permission combinations require multiple relation checks

### Capbit: 64-bit Dynamic Masks

Capbit uses bitmask permissions with dynamic roles:

```rust
// Define any role on the fly with any permission combination
set_role(doc, VIEWER_ROLE, READ)?;
set_role(doc, EDITOR_ROLE, READ | WRITE)?;
set_role(doc, CUSTOM_ROLE, READ | WRITE | DELETE | CUSTOM_BIT_42)?;

// Grant user a role
grant(alice, doc, EDITOR_ROLE)?;
```

**Advantages:**
- 2^64 possible permission combinations per role
- Unlimited role IDs per object (any u64 value)
- Create new roles at runtime, no schema
- Single check evaluates all 64 permission bits in O(1)

### Comparison

| Aspect | Zanzibar | Capbit |
|--------|----------|--------|
| Permissions per relation/role | 1 (boolean) | 64 bits (2^64 combos) |
| Roles per object | Fixed at schema time | Unlimited (2^64 role IDs) |
| New permission type | Schema migration | Runtime, instant |
| Permission check | Per-relation lookup | Single bitmask AND |

**Winner: Capbit** — More granular, more flexible, no schema lock-in.

---

## 2. Graph Structures and Inheritance

### Zanzibar: Relation Graph Traversal

Zanzibar models permissions as a graph of relations:

```
folder:engineering#viewer@group:eng-team
group:eng-team#member@user:alice
```

Check "can alice view folder:engineering?" traverses:
```
alice → eng-team (member) → engineering (viewer) ✓
```

**Characteristics:**
- Arbitrary graph shapes
- Multi-directional traversal
- Computed usersets (unions, intersections)
- Graph changes require updating individual tuples

### Capbit: Inheritance DAG

Capbit models permissions as inheritance chains:

```rust
// Build hierarchy
set_inherit(doc, alice, managers)?;
set_inherit(doc, bob, managers)?;
set_inherit(doc, managers, admins)?;
set_inherit(doc, admins, root)?;

// Grant at any level
grant(managers, doc, READ | WRITE)?;
grant(admins, doc, ADMIN)?;
```

Creates this DAG:
```
           root
            ↑
          admins ─── (ADMIN)
            ↑
         managers ── (READ | WRITE)
          ↑    ↑
       alice  bob
```

**Key insight #1:** One inheritance change restructures entire subtrees.

```rust
// Move entire engineering org under new VP
set_inherit(company, engineering, new_vp)?;
// Done. Thousands of users instantly inherit through new_vp.
```

**Key insight #2:** Subject and object are identical concepts (both u64). Inheritance works in ANY direction:

```rust
// User inherits from group (traditional)
set_inherit(doc, alice, engineering_team)?;

// Object inherits from object (folder → doc)
set_inherit(access_policy, doc, folder)?;
set_inherit(access_policy, folder, workspace)?;

// Resource inherits from resource class
set_inherit(schema, specific_api, api_class)?;

// Any entity can inherit from any other entity
// There is no type system - just IDs and relationships
```

This unified model means:
- Users can inherit from users, groups, roles, or resources
- Resources can inherit from resource hierarchies
- Permission templates can be defined as entities and inherited
- No artificial subject/object distinction limits your model

### Comparison

| Aspect | Zanzibar | Capbit |
|--------|----------|--------|
| Graph shape | Arbitrary | DAG (sufficient for auth) |
| Bulk restructure | Update many tuples | Single inheritance link |
| Propagation | Recompute derived relations | Instant (direct storage) |
| Traversal direction | Multi-directional | Upward (correct for inheritance) |

**Winner: Capbit** — Bulk restructuring with single operation, instant propagation.

---

## 3. Consistency Model (Engine-Level)

**Important note:** Zanzibar's "zookies" and caching discussions in the original paper relate to *distributed* consistency (cross-region replication). That's a deployment layer concern, not an engine property. Here we compare the engines themselves.

### Engine-Level Consistency

Both engines, when queried locally, read from their storage backend:

**Zanzibar engine:** Queries Spanner (or equivalent) for relation tuples, traverses graph.

**Capbit engine:** Queries LMDB for bitmasks, traverses inheritance chain.

At the local level, both provide consistent reads from their respective stores. The difference is in the *model*:

### Model Comparison

| Aspect | Zanzibar | Capbit |
|--------|----------|--------|
| Update model | Write tuple, invalidate computed relations | Write mask, instantly visible |
| Computed permissions | Derived from graph (may need recomputation) | Direct bitmask (no derivation) |
| Inheritance update | Update tuples for affected relations | Single link change, computed at read |

### Capbit's Advantage: No Derived State

Capbit stores permissions directly as bitmasks. There's no "computed" permission that could be stale:

```rust
fn resolve(d: &Dbs, tx: &RoTxn, mut s: u64, o: u64) -> Result<u64> {
    let mut mask = 0u64;
    for _ in 0..=10 {
        let role = d.caps.get(tx, s, o)?;
        mask |= if role == 0 { 0 } else { d.roles.get(tx, &key(o, role))?.unwrap_or(role) };
        match d.inh.get(tx, &key(o, s))? { Some(p) => s = p, None => break }
    }
    Ok(mask)
}
```

Every check computes the current mask from current data. No caching layer needed at the engine level.

### Distributed Consistency (See Section 5)

Zanzibar's zookies, cache invalidation, and eventual consistency are concerns of the *distributed deployment*, not the engine. Any authorization engine (including Capbit) would need similar mechanisms if deployed globally. This is addressed in Section 5.

**Winner: Capbit** — Simpler model with no derived/cached state at engine level.

---

## 4. Performance (Engine-Level Comparison)

**Important note:** A fair performance comparison must compare engines at the same level. Comparing Capbit's local latency to Zanzibar's network latency is apples to oranges. Below we compare the *engine* characteristics.

### Capbit Engine (Benchmarks on mobile ARM64)

```
Single check latency:     2-3 µs
Batch grant throughput:   200-300K/sec
Inheritance depth 10:     ~17 µs
Concurrent reads (8 threads): 2.1M checks/sec
1M grants setup:          ~5 seconds
```

### Engine-Level Comparison

| Metric | Zanzibar Engine | Capbit Engine |
|--------|-----------------|---------------|
| Check operation | Graph traversal + multiple lookups | Single bitmask AND |
| Permission evaluation | Boolean per relation | 64 bits in O(1) |
| Storage backend | Spanner (distributed) | LMDB (embedded) |
| Dependencies | Heavy infrastructure | Single library |

The key insight: Capbit's bitmask model is fundamentally cheaper to evaluate than Zanzibar's graph traversal model, *at the engine level*.

**Winner: Capbit** — Simpler evaluation model, fewer operations per check.

---

## 5. Global Distribution: An Orthogonal Concern

A common misconception is that Zanzibar's distributed architecture is an inherent advantage. In reality, **global distribution is a deployment layer, not a model property**.

### The Layered Architecture View

```
┌─────────────────────────────────────┐
│     Distribution Layer (optional)   │  ← Replication, consensus, routing
├─────────────────────────────────────┤
│         Authorization Engine        │  ← Permission model, evaluation
├─────────────────────────────────────┤
│           Storage Layer             │  ← Persistence, transactions
└─────────────────────────────────────┘
```

Zanzibar bundles all three layers into one system. Capbit focuses on the engine layer, leaving distribution as a separate concern.

### Adding Distribution to Capbit

Capbit can be distributed using standard techniques:

**Option 1: Read replicas with async replication**
```
┌──────────┐     ┌──────────┐     ┌──────────┐
│ Capbit   │────▶│ Capbit   │────▶│ Capbit   │
│ Primary  │     │ Replica  │     │ Replica  │
│ (writes) │     │ (reads)  │     │ (reads)  │
└──────────┘     └──────────┘     └──────────┘
```

**Option 2: Raft/Paxos consensus for strong consistency**
```
┌──────────┐     ┌──────────┐     ┌──────────┐
│ Capbit   │◀───▶│ Capbit   │◀───▶│ Capbit   │
│ Node 1   │     │ Node 2   │     │ Node 3   │
└──────────┘     └──────────┘     └──────────┘
     Raft consensus layer
```

**Option 3: Edge deployment with eventual consistency**
```
┌─────────────────────────────────────────────┐
│              Central Capbit                 │
│            (source of truth)                │
└──────────────────┬──────────────────────────┘
                   │ sync
    ┌──────────────┼──────────────┐
    ▼              ▼              ▼
┌────────┐    ┌────────┐    ┌────────┐
│ Edge   │    │ Edge   │    │ Edge   │
│ Capbit │    │ Capbit │    │ Capbit │
└────────┘    └────────┘    └────────┘
```

### Why Separation Matters

| Aspect | Bundled (Zanzibar) | Separated (Capbit) |
|--------|--------------------|--------------------|
| Deployment flexibility | One size fits all | Choose your topology |
| Overhead for single-node | Full distributed stack | Zero overhead |
| Edge/embedded use | Not possible | Native |
| Upgrade distribution layer | Tied to auth system | Independent |

**Conclusion:** Zanzibar's distribution is not a capability Capbit lacks—it's a deployment choice. Capbit's lean engine can be wrapped in any distribution layer appropriate for your scale.

---

## 6. Operational Simplicity

### A Note on Size Comparisons

Zanzibar's codebase size is largely a consequence of bundling the distribution layer (Spanner integration, cache invalidation, multi-region replication) with the authorization engine. Comparing total codebase sizes directly would be misleading.

The meaningful comparison is: **because Capbit decouples the authorization engine from distribution, it can be embedded extremely cheaply.** If you need distribution, you add it as a separate layer (see Section 5). If you don't, you pay zero overhead.

### Zanzibar

Requires:
- Distributed database (Spanner or equivalent)
- Multiple service replicas
- Cache layer
- Schema management
- Monitoring and alerting
- Team to operate

### Capbit

Requires:
- One file (LMDB database)
- ~280 lines of authorization logic
- Embed directly in your application

```rust
// Complete setup
capbit::init("/path/to/db")?;
capbit::grant(alice, doc, READ | WRITE)?;
capbit::check(alice, doc, READ)?;
```

### Comparison

| Aspect | Zanzibar | Capbit |
|--------|----------|--------|
| Infrastructure | Distributed system | **Single file** |
| Decoupling | Bundled with distribution | **Auth logic only (~280 lines)** |
| Operations | Team required | **Zero-ops** |
| Embedding | Not practical | **Native** |

**Winner: Capbit** — Embeddable, zero operational overhead.

---

## 7. Extensibility

### Capbit's Extension Points

**Wider permission masks:**
```rust
// Trivial change from u64 to u128 or wider
type Mask = u128;  // Now 128 permission bits
```

**Unbounded inheritance:**
```rust
// Change loop limit or remove entirely
for _ in 0..=100 {  // or: loop {
```

**Conditional roles:**
```rust
// Role IDs can encode conditions
const WEEKDAY_ONLY: u64 = 1 << 32 | base_role;
const GEO_RESTRICTED: u64 = 2 << 32 | base_role;

// Extended resolve() checks conditions
```

**Custom resolution logic:**
```rust
// Fork resolve() for domain-specific logic
fn custom_resolve(s: u64, o: u64, context: &Context) -> Result<u64> {
    let base = resolve(s, o)?;
    if context.is_weekend && requires_weekday(base) {
        return Ok(base & !WEEKDAY_BITS);
    }
    Ok(base)
}
```

---

## 8. System-Scoped Permissions

### The Problem with Global Permission Bits

In many authorization systems, certain permission bits have **global meaning**. If `ADMIN` means "system administrator" everywhere, users can't freely use that bit on their own objects without accidentally triggering system-level checks.

### Capbit's Solution: The `_system` Object

All system permission checks happen in the context of a special `_system` object:

```rust
// Bootstrap creates _system and _root_user
let (system, root_user) = bootstrap()?;

// root_user has all bits on _system
check(root_user, system, u64::MAX)?;  // true
```

Protected operations check the **actor's permissions on `_system`**, not on the target object:

```rust
// This checks: does actor have GRANT on _system?
protected_grant(actor, subject, object, mask)?;

// NOT: does actor have GRANT on object?
```

### User Freedom

Users can use **any bit** on their own objects without system interference:

```rust
const MY_ADMIN: u64 = 1 << 63;  // Same bit value as ADMIN

// Alice uses "ADMIN" bit on her document - system doesn't care
grant(alice, my_doc, MY_ADMIN)?;
check(alice, my_doc, MY_ADMIN)?;  // true

// System only cares about permissions on _system
protected_grant(alice, bob, other_doc, READ)?;  // Fails - alice has no GRANT on _system
```

### Permission Delegation

Root can delegate system permissions to operators:

```rust
let (system, root_user) = bootstrap()?;
let operator = create_entity("operator")?;

// Grant operator the ability to manage permissions (but not roles)
protected_grant(root_user, operator, system, GRANT)?;

// Now operator can use protected_grant/protected_revoke
protected_grant(operator, alice, doc, READ)?;  // Works

// But not protected_set_role (needs ADMIN on _system)
protected_set_role(operator, doc, 1, READ)?;  // Fails
```

### Comparison

| Aspect | Global Permissions | Scoped to `_system` |
|--------|-------------------|---------------------|
| Bit meaning | Fixed globally | Context-dependent |
| User freedom | Limited by reserved bits | Full 64 bits available |
| Permission checks | Implicit/magical | Explicit against `_system` |
| Delegation | Often all-or-nothing | Fine-grained (GRANT vs ADMIN vs VIEW) |

---

## 9. When to Use Each

### Use Zanzibar When:
- You're Google-scale (trillions of ACLs)
- You need multi-region global distribution
- You have dedicated infrastructure teams
- Your permission model is well-defined and stable

### Use Capbit When:
- You need sub-millisecond authorization
- You want embedded, in-process checks
- Your permission model evolves at runtime
- You need instant permission updates
- You want zero operational overhead
- You're building edge/IoT/embedded systems
- You want simplicity without sacrificing power

---

## 10. Summary

| Dimension | Zanzibar | Capbit | Winner |
|-----------|----------|--------|--------|
| Permission flexibility | Boolean relations | 64-bit masks | **Capbit** |
| Roles per object | Schema-fixed | Unlimited | **Capbit** |
| Schema requirement | Yes | No | **Capbit** |
| Entity model | Typed namespaces | Unified u64 IDs | **Capbit** |
| Inheritance direction | Subject→Object | Any→Any | **Capbit** |
| Graph restructuring | Many updates | One change | **Capbit** |
| Update propagation | Delayed (graph recomputation) | Instant (direct storage) | **Capbit** |
| Engine complexity | Graph traversal | Bitmask AND | **Capbit** |
| Derived state | Computed from relations | None (direct masks) | **Capbit** |
| Permission bit freedom | Reserved bits | Full 64 bits for users | **Capbit** |
| System permission scope | Global/implicit | Explicit (`_system` object) | **Capbit** |
| Embedding | Bundled with distribution | ~280 lines, embeddable | **Capbit** |
| Global distribution | Bundled | Add as needed | Tie* |
| Ecosystem maturity | Established | New | Zanzibar |

*Distribution is an orthogonal deployment layer, not an engine capability (see Section 5). Zanzibar's size is largely due to bundling distribution—Capbit's decoupling enables cheap embedding.

**Conclusion:** Capbit provides a more expressive and simpler authorization model than Zanzibar for the vast majority of applications. Zanzibar's bundled distribution is not an advantage—it's overhead for anyone not operating at Google scale.

For everyone else, Capbit offers:
- **More power** (64-bit masks, unlimited roles, instant updates)
- **Simpler engine** (bitmask AND vs graph traversal)
- **Cheap embedding** (decoupled from distribution, ~280 lines of authorization logic)
- **Deployment flexibility** (add distribution only when needed)

---

## Appendix: Quick Reference

### Capbit Core API

```rust
// Initialize
init("/path/to/db")?;

// Bootstrap (creates _system and _root_user)
let (system, root) = bootstrap()?;

// Entities (human-readable names for u64 IDs)
let alice = create_entity("alice")?;        // Auto-increment ID
let doc = create_entity("quarterly-report")?;
let id = get_id_by_label("alice")?;         // Lookup by name
let name = get_label(alice)?;               // Get name from ID

// Write operations (require actor with permission on _system)
grant(actor, subject, object, mask)?;       // Requires GRANT
revoke(actor, subject, object)?;            // Requires GRANT
set_role(actor, object, role_id, mask)?;    // Requires ADMIN
set_inherit(actor, object, child, parent)?; // Requires ADMIN
list_for_object(actor, object)?;            // Requires VIEW

// Read operations (no actor needed)
check(subject, object, required)?;
get_mask(subject, object)?;
get_role(object, role_id)?;
list_for_subject(subject)?;

// Internal batch (bypasses protection)
transact(|tx| {
    tx.grant(subject, object, mask)?;
    tx.set_role(object, role, mask)?;
    tx.create_entity("name")?;
    Ok(())
})?;
```

### Permission Constants

```rust
// System capabilities (checked against _system)
pub const READ: u64    = 1;
pub const WRITE: u64   = 1 << 1;
pub const DELETE: u64  = 1 << 2;
pub const CREATE: u64  = 1 << 3;
pub const GRANT: u64   = 1 << 4;   // grant, revoke, batch_grant, batch_revoke
pub const EXECUTE: u64 = 1 << 5;
pub const VIEW: u64    = 1 << 62;  // list_for_object
pub const ADMIN: u64   = 1 << 63;  // set_role, set_inherit, remove_inherit

// On your own objects, all 64 bits are free to use
pub const MY_PUBLISH: u64 = 1 << 0;  // Reuse bit 0
pub const MY_APPROVE: u64 = 1 << 6;
```
