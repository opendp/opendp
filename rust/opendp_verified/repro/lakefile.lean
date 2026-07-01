import Lake
open System Lake DSL

package OpenDPVerified

-- This is a self-contained lake sub-package: its root is this directory
-- (rust/opendp_verified/repro/). The shared `aeneas`/`SampCert` checkouts live at
-- the git repo root, three levels up. Keeping the project here (with its own
-- lean-toolchain + lake-manifest) means it never touches the root lake files the
-- parallel `eddie/lean` PR owns.
def localSampCertDir : FilePath := "../../../SampCert"
def localAeneasDir : FilePath := "../../../aeneas/backends/lean"

meta if run_io localSampCertDir.pathExists then
  require SampCert from "../../../SampCert"
else
  require SampCert from git
    "https://github.com/Shoeboxam/SampCert.git" @ "9cb29f35bf56d160c199a56438add7f89542b83d"

require aeneas from "../../../aeneas/backends/lean"

@[default_target]
lean_lib OpenDPVerified where
  srcDir := "."
  roots := #[`OpenDPVerified]
  globs := #[
    .one `OpenDPVerified,
    .submodules `Generated,
    .submodules `src
  ]
  allowImportAll := true
