import argparse
import subprocess


def log_until(match):
    lines = subprocess.check_output(['git', 'log', '--oneline'], text=True).splitlines()
    if match is None:
        return '\n'.join(lines)
    head = []
    for line in lines:
        if match in line:
            break
        head.append(line)
    return '\n'.join(head)


def parse_log(log):
    return log


def main():
    parser = argparse.ArgumentParser(description="Helps generate CHANGELOG entries")
    parser.add_argument("--until", help="Read from git log until match found", required=False)
    args = parser.parse_args()
    log = log_until(args.until)
    print(parse_log(log))


if __name__ == "__main__":
    main()