# Comprehensive Test Suite Plan for Capbit

## Current State: 188 tests across 15 files

| File | Tests | Coverage | Status |
|------|-------|----------|--------|
| attack_vectors.rs | 9 | Security attacks | Original |
| attack_vectors_extended.rs | 15 | Advanced security | Phase 1 ✅ |
| permission_boundaries.rs | 16 | Capability edge cases | Phase 1 ✅ |
| revocation.rs | 11 | Permission removal | Phase 1 ✅ |
| authorized_operations.rs | 17 | Client abilities (happy path) | Phase 2 ✅ |
| input_validation.rs | 18 | Edge cases & validation | Phase 2 ✅ |
| inheritance_advanced.rs | 12 | Complex inheritance | Phase 2 ✅ |
| batch_operations.rs | 13 | Batch & WriteBatch API | Phase 2 ✅ |
| query_operations.rs | 15 | Query completeness | Phase 2 ✅ |
| type_system.rs | 17 | Type lifecycle | Phase 2 ✅ |
| protected_api.rs | 23 | v2 API | Original |
| integration.rs | 9 | v1 API | Original |
| simulation.rs | 2 | End-to-end scenarios | Original |
| benchmarks.rs | 7 | Performance | Original |
| demo_verbose.rs | 1 | Interactive demo | Original |

---

## Gap Analysis: Missing Test Categories

### A. Permission Boundary Testing (0 tests)
**Problem:** No tests verify exact capability boundaries - what happens at the edge of permissions.

### B. Input Validation & Edge Cases (0 tests)
**Problem:** No tests for malformed inputs, empty strings, special characters, long strings.

### C. Revocation & Permission Loss (0 tests)
**Problem:** No tests for what happens when permissions are revoked mid-hierarchy.

### D. Race Conditions & Ordering (0 tests)
**Problem:** No tests for concurrent access patterns or operation ordering.

### E. Advanced Inheritance Patterns (partial)
**Problem:** Only basic inheritance tested. No diamond patterns, wide inheritance, or mixed paths.

### F. Confused Deputy & Indirect Attacks (partial)
**Problem:** Limited coverage of indirect attack vectors.

### G. Type System Edge Cases (0 tests)
**Problem:** No tests for type lifecycle edge cases.

### H. Batch Operation Security (0 tests)
**Problem:** No tests for authorization in batch operations.

### I. Query Completeness (0 tests)
**Problem:** No tests verifying query functions return complete/correct results.

### J. Data Consistency (0 tests)
**Problem:** No tests for database index consistency.

---

## Proposed New Tests: ~80 additional tests

### File: tests/permission_boundaries.rs (15 tests)

```
1.  has_exact_capability_passes
2.  has_less_than_required_fails
3.  has_superset_of_required_passes
4.  zero_capability_always_passes
5.  max_u64_capability_check
6.  single_bit_difference_fails
7.  multiple_roles_combine_correctly
8.  inherited_caps_bounded_correctly
9.  type_level_vs_instance_level_priority
10. overlapping_grants_merge_correctly
11. capability_on_wrong_scope_ignored
12. relation_without_capability_gives_zero
13. capability_without_grant_gives_zero
14. deleted_grant_removes_capability
15. updated_capability_affects_existing_grants
```

### File: tests/input_validation.rs (12 tests)

```
1.  empty_entity_id_rejected
2.  empty_type_rejected
3.  empty_relation_rejected
4.  missing_colon_in_entity_id_rejected
5.  multiple_colons_in_entity_id
6.  special_chars_in_entity_name
7.  unicode_in_entity_name
8.  slash_in_entity_name_escaped
9.  very_long_entity_id
10. whitespace_in_entity_name
11. null_bytes_rejected
12. reserved_type_name_protection
```

### File: tests/revocation.rs (10 tests)

```
1.  delete_grant_removes_access
2.  delete_grant_from_hierarchy_middle
3.  revoke_delegator_permissions_effect
4.  revoke_root_type_access
5.  delete_capability_definition
6.  delete_entity_removes_all_grants
7.  revoke_one_of_multiple_relations
8.  revoke_inheritance_source
9.  cascade_effect_on_delegatees
10. re_grant_after_revocation
```

### File: tests/inheritance_advanced.rs (12 tests)

```
1.  diamond_inheritance_pattern
2.  wide_inheritance_many_sources
3.  mixed_direct_and_inherited
4.  inheritance_depth_20_levels
5.  inheritance_with_different_relations
6.  multiple_paths_to_same_source
7.  inheritance_from_deleted_source
8.  partial_capability_inheritance
9.  inheritance_chain_broken_middle
10. self_inheritance_ignored
11. inheritance_order_independence
12. inherited_plus_direct_combine
```

