import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { existsSync, rmSync } from 'fs';
import path from 'path';

// Will work once native module is built
// import capbit from '../index.js';

const TEST_DB_PATH = path.join(import.meta.dirname, '../data/test.mdb');

describe('capbit', () => {
  // beforeAll(() => {
  //   capbit.init(TEST_DB_PATH);
  // });

  // afterAll(() => {
  //   capbit.close();
  //   if (existsSync(TEST_DB_PATH)) {
  //     rmSync(TEST_DB_PATH, { recursive: true });
  //   }
  // });

  describe('relationships', () => {
    it.skip('should set and get relationships', () => {
      // const READ = 0x01;
      // const WRITE = 0x02;
      // const EDITOR = 0x02;

      // capbit.setRelationship('john', EDITOR, 'project42');
      // const rels = capbit.getRelationships('john', 'project42');

      // expect(rels).toContain(EDITOR);
    });

    it.skip('should delete relationships', () => {
      // const VIEWER = 0x01;

      // capbit.setRelationship('bob', VIEWER, 'project42');
      // capbit.deleteRelationship('bob', VIEWER, 'project42');
      // const rels = capbit.getRelationships('bob', 'project42');

      // expect(rels).not.toContain(VIEWER);
    });
  });

  describe('capabilities', () => {
    it.skip('should define per-entity capability semantics', () => {
      // const EDITOR = 0x02;
      // const READ = 0x01;
      // const WRITE = 0x02;

      // // Same relationship, different capabilities per entity
      // capbit.setCapability('slack', EDITOR, READ | WRITE | 0x04 | 0x08);  // full access
      // capbit.setCapability('github', EDITOR, READ | WRITE);  // limited

      // const slackCaps = capbit.getCapability('slack', EDITOR);
      // const githubCaps = capbit.getCapability('github', EDITOR);

      // expect(slackCaps).toBe(0x0F);
      // expect(githubCaps).toBe(0x03);
    });
  });

  describe('inheritance', () => {
    it.skip('should inherit relationships', () => {
      // const ADMIN = 0x04;
      // const ALL_CAPS = 0x0F;

      // // Mary is admin on sales
      // capbit.setRelationship('mary', ADMIN, 'sales');
      // capbit.setCapability('sales', ADMIN, ALL_CAPS);

      // // John inherits mary's relationship to sales
      // capbit.setInheritance('john', 'sales', 'mary');

      // const sources = capbit.getInheritance('john', 'sales');
      // expect(sources).toContain('mary');
    });
  });

  describe('access checks', () => {
    it.skip('should check direct access', () => {
      // const EDITOR = 0x02;
      // const READ = 0x01;
      // const WRITE = 0x02;

      // capbit.setCapability('docs', EDITOR, READ | WRITE);
      // capbit.setRelationship('alice', EDITOR, 'docs');

      // expect(capbit.hasCapability('alice', 'docs', READ)).toBe(true);
      // expect(capbit.hasCapability('alice', 'docs', WRITE)).toBe(true);
      // expect(capbit.hasCapability('alice', 'docs', 0x04)).toBe(false);
    });

    it.skip('should check inherited access', () => {
      // const ADMIN = 0x04;
      // const ALL_CAPS = 0x0F;

      // capbit.setCapability('team-resource', ADMIN, ALL_CAPS);
      // capbit.setRelationship('team-lead', ADMIN, 'team-resource');
      // capbit.setInheritance('team-member', 'team-resource', 'team-lead');

      // const caps = capbit.checkAccess('team-member', 'team-resource');
      // expect(caps & 0x04).toBeTruthy();  // has DELETE
    });
  });

  describe('labels', () => {
    it.skip('should store human-readable labels', () => {
      // capbit.setRelLabel('myapp', 0x02, 'editor');
      // capbit.setCapLabel('myapp', 0x03, 'read,write');
      // Labels are for debugging/display, not returned by current API
    });
  });
});
