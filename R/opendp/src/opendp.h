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
 * A mathematical function which maps values from an input [`Domain`] to an output [`Domain`].
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
typedef struct Measurement_AnyDomain__AnyObject__AnyMetric__AnyMeasure Measurement_AnyDomain__AnyObject__AnyMetric__AnyMeasure;

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
typedef struct Transformation_AnyDomain__AnyDomain__AnyMetric__AnyMetric Transformation_AnyDomain__AnyDomain__AnyMetric__AnyMetric;

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
typedef struct Measurement_AnyDomain__AnyObject__AnyMetric__AnyMeasure AnyMeasurement;

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
typedef struct Transformation_AnyDomain__AnyDomain__AnyMetric__AnyMetric AnyTransformation;

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

typedef struct FfiResult_____AnyObject *(*CallbackFn)(const struct AnyObject*);

typedef struct FfiResult_____AnyObject *(*TransitionFn)(const struct AnyObject*, c_bool);

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

typedef bool (*RefCountFn)(const void*, bool);

typedef struct ExtrinsicObject {
  const void *ptr;
  RefCountFn count;
} ExtrinsicObject;

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

struct FfiResult_____AnyObject opendp_accuracy__describe_polars_measurement_accuracy(const AnyMeasurement *measurement,
                                                                                     const struct AnyObject *alpha);

struct FfiResult_____AnyMeasurement opendp_combinators__make_population_amplification(const AnyMeasurement *measurement,
                                                                                      unsigned int population_size);

struct FfiResult_____AnyMeasurement opendp_combinators__make_chain_mt(const AnyMeasurement *measurement1,
                                                                      const AnyTransformation *transformation0);

struct FfiResult_____AnyTransformation opendp_combinators__make_chain_tt(const AnyTransformation *transformation1,
                                                                         const AnyTransformation *transformation0);

struct FfiResult_____AnyMeasurement opendp_combinators__make_chain_pm(const AnyFunction *postprocess1,
                                                                      const AnyMeasurement *measurement0);

struct FfiResult_____AnyMeasurement opendp_combinators__make_basic_composition(const struct AnyObject *measurements);

struct FfiResult_____AnyMeasurement opendp_combinators__make_sequential_composition(const struct AnyDomain *input_domain,
                                                                                    const struct AnyMetric *input_metric,
                                                                                    const struct AnyMeasure *output_measure,
                                                                                    const struct AnyObject *d_in,
                                                                                    const struct AnyObject *d_mids);

struct FfiResult_____AnyMeasurement opendp_combinators__make_zCDP_to_approxDP(const AnyMeasurement *measurement);

struct FfiResult_____AnyMeasurement opendp_combinators__make_pureDP_to_fixed_approxDP(const AnyMeasurement *measurement);

struct FfiResult_____AnyMeasurement opendp_combinators__make_pureDP_to_zCDP(const AnyMeasurement *measurement);

struct FfiResult_____AnyMeasurement opendp_combinators__make_fix_delta(const AnyMeasurement *measurement,
                                                                       double delta);

/**
 * Internal function. Free the memory associated with `error`.
 *
 * # Returns
 * A boolean, where true indicates successful free
 */
bool opendp_core___error_free(struct FfiError *this_);

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
 * Get the input (carrier) data type of `this`.
 *
 * # Arguments
 * * `this` - The measurement to retrieve the type from.
 */
struct FfiResult_____c_char opendp_core__measurement_input_carrier_type(AnyMeasurement *this_);

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

struct FfiResult_____AnyFunction opendp_core__new_function(CallbackFn function, const char *TO);

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
 * Invoke the `queryable` with `query`. Returns a differentially private release.
 *
 * # Arguments
 * * `queryable` - Queryable to eval.
 * * `query` - Input data to supply to the measurement. A member of the measurement's input domain.
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

