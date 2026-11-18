#ifndef OPENDP_R_OWNERSHIP_H
#define OPENDP_R_OWNERSHIP_H

#include <Rinternals.h>

// R's API is single-threaded. Scopes nest when a Rust callback reenters R.
SEXP opendp_with_scope(SEXP (*body)(void *), void *data);
void opendp_register_resource(void *ptr, void (*release)(void *));
void opendp_forget_resource(void *ptr);

#endif
