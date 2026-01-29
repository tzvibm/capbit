# Capbit

High-performance access control with string-based relationships and bitmask capabilities.

## Features

- **O(1) Evaluation**: Bitmask AND operations for permission checks
- **O(log N) Lookup**: LMDB B-tree storage
- **String Relationships**: Human-readable types ("editor", "viewer", "member")
- **Type Agnostic**: Everything is an entity—users, teams, resources, anything
- **Per-Entity Semantics**: Each entity defines what relationships mean to it
- **Inheritance**: Inherit relationships without graph traversal
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

// Define capability bits
const READ = 0x01, WRITE = 0x02, DELETE = 0x04;

// Set up: "editor" on "project42" grants read+write
capbit.setCapability('project42', 'editor', READ | WRITE);

// John is an editor
capbit.setRelationship('john', 'editor', 'project42');

// Check access
capbit.hasCapability('john', 'project42', WRITE);  // true
capbit.hasCapability('john', 'project42', DELETE); // false
```

## API

### Initialization

- `init(dbPath)` - Initialize LMDB environment
- `close()` - Close database

### Relationships

- `setRelationship(subject, relType, object)` - Create relationship (relType is a string like "editor")
- `getRelationships(subject, object)` - Get all relationship types as strings
- `deleteRelationship(subject, relType, object)` - Remove relationship

### Capabilities

- `setCapability(entity, relType, capMask)` - Define what a relationship grants
- `getCapability(entity, relType)` - Get capability mask for relationship type

### Inheritance

- `setInheritance(subject, object, source)` - Subject inherits source's relationship to object
- `getInheritance(subject, object)` - Get inheritance sources for subject
- `deleteInheritance(subject, object, source)` - Remove inheritance rule
- `getInheritorsFromSource(source, object)` - Get all subjects inheriting from source
- `getInheritanceForObject(object)` - Get all inheritance rules for object (audit)

### Labels

- `setCapLabel(entity, capBit, label)` - Human-readable capability name
- `getCapLabel(entity, capBit)` - Get capability label

### Access Checks

- `checkAccess(subject, object, maxDepth?)` - Get effective capability mask
- `hasCapability(subject, object, requiredCap)` - Check specific capability

### Query Operations

- `listAccessible(subject)` - List all [object, relType] pairs for subject
- `listSubjects(object)` - List all [subject, relType] pairs for object

### Batch Operations

- `batchSetRelationships(entries)` - Batch set relationships
- `batchSetCapabilities(entries)` - Batch set capabilities
- `batchSetInheritance(entries)` - Batch set inheritance

## License

MIT
