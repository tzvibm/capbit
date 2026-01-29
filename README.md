# Capbit

Entity-relationship bitmask access control system.

## Features

- **O(1) Evaluation**: Bitmask AND operations for permission checks
- **O(log N) Lookup**: LMDB B-tree storage
- **Type Agnostic**: Everything is an entity—users, teams, resources, anything
- **Per-Entity Semantics**: Each entity defines what relationships mean to it
- **Inheritance**: Inherit relationships without graph traversal
- **Policies**: Conditional access based on context (time, location, etc.)
- **Bidirectional**: Query "what can X access" or "who can access X"

## Installation

```bash
npm install
npm run build
```

## Quick Start

```javascript
const capbit = require('./capbit.node');

// Initialize database
capbit.init('./data/capbit.mdb');

// Define constants
const READ = 0x01, WRITE = 0x02, DELETE = 0x04;
const EDITOR = 0x02, VIEWER = 0x01;

// Set up: "editor" on "project42" grants read+write
capbit.setCapability('project42', EDITOR, READ | WRITE);

// John is an editor
capbit.setRelationship('john', EDITOR, 'project42');

// Check access
capbit.hasCapability('john', 'project42', WRITE);  // true
capbit.hasCapability('john', 'project42', DELETE); // false
```

## API

### Initialization

- `init(dbPath)` - Initialize LMDB environment

### Relationships

- `setRelationship(subject, relMask, object)` - Create relationship
- `getRelationships(subject, object)` - Get all relationship masks
- `deleteRelationship(subject, relMask, object)` - Remove relationship

### Capabilities

- `setCapability(entity, relMask, capMask)` - Define what a relationship grants
- `getCapability(entity, relMask)` - Get capability mask for relationship

### Inheritance

- `setInheritance(subject, object, source)` - Subject inherits source's relationship to object
- `getInheritance(subject, object)` - Get inheritance sources

### Labels

- `setRelLabel(entity, relMask, label)` - Human-readable relationship name
- `setCapLabel(entity, capMask, label)` - Human-readable capability name

### Access Checks

- `checkAccess(subject, object, maxDepth?)` - Get effective capability mask
- `hasCapability(subject, object, requiredCap)` - Check specific capability

## License

MIT
