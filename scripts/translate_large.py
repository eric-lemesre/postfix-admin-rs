#!/usr/bin/env python3
"""Translate large markdown files by splitting into sections.

Splits on H2 (##) boundaries, translates each section separately,
then reassembles.
"""

import json
import os
import re
import sys
import urllib.request

OLLAMA_URL = "http://localhost:11434/api/generate"
MODEL = os.environ.get("OLLAMA_MODEL", "magistral:latest")
TEMPERATURE = 0.1
NUM_PREDICT = 8192
NUM_CTX = 16384

SYSTEM_PROMPT = """You are a professional technical translator. Translate the following French markdown section to English.

STRICT RULES:
1. Preserve ALL markdown formatting exactly (headings, lists, bold, italic, code blocks, tables, links, images)
2. Do NOT translate content inside code blocks (```...```) or inline code (`...`)
3. Do NOT translate URLs, file paths, or link targets
4. Keep technical terms in their original form when they are standard (API, REST, gRPC, TOTP, RBAC, DKIM, SQL, CSS, JavaScript, Rust, etc.)
5. Translate heading text but keep the same heading level (#, ##, ###, etc.)
6. Preserve all link references and anchor names
7. Do NOT add any commentary, notes, or explanations - output ONLY the translated section
8. Translate naturally - produce fluent English, not word-for-word translation
9. Keep the exact same document structure and line breaks
10. Preserve emoji if present
11. Do NOT wrap the output in a code block - output raw markdown directly"""


def split_into_sections(content: str) -> list[str]:
    """Split markdown content on H2 boundaries."""
    lines = content.split('\n')
    sections = []
    current = []

    for line in lines:
        if line.startswith('## ') and current:
            sections.append('\n'.join(current))
            current = [line]
        else:
            current.append(line)

    if current:
        sections.append('\n'.join(current))

    return sections


def translate_section(text: str, section_num: int, total: int) -> str:
    """Translate a single section via Ollama."""
    prompt = (
        "Translate this French markdown section to English. "
        "Output ONLY the translated markdown, nothing else.\n\n---\n"
        + text
        + "\n---"
    )

    payload = json.dumps({
        "model": MODEL,
        "prompt": prompt,
        "system": SYSTEM_PROMPT,
        "stream": False,
        "options": {
            "temperature": TEMPERATURE,
            "num_predict": NUM_PREDICT,
            "num_ctx": NUM_CTX,
        },
    }).encode("utf-8")

    req = urllib.request.Request(
        OLLAMA_URL,
        data=payload,
        headers={"Content-Type": "application/json"},
    )

    print(f"  Section {section_num}/{total} ({len(text)} chars)...", end=" ", flush=True)

    try:
        with urllib.request.urlopen(req, timeout=300) as resp:
            data = json.loads(resp.read().decode("utf-8"))
    except Exception as e:
        print(f"ERROR: {e}")
        return None

    result = data.get("response", "")
    # Strip code fences
    result = re.sub(r"^```(?:markdown|md)?\s*\n", "", result)
    result = re.sub(r"\n```\s*$", "", result)

    if not result.strip():
        print("ERROR: empty response")
        return None

    print("OK")
    return result


def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <input.md> <output.md>", file=sys.stderr)
        sys.exit(1)

    input_path = sys.argv[1]
    output_path = sys.argv[2]

    with open(input_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Check if banner already present (from add-lang-banners.py)
    banner = None
    if content.startswith('> **Language:**'):
        first_nl = content.index('\n\n')
        banner = content[:first_nl]
        content = content[first_nl + 2:]

    sections = split_into_sections(content)
    print(f"Split into {len(sections)} sections")

    translated_sections = []
    for i, section in enumerate(sections, 1):
        result = translate_section(section, i, len(sections))
        if result is None:
            print(f"Failed at section {i}, aborting")
            sys.exit(1)
        translated_sections.append(result)

    translated = '\n'.join(translated_sections)

    # Read existing banner from EN file if it exists
    en_banner = None
    if os.path.exists(output_path):
        with open(output_path, 'r', encoding='utf-8') as f:
            existing = f.read()
        if existing.startswith('> **Language:**'):
            en_nl = existing.index('\n\n')
            en_banner = existing[:en_nl]

    with open(output_path, 'w', encoding='utf-8') as f:
        if en_banner:
            f.write(en_banner + '\n\n')
        f.write(translated)
        if not translated.endswith('\n'):
            f.write('\n')

    print(f"Done! Output: {output_path}")


if __name__ == '__main__':
    main()
