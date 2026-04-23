import Lake
open System Lake DSL

package OpenDPVerified

def localSampCertDir : FilePath := "SampCert"
def localAeneasDir : FilePath := "aeneas/backends/lean"

meta if run_io localSampCertDir.pathExists then
  require SampCert from "SampCert"
else
  require SampCert from git
    "https://github.com/Shoeboxam/SampCert.git" @ "9cb29f35bf56d160c199a56438add7f89542b83d"

require aeneas from "aeneas/backends/lean"

@[default_target]
lean_lib OpenDPVerified where
  srcDir := "rust/opendp_verified"
  roots := #[`OpenDPVerified]
  globs := #[
    .one `OpenDPVerified,
    .submodules `Generated,
    .submodules `src
  ]
  allowImportAll := true
