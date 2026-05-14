#!/usr/bin/env bash
# Real-world corpus regression guard for the Waltzing tree-sitter grammar.
#
# Parses every `.wtz` file in the sibling `cli/` and `libraries/waltzing-ui/`
# trees and counts ERROR / MISSING nodes. The grammar parses the whole corpus
# cleanly (budget 0); this guards against *regressions* — any grammar change
# that introduces a single ERROR / MISSING node fails the check.
#
# Usage:  bash test/corpus-check.sh   (run from the tree-sitter/ directory)
#         npm run corpus-check
#
# History:
#   922  — true baseline, before any 2026-05 grammar fixes
#   887  — after path/ref-mut/@if-let-attr/embedded-attr-value fixes (4b34663)
#    89  — after the expression-grammar overhaul: rust_path patterns,
#          path/reference/unary/binary/cast/closure/match/if expressions,
#          array/raw-string literals, doctype, @**/@*** comments, @out args
#    10  — block expressions, <style>/<script> raw text, `<` (whitespace)
#          comparison, for-loop ranges, @"" / @if attr values, fn rust bodies
#     0  — :: in expression_path, tokenized string_literal, `"` excluded
#          from rust_expression — entire corpus parses cleanly
set -euo pipefail

# The grammar parses the whole corpus with zero errors — keep it that way.
ERROR_BUDGET=0

cd "$(dirname "$0")/.."
TS="npx tree-sitter"

CORPUS_DIRS=(../cli ../libraries/waltzing-ui)

total=0
files=0
clean=0
worst=""

for dir in "${CORPUS_DIRS[@]}"; do
  [ -d "$dir" ] || continue
  while IFS= read -r f; do
    files=$((files + 1))
    n=$($TS parse "$f" 2>/dev/null | grep -c "ERROR\|MISSING" || true)
    total=$((total + n))
    if [ "$n" -eq 0 ]; then
      clean=$((clean + 1))
    else
      worst="${worst}\n  ${n}\t${f}"
    fi
  done < <(find "$dir" -name "*.wtz" 2>/dev/null)
done

echo "corpus-check: ${files} files | ${clean} clean | ${total} ERROR/MISSING nodes (budget ${ERROR_BUDGET})"

if [ "$total" -gt "$ERROR_BUDGET" ]; then
  echo ""
  echo "REGRESSION: ${total} > budget ${ERROR_BUDGET}. Files with errors:"
  echo -e "$worst" | sort -rn | head -20
  exit 1
fi

if [ "$total" -lt "$ERROR_BUDGET" ]; then
  echo "IMPROVED: ${total} < budget ${ERROR_BUDGET} — lower ERROR_BUDGET in this script to lock it in."
fi
exit 0
