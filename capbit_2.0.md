# Capbit 2.0: Modal Authorization Architecture

## First Principles

Authorization reduces to five things:

**1. Resources exist.** A document, a server, a database, an API endpoint. Anything that actions can be performed on.

**2. Actions can be applied to resources.** Read, write, delete, grant, define-actions, create-resource. These are the verbs.

**3. Entities want to perform actions on resources.** Users, services, agents. These are the actors.

**4. Relationships connect entities to resources.** An entity has a relationship to a resource that gives them specific actions on it. This is the only authorization primitive.

**5. Everything is an action on a resource — including governance.** Defining what actions exist on a resource is itself an action on that resource. Granting a relationship to an entity is itself an action on that resource. Ownership is having all actions, including the meta-actions (grant, revoke, define, delete). There is no separate "admin layer" or "policy engine." The system governs itself through the same mechanism it governs everything else.

### The self-governing recursion

The system as a whole is the first resource. It has actions: create-resource, delete-resource, manage-system. The root entity has all actions on the system resource.

Creating a new resource is the root entity (or anyone with the create-resource action) exercising an action on the system resource. The new resource then declares its own actions, governed by its own relationships. Granting someone access to that resource is an action on that resource — which requires having the grant action, which is itself governed by the relationship system.

Bootstrap is: one resource (system) exists, one entity (root) has all actions on it. Everything else follows.

There is no special case. No separate bootstrap logic. No distinction between "the authorization system" and "the things it authorizes." One recursive pattern: **resources with actions, entities with relationships, all the way down.**

### Why existing models overcomplicate this

The first principles are simple. Academic models made them complex by starting from formalisms and working backward to the problem:

- **Fong's ReBAC** modeled relationships as a Kripke structure and policies as modal formulas. This introduced bisimulation barriers, exponential blowup, and monotonicity constraints — problems that don't exist in the actual domain. They're artifacts of the mathematical lens.
- **Zanzibar** introduced a schema language with userset rewrites, computed relations, and type hierarchies. Now you need a deployment pipeline for policy changes and can't override one object without inventing a new type.
- **ABAC** made every check an unbounded runtime evaluation of attribute expressions.
- **RBAC** collapsed relationships into static role assignments, losing the structure entirely.

The domain says: resources have actions, entities have relationships, governance is self-referential. Capbit 2.0 stores exactly that — with a modal qualifier to express how strongly each fact holds.

---

## The Problem Space

### What authorization has always struggled with

**1. Binary grants can't express strength.**
Traditional systems — RBAC, Zanzibar, capbit v0.5 — treat access as binary: you have it or you don't. There's no way to say "Alice definitely has access" vs "Bob might have access pending approval" vs "Eve is explicitly forbidden." All three are fundamentally different authorization states, but binary systems collapse them into has/hasn't.

**2. No explicit deny — only absence.**
If Bob has no tuple, does the system have no opinion, or has it decided Bob should be blocked? Absence and denial are conflated. This forces closed-world assumptions (everything not granted is denied) which break in federated or decentralized systems where incomplete information is normal.

**3. MAC and DAC are separate systems.**
Mandatory Access Control (structural, non-discretionary) and Discretionary Access Control (user-managed, revocable) have always been implemented as separate layers. Organizations run both and hope they don't conflict. There's no unified model.

**4. Delegation is unqualified.**
When Alice delegates to Bob, the delegation carries no metadata about its strength, conditions, or limits. Is it permanent? Temporary? Conditional on approval? Can Bob re-delegate? Current systems either allow full delegation or none.

**5. Counting, temporal, and environmental policies require separate mechanisms.**
"At least 3 approvers" needs a workflow engine. "Access until March 1st" needs a TTL system. "Only on VPN" needs an attribute engine. Each concern lives in a different system with its own model.

**6. Resolution direction is fixed.**
Most systems resolve forward only: given a subject, determine access. The reverse question — "who can access this resource?" — requires graph walking, recursive expansion, or full scans. Auditability suffers.

### How Fong approached it (ReBAC, 2011)

Philip Fong's insight: authorization should be based on **relationships** in a social network, not static role assignments. He modeled the social network as a **Kripke structure** (worlds = users, accessibility relations = relationships) and used **modal logic** as the policy language.

In plain terms: imagine every user is a node in a graph. Friendship, management, team membership — these are all edges. Fong's idea was to write authorization policies as questions about the shape of that graph.

- "There exists a friend who has access" — diamond operator (◇), existential
- "All managers must approve" — box operator (□), universal
- Policies are modal formulas evaluated by walking the graph

