import configparser
import semver
import toml

# all version numbers should be:
version = open("VERSION", 'r').read().strip()
version = semver.Version.parse(version)
assert version.prerelease != "dev", "Please update the repository with the current version."
print("Checking if all version numbers are synchronized at", version)

# From channel_tool.py
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

python_version = get_python_version(version)
config = configparser.RawConfigParser()
config.read('python/setup.cfg')
assert config['metadata']['version'] == str(python_version), \
    "python/setup.cfg package version is incorrect"

# check that opendp crate version is set properly
# cargo doesn't like build metadata in dependency references, so we strip that for those.
stripped_version = version.replace(build=None)
opendp_toml = toml.load('rust/Cargo.toml')
assert opendp_toml['workspace']['package']['version'] == version, \
    "rust/Cargo.toml workspace version is incorrect"
assert opendp_toml['dependencies']['opendp_derive']['version'] == version, \
    "rust/Cargo.toml dependency opendp_derive version is incorrect"
assert opendp_toml['build-dependencies']['opendp_tooling']['version'] == version, \
    "rust/Cargo.toml build-dependency opendp_tooling version is incorrect"
opendp_derive_toml = toml.load('rust/opendp_derive/Cargo.toml')
assert opendp_derive_toml['dependencies']['opendp_tooling']['version'] == version, \
    "rust/Cargo.toml dependency opendp_derive version is incorrect"

print("All version numbers are synchronized")
