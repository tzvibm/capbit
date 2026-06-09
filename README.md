# Capbit

Authorization as atomized data.

Embedded capability-based access control: u64 IDs, u64 permission bitmasks,
policy-qualified edges, groups with write-time materialized closure, qualified
delegation chains, and an audit log written atomically with every mutation.
Backed by [fjall](https://github.com/fjall-rs/fjall) (LSM-tree storage).

## Core Idea

|  | Relationships | Semantics |
|---|---|---|
| **ReBAC** | Stored | Computed (code) |
| **Zanzibar** | Atomized | Data (schema) |
| **Capbit** | Atomized | Atomized data |

**Zanzibar's insight**: authorization semantics belong in data, not application
code. It delivered by storing semantics as a schema manifest.

**Capbit's refinement**: semantics should be atomized data — independent tuples,
not a schema blob. There is no schema language and no policy evaluator: a check
reads tuples, ORs some u64s, and does one AND.

**The thesis**: all expansion happens at write time; every check is a bounded
number of key reads; every edge carries a policy qualifier; and the system
governs itself — granting, revoking, defining roles, and creating objects are
permission bits checked by the same resolution as everything else.

See [DESIGN.md](DESIGN.md) for the full architecture, complexity analysis, and
an honest comparison with SpiceDB/OpenFGA/Cedar.

## Quick Start

```rust
use capbit::*;

let cb = Capbit::open("data")?;
let (sys, root) = cb.bootstrap()?;            // _SYSTEM + _ROOT, once

// Objects are self-governing: creator becomes owner atomically.
cb.create_object(root, 50)?;                  // a document
cb.create_object(root, 90)?;                  // a group

// Direct grant, policy-qualified.
cb.grant(root, 10, 50, _VIEWER, Policy::Necessary)?;
assert!(cb.check(10, 50, APP_READ)?);

// Groups: membership is just a grant of _MEMBER on the group object.
cb.add_member(root, 11, 90, Policy::Necessary)?;
cb.grant(root, 90, 50, _EDITOR, Policy::Necessary)?;
assert!(cb.check(11, 50, APP_WRITE)?);        // through the group, no traversal

// Exclusion: "everyone in the group except 12" is one tuple.
cb.add_member(root, 12, 90, Policy::Not)?;

// Delegation that can only attenuate, centrally revocable, queryable.
cb.delegate(root, 13, 50, _EDITOR, 11, Policy::Possible)?;
let m = cb.get_masks(13, 50)?;                // { necessary, possible, denied }

// Audit: every mutation was logged atomically.
let log = cb.audit_read(root, None, 100)?;
```

## Data Structure

```
OBJECTS:         (object, role) → mask                 role definitions
PARENTS:         (object) → parent                     type-as-object fallback
PARENTS_REV:     (parent, object) → 1
SUBJECTS:        (subject, object, role) → policy      grants (membership = _MEMBER)
SUBJECTS_REV:    (object, subject, role) → policy
CLOSURE:         (member, group) → policy              materialized membership
CLOSURE_REV:     (group, member) → policy
DELEGATIONS:     (subject, object, role) → (parent, policy)
DELEGATIONS_REV: (object, subject, role) → (parent, policy)
AUDIT:           (seq) → (ts, actor, op, args)         append-only
```

Ten partitions. Every audit question is a prefix scan; every mutation is one
atomic batch. Implementable on any ordered-key store.

## Policies

Every edge — grant, membership, delegation — carries a strength:

```
Necessary   structural, mandatory (MAC-like)
Possible    discretionary, conditional (DAC-like)
Not         explicit deny / exclusion — overrides everything
```

Composition along any chain is `min`: access can only weaken through
indirection, never strengthen. Resolution returns three buckets
(`necessary`, `possible`, `denied`); denied bits override.

## Permission Bits

Bits 0..16 govern capbit itself (`_GRANT`, `_REVOKE`, `_DEFINE`, `_DELEGATE`,
`_CREATE_OBJ`, ...). Bits 16..64 are the application's (`app_bit(n)`). The two
spaces are disjoint by construction.

```rust
pub const APP_READ:  u64 = app_bit(0);
pub const APP_WRITE: u64 = app_bit(1);
```

## Groups Without Expansion

A group is an object; membership is a grant. Nested membership is folded into a
materialized closure at **write** time, so checks never walk the group graph —
the cost Zanzibar pays on every check, capbit pays once per membership change.

Reverse queries return the compressed answer (groups as rows) with on-demand,
paginated expansion, plus set algebra over populations:

```rust
cb.members_of(actor, eng, None, 1000)?;            // effective members, paginated
cb.population_intersect(actor, prod_access, no_mfa)?;  // compliance as a primitive
cb.population_subtract(actor, eng, contractors)?;
```

## Web UI

```bash
cargo run --features ui --bin ui
```

Opens at http://localhost:3000 — a dev/test harness for every API operation.
It takes the actor from the request body; real embeddings must derive the actor
from an authenticated principal.

## Testing

```bash
cargo test
```

## License

[PolyForm Noncommercial 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/) - See LICENSE
