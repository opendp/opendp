from pathlib import Path
from subprocess import check_output
import re
import fileinput


def get_commits(path=None):
    cmd = ["git", "log", "--oneline"]
    if path is not None:
        cmd.append(path.absolute())
    output = check_output(cmd, text=True)
    git_log = output.splitlines()
    return [line.split(" ")[0] for line in git_log]


ignores = {
    "a695b8c99",  # Chuck's whitespace change
    "5d05f55ee",  # Mike's template change
}


def process(line: str, replacement: str):
    updated = re.sub(
        r"\\asOfCommit\{([^}]+)\}\{([^}]+)\}",
        rf"\\asOfCommit{{\1}}{{{replacement}}}",
        line,
    )
    print(updated, end="")


def main():
    known_commits = get_commits()

    root = Path(__file__).parent.parent.parent
    for path in root.glob("**/*.tex"):
        text = path.read_text()
        as_of_commits = re.findall(r"\\asOfCommit\{[^}]+\}\{([^}]+)\}", text)
        assert len(as_of_commits) <= 1
        if as_of_commits:
            if (commit := as_of_commits[0]) not in known_commits:
                print(f"{path.name}: {commit} not in git log")
                path_commits = get_commits(path)
                for path_commit in path_commits:
                    if path_commit in ignores:
                        continue
                    print(f"\tupdate to {path_commit}")
                    with fileinput.input(
                        files=path, encoding="utf-8", inplace=True
                    ) as f:
                        for line in f:
                            process(line, path_commit)
                    break


if __name__ == "__main__":
    main()
