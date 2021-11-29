import argparse
import os
import subprocess
import sys


def log(message, command=False):
    prefix = "$" if command else "#"
    print(f"{prefix} {message}", file=sys.stderr)


def run_command(description, args, capture_output=True, shell=True):
    if description:
        log(description)
    printed_args = args.join(" ") if type(args) == list else args
    log(printed_args, command=True)
    stdout = subprocess.PIPE if capture_output else None
    completed_process = subprocess.run(args, stdout=stdout, shell=shell, check=True, encoding="utf-8")
    return completed_process.stdout.rstrip() if capture_output else None


def rust(args):
    log(f"*** PUBLISHING RUST LIBRARY ***")
    os.environ["CARGO_REGISTRY_TOKEN"] = os.environ["CRATES_IO_API_TOKEN"]
    run_command("Logging into crates.io", f"cargo login")
    run_command("Publishing opendp crate", "cargo publish --verbose --manifest-path=opendp/Cargo.toml")
    run_command("Letting crates.io index settle", f"sleep {args.settle_time}")
    run_command("Publishing opendp-ffi crate", "cargo publish --verbose --manifest-path=opendp-ffi/Cargo.toml")


def python(args):
    log(f"*** PUBLISHING PYTHON LIBRARY ***")
    # https://pypi.org/help/#apitoken
    os.environ["TWINE_USERNAME"] = "__token__"
    os.environ["TWINE_PASSWORD"] = os.environ["PYPI_API_TOKEN"]
    run_command("Publishing opendp package", "python3 -m twine upload --verbose --skip-existing python/wheelhouse/*")


def meta(args):
    meta_args = [
        f"rust -r {args.rust_token}",
        f"python -p {args.python_token}",
    ]
    for args in meta_args:
        _main(f"meta {args}".split())


def _main(argv):
    parser = argparse.ArgumentParser(description="OpenDP build tool")
    subparsers = parser.add_subparsers(dest="COMMAND", help="Command to run")
    subparsers.required = True

    subparser = subparsers.add_parser("rust", help="Publish Rust library")
    subparser.set_defaults(func=rust)
    subparser.add_argument("-t", "--token", required=True)
    subparser.add_argument("-s", "--settle-time", default=60)

    subparser = subparsers.add_parser("python", help="Publish Python library")
    subparser.set_defaults(func=python)

    subparser = subparsers.add_parser("all", help="Publish everything")
    subparser.set_defaults(func=meta, command="all")

    args = parser.parse_args(argv[1:])
    args.func(args)


def main():
    _main(sys.argv)


if __name__ == "__main__":
    main()
