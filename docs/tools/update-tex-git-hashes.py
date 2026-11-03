from pathlib import Path
from subprocess import check_output
import re
import fileinput


def get_all_repo_commits():
    cmd = ["git", "log", "--oneline"]
    output = check_output(cmd, text=True)
    git_log = output.splitlines()
    return [line.split(" ")[0] for line in git_log]


def get_earliest_occurence(path: Path, commit: str):
    cmd = ["git", "log", "--oneline", f"-S'{commit}'", path]
    output = check_output(cmd, text=True)
    git_log = output.splitlines()
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

    root = Path(__file__).parent.parent.parent
    for path in root.glob("**/*.tex"):
        text = path.read_text()
        as_of_commits = re.findall(r"\\asOfCommit\{[^}]+\}\{([^}]+)\}", text)
        assert len(as_of_commits) == 1
        as_of_commit = as_of_commits[0]
        if as_of_commit in repo_commits:
            print(f"{path.name}: {commit} is in git log")
            continue
        print(f"{path.name}: {commit} not in git log")
        earliest_commit = get_earliest_occurence(path, commit)
        replace_with_earliest_commit(path, earliest_commit)


if __name__ == "__main__":
    main()
