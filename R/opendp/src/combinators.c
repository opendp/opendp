// Auto-generated. Do not edit.
#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>

#include "convert.h"
#include "convert_elements.h"
#include "Ropendp.h"
#include "opendp.h"
#include "opendp_extras.h"


SEXP combinators__make_approximate(
    SEXP measurement, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurement);
    PROTECT(log);

    AnyMeasurement * c_measurement = sexp_to_anymeasurementptr(measurement);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_approximate(c_measurement);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP combinators__make_basic_composition(
    SEXP measurements, SEXP T_measurements, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurements);
    PROTECT(T_measurements);
    PROTECT(log);

    AnyObject * c_measurements = sexp_to_anyobjectptr(measurements, T_measurements);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_basic_composition(c_measurements);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP combinators__make_chain_mt(
    SEXP measurement1, SEXP transformation0, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurement1);
    PROTECT(transformation0);
    PROTECT(log);

    AnyMeasurement * c_measurement1 = sexp_to_anymeasurementptr(measurement1);
    AnyTransformation * c_transformation0 = sexp_to_anytransformationptr(transformation0);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_chain_mt(c_measurement1, c_transformation0);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP combinators__make_chain_pm(
    SEXP postprocess1, SEXP measurement0, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(postprocess1);
    PROTECT(measurement0);
    PROTECT(log);

    AnyFunction * c_postprocess1 = sexp_to_anyfunctionptr(postprocess1);
    AnyMeasurement * c_measurement0 = sexp_to_anymeasurementptr(measurement0);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_chain_pm(c_postprocess1, c_measurement0);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP combinators__make_chain_tt(
    SEXP transformation1, SEXP transformation0, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(transformation1);
    PROTECT(transformation0);
    PROTECT(log);

    AnyTransformation * c_transformation1 = sexp_to_anytransformationptr(transformation1);
    AnyTransformation * c_transformation0 = sexp_to_anytransformationptr(transformation0);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_combinators__make_chain_tt(c_transformation1, c_transformation0);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP combinators__make_fix_delta(
    SEXP measurement, SEXP delta, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurement);
    PROTECT(delta);
    PROTECT(log);

    AnyMeasurement * c_measurement = sexp_to_anymeasurementptr(measurement);
    double c_delta = Rf_asReal(delta);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_fix_delta(c_measurement, c_delta);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP combinators__make_fixed_approxDP_to_approxDP(
    SEXP measurement, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurement);
    PROTECT(log);

    AnyMeasurement * c_measurement = sexp_to_anymeasurementptr(measurement);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_fixed_approxDP_to_approxDP(c_measurement);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP combinators__make_population_amplification(
    SEXP measurement, SEXP population_size, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurement);
    PROTECT(population_size);
    PROTECT(log);

    AnyMeasurement * c_measurement = sexp_to_anymeasurementptr(measurement);
    size_t c_population_size = (size_t)Rf_asInteger(population_size);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_population_amplification(c_measurement, c_population_size);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP combinators__make_pureDP_to_zCDP(
    SEXP measurement, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurement);
    PROTECT(log);

    AnyMeasurement * c_measurement = sexp_to_anymeasurementptr(measurement);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_pureDP_to_zCDP(c_measurement);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP combinators__make_select_private_candidate(
    SEXP measurement, SEXP stop_probability, SEXP threshold, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurement);
    PROTECT(stop_probability);
    PROTECT(threshold);
    PROTECT(log);

    AnyMeasurement * c_measurement = sexp_to_anymeasurementptr(measurement);
    double c_stop_probability = Rf_asReal(stop_probability);
    double c_threshold = Rf_asReal(threshold);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_select_private_candidate(c_measurement, c_stop_probability, c_threshold);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP combinators__make_sequential_composition(
    SEXP input_domain, SEXP input_metric, SEXP output_measure, SEXP d_in, SEXP d_mids, SEXP QO, SEXP T_d_in, SEXP T_d_mids, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(output_measure);
    PROTECT(d_in);
    PROTECT(d_mids);
    PROTECT(QO);
    PROTECT(T_d_in);
    PROTECT(T_d_mids);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyMeasure * c_output_measure = sexp_to_anymeasureptr(output_measure);
    AnyObject * c_d_in = sexp_to_anyobjectptr(d_in, T_d_in);
    AnyObject * c_d_mids = sexp_to_anyobjectptr(d_mids, T_d_mids);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_sequential_composition(c_input_domain, c_input_metric, c_output_measure, c_d_in, c_d_mids);

    UNPROTECT(9);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP combinators__make_zCDP_to_approxDP(
    SEXP measurement, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurement);
    PROTECT(log);

    AnyMeasurement * c_measurement = sexp_to_anymeasurementptr(measurement);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_combinators__make_zCDP_to_approxDP(c_measurement);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}

