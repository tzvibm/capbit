# Capbit Design

The consolidated architecture behind v0.5. Distilled from the modal-authorization
exploration in `capbit_2.0.md`, with the strawmen removed and the scaling gaps
closed. This document records what is built, why, and what is deliberately deferred.

## Thesis

**All expansion happens at write time. Every check is a bounded number of key
reads. Every edge carries a policy qualifier. The system governs itself.**

Zanzibar-family systems (SpiceDB, OpenFGA, Permify) evaluate schema rewrite rules
at check time; their check cost is a function of schema shape and group nesting.
Cedar evaluates a policy language per check. Capbit has no evaluator: a check
reads tuples, ORs some u64s, and does one AND. The cost is statable in advance
as a count of point reads.

## What is distinctive (and what is deliberately not claimed)

Four properties survive honest comparison with the 2026 landscape:

1. **Self-governing mutation authorization.** In Zanzibar-family systems the
   write API is trusted; "who may share this object" lives in application code.
   In capbit, grant/revoke/define/create/delete are permission bits checked by
   the same resolution as everything else. Bootstrap is one special case
   (`_SYSTEM` + `_ROOT`); everything after is the same recursion.

2. **Qualified delegation as queryable data.** Capability tokens (Biscuit, UCAN)
   attenuate on delegation but are bearer artifacts — unqueryable, revocable only
   by expiry. Capbit stores attenuating delegation chains as indexed edges:
   centrally revocable, reverse-queryable, audit-friendly.

3. **Deny-aware, attenuating group membership.** Membership edges carry policy.
   `Not`-membership expresses "everyone in eng except X" as one tuple.
   `Possible`-membership weakens everything reached through it. Neither is
   expressible as data in shipping systems.

4. **Embedded co-location.** In-process checks are microseconds, which makes
   per-row authorization affordable without precomputation, and the single-store
   architecture dissolves the new-enemy problem (no zookies needed: a check after
   a revoke sees the revoke).

Not claimed: atomized tuples (Zanzibar), bitmask access masks (Windows NT ACLs),
grant-as-permission (SQL `WITH GRANT OPTION`, 1976), explicit deny (deny ACEs,
Cedar `forbid`), conditional/temporal tuples (SpiceDB caveats, OpenFGA conditions).

## Data model

Ten partitions, every queryable dimension in the key:

```
OBJECTS:         (object, role) → mask
PARENTS:         (object) → parent                      type-as-object
PARENTS_REV:     (parent, object) → 1
SUBJECTS:        (subject, object, role) → policy       grants
SUBJECTS_REV:    (object, subject, role) → policy
CLOSURE:         (member, group) → policy               materialized membership
CLOSURE_REV:     (group, member) → policy
DELEGATIONS:     (subject, object, role) → (parent, policy)
DELEGATIONS_REV: (object, subject, role) → (parent, policy)
AUDIT:           (seq) → (ts_ms, actor, op, args[4])    append-only
```

### Policy lattice

`Not (0) < Possible (1) < Necessary (2)`. Composition along any chain is `min`
(delegation and membership can only attenuate). `Not` absorbs: a deny edge
blocks its path; denied mask bits are subtracted from both positive buckets.
The extended modal zoo from `capbit_2.0.md` (quorum, temporal, conditional,
nominals) is intentionally **not** implemented: quorum is workflow state, not
access state (it makes check results flip with unrelated grants and destroys
cacheability); temporal/conditional already shipped elsewhere as caveats and
are deferrable; and `min` is only well-defined on a total order — restricting
to the core three keeps composition sound.

### Bit space

Bits 0..16 are meta-permissions governing capbit itself (`_READ_META`, `_DEFINE`,
`_GRANT`, `_REVOKE`, `_DELEGATE`, `_CREATE_OBJ`, `_DELETE_OBJ`, `_SET_PARENT`,
`_AUDIT`). Bits 16..64 belong to the application (`app_bit(n)`). The v0.4 scheme
(22 bits, paired ROLE/MASK flags) collapsed to 9 meaningful bits; meta and app
spaces are disjoint by construction so application masks cannot accidentally
confer governance rights.

## Resolution

`get_masks(sub, obj)` produces `{necessary, possible, denied}`:

1. **Direct**: prefix scan `SUBJECTS(sub, obj)` → bucket each role's mask by the
   grant's policy. Undefined roles resolve to mask 0 (the v0.4
   `unwrap_or(role)` escalation is gone; granting undefined roles is rejected
   at write time as well).
2. **Groups**: prefix scan `CLOSURE(sub)` → for each positively-held group, scan
   its grants on `obj`; effective policy = `min(membership, grant)`; a group's
   `Not` grant denies its members; an excluded (`Not`) member gets nothing.
