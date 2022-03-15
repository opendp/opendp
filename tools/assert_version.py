import configparser
import toml

# all version numbers should be:
version = open("VERSION", 'r').read().strip()
assert version != "0.0.0-development", "Please update the repository with the current version."
print("Checking if all version numbers are synchronized at", version)

# check that python version is set properly
config = configparser.RawConfigParser()
config.read('python/setup.cfg')
assert config['metadata']['version'] == version, \
    "python/setup.cfg package version is incorrect"

# check that opendp crate version is set properly
opendp_toml = toml.load('../rust/Cargo.toml')
assert opendp_toml['package']['version'] == version, \
    "rust/Cargo.toml crate version is incorrect"

print("All version numbers are synchronized")
