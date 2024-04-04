# This is NOT loaded automatically by R!
# Instead, we'll use R_PROFILE to point to this file.
options(error = function() {
  traceback()
  if (interactive()) {
    browser()
  } else {
    q(status = 1L)
  }
})

print("profile loaded")
