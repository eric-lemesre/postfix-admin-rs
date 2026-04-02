#!/usr/bin/env python3
"""Validate the multilingual documentation restructuring.

Checks:
1. All EN files are non-empty
2. Structural comparison (headings, code blocks, links) between FR and EN
3. Language banners present
4. LICENSE links resolve correctly
5. docs/en/ and docs/fr/ have the same file structure
"""

import os
import re
import sys

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
NC = '\033[0m'

errors = 0
warnings = 0


def log_ok(msg):
    print(f"{GREEN}[OK]{NC} {msg}")


def log_warn(msg):
    global warnings
    warnings += 1
    print(f"{YELLOW}[WARN]{NC} {msg}")


def log_error(msg):
    global errors
    errors += 1
    print(f"{RED}[ERROR]{NC} {msg}")


def check_file_structure():
    """Verify docs/en/ and docs/fr/ have the same files."""
    print("\n=== File Structure ===")
    en_dir = os.path.join(PROJECT_ROOT, 'docs', 'en')
    fr_dir = os.path.join(PROJECT_ROOT, 'docs', 'fr')

    en_files = set()
    fr_files = set()

    for root, dirs, files in os.walk(en_dir):
        for f in files:
            if f.endswith('.md'):
                rel = os.path.relpath(os.path.join(root, f), en_dir)
                en_files.add(rel)

    for root, dirs, files in os.walk(fr_dir):
        for f in files:
            if f.endswith('.md'):
                rel = os.path.relpath(os.path.join(root, f), fr_dir)
                fr_files.add(rel)

    if en_files == fr_files:
        log_ok(f"Structure matches: {len(en_files)} files in both en/ and fr/")
    else:
        only_en = en_files - fr_files
        only_fr = fr_files - en_files
        if only_en:
            log_error(f"Only in en/: {only_en}")
        if only_fr:
            log_error(f"Only in fr/: {only_fr}")


def check_non_empty():
    """Verify no EN files are empty."""
    print("\n=== Non-empty Check ===")
    en_dir = os.path.join(PROJECT_ROOT, 'docs', 'en')
    empty_count = 0
    total = 0

    for root, dirs, files in os.walk(en_dir):
        for f in files:
            if f.endswith('.md'):
                total += 1
                filepath = os.path.join(root, f)
                if os.path.getsize(filepath) == 0:
                    rel = os.path.relpath(filepath, PROJECT_ROOT)
                    log_error(f"Empty file: {rel}")
                    empty_count += 1

    # Also check root EN files
    for name in ['README.md', 'CONTRIBUTING.md', 'SECURITY.md']:
        filepath = os.path.join(PROJECT_ROOT, name)
        total += 1
        if os.path.exists(filepath) and os.path.getsize(filepath) == 0:
            log_error(f"Empty file: {name}")
            empty_count += 1

    if empty_count == 0:
        log_ok(f"All {total} EN files are non-empty")


def check_structural_comparison():
    """Compare headings, code blocks, and links between FR and EN."""
    print("\n=== Structural Comparison ===")
    en_dir = os.path.join(PROJECT_ROOT, 'docs', 'en')
    fr_dir = os.path.join(PROJECT_ROOT, 'docs', 'fr')
    checked = 0

    for root, dirs, files in os.walk(fr_dir):
        dirs.sort()
        for fname in sorted(files):
            if not fname.endswith('.md'):
                continue
            fr_path = os.path.join(root, fname)
            rel = os.path.relpath(fr_path, fr_dir)
            en_path = os.path.join(en_dir, rel)

            if not os.path.exists(en_path):
                continue

            with open(fr_path, 'r', encoding='utf-8') as f:
                fr = f.read()
            with open(en_path, 'r', encoding='utf-8') as f:
                en = f.read()

            # Headings
            fr_h = len(re.findall(r'^#+\s', fr, re.MULTILINE))
            en_h = len(re.findall(r'^#+\s', en, re.MULTILINE))
            if abs(fr_h - en_h) > 2:  # Allow small differences
                log_warn(f"{rel}: heading count differs significantly (FR={fr_h}, EN={en_h})")

            # Code blocks
            fr_cb = fr.count('```')
            en_cb = en.count('```')
            if abs(fr_cb - en_cb) > 2:
                log_warn(f"{rel}: code block markers differ (FR={fr_cb}, EN={en_cb})")

            # Links
            link_re = re.compile(r'\[.*?\]\(.*?\)')
            fr_l = len(link_re.findall(fr))
            en_l = len(link_re.findall(en))
            if abs(fr_l - en_l) > 2:
                log_warn(f"{rel}: link count differs (FR={fr_l}, EN={en_l})")

            checked += 1

    log_ok(f"Checked structural similarity for {checked} file pairs")


