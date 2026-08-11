# R + Polars integration: packaging & deployment analysis

Status: investigation of why a *live* r-polars is unavailable for testing, and
what it implies for shipping the OpenDP R Polars integration. Findings below are
evidence-backed; recommendations are marked as such.

## TL;DR

The integration exchanges **serialized Polars logical plans / expressions**
between R (`r-polars`) and OpenDP's Rust, which each **bundle their own copy of
Polars**. Polars binary DSL serialization is *version-locked by design* and the
OpenDP plugin path needs a Polars feature (`ffi_plugin`) that **stock r-polars
does not enable**. So the integration currently requires a *specific, patched*
r-polars build. That is the root packaging/deployment problem — not a missing
`install.packages()`.

## Why a live r-polars is "missing" here — three independent layers

1. **Not installed.** No `polars` in the library. Installing from r-universe
   (`rpolars.r-universe.dev`) pulls a **source** tarball (compiles the Rust
   backend) and currently fails on missing R deps (`rlang`, `S7`).
2. **Wrong version / schema.** r-universe serves polars **1.13**; the OpenDP
   build pins **r-polars 1.7.0 / py-polars 1.36.1 / DSL hash `4aade…`**
   (`R/polars.R::.assert_r_polars_compatibility`). Stock r-polars `main` is now
   at `PY_VERSION = "1.42.1"` with a different DSL hash — its bundled Polars is
   not DSL-compatible with OpenDP's bundled Polars 0.52.0.
3. **`ffi_plugin` absent from stock.** `pola-rs/r-polars`'s `Cargo.toml` does
   **not** enable the `ffi_plugin` feature on its Polars deps. Without it,
   r-polars' Polars cannot (de)serialize the `FfiPlugin` DSL variant that every
   OpenDP `dp_*` expression is built from → `deserialize_expr` fails with
   "unknown variant `FfiPlugin`". Confirmed at the source.

`DSL_SCHEMA_HASH_CURRENT` and `PY_VERSION` **are real stock r-polars symbols**
(`R/generated-polars-version.R`), so the compat check is not reading bespoke
symbols — but their values change every r-polars release, and OpenDP's expected
hash corresponds to the **`ffi_plugin`-patched** 1.7.0 build. Net effect: a
stock r-polars fails the compat check *by design*, which is correct (it would
not work), but means **only a patched r-polars ever passes**.

## The fundamental coupling (this is architectural, not a bug)

- OpenDP-Rust statically links Polars `0.52.0` (== py-polars `1.36.1`).
- r-polars statically links *its own* Polars.
- The two communicate only by passing **serialized DSL bytes**. Polars does not
  support cross-version DSL interchange and guards it with `DSL_SCHEMA_HASH`
  (see pola-rs/polars#24054); `Expr.deserialize` docs state serialization "is
  not stable across Polars versions."

Consequence: OpenDP is coupled to **one specific r-polars release** (the one
whose bundled Polars matches OpenDP's), and must re-pin whenever it bumps its
own Polars. This is inherent to the "two independently-compiled Polars" design.

## Build/packaging bugs found (fixable, independent of the above)

1. **`opendp.h` is uncompilable when generated under `polars-plugin`.** `cbindgen`
   picks up `rust/src/polars/plugin_ffi.rs`'s `#[no_mangle]` functions and emits
   declarations referencing `SeriesExport` / `ArrowSchema` / `CallerContext` —
   types it cannot resolve → the R package's C compilation fails
   (`unknown type name 'SeriesExport'`). The header is only correct when
   generated under `polars-ffi`, where `plugin_ffi.rs` is `#[cfg]`-excluded.
   *Recommended fix:* exclude those runtime-only ABI symbols from cbindgen
   (a `cbindgen.toml` `export.exclude`, or `/// cbindgen:ignore`) so the header
   is correct regardless of which polars feature generated it. They are called
   by Polars via `dlsym` at runtime, never from OpenDP's C glue, so they should
   never appear in `opendp.h`. *(Worked around this session by stripping the
   block from the generated header.)*
2. **`polars` was missing from `DESCRIPTION`.** `polars.R` uses
   `requireNamespace("polars")` but it was not declared. Fixed: added
   `polars (>= 1.7.0)` to `Suggests`.
3. **No committed recipe for the required patched r-polars.** The pinned hash
   `4aade…` was produced by a manual local build; nothing in `.github/`,
   `tools/`, or `docs/` reproduces it. This is the biggest deployment gap:
   there is currently no reproducible way for a user or CI to obtain a working
   r-polars.

## What was validated this session (no r-polars needed)

The OpenDP R package **builds and installs** (release lib via the Makevars
`OPENDP_LIB_DIR` dev fallback, after stripping bug #1's header block). The
r-polars-independent tests **pass at runtime** — `wild_expr_domain` construction
(row-by-row + all aggregation-margin variants) and every `dp_*` parameter
validation case. This exercises the `.Call` arity, `sexp_to_*` conversions, and
C glue that compile-checks cannot catch. The 12 plugin/DSL tests skip cleanly
with "{polars} is not installed".

## Recommendations

**Short term (unblock testing + honest deployment):**
- Fix bug #1 (cbindgen exclusion) so any build produces a compilable header.
- Commit a build recipe (script + CI job) that produces the required
  `r-polars 1.7.0 + ffi_plugin` and records the resulting DSL hash. Without
  this, the plugin path is not reproducibly buildable by anyone.

**Long term (make stock r-polars viable):**
- *Upstream `ffi_plugin`* into `pola-rs/r-polars` (enable the feature on its
  `polars-lazy`/`polars-plan` deps). This is the cleanest fix: it removes the
  need for a *patched* r-polars, leaving only the (unavoidable) version pin.
- Alternatively, OpenDP **hosts a pinned r-polars** (own r-universe or bundled
  binary). Works today but adds maintenance and can conflict with a user's own
  r-polars.

**Robustness (recommended, needs verification):** replace the hard
exact-`"1.7.0"`-version + single-hash assert with (a) **feature-detect**
`ffi_plugin` availability and (b) **defer to Polars' native `DSL_SCHEMA_HASH`
guard** at deserialize time (which already produces a clear incompatibility
error). This would let OpenDP track compatible r-polars releases instead of
snapping on every patch bump — *provided* the native guard also covers the
`ffi_plugin`/ABI case (verify before relying on it). The version pin to a
matching Polars remains fundamental regardless.

## Getting a live plugin-capable r-polars (recipe + caveat)

To run the 12 skipped plugin tests locally: build r-polars from the Polars rev
matching OpenDP (`py-1.36.1`) with `ffi_plugin` enabled, install its R deps
(`rlang`, `S7`, …). **Caveat:** a fresh build is not guaranteed to reproduce the
byte-identical `DSL_SCHEMA_HASH` `4aade…`; if it differs, `.assert_r_polars_compatibility`
still rejects it. That mismatch *is* the deployment problem in miniature —
every consumer needs the byte-identical build — which is exactly why the
recipe (short-term rec above) must be committed and its hash pinned together.
