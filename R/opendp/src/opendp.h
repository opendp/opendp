#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct AnyDomain AnyDomain;

typedef struct AnyMeasure AnyMeasure;

typedef struct AnyMetric AnyMetric;

/**
 * A struct that can wrap any object.
 */
typedef struct AnyObject AnyObject;

/**
 * A mathematical function.
 */
typedef struct Function_AnyObject__AnyObject Function_AnyObject__AnyObject;

/**
 * A randomized mechanism with certain privacy characteristics.
 *
 * The trait bounds provided by the Rust type system guarantee that:
 * * `input_domain` and `output_domain` are valid domains
 * * `input_metric` is a valid metric
 * * `output_measure` is a valid measure
 *
 * It is, however, left to constructor functions to prove that:
 * * `input_metric` is compatible with `input_domain`
 * * `privacy_map` is a mapping from the input metric to the output measure
 */
typedef struct Measurement_AnyDomain__AnyMetric__AnyMeasure__AnyObject Measurement_AnyDomain__AnyMetric__AnyMeasure__AnyObject;

/**
 * A privacy odometer that can track privacy loss over multiple queries.
 *
 * The odometer is defined in terms of [HSTV+](https://arxiv.org/abs/2309.05901),
 * but the truncated view as defined in Definition 1.13 is also parameterized by $d_{in}$,
 * and $(\epsilon, \delta)$ is generalized to $d_{out}$.
 */
typedef struct Odometer_AnyDomain__AnyMetric__AnyMeasure__AnyObject__AnyObject Odometer_AnyDomain__AnyMetric__AnyMeasure__AnyObject__AnyObject;

/**
 * A data transformation with certain stability characteristics.
 *
 * The trait bounds provided by the Rust type system guarantee that:
 * * `input_domain` and `output_domain` are valid domains
 * * `input_metric` and `output_metric` are valid metrics
 *
 * It is, however, left to constructor functions to prove that:
 * * metrics are compatible with domains
 * * `function` is a mapping from the input domain to the output domain
 * * `stability_map` is a mapping from the input metric to the output metric
 */
typedef struct Transformation_AnyDomain__AnyMetric__AnyDomain__AnyMetric Transformation_AnyDomain__AnyMetric__AnyDomain__AnyMetric;

typedef struct FfiError {
  char *variant;
  char *message;
  char *backtrace;
} FfiError;

enum FfiResult_____AnyObject_Tag {
  Ok_____AnyObject,
  Err_____AnyObject,
};
typedef uint32_t FfiResult_____AnyObject_Tag;

