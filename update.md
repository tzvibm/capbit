# Capbit v2: Modal Authorization Architecture

## Overview

Upgrade capbit from flat bitmask authorization to a **modally-qualified three-tuple architecture**. Every relationship, delegation, and permission gains a modal operator that qualifies the *strength* of the authorization — unifying MAC, DAC, and explicit deny in a single model while preserving capbit's core principle: authorization as atomized data.

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

### How Fong approached it (ReBAC, 2011)

Philip Fong's insight: authorization should be based on **relationships** in a social network, not static role assignments. He modeled the social network as a **Kripke structure** (worlds = users, accessibility relations = relationships) and used **modal logic** as the policy language.

- ◇φ (diamond) = "there exists a neighbor where φ holds" — used for path reachability ("friend-of-friend")
- □φ (box) = "for all neighbors, φ holds" — used for universal constraints
- Policies are modal formulas evaluated against the graph

**What Fong solved:**
- Policies based on social structure rather than static roles
- Delegation of trust through relationship paths
- Contextual relationships as first-class concept
- A formal policy language with decidable evaluation

**What Fong couldn't solve:**

- **Bisimulation barrier.** Modal logic cannot distinguish bisimilar graph structures. Two different social networks that "look the same" to modal formulas satisfy the same policies. This makes **counting policies inexpressible** — "at least 3 common friends" (cf_k for k > 2) and "belong to a clique of size k" (clique_k for k > 2) are not definable because their models are bisimilar to simpler cases.

- **Exponential blowup.** Expressing "at least k friends" requires exponentially large formulas in the modal language. Bruns, Fong, Siahaan, and Huth (2012) proposed **hybrid logic with nominals** (@ᵢ operators that name specific worlds) to fix this, but at the cost of a more complex logical framework.

- **Monotonicity constraint.** Policies that grant access based on relationship *absence* ("grant if no competitor relationship exists") require complete knowledge of the entire social network. Fong noted that restricting to monotonic policies enables decentralized implementation but limits expressiveness.

- **No deny.** ¬ in Fong's logic negates a *formula*, not an *authorization*. There's no way to say "this subject is explicitly prohibited" as distinct from "this subject has no matching relationship."

- **No MAC/DAC distinction.** All relationships are equally weighted. There's no way to distinguish structural (mandatory) access from discretionary access.

- **No temporal or environmental semantics.** The model is static. Time-bounded access, scheduled activation, and environmental conditions are outside its scope.

- **Graph traversal complexity.** Policy evaluation requires walking the social network graph. Complexity scales as O(L^N × V^(N+1)) where L = edge labels, V = nodes, N = path length.

### How capbit v2 solves it

