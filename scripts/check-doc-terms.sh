#!/usr/bin/env bash
# Check documentation files for stale type/annotation terms.
# Exit 1 if stale terms found, 0 if clean.
# Extend STALE_PATTERNS as the project evolves.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
EXIT_CODE=0

# Each pattern is a PCRE regex applied to doc files.
# Add new patterns here when renames happen.
STALE_PATTERNS=(
    '# sig:.*\bInt\b'
    '# extern:.*\bInt\b'
    'Supported types:.*\bInt\b'
    '\bArray<Int>'
    '\bHash<Str,\s*Int>'
)

DOC_FILES=(
    "$ROOT/README.md"
)

for pattern in "${STALE_PATTERNS[@]}"; do
    for f in "${DOC_FILES[@]}"; do
        if [ -f "$f" ] && grep -nP "$pattern" "$f" 2>/dev/null; then
            echo "  ^^^ STALE TERM in $f" >&2
            EXIT_CODE=1
        fi
    done
done

# Check example comment lines, excluding:
#   - Int( — Perl test generator calls like Int(range=>...)
#   - Z3's Int — refers to Z3's own Int sort, not perlchecker's type
#   - int( — Perl's builtin int() function
if [ -d "$ROOT/examples" ]; then
    while IFS= read -r match; do
        echo "  STALE TERM: $match" >&2
        EXIT_CODE=1
    done < <(grep -rnP '^\s*#.*\bInt\b' "$ROOT/examples/"*.pl 2>/dev/null \
        | grep -vP 'Int\(' \
        | grep -vP "Z3's Int" \
        || true)
fi

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "Doc term check: all clean."
fi
exit $EXIT_CODE
