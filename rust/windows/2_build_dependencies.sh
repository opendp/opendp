# this script should be run in an MSYS2 shell

# exit immediately upon failure, print commands while running
set -e -x
cargo --version
gcc --version

cd gmp-6.2.0
./configure --host=x86_64-w64-mingw32 CFLAGS="-march=x86-64"
make
make check
cd ..

cd mpfr-4.0.2
./configure --host=x86_64-w64-mingw32 --with-gmp-include="../gmp-6.2.0/" --with-gmp-lib="../gmp-6.2.0/.libs/"  --enable-static --disable-shared CFLAGS="-march=x86-64"
make
make check
cd ..

#export PATH=$PATH:/usr/bin/core_perl:/c/ProgramData/chocolatey/bin:/c/msys64/mingw64/bin
#export RUSTFLAGS="-L native=C:\ProgramData\chocolatey\lib\rust\tools\lib\rustlib\x86_64-pc-windows-gnu\lib\self-contained"
#
# These don't actually need to be built directly- they are built when opendp builds
# They are useful to build individually when debugging the build process though
## build gmp-mpfr-sys
#cd gmp-mpfr-sys-1.2.4 || exit
#cargo build --target=x86_64-pc-windows-gnu --release --no-default-features --features=mpfr
#cargo test --target=x86_64-pc-windows-gnu --no-default-features --features=mpfr
#cd ..
#
## build rug
#cd rug-1.9.0 || exit
#cargo build --target=x86_64-pc-windows-gnu --release --no-default-features --features=integer,float,rand
## there are broken trailing semicolons in the rug tests. Everything else passes
##cargo test --target=x86_64-pc-windows-gnu --no-default-features --features=integer,float,rand
#cd ..
