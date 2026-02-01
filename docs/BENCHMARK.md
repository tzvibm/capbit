# Capbit vs Zanzibar Benchmark

Performance comparison between Capbit's bitmask-based authorization and Zanzibar-style tuple-based systems.

## Test Environment

- **Device:** Samsung Galaxy A55 (Termux)
- **CPU:** Exynos 1480 (4+4 cores)
- **RAM:** ~8GB (Termux limited)
- **Storage:** Flash
- **Capbit Version:** v3.0.0

## Methodology

### Zanzibar Reference Numbers (Published)

| Metric | SpiceDB | Google Zanzibar | OpenFGA |
|--------|---------|-----------------|---------|
| Median latency | ~2-5ms | ~5ms | ~3-5ms |
| p99 latency | ~20-50ms | ~50ms | ~30-50ms |
| Throughput | ~1-5K/sec | N/A | ~1-3K/sec |
| Infrastructure | Distributed | Distributed | Distributed |

*Sources: SpiceDB benchmarks, Google Zanzibar paper (2019), OpenFGA docs*

### Why Capbit Should Be Faster

1. **O(1) bitmask evaluation** vs O(depth) graph traversal
2. **Single-node LMDB** vs distributed network hops
3. **Memory-mapped I/O** vs RPC overhead
4. **No tuple parsing** - direct key lookup

---

## Test Results

### TEST 1: Single Check Latency

Random permission checks measuring raw latency.

| Metric | Capbit | Zanzibar (ref) | Improvement |
|--------|--------|----------------|-------------|
| p50 | TBD | ~5ms | TBD |
| p95 | TBD | ~20ms | TBD |
| p99 | TBD | ~50ms | TBD |
| max | TBD | ~100ms+ | TBD |

### TEST 2: Throughput (Sustained)

Checks per second over 60 seconds.

| Metric | Capbit | Zanzibar (ref) | Improvement |
|--------|--------|----------------|-------------|
| ops/sec | TBD | ~1-5K | TBD |
| total ops | TBD | - | - |

### TEST 3: Scale (Entity Count)

Latency at different entity counts.

| Entities | p50 | p99 | ops/sec |
|----------|-----|-----|---------|
| 1,000 | TBD | TBD | TBD |
| 10,000 | TBD | TBD | TBD |
| 50,000 | TBD | TBD | TBD |
| 100,000 | TBD | TBD | TBD |

### TEST 4: Inheritance Depth

Latency with delegation chains.

| Depth | p50 | p99 | Notes |
|-------|-----|-----|-------|
| 0 (direct) | TBD | TBD | No inheritance |
| 3 levels | TBD | TBD | Typical org |
| 5 levels | TBD | TBD | Deep hierarchy |
| 10 levels | TBD | TBD | Stress test |

### TEST 5: Memory Usage

| Scenario | Entities | Grants | DB Size | RSS |
|----------|----------|--------|---------|-----|
| Startup | 100 | 500 | TBD | TBD |
| SaaS | 1K | 5K | TBD | TBD |
| Enterprise | 10K | 50K | TBD | TBD |
| Stress | 100K | 500K | TBD | TBD |

---

## Conclusion

TBD after running benchmarks.

---

*Generated: 2026-01-31*
