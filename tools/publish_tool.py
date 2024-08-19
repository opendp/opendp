import argparse
import datetime
import os

from utils import get_current_branch, get_python_version, get_version, infer_channel, log, run_command, run_command_with_retries, sys


def rust(args):
    log("*** PUBLISHING RUST CRATE ***")
    os.environ["CARGO_REGISTRY_TOKEN"] = os.environ["CRATES_IO_API_TOKEN"]
    # We can't do a dry run of everything, because dependencies won't be available for later crates,
    # but we can at least do any leaf nodes (i.e. opendp_tooling).
    dry_run_arg = " --dry-run" if args.dry_run else "" # Keep dash in this arg.
    run_command("Publishing opendp_tooling crate", f"cargo publish{dry_run_arg} --verbose --manifest-path=rust/opendp_tooling/Cargo.toml")
    if not args.dry_run:
        # As of https://github.com/rust-lang/cargo/pull/11062, cargo publish blocks until the index is propagated,
        # so we don't have to wait here anymore.
        run_command("Publishing opendp_derive crate", "cargo publish --verbose --manifest-path=rust/opendp_derive/Cargo.toml")
        run_command("Publishing opendp crate", "cargo publish --verbose --manifest-path=rust/Cargo.toml")


def python(args):
    log("*** PUBLISHING PYTHON PACKAGE ***")
    # https://pypi.org/help/#apitoken
    os.environ["TWINE_USERNAME"] = "__token__"
    os.environ["TWINE_PASSWORD"] = os.environ["PYPI_API_TOKEN"]
    
    run_command("Publishing opendp package", f"python -m twine upload -r {args.repository} --verbose python/dist/*")
    # Unfortunately, twine doesn't have an option to block until the index is propagated. Polling the index is unreliable,
    # because often the new item will appear, but installs will still fail (probably because of stale caches).
    # So downstream things like sanity test will have to retry.


def sanity(args):
    log("*** RUNNING SANITY TEST ***")
    if args.python_repository not in ("pypi", "testpypi", "local"):
        raise Exception(f"Unknown Python repository {args.python_repository}")
    version = get_version()
    version = get_python_version(version)
    run_command("Creating venv", f"rm -rf {args.venv} && python -m venv {args.venv}")
    if args.python_repository == "local":
        package_name = f"opendp-{version}-cp39-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl"
        package = f"python/dist/{package_name}"
        run_command(f"Installing opendp {version}", f". {args.venv}/bin/activate && pip install {package}")
    else:
        index_url = "https://test.pypi.org/simple" if args.python_repository == "testpypi" else "https://pypi.org/simple"
        package = f"opendp=={version}"
        run_command_with_retries(
            f"Installing opendp {version}", f". {args.venv}/bin/activate && pip install -i {index_url} {package}",
            args.package_timeout,
            args.package_backoff
        )
    if args.fake:
        run_command("Running test script", f". {args.venv}/bin/activate && echo FAKE TEST!!!")
    else:
        run_command("Running test script", f". {args.venv}/bin/activate && python tools/test.py")


def github(args):
    log("*** PUBLISHING GITHUB RELEASE ***")
    version = get_version()
    channel = infer_channel(version)
    branch = get_current_branch()
    if branch != channel:
        raise Exception(f"Version {version} implies channel {channel}, but current branch is {branch}")
    tag = f"v{version}"
    # Just in case, clear out any existing tag, so a new one will be created by GitHub.
    run_command("Clearing tag", f"git push origin :refs/tags/{tag}")
    title = f"OpenDP {version}"
    stripped_version = str(version).replace(".", "")
    date = args.date or datetime.date.today()
    notes = f"[CHANGELOG](https://github.com/opendp/opendp/blob/main/CHANGELOG.md#{stripped_version}---{date})"
    cmd = f"gh release create {tag} --target {channel} -t '{title}' -n '{notes}'"
    if version.prerelease:
        cmd += " -p"
    if args.draft:
        cmd += " -d"
    run_command("Creating GitHub release", cmd)


def _main(argv):
    parser = argparse.ArgumentParser(description="OpenDP build tool")
    subparsers = parser.add_subparsers(dest="COMMAND", help="Command to run")
    subparsers.required = True

    subparser = subparsers.add_parser("rust", help="Publish Rust crate")
    subparser.set_defaults(func=rust)
    subparser.add_argument("--dry_run", action="store_true")
    subparser.add_argument("-t", "--index-check-timeout", type=int, default=300, help="How long to keep checking for index update (0 = don't check)")
    subparser.add_argument("-b", "--index-check-backoff", type=float, default=2.0, help="How much to back off between checks for index update")

    subparser = subparsers.add_parser("python", help="Publish Python package")
    subparser.set_defaults(func=python)
    subparser.add_argument("-r", "--repository", choices=["pypi", "testpypi"], default="pypi", help="Package repository")
    subparser.add_argument("-t", "--index-check-timeout", type=int, default=300, help="How long to keep checking for index update (0 = don't check)")
    subparser.add_argument("-b", "--index-check-backoff", type=float, default=2.0, help="How much to back off between checks for index update")

    subparser = subparsers.add_parser("sanity", help="Run a sanity test")
    subparser.set_defaults(func=sanity)
    subparser.add_argument("-e", "--venv", default="/tmp/sanity-venv", help="Virtual environment directory")
    subparser.add_argument("-r", "--python-repository", choices=["pypi", "testpypi", "local"], default="pypi", help="Python package repository")
    subparser.add_argument("-t", "--package-timeout", type=int, default=0, help="How long to retry package installation attempts (0 = no retries)")
    subparser.add_argument("-b", "--package-backoff", type=float, default=2.0, help="How much to back off between package installation attempts")
    subparser.add_argument("--fake", action="store_true")

    subparser = subparsers.add_parser("github", help="Publish GitHub release")
    subparser.set_defaults(func=github)
    subparser.add_argument("-d", "--date", type=datetime.date.fromisoformat, help="Release date")
    subparser.add_argument("--draft", action="store_true")

    args = parser.parse_args(argv[1:])
    args.func(args)


def main():
    _main(sys.argv)


if __name__ == "__main__":
    main()
