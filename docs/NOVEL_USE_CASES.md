# Novel Use Cases: What Capbit Enables

This document explores use cases that Capbit uniquely enables—scenarios that are **architecturally impossible or impractical** with existing access control systems like Zanzibar, SpiceDB, OpenFGA, OPA, and traditional RBAC.

---

## Why These Use Cases Matter

Most access control comparisons focus on features or performance benchmarks. This document focuses on **what you can build** with Capbit that you simply cannot build with other systems—not because they're slower, but because their architecture fundamentally prevents it.

---

## Use Case 1: Edge & Embedded Access Control

### The Problem

Zanzibar-family systems (SpiceDB, OpenFGA, Ory Keto) are **network-dependent services**:
- Every permission check requires a network round-trip (1-10ms minimum)
- Requires infrastructure (databases, servers, load balancers)
- Fails when offline or disconnected

This makes them **unusable** for edge computing, IoT, mobile offline mode, and embedded systems.

### Capbit's Solution

Capbit is a **single embedded library** with local storage:
- No network dependency
- Works completely offline
- 7-8 μs permission checks
- Runs on resource-constrained devices

### Real-World Applications

| Application | Why Impossible Before | How Capbit Enables It |
|-------------|----------------------|----------------------|
| **Smart Lock Firmware** | Can't wait for network to unlock door | Permission checks embedded in lock |
| **Drone Swarm Coordination** | Drones operate disconnected | Each drone carries full ACL state |
| **Industrial PLC Access** | Air-gapped networks, μs requirements | ACL embedded in controller |
| **Offline-First Collaboration** | CRDTs sync offline, but permissions need server | Local permission evaluation |
| **AR/VR Object Visibility** | 16ms frame budget for rendering | Per-object permission checks inline |
| **Vehicle Telematics** | Tunnel/rural connectivity gaps | Local access decisions |
| **Medical Device Permissions** | Life-critical, can't depend on network | Embedded authorization |

### Example: Smart Building Access

```
┌─────────────────────────────────────────────────────────────┐
│                    SMART BUILDING                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌──────────┐     ┌──────────┐     ┌──────────┐            │
│   │ Door 1   │     │ Door 2   │     │ Door 3   │            │
│   │ Capbit   │     │ Capbit   │     │ Capbit   │            │
│   │ Embedded │     │ Embedded │     │ Embedded │            │
│   └────┬─────┘     └────┬─────┘     └────┬─────┘            │
│        │                │                │                   │
│        └────────────────┼────────────────┘                   │
│                         │                                    │
│                    ┌────▼────┐                               │
│                    │ Sync Hub│  ← Periodic sync when online  │
│                    └─────────┘                               │
│                                                              │
│   Each door makes access decisions LOCALLY in <10μs          │
│   No network latency at the point of access                  │
│   Works during network outages                               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Use Case 2: Per-Entity Permission Semantics

### The Problem

In Zanzibar-family systems, **relation semantics are global**:

```yaml
# SpiceDB Schema - "editor" means the same thing EVERYWHERE
definition document {
  relation editor: user
  permission edit = editor
}
```

To have different "editor" meanings for different documents, you need:
- Schema migrations for each variation
- Relation proliferation (`editor_basic`, `editor_premium`, `editor_full`)
- Complex type hierarchies

### Capbit's Solution

Each entity defines what relations mean **independently**:

```
project:startup
  "contributor" = 0x03  (read + write)

project:enterprise
  "contributor" = 0x1F  (read + write + delete + admin + audit)

Same relation name, different powers per context.
```

### Real-World Applications

| Application | Why Impossible Before | How Capbit Enables It |
|-------------|----------------------|----------------------|
| **SaaS Tiered Plans** | "pro" means different things per product | Per-product capability definitions |
| **Multi-Tenant Customization** | Each tenant wants custom roles | Tenant-local role semantics |
| **API Version Permissions** | v1 "write" ≠ v2 "write" | Per-version capability bitmasks |
| **Regional Compliance** | EU "admin" ≠ US "admin" | Region-specific definitions |
| **White-Label Products** | Same code, different permission models | Per-deployment customization |
| **Feature Experiments** | A/B test permission structures | Per-cohort capability definitions |

### Example: Multi-Product SaaS

```
┌─────────────────────────────────────────────────────────────┐
│                    SAAS PLATFORM                             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   Product: Analytics                                         │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  "starter"  = 0x01  (view dashboards)               │   │
│   │  "pro"      = 0x07  (+ create + export)             │   │
│   │  "enterprise" = 0x1F (+ API + white-label)          │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
│   Product: Storage                                           │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  "starter"  = 0x03  (read + write, 1GB)             │   │
│   │  "pro"      = 0x0F  (+ versioning + sharing)        │   │
│   │  "enterprise" = 0xFF (+ compliance + audit)         │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
│   Same tier names, completely different capabilities!        │
│   No schema changes needed when adding products.             │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Use Case 3: Real-Time Permission-Aware Systems

