# Capbit

Authorization as atomized data.

## Core Idea

|  | Relationships | Semantics |
|---|---|---|
| **ReBAC** | Stored | Computed (code) |
| **Zanzibar** | Atomized | Data (schema) |
| **Capbit** | Atomized | Atomized data |

**Zanzibar's insight**: Authorization semantics belong in data, not application code. It delivered by storing semantics as a schema manifest.

**Capbit's refinement**: Authorization semantics should be atomized data - independent tuples, not a schema blob. Both relationships and authorization semantics are stored as independent, atomized tuples—fully queryable and mutable.

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
Relationships (atomized, multi-role):
  SUBJECTS[(alice, doc:100, EDITOR)] → 1
  SUBJECTS[(alice, doc:100, COMMENTER)] → 1   // alice has two roles
  SUBJECTS[(bob, doc:100, VIEWER)] → 1

Semantics (atomized):
  OBJECTS[(doc:100, EDITOR)] → READ|WRITE|DELETE
  OBJECTS[(doc:100, COMMENTER)] → READ|COMMENT
  OBJECTS[(doc:100, VIEWER)] → READ

Inheritance (atomized, role-specific):
  INHERITS[(alice, doc:100, EDITOR)] → admin_group
```

Capbit's delta: semantics are atomized data, not schema blob.
To resolve: prefix scan + mask lookups. No schema parsing.

## Why It Matters

|  | ReBAC | Zanzibar | Capbit |
|---|---|---|---|
| Query relationships | Expensive | Cheap | Cheap |
| Query semantics | Expensive | Expensive (schema) | Cheap |
| Mutate relationships | Rules change | Tuple write | Tuple write |
| Mutate semantics | Rules change | Schema change | Tuple write |

```rust
// Query: "What does EDITOR mean on doc:100?"
OBJECTS.get(doc_100, EDITOR)  // O(1) - it's just a tuple

// Mutate: "Make EDITOR read-only on doc:100"
OBJECTS.put(doc_100, EDITOR, READ)  // O(1) - just write a tuple

// Explain: "Why can alice write to doc:100?"
SUBJECTS.get(alice, doc_100)  // → EDITOR
OBJECTS.get(doc_100, EDITOR)  // → READ|WRITE
// Two tuple lookups. No schema needed.
```

## Data Structure

```
SUBJECTS:           (subject, object, role) → 1        // grant tuple (multiple roles per subject+object)
SUBJECTS_REV:       (object, subject, role) → 1        // reverse index
OBJECTS:            (object, role) → mask              // semantic tuple
INHERITS:           (subject, object, role) → parent   // role-specific inheritance
INHERITS_BY_OBJ:    (object, role, parent, subject) → 1   // reverse index
INHERITS_BY_PARENT: (parent, object, role, subject) → 1   // reverse index
```

Six partitions with reverse indexes for efficient queries in both directions.

A subject can have multiple roles on an object. Inheritance is role-specific.

Implementable with any btree-based database (LMDB, RocksDB, LSM trees).

## Permission Resolution

```
check(alice, doc:100, WRITE):

1. SUBJECTS.prefix(alice, doc:100) → [EDITOR, COMMENTER]  // all roles for alice
2. for each role: mask |= OBJECTS.get(doc:100, role)      // accumulate masks
3. mask & WRITE == WRITE                                   // bitmask check
```

Prefix scan + mask lookups. No schema parsing, no rule evaluation.

With inheritance:

```
current = alice
mask = 0
loop (max 10):
  for role in SUBJECTS.prefix(current, doc:100):
    mask |= OBJECTS.get(doc:100, role)
    if parent = INHERITS.get(current, doc:100, role):
      current = parent
      break
  else: break  // no roles found
return mask & WRITE == WRITE
```

## Zanzibar Semantics on Capbit

Anything Zanzibar expresses can be expressed in Capbit. Zanzibar provides schema skeleton out of the box - Capbit provides independent tuples.

```rust
// Central governance: all documents share same semantics
fn create_document(actor, doc_id) {
    let template = get_type_template("document");
    for (role, mask) in template.roles {
        create(actor, doc_id, role, mask)?;
    }
}
```

Central governance, shared semantics, type enforcement - all buildable with simple if/else tooling.

## API

```rust
// Initialize
init("data_path")?;

// Bootstrap
let (system, root) = bootstrap()?;

// SUBJECTS table (grants) - subject can have multiple roles on object
grant(actor, subject, object, role)?;
revoke(actor, subject, object, role)?;          // removes specific role
check_subject(subject, object, role)?;

// SUBJECTS list queries
list_roles_for(actor, subject, object)?;        // → Vec<role>
list_grants(actor, subject)?;                   // → Vec<(object, role)>
list_subjects(actor, object)?;                  // → Vec<(subject, role)>

// OBJECTS table (role definitions)
create(actor, object, role, mask)?;
update(actor, object, role, mask)?;
delete(actor, object, role)?;
get_object(actor, object, role)?;
check_object(actor, object, role)?;
list_roles(actor, object)?;                     // → Vec<(role, mask)>

// INHERITS table (role-specific inheritance)
inherit(actor, subject, object, role, parent)?;
remove_inherit(actor, subject, object, role)?;
get_inherit(actor, subject, object, role)?;
check_inherit(actor, subject, object, role)?;

// INHERITS list queries
list_inherits(actor, subject, object)?;                    // → Vec<(role, parent)>
list_inherits_on_obj(actor, object)?;                      // → Vec<(role, parent, subject)>
list_inherits_on_obj_role(actor, object, role)?;           // → Vec<(parent, subject)>
list_inherits_from_parent(actor, parent)?;                 // → Vec<(object, role, subject)>
list_inherits_from_parent_on_obj(actor, parent, object)?;  // → Vec<(role, subject)>

// Resolution (no actor required)
check(subject, object, required)?;
get_mask(subject, object)?;

// Utility
clear()?;
```

## License

[PolyForm Noncommercial 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/) - See LICENSE
