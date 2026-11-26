#include <R.h>
#include <Rinternals.h>
#include <R_ext/Boolean.h>
#include <R_ext/Error.h>

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "convert.h"
#include "opendp.h"

typedef struct CallbackContext {
    int tag;
    int ref_count;
    int accepts_internal;
    SEXP function;
    char *type_name;
} CallbackContext;

struct RetainedSexp {
    int tag;
    int ref_count;
    SEXP value;
};

enum {
    TAG_RETAINED_SEXP = 1,
    TAG_CALLBACK_CONTEXT = 2
};

static char *heap_strdup(const char *value)
{
    size_t len = strlen(value) + 1;
    char *copy = (char *)malloc(len);
    if (!copy)
        error("failed to allocate string");
    memcpy(copy, value, len);
    return copy;
}

static FfiResult_____AnyObject *wrap_success(AnyObject *obj)
{
    return (FfiResult_____AnyObject *)ffiresult_ok(obj);
}

static FfiResult_____AnyObject *wrap_failure(const char *message)
{
    return (FfiResult_____AnyObject *)ffiresult_err("FFI", message, NULL);
}

typedef struct ErrorBoundary {
    SEXP (*exec)(void *);
    void *data;
    FfiResult_____AnyObject *failure;
} ErrorBoundary;

static SEXP boundary_body(void *data)
{
    ErrorBoundary *boundary = (ErrorBoundary *)data;
    return opendp_with_scope(boundary->exec, boundary->data);
}

static SEXP boundary_handler(SEXP condition, void *data)
{
    ErrorBoundary *boundary = (ErrorBoundary *)data;
    PROTECT(condition);
    SEXP details = PROTECT(get_private_func(".callback_error_details"));
    SEXP call = PROTECT(lang2(details, condition));
    SEXP values = PROTECT(Rf_eval(call, R_BaseEnv));
    SEXP message = VECTOR_ELT(values, 1);
    boundary->failure = (FfiResult_____AnyObject *)ffiresult_err(
        Rf_translateCharUTF8(STRING_ELT(VECTOR_ELT(values, 0), 0)),
        message == R_NilValue ? NULL : Rf_translateCharUTF8(STRING_ELT(message, 0)),
        NULL
    );
    UNPROTECT(4);
    return R_NilValue;
}

static void boundary_exec(void *data)
{
    R_tryCatchError(boundary_body, data, boundary_handler, data);
}

static FfiResult_____AnyObject *run_with_error_boundary(SEXP (*exec)(void *), void *data)
{
    ErrorBoundary boundary = {.exec = exec, .data = data, .failure = NULL};
    // Never unwind through Rust, even if converting the condition itself fails.
    if (!R_ToplevelExec(boundary_exec, &boundary))
        return wrap_failure("R callback error could not be converted");
    return boundary.failure;
}

static int count_formals(SEXP function)
{
    if (TYPEOF(function) != CLOSXP)
        return 1;

    int count = 0;
    for (SEXP node = R_ClosureFormals(function); node != R_NilValue; node = CDR(node))
        count++;
    return count;
}

static bool ref_count_callback(const void *ptr, bool increment);

static CallbackContext *new_callback_context(SEXP function, const char *type_name, int accepts_internal)
{
    CallbackContext *ctx = (CallbackContext *)malloc(sizeof(CallbackContext));
    if (!ctx)
        error("failed to allocate callback context");

    ctx->tag = TAG_CALLBACK_CONTEXT;
    ctx->ref_count = 1;
    ctx->accepts_internal = accepts_internal;
    ctx->function = function;
    ctx->type_name = NULL;

    // Register the preserved closure before allocating its type string:
    // R errors longjmp past ordinary C cleanup.
    R_PreserveObject(function);
    opendp_register_resource(ctx, opendp_release_ref_counted);
    ctx->type_name = heap_strdup(type_name);
    return ctx;
}

static void free_callback_context(CallbackContext *ctx)
{
    if (!ctx)
        return;
    R_ReleaseObject(ctx->function);
    free(ctx->type_name);
    free(ctx);
}

