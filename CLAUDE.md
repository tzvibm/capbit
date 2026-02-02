# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Authorization as first-class data.

Zanzibar atomized relationships.
Capbit atomizes the rest.

## Core Framing

|  | Relationships | Semantics |
|---|---|---|
| **ReBAC** | Computed | Computed |
| **Zanzibar** | Atomic | Blobbed |
| **Capbit** | Atomic | Atomic |

- **ReBAC**: Everything computed from rules (limited, expensive)
- **Zanzibar**: Relationships atomic, semantics blobbed (expressive, expensive to query)
- **Capbit**: Everything atomic (expressive, cheap)

Anything Zanzibar expresses can be expressed in Capbit. Zanzibar provides schema skeleton out of the box - Capbit provides primitives and you build the skeleton (simple if/else tooling).

## Data Structure

```
caps:     (subject, object) → role          // relationships (atomic)
roles:    (object, role) → mask             // semantic relationships (atomic)
inherit:  (object, child) → parent          // inheritance relationships (atomic)
```

All relationships. All atomic. Same storage.

## Permission Check Flow

```rust
fn check(subject, object, required) -> bool {
    let role = caps.get(subject, object);      // relationship
    let mask = roles.get(object, role);        // semantic relationship
    mask & required == required
}
```

With inheritance:
```rust
fn get_mask(subject, object) -> u64 {
    let mut mask = 0;
    let mut current = subject;
    loop {
        let role = caps.get(current, object);
        mask |= roles.get(object, role);
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

// Relationships
grant(actor, subject, object, role)       // GRANT required on object
revoke(actor, subject, object)            // GRANT required on object

// Semantic relationships
set_role(actor, object, role, mask)       // ADMIN required on _system

// Inheritance relationships
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
