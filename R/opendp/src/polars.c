#include <R.h>
#include <Rinternals.h>

#include <string.h>

#include "convert.h"
#include "convert_elements.h"
#include "opendp.h"

extern FfiResult_____FfiSlice opendp_data__roundtrip_r_polars_expr(
    const FfiSlice *raw);
extern FfiResult_____FfiSlice opendp_data__make_r_polars_dp_len(
    const double *scale, int is_signed, const char *lib);
extern FfiResult_____FfiSlice opendp_data__make_r_polars_dp_sum(
    const FfiSlice *raw, double lower, double upper, const double *scale,
    const char *lib);
extern FfiResult_____FfiSlice opendp_data__make_r_polars_dp_mean(
    const FfiSlice *raw, double lower, double upper, const double *scale,
    const char *lib);
extern FfiResult_____FfiSlice opendp_data__make_r_polars_dp_median(
    const FfiSlice *raw, const double *candidates, size_t candidates_len,
    const double *scale, const char *lib);
extern FfiResult_____FfiSlice opendp_data__make_r_polars_dp_quantile(
    const FfiSlice *raw, double alpha, const double *candidates,
    size_t candidates_len, const double *scale, const char *lib);
extern FfiResult_____FfiSlice opendp_data__make_r_polars_dp_noise(
    const FfiSlice *raw, const double *scale, const char *lib);
extern FfiResult_____FfiSlice opendp_data__make_r_polars_dp_count(
    const FfiSlice *raw, const double *scale, const char *lib);
extern FfiResult_____FfiSlice opendp_data__make_r_polars_dp_null_count(
    const FfiSlice *raw, const double *scale, const char *lib);
extern FfiResult_____FfiSlice opendp_data__make_r_polars_dp_n_unique(
    const FfiSlice *raw, const double *scale, const char *lib);
extern FfiResult_____FfiSlice opendp_data__onceframe_collect_r_polars(
    AnyObject *onceframe);
extern FfiResult_____AnyDomain opendp_domains__lazyframe_domain_from_slice(
    const FfiSlice *series_domains);
extern FfiResult_____AnyDomain opendp_domains__lazyframe_domain_with_margin_by(
    const AnyDomain *domain, const FfiSlice *by, const uint32_t *max_length,
    const uint32_t *max_groups, const char *invariant);
extern FfiResult_____AnyDomain opendp_domains__wild_expr_domain_from_slice(
    const FfiSlice *series_domains, const FfiSlice *by,
    const uint32_t *max_length, const uint32_t *max_groups,
    const char *invariant);
extern void opendp_data__r_polars_plugin_keepalive(void);

static SEXP ffi_slice_result_to_raw(FfiResult_____FfiSlice result)
{
    if (result.tag == Err_____FfiSlice)
        return extract_error(result.err);

    FfiSlice *output = result.ok;
    SEXP value = PROTECT(allocVector(RAWSXP, (R_xlen_t)output->len));
    memcpy(RAW(value), output->ptr, output->len);
    opendp_data__slice_free(output);
    UNPROTECT(1);
    return value;
}

/* Round-trip an r-polars Expr or LazyFrame through OpenDP (compatibility check). */
SEXP data__roundtrip_polars(SEXP data, SEXP type_name)
{
    if (TYPEOF(data) != RAWSXP)
        error("data must be a raw vector");
    if (TYPEOF(type_name) != STRSXP || XLENGTH(type_name) != 1)
        error("type_name must be a single string");

    const char *type = CHAR(STRING_ELT(type_name, 0));
    if (strcmp(type, "Expr") != 0 && strcmp(type, "LazyFrame") != 0)
        error("type_name must be either 'Expr' or 'LazyFrame'");

    FfiSlice input = {
        .ptr = RAW(data),
        .len = (size_t)XLENGTH(data),
    };
    if (strcmp(type, "Expr") == 0) {
        return ffi_slice_result_to_raw(
            opendp_data__roundtrip_r_polars_expr(&input));
    }

    FfiResult_____AnyObject object_result =
        opendp_data__slice_as_object(&input, "LazyFrame");
    if (object_result.tag == Err_____AnyObject)
        return extract_error(object_result.err);

    AnyObject *object = object_result.ok;
    FfiResult_____FfiSlice slice_result =
        opendp_data__object_as_slice(object);
    if (slice_result.tag == Err_____FfiSlice) {
        opendp_data__object_free(object);
        return extract_error(slice_result.err);
    }

    FfiSlice *output = slice_result.ok;
    SEXP result = PROTECT(allocVector(RAWSXP, (R_xlen_t)output->len));
    memcpy(RAW(result), output->ptr, output->len);

    opendp_data__slice_free(output);
    opendp_data__object_free(object);
    UNPROTECT(1);
    return result;
}

