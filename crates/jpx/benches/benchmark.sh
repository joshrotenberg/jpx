#!/bin/bash
# Benchmark suite for jpx CLI
# Runs various queries against different sized JSON files and records timing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JPX_BIN="${1:-$SCRIPT_DIR/../../target/release/jpx}"
RESULTS_FILE="${2:-$SCRIPT_DIR/benchmark_results.txt}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if jpx binary exists
if [ ! -f "$JPX_BIN" ]; then
    echo -e "${RED}Error: jpx binary not found at $JPX_BIN${NC}"
    echo "Run 'cargo build --release -p jpx' first"
    exit 1
fi

# Generate test data if not exists
if [ ! -f "$SCRIPT_DIR/test_small.json" ]; then
    echo -e "${YELLOW}Generating test data...${NC}"
    python3 "$SCRIPT_DIR/generate_test_data.py"
fi

# Benchmark function - runs command N times and reports stats
benchmark() {
    local name="$1"
    local iterations="${2:-10}"
    shift 2
    local cmd=("$@")

    echo -e "${BLUE}Running: $name${NC}"

    local times=()
    for i in $(seq 1 $iterations); do
        # Use GNU time or fallback to bash time
        if command -v gtime &> /dev/null; then
            t=$(gtime -f "%e" "${cmd[@]}" 2>&1 >/dev/null | tail -1)
        else
            start=$(python3 -c "import time; print(time.time())")
            "${cmd[@]}" > /dev/null 2>&1
            end=$(python3 -c "import time; print(time.time())")
            t=$(python3 -c "print(f'{$end - $start:.3f}')")
        fi
        times+=("$t")
    done

    # Calculate stats using python
    stats=$(python3 -c "
import statistics
times = [float(x) for x in '''${times[*]}'''.split()]
mean = statistics.mean(times)
if len(times) > 1:
    stdev = statistics.stdev(times)
else:
    stdev = 0
min_t = min(times)
max_t = max(times)
print(f'{mean*1000:.2f}ms (min={min_t*1000:.2f}, max={max_t*1000:.2f}, stdev={stdev*1000:.2f})')
")

    echo -e "  ${GREEN}$stats${NC}"
    echo "$name: $stats" >> "$RESULTS_FILE"
}

# Header
echo ""
echo "=========================================="
echo "  jpx Benchmark Suite"
echo "=========================================="
echo "Binary: $JPX_BIN"
echo "Date: $(date)"
echo ""

# Clear results file
echo "# jpx Benchmark Results - $(date)" > "$RESULTS_FILE"
echo "# Binary: $JPX_BIN" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Test 1: Simple field access
echo -e "\n${YELLOW}=== Simple Field Access ===${NC}"
for size in small medium large; do
    benchmark "field_access_$size" 10 "$JPX_BIN" -e "users[0].name" -f "$SCRIPT_DIR/test_$size.json"
done

# Test 2: Array filtering
echo -e "\n${YELLOW}=== Array Filtering ===${NC}"
for size in small medium large; do
    benchmark "filter_$size" 10 "$JPX_BIN" -e "users[?active]" -f "$SCRIPT_DIR/test_$size.json"
done

# Test 3: Projection/Map
echo -e "\n${YELLOW}=== Projection ===${NC}"
for size in small medium large; do
    benchmark "projection_$size" 10 "$JPX_BIN" -e "users[*].{name: name, email: email}" -f "$SCRIPT_DIR/test_$size.json"
done

# Test 4: Extension functions (string)
echo -e "\n${YELLOW}=== Extension: String Functions ===${NC}"
for size in small medium large; do
    benchmark "upper_$size" 10 "$JPX_BIN" -e "users[*].{name: upper(name)}" -f "$SCRIPT_DIR/test_$size.json"
done

# Test 5: Extension functions (array)
echo -e "\n${YELLOW}=== Extension: Array Functions ===${NC}"
for size in small medium; do
    benchmark "unique_tags_$size" 10 "$JPX_BIN" -e "unique(users[*].tags[])" -f "$SCRIPT_DIR/test_$size.json"
done

# Test 6: Chained expressions
echo -e "\n${YELLOW}=== Chained Expressions ===${NC}"
benchmark "chained_small" 10 "$JPX_BIN" -e "users[?active]" -e "[*].name" -e "sort(@)" -f "$SCRIPT_DIR/test_small.json"
benchmark "chained_medium" 10 "$JPX_BIN" -e "users[?active]" -e "[*].name" -e "sort(@)" -f "$SCRIPT_DIR/test_medium.json"

# Test 7: Slurp mode
echo -e "\n${YELLOW}=== Slurp Mode ===${NC}"
benchmark "slurp_1000" 10 "$JPX_BIN" -s -e "[*].name" -f "$SCRIPT_DIR/test_slurp.json"
benchmark "slurp_filter" 10 "$JPX_BIN" -s -e "[?active].name" -f "$SCRIPT_DIR/test_slurp.json"

# Test 8: Compact vs Pretty output
echo -e "\n${YELLOW}=== Output Formatting ===${NC}"
benchmark "pretty_medium" 10 "$JPX_BIN" -e "users[0:10]" -f "$SCRIPT_DIR/test_medium.json"
benchmark "compact_medium" 10 "$JPX_BIN" -c -e "users[0:10]" -f "$SCRIPT_DIR/test_medium.json"

# Test 9: Large dataset
echo -e "\n${YELLOW}=== Large Dataset ===${NC}"
if [ -f "$SCRIPT_DIR/test_xlarge.json" ]; then
    benchmark "parse_xlarge" 5 "$JPX_BIN" -e "length(users)" -f "$SCRIPT_DIR/test_xlarge.json"
    benchmark "filter_xlarge" 5 "$JPX_BIN" -e "users[?score > \`50\`] | length(@)" -f "$SCRIPT_DIR/test_xlarge.json"
fi

echo ""
echo -e "${GREEN}Benchmark complete! Results saved to $RESULTS_FILE${NC}"
echo ""
