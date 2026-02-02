# CLAUDE.md

Project context for Claude Code.

## What is Capbit?

Authorization as first-class data.

## Core Framing

|  | Relationships | Semantics |
|---|---|---|
| **ReBAC** | Stored | Computed |
| **Zanzibar** | Atomized | Coupled |
| **Capbit** | Atomized | Atomized |

- **Stored**: Facts exist but joined at query time
- **Atomized**: Single tuple, queryable as one unit
- **Computed**: Derived from rules
- **Coupled**: Tied to schema

Summary:
- **ReBAC**: Relationships stored, semantics computed (expensive)
- **Zanzibar**: Relationships atomized, semantics coupled to schema
- **Capbit**: Both atomized into independent tuples

## Data Structure

```
caps:     (subject, object) → role          // relationship tuple
roles:    (object, role) → mask             // semantic tuple
inherit:  (object, child) → parent          // inheritance tuple
```

Three independent tuples. Each queryable on its own.

## Permission Check Flow

```rust
fn check(subject, object, required) -> bool {
    let role = caps.get(subject, object);      // relationship tuple
    let mask = roles.get(object, role);        // semantic tuple
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

// Relationship tuples
grant(actor, subject, object, role)       // GRANT required on object
revoke(actor, subject, object)            // GRANT required on object

// Semantic tuples
set_role(actor, object, role, mask)       // ADMIN required on _system

// Inheritance tuples
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
