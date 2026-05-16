#!/usr/bin/env python3
"""
TEMPORARY debug helper for plugin registry validation.

Reuses logic from .github/validate_links.py (loaded dynamically; that file is not modified).
Delete this script when you are done testing.

Default: only plugins/*.json whose filename contains ``viewerofall-`` (your registry entries).

Usage:
  cd /path/to/dms-plugin-registry
  python3 debug_validate_links.py
  python3 debug_validate_links.py plugins/viewerofall-foo.json
  CHANGED_PLUGINS="plugins/viewerofall-foo.json" python3 debug_validate_links.py

Environment:
  GITHUB_TOKEN   optional; same as validate_links.py for GitHub API / raw rate limits
"""

from __future__ import annotations

import importlib.util
import json
import os
import sys
from pathlib import Path
from urllib.parse import urlparse

# --- load validate_links.py without importing as package ---
_REGISTRY_ROOT = Path(__file__).resolve().parent
_VALIDATE_LINKS = _REGISTRY_ROOT / ".github" / "validate_links.py"

if not _VALIDATE_LINKS.is_file():
    print(f"ERROR: expected {_VALIDATE_LINKS}", file=sys.stderr)
    sys.exit(2)

_spec = importlib.util.spec_from_file_location("validate_links", _VALIDATE_LINKS)
if _spec is None or _spec.loader is None:
    print("ERROR: could not load validate_links spec", file=sys.stderr)
    sys.exit(2)
vl = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(vl)


def _divider(title: str) -> None:
    print()
    print("=" * 72)
    print(f"  {title}")
    print("=" * 72)


def _github_owner_repo(repo_url: str) -> tuple[str | None, str | None, str]:
    """Return (owner, repo, error). Ignores /tree/... segments after repo name."""
    parsed = urlparse(repo_url)
    repo_path = parsed.path.strip("/")
    if repo_path.endswith(".git"):
        repo_path = repo_path[:-4]
    parts = repo_path.split("/")
    if len(parts) < 2:
        return None, None, "could not parse owner/repo from repo URL"
    return parts[0], parts[1], ""


def debug_one_plugin(plugin_file: Path) -> list[str]:
    errors: list[str] = []

    _divider(f"FILE: {plugin_file}")

    if not plugin_file.is_file():
        msg = f"Not a file: {plugin_file}"
        print(msg)
        return [msg]

    try:
        text = plugin_file.read_text(encoding="utf-8")
        plugin = json.loads(text)
    except json.JSONDecodeError as e:
        msg = f"Invalid JSON: {e}"
        print(msg)
        return [msg]
    except OSError as e:
        msg = f"Read error: {e}"
        print(msg)
        return [msg]

    print("Parsed registry JSON keys:", sorted(plugin.keys()))
    print()

    # id + camelCase
    plugin_id = plugin.get("id", "<missing>")
    plugin_name = plugin.get("name", plugin_file.stem)
    print(f"id (registry):     {plugin_id!r}")
    print(f"name (registry):   {plugin_name!r}")
    if plugin_id != "<missing>":
        ok_cc = vl.is_camel_case(plugin_id)
        print(f"is_camel_case(id): {ok_cc}")
        if not ok_cc:
            err = f"ID '{plugin_id}' is not camelCase (^[a-z][a-zA-Z0-9]*$)"
            print(f"  -> {err}")
            errors.append(err)
    else:
        errors.append("Missing required 'id' property")
    print()

    # screenshot
    shot = plugin.get("screenshot", "<missing>")
    print(f"screenshot URL: {shot!r}")
    if shot and shot != "<missing>":
        ok, msg = vl.validate_url(shot)
        print(f"validate_url(screenshot): ok={ok} detail={msg or '(none)'}")
        if not ok:
            errors.append(f"Screenshot URL unreachable: {msg}")
    else:
        errors.append("Missing required 'screenshot' property")
    print()

    # repo + path + remote plugin.json
    repo_url = plugin.get("repo", "<missing>")
    subpath = plugin.get("path") or ""
    print(f"repo URL: {repo_url!r}")
    print(f"path (monorepo): {subpath!r}")

    owner, repo, parse_err = (None, None, "")
    if repo_url and repo_url != "<missing>":
        owner, repo, parse_err = _github_owner_repo(repo_url)
        print(f"parsed owner/repo: {owner!r} / {repo!r} {parse_err}")
        ok, msg = vl.validate_url(repo_url)
        print(f"validate_url(repo): ok={ok} detail={msg or '(none)'}")
        if not ok:
            errors.append(f"Repository URL unreachable: {msg}")
        else:
            if subpath:
                api = f"https://api.github.com/repos/{owner}/{repo}/contents/{subpath}"
                print(f"GitHub contents API (path check): {api}")
                ok_p, msg_p = vl.validate_repo_path(repo_url, subpath)
                print(f"validate_repo_path: ok={ok_p} detail={msg_p or '(none)'}")
                if not ok_p:
                    errors.append(f"Path validation failed: {msg_p}")
                elif msg_p:
                    print(f"  note: {msg_p}")

            print()
            print("Fetching remote plugin.json (same branches as validate_links: main, master)...")
            remote, ferr = vl.fetch_plugin_json(repo_url, subpath)
            if remote is None:
                print(f"fetch_plugin_json FAILED: {ferr}")
                errors.append(f"Failed to fetch repository plugin.json: {ferr}")
            else:
                print("Remote plugin.json (subset):")
                for k in ("id", "name", "version", "type", "author"):
                    if k in remote:
                        print(f"  {k}: {remote[k]!r}")
                if "id" in plugin and remote.get("id") is not None:
                    rid, lid = remote.get("id"), plugin["id"]
                    match = rid == lid
                    print(f"id match (remote == registry): {match}  ({rid!r} vs {lid!r})")
                    if not match:
                        errors.append(
                            f"ID mismatch: registry has {lid!r} but repository plugin.json has {rid!r}"
                        )
                if "name" in plugin and remote.get("name") is not None:
                    rn, ln = remote.get("name"), plugin_name
                    match = rn == ln
                    print(f"name match (remote == registry): {match}  ({rn!r} vs {ln!r})")
                    if not match:
                        errors.append(
                            f"Name mismatch: registry has {ln!r} but repository plugin.json has {rn!r}"
                        )

    else:
        errors.append("Missing required 'repo' property")

    # Cross-check: same errors as CI would collect from validate_plugin()
    _divider("CI parity: validate_links.validate_plugin()")
    ci_errors = vl.validate_plugin(plugin_file)
    if ci_errors:
        print("validate_plugin() returned:")
        for e in ci_errors:
            print(f"  - {e}")
    else:
        print("validate_plugin(): no errors (OK)")
    print()

    # Compare our collected vs CI (should match; if not, bug in this debug script)
    if set(errors) != set(ci_errors):
        print("WARNING: debug script error list differs from validate_plugin().")
        print("  debug-only errors:", set(errors) - set(ci_errors))
        print("  CI-only errors:", set(ci_errors) - set(errors))

    return ci_errors