typedef struct FfiResult_____AnyObject {
  FfiResult_____AnyObject_Tag tag;
  union {
    struct {
      struct AnyObject *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____AnyObject;

/**
 * A Measurement with all generic types filled by Any types. This is the type of Measurements
 * passed back and forth over FFI.
 */
typedef struct Measurement_AnyDomain__AnyMetric__AnyMeasure__AnyObject AnyMeasurement;

enum FfiResult_____AnyMeasurement_Tag {
  Ok_____AnyMeasurement,
  Err_____AnyMeasurement,
};
typedef uint32_t FfiResult_____AnyMeasurement_Tag;

typedef struct FfiResult_____AnyMeasurement {
  FfiResult_____AnyMeasurement_Tag tag;
  union {
    struct {
      AnyMeasurement *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____AnyMeasurement;

/**
 * A Transformation with all generic types filled by Any types. This is the type of Transformation
 * passed back and forth over FFI.
 */
typedef struct Transformation_AnyDomain__AnyMetric__AnyDomain__AnyMetric AnyTransformation;

enum FfiResult_____AnyTransformation_Tag {
  Ok_____AnyTransformation,
  Err_____AnyTransformation,
};
typedef uint32_t FfiResult_____AnyTransformation_Tag;

typedef struct FfiResult_____AnyTransformation {
  FfiResult_____AnyTransformation_Tag tag;
  union {
    struct {
      AnyTransformation *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____AnyTransformation;

typedef struct Function_AnyObject__AnyObject AnyFunction;

/**
 * An Odometer with all generic types filled by Any types. This is the type of Odometers
 * passed back and forth over FFI.
 */
typedef struct Odometer_AnyDomain__AnyMetric__AnyMeasure__AnyObject__AnyObject AnyOdometer;

enum FfiResult_____AnyOdometer_Tag {
  Ok_____AnyOdometer,
  Err_____AnyOdometer,
};
typedef uint32_t FfiResult_____AnyOdometer_Tag;

typedef struct FfiResult_____AnyOdometer {
  FfiResult_____AnyOdometer_Tag tag;
  union {
    struct {
      AnyOdometer *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____AnyOdometer;

enum FfiResult_____AnyFunction_Tag {
  Ok_____AnyFunction,
  Err_____AnyFunction,
};
typedef uint32_t FfiResult_____AnyFunction_Tag;

typedef struct FfiResult_____AnyFunction {
  FfiResult_____AnyFunction_Tag tag;
  union {
    struct {
      AnyFunction *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____AnyFunction;

typedef struct ExtrinsicObject {
  const void *_0;
} ExtrinsicObject;

typedef struct CallbackFn {
  struct FfiResult_____AnyObject *(*callback)(const struct AnyObject*);
  struct ExtrinsicObject lifeline;
} CallbackFn;

enum FfiResult_____c_void_Tag {
  Ok_____c_void,
  Err_____c_void,
};
typedef uint32_t FfiResult_____c_void_Tag;

typedef struct FfiResult_____c_void {
  FfiResult_____c_void_Tag tag;
  union {
    struct {
      void *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____c_void;

enum FfiResult_____AnyDomain_Tag {
  Ok_____AnyDomain,
  Err_____AnyDomain,
};
typedef uint32_t FfiResult_____AnyDomain_Tag;

typedef struct FfiResult_____AnyDomain {
  FfiResult_____AnyDomain_Tag tag;
  union {
    struct {
      struct AnyDomain *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____AnyDomain;

enum FfiResult_____AnyMetric_Tag {
  Ok_____AnyMetric,
  Err_____AnyMetric,
};
typedef uint32_t FfiResult_____AnyMetric_Tag;

typedef struct FfiResult_____AnyMetric {
  FfiResult_____AnyMetric_Tag tag;
  union {
    struct {
      struct AnyMetric *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____AnyMetric;

enum FfiResult_____AnyMeasure_Tag {
  Ok_____AnyMeasure,
  Err_____AnyMeasure,
};
typedef uint32_t FfiResult_____AnyMeasure_Tag;

typedef struct FfiResult_____AnyMeasure {
  FfiResult_____AnyMeasure_Tag tag;
  union {
    struct {
      struct AnyMeasure *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____AnyMeasure;

typedef uint8_t c_bool;

enum FfiResult_____c_bool_Tag {
  Ok_____c_bool,
  Err_____c_bool,
};
typedef uint32_t FfiResult_____c_bool_Tag;

typedef struct FfiResult_____c_bool {
  FfiResult_____c_bool_Tag tag;
  union {
    struct {
      c_bool *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____c_bool;

enum FfiResult_____c_char_Tag {
  Ok_____c_char,
  Err_____c_char,
};
typedef uint32_t FfiResult_____c_char_Tag;

typedef struct FfiResult_____c_char {
  FfiResult_____c_char_Tag tag;
  union {
    struct {
      char *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____c_char;

typedef struct TransitionFn {
  struct FfiResult_____AnyObject *(*callback)(const struct AnyObject*, c_bool);
  struct ExtrinsicObject lifeline;
} TransitionFn;

typedef struct FfiSlice {
  const void *ptr;
  uintptr_t len;
} FfiSlice;

enum FfiResult_____FfiSlice_Tag {
  Ok_____FfiSlice,
  Err_____FfiSlice,
};
typedef uint32_t FfiResult_____FfiSlice_Tag;

typedef struct FfiResult_____FfiSlice {
  FfiResult_____FfiSlice_Tag tag;
  union {
    struct {
      struct FfiSlice *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____FfiSlice;

enum FfiResult______AnyObject_Tag {
  Ok______AnyObject,
  Err______AnyObject,
};
typedef uint32_t FfiResult______AnyObject_Tag;

typedef struct FfiResult______AnyObject {
  FfiResult______AnyObject_Tag tag;
  union {
    struct {
      const struct AnyObject *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult______AnyObject;

enum FfiResult_____ExtrinsicObject_Tag {
  Ok_____ExtrinsicObject,
  Err_____ExtrinsicObject,
};
typedef uint32_t FfiResult_____ExtrinsicObject_Tag;

typedef struct FfiResult_____ExtrinsicObject {
  FfiResult_____ExtrinsicObject_Tag tag;
  union {
    struct {
      struct ExtrinsicObject *ok;
    };
    struct {
      struct FfiError *err;
    };
  };
} FfiResult_____ExtrinsicObject;

void _set_ref_count(bool (*callback)(const void*, bool));

void _set_total_cmp(struct FfiResult_____AnyObject *(*callback)(const void*, const void*));

struct FfiResult_____AnyObject opendp_accuracy__summarize_polars_measurement(const AnyMeasurement *measurement,
                                                                             const struct AnyObject *alpha);

struct FfiResult_____AnyMeasurement opendp_combinators__make_population_amplification(const AnyMeasurement *measurement,
                                                                                      unsigned int population_size);

struct FfiResult_____AnyMeasurement opendp_combinators__make_chain_mt(const AnyMeasurement *measurement1,
                                                                      const AnyTransformation *transformation0);

struct FfiResult_____AnyTransformation opendp_combinators__make_chain_tt(const AnyTransformation *transformation1,
                                                                         const AnyTransformation *transformation0);

struct FfiResult_____AnyMeasurement opendp_combinators__make_chain_pm(const AnyFunction *postprocess1,
                                                                      const AnyMeasurement *measurement0);

/**
 * Construct the DP composition \[`measurement0`, `measurement1`, ...\].
 * Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`
 *
 * All metrics and domains must be equivalent.
 *
 * **Composition Properties**
 *
 * * sequential: all measurements are applied to the same dataset
 * * basic: the composition is the linear sum of the privacy usage of each query
 * * noninteractive: all mechanisms specified up-front (but each can be interactive)
 * * compositor: all privacy parameters specified up-front (via the map)
 *
 * # Arguments
 * * `measurements` - A vector of Measurements to compose.
 */
struct FfiResult_____AnyMeasurement opendp_combinators__make_composition(const struct AnyObject *measurements);

/**
 * Construct the DP composition \[`measurement0`, `measurement1`, ...\].
 * Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`
 *
 * All metrics and domains must be equivalent.
 *
 * **Composition Properties**
 *
 * * sequential: all measurements are applied to the same dataset
 * * basic: the composition is the linear sum of the privacy usage of each query
 * * noninteractive: all mechanisms specified up-front (but each can be interactive)
 * * compositor: all privacy parameters specified up-front (via the map)
 *
 * # Arguments
 * * `measurements` - A vector of Measurements to compose.
 */
struct FfiResult_____AnyMeasurement opendp_combinators__make_basic_composition(const struct AnyObject *measurements);

struct FfiResult_____AnyMeasurement opendp_combinators__make_adaptive_composition(const struct AnyDomain *input_domain,
                                                                                  const struct AnyMetric *input_metric,
                                                                                  const struct AnyMeasure *output_measure,
                                                                                  const struct AnyObject *d_in,
                                                                                  const struct AnyObject *d_mids);

struct FfiResult_____AnyMeasurement opendp_combinators__make_sequential_composition(const struct AnyDomain *input_domain,
                                                                                    const struct AnyMetric *input_metric,
                                                                                    const struct AnyMeasure *output_measure,
                                                                                    const struct AnyObject *d_in,
                                                                                    const struct AnyObject *d_mids);

struct FfiResult_____AnyOdometer opendp_combinators__make_fully_adaptive_composition(const struct AnyDomain *input_domain,
                                                                                     const struct AnyMetric *input_metric,
                                                                                     const struct AnyMeasure *output_measure);

struct FfiResult_____AnyMeasurement opendp_combinators__make_select_private_candidate(const AnyMeasurement *measurement,
                                                                                      double stop_probability,
                                                                                      double threshold);

struct FfiResult_____AnyMeasurement opendp_combinators__make_fixed_approxDP_to_approxDP(const AnyMeasurement *measurement);

struct FfiResult_____AnyMeasurement opendp_combinators__make_zCDP_to_approxDP(const AnyMeasurement *measurement);

struct FfiResult_____AnyMeasurement opendp_combinators__make_approximate(const AnyMeasurement *measurement);

struct FfiResult_____AnyMeasurement opendp_combinators__make_pureDP_to_zCDP(const AnyMeasurement *measurement);

struct FfiResult_____AnyMeasurement opendp_combinators__make_privacy_filter(const AnyOdometer *odometer,
                                                                            const struct AnyObject *d_in,
                                                                            const struct AnyObject *d_out);

struct FfiResult_____AnyMeasurement opendp_combinators__make_fix_delta(const AnyMeasurement *measurement,
                                                                       double delta);

/**
 * Internal function. Free the memory associated with `error`.
 *
 * # Returns
 * A boolean, where true indicates successful free
 */
bool opendp_core___error_free(struct FfiError *this_);

struct FfiResult_____AnyFunction opendp_core__new_function(const struct CallbackFn *function,
                                                           const char *TO);

/**
 * Eval the `function` with `arg`.
 *
 * # Arguments
 * * `this` - Function to invoke.
 * * `arg` - Input data to supply to the measurement. A member of the measurement's input domain.
 * * `TI` - Input Type.
 */
struct FfiResult_____AnyObject opendp_core__function_eval(const AnyFunction *this_,
                                                          const struct AnyObject *arg,
                                                          const char *TI);

/**
 * Internal function. Free the memory associated with `this`.
 */
struct FfiResult_____c_void opendp_core___function_free(AnyFunction *this_);

/**
 * Get the input domain from a `measurement`.
 *
 * # Arguments
 * * `this` - The measurement to retrieve the value from.
 */
struct FfiResult_____AnyDomain opendp_core__measurement_input_domain(AnyMeasurement *this_);

/**
 * Get the input domain from a `measurement`.
 *
 * # Arguments
 * * `this` - The measurement to retrieve the value from.
 */
struct FfiResult_____AnyMetric opendp_core__measurement_input_metric(AnyMeasurement *this_);

/**
 * Get the output domain from a `measurement`.
 *
 * # Arguments
 * * `this` - The measurement to retrieve the value from.
 */
struct FfiResult_____AnyMeasure opendp_core__measurement_output_measure(AnyMeasurement *this_);

/**
 * Get the function from a measurement.
 *
 * # Arguments
 * * `this` - The measurement to retrieve the value from.
 */
struct FfiResult_____AnyFunction opendp_core__measurement_function(AnyMeasurement *this_);

/**
 * Use the `measurement` to map a given `d_in` to `d_out`.
 *
 * # Arguments
 * * `measurement` - Measurement to check the map distances with.
 * * `distance_in` - Distance in terms of the input metric.
 */
struct FfiResult_____AnyObject opendp_core__measurement_map(const AnyMeasurement *measurement,
                                                            const struct AnyObject *distance_in);

/**
 * Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
 *
 * # Arguments
 * * `measurement` - Measurement to check the privacy relation of.
 * * `d_in` - Distance in terms of the input metric.
 * * `d_out` - Distance in terms of the output metric.
 *
 * # Returns
 * True indicates that the relation passed at the given distance.
 */
struct FfiResult_____c_bool opendp_core__measurement_check(const AnyMeasurement *measurement,
                                                           const struct AnyObject *distance_in,
                                                           const struct AnyObject *distance_out);

/**
 * Invoke the `measurement` with `arg`. Returns a differentially private release.
 *
 * # Arguments
 * * `this` - Measurement to invoke.
 * * `arg` - Input data to supply to the measurement. A member of the measurement's input domain.
 */
struct FfiResult_____AnyObject opendp_core__measurement_invoke(const AnyMeasurement *this_,
                                                               const struct AnyObject *arg);

/**
 * Internal function. Free the memory associated with `this`.
 */
struct FfiResult_____c_void opendp_core___measurement_free(AnyMeasurement *this_);

/**
 * Get the input (carrier) data type of `this`.
 *
 * # Arguments
 * * `this` - The measurement to retrieve the type from.
 */
struct FfiResult_____c_char opendp_core__measurement_input_carrier_type(AnyMeasurement *this_);

/**
 * Get the input distance type of `measurement`.
 *
 * # Arguments
 * * `this` - The measurement to retrieve the type from.
 */
struct FfiResult_____c_char opendp_core__measurement_input_distance_type(AnyMeasurement *this_);

/**
 * Get the output distance type of `measurement`.
 *
 * # Arguments
 * * `this` - The measurement to retrieve the type from.
 */
struct FfiResult_____c_char opendp_core__measurement_output_distance_type(AnyMeasurement *this_);

/**
 * Get the input domain from a `odometer`.
 *
 * # Arguments
 * * `this` - The odometer to retrieve the value from.
 */
struct FfiResult_____AnyDomain opendp_core__odometer_input_domain(AnyOdometer *this_);

/**
 * Get the input domain from a `odometer`.
 *
 * # Arguments
 * * `this` - The odometer to retrieve the value from.
 */
struct FfiResult_____AnyMetric opendp_core__odometer_input_metric(AnyOdometer *this_);

/**
 * Get the output domain from a `odometer`.
 *
 * # Arguments
 * * `this` - The odometer to retrieve the value from.
 */
struct FfiResult_____AnyMeasure opendp_core__odometer_output_measure(AnyOdometer *this_);

/**
 * Invoke the `odometer` with `arg`. Returns a differentially private release.
 *
 * # Arguments
 * * `this` - Odometer to invoke.
 * * `arg` - Input data to supply to the odometer. A member of the odometer's input domain.
 */
struct FfiResult_____AnyObject opendp_core__odometer_invoke(const AnyOdometer *this_,
                                                            const struct AnyObject *arg);

/**
 * Get the input (carrier) data type of `this`.
 *
 * # Arguments
 * * `this` - The odometer to retrieve the type from.
 */
struct FfiResult_____c_char opendp_core__odometer_input_carrier_type(AnyOdometer *this_);

/**
 * Internal function. Free the memory associated with `this`.
 */
struct FfiResult_____c_void opendp_core___odometer_free(AnyOdometer *this_);

/**
 * Eval the odometer `queryable` with an invoke `query`.
 *
 * # Arguments
 * * `queryable` - Queryable to eval.
 * * `query` - Invoke query to supply to the queryable.
 */
struct FfiResult_____AnyObject opendp_core__odometer_queryable_invoke(struct AnyObject *queryable,
                                                                      const struct AnyObject *query);

/**
 * Get the invoke query type of an odometer `queryable`.
 *
 * # Arguments
 * * `this` - The queryable to retrieve the query type from.
 */
struct FfiResult_____c_char opendp_core__odometer_queryable_invoke_type(struct AnyObject *this_);

/**
 * Get the map query type of an odometer `queryable`.
 *
 * # Arguments
 * * `this` - The queryable to retrieve the query type from.
 */
struct FfiResult_____c_char opendp_core__odometer_queryable_privacy_loss_type(struct AnyObject *this_);

/**
 * Retrieve the privacy loss of an odometer `queryable`.
 *
 * # Arguments
 * * `queryable` - Queryable to eval.
 * * `d_in` - Maximum distance between adjacent inputs in the input domain.
 */
struct FfiResult_____AnyObject opendp_core__odometer_queryable_privacy_loss(struct AnyObject *queryable,
                                                                            const struct AnyObject *d_in);

/**
 * Get the input domain from a `transformation`.
 *
 * # Arguments
 * * `this` - The transformation to retrieve the value from.
 */
struct FfiResult_____AnyDomain opendp_core__transformation_input_domain(AnyTransformation *this_);

/**
 * Get the output domain from a `transformation`.
 *
 * # Arguments
 * * `this` - The transformation to retrieve the value from.
 */
struct FfiResult_____AnyDomain opendp_core__transformation_output_domain(AnyTransformation *this_);

/**
 * Get the input domain from a `transformation`.
 *
 * # Arguments
 * * `this` - The transformation to retrieve the value from.
 */
struct FfiResult_____AnyMetric opendp_core__transformation_input_metric(AnyTransformation *this_);

/**
 * Get the output domain from a `transformation`.
 *
 * # Arguments
 * * `this` - The transformation to retrieve the value from.
 */
struct FfiResult_____AnyMetric opendp_core__transformation_output_metric(AnyTransformation *this_);

/**
 * Use the `transformation` to map a given `d_in` to `d_out`.
 *
 * # Arguments
 * * `transformation` - Transformation to check the map distances with.
 * * `distance_in` - Distance in terms of the input metric.
 */
struct FfiResult_____AnyObject opendp_core__transformation_map(const AnyTransformation *transformation,
                                                               const struct AnyObject *distance_in);

/**
 * Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
 *
 * # Arguments
 * * `measurement` - Measurement to check the privacy relation of.
 * * `d_in` - Distance in terms of the input metric.
 * * `d_out` - Distance in terms of the output metric.
 *
 * # Returns
 * True indicates that the relation passed at the given distance.
 */
struct FfiResult_____c_bool opendp_core__transformation_check(const AnyTransformation *transformation,
                                                              const struct AnyObject *distance_in,
                                                              const struct AnyObject *distance_out);

/**
 * Invoke the `transformation` with `arg`. Returns a differentially private release.
 *
 * # Arguments
 * * `this` - Transformation to invoke.
 * * `arg` - Input data to supply to the transformation. A member of the transformation's input domain.
 */
struct FfiResult_____AnyObject opendp_core__transformation_invoke(const AnyTransformation *this_,
                                                                  const struct AnyObject *arg);

/**
 * Get the function from a transformation.
 *
 * # Arguments
 * * `this` - The transformation to retrieve the value from.
 */
struct FfiResult_____AnyFunction opendp_core__transformation_function(AnyTransformation *this_);

/**
 * Internal function. Free the memory associated with `this`.
 */
struct FfiResult_____c_void opendp_core___transformation_free(AnyTransformation *this_);

/**
 * Get the input (carrier) data type of `this`.
 *
 * # Arguments
 * * `this` - The transformation to retrieve the type from.
 */
struct FfiResult_____c_char opendp_core__transformation_input_carrier_type(AnyTransformation *this_);

/**
 * Get the input distance type of `transformation`.
 *
 * # Arguments
 * * `this` - The transformation to retrieve the type from.
 */
struct FfiResult_____c_char opendp_core__transformation_input_distance_type(AnyTransformation *this_);

/**
 * Get the output distance type of `transformation`.
 *
 * # Arguments
 * * `this` - The transformation to retrieve the type from.
 */
struct FfiResult_____c_char opendp_core__transformation_output_distance_type(AnyTransformation *this_);

/**
 * Eval the `queryable` with `query`. Returns a differentially private release.
 *
 * # Arguments
 * * `queryable` - Queryable to eval.
 * * `query` - The input to the queryable.
 */
struct FfiResult_____AnyObject opendp_core__queryable_eval(struct AnyObject *queryable,
                                                           const struct AnyObject *query);

/**
 * Get the query type of `queryable`.
 *
 * # Arguments
 * * `this` - The queryable to retrieve the query type from.
 */
struct FfiResult_____c_char opendp_core__queryable_query_type(struct AnyObject *this_);

struct FfiResult_____AnyObject opendp_core__new_queryable(const struct TransitionFn *transition,
                                                          const char *Q,
                                                          const char *A);

/**
 * Internal function. Load data from a `slice` into an AnyObject
 *
 * # Arguments
 * * `raw` - A pointer to the slice with data.
 * * `T` - The type of the data in the slice.
 *
 * # Returns
 * An AnyObject that contains the data in `slice`. The AnyObject also captures rust type information.
 */
struct FfiResult_____AnyObject opendp_data__slice_as_object(const struct FfiSlice *raw,
                                                            const char *T);

/**
 * Internal function. Retrieve the type descriptor string of an AnyObject.
 *
 * # Arguments
 * * `this` - A pointer to the AnyObject.
 */
struct FfiResult_____c_char opendp_data__object_type(struct AnyObject *this_);

/**
 * Internal function. Unload data from an AnyObject into an FfiSlicePtr.
 *
 * # Arguments
 * * `obj` - A pointer to the AnyObject to unpack.
 *
 * # Returns
 * An FfiSlice that contains the data in FfiObject, but in a format readable in bindings languages.
 */
struct FfiResult_____FfiSlice opendp_data__object_as_slice(const struct AnyObject *obj);

/**
 * Internal function. Converts an FfiSlice of AnyObjects to an FfiSlice of AnyObjectPtrs.
 *
 * # Arguments
 * * `raw` - A pointer to the slice to free.
 */
struct FfiResult_____FfiSlice opendp_data__ffislice_of_anyobjectptrs(const struct FfiSlice *raw);

/**
 * Internal function. Free the memory associated with `this`, an AnyObject.
 *
 * # Arguments
 * * `this` - A pointer to the AnyObject to free.
 */
struct FfiResult_____c_void opendp_data__object_free(struct AnyObject *this_);

/**
 * Internal function. Compute erfc.
 *
 * Used to prove an upper bound on the error of erfc.
 */
double opendp_data__erfc(double value);

/**
 * Internal function. Free the memory associated with `this`, an FfiSlicePtr.
 * Used to clean up after object_as_slice.
 * Frees the slice, but not what the slice references!
 *
 * # Arguments
 * * `this` - A pointer to the FfiSlice to free.
 */
struct FfiResult_____c_void opendp_data__slice_free(struct FfiSlice *this_);

/**
 * Internal function. Free the memory associated with `this`, a slice containing an Arrow array, schema, and name.
 */
struct FfiResult_____c_void opendp_data__arrow_array_free(void *this_);

/**
 * Internal function. Free the memory associated with `this`, a string.
 * Used to clean up after the type getter functions.
 *
 * # Arguments
 * * `this` - A pointer to the string to free.
 */
struct FfiResult_____c_void opendp_data__str_free(char *this_);

/**
 * Internal function. Free the memory associated with `this`, a bool.
 * Used to clean up after the relation check.
 *
 * # Arguments
 * * `this` - A pointer to the bool to free.
 */
struct FfiResult_____c_void opendp_data__bool_free(c_bool *this_);

/**
 * Internal function. Free the memory associated with `this`, a string.
 * Used to clean up after the type getter functions.
 */
struct FfiResult_____c_void opendp_data__extrinsic_object_free(struct ExtrinsicObject *this_);

/**
 * Internal function. Use a PrivacyProfile to find epsilon at a given `epsilon`.
 *
 * # Arguments
 * * `curve` - The PrivacyProfile.
 * * `epsilon` - What to fix epsilon to compute delta.
 *
 * # Returns
 * Delta at a given `epsilon`.
 */
struct FfiResult_____AnyObject opendp_data__privacy_profile_delta(const struct AnyObject *curve,
                                                                  double epsilon);

/**
 * Internal function. Use an PrivacyProfile to find epsilon at a given `delta`.
 *
 * # Arguments
 * * `profile` - The PrivacyProfile.
 * * `delta` - What to fix delta to compute epsilon.
 *
 * # Returns
 * Epsilon at a given `delta`.
 */
struct FfiResult_____AnyObject opendp_data__privacy_profile_epsilon(const struct AnyObject *profile,
                                                                    double delta);

/**
 * Allocate an empty ArrowArray and ArrowSchema that Rust owns the memory for.
 * The ArrowArray and ArrowSchema are initialized empty, and are populated by the bindings language.
 *
 * # Arguments
 * * `name` - The name of the ArrowArray. A clone of this string owned by Rust will be returned in the slice.
 */
struct FfiResult_____FfiSlice opendp_data__new_arrow_array(const char *name);

/**
 * wrap an AnyObject in an FfiResult::Ok(this)
 *
 * # Arguments
 * * `this` - The AnyObject to wrap.
 */
const struct FfiResult______AnyObject *ffiresult_ok(const struct AnyObject *this_);

/**
 * construct an FfiResult::Err(e)
 *
 * # Arguments
 * * `message` - The error message.
 * * `backtrace` - The error backtrace.
 */
const struct FfiResult______AnyObject *ffiresult_err(char *message, char *backtrace);

/**
 * Internal function. Populate the buffer behind `ptr` with `len` random bytes
 * sampled from a cryptographically secure RNG.
 */
bool opendp_data__fill_bytes(uint8_t *ptr, uintptr_t len);

/**
 * Internal function. Collects a DataFrame from a OnceFrame, exhausting the OnceFrame.
 *
 * # Arguments
 * * `onceframe` - The queryable holding a LazyFrame.
 */
struct FfiResult_____AnyObject opendp_data__onceframe_collect(struct AnyObject *onceframe);

/**
 * Internal function. Extracts a LazyFrame from a OnceFrame,
 * circumventing protections against multiple evaluations.
 *
 * Each collection consumes the entire allocated privacy budget.
 * To remain DP at the advertised privacy level, only collect the LazyFrame once.
 *
 * # Arguments
 * * `onceframe` - The queryable holding a LazyFrame.
 *
 * # Why honest-but-curious?
 * The privacy guarantees only apply if:
 *
 * 1. The LazyFrame (compute plan) is only ever executed once.
 * 2. The analyst does not observe ordering of rows in the output.
 *
 * To ensure that row ordering is not observed:
 *
 * 1. Do not extend the compute plan with order-sensitive computations.
 * 2. Shuffle the output once collected ([in Polars sample all, with shuffling enabled](https://docs.pola.rs/api/python/stable/reference/dataframe/api/polars.DataFrame.sample.html)).
 */
struct FfiResult_____AnyObject opendp_data__onceframe_lazy(struct AnyObject *onceframe);

/**
 * Internal function. Free the memory associated with `this`.
 */
struct FfiResult_____c_void opendp_domains___domain_free(struct AnyDomain *this_);

/**
 * Check membership in a `domain`.
 *
 * # Arguments
 * * `this` - The domain to check membership in.
 * * `val` - A potential element of the domain.
 */
struct FfiResult_____c_bool opendp_domains___member(struct AnyDomain *this_,
                                                    const struct AnyObject *val);

/**
 * Check whether two domains are equal.
 *
 * # Arguments
 * * `left` - Domain to compare.
 * * `right` - Domain to compare.
 */
struct FfiResult_____c_bool opendp_domains___domain_equal(struct AnyDomain *left,
                                                          const struct AnyDomain *right);

/**
 * Debug a `domain`.
 *
 * # Arguments
 * * `this` - The domain to debug (stringify).
 */
struct FfiResult_____c_char opendp_domains__domain_debug(struct AnyDomain *this_);

/**
 * Get the type of a `domain`.
 *
 * # Arguments
 * * `this` - The domain to retrieve the type from.
 */
struct FfiResult_____c_char opendp_domains__domain_type(struct AnyDomain *this_);

/**
 * Get the carrier type of a `domain`.
 *
 * # Arguments
 * * `this` - The domain to retrieve the carrier type from.
 */
struct FfiResult_____c_char opendp_domains__domain_carrier_type(struct AnyDomain *this_);

struct FfiResult_____AnyDomain opendp_domains__atom_domain(const struct AnyObject *bounds,
                                                           const struct AnyObject *nan,
                                                           const char *T);

struct FfiResult_____AnyObject opendp_domains___atom_domain_get_bounds_closed(const struct AnyDomain *domain);

struct FfiResult_____AnyObject opendp_domains___atom_domain_nan(const struct AnyDomain *domain);

struct FfiResult_____AnyDomain opendp_domains__option_domain(const struct AnyDomain *element_domain,
                                                             const char *D);

/**
 * Retrieve the element domain of the option domain.
 *
 * # Arguments
 * * `option_domain` - The option domain from which to retrieve the element domain
 */
struct FfiResult_____AnyDomain opendp_domains___option_domain_get_element_domain(const struct AnyDomain *option_domain);

/**
 * Construct an instance of `VectorDomain`.
 *
 * # Arguments
 * * `atom_domain` - The inner domain.
 */
struct FfiResult_____AnyDomain opendp_domains__vector_domain(const struct AnyDomain *atom_domain,
                                                             const struct AnyObject *size);

/**
 * Retrieve the element domain of the vector domain.
 *
 * # Arguments
 * * `vector_domain` - The vector domain from which to retrieve the element domain
 */
struct FfiResult_____AnyDomain opendp_domains___vector_domain_get_element_domain(const struct AnyDomain *vector_domain);

/**
 * Retrieve the size of vectors in the vector domain.
 *
 * # Arguments
 * * `vector_domain` - The vector domain from which to retrieve the size
 */
struct FfiResult_____AnyObject opendp_domains___vector_domain_get_size(const struct AnyDomain *vector_domain);

/**
 * Construct an instance of `BitVectorDomain`.
 *
 * # Arguments
 * * `max_weight` - The maximum number of positive bits.
 */
struct FfiResult_____AnyDomain opendp_domains__bitvector_domain(const struct AnyObject *max_weight);

/**
 * Construct an instance of `MapDomain`.
 *
 * # Arguments
 * * `key_domain` - domain of keys in the hashmap
 * * `value_domain` - domain of values in the hashmap
 */
struct FfiResult_____AnyDomain opendp_domains__map_domain(const struct AnyDomain *key_domain,
                                                          const struct AnyDomain *value_domain);

/**
 * Construct a new UserDomain.
 * Any two instances of an UserDomain are equal if their string descriptors are equal.
 * Contains a function used to check if any value is a member of the domain.
 *
 * # Arguments
 * * `identifier` - A string description of the data domain.
 * * `member` - A function used to test if a value is a member of the data domain.
 * * `descriptor` - Additional constraints on the domain.
 *
 * # Why honest-but-curious?
 * The identifier must uniquely identify this domain.
 * If the identifier is not uniquely identifying,
 * then two different domains with the same identifier will chain,
 * which can violate transformation stability.
 *
 * In addition, the member function must:
 *
 * 1. be a pure function
 * 2. be sound (only return true if its input is a member of the domain).
 */
struct FfiResult_____AnyDomain opendp_domains__user_domain(char *identifier,
                                                           const struct CallbackFn *member,
                                                           struct ExtrinsicObject *descriptor);

/**
 * Retrieve the descriptor value stored in an extrinsic domain.
 *
 * # Arguments
 * * `domain` - The ExtrinsicDomain to extract the descriptor from
 */
struct FfiResult_____ExtrinsicObject opendp_domains___extrinsic_domain_descriptor(struct AnyDomain *domain);

/**
 * Construct an instance of `ArrayDomain`.
 * Can be used as an argument to a Polars series domain.
 *
 * # Arguments
 * * `element_domain` - The domain of each element in the array.
 * * `width` - The width of the array.
 */
struct FfiResult_____AnyDomain opendp_domains__array_domain(const struct AnyDomain *element_domain,
                                                            uint32_t width);

/**
 * Construct an instance of `CategoricalDomain`.
 * Can be used as an argument to a Polars series domain.
 *
 * # Arguments
 * * `categories` - Optional ordered set of valid string categories
 */
struct FfiResult_____AnyDomain opendp_domains__categorical_domain(const struct AnyObject *categories);

struct FfiResult_____AnyDomain opendp_domains__datetime_domain(char *time_unit, char *time_zone);

/**
 * Construct an instance of `EnumDomain`.
 * Can be used as an argument to a Polars series domain.
 *
 * # Arguments
 * * `categories` - Optional ordered set of string categories
 */
struct FfiResult_____AnyDomain opendp_domains__enum_domain(const struct AnyObject *categories);

/**
 * Construct an instance of `LazyFrameDomain`.
 *
 * # Arguments
 * * `series_domains` - Domain of each series in the lazyframe.
 */
struct FfiResult_____AnyDomain opendp_domains__lazyframe_domain(const struct AnyObject *series_domains);

/**
 * Retrieve the column names of the LazyFrameDomain.
 *
 * # Arguments
 * * `lazyframe_domain` - Domain to retrieve the column names from
 */
struct FfiResult_____AnyObject opendp_domains___lazyframe_domain_get_columns(const struct AnyDomain *lazyframe_domain);

/**
 * Retrieve the series domain at index `column`.
 *
 * # Arguments
 * * `lazyframe_domain` - Domain to retrieve the SeriesDomain from
 * * `name` - Name of the SeriesDomain to retrieve
 */
struct FfiResult_____AnyDomain opendp_domains___lazyframe_domain_get_series_domain(const struct AnyDomain *lazyframe_domain,
                                                                                   const char *name);

/**
 * Retrieve the series domain at index 'column`.
 *
 * # Arguments
 * * `lazyframe_domain` - Domain to retrieve the SeriesDomain from
 * * `by` - grouping columns
 */
struct FfiResult_____AnyObject opendp_domains___lazyframe_domain_get_margin(const struct AnyDomain *lazyframe_domain,
                                                                            const struct AnyObject *by);

struct FfiResult_____AnyObject opendp_domains___lazyframe_from_domain(struct AnyDomain *domain);

struct FfiResult_____AnyDomain opendp_domains__with_margin(struct AnyDomain *frame_domain,
                                                           struct AnyObject *margin);

struct FfiResult_____AnyDomain opendp_domains__series_domain(char *name,
                                                             const struct AnyDomain *element_domain);

/**
 * # Arguments
 * * `series_domain` - The series domain from which to retrieve the name of elements
 */
struct FfiResult_____AnyObject opendp_domains___series_domain_get_name(const struct AnyDomain *series_domain);

struct FfiResult_____AnyDomain opendp_domains___series_domain_get_element_domain(const struct AnyDomain *series_domain);

/**
 * Retrieve whether elements in members of the domain may be null.
 *
 * # Arguments
 * * `series_domain` - The series domain from which to check nullability.
 */
struct FfiResult_____AnyObject opendp_domains___series_domain_get_nullable(const struct AnyDomain *series_domain);

/**
 * Construct a WildExprDomain.
 *
 * # Arguments
 * * `columns` - descriptors for each column in the data
 * * `by` - optional. Set if expression is applied to grouped data
 * * `margin` - descriptors for grouped data
 */
struct FfiResult_____AnyDomain opendp_domains__wild_expr_domain(const struct AnyObject *columns,
                                                                const struct AnyObject *margin);

struct FfiResult_____AnyMeasurement opendp_internal___make_measurement(const struct AnyDomain *input_domain,
                                                                       const struct AnyMetric *input_metric,
                                                                       const struct AnyMeasure *output_measure,
                                                                       const struct CallbackFn *function,
                                                                       const struct CallbackFn *privacy_map,
                                                                       const char *TO);

/**
 * Construct a Transformation from user-defined callbacks.
 * This is meant for internal use, as it does not require "honest-but-curious",
 * unlike `make_user_transformation`.
 *
 * See `make_user_transformation` for correct usage and proof definition for this function.
 *
 * # Arguments
 * * `input_domain` - A domain describing the set of valid inputs for the function.
 * * `input_metric` - The metric from which distances between adjacent inputs are measured.
 * * `output_domain` - A domain describing the set of valid outputs of the function.
 * * `output_metric` - The metric from which distances between outputs of adjacent inputs are measured.
 * * `function` - A function mapping data from `input_domain` to `output_domain`.
 * * `stability_map` - A function mapping distances from `input_metric` to `output_metric`.
 */
struct FfiResult_____AnyTransformation opendp_internal___make_transformation(const struct AnyDomain *input_domain,
                                                                             const struct AnyMetric *input_metric,
                                                                             const struct AnyDomain *output_domain,
                                                                             const struct AnyMetric *output_metric,
                                                                             const struct CallbackFn *function,
                                                                             const struct CallbackFn *stability_map);

/**
 * Construct a new ExtrinsicDomain.
 * This is meant for internal use, as it does not require "honest-but-curious",
 * unlike `user_domain`.
 *
 * See `user_domain` for correct usage and proof definition for this function.
 *
 * # Arguments
 * * `identifier` - A string description of the data domain.
 * * `member` - A function used to test if a value is a member of the data domain.
 * * `descriptor` - Additional constraints on the domain.
 */
struct FfiResult_____AnyDomain opendp_internal___extrinsic_domain(char *identifier,
                                                                  const struct CallbackFn *member,
                                                                  struct ExtrinsicObject *descriptor);

/**
 * Construct a new ExtrinsicDivergence, a privacy measure defined from a bindings language.
 * This is meant for internal use, as it does not require "honest-but-curious",
 * unlike `user_divergence`.
 *
 * See `user_divergence` for correct usage and proof definition for this function.
 *
 * # Arguments
 * * `descriptor` - A string description of the privacy measure.
 */
struct FfiResult_____AnyMeasure opendp_internal___extrinsic_divergence(char *descriptor);

/**
 * Construct a new ExtrinsicDistance.
 * This is meant for internal use, as it does not require "honest-but-curious",
 * unlike `user_distance`.
 *
 * See `user_distance` for correct usage of this function.
 *
 * # Arguments
 * * `identifier` - A string description of the metric.
 * * `descriptor` - Additional constraints on the domain.
 */
struct FfiResult_____AnyMetric opendp_internal___extrinsic_distance(char *identifier,
                                                                    struct ExtrinsicObject *descriptor);

struct FfiResult_____AnyFunction opendp_internal___new_pure_function(const struct CallbackFn *function,
                                                                     const char *TO);

struct FfiResult_____AnyMeasurement opendp_measurements__make_private_expr(const struct AnyDomain *input_domain,
                                                                           const struct AnyMetric *input_metric,
                                                                           const struct AnyMeasure *output_measure,
                                                                           const struct AnyObject *expr,
                                                                           const struct AnyObject *global_scale);

struct FfiResult_____AnyMeasurement opendp_measurements__make_private_lazyframe(const struct AnyDomain *input_domain,
                                                                                const struct AnyMetric *input_metric,
                                                                                const struct AnyMeasure *output_measure,
                                                                                const struct AnyObject *lazyframe,
                                                                                const struct AnyObject *global_scale,
                                                                                const struct AnyObject *threshold);

struct FfiResult_____AnyMeasurement opendp_measurements__make_user_measurement(const struct AnyDomain *input_domain,
                                                                               const struct AnyMetric *input_metric,
                                                                               const struct AnyMeasure *output_measure,
                                                                               const struct CallbackFn *function,
                                                                               const struct CallbackFn *privacy_map,
                                                                               const char *TO);

struct FfiResult_____AnyMeasurement opendp_measurements__make_private_quantile(const struct AnyDomain *input_domain,
                                                                               const struct AnyMetric *input_metric,
                                                                               const struct AnyMeasure *output_measure,
                                                                               const struct AnyObject *candidates,
                                                                               double alpha,
                                                                               double scale);

struct FfiResult_____AnyMeasurement opendp_measurements__make_gaussian(const struct AnyDomain *input_domain,
                                                                       const struct AnyMetric *input_metric,
                                                                       double scale,
                                                                       const int32_t *k,
                                                                       const char *MO);

struct FfiResult_____AnyMeasurement opendp_measurements__make_geometric(const struct AnyDomain *input_domain,
                                                                        const struct AnyMetric *input_metric,
                                                                        double scale,
                                                                        const struct AnyObject *bounds,
                                                                        const char *MO);

struct FfiResult_____AnyMeasurement opendp_measurements__make_laplace(const struct AnyDomain *input_domain,
                                                                      const struct AnyMetric *input_metric,
                                                                      double scale,
                                                                      const int32_t *k,
                                                                      const char *MO);

struct FfiResult_____AnyMeasurement opendp_measurements__make_noise(const struct AnyDomain *input_domain,
                                                                    const struct AnyMetric *input_metric,
                                                                    const struct AnyMeasure *output_measure,
                                                                    double scale,
                                                                    const int32_t *k);

struct FfiResult_____AnyMeasurement opendp_measurements__make_laplace_threshold(const struct AnyDomain *input_domain,
                                                                                const struct AnyMetric *input_metric,
                                                                                double scale,
                                                                                const void *threshold,
                                                                                const int32_t *k,
                                                                                const char *MO);

struct FfiResult_____AnyMeasurement opendp_measurements__make_gaussian_threshold(const struct AnyDomain *input_domain,
                                                                                 const struct AnyMetric *input_metric,
                                                                                 double scale,
                                                                                 const void *threshold,
                                                                                 const int32_t *k,
                                                                                 const char *MO);

struct FfiResult_____AnyMeasurement opendp_measurements__make_noise_threshold(const struct AnyDomain *input_domain,
                                                                              const struct AnyMetric *input_metric,
                                                                              const struct AnyMeasure *output_measure,
                                                                              double scale,
                                                                              const void *threshold,
                                                                              const int32_t *k);

struct FfiResult_____AnyMeasurement opendp_measurements__make_noisy_max(const struct AnyDomain *input_domain,
                                                                        const struct AnyMetric *input_metric,
                                                                        const struct AnyMeasure *output_measure,
                                                                        double scale,
                                                                        c_bool negate);

struct FfiResult_____AnyMeasurement opendp_measurements__make_report_noisy_max_gumbel(const struct AnyDomain *input_domain,
                                                                                      const struct AnyMetric *input_metric,
                                                                                      double scale,
                                                                                      const char *optimize);

struct FfiResult_____AnyMeasurement opendp_measurements__make_noisy_top_k(const struct AnyDomain *input_domain,
                                                                          const struct AnyMetric *input_metric,
                                                                          const struct AnyMeasure *output_measure,
                                                                          uint32_t k,
                                                                          double scale,
                                                                          c_bool negate);

struct FfiResult_____AnyMeasurement opendp_measurements__make_randomized_response_bool(double prob,
                                                                                       c_bool constant_time);

struct FfiResult_____AnyMeasurement opendp_measurements__make_randomized_response(const struct AnyObject *categories,
                                                                                  double prob,
                                                                                  const char *T);

struct FfiResult_____AnyMeasurement opendp_measurements__make_randomized_response_bitvec(const struct AnyDomain *input_domain,
                                                                                         const struct AnyMetric *input_metric,
                                                                                         double f,
                                                                                         c_bool constant_time);

struct FfiResult_____AnyObject opendp_measurements__debias_randomized_response_bitvec(const struct AnyObject *answers,
                                                                                      double f);

struct FfiResult_____AnyMeasurement opendp_measurements__make_canonical_noise(const struct AnyDomain *input_domain,
                                                                              const struct AnyMetric *input_metric,
                                                                              double d_in,
                                                                              const struct AnyObject *d_out);

struct FfiResult_____AnyMeasurement opendp_measurements__make_alp_queryable(const struct AnyDomain *input_domain,
                                                                            const struct AnyMetric *input_metric,
                                                                            double scale,
                                                                            const void *total_limit,
                                                                            const void *value_limit,
                                                                            const uint32_t *size_factor,
                                                                            const uint32_t *alpha);

/**
 * Internal function. Free the memory associated with `this`.
 */
struct FfiResult_____c_void opendp_measures___measure_free(struct AnyMeasure *this_);

/**
 * Check whether two measures are equal.
 *
 * # Arguments
 * * `left` - Measure to compare.
 * * `right` - Measure to compare.
 */
struct FfiResult_____c_bool opendp_measures___measure_equal(struct AnyMeasure *left,
                                                            const struct AnyMeasure *right);

/**
 * Debug a `measure`.
 *
 * # Arguments
 * * `this` - The measure to debug (stringify).
 */
struct FfiResult_____c_char opendp_measures__measure_debug(struct AnyMeasure *this_);

/**
 * Get the type of a `measure`.
 *
 * # Arguments
 * * `this` - The measure to retrieve the type from.
 */
struct FfiResult_____c_char opendp_measures__measure_type(struct AnyMeasure *this_);

/**
 * Get the distance type of a `measure`.
 *
 * # Arguments
 * * `this` - The measure to retrieve the distance type from.
 */
struct FfiResult_____c_char opendp_measures__measure_distance_type(struct AnyMeasure *this_);

/**
 * Privacy measure used to define $\epsilon$-pure differential privacy.
 *
 * In the following proof definition, $d$ corresponds to $\epsilon$ when also quantified over all adjacent datasets.
 * That is, $\epsilon$ is the greatest possible $d$
 * over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
 * $M(\cdot)$ is a measurement (commonly known as a mechanism).
 * The measurement's input metric defines the notion of adjacency,
 * and the measurement's input domain defines the set of possible datasets.
 *
 * # Proof Definition
 *
 * For any two distributions $Y, Y'$ and any non-negative $d$,
 * $Y, Y'$ are $d$-close under the max divergence measure whenever
 *
 * $D_\infty(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S]}{\Pr[Y' \in S]} \Big] \leq d$.
 */
struct FfiResult_____AnyMeasure opendp_measures__max_divergence(void);

/**
 * Privacy measure used to define $\epsilon(\delta)$-approximate differential privacy.
 *
 * In the following proof definition, $d$ corresponds to a privacy profile when also quantified over all adjacent datasets.
 * That is, a privacy profile $\epsilon(\delta)$ is no smaller than $d(\delta)$ for all possible choices of $\delta$,
 * and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
 * $M(\cdot)$ is a measurement (commonly known as a mechanism).
 * The measurement's input metric defines the notion of adjacency,
 * and the measurement's input domain defines the set of possible datasets.
 *
 * The distance $d$ is of type PrivacyProfile, so it can be invoked with an $\epsilon$
 * to retrieve the corresponding $\delta$.
 *
 * # Proof Definition
 *
 * For any two distributions $Y, Y'$ and any curve $d(\cdot)$,
 * $Y, Y'$ are $d$-close under the smoothed max divergence measure whenever,
 * for any choice of non-negative $\epsilon$, and $\delta = d(\epsilon)$,
 *
 * $D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon$.
 *
 * Note that $\epsilon$ and $\delta$ are not privacy parameters $\epsilon$ and $\delta$ until quantified over all adjacent datasets,
 * as is done in the definition of a measurement.
 */
struct FfiResult_____AnyMeasure opendp_measures__smoothed_max_divergence(void);

/**
 * Privacy measure used to define $(\epsilon, \delta)$-approximate differential privacy.
 *
 * In the following definition, $d$ corresponds to $(\epsilon, \delta)$ when also quantified over all adjacent datasets.
 * That is, $(\epsilon, \delta)$ is no smaller than $d$ (by product ordering),
 * over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
 * $M(\cdot)$ is a measurement (commonly known as a mechanism).
 * The measurement's input metric defines the notion of adjacency,
 * and the measurement's input domain defines the set of possible datasets.
 *
 * # Proof Definition
 *
 * For any two distributions $Y, Y'$ and any 2-tuple $d$ of non-negative numbers $\epsilon$ and $\delta$,
 * $Y, Y'$ are $d$-close under the fixed smoothed max divergence measure whenever
 *
 * $D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon$.
 *
 * Note that this $\epsilon$ and $\delta$ are not privacy parameters $\epsilon$ and $\delta$ until quantified over all adjacent datasets,
 * as is done in the definition of a measurement.
 */
struct FfiResult_____AnyMeasure opendp_measures__fixed_smoothed_max_divergence(void);

struct FfiResult_____AnyMeasure opendp_measures__approximate(const struct AnyMeasure *measure);

/**
 * Retrieve the inner privacy measure of an approximate privacy measure.
 *
 * # Arguments
 * * `privacy_measure` - The privacy measure to inspect
 */
struct FfiResult_____AnyMeasure opendp_measures___approximate_divergence_get_inner_measure(const struct AnyMeasure *privacy_measure);

/**
 * Privacy measure used to define $\rho$-zero concentrated differential privacy.
 *
 * In the following proof definition, $d$ corresponds to $\rho$ when also quantified over all adjacent datasets.
 * That is, $\rho$ is the greatest possible $d$
 * over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
 * $M(\cdot)$ is a measurement (commonly known as a mechanism).
 * The measurement's input metric defines the notion of adjacency,
 * and the measurement's input domain defines the set of possible datasets.
 *
 * # Proof Definition
 *
 * For any two distributions $Y, Y'$ and any non-negative $d$,
 * $Y, Y'$ are $d$-close under the zero-concentrated divergence measure if,
 * for every possible choice of $\alpha \in (1, \infty)$,
 *
 * $D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d \cdot \alpha$.
 */
struct FfiResult_____AnyMeasure opendp_measures__zero_concentrated_divergence(void);

/**
 * Privacy measure used to define $\epsilon(\alpha)$-Rényi differential privacy.
 *
 * In the following proof definition, $d$ corresponds to an RDP curve when also quantified over all adjacent datasets.
 * That is, an RDP curve $\epsilon(\alpha)$ is no smaller than $d(\alpha)$ for any possible choices of $\alpha$,
 * and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
 * $M(\cdot)$ is a measurement (commonly known as a mechanism).
 * The measurement's input metric defines the notion of adjacency,
 * and the measurement's input domain defines the set of possible datasets.
 *
 * # Proof Definition
 *
 * For any two distributions $Y, Y'$ and any curve $d$,
 * $Y, Y'$ are $d$-close under the Rényi divergence measure if,
 * for any given $\alpha \in (1, \infty)$,
 *
 * $D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d(\alpha)$
 *
 * Note that this $\epsilon$ and $\alpha$ are not privacy parameters $\epsilon$ and $\alpha$ until quantified over all adjacent datasets,
 * as is done in the definition of a measurement.
 */
struct FfiResult_____AnyMeasure opendp_measures__renyi_divergence(void);

/**
 * Privacy measure with meaning defined by an OpenDP Library user (you).
 *
 * Any two instances of UserDivergence are equal if their string descriptors are equal.
 *
 * # Proof definition
 *
 * For any two distributions $Y, Y'$ and any $d$,
 * $Y, Y'$ are $d$-close under the user divergence measure ($D_U$) if,
 *
 * $D_U(Y, Y') \le d$.
 *
 * For $D_U$ to qualify as a privacy measure, then for any postprocessing function $f$,
 * $D_U(Y, Y') \ge D_U(f(Y), f(Y'))$.
 *
 * # Arguments
 * * `descriptor` - A string description of the privacy measure.
 *
 * # Why honest-but-curious?
 * The essential requirement of a privacy measure is that it is closed under postprocessing.
 * Your privacy measure `D` must satisfy that, for any pure function `f` and any two distributions `Y, Y'`, then $D(Y, Y') \ge D(f(Y), f(Y'))$.
 *
 * Beyond this, you should also consider whether your privacy measure can be used to provide meaningful privacy guarantees to your privacy units.
 */
struct FfiResult_____AnyMeasure opendp_measures__user_divergence(char *descriptor);

struct FfiResult_____AnyObject opendp_measures__new_privacy_profile(const struct CallbackFn *curve);

/**
 * Internal function. Free the memory associated with `this`.
 */
struct FfiResult_____c_void opendp_metrics___metric_free(struct AnyMetric *this_);

/**
 * Check whether two metrics are equal.
 *
 * # Arguments
 * * `left` - Metric to compare.
 * * `right` - Metric to compare.
 */
struct FfiResult_____c_bool opendp_metrics___metric_equal(struct AnyMetric *left,
                                                          const struct AnyMetric *right);

/**
 * Debug a `metric`.
 *
 * # Arguments
 * * `this` - The metric to debug (stringify).
 */
struct FfiResult_____c_char opendp_metrics__metric_debug(struct AnyMetric *this_);

/**
 * Get the type of a `metric`.
 *
 * # Arguments
 * * `this` - The metric to retrieve the type from.
 */
struct FfiResult_____c_char opendp_metrics__metric_type(struct AnyMetric *this_);

/**
 * Get the distance type of a `metric`.
 *
 * # Arguments
 * * `this` - The metric to retrieve the distance type from.
 */
struct FfiResult_____c_char opendp_metrics__metric_distance_type(struct AnyMetric *this_);

/**
 * Construct an instance of the `SymmetricDistance` metric.
 */
struct FfiResult_____AnyMetric opendp_metrics__symmetric_distance(void);

/**
 * Construct an instance of the `InsertDeleteDistance` metric.
 */
struct FfiResult_____AnyMetric opendp_metrics__insert_delete_distance(void);

/**
 * Construct an instance of the `ChangeOneDistance` metric.
 */
struct FfiResult_____AnyMetric opendp_metrics__change_one_distance(void);

/**
 * Construct an instance of the `HammingDistance` metric.
 */
struct FfiResult_____AnyMetric opendp_metrics__hamming_distance(void);

struct FfiResult_____AnyMetric opendp_metrics__absolute_distance(const char *T);

struct FfiResult_____AnyMetric opendp_metrics__l1_distance(const char *T);

struct FfiResult_____AnyMetric opendp_metrics__l2_distance(const char *T);

/**
 * Construct an instance of the `DiscreteDistance` metric.
 */
struct FfiResult_____AnyMetric opendp_metrics__discrete_distance(void);

struct FfiResult_____AnyMetric opendp_metrics__l01inf_distance(const struct AnyMetric *metric);

struct FfiResult_____AnyMetric opendp_metrics__l02inf_distance(const struct AnyMetric *metric);

struct FfiResult_____AnyMetric opendp_metrics__linf_distance(c_bool monotonic, const char *T);

/**
 * Construct a new UserDistance.
 * Any two instances of an UserDistance are equal if their string descriptors are equal.
 *
 * # Arguments
 * * `identifier` - A string description of the metric.
 * * `descriptor` - Additional constraints on the domain.
 *
 * # Why honest-but-curious?
 * Your definition of `d` must satisfy the requirements of a pseudo-metric:
 *
 * 1. for any $x$, $d(x, x) = 0$
 * 2. for any $x, y$, $d(x, y) \ge 0$ (non-negativity)
 * 3. for any $x, y$, $d(x, y) = d(y, x)$ (symmetry)
 * 4. for any $x, y, z$, $d(x, z) \le d(x, y) + d(y, z)$ (triangle inequality)
 */
struct FfiResult_____AnyMetric opendp_metrics__user_distance(char *identifier,
                                                             struct ExtrinsicObject *descriptor);

/**
 * Retrieve the descriptor value stored in an extrinsic metric.
 *
 * # Arguments
 * * `metric` - The ExtrinsicDistance to extract the descriptor from
 */
struct FfiResult_____ExtrinsicObject opendp_metrics___extrinsic_metric_descriptor(struct AnyMetric *metric);

/**
 * Construct an instance of the `SymmetricIdDistance` metric.
 */
struct FfiResult_____AnyMetric opendp_metrics__symmetric_id_distance(const struct AnyObject *identifier);

/**
 * Retrieve the identifier of a `SymmetricIdDistance` metric.
 */
struct FfiResult_____AnyObject opendp_metrics___symmetric_id_distance_get_identifier(const struct AnyMetric *metric);

/**
 * Construct an instance of the `ChangeOneIdDistance` metric.
 */
struct FfiResult_____AnyMetric opendp_metrics__change_one_id_distance(const struct AnyObject *identifier);

/**
 * Retrieve the identifier of a `ChangeOneIdDistance` metric.
 */
struct FfiResult_____AnyObject opendp_metrics___change_one_id_distance_get_identifier(const struct AnyMetric *metric);

/**
 * `frame_distance` is a higher-order metric that contains multiple distance bounds for different groupings of data.
 */
struct FfiResult_____AnyMetric opendp_metrics__frame_distance(const struct AnyMetric *inner_metric);

/**
 * Retrieve the inner metric of a `FrameDistance` metric.
 */
struct FfiResult_____AnyMetric opendp_metrics___frame_distance_get_inner_metric(const struct AnyMetric *metric);

/**
 * Infer a bound when grouping by `by`.
 */
struct FfiResult_____AnyObject opendp_metrics___get_bound(const struct AnyObject *bounds,
                                                          const struct AnyObject *by);

struct FfiResult_____AnyTransformation opendp_transformations__make_stable_lazyframe(const struct AnyDomain *input_domain,
                                                                                     const struct AnyMetric *input_metric,
                                                                                     const struct AnyObject *lazyframe,
                                                                                     const char *MO);

struct FfiResult_____AnyTransformation opendp_transformations__make_stable_expr(const struct AnyDomain *input_domain,
                                                                                const struct AnyMetric *input_metric,
                                                                                const struct AnyObject *expr);

struct FfiResult_____AnyTransformation opendp_transformations__make_sized_bounded_covariance(unsigned int size,
                                                                                             const struct AnyObject *bounds_0,
                                                                                             const struct AnyObject *bounds_1,
                                                                                             unsigned int ddof,
                                                                                             const char *S);

struct FfiResult_____AnyTransformation opendp_transformations__make_b_ary_tree(const struct AnyDomain *input_domain,
                                                                               const struct AnyMetric *input_metric,
                                                                               uint32_t leaf_count,
                                                                               uint32_t branching_factor);

uint32_t opendp_transformations__choose_branching_factor(uint32_t size_guess);

struct FfiResult_____AnyFunction opendp_transformations__make_consistent_b_ary_tree(uint32_t branching_factor,
                                                                                    const char *TIA,
                                                                                    const char *TOA);

struct FfiResult_____AnyTransformation opendp_transformations__make_df_cast_default(const struct AnyDomain *input_domain,
                                                                                    const struct AnyMetric *input_metric,
                                                                                    const struct AnyObject *column_name,
                                                                                    const char *TIA,
                                                                                    const char *TOA);

struct FfiResult_____AnyTransformation opendp_transformations__make_df_is_equal(const struct AnyDomain *input_domain,
                                                                                const struct AnyMetric *input_metric,
                                                                                const struct AnyObject *column_name,
                                                                                const struct AnyObject *value,
                                                                                const char *TIA);

struct FfiResult_____AnyTransformation opendp_transformations__make_split_lines(void);

struct FfiResult_____AnyTransformation opendp_transformations__make_split_records(const char *separator);

struct FfiResult_____AnyTransformation opendp_transformations__make_create_dataframe(const struct AnyObject *col_names,
                                                                                     const char *K);

struct FfiResult_____AnyTransformation opendp_transformations__make_split_dataframe(const char *separator,
                                                                                    const struct AnyObject *col_names,
                                                                                    const char *K);

struct FfiResult_____AnyTransformation opendp_transformations__make_select_column(const struct AnyObject *key,
                                                                                  const char *K,
                                                                                  const char *TOA);

struct FfiResult_____AnyTransformation opendp_transformations__make_subset_by(const struct AnyObject *indicator_column,
                                                                              const struct AnyObject *keep_columns,
                                                                              const char *TK);

struct FfiResult_____AnyTransformation opendp_transformations__make_quantile_score_candidates(const struct AnyDomain *input_domain,
                                                                                              const struct AnyMetric *input_metric,
                                                                                              const struct AnyObject *candidates,
                                                                                              double alpha);

struct FfiResult_____AnyTransformation opendp_transformations__make_identity(const struct AnyDomain *domain,
                                                                             const struct AnyMetric *metric);

struct FfiResult_____AnyTransformation opendp_transformations__make_is_equal(const struct AnyDomain *input_domain,
                                                                             const struct AnyMetric *input_metric,
                                                                             const struct AnyObject *value);

struct FfiResult_____AnyTransformation opendp_transformations__make_is_null(const struct AnyDomain *input_domain,
                                                                            const struct AnyMetric *input_metric);

struct FfiResult_____AnyTransformation opendp_transformations__make_sum(const struct AnyDomain *input_domain,
                                                                        const struct AnyMetric *input_metric);

struct FfiResult_____AnyTransformation opendp_transformations__make_sized_bounded_int_checked_sum(unsigned int size,
                                                                                                  const struct AnyObject *bounds,
                                                                                                  const char *T);

struct FfiResult_____AnyTransformation opendp_transformations__make_bounded_int_monotonic_sum(const struct AnyObject *bounds,
                                                                                              const char *T);

struct FfiResult_____AnyTransformation opendp_transformations__make_sized_bounded_int_monotonic_sum(unsigned int size,
                                                                                                    const struct AnyObject *bounds,
                                                                                                    const char *T);

struct FfiResult_____AnyTransformation opendp_transformations__make_bounded_int_ordered_sum(const struct AnyObject *bounds,
                                                                                            const char *T);

struct FfiResult_____AnyTransformation opendp_transformations__make_sized_bounded_int_ordered_sum(unsigned int size,
                                                                                                  const struct AnyObject *bounds,
                                                                                                  const char *T);

struct FfiResult_____AnyTransformation opendp_transformations__make_bounded_int_split_sum(const struct AnyObject *bounds,
                                                                                          const char *T);

struct FfiResult_____AnyTransformation opendp_transformations__make_sized_bounded_int_split_sum(unsigned int size,
                                                                                                const struct AnyObject *bounds,
                                                                                                const char *T);

struct FfiResult_____AnyTransformation opendp_transformations__make_bounded_float_checked_sum(unsigned int size_limit,
                                                                                              const struct AnyObject *bounds,
                                                                                              const char *S);

struct FfiResult_____AnyTransformation opendp_transformations__make_sized_bounded_float_checked_sum(unsigned int size,
                                                                                                    const struct AnyObject *bounds,
                                                                                                    const char *S);

struct FfiResult_____AnyTransformation opendp_transformations__make_bounded_float_ordered_sum(unsigned int size_limit,
                                                                                              const struct AnyObject *bounds,
                                                                                              const char *S);

struct FfiResult_____AnyTransformation opendp_transformations__make_sized_bounded_float_ordered_sum(unsigned int size,
                                                                                                    const struct AnyObject *bounds,
                                                                                                    const char *S);

struct FfiResult_____AnyTransformation opendp_transformations__make_sum_of_squared_deviations(const struct AnyDomain *input_domain,
                                                                                              const struct AnyMetric *input_metric,
                                                                                              const char *S);

struct FfiResult_____AnyTransformation opendp_transformations__make_count(const struct AnyDomain *input_domain,
                                                                          const struct AnyMetric *input_metric,
                                                                          const char *TO);

struct FfiResult_____AnyTransformation opendp_transformations__make_count_distinct(const struct AnyDomain *input_domain,
                                                                                   const struct AnyMetric *input_metric,
                                                                                   const char *TO);

struct FfiResult_____AnyTransformation opendp_transformations__make_count_by_categories(const struct AnyDomain *input_domain,
                                                                                        const struct AnyMetric *input_metric,
                                                                                        const struct AnyObject *categories,
                                                                                        c_bool null_category,
                                                                                        const char *MO,
                                                                                        const char *TO);

struct FfiResult_____AnyTransformation opendp_transformations__make_count_by(const struct AnyDomain *input_domain,
                                                                             const struct AnyMetric *input_metric,
                                                                             const char *TV);

struct FfiResult_____AnyFunction opendp_transformations__make_cdf(const char *TA);

struct FfiResult_____AnyFunction opendp_transformations__make_quantiles_from_counts(const struct AnyObject *bin_edges,
                                                                                    const struct AnyObject *alphas,
                                                                                    const char *interpolation,
                                                                                    const char *TA,
                                                                                    const char *F);

struct FfiResult_____AnyTransformation opendp_transformations__make_mean(const struct AnyDomain *input_domain,
                                                                         const struct AnyMetric *input_metric);

struct FfiResult_____AnyTransformation opendp_transformations__make_variance(const struct AnyDomain *input_domain,
                                                                             const struct AnyMetric *input_metric,
                                                                             unsigned int ddof,
                                                                             const char *S);

struct FfiResult_____AnyTransformation opendp_transformations__make_impute_uniform_float(const struct AnyDomain *input_domain,
                                                                                         const struct AnyMetric *input_metric,
                                                                                         const struct AnyObject *bounds);

struct FfiResult_____AnyTransformation opendp_transformations__make_impute_constant(const struct AnyDomain *input_domain,
                                                                                    const struct AnyMetric *input_metric,
                                                                                    const struct AnyObject *constant);

struct FfiResult_____AnyTransformation opendp_transformations__make_drop_null(const struct AnyDomain *input_domain,
                                                                              const struct AnyMetric *input_metric);

struct FfiResult_____AnyTransformation opendp_transformations__make_find(const struct AnyDomain *input_domain,
                                                                         const struct AnyMetric *input_metric,
                                                                         const struct AnyObject *categories);

struct FfiResult_____AnyTransformation opendp_transformations__make_find_bin(const struct AnyDomain *input_domain,
                                                                             const struct AnyMetric *input_metric,
                                                                             const struct AnyObject *edges);

struct FfiResult_____AnyTransformation opendp_transformations__make_index(const struct AnyDomain *input_domain,
                                                                          const struct AnyMetric *input_metric,
                                                                          const struct AnyObject *categories,
                                                                          const struct AnyObject *null,
                                                                          const char *TOA);

struct FfiResult_____AnyTransformation opendp_transformations__make_lipschitz_float_mul(const struct AnyDomain *input_domain,
                                                                                        const struct AnyMetric *input_metric,
                                                                                        const void *constant,
                                                                                        const struct AnyObject *bounds);

/**
 * Construct a Transformation from user-defined callbacks.
 *
 * # Arguments
 * * `input_domain` - A domain describing the set of valid inputs for the function.
 * * `input_metric` - The metric from which distances between adjacent inputs are measured.
 * * `output_domain` - A domain describing the set of valid outputs of the function.
 * * `output_metric` - The metric from which distances between outputs of adjacent inputs are measured.
 * * `function` - A function mapping data from `input_domain` to `output_domain`.
 * * `stability_map` - A function mapping distances from `input_metric` to `output_metric`.
 *
 * # Why honest-but-curious?
 *
 * This constructor only returns a valid transformation if for every pair of elements $x, x'$ in `input_domain`,
 * and for every pair `(d_in, d_out)`,
 * where `d_in` has the associated type for `input_metric` and `d_out` has the associated type for `output_metric`,
 * if $x, x'$ are `d_in`-close under `input_metric`, `stability_map(d_in)` does not raise an exception,
 * and `stability_map(d_in) <= d_out`,
 * then `function(x), function(x')` are d_out-close under `output_metric`.
 *
 * In addition, for every element $x$ in `input_domain`, `function(x)` is a member of `output_domain` or raises a data-independent runtime exception.
 *
 * In addition, `function` must not have side-effects, and `stability_map` must be a pure function.
 */
struct FfiResult_____AnyTransformation opendp_transformations__make_user_transformation(const struct AnyDomain *input_domain,
                                                                                        const struct AnyMetric *input_metric,
                                                                                        const struct AnyDomain *output_domain,
                                                                                        const struct AnyMetric *output_metric,
                                                                                        const struct CallbackFn *function,
                                                                                        const struct CallbackFn *stability_map);

struct FfiResult_____AnyTransformation opendp_transformations__make_clamp(const struct AnyDomain *input_domain,
                                                                          const struct AnyMetric *input_metric,
                                                                          const struct AnyObject *bounds);

struct FfiResult_____AnyTransformation opendp_transformations__make_cast(const struct AnyDomain *input_domain,
                                                                         const struct AnyMetric *input_metric,
                                                                         const char *TOA);

struct FfiResult_____AnyTransformation opendp_transformations__make_cast_default(const struct AnyDomain *input_domain,
                                                                                 const struct AnyMetric *input_metric,
                                                                                 const char *TOA);

struct FfiResult_____AnyTransformation opendp_transformations__make_cast_inherent(const struct AnyDomain *input_domain,
                                                                                  const struct AnyMetric *input_metric,
                                                                                  const char *TOA);

struct FfiResult_____AnyTransformation opendp_transformations__make_ordered_random(const struct AnyDomain *input_domain,
                                                                                   const struct AnyMetric *input_metric);

struct FfiResult_____AnyTransformation opendp_transformations__make_unordered(const struct AnyDomain *input_domain,
                                                                              const struct AnyMetric *input_metric);

struct FfiResult_____AnyTransformation opendp_transformations__make_metric_bounded(const struct AnyDomain *input_domain,
                                                                                   const struct AnyMetric *input_metric);

struct FfiResult_____AnyTransformation opendp_transformations__make_metric_unbounded(const struct AnyDomain *input_domain,
                                                                                     const struct AnyMetric *input_metric);

struct FfiResult_____AnyTransformation opendp_transformations__make_resize(const struct AnyDomain *input_domain,
                                                                           const struct AnyMetric *input_metric,
                                                                           unsigned int size,
                                                                           const struct AnyObject *constant,
                                                                           const char *MO);
