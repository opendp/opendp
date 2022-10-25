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
    
    if args.dry_run:
        raise NotImplementedError("dry runs aren't supported on workspaces")

    import toml
    # write the version into the opendp crate dependencies
    opendp_toml = toml.load('rust/Cargo.toml')
    version = opendp_toml['workspace']['package']['version']
    opendp_toml['dependencies']['opendp_derive']['version'] = version
    opendp_toml['build-dependencies']['opendp_tooling']['version'] = version
    toml.dump(opendp_toml, 'rust/Cargo.toml')

    # write the version into the derive crate dependencies
    opendp_derive_toml = toml.load('rust/opendp_derive/Cargo.toml')
    opendp_derive_toml['dependencies']['opendp_tooling']['version'] = version
    toml.dump(opendp_toml, 'rust/opendp_derive/Cargo.toml')

    run_command("Publishing opendp_tooling crate", f"cargo publish --verbose --manifest-path=rust/opendp_tooling/Cargo.toml")
    run_command("Letting crates.io index settle", f"sleep {args.settle_time}")
    run_command("Publishing opendp_derive crate", f"cargo publish --verbose --manifest-path=rust/opendp_derive/Cargo.toml")
    run_command("Letting crates.io index settle", f"sleep {args.settle_time}")
    run_command("Publishing opendp crate", f"cargo publish --verbose --manifest-path=rust/Cargo.toml")


def python(args):
    log(f"*** PUBLISHING PYTHON LIBRARY ***")
    # https://pypi.org/help/#apitoken
    os.environ["TWINE_USERNAME"] = "__token__"
    os.environ["TWINE_PASSWORD"] = os.environ["PYPI_API_TOKEN"]
    dry_run_arg = " --repository testpypi" if args.dry_run else ""
    run_command("Publishing opendp package", f"python3 -m twine upload{dry_run_arg} --verbose --skip-existing python/wheelhouse/*")


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
