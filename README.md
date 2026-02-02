# Capbit

Authorization as first-class data.

## Core Idea

All three have the base relationship: `(subject, role, object)`

The difference is how role meaning and inheritance are stored.

|  | Relationships | Semantic Relationships |
|---|---|---|
| **ReBAC** | Atomic | Computed |
| **Zanzibar** | Atomic | Relationships, but blobbed |
| **Capbit** | Atomic | Atomic |

- **ReBAC**: Role meaning computed from rules. Limited expressiveness, expensive to query.
- **Zanzibar**: Role meaning encoded as relationships in schema blob. Expressive but expensive to query.
- **Capbit**: Role meaning stored as atomic relationships. Expressive and cheap to query.

Zanzibar's insight: semantics are relationships too.
Capbit's insight: store them the same way.

## The Progression

### ReBAC

```
Relationships (atomic):
  (alice, owner, doc:100)
  (alice, member, engineering)

Semantics (computed):
  can_write(U, D) :- owns(U, D).
  can_write(U, D) :- member_of(U, G), team_access(G, D).
```

Relationships are data. Semantics are code.

### Zanzibar

```
Relationships (atomic):
  (doc:100, owner, alice)
  (doc:100, editor, bob)

Semantics (relationships, but blobbed):
  type document {
    relation owner: user
    relation editor: user
    permission write = owner + editor
  }
```

Relationships are data. Semantics are relationships too - but stored as schema blob.

### Capbit

```
Relationships (atomic):
  caps[(alice, doc:100)] → EDITOR
  caps[(bob, doc:100)] → VIEWER

Semantics (atomic):
  roles[(doc:100, EDITOR)] → READ|WRITE|DELETE
  roles[(doc:100, VIEWER)] → READ

Inheritance (atomic):
  inherit[(doc:100, alice)] → admin_group
```

All relationships. All atomic. Same storage.

## Why It Matters

|  | ReBAC | Zanzibar | Capbit |
|---|---|---|---|
| Query relationships | Cheap | Cheap | Cheap |
| Define semantics | Limited | Expressive | Expressive |
| Query semantics | Expensive | Expensive | Cheap |
| Mutate semantics | Rules change | Schema change | Data write |

```rust
// Query: "What does EDITOR mean on doc:100?"
roles.get(doc_100, EDITOR)  // O(1)

// Mutate: "Make EDITOR read-only on doc:100"
roles.put(doc_100, EDITOR, READ)  // O(1)

// Explain: "Why can alice write to doc:100?"
caps.get(alice, doc_100)  // → EDITOR
roles.get(doc_100, EDITOR)  // → READ|WRITE
// Two lookups, cheap.
```

## Data Structure

```
caps:     (subject, object) → role          // relationships (atomic)
roles:    (object, role) → mask             // semantic relationships (atomic)
inherit:  (object, child) → parent          // inheritance relationships (atomic)
```

## Permission Resolution

```
check(alice, doc:100, WRITE):

1. caps.get(alice, doc:100) → EDITOR           // relationship lookup
2. roles.get(doc:100, EDITOR) → READ|WRITE     // semantic lookup
3. (READ|WRITE) & WRITE == WRITE               // bitmask check
```

Two index lookups. No schema parsing, no rule evaluation.

## Zanzibar Semantics on Capbit

Anything Zanzibar expresses can be expressed in Capbit. Zanzibar provides schema skeleton out of the box - Capbit provides primitives.

```rust
// Central governance: all documents share same semantic relationships
fn create_document(actor, doc_id) {
    let template = get_type_template("document");
    for (role, mask) in template.roles {
        set_role(actor, doc_id, role, mask)?;
    }
}
```

Central governance, shared semantics, type enforcement - all buildable on atomic primitives with simple if/else tooling.

## API

```rust
// Bootstrap
let (system, root) = bootstrap()?;

// Relationships
grant(actor, subject, object, role)?;
revoke(actor, subject, object)?;

// Semantic relationships
set_role(actor, object, role, mask)?;

// Check
check(subject, object, required)?;
get_mask(subject, object)?;

// Inheritance
set_inherit(actor, object, child, parent)?;
```

## License

CNCOSL - See LICENSE
