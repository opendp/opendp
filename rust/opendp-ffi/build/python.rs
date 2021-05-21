use std::path::PathBuf;

use indexmap::map::IndexMap;

use crate::{Argument, Function, Module, RuntimeTypeSelector};
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

pub fn make_bindings(modules: IndexMap<String, Module>) -> IndexMap<PathBuf, String> {
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
        .map(|(module_name, module)|
            (PathBuf::from(format!("{}.py", module_name)), make_module(module_name, module, &typemap, &hierarchy)))
        .collect()
}

fn make_module(
    module_name: String,
    module: Module,
    typemap: &HashMap<String, String>,
    hierarchy: &HashMap<String, Vec<String>>
) -> String {
    let functions = module.into_iter()
        .map(|(func_name, func)| make_function(&module_name, &func_name, &func, typemap, hierarchy))
        .collect::<Vec<String>>()
        .join("\n");

    format!(r#"# Auto-generated. Do not edit.
import ctypes
from typing import Type, Union
from opendp.v1.convert import py_to_ptr, py_to_c, py_to_object, c_to_py
from opendp.v1.mod import lib, unwrap, FfiTransformationPtr, FfiMeasurementPtr, FfiResult, FfiObject, FfiSlice, FfiError, FfiObjectPtr, FfiSlicePtr, BoolPtr
from opendp.v1.typing import RuntimeType, RuntimeTypeDescriptor, DatasetMetric, SensitivityMetric

{functions}"#,
            functions=functions)
}

fn make_function(
    module_name: &String,
    func_name: &String, func: &Function,
    typemap: &HashMap<String, String>,
    hierarchy: &HashMap<String, Vec<String>>
) -> String {
    println!("generating: {}", func_name);
    let mut args = func.args.iter()
        .map(|arg| make_arg_input(arg, func, hierarchy))
        .collect::<Vec<_>>();
    args.sort_by(|(_, l_is_default), (_, r_is_default)| l_is_default.cmp(r_is_default));
    let args = args.into_iter().map(|v| v.0).collect::<Vec<_>>().join(",\n");

    let sig_return = get_arg_type_hint(&func.ret, hierarchy)
        .map(|v| format!(" -> {}", v))
        .unwrap_or_else(String::new);

    format!(r#"
def {func_name}(
{args}
){sig_return}:
{docstring}
{body}
"#,
            func_name=func_name,
            args=indent(args),
            sig_return=sig_return,
            docstring=indent(make_docstring(&func, hierarchy)),
            body=indent(make_body(module_name, func_name, &func, typemap)))
}

fn indent(text: String) -> String {
    text.split("\n").map(|v| format!("    {}", v)).collect::<Vec<_>>().join("\n")
}

fn get_unwrapped_arg_c_type(arg: &Argument, typemap: &HashMap<String, String>) -> String {
    assert_eq!(&arg.c_type()[..9], "FfiResult");
    typemap.get(&arg.c_type()[10..arg.c_type().len() - 1]).unwrap().clone()
}
fn get_arg_c_type_origin(arg: &Argument, typemap: &HashMap<String, String>) -> String {
    typemap.get(&arg.c_type_origin()).cloned().expect("ctype not recognized in typemap")
}

