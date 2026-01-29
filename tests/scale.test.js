import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { existsSync, rmSync, statSync } from 'fs';
import { createRequire } from 'module';
import path from 'path';

const require = createRequire(import.meta.url);
const capbit = require('../capbit.node');

const TEST_DB_PATH = path.join(import.meta.dirname, '../data/scale-test.mdb');

const SCALE = {
  mode: process.env.SCALE_TEST_MODE || 'quick',

  get config() {
    const configs = {
      quick: {
        entityCount: 10_000,
        relationshipsPerEntity: 50,
        resourceCount: 1_000,
        batchSize: 10_000,
        sampleSize: 1_000,
      },
      medium: {
        entityCount: 1_000_000,
        relationshipsPerEntity: 50,
        resourceCount: 10_000,
        batchSize: 50_000,
        sampleSize: 10_000,
      },
      full: {
        entityCount: 100_000_000,
        relationshipsPerEntity: 50,
        resourceCount: 1_000_000,
        batchSize: 100_000,
        sampleSize: 100_000,
      },
    };
    return configs[this.mode] || configs.quick;
  }
};

function formatNumber(n) {
  return n.toLocaleString();
}

function formatBytes(bytes) {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  while (bytes >= 1024 && i < units.length - 1) {
    bytes /= 1024;
    i++;
  }
  return `${bytes.toFixed(2)} ${units[i]}`;
}

