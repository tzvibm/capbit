import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { existsSync, rmSync } from 'fs';
import { createRequire } from 'module';
import path from 'path';

const require = createRequire(import.meta.url);
const capbit = require('../capbit.node');

const TEST_DB_PATH = path.join(import.meta.dirname, '../data/inheritance-test.mdb');

// Capability bits
const READ = 0x01;
const WRITE = 0x02;
const DELETE = 0x04;
const ADMIN = 0x08;
const ALL_CAPS = 0xFF;

describe('deep inheritance tests', () => {
  beforeAll(() => {
    if (existsSync(TEST_DB_PATH)) {
      rmSync(TEST_DB_PATH, { recursive: true });
    }
    capbit.init(TEST_DB_PATH);
  });

  afterAll(() => {
    capbit.close();
    if (existsSync(TEST_DB_PATH)) {
      rmSync(TEST_DB_PATH, { recursive: true });
    }
  });

  describe('inheritance chain depth', () => {
    it('should handle depth 10 inheritance chain', () => {
      const depth = 10;
      const resource = 'resource-depth-10';

      // Setup: entity-0 has direct access
      capbit.setCapability(resource, 'admin', ALL_CAPS);
      capbit.setRelationship('chain10-0', 'admin', resource);

      // Create chain: chain10-1 inherits from chain10-0, etc.
      for (let i = 1; i < depth; i++) {
        capbit.setInheritance(`chain10-${i}`, resource, `chain10-${i - 1}`);
      }

      // Test: entity at end of chain should have access
      const caps = capbit.checkAccess(`chain10-${depth - 1}`, resource);
      expect(caps).toBe(ALL_CAPS);
      expect(capbit.hasCapability(`chain10-${depth - 1}`, resource, READ)).toBe(true);
    });

    it('should handle depth 50 inheritance chain', () => {
      const depth = 50;
      const resource = 'resource-depth-50';

      capbit.setCapability(resource, 'member', 0x0F);
      capbit.setRelationship('chain50-0', 'member', resource);

      for (let i = 1; i < depth; i++) {
        capbit.setInheritance(`chain50-${i}`, resource, `chain50-${i - 1}`);
      }

      const start = performance.now();
      const caps = capbit.checkAccess(`chain50-${depth - 1}`, resource);
      const duration = performance.now() - start;

      console.log(`  Depth 50 access check: ${duration.toFixed(4)}ms`);
      expect(caps).toBe(0x0F);
    });

    it('should handle depth 100 inheritance chain', () => {
      const depth = 100;
      const resource = 'resource-depth-100';

      capbit.setCapability(resource, 'viewer', 0x07);
      capbit.setRelationship('chain100-0', 'viewer', resource);

      for (let i = 1; i < depth; i++) {
        capbit.setInheritance(`chain100-${i}`, resource, `chain100-${i - 1}`);
      }

      const start = performance.now();
      const caps = capbit.checkAccess(`chain100-${depth - 1}`, resource);
      const duration = performance.now() - start;

      console.log(`  Depth 100 access check: ${duration.toFixed(4)}ms`);
      expect(caps).toBe(0x07);
    });

    it('should handle depth 100 with repeated checks (performance)', () => {
      const resource = 'resource-depth-100';
      const iterations = 1000;

      const start = performance.now();
      for (let i = 0; i < iterations; i++) {
        capbit.checkAccess('chain100-99', resource);
      }
      const duration = performance.now() - start;

      console.log(`  ${iterations} checks at depth 100: ${duration.toFixed(2)}ms total`);
      console.log(`  Average: ${(duration / iterations).toFixed(4)}ms per check`);

      expect(duration / iterations).toBeLessThan(10);
    });
  });

  describe('inheritance graph (multiple parents)', () => {
    it('should handle diamond inheritance pattern', () => {
      const resource = 'diamond-resource';

      // Diamond: D inherits from B and C, both inherit from A
      //       A (has "reader")
      //      / \
      //     B   C (B has "writer", C has "deleter")
      //      \ /
      //       D

      capbit.setCapability(resource, 'reader', READ);
      capbit.setCapability(resource, 'writer', WRITE);
      capbit.setCapability(resource, 'deleter', DELETE);

      capbit.setRelationship('diamond-A', 'reader', resource);
      capbit.setRelationship('diamond-B', 'writer', resource);
      capbit.setRelationship('diamond-C', 'deleter', resource);

      capbit.setInheritance('diamond-B', resource, 'diamond-A');
      capbit.setInheritance('diamond-C', resource, 'diamond-A');
      capbit.setInheritance('diamond-D', resource, 'diamond-B');
      capbit.setInheritance('diamond-D', resource, 'diamond-C');

      // D should have READ + WRITE + DELETE
      const caps = capbit.checkAccess('diamond-D', resource);
      expect(caps & READ).toBeTruthy();
      expect(caps & WRITE).toBeTruthy();
      expect(caps & DELETE).toBeTruthy();
      console.log(`  Diamond inheritance caps: 0x${caps.toString(16)} (expected: 0x07)`);
    });

    it('should handle wide inheritance (many parents)', () => {
      const resource = 'wide-resource';
      const relTypes = ['reader', 'writer', 'editor', 'admin', 'owner', 'manager', 'contributor', 'reviewer'];

      // Entity inherits from 20 different sources
      for (let i = 0; i < 20; i++) {
        const relType = relTypes[i % relTypes.length];
        const capBit = 1 << (i % 8);
        capbit.setCapability(resource, relType, capBit);
        capbit.setRelationship(`wide-source-${i}`, relType, resource);
        capbit.setInheritance('wide-child', resource, `wide-source-${i}`);
      }

      const start = performance.now();
      const caps = capbit.checkAccess('wide-child', resource);
      const duration = performance.now() - start;

      console.log(`  Wide inheritance (20 parents) check: ${duration.toFixed(4)}ms`);
      console.log(`  Effective caps: 0x${caps.toString(16)}`);
      expect(caps).toBe(0xFF);
    });
  });

  describe('cycle detection', () => {
    it('should handle circular inheritance without infinite loop', () => {
      const resource = 'cycle-resource';

      capbit.setCapability(resource, 'member', 0x0F);
      capbit.setRelationship('cycle-A', 'member', resource);

      // Create cycle: A -> B -> C -> A
      capbit.setInheritance('cycle-B', resource, 'cycle-A');
      capbit.setInheritance('cycle-C', resource, 'cycle-B');
      capbit.setInheritance('cycle-A', resource, 'cycle-C'); // Creates cycle!

      const start = performance.now();
      const caps = capbit.checkAccess('cycle-C', resource);
      const duration = performance.now() - start;

      console.log(`  Cycle detection check: ${duration.toFixed(4)}ms`);
      expect(caps).toBe(0x0F);
      expect(duration).toBeLessThan(100);
    });
  });

  describe('depth limit enforcement', () => {
    it('should respect max_depth parameter', () => {
      const resource = 'limited-resource';

      capbit.setCapability(resource, 'owner', ALL_CAPS);
      capbit.setRelationship('limited-0', 'owner', resource);

      // Create 20-deep chain
      for (let i = 1; i < 20; i++) {
        capbit.setInheritance(`limited-${i}`, resource, `limited-${i - 1}`);
      }

      // With max_depth=5, entity at depth 10 should NOT have access
      const capsLimited = capbit.checkAccess('limited-10', resource, 5);
      const capsUnlimited = capbit.checkAccess('limited-10', resource, 100);

      console.log(`  Depth 10 with max_depth=5: 0x${capsLimited.toString(16)}`);
      console.log(`  Depth 10 with max_depth=100: 0x${capsUnlimited.toString(16)}`);

      expect(capsLimited).toBe(0);
      expect(capsUnlimited).toBe(ALL_CAPS);
    });
  });

  describe('inheritance query operations', () => {
    it('should get inheritors from source', () => {
      const resource = 'query-resource-1';

      // Setup: source has access, multiple subjects inherit from it
      capbit.setCapability(resource, 'admin', ALL_CAPS);
      capbit.setRelationship('query-source', 'admin', resource);

      capbit.setInheritance('query-inheritor-1', resource, 'query-source');
      capbit.setInheritance('query-inheritor-2', resource, 'query-source');
      capbit.setInheritance('query-inheritor-3', resource, 'query-source');

      const inheritors = capbit.getInheritorsFromSource('query-source', resource);

      expect(inheritors).toContain('query-inheritor-1');
      expect(inheritors).toContain('query-inheritor-2');
      expect(inheritors).toContain('query-inheritor-3');
      expect(inheritors.length).toBe(3);
    });

    it('should get all inheritance rules for an object', () => {
      const resource = 'query-resource-2';

      // Setup: multiple inheritance rules for the same object
      capbit.setCapability(resource, 'editor', WRITE);
      capbit.setRelationship('obj-source-A', 'editor', resource);
      capbit.setRelationship('obj-source-B', 'editor', resource);

      capbit.setInheritance('obj-child-1', resource, 'obj-source-A');
      capbit.setInheritance('obj-child-2', resource, 'obj-source-A');
      capbit.setInheritance('obj-child-3', resource, 'obj-source-B');

      const rules = capbit.getInheritanceForObject(resource);

      expect(rules.length).toBe(3);

      // Each rule is [source, subject]
      const ruleStrings = rules.map(([src, subj]) => `${subj}<-${src}`);
      expect(ruleStrings).toContain('obj-child-1<-obj-source-A');
      expect(ruleStrings).toContain('obj-child-2<-obj-source-A');
      expect(ruleStrings).toContain('obj-child-3<-obj-source-B');
    });

    it('should delete inheritance rules', () => {
      const resource = 'delete-resource';

      capbit.setCapability(resource, 'member', READ);
      capbit.setRelationship('delete-source', 'member', resource);
      capbit.setInheritance('delete-child', resource, 'delete-source');

      // Verify access before delete
      expect(capbit.hasCapability('delete-child', resource, READ)).toBe(true);

      // Delete the inheritance
      const deleted = capbit.deleteInheritance('delete-child', resource, 'delete-source');
      expect(deleted).toBe(true);

      // Verify no access after delete
      expect(capbit.hasCapability('delete-child', resource, READ)).toBe(false);

      // Verify it's gone from all indexes
      const sources = capbit.getInheritance('delete-child', resource);
      expect(sources).not.toContain('delete-source');

      const inheritors = capbit.getInheritorsFromSource('delete-source', resource);
      expect(inheritors).not.toContain('delete-child');

      const rules = capbit.getInheritanceForObject(resource);
      const hasRule = rules.some(([src, subj]) => src === 'delete-source' && subj === 'delete-child');
      expect(hasRule).toBe(false);
    });
  });
});