fn get_arg_type_hint(arg: &Argument, hierarchy: &HashMap<String, Vec<String>>) -> Option<String> {
    if arg.hint.is_some() {
        return arg.hint.clone()
    }
    if arg.is_type {
        return Some("RuntimeTypeDescriptor".to_string())
    }
    arg.c_type.clone().and_then(|mut c_type| {
        if c_type.starts_with("FfiResult<") {
            c_type = c_type[11..c_type.len() - 1].to_string();
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

fn make_arg_input(arg: &Argument, func: &Function, hierarchy: &HashMap<String, Vec<String>>) -> (String, bool) {
    let default = arg.default.as_ref().cloned()
        .or_else(|| find_public_example(func, arg).map(|_| "None".to_string()));
    (format!(
        r#"{name}{hint}{default}"#,
        name = arg.name(),
        hint = get_arg_type_hint(arg, hierarchy)
            .map(|hint| format!(": {}", hint))
            .unwrap_or_else(String::new),
        default = default.as_ref()
            .map(|default| format!(" = {}", default))
            .unwrap_or_else(String::new)), default.is_some())
}

fn make_docstring(func: &Function, hierarchy: &HashMap<String, Vec<String>>) -> String {
    let description = func.description.as_ref()
        .map(|v| format!("{}\n", v))
        .unwrap_or_else(String::new);
    let doc_args = func.args.iter()
        .map(|v| make_doc_arg(v))
        .collect::<Vec<String>>()
        .join("\n");

    format!(r#""""
{description}{doc_args}{ret_arg}
""""#,
            description=description,
            doc_args=doc_args,
            ret_arg=make_doc_return_arg(&func.ret, hierarchy))

}

fn make_doc_arg(arg: &Argument) -> String {
    format!(r#":param {name}: {description}"#,
            name=arg.name.clone().unwrap_or_else(String::new),
            // type_=get_python_type(arg, is_type)
            //     .map(|v| format!("{} ", v))
            //     .unwrap_or_else(String::new),
            description=arg.description.clone().unwrap_or_else(String::new))
}

fn make_doc_return_arg(arg: &Argument, hierarchy: &HashMap<String, Vec<String>>) -> String {
    let mut ret = Vec::new();
    if let Some(description) = &arg.description {
        ret.push(format!(":return: {description}", description=description));
    }
    if let Some(type_) = get_arg_type_hint(arg, hierarchy) {
        ret.push(format!(":rtype: {type_}", type_=type_));
    }
    if !ret.is_empty() { ret.insert(0, String::new()); }
    ret.join("\n")
}

fn make_body(
    module_name: &String,
    func_name: &String, func: &Function,
    typemap: &HashMap<String, String>
) -> String {
    format!(r#"# parse type args
{type_arg_formatter}

# translate arguments to c types
{data_formatter}

# call library function
{make_call}"#,
            type_arg_formatter = make_type_arg_formatter(func),
            data_formatter = make_data_formatter(func, typemap),
            make_call = make_call(module_name, func_name, func, typemap))
}

fn find_public_example(func: &Function, type_arg: &Argument) -> Option<String> {
    let name = type_arg.name.as_ref().expect("type args must be named");
    func.args.iter()
        .find(|arg| arg.rust_type == Some(name.into()))
        .and_then(|arg| arg.name.clone())
}

fn make_type_arg_formatter(func: &Function) -> String {
    func.args.iter()
        .filter(|arg| arg.is_type)
        .map(|type_arg| {
            let name = type_arg.name.as_ref().expect("type args must be named");
            if let Some(example) = find_public_example(func, type_arg) {
                format!(r#"{name} = RuntimeType.parse_or_infer(type_name={name}, public_example={example})"#,
                        name = name, example = example)
            } else {
                format!(r#"{name} = RuntimeType.parse(type_name={name})"#,
                        name = name)
            }
        })
        // additional types that are constructed by introspecting existing types
        .chain(func.derived_types.iter()
            .map(|type_spec|
                format!("{name} = {derivation}",
                        name = type_spec.name(),
                        derivation = type_spec.rust_type.as_ref().unwrap().to_python())))
        .collect::<Vec<_>>()
        .join("\n")
}

impl RuntimeTypeSelector {
    fn to_python(&self) -> String {
        match self {
            RuntimeTypeSelector::Name(name) =>
                name.clone(),
            RuntimeTypeSelector::Arg { arg: (selector, index) } =>
                format!("{}.args[{}]", selector.to_python(), index)
        }
    }
}

fn make_data_formatter(func: &Function, typemap: &HashMap<String, String>) -> String {
    func.args.iter()
        // .filter(|arg| !arg.is_type)
        .map(|arg| match arg.c_type() {
            c_type if c_type.ends_with("void *") => {
                let type_name = arg.rust_type.clone().unwrap_or("None".into());
                format!(r#"{name} = py_to_ptr({name}, type_name={type_name})"#, name = arg.name(), type_name = type_name.to_python())
            },
            c_type if c_type.ends_with("FfiObject *") => {
                if let Some(type_name) = arg.rust_type.clone() {
                    format!(r#"{name} = py_to_object({name}, type_name={type_name})"#, name = arg.name(), type_name = type_name.to_python())
                } else {
                    format!(r#"{name} = py_to_object({name})"#, name = arg.name())
                }
            },
            _ =>
                format!(r#"{name} = py_to_c({name}, c_type={c_type})"#, name = arg.name(), c_type = get_arg_c_type_origin(arg, typemap))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn make_call(
    module_name: &String,
    func_name: &String, func: &Function,
    typemap: &HashMap<String, String>
) -> String {

    let mut call = format!(r#"function({args})"#,
            args=func.args.iter()
                // .chain(func.type_args.iter())
                .map(|arg| arg.name())
                .collect::<Vec<_>>().join(", "));
    let ctype_restype = get_arg_c_type_origin(&func.ret, typemap);
    if ctype_restype == "FfiResult" {
        call = format!(r#"unwrap({call}, {restype})"#,
            call=call, restype=get_unwrapped_arg_c_type(&func.ret, typemap))
    }
    format!(r#"function = lib.opendp_{module_name}__{func_name}
function.argtypes = [{ctype_args}]
function.restype = {ctype_restype}

return c_to_py({call})"#,
            module_name = module_name,
            func_name = func_name,
            ctype_args = func.args.iter()
                .map(|v| get_arg_c_type_origin(v, typemap))
                .collect::<Vec<_>>().join(", "),
            ctype_restype = ctype_restype,
            call = call)
}