SEXP data__make_r_polars_dp_len(SEXP scale, SEXP is_signed, SEXP lib)
{
    double scale_value;
    const double *scale_ptr = NULL;
    if (!isNull(scale)) {
        scale_value = asReal(scale);
        scale_ptr = &scale_value;
    }
    const char *lib_path = CHAR(asChar(lib));
    return ffi_slice_result_to_raw(
        opendp_data__make_r_polars_dp_len(scale_ptr, asLogical(is_signed),
                                          lib_path));
}

SEXP data__make_r_polars_dp_sum(
    SEXP data, SEXP bounds, SEXP scale, SEXP lib)
{
    if (TYPEOF(data) != RAWSXP)
        error("data must be a raw vector");
    if (!isReal(bounds) || XLENGTH(bounds) != 2)
        error("bounds must be a numeric vector of length two");

    FfiSlice input = {
        .ptr = RAW(data),
        .len = (size_t)XLENGTH(data),
    };
    double scale_value;
    const double *scale_ptr = NULL;
    if (!isNull(scale)) {
        scale_value = asReal(scale);
        scale_ptr = &scale_value;
    }
    const char *lib_path = CHAR(asChar(lib));
    return ffi_slice_result_to_raw(opendp_data__make_r_polars_dp_sum(
        &input, REAL(bounds)[0], REAL(bounds)[1], scale_ptr, lib_path));
}

SEXP data__make_r_polars_dp_mean(
    SEXP data, SEXP bounds, SEXP scale, SEXP lib)
{
    if (TYPEOF(data) != RAWSXP)
        error("data must be a raw vector");
    if (!isReal(bounds) || XLENGTH(bounds) != 2)
        error("bounds must be a numeric vector of length two");

    FfiSlice input = {
        .ptr = RAW(data),
        .len = (size_t)XLENGTH(data),
    };
    double scale_value;
    const double *scale_ptr = NULL;
    if (!isNull(scale)) {
        scale_value = asReal(scale);
        scale_ptr = &scale_value;
    }
    const char *lib_path = CHAR(asChar(lib));
    return ffi_slice_result_to_raw(opendp_data__make_r_polars_dp_mean(
        &input, REAL(bounds)[0], REAL(bounds)[1], scale_ptr, lib_path));
}

SEXP data__make_r_polars_dp_median(
    SEXP data, SEXP candidates, SEXP scale, SEXP lib)
{
    if (TYPEOF(data) != RAWSXP)
        error("data must be a raw vector");
    if (!isReal(candidates) || XLENGTH(candidates) == 0)
        error("candidates must be a non-empty numeric vector");

    FfiSlice input = {
        .ptr = RAW(data),
        .len = (size_t)XLENGTH(data),
    };
    double scale_value;
    const double *scale_ptr = NULL;
    if (!isNull(scale)) {
        scale_value = asReal(scale);
        scale_ptr = &scale_value;
    }
    const char *lib_path = CHAR(asChar(lib));
    return ffi_slice_result_to_raw(opendp_data__make_r_polars_dp_median(
        &input, REAL(candidates), (size_t)XLENGTH(candidates), scale_ptr,
        lib_path));
}

SEXP data__make_r_polars_dp_quantile(
    SEXP data, SEXP alpha, SEXP candidates, SEXP scale, SEXP lib)
{
    if (TYPEOF(data) != RAWSXP)
        error("data must be a raw vector");
    if (!isReal(alpha) || XLENGTH(alpha) != 1)
        error("alpha must be a single number");
    if (!isReal(candidates) || XLENGTH(candidates) == 0)
        error("candidates must be a non-empty numeric vector");

    FfiSlice input = {
        .ptr = RAW(data),
        .len = (size_t)XLENGTH(data),
    };
    double scale_value;
    const double *scale_ptr = NULL;
    if (!isNull(scale)) {
        scale_value = asReal(scale);
        scale_ptr = &scale_value;
    }
    const char *lib_path = CHAR(asChar(lib));
    return ffi_slice_result_to_raw(opendp_data__make_r_polars_dp_quantile(
        &input, asReal(alpha), REAL(candidates), (size_t)XLENGTH(candidates),
        scale_ptr, lib_path));
}

/* Shared dispatcher for the (expr, scale) plugin constructors. */
typedef FfiResult_____FfiSlice (*unary_scale_fn)(
    const FfiSlice *raw, const double *scale, const char *lib);

static SEXP call_unary_scale_plugin(
    SEXP data, SEXP scale, SEXP lib, unary_scale_fn fn)
{
    if (TYPEOF(data) != RAWSXP)
        error("data must be a raw vector");

    FfiSlice input = {
        .ptr = RAW(data),
        .len = (size_t)XLENGTH(data),
    };
    double scale_value;
    const double *scale_ptr = NULL;
    if (!isNull(scale)) {
        scale_value = asReal(scale);
        scale_ptr = &scale_value;
    }
    const char *lib_path = CHAR(asChar(lib));
    return ffi_slice_result_to_raw(fn(&input, scale_ptr, lib_path));
}