function formatDuration(ms) {
  if (ms < 1000) return `${ms.toFixed(2)}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
  return `${(ms / 60000).toFixed(2)}min`;
}

function getDbSize() {
  try {
    const dataFile = path.join(TEST_DB_PATH, 'data.mdb');
    if (existsSync(dataFile)) {
      return statSync(dataFile).size;
    }
    if (existsSync(TEST_DB_PATH)) {
      const stat = statSync(TEST_DB_PATH);
      return stat.isDirectory() ? 0 : stat.size;
    }
  } catch {
    return 0;
  }
  return 0;
}

// Relationship types (now strings!)
const REL_TYPES = ['viewer', 'editor', 'admin', 'owner', 'member', 'manager', 'contributor', 'reviewer'];

function randomRelType() {
  return REL_TYPES[Math.floor(Math.random() * REL_TYPES.length)];
}

function randomCapMask() {
  return Math.floor(Math.random() * 0xFF) + 1;
}

describe('capbit scale tests', () => {
  const config = SCALE.config;

  beforeAll(() => {
    console.log('\n' + '='.repeat(60));
    console.log(`SCALE TEST MODE: ${SCALE.mode.toUpperCase()}`);
    console.log(`Entities: ${formatNumber(config.entityCount)}`);
    console.log(`Relationships per entity: ${config.relationshipsPerEntity}`);
    console.log(`Total relationships: ${formatNumber(config.entityCount * config.relationshipsPerEntity)}`);
    console.log(`Resources: ${formatNumber(config.resourceCount)}`);
    console.log(`Batch size: ${formatNumber(config.batchSize)}`);
    console.log('='.repeat(60) + '\n');

    if (existsSync(TEST_DB_PATH)) {
      rmSync(TEST_DB_PATH, { recursive: true });
    }

    capbit.init(TEST_DB_PATH);
  });

  afterAll(() => {
    const finalSize = getDbSize();
    console.log(`\nFinal database size: ${formatBytes(finalSize)}`);

    capbit.close();

    if (existsSync(TEST_DB_PATH)) {
      rmSync(TEST_DB_PATH, { recursive: true });
    }
  });

  describe('write performance (batched)', () => {
    it('should handle bulk capability definitions', () => {
      const start = performance.now();
      const batch = [];

      for (let r = 0; r < config.resourceCount; r++) {
        const resourceId = `resource-${r}`;
        for (const relType of REL_TYPES) {
          batch.push([resourceId, relType, String(randomCapMask())]);
        }
      }

      const count = capbit.batchSetCapabilities(batch);
      const duration = performance.now() - start;
      const opsPerSec = (count / duration) * 1000;

      console.log(`  Capability definitions: ${formatNumber(count)} in ${formatDuration(duration)}`);
      console.log(`  Throughput: ${formatNumber(Math.round(opsPerSec))} ops/sec`);

      expect(count).toBe(config.resourceCount * REL_TYPES.length);
    });

    it('should handle bulk relationship writes', () => {
      const start = performance.now();
      let totalRelationships = 0;
      let batch = [];
      let batchStart = performance.now();

      for (let e = 0; e < config.entityCount; e++) {
        const entityId = `entity-${e}`;

        for (let r = 0; r < config.relationshipsPerEntity; r++) {
          const resourceIdx = Math.floor(Math.random() * config.resourceCount);
          const resourceId = `resource-${resourceIdx}`;
          const relType = randomRelType();

          batch.push([entityId, relType, resourceId]);
        }

        if (batch.length >= config.batchSize) {
          const count = capbit.batchSetRelationships(batch);
          totalRelationships += count;

          const batchDuration = performance.now() - batchStart;
          const progress = ((e + 1) / config.entityCount * 100).toFixed(1);
          const batchOpsPerSec = (count / batchDuration) * 1000;

          console.log(`  Progress: ${progress}% (${formatNumber(totalRelationships)} rels, ${formatNumber(Math.round(batchOpsPerSec))} rel/sec)`);

          batch = [];
          batchStart = performance.now();
        }
      }

      if (batch.length > 0) {
        const count = capbit.batchSetRelationships(batch);
        totalRelationships += count;
      }

      const duration = performance.now() - start;
      const opsPerSec = (totalRelationships / duration) * 1000;

      console.log(`\n  Total relationships written: ${formatNumber(totalRelationships)}`);
      console.log(`  Total duration: ${formatDuration(duration)}`);
      console.log(`  Average throughput: ${formatNumber(Math.round(opsPerSec))} ops/sec`);
      console.log(`  Database size: ${formatBytes(getDbSize())}`);

      expect(totalRelationships).toBe(config.entityCount * config.relationshipsPerEntity);
    });

    it('should handle bulk inheritance writes', () => {
      const start = performance.now();
      const inheritanceCount = Math.floor(config.entityCount / 10);
      const batch = [];

      for (let i = 0; i < inheritanceCount; i++) {
        const entityIdx = Math.floor(Math.random() * config.entityCount);
        const sourceIdx = Math.floor(Math.random() * config.entityCount);
        const resourceIdx = Math.floor(Math.random() * config.resourceCount);

        if (entityIdx !== sourceIdx) {
          batch.push([
            `entity-${entityIdx}`,
            `resource-${resourceIdx}`,
            `entity-${sourceIdx}`
          ]);
        }
      }

      const count = capbit.batchSetInheritance(batch);
      const duration = performance.now() - start;
      const opsPerSec = (count / duration) * 1000;

      console.log(`  Inheritance rules written: ${formatNumber(count)}`);
      console.log(`  Duration: ${formatDuration(duration)}`);
      console.log(`  Throughput: ${formatNumber(Math.round(opsPerSec))} ops/sec`);

      expect(count).toBeGreaterThan(0);
    });
  });

  describe('read performance', () => {
    it('should perform fast relationship lookups', () => {
      const lookups = config.sampleSize;
      const times = [];

      for (let i = 0; i < lookups; i++) {
        const entityIdx = Math.floor(Math.random() * config.entityCount);
        const resourceIdx = Math.floor(Math.random() * config.resourceCount);

        const start = performance.now();
        capbit.getRelationships(`entity-${entityIdx}`, `resource-${resourceIdx}`);
        times.push(performance.now() - start);
      }

      const totalDuration = times.reduce((a, b) => a + b, 0);
      const avgTime = totalDuration / lookups;
      const maxTime = Math.max(...times);
      const minTime = Math.min(...times);
      const sorted = [...times].sort((a, b) => a - b);
      const p99 = sorted[Math.floor(lookups * 0.99)];

      console.log(`  Relationship lookups: ${formatNumber(lookups)}`);
      console.log(`  Average: ${avgTime.toFixed(4)}ms`);
      console.log(`  Min: ${minTime.toFixed(4)}ms`);
      console.log(`  Max: ${maxTime.toFixed(4)}ms`);
      console.log(`  P99: ${p99.toFixed(4)}ms`);
      console.log(`  Throughput: ${formatNumber(Math.round((lookups / totalDuration) * 1000))} ops/sec`);

      expect(avgTime).toBeLessThan(10);
    });

    it('should perform fast access checks', () => {
      const checks = config.sampleSize;
      const times = [];
      let hasAccess = 0;

      for (let i = 0; i < checks; i++) {
        const entityIdx = Math.floor(Math.random() * config.entityCount);
        const resourceIdx = Math.floor(Math.random() * config.resourceCount);

        const start = performance.now();
        const caps = capbit.checkAccess(`entity-${entityIdx}`, `resource-${resourceIdx}`);
        times.push(performance.now() - start);

        if (caps > 0) hasAccess++;
      }

      const totalDuration = times.reduce((a, b) => a + b, 0);
      const avgTime = totalDuration / checks;
      const maxTime = Math.max(...times);
      const sorted = [...times].sort((a, b) => a - b);
      const p99 = sorted[Math.floor(checks * 0.99)];

      console.log(`  Access checks: ${formatNumber(checks)}`);
      console.log(`  Average: ${avgTime.toFixed(4)}ms`);
      console.log(`  Max: ${maxTime.toFixed(4)}ms`);
      console.log(`  P99: ${p99.toFixed(4)}ms`);
      console.log(`  Throughput: ${formatNumber(Math.round((checks / totalDuration) * 1000))} ops/sec`);
      console.log(`  Hit rate: ${((hasAccess / checks) * 100).toFixed(1)}%`);

      expect(avgTime).toBeLessThan(10);
    });

    it('should perform fast hasCapability checks', () => {
      const checks = config.sampleSize;
      const times = [];
      let granted = 0;

      for (let i = 0; i < checks; i++) {
        const entityIdx = Math.floor(Math.random() * config.entityCount);
        const resourceIdx = Math.floor(Math.random() * config.resourceCount);
        const capBit = 1 << (Math.floor(Math.random() * 8));

        const start = performance.now();
        const result = capbit.hasCapability(
          `entity-${entityIdx}`,
          `resource-${resourceIdx}`,
          capBit
        );
        times.push(performance.now() - start);

        if (result) granted++;
      }

      const totalDuration = times.reduce((a, b) => a + b, 0);
      const avgTime = totalDuration / checks;
      const sorted = [...times].sort((a, b) => a - b);
      const p99 = sorted[Math.floor(checks * 0.99)];

      console.log(`  hasCapability checks: ${formatNumber(checks)}`);
      console.log(`  Average: ${avgTime.toFixed(4)}ms`);
      console.log(`  P99: ${p99.toFixed(4)}ms`);
      console.log(`  Throughput: ${formatNumber(Math.round((checks / totalDuration) * 1000))} ops/sec`);
      console.log(`  Grant rate: ${((granted / checks) * 100).toFixed(1)}%`);

      expect(avgTime).toBeLessThan(10);
    });
  });

  describe('mixed workload performance', () => {
    it('should handle concurrent read/write patterns', () => {
      const operations = config.sampleSize;
      const readRatio = 0.9;
      const times = { read: [], write: [] };

      for (let i = 0; i < operations; i++) {
        const entityIdx = Math.floor(Math.random() * config.entityCount);
        const resourceIdx = Math.floor(Math.random() * config.resourceCount);

        if (Math.random() < readRatio) {
          const start = performance.now();
          capbit.checkAccess(`entity-${entityIdx}`, `resource-${resourceIdx}`);
          times.read.push(performance.now() - start);
        } else {
          const start = performance.now();
          capbit.setRelationship(
            `entity-${entityIdx}`,
            randomRelType(),
            `resource-${resourceIdx}`
          );
          times.write.push(performance.now() - start);
        }
      }

      const readAvg = times.read.reduce((a, b) => a + b, 0) / times.read.length;
      const writeAvg = times.write.reduce((a, b) => a + b, 0) / times.write.length;

      console.log(`  Mixed operations: ${formatNumber(operations)}`);
      console.log(`  Reads: ${formatNumber(times.read.length)} (avg: ${readAvg.toFixed(4)}ms)`);
      console.log(`  Writes: ${formatNumber(times.write.length)} (avg: ${writeAvg.toFixed(4)}ms)`);

      expect(readAvg).toBeLessThan(10);
      expect(writeAvg).toBeLessThan(20);
    });
  });

  describe('edge cases at scale', () => {
    it('should handle entities with maximum relationships', () => {
      const heavyEntityId = 'heavy-entity';
      const relCount = Math.min(config.resourceCount, 10000);
      const batch = [];

      for (let r = 0; r < relCount; r++) {
        batch.push([heavyEntityId, randomRelType(), `resource-${r}`]);
      }

      const start = performance.now();
      capbit.batchSetRelationships(batch);
      const writeDuration = performance.now() - start;

      const checkStart = performance.now();
      for (let i = 0; i < 100; i++) {
        const resourceIdx = Math.floor(Math.random() * config.resourceCount);
        capbit.checkAccess(heavyEntityId, `resource-${resourceIdx}`);
      }
      const checkDuration = performance.now() - checkStart;

      console.log(`  Heavy entity setup (${formatNumber(relCount)} rels): ${formatDuration(writeDuration)}`);
      console.log(`  Access check avg: ${(checkDuration / 100).toFixed(4)}ms`);

      expect(checkDuration / 100).toBeLessThan(10);
    });

    it('should handle deep inheritance chains', () => {
      const chainLength = 10;
      const chainResource = 'chain-resource';

      capbit.setCapability(chainResource, 'owner', 0xFF);
      capbit.setRelationship('entity-chain-0', 'owner', chainResource);

      for (let i = 1; i < chainLength; i++) {
        capbit.setInheritance(`entity-chain-${i}`, chainResource, `entity-chain-${i - 1}`);
      }

      const start = performance.now();
      const iterations = 1000;
      for (let i = 0; i < iterations; i++) {
        capbit.checkAccess(`entity-chain-${chainLength - 1}`, chainResource);
      }
      const duration = performance.now() - start;

      console.log(`  Chain length: ${chainLength}`);
      console.log(`  Access check avg: ${(duration / iterations).toFixed(4)}ms`);

      expect(duration / iterations).toBeLessThan(10);
    });
  });

  describe('memory and storage', () => {
    it('should report database statistics', () => {
      const dbSize = getDbSize();
      const totalRelationships = config.entityCount * config.relationshipsPerEntity;
      const bytesPerRelationship = totalRelationships > 0 ? dbSize / totalRelationships : 0;

      console.log(`  Database size: ${formatBytes(dbSize)}`);
      console.log(`  Total relationships: ${formatNumber(totalRelationships)}`);
      console.log(`  Bytes per relationship: ${bytesPerRelationship.toFixed(2)}`);

      if (bytesPerRelationship > 0) {
        console.log(`  Estimated size at 5B relationships: ${formatBytes(bytesPerRelationship * 5_000_000_000)}`);
      }

      expect(bytesPerRelationship).toBeLessThan(500);
    });
  });
});