def _is_viewerofall_plugin_json(path: Path) -> bool:
    """True if this registry file is one of viewerofall's plugins (filename contains viewerofall-)."""
    return "viewerofall-" in path.name


def _filter_viewerofall_only(paths: list[Path]) -> tuple[list[Path], list[Path]]:
    """Return (kept, skipped) for paths that are / are not viewerofall plugin JSONs."""
    kept, skipped = [], []
    for p in paths:
        if _is_viewerofall_plugin_json(p):
            kept.append(p)
        else:
            skipped.append(p)
    return kept, skipped


def _get_changed_plugin_files() -> set[str]:
    raw = os.environ.get("CHANGED_PLUGINS", "").strip()
    if not raw:
        return set()
    return {Path(p.strip()).name for p in raw.splitlines() if p.strip()}


def main() -> None:
    plugins_dir = _REGISTRY_ROOT / "plugins"
    if not plugins_dir.is_dir():
        print(f"ERROR: plugins/ not found under {_REGISTRY_ROOT}", file=sys.stderr)
        sys.exit(2)

    argv_files = [Path(a) for a in sys.argv[1:] if not a.startswith("-")]
    changed = _get_changed_plugin_files()

    if argv_files:
        resolved = [f.resolve() if f.is_absolute() else _REGISTRY_ROOT / f for f in argv_files]
        plugin_files, skipped_argv = _filter_viewerofall_only(resolved)
        for p in skipped_argv:
            print(f"SKIP (not viewerofall-*): {p}", file=sys.stderr)
        if skipped_argv:
            print("", file=sys.stderr)
    elif changed:
        candidates = [p for p in sorted(plugins_dir.glob("*.json")) if p.name in changed]
        plugin_files, skipped_changed = _filter_viewerofall_only(candidates)
        for p in skipped_changed:
            print(
                f"SKIP CHANGED_PLUGINS entry (not viewerofall-* filename): {p.name}",
                file=sys.stderr,
            )
        if not plugin_files:
            print(
                f"No viewerofall-* plugins in CHANGED_PLUGINS={sorted(changed)!r} "
                f"(filenames must contain 'viewerofall-').",
                file=sys.stderr,
            )
            sys.exit(0)
    else:
        plugin_files = [p for p in sorted(plugins_dir.glob("*.json")) if _is_viewerofall_plugin_json(p)]
        print(
            f"NOTE: validating {len(plugin_files)} viewerofall-* plugin file(s) only.\n",
            file=sys.stderr,
        )

    if not plugin_files:
        print(
            "No plugin files to validate (need filename containing 'viewerofall-').",
            file=sys.stderr,
        )
        sys.exit(0)

    print("validate_links.py loaded from:", _VALIDATE_LINKS)
    print("GITHUB_TOKEN set:", bool(os.environ.get("GITHUB_TOKEN")))
    print()

    all_ci_errors: dict[str, list[str]] = {}
    for pf in plugin_files:
        ci_errs = debug_one_plugin(pf)
        if ci_errs:
            all_ci_errors[pf.name] = ci_errs

    _divider("SUMMARY")
    if all_ci_errors:
        print(f"FAILED: {len(all_ci_errors)} plugin file(s)\n")
        for name, errs in sorted(all_ci_errors.items()):
            print(f"  {name}")
            for e in errs:
                print(f"    - {e}")
        sys.exit(1)
    print("All checked plugins passed validate_plugin().")
    sys.exit(0)


if __name__ == "__main__":
    main()