### File: tests/attack_vectors_extended.rs (15 tests)

```
1.  confused_deputy_via_delegation
2.  capability_bit_overflow
3.  scope_path_traversal_attempt
4.  grant_on_type_vs_instance_confusion
5.  time_of_check_time_of_use
6.  privilege_accumulation_attack
7.  indirect_escalation_via_chain
8.  impersonate_via_same_id_different_type
9.  mass_delegation_resource_exhaustion
10. type_confusion_attack
11. capability_definition_hijack
12. zombie_permission_after_delete
13. cross_type_permission_leak
14. bootstrap_race_condition
15. meta_key_injection
```

### File: tests/batch_operations.rs (8 tests)

```
1.  batch_all_authorized_succeeds
2.  batch_one_unauthorized_fails_all
3.  batch_empty_succeeds
4.  batch_duplicate_operations
5.  batch_conflicting_operations
6.  batch_with_mixed_operation_types
7.  batch_ordering_matters
8.  batch_atomic_rollback_on_failure
```

### File: tests/query_operations.rs (10 tests)

```
1.  list_accessible_returns_all
2.  list_accessible_excludes_revoked
3.  list_accessible_includes_inherited
4.  list_subjects_returns_all
5.  list_subjects_excludes_revoked
6.  query_empty_result
7.  query_large_result_set
8.  get_relationships_complete
9.  get_inheritance_complete
10. query_after_modifications
```

### File: tests/type_system.rs (8 tests)

```
1.  create_custom_type
2.  create_entity_of_custom_type
3.  delete_type_with_entities
4.  reserved_underscore_types
5.  type_case_sensitivity
6.  type_already_exists_error
7.  entity_type_mismatch
8.  bootstrap_types_immutable
```

---

## Implementation Files Created

| File | Action | Tests | Status |
|------|--------|-------|--------|
| tests/attack_vectors_extended.rs | CREATE | 15 | DONE ✅ |
| tests/permission_boundaries.rs | CREATE | 16 | DONE ✅ |
| tests/revocation.rs | CREATE | 11 | DONE ✅ |
| tests/authorized_operations.rs | CREATE | 17 | DONE ✅ |
| tests/input_validation.rs | CREATE | 18 | DONE ✅ |
| tests/inheritance_advanced.rs | CREATE | 12 | DONE ✅ |
| tests/batch_operations.rs | CREATE | 13 | DONE ✅ |
| tests/query_operations.rs | CREATE | 15 | DONE ✅ |
| tests/type_system.rs | CREATE | 17 | DONE ✅ |

**Total New Tests: 134**
**Final Total: 188 tests**

---

## Implementation Plan: COMPLETED

### Phase 1: COMPLETED (42 tests added)
1. **tests/attack_vectors_extended.rs** - 15 tests (Security critical) ✅
2. **tests/permission_boundaries.rs** - 16 tests (Core correctness) ✅
3. **tests/revocation.rs** - 11 tests (Security critical) ✅

### Phase 2: COMPLETED (92 tests added)
4. **tests/authorized_operations.rs** - 17 tests (Client abilities) ✅
5. **tests/input_validation.rs** - 18 tests (Edge cases) ✅
6. **tests/inheritance_advanced.rs** - 12 tests (Complex inheritance) ✅
7. **tests/batch_operations.rs** - 13 tests (Batch API) ✅
8. **tests/query_operations.rs** - 15 tests (Query completeness) ✅
9. **tests/type_system.rs** - 17 tests (Type lifecycle) ✅

---

## Test Patterns to Follow

Each test file should use the same setup pattern:
```rust
use capbit::{init, bootstrap, protected, check_access, SystemCap, clear_all, test_lock};
use tempfile::TempDir;
use std::sync::Once;

static INIT: Once = Once::new();
static mut TEST_DIR: Option<TempDir> = None;

fn setup() { /* ... */ }
fn setup_bootstrapped() -> MutexGuard<'static, ()> { /* ... */ }
```

Each test should:
1. Setup clean state
2. Arrange the scenario
3. Act (perform the operation)
4. Assert expected behavior
5. Clean up (automatic via test isolation)

---

## Verification

Run all tests:
```bash
cargo test -- --nocapture 2>&1 | grep -E "^test |passed|failed"
cargo test 2>&1 | tail -5  # Summary line
```

**Final Result: `test result: ok. 188 passed; 0 failed`**
