#!/usr/bin/env bash
# Check that docs/PERL-SUBSET.md matches the implemented subset.
# Compares: type names, builtin functions, operators.
# Exit 1 if mismatches found, 0 if in sync.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
SPEC="$ROOT/docs/PERL-SUBSET.md"
ANNOTATIONS="$ROOT/src/annotations/mod.rs"
AST="$ROOT/src/ast/mod.rs"
PEST="$ROOT/src/parser/perl_subset.pest"

EXIT_CODE=0

err() { echo "MISMATCH: $1" >&2; EXIT_CODE=1; }

# --- 1. Check annotation type names ---
# Extract types accepted by parse_type() in annotations/mod.rs
IMPL_TYPES=$(grep -oP '"[^"]+"\s*=>\s*Ok\(Type::' "$ANNOTATIONS" | grep -oP '"[^"]+"' | tr -d '"' | sort)

for t in $IMPL_TYPES; do
    if ! grep -qF "$t" "$SPEC"; then
        err "Type '$t' is in parse_type() but not documented in PERL-SUBSET.md"
    fi
done

# --- 2. Check builtin functions ---
# Extract Builtin enum variants from ast/mod.rs
IMPL_BUILTINS=$(sed -n '/^pub enum Builtin/,/^}/p' "$AST" | grep -oP '^\s+(\w+),' | tr -d ' ,' | sort)

# Extract _call rules from pest grammar (these are the actual builtin names)
PEST_BUILTINS=$(grep -oP '^\w+_call' "$PEST" | sed 's/_call$//' | sort)

# Map Builtin enum names to their Perl-facing names (lowercase)
for b in $PEST_BUILTINS; do
    # Convert snake_case builtin name to how it appears in docs
    case "$b" in
        scalar|pop|length|substr|index|abs|min|max|ord|chr|chomp|reverse|int|contains|starts_with|ends_with|replace|char_at|defined|exists)
            if ! grep -qP "\`${b}\(" "$SPEC"; then
                err "Builtin '$b()' is in grammar but not documented in PERL-SUBSET.md"
            fi
            ;;
        *)
            err "Unknown builtin '$b' in grammar — add to check-subset-sync.sh and PERL-SUBSET.md"
            ;;
    esac
done

# Check that each Builtin enum variant has a corresponding _call rule
for b in $IMPL_BUILTINS; do
    lower=$(echo "$b" | sed 's/\([A-Z]\)/_\L\1/g' | sed 's/^_//' | tr '[:upper:]' '[:lower:]')
    case "$lower" in
        starts_with|ends_with|char_at) search="${lower}" ;;
        *) search="${lower}" ;;
    esac
    if ! echo "$PEST_BUILTINS" | grep -qx "$search"; then
        err "Builtin enum variant '$b' has no corresponding ${search}_call rule in grammar"
    fi
done

# --- 3. Check operators ---
# Extract op_ rules from pest
PEST_OPS=$(grep -oP '^op_\w+' "$PEST" | sort)

EXPECTED_OPS="op_add op_and op_bitand op_bitnot op_bitor op_bitxor op_cmp op_concat op_div op_eq op_ge op_gt op_le op_low_and op_low_not op_low_or op_lt op_mod op_mul op_ne op_neg op_not op_or op_pow op_repeat op_seq op_sge op_sgt op_shl op_shr op_sle op_slt op_sne op_spaceship op_sub"

for op in $PEST_OPS; do
    case "$op" in
        op_add|op_sub|op_mul|op_div|op_mod|op_pow) symbol_section="Arithmetic Operators" ;;
        op_concat|op_repeat) symbol_section="String Operators" ;;
        op_eq|op_ne|op_lt|op_le|op_gt|op_ge|op_spaceship) symbol_section="Numeric Comparison" ;;
        op_seq|op_sne|op_slt|op_sle|op_sgt|op_sge|op_cmp) symbol_section="String Comparison" ;;
        op_and|op_or|op_not|op_low_and|op_low_or|op_low_not) symbol_section="Logical Operators" ;;
        op_bitand|op_bitor|op_bitxor|op_bitnot|op_shl|op_shr) symbol_section="Bitwise Operators" ;;
        op_neg) symbol_section="Arithmetic" ;;
        *) err "Unknown operator '$op' in grammar — add to check-subset-sync.sh and PERL-SUBSET.md" ;;
    esac
done

# --- 4. Check control flow statements ---
# Extract statement types from the stmt rule in pest
PEST_STMTS=$(grep '^stmt = ' "$PEST" | grep -oP '\w+_stmt' | sort -u)

for s in $PEST_STMTS; do
    base=$(echo "$s" | sed 's/_stmt$//')
    case "$base" in
        return|if|while|for|foreach|unless|until|do_while|do_until|die|warn|print|say|last|next|push|assign|declare|array_assign|hash_assign|list_assign|inc|dec|array_init|deref_assign|arrow_array_assign|arrow_hash_assign)
            # These should all be documented in PERL-SUBSET.md in some form
            ;;
        *)
            err "Statement type '$s' in grammar but may not be documented in PERL-SUBSET.md"
            ;;
    esac
done

# --- 5. Check annotation directives ---
for directive in "# sig:" "# pre:" "# post:" "# extern:" "# ghost:" "# assert:" "# inv:"; do
    if ! grep -qF "$directive" "$SPEC"; then
        err "Annotation directive '$directive' not documented in PERL-SUBSET.md"
    fi
done

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "Subset sync check: PERL-SUBSET.md matches implementation."
fi
exit $EXIT_CODE