**What Fong solved:**
- Policies based on social structure rather than static roles
- Delegation of trust through relationship paths
- Contextual relationships as first-class concept
- A formal policy language with decidable evaluation

**What Fong couldn't solve:**

- **Bisimulation barrier.** Modal logic cannot distinguish bisimilar graph structures. Two different social networks that "look the same" to modal formulas satisfy the same policies. This makes **counting policies inexpressible** — "at least 3 common friends" (cf_k for k > 2) and "belong to a clique of size k" (clique_k for k > 2) are not definable because their models are bisimilar to simpler cases.

- **Exponential blowup.** Expressing "at least k friends" requires exponentially large formulas in the modal language. Bruns, Fong, Siahaan, and Huth (2012) proposed **hybrid logic with nominals** (@_i operators that name specific worlds) to fix this, but at the cost of a more complex logical framework.

- **Monotonicity constraint.** Policies that grant access based on relationship *absence* ("grant if no competitor relationship exists") require complete knowledge of the entire social network. Fong noted that restricting to monotonic policies enables decentralized implementation but limits expressiveness.

- **No deny.** Negation in Fong's logic negates a *formula*, not an *authorization*. There's no way to say "this subject is explicitly prohibited" as distinct from "this subject has no matching relationship."

- **No MAC/DAC distinction.** All relationships are equally weighted. There's no way to distinguish structural (mandatory) access from discretionary access.

- **No temporal or environmental semantics.** The model is static. Time-bounded access, scheduled activation, and environmental conditions are outside its scope.

- **Graph traversal complexity.** Policy evaluation requires walking the social network graph. Complexity scales with edge labels, node count, and path length.

### How capbit 2.0 solves it

Start from first principles: resources have actions, entities have relationships, governance is an action on the resource it governs. Now add one thing: a **modal qualifier** on every stored fact, expressing how strongly it holds.

