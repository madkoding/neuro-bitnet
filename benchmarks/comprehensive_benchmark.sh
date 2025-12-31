#!/bin/bash
# Comprehensive Benchmark for neuro-bitnet RAG System
# Benchmark completo para el sistema RAG neuro-bitnet

set -e

BINARY="./target/release/neuro"
RESULTS_DIR="./benchmarks/results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$RESULTS_DIR/benchmark_$TIMESTAMP.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  NEURO-BITNET COMPREHENSIVE BENCHMARK  ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Initialize counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Arrays to store results
declare -a TEST_NAMES
declare -a TEST_CATEGORIES
declare -a TEST_RESULTS
declare -a TEST_OUTPUTS
declare -a TEST_TIMES
declare -a TEST_EXPECTED

# Function to run a classification test
run_classify_test() {
    local name="$1"
    local query="$2"
    local expected="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    local start_time=$(date +%s%N)
    local output=$($BINARY classify "$query" 2>/dev/null || echo "ERROR")
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 )) # milliseconds
    
    TEST_NAMES+=("$name")
    TEST_CATEGORIES+=("classify")
    TEST_OUTPUTS+=("$output")
    TEST_TIMES+=("$duration")
    TEST_EXPECTED+=("$expected")
    
    # Check if output contains expected category (case insensitive)
    if echo "$output" | grep -qi "$expected"; then
        TEST_RESULTS+=("PASS")
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo -e "  ${GREEN}✓${NC} $name (${duration}ms)"
    else
        TEST_RESULTS+=("FAIL")
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "  ${RED}✗${NC} $name - Expected: $expected, Got: $output (${duration}ms)"
    fi
}

# Function to run search test
run_search_test() {
    local name="$1"
    local query="$2"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    local start_time=$(date +%s%N)
    local output=$($BINARY search "$query" --count 3 2>/dev/null || echo "ERROR")
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))
    
    TEST_NAMES+=("$name")
    TEST_CATEGORIES+=("search")
    TEST_OUTPUTS+=("$(echo "$output" | head -5)")
    TEST_TIMES+=("$duration")
    TEST_EXPECTED+=("results")
    
    if [[ "$output" != "ERROR" ]] && [[ -n "$output" ]]; then
        TEST_RESULTS+=("PASS")
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo -e "  ${GREEN}✓${NC} $name (${duration}ms)"
    else
        TEST_RESULTS+=("FAIL")
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "  ${RED}✗${NC} $name - No results (${duration}ms)"
    fi
}

# Function to run embed test
run_embed_test() {
    local name="$1"
    local text="$2"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    local start_time=$(date +%s%N)
    local output=$($BINARY embed "$text" 2>/dev/null || echo "ERROR")
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))
    
    TEST_NAMES+=("$name")
    TEST_CATEGORIES+=("embed")
    TEST_OUTPUTS+=("$(echo "$output" | head -3)")
    TEST_TIMES+=("$duration")
    TEST_EXPECTED+=("384 dimensions")
    
    if echo "$output" | grep -qE "(384|dimension|embedding)"; then
        TEST_RESULTS+=("PASS")
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo -e "  ${GREEN}✓${NC} $name (${duration}ms)"
    else
        TEST_RESULTS+=("FAIL")
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "  ${RED}✗${NC} $name (${duration}ms)"
    fi
}

echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}  1. CLASSIFICATION TESTS - MATH        ${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

run_classify_test "Simple addition" "What is 2 + 2?" "math"
run_classify_test "Multiplication" "Calculate 15 * 23" "math"
run_classify_test "Square root" "What is the square root of 144?" "math"
run_classify_test "Equation solving" "Solve x^2 + 5x + 6 = 0" "math"
run_classify_test "Division" "How much is 100 divided by 4?" "math"
run_classify_test "Percentage" "What is 25% of 200?" "math"
run_classify_test "Calculus" "Find the derivative of x^3" "math"
run_classify_test "Statistics" "Calculate the mean of 10, 20, 30" "math"
run_classify_test "Algebra expression" "Simplify 3x + 2x - x" "math"
run_classify_test "Geometry" "What is the area of a circle with radius 5?" "math"

echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}  2. CLASSIFICATION TESTS - CODE        ${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

run_classify_test "Python function" "Write a Python function to sort a list" "code"
run_classify_test "JavaScript class" "How do I create a class in JavaScript?" "code"
run_classify_test "Rust example" "Show me a Rust struct example" "code"
run_classify_test "Debug code" "Debug this code snippet" "code"
run_classify_test "Algorithm" "Implement a binary search algorithm" "code"
run_classify_test "API request" "How to make an HTTP request in Python?" "code"
run_classify_test "Database query" "Write a SQL query to select all users" "code"
run_classify_test "Regex pattern" "Create a regex to match email addresses" "code"
run_classify_test "Code review" "Review this function for bugs" "code"
run_classify_test "Refactor" "Refactor this code for better performance" "code"

echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}  3. CLASSIFICATION TESTS - GREETING    ${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

run_classify_test "Hello" "Hello!" "greeting"
run_classify_test "Hi there" "Hi there" "greeting"
run_classify_test "Good morning" "Good morning" "greeting"
run_classify_test "How are you" "Hey, how are you?" "greeting"
run_classify_test "Greetings" "Greetings!" "greeting"
run_classify_test "What's up" "What's up?" "greeting"
run_classify_test "Good evening" "Good evening" "greeting"
run_classify_test "Nice to meet" "Nice to meet you" "greeting"
run_classify_test "Your name" "What is your name?" "greeting"
run_classify_test "Who are you" "Who are you?" "greeting"

echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}  4. CLASSIFICATION TESTS - FACTUAL     ${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

run_classify_test "Capital" "What is the capital of France?" "factual"
run_classify_test "Inventor" "Who invented the telephone?" "factual"
run_classify_test "Historical event" "When was World War II?" "factual"
run_classify_test "Location" "Where is Mount Everest?" "factual"
run_classify_test "Planets" "How many planets are in our solar system?" "factual"
run_classify_test "Definition" "What is photosynthesis?" "factual"
run_classify_test "Population" "What is the population of China?" "factual"
run_classify_test "Distance" "How far is the Moon from Earth?" "factual"
run_classify_test "Discovery" "Who discovered penicillin?" "factual"
run_classify_test "Element" "What is the atomic number of gold?" "factual"

echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}  5. CLASSIFICATION TESTS - REASONING   ${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

run_classify_test "Analysis" "Analyze the pros and cons of remote work" "reasoning"
run_classify_test "Compare" "Compare Python and JavaScript" "reasoning"
run_classify_test "Why question" "Why is the sky blue?" "reasoning"
run_classify_test "Evaluate" "Evaluate the impact of AI on jobs" "reasoning"
run_classify_test "Pros cons" "What are the advantages of electric cars?" "reasoning"
run_classify_test "Explain why" "Explain why climate change happens" "reasoning"
run_classify_test "Critical thinking" "Critique this argument" "reasoning"
run_classify_test "Decision" "Should I learn Python or JavaScript first?" "reasoning"
run_classify_test "Tradeoffs" "What are the tradeoffs of microservices?" "reasoning"
run_classify_test "Hypothesis" "What would happen if gravity didn't exist?" "reasoning"

echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}  6. CLASSIFICATION TESTS - TOOLS       ${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

run_classify_test "Web search" "Search the web for latest news" "tools"
run_classify_test "Generate image" "Generate an image of a sunset" "tools"
run_classify_test "Translate" "Translate this to Spanish" "tools"
run_classify_test "Weather" "What's the weather today?" "tools"
run_classify_test "Calendar" "Add a meeting to my calendar" "tools"
run_classify_test "Send email" "Send an email to John" "tools"
run_classify_test "Download" "Download this file" "tools"
run_classify_test "Set reminder" "Set a reminder for tomorrow" "tools"
run_classify_test "Stock price" "What is the stock price of Apple?" "tools"
run_classify_test "News" "Show me the latest news" "tools"

echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}  7. CLASSIFICATION - EDGE CASES        ${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

run_classify_test "Empty-like query" "..." "conversational"
run_classify_test "Single word" "Help" "conversational"
run_classify_test "Mixed math-code" "Write code to calculate fibonacci" "code"
run_classify_test "Ambiguous" "Tell me something interesting" "conversational"
run_classify_test "Long query" "I need help understanding how machine learning algorithms work and how they can be applied to solve real-world problems in various industries" "reasoning"
run_classify_test "Spanish math" "Cuanto es 5 mas 3?" "math"
run_classify_test "Code with numbers" "def calculate(x): return x * 2" "code"
run_classify_test "Greeting with question" "Hi, can you help me?" "greeting"
run_classify_test "Factual about code" "Who created Python?" "factual"
run_classify_test "Math word problem" "If I have 5 apples and give away 2, how many do I have?" "math"

echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}  8. WEB SEARCH TESTS                   ${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

run_search_test "Programming language" "Rust programming language"
run_search_test "Technology" "Machine learning"
run_search_test "Science" "Quantum computing"
run_search_test "History" "World War II"
run_search_test "Geography" "Mount Everest"

echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}  9. EMBEDDING TESTS                    ${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

run_embed_test "Short text" "Hello world"
run_embed_test "Sentence" "The quick brown fox jumps over the lazy dog"
run_embed_test "Technical" "Machine learning is a subset of artificial intelligence"
run_embed_test "Code snippet" "def hello(): print('Hello')"
run_embed_test "Long text" "This is a longer piece of text that should still be embedded correctly by the model"

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}           BENCHMARK SUMMARY            ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "  Total Tests:  ${TOTAL_TESTS}"
echo -e "  ${GREEN}Passed:       ${PASSED_TESTS}${NC}"
echo -e "  ${RED}Failed:       ${FAILED_TESTS}${NC}"
echo ""

