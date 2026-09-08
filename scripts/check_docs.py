#!/usr/bin/env python3
"""Check inline/reference Markdown file links and paired English/Chinese pages.

Uses Git's file list so generated/vendor/local notes are excluded. Checks local
file targets only, not heading anchors or external URLs. Requires only Python.
"""

from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]


def markdown_files():
    paths = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", "*.md"],
        cwd=ROOT,
    ).decode().split("\0")
    return sorted({p for p in paths if p and not p.startswith("third_party/")})


def check():
    errors = []
    files = markdown_files()
    targets = 0
    for name in files:
        path = ROOT / name
        if not path.exists():
            continue  # A tracked deletion is not an active document.
        fenced = False
        fence_char = ""
        fence_size = 0
        for number, line in enumerate(path.read_text().splitlines(), 1):
            fence = re.match(r"^\s{0,3}(`{3,}|~{3,})", line)
            if fence:
                token = fence[1]
                if not fenced:
                    fenced, fence_char, fence_size = True, token[0], len(token)
                elif token[0] == fence_char and len(token) >= fence_size:
                    fenced = False
                continue
            if fenced:
                continue
            # Markdown used in this repo: inline links/images and definitions.
            links = re.findall(r"\]\(\s*(<[^>]+>|[^\s)]+)", line)
            definition = re.match(r"^\s{0,3}\[[^]]+\]:\s*(<[^>]+>|\S+)", line)
            if definition:
                links.append(definition[1])
            for link in links:
                link = link.strip("<>")
                url = urlsplit(link)
                if url.scheme or url.netloc or not url.path:
                    continue
                targets += 1
                target = unquote(url.path)
                resolved = ROOT / target.lstrip("/") if target.startswith("/") else path.parent / target
                if not resolved.exists():
                    errors.append(f"{name}:{number}: missing local target {link}")
    for lang, other in (("en", "zh"), ("zh", "en")):
        for path in (ROOT / "docs" / lang).glob("*.md"):
            if not (ROOT / "docs" / other / path.name).exists():
                errors.append(f"{path.relative_to(ROOT)}: missing {other} counterpart")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"OK: {len(files)} Markdown files, {targets} local file links, paired en/zh pages")
    return 0


if __name__ == "__main__":
    raise SystemExit(check())
