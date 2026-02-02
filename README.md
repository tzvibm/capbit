# Capbit

Authorization as first-class data.

## Core Idea

|  | Relationships | Authorization |
|---|---|---|
| **ReBAC** | Atomic | Computed |
| **Zanzibar** | Atomic | Encoded |
| **Capbit** | Atomic | Atomic |

All three store relationships as indexed tuples.

The difference is authorization:

- **ReBAC**: Computes authorization from rules. Relationships are data, permissions are not.
- **Zanzibar**: Encodes authorization in schema blob. Expressive but not queryable as data.
- **Capbit**: Stores authorization as indexed tuples. Permissions are data.

Capbit makes authorization first-class data.

## Why It Matters

|  | ReBAC | Zanzibar | Capbit |
|---|---|---|---|
| Query permissions | No | No | Yes |
| Mutate permissions | No | No | Yes |
| Explain permissions | No | No | Yes |

**ReBAC** can answer "who is related to whom?" but not "what permissions exist?" - because permissions are computed, not represented.

**Zanzibar** encodes permissions in schema (expressive) but you can't query or mutate them as data.

**Capbit** stores permissions as indexed tuples. You can:

```rust
// Query: "What does EDITOR mean on doc:100?"
roles.get(doc_100, EDITOR)  // O(1)

// Mutate: "Make EDITOR read-only on doc:100"
roles.put(doc_100, EDITOR, READ)  // O(1)

// Explain: "Why can alice write to doc:100?"
caps.get(alice, doc_100)  // → EDITOR
roles.get(doc_100, EDITOR)  // → READ|WRITE
// Because alice has EDITOR role, and EDITOR means READ|WRITE on this object
```

## Data Structure

```
caps:     (subject, object) → role_id       // relationships (atomic)
roles:    (object, role_id) → mask          // authorization (atomic)
inherit:  (object, child) → parent          // inheritance (atomic)
```

Both relationships and authorization are indexed data.

## Permission Resolution

```
check(alice, doc:100, WRITE):

1. caps.get(alice, doc:100) → EDITOR           // relationship lookup
2. roles.get(doc:100, EDITOR) → READ|WRITE     // authorization lookup
3. (READ|WRITE) & WRITE == WRITE               // bitmask check
```

Two index lookups. No schema parsing, no rule evaluation.

## Trade-offs

| | Zanzibar | Capbit |
|---|---|---|
| Authorization storage | Encoded (schema) | Atomic (index) |
| Expressiveness | Yes | Yes |
| Query/mutate/explain | No | Yes |
| Shared semantics | Automatic | Manual |
| Central governance | Yes | No |

Zanzibar trades atomicity for central governance.
Capbit trades central governance for atomicity.

## API

```rust
// Bootstrap
let (system, root) = bootstrap()?;

// Relationships
grant(actor, subject, object, role_id)?;
revoke(actor, subject, object)?;

// Authorization
set_role(actor, object, role_id, mask)?;

// Check
check(subject, object, required)?;
get_mask(subject, object)?;

// Inheritance
set_inherit(actor, object, child, parent)?;
```

## License

CNCOSL - See LICENSE
