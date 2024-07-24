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
#include "convert_elements.h"

const char *ATOM_TYPES[] = {"u32", "u64", "i32", "i64", "f32", "f64", "usize", "bool", "String"};

NORET void error_unknown_type(const char *lhs, const char *rhs)
{
    error("%s %s", lhs, rhs);
}

#define CharPtr(x) (const char *)CHAR(STRING_ELT(x, 0))

// extracts origin from a runtime type
// f("Vec<i32>") -> "Vec"
SEXP get_origin(SEXP rust_type)
{
    return VECTOR_ELT(rust_type, 0);
}

// extracts arguments from a runtime type
// f("A<B, C, D>") -> "B, C, D"
SEXP get_args(SEXP rust_type)
{
    return VECTOR_ELT(rust_type, 1);
}

SEXP get_private_func(const char *func_name)
{
    SEXP namespace = PROTECT(mkString("opendp"));
    SEXP func_name_sexp = PROTECT(mkString(func_name));
    SEXP get_func_call = PROTECT(lang3(install("getFromNamespace"), func_name_sexp, namespace));
    int errorOccurred;
    SEXP func = PROTECT(R_tryEval(get_func_call, R_GlobalEnv, &errorOccurred));
    if (errorOccurred)
        error("failed to get from namespace");
    UNPROTECT(4);
    return func;
}

unsigned char str_equal(const char *str1, const char *str2)
{
    return strcmp(str1, str2) == 0;
}

unsigned char is_in(const char *str, const char **arr)
{
    for (int i = 0; i <= sizeof(arr); i++)
        if (str_equal(str, arr[i]))
            return 1;
    return 0;
}

SEXP extract_error(FfiError *err)
{
    int msg_len = strlen(err->variant) + strlen(err->message) + 6;
    char *msg = (char *)R_alloc(1, msg_len * sizeof(char));
    snprintf(msg, msg_len, "[%s] : %s", err->variant, err->message);

    if (str_equal(err->backtrace, "backtrace disabled"))
        error("%s", msg);
    else
    {
        int msg_len_bt = strlen(msg) + strlen(err->backtrace) + 2;
        char *msg_backtrace = (char *)R_alloc(1, msg_len_bt * sizeof(char));
        snprintf(msg_backtrace, msg_len_bt, "%s\n%s", msg, err->backtrace);
        error("%s", msg_backtrace);
    }

    return (R_NilValue);
}

const char *sexp_to_charptr(SEXP type_name)
{
    PROTECT(type_name = AS_CHARACTER(type_name));
    const char *c_type_name = CharPtr(type_name);
    UNPROTECT(1);
    return c_type_name;
}

void *sexp_to_voidptr(SEXP input, SEXP rust_type)
{
    PROTECT(input);
    PROTECT(rust_type);

    if (str_equal(sexp_to_charptr(get_origin(rust_type)), "Option"))
    {
        if (input == R_NilValue)
        {
            UNPROTECT(2);
            return NULL;
        }
        else
        {
            rust_type = VECTOR_ELT(get_args(rust_type), 0);
        }
    }

    void *output;
    const char *c_rust_type = sexp_to_charptr(rust_type);

    if (str_equal(c_rust_type, "String"))
    {
        char **output_str = (char **)R_alloc(LENGTH(input), sizeof(char *));
        for (int i = 0; i < LENGTH(input); i++)
            output_str[i] = (char *)CHAR(STRING_ELT(input, i));
        output = output_str;
    }
    else if (str_equal(c_rust_type, "f64"))
        output = REAL(input);

    else if (str_equal(c_rust_type, "f32"))
    {
        float *output_f32 = (float *)R_alloc(LENGTH(input), sizeof(float));
        for (int i = 0; i < LENGTH(input); i++)
            output_f32[i] = (float)REAL(input)[i];
        output = output_f32;
    }

    else if (str_equal(c_rust_type, "i32"))
        output = INTEGER(input);

    else if (str_equal(c_rust_type, "u32"))
    {
        unsigned int *output_u32 = (unsigned int *)R_alloc(LENGTH(input), sizeof(unsigned int));
        for (int i = 0; i < LENGTH(input); i++)
        {
            if (INTEGER(input)[i] < 0)
                error("u32 cannot be negative");
            output_u32[i] = (unsigned int)INTEGER(input)[i];
        }
        output = output_u32;
    }
    else if (str_equal(c_rust_type, "i64"))
    {
        long int *output_i64 = (long int *)R_alloc(LENGTH(input), sizeof(long int));
        for (int i = 0; i < LENGTH(input); i++)
            output_i64[i] = (long int)INTEGER(input)[i];
        output = output_i64;
    }
    else if (str_equal(c_rust_type, "u64"))
    {
        long unsigned int *output_u64 = (long unsigned int *)R_alloc(LENGTH(input), sizeof(long unsigned int));
        for (int i = 0; i < LENGTH(input); i++)
        {
            if (INTEGER(input)[i] < 0)
                error("u64 cannot be negative");
            output_u64[i] = (long unsigned int)INTEGER(input)[i];
        }
        output = output_u64;
    }
    else if (str_equal(c_rust_type, "usize"))
    {
        size_t *output_size_t = (size_t *)R_alloc(LENGTH(input), sizeof(size_t));
        for (int i = 0; i < LENGTH(input); i++)
        {
            if (INTEGER(input)[i] < 0)
                error("usize cannot be negative");
            output_size_t[i] = (size_t)INTEGER(input)[i];
        }
        output = output_size_t;
    }

    else if (str_equal(c_rust_type, "bool"))
    {
        bool *output_bool = (bool *)R_alloc(LENGTH(input), sizeof(bool));
        for (int i = 0; i < LENGTH(input); i++)
            output_bool[i] = (bool)LOGICAL(input)[i];
        output = output_bool;
    }

    else
        error_unknown_type("sexp_to_voidptr unknown type:", c_rust_type);

    UNPROTECT(2);
    return output;
}

