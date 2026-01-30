import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { existsSync, rmSync } from 'fs';
import { createRequire } from 'module';
import path from 'path';

const require = createRequire(import.meta.url);
const capbit = require('../capbit.node');

const TEST_DB_PATH = path.join(import.meta.dirname, '../data/test.mdb');

// Capability bits (still bitmasks for O(1) evaluation)
const READ = 0x01;
const WRITE = 0x02;
const DELETE = 0x04;
const ADMIN = 0x08;

describe('capbit', () => {
  beforeAll(() => {
    capbit.init(TEST_DB_PATH);
  });

  afterAll(() => {
    capbit.close();
    if (existsSync(TEST_DB_PATH)) {
      rmSync(TEST_DB_PATH, { recursive: true });
    }
  });

  describe('relationships (string-based)', () => {
    it('should set and get relationships', () => {
      capbit.setRelationship('john', 'editor', 'project42');
      const rels = capbit.getRelationships('john', 'project42');

      expect(rels).toContain('editor');
    });

    it('should handle multiple relationship types', () => {
      capbit.setRelationship('jane', 'editor', 'doc1');
      capbit.setRelationship('jane', 'viewer', 'doc1');
      capbit.setRelationship('jane', 'commenter', 'doc1');

      const rels = capbit.getRelationships('jane', 'doc1');
      expect(rels).toContain('editor');
      expect(rels).toContain('viewer');
      expect(rels).toContain('commenter');
      expect(rels.length).toBe(3);
    });

    it('should delete relationships', () => {
      capbit.setRelationship('bob', 'viewer', 'project42');
      capbit.deleteRelationship('bob', 'viewer', 'project42');
      const rels = capbit.getRelationships('bob', 'project42');

      expect(rels).not.toContain('viewer');
    });
  });

  describe('capabilities', () => {
    it('should define per-entity capability semantics', () => {
      // Same relationship type, different capabilities per entity
      capbit.setCapability('slack', 'editor', READ | WRITE | DELETE | ADMIN);
      capbit.setCapability('github', 'editor', READ | WRITE);

      const slackCaps = capbit.getCapability('slack', 'editor');
      const githubCaps = capbit.getCapability('github', 'editor');

      expect(slackCaps).toBe(0x0F);
      expect(githubCaps).toBe(0x03);
    });
  });

  describe('inheritance', () => {
    it('should inherit relationships', () => {
      // Mary is admin on sales
      capbit.setRelationship('mary', 'admin', 'sales');
      capbit.setCapability('sales', 'admin', READ | WRITE | DELETE | ADMIN);

      // John inherits mary's relationship to sales
      capbit.setInheritance('john-inherit', 'sales', 'mary');

      const sources = capbit.getInheritance('john-inherit', 'sales');
      expect(sources).toContain('mary');
    });
  });

  describe('access checks', () => {
    it('should check direct access', () => {
      capbit.setCapability('docs', 'editor', READ | WRITE);
      capbit.setRelationship('alice', 'editor', 'docs');

      expect(capbit.hasCapability('alice', 'docs', READ)).toBe(true);
      expect(capbit.hasCapability('alice', 'docs', WRITE)).toBe(true);
      expect(capbit.hasCapability('alice', 'docs', DELETE)).toBe(false);
    });

    it('should check inherited access', () => {
      capbit.setCapability('team-resource', 'admin', READ | WRITE | DELETE | ADMIN);
      capbit.setRelationship('team-lead', 'admin', 'team-resource');
      capbit.setInheritance('team-member', 'team-resource', 'team-lead');

      const caps = capbit.checkAccess('team-member', 'team-resource');
      expect(caps & DELETE).toBeTruthy();
    });
  });

  describe('labels', () => {
    it('should store human-readable labels for capability bits', () => {
      capbit.setCapLabel('myapp', READ, 'read');
      capbit.setCapLabel('myapp', WRITE, 'write');

      const readLabel = capbit.getCapLabel('myapp', READ);
      const writeLabel = capbit.getCapLabel('myapp', WRITE);

      expect(readLabel).toBe('read');
      expect(writeLabel).toBe('write');
    });
  });

  describe('query operations', () => {
    it('should list accessible entities for a subject', () => {
      capbit.setRelationship('query-user', 'member', 'org1');
      capbit.setRelationship('query-user', 'viewer', 'doc1');
      capbit.setRelationship('query-user', 'editor', 'doc2');

      const accessible = capbit.listAccessible('query-user');
      expect(accessible.length).toBeGreaterThanOrEqual(3);

      const objects = accessible.map(([obj, _rel]) => obj);
      expect(objects).toContain('org1');
      expect(objects).toContain('doc1');
      expect(objects).toContain('doc2');
    });

    it('should list subjects with access to an object', () => {
      capbit.setRelationship('user1', 'editor', 'shared-doc');
      capbit.setRelationship('user2', 'viewer', 'shared-doc');
      capbit.setRelationship('user3', 'commenter', 'shared-doc');

      const subjects = capbit.listSubjects('shared-doc');
      expect(subjects.length).toBeGreaterThanOrEqual(3);

      const users = subjects.map(([subj, _rel]) => subj);
      expect(users).toContain('user1');
      expect(users).toContain('user2');
      expect(users).toContain('user3');
    });
  });

  describe('WriteBatch (explicit transactions)', () => {
    it('should execute multiple operations atomically', () => {
      const batch = new capbit.WriteBatch();

      // Add multiple operations
      batch.setCapability('batch-resource', 'admin', READ | WRITE | DELETE | ADMIN);
      batch.setRelationship('batch-user1', 'admin', 'batch-resource');
      batch.setRelationship('batch-user2', 'editor', 'batch-resource');
      batch.setCapability('batch-resource', 'editor', READ | WRITE);

      expect(batch.length).toBe(4);

      // Execute all in one transaction
      const epoch = batch.execute();
      expect(epoch).toBeGreaterThan(0);

      // Verify all operations applied
      expect(capbit.hasCapability('batch-user1', 'batch-resource', ADMIN)).toBe(true);
      expect(capbit.hasCapability('batch-user2', 'batch-resource', WRITE)).toBe(true);
      expect(capbit.hasCapability('batch-user2', 'batch-resource', ADMIN)).toBe(false);
    });

    it('should support method chaining', () => {
      const batch = new capbit.WriteBatch();

      batch
        .setCapability('chain-res', 'owner', READ | WRITE | DELETE)
        .setRelationship('chain-user', 'owner', 'chain-res')
        .setInheritance('chain-inheritor', 'chain-res', 'chain-user');

      expect(batch.length).toBe(3);
      batch.execute();

      // Verify inherited access
      expect(capbit.hasCapability('chain-inheritor', 'chain-res', DELETE)).toBe(true);
    });

    it('should clear operations', () => {
      const batch = new capbit.WriteBatch();
      batch.setRelationship('a', 'b', 'c');
      expect(batch.length).toBe(1);

      batch.clear();
      expect(batch.length).toBe(0);
    });

    it('should handle delete operations in batch', () => {
      // Setup
      capbit.setCapability('del-batch-res', 'member', READ);
      capbit.setRelationship('del-batch-user', 'member', 'del-batch-res');
      expect(capbit.hasCapability('del-batch-user', 'del-batch-res', READ)).toBe(true);

      // Delete in batch
      const batch = new capbit.WriteBatch();
      batch.deleteRelationship('del-batch-user', 'member', 'del-batch-res');
      batch.execute();

      expect(capbit.hasCapability('del-batch-user', 'del-batch-res', READ)).toBe(false);
    });
  });
});
