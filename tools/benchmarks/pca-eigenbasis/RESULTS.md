# Results

Latency in ms: median [Q25, Q75]. Proposals are means over 128 separate
seeded diagnostic outputs; acceptance is outputs / proposals. Timing uses the
uninstrumented CSPRNG (31 calls after 3 warmups). No case exceeded the time budget.

| d | Spectrum | κ | Base ms [IQR] | Eigenbasis ms [IQR] | Speedup | Proposals base / opt | Acceptance base / opt |
|--:|:--|--:|:--|:--|--:|:--|:--|
| 4 | isotropic | 0 | 1.034 [1.022, 1.062] | 1.001 [0.980, 1.018] | 1.03× | 1.00 / 1.00 | 100.0% / 100.0% |
| 4 | isotropic | 10 | 1.008 [1.002, 1.013] | 1.022 [1.001, 1.107] | 0.99× | 1.00 / 1.00 | 100.0% / 100.0% |
| 4 | isotropic | 1000 | 1.022 [1.017, 1.035] | 0.999 [0.993, 1.015] | 1.02× | 1.00 / 1.00 | 100.0% / 100.0% |
| 4 | moderate | 0 | 1.041 [1.029, 1.055] | 0.990 [0.986, 1.020] | 1.05× | 1.00 / 1.00 | 100.0% / 100.0% |
| 4 | moderate | 10 | 1.089 [1.065, 1.112] | 1.005 [0.980, 1.051] | 1.08× | 1.65 / 1.63 | 60.7% / 61.2% |
| 4 | moderate | 1000 | 1.090 [1.068, 1.142] | 1.031 [1.025, 1.046] | 1.06× | 2.61 / 2.44 | 38.3% / 41.0% |
| 4 | strong | 0 | 1.012 [1.007, 1.028] | 0.999 [0.991, 1.024] | 1.01× | 1.00 / 1.00 | 100.0% / 100.0% |
| 4 | strong | 10 | 1.074 [1.047, 1.129] | 1.019 [1.011, 1.031] | 1.05× | 2.17 / 2.01 | 46.0% / 49.8% |
| 4 | strong | 1000 | 1.089 [1.075, 1.120] | 1.021 [1.014, 1.039] | 1.07× | 2.61 / 2.45 | 38.3% / 40.8% |
| 16 | isotropic | 0 | 1.081 [1.070, 1.086] | 1.037 [0.986, 1.057] | 1.04× | 1.00 / 1.00 | 100.0% / 100.0% |
| 16 | isotropic | 10 | 1.085 [1.042, 1.112] | 1.010 [1.000, 1.032] | 1.07× | 1.00 / 1.00 | 100.0% / 100.0% |
| 16 | isotropic | 1000 | 1.068 [1.023, 1.090] | 1.019 [0.999, 1.026] | 1.05× | 1.00 / 1.00 | 100.0% / 100.0% |
| 16 | moderate | 0 | 1.088 [1.077, 1.098] | 1.037 [1.002, 1.102] | 1.05× | 1.00 / 1.00 | 100.0% / 100.0% |
| 16 | moderate | 10 | 1.123 [1.103, 1.158] | 1.095 [1.078, 1.186] | 1.02× | 1.11 / 1.07 | 90.1% / 93.4% |
| 16 | moderate | 1000 | 1.426 [1.285, 1.516] | 1.147 [1.131, 1.176] | 1.24× | 4.62 / 4.72 | 21.7% / 21.2% |
| 16 | strong | 0 | 1.082 [1.067, 1.096] | 1.040 [1.011, 1.070] | 1.04× | 1.00 / 1.00 | 100.0% / 100.0% |
| 16 | strong | 10 | 1.154 [1.138, 1.228] | 1.072 [1.063, 1.102] | 1.08× | 1.34 / 1.48 | 74.4% / 67.4% |
| 16 | strong | 1000 | 1.324 [1.260, 1.437] | 1.142 [1.112, 1.188] | 1.16× | 4.84 / 4.72 | 20.6% / 21.2% |
| 64 | isotropic | 0 | 1.350 [1.324, 1.368] | 1.236 [1.201, 1.251] | 1.09× | 1.00 / 1.00 | 100.0% / 100.0% |
| 64 | isotropic | 10 | 1.379 [1.368, 1.412] | 1.273 [1.248, 1.326] | 1.08× | 1.00 / 1.00 | 100.0% / 100.0% |
| 64 | isotropic | 1000 | 1.344 [1.330, 1.378] | 1.228 [1.192, 1.259] | 1.09× | 1.00 / 1.00 | 100.0% / 100.0% |
| 64 | moderate | 0 | 1.504 [1.414, 1.623] | 1.321 [1.309, 1.350] | 1.14× | 1.00 / 1.00 | 100.0% / 100.0% |
| 64 | moderate | 10 | 1.719 [1.703, 1.771] | 1.342 [1.328, 1.366] | 1.28× | 1.01 / 1.01 | 99.2% / 99.2% |
| 64 | moderate | 1000 | 3.228 [2.340, 4.846] | 1.692 [1.629, 1.823] | 1.91× | 8.76 / 6.99 | 11.4% / 14.3% |
| 64 | strong | 0 | 1.384 [1.374, 1.399] | 1.354 [1.324, 1.389] | 1.02× | 1.00 / 1.00 | 100.0% / 100.0% |
| 64 | strong | 10 | 1.749 [1.689, 1.887] | 1.347 [1.336, 1.356] | 1.30× | 1.00 / 1.00 | 100.0% / 100.0% |
| 64 | strong | 1000 | 4.771 [3.210, 7.246] | 1.559 [1.476, 1.687] | 3.06× | 9.48 / 8.20 | 10.6% / 12.2% |
| 128 | isotropic | 0 | 2.174 [2.101, 2.207] | 1.578 [1.559, 1.605] | 1.38× | 1.00 / 1.00 | 100.0% / 100.0% |
| 128 | isotropic | 10 | 2.162 [2.095, 2.233] | 1.591 [1.554, 1.655] | 1.36× | 1.00 / 1.00 | 100.0% / 100.0% |
| 128 | isotropic | 1000 | 2.133 [2.011, 2.195] | 1.567 [1.552, 1.579] | 1.36× | 1.00 / 1.00 | 100.0% / 100.0% |
| 128 | moderate | 0 | 2.279 [2.208, 2.332] | 2.015 [1.962, 2.079] | 1.13× | 1.00 / 1.00 | 100.0% / 100.0% |
| 128 | moderate | 10 | 3.693 [3.616, 3.818] | 2.008 [1.947, 2.046] | 1.84× | 1.00 / 1.00 | 100.0% / 100.0% |
| 128 | moderate | 1000 | 7.486 [5.136, 17.020] | 2.458 [2.219, 2.670] | 3.05× | 7.38 / 8.63 | 13.5% / 11.6% |
| 128 | strong | 0 | 2.340 [2.260, 2.384] | 1.989 [1.945, 2.038] | 1.18× | 1.00 / 1.00 | 100.0% / 100.0% |
| 128 | strong | 10 | 3.716 [3.593, 3.760] | 2.019 [1.952, 2.099] | 1.84× | 1.00 / 1.00 | 100.0% / 100.0% |
| 128 | strong | 1000 | 13.544 [7.934, 21.858] | 2.555 [2.320, 2.730] | 5.30× | 10.88 / 11.51 | 9.2% / 8.7% |
| 256 | isotropic | 0 | 6.609 [6.474, 6.696] | 3.242 [3.123, 3.312] | 2.04× | 1.00 / 1.00 | 100.0% / 100.0% |
| 256 | isotropic | 10 | 6.705 [6.549, 6.901] | 3.310 [3.194, 3.497] | 2.03× | 1.00 / 1.00 | 100.0% / 100.0% |
| 256 | isotropic | 1000 | 6.477 [6.419, 6.556] | 3.141 [3.102, 3.200] | 2.06× | 1.00 / 1.00 | 100.0% / 100.0% |
| 256 | moderate | 0 | 7.289 [7.194, 7.385] | 4.591 [4.550, 4.671] | 1.59× | 1.00 / 1.00 | 100.0% / 100.0% |
| 256 | moderate | 10 | 11.353 [11.125, 11.505] | 4.816 [4.686, 4.895] | 2.36× | 1.00 / 1.00 | 100.0% / 100.0% |
| 256 | moderate | 1000 | 21.193 [11.634, 37.126] | 5.067 [4.818, 5.329] | 4.18× | 5.50 / 4.80 | 18.2% / 20.8% |
| 256 | strong | 0 | 7.273 [7.181, 7.424] | 4.655 [4.486, 4.780] | 1.56× | 1.00 / 1.00 | 100.0% / 100.0% |
| 256 | strong | 10 | 11.281 [11.117, 11.365] | 4.579 [4.497, 4.690] | 2.46× | 1.00 / 1.00 | 100.0% / 100.0% |
| 256 | strong | 1000 | 16.589 [11.267, 19.464] | 5.368 [5.038, 5.758] | invalid base* | 1.92 / 14.45 | 52.0% / 6.9% |