SEXP voidptr_to_sexp(void *input, SEXP rust_type, size_t len)
{
    PROTECT(rust_type);
    const char *c_origin = sexp_to_charptr(get_origin(rust_type));
    SEXP result;

    if (str_equal(c_origin, "String"))
    {
        // STRSXP is a vector of strings, where each element is a string
        result = PROTECT(allocVector(STRSXP, len));
        for (int i = 0; i < len; i++)
            SET_STRING_ELT(result, i, mkChar(((char **)input)[i]));
        UNPROTECT(1);
    }
    else if (str_equal(c_origin, "f64"))
    {
        result = PROTECT(allocVector(REALSXP, len));
        for (int i = 0; i < len; i++)
        {
            REAL(result)
            [i] = ((double *)input)[i];
        }
        UNPROTECT(1);
    }
    else if (str_equal(c_origin, "f32"))
    {
        result = PROTECT(allocVector(REALSXP, len));
        for (int i = 0; i < len; i++)
        {
            float input_f32 = ((float *)input)[i];
            REAL(result)
            [i] = (double)input_f32;
        }
        UNPROTECT(1);
    }
    else if (str_equal(c_origin, "i32"))
    {
        result = PROTECT(allocVector(INTSXP, len));
        for (int i = 0; i < len; i++)
        {
            INTEGER(result)
            [i] = ((int *)input)[i];
        }
        UNPROTECT(1);
    }
    else if (str_equal(c_origin, "u32"))
    {
        result = PROTECT(allocVector(INTSXP, len));
        for (int i = 0; i < len; i++)
        {
            unsigned int input_u32 = ((unsigned int *)input)[i];
            if (input_u32 > INT_MAX)
                error("u32 cannot be greater than INT_MAX");
            INTEGER(result)
            [i] = (int)input_u32;
        }
        UNPROTECT(1);
    }
    else if (str_equal(c_origin, "i64"))
    {
        result = PROTECT(allocVector(INTSXP, len));
        for (int i = 0; i < len; i++)
        {
            long int input_i64 = ((long int *)input)[i];
            if (input_i64 > INT_MAX)
                error("i64 cannot be greater than INT_MAX");
            if (input_i64 < INT_MIN)
                error("i64 cannot be less than INT_MIN");
            INTEGER(result)
            [i] = (int)input_i64;
        }
        UNPROTECT(1);
    }
    else if (str_equal(c_origin, "u64"))
    {
        result = PROTECT(allocVector(INTSXP, len));
        for (int i = 0; i < len; i++)
        {
            long unsigned int input_u64 = ((long unsigned int *)input)[i];
            if (input_u64 > INT_MAX)
                error("u64 cannot be greater than INT_MAX");
            INTEGER(result)
            [i] = (int)input_u64;
        }
        UNPROTECT(1);
    }
    else if (str_equal(c_origin, "usize"))
    {
        result = PROTECT(allocVector(INTSXP, len));
        for (int i = 0; i < len; i++)
        {
            size_t input_size_t = ((size_t *)input)[i];
            if (input_size_t > INT_MAX)
                error("usize cannot be greater than INT_MAX");
            INTEGER(result)
            [i] = (int)input_size_t;
        }
        UNPROTECT(1);
    }

    else if (str_equal(c_origin, "bool"))
    {
        result = PROTECT(allocVector(LGLSXP, len));
        for (int i = 0; i < len; i++)
        {
            LOGICAL(result)
            [i] = ((bool *)input)[i];
        }
        UNPROTECT(1);
    }
    else
        error_unknown_type("voidptr_to_sexp unknown type:", c_origin);

    UNPROTECT(1);
    return result;
}

