import argparse
import configparser
import datetime
import os
import subprocess
import sys

import semver


def log(message, command=False):
    prefix = "$" if command else "#"
    print(f"{prefix} {message}", file=sys.stderr)


def run_command(description, cmd, capture_output=False, shell=True):
    if description:
        log(description)
    printed_cmd = " ".join(cmd) if type(cmd) == list else cmd
    log(printed_cmd, command=True)
    stdout = subprocess.PIPE if capture_output else None
    completed_process = subprocess.run(cmd, stdout=stdout, shell=shell, check=True, encoding="utf-8")
    return completed_process.stdout.rstrip() if capture_output else None


def get_version(version_str=None):
    if not version_str:
        with open("VERSION") as f:
            version_str = f.read().strip()
    return semver.Version.parse(version_str)


def infer_channel(version):
    if version.prerelease is None:
        return "stable"
    channel = version.prerelease.split(".", 1)[0]
    if channel not in ("dev", "nightly", "beta"):
        raise Exception(f"Unable to infer channel from version {version}")
    return channel


def get_current_branch():
    return run_command(f"Determining current branch", "git branch --show-current", capture_output=True)


def rust(args):
    log(f"*** PUBLISHING RUST CRATE ***")
    os.environ["CARGO_REGISTRY_TOKEN"] = os.environ["CRATES_IO_API_TOKEN"]
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
    # So downstream things like sanity test will have to retry.


def github(args):
    log(f"*** PUBLISHING GITHUB RELEASE ***")
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
    subparser.add_argument("-n", "--dry-run", dest="dry_run", action="store_true", default=False)
    subparser.add_argument("-nn", "--no-dry-run", dest="dry_run", action="store_false")
    subparser.add_argument("-t", "--index-check-timeout", type=int, default=300, help="How long to keep checking for index update (0 = don't check)")
    subparser.add_argument("-b", "--index-check-backoff", type=float, default=2.0, help="How much to back off between checks for index update")

    subparser = subparsers.add_parser("python", help="Publish Python package")
    subparser.set_defaults(func=python)
    subparser.add_argument("-r", "--repository", choices=["pypi", "testpypi"], default="pypi", help="Package repository")
    subparser.add_argument("-t", "--index-check-timeout", type=int, default=300, help="How long to keep checking for index update (0 = don't check)")
    subparser.add_argument("-b", "--index-check-backoff", type=float, default=2.0, help="How much to back off between checks for index update")

    subparser = subparsers.add_parser("github", help="Publish GitHub release")
    subparser.set_defaults(func=github)
    subparser.add_argument("-d", "--date", type=datetime.date.fromisoformat, help="Release date")
    subparser.add_argument("-n", "--draft", dest="draft", action="store_true", default=False)
    subparser.add_argument("-nn", "--no-draft", dest="draft", action="store_false")

    args = parser.parse_args(argv[1:])
    args.func(args)


def main():
    _main(sys.argv)


if __name__ == "__main__":
    main()
