# Capbit Database Simulation: Small Organization

This document traces the complete database state from genesis through normal operations.

**Organization**: Acme Corp
- Root admin
- HR team (manages users)
- Engineering team (manages apps/resources)
- Regular employees

---

## Phase 0: Empty Database

```
types/          (empty)
entities/       (empty)
grants/         (empty)
grants_rev/     (empty)
capabilities/   (empty)
delegations/    (empty)
meta/           (empty)
```

---

## Phase 1: Genesis Bootstrap

**Operation**: `bootstrap("root")`

This is the ONLY trusted operation that runs without permission checks.

### Step 1.1: Create Meta-Type

The type that controls creation of other types.

```rust
// Internal, no permission check
create_type_internal("_type")
```

```
types/
└── _type → { creator: "_system", epoch: 1000 }
```

### Step 1.2: Create Core Entity Types

```rust
create_type_internal("user")
create_type_internal("team")
create_type_internal("app")
create_type_internal("resource")
```

```
types/
├── _type    → { creator: "_system", epoch: 1000 }
├── user     → { creator: "_system", epoch: 1001 }
├── team     → { creator: "_system", epoch: 1002 }
├── app      → { creator: "_system", epoch: 1003 }
└── resource → { creator: "_system", epoch: 1004 }
```

### Step 1.3: Create Type Entities (for permission control)

Each type needs an entity so we can grant permissions ON it.

```rust
create_entity_internal("_type", "_type")    // _type:_type - meta-type entity
create_entity_internal("_type", "user")     // _type:user - user type entity
create_entity_internal("_type", "team")     // _type:team - team type entity
create_entity_internal("_type", "app")      // _type:app - app type entity
create_entity_internal("_type", "resource") // _type:resource - resource type entity
```

```
entities/
├── _type:_type    → { creator: "_system", epoch: 1005 }
├── _type:user     → { creator: "_system", epoch: 1006 }
├── _type:team     → { creator: "_system", epoch: 1007 }
├── _type:app      → { creator: "_system", epoch: 1008 }
└── _type:resource → { creator: "_system", epoch: 1009 }
```

### Step 1.4: Define Capabilities on Type Entities

What does "admin" relation mean on each type entity?

```rust
// Admin on _type:_type can create/delete types
set_capability_internal("_type:_type", "admin", TYPE_CREATE | TYPE_DELETE)

// Admin on _type:user can create/delete users
set_capability_internal("_type:user", "admin", ENTITY_CREATE | ENTITY_DELETE)

// Admin on _type:team can create/delete teams
set_capability_internal("_type:team", "admin", ENTITY_CREATE | ENTITY_DELETE)

// Admin on _type:app can create/delete apps
set_capability_internal("_type:app", "admin", ENTITY_CREATE | ENTITY_DELETE)

// Admin on _type:resource can create/delete resources
set_capability_internal("_type:resource", "admin", ENTITY_CREATE | ENTITY_DELETE)
```

```
capabilities/
├── _type:_type/admin    → 0x0003  (TYPE_CREATE | TYPE_DELETE)
├── _type:user/admin     → 0x000C  (ENTITY_CREATE | ENTITY_DELETE)
├── _type:team/admin     → 0x000C  (ENTITY_CREATE | ENTITY_DELETE)
├── _type:app/admin      → 0x000C  (ENTITY_CREATE | ENTITY_DELETE)
└── _type:resource/admin → 0x000C  (ENTITY_CREATE | ENTITY_DELETE)
```

### Step 1.5: Create Root User Entity

```rust
create_entity_internal("user", "root")
```

```
entities/
├── _type:_type    → { creator: "_system", epoch: 1005 }
├── _type:user     → { creator: "_system", epoch: 1006 }
├── _type:team     → { creator: "_system", epoch: 1007 }
├── _type:app      → { creator: "_system", epoch: 1008 }
├── _type:resource → { creator: "_system", epoch: 1009 }
└── user:root      → { creator: "_system", epoch: 1010 }
```

### Step 1.6: Grant Root Admin on All Type Entities

```rust
set_grant_internal("user:root", "admin", "_type:_type")
set_grant_internal("user:root", "admin", "_type:user")
set_grant_internal("user:root", "admin", "_type:team")
set_grant_internal("user:root", "admin", "_type:app")
set_grant_internal("user:root", "admin", "_type:resource")
```

