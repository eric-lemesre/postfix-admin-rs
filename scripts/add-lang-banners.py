#!/usr/bin/env python3
"""Add language navigation banners to all documentation files.

EN: > **Language:** English | [Francais](../fr/<path>)
FR: > **Language:** [English](../en/<path>) | Francais

Root EN: > **Language:** English | [Francais](README.fr.md)
Root FR: > **Language:** [English](README.md) | Francais
"""

import os
import sys

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

BANNER_MARKER = '> **Language:**'


def already_has_banner(content: str) -> bool:
    return BANNER_MARKER in content.split('\n')[0] if content else False


def add_banner(filepath: str, banner: str) -> bool:
    """Add a language banner at the top of a file. Returns True if changed."""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    if already_has_banner(content):
        return False

    new_content = banner + '\n\n' + content
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(new_content)
    return True


def process_docs():
    """Process all files in docs/en/ and docs/fr/"""
    changed = 0

    for lang in ['en', 'fr']:
        other = 'fr' if lang == 'en' else 'en'
        docs_dir = os.path.join(PROJECT_ROOT, 'docs', lang)

        for root, dirs, files in os.walk(docs_dir):
            dirs.sort()
            for fname in sorted(files):
                if not fname.endswith('.md'):
                    continue

                filepath = os.path.join(root, fname)
                # Relative path within the language tree
                rel = os.path.relpath(filepath, docs_dir)
                # Path to the other language version: ../other_lang/rel
                other_path = f'../{other}/{rel}'

                if lang == 'en':
                    banner = f'> **Language:** English | [Francais]({other_path})'
                else:
                    banner = f'> **Language:** [English]({other_path}) | Francais'

                if add_banner(filepath, banner):
                    relpath = os.path.relpath(filepath, PROJECT_ROOT)
                    print(f"ADDED: {relpath}")
                    changed += 1

    return changed


def process_root():
    """Process root-level files"""
    changed = 0

    pairs = [
        ('README.md', 'README.fr.md'),
        ('CONTRIBUTING.md', 'CONTRIBUTING.fr.md'),
        ('SECURITY.md', 'SECURITY.fr.md'),
    ]

    for en_name, fr_name in pairs:
        en_path = os.path.join(PROJECT_ROOT, en_name)
        fr_path = os.path.join(PROJECT_ROOT, fr_name)

        # EN version
        if os.path.exists(en_path):
            banner = f'> **Language:** English | [Francais]({fr_name})'
            if add_banner(en_path, banner):
                print(f"ADDED: {en_name}")
                changed += 1

        # FR version
        if os.path.exists(fr_path):
            banner = f'> **Language:** [English]({en_name}) | Francais'
            if add_banner(fr_path, banner):
                print(f"ADDED: {fr_name}")
                changed += 1

    return changed


def main():
    total = 0
    total += process_root()
    total += process_docs()
    print(f"\nTotal: {total} files updated with language banners")


if __name__ == '__main__':
    main()
