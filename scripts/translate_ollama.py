#!/usr/bin/env python3
"""Translate a single markdown file from French to English via Ollama API."""

import json
import re
import sys
import urllib.request

OLLAMA_URL = "http://localhost:11434/api/generate"
MODEL = "magistral:latest"
TEMPERATURE = 0.1
NUM_PREDICT = 16384
NUM_CTX = 32768

SYSTEM_PROMPT = """You are a professional technical translator. Translate the following French markdown document to English.

STRICT RULES:
1. Preserve ALL markdown formatting exactly (headings, lists, bold, italic, code blocks, tables, links, images)
2. Do NOT translate content inside code blocks (```...```) or inline code (`...`)
3. Do NOT translate URLs, file paths, or link targets
4. Keep technical terms in their original form when they are standard (API, REST, gRPC, TOTP, RBAC, DKIM, SQL, CSS, JavaScript, Rust, etc.)
5. Translate heading text but keep the same heading level (#, ##, ###, etc.)
6. Preserve all link references and anchor names
7. Do NOT add any commentary, notes, or explanations - output ONLY the translated document
8. Translate naturally - produce fluent English, not word-for-word translation
9. Keep the exact same document structure and line breaks
10. Preserve emoji if present
11. Do NOT wrap the output in a code block - output raw markdown directly"""


def translate(input_path: str, output_path: str) -> bool:
    """Translate a single file. Returns True on success."""
    with open(input_path, "r", encoding="utf-8") as f:
        content = f.read()

    prompt = (
        "Translate the following French markdown document to English. "
        "Output ONLY the translated markdown, nothing else.\n\n---\n"
        + content
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

    try:
        with urllib.request.urlopen(req, timeout=600) as resp:
            data = json.loads(resp.read().decode("utf-8"))
    except Exception as e:
        print(f"ERROR: API call failed: {e}", file=sys.stderr)
        return False

    text = data.get("response", "")
    if not text.strip():
        print("ERROR: Empty response from Ollama", file=sys.stderr)
        return False

    # Strip code fences if the model wrapped the output
    text = re.sub(r"^```(?:markdown|md)?\s*\n", "", text)
    text = re.sub(r"\n```\s*$", "", text)

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(text)
        if not text.endswith("\n"):
            f.write("\n")

    return True


def validate(fr_path: str, en_path: str) -> list[str]:
    """Compare structural elements. Returns list of warnings."""
    warnings = []

    with open(fr_path, "r", encoding="utf-8") as f:
        fr = f.read()
    with open(en_path, "r", encoding="utf-8") as f:
        en = f.read()

    # Headings
    fr_h = len(re.findall(r"^#+\s", fr, re.MULTILINE))
    en_h = len(re.findall(r"^#+\s", en, re.MULTILINE))
    if fr_h != en_h:
        warnings.append(f"heading count: FR={fr_h} EN={en_h}")

    # Code blocks
    fr_cb = fr.count("```")
    en_cb = en.count("```")
    if fr_cb != en_cb:
        warnings.append(f"code block markers: FR={fr_cb} EN={en_cb}")

    # Links
    link_re = re.compile(r"\[.*?\]\(.*?\)")
    fr_l = len(link_re.findall(fr))
    en_l = len(link_re.findall(en))
    if fr_l != en_l:
        warnings.append(f"link count: FR={fr_l} EN={en_l}")

    # Empty check
    if not en.strip():
        warnings.append("translated file is empty")

    return warnings


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <input_fr.md> <output_en.md>", file=sys.stderr)
        sys.exit(1)

    input_path = sys.argv[1]
    output_path = sys.argv[2]

    ok = translate(input_path, output_path)
    if not ok:
        sys.exit(1)

    warns = validate(input_path, output_path)
    if warns:
        for w in warns:
            print(f"WARN: {w}", file=sys.stderr)
        sys.exit(2)  # success but with warnings

    sys.exit(0)