*The dimension-256 strong-spectrum κ=1000 baseline emitted overflow and invalid
multiply warnings (see `baseline.txt`). Its direct acceptance expression can
produce `inf` (spurious acceptance) or `NaN` (spurious rejection). Its latency and
counts are observations of the old code, **not** an equivalent-distribution
speedup. For this spectrum b≈1.14688, log(M)≈564.815; the power term alone
overflows when wᵀAw exceeds ≈146.227. The log-domain implementation completes
without warnings and uses 14.45 proposals/output versus the invalid baseline’s
1.92. The moderate-spectrum dimension-256 κ=1000 case has a much higher power
overflow threshold (≈439.298); no warnings were observed there in this run.

All zero-concentration and isotropic cases used exactly one proposal/output in
both diagnostic runs. For the remaining (numerically valid) high-concentration
anisotropic cases, mean proposal counts ranged from 2.61 to 10.88 (base) and
2.44 to 14.45 (optimized, including the dimension-256 strong case). Different
coordinates consume the seeded stream differently; counts need not match.
Latency IQRs are wide in rejection-heavy baseline cases; speedups are ratios of
observed medians, not per-proposal microbenchmark estimates. In particular, the
moderate-spectrum κ=1000 dimension-128 case is faster despite a higher diagnostic
proposal mean in the optimized run.

Source, script, library and all 45 matrix hashes were checked. Both runs use
Python 3.14.3, NumPy 2.4.6, Apple Accelerate BLAS/LAPACK,
Apple M4 Pro, macOS-26.5.2-arm64-arm-64bit-Mach-O, single-thread limits.

Base measured commit: `9e02e1bd6143a17fa42d8a558a40ef81b42a751a`.
Optimized measured commit: `c0cd22d043c2545a5297a9adfef428ec55064f4f`.
Optimized sampler SHA-256: `396ed7ae237d6034eec20007215975ac41ca9a4d2c90fb24658823f10df0f154`.
Results/docs are subsequently amended into that one development commit; its
final Git SHA therefore differs, but the measured sampler is byte-for-byte
unchanged. `optimized.json` retains the exact measured commit and source hash.
The final deliverable SHA is reported in the delivery response.
