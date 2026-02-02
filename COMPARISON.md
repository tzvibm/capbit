# Capbit vs Zanzibar vs ReBAC

## The Progression

### ReBAC: Atomic assignments, computed semantics

Role assignments are atomic and cheap to query. But role semantics are computed from rules - limited expressiveness, expensive to query.

```
Assignments (indexed):
  owns(alice, doc:100)
  member_of(alice, engineering)

Rules (code):
  can_write(U, D) :- owns(U, D).
  can_write(U, D) :- member_of(U, G), team_access(G, D).
```

You can ask "what permissions exist?" - but you have to evaluate rules. Expensive.

You can't easily define complex semantics - rules have limited expressiveness.

**ReBAC is cheap on assignments, expensive and limited on semantics.**

### Zanzibar: Atomic assignments, encoded semantics

Role assignments are atomic and cheap. Role semantics are encoded in schema blob - expressive but expensive to query.

```
Schema (manifest):
  type document {
    relation owner: user
    relation editor: user
    permission write = owner + editor
  }

Assignments (indexed):
  (doc:100, owner, alice)
  (doc:100, editor, bob)
```

You can define complex semantics - schema is expressive.

You can query semantics - but you have to parse the schema. Expensive.

**Zanzibar is cheap on assignments, expressive but expensive on semantics.**

### Capbit: Atomic assignments, atomic semantics

Both role assignments and role semantics are atomic, indexed, cheap.

```
Assignments (indexed):
  caps[(alice, doc:100)] → EDITOR
  caps[(bob, doc:100)] → VIEWER

Semantics (indexed):
  roles[(doc:100, EDITOR)] → READ|WRITE|DELETE
  roles[(doc:100, VIEWER)] → READ
```

You can define any semantics - it's just data.

You can query semantics - it's just an index lookup. Cheap.

**Capbit is cheap on both assignments and semantics.**

## The Key Difference

|  | Role Assignments | Role Semantics |
|---|---|---|
| **ReBAC** | Atomic | Computed |
| **Zanzibar** | Atomic | Encoded |
| **Capbit** | Atomic | Atomic |

|  | ReBAC | Zanzibar | Capbit |
|---|---|---|---|
| Query assignments | Cheap | Cheap | Cheap |
| Mutate assignments | Cheap | Cheap | Cheap |
| Define semantics | Limited | Expressive | Expressive |
| Query semantics | Expensive | Expensive | Cheap |
| Mutate semantics | Rules change | Schema change | Data write |

## What You Can Do

### Query: "What does EDITOR mean on doc:100?"

**ReBAC:** Evaluate rules. Expensive, limited answer.

**Zanzibar:** Parse schema for document type. Expensive.

**Capbit:**
```
roles.get(doc:100, EDITOR)  // O(1), cheap
```

### Mutate: "Make EDITOR read-only on doc:100"

**ReBAC:** Change rules. Affects everything.

**Zanzibar:**
- Option A: Change schema (affects ALL documents)
- Option B: Create new type
- Option C: Different relation name

**Capbit:**
```
roles.put(doc:100, EDITOR, READ)  // O(1), cheap
```

### Explain: "Why can alice write to doc:100?"

**ReBAC:** Trace rule evaluation. Expensive.

**Zanzibar:** Trace schema traversal + tuple expansion. Expensive.

**Capbit:**
```
caps.get(alice, doc:100)     // → EDITOR
roles.get(doc:100, EDITOR)   // → READ|WRITE
// Alice has EDITOR. EDITOR means READ|WRITE on doc:100.
// Two lookups, cheap.
```

## Zanzibar Semantics on Capbit

Anything Zanzibar expresses can be expressed in Capbit. The difference:

- **Zanzibar**: Provides schema skeleton out of the box
- **Capbit**: Provides atomic primitives, you build the skeleton

```rust
// Central governance: all documents share same role definitions
fn create_document(actor, doc_id) {
    let template = get_type_template("document");
    for (role, mask) in template.roles {
        set_role(actor, doc_id, role, mask)?;
    }
}

// Type enforcement: validate before mutation
fn set_role_checked(actor, obj, role, mask) {
    let type_id = get_type(obj);
    if !type_allows_role_override(type_id) {
        return Err("type does not allow role override");
    }
    set_role(actor, obj, role, mask)
}
```

Simple if/else tooling. Not a fundamental limitation.

## Summary

Capbit makes role semantics first-class data - atomic, indexed, cheap to query and mutate.

ReBAC and Zanzibar can do the same things, but semantics are either computed (limited) or encoded (expensive).

Capbit completes the move: both assignments and semantics are cheap.