```
grants/
├── user:root/admin/_type:_type    → 1011
├── user:root/admin/_type:user     → 1012
├── user:root/admin/_type:team     → 1013
├── user:root/admin/_type:app      → 1014
└── user:root/admin/_type:resource → 1015

grants_rev/
├── _type:_type/admin/user:root    → 1011
├── _type:user/admin/user:root     → 1012
├── _type:team/admin/user:root     → 1013
├── _type:app/admin/user:root      → 1014
└── _type:resource/admin/user:root → 1015
```

### Step 1.7: Mark Bootstrap Complete

```rust
set_meta("bootstrapped", "true")
set_meta("bootstrap_epoch", "1015")
set_meta("root_entity", "user:root")
```

```
meta/
├── bootstrapped    → "true"
├── bootstrap_epoch → "1015"
└── root_entity     → "user:root"
```

### Genesis Complete - Full Database State

```
types/
├── _type    → { creator: "_system", epoch: 1000 }
├── user     → { creator: "_system", epoch: 1001 }
├── team     → { creator: "_system", epoch: 1002 }
├── app      → { creator: "_system", epoch: 1003 }
└── resource → { creator: "_system", epoch: 1004 }

entities/
├── _type:_type    → { creator: "_system", epoch: 1005 }
├── _type:user     → { creator: "_system", epoch: 1006 }
├── _type:team     → { creator: "_system", epoch: 1007 }
├── _type:app      → { creator: "_system", epoch: 1008 }
├── _type:resource → { creator: "_system", epoch: 1009 }
└── user:root      → { creator: "_system", epoch: 1010 }

capabilities/
├── _type:_type/admin    → 0x0003
├── _type:user/admin     → 0x000C
├── _type:team/admin     → 0x000C
├── _type:app/admin      → 0x000C
└── _type:resource/admin → 0x000C

grants/
├── user:root/admin/_type:_type    → 1011
├── user:root/admin/_type:user     → 1012
├── user:root/admin/_type:team     → 1013
├── user:root/admin/_type:app      → 1014
└── user:root/admin/_type:resource → 1015

grants_rev/
├── _type:_type/admin/user:root    → 1011
├── _type:user/admin/user:root     → 1012
├── _type:team/admin/user:root     → 1013
├── _type:app/admin/user:root      → 1014
└── _type:resource/admin/user:root → 1015

meta/
├── bootstrapped    → "true"
├── bootstrap_epoch → "1015"
└── root_entity     → "user:root"
```

---

## Phase 2: Root Sets Up Organization

Now all operations require permission checks.

### Step 2.1: Root Creates Teams

```rust
// Requester: user:root
// Operation: create_entity("user:root", "team", "hr")

// Permission check:
//   Does user:root have ENTITY_CREATE on _type:team?
//   grants[user:root/admin/_type:team] exists → YES
//   capabilities[_type:team/admin] = 0x000C (includes ENTITY_CREATE)
//   ✓ AUTHORIZED

create_entity("user:root", "team", "hr")
create_entity("user:root", "team", "engineering")
create_entity("user:root", "team", "sales")
```

```
entities/
├── ... (previous)
├── team:hr          → { creator: "user:root", epoch: 2000 }
├── team:engineering → { creator: "user:root", epoch: 2001 }
└── team:sales       → { creator: "user:root", epoch: 2002 }
```

### Step 2.2: Root Creates Initial Users

```rust
create_entity("user:root", "user", "alice")   // HR lead
create_entity("user:root", "user", "bob")     // Engineering lead
create_entity("user:root", "user", "charlie") // Sales lead
create_entity("user:root", "user", "dave")    // Engineer
create_entity("user:root", "user", "eve")     // Engineer
```

```
entities/
├── ... (previous)
├── user:alice   → { creator: "user:root", epoch: 2003 }
├── user:bob     → { creator: "user:root", epoch: 2004 }
├── user:charlie → { creator: "user:root", epoch: 2005 }
├── user:dave    → { creator: "user:root", epoch: 2006 }
└── user:eve     → { creator: "user:root", epoch: 2007 }
```

### Step 2.3: Root Defines Team Membership Relations

