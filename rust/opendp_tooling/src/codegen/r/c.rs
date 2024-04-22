use std::collections::HashMap;

use crate::{
    codegen::{r::BLACKLIST, tab_c},
    Argument, Function, TypeRecipe,
};

/// Generate the R/src/lib.c file.
/// This file registers all the C functions that R can interface with.
pub fn generate_lib_c(modules: &HashMap<String, Vec<Function>>) -> String {
    // code generation should be deterministic- sort by module
    let mut modules = modules.iter().collect::<Vec<(&String, &Vec<Function>)>>();
    modules.sort_by_key(|(module_name, _)| *module_name);

    // list all functions in all modules
    let function_stubs = modules.iter().map(|(module_name, module)| {

        module.iter()
        .filter(|func| func.has_ffi)
        // don't register functions on the blacklist
        .filter(|func| !BLACKLIST.contains(&func.name.as_str()))
        // R wants to know the name and number of arguments of each function
        .map(|func| {
            // takes into account extra type arguments used when converting R data to C
            let num_args = flatten_args_for_c(func).len() + 1;
            let func_name = &func.name;
            format!(r#"    {{"{module_name}__{func_name}", (DL_FUNC) &{module_name}__{func_name}, {num_args}}},"#)
        }).collect::<Vec<_>>().join("\n")
    }).collect::<Vec<_>>().join("\n");

    format!(
        r#"// Auto-generated. Do not edit.
#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>

// Import C headers for rust API
#include "Ropendp.h"

SEXP AnyObject_tag;
SEXP AnyTransformation_tag;
SEXP AnyMeasurement_tag;
SEXP AnyDomain_tag;
SEXP AnyMetric_tag;
SEXP AnyMeasure_tag;
SEXP AnyFunction_tag;

static R_CMethodDef R_CDef[] = {{
{function_stubs}
    {{NULL, NULL, 0}},
}};

void R_init_opendp(DllInfo *dll)
{{
    R_registerRoutines(dll, R_CDef, NULL, NULL, NULL);
    // here we create the tags for the external pointers
    AnyObject_tag = install("AnyObject_TAG");
    AnyTransformation_tag = install("AnyTransformation_TAG");
    AnyMeasurement_tag = install("AnyMeasurement_TAG");
    AnyDomain_tag = install("AnyDomain_TAG");
    AnyMetric_tag = install("AnyMetric_TAG");
    AnyMeasure_tag = install("AnyMeasure_TAG");
    AnyFunction_tag = install("AnyFunction_TAG");
    R_useDynamicSymbols(dll, TRUE);
}}
"#
    )
}

/// Generate the Ropendp.h file.
/// The generated .c files implement the functions declared here.
#[allow(non_snake_case)]
pub fn generate_Ropendp_h(modules: &HashMap<String, Vec<Function>>) -> String {
    // code generation should be deterministic- sort by module
    let mut modules = modules.iter().collect::<Vec<(&String, &Vec<Function>)>>();
    modules.sort_by_key(|(module_name, _)| *module_name);

    // list all functions in all modules
    let headers = (modules.iter())
        .map(|(module_name, module)| {
            (module.iter())
                .filter(|func| func.has_ffi)
                .filter(|func| !BLACKLIST.contains(&func.name.as_str()))
                .map(|func| {
                    let args = (flatten_args_for_c(func).iter())
                        .map(|arg| format!("{}, ", generate_c_input_argument(arg)))
                        .collect::<Vec<_>>()
                        .join("");
                    let func_name = &func.name;
                    format!(r#"SEXP {module_name}__{func_name}({args}SEXP log);"#)
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"// Auto-generated. Do not edit.
#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>

extern SEXP AnyObject_tag;
extern SEXP AnyTransformation_tag;
extern SEXP AnyMeasurement_tag;
extern SEXP AnyDomain_tag;
extern SEXP AnyMetric_tag;
extern SEXP AnyMeasure_tag;
extern SEXP AnyFunction_tag;

{headers}
"#
    )
}

/// Generates all code for an opendp R module.
/// Each call corresponds to one R file.
pub fn generate_c_module(module_name: &str, module: &Vec<Function>) -> String {
    let funcs = module
        .into_iter()
        .filter(|func| func.has_ffi)
        .filter(|func| !BLACKLIST.contains(&func.name.as_str()))
        .map(|func| generate_c_function(module_name, &func))
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        r#"// Auto-generated. Do not edit.
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

{funcs}
"#
    )
}

/// Each call generates the c glue
/// between R SEXP objects and OpenDP Library c FFI for one library function.
fn generate_c_function(module_name: &str, func: &Function) -> String {
    println!("generating C: {}", func.name);
    // a comma is added to the last arg because all functions take a log argument
    let args = (flatten_args_for_c(func).iter())
        .map(|arg| format!("{}, ", generate_c_input_argument(arg)))
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"
SEXP {module_name}__{func_name}(
{args}
) {{
{body}
}}
"#,
        func_name = func.name,
        args = tab_c(format!("{args}SEXP log")),
        body = tab_c(generate_c_body(module_name, func))
    )
}

/// generate the c function body, consisting of data converters, and the call
/// - data converters convert from R to c representations according to the T_* type arguments
/// - the call code first makes the call, then handles errors, and converts the response to an R SEXP
fn generate_c_body(module_name: &str, func: &Function) -> String {
    format!(
        r#"{data_converter}
{make_call}"#,
        data_converter = generate_data_converter(&func),
        make_call = generate_c_call(module_name, func)
    )
}

/// Generates an input argument.
/// This is pretty simple compared to R or C because all types are erased and this isn't user-facing
fn generate_c_input_argument(arg: &Argument) -> String {
    format!("SEXP {}", arg.name())
}

/// Generates code to convert all arguments to their C representations.
fn generate_data_converter(func: &Function) -> String {
    // all R objects being manipulated need to be protected to prevent them from being freed by R gc
    // without PROTECT, R doesn't know we're manipulating/reading/using this memory
    // will be unprotected in `generate_c_call` at the appropriate time
    let protect = flatten_args_for_c(func)
        .iter()
        .map(|arg| format!("PROTECT({});", arg.name()))
        .collect::<Vec<_>>()
        .join("\n");

    // generates code that calls the appropriate function to read the memory behind the SEXP
    let data_converter: String = (func.args.iter())
        .map(|arg| {
            // types are normalized/always packaged in readable runtime types
            if arg.is_type {
                return format!("char * c_{name} = rt_to_string({name});", name = arg.name());
            }
            format!(
                r#"{c_type} c_{name} = {converter};"#,
                c_type = arg.c_type(),
                name = arg.name(),
                converter = r_to_c(arg)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    if data_converter.is_empty() {
        "// No arguments to convert to c types.\nPROTECT(log);".to_string()
    } else {
        format!(
            r#"// Convert arguments to c types.
{protect}
PROTECT(log);

{data_converter}
"#
        )
    }
}

/// Generates code for calling the OpenDP Library and unpacking the result.
/// - makes the call assuming that arguments have already been converted to C by `generate_data_converter`
/// - handles errors returned by the Rust
/// - converts the response to SEXP
fn generate_c_call(module_name: &str, func: &Function) -> String {
    // all converted arguments were prefixed with `c_`
    let args = (func.args.iter())
        .map(|arg| format!("c_{}", arg.name()))
        .collect::<Vec<_>>()
        .join(", ");

    // mirrors the PROTECT calls in `generate_data_converter`. One addition for `log`
    let unprotect = format!("UNPROTECT({});", flatten_args_for_c(func).len() + 1);

    let mut ret = func.ret.clone();
    // always save the output to a variable called `_result`
    ret.name = Some("_result".to_string());

    let convert_response = c_to_r(ret.clone());

    format!(
        r#"// Call library function.
{ret_type} {ret_name} = opendp_{module_name}__{func_name}({args});

{unprotect}
{convert_response}"#,
        ret_type = mangle(ret.c_type().as_str()),
        ret_name = ret.name(),
        func_name = func.name,
    )
}

// fn set_dependencies(
//     dependencies: &Vec<TypeRecipe>
// ) -> String {
//     if dependencies.is_empty() {
//         String::new()
//     } else {
//         let dependencies = dependencies.iter().map(|dep| dep.to_r()).collect::<Vec<String>>().join(", ");
//         format!("output._depends_on({dependencies})")
//     }
// }

/// Generate code to convert an SEXP to OpenDP Library C FFI representation.
///
/// Reads the type information in `arg` to know which hand-written C function to call to perform the data conversion.
fn r_to_c(arg: &Argument) -> String {
    let c_type = arg.c_type();
    let name = arg.name();
    let rust_type = (arg.rust_type.clone())
        .map(|rt| match rt {
            TypeRecipe::Name(v) => v,
            TypeRecipe::None => "R_NilValue".to_string(),
            _ => format!("T_{name}"),
        })
        .unwrap_or_else(|| "unknown rust type".to_string());

    // conversions for primitive types are handled directly by R internals functions
    // https://github.com/hadley/r-internals/blob/master/vectors.md
    // other conversions are handled by hand-written functions from `convert.c` and `convert_elements.c`
    match &c_type {
        ty if ty == "void *" => format!("sexp_to_voidptr({name}, {rust_type})"),
        ty if ty == "AnyObject *" => format!("sexp_to_anyobjectptr({name}, {rust_type})"),
        ty if ty == "AnyTransformation *" => format!("sexp_to_anytransformationptr({name})"),
        ty if ty == "AnyMeasurement *" => format!("sexp_to_anymeasurementptr({name})"),
        ty if ty == "AnyDomain *" => format!("sexp_to_anydomainptr({name})"),
        ty if ty == "AnyMetric *" => format!("sexp_to_anymetricptr({name})"),
        ty if ty == "AnyMeasure *" => format!("sexp_to_anymeasureptr({name})"),
        ty if ty == "AnyFunction *" => format!("sexp_to_anyfunctionptr({name})"),
        ty if ty == "char *" => format!("(char *)CHAR(STRING_ELT({name}, 0))"),
        ty if ty == "int32_t" => format!("(int32_t)Rf_asInteger({name})"),
        ty if ty == "double" => format!("Rf_asReal({name})"),
        ty if ty == "size_t" => format!("(size_t)Rf_asInteger({name})"),
        ty if ty == "uint32_t" => format!("(unsigned int)Rf_asInteger({name})"),
        ty if ty == "bool" => format!("asLogical({name})"),
        _ => format!("\"UNKNOWN TYPE: {c_type}\""),
    }
}

/// Generate code to convert data from the OpenDP Library C FFI to an R SEXP
///
/// Reads the type information in `arg` to know which hand-written C function to call to perform the data conversion.
fn c_to_r(arg: Argument) -> String {
    let name = arg.name();

    // handle errors
    if arg.c_type().starts_with("FfiResult<") {
        let mut inner = arg.clone();

        // STEP 1. update `arg` to what it would look like after handling the error

        // mutate the c_type to that of the data stored inside the result
        (inner.c_type.as_mut()).map(|ct| *ct = ct["FfiResult<".len()..ct.len() - 1].to_string());

        // when the rust_type is set, also mutate the
        if let Some(TypeRecipe::Nest { origin, args }) = inner.rust_type {
            if origin != "FfiResult" {
                panic!("expected FfiResult, found {}", origin)
            }
            // once the result is handled, you're left with the type of the data inside the result
            inner.rust_type = Some(args[0].clone());
        }

        // name the variable used to hold the successful unwrapped return value
        let inner_name = "_return_value".to_string();
        inner.name = Some(inner_name.clone());

        // STEP 2. handle the error
        let inner_type = mangle(inner.c_type().as_str());

        return format!(
            r#"if({name}.tag == Err_____{inner_type})
    return(extract_error({name}.err));
{inner_type}* {inner_name} = {name}.ok;
{inner}"#,
            inner = c_to_r(inner)
        );
    }

    // const qualifiers on pointers are discarded because it won't change how data is handled
    let converter = match arg.c_type().replace("const ", "") {
        ty if ty == "void *" => {
            // when the c type is a void pointer, lean on the rust type information
            let rust_type = arg.rust_type.clone().unwrap().to_r(None);
            format!("voidptr_to_sexp({name}, {rust_type})")
        }
        // call out to hand-written code from `convert.c` and `convert_elements.c`
        ty if ty == "AnyObject *" => format!("anyobjectptr_to_sexp({name})"),
        ty if ty == "AnyTransformation *" => format!("anytransformationptr_to_sexp({name}, log)"),
        ty if ty == "AnyMeasurement *" => format!("anymeasurementptr_to_sexp({name}, log)"),
        ty if ty == "AnyDomain *" => format!("anydomainptr_to_sexp({name}, log)"),
        ty if ty == "AnyMetric *" => format!("anymetricptr_to_sexp({name}, log)"),
        ty if ty == "AnyMeasure *" => format!("anymeasureptr_to_sexp({name}, log)"),
        ty if ty == "AnyFunction *" => format!("anyfunctionptr_to_sexp({name}, log)"),
        // https://github.com/hadley/r-internals/blob/master/vectors.md
        ty if ty == "char *" => format!("ScalarString(mkChar({name}))"),
        ty if ty == "double" => format!("ScalarReal(*(double *){name})"),
        ty if ty == "int32_t" => format!("ScalarInteger(*(int *){name})"),
        ty if ty == "size_t" => format!("ScalarInteger(*(size_t *){name})"),
        ty if ty == "uint32_t" => format!("ScalarInteger((int){name})"),
        ty if ty == "bool" => format!("ScalarLogical({name})"),
        ty if ty == "bool *" => format!("ScalarLogical(*(bool *){name})"),

        _ => format!("\"UNKNOWN RET TYPE: {}\"", arg.c_type()),
    };

    format!("return({converter});")
}

/// Converts a Rust type like A<B> to align with the corresponding C type cbindgen generates
fn mangle(name: &str) -> String {
    name.replace("<", "_____")
        .replace(">", "")
        .replace(" *", "")
        .replace("const ", "")
        // TODO: cleaner approach?
        .replace("char", "c_char")
        .replace("void", "c_void")
        .replace("bool", "c_bool")
}

/// Derive the set of arguments needed by a C wrapper around an OpenDP Library function.
///
/// Each named argument also has a type argument to unambiguously determine how data should be loaded
fn flatten_args_for_c(function: &Function) -> Vec<Argument> {
    let mut args = function.args.clone();

    // then collect all additional type arguments needed to aid data conversion
    let derived_types_args = (function.derived_types.iter())
        .filter(|arg| {
            // if the derived type arg shadows the function arg,
            //    then replace the function arg and don't add it again
            if let Some(fnarg) = args.iter_mut().find(|fnarg| arg.name() == fnarg.name()) {
                *fnarg = (*arg).clone();
                false
            } else {
                true
            }
        })
        .cloned()
        .collect::<Vec<_>>();
    args.extend(derived_types_args);

    // finally add type ascriptions for named arguments
    args.extend(
        (function.args.iter())
            .filter(|arg| arg.has_implicit_type())
            .map(|arg| Argument {
                name: Some(format!("T_{}", arg.name.clone().unwrap())),
                is_type: true,
                ..Default::default()
            }),
    );
    args
}
