# If this is in CWD, will be loaded before scripts are run.
options(error = function() {
  traceback()
  if (interactive()) {
    recover()
  } else {
    q(status = 1L)
  }
}, show.error.locations = TRUE
)

print("profile loaded")