# Calculate pass rate
PASS_RATE=$(echo "scale=2; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc)
echo -e "  Pass Rate:    ${PASS_RATE}%"
echo ""

# Calculate average time per category
echo -e "${YELLOW}Average Response Times:${NC}"
classify_times=()
search_times=()
embed_times=()

for i in "${!TEST_CATEGORIES[@]}"; do
    case "${TEST_CATEGORIES[$i]}" in
        "classify") classify_times+=("${TEST_TIMES[$i]}") ;;
        "search") search_times+=("${TEST_TIMES[$i]}") ;;
        "embed") embed_times+=("${TEST_TIMES[$i]}") ;;
    esac
done

# Calculate averages
if [ ${#classify_times[@]} -gt 0 ]; then
    sum=0
    for t in "${classify_times[@]}"; do sum=$((sum + t)); done
    avg=$((sum / ${#classify_times[@]}))
    echo -e "  Classify:     ${avg}ms"
fi

if [ ${#search_times[@]} -gt 0 ]; then
    sum=0
    for t in "${search_times[@]}"; do sum=$((sum + t)); done
    avg=$((sum / ${#search_times[@]}))
    echo -e "  Search:       ${avg}ms"
fi

if [ ${#embed_times[@]} -gt 0 ]; then
    sum=0
    for t in "${embed_times[@]}"; do sum=$((sum + t)); done
    avg=$((sum / ${#embed_times[@]}))
    echo -e "  Embed:        ${avg}ms"
fi

echo ""
echo -e "${BLUE}========================================${NC}"

# Generate JSON results
cat > "$RESULTS_FILE" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "summary": {
    "total_tests": $TOTAL_TESTS,
    "passed": $PASSED_TESTS,
    "failed": $FAILED_TESTS,
    "pass_rate": $PASS_RATE
  },
  "tests": [
EOF

for i in "${!TEST_NAMES[@]}"; do
    comma=","
    if [ $i -eq $((${#TEST_NAMES[@]} - 1)) ]; then
        comma=""
    fi
    cat >> "$RESULTS_FILE" << EOF
    {
      "name": "${TEST_NAMES[$i]}",
      "category": "${TEST_CATEGORIES[$i]}",
      "result": "${TEST_RESULTS[$i]}",
      "expected": "${TEST_EXPECTED[$i]}",
      "time_ms": ${TEST_TIMES[$i]}
    }$comma
EOF
done

cat >> "$RESULTS_FILE" << EOF
  ]
}
EOF

echo ""
echo -e "Results saved to: ${RESULTS_FILE}"
echo ""

# Export for report generation
export BENCHMARK_RESULTS="$RESULTS_FILE"
export TOTAL_TESTS PASSED_TESTS FAILED_TESTS PASS_RATE
