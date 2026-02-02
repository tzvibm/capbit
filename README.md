# Capbit

Authorization as first-class data.

## Core Idea

|  | Role Assignments | Role Semantics |
|---|---|---|
| **ReBAC** | Atomic | Computed |
| **Zanzibar** | Atomic | Encoded |
| **Capbit** | Atomic | Atomic |

All three store role assignments (who has what role) as indexed tuples.

The difference is role semantics (what roles mean):

- **ReBAC**: Computes from rules. Limited expressiveness.
- **Zanzibar**: Encodes in schema blob. Expressive but expensive to query.
- **Capbit**: Stores as indexed tuples. Expressive and cheap to query.

Capbit makes role semantics first-class data.

## Why It Matters

|  | ReBAC | Zanzibar | Capbit |
|---|---|---|---|
| Query assignments | Cheap | Cheap | Cheap |
| Mutate assignments | Cheap | Cheap | Cheap |
| Define semantics | Limited | Expressive | Expressive |
| Query semantics | Expensive | Expensive | Cheap |
| Mutate semantics | Rules change | Schema change | Data write |

**ReBAC** can query semantics (evaluate rules) but it's expensive and expressiveness is limited.

**Zanzibar** can query semantics (parse schema) but it's expensive. Expressiveness is good.

**Capbit** can query semantics cheaply. It's just an index lookup:

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
caps:     (subject, object) → role_id       // role assignments (atomic)
roles:    (object, role_id) → mask          // role semantics (atomic)
inherit:  (object, child) → parent          // inheritance (atomic)
```

Both assignments and semantics are indexed data.

## Permission Resolution

```
check(alice, doc:100, WRITE):

1. caps.get(alice, doc:100) → EDITOR           // assignment lookup
2. roles.get(doc:100, EDITOR) → READ|WRITE     // semantics lookup
3. (READ|WRITE) & WRITE == WRITE               // bitmask check
```

Two index lookups. No schema parsing, no rule evaluation.

## Zanzibar Semantics on Capbit

Anything Zanzibar expresses can be expressed in Capbit. The difference is Zanzibar provides schema skeleton out of the box - Capbit provides primitives.

```rust
// Zanzibar: "all documents share editor semantics" (built-in)
// Capbit: implement as tooling

fn create_document(actor, doc_id) {
    // Copy role definitions from document type template
    let template = get_type_template("document");
    for (role, mask) in template.roles {
        set_role(actor, doc_id, role, mask)?;
    }
}
```

Central governance, shared semantics, type enforcement - all buildable on atomic primitives.

## API

```rust
// Bootstrap
let (system, root) = bootstrap()?;

// Role assignments
grant(actor, subject, object, role_id)?;
revoke(actor, subject, object)?;

// Role semantics
set_role(actor, object, role_id, mask)?;

// Check
check(subject, object, required)?;
get_mask(subject, object)?;

// Inheritance
set_inherit(actor, object, child, parent)?;
```

## License

CNCOSL - See LICENSE
