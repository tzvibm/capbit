# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Authorization as first-class data. Both role assignments and role semantics stored as indexed tuples.

## Core Framing

|  | Role Assignments | Role Semantics |
|---|---|---|
| **ReBAC** | Atomic | Computed |
| **Zanzibar** | Atomic | Encoded |
| **Capbit** | Atomic | Atomic |

|  | ReBAC | Zanzibar | Capbit |
|---|---|---|---|
| Query assignments | Cheap | Cheap | Cheap |
| Define semantics | Limited | Expressive | Expressive |
| Query semantics | Expensive | Expensive | Cheap |
| Mutate semantics | Rules change | Schema change | Data write |

Capbit makes role semantics first-class data.

Anything Zanzibar expresses can be expressed in Capbit. Zanzibar provides schema skeleton out of the box - Capbit provides primitives and you build the skeleton (simple if/else tooling).

## Data Structure

```
caps:     (subject, object) → role_id       // role assignments (atomic)
roles:    (object, role_id) → mask          // role semantics (atomic)
inherit:  (object, child) → parent          // inheritance (atomic)
```

Key insight: `roles` is keyed by `(object, role_id)`, so semantics are per-object indexed data, not schema.

## Permission Check Flow

```rust
fn check(subject, object, required) -> bool {
    let role_id = caps.get(subject, object);
    let mask = roles.get(object, role_id);
    mask & required == required
}
```

With inheritance:
```rust
fn get_mask(subject, object) -> u64 {
    let mut mask = 0;
    let mut current = subject;
    loop {
        let role_id = caps.get(current, object);
        mask |= roles.get(object, role_id);
        match inherit.get(object, current) {
            Some(parent) => current = parent,
            None => break,
        }
    }
    mask
}
```

## API

```rust
// Bootstrap
bootstrap() -> (system_id, root_user_id)

// Write (require actor with permission)
grant(actor, subject, object, role_id)    // GRANT required on object
revoke(actor, subject, object)            // GRANT required on object
set_role(actor, object, role, mask)       // ADMIN required on _system
set_inherit(actor, object, child, parent) // ADMIN required on _system

// Read (no actor)
check(subject, object, required) -> bool
get_mask(subject, object) -> u64
get_role(object, role) -> u64

// Internal (bypasses protection)
transact(|tx| { tx.grant(...); Ok(()) })
```

## Testing

```bash
cargo test -- --test-threads=1
```