**The key move:** instead of using modal operators as *quantifiers over graph neighbors* (Fong's approach), use them as *qualifiers on stored tuples*. The operators □, ◇, ¬ don't ask "is there a path?" — they state "this fact holds with this strength."

This sidesteps the graph-topology problems entirely:
- No Kripke structure to traverse — just indexed tuple lookups
- No bisimulation barrier — tuples are concrete facts, not graph-indistinguishable structures
- No formula blowup — counting is a query over stored data, not a formula construction
- Explicit deny is a stored tuple (¬), not inference from absence
- MAC vs DAC is □ vs ◇ on the same tuple
- Temporal and environmental conditions are extended modal operators on the same tuple

The three-tuple model with modal qualification captures everything Fong's ReBAC can express, everything his hybrid logic extension added, and capabilities that neither framework addresses (MAC/DAC unification, temporal bounds, environmental conditions, compound policies).

---

## Current → New

### Current (v0.5)

```
SUBJECTS:   (subject, object, role) → 1         flat grant
INHERITS:   (subject, object, role) → parent     single-parent delegation
OBJECTS:    (object, role) → mask                permission bitmask
```

Three concerns, three partitions, but no modality. Every grant is equally strong. No deny. No conditional access. Inheritance is a parent pointer, not a qualified delegation.

### New (v2)

```
Tuple 1 — RELATIONS:    (subject, object, context, modal) → value
Tuple 2 — DELEGATIONS:  (subject, object, context, modal, target) → value
Tuple 3 — PERMISSIONS:  (object, context, modal) → mask
```

Same three concerns, same atomized tuples, now with modal qualification at every level.

- `role` becomes `context` — a role is one kind of context, context is the broader concept
- `modal` (u8) is new — qualifies every tuple with □, ◇, ¬, or an extended operator
- Delegation `target` moves into the key, enabling multiple delegations per (subject, object, context)
- `mask` (u64 bitmask) is unchanged — capbit keeps bitmask efficiency

## Core Modals

```
□  (necessary)   — structural, mandatory, MAC-like
◇  (possible)    — discretionary, conditional, DAC-like
¬  (deny)        — explicit prohibition, overrides □ and ◇
```

### What They Mean on Each Tuple

**Tuple 1 — Relationships:**
```
(Alice, Document1, editor, □)    Alice is necessarily an editor
(Bob,   Document1, editor, ◇)    Bob is possibly an editor (conditional)
(Eve,   Document1, editor, ¬)    Eve is explicitly not an editor
```

Three subjects, same object, same context, different modal strength. The distinction between *absence* (no tuple — system has no opinion) and *negation* (¬ tuple — system explicitly decided) is critical.

**Tuple 2 — Delegations:**
```
(Alice, Document1, editor, □, Bob)     Alice necessarily delegates editor to Bob
(Alice, Document1, editor, ◇, Carol)   Alice conditionally delegates editor to Carol
(Alice, Document1, editor, ¬, Eve)     Alice explicitly blocks Eve from delegation
```

**Tuple 3 — Permissions:**
```
(Document1, editor, □) → READ|WRITE|COMMENT     editors necessarily can rwc
(Document1, editor, ◇) → DELETE                  editors possibly can delete
(Document1, editor, ¬) → ADMIN                   editors necessarily cannot admin
```

Same bitmask. Different masks per modality. A single (object, context) can have up to three permission entries.

## Modal Composition

When access flows through delegation chains or across relationship + permission joins, modalities compose:

```
□ × □ = □       necessary through necessary = necessary
□ × ◇ = ◇       necessary through possible = possible
◇ × □ = ◇       possible through necessary = possible
◇ × ◇ = ◇       possible through possible = possible
¬ × _ = ¬       deny through anything = deny
_ × ¬ = ¬       anything through deny = deny
```

The lattice: `□ > ◇ > ¬`. Composition takes the minimum. Deny absorbs everything.

### Evaluation

Direct access composes Tuple 1 modality × Tuple 3 modality:
```
(Alice, Document1, editor, □)  ×  (Document1, editor, □, READ|WRITE)  =  □ READ|WRITE
(Bob, Document1, editor, ◇)   ×  (Document1, editor, □, READ|WRITE)  =  ◇ READ|WRITE
```

Delegated access adds Tuple 2 modality into the chain:
```
Alice's relationship:  □
Delegation to Carol:   ◇
Permission:            □
Effective:  □ × ◇ × □ = ◇    Carol possibly can READ|WRITE
```

The weakest link determines the ceiling. Permissions can only weaken through chains, never strengthen.

### Resolution Result

Resolution returns three buckets instead of one flat mask:

```
ModalMask {
    necessary: u64,    □ bits — guaranteed
    possible:  u64,    ◇ bits — conditional
    denied:    u64,    ¬ bits — prohibited
}
```

Deny override: `necessary &= !denied; possible &= !denied;`

Backward-compatible flat check: `(necessary | possible) & !denied`

## What □/◇/¬ Unifies

```
□  =  MAC    Mandatory Access Control — structural, non-discretionary
◇  =  DAC    Discretionary Access Control — conditional, revocable
¬  =  Deny   Explicit prohibition — overrides all grants
```

Three historically separate systems, one modal byte.

---

## Extended Modals

The modal field is an extensible slot. The core three handle most authorization. The extended modals add counting, compound logic, path traversal, named constraints, temporal bounds, and conditional evaluation.

Extended modals store parameters in the **value** field. The key structure never changes — the modal byte is always 1 byte. Only the value interpretation varies.

---

### ◇≥k — Graded (Quorum / Threshold)

"At least k must satisfy this."

**On Tuple 1:**
```
(Alice, Document1, reviewer, ◇≥3)
```
Alice's reviewer relationship only activates if at least 3 reviewer relations exist on Document1. Quorum on the relationship.

**On Tuple 2:**
```
(Alice, Document1, editor, ◇≥2, Bob)
```
Bob's delegated access activates only if at least 2 delegations target Bob as editor. Multi-party authorization — no single delegator is sufficient.

**On Tuple 3:**
```
(Document1, approver, ◇≥3) → APPROVE
```
Approve permission only activates when at least 3 approvers exist.

**Solves Fong's counting problem.** cf_k and clique_k were inexpressible in his modal logic due to the bisimulation barrier. Here, counting is a query over stored tuples, not a formula over graph topology.

**Composition:**
```
□ × ◇≥k = ◇≥k                    necessary through quorum = quorum
◇≥j × ◇≥k = ◇≥max(j,k)          chained quorums → strictest threshold
¬ × ◇≥k = ¬                       deny absorbs
◇≥1 ≡ ◇                           at least one = possible
```

---

### ∧{...} / ∨{...} — Conjunction / Disjunction

"Need ALL of these contexts" or "Need ANY of these contexts."

**Conjunction on Tuple 3:**
```
(Document1, publish, ∧{editor, reviewer, legal}) → PUBLISH
```
Publishing requires the subject to hold editor AND reviewer AND legal contexts. System checks Tuple 1 for all three.

**Disjunction on Tuple 3:**
```
(Document1, access, ∨{owner, editor, admin}) → READ
```
Read access requires owner OR editor OR admin. Any one match suffices.

The resulting modality is the weakest (for ∧) or strongest (for ∨) among matched contexts.

**Use cases:**
- Separation of duties: ∧{author, reviewer, approver}
- Flexible access: ∨{owner, admin, superuser}
- Compliance gates: ∧{training_complete, background_checked, nda_signed}

---

### ◇* / ◇≤n — Transitive / Bounded Path

"Reachable through a chain of delegations."

**On Tuple 2:**
```
(Alice, ProjectX, collaborator, ◇*, Bob)     unbounded transitive delegation
(Alice, ProjectX, collaborator, ◇≤3, Bob)    delegation chain stops after 3 hops
```

◇* removes the global depth limit. ◇≤n sets it per-edge. This gives per-delegation control over blast radius rather than a single system-wide limit.

**Composition:**
```
◇* × ◇* = ◇*                unbounded through unbounded = unbounded
◇* × ◇≤n = ◇≤n              unbounded through bounded = bounded
◇≤j × ◇≤k = ◇≤min(j,k)     bounded through bounded = stricter bound
```

**Recaptures** Fong's path-based policies (friend-of-friend) as explicit, stored delegation chains.

---

### @ᵢ — Nominal (Named Entity Constraint)

"Specific named entities must be present."

This is what hybrid logic added to fix Fong's expressiveness. In Fong's framework, adding nominals required changing the entire logic. Here it's another modal value.

**On Tuple 2:**
```
(Alice, Document1, editor, @{Carol, Dave}, Bob)
```
Bob gets editor access through Alice, but only if Carol and Dave also have editor relations on Document1.

**On Tuple 3:**
```
(Document1, release, @{legal_team}) → APPROVE
```
Approve permission requires legal_team to have a relation on Document1.

**Use cases:**
- "Bob can access only if his manager also has access"
- "Release requires legal_team to have signed off"
- Four-eyes principle: "Both primary and secondary approver must be present"

---

### □_until / ◇_after / □_during — Temporal

"This authorization has a time dimension."

```
(Alice, Document1, editor, □_until(2026-03-01))     necessary until March 1st
(Bob, Document1, editor, ◇_after(2026-03-01))       possible after March 1st
(Document1, maintenance, □_during(t₁, t₂)) → WRITE  write only during interval
```

Evaluated at resolution time: `now < t`, `now >= t`, `t₁ <= now < t₂`.

No separate TTL mechanism. Temporality is a modal qualifier on the same tuples. Expired tuples can be cleaned up lazily.

**Use cases:**
- Temporary elevated access: □_until for incident response
- Scheduled access: ◇_after for new employee onboarding
- Maintenance windows: □_during for deploy permissions
- Contractor access: □_until with contract end date

**Composition:**
```
□_until(t₁) × □_until(t₂) = □_until(min(t₁, t₂))     earlier expiry wins
◇_after(t₁) × ◇_after(t₂) = ◇_after(max(t₁, t₂))    later activation wins
```

---

### →{condition} — Conditional

"This authorization applies only when an external condition holds."

```
(Document1, editor, →{on_vpn}) → DELETE
```
Editors can delete only when condition `on_vpn` is satisfied. The condition is a reference to a pluggable evaluator — the authorization model doesn't know what it checks.

**Examples of conditions:**
- Subject is on corporate VPN
- MFA completed in the last 30 minutes
- System is not in read-only mode
- Subject's department matches object's department

**This is how capbit absorbs ABAC without becoming ABAC.** Attribute evaluation stays outside the tuple model. The tuple says "there's a gate here" and points to it. Conditional access always resolves at ◇ strength (it's inherently discretionary).

**Composition:**
```
→{c₁} × →{c₂} = →{c₁ ∧ c₂}    both conditions must hold
```

---

### Compound Expressions

A single tuple can combine multiple extended modals:

```
(Document1, release, ◇≥2 ∧ @{legal_team} ∧ □_until(2026-12-31)) → APPROVE
```

"Approving Document1 release requires at least 2 approvers, legal_team must have a relation, and this is valid until end of 2026."

All sub-modals must pass (implicit conjunction). The effective modality is the weakest among them. One tuple expressing a policy that would require multiple interacting mechanisms in any other authorization model.

---

## Full Composition Table

```
Modal A        ×  Modal B       =  Result
───────────────────────────────────────────────
□              ×  □             =  □
□              ×  ◇             =  ◇
□              ×  ¬             =  ¬
□              ×  ◇≥k           =  ◇≥k
◇              ×  □             =  ◇
◇              ×  ◇             =  ◇
◇              ×  ¬             =  ¬
◇              ×  ◇≥k           =  ◇≥k
◇≥j            ×  ◇≥k           =  ◇≥max(j,k)
◇*             ×  ◇≤n           =  ◇≤n
◇≤j            ×  ◇≤k           =  ◇≤min(j,k)
□_until(t₁)    ×  □_until(t₂)   =  □_until(min(t₁,t₂))
◇_after(t₁)    ×  ◇_after(t₂)   =  ◇_after(max(t₁,t₂))
→{c₁}          ×  →{c₂}         =  →{c₁ ∧ c₂}
¬              ×  anything      =  ¬
```

General rule: composition takes the stricter/weaker of the two. Deny always wins. Temporal bounds narrow. Quorum thresholds take the maximum. Path bounds take the minimum.

## Expressiveness Comparison

| Policy Type | Fong Modal | Fong + Hybrid | Zanzibar | ABAC | Capbit v2 |
|---|---|---|---|---|---|
| Path reachability | ◇◇φ | ◇◇φ | Schema unions | N/A | ◇* / ◇≤n |
| Counting (at least k) | **No** | Exponential | **No** | **No** | ◇≥k |
| Named individuals | **No** | @ᵢ nominals | **No** | **No** | @ᵢ |
| Conjunction of contexts | ∧ in formula | ∧ in formula | Schema intersect | Rule combo | ∧{...} |
| Disjunction of contexts | ∨ in formula | ∨ in formula | Schema union | Rule combo | ∨{...} |
| Explicit deny | ¬φ (formula) | ¬φ (formula) | Exclusion | Deny rule | ¬ |
| MAC/DAC distinction | **No** | **No** | **No** | Attributes | □ vs ◇ |
| Temporal constraints | **No** | **No** | Zookies (weak) | Time attrs | □_until / ◇_after / □_during |
| Environmental conditions | **No** | **No** | **No** | Yes | →{condition} |
| Compound expressions | **No** | **No** | **No** | **No** | Compound modal |
| Multi-party authorization | **No** | **No** | **No** | **No** | ◇≥k on Tuple 2 |
| Transitive delegation | Graph walk | Graph walk | Schema computed | N/A | ◇* |
| Bounded delegation | Depth limit | Depth limit | **No** | N/A | ◇≤n per-edge |

## What v2 Gains Over v0.5

| Capability | v0.5 | v2 |
|-----------|------|-----|
| Flat grants | Yes | Yes (□) |
| Explicit deny | No | Yes (¬) |
| Conditional/temporary access | No | Yes (◇) |
| MAC/DAC unification | No | Yes (□ vs ◇) |
| Multiple delegations per context | No (single parent) | Yes |
| Modal-qualified delegation | No | Yes |
| Three-tier permission response | No (binary) | Yes (necessary/possible/denied) |
| Deny-override | No | Yes (¬ absorbs □ and ◇) |
| Quorum/threshold | No | Yes (◇≥k) |
| Temporal access | No | Yes (□_until, ◇_after, □_during) |
| Conditional/environmental | No | Yes (→{condition}) |
| Compound policies | No | Yes |
| Bitmask efficiency | Yes | Yes (unchanged) |
| Atomized semantics | Yes | Yes (unchanged) |

## What v2 Gains Over Fong's ReBAC

| Fong Limitation | v2 Status |
|------|------|
| Bisimulation / counting barrier | Avoided — explicit tuples, not graph formulas |
| Monotonicity constraint | Solved — ¬ is explicit denial, not inference from absence |
| Graph traversal complexity | Solved — join-based evaluation |
| No native deny | Solved — ¬ as first-class modal |
| Exponential formula blowup | Avoided — counting at query layer (◇≥k) |
| No MAC/DAC unification | Solved — □ vs ◇ |
| Single-owner assumption | Solved — tuple multiplicity |
| RBAC interoperability | Subsumed — RBAC is context assignment + permission lookup |
| No temporal semantics | Solved — □_until, ◇_after, □_during |
| No environmental conditions | Solved — →{condition} bridges to ABAC |
| No compound policies | Solved — compound modal expressions |

## Summary

Capbit v2 adds a single byte (modal operator) to every tuple. The core three operators (□, ◇, ¬) unify MAC, DAC, and deny. The extended operators (◇≥k, ∧/∨, ◇*/◇≤n, @ᵢ, □_until/◇_after/□_during, →{condition}, compound) capture and exceed the expressiveness of Fong's ReBAC, its hybrid logic extension, Zanzibar, and traditional ABAC — all within the same three-tuple structure, all composable through the same lattice, all preserving capbit's atomized bitmask efficiency.

The modal position is a **pluggable policy algebra**. New operators are added by assigning a byte value and defining resolution semantics. The tuple model never changes.