RetainedSexp *new_retained_sexp(SEXP value)
{
    RetainedSexp *retained = (RetainedSexp *)malloc(sizeof(RetainedSexp));
    if (!retained)
        error("failed to allocate extrinsic object");

    retained->tag = TAG_RETAINED_SEXP;
    retained->ref_count = 1;
    retained->value = value;
    // Extrinsic descriptors may be cloned and compared on the Rust side, so
    // keep the underlying R object alive until the shared refcount reaches 0.
    R_PreserveObject(value);
    return retained;
}

static void free_retained_sexp(RetainedSexp *retained)
{
    if (!retained)
        return;
    R_ReleaseObject(retained->value);
    free(retained);
}

SEXP extrinsic_object_to_sexp(const ExtrinsicObject *obj)
{
    RetainedSexp *retained = (RetainedSexp *)obj->_0;
    if (!retained || retained->tag != TAG_RETAINED_SEXP)
        error("expected an extrinsic object");
    return retained->value;
}

static bool ref_count_callback(const void *ptr, bool increment)
{
    if (!ptr)
        return true;

    // Rust shares one retain/release hook for both descriptor objects and
    // callback userdata, so dispatch on the leading tag.
    int *tag = (int *)ptr;
    if (*tag == TAG_RETAINED_SEXP)
    {
        RetainedSexp *retained = (RetainedSexp *)ptr;
        retained->ref_count += increment ? 1 : -1;
        if (retained->ref_count < 0)
            return false;
        if (retained->ref_count == 0)
            free_retained_sexp(retained);
        return true;
    }

    if (*tag == TAG_CALLBACK_CONTEXT)
    {
        CallbackContext *ctx = (CallbackContext *)ptr;
        ctx->ref_count += increment ? 1 : -1;
        if (ctx->ref_count < 0)
            return false;
        if (ctx->ref_count == 0)
            free_callback_context(ctx);
        return true;
    }

    return false;
}

void opendp_release_ref_counted(void *ptr)
{
    ref_count_callback(ptr, false);
}

static void extrinsic_object_cleanup(void *obj)
{
    opendp_data__extrinsic_object_free((ExtrinsicObject *)obj);
}

SEXP owned_extrinsic_object_to_sexp(ExtrinsicObject *obj)
{
    opendp_register_resource(obj, extrinsic_object_cleanup);
    return extrinsic_object_to_sexp(obj);
}

typedef struct CompareContext {
    SEXP left;
    SEXP right;
    int cmp;
    FfiResult_____AnyObject *result;
} CompareContext;

static void compare_exec(void *data)
{
    CompareContext *ctx = (CompareContext *)data;

    // Match the existing ExtrinsicObject contract: equality first, then try a
    // total order via < and >.
    SEXP identical_call = PROTECT(lang3(install("identical"), ctx->left, ctx->right));
    SEXP identical = PROTECT(Rf_eval(identical_call, R_GlobalEnv));
    if (LOGICAL(identical)[0])
    {
        ctx->cmp = 0;
        UNPROTECT(2);
        return;
    }
    UNPROTECT(2);

    SEXP less_call = PROTECT(lang3(install("<"), ctx->left, ctx->right));
    SEXP less = PROTECT(Rf_eval(less_call, R_GlobalEnv));
    if (TYPEOF(less) == LGLSXP && XLENGTH(less) == 1 && LOGICAL(less)[0] == TRUE)
    {
        ctx->cmp = -1;
        UNPROTECT(2);
        return;
    }
    UNPROTECT(2);

    SEXP greater_call = PROTECT(lang3(install(">"), ctx->left, ctx->right));
    SEXP greater = PROTECT(Rf_eval(greater_call, R_GlobalEnv));
    if (TYPEOF(greater) == LGLSXP && XLENGTH(greater) == 1 && LOGICAL(greater)[0] == TRUE)
    {
        ctx->cmp = 1;
        UNPROTECT(2);
        return;
    }
    UNPROTECT(2);

    error("objects are not comparable");
}

static SEXP compare_and_convert_exec(void *data)
{
    CompareContext *ctx = (CompareContext *)data;
    compare_exec(ctx);
    SEXP cmp = PROTECT(ScalarInteger(ctx->cmp));
    SEXP type_name = PROTECT(parse_runtime_type("i8"));
    ctx->result = wrap_success(sexp_to_anyobjectptr(cmp, type_name));
    UNPROTECT(2);
    return R_NilValue;
}

