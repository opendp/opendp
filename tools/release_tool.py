import argparse
import datetime
import io
import re
import subprocess
import sys
import time

import semver
import tomlkit


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


def run_command_with_retries(description, args, retries=10, wait_time_seconds=30):
    while retries > 0:
        try:
            return run_command(description, args)
        except Exception as e:
            was_final_attempt = retries == 1
            if was_final_attempt:
                raise e
            log(f"Waiting {wait_time_seconds} Seconds | Retries Left: {retries - 1}")
            time.sleep(wait_time_seconds)
        retries -= 1


def get_version(version_str=None):
    if not version_str:
        with open("VERSION") as f:
            version_str = f.read().strip()
    return semver.Version.parse(version_str)


def update_file(path, load, munge, dump, binary=False):
    log(f"Updating {path}")
    b = "b" if binary else ""
    with open(path, f"r{b}") as f:
        data = load(f)
    new_data = munge(data)
    with open(path, f"w{b}") as f:
        dump(new_data, f)


def sync_train(args):
    log(f"*** SYNCING RELEASE TRAIN FROM UPSTREAM ***")
    train_to_upstream = {"nightly": "main", "beta": "nightly", "stable": "beta"}
    if args.train not in train_to_upstream:
        raise Exception(f"Unknown train {args.train}")
    upstream = train_to_upstream[args.train] if args.upstream is None else args.upstream
    log(f"Syncing {args.train} <= {upstream}")
    if args.train == "nightly":
        # For nightly, we don't care about history, so we just reset the branch.
        run_command(f"Resetting train to upstream", f"git switch -C {args.train} {upstream}")
    else:
        # For beta & stable, we want to preserve history, so we merge.
        # git doesn't have a "theirs" merge strategy, so we have to simulate it.
        # Technique from https://stackoverflow.com/a/4912267
        run_command(f"Creating temporary branch based on upstream", f"git switch -c tmp {upstream}")
        run_command(f"Merging train (keeping all upstream)", f"git merge -s ours {args.train}")
        run_command(f"Switching to train", f"git switch {args.train}")
        run_command(f"Merging temporary branch", f"git merge tmp")
        run_command(f"Deleting temporary branch", f"git branch -D tmp")


def update_version(version):
    log(f"Updating version references to {version}")

    # Main version file
    with open("VERSION", "w") as f:
        print(version, file=f)

    # cargo versions
    # cargo doesn't like build metadata in dependency references, so we strip that for those.
    stripped_version = version.replace(build=None)
    def munge_cargo_root(toml):
        toml["workspace"]["package"]["version"] = str(version)
        toml["dependencies"]["opendp_derive"]["version"] = str(stripped_version)
        toml["build-dependencies"]["opendp_tooling"]["version"] = str(stripped_version)
        return toml
    update_file("rust/Cargo.toml", tomlkit.load, munge_cargo_root, tomlkit.dump)
    def munge_cargo_opendp_derive(toml):
        toml["dependencies"]["opendp_tooling"]["version"] = str(stripped_version)
        return toml
    update_file("rust/opendp_derive/Cargo.toml", tomlkit.load, munge_cargo_opendp_derive, tomlkit.dump)

    # Python version
    # Python doesn't allow arbitrary prerelease tags, supporting only (a|b|rc) or synonyms (alpha|beta|c|pre|preview),
    # so we map nightly -> alpha.
    is_nightly = version.prerelease is not None and version.prerelease.startswith("nightly.")
    python_version = version.replace(prerelease=f"alpha.{version.prerelease[8:]}") if is_nightly else version
    def munge_setup(lines):
        version_line = f"version = {python_version}\n"
        return [version_line if line.startswith("version = ") else line for line in lines]
    update_file("python/setup.cfg", io.IOBase.readlines, munge_setup, lambda data, f: f.writelines(data))


def configure_train(args):
    log(f"*** CONFIGURING RELEASE TRAIN ***")
    if args.train not in ("dev", "nightly", "beta", "stable"):
        raise Exception(f"Unknown train {args.train}")
    version = get_version()
    if args.train == "dev":
        version = version.replace(prerelease="dev", build=None)
    elif args.train in ["nightly", "beta"]:
        date = datetime.date.today().strftime("%Y%m%d")
        prerelease = f"{args.train}.{date}001"
        version = version.replace(prerelease=prerelease, build=None)
    elif args.train == "stable":
        version = version.replace(prerelease=None, build=None)
    update_version(version)


def first_match(lines, pattern):
    matcher = re.compile(pattern)
    for i, line in enumerate(lines):
        match = matcher.match(line)
        if match is not None:
            return i, match
    raise Exception


