import subprocess
import sys
import time
from pathlib import Path

import semver


def log(message, command=False):
    prefix = "$" if command else "#"
    print(f"{prefix} {message}", file=sys.stderr)


def run_command(description, cmd, capture_output=False, shell=True):
    if description:
        log(description)
    printed_cmd = " ".join(cmd) if isinstance(cmd, list) else cmd
    log(printed_cmd, command=True)
    stdout = subprocess.PIPE if capture_output else None
    completed_process = subprocess.run(cmd, stdout=stdout, shell=shell, check=True, encoding="utf-8")
    return completed_process.stdout.rstrip() if capture_output else None


def run_command_with_retries(description, args, timeout, backoff, capture_output=False, shell=True):
    start = time.time()
    wait = 1.0
    while True:
        try:
            return run_command(description, args, capture_output=capture_output, shell=shell)
        except Exception as e:
            elapsed = time.time() - start
            if elapsed >= timeout:
                raise e
        w = min(wait, timeout - elapsed)
        log(f"Retrying in {w:.1f} seconds")
        time.sleep(w)
        wait *= backoff


def get_version(version_str=None):
    if not version_str:
        version_str = (Path(__file__).parent.parent / 'VERSION').read_text().strip()
    return semver.Version.parse(version_str)


def get_python_version(version):
    # Python (PEP 440) has several annoying quirks that make it not quite compatible with semantic versioning:
    # 1. Python doesn't allow arbitrary tags, only (a|b|rc|post|dev). (You can use (alpha|beta|c|pre|preview|rev|r),
    #    but they'll be mapped to (a|b|rc|rc|rc|post|post) respectively.)
    #    So "1.2.3-nightly.456" will fail, and "1.2.3-alpha.456" gets mapped to "1.2.3a456" (see #2).
    # 2. Python doesn't allow separators between the main version and the tag, nor within the tag.
    #    So "1.2.3-a.456" gets mapped to "1.2.3a456"
    # 3. HOWEVER, Python treats tags "post" and "dev" differently, and in these cases uses a "." separator between
    #    the main version and the tag (but still doesn't allow separators within the tag).
    #    So "1.2.3-dev.456" gets mapped to "1.2.3.dev456".
    # 4. Python requires that all tags have a numeric suffix, and will assume 0 if none is present.
    #    So "1.2.3-dev" gets mapped to "1.2.3.dev0" (by #3 & #4).
    # We don't use all these variations, only (dev|nightly|beta), but if that ever changes, hopefully we won't
    # have to look at this whole mess again.
    tag_to_py_tag = {
        "nightly": "a",
        "beta": "b",
        "c": "rc",
        "pre": "rc",
        "preview": "rc",
        "rev": "post",
        "r": "post",
    }
    if version.prerelease is not None:
        split = version.prerelease.split(".", 2)
        tag = split[0]
        py_tag = tag_to_py_tag.get(tag, tag)
        date = split[1] if len(split) >= 2 else None
        counter = split[2] if len(split) >= 3 else None
        py_n = f"{date}{counter:>03}" if date and counter else (date if date else "0")
        py_separator = "." if py_tag in ("post", "dev") else ""
    else:
        py_tag = None
        py_n = None
        py_separator = None
    # semver can't represent the rendered Python version, so we generate a string.
    if py_tag is not None:
        return f"{version.major}.{version.minor}.{version.patch}{py_separator}{py_tag}{py_n}"
    else:
        return str(version)


def get_r_version(version):
    # r versions cannot represent pre-releases.
    # Can only use . or -, and both are treated interchangeably
    # This means a "prerelease" named like 0.1.0.202308141 is considered greater than 0.1.0
    # Therefore the pre-release designation is just removed completely
    # https://cran.r-project.org/doc/manuals/R-exts.html#The-DESCRIPTION-file
    return f"{version.major}.{version.minor}.{version.patch}"


def infer_channel(version):
    if version.prerelease is None:
        return "stable"
    channel = version.prerelease.split(".", 1)[0]
    if channel not in ("dev", "nightly", "beta"):
        raise Exception(f"Unable to infer channel from version {version}")
    return channel


def get_current_branch():
    return run_command("Determining current branch", "git branch --show-current", capture_output=True)


def main():
    version = get_version()
    python_version = get_python_version(version)
    print(f"{version} ({python_version})")


if __name__ == "__main__":
    main()
