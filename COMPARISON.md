# Capbit vs Zanzibar

## The Difference

Both systems store authorization as tuples. The difference is where **role semantics** live.

### Zanzibar: Schema + Tuples

```
SCHEMA (parsed manifest):
  type document {
    relation editor: user
    permission write = editor
  }

TUPLES (indexed):
  (doc:100, editor, alice)
  (doc:200, editor, bob)
```

"editor implies write" is defined **once in the schema**, applies to all documents.

### Capbit: Indexed Tuples Only

```
ROLES INDEX:
  (doc:100, EDITOR) → WRITE
  (doc:200, EDITOR) → READ      // different!

CAPS INDEX:
  (alice, doc:100) → EDITOR
  (bob, doc:200) → EDITOR
```

"what EDITOR means" is defined **per object**, stored as indexed data.

## Consequences

### Zanzibar

**Schema is type-level:**
- All documents share the same role definitions
- Changing what "editor" means requires schema change
- To have doc:100 behave differently, create a new type

**Good for:**
- Homogeneous objects (all docs work the same)
- Central governance (schema is source of truth)
- Consistency (same role = same meaning everywhere)

### Capbit

**Roles are instance-level indexed data:**
- Each object defines its own role semantics
- Changing what EDITOR means on doc:100 is an index write
- No schema to parse or manage

**Good for:**
- Heterogeneous objects (each doc can be different)
- Object autonomy (owner controls semantics)
- Queryability (role definitions are indexed, not parsed)

## What Capbit Does NOT Provide

| Capbit lacks | Why it matters |
|---|---|
| Shared schemas | Must duplicate role definitions across objects |
| Type system | No enforcement of "all documents behave alike" |
| Central governance | No single source of truth for role semantics |

## What Capbit Provides

| Capbit has | Why it matters |
|---|---|
| Schema-free | No manifest to parse, version, deploy |
| Instance-level roles | Each object controls its own semantics |
| Queryable role definitions | "What does EDITOR mean on X?" is O(1) index lookup |
| Unified storage | Everything is indexed data, single consistency model |

## Workflow Comparison

### Change what "editor" means for one document

**Zanzibar:**
```
Option A: Change schema (affects ALL documents)
Option B: Create new type "document_restricted"
Option C: Use different relation name
```

**Capbit:**
```
roles.put(doc_100, EDITOR, READ)  // done, O(1)
```

### Ensure all documents have same role semantics

**Zanzibar:**
```
Define once in schema. Enforced automatically.
```

**Capbit:**
```
Must copy role definitions to each document.
Or define on a "type" object and inherit/reference.
No automatic enforcement.
```

### Audit: "Which objects let EDITOR delete?"

**Zanzibar:**
```
Parse schema, find types where editor implies delete.
List objects of those types.
```

**Capbit:**
```
roles.scan()
  .filter(|(obj, role, mask)| role == EDITOR && mask & DELETE)

Single index scan.
```

## Summary

```
Zanzibar = Schema (type-level manifest) + Tuples (instance-level index)
Capbit   = Index only (everything instance-level)
```

Zanzibar separates "what roles mean" (schema) from "who has roles" (tuples).

Capbit stores both as indexed data.

Neither is universally better. Choose based on whether your objects are homogeneous (Zanzibar) or heterogeneous (Capbit).
