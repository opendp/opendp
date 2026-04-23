# Permute-and-Flip proof organization (reorganized)

This tree separates **permanent proof content** from **temporary hax workaround
scaffolding** as sharply as possible.

## Stable / intended-to-survive modules

- `ControlFlow/ExtractedStructure.lean`
  - Normalizes the hax-extracted implementation into proof-friendly branches and
    loop components.
  - Contains the concrete `fold_range_return` control-flow facts.

- `ControlFlow/ExtractedFunctionReduction.lean`
  - Transports the structured normalization back to the actual extracted
    `permute_and_flip_without_replacement` function.
  - Contains branch-reduction lemmas and law transport under computation
    equality.

- `Semantic/SampCertBridge.lean`
  - The mathematical bridge from an OpenDP-facing semantic law and normalization
    to SampCert's executable `permuteAndFlipSLang` theorem.
  - This should remain even once hax is complete, because it is not a hax
    artifact: it is the actual OpenDP-to-SampCert connection.

## Temporary / hax-workaround modules

- `Temporary/SamplerReplacement.lean`
  - Replaces extracted sampler bodies with semantic samplers because the current
    hax-generated Lean sampler layer still contains placeholders.

- `Temporary/RuntimeSemantics.lean`
  - Introduces an abstract law interpreter for extracted `RustM` computations and
    packages sampler-law assumptions at the runtime/law level.

- `Temporary/FinalBoundary.lean`
  - The final assumption bundle and theorem schema for the actual extracted
    function. This is the narrowest remaining swap-out surface while hax is
    being improved.

## Design goal

If hax's Lean backend becomes complete for the sampler/runtime layer, the entire
`Temporary/` directory should shrink dramatically or disappear. The rest of the
codebase should continue to make sense unchanged.
