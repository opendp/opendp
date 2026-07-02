import Lake
open System Lake DSL

package OpenDPVerified

-- The lake package is rooted here (rust/opendp_verified/), beside the Rust
-- crate it verifies: lean-toolchain, lake-manifest.json, .lake/, Generated/
-- and the handwritten proofs (src/) all live in this directory. The shared
-- `aeneas`/`SampCert` checkouts are gitignored at the git repo root, two
-- levels up (tools/check_lean_pins.sh guards their pins).
def localSampCertDir : FilePath := "../../SampCert"
def localAeneasDir : FilePath := "../../aeneas/backends/lean"

meta if run_io localSampCertDir.pathExists then
  require SampCert from "../../SampCert"
else
  require SampCert from git
    "https://github.com/Shoeboxam/SampCert.git" @ "9cb29f35bf56d160c199a56438add7f89542b83d"

require aeneas from "../../aeneas/backends/lean"

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