Instead of using modal operators as *quantifiers over graph neighbors* (Fong's approach), use them as *qualifiers on stored tuples*. The operators don't ask "is there a path?" — they state "this fact holds with this strength."

The three tuples map directly to the first principles:

- **Resource declaration** (what actions exist on this resource, under what modal policy) — Tuple 3
- **Entity relationship** (this entity has this action on this resource, with this modal strength) — Tuple 1
- **Inheritance** (this entity inherits actions from another entity, with this modal strength) — Tuple 2

Granting is an action. Defining actions is an action. Both are governed by the same tuples. The system resource bootstraps the recursion.

This sidesteps the graph-topology problems entirely:
- No Kripke structure to traverse — just indexed tuple lookups
- No bisimulation barrier — tuples are concrete facts, not graph-indistinguishable structures
- No formula blowup — counting is a query over stored data, not a formula construction
- Explicit deny is a stored tuple, not inference from absence
- MAC vs DAC is a modal flag on the same tuple
- Temporal and environmental conditions are extended modal operators on the same tuple

The three-tuple model with modal qualification captures everything Fong's ReBAC can express, everything his hybrid logic extension added, and capabilities that neither framework addresses — while staying true to first principles: resources, actions, entities, relationships, self-governance.

---

## Current to New

### Current (v0.5)

```
SUBJECTS:   (subject, object, role) -> 1         flat grant
INHERITS:   (subject, object, role) -> parent     single-parent delegation
OBJECTS:    (object, role) -> mask                permission bitmask
```

v0.5 already follows first principles: the system is a resource, ownership is an action, granting is governed by the same permission bits. But every grant is equally strong. No deny. No conditional access. Inheritance is a parent pointer with no qualification.

### New (v2.0)

```
Tuple 1 — RELATIONS:    (entity, resource, action, modal) -> value
Tuple 2 — DELEGATIONS:  (entity, resource, action, modal, target) -> value
Tuple 3 — DECLARATIONS: (resource, action, modal) -> mask
```

The three tuples map to first principles:
- **Tuple 3** — the resource declares what actions exist on it and the modal policy for each
- **Tuple 1** — an entity has a relationship granting them an action on a resource
- **Tuple 2** — an entity inherits an action from another entity, with its own modal

Granting a relationship (Tuple 1) is itself an action on the resource, governed by the same Tuple 3 declarations and Tuple 1 relationships. The system is self-governing.

- `role` becomes `action` — a role is a named bundle of actions, action is the primitive
- `modal` (u16 bitmask) is new — qualifies every tuple (see Modal Encoding below)
- Delegation `target` moves into the key, enabling multiple delegations per (entity, resource, action)
- `mask` (u64 bitmask) is unchanged — capbit keeps bitmask efficiency

---

## Core Modals

```
Box  (necessary)   — structural, mandatory, MAC-like
Diamond  (possible)    — discretionary, conditional, DAC-like
Not  (deny)        — explicit prohibition, overrides all
```

### What They Mean on Each Tuple

**Tuple 1 — Relationships:**
```
(Alice, Document1, editor, Box)    Alice is necessarily an editor
(Bob,   Document1, editor, Diamond)    Bob is possibly an editor (conditional)
(Eve,   Document1, editor, Not)    Eve is explicitly not an editor
```

Three subjects, same object, same context, different modal strength. The distinction between *absence* (no tuple — system has no opinion) and *negation* (Not tuple — system explicitly decided) is critical.

**Tuple 2 — Delegations:**
```
(Alice, Document1, editor, Box, Bob)     Alice necessarily delegates editor to Bob
(Alice, Document1, editor, Diamond, Carol)   Alice conditionally delegates editor to Carol
(Alice, Document1, editor, Not, Eve)     Alice explicitly blocks Eve from delegation
```

**Tuple 3 — Permissions:**
```
(Document1, editor, Box) -> READ|WRITE|COMMENT     editors necessarily can rwc
(Document1, editor, Diamond) -> DELETE                  editors possibly can delete
(Document1, editor, Not) -> ADMIN                   editors necessarily cannot admin
```

Same bitmask. Different masks per modality. A single (object, context) can have up to three permission entries.

---

## Modal Encoding

Modals are encoded as **u16 bitmask flags**, not symbolic bytes. This makes modal values queryable, composable, and indexable as a single numeric field.

```
Bit 0:  Box (necessary)
Bit 1:  Diamond (possible)
Bit 2:  Not (deny)
Bit 3:  Diamond-geq-k (graded/quorum)
Bit 4:  And (conjunction)
Bit 5:  Or (disjunction)
Bit 6:  Diamond-star (transitive path)
Bit 7:  Diamond-leq-n (bounded path)
Bit 8:  At (nominal)
Bit 9:  Box-until (temporal: necessary until)
Bit 10: Diamond-after (temporal: possible after)
Bit 11: Box-during (temporal: necessary during)
Bit 12: Arrow-condition (conditional/ABAC bridge)
```

A compound modal is simply multiple bits set: `Box | Diamond-geq-k | At` = bits 0, 3, 8 = `0b100001001`. Parameters (k values, timestamps, condition references, nominal IDs) are stored in the tuple value field. The key contains the u16 modal bitmask; the value contains the parameters.

This means:
- Forward queries: exact key match on the full modal bitmask
- Reverse queries: prefix scan + bitmask AND to filter by operator
- Composition: bitwise operations on the u16 field
- No parsing, no string matching, no symbolic evaluation

---

## Modal Composition

When access flows through delegation chains or across relationship + permission joins, modalities compose:

```
Box x Box = Box           necessary through necessary = necessary
Box x Diamond = Diamond       necessary through possible = possible
Diamond x Box = Diamond       possible through necessary = possible
Diamond x Diamond = Diamond       possible through possible = possible
Not x _ = Not           deny through anything = deny
_ x Not = Not           anything through deny = deny
```

The lattice: `Box > Diamond > Not`. Composition takes the minimum. Deny absorbs everything.

### Three-Layer Composition

This is the critical insight for resolution efficiency. Every access check composes **three modals** — one from each tuple type:

```
final_modal = min(permission_modal, relationship_modal, inheritance_modal)
```

**Layer 1 — Permission tuple modal:** What strength does the *resource* demand?
```
(Document1, editor, Box) -> READ|WRITE
```
The resource says: "READ|WRITE through editor requires Box (mandatory)."

**Layer 2 — Relationship tuple modal:** What strength does the *subject's grant* carry?
```
(Alice, Document1, editor, Box)       Alice: necessary grant
(Bob, Document1, editor, Diamond)         Bob: discretionary grant
```

**Layer 3 — Delegation tuple modal:** What strength does the *inheritance link* carry?
```
(Charlie, Document1, editor, Diamond, Alice)   Charlie inherits from Alice, discretionary
```

**Composition for direct access** (Tuple 1 x Tuple 3):
```
Alice:  min(Box, Box) = Box       Alice necessarily has READ|WRITE
Bob:    min(Diamond, Box) = Diamond   Bob possibly has READ|WRITE (permission demands Box, Bob only has Diamond)
```

**Composition for inherited access** (Tuple 1 x Tuple 2 x Tuple 3):
```
Charlie inherits from Alice:
  Permission modal:    Box   (resource demands necessary)
  Alice's grant modal: Box   (Alice has necessary)
  Inheritance modal:   Diamond   (link is discretionary)
  Final: min(Box, Box, Diamond) = Diamond   Charlie possibly has READ|WRITE
```

The weakest link determines the ceiling. A discretionary inheritance from a mandatory grant produces a discretionary result. A Not anywhere produces denial. Permissions can only weaken through chains, never strengthen.

### Resolution Result

Resolution returns three buckets instead of one flat mask:

```
ModalMask {
    necessary: u64,    Box bits — guaranteed access
    possible:  u64,    Diamond bits — conditional access
    denied:    u64,    Not bits — prohibited access
}
```

Deny override: `necessary &= !denied; possible &= !denied;`

Backward-compatible flat check: `(necessary | possible) & !denied`

---

## Resolution Architecture

### Forward Resolution (subject -> permission)

"Does Alice have READ on doc:42?"

```
1. Read SUBJECTS(Alice, doc:42) -> context + modal      // one key lookup
2. Read OBJECTS(doc:42, context) -> permission + modal   // one key lookup
3. Compose: min(relationship_modal, permission_modal)    // one min() op
4. If Box: granted. If Diamond: conditional. If Not: denied.
```

**O(1) for direct access.** Two key lookups, one integer comparison. No graph walk.

For inherited access, add one more read:

```
1. Read SUBJECTS(Charlie, doc:42) -> miss               // one key lookup
2. Read INHERITS(Charlie, doc:42, context) -> (modal, Alice)  // one key lookup
3. Read SUBJECTS(Alice, doc:42) -> context + modal       // one key lookup
4. Read OBJECTS(doc:42, context) -> permission + modal    // one key lookup
5. Compose: min(perm_modal, alice_modal, inherit_modal)  // one min() op
```

**O(1) for inherited access.** Fixed number of reads regardless of system size.

### Reverse Resolution (resource -> who has access?)

"Who can access doc:42?"

```
1. Read OBJECTS(doc:42, *) -> all (context, modal, permission) entries
   The resource's own modals tell you what kind of enforcement to expect.
   Box permission -> only Box grants qualify, skip delegation chains.
   Not permission -> stop, nothing to resolve.

2. Scan SUBJECTS_REV(doc:42, *) -> all (subject, context, modal) entries
   Direct holders. For each: compose min(perm_modal, rel_modal).

3. Scan INHERITS_BY_OBJ(doc:42, *) -> all (child, context, modal, parent) entries
   Inherited holders. For each: look up parent's relationship modal,
   compose min(perm_modal, parent_modal, inherit_modal).
```

Two flat prefix scans. For each result, one min() operation. Cost is proportional to **number of actual holders**, not graph complexity.

### The Modal as Query Planner

The permission tuple's modal acts as a filter before resolution even begins:

- **Box permission:** Only subjects with Box relationships qualify. No need to walk delegation chains — discretionary grants can't satisfy mandatory requirements.
- **Diamond permission:** Both Box and Diamond grants qualify. Check delegations too.
- **Not permission:** The permission itself is denied. Return empty immediately.

The resource tells the resolver which indexes to consult. This is information that Zanzibar's schema-based approach cannot provide without interpreting the schema first.

---

## Type-as-Object Pattern

Instead of Zanzibar's separate schema/type system, types are objects in capbit. A "document type" is just another object that individual documents inherit from.

```
Tuple 3 (type-level):
(doctype:7, editor, Box) -> READ|WRITE|COMMENT
(doctype:7, viewer, Box) -> READ

Tuple 2 (inheritance):
(doc:42, doctype:7, type, Box, doctype:7)    doc:42 inherits from doctype:7
(doc:43, doctype:7, type, Box, doctype:7)    doc:43 inherits from doctype:7
```

All documents of this type inherit the same modal permissions. To change policy for all documents: update one tuple on `doctype:7`. O(1) policy change.

To override one specific document:
```
(doc:42, editor, Box) -> READ|WRITE|COMMENT|DELETE    doc:42 editors can also delete
```

The per-object tuple takes precedence. Uniform policy via type inheritance, per-object exceptions via direct tuples.

**What this eliminates:**
- Zanzibar needs a schema file plus a deployment pipeline to change types. Capbit changes a tuple.
- Zanzibar cannot override per-object without creating a new type. Capbit just adds a tuple.
- The "4 million tuples for 4 million documents" problem disappears: 1 type tuple + parent pointers, not per-document permission duplication.

**Reverse query on types:**
- "Who can access any document?" = reverse scan on `doctype:7`
- "Who can access doc:42 specifically?" = reverse scan on both `doc:42` and `doctype:7`, merge results

---

## Efficiency Analysis

### The Conservation Principle

Computation is physics-like: if a policy requires fine-grained conditions, that computation happens regardless of which authorization model you use. Models don't eliminate computation — they redistribute it between write time, check time, and schema time.

What differs is *where* each model pays:

| Model | Write time | Check time | Schema/policy change |
|---|---|---|---|
| **Zanzibar** | O(1) tuple write | O(depth x branching) recursive expansion | O(1) schema edit, requires deploy |
| **ABAC** | O(1) attribute write | O(attributes) runtime evaluation | O(1) policy edit |
| **Fong ReBAC** | O(1) edge addition | O(graph walk) model checking | Rewrite modal formula |
| **Capbit 2.0** | O(1) tuple write | O(1) direct, O(3) inherited | O(1) type-object edit |

### Why Capbit 2.0 Has Fixed Check-Time Cost

Capbit pushes evaluation to **write time**. When you store `(Charlie, doc:42, editor, Diamond, Alice)`, the modal Diamond is decided and persisted at grant time. At check time, you read the stored modal — you don't compute it.

Compare what each system does for "does Charlie have READ on doc:42 through Alice?":

| Step | Zanzibar | Capbit 2.0 |
|---|---|---|
| 1 | Read doc:42 type schema | Read OBJECTS(doc:42, editor) -> Box |
| 2 | Parse userset_rewrite rules | Read INHERITS(Charlie, doc:42, editor) -> (Diamond, Alice) |
| 3 | Expand viewer = direct OR member of group | Read SUBJECTS(Alice, doc:42) -> Box |
| 4 | Check direct -> miss | min(Box, Box, Diamond) = Diamond -> conditional |
| 5 | Expand groups Charlie belongs to | done |
| 6 | For each group, check if group has viewer | — |
| 7 | Recurse if nested groups | — |

Zanzibar: **variable depth**, depends on group nesting and rewrite rule complexity. Each expansion is a new tuple read. You cannot know the cost without walking the graph.

Capbit 2.0: **3 reads, 1 min() operation.** Always. The modal composition is a single integer comparison on pre-stored values.

### Reverse Query Comparison

"Who can access doc:42?"

| Model | Method | Cost |
|---|---|---|
| **Zanzibar** | For every relation type in schema, expand all userset rewrites, recursively resolve groups | O(depth x branching) per type — unbounded |
| **ABAC** | Evaluate every subject's attributes against policy | O(subjects x attributes) — full scan |
| **Fong ReBAC** | Walk social graph evaluating modal formula at each node | O(graph size) — depends on topology |
| **Capbit 2.0** | Prefix scan SUBJECTS_REV(doc:42) + INHERITS_BY_OBJ(doc:42), min() per result | O(holders) — proportional to result count |

Capbit's reverse query cost is proportional to the number of subjects who actually have access, not the total system size or graph complexity. The reverse indexes make this a flat scan, not a recursive expansion.

### Where Capbit 2.0 Pays More

- **Write amplification:** Each grant writes to multiple partitions (subjects + subjects_rev, inherits + inherits_by_obj + inherits_by_parent). Zanzibar writes one tuple per relation.
- **Storage:** Reverse indexes duplicate data for query efficiency. The type-as-object pattern mitigates per-object duplication, but reverse indexes remain.
- **Extended modals:** Operators like Arrow-condition (conditional) and Diamond-geq-k (quorum) add check-time computation. These pull capbit toward ABAC territory for those specific policies. The core Box/Diamond/Not system remains O(1).

### Summary Table

| Metric | Zanzibar | ABAC | Fong ReBAC | Capbit 2.0 |
|---|---|---|---|---|
| Forward check (simple) | O(1) | O(attrs) | O(walk) | **O(1)** |
| Forward check (delegated) | O(depth) | O(attrs) | O(walk) | **O(3 reads)** |
| Reverse query | O(depth x branching) | O(full scan) | O(graph) | **O(holders)** |
| Policy change (uniform) | O(1) schema deploy | O(1) rule edit | Formula rewrite | **O(1) type-object edit** |
| Policy change (per-object) | New type needed | Attribute update | Not supported | **O(1) tuple add** |
| Write cost | O(1) | O(1) | O(1) | O(partitions) |
| Storage overhead | Low | Low | Medium (graph) | Medium (reverse indexes) |
| MAC/DAC unified | No | Via attributes | No | **Yes (Box vs Diamond)** |
| Explicit deny | Via exclusion | Deny rules | Formula negation | **First-class (Not)** |
| Auditability | Schema + tuples | Policy + attributes | Formula + graph | **Tuples only** |

---

## Extended Modals

The modal field is an extensible u16 bitmask. The core three handle most authorization. The extended modals add counting, compound logic, path traversal, named constraints, temporal bounds, and conditional evaluation.

Extended modals store parameters in the **value** field. The key contains the u16 modal bitmask; the value contains the parameters and/or the permission mask. The key structure never changes.

---

### Diamond-geq-k — Graded (Quorum / Threshold)

"At least k must satisfy this."

**On Tuple 1:**
```
(Alice, Document1, reviewer, Diamond-geq-3)
```
Alice's reviewer relationship only activates if at least 3 reviewer relations exist on Document1. Quorum on the relationship.

**On Tuple 2:**
```
(Alice, Document1, editor, Diamond-geq-2, Bob)
```
Bob's delegated access activates only if at least 2 delegations target Bob as editor. Multi-party authorization — no single delegator is sufficient.

**On Tuple 3:**
```
(Document1, approver, Diamond-geq-3) -> APPROVE
```
Approve permission only activates when at least 3 approvers exist.

**Solves Fong's counting problem.** cf_k and clique_k were inexpressible in his modal logic due to the bisimulation barrier. Here, counting is a query over stored tuples, not a formula over graph topology.

**Composition:**
```
Box x Diamond-geq-k = Diamond-geq-k            necessary through quorum = quorum
Diamond-geq-j x Diamond-geq-k = Diamond-geq-max(j,k)    chained quorums -> strictest threshold
Not x Diamond-geq-k = Not                   deny absorbs
Diamond-geq-1 = Diamond                     at least one = possible
```

---

### And / Or — Conjunction / Disjunction

"Need ALL of these contexts" or "Need ANY of these contexts."

**Conjunction on Tuple 3:**
```
(Document1, publish, And{editor, reviewer, legal}) -> PUBLISH
```
Publishing requires the subject to hold editor AND reviewer AND legal contexts. System checks Tuple 1 for all three.

**Disjunction on Tuple 3:**
```
(Document1, access, Or{owner, editor, admin}) -> READ
```
Read access requires owner OR editor OR admin. Any one match suffices.

The resulting modality is the weakest (for And) or strongest (for Or) among matched contexts.

**Use cases:**
- Separation of duties: And{author, reviewer, approver}
- Flexible access: Or{owner, admin, superuser}
- Compliance gates: And{training_complete, background_checked, nda_signed}

---

### Diamond-star / Diamond-leq-n — Transitive / Bounded Path

"Reachable through a chain of delegations."

**On Tuple 2:**
```
(Alice, ProjectX, collaborator, Diamond-star, Bob)     unbounded transitive delegation
(Alice, ProjectX, collaborator, Diamond-leq-3, Bob)    delegation chain stops after 3 hops
```

Diamond-star removes the global depth limit. Diamond-leq-n sets it per-edge. This gives per-delegation control over blast radius rather than a single system-wide limit.

**Composition:**
```
Diamond-star x Diamond-star = Diamond-star              unbounded through unbounded = unbounded
Diamond-star x Diamond-leq-n = Diamond-leq-n            unbounded through bounded = bounded
Diamond-leq-j x Diamond-leq-k = Diamond-leq-min(j,k)   bounded through bounded = stricter bound
```

**Recaptures** Fong's path-based policies (friend-of-friend) as explicit, stored delegation chains.

---

### At — Nominal (Named Entity Constraint)

"Specific named entities must be present."

This is what hybrid logic added to fix Fong's expressiveness. In Fong's framework, adding nominals required changing the entire logic. Here it's another modal flag.

**On Tuple 2:**
```
(Alice, Document1, editor, At{Carol, Dave}, Bob)
```
Bob gets editor access through Alice, but only if Carol and Dave also have editor relations on Document1.

**On Tuple 3:**
```
(Document1, release, At{legal_team}) -> APPROVE
```
Approve permission requires legal_team to have a relation on Document1.

**Use cases:**
- "Bob can access only if his manager also has access"
- "Release requires legal_team to have signed off"
- Four-eyes principle: "Both primary and secondary approver must be present"

---

### Box-until / Diamond-after / Box-during — Temporal

"This authorization has a time dimension."

```
(Alice, Document1, editor, Box-until(2026-03-01))     necessary until March 1st
(Bob, Document1, editor, Diamond-after(2026-03-01))       possible after March 1st
(Document1, maintenance, Box-during(t1, t2)) -> WRITE  write only during interval
```

Evaluated at resolution time: `now < t`, `now >= t`, `t1 <= now < t2`.

No separate TTL mechanism. Temporality is a modal qualifier on the same tuples. Expired tuples can be cleaned up lazily.

**Use cases:**
- Temporary elevated access: Box-until for incident response
- Scheduled access: Diamond-after for new employee onboarding
- Maintenance windows: Box-during for deploy permissions
- Contractor access: Box-until with contract end date

**Composition:**
```
Box-until(t1) x Box-until(t2) = Box-until(min(t1, t2))     earlier expiry wins
Diamond-after(t1) x Diamond-after(t2) = Diamond-after(max(t1, t2))    later activation wins
```

---

### Arrow-condition — Conditional

"This authorization applies only when an external condition holds."

```
(Document1, editor, Arrow{on_vpn}) -> DELETE
```
Editors can delete only when condition `on_vpn` is satisfied. The condition is a reference to a pluggable evaluator — the authorization model doesn't know what it checks.

**Examples of conditions:**
- Subject is on corporate VPN
- MFA completed in the last 30 minutes
- System is not in read-only mode
- Subject's department matches object's department

**This is how capbit absorbs ABAC without becoming ABAC.** Attribute evaluation stays outside the tuple model. The tuple says "there's a gate here" and points to it. Conditional access always resolves at Diamond strength (it's inherently discretionary — the condition may or may not hold).

**Composition:**
```
Arrow{c1} x Arrow{c2} = Arrow{c1 AND c2}    both conditions must hold
```

---

### Compound Expressions

A single tuple can combine multiple extended modals via the bitmask:

```
(Document1, release, Diamond-geq-2 | At{legal_team} | Box-until(2026-12-31)) -> APPROVE
```

"Approving Document1 release requires at least 2 approvers, legal_team must have a relation, and this is valid until end of 2026."

The u16 modal field has bits 3, 8, and 9 set. Parameters (k=2, nominal=legal_team, timestamp=2026-12-31) are in the value. All sub-modals must pass (implicit conjunction). The effective modality is the weakest among them. One tuple expressing a policy that would require multiple interacting mechanisms in any other authorization model.

---

## Full Composition Table

```
Modal A              x  Modal B             =  Result
---------------------------------------------------------------------
Box                  x  Box                 =  Box
Box                  x  Diamond                 =  Diamond
Box                  x  Not                 =  Not
Box                  x  Diamond-geq-k           =  Diamond-geq-k
Diamond                  x  Box                 =  Diamond
Diamond                  x  Diamond                 =  Diamond
Diamond                  x  Not                 =  Not
Diamond                  x  Diamond-geq-k           =  Diamond-geq-k
Diamond-geq-j            x  Diamond-geq-k           =  Diamond-geq-max(j,k)
Diamond-star             x  Diamond-leq-n           =  Diamond-leq-n
Diamond-leq-j            x  Diamond-leq-k           =  Diamond-leq-min(j,k)
Box-until(t1)        x  Box-until(t2)       =  Box-until(min(t1,t2))
Diamond-after(t1)        x  Diamond-after(t2)       =  Diamond-after(max(t1,t2))
Arrow{c1}            x  Arrow{c2}           =  Arrow{c1 AND c2}
Not                  x  anything            =  Not
```

General rule: composition takes the stricter/weaker of the two. Deny always wins. Temporal bounds narrow. Quorum thresholds take the maximum. Path bounds take the minimum.

---

## Expressiveness Comparison

| Policy Type | Fong Modal | Fong + Hybrid | Zanzibar | ABAC | Capbit 2.0 |
|---|---|---|---|---|---|
| Path reachability | Yes | Yes | Schema unions | N/A | Diamond-star / Diamond-leq-n |
| Counting (at least k) | **No** | Exponential | **No** | **No** | Diamond-geq-k |
| Named individuals | **No** | @_i nominals | **No** | **No** | At |
| Conjunction of contexts | In formula | In formula | Schema intersect | Rule combo | And |
| Disjunction of contexts | In formula | In formula | Schema union | Rule combo | Or |
| Explicit deny | Formula negation | Formula negation | Exclusion | Deny rule | Not (first-class) |
| MAC/DAC distinction | **No** | **No** | **No** | Attributes | Box vs Diamond |
| Temporal constraints | **No** | **No** | Zookies (weak) | Time attrs | Box-until / Diamond-after / Box-during |
| Environmental conditions | **No** | **No** | **No** | Yes | Arrow-condition |
| Compound expressions | **No** | **No** | **No** | **No** | Compound modal bitmask |
| Multi-party authorization | **No** | **No** | **No** | **No** | Diamond-geq-k on Tuple 2 |
| Transitive delegation | Graph walk | Graph walk | Schema computed | N/A | Diamond-star |
| Bounded delegation | Depth limit | Depth limit | **No** | N/A | Diamond-leq-n per-edge |
| Per-object override | **No** | **No** | New type needed | Attribute | Direct tuple |
| Reverse query efficiency | O(graph) | O(graph) | O(depth x branching) | O(full scan) | O(holders) |

---

## What v2.0 Gains Over v0.5

| Capability | v0.5 | v2.0 |
|-----------|------|-----|
| Flat grants | Yes | Yes (Box) |
| Explicit deny | No | Yes (Not) |
| Conditional/temporary access | No | Yes (Diamond) |
| MAC/DAC unification | No | Yes (Box vs Diamond) |
| Multiple delegations per context | No (single parent) | Yes |
| Modal-qualified delegation | No | Yes |
| Three-tier permission response | No (binary) | Yes (necessary/possible/denied) |
| Deny-override | No | Yes (Not absorbs Box and Diamond) |
| Quorum/threshold | No | Yes (Diamond-geq-k) |
| Temporal access | No | Yes (Box-until, Diamond-after, Box-during) |
| Conditional/environmental | No | Yes (Arrow-condition) |
| Compound policies | No | Yes (bitmask composition) |
| Type-as-object inheritance | No | Yes |
| Reverse resolution efficiency | BFS walk (max 10) | O(holders) prefix scan |
| Modal as query planner | No | Yes (resource modal filters indexes) |
| Bitmask efficiency | Yes | Yes (unchanged) |
| Atomized semantics | Yes | Yes (unchanged) |

---

## What v2.0 Gains Over Fong's ReBAC

| Fong Limitation | v2.0 Status |
|------|------|
| Bisimulation / counting barrier | Avoided — explicit tuples, not graph formulas |
| Monotonicity constraint | Solved — Not is explicit denial, not inference from absence |
| Graph traversal complexity | Solved — fixed-depth key lookups, reverse indexes |
| No native deny | Solved — Not as first-class modal |
| Exponential formula blowup | Avoided — counting at query layer (Diamond-geq-k) |
| No MAC/DAC unification | Solved — Box vs Diamond |
| Single-owner assumption | Solved — tuple multiplicity |
| RBAC interoperability | Subsumed — RBAC is context assignment + permission lookup |
| No temporal semantics | Solved — Box-until, Diamond-after, Box-during |
| No environmental conditions | Solved — Arrow-condition bridges to ABAC |
| No compound policies | Solved — compound modal bitmask expressions |
| Variable resolution cost | Solved — O(1) direct, O(3) inherited, always |

---

## Summary

The domain is simple: resources exist, actions can be applied to them, entities have relationships that grant them actions, and governance is itself an action on the resource it governs. One recursive pattern, all the way down.

Academic models overcomplicated this by starting from formalisms — Kripke structures, schema languages, attribute algebras — and working backward to the problem. Capbit 2.0 starts from first principles and adds one thing: a u16 bitmask modal qualifier on every tuple, expressing how strongly each fact holds.

The core three operators (Box, Diamond, Not) unify MAC, DAC, and deny. The extended operators (Diamond-geq-k, And/Or, Diamond-star/Diamond-leq-n, At, Box-until/Diamond-after/Box-during, Arrow-condition, compound) capture and exceed the expressiveness of Fong's ReBAC, its hybrid logic extension, Zanzibar, and traditional ABAC.

The three-layer modal composition — declaration modal x relationship modal x inheritance modal = min() — converts authorization from a graph-walking problem into a fixed-cost lookup problem. The resource's own modal acts as a query planner, telling the resolver which indexes to consult before resolution begins. Reverse queries are flat prefix scans proportional to result count, not recursive graph expansions.

The type-as-object pattern eliminates the storage explosion that per-object tuples would otherwise require, while preserving per-object override capability that schema-based systems like Zanzibar cannot offer without creating new types.

Three tuples. Three first principles. One modal qualifier. Everything else follows.
