#include <R.h>
#include <Rinternals.h>
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
    void *constructor;
    bool (*free_error)(FfiError *);
    void *callable;
} DirectCallbackContext;

static SEXP retain_callback(void *data)
{
    DirectCallbackContext *ctx = (DirectCallbackContext *)data;
    if (ctx->transition)
    {
        TransitionFn callback = ((TransitionFn (*)(SEXP, const char *))ctx->converter)(ctx->function, "i32");
        FfiResult_____AnyObject result = ((FfiResult_____AnyObject (*)(const TransitionFn *, const char *, const char *))ctx->constructor)(&callback, "i32", "i32");
        if (result.tag != Ok_____AnyObject)
        {
            ctx->free_error(result.err);
            Rf_error("failed to construct test queryable");
        }
        ctx->callable = result.ok;
    }
    else
    {
        CallbackFn callback = ((CallbackFn (*)(SEXP, const char *))ctx->converter)(ctx->function, "i32");
        FfiResult_____AnyFunction result = ((FfiResult_____AnyFunction (*)(const CallbackFn *, const char *))ctx->constructor)(&callback, "i32");
        if (result.tag != Ok_____AnyFunction)
        {
            ctx->free_error(result.err);
            Rf_error("failed to construct test function");
        }
        ctx->callable = result.ok;
    }
    return R_NilValue;
}

// Rust retains the callback and consumes its result. Evaluate after the
// creating scope has ended, with no generated R wrapper scope.
SEXP test_callback_without_wrapper(
    SEXP function, SEXP transition, SEXP scope, SEXP converter,
    SEXP constructor, SEXP evaluate, SEXP callable_free, SEXP to_object,
    SEXP object_free, SEXP error_free, SEXP runtime_type)
{
    DirectCallbackContext ctx = {
        .function = function,
        .transition = Rf_asLogical(transition),
        .converter = R_ExternalPtrAddr(converter),
        .constructor = R_ExternalPtrAddr(constructor),
        .free_error = R_ExternalPtrAddr(error_free)
    };
    SEXP (*with_scope)(SEXP (*)(void *), void *) = R_ExternalPtrAddr(scope);
    with_scope(retain_callback, &ctx);

    SEXP value = PROTECT(ScalarInteger(7));
    AnyObject *arg = ((AnyObject *(*)(SEXP, SEXP))R_ExternalPtrAddr(to_object))(value, runtime_type);
    UNPROTECT(1);

    FfiResult_____AnyObject result = ctx.transition
        ? ((FfiResult_____AnyObject (*)(AnyObject *, const AnyObject *))R_ExternalPtrAddr(evaluate))(ctx.callable, arg)
        : ((FfiResult_____AnyObject (*)(const AnyFunction *, const AnyObject *, const char *))R_ExternalPtrAddr(evaluate))(ctx.callable, arg, "i32");
    int success = result.tag == Ok_____AnyObject;
    FfiResult_____c_void (*free_object)(AnyObject *) = R_ExternalPtrAddr(object_free);
    if (success)
        free_object(result.ok);
    else
        ctx.free_error(result.err);

    free_object(arg);
    if (ctx.transition)
        free_object(ctx.callable);
    else
        ((FfiResult_____c_void (*)(AnyFunction *))R_ExternalPtrAddr(callable_free))(ctx.callable);
    return ScalarLogical(success);
}
