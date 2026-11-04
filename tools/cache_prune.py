"""Reclaim the Rust build caches that a finished release build has made dead.

Cargo never deletes anything, and in this workspace three caches grow with
every engine pin move and every release build until the disk fills:

1. ``~/.cargo/git/checkouts/pdfcer-*/<rev>/`` holds one full copy of the engine
   per revision ever locked. Only the revision in ``Cargo.lock`` is live, and
   cargo recreates any other one from the bare ``db/`` repository on demand.
2. ``target/release/deps`` keeps every superseded artifact. A file is live
   exactly when its ``-<16 hex>`` metadata hash appears among the
   ``compiler-artifact`` messages of a no-op ``cargo build --message-format=json``
   (cargo reports fresh units too). Everything else is dead. Age is the wrong
   test: cargo legitimately reuses a third-party ``.rlib`` built weeks ago.
3. ``target/<profile>/incremental/<crate>-<hash>/`` keeps one directory per
   build configuration a crate has ever had. Only the newest per crate is used
   by the next build. The older ones are dead.

``target/debug`` is built only by the gates and tests and is never shipped, so
when it passes ``DEBUG_LIMIT_GB`` it is deleted whole. That costs one debug
rebuild.

Every step is safe to lose. Cargo rebuilds any output that is missing, so a
wrong deletion costs build time and can never produce a wrong binary.

Called at the end of ``package-portable.py``, and runnable on its own:
``python tools/cache_prune.py``. Run alone it lists the live set with this
shell's environment, so if the last build was packaged with a
``PDFCER_BUILD_STAMP`` cargo rebuilds the GUI once first.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import stat
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TARGET = REPO / "target"
CHECKOUTS = Path.home() / ".cargo" / "git" / "checkouts"

#: Size above which ``target/debug`` is deleted whole.
DEBUG_LIMIT_GB = 20

_HASH = re.compile(r"-([0-9a-f]{16})(?:\.[A-Za-z0-9_.]+)?$")


def _rm(path: Path) -> None:
    def force(func, p, _exc):
        os.chmod(p, stat.S_IWRITE)
        func(p)

    if path.is_dir():
        shutil.rmtree(path, onerror=force)
    else:
        path.unlink(missing_ok=True)


def _size(path: Path) -> int:
    total = 0
    for root, _dirs, files in os.walk(path):
        for f in files:
            try:
                total += os.lstat(os.path.join(root, f)).st_size
            except OSError:
                pass
    return total


def prune_pin_checkouts() -> int:
    """Delete every engine checkout whose revision ``Cargo.lock`` does not name."""
    lock = (REPO / "Cargo.lock").read_text(encoding="utf-8")
    live = set(re.findall(r"git\+file:///D:/Dev/pdfcer\?[^#\"]*#([0-9a-f]{7})", lock))
    if not live:
        return 0  # The lock does not name the engine by git; touch nothing.
    freed = 0
    for repo_dir in CHECKOUTS.glob("pdfcer-*"):
        for rev in repo_dir.iterdir():
            if rev.is_dir() and rev.name not in live:
                freed += _size(rev)
                _rm(rev)
    return freed


def live_release_hashes(env: dict[str, str]) -> set[str] | None:
    """The metadata hashes of every live release unit, or None if cargo failed.

    ``env`` must match the build just made: ``PDFCER_BUILD_STAMP`` is a rerun
    trigger, so a different value would rebuild the GUI instead of reporting it.
    """
    proc = subprocess.run(
        ["cargo", "build", "--release", "--workspace", "--locked", "--message-format=json"],
        cwd=REPO,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    if proc.returncode != 0:
        return None
    live: set[str] = set()
    for line in proc.stdout.splitlines():
        if '"compiler-artifact"' not in line:
            continue
        msg = json.loads(line)
        for name in msg.get("filenames", []):
            m = _HASH.search(Path(name).name)
            if m:
                live.add(m.group(1))
    return live or None


def prune_release_deps(env: dict[str, str]) -> int:
    live = live_release_hashes(env)
    if live is None:
        print("cache-prune: cargo could not list the live set; release/deps left alone")
        return 0
    freed = 0
    for f in (TARGET / "release" / "deps").iterdir():
        m = _HASH.search(f.name)
        if m and m.group(1) not in live:
            freed += f.stat().st_size if f.is_file() else _size(f)
            _rm(f)
    return freed


def prune_incremental(profile: str) -> int:
    """Keep only the newest incremental directory per crate."""
    inc = TARGET / profile / "incremental"
    if not inc.is_dir():
        return 0
    newest: dict[str, Path] = {}
    stale: list[Path] = []
    for d in inc.iterdir():
        crate = d.name.rsplit("-", 1)[0]
        best = newest.get(crate)
        if best is None:
            newest[crate] = d
        elif d.stat().st_mtime > best.stat().st_mtime:
            stale.append(best)
            newest[crate] = d
        else:
            stale.append(d)
    freed = 0
    for d in stale:
        freed += _size(d)
        _rm(d)
    return freed


def prune_debug() -> int:
    debug = TARGET / "debug"
    if not debug.is_dir():
        return 0
    size = _size(debug)
    if size < DEBUG_LIMIT_GB * 1024**3:
        return prune_incremental("debug")
    _rm(debug)
    return size


def run(env: dict[str, str] | None = None) -> None:
    """Run every prune and print one line with the total reclaimed."""
    env = dict(os.environ if env is None else env)
    steps = [
        ("old engine copies", prune_pin_checkouts),
        ("dead release artifacts", lambda: prune_release_deps(env)),
        ("old release incremental", lambda: prune_incremental("release")),
        ("debug cache", prune_debug),
    ]
    parts = []
    total = 0
    for label, step in steps:
        try:
            freed = step()
        except OSError as exc:  # A locked file only costs this step.
            print(f"cache-prune: {label}: {exc}")
            continue
        total += freed
        if freed:
            parts.append(f"{label} {freed / 1024**3:.1f} GB")
    detail = "; ".join(parts) if parts else "nothing dead"
    print(f"cache-prune: reclaimed {total / 1024**3:.1f} GB ({detail})")


if __name__ == "__main__":
    run()
