# Capbit v2 - User Guide

A simple guide to understanding how Capbit manages permissions and access control.

---

## What is Capbit?

Capbit is a **permission system** that controls who can do what. Think of it like a bouncer at a club - it checks if you're allowed in before letting you through.

```
    You want to enter the office?

    Capbit checks: "Does this person have the 'enter' capability?"

    YES → Go ahead!
    NO  → Access denied.
```

---

## The Two-Layer Capability Model

Capbit has two distinct layers of capabilities:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CAPBIT LAYERS                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   LAYER 1: System Capabilities                                       │
│   ────────────────────────────                                       │
│   Where: _type:user, _type:team, _type:resource, etc.               │
│   What: Controls who can create entities, define grants, etc.        │
│   Who sets: Only root (from bootstrap) and delegatees               │
│   Fixed meanings: TYPE_CREATE, ENTITY_CREATE, GRANT_WRITE, etc.     │
│                                                                      │
│   LAYER 2: Org-Defined Capabilities                                  │
│   ─────────────────────────────────                                  │
│   Where: resource:office, team:sales, app:backend, etc.             │
│   What: Whatever YOUR organization decides                          │
│   Who sets: Anyone with CAP_WRITE on that entity                    │
│   Flexible meanings: You define what each bit means per entity      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Example: Office Access Control

```
On resource:office, your org defines:
  bit0 (0x01) = enter building
  bit1 (0x02) = use printer
  bit2 (0x04) = use fax
  bit3 (0x08) = open safe
  bit4 (0x10) = access server room
  bit5 (0x20) = can grant others access

These meanings are YOUR organization's choice, not Capbit's!
```

---

## Core Concepts

### 1. Entities (Things)

Everything in Capbit is an **entity** with a type:

```
    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
    │  user:alice  │    │  team:sales  │    │  app:myapp   │
    └──────────────┘    └──────────────┘    └──────────────┘
         Person              Group             Application
```

Format: `type:name`

Common types:
- `user:` - People (alice, bob, admin)
- `team:` - Groups (engineering, hr, sales)
- `app:` - Applications (backend, frontend)
- `resource:` - Things to protect (documents, files)

### 2. Capabilities (Actions)

**Capabilities are ACTIONS.** Each capability is a single bit representing something you can DO.

```
    On resource:office, you define what actions are possible:
    ┌─────────────────────────────────────────────────┐
    │  "enter"      = 0x01  (bit0) - enter building   │
    │  "print"      = 0x02  (bit1) - use printer      │
    │  "fax"        = 0x04  (bit2) - use fax machine  │
    │  "safe"       = 0x08  (bit3) - open safe        │
    │  "server"     = 0x10  (bit4) - access server rm │
    │  "can-grant"  = 0x20  (bit5) - grant to others  │
    └─────────────────────────────────────────────────┘

    Each capability = one action = one bit
```

### 3. Grants (Sets of Actions)

**Grants assign capabilities to users.** Multiple grants accumulate via OR to form roles.

```
    ALICE'S GRANTS ON resource:office:
    ┌─────────────────────────────────────────────────┐
    │  Grant: "enter" ─────────────► 0x01             │
    │  Grant: "print" ─────────────► 0x02             │
    │  Grant: "fax"   ─────────────► 0x04             │
    │  ────────────────────────────────────           │
    │  Total (via OR): 0x07 = enter + print + fax     │
    └─────────────────────────────────────────────────┘

    user:alice ──── "enter" ────► resource:office
    user:alice ──── "print" ────► resource:office
    user:alice ──── "fax"   ────► resource:office

    Alice's effective capabilities = 0x07 (all three grants combined)
```

Multiple grants on the same scope accumulate. This is how you build "roles" - by granting multiple actions.

### 4. For System Operations (Layer 1)

On `_type:*` entities, Capbit uses SystemCap with fixed meanings:

```
    ┌─────────────────────────────────────────────────┐
    │  Relation: "admin" on _type:user                │
    ├─────────────────────────────────────────────────┤
    │  ✓ ENTITY_CREATE - Create new users             │
    │  ✓ ENTITY_DELETE - Delete users                 │
    │  ✓ GRANT_WRITE   - Create grants                │
    │  ✓ CAP_WRITE     - Define capabilities          │
    └─────────────────────────────────────────────────┘
```

---

## How It Works: A Visual Story

