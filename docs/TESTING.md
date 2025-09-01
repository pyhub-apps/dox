# Testing Framework Documentation

This document describes the comprehensive testing framework for the dox Rust application.

## Table of Contents

- [Overview](#overview)
- [Test Structure](#test-structure)
- [Running Tests](#running-tests)
- [Writing Tests](#writing-tests)
- [Code Coverage](#code-coverage)
- [Benchmarks](#benchmarks)
- [CI/CD Integration](#cicd-integration)

## Overview

The dox testing framework provides multiple layers of testing to ensure code quality and reliability:

- **Unit Tests**: Test individual functions and modules
- **Integration Tests**: Test complete workflows and command interactions
- **Property-Based Tests**: Verify properties hold for arbitrary inputs
- **Benchmark Tests**: Measure and track performance
- **Documentation Tests**: Ensure code examples in docs are correct

## Test Structure

```
dox/
├── src/
│   └── */tests.rs           # Unit tests for each module
├── tests/
│   ├── common/
│   │   └── mod.rs           # Shared test utilities
│   ├── integration_test.rs  # Integration tests
│   └── property_tests.rs    # Property-based tests
├── benches/
│   └── replace_benchmark.rs # Performance benchmarks
└── .github/workflows/
    └── test.yml             # CI/CD configuration
```

## Running Tests

### All Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release

# Run specific test
cargo test test_name
```

### Unit Tests
```bash
# Run unit tests only
cargo test --lib

# Run tests for specific module
cargo test --lib replace
```

### Integration Tests
```bash
# Run integration tests
cargo test --test integration_test

# Run specific integration test
cargo test --test integration_test test_replace_command
```

### Property Tests
```bash
# Run property-based tests
cargo test --test property_tests

# Run with more test cases
PROPTEST_CASES=1000 cargo test --test property_tests
```

### Benchmarks
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench replace_benchmark

# Compare with baseline
cargo bench --bench replace_benchmark -- --baseline main
```

### Documentation Tests
```bash
# Run documentation tests
cargo test --doc
```

## Writing Tests

### Unit Tests

Unit tests are placed in a `tests` module within each source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function() {
        assert_eq!(function(input), expected);
    }
}
```

### Parameterized Tests with rstest

Use `rstest` for data-driven testing:

```rust
use rstest::*;

#[rstest]
#[case("input1", "expected1")]
#[case("input2", "expected2")]
fn test_with_parameters(
    #[case] input: &str,
    #[case] expected: &str
) {
    assert_eq!(process(input), expected);
}
```

### Property-Based Tests

Use `proptest` for property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(
        input in "[a-z]{1,10}"
    ) {
        let result = process(&input);
        assert!(result.len() >= input.len());
    }
}
```

### Test Fixtures

Use the `TestFixture` helper for file-based tests:

```rust
use tests::common::TestFixture;

#[test]
fn test_with_files() {
    let fixture = TestFixture::new();
    
    // Create test files
    let file = fixture.create_file("test.txt", "content");
    
    // Run test
    let result = process(&file);
    
    // Verify
    assert!(fixture.file_exists("output.txt"));
}
```

### Snapshot Testing

Use `insta` for snapshot testing:

```rust
use insta::assert_snapshot;

#[test]
fn test_output_format() {
    let result = format_output(data);
    assert_snapshot!(result);
}
```

## Code Coverage

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out html

# Generate coverage with specific features
cargo tarpaulin --all-features --out xml

# Exclude files from coverage
cargo tarpaulin --exclude-files "*/tests/*" --out lcov
```

### Coverage Goals

- **Unit Tests**: > 80% coverage
- **Critical Paths**: 100% coverage
- **Error Handling**: > 90% coverage
- **Overall**: > 75% coverage

### View Coverage Report

```bash
# Generate HTML report
cargo tarpaulin --out html

# Open report
open tarpaulin-report.html
```

## Benchmarks

### Writing Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("function_name", |b| {
        b.iter(|| {
            function(black_box(input))
        });
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run and save baseline
cargo bench -- --save-baseline main

# Compare with baseline
cargo bench -- --baseline main

# Generate HTML report
cargo bench -- --output-format html
```

### Benchmark Results

Results are saved in `target/criterion/`. View the HTML report:

```bash
open target/criterion/report/index.html
```

## CI/CD Integration

### GitHub Actions Workflow

The project uses GitHub Actions for continuous testing:

1. **Test Matrix**: Tests on Linux, macOS, and Windows
2. **Rust Versions**: Tests on stable and beta
3. **Code Quality**: Runs clippy and rustfmt
4. **Coverage**: Generates coverage reports with tarpaulin
5. **Benchmarks**: Runs performance benchmarks on main branch

### Local CI Simulation

```bash
# Run all CI checks locally
./scripts/ci-local.sh

# Or manually:
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo bench --no-run
```

## Test Data

### Sample Files

Test fixtures are available in `tests/common/`:

- **YAML Rules**: `sample_rules::BASIC_REPLACE`
- **Document Templates**: `sample_docs::MARKDOWN_TEMPLATE`
- **Test Documents**: Created with `TestFixture::create_word_doc()`

### Performance Test Data

Large files for performance testing:

```rust
// Generate large test file
let large_text = "test content".repeat(10000);
```

## Debugging Tests

### Enable Debug Output

```bash
# Run with debug output
RUST_LOG=debug cargo test

# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run single test with output
cargo test test_name -- --nocapture
```

### Test Isolation

Tests that modify global state should use `serial_test`:

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_with_global_state() {
    // Test that needs isolation
}
```

## Best Practices

### 1. Test Naming
- Use descriptive names: `test_replace_multiple_patterns`
- Group related tests: `mod replace_tests`

### 2. Test Independence
- Each test should be independent
- Use fixtures for setup/teardown
- Don't rely on test execution order

### 3. Assertions
- Use specific assertions: `assert_eq!` over `assert!`
- Include helpful messages: `assert!(result, "Failed because...")`

### 4. Performance
- Keep unit tests fast (< 1 second)
- Use `--release` for integration tests if needed
- Move slow tests to separate test files

### 5. Coverage
- Aim for high coverage but prioritize critical paths
- Test error conditions and edge cases
- Don't test trivial getters/setters

## Troubleshooting

### Common Issues

1. **Tests Failing on CI but Passing Locally**
   - Check for platform-specific behavior
   - Verify file paths use `Path` not strings
   - Check for timing-dependent tests

2. **Slow Tests**
   - Use `cargo test --release` for performance
   - Consider moving to integration tests
   - Check for unnecessary I/O operations

3. **Flaky Tests**
   - Add retries for network operations
   - Use proper synchronization for concurrent tests
   - Increase timeouts for CI environments

4. **Coverage Gaps**
   - Run `cargo tarpaulin --print-summary`
   - Check for untested error paths
   - Add tests for edge cases

## Future Improvements

- [ ] Add mutation testing with `cargo-mutants`
- [ ] Implement fuzz testing for parsers
- [ ] Add contract testing for API clients
- [ ] Create performance regression detection
- [ ] Add visual regression testing for CLI output