What does "member" and "lead" mean on a team?

```rust
// Anyone can be a member or lead of a team
// These are just relations, capabilities are defined per-team

// Define what "lead" means on team:hr
set_capability("user:root", "team:hr", "lead", GRANT_WRITE | GRANT_DELETE)
set_capability("user:root", "team:hr", "member", GRANT_READ)

// Define what "lead" means on team:engineering
set_capability("user:root", "team:engineering", "lead", GRANT_WRITE | GRANT_DELETE)
set_capability("user:root", "team:engineering", "member", GRANT_READ)

// Define what "lead" means on team:sales
set_capability("user:root", "team:sales", "lead", GRANT_WRITE | GRANT_DELETE)
set_capability("user:root", "team:sales", "member", GRANT_READ)
```

Wait - we need to think about WHO can set capabilities on teams. Let me reconsider...

**Question**: Who controls what relations mean on `team:hr`?

Options:
1. Root controls everything
2. The team lead controls their team's capability definitions
3. Whoever has CAP_WRITE on the team

Let's say: **Whoever has CAP_WRITE on the scope can define capabilities for that scope.**

```rust
// First, define what "owner" means on teams (can define capabilities)
set_capability_internal("team:hr", "owner", CAP_WRITE | CAP_DELETE | GRANT_WRITE | GRANT_DELETE)
set_capability_internal("team:engineering", "owner", CAP_WRITE | CAP_DELETE | GRANT_WRITE | GRANT_DELETE)
set_capability_internal("team:sales", "owner", CAP_WRITE | CAP_DELETE | GRANT_WRITE | GRANT_DELETE)

// Root is owner of all teams initially
set_grant("user:root", "owner", "team:hr")
set_grant("user:root", "owner", "team:engineering")
set_grant("user:root", "owner", "team:sales")
```

```
capabilities/
├── ... (previous)
├── team:hr/owner          → 0x0360  (CAP_WRITE | CAP_DELETE | GRANT_WRITE | GRANT_DELETE)
├── team:engineering/owner → 0x0360
└── team:sales/owner       → 0x0360

grants/
├── ... (previous)
├── user:root/owner/team:hr          → 2010
├── user:root/owner/team:engineering → 2011
└── user:root/owner/team:sales       → 2012

grants_rev/
├── ... (previous)
├── team:hr/owner/user:root          → 2010
├── team:engineering/owner/user:root → 2011
└── team:sales/owner/user:root       → 2012
```

### Step 2.4: Root Defines Team Relations

Now root (as owner) can define what relations mean on each team.

```rust
// On team:hr
set_capability("user:root", "team:hr", "lead", GRANT_WRITE | GRANT_READ)
set_capability("user:root", "team:hr", "member", GRANT_READ)

// On team:engineering
set_capability("user:root", "team:engineering", "lead", GRANT_WRITE | GRANT_READ)
set_capability("user:root", "team:engineering", "member", GRANT_READ)

// On team:sales
set_capability("user:root", "team:sales", "lead", GRANT_WRITE | GRANT_READ)
set_capability("user:root", "team:sales", "member", GRANT_READ)
```

```
capabilities/
├── ... (previous)
├── team:hr/lead             → 0x0030  (GRANT_WRITE | GRANT_READ)
├── team:hr/member           → 0x0010  (GRANT_READ)
├── team:engineering/lead    → 0x0030
├── team:engineering/member  → 0x0010
├── team:sales/lead          → 0x0030
└── team:sales/member        → 0x0010
```

### Step 2.5: Root Assigns Team Leads

```rust
set_grant("user:root", "user:alice", "lead", "team:hr")
set_grant("user:root", "user:bob", "lead", "team:engineering")
set_grant("user:root", "user:charlie", "lead", "team:sales")
```

```
grants/
├── ... (previous)
├── user:alice/lead/team:hr           → 2020
├── user:bob/lead/team:engineering    → 2021
└── user:charlie/lead/team:sales      → 2022

grants_rev/
├── ... (previous)
├── team:hr/lead/user:alice           → 2020
├── team:engineering/lead/user:bob    → 2021
└── team:sales/lead/user:charlie      → 2022
```

### Step 2.6: Root Delegates User Management to HR

**This is the key delegation step.**

