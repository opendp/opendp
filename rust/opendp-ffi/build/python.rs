use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use indexmap::map::IndexMap;
use serde_json::Value;

use crate::{Argument, Function, Module, RuntimeType};

/// Top-level function to generate python bindings, including all modules.
pub fn generate_bindings(modules: IndexMap<String, Module>) -> IndexMap<PathBuf, String> {
    let mut contents = String::new();
    File::open("build/python_typemap.json")
        .expect("python typemap not found")
        .read_to_string(&mut contents)
        .expect("failed reading python typemap json");
    let typemap: HashMap<String, String> = serde_json::from_str(&contents).unwrap();

    let mut contents = String::new();
    File::open("build/type_hierarchy.json")
        .expect("type hierarchy not found")
        .read_to_string(&mut contents)
        .expect("failed reading type hierarchy json");
    let hierarchy: HashMap<String, Vec<String>> = serde_json::from_str(&contents).unwrap();

    modules.into_iter()
        .map(|(module_name, module)| (
            PathBuf::from(format!("{}.py", if module_name == "data".to_string() {"_data".to_string()} else {module_name.clone()})),
            generate_module(module_name, module, &typemap, &hierarchy)
        ))
        .collect()
}

/// Generates all code for an opendp python module.
/// Each call corresponds to one python file.
fn generate_module(
    module_name: String,
    module: Module,
    typemap: &HashMap<String, String>,
    hierarchy: &HashMap<String, Vec<String>>,
) -> String {
    let all = module.keys().map(|v| format!("    \"{}\"", v)).collect::<Vec<_>>().join(",\n");
    let functions = module.into_iter()
        .map(|(func_name, func)| generate_function(&module_name, &func_name, &func, typemap, hierarchy))
        .collect::<Vec<String>>()
        .join("\n");

    format!(r#"# Auto-generated. Do not edit.
from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *

__all__ = [
{all}
]

{functions}"#, all = all, functions = functions)
}

fn generate_function(
    module_name: &String,
    func_name: &String, func: &Function,
    typemap: &HashMap<String, String>,
    hierarchy: &HashMap<String, Vec<String>>,
) -> String {
    println!("generating: {}", func_name);
    let mut args = func.args.iter()
        .map(|arg| generate_input_argument(arg, func, hierarchy))
        .collect::<Vec<_>>();
    args.sort_by(|(_, l_is_default), (_, r_is_default)| l_is_default.cmp(r_is_default));
    let args = args.into_iter().map(|v| v.0).collect::<Vec<_>>().join(",\n");

    let sig_return = func.ret.python_type_hint(hierarchy)
        .map(|v| format!(" -> {}", v))
        .unwrap_or_else(String::new);

    format!(r#"
def {func_name}(
{args}
){sig_return}:
{docstring}
{body}
"#,
            func_name = func_name,
            args = crate::indent(args),
            sig_return = sig_return,
            docstring = crate::indent(generate_docstring(&func, func_name, hierarchy)),
            body = crate::indent(generate_body(module_name, func_name, &func, typemap)))
}


impl Argument {
    /// retrieve the python ctype corresponding to the type inside FfiResult<*>
    fn python_unwrapped_ctype(&self, typemap: &HashMap<String, String>) -> String {
        assert_eq!(&self.c_type()[..9], "FfiResult");
        typemap.get(&self.c_type()[10..self.c_type().len() - 1]).unwrap().clone()
    }
    /// retrieve the python ctypes corresponding to the origin of a type (subtypes/args omitted)
    fn python_origin_ctype(&self, typemap: &HashMap<String, String>) -> String {
        typemap.get(&self.c_type_origin()).cloned().expect("ctype not recognized in typemap")
    }
    fn python_type_hint(&self, hierarchy: &HashMap<String, Vec<String>>) -> Option<String> {
        if self.hint.is_some() {
            return self.hint.clone()
        }
        if self.is_type {
            return Some("RuntimeTypeDescriptor".to_string())
        }
        self.c_type.clone().and_then(|mut c_type| {
            if c_type.starts_with("FfiResult<") {
                c_type = c_type[10..c_type.len() - 1].to_string();
            }
            if c_type.ends_with("AnyTransformation *") {
                return Some("Transformation".to_string())
            }
            if c_type.ends_with("AnyMeasurement *") {
                return Some("Measurement".to_string())
            }
            if c_type.ends_with("AnyObject *") {
                // py_to_object converts Any to AnyObjectPtr
                return Some("Any".to_string())
            }
            if c_type.ends_with("FfiSlice *") {
                // py_to_c converts Any to FfiSlicePtr
                return Some("Any".to_string())
            }

            hierarchy.iter()
                .find(|(_k, members)| members.contains(&c_type))
                .and_then(|(k, _)| Some(match k.as_str() {
                    k if k == "integer" => "int",
                    k if k == "float" => "float",
                    k if k == "string" => "str",
                    k if k == "bool" => "bool",
                    _ => return None
                }))
                .map(|v| v.to_string())
        })
    }
}

/// generate an input argument, complete with name, hint and default.
/// also returns a bool to make it possible to move arguments with defaults to the end of the signature.
fn generate_input_argument(arg: &Argument, func: &Function, hierarchy: &HashMap<String, Vec<String>>) -> (String, bool) {
    let default = if let Some(default) = &arg.default {
        Some(match default {
            Value::Null => "None".to_string(),
            Value::Bool(value) => if *value {"True"} else {"False"}.to_string(),
            Value::Number(number) => number.to_string(),
            Value::String(string) => format!("\"{}\"", string),
            Value::Array(array) => format!("{:?}", array),
            Value::Object(_) => unimplemented!()
        })
    } else {
        // let default value be None if it is a type arg and there is a public example
        generate_public_example(func, arg).map(|_| "None".to_string())
    };
    (format!(
        r#"{name}{hint}{default}"#,
        name = arg.name(),
        hint = arg.python_type_hint(hierarchy)
            .map(|hint| format!(": {}", hint))
            .unwrap_or_else(String::new),
        default = default.as_ref()
            .map(|default| format!(" = {}", default))
            .unwrap_or_else(String::new)), default.is_some())
}

/// generate a docstring for the current function, with the function description, args, and return
/// in Sphinx format: https://sphinx-rtd-tutorial.readthedocs.io/en/latest/docstrings.html
fn generate_docstring(func: &Function, func_name: &String, hierarchy: &HashMap<String, Vec<String>>) -> String {
    let mut description = func.description.as_ref()
        .map(|v| format!("{}\n", v))
        .unwrap_or_else(String::new);
    if let Some(proof) = &func.proof {
        description = format!(r#"{description}

`This constructor is supported by the linked proof. <{proof}>`_
"#,
        description=description,
        proof=proof);
    }

    let doc_args = func.args.iter()
        .map(|v| generate_docstring_arg(v, hierarchy))
        .collect::<Vec<String>>()
        .join("\n");

    let raises = format!(
        r#":raises AssertionError: if an argument's type differs from the expected type
:raises UnknownTypeError: if a type-argument fails to parse{opendp_raise}"#,
        opendp_raise = if func.ret.c_type_origin() == "FfiResult" {
            "\n:raises OpenDPException: packaged error from the core OpenDP library"
        } else { "" });

    format!(r#""""{description}
{doc_args}{ret_arg}
{raises}
""""#,
            description = description,
            doc_args = doc_args,
            ret_arg = generate_docstring_return_arg(&func.ret, func_name, hierarchy),
            raises = raises)
}

/// generate the part of a docstring corresponding to an argument
fn generate_docstring_arg(arg: &Argument, hierarchy: &HashMap<String, Vec<String>>) -> String {
    let name = arg.name.clone().unwrap_or_else(String::new);
    format!(r#":param {name}: {description}{type_}"#,
            name = name,
            type_ = arg.python_type_hint(hierarchy)
                .map(|v| format!("\n:type {}: {}", name, v))
                .unwrap_or_else(String::new),
            description = arg.description.clone().unwrap_or_else(String::new))
}

/// generate the part of a docstring corresponding to a return argument
fn generate_docstring_return_arg(arg: &Argument, func_name: &String, hierarchy: &HashMap<String, Vec<String>>) -> String {
    let mut ret = Vec::new();
    if let Some(description) = &arg.description {
        ret.push(format!(":return: {description}", description = description));
    } else if func_name.starts_with("make_") {
        ret.push(format!(":return: A {name} step.", name = func_name.replace("make_", "")))
    }
    if let Some(type_) = arg.python_type_hint(hierarchy) {
        ret.push(format!(":rtype: {type_}", type_ = type_));
    }
    if !ret.is_empty() { ret.insert(0, String::new()); }
    ret.join("\n")
}

/// generate the function body, consisting of type args formatters, data converters, and the call
/// - type arg formatters make every type arg a RuntimeType, and construct derived RuntimeTypes
/// - data converters convert from python to c representations according to the formatted type args
/// - the call constructs and retrieves the ffi function name, sets ctypes,
///     makes the call, handles errors, and converts the response to python
fn generate_body(
    module_name: &String,
    func_name: &String, func: &Function,
    typemap: &HashMap<String, String>,
) -> String {
    format!(r#"{flag_checker}{type_arg_formatter}
{data_converter}
{make_call}"#,
            flag_checker = generate_flag_check(&func.features),
            type_arg_formatter = generate_type_arg_formatter(func),
            data_converter = generate_data_converter(func, typemap),
            make_call = generate_call(module_name, func_name, func, typemap))
}

// generate code that checks that a set of feature flags are enabled
fn generate_flag_check(features: &Vec<String>) -> String {
    if features.is_empty() {
        String::default()
    } else {
        format!("assert_features({})\n\n", features.iter()
            .map(|f| format!("\"{}\"", f))
            .collect::<Vec<_>>()
            .join(", "))
    }
}

/// generate code that provides an example of the type of the type_arg
fn generate_public_example(func: &Function, type_arg: &Argument) -> Option<String> {
    // the json has supplied explicit instructions to find an example
    if let Some(example) = &type_arg.example {
        return Some(example.to_python())
    }

    let type_name = type_arg.name.as_ref().unwrap();

    // rewrite args to remove references to derived types
    let mut args = func.args.clone();
    args.iter_mut()
        .filter(|arg| arg.rust_type.is_some())
        .for_each(|arg| arg.rust_type = Some(crate::flatten_runtime_type(
            arg.rust_type.as_ref().unwrap(), &func.derived_types)));

    // code generation
    args.iter()
        .filter_map(|arg| match &arg.rust_type {
            Some(RuntimeType::Name(name)) => (name == type_name).then(|| arg.name()),
            Some(RuntimeType::Raise { origin, args }) =>
                if origin == "Vec" {
                    if let RuntimeType::Name(arg_name) = &*args[0] {
                        if arg_name == type_name {
                            Some(format!("next(iter({name}), None)", name = arg.name()))
                        } else { None }
                    } else { None }
                } else { None }
            _ => None
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
                format!(r#"{name} = RuntimeType.parse_or_infer(type_name={name}, public_example={example}{generics})"#,
                        name = name, example = example, generics = generics)
            } else {
                format!(r#"{name} = RuntimeType.parse(type_name={name}{generics})"#,
                        name = name, generics = generics)
            }
        })
        // additional types that are constructed by introspecting existing types
        .chain(func.derived_types.iter()
            .map(|type_spec|
                format!("{name} = {derivation}",
                        name = type_spec.name(),
                        derivation = type_spec.rust_type.as_ref().unwrap().to_python())))
        .chain(func.args.iter()
            .filter(|arg| !arg.generics.is_empty())
            .map(|arg|
                format!("{name} = {name}.substitute({args})",
                        name=arg.name.as_ref().unwrap(),
                        args=arg.generics.iter()
                            .map(|generic| format!("{generic}={generic}", generic = generic))
                            .collect::<Vec<_>>().join(", "))))
        .collect::<Vec<_>>()
        .join("\n");


    if type_arg_formatter.is_empty() {
        "# No type arguments to standardize.".to_string()
    } else {
        format!(r#"# Standardize type arguments.
{formatter}
"#, formatter = type_arg_formatter)
    }
}

impl RuntimeType {
    /// translate the abstract derived_types info into python RuntimeType constructors
    fn to_python(&self) -> String {
        match self {
            Self::Name(name) =>
                name.clone(),
            Self::Lower { root: arg, index } =>
                format!("{}.args[{}]", arg.to_python(), index),
            Self::Function { function, params } =>
                format!("{function}({params})", function = function, params = params.iter()
                    .map(|v| v.to_python())
                    .collect::<Vec<_>>().join(", ")),
            Self::Raise { origin, args } =>
                format!("RuntimeType(origin='{origin}', args=[{args}])",
                        origin = origin,
                        args = args.iter().map(|arg| arg.to_python()).collect::<Vec<_>>().join(", ")),
        }
    }
}

/// the generated code ensures that all arguments have been converted to their c representations
fn generate_data_converter(func: &Function, typemap: &HashMap<String, String>) -> String {
    let data_converter: String = func.args.iter()
        .filter(|arg| !arg.do_not_convert)
        .map(|arg| format!(
            r#"{name} = py_to_c({name}, c_type={c_type}{rust_type_arg})"#,
            name = arg.name(),
            c_type = arg.python_origin_ctype(typemap),
            rust_type_arg = arg.rust_type.as_ref()
                .map(|r_type| format!(", type_name={}", r_type.to_python()))
                .unwrap_or_else(|| "".to_string())))
        .collect::<Vec<_>>()
        .join("\n");

    if data_converter.is_empty() {
        "# No arguments to convert to c types.".to_string()
    } else {
        format!(r#"# Convert arguments to c types.
{converter}
"#, converter = data_converter)
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
    func_name: &String, func: &Function,
    typemap: &HashMap<String, String>,
) -> String {
    let mut call = format!(r#"function({args})"#,
                           args = func.args.iter()
                               // .chain(func.type_args.iter())
                               .map(|arg| arg.name())
                               .collect::<Vec<_>>().join(", "));
    let ctype_restype = func.ret.python_origin_ctype(typemap);
    if ctype_restype == "FfiResult" {
        call = format!(r#"unwrap({call}, {restype})"#,
                       call = call, restype = func.ret.python_unwrapped_ctype(typemap))
    }
    if !func.ret.do_not_convert { call = format!(r#"c_to_py({})"#, call) }
    format!(r#"# Call library function.
function = lib.opendp_{module_name}__{func_name}
function.argtypes = [{ctype_args}]
function.restype = {ctype_restype}

return {call}"#,
            module_name = module_name,
            func_name = func_name,
            ctype_args = func.args.iter()
                .map(|v| v.python_origin_ctype(typemap))
                .collect::<Vec<_>>().join(", "),
            ctype_restype = ctype_restype,
            call = call)
}
