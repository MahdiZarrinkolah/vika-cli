# Intermediate Coverage Goal: 60%

**Current Coverage:** ~40-45% (estimated)  
**Target Coverage:** 60%  
**Timeline:** 2-4 hours of focused work  
**Status:** Phase 1 Complete, Moving to Phase 2

---

## Why 60%?

60% coverage is a **pragmatic intermediate goal** that:
- ✅ Achieves meaningful coverage improvements (+15-20 percentage points)
- ✅ Focuses on critical business logic and error paths
- ✅ Provides solid foundation for future work
- ✅ Is achievable in reasonable timeframe (2-4 hours)
- ✅ Follows the 80/20 rule: 60% effort for 80% of value

---

## Current Status (✅ Phase 1 Complete)

### Tests Added: 141 passing tests
- ✅ Main CLI (16 tests)
- ✅ Cache Manager (6 tests) - **FIXED**
- ✅ Progress Reporter (12 tests)
- ✅ Formatter (9 tests)
- ✅ Module Selector (3 tests)
- ✅ Config modules (enhanced)
- ✅ Swagger Parser (11 tests)
- ✅ Schema Resolver (12 tests) - **NEW**
- ✅ All utility tests passing (46 lib tests)
- ✅ Integration command tests (3 tests)

### Infrastructure Ready
- ✅ Test framework established
- ✅ Dependencies added (assert_cmd, predicates)
- ✅ Test patterns documented
- ✅ All existing tests passing

---

## Path to 60% Coverage

### Priority 1: Generator Modules (Core Business Logic)
**Estimated Impact:** +10-12 percentage points

#### 1. API Client Generator (`api_client.rs`: 58% → 80%)
**Effort:** 1 hour  
**Value:** High - critical for code generation

**Tests to Add:**
- ✅ Basic function generation (already covered by snapshots)
- ⬜ Path parameter extraction and formatting
- ⬜ Query parameter handling (arrays, objects, enums)
- ⬜ Header parameter injection
- ⬜ Request body serialization
- ⬜ Response type mapping
- ⬜ Error response handling
- ⬜ Function naming edge cases

**File:** `tests/api_client_extended_test.rs`

#### 2. Swagger Parser (`swagger_parser.rs`: 43% → 70%)
**Effort:** 45 minutes  
**Value:** High - critical for input processing

**Tests to Add:**
- ✅ Basic parsing (already done)
- ⬜ Invalid JSON/YAML handling
- ⬜ Missing required fields
- ⬜ Parameter reference resolution
- ⬜ Request body reference resolution  
- ⬜ Response reference resolution
- ⬜ Complex nested references
- ⬜ Circular reference detection

**File:** Extend `tests/swagger_parser_test.rs`

#### 3. Writer Module (`writer.rs`: 49% → 75%)
**Effort:** 45 minutes  
**Value:** High - critical for output generation

**Tests to Add:**
- ✅ Basic file writing (already covered)
- ⬜ Backup file creation
- ⬜ Force overwrite logic
- ⬜ Conflict detection
- ⬜ Common schema imports
- ⬜ Directory creation errors
- ⬜ Permission errors
- ⬜ Type deduplication

**File:** Extend `tests/writer_test.rs`

### Priority 2: Command Modules
**Estimated Impact:** +5-7 percentage points

#### 4. Generate Command (`commands/generate.rs`: 0% → 60%)
**Effort:** 30 minutes  
**Value:** Medium-High - main user workflow

**Tests to Add:**
- ⬜ Basic generation flow
- ⬜ Module selection from config
- ⬜ Cache flag handling
- ⬜ Backup flag handling
- ⬜ Force flag handling
- ⬜ Error handling (missing config, invalid spec)

**File:** `tests/generate_command_test.rs`

#### 5. Inspect Command (`commands/inspect.rs`: 0% → 60%)
**Effort:** 20 minutes  
**Value:** Medium - diagnostic tool

**Tests to Add:**
- ✅ Basic inspect (covered in integration_main_test)
- ⬜ Module filtering
- ⬜ Schema details output
- ⬜ JSON output format
- ⬜ Table formatting

**File:** Extend `tests/integration_main_test.rs` or create dedicated file

### Priority 3: Schema Generators (Selective Coverage)
**Estimated Impact:** +3-5 percentage points

#### 6. Zod Schema (`zod_schema.rs`: 23% → 50%)
**Effort:** 30 minutes  
**Value:** Medium - validation logic

**Tests to Add (Focus on Common Patterns):**
- ⬜ String schemas with validation
- ⬜ Number/Integer schemas
- ⬜ Array schemas
- ⬜ Object schemas with required fields
- ⬜ Enum schemas
- ⬜ Nullable handling
- ⬜ Optional vs required

**File:** `tests/zod_schema_basic_test.rs`