### The Problem

Network latency (1-10ms per check) makes **per-operation permission checks** impractical:

| System | Frame Budget | Network ACL Check | Result |
|--------|-------------|-------------------|--------|
| 60fps Game | 16ms | 5ms | 30% budget lost |
| 120Hz VR | 8ms | 5ms | 62% budget lost |
| Audio (48kHz) | 10ms buffer | 5ms | Glitches |
| HFT Trading | <100μs | 5ms | 50x too slow |

Current workarounds sacrifice either security or functionality:
- Cache permissions → stale data, consistency issues
- Batch pre-authorize → coarse granularity
- Skip checks in hot path → security gaps

### Capbit's Solution

7-8 μs permission checks allow **inline authorization** in real-time systems.

### Real-World Applications

| Application | Timing Requirement | How Capbit Enables It |
|-------------|-------------------|----------------------|
| **3D Object Visibility** | 16ms/frame | Per-object access check before render |
| **Secure Audio Routing** | 10ms buffer | Per-channel permission checks |
| **Trading Order Validation** | <100μs | Inline permission verification |
| **Game Ability Systems** | 8ms tick | Real-time ability permission checks |
| **Robotics Safety** | 1ms cycle | Permission-gated actuator commands |
| **Network Packet Filtering** | μs per packet | Per-flow access decisions |

### Example: Permission-Aware 3D Engine

```
┌─────────────────────────────────────────────────────────────┐
│                    3D RENDERING PIPELINE                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   Frame Budget: 16ms (60fps)                                 │
│                                                              │
│   Traditional ACL (5ms per check):                           │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Check object 1: 5ms  ─┐                            │   │
│   │  Check object 2: 5ms   ├─► 15ms just for 3 objects! │   │
│   │  Check object 3: 5ms  ─┘                            │   │
│   │  Render: 1ms remaining ✗                            │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
│   Capbit (8μs per check):                                    │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Check 1000 objects: 8ms total                      │   │
│   │  Render visible objects: 8ms                        │   │
│   │  Frame complete! ✓                                  │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
│   Use cases:                                                 │
│   - Classified data visualization (need-to-know rendering)  │
│   - Multi-tenant 3D environments                            │
│   - Collaborative CAD with access control                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Use Case 4: Capability Accumulation Model

### The Problem

Zanzibar uses **relation existence** as the permission primitive:
- You either have a relation or you don't
- Combining permissions requires explicit schema rules
- Adding new combination rules requires schema changes

```yaml
# SpiceDB: Must explicitly define every combination
permission view = viewer + editor + admin
permission edit = editor + admin
permission delete = admin
# What about viewer + commenter + suggester? More rules...
```

### Capbit's Solution

Multiple grants **OR together** automatically:

```
Alice has:
  - "viewer" on doc:123     → 0x01 (read)
  - "commenter" on doc:123  → 0x04 (comment)

Effective capability: 0x01 | 0x04 = 0x05 (read + comment)

No schema changes. Algebraically composable.
```

### Real-World Applications

| Application | Why Impossible Before | How Capbit Enables It |
|-------------|----------------------|----------------------|
| **Skill-Based Access** | Each skill = separate relation | Skills OR together naturally |
| **Badge/Achievement Systems** | Complex badge→permission mapping | Each badge adds capability bits |
| **Role Stacking** | "Manager + Engineer" needs explicit rule | Roles OR automatically |
| **Progressive Unlocks** | Track "levels" explicitly | Capabilities accumulate |
| **Certification-Based Access** | Certification combos need rules | Certifications add bits |
| **Training Progression** | Module completion → access | Each module adds capabilities |

### Example: Developer Competency System

```
┌─────────────────────────────────────────────────────────────┐
│                 COMPETENCY-BASED ACCESS                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   Competencies defined as capability bits:                   │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  bit0 = Python certified                            │   │
│   │  bit1 = Security cleared                            │   │
│   │  bit2 = On-call trained                             │   │
│   │  bit3 = Production access                           │   │
│   │  bit4 = Database admin                              │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
│   Alice's journey:                                           │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Day 1:   Hired                    → 0x00           │   │
│   │  Week 2:  Python certified         → 0x01           │   │
│   │  Month 1: Security cleared         → 0x03           │   │
│   │  Month 3: On-call trained          → 0x07           │   │
│   │  Month 6: Production access granted→ 0x0F           │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
│   Production incident access requires: 0x07                  │
│   (Python + Security + On-call)                              │
│                                                              │
│   check: (alice_caps & 0x07) == 0x07? → YES after month 3   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Use Case 5: Self-Enforcing Authorization

