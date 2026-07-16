#!/usr/bin/env python3
"""Check that repository-local Markdown link destinations exist."""

from pathlib import Path
from urllib.parse import unquote, urlsplit
import sys


ROOT = Path(__file__).resolve().parents[1]


def without_fenced_code(markdown: str) -> str:
    output = []
    fence = None
    for line in markdown.splitlines(keepends=True):
        stripped = line.lstrip()
        marker = stripped[:3]
        if marker in {"```", "~~~"}:
            if fence is None:
                fence = marker
            elif fence == marker:
                fence = None
            output.append("\n" if line.endswith("\n") else "")
        elif fence is None:
            output.append(line)
        else:
            output.append("\n" if line.endswith("\n") else "")
    return "".join(output)


def inline_destinations(markdown: str):
    text = without_fenced_code(markdown)
    offset = 0
    while True:
        link_start = text.find("](", offset)
        if link_start == -1:
            return

        cursor = link_start + 2
        while cursor < len(text) and text[cursor].isspace():
            cursor += 1

        if cursor < len(text) and text[cursor] == "<":
            end = text.find(">", cursor + 1)
            destination = text[cursor + 1 : end] if end != -1 else ""
            offset = end + 1 if end != -1 else cursor + 1
        else:
            start = cursor
            nested = 0
            while cursor < len(text):
                character = text[cursor]
                if character == "\\":
                    cursor += 2
                    continue
                if character == "(":
                    nested += 1
                elif character == ")":
                    if nested == 0:
                        break
                    nested -= 1
                elif character.isspace() and nested == 0:
                    break
                cursor += 1
            destination = text[start:cursor]
            offset = cursor + 1

        yield text.count("\n", 0, link_start) + 1, destination


def main() -> int:
    failures = []
    for document in sorted(ROOT.rglob("*.md")):
        if ".git" in document.parts or "target" in document.parts:
            continue
        markdown = document.read_text(encoding="utf-8")
        for line, destination in inline_destinations(markdown):
            try:
                parsed = urlsplit(destination)
            except ValueError:
                failures.append((document, line, destination, "invalid link destination"))
                continue
            if parsed.scheme or parsed.netloc or destination.startswith("#"):
                continue

            path_text = unquote(parsed.path)
            if path_text.startswith("/"):
                failures.append((document, line, destination, "repository-absolute link"))
                continue

            target = document if not path_text else document.parent / path_text
            if not target.exists():
                failures.append((document, line, destination, "target does not exist"))

    if failures:
        for document, line, destination, reason in failures:
            relative = document.relative_to(ROOT)
            print(f"{relative}:{line}: {destination}: {reason}", file=sys.stderr)
        return 1

    print("All repository-local Markdown link targets exist.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
