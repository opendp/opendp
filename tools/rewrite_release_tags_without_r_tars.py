#!/usr/bin/env python3

from __future__ import annotations

import argparse
import os
import subprocess
import tempfile


TAR_PATHS = (
    "R/opendp/src/vendor.tar.xz",
    "R/opendp/src/source.tar.xz",
    "R/opendp/src/binary.tar.xz",
)
BACKUP_PREFIX = "refs/rewrites/no-r-tars/"


def git(*args: str, input_text: str | None = None, env: dict[str, str] | None = None) -> str:
    result = subprocess.run(
        ["git", *args],
        check=True,
        text=True,
        input=input_text,
        capture_output=True,
        env=env,
    )
    return result.stdout.strip()


def commit_metadata(commit: str) -> dict[str, object]:
    fmt = "%T%x00%P%x00%an%x00%ae%x00%aI%x00%cn%x00%ce%x00%cI%x00%B"
    out = git("show", "-s", f"--format={fmt}", commit)
    tree, parents, author_name, author_email, author_date, committer_name, committer_email, committer_date, message = (
        out.split("\x00", 8)
    )
    return {
        "tree": tree,
        "parents": parents.split() if parents else [],
        "author_name": author_name,
        "author_email": author_email,
        "author_date": author_date,
        "committer_name": committer_name,
        "committer_email": committer_email,
        "committer_date": committer_date,
        "message": message,
    }


def tree_has_tar_payload(commit: str) -> bool:
    return bool(git("ls-tree", "-r", "--name-only", commit, "--", *TAR_PATHS))


def rewrite_tree_without_tars(commit: str) -> str:
    with tempfile.NamedTemporaryFile(prefix="git-index-", delete=True) as index_file:
        env = os.environ.copy()
        env["GIT_INDEX_FILE"] = index_file.name
        git("read-tree", commit, env=env)
        subprocess.run(
            ["git", "rm", "--cached", "--ignore-unmatch", "--quiet", "--", *TAR_PATHS],
            check=True,
            text=True,
            env=env,
            capture_output=True,
        )
        return git("write-tree", env=env)


def commit_tree(tree: str, parents: list[str], metadata: dict[str, object]) -> str:
    env = os.environ.copy()
    env.update(
        {
            "GIT_AUTHOR_NAME": str(metadata["author_name"]),
            "GIT_AUTHOR_EMAIL": str(metadata["author_email"]),
            "GIT_AUTHOR_DATE": str(metadata["author_date"]),
            "GIT_COMMITTER_NAME": str(metadata["committer_name"]),
            "GIT_COMMITTER_EMAIL": str(metadata["committer_email"]),
            "GIT_COMMITTER_DATE": str(metadata["committer_date"]),
        }
    )
    args = ["commit-tree", tree]
    for parent in parents:
        args.extend(["-p", parent])
    return git(*args, input_text=str(metadata["message"]), env=env)


def ancestry_in_topological_order(commit: str) -> list[str]:
    out = git("rev-list", "--topo-order", "--reverse", commit)
    return out.splitlines() if out else []


def rewrite_root(commit: str, memo: dict[str, str], apply: bool) -> str:
    for current in ancestry_in_topological_order(commit):
        if current in memo:
            continue

        metadata = commit_metadata(current)
        old_parents = list(metadata["parents"])
        new_parents = [memo.get(parent, parent) for parent in old_parents]
        old_tree = str(metadata["tree"])
        has_tar_payload = tree_has_tar_payload(current)

        if not has_tar_payload and new_parents == old_parents:
            memo[current] = current
            continue

        if not apply:
            memo[current] = f"REWRITTEN:{current}"
            continue

        new_tree = rewrite_tree_without_tars(current) if has_tar_payload else old_tree
        memo[current] = commit_tree(new_tree, new_parents, metadata)

    return memo[commit]


def main() -> None:
    parser = argparse.ArgumentParser(description="Rewrite selected release tags so their reachable history no longer contains R tar payloads.")
    parser.add_argument("--pattern", default="v*", help="Tag glob to rewrite")
    parser.add_argument("--apply", action="store_true", help="Update tag refs in-place")
    parser.add_argument(
        "--backup-prefix",
        default=BACKUP_PREFIX,
        help="Namespace for backup refs that preserve the original tag targets",
    )
    args = parser.parse_args()

    tags = git("tag", "--list", args.pattern).splitlines()
    memo: dict[str, str] = {}

    for tag in tags:
        commit = git("rev-parse", f"{tag}^{{commit}}")
        rewritten = rewrite_root(commit, memo, apply=args.apply)
        if rewritten == commit:
            print(f"skip {tag} {commit} (no rewrite needed)")
            continue
        if not args.apply:
            print(f"would rewrite {tag} {commit}")
            continue
        backup_ref = f"{args.backup_prefix}{tag}"
        git("update-ref", backup_ref, commit)
        git("update-ref", f"refs/tags/{tag}", rewritten)
        print(f"rewrite {tag} {commit} -> {rewritten}")


if __name__ == "__main__":
    main()
