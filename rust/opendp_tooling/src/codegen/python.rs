use std::collections::HashMap;
use std::path::PathBuf;

use crate::{Argument, Function, TypeRecipe, Value};

use crate::codegen::indent;

use super::flatten_type_recipe;

/// Top-level function to generate python bindings, including all modules.
pub fn generate_bindings(modules: HashMap<String, Vec<Function>>) -> HashMap<PathBuf, String> {
    let typemap: HashMap<String, String> =
        serde_json::from_str(&include_str!("python_typemap.json")).unwrap();
    let hierarchy: HashMap<String, Vec<String>> =
        serde_json::from_str(&include_str!("type_hierarchy.json")).unwrap();

    modules
        .into_iter()
        .map(|(module_name, module)| {
            (
                PathBuf::from(format!(
                    "{}.py",
                    if &module_name == "data" {
                        "_data"
                    } else {
                        module_name.as_str()
                    }
                )),
                generate_module(module_name, module, &typemap, &hierarchy),
            )
        })
        .collect()
}

/// Generates all code for an opendp python module.
/// Each call corresponds to one python file.
fn generate_module(
    module_name: String,
    module: Vec<Function>,
    typemap: &HashMap<String, String>,
    hierarchy: &HashMap<String, Vec<String>>,
) -> String {
    let all = module
        .iter()
        .map(|func| format!("    \"{}\"", func.name))
        .chain(
            module
                .iter()
                .filter(|func| func.supports_partial)
                .map(|func| format!("    \"{}\"", func.name.replacen("make_", "then_", 1))),
        )
        .collect::<Vec<_>>()
        .join(",\n");
    let functions = module
        .into_iter()
        .map(|func| generate_function(&module_name, &func, typemap, hierarchy))
        .collect::<Vec<String>>()
        .join("\n");

    // the comb module needs access to core functions for type introspection on measurements/transformations
    let constructor_mods = ["combinators", "measurements", "transformations"];

    let extra_imports = if constructor_mods.contains(&module_name.as_str()) {
        r#"from opendp.core import *
from opendp.domains import *
from opendp.metrics import *
from opendp.measures import *"#
    } else {
        ""
    };

    format!(
        r#"# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
{extra_imports}
__all__ = [
{all}
]

{functions}"#
    )
}

fn generate_function(
    module_name: &String,
    func: &Function,
    typemap: &HashMap<String, String>,
    hierarchy: &HashMap<String, Vec<String>>,
) -> String {
    println!("generating: {}", func.name);
    let mut args = func
        .args
        .iter()
        .map(|arg| generate_input_argument(arg, func, hierarchy))
        .collect::<Vec<_>>();
    args.sort_by(|(_, l_is_default), (_, r_is_default)| l_is_default.cmp(r_is_default));
    let args = args.into_iter().map(|v| v.0).collect::<Vec<_>>();

    let sig_return = func
        .ret
        .python_type_hint(hierarchy)
        .map(|v| format!(" -> {}", v))
        .unwrap_or_else(String::new);

    let docstring = indent(generate_docstring(func, hierarchy));
    let body = indent(generate_body(module_name, func, typemap));

    let then_func = if func.supports_partial {
        format!(
            r#"

def {then_name}(
{then_args}
):
    return PartialConstructor(lambda {dom_met}: {name}(
{args}))
"#,
            then_name = func.name.replacen("make_", "then_", 1),
            then_args = indent(args[2..].join(",\n")),
            dom_met = func.args[..2]
                .iter()
                .map(|arg| arg.name())
                .collect::<Vec<_>>()
                .join(", "),
            name = func.name,
            args = indent(indent(
                func.args
                    .iter()
                    .map(|arg| format!("{name}={name}", name = arg.name()))
                    .collect::<Vec<_>>()
                    .join(",\n")
            ))
        )
    } else {
        String::new()
    };

    format!(
        r#"
@versioned
def {func_name}(
{args}
){sig_return}:
{docstring}
{body}{then_func}
"#,
        func_name = func.name,
        args = indent(args.join(",\n"))
    )
}

/// generate an input argument, complete with name, hint and default.
/// also returns a bool to make it possible to move arguments with defaults to the end of the signature.
fn generate_input_argument(
    arg: &Argument,
    func: &Function,
    hierarchy: &HashMap<String, Vec<String>>,
) -> (String, bool) {
    let default = if let Some(default) = &arg.default {
        Some(match default {
            Value::Null => "None".to_string(),
            Value::Bool(value) => if *value { "True" } else { "False" }.to_string(),
            Value::Integer(int) => int.to_string(),
            Value::Float(float) => float.to_string(),
            Value::String(string) => format!("\"{}\"", string),
        })
    } else {
        // let default value be None if it is a type arg and there is a public example
        generate_public_example(func, arg).map(|_| "None".to_string())
    };
    (
        format!(
            r#"{name}{hint}{default}"#,
            name = arg.name(),
            hint = arg
                .python_type_hint(hierarchy)
                // make argument optional if there is a default
                .map(|hint| if default.is_some() {
                    format!("Optional[{}]", hint)
                } else {
                    hint
                })
                // don't hint for args that are not converted
                .filter(|_| !arg.do_not_convert)
                .map(|hint| format!(": {}", hint))
                .unwrap_or_else(String::new),
            default = default
                .as_ref()
                .map(|default| format!(" = {}", default))
                .unwrap_or_else(String::new)
        ),
        default.is_some(),
    )
}

