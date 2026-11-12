import opendp.prelude as dp

# Enable features for the test suite by default; gating tests override this locally.
dp.enable_features("idealized-numerics", "contrib", "honest-but-curious")