// forward declaration
SEXP anyobjectptr_to_sexp(AnyObject *obj);

FfiSlice scalar_to_slice(SEXP value, SEXP type_name)
{
    const char *c_origin = sexp_to_charptr(get_origin(type_name));

    FfiSlice result = {NULL, (uintptr_t)LENGTH(value)};

    if (is_in(c_origin, ATOM_TYPES))
        result.ptr = sexp_to_voidptr(value, type_name);

    else if (str_equal(c_origin, "AnyMeasurementPtr"))
    {
        // TODO: does this ever get freed?
        AnyMeasurement **p = malloc(LENGTH(value) * sizeof(AnyMeasurement *));
        for (int i = 0; i < LENGTH(value); i++)
            p[i] = sexp_to_anymeasurementptr(VECTOR_ELT(value, i));
        result.ptr = p;
    }
    else
        error_unknown_type("scalar_to_slice unknown type:", c_origin);

    return result;
}

// all scalars are returned as vectors, because R doesn't have a scalar type
SEXP slice_to_scalar(FfiSlice *raw, SEXP type_name)
{
    PROTECT(type_name);
    const char *c_origin = sexp_to_charptr(get_origin(type_name));

    SEXP result;
    if (is_in(c_origin, ATOM_TYPES))
        result = voidptr_to_sexp((void *)raw->ptr, type_name, raw->len);

    else if (str_equal(c_origin, "AnyObject"))
    {
        result = PROTECT(allocVector(VECSXP, raw->len));
        FfiResult_____FfiSlice result_slice = opendp_data__ffislice_of_anyobjectptrs(raw);
        if (result_slice.tag == Err_____FfiSlice)
            extract_error(result_slice.err);
        FfiSlice *slice = result_slice.ok;
        for (int i = 0; i < raw->len; i++)
            SET_VECTOR_ELT(result, i, anyobjectptr_to_sexp(((AnyObject **)slice->ptr)[i]));
        UNPROTECT(1);
    }
    else
        error_unknown_type("slice_to_scalar unknown type:", c_origin);

    UNPROTECT(1);
    return result;
}

FfiSlice vector_to_slice(SEXP value, SEXP type_name)
{
    PROTECT(value);
    PROTECT(type_name);
    SEXP atom_type = VECTOR_ELT(get_args(type_name), 0);
    // all SEXP are vectors, so scalars are also vectors
    FfiSlice slice = scalar_to_slice(value, atom_type);
    UNPROTECT(2);
    return slice;
}

SEXP slice_to_vector(FfiSlice *raw, SEXP type_name)
{
    PROTECT(type_name);
    SEXP atom_type = VECTOR_ELT(get_args(type_name), 0);
    UNPROTECT(1);
    return slice_to_scalar(raw, atom_type);
}

FfiSlice bitvector_to_slice(SEXP value, SEXP type_name)
{
    PROTECT(value);
    FfiSlice slice = {RAW(value), LENGTH(value) * 8};
    UNPROTECT(1);
    return slice;
}

SEXP slice_to_bitvector(FfiSlice *raw, SEXP type_name)
{
    uintptr_t n_bytes = (raw->len + 7) / 8;
    SEXP buffer = Rf_allocVector(RAWSXP, n_bytes);
    Rbyte* ptr = RAW(buffer);
    memcpy(ptr, raw->ptr, n_bytes);
    return buffer;
}

