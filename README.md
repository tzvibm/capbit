# Capbit

Authorization as first-class data.

## Core Idea

Zanzibar stored relationships. Capbit decouples them from schema.

|  | Relationships | Semantics |
|---|---|---|
| **ReBAC** | Stored | Computed |
| **Zanzibar** | Atomic tuple | Coupled to schema |
| **Capbit** | Atomic tuple | Atomic tuple |

## The Progression

### ReBAC

Relationships exist but semantic relationships are computed.

```
Stored:
  (alice, owner, doc:100)
  (bob, member, engineering)

Computed (rules):
  can_write(U, D) :- owns(U, D).
  can_write(U, D) :- member_of(U, G), team_access(G, D).
```

To get full expressiveness, need to compute:
- All roles per subject
- All roles per object
- All permissions per role per object
- All users per group

Expensive. Complex queries require rule evaluation.

### Zanzibar

Made (subject, role, object) an atomic tuple. Relationships stored, not computed.

```
Relationship tuples (atomic):
  (doc:100, owner, alice)
  (doc:100, editor, bob)

Schema blob (coupled):
  type document {
    relation owner: user
    relation editor: user
    permission write = owner + editor
  }
```

Relationship tuple is atomic - but coupled to schema for meaning. You need both to resolve.

### Capbit

Three separate atomic tuples. Each stands alone.

```
Relationships (atomic):
  caps[(alice, doc:100)] → EDITOR
  caps[(bob, doc:100)] → VIEWER

Semantics (atomic, separate):
  roles[(doc:100, EDITOR)] → READ|WRITE|DELETE
  roles[(doc:100, VIEWER)] → READ

Inheritance (atomic, separate):
  inherit[(doc:100, alice)] → admin_group
```

Semantic relationships are their own tuples, not embedded in schema.

## Why It Matters

|  | ReBAC | Zanzibar | Capbit |
|---|---|---|---|
| Query relationships | Expensive | Cheap | Cheap |
| Query semantics | Expensive | Expensive (parse schema) | Cheap |
| Mutate semantics | Rules change | Schema change | Data write |
| Coupling | Computed | Tuple + schema | Decoupled |

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
roles:    (object, role) → mask             // semantic tuple (separate!)
inherit:  (object, child) → parent          // inheritance tuple (separate!)
```

All tuples. All atomic. All decoupled.

## Permission Resolution

```
check(alice, doc:100, WRITE):

1. caps.get(alice, doc:100) → EDITOR           // relationship lookup
2. roles.get(doc:100, EDITOR) → READ|WRITE     // semantic lookup
3. (READ|WRITE) & WRITE == WRITE               // bitmask check
```

Two tuple lookups. No schema parsing, no rule evaluation.

## Zanzibar Semantics on Capbit

Anything Zanzibar expresses can be expressed in Capbit. Zanzibar provides schema skeleton out of the box - Capbit provides decoupled primitives.

```rust
// Central governance: all documents share same semantics
fn create_document(actor, doc_id) {
    let template = get_type_template("document");
    for (role, mask) in template.roles {
        set_role(actor, doc_id, role, mask)?;
    }
}
```

Central governance, shared semantics, type enforcement - all buildable on decoupled tuples with simple if/else tooling.

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