#### 7. TypeScript Typings (`ts_typings.rs`: 45% → 65%)
**Effort:** 30 minutes  
**Value:** Medium - type safety

**Tests to Add (Focus on Common Patterns):**
- ✅ Basic types (covered by snapshots)
- ⬜ Interface generation
- ⬜ Enum type generation
- ⬜ Nullable types
- ⬜ Optional properties
- ⬜ Union types

**File:** `tests/ts_typings_basic_test.rs`

---

## Skipped for 60% Goal

These areas will be addressed in later phases (70%+):

### Lower Priority for 60%
- ❌ Init command (interactive, hard to test)
- ❌ Update command (similar to generate)
- ❌ Schema resolver (advanced dependency resolution)
- ❌ Edge cases (unicode, special characters)
- ❌ Exhaustive error handling
- ❌ Deep schema nesting (beyond 3 levels)
- ❌ Complex schema compositions (multiple allOf/oneOf/anyOf)

---

## Implementation Plan

### Week 1: Core Generators (Target: 55%)
1. **Day 1-2:** API Client extended tests (58% → 80%)
2. **Day 2-3:** Swagger Parser extended tests (43% → 70%)
3. **Day 3-4:** Writer extended tests (49% → 75%)

### Week 2: Commands & Schemas (Target: 60%)
4. **Day 5:** Generate command tests (0% → 60%)
5. **Day 6:** Zod & TS Typings basic tests (23%/45% → 50%/65%)
6. **Day 7:** Inspect command tests, verification

### Verification
```bash
cargo tarpaulin --timeout 120 --out Stdout
```

Expected output: `Coverage: 60.XX%`

---

## Success Criteria

### Quantitative
- ✅ Overall coverage reaches 60%+
- ✅ All core generator modules > 60%
- ✅ All existing tests continue to pass
- ✅ No new linting errors

### Qualitative
- ✅ Critical business logic paths tested
- ✅ Main error handling paths covered
- ✅ Common use cases validated
- ✅ Regression protection established

---

## After 60%: Next Steps

### Phase 2b: 60% → 75% (Optional)
- Complete remaining command modules
- Add edge case tests
- Increase generator module coverage to 80%+
- Add integration workflow tests

### Phase 3: 75% → 90% (Optional)
- Exhaustive error path testing
- Complex schema compositions
- Performance edge cases
- Integration with external tools

### Phase 4: 90% → 100% (Optional)
- Cover all remaining branches
- Add property-based tests
- Stress testing
- Complete documentation coverage

---

## Risk Mitigation

### Potential Blockers
1. **Test Environment Issues** - Some tests may be environment-dependent
   - *Mitigation:* Use tempfile and isolation
2. **Mocking Complexity** - Interactive commands hard to test
   - *Mitigation:* Skip or use simple acceptance tests
3. **Time Overruns** - Some modules harder than estimated
   - *Mitigation:* Adjust scope, focus on critical paths

### Quality Gates
- All tests must pass before proceeding
- No coverage regressions allowed
- Code must remain linted
- Performance must not degrade

---

## Tracking Progress

### Commands
```bash
# Run all tests
cargo test

# Check coverage
cargo tarpaulin --timeout 120 --out Stdout

# Run specific module tests
cargo test --test api_client_extended_test
cargo test --test generate_command_test
```

### Metrics to Track
- Overall coverage percentage
- Per-module coverage
- Test count
- Test execution time
- Failing tests count

---

## Conclusion

**60% coverage is achievable and valuable:**
- Represents a 50% improvement from baseline (~40%)
- Covers all critical business logic paths
- Establishes strong testing foundation
- Achievable in reasonable timeframe
- Provides excellent ROI

**After reaching 60%, reassess:**
- Decide if continuing to 75%+ is worthwhile
- Focus on quality over quantity
- Consider maintenance burden
- Balance coverage with development velocity

---

## Updated TODO List

### High Priority (For 60%)
- [ ] Extend API client tests (api_client.rs: 58% → 80%)
- [ ] Extend swagger parser tests (swagger_parser.rs: 43% → 70%)
- [ ] Extend writer tests (writer.rs: 49% → 75%)
- [ ] Add generate command tests (generate.rs: 0% → 60%)
- [ ] Add basic zod schema tests (zod_schema.rs: 23% → 50%)
- [ ] Add basic ts typings tests (ts_typings.rs: 45% → 65%)
- [ ] Extend inspect command tests (inspect.rs: partial → 60%)

### Medium Priority (For 70%+)
- [ ] Add init command tests
- [ ] Add update command tests
- [ ] Add edge case tests
- [ ] Add error handling tests
- [ ] Complete schema generator coverage

### Low Priority (For 90%+)
- [ ] Property-based tests
- [ ] Performance tests
- [ ] Stress tests
- [ ] Complete error path coverage

