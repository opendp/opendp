# OpenDP Migration

This file documents the migration history of OpenDP. The links on each version number will take you to a comparison
showing the source changes from the previous version.

Please keep this up to date, following the [instructions](#instructions) below.


## [Unreleased](https://github.com/opendp/opendp/compare/stable...HEAD)

- `make_base_gaussian`'s output measure is now ZeroConcentratedDivergence.
    - This means the output distance is now a single scalar, rho (it used to be an (ε, δ) tuple)
    - Use `adp_meas = opendp.comb.make_zCDP_to_approxDP(zcdp_meas)` to convert to an ε(δ) curve. 
    - Use `fadp_meas = opendp.comb.make_fix_delta(adp_meas)` to change output distance from an ε(δ) curve to an (ε, δ) tuple
        - `fadp_meas.check(d_in, (ε, δ))` is equivalent to the check on `make_base_gaussian` in 0.4
- replace `make_base_analytic_gaussian` with `make_base_gaussian`
- replace `make_base_geometric` with `make_base_discrete_laplace`
- `make_basic_composition` accepts a list of measurements as its first argument (it used to have two arguments)
- slight increase in sensitivities/privacy utilization across the library as a byproduct of floating-point attack mitigations

## Instructions
The format of this file is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). It is processed by
scripts when generating a release, so please maintain the existing format.

Whenever you're preparing a significant commit, add a bullet list entry summarizing the change under the
[Unreleased](#unreleased) heading at the top. 

When a new version is released, a script will turn the Unreleased heading into a new heading with appropriate values for the version, date, and link. 
Then the script will generate a new Unreleased section for future work.
Please keep the existing dummy heading and link as they are, so that things operate correctly. Thanks!