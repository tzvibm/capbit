# Capbit

Schema-free authorization with indexed role definitions.

## Core Idea

Authorization systems like Zanzibar separate **schema** (what roles mean) from **data** (who has what role):

```
Schema (parsed manifest):     "editor" implies write permission
Data (indexed tuples):        (doc:100, editor, alice)
```

Capbit stores both as **indexed data**:

```
roles index:    (doc:100, EDITOR) → READ|WRITE    // what roles mean
caps index:     (alice, doc:100) → EDITOR          // who has what role
```

Role semantics are just another indexed lookup, not a separate manifest.

## Data Structure

```
caps:     (subject, object) → role_id       // who has what
caps_rev: (object, subject) → role_id       // reverse index
roles:    (object, role_id) → mask          // what roles mean (per object)
inherit:  (object, child) → parent          // permission inheritance
```

All pointer-based index lookups. O(1) with bloom filters, O(log n) worst case.

## Permission Resolution

```
check(alice, doc:100, WRITE):

1. caps.get(alice, doc:100) → EDITOR           // O(1) lookup
2. roles.get(doc:100, EDITOR) → READ|WRITE     // O(1) lookup
3. (READ|WRITE) & WRITE == WRITE               // bitmask AND
4. return true
```

With inheritance:

```
check(alice, doc:100, WRITE):

current = alice
mask = 0

loop:
  role = caps.get(current, doc:100)
  mask |= roles.get(doc:100, role)

  parent = inherit.get(doc:100, current)
  if parent: current = parent
  else: break

return mask & WRITE == WRITE
```

## Per-Object Role Semantics

Same role ID, different meanings per object:

```rust
// doc:100 - full access editor
set_role(root, doc_100, EDITOR, READ | WRITE | DELETE)?;

// doc:200 - restricted editor (read only)
set_role(root, doc_200, EDITOR, READ)?;

// Alice is EDITOR on both
grant(root, alice, doc_100, EDITOR)?;
grant(root, alice, doc_200, EDITOR)?;

check(alice, doc_100, DELETE)?  // true
check(alice, doc_200, DELETE)?  // false - same role, different meaning
```

This is a data write in Capbit. In Zanzibar, you'd need separate types.

## Queryability

Since role definitions are indexed data:

```rust
// "What does EDITOR mean on doc:100?"
roles.get(doc_100, EDITOR)  // O(1)

// "Which objects let EDITOR delete?"
roles.scan()
  .filter(|(obj, role, mask)| role == EDITOR && mask & DELETE)

// "Who can access doc:100?"
caps_rev.prefix_scan(doc_100)  // O(k) where k = number of grants
```

In Zanzibar, querying role semantics means parsing the schema manifest.

## Trade-offs vs Zanzibar

| | Zanzibar | Capbit |
|---|---|---|
| Role definitions | Schema (type-level) | Index (instance-level) |
| Shared semantics | Natural (one schema) | Manual (copy or inherit) |
| Unique semantics | Awkward (type per object) | Natural (just data) |
| Query role meanings | Parse schema | Index lookup |
| Modify role meanings | Schema change | Data write |

**Zanzibar**: Better when objects are homogeneous.
**Capbit**: Better when objects are heterogeneous.

## API

```rust
// Bootstrap
let (system, root) = bootstrap()?;

// Define role semantics (ADMIN required)
set_role(actor, object, role_id, mask)?;

// Grant roles (GRANT required)
grant(actor, subject, object, role_id)?;
revoke(actor, subject, object)?;

// Check permissions
check(subject, object, required)?;
get_mask(subject, object)?;

// Inheritance (ADMIN required)
set_inherit(actor, object, child, parent)?;
```

## Testing

```bash
cargo test -- --test-threads=1
```

## License

CNCOSL - See LICENSE