```rust
// HR team (as an entity) gets admin on _type:user
set_grant("user:root", "team:hr", "admin", "_type:user")
```

```
grants/
├── ... (previous)
└── team:hr/admin/_type:user → 2030

grants_rev/
├── ... (previous)
└── _type:user/admin/team:hr → 2030
```

But wait - `team:hr` is a team entity, not a user. How does `user:alice` (HR lead) inherit this?

**We need delegation!**

```rust
// Alice (HR lead) delegates from team:hr for _type:user permissions
set_delegation("user:root", "user:alice", "_type:user", "team:hr")
```

```
delegations/
└── user:alice/_type:user/team:hr → 2031

delegations_by_del/
└── team:hr/_type:user/user:alice → 2031

delegations_by_scope/
└── _type:user/team:hr/user:alice → 2031
```

**Now the access check for alice creating a user:**

```rust
// Can user:alice create user:frank?
check_access("user:alice", "_type:user")

// Step 1: Direct grants for user:alice on _type:user
//   grants[user:alice/*/_type:user] → none found

// Step 2: Check delegations
//   delegations[user:alice/_type:user/*] → team:hr
//
//   Recurse: check_access("team:hr", "_type:user")
//     grants[team:hr/*/_type:user] → team:hr/admin/_type:user exists!
//     capabilities[_type:user/admin] → 0x000C (ENTITY_CREATE | ENTITY_DELETE)
//
//   Return 0x000C

// Result: user:alice has 0x000C on _type:user
// ENTITY_CREATE (0x0004) is included
// ✓ AUTHORIZED
```

---

## Phase 3: Delegated Operations

### Step 3.1: Alice (HR) Creates New Employee

```rust
// Requester: user:alice
// Operation: create_entity("user:alice", "user", "frank")

// Permission check (as shown above): ✓ AUTHORIZED

create_entity("user:alice", "user", "frank")
```

```
entities/
├── ... (previous)
└── user:frank → { creator: "user:alice", epoch: 3000 }
```

### Step 3.2: Bob (Engineering Lead) Adds Team Members

Bob is lead of team:engineering. Can he add members?

```rust
// Can user:bob grant "member" relation on team:engineering?
check_access("user:bob", "team:engineering")

// Step 1: Direct grants
//   grants[user:bob/*/team:engineering] → user:bob/lead/team:engineering
//   capabilities[team:engineering/lead] → 0x0030 (GRANT_WRITE | GRANT_READ)

// Result: 0x0030
// GRANT_WRITE (0x0020) is included
// ✓ AUTHORIZED
```

```rust
set_grant("user:bob", "user:dave", "member", "team:engineering")
set_grant("user:bob", "user:eve", "member", "team:engineering")
```

```
grants/
├── ... (previous)
├── user:dave/member/team:engineering → 3010
└── user:eve/member/team:engineering  → 3011

grants_rev/
├── ... (previous)
├── team:engineering/member/user:dave → 3010
└── team:engineering/member/user:eve  → 3011
```

### Step 3.3: Engineering Creates an App

First, root needs to delegate app creation to engineering.

```rust
// Root delegates app management to engineering team
set_grant("user:root", "team:engineering", "admin", "_type:app")

// Bob delegates from team:engineering for app permissions
set_delegation("user:root", "user:bob", "_type:app", "team:engineering")
```

```
grants/
├── ... (previous)
└── team:engineering/admin/_type:app → 3020

delegations/
├── ... (previous)
└── user:bob/_type:app/team:engineering → 3021
```

Now Bob can create apps:

```rust
create_entity("user:bob", "app", "backend-api")
create_entity("user:bob", "app", "frontend-web")
```

```
entities/
├── ... (previous)
├── app:backend-api  → { creator: "user:bob", epoch: 3030 }
└── app:frontend-web → { creator: "user:bob", epoch: 3031 }
```

### Step 3.4: Define App Access Relations

Bob defines what relations mean on his apps:

```rust
// First Bob needs to be owner of the apps he created
// (This could be automatic on creation, or explicit)
set_grant_internal("user:bob", "owner", "app:backend-api")   // creator auto-grant?
set_grant_internal("user:bob", "owner", "app:frontend-web")

// Define owner capabilities on apps
set_capability("user:bob", "app:backend-api", "owner", CAP_WRITE | GRANT_WRITE | GRANT_DELETE)
set_capability("user:bob", "app:backend-api", "developer", 0x0F)  // custom app-level caps
set_capability("user:bob", "app:backend-api", "viewer", 0x01)

set_capability("user:bob", "app:frontend-web", "owner", CAP_WRITE | GRANT_WRITE | GRANT_DELETE)
set_capability("user:bob", "app:frontend-web", "developer", 0x0F)
set_capability("user:bob", "app:frontend-web", "viewer", 0x01)
```

```
capabilities/
├── ... (previous)
├── app:backend-api/owner     → 0x0160
├── app:backend-api/developer → 0x000F
├── app:backend-api/viewer    → 0x0001
├── app:frontend-web/owner    → 0x0160
├── app:frontend-web/developer→ 0x000F
└── app:frontend-web/viewer   → 0x0001

grants/
├── ... (previous)
├── user:bob/owner/app:backend-api  → 3040
└── user:bob/owner/app:frontend-web → 3041
```

### Step 3.5: Bob Grants App Access to Team

```rust
set_grant("user:bob", "user:dave", "developer", "app:backend-api")
set_grant("user:bob", "user:eve", "developer", "app:frontend-web")
```

```
grants/
├── ... (previous)
├── user:dave/developer/app:backend-api  → 3050
└── user:eve/developer/app:frontend-web  → 3051
```

---

## Phase 4: Full Database State

After all operations:

```
=== types/ ===
_type    → { creator: "_system", epoch: 1000 }
user     → { creator: "_system", epoch: 1001 }
team     → { creator: "_system", epoch: 1002 }
app      → { creator: "_system", epoch: 1003 }
resource → { creator: "_system", epoch: 1004 }

=== entities/ ===
_type:_type       → { creator: "_system", epoch: 1005 }
_type:user        → { creator: "_system", epoch: 1006 }
_type:team        → { creator: "_system", epoch: 1007 }
_type:app         → { creator: "_system", epoch: 1008 }
_type:resource    → { creator: "_system", epoch: 1009 }
user:root         → { creator: "_system", epoch: 1010 }
team:hr           → { creator: "user:root", epoch: 2000 }
team:engineering  → { creator: "user:root", epoch: 2001 }
team:sales        → { creator: "user:root", epoch: 2002 }
user:alice        → { creator: "user:root", epoch: 2003 }
user:bob          → { creator: "user:root", epoch: 2004 }
user:charlie      → { creator: "user:root", epoch: 2005 }
user:dave         → { creator: "user:root", epoch: 2006 }
user:eve          → { creator: "user:root", epoch: 2007 }
user:frank        → { creator: "user:alice", epoch: 3000 }
app:backend-api   → { creator: "user:bob", epoch: 3030 }
app:frontend-web  → { creator: "user:bob", epoch: 3031 }

=== capabilities/ ===
# Type-level (who can create entities of each type)
_type:_type/admin        → 0x0003  (TYPE_CREATE | TYPE_DELETE)
_type:user/admin         → 0x000C  (ENTITY_CREATE | ENTITY_DELETE)
_type:team/admin         → 0x000C
_type:app/admin          → 0x000C
_type:resource/admin     → 0x000C

# Team-level (what relations mean on teams)
team:hr/owner            → 0x0360  (CAP_WRITE | CAP_DELETE | GRANT_WRITE | GRANT_DELETE)
team:hr/lead             → 0x0030  (GRANT_WRITE | GRANT_READ)
team:hr/member           → 0x0010  (GRANT_READ)
team:engineering/owner   → 0x0360
team:engineering/lead    → 0x0030
team:engineering/member  → 0x0010
team:sales/owner         → 0x0360
team:sales/lead          → 0x0030
team:sales/member        → 0x0010

# App-level (what relations mean on apps)
app:backend-api/owner     → 0x0160
app:backend-api/developer → 0x000F
app:backend-api/viewer    → 0x0001
app:frontend-web/owner    → 0x0160
app:frontend-web/developer→ 0x000F
app:frontend-web/viewer   → 0x0001

=== grants/ ===
# Root's type-level grants
user:root/admin/_type:_type         → 1011
user:root/admin/_type:user          → 1012
user:root/admin/_type:team          → 1013
user:root/admin/_type:app           → 1014
user:root/admin/_type:resource      → 1015

# Root's team ownership
user:root/owner/team:hr             → 2010
user:root/owner/team:engineering    → 2011
user:root/owner/team:sales          → 2012

# Team leads
user:alice/lead/team:hr             → 2020
user:bob/lead/team:engineering      → 2021
user:charlie/lead/team:sales        → 2022

# Type delegation to teams
team:hr/admin/_type:user            → 2030
team:engineering/admin/_type:app    → 3020

# Team memberships
user:dave/member/team:engineering   → 3010
user:eve/member/team:engineering    → 3011

# App ownership and access
user:bob/owner/app:backend-api      → 3040
user:bob/owner/app:frontend-web     → 3041
user:dave/developer/app:backend-api → 3050
user:eve/developer/app:frontend-web → 3051

=== grants_rev/ ===
(reverse of all grants/ entries)

=== delegations/ ===
user:alice/_type:user/team:hr                → 2031
user:bob/_type:app/team:engineering          → 3021

=== delegations_by_del/ ===
team:hr/_type:user/user:alice                → 2031
team:engineering/_type:app/user:bob          → 3021

=== delegations_by_scope/ ===
_type:user/team:hr/user:alice                → 2031
_type:app/team:engineering/user:bob          → 3021

=== meta/ ===
bootstrapped    → "true"
bootstrap_epoch → "1015"
root_entity     → "user:root"
```