SEXP data__make_r_polars_dp_noise(SEXP data, SEXP scale, SEXP lib)
{
    return call_unary_scale_plugin(
        data, scale, lib, opendp_data__make_r_polars_dp_noise);
}

SEXP data__make_r_polars_dp_count(SEXP data, SEXP scale, SEXP lib)
{
    return call_unary_scale_plugin(
        data, scale, lib, opendp_data__make_r_polars_dp_count);
}

SEXP data__make_r_polars_dp_null_count(SEXP data, SEXP scale, SEXP lib)
{
    return call_unary_scale_plugin(
        data, scale, lib, opendp_data__make_r_polars_dp_null_count);
}

SEXP data__make_r_polars_dp_n_unique(SEXP data, SEXP scale, SEXP lib)
{
    return call_unary_scale_plugin(
        data, scale, lib, opendp_data__make_r_polars_dp_n_unique);
}

SEXP measurements__make_private_r_polars_lazyframe(
    SEXP input_domain, SEXP input_metric, SEXP output_measure, SEXP data,
    SEXP global_scale, SEXP threshold, SEXP log)
{
    /*
     * Keep runtime-only rewritten-plugin ABI symbols in the package DLL when
     * Rust is linked as a static archive. This constructs no plugin node.
     */
    opendp_data__r_polars_plugin_keepalive();
    if (TYPEOF(data) != RAWSXP)
        error("lazyframe must serialize to a raw vector");

    FfiSlice input = {
        .ptr = RAW(data),
        .len = (size_t)XLENGTH(data),
    };
    FfiResult_____AnyObject object_result =
        opendp_data__slice_as_object(&input, "LazyFrame");
    if (object_result.tag == Err_____AnyObject)
        return extract_error(object_result.err);

    AnyObject *scale_object = NULL;
    AnyObject *threshold_object = NULL;
    if (!isNull(global_scale)) {
        SEXP type = PROTECT(parse_runtime_type("f64"));
        scale_object = sexp_to_anyobjectptr(global_scale, type);
        UNPROTECT(1);
    }
    if (!isNull(threshold)) {
        SEXP type = PROTECT(parse_runtime_type("u32"));
        threshold_object = sexp_to_anyobjectptr(threshold, type);
        UNPROTECT(1);
    }

    FfiResult_____AnyMeasurement result =
        opendp_measurements__make_private_lazyframe(
            sexp_to_anydomainptr(input_domain),
            sexp_to_anymetricptr(input_metric),
            sexp_to_anymeasureptr(output_measure),
            object_result.ok,
            scale_object,
            threshold_object);

    opendp_data__object_free(object_result.ok);
    if (scale_object != NULL)
        opendp_data__object_free(scale_object);
    if (threshold_object != NULL)
        opendp_data__object_free(threshold_object);

    if (result.tag == Err_____AnyMeasurement)
        return extract_error(result.err);
    return anymeasurementptr_to_sexp(result.ok, log);
}

SEXP core__invoke_r_polars_lazyframe(SEXP measurement, SEXP data)
{
    if (TYPEOF(data) != RAWSXP)
        error("private_data must serialize to a raw vector");

    FfiSlice input = {
        .ptr = RAW(data),
        .len = (size_t)XLENGTH(data),
    };
    FfiResult_____AnyObject object_result =
        opendp_data__slice_as_object(&input, "LazyFrame");
    if (object_result.tag == Err_____AnyObject)
        return extract_error(object_result.err);

    FfiResult_____AnyObject result = opendp_core__measurement_invoke(
        sexp_to_anymeasurementptr(measurement), object_result.ok);
    opendp_data__object_free(object_result.ok);
    if (result.tag == Err_____AnyObject)
        return extract_error(result.err);
    return anyqueryableptr_to_sexp(result.ok, R_NilValue);
}

SEXP data__onceframe_collect_r_polars(SEXP onceframe)
{
    return ffi_slice_result_to_raw(opendp_data__onceframe_collect_r_polars(
        sexp_to_anyqueryableptr(onceframe)));
}

SEXP domains__r_polars_series_domain(
    SEXP name, SEXP element_domain, SEXP log)
{
    FfiResult_____AnyDomain result = opendp_domains__series_domain(
        (char *)CHAR(asChar(name)), sexp_to_anydomainptr(element_domain));
    if (result.tag == Err_____AnyDomain)
        return extract_error(result.err);
    return anydomainptr_to_sexp(result.ok, log);
}

