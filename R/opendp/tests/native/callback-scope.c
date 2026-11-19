#include <R.h>
#include <Rinternals.h>
#include <stdlib.h>
#include <stdbool.h>
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

// An opaque R wrapper must never take ownership of an untracked pointer.
typedef struct {
    void (*forget)(void *);
    int unregistered;
} UnregisteredResourceContext;

static SEXP forget_unregistered(void *data)
{
    UnregisteredResourceContext *ctx = (UnregisteredResourceContext *)data;
    ctx->forget(&ctx->unregistered);
    return R_NilValue;
}

SEXP test_forget_unregistered(SEXP scope, SEXP forget)
{
    SEXP (*with_scope)(SEXP (*)(void *), void *) = R_ExternalPtrAddr(scope);
    UnregisteredResourceContext ctx = {R_ExternalPtrAddr(forget), 0};
    return with_scope(forget_unregistered, &ctx);
}

typedef struct {
    SEXP function;
    int transition;
    void *converter;
    void (*forget)(void *);
    CallbackFn callback;
    TransitionFn transition_callback;
} DirectCallbackContext;

static SEXP retain_callback(void *data)
{
    DirectCallbackContext *ctx = (DirectCallbackContext *)data;
    if (ctx->transition)
    {
        ctx->transition_callback = ((TransitionFn (*)(SEXP, const char *))ctx->converter)(ctx->function, "i32");
        ctx->forget((void *)ctx->transition_callback.userdata._0);
    }
    else
    {
        ctx->callback = ((CallbackFn (*)(SEXP, const char *))ctx->converter)(ctx->function, "i32");
        ctx->forget((void *)ctx->callback.userdata._0);
    }
    return R_NilValue;
}

// Call the retained callback after the creating scope has ended. No generated
// wrapper scope exists to own the result of its borrowed argument conversion.
SEXP test_callback_without_wrapper(
    SEXP function, SEXP transition, SEXP scope, SEXP converter,
    SEXP forget, SEXP release, SEXP to_object, SEXP object_free, SEXP runtime_type)
{
    DirectCallbackContext ctx = {
        .function = function,
        .transition = Rf_asLogical(transition),
        .converter = R_ExternalPtrAddr(converter),
        .forget = R_ExternalPtrAddr(forget)
    };
    SEXP (*with_scope)(SEXP (*)(void *), void *) = R_ExternalPtrAddr(scope);
    with_scope(retain_callback, &ctx);

    SEXP value = PROTECT(ScalarInteger(7));
    AnyObject *arg = ((AnyObject *(*)(SEXP, SEXP))R_ExternalPtrAddr(to_object))(value, runtime_type);
    UNPROTECT(1);

    const void *userdata = ctx.transition
        ? ctx.transition_callback.userdata._0 : ctx.callback.userdata._0;
    FfiResult_____AnyObject *result = ctx.transition
        ? ctx.transition_callback.callback(arg, false, userdata)
        : ctx.callback.callback(arg, userdata);
    int success = result && result->tag == Ok_____AnyObject;
    FfiResult_____c_void (*free_object)(AnyObject *) = R_ExternalPtrAddr(object_free);
    if (result)
    {
        if (success)
            free_object(result->ok);
        else
        {
            // The existing R callback stub allocates FfiError with C malloc.
            FfiError *err = result->err;
            free(err->variant);
            free(err->message);
            free(err->backtrace);
            free(err);
        }
        free(result);
    }
    free_object(arg);
    ((void (*)(void *))R_ExternalPtrAddr(release))((void *)userdata);
    return ScalarLogical(success);
}