def check_language_banners():
    """Verify all files have language banners."""
    print("\n=== Language Banners ===")
    missing = 0
    total = 0

    for lang in ['en', 'fr']:
        docs_dir = os.path.join(PROJECT_ROOT, 'docs', lang)
        for root, dirs, files in os.walk(docs_dir):
            for f in files:
                if f.endswith('.md'):
                    total += 1
                    filepath = os.path.join(root, f)
                    with open(filepath, 'r', encoding='utf-8') as fh:
                        first_line = fh.readline()
                    if '> **Language:**' not in first_line:
                        rel = os.path.relpath(filepath, PROJECT_ROOT)
                        log_warn(f"Missing banner: {rel}")
                        missing += 1

    for name in ['README.md', 'README.fr.md', 'CONTRIBUTING.md', 'CONTRIBUTING.fr.md',
                 'SECURITY.md', 'SECURITY.fr.md']:
        filepath = os.path.join(PROJECT_ROOT, name)
        if os.path.exists(filepath):
            total += 1
            with open(filepath, 'r', encoding='utf-8') as fh:
                first_line = fh.readline()
            if '> **Language:**' not in first_line:
                log_warn(f"Missing banner: {name}")
                missing += 1

    if missing == 0:
        log_ok(f"All {total} files have language banners")


def check_license_links():
    """Verify LICENSE links point to valid paths."""
    print("\n=== LICENSE Links ===")
    license_path = os.path.join(PROJECT_ROOT, 'LICENSE')
    if not os.path.exists(license_path):
        log_warn("LICENSE file not found at project root")
        return

    for lang in ['en', 'fr']:
        overview = os.path.join(PROJECT_ROOT, 'docs', lang, 'features', '00-overview.md')
        if os.path.exists(overview):
            with open(overview, 'r', encoding='utf-8') as f:
                content = f.read()
            if '../../../LICENSE' in content:
                log_ok(f"docs/{lang}/features/00-overview.md: LICENSE link correct (../../../LICENSE)")
            elif '../../LICENSE' in content:
                log_error(f"docs/{lang}/features/00-overview.md: LICENSE link incorrect (../../LICENSE instead of ../../../LICENSE)")
            else:
                log_warn(f"docs/{lang}/features/00-overview.md: no LICENSE link found")


def check_root_files():
    """Verify root files exist in both languages."""
    print("\n=== Root Files ===")
    for name in ['README', 'CONTRIBUTING', 'SECURITY']:
        en = os.path.join(PROJECT_ROOT, f'{name}.md')
        fr = os.path.join(PROJECT_ROOT, f'{name}.fr.md')
        if os.path.exists(en) and os.path.exists(fr):
            log_ok(f"{name}: both EN and FR versions exist")
        else:
            if not os.path.exists(en):
                log_error(f"{name}.md (EN) missing")
            if not os.path.exists(fr):
                log_error(f"{name}.fr.md (FR) missing")


def main():
    global errors, warnings
    print("=== PostfixAdminRust Documentation Validation ===")

    check_file_structure()
    check_non_empty()
    check_root_files()
    check_language_banners()
    check_license_links()
    check_structural_comparison()

    print(f"\n=== Summary ===")
    print(f"Errors: {errors}")
    print(f"Warnings: {warnings}")

    if errors > 0:
        sys.exit(1)
    sys.exit(0)


if __name__ == '__main__':
    main()
