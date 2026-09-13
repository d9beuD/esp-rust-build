#!/usr/bin/env python3
import pathlib
import re
import sys
import urllib.error
import urllib.request

URL = re.compile(r"https?://[^\s<>()]+")
TRAILING = ".,;:!?"


def main() -> int:
    failures = []
    for path in pathlib.Path(".").rglob("*.md"):
        for url in URL.findall(path.read_text(encoding="utf-8")):
            url = url.rstrip(TRAILING)
            try:
                with urllib.request.urlopen(url, timeout=20) as response:
                    if response.status >= 400:
                        raise urllib.error.HTTPError(url, response.status, "HTTP error", response.headers, None)
            except (OSError, urllib.error.URLError, urllib.error.HTTPError) as error:
                failures.append(f"{path}: {url}: {error}")
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