struct FfiResult_____AnyObject opendp_core__new_queryable(TransitionFn transition,
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
 * An AnyObject that contains the data in `slice`.
 * The AnyObject also captures rust type information.
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
 * Internal function. Use an SMDCurve to find epsilon at a given `delta`.
 *
 * # Arguments
 * * `curve` - The SMDCurve.
 * * `delta` - What to fix delta to compute epsilon.
 *
 * # Returns
 * Epsilon at a given `delta`.
 */
struct FfiResult_____AnyObject opendp_data__smd_curve_epsilon(const struct AnyObject *curve,
                                                              double delta);

/**
 * Internal function. Use an SMDCurve to find beta at a given `alpha`.
 *
 * # Arguments
 * * `curve` - The SMDCurve.
 * * `alpha` - What to fix alpha to compute beta.
 *
 * # Returns
 * Beta at a given `alpha`.
 */
struct FfiResult_____AnyObject opendp_data__smd_curve_beta(const struct AnyObject *curve,
                                                           double alpha);

/**
 * Internal function. Use an SMDCurve to construct a piecewise linear supporting function.
 *
 * # Arguments
 * * `curve` - The SMDCurve.
 * * `num_approximations` - Number of supporting functions to create.
 *
 * # Returns
 * `α(β)` tradeoff function.
 */
struct FfiResult_____AnyFunction opendp_data__smd_curve_tradeoff(const struct AnyObject *curve,
                                                                 const struct AnyObject *num_approximations);

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
 * Requires ``honest-but-curious`` because the privacy guarantees only apply if:
 * 1. The LazyFrame (compute plan) is only ever executed once.
 * 2. The analyst does not observe ordering of rows in the output. To ensure this, shuffle the output.
 *
 * # Arguments
 * * `onceframe` - The queryable holding a LazyFrame.
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
struct FfiResult_____c_bool opendp_domains__member(struct AnyDomain *this_,
                                                   const struct AnyObject *val);

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
                                                           c_bool nullable,
                                                           const char *T);

struct FfiResult_____AnyDomain opendp_domains__option_domain(const struct AnyDomain *element_domain,
                                                             const char *D);

/**
 * Construct an instance of `VectorDomain`.
 *
 * # Arguments
 * * `atom_domain` - The inner domain.
 */
struct FfiResult_____AnyDomain opendp_domains__vector_domain(const struct AnyDomain *atom_domain,
                                                             const struct AnyObject *size);

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
 */
struct FfiResult_____AnyDomain opendp_domains__user_domain(char *identifier,
                                                           CallbackFn member,
                                                           struct ExtrinsicObject *descriptor);

/**
 * Retrieve the descriptor value stored in a user domain.
 *
 * # Arguments
 * * `domain` - The UserDomain to extract the descriptor from
 */
struct FfiResult_____ExtrinsicObject opendp_domains___user_domain_descriptor(struct AnyDomain *domain);

/**
 * Construct an instance of `LazyFrameDomain`.
 *
 * # Arguments
 * * `series_domains` - Domain of each series in the lazyframe.
 */
struct FfiResult_____AnyDomain opendp_domains__lazyframe_domain(struct AnyObject *series_domains);

struct FfiResult_____AnyObject opendp_domains___lazyframe_from_domain(struct AnyDomain *domain);

struct FfiResult_____AnyDomain opendp_domains__with_margin(struct AnyDomain *frame_domain,
                                                           struct AnyObject *by,
                                                           struct AnyObject *max_partition_length,
                                                           struct AnyObject *max_num_partitions,
                                                           struct AnyObject *max_partition_contributions,
                                                           struct AnyObject *max_influenced_partitions,
                                                           char *public_info);

struct FfiResult_____AnyDomain opendp_domains__series_domain(char *name,
                                                             const struct AnyDomain *element_domain);

/**
 * Construct an ExprDomain from a LazyFrameDomain.
 *
 * Must pass either `context` or `grouping_columns`.
 *
 * # Arguments
 * * `lazyframe_domain` - the domain of the LazyFrame to be constructed
 * * `grouping_columns` - set when creating an expression that aggregates
 */
struct FfiResult_____AnyDomain opendp_domains__expr_domain(const struct AnyDomain *lazyframe_domain,
                                                           const struct AnyObject *grouping_columns);

struct FfiResult_____AnyMeasurement opendp_measurements__make_gaussian(const struct AnyDomain *input_domain,
                                                                       const struct AnyMetric *input_metric,
                                                                       double scale,
                                                                       const int32_t *k,
                                                                       const char *MO);