### The Organization

```
                         ┌─────────────┐
                         │  user:root  │
                         │  (founder)  │
                         └──────┬──────┘
                                │ owns everything
           ┌────────────────────┼────────────────────┐
           ▼                    ▼                    ▼
    ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
    │   team:hr   │      │ team:eng    │      │ team:sales  │
    │             │      │             │      │             │
    │ lead: alice │      │ lead: bob   │      │ lead: charlie│
    └─────────────┘      └─────────────┘      └─────────────┘
                               │
                    ┌──────────┴──────────┐
                    ▼                     ▼
              ┌──────────┐          ┌──────────┐
              │user:dave │          │user:eve  │
              │ (member) │          │ (member) │
              └──────────┘          └──────────┘
```

### Permission Flow

```
    Who can add members to team:engineering?

    ┌─────────────┐     Check      ┌─────────────────┐
    │  user:bob   │ ─────────────► │ team:engineering│
    │             │                │                 │
    │ role: lead  │                │ lead powers:    │
    │             │                │ ✓ GRANT_WRITE   │
    └─────────────┘                └─────────────────┘
           │                              │
           └──────────► ALLOWED! ◄────────┘


    ┌─────────────┐     Check      ┌─────────────────┐
    │ user:dave   │ ─────────────► │ team:engineering│
    │             │                │                 │
    │ role: member│                │ member powers:  │
    │             │                │ ✗ GRANT_WRITE   │
    └─────────────┘                └─────────────────┘
           │                              │
           └──────────► BLOCKED! ◄────────┘
```

---

## Real Example: Running the Demo

Run this command to see Capbit in action:

```bash
cargo test demo_simulation -- --nocapture
```

### What You'll See:

```
══════════════════════════════════════════════════════════
  CAPBIT v2 SIMULATION: Acme Corp Organization
══════════════════════════════════════════════════════════

┌─ STEP 1: System bootstrapped
│  bootstrap("root") → user:root created
│  Root has full admin on all types:
    user:root → _type:user = 0x1ffc
      ✓ ENTITY_CREATE
      ✓ GRANT_WRITE
      ✓ CAP_WRITE

┌─ STEP 2: Root creates teams
│  ✓ team:hr created
│  ✓ team:engineering created
│  ✓ team:sales created

┌─ STEP 3: Root creates users
│  ✓ user:alice created
│  ✓ user:bob created
│  ✓ user:charlie created
│  ✓ user:dave created
│  ✓ user:eve created

┌─ STEP 4: Root defines team roles
│  owner  = full control
│  lead   = can add members
│  member = read only

┌─ STEP 5: Root assigns team leads
│  ✓ Alice → lead of team:hr
│  ✓ Bob → lead of team:engineering
│  ✓ Charlie → lead of team:sales

┌─ STEP 6: Bob adds team members
│  ✓ Bob added Dave as member
│  ✓ Bob added Eve as member

┌─ STEP 7: ATTACK - Dave tries to add members
│  ✓ BLOCKED: Dave doesn't have permission

┌─ STEP 8: HR delegation
│  ✓ Alice can now manage all users

┌─ STEP 9: Alice creates new user
│  ✓ Alice created user:frank

┌─ STEP 10: ATTACK - Alice tries to create team
│  ✓ BLOCKED: Alice can't create teams

══════════════════════════════════════════════════════════
  SIMULATION COMPLETE - All security checks passed!
══════════════════════════════════════════════════════════
```

---

## Scenarios Tested

### Scenario 1: Basic Setup

| Action | Who | Result |
|--------|-----|--------|
| Create root user | System | Root has all powers |
| Create teams | Root | Teams created |
| Create users | Root | Users created |
| Assign leads | Root | Leads assigned |

### Scenario 2: Team Management

```
    Bob (lead) wants to add Dave to engineering

    ┌────────────────────────────────────────┐
    │  Bob's permissions on team:engineering │
    │  ✓ GRANT_WRITE (can add members)       │
    └────────────────────────────────────────┘

    Result: ✓ ALLOWED
```

### Scenario 3: Privilege Escalation Attack

```
    Dave (member) tries to add someone

    ┌────────────────────────────────────────┐
    │  Dave's permissions on team:engineering│
    │  ✗ GRANT_WRITE (cannot add members)    │
    │  ✓ GRANT_READ (can only view)          │
    └────────────────────────────────────────┘

    Result: ✗ BLOCKED - "lacks permission"
```

