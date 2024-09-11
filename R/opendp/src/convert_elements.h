#include <Rinternals.h>
#include "opendp.h"

// transformation
AnyTransformation *sexp_to_anytransformationptr(SEXP data);
SEXP anytransformationptr_to_sexp(AnyTransformation *input, SEXP info);

// measurement
AnyMeasurement *sexp_to_anymeasurementptr(SEXP data);
SEXP anymeasurementptr_to_sexp(AnyMeasurement *input, SEXP info);

// domain
AnyDomain *sexp_to_anydomainptr(SEXP data);
SEXP anydomainptr_to_sexp(AnyDomain *input, SEXP info);

// metric
AnyMetric *sexp_to_anymetricptr(SEXP data);
SEXP anymetricptr_to_sexp(AnyMetric *input, SEXP info);

// measure
AnyMeasure *sexp_to_anymeasureptr(SEXP data);
SEXP anymeasureptr_to_sexp(AnyMeasure *input, SEXP info);

// function
AnyFunction *sexp_to_anyfunctionptr(SEXP data);
SEXP anyfunctionptr_to_sexp(AnyFunction *input, SEXP info);

// privacy profile
AnyObject *sexp_to_privacyprofileptr(SEXP data);
SEXP privacyprofileptr_to_sexp(AnyObject *input, SEXP info);

// queryable
AnyObject *sexp_to_anyqueryableptr(SEXP data);
SEXP anyqueryableptr_to_sexp(AnyObject *input, SEXP info);
