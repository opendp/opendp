use std::collections::{HashMap, HashSet};

use crate::{
    codegen::{indent, r::BLACKLIST},
    Argument, Function, TypeRecipe,
};

pub fn generate_c_lib(modules: &HashMap<String, Vec<Function>>) -> String {
    let mut modules = modules.iter().collect::<Vec<(&String, &Vec<Function>)>>();
    modules.sort_by_key(|(module_name, _)| *module_name);

    let args = modules.iter().map(|(module_name, module)| {

        module.iter()
        .filter(|func| !BLACKLIST.contains(&func.name.as_str()))
        .map(|func| {
            format!(r#"    {{"{module_name}__{func_name}", (DL_FUNC) &{module_name}__{func_name}, {num_args}}},"#, 
            module_name=module_name, 
            func_name=func.name, 
            num_args=flatten_args_for_c(func).len() + 1)
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
{args}
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

pub fn generate_c_headers(modules: &HashMap<String, Vec<Function>>) -> String {
    let mut modules = modules.iter().collect::<Vec<(&String, &Vec<Function>)>>();
    modules.sort_by_key(|(module_name, _)| *module_name);

    let headers = (modules.iter())
        .map(|(module_name, module)| {
            (module.iter())
                .filter(|func| !BLACKLIST.contains(&func.name.as_str()))
                .map(|func| {
                    let args = (flatten_args_for_c(func).iter())
                        .map(|arg| format!("{}, ", generate_c_input_argument(arg)))
                        .collect::<Vec<_>>()
                        .join("");
                    format!(
                        r#"SEXP {module_name}__{func_name}({args}SEXP log);"#,
                        module_name = module_name,
                        func_name = func.name,
                        args = args
                    )
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

fn generate_c_function(module_name: &str, func: &Function) -> String {
    println!("generating C: {}", func.name);
    let args = (flatten_args_for_c(func).iter())
        .filter(|arg| !BLACKLIST.contains(&arg.name().as_str()))
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
        args = indent(format!("{args}SEXP log")),
        body = indent(generate_c_body(module_name, func))
    )
}

/// generate the function body, consisting of type args formatters, data converters, and the call
/// - type arg formatters make every type arg a RuntimeType, and construct derived RuntimeTypes
/// - data converters convert from python to c representations according to the formatted type args
/// - the call constructs and retrieves the ffi function name, sets ctypes,
///     makes the call, handles errors, and converts the response to python
fn generate_c_body(module_name: &str, func: &Function) -> String {
    format!(
        r#"{data_converter}
{make_call}"#,
        data_converter = generate_data_converter(&func),
        make_call = generate_c_call(module_name, func)
    )
}

/// generate an input argument, complete with name, hint and default.
/// also returns a bool to make it possible to move arguments with defaults to the end of the signature.
fn generate_c_input_argument(arg: &Argument) -> String {
    format!("SEXP {}", arg.name())
}

/// the generated code ensures that all arguments have been converted to their c representations
fn generate_data_converter(func: &Function) -> String {
    let protect = flatten_args_for_c(func)
        .iter()
        .map(|arg| format!("PROTECT({});", arg.name()))
        .collect::<Vec<_>>()
        .join("\n");
    let data_converter: String = (func.args.iter())
        .map(|arg| {
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
        "// No arguments to convert to c types.".to_string()
    } else {
        format!(
            r#"// Convert arguments to c types.
{protect}
{data_converter}
"#
        )
    }
}

/// the generated code
/// - converts the arguments (SEXP) to C
/// - makes the call assuming that the arguments have been converted to C
/// - handles errors
/// - converts the response to SEXP
fn generate_c_call(module_name: &str, func: &Function) -> String {
    let args = (func.args.iter())
        .map(|arg| format!("c_{}", arg.name()))
        .collect::<Vec<_>>()
        .join(", ");

    let unprotect = format!("UNPROTECT({});", flatten_args_for_c(func).len());

    let mut ret = func.ret.clone();
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

fn c_to_r(arg: Argument) -> String {
    let name = arg.name();

    // handle errors
    if arg.c_type().starts_with("FfiResult<") {
        let mut inner = arg.clone();
        (inner.c_type.as_mut()).map(|ct| *ct = ct["FfiResult<".len()..ct.len() - 1].to_string());
        if let Some(TypeRecipe::Nest { origin, args }) = inner.rust_type {
            if origin != "FfiResult" {
                panic!("expected FfiResult, found {}", origin)
            }
            inner.rust_type = Some(args[0].clone());
        }
        let inner_name = "_return_value".to_string();
        inner.name = Some(inner_name.clone());

        let inner_type = mangle(inner.c_type().as_str());

        return format!(
            r#"if({name}.tag == Err_____{inner_type})
    return(extract_error({name}.err));
{inner_type}* {inner_name} = {name}.ok;
{inner}"#,
            inner = c_to_r(inner)
        );
    }

    let converter = match arg.c_type().replace("const ", "") {
        ty if ty == "void *" => {
            let rust_type = arg.rust_type.clone().unwrap().to_r(None);
            format!("voidptr_to_sexp({name}, {rust_type})")
        },
        ty if ty == "AnyObject *" => format!("anyobjectptr_to_sexp({name})"),
        ty if ty == "AnyTransformation *" => format!("anytransformationptr_to_sexp({name}, log)"),
        ty if ty == "AnyMeasurement *" => format!("anymeasurementptr_to_sexp({name}, log)"),
        ty if ty == "AnyDomain *" => format!("anydomainptr_to_sexp({name}, log)"),
        ty if ty == "AnyMetric *" => format!("anymetricptr_to_sexp({name}, log)"),
        ty if ty == "AnyMeasure *" => format!("anymeasureptr_to_sexp({name}, log)"),
        ty if ty == "AnyFunction *" => format!("anyfunctionptr_to_sexp({name}, log)"),
        ty if ty == "char *" => format!("ScalarString(mkChar({name}))"),
        ty if ty == "int32_t" => format!("ScalarInteger(*(int *){name})"),
        ty if ty == "size_t" => format!("ScalarInteger(*(size_t *){name})"),
        ty if ty == "uint32_t" => format!("ScalarInteger((int){name})"),
        ty if ty == "bool" => format!("ScalarInteger({name})"),
        ty if ty == "bool *" => format!("ScalarInteger(*(bool *){name})"),

        _ => format!("\"UNKNOWN RET TYPE: {}\"", arg.c_type()),
    };

    format!("return({converter});")
}

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

fn flatten_args_for_c(function: &Function) -> Vec<Argument> {
    let mut args = function.args.clone();
    let arg_names = function
        .args
        .iter()
        .map(|arg| arg.name().clone())
        .collect::<HashSet<_>>();
    args.extend(
        function
            .derived_types
            .iter()
            .filter(|arg| !arg_names.contains(&arg.name()))
            .cloned(),
    );

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