### Scenario 4: Delegation

```
    Root delegates user management to HR team

    Before:
    ┌─────────────┐                    ┌─────────────┐
    │ user:alice  │ ─── no access ───► │ _type:user  │
    └─────────────┘                    └─────────────┘

    After delegation:
    ┌─────────────┐                    ┌─────────────┐
    │ user:alice  │ ─── inherits ────► │   team:hr   │
    └─────────────┘    permissions     └──────┬──────┘
                                              │
                                              ▼
                                       ┌─────────────┐
                                       │ _type:user  │
                                       │ ✓ CREATE    │
                                       │ ✓ DELETE    │
                                       └─────────────┘

    Now Alice can create users!
```

### Scenario 5: Scope Limitation

```
    Alice (HR) tries to create a team

    Alice has permissions on: _type:user ✓
    Alice has permissions on: _type:team ✗

    Result: ✗ BLOCKED

    "You can manage users, but not teams"
```

---

## All Test Scenarios

Capbit includes **192 tests** covering:

### Security Tests (26 tests)
| Test | What it checks |
|------|---------------|
| Entity Spoofing | Can't create fake entities |
| Self-Grant Escalation | Can't give yourself more power |
| Scope Confusion | Can't grant on wrong target |
| Delegation Amplification | Can't inherit more than source has |
| Bootstrap Replay | Can't re-run setup to become root |
| Circular Delegation | System handles loops safely |
| Type Mutation | Can't create types without permission |
| Unauthorized Deletion | Can't delete what you don't control |
| Non-existent Scope | Can't grant on things that don't exist |
| **Fake SystemCap Bitmask** | Having SystemCap values on your entity doesn't grant system powers |
| **Scope Isolation** | Capabilities are checked on the correct scope, not just any scope |

### Functionality Tests (150+ tests)
| Area | Tests |
|------|-------|
| Permission Boundaries | 16 tests - Capability edge cases |
| Revocation | 11 tests - Permission removal, cascade |
| Authorized Operations | 17 tests - Client abilities (happy path) |
| Input Validation | 18 tests - Edge cases, special chars |
| Inheritance | 12 tests - Diamond, wide, deep patterns |
| Batch Operations | 13 tests - WriteBatch, atomic ops |
| Query Operations | 15 tests - Query completeness |
| Type System | 19 tests - Type lifecycle, permissions |
| Protected API | 23 tests - v2 API authorization |

### Integration & Simulation (11 tests)
| Test | What it checks |
|------|---------------|
| Relationships | Basic connections work |
| Capabilities | Powers are assigned correctly |
| Inheritance | Delegation chains work |
| Acme Corp | Full organization scenario |
| App Access | Application permissions |

---

## Using Capbit in Code

### Initialize

```rust
use capbit::{init, bootstrap};

// Start the database
init("/path/to/database").unwrap();

// Create root user (only runs once!)
bootstrap("admin").unwrap();
```

### Create Structure

```rust
use capbit::protected;

// Create teams (requires ENTITY_CREATE on _type:team)
protected::create_entity("user:admin", "team", "engineering").unwrap();
protected::create_entity("user:admin", "team", "sales").unwrap();

// Create users
protected::create_entity("user:admin", "user", "alice").unwrap();
protected::create_entity("user:admin", "user", "bob").unwrap();
```

### Define Capabilities (Actions)

```rust
use capbit::protected;

// Capabilities are ACTIONS - each with a single bit
// Define what actions are possible on this entity

protected::set_capability(
    "user:admin",           // who's doing this
    "resource:office",      // the entity
    "enter",                // action name
    0x01                    // bit0
).unwrap();

protected::set_capability(
    "user:admin",
    "resource:office",
    "print",                // action name
    0x02                    // bit1
).unwrap();

protected::set_capability(
    "user:admin",
    "resource:office",
    "fax",                  // action name
    0x04                    // bit2
).unwrap();

protected::set_capability(
    "user:admin",
    "resource:office",
    "can-grant",            // action name - ability to grant others
    0x20                    // bit5
).unwrap();
```

### Create Grants (Sets of Actions)

Grants assign capabilities (actions) to users. Multiple grants accumulate via OR.

