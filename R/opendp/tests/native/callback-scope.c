#include <R.h>
#include <Rinternals.h>
#include "convert.h"

// Pass addresses explicitly: the test DLL does not link against package-private
// functions and works even when R loaded opendp with local symbol visibility.
typedef struct {
    SEXP function;
    int transition;
    void *converter;
} TestContext;

static SEXP fail_after_conversion(void *data)
{
    TestContext *ctx = (TestContext *)data;
    if (ctx->transition)
        ((TransitionFn (*)(SEXP, const char *))ctx->converter)(ctx->function, "i32");
    else
        ((CallbackFn (*)(SEXP, const char *))ctx->converter)(ctx->function, "i32");
    // Models a later argument conversion raising before the library call.
    Rf_error("later argument conversion failed");
    return R_NilValue;
}

SEXP test_callback_scope(SEXP function, SEXP transition, SEXP scope, SEXP converter)
{
    TestContext ctx = {function, Rf_asLogical(transition), R_ExternalPtrAddr(converter)};
    SEXP (*with_scope)(SEXP (*)(void *), void *) = R_ExternalPtrAddr(scope);
    return with_scope(fail_after_conversion, &ctx);
}
