import argparse
import datetime
import json
import os
import platform
import subprocess
import sys
import types

import semver


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


def clone_repo(url, dir):
    run_command(f"Cloning repo {url} -> {dir}", f"git clone {url} {dir}")
    os.chdir(dir)


def fetch_repo(dir):
    os.chdir(dir)
    run_command(f"Fetching repo", f"git fetch")


def get_base_version(base_version):
    base_version = base_version or run_command("Getting base version", f"git show origin/stable:./VERSION")
    base_version = semver.VersionInfo.parse(base_version)
    return base_version


def get_target_version(base_version, release_type):
    target_version = base_version.next_version(release_type)
    return target_version


def resolve_target_version(target_version, prerelease_number):
    for _ in range(prerelease_number or 0):
        target_version = target_version.bump_prerelease()
    return target_version


def get_cached_version():
    with open("VERSION") as f:
        cached_version = f.read().strip()
    cached_version = semver.VersionInfo.parse(cached_version)
    return cached_version


def get_branch(version):
    branch = f"release/{version.major}.{version.minor}.x"
    return branch


def get_tag(version):
    tag = f"v{version}"
    return tag


def write_conf(type, base_version, target_version, branch, tag, ref):
    # To keep links in sync, we need a consistent date. If it takes a long time to get through the process,
    # there's a chance that the date the release finally gets published will be later than the date we cache here,
    # but it's not a huge deal.
    date = datetime.date.today().isoformat()
    conf = dict(type=type, base_version=str(base_version), target_version=str(target_version), branch=branch, tag=tag, ref=ref, date=date)
    log(f"conf = {conf}")
    with open(".release_conf.json", "w") as f:
        json.dump(conf, f)
    return conf


def read_conf(args):
    os.chdir(args.repo_dir)
    with open(".release_conf.json", "r") as f:
        conf = types.SimpleNamespace(**json.load(f))
    conf.base_version = semver.VersionInfo.parse(conf.base_version)
    conf.target_version = semver.VersionInfo.parse(conf.target_version)
    return conf


def create_branch(type, branch, ref, force=False):
    create_arg = "-C" if force else "-c"
    run_command(f"Creating {type} branch {branch} -> {ref}", f"git switch {create_arg} {branch} {ref}")


def switch_branch(type, branch):
    run_command(f"Switching to {type} branch {branch}", f"git switch {branch}")


def commit(what, files, message=None):
    message = message or f"Updated {what}."
    run_command(f"Committing {what}", f"git add {' '.join(files)} && git commit -m '{message}'")


def push(what, force=False):
    force_arg = " --force-with-lease" if force else ""
    run_command(f"Pushing {what}", f"git push{force_arg} origin HEAD")


def init(args):
    log(f"*** INITIALIZING RELEASE ***")
    if args.clone:
        clone_repo(args.repo_url, args.repo_dir)
    else:
        fetch_repo(args.repo_dir)
    base_version = get_base_version(args.base_version)
    target_version = get_target_version(base_version, args.type)
    branch = get_branch(target_version)
    tag = get_tag(target_version)
    args.ref = args.ref or "main"
    write_conf(args.type, base_version, target_version, branch, tag, args.ref)
    if args.type == "patch":
        switch_branch("release", branch)
    else:
        create_branch("release", branch, args.ref, force=args.force)


def cherry(args):
    log(f"*** CHERRY PICKING COMMITS ***")
    conf = read_conf(args)
    if args.commit:
        run_command("Cherry picking commits", f"git cherry-pick -x {' '.join(args.commit)}")


def changelog(args):
    log(f"*** UPDATING CHANGELOG ***")
    conf = read_conf(args)
    # There are two possibilities for changelog items:
    # * All unreleased items are going in this release (more common, when the release contains the current main branch).
    #   In this case, we expect a single heading "Unreleased", containing all unreleased items, which will be promoted
    #   to a versioned section; for future items, a new "Unreleased" section will be generated.
    # * A subset of unreleased items are going in this release (less common, when the release omits some changes).
    #   In this case, we expect a heading "Unreleased (<VERSION>)", containing the subset of unreleased items going in
    #   this release, which will be promoted to a versioned section, and another "Unreleased" section containing the
    #   remaining unreleased items, which will be left as is (for future items).
    full_heading_regex = "^## \\[Unreleased\\](.*)$"
    escaped_target_version = str(conf.target_version).replace(".", "\\.")
    subset_heading_regex = f"^## \\[Unreleased ({escaped_target_version})\\](.*)$"
    subset = run_command("Looking for subset heading", f"grep -q '{subset_heading_regex}' CHANGELOG.md && echo True || echo False") == "True"
    heading_regex = subset_heading_regex if subset else full_heading_regex
    base_tag = get_tag(conf.base_version)
    diff_url = f"https://github.com/opendp/opendp/compare/{base_tag}...{conf.tag}"
    replacement = f"## [{conf.target_version}] - {conf.date}\\n[{conf.target_version}]: {diff_url}"
    # If this isn't a subset, prepend a new unreleased section.
    if not subset:
        replacement = "## [Unreleased](https://github.com/opendp/opendp/compare/stable...HEAD)\\n\\n\\n" + replacement
    substitution_arg = f"-e 's|{heading_regex}|{replacement}|'"
    inplace_arg = "-i ''" if platform.system() == "Darwin" else "-i"
    run_command("Updating CHANGELOG", f"sed {inplace_arg} {substitution_arg} CHANGELOG.md")
    commit("CHANGELOG", ["CHANGELOG.md"], f"RELEASE_TOOL: Updated CHANGELOG.md for {conf.target_version}.")


