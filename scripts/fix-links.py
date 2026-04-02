#!/usr/bin/env python3
"""Fix internal links after multilingual restructuring.

Handles:
1. Root files (README.md, CONTRIBUTING.md, SECURITY.md): docs/X -> docs/en/X
2. Root files FR (*.fr.md): docs/X -> docs/fr/X
3. docs/fr/features/00-overview.md: ../../LICENSE -> ../../../LICENSE
4. docs/en/features/00-overview.md: ../../LICENSE -> ../../../LICENSE
5. SECURITY.md EN: #10-securite -> #10-security in link targets
6. Root EN files: CONTRIBUTING.md link text may need adjustment
"""

import os
import re
import sys

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def fix_root_en_links(content: str) -> str:
    """Fix links in root EN files: docs/X -> docs/en/X"""
    # Match markdown links pointing to docs/ (but not docs/en/ or docs/fr/)
    def replace_docs_link(m):
        prefix = m.group(1)
        path = m.group(2)
        suffix = m.group(3)
        return f"{prefix}docs/en/{path}{suffix}"

    content = re.sub(
        r'(\[.*?\]\()docs/(?!en/|fr/)(.*?)(\))',
        replace_docs_link,
        content,
    )
    return content


def fix_root_fr_links(content: str) -> str:
    """Fix links in root FR files: docs/X -> docs/fr/X"""
    def replace_docs_link(m):
        prefix = m.group(1)
        path = m.group(2)
        suffix = m.group(3)
        return f"{prefix}docs/fr/{path}{suffix}"

    content = re.sub(
        r'(\[.*?\]\()docs/(?!en/|fr/)(.*?)(\))',
        replace_docs_link,
        content,
    )
    return content


def fix_security_en_anchor(content: str) -> str:
    """Fix the anchor #10-securite -> #10-security in SECURITY.md EN"""
    content = content.replace('#10-securite', '#10-security')
    return content


def fix_license_depth(content: str) -> str:
    """Fix ../../LICENSE -> ../../../LICENSE (one level deeper after restructuring)"""
    content = content.replace('](../../LICENSE)', '](../../../LICENSE)')
    return content


def fix_contributing_fr_link(content: str) -> str:
    """In README.fr.md, update CONTRIBUTING.md link to CONTRIBUTING.fr.md"""
    content = content.replace('](CONTRIBUTING.md)', '](CONTRIBUTING.fr.md)')
    return content


def fix_readme_en_contributing_link(content: str) -> str:
    """In README.md EN, ensure CONTRIBUTING.md link stays as is (already correct)"""
    # No change needed - CONTRIBUTING.md is the EN version
    return content


def process_file(filepath: str) -> bool:
    """Process a single file. Returns True if changes were made."""
    with open(filepath, 'r', encoding='utf-8') as f:
        original = f.read()

    content = original
    basename = os.path.basename(filepath)
    relpath = os.path.relpath(filepath, PROJECT_ROOT)

    # Root EN files
    if relpath == 'README.md':
        content = fix_root_en_links(content)
        content = fix_readme_en_contributing_link(content)
    elif relpath == 'CONTRIBUTING.md':
        content = fix_root_en_links(content)
    elif relpath == 'SECURITY.md':
        content = fix_root_en_links(content)
        content = fix_security_en_anchor(content)
    # Root FR files
    elif relpath == 'README.fr.md':
        content = fix_root_fr_links(content)
        content = fix_contributing_fr_link(content)
    elif relpath == 'CONTRIBUTING.fr.md':
        content = fix_root_fr_links(content)
    elif relpath == 'SECURITY.fr.md':
        content = fix_root_fr_links(content)
    # docs/fr/ and docs/en/ files with LICENSE links
    elif relpath.startswith('docs/') and '../../LICENSE' in content:
        content = fix_license_depth(content)

    if content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False


def main():
    files_to_check = [
        'README.md', 'README.fr.md',
        'CONTRIBUTING.md', 'CONTRIBUTING.fr.md',
        'SECURITY.md', 'SECURITY.fr.md',
    ]

    changed = 0
    unchanged = 0

    # Process root files
    for name in files_to_check:
        filepath = os.path.join(PROJECT_ROOT, name)
        if os.path.exists(filepath):
            if process_file(filepath):
                print(f"FIXED: {name}")
                changed += 1
            else:
                print(f"OK:    {name} (no changes needed)")
                unchanged += 1

    # Process docs/ files (both en/ and fr/)
    for lang in ['en', 'fr']:
        docs_dir = os.path.join(PROJECT_ROOT, 'docs', lang)
        for root, dirs, files in os.walk(docs_dir):
            for fname in files:
                if fname.endswith('.md'):
                    filepath = os.path.join(root, fname)
                    relpath = os.path.relpath(filepath, PROJECT_ROOT)
                    if process_file(filepath):
                        print(f"FIXED: {relpath}")
                        changed += 1
                    else:
                        unchanged += 1

    print(f"\nSummary: {changed} files fixed, {unchanged} unchanged")


if __name__ == '__main__':
    main()
