# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Authorization as first-class data. Both relationships and authorization stored as indexed tuples.

## Core Framing

|  | Relationships | Authorization |
|---|---|---|
| **ReBAC** | Atomic | Computed |
| **Zanzibar** | Atomic | Encoded |
| **Capbit** | Atomic | Atomic |

|  | ReBAC | Zanzibar | Capbit |
|---|---|---|---|
| Query permissions | No | No | Yes |
| Mutate permissions | No | No | Yes |
| Explain permissions | No | No | Yes |

Capbit makes authorization first-class data.

## Data Structure

```
caps:     (subject, object) → role_id       // relationships (atomic)
roles:    (object, role_id) → mask          // authorization (atomic)
inherit:  (object, child) → parent          // inheritance (atomic)
```

Key insight: `roles` is keyed by `(object, role_id)`, so authorization is per-object indexed data, not schema.

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
