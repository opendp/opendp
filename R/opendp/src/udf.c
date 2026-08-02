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

static char *get_last_error_message(void)
{
    int error_occurred = 0;
    SEXP call = PROTECT(lang1(install("geterrmessage")));
    SEXP message = PROTECT(R_tryEval(call, R_GlobalEnv, &error_occurred));
    if (error_occurred)
    {
        UNPROTECT(2);
        return heap_strdup("unknown R error");
    }
    char *result = heap_strdup(sexp_to_charptr(message));
    UNPROTECT(2);
    return result;
}

static FfiResult_____AnyObject *wrap_success(AnyObject *obj)
{
    FfiResult_____AnyObject *result = (FfiResult_____AnyObject *)malloc(sizeof(FfiResult_____AnyObject));
    if (!result)
        error("failed to allocate callback result");
    result->tag = Ok_____AnyObject;
    result->ok = obj;
    return result;
}

static FfiResult_____AnyObject *wrap_failure_details(const char *message, const char *backtrace)
{
    FfiResult_____AnyObject *result = (FfiResult_____AnyObject *)malloc(sizeof(FfiResult_____AnyObject));
    FfiError *err = (FfiError *)malloc(sizeof(FfiError));
    if (!result || !err)
        error("failed to allocate callback error");
    err->variant = heap_strdup("FailedFunction");
    err->message = heap_strdup(message);
    err->backtrace = heap_strdup(backtrace);
    result->tag = Err_____AnyObject;
    result->err = err;
    return result;
}