def changelog(args):
    log(f"*** UPDATING CHANGELOG ***")
    if args.from_stable:
        # Pull the CHANGELOG from stable, then insert a new Upcoming Release section at top.
        changelog = run_command("Getting CHANGELOG from stable", f"git show origin/stable:CHANGELOG.md", capture_output=True)
        lines = io.StringIO(changelog).readlines()
        # This tells us where to insert.
        i, _match = first_match(lines, r"^## \[(\d+\.\d+\.\d+)].*$")
        log(f"Inserting new Upcoming Release section")
        lines[i:i] = [f"## [Upcoming Release](https://github.com/opendp/opendp/compare/stable...HEAD) - TBD\n", "\n", "\n"]
    else:
        version = get_version()
        # Load the CHANGELOG from local, then replace the UNRELEASED heading with the current version.
        log(f"Reading CHANGELOG")
        with open("CHANGELOG.md") as f:
            lines = f.readlines()
        # This tells us the previous released version.
        _i, match = first_match(lines, r"^## \[(\d+\.\d+\.\d+)].*$")
        previous_version = match.group(1)
        # This tells us where to replace.
        i, _match = first_match(lines, r"^## \[Upcoming Release\].*$")
        date = datetime.date.today().isoformat()
        log(f"Updating Upcoming Release heading to {version}")
        lines[i] = f"## [{version}](https://github.com/opendp/opendp/compare/v{previous_version}...v{version}) - {date}\n"

    with open("CHANGELOG.md", "w") as f:
        f.writelines(lines)


def format_python_version(version):
    # Python doesn't like versions of the form "X.Y.Z-rc.N" (even though they're correct), and collapses them
    # to "X.Y.ZrcN", but semver can't handle those, so we map to strings.
    if version.prerelease:
        version = f"{version.major}.{version.minor}.{version.patch}rc{version.prerelease.split('.')[1]}"
    else:
        version = str(version)
    return version


def sanity(args):
    log(f"*** RUNNING SANITY TEST ***")
    version = get_version()
    version = format_python_version(version)
    run_command("Creating venv", f"rm -rf {args.venv} && python -m venv {args.venv}")
    package = f"opendp=={version}" if args.published else f"python/wheelhouse/opendp-{version}-py3-none-any.whl"
    run_command_with_retries(f"Installing opendp {version}", f"source {args.venv}/bin/activate && pip install {package}")
    run_command("Running test script", f"source {args.venv}/bin/activate && python tools/test.py")


def bump_version(args):
    log(f"*** BUMPING VERSION NUMBER ***")
    if args.set:
        version = get_version(args.set)
    else:
        if args.position not in ("major", "minor", "patch"):
            raise Exception(f"Unknown position {args.position}")
        version = get_version()
        if args.position == "major":
            version = version.bump_major()
        elif args.position == "minor":
            version = version.bump_minor()
        elif args.position == "patch":
            version = version.bump_patch()
        version = version.replace(prerelease="dev", build=None)
    update_version(version)


def _main(argv):
    parser = argparse.ArgumentParser(description="OpenDP release tool")
    subparsers = parser.add_subparsers(dest="COMMAND", help="Command to run")
    subparsers.required = True

    subparser = subparsers.add_parser("sync", help="Sync the release train")
    subparser.set_defaults(func=sync_train)
    subparser.add_argument("-t", "--train", choices=["nightly", "beta", "stable"], default="nightly", help="Which train to target")
    subparser.add_argument("-u", "--upstream", help="Upstream ref")

    subparser = subparsers.add_parser("configure", help="Configure the release train")
    subparser.set_defaults(func=configure_train)
    subparser.add_argument("-t", "--train", choices=["dev", "nightly", "beta", "stable"], default="dev", help="Which train to target")

    subparser = subparsers.add_parser("changelog", help="Update CHANGELOG file")
    subparser.set_defaults(func=changelog)
    subparser.add_argument("-s", "--from-stable", dest="from_stable", action="store_true", default=False)
    subparser.add_argument("-ns", "--no-from-stable", dest="from_stable", action="store_false")

    subparser = subparsers.add_parser("sanity", help="Run sanity test")
    subparser.set_defaults(func=sanity)
    subparser.add_argument("-e", "--venv", default="sanity-venv", help="Virtual environment directory")
    subparser.add_argument("-p", "--published", dest="published", action="store_true", default=False)
    subparser.add_argument("-np", "--no-published", dest="published", action="store_false")

    subparser = subparsers.add_parser("bump_version", help="Bump the version number (assumes dev train)")
    subparser.set_defaults(func=bump_version)
    subparser.add_argument("-p", "--position", choices=["major", "minor", "patch"], default="patch")
    subparser.add_argument("-s", "--set", help="Set the version to a specific value")

    args = parser.parse_args(argv[1:])
    args.func(args)


def main():
    _main(sys.argv)


if __name__ == "__main__":
    main()