---

## Workflow Verification

### Can user:alice create a new user?

```
check_access("user:alice", "_type:user", ENTITY_CREATE)

1. Direct grants: grants[user:alice/*/_type:user] → none
2. Delegations: delegations[user:alice/_type:user/*] → team:hr
3. Recurse: check_access("team:hr", "_type:user")
   - grants[team:hr/*/_type:user] → team:hr/admin/_type:user ✓
   - capabilities[_type:user/admin] → 0x000C
4. Result: 0x000C includes ENTITY_CREATE (0x0004)
✓ AUTHORIZED
```

### Can user:alice create a new team?

```
check_access("user:alice", "_type:team", ENTITY_CREATE)

1. Direct grants: grants[user:alice/*/_type:team] → none
2. Delegations: delegations[user:alice/_type:team/*] → none
3. Result: 0x0000
✗ NOT AUTHORIZED (only root can create teams)
```

### Can user:bob add a member to team:engineering?

```
check_access("user:bob", "team:engineering", GRANT_WRITE)

1. Direct grants: grants[user:bob/*/team:engineering] → user:bob/lead/team:engineering ✓
2. capabilities[team:engineering/lead] → 0x0030
3. Result: 0x0030 includes GRANT_WRITE (0x0020)
✓ AUTHORIZED
```

### Can user:dave add a member to team:engineering?

```
check_access("user:dave", "team:engineering", GRANT_WRITE)

1. Direct grants: grants[user:dave/*/team:engineering] → user:dave/member/team:engineering
2. capabilities[team:engineering/member] → 0x0010 (only GRANT_READ)
3. Result: 0x0010 does NOT include GRANT_WRITE
✗ NOT AUTHORIZED (dave is just a member, not lead)
```

### Can user:eve access app:backend-api?

```
check_access("user:eve", "app:backend-api", 0x01)

1. Direct grants: grants[user:eve/*/app:backend-api] → none
2. Delegations: none
3. Result: 0x0000
✗ NOT AUTHORIZED (eve only has access to frontend-web)
```

### Can user:frank (new hire) do anything?

```
Frank exists as user:frank but has no grants.
All check_access calls return 0x0000.
Frank needs to be granted relations by someone with GRANT_WRITE.
```

---

## Open Design Questions Discovered

1. **Auto-ownership on create**: Should creators automatically get "owner" relation on entities they create?

2. **Relation validation**: When granting `user:dave/member/team:engineering`, should we verify both entities exist?

3. **Capability inheritance**: Should team members inherit team's capabilities? (Currently they don't - they delegate)

4. **Relation transitivity**: If alice is lead of HR, and HR has admin on _type:user, does alice need explicit delegation or should it be automatic?

5. **Type-specific relations**: Should "lead" and "member" be defined once globally, or per-team as shown?