FfiSlice tuple_to_slice(SEXP value, SEXP type_name)
{
    PROTECT(value);
    PROTECT(type_name);
    size_t len = LENGTH(value);
    void **array = (void **)malloc(len * sizeof(void *));

    // extract substring
    SEXP args = get_args(type_name);

    if (TYPEOF(value) == VECSXP)
        for (size_t i = 0; i < len; i++)
            array[i] = sexp_to_voidptr(VECTOR_ELT(value, i), VECTOR_ELT(args, i));
    else if (TYPEOF(value) == INTSXP)
        for (size_t i = 0; i < len; i++)
            array[i] = sexp_to_voidptr(ScalarInteger(*(INTEGER(value) + i)), VECTOR_ELT(args, i));
    else if (TYPEOF(value) == REALSXP)
        for (size_t i = 0; i < len; i++)
            array[i] = sexp_to_voidptr(ScalarReal(*(REAL(value) + i)), VECTOR_ELT(args, i));
    else
        error_unknown_type("tuple_to_slice unknown type:", type2char(TYPEOF(value)));

    UNPROTECT(2);
    FfiSlice result = {array, len};
    return result;
}

SEXP slice_to_tuple(FfiSlice *raw, SEXP type_name)
{
    PROTECT(type_name);
    SEXP args = get_args(type_name);
    size_t len = LENGTH(args);
    SEXP result = PROTECT(allocVector(VECSXP, len));

    for (size_t i = 0; i < len; i++)
        SET_VECTOR_ELT(result, i, voidptr_to_sexp(((void **)raw->ptr)[i], VECTOR_ELT(args, i), 1));
    UNPROTECT(2);
    return result;
}

// foward declaration
AnyObject *sexp_to_anyobjectptr(SEXP data, SEXP type_name);

FfiSlice hashmap_to_slice(SEXP value, SEXP type_name)
{
    PROTECT(value);
    PROTECT(type_name);
    SEXP args = get_args(type_name);

    int errorOccurred;
    SEXP hashitems_call = PROTECT(lang3(install("hashitems"), value, type_name));
    SEXP hashitems = PROTECT(R_tryEval(hashitems_call, R_GlobalEnv, &errorOccurred));
    if (errorOccurred)
        error("Error getting hash items");

    SEXP key_rt_call = PROTECT(lang2(install("as_rt_vec"), VECTOR_ELT(args, 0)));
    SEXP key_rt = PROTECT(R_tryEval(key_rt_call, R_GlobalEnv, &errorOccurred));
    if (errorOccurred)
        error("Error getting key type");

    SEXP val_rt_call = PROTECT(lang2(install("as_rt_vec"), VECTOR_ELT(args, 1)));
    SEXP val_rt = PROTECT(R_tryEval(val_rt_call, R_GlobalEnv, &errorOccurred));
    if (errorOccurred)
        error("Error getting val type");

    void *ptr = malloc(2 * sizeof(void *));
    ((void **)ptr)[0] = sexp_to_anyobjectptr(VECTOR_ELT(hashitems, 0), key_rt);
    ((void **)ptr)[1] = sexp_to_anyobjectptr(VECTOR_ELT(hashitems, 1), val_rt);

    FfiSlice result = {ptr, 2};
    UNPROTECT(8);
    return result;
}

SEXP slice_to_hashmap(FfiSlice *raw, SEXP type_name)
{
    PROTECT(type_name);
    void **backing = (void **)raw->ptr;
    SEXP keys = anyobjectptr_to_sexp(backing[0]);
    SEXP vals = anyobjectptr_to_sexp(backing[1]);

    int errorOccurred;
    SEXP hashtab_call = PROTECT(lang3(install("new_hashtab"), keys, vals));
    SEXP hashtab = PROTECT(R_tryEval(hashtab_call, R_GlobalEnv, &errorOccurred));
    if (errorOccurred)
        error("Error creating hashmap");

    UNPROTECT(3);
    return hashtab;
}

FfiSlice sexp_to_slice(SEXP value, SEXP type_name)
{
    PROTECT(value);
    PROTECT(type_name);
    const char *c_origin = sexp_to_charptr(get_origin(type_name));

    FfiSlice result;

    if (str_equal(c_origin, "AnyMeasurement"))
    {
        FfiSlice t = {.ptr = sexp_to_anymeasurementptr(value), .len = 1};
        result = t;
    }

    else if (str_equal(c_origin, "Vec"))
        result = vector_to_slice(value, type_name);

    else if (str_equal(c_origin, "BitVector"))
        result = bitvector_to_slice(value, type_name);

    else if (str_equal(c_origin, "HashMap"))
        result = hashmap_to_slice(value, type_name);

    else if (str_equal(c_origin, "Tuple"))
        result = tuple_to_slice(value, type_name);

    else if (is_in(c_origin, ATOM_TYPES))
        result = scalar_to_slice(value, type_name);

    else
        error_unknown_type("sexp_to_slice unknown type:", c_origin);

    UNPROTECT(2);
    return result;
}