```rust
// Grant Bob the "enter" action on the office
protected::set_grant(
    "user:admin",        // who's granting
    "user:bob",          // who receives (seeker)
    "enter",             // action/capability name
    "resource:office"    // entity (scope)
).unwrap();

// Grant Bob the "print" action too
protected::set_grant(
    "user:admin",
    "user:bob",
    "print",
    "resource:office"
).unwrap();

// Bob now has 0x03 (enter + print) on resource:office
// Multiple grants accumulate via OR to build his effective permissions

// Alice (with can-grant action) can grant others
protected::set_grant(
    "user:alice",
    "user:charlie",
    "enter",             // Charlie can only enter
    "resource:office"
).unwrap();
```

### Check Permissions

```rust
use capbit::{check_access, has_capability, SystemCap};

// Get all permissions someone has
let caps = check_access("user:bob", "team:engineering", None).unwrap();
println!("Bob's capabilities: 0x{:04x}", caps);

// Check specific permission
if has_capability("user:bob", "team:engineering", SystemCap::GRANT_WRITE).unwrap() {
    println!("Bob can add team members!");
}
```

---

## Permission Bits Reference

### Layer 1: System Capabilities (for `_type:*` scopes only)

These have fixed meanings and are used by the protected API:

| SystemCap | Hex | What it allows |
|-----------|-----|----------------|
| TYPE_CREATE | 0x0001 | Create new types |
| TYPE_DELETE | 0x0002 | Delete types |
| ENTITY_CREATE | 0x0004 | Create entities |
| ENTITY_DELETE | 0x0008 | Delete entities |
| GRANT_READ | 0x0010 | See relationships |
| GRANT_WRITE | 0x0020 | Add relationships |
| GRANT_DELETE | 0x0040 | Remove relationships |
| CAP_READ | 0x0080 | See capability definitions |
| CAP_WRITE | 0x0100 | Define capabilities |
| CAP_DELETE | 0x0200 | Remove capability definitions |
| DELEGATE_READ | 0x0400 | See delegations |
| DELEGATE_WRITE | 0x0800 | Create delegations |
| DELEGATE_DELETE | 0x1000 | Remove delegations |

### Layer 2: Org-Defined Capabilities (for YOUR entities)

**You define what each bit means per entity!**

```
Example: resource:office
  bit0 = enter building
  bit1 = use printer
  bit2 = use fax

Example: app:api-gateway
  bit0 = read API
  bit1 = write API
  bit2 = delete API
  bit3 = bulk operations
```

### Common System Role Combinations

| Role | Bits | Description |
|------|------|-------------|
| ENTITY_ADMIN | 0x1ffc | Full entity management |
| GRANT_ADMIN | 0x0070 | Full relationship control |
| READ_ONLY | 0x0490 | View everything |

---

## Summary

```
┌────────────────────────────────────────────────────────┐
│                    CAPBIT v2                           │
├────────────────────────────────────────────────────────┤
│                                                        │
│  ✓ Three Core Concepts                                 │
│    Entities = things (user:alice, resource:office)     │
│    Capabilities = actions (enter, print, fax - bits)   │
│    Grants = sets of actions assigned to users          │
│                                                        │
│  ✓ Two-Layer Capability Model                          │
│    Layer 1: System caps on _type:* (protected)         │
│    Layer 2: Org-defined caps (you define actions)      │
│                                                        │
│  ✓ Grant Accumulation                                  │
│    Multiple grants OR together to form effective caps  │
│                                                        │
│  ✓ Typed Entities                                      │
│    user:alice, team:sales, app:backend                 │
│                                                        │
│  ✓ Scope Isolation Security                            │
│    Caps on your entity ≠ system powers                 │
│                                                        │
│  ✓ Delegation                                          │
│    Inherited grants (bounded by delegator)             │
│                                                        │
│  ✓ Attack Resistant                                    │
│    192 tests including security scenarios              │
│                                                        │
│  ✓ Fast                                                │
│    O(log N) lookups, O(1) bitmask evaluation           │
│                                                        │
└────────────────────────────────────────────────────────┘
```

---

## Quick Commands

```bash
# Run all tests
cargo test

# Run with output visible
cargo test -- --nocapture

# Run specific test
cargo test demo_simulation -- --nocapture

# Run security tests
cargo test attack

# Build the library
cargo build --release
```

---

*Capbit v2 - Simple, secure, fast permission management.*
