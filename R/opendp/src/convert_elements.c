#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>
#include <R_ext/Callbacks.h>

// Import C headers for rust API
#include "Ropendp.h"
#include "opendp.h"
#include "convert.h"

// TRANSFORMATION
#define Check_AnyTransformation_Ptr(s)                                    \
    do                                                                    \
    {                                                                     \
        if (TYPEOF(s) != EXTPTRSXP ||                                     \
            R_ExternalPtrTag(s) != AnyTransformation_tag)                 \
            error("expected a transformation, but got a different type"); \
        if (!R_ExternalPtrAddr(s))                                        \
            error("Got null pointer. Reusing a transformation from a previous session is not supported."); \
    } while (0)

AnyTransformation *sexp_to_anytransformationptr(SEXP value)
{
    PROTECT(value);

    int errorOccurred;
    SEXP class_expr = lang2(install("class"), value);
    SEXP class = R_tryEval(class_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("could not determine class");

    if (str_equal(sexp_to_charptr(class), "transformation"))
    {
        SEXP call = PROTECT(Rf_lang2(value, PROTECT(Rf_mkString("ptr"))));
        value = Rf_eval(call, R_GlobalEnv);
        UNPROTECT(2);
    }
    Check_AnyTransformation_Ptr(value);
    UNPROTECT(1);
    return (AnyTransformation *)R_ExternalPtrAddr(value);
}

void odp_AnyTransformation_finalizer(SEXP XPtr)
{
    if (NULL == R_ExternalPtrAddr(XPtr))
        return;
    Check_AnyTransformation_Ptr(XPtr);
    AnyTransformation *ptr = (AnyTransformation *)R_ExternalPtrAddr(XPtr);
    opendp_core___transformation_free(ptr);
    R_ClearExternalPtr(XPtr);
}

SEXP anytransformationptr_to_sexp(AnyTransformation *input, SEXP log)
{
    SEXP XPtr = PROTECT(R_MakeExternalPtr(input, AnyTransformation_tag, R_NilValue));
    R_RegisterCFinalizerEx(XPtr, odp_AnyTransformation_finalizer, TRUE);

    int errorOccurred;
    SEXP new_transformation = PROTECT(get_private_func("new_transformation"));
    SEXP transformation_expr = lang3(new_transformation, XPtr, log);
    SEXP transformation = R_tryEval(transformation_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("failed to construct transformation");

    UNPROTECT(2);
    return transformation;
}

// MEASUREMENT
#define Check_AnyMeasurement_Ptr(s)                                    \
    do                                                                 \
    {                                                                  \
        if (TYPEOF(s) != EXTPTRSXP ||                                  \
            R_ExternalPtrTag(s) != AnyMeasurement_tag)                 \
            error("expected a measurement, but got a different type"); \
        if (!R_ExternalPtrAddr(s))                                     \
            error("Got null pointer. Reusing a measurement from a previous session is not supported."); \
    } while (0)

AnyMeasurement *sexp_to_anymeasurementptr(SEXP value)
{
    PROTECT(value);

    int errorOccurred;
    SEXP class_expr = lang2(install("class"), value);
    SEXP class = R_tryEval(class_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("could not determine class");

    if (str_equal(sexp_to_charptr(class), "measurement"))
    {
        SEXP call = PROTECT(Rf_lang2(value, PROTECT(Rf_mkString("ptr"))));
        value = Rf_eval(call, R_GlobalEnv);
        UNPROTECT(2);
    }
    Check_AnyMeasurement_Ptr(value);
    UNPROTECT(1);
    return (AnyMeasurement *)R_ExternalPtrAddr(value);
}

void odp_AnyMeasurement_finalizer(SEXP XPtr)
{
    if (NULL == R_ExternalPtrAddr(XPtr))
        return;
    Check_AnyMeasurement_Ptr(XPtr);
    AnyMeasurement *ptr = (AnyMeasurement *)R_ExternalPtrAddr(XPtr);
    opendp_core___measurement_free(ptr);
    R_ClearExternalPtr(XPtr);
}

SEXP anymeasurementptr_to_sexp(AnyMeasurement *input, SEXP log)
{
    SEXP XPtr = PROTECT(R_MakeExternalPtr(input, AnyMeasurement_tag, R_NilValue));
    R_RegisterCFinalizerEx(XPtr, odp_AnyMeasurement_finalizer, TRUE);

    int errorOccurred;
    SEXP new_measurement = PROTECT(get_private_func("new_measurement"));
    SEXP measurement_expr = lang3(new_measurement, XPtr, log);
    SEXP measurement = R_tryEval(measurement_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("failed to construct measurement");

    UNPROTECT(2);
    return measurement;
}

// DOMAIN
#define Check_AnyDomain_Ptr(s)                                    \
    do                                                            \
    {                                                             \
        if (TYPEOF(s) != EXTPTRSXP ||                             \
            R_ExternalPtrTag(s) != AnyDomain_tag)                 \
            error("expected a domain, but got a different type"); \
        if (!R_ExternalPtrAddr(s))                                \
            error("Got null pointer. Reusing a domain from a previous session is not supported."); \
    } while (0)

AnyDomain *sexp_to_anydomainptr(SEXP value)
{
    PROTECT(value);

    int errorOccurred;
    SEXP class_expr = lang2(install("class"), value);
    SEXP class = R_tryEval(class_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("could not determine class");

    if (str_equal(sexp_to_charptr(class), "domain"))
    {
        SEXP call = PROTECT(Rf_lang2(value, PROTECT(Rf_mkString("ptr"))));
        value = Rf_eval(call, R_GlobalEnv);
        UNPROTECT(2);
    }
    Check_AnyDomain_Ptr(value);
    UNPROTECT(1);
    return (AnyDomain *)R_ExternalPtrAddr(value);
}

void odp_AnyDomain_finalizer(SEXP XPtr)
{
    if (NULL == R_ExternalPtrAddr(XPtr))
        return;
    Check_AnyDomain_Ptr(XPtr);
    AnyDomain *ptr = (AnyDomain *)R_ExternalPtrAddr(XPtr);
    opendp_domains___domain_free(ptr);
    R_ClearExternalPtr(XPtr);
}

SEXP anydomainptr_to_sexp(AnyDomain *input, SEXP log)
{
    SEXP XPtr = PROTECT(R_MakeExternalPtr(input, AnyDomain_tag, R_NilValue));
    R_RegisterCFinalizerEx(XPtr, odp_AnyDomain_finalizer, TRUE);

    int errorOccurred;
    SEXP new_domain = get_private_func("new_domain");
    SEXP domain_expr = lang3(new_domain, XPtr, log);
    SEXP domain = R_tryEval(domain_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("failed to construct domain");

    UNPROTECT(1);
    return domain;
}

// METRIC
#define Check_AnyMetric_Ptr(s)                                    \
    do                                                            \
    {                                                             \
        if (TYPEOF(s) != EXTPTRSXP ||                             \
            R_ExternalPtrTag(s) != AnyMetric_tag)                 \
            error("expected a metric, but got a different type"); \
        if (!R_ExternalPtrAddr(s))                                \
            error("Got null pointer. Reusing a metric from a previous session is not supported."); \
    } while (0)

AnyMetric *sexp_to_anymetricptr(SEXP value)
{
    PROTECT(value);

    int errorOccurred;
    SEXP class_expr = lang2(install("class"), value);
    SEXP class = R_tryEval(class_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("could not determine class");

    if (str_equal(sexp_to_charptr(class), "metric"))
    {
        SEXP call = PROTECT(Rf_lang2(value, PROTECT(Rf_mkString("ptr"))));
        value = Rf_eval(call, R_GlobalEnv);
        UNPROTECT(2);
    }
    Check_AnyMetric_Ptr(value);

    UNPROTECT(1);
    return (AnyMetric *)R_ExternalPtrAddr(value);
}

void odp_AnyMetric_finalizer(SEXP XPtr)
{
    if (NULL == R_ExternalPtrAddr(XPtr))
        return;
    Check_AnyMetric_Ptr(XPtr);
    AnyMetric *ptr = (AnyMetric *)R_ExternalPtrAddr(XPtr);
    opendp_metrics___metric_free(ptr);
    R_ClearExternalPtr(XPtr);
}

SEXP anymetricptr_to_sexp(AnyMetric *input, SEXP log)
{
    SEXP XPtr = PROTECT(R_MakeExternalPtr(input, AnyMetric_tag, R_NilValue));
    R_RegisterCFinalizerEx(XPtr, odp_AnyMetric_finalizer, TRUE);

    int errorOccurred;
    SEXP new_metric = get_private_func("new_metric");
    SEXP metric_expr = lang3(new_metric, XPtr, log);
    SEXP metric = R_tryEval(metric_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("failed to construct metric");

    UNPROTECT(1);
    return metric;
}

// MEASURE
#define Check_AnyMeasure_Ptr(s)                                    \
    do                                                             \
    {                                                              \
        if (TYPEOF(s) != EXTPTRSXP ||                              \
            R_ExternalPtrTag(s) != AnyMeasure_tag)                 \
            error("expected a measure, but got a different type"); \
        if (!R_ExternalPtrAddr(s))                                 \
            error("Got null pointer. Reusing a measure from a previous session is not supported."); \
    } while (0)

AnyMeasure *sexp_to_anymeasureptr(SEXP value)
{
    PROTECT(value);

    int errorOccurred;
    SEXP class_expr = lang2(install("class"), value);
    SEXP class = R_tryEval(class_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("could not determine class");

    if (str_equal(sexp_to_charptr(class), "measure"))
    {
        SEXP call = PROTECT(Rf_lang2(value, PROTECT(Rf_mkString("ptr"))));
        value = Rf_eval(call, R_GlobalEnv);
        UNPROTECT(2);
    }
    Check_AnyMeasure_Ptr(value);

    UNPROTECT(1);
    return (AnyMeasure *)R_ExternalPtrAddr(value);
}

void odp_AnyMeasure_finalizer(SEXP XPtr)
{
    if (NULL == R_ExternalPtrAddr(XPtr))
        return;
    Check_AnyMeasure_Ptr(XPtr);
    AnyMeasure *ptr = (AnyMeasure *)R_ExternalPtrAddr(XPtr);
    opendp_measures___measure_free(ptr);
    R_ClearExternalPtr(XPtr);
}

SEXP anymeasureptr_to_sexp(AnyMeasure *input, SEXP log)
{
    SEXP XPtr = PROTECT(R_MakeExternalPtr(input, AnyMeasure_tag, R_NilValue));
    R_RegisterCFinalizerEx(XPtr, odp_AnyMeasure_finalizer, TRUE);

    int errorOccurred;
    SEXP new_measure = get_private_func("new_measure");
    SEXP measure_expr = lang3(new_measure, XPtr, log);
    SEXP measure = R_tryEval(measure_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("failed to construct measure");

    UNPROTECT(1);
    return measure;
}

// FUNCTION
#define Check_AnyFunction_Ptr(s)                                    \
    do                                                              \
    {                                                               \
        if (TYPEOF(s) != EXTPTRSXP ||                               \
            R_ExternalPtrTag(s) != AnyFunction_tag)                 \
            error("expected a function, but got a different type"); \
        if (!R_ExternalPtrAddr(s))                                  \
            error("Got null pointer. Reusing an OpenDP function from a previous session is not supported."); \
    } while (0)

AnyFunction *sexp_to_anyfunctionptr(SEXP value)
{
    PROTECT(value);

    int errorOccurred;
    SEXP class_expr = lang2(install("class"), value);
    SEXP class = R_tryEval(class_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("could not determine class");

    if (str_equal(sexp_to_charptr(class), "opendp_function"))
    {
        SEXP call = PROTECT(Rf_lang2(value, PROTECT(Rf_mkString("ptr"))));
        value = Rf_eval(call, R_GlobalEnv);
        UNPROTECT(2);
    }
    Check_AnyFunction_Ptr(value);

    UNPROTECT(1);
    return (AnyFunction *)R_ExternalPtrAddr(value);
}

void odp_AnyFunction_finalizer(SEXP XPtr)
{
    if (NULL == R_ExternalPtrAddr(XPtr))
        return;
    Check_AnyFunction_Ptr(XPtr);
    AnyFunction *ptr = (AnyFunction *)R_ExternalPtrAddr(XPtr);
    opendp_core___function_free(ptr);
    R_ClearExternalPtr(XPtr);
}

SEXP anyfunctionptr_to_sexp(AnyFunction *input, SEXP log)
{
    SEXP XPtr = PROTECT(R_MakeExternalPtr(input, AnyFunction_tag, R_NilValue));
    R_RegisterCFinalizerEx(XPtr, odp_AnyFunction_finalizer, TRUE);

    int errorOccurred;
    SEXP new_function = get_private_func("new_function");
    SEXP function_expr = lang3(new_function, XPtr, log);
    SEXP function = R_tryEval(function_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("failed to construct function");

    UNPROTECT(1);
    return function;
}

// AnyObject
#define Check_AnyObject_Ptr(s)                                        \
    do                                                                \
    {                                                                 \
        if (TYPEOF(s) != EXTPTRSXP ||                                 \
            R_ExternalPtrTag(s) != AnyObject_tag)                     \
            error("expected an AnyObject, but got a different type"); \
        if (!R_ExternalPtrAddr(s))                                    \
            error("Got null pointer. Reusing an AnyObject from a previous session is not supported."); \
    } while (0)

void odp_AnyObject_finalizer(SEXP XPtr)
{
    if (NULL == R_ExternalPtrAddr(XPtr))
        return;
    Check_AnyObject_Ptr(XPtr);
    AnyObject *ptr = (AnyObject *)R_ExternalPtrAddr(XPtr);
    opendp_data__object_free(ptr);
    R_ClearExternalPtr(XPtr);
}

// AnyObject: PrivacyProfile
AnyObject *sexp_to_privacyprofileptr(SEXP value)
{
    PROTECT(value);

    int errorOccurred;
    SEXP class_expr = lang2(install("class"), value);
    SEXP class = R_tryEval(class_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("could not determine class");

    if (str_equal(sexp_to_charptr(class), "privacy_profile"))
    {
        SEXP call = PROTECT(Rf_lang2(value, PROTECT(Rf_mkString("ptr"))));
        value = Rf_eval(call, R_GlobalEnv);
        UNPROTECT(2);
    }
    Check_AnyObject_Ptr(value);

    UNPROTECT(1);
    return (AnyObject *)R_ExternalPtrAddr(value);
}

SEXP privacyprofileptr_to_sexp(AnyObject *input, SEXP info)
{
    SEXP XPtr = PROTECT(R_MakeExternalPtr(input, AnyObject_tag, info));
    R_RegisterCFinalizerEx(XPtr, odp_AnyObject_finalizer, TRUE);

    int errorOccurred;
    SEXP new_privacy_profile = get_private_func("new_privacy_profile");
    SEXP privacy_profile_expr = lang2(new_privacy_profile, XPtr);
    SEXP privacy_profile = R_tryEval(privacy_profile_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("failed to construct privacy profile");

    UNPROTECT(1);
    return privacy_profile;
}

// AnyObject: Queryable
AnyObject *sexp_to_anyqueryableptr(SEXP value)
{
    PROTECT(value);

    int errorOccurred;
    SEXP class_expr = lang2(install("class"), value);
    SEXP class = R_tryEval(class_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("could not determine class");

    if (str_equal(sexp_to_charptr(class), "queryable"))
    {
        SEXP call = PROTECT(Rf_lang2(value, PROTECT(Rf_mkString("ptr"))));
        value = Rf_eval(call, R_GlobalEnv);
        UNPROTECT(2);
    }
    Check_AnyObject_Ptr(value);

    UNPROTECT(1);
    return (AnyObject *)R_ExternalPtrAddr(value);
}

SEXP anyqueryableptr_to_sexp(AnyObject *input, SEXP info)
{
    SEXP XPtr = PROTECT(R_MakeExternalPtr(input, AnyObject_tag, info));
    R_RegisterCFinalizerEx(XPtr, odp_AnyObject_finalizer, TRUE);

    int errorOccurred;
    SEXP new_queryable = get_private_func("new_queryable");
    SEXP queryable_expr = lang2(new_queryable, XPtr);
    SEXP queryable = R_tryEval(queryable_expr, R_GlobalEnv, &errorOccurred);
    if (errorOccurred)
        error("failed to construct queryable");

    UNPROTECT(1);
    return queryable;
}
