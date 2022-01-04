# this script should be run in an MSYS2 shell

# exit immediately upon failure, print commands while running
set -e -x

# these need to be built from source because
# 1. The gmp-mpfr-sys crate is very particular about the library version being used
# 2. pacman only stores latest
# 3. the binaries linked by precompiled .a files differ across machines
curl https://gmplib.org/download/gmp/gmp-6.2.0.tar.bz2 -o gmp-6.2.0.tar.bz2
tar xf gmp-6.2.0.tar.bz2

curl https://www.mpfr.org/mpfr-4.0.2/mpfr-4.0.2.tar.gz -o mpfr-4.0.2.tar.gz
tar xf mpfr-4.0.2.tar.gz

# clone the gmp-mpfr-sys crate
rm -rf gmp-mpfr-sys-1.2.4
git clone https://gitlab.com/tspiteri/gmp-mpfr-sys.git gmp-mpfr-sys-1.2.4
cd gmp-mpfr-sys-1.2.4 || exit
git checkout tags/v1.2.4
# patch the gmp-mpfr-sys build.rs to use the pre-built static libs
git apply --ignore-whitespace ../gmp-mpfr-sys-1.2.4.patch
cd ..

# clone the rug crate
rm -rf rug-1.9.0
git clone https://gitlab.com/tspiteri/rug.git rug-1.9.0
cd rug-1.9.0 || exit
git checkout tags/v1.9.0
# patch the rug cargo.toml to use the pre-built static libs
git apply --ignore-whitespace ../rug-1.9.0.patch
cd ..

CARGO_TOML_PATH="$(realpath "../opendp/Cargo.toml")"
sed -i "s|rug = { version = \"1.9.0\"|rug = { path = \"../windows/rug-1.9.0\"|g" "$CARGO_TOML_PATH"
sed -i "s|gmp-mpfr-sys = { version = \"=1.2.4\"|gmp-mpfr-sys = { path = \"../windows/gmp-mpfr-sys-1.2.4\"|g" "$CARGO_TOML_PATH"
