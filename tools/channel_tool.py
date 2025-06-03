import argparse
import configupdater
import datetime
import re
import zoneinfo
from pathlib import Path

import tomlkit
from debmutate.control import ControlEditor

from utils import get_current_branch, get_python_version, get_r_version, get_version, infer_channel, log, run_command, semver, sys

CHANGELOG_PATH = (Path(__file__).parent.parent / 'CHANGELOG.md')
URL_BASE = "https://github.com/opendp/opendp/compare/"


def get_changelog_lines():
    return CHANGELOG_PATH.read_text().splitlines()


def ensure_branch(branch):
    # GH checkout action doesn't fetch all branches unless you force it, in which case main seems to be omitted.
    # So we fetch the branch from origin, but only if we're not already on it (which would cause the fetch to fail).
    if get_current_branch() != branch:
        run_command(f"Fetching branch {branch}", f"git fetch origin {branch}:{branch}")


def initialize(args):
    log("*** INITIALIZING CHANNEL FROM UPSTREAM ***")
    if args.channel not in ("dev", "nightly", "beta", "stable"):
        raise Exception(f"Unknown channel {args.channel}")
    if args.sync:
        channel_to_upstream = {"dev": get_current_branch(), "nightly": "main", "beta": "nightly", "stable": "beta"}
        upstream = channel_to_upstream[args.channel] if args.upstream is None else args.upstream
        ensure_branch(upstream)
        log(f"Syncing {args.channel} <= {upstream}")
        if args.preserve:
            # We're preserving channel history, so we need to do a merge.
            # git doesn't have a "theirs" merge strategy, so we have to simulate it.
            # Technique from https://stackoverflow.com/a/4912267
            run_command("Creating temporary branch based on upstream", f"git switch -c tmp {upstream}")
            run_command("Merging channel (keeping all upstream)", f"git merge -s ours {args.channel}")
            run_command("Switching to channel", f"git switch {args.channel}")
            run_command("Merging temporary branch", "git merge tmp")
            run_command("Deleting temporary branch", "git branch -D tmp")
        else:
            # We're not preserving channel history, so we can just reset the branch.
            run_command("Resetting channel to upstream", f"git switch -C {args.channel} {upstream}")
    else:
        ensure_branch(args.channel)
        run_command("Switching to channel", f"git switch {args.channel}")


def date(args):
    log("*** GENERATING RELEASE DATE ***")
    if args.time_zone is not None:
        tz = zoneinfo.ZoneInfo(args.time_zone)
        date = datetime.datetime.now(tz).date()
    else:
        date = datetime.date.today()
    print(date.isoformat())


def infer_counter(version, date, args):
    if args.counter:
        return args.counter
    if version.prerelease is None:
        return 1
    match = re.match(fr"^{args.channel}\.(\d+)\.(\d+)", version.prerelease)
    if match is None:
        return 1
    version_date = match.group(1)
    version_counter = match.group(2)
    if not version_date == date.strftime('%Y%m%d'):
        return 1
    return int(version_counter) + 1


def update_file(path, load, munge, dump, binary=False):
    log(f"Updating {path}")
    b = "b" if binary else ""
    with open(path, f"r{b}") as f:
        data = load(f)
    new_data = munge(data)
    with open(path, f"w{b}") as f:
        dump(new_data, f)


def update_version(version):
    log(f"Updating version references to {version}")
    python_version = get_python_version(version)
    r_version = get_r_version(version)

    # Main version file
    with open("VERSION", "w") as f:
        print(version, file=f)

    # cargo versions
    # cargo doesn't like build metadata in dependency references, so we strip that for those.
    stripped_version = version.replace(build=None)
    def munge_cargo_root(toml):
        toml["workspace"]["package"]["version"] = str(version)
        toml["dependencies"]["opendp_derive"]["version"] = str(stripped_version)
        toml["dependencies"]["opendp_tooling"]["version"] = str(stripped_version)
        toml["build-dependencies"]["opendp_tooling"]["version"] = str(stripped_version)
        return toml
    update_file("rust/Cargo.toml", tomlkit.load, munge_cargo_root, tomlkit.dump)
    def munge_cargo_opendp_derive(toml):
        toml["dependencies"]["opendp_tooling"]["version"] = str(stripped_version)
        return toml
    update_file("rust/opendp_derive/Cargo.toml", tomlkit.load, munge_cargo_opendp_derive, tomlkit.dump)

    # Python config
    def load_python_config(f):
        config = configupdater.ConfigUpdater()
        config.read_file(f)
        return config
    def munge_python_config(config):
        config["metadata"]["version"].value = str(python_version)
        return config
    def dump_python_config(config, f):
        config.write(f)
    update_file("python/setup.cfg", load_python_config, munge_python_config, dump_python_config)
    
    # R Package
    log("Updating R/opendp/DESCRIPTION")
    with ControlEditor(path='R/opendp/DESCRIPTION') as control:
        # while it might not look like it, this mutates the DESCRIPTION file in-place
        next(iter(control.paragraphs))["Version"] = r_version


