#!/usr/bin/env bash
# translate-docs.sh - Translate French markdown documentation to English via Ollama
# Uses translate_ollama.py for individual file translation

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TRANSLATOR="$SCRIPT_DIR/translate_ollama.py"
EN_DIR="$PROJECT_ROOT/docs/en"
FR_DIR="$PROJECT_ROOT/docs/fr"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC} $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

main() {
    log_info "Starting translation"
    log_info "Source: $FR_DIR"
    log_info "Target: $EN_DIR"

    # Check Ollama is running
    if ! curl -s "http://localhost:11434/" > /dev/null 2>&1; then
        log_error "Ollama is not running at localhost:11434"
        exit 1
    fi

    local mode="${1:-docs}"
    local files_fr=()
    local files_en=()

    case "$mode" in
        docs)
            while IFS= read -r f; do
                files_fr+=("$f")
                files_en+=("${f/$FR_DIR/$EN_DIR}")
            done < <(find "$FR_DIR" -type f -name "*.md" -printf '%s %p\n' | sort -n | awk '{print $2}')
            ;;
        root)
            for name in README CONTRIBUTING SECURITY; do
                files_fr+=("$PROJECT_ROOT/${name}.fr.md")
                files_en+=("$PROJECT_ROOT/${name}.md")
            done
            ;;
        all)
            while IFS= read -r f; do
                files_fr+=("$f")
                files_en+=("${f/$FR_DIR/$EN_DIR}")
            done < <(find "$FR_DIR" -type f -name "*.md" -printf '%s %p\n' | sort -n | awk '{print $2}')
            for name in README CONTRIBUTING SECURITY; do
                files_fr+=("$PROJECT_ROOT/${name}.fr.md")
                files_en+=("$PROJECT_ROOT/${name}.md")
            done
            ;;
        *)
            if [[ -f "$mode" ]]; then
                files_fr+=("$mode")
                local en_path="${mode/$FR_DIR/$EN_DIR}"
                files_en+=("$en_path")
            else
                log_error "Unknown mode or file: $mode"
                echo "Usage: $0 [docs|root|all|<filepath>]"
                exit 1
            fi
            ;;
    esac

    local total=${#files_fr[@]}
    local success=0
    local failed=0
    local warnings=0
    local counter=0

    log_info "Files to translate: $total"
    echo "---"

    for i in "${!files_fr[@]}"; do
        local fr_file="${files_fr[$i]}"
        local en_file="${files_en[$i]}"
        local rel_path="${fr_file#$PROJECT_ROOT/}"
        ((counter++)) || true

        mkdir -p "$(dirname "$en_file")"

        local size
        size=$(wc -c < "$fr_file")
        log_info "[$counter/$total] Translating: $rel_path ($size bytes)"

        local ret=0
        python3 "$TRANSLATOR" "$fr_file" "$en_file" 2>&1 || ret=$?

        if [[ "$ret" -eq 0 ]]; then
            log_ok "Translated: $rel_path"
            ((success++)) || true
        elif [[ "$ret" -eq 2 ]]; then
            log_warn "Translated with warnings: $rel_path"
            ((success++)) || true
            ((warnings++)) || true
        else
            log_error "Failed: $rel_path"
            ((failed++)) || true
        fi

        echo "---"
    done

    echo ""
    log_info "=== Translation Summary ==="
    log_info "Total: $total"
    log_ok "Success: $success"
    if [[ "$warnings" -gt 0 ]]; then
        log_warn "With warnings: $warnings"
    fi
    if [[ "$failed" -gt 0 ]]; then
        log_error "Failed: $failed"
    fi

    if [[ "$failed" -gt 0 ]]; then
        exit 1
    fi
}

main "$@"