/// generate a docstring for the current function, with the function description, args, and return
/// in Sphinx format: https://sphinx-rtd-tutorial.readthedocs.io/en/latest/docstrings.html
fn generate_docstring(func: &Function, hierarchy: &HashMap<String, Vec<String>>) -> String {
    let description = (func.description.as_ref())
        .map(|v| format!("{}\n", v))
        .unwrap_or_else(String::new);

    let doc_args = func
        .args
        .iter()
        .map(|v| generate_docstring_arg(v, hierarchy))
        .collect::<Vec<String>>()
        .join("\n");

    let raises = format!(
        r#":raises TypeError: if an argument's type differs from the expected type
:raises UnknownTypeError: if a type argument fails to parse{opendp_raise}"#,
        opendp_raise = if func.ret.c_type_origin() == "FfiResult" {
            "\n:raises OpenDPException: packaged error from the core OpenDP library"
        } else {
            ""
        }
    );

    format!(
        r#"r"""{description}
{doc_args}{ret_arg}
{raises}
""""#,
        description = description,
        doc_args = doc_args,
        ret_arg = generate_docstring_return_arg(&func.ret, hierarchy),
        raises = raises
    )
}

/// generate the part of a docstring corresponding to an argument
fn generate_docstring_arg(arg: &Argument, hierarchy: &HashMap<String, Vec<String>>) -> String {
    let name = arg.name.clone().unwrap_or_default();
    format!(
        r#":param {name}: {description}{type_}"#,
        name = name,
        type_ = arg
            .python_type_hint(hierarchy)
            .map(|v| if v.as_str() == "RuntimeTypeDescriptor" {
                ":py:ref:`RuntimeTypeDescriptor`".to_string()
            } else {
                v
            })
            .map(|v| format!("\n:type {}: {}", name, v))
            .unwrap_or_default(),
        description = arg.description.clone().unwrap_or_default()
    )
}

/// generate the part of a docstring corresponding to a return argument
fn generate_docstring_return_arg(
    arg: &Argument,
    hierarchy: &HashMap<String, Vec<String>>,
) -> String {
    let mut ret = Vec::new();
    if let Some(description) = &arg.description {
        ret.push(format!(":return: {description}", description = description));
    }
    if let Some(type_) = arg.python_type_hint(hierarchy) {
        ret.push(format!(":rtype: {type_}", type_ = type_));
    }
    if !ret.is_empty() {
        ret.insert(0, String::new());
    }
    ret.join("\n")
}

/// generate the function body, consisting of type args formatters, data converters, and the call
/// - type arg formatters make every type arg a RuntimeType, and construct derived RuntimeTypes
/// - data converters convert from python to c representations according to the formatted type args
/// - the call constructs and retrieves the ffi function name, sets ctypes,
///     makes the call, handles errors, and converts the response to python
fn generate_body(
    module_name: &String,
    func: &Function,
    typemap: &HashMap<String, String>,
) -> String {
    format!(
        r#"{flag_checker}{type_arg_formatter}
{data_converter}
{make_call}
{set_dependencies}
return output"#,
        flag_checker = generate_flag_check(&func.features),
        type_arg_formatter = generate_type_arg_formatter(func),
        data_converter = generate_data_converter(func, typemap),
        set_dependencies = set_dependencies(&func.dependencies),
        make_call = generate_call(module_name, func, typemap)
    )
}

// generate code that checks that a set of feature flags are enabled
fn generate_flag_check(features: &Vec<String>) -> String {
    if features.is_empty() {
        String::default()
    } else {
        format!(
            "assert_features({})\n\n",
            features
                .iter()
                .map(|f| format!("\"{}\"", f))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// generate code that provides an example of the type of the type_arg
fn generate_public_example(func: &Function, type_arg: &Argument) -> Option<String> {
    // the json has supplied explicit instructions to find an example
    if let Some(example) = &type_arg.example {
        return Some(example.to_python());
    }

    let type_name = type_arg.name.as_ref().unwrap();

    // rewrite args to remove references to derived types
    let mut args = func.args.clone();
    args.iter_mut()
        .filter(|arg| arg.rust_type.is_some())
        .for_each(|arg| {
            arg.rust_type = Some(flatten_type_recipe(
                arg.rust_type.as_ref().unwrap(),
                &func.derived_types,
            ))
        });

    // code generation
    args.iter()
        .filter_map(|arg| match &arg.rust_type {
            Some(TypeRecipe::Name(name)) => (name == type_name).then(|| arg.name()),
            Some(TypeRecipe::Nest { origin, args }) => {
                if origin == "Vec" {
                    if let TypeRecipe::Name(arg_name) = &args[0] {
                        if arg_name == type_name {
                            Some(format!("get_first({name})", name = arg.name()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .next()
}

/// the generated code ensures every type arg is a RuntimeType, and constructs derived RuntimeTypes
fn generate_type_arg_formatter(func: &Function) -> String {
    let type_arg_formatter: String = func.args.iter()
        .filter(|arg| arg.is_type)
        .map(|type_arg| {
            let name = type_arg.name.as_ref().expect("type args must be named");
            let generics = if type_arg.generics.is_empty() {
                "".to_string()
            } else {
                format!(", generics=[{}]", type_arg.generics.iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>().join(", "))
            };
            if let Some(example) = generate_public_example(func, type_arg) {
                format!(r#"{name} = RuntimeType.parse_or_infer(type_name={name}, public_example={example}{generics})"#)
            } else {
                format!(r#"{name} = RuntimeType.parse(type_name={name}{generics})"#)
            }
        })
        // additional types that are constructed by introspecting existing types
        .chain(func.derived_types.iter()
            .map(|type_spec|
                format!("{name} = {derivation} # type: ignore",
                        name = type_spec.name(),
                        derivation = type_spec.rust_type.as_ref().unwrap().to_python())))
        .chain(func.args.iter()
            .filter(|arg| !arg.generics.is_empty())
            .map(|arg|
                format!("{name} = {name}.substitute({args}) # type: ignore",
                        name=arg.name.as_ref().unwrap(),
                        args=arg.generics.iter()
                            .map(|generic| format!("{generic}={generic}", generic = generic))
                            .collect::<Vec<_>>().join(", "))))
        .collect::<Vec<_>>()
        .join("\n");

    if type_arg_formatter.is_empty() {
        "# No type arguments to standardize.".to_string()
    } else {
        format!(
            r#"# Standardize type arguments.
{formatter}
"#,
            formatter = type_arg_formatter
        )
    }
}

/// the generated code ensures that all arguments have been converted to their c representations
fn generate_data_converter(func: &Function, typemap: &HashMap<String, String>) -> String {
    let data_converter: String = func
        .args
        .iter()
        .map(|arg| {
            let name = arg.name();
            if arg.do_not_convert {
                return format!("c_{name} = {name}");
            };
            format!(
                r#"c_{name} = py_to_c({name}, c_type={c_type}{rust_type_arg})"#,
                c_type = arg.python_origin_ctype(typemap),
                rust_type_arg = arg
                    .rust_type
                    .as_ref()
                    .map(|r_type| format!(", type_name={}", r_type.to_python()))
                    .unwrap_or_else(|| "".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    if data_converter.is_empty() {
        "# No arguments to convert to c types.".to_string()
    } else {
        format!(
            r#"# Convert arguments to c types.
{converter}
"#,
            converter = data_converter
        )
    }
}

/// the generated code
/// - constructs and retrieves the ffi function name
/// - sets argtypes and restype on the ctypes function
/// - makes the call assuming that the arguments have been converted to C
/// - handles errors
/// - converts the response to python
fn generate_call(
    module_name: &String,
    func: &Function,
    typemap: &HashMap<String, String>,
) -> String {
    let mut call = format!(
        r#"lib_function({args})"#,
        args = func
            .args
            .iter()
            .map(|arg| format!("c_{}", arg.name()))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let ctype_restype = func.ret.python_origin_ctype(typemap);
    if ctype_restype == "FfiResult" {
        call = format!(
            r#"unwrap({call}, {restype})"#,
            call = call,
            restype = func.ret.python_unwrapped_ctype(typemap)
        )
    }
    if !func.ret.do_not_convert {
        call = format!(r#"c_to_py({})"#, call)
    }
    format!(
        r#"# Call library function.
lib_function = lib.opendp_{module_name}__{func_name}
lib_function.argtypes = [{ctype_args}]
lib_function.restype = {ctype_restype}

output = {call}"#,
        module_name = module_name,
        func_name = func.name,
        ctype_args = func
            .args
            .iter()
            .map(|v| v.python_origin_ctype(typemap))
            .collect::<Vec<_>>()
            .join(", "),
        ctype_restype = ctype_restype,
        call = call
    )
}

fn set_dependencies(dependencies: &Vec<TypeRecipe>) -> String {
    if dependencies.is_empty() {
        String::new()
    } else {
        let dependencies = dependencies
            .iter()
            .map(|dep| dep.to_python())
            .collect::<Vec<String>>()
            .join(", ");
        format!("output._depends_on({dependencies})")
    }
}
