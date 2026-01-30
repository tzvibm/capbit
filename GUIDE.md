# Capbit v2 - User Guide

A simple guide to understanding how Capbit manages permissions and access control.

---

## What is Capbit?

Capbit is a **permission system** that controls who can do what. Think of it like a bouncer at a club - it checks if you're allowed in before letting you through.

```
    You want to edit a document?

    Capbit checks: "Does this person have edit permission?"

    YES → Go ahead!
    NO  → Access denied.
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

### 2. Relationships (Roles)

Relationships connect entities with a role:

```
    user:alice ──── "lead" ────► team:hr

    "Alice is the lead of the HR team"
```

Common roles:
- `owner` - Full control
- `admin` - Can manage
- `lead` - Can add members
- `member` - Basic access
- `viewer` - Read only

### 3. Capabilities (What You Can Do)

Each role grants specific powers:

```
    ┌─────────────────────────────────────────────────┐
    │  Role: "lead" on team:engineering               │
    ├─────────────────────────────────────────────────┤
    │  ✓ GRANT_READ   - See who's on the team        │
    │  ✓ GRANT_WRITE  - Add new members              │
    │  ✗ CAP_WRITE    - Cannot change role powers    │
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

Capbit includes **47 tests** covering:

### Security Tests (9 tests)
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

### Functionality Tests (23 tests)
| Area | Tests |
|------|-------|
| Bootstrap | 6 tests - System initialization |
| Entities | 4 tests - Create, delete, validation |
| Grants | 3 tests - Adding/removing relationships |
| Capabilities | 2 tests - Defining role powers |
| Delegations | 3 tests - Inheritance system |
| Access Checks | 5 tests - Permission evaluation |

### Integration Tests (9 tests)
| Test | What it checks |
|------|---------------|
| Relationships | Basic connections work |
| Capabilities | Powers are assigned correctly |
| Inheritance | Delegation chains work |
| Batch Operations | Bulk updates work |
| Query Operations | Searching works |
| Cycle Detection | Handles circular references |

### Simulation Tests (2 tests)
| Test | What it checks |
|------|---------------|
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

### Define Roles

```rust
use capbit::{protected, SystemCap};

// Define what "lead" means on this team
protected::set_capability(
    "user:admin",           // who's doing this
    "team:engineering",     // where
    "lead",                 // role name
    SystemCap::GRANT_WRITE | SystemCap::GRANT_READ  // powers
).unwrap();

// Define "member" role
protected::set_capability(
    "user:admin",
    "team:engineering",
    "member",
    SystemCap::GRANT_READ   // read-only
).unwrap();
```

### Assign Roles

```rust
// Make bob the lead
protected::set_grant(
    "user:admin",        // who's granting
    "user:bob",          // who receives
    "lead",              // role
    "team:engineering"   // where
).unwrap();

// Add alice as member
protected::set_grant(
    "user:bob",          // bob can do this (he's lead)
    "user:alice",
    "member",
    "team:engineering"
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

| Capability | Hex | What it allows |
|------------|-----|----------------|
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

### Common Combinations

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
│  ✓ Protected Mutations                                 │
│    Every change requires permission                    │
│                                                        │
│  ✓ Typed Entities                                      │
│    user:alice, team:sales, app:backend                 │
│                                                        │
│  ✓ Flexible Roles                                      │
│    Define what each role can do per-entity             │
│                                                        │
│  ✓ Delegation                                          │
│    Pass your permissions to others (bounded)           │
│                                                        │
│  ✓ Attack Resistant                                    │
│    47 tests including security scenarios               │
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
