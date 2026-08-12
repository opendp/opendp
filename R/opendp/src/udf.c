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

static bool ref_count_callback(const void *ptr, bool increment);

static FfiResult_____AnyObject *wrap_success(AnyObject *obj)
{
    return (FfiResult_____AnyObject *)ffiresult_ok(obj);
}

static FfiResult_____AnyObject *wrap_failure_details(
    const char *variant,
    const char *message,
    const char *backtrace,
    const ExtrinsicObject *callback
)
{
    // Rust owns validation and allocation of callback errors.
    return (FfiResult_____AnyObject *)ffiresult_err(variant, message, backtrace, callback);
}

static FfiResult_____AnyObject *wrap_failure(const char *message)
{
    return wrap_failure_details("FFI", message, "backtrace disabled", NULL);
}

static SEXP condition_field(SEXP condition, const char *name)
{
    if (TYPEOF(condition) != VECSXP)
        return R_NilValue;
    SEXP names = getAttrib(condition, R_NamesSymbol);
    if (TYPEOF(names) != STRSXP || XLENGTH(names) != XLENGTH(condition))
        return R_NilValue;
    for (R_xlen_t i = 0; i < XLENGTH(names); i++)
        if (STRING_ELT(names, i) != NA_STRING && strcmp(CHAR(STRING_ELT(names, i)), name) == 0)
            return VECTOR_ELT(condition, i);
    return R_NilValue;
}

static const char *scalar_string(SEXP value)
{
    if (TYPEOF(value) != STRSXP || XLENGTH(value) != 1 || STRING_ELT(value, 0) == NA_STRING)
        return NULL;
    return CHAR(STRING_ELT(value, 0));
}

typedef struct ErrorBoundary {
    void (*exec)(void *);
    void *data;
    FfiResult_____AnyObject *failure;
} ErrorBoundary;

static SEXP boundary_body(void *data)
{
    ErrorBoundary *boundary = (ErrorBoundary *)data;
    boundary->exec(boundary->data);
    return R_NilValue;
}

static SEXP boundary_handler(SEXP condition, void *data)
{
    ErrorBoundary *boundary = (ErrorBoundary *)data;
    // Root the actual condition until Rust has copied its diagnostic fields.
    PROTECT(condition);
    SEXP call = PROTECT(lang2(install("conditionMessage"), condition));
    SEXP message = PROTECT(Rf_eval(call, R_BaseEnv));
    const char *variant = NULL;
    const char *raw_message = NULL;
    const char *backtrace = NULL;
    if (inherits(condition, "opendp_error"))
    {
        variant = scalar_string(condition_field(condition, "variant"));
        raw_message = scalar_string(condition_field(condition, "raw_message"));
        backtrace = scalar_string(condition_field(condition, "backtrace"));
    }
    const char *message_text = scalar_string(message);
    // Never infer provenance from text. Rust validates unknown variant names.
    // Finish all R operations before acquiring the temporary retained reference.
    ExtrinsicObject callback = {(const void *)new_retained_sexp(condition)};
    boundary->failure = wrap_failure_details(
        variant ? variant : "FFI",
        variant && raw_message ? raw_message : (message_text ? message_text : ""),
        variant && backtrace ? backtrace : "backtrace disabled",
        &callback
    );
    ref_count_callback(callback._0, false);
    UNPROTECT(3);
    return R_NilValue;
}

static void boundary_exec(void *data)
{
    R_tryCatchError(boundary_body, data, boundary_handler, data);
}

static FfiResult_____AnyObject *run_with_error_boundary(void (*exec)(void *), void *data)
{
    ErrorBoundary boundary = {.exec = exec, .data = data, .failure = NULL};
    // Must run on R's owning thread, like all R API calls and retained objects.
    // Keep the top-level guard even if catching or converting a condition fails.
    if (!R_ToplevelExec(boundary_exec, &boundary))
        // Do not call R again outside the guard to format a secondary failure.
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

static CallbackContext *new_callback_context(SEXP function, const char *type_name, int accepts_internal)
{
    // Preserve before acquiring C resources: R_PreserveObject may jump.
    R_PreserveObject(function);
    CallbackContext *ctx = (CallbackContext *)malloc(sizeof(CallbackContext));
    if (ctx)
    {
        ctx->type_name = (char *)malloc(strlen(type_name) + 1);
        if (ctx->type_name)
        {
            strcpy(ctx->type_name, type_name);
            ctx->tag = TAG_CALLBACK_CONTEXT;
            ctx->ref_count = 1;
            ctx->accepts_internal = accepts_internal;
            ctx->function = function;
            return ctx;
        }
        free(ctx);
    }
    R_ReleaseObject(function);
    error("failed to allocate callback context");
    return NULL;
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
    // R_PreserveObject may jump; do not acquire C resources until it succeeds.
    R_PreserveObject(value);
    RetainedSexp *retained = (RetainedSexp *)malloc(sizeof(RetainedSexp));
    if (!retained)
    {
        R_ReleaseObject(value);
        error("failed to allocate extrinsic object");
    }
    retained->tag = TAG_RETAINED_SEXP;
    retained->ref_count = 1;
    retained->value = value;
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

static void compare_and_convert_exec(void *data)
{
    CompareContext *ctx = (CompareContext *)data;
    compare_exec(ctx);
    SEXP cmp = PROTECT(ScalarInteger(ctx->cmp));
    SEXP type_name = PROTECT(parse_runtime_type("i8"));
    ctx->result = wrap_success(sexp_to_anyobjectptr(cmp, type_name));
    UNPROTECT(2);
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
    FfiResult_____AnyObject *result;
} CallbackInvocation;

static void callback_exec(void *data)
{
    CallbackInvocation *invocation = (CallbackInvocation *)data;
    // Convert the Rust-owned argument into an R value, evaluate the closure,
    // then convert the result back using the callback's declared output type.
    SEXP arg = PROTECT(anyobjectptr_to_sexp((AnyObject *)invocation->arg));
    SEXP call = PROTECT(lang2(invocation->ctx->function, arg));
    SEXP out = PROTECT(Rf_eval(call, R_GlobalEnv));
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
        SEXP out = PROTECT(Rf_eval(call, R_GlobalEnv));
        SEXP type_name = PROTECT(parse_runtime_type(invocation->ctx->type_name));
        AnyObject *result = sexp_to_anyobjectptr(out, type_name);
        invocation->result = wrap_success(result);
        UNPROTECT(5);
        return;
    }

    call = PROTECT(lang2(invocation->ctx->function, arg));
    SEXP out = PROTECT(Rf_eval(call, R_GlobalEnv));
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
    FfiResult_____AnyObject *failure = run_with_error_boundary(callback_exec, &invocation);
    if (failure)
        return failure;
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
    FfiResult_____AnyObject *failure = run_with_error_boundary(transition_exec, &invocation);
    if (failure)
        return failure;
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
