import argparse
import subprocess
from pathlib import Path


def convert(nb_path):
    rst_lines = subprocess.check_output(
        ['jupyter', 'nbconvert', nb_path, '--to', 'rst', '--stdout'],
        text=True
    ).splitlines()
    print(rst_lines[:10])


def main():
    parser = argparse.ArgumentParser(
        description="Wraps nbconvert, also converting code cells to tabbed doctests")
    parser.add_argument("nb_path", help="Notebook to convert", type=Path)
    args = parser.parse_args()
    convert(args.nb_path)


if __name__ == "__main__":
    main()