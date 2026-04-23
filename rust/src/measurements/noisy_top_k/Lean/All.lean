import ControlFlow.ExtractedStructure
import ControlFlow.ExtractedFunctionReduction
import Semantic.SampCertBridge
import Temporary.SamplerReplacement
import Temporary.RuntimeSemantics
import Temporary.FinalBoundary

/-!
# Permute-and-Flip proof bundle

Entry point importing the organized permute-and-flip proof development.

`Temporary/` contains only the hax-specific workaround layer. Everything else is
intended to remain after hax's sampler/runtime extraction is completed.
-/
