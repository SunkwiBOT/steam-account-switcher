#!/usr/bin/env python3
"""Keeps the project version in sync across the files that declare it.

    python3 .github/bump-version.py --current          # version the project declares
    python3 .github/bump-version.py --check-next 0.4.2 # may it follow the last release?
    python3 .github/bump-version.py 0.4.2              # write it everywhere

The release workflow passes the version typed when it is started by hand.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

APP_DIR = Path(__file__).resolve().parent.parent
TAURI_CONF = APP_DIR / "src-tauri" / "tauri.conf.json"
WORKSPACE_CARGO = APP_DIR / "Cargo.toml"
CARGO_LOCK = APP_DIR / "Cargo.lock"
PACKAGE_JSON = APP_DIR / "package.json"
PACKAGE_LOCK = APP_DIR / "package-lock.json"
PACKAGES = ("steam-account-switcher", "steam-core")


def current_version() -> str:
    return json.loads(TAURI_CONF.read_text(encoding="utf-8"))["version"]


def released_versions() -> list[str]:
    """Published versions, newest first."""
    try:
        output = subprocess.run(
            ["git", "tag", "--list", "v*", "--sort=-v:refname"],
            cwd=APP_DIR,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError):
        return []
    return [line.strip().removeprefix("v") for line in output.splitlines() if line.strip()]


def check_next(candidate: str) -> str | None:
    """Refuses anything that is not the natural follower of the last release."""
    if not re.fullmatch(r"\d+\.\d+\.\d+", candidate):
        raise SystemExit(f"invalid version: {candidate!r} (expected X.Y.Z)")

    released = released_versions()
    if candidate in released:
        raise SystemExit(f"v{candidate} is already published")

    if not released:
        return None

    major, minor, patch = (int(part) for part in released[0].split("."))
    allowed = {
        f"{major}.{minor}.{patch + 1}",  # patch release
        f"{major}.{minor + 1}.0",  # feature release
        f"{major + 1}.0.0",  # breaking release
    }
    if candidate not in allowed:
        raise SystemExit(
            f"v{candidate} does not follow v{released[0]}: "
            f"expected one of {', '.join(sorted(allowed))}"
        )
    return released[0]


def write_version(version: str) -> list[str]:
    """Rewrites every version declaration, returning the files that changed."""
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise SystemExit(f"invalid version: {version!r} (expected X.Y.Z)")

    changed: list[str] = []

    # Rewrite only the version line: re-serialising the whole file would
    # reflow the hand written formatting for nothing.
    conf_text = TAURI_CONF.read_text(encoding="utf-8")
    updated_conf, count = re.subn(
        r'^(\s*"version":\s*)"[^"]*"',
        rf'\1"{version}"',
        conf_text,
        count=1,
        flags=re.MULTILINE,
    )
    if count and updated_conf != conf_text:
        TAURI_CONF.write_text(updated_conf, encoding="utf-8")
        changed.append(str(TAURI_CONF.relative_to(APP_DIR)))

    # The workspace manifest carries the version for every crate.
    cargo_text = WORKSPACE_CARGO.read_text(encoding="utf-8")
    updated_cargo, count = re.subn(
        r'(^\[workspace\.package\][\s\S]*?^version = )"[^"]*"',
        rf'\1"{version}"',
        cargo_text,
        count=1,
        flags=re.MULTILINE,
    )
    if count and updated_cargo != cargo_text:
        WORKSPACE_CARGO.write_text(updated_cargo, encoding="utf-8")
        changed.append(str(WORKSPACE_CARGO.relative_to(APP_DIR)))

    # Cargo.lock repeats it for each of our own packages.
    lock_text = CARGO_LOCK.read_text(encoding="utf-8")
    updated_lock = lock_text
    for package in PACKAGES:
        updated_lock = re.sub(
            rf'(name = "{re.escape(package)}"\nversion = )"[^"]*"',
            rf'\1"{version}"',
            updated_lock,
        )
    if updated_lock != lock_text:
        CARGO_LOCK.write_text(updated_lock, encoding="utf-8")
        changed.append(str(CARGO_LOCK.relative_to(APP_DIR)))

    # The interface package follows the same version, so tooling that reads it
    # (npm, editors) never shows a version the build does not have.
    package_text = PACKAGE_JSON.read_text(encoding="utf-8")
    updated_package, count = re.subn(
        r'^(\s*"version":\s*)"[^"]*"',
        rf'\1"{version}"',
        package_text,
        count=1,
        flags=re.MULTILINE,
    )
    if count and updated_package != package_text:
        PACKAGE_JSON.write_text(updated_package, encoding="utf-8")
        changed.append(str(PACKAGE_JSON.relative_to(APP_DIR)))

    lock_package_text = PACKAGE_LOCK.read_text(encoding="utf-8")
    updated_package_lock = re.sub(
        r'^(  "version":\s*)"[^"]*"',
        rf'\1"{version}"',
        lock_package_text,
        count=1,
        flags=re.MULTILINE,
    )
    updated_package_lock = re.sub(
        r'^(\s*"": \{\n\s*"name": "[^"]*",\n\s*"version":\s*)"[^"]*"',
        rf'\1"{version}"',
        updated_package_lock,
        count=1,
        flags=re.MULTILINE,
    )
    if updated_package_lock != lock_package_text:
        PACKAGE_LOCK.write_text(updated_package_lock, encoding="utf-8")
        changed.append(str(PACKAGE_LOCK.relative_to(APP_DIR)))

    return changed


def main() -> int:
    args = sys.argv[1:]
    if args == ["--current"]:
        print(current_version())
        return 0
    if len(args) == 2 and args[0] == "--check-next":
        previous = check_next(args[1])
        print(
            f"v{args[1]} follows v{previous}" if previous else f"v{args[1]} is the first release"
        )
        return 0
    if len(args) != 1:
        print(__doc__, file=sys.stderr)
        return 1

    changed = write_version(args[0])
    print(f"version {args[0]}: {', '.join(changed) if changed else 'already up to date'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