def version(args):
    log(f"*** UPDATING VERSION ***")
    conf = read_conf(args)
    cached_version = get_cached_version()
    # Resolve the target version with the prerelease number.
    resolved_target_version = resolve_target_version(conf.target_version, args.prerelease_number)
    log(f"Updating version -> {resolved_target_version}")
    versioned_files = [
        "VERSION",
        "rust/opendp/Cargo.toml",
        "rust/opendp-ffi/Cargo.toml",
        "python/setup.cfg",
    ]
    log(f"Updating versioned files")
    inplace_arg = "-i ''" if platform.system() == "Darwin" else "-i"
    run_command(None, f"echo {resolved_target_version} >VERSION")
    run_command(None, f"sed {inplace_arg} 's/^version = \"{cached_version}\"$/version = \"{resolved_target_version}\"/' rust/opendp/Cargo.toml")
    run_command(None, f"sed {inplace_arg} 's/^version = \"{cached_version}\"$/version = \"{resolved_target_version}\"/' rust/opendp-ffi/Cargo.toml")
    run_command(None, f"sed {inplace_arg} 's/^version = {cached_version}$/version = {resolved_target_version}/' python/setup.cfg")
    commit("versioned files", versioned_files, f"RELEASE_TOOL: Set version to {resolved_target_version}.")


def python_version(version):
    # Python doesn't like versions of the form "X.Y.Z-rc.N" (even though they're correct), and collapses them
    # to "X.Y.ZrcN", but semver can't handle those, so we map to strings.
    if version.prerelease:
        version = f"{version.major}.{version.minor}.{version.patch}rc{version.prerelease.split('.')[1]}"
    else:
        version = str(version)
    return version


def sanity(venv, version, published=False):
    version = python_version(version)
    run_command("Creating venv", f"rm -rf {venv} && python -m venv {venv}")
    package = f"opendp=={version}" if published else f"python/wheelhouse/opendp-{version}-py3-none-any.whl"
    run_command(f"Installing opendp {version}", f"source {venv}/bin/activate && pip install {package}")
    run_command("Running test script", f"source {venv}/bin/activate && python python/example/test.py")


def preflight(args):
    log(f"*** RUNNING PREFLIGHT TEST ***")
    conf = read_conf(args)
    # We may be doing a prerelease, so use the version that was cached in the VERSION file.
    cached_version = get_cached_version()
    run_command(f"Building locally", "python tools/build_tool.py all")
    sanity(args.venv, cached_version, published=False)


def create(args):
    log(f"*** CREATING RELEASE ***")
    conf = read_conf(args)
    push("release", args.force)
    # We may be doing a prerelease, so use the version that was cached in the VERSION file.
    cached_version = get_cached_version()
    resolved_tag = get_tag(cached_version)
    # Just in case, clear out any existing tag, so a new one will be created by GitHub.
    run_command("Clearing tag", f"git push origin :refs/tags/{resolved_tag}")
    title = f"OpenDP {cached_version}"
    notes = f"[Changelog](https://github.com/opendp/opendp/blob/main/CHANGELOG.md#{conf.target_version}---{conf.date})"
    prerelease_arg = " -p" if cached_version.prerelease else ""
    draft_arg = " -d" if args.draft else ""
    run_command("Creating GitHub Release", f"gh release create {resolved_tag} --target {conf.branch} -t '{title}' -n '{notes}'{prerelease_arg}{draft_arg}")


def watch(args):
    log(f"*** WATCHING RELEASE ***")
    conf = read_conf(args)
    # Assumes most recent workflow is ours!
    line = run_command("Listing workflows", f"gh run list -w Release | head -n 1")
    descriptor = line.split("\t")
    if len(descriptor) != 9:
        raise Exception("Couldn't parse workflow descriptor", line)
    id = descriptor[6]
    run_command(f"Watching workflow {line.strip()}", f"gh run watch {id} --exit-status", capture_output=False)


def postflight(args):
    log(f"*** RUNNING TEST ***")
    conf = read_conf(args)
    # We may be doing a prerelease, so use the version that was cached in the VERSION file.
    cached_version = get_cached_version()
    sanity(args.venv, cached_version, published=True)


