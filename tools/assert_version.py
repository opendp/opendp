import configparser
import tomlkit
from debmutate.control import ControlEditor

from utils import get_version, get_python_version, get_r_version

# all version numbers should be:
version = get_version()
python_version = get_python_version(version)
r_version = get_r_version(version)

assert version.prerelease is None or "dev" not in version.prerelease, "Please configure the channel with a non-dev version."
print("Checking if all version numbers are synchronized at", version, python_version)

# check that opendp crate version is set properly
# cargo doesn't like build metadata in dependency references, so we strip that for those.
stripped_version = version.replace(build=None)
opendp_toml = tomlkit.load(open('rust/Cargo.toml'))
assert opendp_toml['workspace']['package']['version'] == version, \
    "rust/Cargo.toml workspace version is incorrect"
assert opendp_toml['dependencies']['opendp_derive']['version'] == version, \
    "rust/Cargo.toml dependency opendp_derive version is incorrect"
assert opendp_toml['build-dependencies']['opendp_tooling']['version'] == version, \
    "rust/Cargo.toml build-dependency opendp_tooling version is incorrect"
opendp_derive_toml = tomlkit.load(open('rust/opendp_derive/Cargo.toml'))
assert opendp_derive_toml['dependencies']['opendp_tooling']['version'] == version, \
    "rust/Cargo.toml dependency opendp_derive version is incorrect"

config = configparser.RawConfigParser()
config.read('python/setup.cfg')
assert config['metadata']['version'] == str(python_version), \
    "python/setup.cfg package version is incorrect"

with ControlEditor(path='R/opendp/DESCRIPTION') as control:
    version = next(iter(control.paragraphs))["Version"]
    assert version == r_version

print("All version numbers are synchronized")