def configure(args):
    log("*** CONFIGURING CHANNEL ***")
    if args.channel not in ("dev", "nightly", "beta", "stable"):
        raise Exception(f"Unknown channel {args.channel}")
    version = get_version()

    if args.channel in ("dev", "nightly", "beta"):
        # dev/nightly/beta have a tag with the date and a counter
        date = args.date or datetime.date.today()
        counter = infer_counter(version, date, args)
        prerelease = f"{args.channel}.{date.strftime('%Y%m%d')}.{counter}"
        version = version.replace(prerelease=prerelease, build=None)
    elif args.channel == "stable":
        # stable has no tag
        version = version.finalize_version()
    update_version(version)


def match_first_changelog_header(lines):
    pattern = fr"^## \[(\d+\.\d+\.\d+(?:-\S+)?)\]\({re.escape(URL_BASE)}(\S+)\.\.\.\S+\) - \S+$"
    matcher = re.compile(pattern)
    for i, line in enumerate(lines):
        match = matcher.match(line)
        if match is not None:
            return i, match
    raise Exception("Didn't match pattern in CHANGELOG")


def changelog(args):
    log("*** UPDATING CHANGELOG ***")
    version = get_version()
    channel = infer_channel(version)
    date = args.date or datetime.date.today()

    log("Reading CHANGELOG")
    changelog_lines = get_changelog_lines()
    i, match = match_first_changelog_header(changelog_lines)
    heading_version = semver.Version.parse(match.group(1))
    diff_source = match.group(2)

    if args.prepend:
        if channel != "dev":
            raise Exception("Can only prepend on dev channel")
        # Check that the VERSION file has been bumped above the existing heading version.
        if version.finalize_version() <= heading_version.finalize_version():
            raise Exception(f"Prepending new heading, but VERSION {version} hasn't been bumped above heading version {heading_version}")
        new_heading_version = heading_version.finalize_version()
        diff_target = f"v{heading_version.finalize_version()}"
    else:
        # Check that the VERSION file matches the existing heading version.
        if version.finalize_version() != heading_version.finalize_version():
            raise Exception(f"VERSION {version} isn't compatible with heading version {heading_version}")
        new_heading_version = version
        diff_target = f"v{version}" if channel == "stable" else (channel if channel != "dev" else "HEAD")
        if channel == "dev":
            date = "TBD"

    old_line = changelog_lines[i]
    new_line = f"## [{new_heading_version}]({URL_BASE}{diff_source}...{diff_target}) - {date}\n"
    changelog_lines[i] = new_line
    log(f"{old_line=}\n# {new_line=}")

    if args.prepend:
        # Prepend a new heading for the current version.
        diff_source = diff_target
        log(f"Prepending new heading for {version}")
        changelog_lines[i:i] = [f"## [{version}]({URL_BASE}{diff_source}...HEAD) - TBD\n", "\n", "\n"]

    CHANGELOG_PATH.write_text('\n'.join(changelog_lines))


def bump_version(args):
    log("*** BUMPING VERSION NUMBER ***")
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
    parser = argparse.ArgumentParser(description="OpenDP channel tool")
    subparsers = parser.add_subparsers(dest="COMMAND", help="Command to run")
    subparsers.required = True

    subparser = subparsers.add_parser("initialize", help="Initialize the channel")
    subparser.set_defaults(func=initialize)
    subparser.add_argument("-c", "--channel", choices=["dev", "nightly", "beta", "stable"], default="nightly", help="Which channel to target")
    subparser.add_argument("--sync", action="store_true", help="Sync the channel from upstream")
    subparser.add_argument("-u", "--upstream", help="Upstream ref")
    subparser.add_argument("-p", "--preserve", dest="preserve", action="store_true")

    subparser = subparsers.add_parser("date", help="Generate release date")
    subparser.set_defaults(func=date)
    subparser.add_argument("-z", "--time-zone", help="Time zone for date resolution")

    subparser = subparsers.add_parser("configure", help="Configure the channel")
    subparser.set_defaults(func=configure)
    subparser.add_argument("-c", "--channel", choices=["dev", "nightly", "beta", "stable"], default="dev", help="Which channel to target")
    subparser.add_argument("-d", "--date", type=datetime.date.fromisoformat, help="Release date")
    subparser.add_argument("-i", "--counter", type=int, default=0, help="Intra-date version counter")

    subparser = subparsers.add_parser("changelog", help="Update the CHANGELOG file")
    subparser.set_defaults(func=changelog)
    subparser.add_argument("-d", "--date", type=datetime.date.fromisoformat, help="Release date")
    subparser.add_argument("--prepend", action="store_true", help="Prepend new empty heading (for dev only)")

    subparser = subparsers.add_parser("bump_version", help="Bump the version number (assumes dev channel)")
    subparser.set_defaults(func=bump_version)
    subparser.add_argument("-p", "--position", choices=["major", "minor", "patch"], default="minor")
    subparser.add_argument("-s", "--set", help="Set the version to a specific value")

    args = parser.parse_args(argv[1:])
    args.func(args)


def main():
    _main(sys.argv)


if __name__ == "__main__":
    main()
