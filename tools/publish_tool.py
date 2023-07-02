import argparse
import configparser
import os
import re
import subprocess
import sys
import time


def log(message, command=False):
    prefix = "$" if command else "#"
    print(f"{prefix} {message}", file=sys.stderr)


def run_command(description, args, capture_output=False, shell=True):
    if description:
        log(description)
    printed_args = args.join(" ") if type(args) == list else args
    log(printed_args, command=True)
    stdout = subprocess.PIPE if capture_output else None
    completed_process = subprocess.run(args, stdout=stdout, shell=shell, check=True, encoding="utf-8")
    return completed_process.stdout.rstrip() if capture_output else None


def rust(args):
    log(f"*** PUBLISHING RUST CRATE ***")
    os.environ["CARGO_REGISTRY_TOKEN"] = os.environ["CRATES_IO_API_TOKEN"]
    with open("VERSION") as f:
        version = f.read().strip()
    pattern = fr'"vers":\s*"{version}"'

    # We can't do a dry run of everything, because dependencies won't be available for later crates,
    # but we can at least do any leaf nodes (i.e. opendp_tooling).
    dry_run_arg = " --dry-run" if args.dry_run else ""
    run_command("Publishing opendp_tooling crate", f"cargo publish{dry_run_arg} --verbose --manifest-path=rust/opendp_tooling/Cargo.toml")
    if not args.dry_run:
        # As of https://github.com/rust-lang/cargo/pull/11062, cargo publish blocks until the index is propagated,
        # so we don't have to wait here anymore.
        run_command("Publishing opendp_derive crate", f"cargo publish --verbose --manifest-path=rust/opendp_derive/Cargo.toml")
        run_command("Publishing opendp crate", f"cargo publish --verbose --manifest-path=rust/Cargo.toml")


def python(args):
    log(f"*** PUBLISHING PYTHON PACKAGE ***")
    # https://pypi.org/help/#apitoken
    os.environ["TWINE_USERNAME"] = "__token__"
    os.environ["TWINE_PASSWORD"] = os.environ["PYPI_API_TOKEN"]
    config = configparser.RawConfigParser()
    config.read("python/setup.cfg")
    version = config["metadata"]["version"]
    wheel = f"opendp-{version}-py3-none-any.whl"
    run_command("Publishing opendp package", f"python -m twine upload -r {args.repository} --verbose python/wheelhouse/{wheel}")
    # Unfortunately, twine doesn't have an option to block until the index is propagated. Polling the index is unreliable,
    # because often the new item will appear, but installs will still fail (probably because of stale caches).
    # So downstream things like in release_tool.py sanity will have to retry.


def _main(argv):
    parser = argparse.ArgumentParser(description="OpenDP build tool")
    subparsers = parser.add_subparsers(dest="COMMAND", help="Command to run")
    subparsers.required = True

    subparser = subparsers.add_parser("rust", help="Publish Rust crate")
    subparser.set_defaults(func=rust)
    subparser.add_argument("-n", "--dry-run", dest="dry_run", action="store_true", default=False)
    subparser.add_argument("-nn", "--no-dry-run", dest="dry_run", action="store_false")
    subparser.add_argument("-t", "--index-check-timeout", type=int, default=300, help="How long to keep checking for index update (0 = don't check)")
    subparser.add_argument("-b", "--index-check-backoff", type=float, default=2.0, help="How much to back off between checks for index update")

    subparser = subparsers.add_parser("python", help="Publish Python package")
    subparser.set_defaults(func=python)
    subparser.add_argument("-r", "--repository", choices=["pypi", "testpypi"], default="pypi", help="Package repository")
    subparser.add_argument("-t", "--index-check-timeout", type=int, default=300, help="How long to keep checking for index update (0 = don't check)")
    subparser.add_argument("-b", "--index-check-backoff", type=float, default=2.0, help="How much to back off between checks for index update")

    args = parser.parse_args(argv[1:])
    args.func(args)


def main():
    _main(sys.argv)


if __name__ == "__main__":
    main()