### The Problem

In Zanzibar-family systems, **mutations are not authorized by the ACL system**:

```go
// Application code MUST check before calling SpiceDB
if userCanGrant(ctx, user, resource) {  // Custom check!
    client.WriteRelationships(...)       // SpiceDB trusts caller
}
// Bug in userCanGrant() = security breach
// Forgot to call userCanGrant() = security breach
```

The ACL system is a **data store**, not an **enforcer**.

### Capbit's Solution

**Every mutation checks authorization internally**:

```rust
// This WILL FAIL if requester lacks GRANT_WRITE on scope
protected::set_grant(requester, seeker, relation, scope)?;

// No way to bypass - authorization is in the storage layer
```

### Real-World Applications

| Application | Why Impossible Before | How Capbit Enables It |
|-------------|----------------------|----------------------|
| **Self-Service Portals** | Must trust application code | Built-in authorization bounds |
| **Decentralized Admin** | Delegation logic in app layer | Delegation enforced at storage |
| **Audit-Complete Systems** | ACL doesn't know who mutated | Actor recorded per mutation |
| **Permission Marketplaces** | Trust boundaries unclear | Grants bounded by delegator |
| **GitOps for Permissions** | Drift between code and state | Mutations fail if unauthorized |

### Example: Delegated Team Management

```
┌─────────────────────────────────────────────────────────────┐
│               SELF-ENFORCING DELEGATION                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   Root delegates to HR Lead:                                 │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  HR Lead can manage users                           │   │
│   │  (ENTITY_CREATE | ENTITY_DELETE on _type:user)      │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
│   HR Lead tries to:                                          │
│                                                              │
│   ✓ Create user:frank                                        │
│     → Allowed (has ENTITY_CREATE on _type:user)              │
│                                                              │
│   ✗ Create team:newteam                                      │
│     → BLOCKED at storage layer                               │
│     → "lacks permission 0x0004 on _type:team"                │
│                                                              │
│   ✗ Grant themselves admin on _type:team                     │
│     → BLOCKED at storage layer                               │
│     → Cannot escalate beyond delegation                      │
│                                                              │
│   No application code needed to enforce these boundaries!    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Use Case 6: Deterministic Performance SLAs

### The Problem

Graph traversal systems have **data-dependent performance**:

```
Simple query:    user → doc           = 1ms
With groups:     user → group → doc   = 3ms
Deep nesting:    user → g1 → g2 → g3 → doc = 10ms
Pathological:    Complex graph        = 100ms+

P99 latency can be 100x P50 latency!
```

This makes **SLA guarantees impossible**:
- Can't promise consistent latency
- "Poison" permission patterns can DoS the system
- Heavy users affect other tenants

### Capbit's Solution

Performance is **O(log N)** regardless of permission structure:
- No graph traversal
- Bounded inheritance depth (configurable)
- Cycle-safe by design

```
Simple query:    7μs
Complex query:   8μs
Pathological:    25μs (with 3-level inheritance)

Variance: ~3x, not 100x
```

### Real-World Applications

| Application | Why Impossible Before | How Capbit Enables It |
|-------------|----------------------|----------------------|
| **SLA-Bound APIs** | Can't guarantee permission check latency | Deterministic complexity |
| **Rate-Limited Services** | Permission check cost varies | Consistent resource usage |
| **Fair Multi-Tenancy** | Heavy tenants affect others | Isolated performance |
| **Cost-Based Billing** | Can't predict check costs | Predictable per-check cost |
| **Real-Time Bidding** | Must respond in fixed time | Guaranteed latency bounds |

### Example: Multi-Tenant SLA Isolation

```
┌─────────────────────────────────────────────────────────────┐
│                  MULTI-TENANT ISOLATION                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   Zanzibar-style system:                                     │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Tenant A: Simple permissions      → 2ms            │   │
│   │  Tenant B: Deep group nesting      → 50ms           │   │
│   │  Tenant C: Circular references     → 200ms + cycle  │   │
│   │                                       detection      │   │
│   │                                                      │   │
│   │  Tenant B's query structure affects Tenant A's       │   │
│   │  latency when sharing infrastructure!                │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
│   Capbit:                                                    │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Tenant A: 8μs                                      │   │
│   │  Tenant B: 8μs                                      │   │
│   │  Tenant C: 8μs                                      │   │
│   │                                                      │   │
│   │  Structure doesn't matter. All tenants get same     │   │
│   │  performance. SLA: <50μs guaranteed.                │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Use Case 7: Efficient Universal Permissions

