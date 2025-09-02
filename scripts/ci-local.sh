#!/bin/bash
# Local CI simulation script
# Run all CI checks locally before pushing

set -e

echo "🔍 Running local CI checks..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        exit 1
    fi
}

# 1. Format check
echo "📝 Checking code formatting..."
cargo fmt -- --check
print_status $? "Code formatting"
echo ""

# 2. Clippy lints
echo "📎 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
print_status $? "Clippy lints"
echo ""

# 3. Build
echo "🔨 Building project..."
cargo build --all-features
print_status $? "Build"
echo ""

# 4. Unit tests
echo "🧪 Running unit tests..."
cargo test --lib
print_status $? "Unit tests"
echo ""

# 5. Integration tests
echo "🔗 Running integration tests..."
cargo test --test '*'
print_status $? "Integration tests"
echo ""

# 6. Doc tests
echo "📚 Running documentation tests..."
cargo test --doc
print_status $? "Documentation tests"
echo ""

# 7. Check benchmarks compile
echo "⚡ Checking benchmarks..."
cargo bench --no-run
print_status $? "Benchmarks compile"
echo ""

# 8. Optional: Run property tests with fewer cases
echo "🎲 Running property tests (quick)..."
PROPTEST_CASES=32 cargo test --test property_tests
print_status $? "Property tests"
echo ""

# 9. Optional: Generate coverage report
if command -v cargo-tarpaulin &> /dev/null; then
    echo "📊 Generating coverage report..."
    cargo tarpaulin --print-summary --skip-clean
    echo ""
else
    echo -e "${YELLOW}ℹ${NC} Skipping coverage (cargo-tarpaulin not installed)"
    echo "  Install with: cargo install cargo-tarpaulin"
    echo ""
fi

echo -e "${GREEN}✅ All CI checks passed!${NC}"
echo ""
echo "Ready to push to GitHub 🚀"