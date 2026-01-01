#!/bin/bash
# Benchmark: BitNet translation with the model itself
# Tests the new simplified translation approach

export LC_ALL=C  # Force C locale for consistent number formatting

NEURO="./target/release/neuro"
ITERATIONS=5

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=============================================="
echo " BitNet Translation Benchmark (Model-based)"
echo "=============================================="
echo ""
echo "Testing with $ITERATIONS iterations per question"
echo ""

# Questions to test (Spanish)
declare -A QUESTIONS
QUESTIONS["¿Cuál es la capital de Francia?"]="Paris|París"
QUESTIONS["¿Cuántos continentes hay?"]="7|seven|siete"
QUESTIONS["¿Cuál es el planeta más grande del sistema solar?"]="Jupiter|Júpiter"
QUESTIONS["¿Quién pintó la Mona Lisa?"]="Leonardo|Vinci|da Vinci"
QUESTIONS["¿Quién escribió Don Quijote?"]="Cervantes|Miguel"

total_correct_no_translate=0
total_correct_translate=0
total_no_translate=0
total_translate=0
total_time_no_translate=0
total_time_translate=0

echo "=========================================="
echo " Without Translation (Spanish direct)"
echo "=========================================="

for question in "${!QUESTIONS[@]}"; do
    expected="${QUESTIONS[$question]}"
    correct=0
    total_time=0
    
    for i in $(seq 1 $ITERATIONS); do
        start_time=$(date +%s%3N)
        response=$($NEURO ask "$question" --max-tokens 30 --format json 2>&1 | grep '"answer"' | sed 's/.*"answer": "\([^"]*\)".*/\1/' || echo "")
        end_time=$(date +%s%3N)
        
        elapsed=$((end_time - start_time))
        total_time=$((total_time + elapsed))
        
        if echo "$response" | grep -qiE "$expected"; then
            ((correct++))
        fi
    done
    
    avg_time=$((total_time / ITERATIONS))
    pct=$(awk "BEGIN {printf \"%.1f\", $correct * 100 / $ITERATIONS}")
    
    total_correct_no_translate=$((total_correct_no_translate + correct))
    total_no_translate=$((total_no_translate + ITERATIONS))
    total_time_no_translate=$((total_time_no_translate + total_time))
    
    printf "  %-50s %d/%d (%.1f%%) [%dms]\n" "$question" "$correct" "$ITERATIONS" "$pct" "$avg_time"
done

echo ""
echo "=========================================="
echo " With Translation (BitNet translates)"
echo "=========================================="

for question in "${!QUESTIONS[@]}"; do
    expected="${QUESTIONS[$question]}"
    correct=0
    total_time=0
    
    for i in $(seq 1 $ITERATIONS); do
        start_time=$(date +%s%3N)
        response=$($NEURO ask "$question" --translate --max-tokens 30 --format json 2>&1 | grep '"answer"' | sed 's/.*"answer": "\([^"]*\)".*/\1/' || echo "")
        end_time=$(date +%s%3N)
        
        elapsed=$((end_time - start_time))
        total_time=$((total_time + elapsed))
        
        if echo "$response" | grep -qiE "$expected"; then
            ((correct++))
        fi
    done
    
    avg_time=$((total_time / ITERATIONS))
    pct=$(awk "BEGIN {printf \"%.1f\", $correct * 100 / $ITERATIONS}")
    
    total_correct_translate=$((total_correct_translate + correct))
    total_translate=$((total_translate + ITERATIONS))
    total_time_translate=$((total_time_translate + total_time))
    
    printf "  %-50s %d/%d (%.1f%%) [%dms]\n" "$question" "$correct" "$ITERATIONS" "$pct" "$avg_time"
done

echo ""
echo "=========================================="
echo " Summary"
echo "=========================================="

pct_no_translate=$(awk "BEGIN {printf \"%.1f\", $total_correct_no_translate * 100 / $total_no_translate}")
pct_translate=$(awk "BEGIN {printf \"%.1f\", $total_correct_translate * 100 / $total_translate}")
avg_no_translate=$((total_time_no_translate / total_no_translate))
avg_translate=$((total_time_translate / total_translate))

echo ""
printf "Without translation: %d/%d (%.1f%%) avg %dms\n" "$total_correct_no_translate" "$total_no_translate" "$pct_no_translate" "$avg_no_translate"
printf "With translation:    %d/%d (%.1f%%) avg %dms\n" "$total_correct_translate" "$total_translate" "$pct_translate" "$avg_translate"
echo ""

improvement=$(awk "BEGIN {printf \"%.1f\", $pct_translate - $pct_no_translate}")
if awk "BEGIN {exit !($pct_translate > $pct_no_translate)}"; then
    echo -e "${GREEN}✓ Translation improves accuracy by +${improvement}%${NC}"
else
    echo -e "${RED}✗ Translation decreases accuracy by ${improvement}%${NC}"
fi

time_diff=$((avg_translate - avg_no_translate))
echo "Time overhead: +${time_diff}ms per query"