static FfiResult_____AnyObject *wrap_failure(const char *message)
{
    return wrap_failure_details(message, "backtrace disabled");
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

static CallbackContext *new_callback_context(SEXP function, const char *type_name, int accepts_internal)
{
    CallbackContext *ctx = (CallbackContext *)malloc(sizeof(CallbackContext));
    if (!ctx)
        error("failed to allocate callback context");

    ctx->tag = TAG_CALLBACK_CONTEXT;
    ctx->ref_count = 1;
    ctx->accepts_internal = accepts_internal;
    ctx->function = function;
    ctx->type_name = heap_strdup(type_name);

    // The callback may outlive the current R call stack, so hold on to the
    // closure until Rust releases the userdata.
    R_PreserveObject(function);
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

typedef struct CompareContext {
    SEXP left;
    SEXP right;
    int cmp;
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

static FfiResult_____AnyObject *total_cmp_callback(const void *left_ptr, const void *right_ptr)
{
    RetainedSexp *left = (RetainedSexp *)left_ptr;
    RetainedSexp *right = (RetainedSexp *)right_ptr;

    if (!left || !right || left->tag != TAG_RETAINED_SEXP || right->tag != TAG_RETAINED_SEXP)
        return wrap_failure("extrinsic comparison expects retained R objects");

    CompareContext ctx = {.left = left->value, .right = right->value, .cmp = 0};
    if (!R_ToplevelExec(compare_exec, &ctx))
    {
        char *message = get_last_error_message();
        FfiResult_____AnyObject *result = wrap_failure_details(
            "Exception in user-defined comparison",
            message
        );
        free(message);
        return result;
    }

    SEXP cmp = PROTECT(ScalarInteger(ctx.cmp));
    SEXP type_name = PROTECT(mkString("i8"));
    AnyObject *result = sexp_to_anyobjectptr(cmp, type_name);
    UNPROTECT(2);
    return wrap_success(result);
}

typedef struct CallbackInvocation {
    CallbackContext *ctx;
    const AnyObject *arg;
    c_bool is_internal;
    FfiResult_____AnyObject *result;
} CallbackInvocation;

static void callback_exec(void *data)
{
    CallbackInvocation *invocation = (CallbackInvocation *)data;
    // Convert the Rust-owned argument into an R value, evaluate the closure,
    // then convert the result back using the callback's declared output type.
    SEXP arg = PROTECT(anyobjectptr_to_sexp((AnyObject *)invocation->arg));
    SEXP call = PROTECT(lang2(invocation->ctx->function, arg));
    int error_occurred = 0;
    SEXP out = PROTECT(R_tryEvalSilent(call, R_GlobalEnv, &error_occurred));
    if (error_occurred)
    {
        char *message = get_last_error_message();
        invocation->result = wrap_failure_details("Exception in user-defined function", message);
        free(message);
        UNPROTECT(3);
        return;
    }
    SEXP type_name = PROTECT(parse_runtime_type(invocation->ctx->type_name));
    AnyObject *result = sexp_to_anyobjectptr(out, type_name);
    invocation->result = wrap_success(result);
    UNPROTECT(4);
}

static void transition_exec(void *data)
{
    CallbackInvocation *invocation = (CallbackInvocation *)data;
    SEXP arg = PROTECT(anyobjectptr_to_sexp((AnyObject *)invocation->arg));
    SEXP call;
    // Transition callbacks optionally accept the Rust-side `is_internal` flag.
    // The current R API infers that shape from closure arity.
    if (invocation->ctx->accepts_internal)
    {
        SEXP is_internal = PROTECT(ScalarLogical((bool)invocation->is_internal));
        call = PROTECT(lang3(invocation->ctx->function, arg, is_internal));
        int error_occurred = 0;
        SEXP out = PROTECT(R_tryEvalSilent(call, R_GlobalEnv, &error_occurred));
        if (error_occurred)
        {
            char *message = get_last_error_message();
            invocation->result = wrap_failure_details("Exception in user-defined transition", message);
            free(message);
            UNPROTECT(4);
            return;
        }
        SEXP type_name = PROTECT(parse_runtime_type(invocation->ctx->type_name));
        AnyObject *result = sexp_to_anyobjectptr(out, type_name);
        invocation->result = wrap_success(result);
        UNPROTECT(5);
        return;
    }

    call = PROTECT(lang2(invocation->ctx->function, arg));
    int error_occurred = 0;
    SEXP out = PROTECT(R_tryEvalSilent(call, R_GlobalEnv, &error_occurred));
    if (error_occurred)
    {
        char *message = get_last_error_message();
        invocation->result = wrap_failure_details("Exception in user-defined transition", message);
        free(message);
        UNPROTECT(3);
        return;
    }
    SEXP type_name = PROTECT(parse_runtime_type(invocation->ctx->type_name));
    AnyObject *result = sexp_to_anyobjectptr(out, type_name);
    invocation->result = wrap_success(result);
    UNPROTECT(4);
}

static FfiResult_____AnyObject *callback_stub(const AnyObject *arg, const c_void *userdata)
{
    CallbackContext *ctx = (CallbackContext *)userdata;
    if (!ctx || ctx->tag != TAG_CALLBACK_CONTEXT)
        return wrap_failure("callback context is invalid");

    // R_ToplevelExec converts R exceptions into a failed return instead of
    // unwinding through the foreign stack.
    CallbackInvocation invocation = {
        .ctx = ctx,
        .arg = arg,
        .is_internal = false,
        .result = NULL
    };
    if (!R_ToplevelExec(callback_exec, &invocation))
    {
        char *message = get_last_error_message();
        FfiResult_____AnyObject *result = wrap_failure_details(
            "Exception in user-defined function",
            message
        );
        free(message);
        return result;
    }
    return invocation.result;
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
        .result = NULL
    };
    if (!R_ToplevelExec(transition_exec, &invocation))
    {
        char *message = get_last_error_message();
        FfiResult_____AnyObject *result = wrap_failure_details(
            "Exception in user-defined transition",
            message
        );
        free(message);
        return result;
    }
    return invocation.result;
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

void callbackfn_release(CallbackFn *function)
{
    if (!function)
        return;
    ref_count_callback(function->userdata._0, false);
}

void transitionfn_release(TransitionFn *transition)
{
    if (!transition)
        return;
    ref_count_callback(transition->userdata._0, false);
}

void init_udf_support(void)
{
    // Register process-global hooks used by Rust for cloned extrinsic objects
    // and callback userdata.
    _set_ref_count(ref_count_callback);
    _set_total_cmp(total_cmp_callback);
}
