// Manually-written type definitions that cbindgen was unable to write correctly.
// The respective rust code has annotations that make cbindgen ignore them

/**
 * A Transformation with all generic types filled by Any types. This is the type of Transformation
 * passed back and forth over FFI.
 */
typedef struct AnyTransformation AnyTransformation;

/**
 * A Measurement with all generic types filled by Any types. This is the type of Measurements
 * passed back and forth over FFI.
 */
typedef struct AnyMeasurement AnyMeasurement;
