# Capbit vs Zanzibar vs ReBAC

## The Progression

### ReBAC: Atomic relationships, computed authorization

Everything is atomic and queryable. But authorization is not expressive - because it's computed, not represented.

```
Relationships (indexed):
  owns(alice, doc:100)
  member_of(alice, engineering)

Rules (code):
  can_write(U, D) :- owns(U, D).
  can_write(U, D) :- member_of(U, G), team_access(G, D).
```

You can ask:
- "Who is related to whom?"
- "What edges exist?"

You cannot ask:
- "What permissions exist?"
- "Why does this permission exist?"
- "What would change if I removed X?"

Because permissions are not data.

**ReBAC is structurally clean but authorization-poor.**

### Zanzibar: Atomic relationships, encoded authorization

Authorization becomes expressive. But expressiveness is stored as a non-atomic blob:
- Schema
- Rewrite rules
- Traversal semantics

```
Schema (manifest):
  type document {
    relation owner: user
    relation editor: user
    permission write = owner + editor
  }

Relationships (indexed):
  (doc:100, owner, alice)
  (doc:100, editor, bob)
```

You gain:
- Conditional permissions
- Inheritance
- Composition

You lose:
- Atomic inspection
- Atomic mutation
- Direct queryability

Authorization truth is encoded, not stored.

**Zanzibar moves expressiveness into code-like structures.**

### Capbit: Atomic relationships, atomic authorization

Authorization is:
- Indexed data
- Addressable
- Mutable
- Composable

Exactly like relationships already were.

```
Relationships (indexed):
  caps[(alice, doc:100)] → EDITOR
  caps[(bob, doc:100)] → VIEWER

Authorization (indexed):
  roles[(doc:100, EDITOR)] → READ|WRITE|DELETE
  roles[(doc:100, VIEWER)] → READ
```

Authorization is no longer:
- A traversal
- A derivation
- A DSL evaluation

It is data.

**Capbit makes authorization first-class data.**

## The Key Difference

|  | ReBAC | Zanzibar | Capbit |
|---|---|---|---|
| Relationships | Atomic | Atomic | Atomic |
| Authorization | Computed | Encoded | Atomic |
| Query permissions | No | No | Yes |
| Mutate permissions | No | No | Yes |
| Explain permissions | No | No | Yes |

This is not incremental - it's categorical.

## What You Can Do

### Query: "What does EDITOR mean on doc:100?"

**ReBAC:** Evaluate rules. No direct answer.

**Zanzibar:** Parse schema for document type. Extract editor permissions.

**Capbit:**
```
roles.get(doc:100, EDITOR)  // O(1)
```

### Mutate: "Make EDITOR read-only on doc:100"

**ReBAC:** Change rules. Affects everything.

**Zanzibar:**
- Option A: Change schema (affects ALL documents)
- Option B: Create new type
- Option C: Different relation name

**Capbit:**
```
roles.put(doc:100, EDITOR, READ)  // done
```

### Explain: "Why can alice write to doc:100?"

**ReBAC:** Trace rule evaluation. Complex.

**Zanzibar:** Trace schema traversal + tuple expansion.

**Capbit:**
```
caps.get(alice, doc:100)     // → EDITOR
roles.get(doc:100, EDITOR)   // → READ|WRITE
// Alice has EDITOR. EDITOR means READ|WRITE on doc:100.
```

### Ensure shared semantics

**ReBAC:** Rules are shared. Automatic.

**Zanzibar:** Schema is shared. Automatic.

**Capbit:** Must copy role definitions or inherit from type object. Manual.

## Trade-offs

| | Zanzibar | Capbit |
|---|---|---|
| Authorization atomicity | No | Yes |
| Expressiveness | Yes | Yes |
| Query/mutate/explain | No | Yes |
| Central governance | Yes | No |
| Shared semantics | Automatic | Manual |

Zanzibar trades atomicity for central governance.
Capbit trades central governance for atomicity.

## Summary

Capbit brings atomicity, queryability, and manipulability to the authorization layer itself, completing what ReBAC started and Zanzibar partially solved.