struct FfiResult_____AnyMeasurement opendp_measurements__make_geometric(const struct AnyDomain *input_domain,
                                                                        const struct AnyMetric *input_metric,
                                                                        double scale,
                                                                        const struct AnyObject *bounds);

struct FfiResult_____AnyMeasurement opendp_measurements__make_report_noisy_max_gumbel(const struct AnyDomain *input_domain,
                                                                                      const struct AnyMetric *input_metric,
                                                                                      double scale,
                                                                                      const char *optimize);

struct FfiResult_____AnyMeasurement opendp_measurements__make_laplace(const struct AnyDomain *input_domain,
                                                                      const struct AnyMetric *input_metric,
                                                                      double scale,
                                                                      const int32_t *k);

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
                                                                               CallbackFn function,
                                                                               CallbackFn privacy_map,
                                                                               const char *TO);

struct FfiResult_____AnyMeasurement opendp_measurements__make_laplace_threshold(const struct AnyDomain *input_domain,
                                                                                const struct AnyMetric *input_metric,
                                                                                double scale,
                                                                                const void *threshold,
                                                                                long k);

struct FfiResult_____AnyMeasurement opendp_measurements__make_randomized_response_bool(double prob,
                                                                                       c_bool constant_time);

struct FfiResult_____AnyMeasurement opendp_measurements__make_randomized_response(const struct AnyObject *categories,
                                                                                  double prob,
                                                                                  c_bool constant_time,
                                                                                  const char *T);

struct FfiResult_____AnyMeasurement opendp_measurements__make_alp_queryable(const struct AnyDomain *input_domain,
                                                                            const struct AnyMetric *input_metric,
                                                                            const void *scale,
                                                                            const void *total_limit,
                                                                            const void *value_limit,
                                                                            const unsigned int *size_factor,
                                                                            const void *alpha,
                                                                            const char *CO);

/**
 * Internal function. Free the memory associated with `this`.
 */
struct FfiResult_____c_void opendp_measures___measure_free(struct AnyMeasure *this_);

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

struct FfiResult_____AnyMeasure opendp_measures__max_divergence(const char *T);

struct FfiResult_____AnyMeasure opendp_measures__smoothed_max_divergence(const char *T);

struct FfiResult_____AnyMeasure opendp_measures__fixed_smoothed_max_divergence(const char *T);

struct FfiResult_____AnyMeasure opendp_measures__zero_concentrated_divergence(const char *T);

/**
 * Construct a new UserDivergence.
 * Any two instances of an UserDivergence are equal if their string descriptors are equal.
 *
 * # Arguments
 * * `descriptor` - A string description of the privacy measure.
 */
struct FfiResult_____AnyMeasure opendp_measures__user_divergence(char *descriptor);

/**
 * Internal function. Free the memory associated with `this`.
 */
struct FfiResult_____c_void opendp_metrics___metric_free(struct AnyMetric *this_);

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

struct FfiResult_____AnyMetric opendp_metrics__partition_distance(const struct AnyMetric *metric);

struct FfiResult_____AnyMetric opendp_metrics__linf_distance(c_bool monotonic, const char *T);

/**
 * Construct a new UserDistance.
 * Any two instances of an UserDistance are equal if their string descriptors are equal.
 *
 * # Arguments
 * * `descriptor` - A string description of the metric.
 */
struct FfiResult_____AnyMetric opendp_metrics__user_distance(char *descriptor);

struct FfiResult_____AnyTransformation opendp_transformations__make_stable_lazyframe(const struct AnyDomain *input_domain,
                                                                                     const struct AnyMetric *input_metric,
                                                                                     const struct AnyObject *lazyframe);

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
                                                                             const char *MO,
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

struct FfiResult_____AnyTransformation opendp_transformations__make_lipschitz_float_mul(const void *constant,
                                                                                        const struct AnyObject *bounds,
                                                                                        const char *D,
                                                                                        const char *M);

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
 */
struct FfiResult_____AnyTransformation opendp_transformations__make_user_transformation(const struct AnyDomain *input_domain,
                                                                                        const struct AnyMetric *input_metric,
                                                                                        const struct AnyDomain *output_domain,
                                                                                        const struct AnyMetric *output_metric,
                                                                                        CallbackFn function,
                                                                                        CallbackFn stability_map);

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
