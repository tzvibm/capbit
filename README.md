# Capbit

Authorization as atomized data.

## Core Idea

|  | Relationships | Semantics |
|---|---|---|
| **ReBAC** | Stored | Computed (code) |
| **Zanzibar** | Atomized | Data (schema) |
| **Capbit** | Atomized | Atomized data |

**Zanzibar's insight**: Authorization semantics belong in data, not application code. It delivered by storing semantics as a schema manifest.

**Capbit's refinement**: Authorization semantics should be atomized data - independent tuples, not a schema blob. Both relationships and semantics are stored as independent, atomized tuples.

Definitions:
- **Stored**: Facts exist but joined at query time
- **Atomized**: Single tuple - queryable, mutable, and addressable independently
- **Computed**: Derived from rules at runtime
- **Data (schema)**: Stored in manifest, interpreted at runtime

## The Progression

### ReBAC

Relationships stored as facts. Semantics computed from rules.

```
Relationships (stored):
  owns(alice, doc:100)
  member_of(bob, engineering)

Semantics (computed):
  can_write(U, D) :- owns(U, D).
  can_write(U, D) :- member_of(U, G), team_access(G, D).
```

To resolve: evaluate rules against facts. Expensive.

### Zanzibar

Relationships atomized. Semantics moved from code to data (schema manifest).

```
Relationships (atomized):
  (doc:100, owner, alice)
  (doc:100, editor, bob)

Semantics (data, but schema):
  type document {
    relation owner: user
    relation editor: user
    permission write = owner + editor
  }
```

Zanzibar's win: semantics are data, not application code.
Zanzibar's limitation: schema is not atomized - must parse to query.

### Capbit

Relationships atomized. Semantics atomized. Independent tuples.

```
Relationships (atomized):
  caps[(alice, doc:100)] → EDITOR
  caps[(bob, doc:100)] → VIEWER

Semantics (atomized):
  roles[(doc:100, EDITOR)] → READ|WRITE|DELETE
  roles[(doc:100, VIEWER)] → READ

Inheritance (atomized):
  inherit[(doc:100, alice)] → admin_group
```

Capbit's delta: semantics are atomized data, not schema blob.
To resolve: two tuple lookups. No schema parsing.

## Why It Matters

|  | ReBAC | Zanzibar | Capbit |
|---|---|---|---|
| Query relationships | Expensive | Cheap | Cheap |
| Query semantics | Expensive | Expensive (schema) | Cheap |
| Mutate relationships | Rules change | Tuple write | Tuple write |
| Mutate semantics | Rules change | Schema change | Tuple write |

```rust
// Query: "What does EDITOR mean on doc:100?"
roles.get(doc_100, EDITOR)  // O(1) - it's just a tuple

// Mutate: "Make EDITOR read-only on doc:100"
roles.put(doc_100, EDITOR, READ)  // O(1) - just write a tuple

// Explain: "Why can alice write to doc:100?"
caps.get(alice, doc_100)  // → EDITOR
roles.get(doc_100, EDITOR)  // → READ|WRITE
// Two tuple lookups. No schema needed.
```

## Data Structure

```
caps:     (subject, object) → role          // relationship tuple
roles:    (object, role) → mask             // semantic tuple
inherit:  (object, child) → parent          // inheritance tuple
```

Three independent tuples. Each queryable on its own.

## Permission Resolution

```
check(alice, doc:100, WRITE):

1. caps.get(alice, doc:100) → EDITOR           // relationship lookup
2. roles.get(doc:100, EDITOR) → READ|WRITE     // semantic lookup
3. (READ|WRITE) & WRITE == WRITE               // bitmask check
```

Two tuple lookups. No schema parsing, no rule evaluation.

## Zanzibar Semantics on Capbit

Anything Zanzibar expresses can be expressed in Capbit. Zanzibar provides schema skeleton out of the box - Capbit provides independent tuples.

```rust
// Central governance: all documents share same semantics
fn create_document(actor, doc_id) {
    let template = get_type_template("document");
    for (role, mask) in template.roles {
        set_role(actor, doc_id, role, mask)?;
    }
}
```

Central governance, shared semantics, type enforcement - all buildable with simple if/else tooling.

## API

```rust
// Bootstrap
let (system, root) = bootstrap()?;

// Relationship tuples
grant(actor, subject, object, role)?;
revoke(actor, subject, object)?;

// Semantic tuples
set_role(actor, object, role, mask)?;

// Check
check(subject, object, required)?;
get_mask(subject, object)?;

// Inheritance tuples
set_inherit(actor, object, child, parent)?;
```

## License

CNCOSL - See LICENSE