SEXP slice_to_sexp(FfiSlice *raw, SEXP type_name)
{
    const char *c_origin = sexp_to_charptr(get_origin(type_name));

    SEXP result;

    if (str_equal(c_origin, "Vec"))
        result = slice_to_vector(raw, type_name);
    
    else if (str_equal(c_origin, "BitVector"))
        result = slice_to_bitvector(raw, type_name);

    else if (str_equal(c_origin, "HashMap"))
        result = slice_to_hashmap(raw, type_name);

    else if (str_equal(c_origin, "Tuple"))
        result = slice_to_tuple(raw, type_name);

    else if (is_in(c_origin, ATOM_TYPES))
        result = slice_to_scalar(raw, type_name);

    else
        error_unknown_type("slice_to_sexp unknown type:", c_origin);

    return result;
}

char *rt_to_string(SEXP type_name)
{
    int errorOccurred;
    SEXP rt_to_string = PROTECT(get_private_func("rt_to_string"));
    SEXP rt_to_string_call = PROTECT(lang2(rt_to_string, type_name));
    SEXP string_type_name = PROTECT(R_tryEval(rt_to_string_call, R_GlobalEnv, &errorOccurred));
    if (errorOccurred)
        error("failed to parse type");

    UNPROTECT(3);
    return (char *)sexp_to_charptr(string_type_name);
}

FfiSlice *sexp_to_ffisliceptr(SEXP data, SEXP type_name)
{
    FfiSlice *slice = malloc(sizeof(FfiSlice));
    *slice = sexp_to_slice(data, type_name);
    return slice;
}


AnyObject *sexp_to_anyobjectptr(SEXP data, SEXP type_name)
{
    // Convert arguments to c types.
    PROTECT(data);
    PROTECT(type_name);

    if (isNull(type_name))
    {
        UNPROTECT(2);
        return (AnyObject *)R_ExternalPtrAddr(data);
    }

    const char *c_origin = sexp_to_charptr(get_origin(type_name));
    if (str_equal(c_origin, "Option"))
    {
        if (isNull(data))
        {
            UNPROTECT(2);
            return NULL;
        }
        else
            type_name = VECTOR_ELT(get_args(type_name), 0);
    }

    const char *c_type_name = rt_to_string(type_name);

    FfiSlice slice = sexp_to_slice(data, type_name);

    // convert the FfiSlice to an AnyObject (or error).
    // An AnyObject is an opaque C struct that contains the data in a rust-specific representation
    // AnyObjects may be used interchangeably in the majority of library APIs
    FfiResult_____AnyObject result = opendp_data__slice_as_object(&slice, c_type_name);

    UNPROTECT(2);

    if (result.tag == Err_____AnyObject)
        extract_error(result.err);

    return result.ok;
}

SEXP anyobjectptr_to_sexp(AnyObject *obj)
{
    FfiResult_____c_char type_name_result = opendp_data__object_type(obj);
    if (type_name_result.tag == Err_____c_char)
        extract_error(type_name_result.err);
    char *c_type_name = type_name_result.ok;

    SEXP r_type_name;
    PROTECT(r_type_name = allocVector(STRSXP, 1));
    SET_STRING_ELT(r_type_name, 0, mkChar(c_type_name));

    int errorOccurred;
    SEXP rt_parse = PROTECT(get_private_func("rt_parse"));
    SEXP rt_parse_call = PROTECT(lang2(rt_parse, r_type_name));
    SEXP type_name = PROTECT(R_tryEval(rt_parse_call, R_GlobalEnv, &errorOccurred));
    if (errorOccurred)
        error("failed to parse type");

    const char *c_origin = sexp_to_charptr(get_origin(type_name));
    if (str_equal(c_origin, "PrivacyProfile"))
    {
        SEXP profile = privacyprofileptr_to_sexp(obj, R_NilValue);
        UNPROTECT(4);
        return profile;
    }

    if (str_equal(c_origin, "AnyQueryable"))
    {
        SEXP queryable = anyqueryableptr_to_sexp(obj, R_NilValue);
        UNPROTECT(4);
        return queryable;
    }

    FfiResult_____FfiSlice slice_result = opendp_data__object_as_slice(obj);
    if (slice_result.tag == Err_____FfiSlice)
        extract_error(slice_result.err);
    FfiSlice *slice = slice_result.ok;

    SEXP value = slice_to_sexp(slice, type_name);
    UNPROTECT(4);
    return value;
}

// https://stackoverflow.com/questions/73520428/is-there-a-way-of-passing-an-r-function-object-closxp-to-c-level-code-and-re