SEXP domains__r_polars_lazyframe_domain(SEXP series_domains, SEXP log)
{
    if (TYPEOF(series_domains) != VECSXP)
        error("series_domains must be a list");
    R_xlen_t len = XLENGTH(series_domains);
    AnyDomain **domains =
        (AnyDomain **)R_alloc((size_t)len, sizeof(AnyDomain *));
    for (R_xlen_t index = 0; index < len; index++)
        domains[index] =
            sexp_to_anydomainptr(VECTOR_ELT(series_domains, index));
    FfiSlice slice = {
        .ptr = domains,
        .len = (size_t)len,
    };
    FfiResult_____AnyDomain result =
        opendp_domains__lazyframe_domain_from_slice(&slice);
    if (result.tag == Err_____AnyDomain)
        return extract_error(result.err);
    return anydomainptr_to_sexp(result.ok, log);
}

SEXP domains__r_polars_with_margin(
    SEXP domain, SEXP by, SEXP max_length, SEXP max_groups, SEXP invariant,
    SEXP log)
{
    if (TYPEOF(by) != STRSXP)
        error("by must be a character vector");
    R_xlen_t len = XLENGTH(by);
    const char **names =
        (const char **)R_alloc((size_t)len, sizeof(const char *));
    for (R_xlen_t index = 0; index < len; index++)
        names[index] = CHAR(STRING_ELT(by, index));
    FfiSlice slice = {
        .ptr = (void *)names,
        .len = (size_t)len,
    };
    uint32_t max_length_value;
    uint32_t max_groups_value;
    const uint32_t *max_length_ptr = NULL;
    const uint32_t *max_groups_ptr = NULL;
    if (!isNull(max_length)) {
        max_length_value = (uint32_t)asReal(max_length);
        max_length_ptr = &max_length_value;
    }
    if (!isNull(max_groups)) {
        max_groups_value = (uint32_t)asReal(max_groups);
        max_groups_ptr = &max_groups_value;
    }
    const char *invariant_ptr = NULL;
    if (!isNull(invariant))
        invariant_ptr = CHAR(asChar(invariant));
    FfiResult_____AnyDomain result =
        opendp_domains__lazyframe_domain_with_margin_by(
            sexp_to_anydomainptr(domain), &slice, max_length_ptr,
            max_groups_ptr, invariant_ptr);
    if (result.tag == Err_____AnyDomain)
        return extract_error(result.err);
    return anydomainptr_to_sexp(result.ok, log);
}

SEXP domains__r_polars_wild_expr_domain(
    SEXP series_domains, SEXP by, SEXP max_length, SEXP max_groups,
    SEXP invariant, SEXP log)
{
    if (TYPEOF(series_domains) != VECSXP)
        error("series_domains must be a list");
    R_xlen_t len = XLENGTH(series_domains);
    AnyDomain **domains =
        (AnyDomain **)R_alloc((size_t)len, sizeof(AnyDomain *));
    for (R_xlen_t index = 0; index < len; index++)
        domains[index] =
            sexp_to_anydomainptr(VECTOR_ELT(series_domains, index));
    FfiSlice columns = {
        .ptr = domains,
        .len = (size_t)len,
    };

    /* NULL `by` selects a row-by-row domain; a character vector (possibly
     * empty) selects an aggregation margin. */
    FfiSlice by_slice;
    const FfiSlice *by_ptr = NULL;
    if (!isNull(by)) {
        if (TYPEOF(by) != STRSXP)
            error("by must be a character vector or NULL");
        R_xlen_t by_len = XLENGTH(by);
        const char **names =
            (const char **)R_alloc((size_t)by_len, sizeof(const char *));
        for (R_xlen_t index = 0; index < by_len; index++)
            names[index] = CHAR(STRING_ELT(by, index));
        by_slice.ptr = (void *)names;
        by_slice.len = (size_t)by_len;
        by_ptr = &by_slice;
    }

    uint32_t max_length_value;
    uint32_t max_groups_value;
    const uint32_t *max_length_ptr = NULL;
    const uint32_t *max_groups_ptr = NULL;
    if (!isNull(max_length)) {
        max_length_value = (uint32_t)asReal(max_length);
        max_length_ptr = &max_length_value;
    }
    if (!isNull(max_groups)) {
        max_groups_value = (uint32_t)asReal(max_groups);
        max_groups_ptr = &max_groups_value;
    }
    const char *invariant_ptr = NULL;
    if (!isNull(invariant))
        invariant_ptr = CHAR(asChar(invariant));

    FfiResult_____AnyDomain result =
        opendp_domains__wild_expr_domain_from_slice(
            &columns, by_ptr, max_length_ptr, max_groups_ptr, invariant_ptr);
    if (result.tag == Err_____AnyDomain)
        return extract_error(result.err);
    return anydomainptr_to_sexp(result.ok, log);
}