### The Problem

Zanzibar stores **one tuple per permission relationship**:

```
"All users can read public docs"

Zanzibar approach:
  user:alice#reader@doc:public1
  user:alice#reader@doc:public2
  user:bob#reader@doc:public1
  user:bob#reader@doc:public2
  ... (N users × M public docs = N×M tuples)
```

Group membership helps but:
- Still need `doc:X#reader@group:everyone` per doc
- Group changes require updating all member tuples
- Materialized views explode storage

### Capbit's Solution

Capability definitions are **per entity, per relation type**:

```
doc:public1 defines: "reader" = 0x01
doc:public2 defines: "reader" = 0x01

Grant everyone "reader" on doc:public1 = 1 relationship
Grant everyone "reader" on doc:public2 = 1 relationship

Total: 2 relationships + 2 capability definitions
(Not 2 × N_users tuples)
```

### Real-World Applications

| Application | Why Impossible Before | How Capbit Enables It |
|-------------|----------------------|----------------------|
| **Public Content** | Need tuple per user per content | One grant covers all |
| **Default Permissions** | Materialized across all entities | Template capability definitions |
| **Broadcast Access** | "Everyone can X" = explosion | Single relation type |
| **Template Instantiation** | Copy N tuples per instance | Copy capability definitions |
| **Organization Defaults** | Per-user baseline tuples | Org-level grants |

### Example: Content Platform

```
┌─────────────────────────────────────────────────────────────┐
│                   CONTENT PLATFORM                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   1 million users, 100K public documents                     │
│                                                              │
│   Zanzibar approach:                                         │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Option A: Direct tuples                            │   │
│   │    1M users × 100K docs = 100 BILLION tuples        │   │
│   │    (Obviously infeasible)                           │   │
│   │                                                      │   │
│   │  Option B: Group membership                         │   │
│   │    group:public#member@user:* = 1M tuples           │   │
│   │    doc:X#reader@group:public = 100K tuples          │   │
│   │    Total: 1.1M tuples                               │   │
│   │    But: group changes require 1M updates            │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
│   Capbit approach:                                           │
│   ┌─────────────────────────────────────────────────────┐   │
│   │  Define "public" capability on each public doc      │   │
│   │    100K capability definitions                      │   │
│   │                                                      │   │
│   │  Grant "public" to the public group once            │   │
│   │    1 grant                                          │   │
│   │                                                      │   │
│   │  Users inherit from public group                    │   │
│   │    Query-time inheritance, no materialization       │   │
│   │                                                      │   │
│   │  Total storage: ~100K records                       │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Summary: Capability Matrix

| Novel Use Case | Zanzibar | SpiceDB | OpenFGA | OPA | Casbin | **Capbit** |
|---------------|----------|---------|---------|-----|--------|-----------|
| Edge/Embedded Deployment | ❌ | ❌ | ❌ | ⚠️ | ⚠️ | ✅ |
| Per-Entity Semantics | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| Real-Time (<100μs) | ❌ | ❌ | ❌ | ❌ | ⚠️ | ✅ |
| Capability Accumulation | ⚠️ | ⚠️ | ⚠️ | ✅ | ❌ | ✅ |
| Self-Enforcing Mutations | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Deterministic SLAs | ❌ | ❌ | ❌ | ❌ | ⚠️ | ✅ |
| No Tuple Explosion | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |

Legend: ✅ = Fully supported | ⚠️ = Partial/workaround | ❌ = Architecturally impossible

---

## Conclusion

Capbit enables application categories that are **architecturally impossible** with existing systems:

1. **Edge/Embedded**: IoT, offline-first, air-gapped systems
2. **Context-Aware Permissions**: Per-entity, per-tenant, per-version semantics
3. **Real-Time Authorization**: Games, AR/VR, trading, robotics
4. **Additive Capabilities**: Skills, badges, certifications, progressive unlocks
5. **Self-Enforcing Security**: Delegation bounds enforced at storage layer
6. **Predictable Performance**: SLA-bound, fair multi-tenancy
7. **Efficient Universals**: Broadcast permissions without tuple explosion

These aren't performance improvements—they're **new categories of applications** that couldn't exist before.

---

*Capbit: Access control for the edge, embedded, and real-time era.*
