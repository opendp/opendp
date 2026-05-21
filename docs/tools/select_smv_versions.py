#!/usr/bin/env python3
"""Prepare a sphinx-multiversion tag whitelist for recent OpenDP releases."""

import re
import subprocess
from pathlib import Path

DEFAULT_NUM_RELEASES = 8
TAG_PATTERN = re.compile(r"v(\d+)\.(\d+)\.(\d+)$")


def list_release_tags(repo_root: Path) -> list[str]:
    args = "git tag --list v*".split()
    return subprocess.check_output(args, cwd=repo_root, text=True).splitlines()


def select_recent_release_tags(tags: list[str]) -> str:
    """
    Return a regex matching the latest patch release from the most recent release lines.

    >>> select_recent_release_tags(
    ...     ["v0.11.0", "v0.11.1", "v0.12.0", "v0.12.0-rc.1", "not-a-tag"],
    ... )
    '^(v0\\\\.11\\\\.1|v0\\\\.12\\\\.0)$'
    """
    pareto_versions: dict[tuple[int, int], tuple[int, str]] = {}
    for tag in tags:
        match = TAG_PATTERN.fullmatch(tag)
        if match is None:
            continue

        major, minor, patch = map(int, match.groups())
        current = pareto_versions.get((major, minor))
        if current is None or patch > current[0]:
            pareto_versions[(major, minor)] = (patch, tag)

    selected_versions = sorted(pareto_versions, reverse=True)[:DEFAULT_NUM_RELEASES]
    selected_tags = sorted(pareto_versions[v][1] for v in selected_versions)
    return r"^(%s)$" % "|".join(re.escape(tag) for tag in selected_tags)


if __name__ == "__main__":
    repo_root = Path(__file__).resolve().parents[2]
    print(select_recent_release_tags(list_release_tags(repo_root)))
