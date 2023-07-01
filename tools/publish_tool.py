import argparse
import configparser
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
    # We can't do a dry run of everything, because dependencies won't be available for later crates,
    # but we can at least do any leaf nodes (i.e. opendp_tooling).
    dry_run_arg = " --dry-run" if args.dry_run else ""
    run_command("Publishing opendp_tooling crate", f"cargo publish{dry_run_arg} --verbose --manifest-path=rust/opendp_tooling/Cargo.toml", capture_output=False)
    if not args.dry_run:
        run_command("Letting crates.io index settle", f"sleep {args.settle_time}")
        run_command("Publishing opendp_derive crate", f"cargo publish --verbose --manifest-path=rust/opendp_derive/Cargo.toml", capture_output=False)
        run_command("Letting crates.io index settle", f"sleep {args.settle_time}")
        run_command("Publishing opendp crate", f"cargo publish --verbose --manifest-path=rust/Cargo.toml", capture_output=False)


def python(args):
    log(f"*** PUBLISHING PYTHON LIBRARY ***")
    # https://pypi.org/help/#apitoken
    os.environ["TWINE_USERNAME"] = "__token__"
    os.environ["TWINE_PASSWORD"] = os.environ["PYPI_API_TOKEN"]
    config = configparser.RawConfigParser()
    config.read("python/setup.cfg")
    version = config["metadata"]["version"]
    wheel = f"opendp-{version}-py3-none-any.whl"
    dry_run_arg = " --repository testpypi" if args.dry_run else ""
    run_command("Publishing opendp package", f"python3 -m twine upload{dry_run_arg} --verbose python/wheelhouse/{wheel}", capture_output=False)


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
    subparser.add_argument("-n", "--dry-run", dest="dry_run", action="store_true", default=False)
    subparser.add_argument("-nn", "--no-dry-run", dest="dry_run", action="store_false")
    subparser.add_argument("-s", "--settle-time", default=60)

    subparser = subparsers.add_parser("python", help="Publish Python library")
    subparser.set_defaults(func=python)
    subparser.add_argument("-n", "--dry-run", dest="dry_run", action="store_true", default=False)
    subparser.add_argument("-nn", "--no-dry-run", dest="dry_run", action="store_false")

    subparser = subparsers.add_parser("all", help="Publish everything")
    subparser.set_defaults(func=meta, command="all")

    args = parser.parse_args(argv[1:])
    args.func(args)


def main():
    _main(sys.argv)


if __name__ == "__main__":
    main()
