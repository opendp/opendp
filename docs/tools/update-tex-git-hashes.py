from pathlib import Path
from subprocess import check_output
import re
import fileinput
import sys


def get_all_repo_commits():
    cmd = ["git", "log", "--oneline"]
    output = check_output(cmd, text=True)
    git_log = output.splitlines()
    print(f"{len(git_log)} lines of output from: {' '.join(cmd)}")
    return [line.split(" ")[0] for line in git_log]


def get_earliest_occurence(path: Path, commit: str):
    cmd = ["git", "log", "--oneline", "-S", commit, str(path)]
    output = check_output(cmd, text=True)
    git_log = output.splitlines()
    print(f"{len(git_log)} lines of output from: {' '.join(cmd)}")
    if not git_log:
        return None
    return git_log[-1].split(" ")[0]


def process(line: str, replacement: str):
    updated = re.sub(
        r"\\asOfCommit\{([^}]+)\}\{([^}]+)\}",
        rf"\\asOfCommit{{\1}}{{{replacement}}}",
        line,
    )
    print(updated, end="")


def replace_with_earliest_commit(path: Path, earliest_commit: str):
    with fileinput.input(files=path, encoding="utf-8", inplace=True) as f:
        for line in f:
            process(line, earliest_commit)


def main():
    repo_commits = get_all_repo_commits()
    failures = []

    root = Path(__file__).parent.parent.parent
    for path in root.glob("**/*.tex"):
        print(f"\n{path}...")
        text = path.read_text()
        as_of_commits = re.findall(r"\\asOfCommit\{[^}]+\}\{([^}]+)\}", text)
        assert len(as_of_commits) <= 1
        if not as_of_commits:
            print(f"{path.name}: no asOfCommit")
            continue
        as_of_commit = as_of_commits[0]
        if any(repo_commit.startswith(as_of_commit) for repo_commit in repo_commits):
            print(f"{path.name}: {as_of_commit} IS in git log: No rewrite needed!")
            continue
        print(f"{path.name}: {as_of_commit} NOT in git log: Will try to rewrite...")
        earliest_commit = get_earliest_occurence(path, as_of_commit)
        if earliest_commit is None:
            print(f"{path.name}: Failed to find earliest occurence of {as_of_commit}")
            failures.append(str(path))
            continue
        replace_with_earliest_commit(path, earliest_commit)
        print(f"{path.name}: {as_of_commit} -> {earliest_commit} in place")

    if failures:
        print(f"failures:\n{'\n'.join(failures)}")
        sys.exit(1)


if __name__ == "__main__":
    main()
