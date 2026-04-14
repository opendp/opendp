import opendp.prelude as dp

# We don't have tests that require a feature *not* to be enabled,
# so more stable just to enable everything at the start.
dp.enable_features('idealized-numerics', 'contrib', 'honest-but-curious')