def reconcile(args):
    log(f"*** RECONCILING ***")
    conf = read_conf(args)
    reconciliation_branch = f"{conf.target_version}-reconciliation"
    reconciled_files = ["CHANGELOG.md"]
    create_branch("reconciliation", reconciliation_branch, "main", args.force)
    run_command("Copying reconciled files from release branch", f"git restore -s {conf.branch} -- {' '.join(reconciled_files)}")
    commit("reconciled files", reconciled_files, f"RELEASE_TOOL: Reconciled files from {conf.target_version}.")
    push("reconciled files", args.force)
    draft_arg = " -d" if args.draft else ""
    run_command("Creating reconciliation PR", f"gh pr create -B main -f{draft_arg}")


def meta(args):
    init_args = [f"init -t {args.command}"]
    cherry_args = [f"cherry {' '.join(args.commit)}"] if args.command == "patch" else []
    body_args = [
        "changelog",
        "version -p 1",
        "preflight",
        "create",
        "watch",
        "postflight",
        "version",
        "preflight",
        "create",
        "watch",
        "postflight",
    ]
    reconcile_args = [] if args.command == "patch" else []
    meta_args = init_args + cherry_args + body_args + reconcile_args
    for args in meta_args:
        _main(f"meta {args}".split())


def _main(argv):
    parser = argparse.ArgumentParser(description="OpenDP release tool")
    parser.add_argument("-u", "--repo-url", default="git@github.com:opendp/opendp.git", help="Remote repo URL")
    parser.add_argument("-d", "--repo-dir", default="/tmp/opendp-release", help="Local repo directory")
    subparsers = parser.add_subparsers(dest="COMMAND", help="Command to run")
    subparsers.required = True

    subparser = subparsers.add_parser("init", help="Initialize the release process")
    subparser.set_defaults(func=init)
    subparser.add_argument("-c", "--clone", dest="clone", action="store_true", default=True)
    subparser.add_argument("-nc", "--no-clone", dest="clone", action="store_false")
    subparser.add_argument("-b", "--base-version")
    subparser.add_argument("-t", "--type", choices=["major", "minor", "patch"], required=True)
    subparser.add_argument("-r", "--ref")
    subparser.add_argument("-f", "--force", dest="force", action="store_true", default=False)
    subparser.add_argument("-nf", "--no-force", dest="force", action="store_false")

    subparser = subparsers.add_parser("cherry", help="Cherry pick commits")
    subparser.set_defaults(func=cherry)
    subparser.add_argument("commit", nargs="+")

    subparser = subparsers.add_parser("changelog", help="Update CHANGELOG file")
    subparser.set_defaults(func=changelog)

    subparser = subparsers.add_parser("version", help="Update versioned files")
    subparser.set_defaults(func=version)
    subparser.add_argument("-p", "--prerelease-number", type=int)

    subparser = subparsers.add_parser("preflight", help="Run preflight test")
    subparser.set_defaults(func=preflight)
    subparser.add_argument("-e", "--venv", default="preflight-venv", help="Virtual environment directory")

    subparser = subparsers.add_parser("create", help="Create a release")
    subparser.set_defaults(func=create)
    subparser.add_argument("-f", "--force", dest="force", action="store_true", default=False)
    subparser.add_argument("-nf", "--no-force", dest="force", action="store_false")
    subparser.add_argument("-n", "--draft", dest="draft", action="store_true", default=False)
    subparser.add_argument("-nn", "--no-draft", dest="draft", action="store_false")

    subparser = subparsers.add_parser("watch", help="Watch release progress")
    subparser.set_defaults(func=watch)

    subparser = subparsers.add_parser("postflight", help="Run postflight test")
    subparser.set_defaults(func=postflight)
    subparser.add_argument("-e", "--venv", default="postflight-venv", help="Virtual environment directory")

    subparser = subparsers.add_parser("reconcile", help="Reconcile after the final release")
    subparser.set_defaults(func=reconcile)
    subparser.add_argument("-f", "--force", dest="force", action="store_true", default=False)
    subparser.add_argument("-nf", "--no-force", dest="force", action="store_false")
    subparser.add_argument("-n", "--draft", dest="draft", action="store_true", default=False)
    subparser.add_argument("-nn", "--no-draft", dest="draft", action="store_false")

    subparser = subparsers.add_parser("major", help="Execute a typical major release")
    subparser.set_defaults(func=meta, command="major")
    subparser = subparsers.add_parser("minor", help="Execute a typical minor release")
    subparser.set_defaults(func=meta, command="minor")
    subparser = subparsers.add_parser("patch", help="Execute a typical patch release")
    subparser.set_defaults(func=meta, command="patch")
    subparser.add_argument("commit", nargs="+")

    args = parser.parse_args(argv[1:])
    args.func(args)


def main():
    _main(sys.argv)


if __name__ == "__main__":
    main()