static FfiResult_____AnyObject *total_cmp_callback(const void *left_ptr, const void *right_ptr)
{
    RetainedSexp *left = (RetainedSexp *)left_ptr;
    RetainedSexp *right = (RetainedSexp *)right_ptr;

    if (!left || !right || left->tag != TAG_RETAINED_SEXP || right->tag != TAG_RETAINED_SEXP)
        return wrap_failure("extrinsic comparison expects retained R objects");

    CompareContext ctx = {.left = left->value, .right = right->value, .cmp = 0, .result = NULL};
    FfiResult_____AnyObject *failure = run_with_error_boundary(compare_and_convert_exec, &ctx);
    return failure ? failure : ctx.result;
}

typedef struct CallbackInvocation {
    CallbackContext *ctx;
    const AnyObject *arg;
    c_bool is_internal;
    bool is_transition;
    FfiResult_____AnyObject *result;
} CallbackInvocation;

static SEXP callback_exec(void *data)
{
    CallbackInvocation *invocation = (CallbackInvocation *)data;
    // Convert the borrowed Rust argument, evaluate the R closure, then
    // transfer the result back to Rust.
    SEXP arg = PROTECT(borrowed_anyobjectptr_to_sexp((AnyObject *)invocation->arg));
    bool accepts_internal = invocation->is_transition && invocation->ctx->accepts_internal;
    SEXP call;
    if (accepts_internal)
    {
        SEXP is_internal = PROTECT(ScalarLogical((bool)invocation->is_internal));
        call = PROTECT(lang3(invocation->ctx->function, arg, is_internal));
    }
    else
        call = PROTECT(lang2(invocation->ctx->function, arg));

    SEXP out = PROTECT(Rf_eval(call, R_GlobalEnv));
    SEXP type_name = PROTECT(parse_runtime_type(invocation->ctx->type_name));
    AnyObject *result = sexp_to_anyobjectptr(out, type_name);
    invocation->result = wrap_success(result);
    UNPROTECT(accepts_internal ? 5 : 4);
    return R_NilValue;
}

static FfiResult_____AnyObject *callback_stub(const AnyObject *arg, const c_void *userdata)
{
    CallbackContext *ctx = (CallbackContext *)userdata;
    if (!ctx || ctx->tag != TAG_CALLBACK_CONTEXT)
        return wrap_failure("callback context is invalid");

    // R_ToplevelExec catches R errors before they can unwind through Rust.
    CallbackInvocation invocation = {
        .ctx = ctx,
        .arg = arg,
        .is_internal = false,
        .is_transition = false,
        .result = NULL
    };
    FfiResult_____AnyObject *failure = run_with_error_boundary(callback_exec, &invocation);
    return failure ? failure : invocation.result;
}

static FfiResult_____AnyObject *transition_stub(
    const AnyObject *arg,
    c_bool is_internal,
    const c_void *userdata)
{
    CallbackContext *ctx = (CallbackContext *)userdata;
    if (!ctx || ctx->tag != TAG_CALLBACK_CONTEXT)
        return wrap_failure("transition context is invalid");

    CallbackInvocation invocation = {
        .ctx = ctx,
        .arg = arg,
        .is_internal = is_internal,
        .is_transition = true,
        .result = NULL
    };
    FfiResult_____AnyObject *failure = run_with_error_boundary(callback_exec, &invocation);
    return failure ? failure : invocation.result;
}

CallbackFn sexp_to_callbackfn(SEXP function, const char *type_name)
{
    CallbackContext *ctx = new_callback_context(function, type_name, false);
    CallbackFn wrapped = {
        .callback = callback_stub,
        .userdata = {(const void *)ctx}
    };
    return wrapped;
}

TransitionFn sexp_to_transitionfn(SEXP function, const char *type_name)
{
    // Transition closures may optionally accept a second `is_internal`
    // argument. Preserve that fact in userdata so the stub can adapt calls.
    CallbackContext *ctx = new_callback_context(function, type_name, count_formals(function) > 1);
    TransitionFn wrapped = {
        .callback = transition_stub,
        .userdata = {(const void *)ctx}
    };
    return wrapped;
}

void init_udf_support(void)
{
    // Register process-global hooks used by Rust for cloned extrinsic objects
    // and callback userdata.
    _set_ref_count(ref_count_callback);
    _set_total_cmp(total_cmp_callback);
}
