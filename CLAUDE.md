# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Authorization as atomized data. Embedded capability-based access control: u64 IDs,
u64 permission bitmasks, policy-qualified edges, groups with write-time materialized
closure, qualified delegation chains, and an atomic audit log. Backed by fjall
(LSM-tree storage). Instance-based (`Capbit::open(path)`), no global state.

## Design Thesis

All expansion happens at **write time**; every check is a **bounded number of key
reads**; every edge (grant, membership, delegation) carries a **policy qualifier**;
the system **governs itself** — granting, revoking, defining roles, and creating
objects are permission bits resolved by the same `check()` as everything else.

See `DESIGN.md` for the full architecture and rationale, `capbit_2.0.md` for the
historical design exploration it was distilled from.

## Core Concepts

- **Policy** (on every edge): `Necessary` (structural/MAC-like), `Possible`
  (discretionary/DAC-like), `Not` (explicit deny/exclusion). Composes as `min`
  along chains; `Not` absorbs. Deny bits override positive bits.
- **Groups**: a group is just an object; membership is a grant of the reserved
  `_MEMBER` role on it. Nested membership is folded into a materialized closure
  at write time — checks never traverse the group graph.
- **Delegation**: per-(subject, object, role) qualified edges. Resolution walks
  the chain (bounded, cycle-checked) until an ancestor actually holds the role;
  strength only attenuates.
- **Type-as-object**: an object's role lookups fall back through its parent chain.
  One role change on a type object repolicies every instance.
- **Audit**: every mutation appends an audit row in the same atomic batch.

## Partitions (10)

```
OBJECTS:         (object, role) → mask                 role definitions
PARENTS:         (object) → parent                     type-as-object
PARENTS_REV:     (parent, object) → 1
SUBJECTS:        (subject, object, role) → policy      grants (membership = role _MEMBER)
SUBJECTS_REV:    (object, subject, role) → policy
CLOSURE:         (member, group) → policy              materialized effective membership
CLOSURE_REV:     (group, member) → policy
DELEGATIONS:     (subject, object, role) → (parent, policy)
DELEGATIONS_REV: (object, subject, role) → (parent, policy)
AUDIT:           (seq) → (ts, actor, op, args[4])      append-only
```

## Constants

```rust
// Reserved IDs
pub const _SYSTEM: u64 = 1;
pub const _ROOT: u64 = 2;

// Reserved roles
pub const _OWNER: u64 = 1;   // default mask: u64::MAX
pub const _ADMIN: u64 = 2;   // READ_META|GRANT|REVOKE|DELEGATE|SET_PARENT + app bits
pub const _EDITOR: u64 = 3;  // READ_META + APP_READ|APP_WRITE
pub const _VIEWER: u64 = 4;  // READ_META + APP_READ
pub const _MEMBER: u64 = 5;  // granting this on a group object = membership

// Meta bits (0..16) govern capbit itself:
// _READ_META, _DEFINE, _GRANT, _REVOKE, _DELEGATE,
// _CREATE_OBJ, _DELETE_OBJ, _SET_PARENT, _AUDIT
// App bits (16..64): app_bit(n), conventions APP_READ/APP_WRITE/APP_DELETE
```

## API (all methods on a `Capbit` instance)

```rust
let cb = Capbit::open("data_path")?;          // or open_with(path, Options { durable })
let (sys, root) = cb.bootstrap()?;            // once: _SYSTEM roles + _ROOT owner

// Object lifecycle (self-governing: _CREATE_OBJ checked on _SYSTEM,
// creator becomes owner atomically)
cb.create_object(actor, obj)?;
cb.delete_object(actor, obj)?;                // cascades everything
cb.set_parent(actor, obj, type_obj)?;         // type-as-object
cb.clear_parent(actor, obj)?;
cb.get_parent(actor, obj)?;

// Role definitions
cb.define_role(actor, obj, role, mask)?;      // _DEFINE; Exists if defined
cb.update_role(actor, obj, role, mask)?;
cb.delete_role(actor, obj, role)?;            // cascades grants + delegations
cb.get_role(actor, obj, role)?;
cb.list_roles(actor, obj)?;
cb.role_mask(obj, role)?;                     // effective, follows parent chain

// Grants (policy-qualified; granting undefined roles is rejected)
cb.grant(actor, sub, obj, role, Policy::Necessary)?;
cb.revoke(actor, sub, obj, role)?;
cb.check_subject(sub, obj, role)?;            // → Option<Policy>
cb.list_roles_for(actor, sub, obj)?;
cb.list_grants(actor, sub)?;
cb.list_subjects(actor, obj, after, limit)?;  // paginated, compressed (groups as rows)

// Groups (membership = grant of _MEMBER; closure maintained atomically)
cb.add_member(actor, member, group, policy)?; // Policy::Not = exclusion
cb.remove_member(actor, member, group)?;
cb.groups_of(actor, ent)?;                    // effective groups (closure)
cb.members_of(actor, group, after, limit)?;   // effective members, paginated
cb.population_intersect(actor, a, b)?;        // set algebra over members
cb.population_subtract(actor, a, b)?;

// Delegation (attenuating chains)
cb.delegate(actor, sub, obj, role, parent, policy)?;
cb.undelegate(actor, sub, obj, role)?;
cb.get_delegation(actor, sub, obj, role)?;
cb.list_delegations_on(actor, obj)?;

// Resolution (no actor required)
cb.check(sub, obj, required)?;                // flat bool
cb.get_mask(sub, obj)?;                       // flat mask, post-deny
cb.get_masks(sub, obj)?;                      // Masks { necessary, possible, denied }

// Audit
cb.audit_read(actor, after_seq, limit)?;      // _AUDIT on _SYSTEM

// Utility (dev/test)
cb.clear(actor)?;                             // _DELETE_OBJ on _SYSTEM; wipes all
```

## Errors

`Error::{Denied, Exists, NotFound, Cycle, Storage(String)}` — typed, matchable.

## Web UI

```bash
cargo run --features ui --bin ui
```

Opens at http://localhost:3000 (binds 127.0.0.1). Dev/test harness only — the
actor comes from the request body; production embeddings must derive the actor
from an authenticated principal.

## Testing

```bash
cargo test
```

Tests are instance-based on tempdirs and run in parallel.

## License

PolyForm Noncommercial 1.0.0 - no commercial use without separate license.
