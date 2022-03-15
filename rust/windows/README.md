# OpenDP for Windows
Reproducible build for the [OpenDP library](https://github.com/opendp/opendp) with GMP/MPFR on Windows.
This repository provides a script that patches a clone of the OpenDP library.
For a simpler, developer install with non-secure noise, simply run this command:

```shell
cargo build --no-default-features
```
If just working with rust, then run the command from the opendp repo's `rust/opendp`.
If working with bindings languages, then run the command from `rust/opendp-ffi`.

If you want to build on Windows with secure noise, then follow the instructions below.
These instructions are part of the process for preparing a multiplatform OpenDP release on PyPi.

1. The GMP builds use native symlinks, which break on Windows. For these to work, you need to enable developer mode on Windows 10:
   ```
   Settings->Update and Security->For Developers
   Set the “Developer Mode” toggle at the top to be on.
   ```
   Additionally, if you want to run the build steps from a non-admin shell, you need to configure device policy to allow non-admins to make native symlinks.  Instructions at:    
   
   https://github.com/git-for-windows/git/wiki/Symbolic-Links#allowing-non-administrators-to-create-symbolic-links


1. Install choco `https://chocolatey.org/install`
   
1. Install dependencies
   ```cmd
   choco install rust
   rustup toolchain install stable-x86_64-pc-windows-gnu
   rustup default stable-x86_64-pc-windows-gnu
   choco install 7zip
   choco install curl
   choco install msys2
   ```
   
1. Set the environment variable:
   `MSYS=winsymlinks:nativestrict`
   
1. Set up wsl, then restart:
   `wsl --install`
   
1. Install MSYS2 dependencies and patch rust
   ```bash
   pacman -S --noconfirm git mingw-w64-x86_64-toolchain openssl-devel m4 vim diffutils make
   cp -f /mingw64/x86_64-w64-mingw32/lib/{*.a,*.o} /c/ProgramData/chocolatey/lib/rust/tools/lib/rustlib/x86_64-pc-windows-gnu/lib/self-contained
   ```
   See more in-depth comments in the [Technical Details](#technical-details) section.
   
1. Download sources and apply patches to the sources
   ```cmd
   chmod +x rust/windows/1_download_and_patch.sh
   (cd rust/windows && bash 1_download_and_patch.sh)
   ```
   
1. Build gmp and mpfr. Run these commands in an elevated MSYS2 shell
   ```cmd
   chmod +x rust/windows/2_build_dependencies.sh
   (cd rust/windows && bash 2_build_dependencies.sh)
   ```
   
1. Build the opendp library! This must also be run in an MSYS2 shell.
   ```cmd
   export RUSTFLAGS="-L native=C:\ProgramData\chocolatey\lib\rust\tools\lib\rustlib\x86_64-pc-windows-gnu\lib\self-contained"
   (cd rust && cargo build)
   ```
   If you add the RUSTFLAGS command to .bashrc, you don't need to set it every time you run cargo commands.
   Use `cargo build --release` for slower compilation and faster runtime.
   The .dll located in `/rust/target/{release|debug}/opendp.dll` can be used in language bindings.
   The python package should just work as-is once the build succeeds.


# Technical Details
The rust compiler is modified to use the currently-installed mingw binaries.
This is the `cp` in step 5.
[The `cp` fix is based on this comment.](https://github.com/rust-lang/rust/issues/47048#issuecomment-569225821)
Re-run this `cp` anytime you update Rust or mingw. The motivation for this is:
1. The rust install bundles an outdated version of mingw.
2. We use a newer mingw to build gmp and mpfr.
3. You can't mix multiple versions of mingw to compile the same binary.

GMP and MPFR are built from source. This is because:
1. The gmp-mpfr-sys crate is very particular about the library version being used
2. Pacman only stores latest
3. The binaries linked by precompiled .a files differ across machines

The build script for the rust crate gmp-mpfr-sys is modified to read the gmp and mpfr built binaries.  
The Cargo.toml for the rust crate rug is modified to point to the customized gmp-mpfr-sys.  
The Cargo.toml for the opendp crate is modified to point to the modified rug and gmp-mpfr-sys crates.

# Attribution
This patch is largely based on build instructions by Wenming Ye at AWS and Joshua Allen at Microsoft.
If you run your own build, please share your experiences to help us improve the build process.