3. **Delegation**: for each delegation edge, walk the chain (≤10 hops, visited
   set) until an ancestor holds the role directly or via groups; strength =
   `min` over all links.
4. Deny override: `necessary &= !denied; possible &= !denied`.

No recursion at check time. The group graph is never traversed during a check —
that work happened at write time.

## Group closure maintenance

Membership is a grant of `_MEMBER` on the group object — no separate concept,
and group administration is self-governed by the group's own `_GRANT`/`_REVOKE`
bits. On any membership change, the affected set (`{member} ∪ CLOSURE_REV(member)`)
gets its closure recomputed and diffed, in the same atomic batch as the edge
change, with the pending edge applied as an override so the computation sees
uncommitted state.

Closure computation per entity (group graphs are small; this is cheap):

- **Pass 1**: widest-path reachability over positive edges — strength is `min`
  along a path, `max` across paths. `Not` edges from any reachable node mark
  their target denied. Deny sourcing is deliberately conservative: it uses the
  unrestricted reachability set, so a denial holds even if its source group is
  itself only reachable through another denied node.
- **Pass 2**: reachability again with denied groups impassable. Result: positive
  strengths plus explicit `Not` rows.

Positive cycles are rejected at write time (`Cycle`); the relaxation also
terminates under cycles regardless, since strengths only increase and the
lattice has height 2.

Cost model: adding a member is 1 grant + recompute for the affected set
(typically just the member). Nesting a populated group recomputes closures for
its members — O(members × small-graph BFS), batched, the rare operation paying
for every future check. This is the conservation principle: the same expansion
Zanzibar performs on every check, performed once per write.

## Hot objects and audit queries

- `list_subjects` returns the **compressed** answer — groups as single rows —
  paginated by cursor. Expansion is on demand via `members_of` (also paginated).
  Reverse queries are O(grant rows written), not O(users reached).
- **Population algebra**: `population_intersect` / `population_subtract` are
  sorted merge-joins over `CLOSURE_REV` — "in A but not in B" style compliance
  questions as primitives, no offline expansion jobs. (Roaring-bitmap-backed
  population indexes are the planned optimization once benchmarks justify the
  dependency; the API shape is already final.)

## Consistency, durability, audit

- Mutations are serialized under one write lock; the authorization check and the
  write happen under the same lock, so a concurrent revoke cannot race a grant
  (the embedded analogue of the new-enemy problem is closed). Reads are
  lock-free. Multi-partition writes are single atomic batches — no dangling
  reverse-index states.
- Durability defaults to fsync-per-mutation (`Options { durable: true }`);
  losing a revoke is a security event, not data loss. Relaxable for bulk loads.
- Every mutation appends an `AUDIT` row in the same batch: who did what, when,
  to whom — the current-state indexes answer "who can", the log answers "who
  did". Reading it requires the `_AUDIT` bit on `_SYSTEM`.

## Complexity (N = total tuples; log N ≈ free with LSM bloom filters)

| Operation | Cost |
|---|---|
| check / get_masks | grants(sub,obj) + groups(sub) × grants(group,obj) + delegation hops — single-digit point reads typical, hard-bounded |
| grant / revoke (non-member) | 2 batched writes + 1 audit row |
| add/remove member | 1 grant + closure recompute over affected set |
| nest/unnest a populated group | O(members) recompute, batched, rare |
| who-has-access (compressed) | O(grant rows), paginated |
| population algebra | O(merged member counts), sorted streams |
| role/policy change (type-wide) | 1 write, effective immediately |

Storage: ~100 bytes per grant across both indexes; a billion grants fits one
NVMe node. The single-node ceiling is deliberate — see deferred work.

## Deferred (in priority order)

1. **Replication**: authz data is small, read-heavy, latency-critical — the
   embedded core should gain a sync protocol (authoritative writer, replicated
   local readers) rather than become a server. This is the DuckDB/SQLite arc.
2. **Roaring bitmap population indexes** with delta-merge maintenance.
3. **Conditional/temporal qualifiers** (caveat-style) as a fourth edge field,
   only with a sound composition rule.
4. **Name/metadata partition** so bit meanings and entity names are auditable
   data, not application-code convention — the current gap in the "semantics
   as data" claim.
5. **String-ID interning** as an optional layer.

## Production posture notes

- The bundled web UI is a dev harness: it takes `actor` from the request body.
  Any real embedding must derive the actor from an authenticated principal.
- The PolyForm Noncommercial license forecloses commercial embedding; this is a
  deliberate decision point, not an oversight.
