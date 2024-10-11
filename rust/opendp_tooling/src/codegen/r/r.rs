use std::{collections::HashMap, fs, iter::once};

use crate::{
    codegen::{flatten_type_recipe, tab_r},
    Argument, Function, TypeRecipe, Value,
};

use super::BLACKLIST;

static ATOM_TYPES: &'static [&'static str] = &[
    "u32", "u64", "i32", "i64", "f32", "f64", "usize", "bool", "String",
];

/// Generates all code for an OpenDP R module.
/// Each call corresponds to one R file.
pub fn generate_r_module(
    module_name: &str,
    module: &Vec<Function>,
    hierarchy: &HashMap<String, Vec<String>>,
) -> String {
    let body = module
        .into_iter()
        .filter(|func| func.has_ffi)
        .filter(|func| !BLACKLIST.contains(&func.name.as_str()))
        .map(|func| generate_r_function(module_name, &func, hierarchy))
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        "# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL

{body}"
    )
}

/// Generate the code for a user-facing R function.
///
/// This internally calls a C function that wraps the Rust OpenDP Library
pub(crate) fn generate_r_function(
    module_name: &str,
    func: &Function,
    hierarchy: &HashMap<String, Vec<String>>,
) -> String {
    println!("generating R: {}", func.name);
    let mut args = (func.args.iter())
        .map(|arg| generate_r_input_argument(arg, func))
        .collect::<Vec<_>>();

    // move default arguments to end
    args.sort_by(|(_, l_is_default), (_, r_is_default)| l_is_default.cmp(r_is_default));

    let args = args.into_iter().map(|v| v.0).collect::<Vec<_>>();

    let then_func = if func.name.starts_with("make_") {
        let offset = if func.supports_partial { 2 } else { 0 };
        let pre_args_nl = if args.len() > 0 { "\n" } else { "" };
        format!(
            r#"

{then_docs}
{then_name} <- function(
{then_args}
) {{
{then_log}
  make_chain_dyn(
    {name}({pre_args_nl}{args}),
    lhs,
    log)
}}"#,
            then_docs = generate_then_doc_block(module_name, func, hierarchy),
            then_name = func.name.replacen("make_", "then_", 1),
            then_args = tab_r(
                once("lhs".to_string())
                    .chain(args[offset..].to_owned())
                    .collect::<Vec<_>>()
                    .join(",\n")
            ),
            then_log = tab_r(generate_logger(module_name, func, true)),
            name = func.name,
            args = tab_r(tab_r(tab_r(
                if func.supports_partial {
                    vec![
                        "output_domain(lhs)".to_string(),
                        "output_metric(lhs)".to_string(),
                    ]
                } else {
                    vec![]
                }
                .into_iter()
                .chain(func.args[offset..].iter().map(|arg| {
                    let name = if arg.is_type {
                        format!(".{}", arg.name())
                    } else {
                        sanitize_r(arg.name(), arg.is_type)
                    };
                    format!("{name} = {name}")
                }))
                .collect::<Vec<_>>()
                .join(",\n")
            )))
        )
    } else {
        String::default()
    };

    format!(
        r#"
{doc_block}
{func_name} <- function(
{args}
) {{
{body}
}}{then_func}
"#,
        doc_block = generate_doc_block(module_name, func, hierarchy),
        func_name = func.name.trim_start_matches("_"),
        args = tab_r(args.join(",\n")),
        body = tab_r(generate_r_body(module_name, func))
    )
}

/// generate an input argument, complete with name, hint and default.
/// also returns a bool to make it possible to move arguments with defaults to the end of the signature.
fn generate_r_input_argument(arg: &Argument, func: &Function) -> (String, bool) {
    let default = if let Some(default) = &arg.default {
        Some(match default.clone() {
            Value::Null => "NULL".to_string(),
            Value::Bool(value) => if value { "TRUE" } else { "FALSE" }.to_string(),
            Value::Integer(int) => format!("{}L", int),
            Value::Float(float) => float.to_string(),
            Value::String(mut string) => {
                if arg.is_type {
                    string = arg.generics.iter().fold(string, |string, generic| {
                        // TODO: kind of hacky. avoids needing a full parser for the type string
                        // replace all instances of the generic with .generic
                        string
                            .replace(&format!("{},", generic), &format!(".{},", generic))
                            .replace(&format!("{}>", generic), &format!(".{}>", generic))
                            .replace(&format!("{})", generic), &format!(".{})", generic))
                    });
                }
                format!("\"{}\"", string)
            }
        })
    } else {
        // let default value be None if it is a type arg and there is a public example
        generate_public_example(func, arg).map(|_| "NULL".to_string())
    };
    (
        format!(
            r#"{name}{default}"#,
            name = sanitize_r(arg.name(), arg.is_type),
            default = default
                .as_ref()
                .map(|default| format!(" = {}", default))
                .unwrap_or_else(String::new)
        ),
        default.is_some(),
    )
}

fn find_quoted(s: &str, pat: char) -> Option<(usize, usize)> {
    let left = s.find(pat)?;
    let right = left + 1 + s[left + 1..].find(pat)?;
    Some((left, right))
}

// R wants special formatting for latex
fn escape_latex(s: &str) -> String {
    // if a substring is enclosed in latex "$"
    if let Some((l, u)) = find_quoted(s, '$') {
        [
            escape_latex(&s[..l]).as_str(),
            "\\eqn{",
            &s[l + 1..u],
            "}",
            escape_latex(&s[u + 1..]).as_str(),
        ]
        .join("")
    } else {
        s.to_string()
    }
}

// docstrings in R lead with a one-line title
fn generate_constructor_title(name: &String) -> String {
    if name.starts_with("make") {
        format!(
            "{} constructor\n\n",
            name.trim_start_matches("make_").replace("_", " ")
        )
    } else {
        String::new()
    }
}

/// generate a documentation block for the current function, with the function description, args, and return
/// in Roxygen format: https://mpn.metworx.com/packages/roxygen2/7.1.1/articles/rd-formatting.html
fn generate_doc_block(
    module_name: &str,
    func: &Function,
    hierarchy: &HashMap<String, Vec<String>>,
) -> String {
    let title = generate_constructor_title(&func.name);
    let description = (func.description.as_ref())
        .map(|v| {
            v.split("\n")
                .map(escape_latex)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .map(|v| format!("{title}{}\n", v))
        .unwrap_or_else(String::new);

    let concept = format!("@concept {}\n", module_name);

    let doc_args = (func.args.iter())
        .map(|v| generate_docstring_arg(v))
        .collect::<Vec<String>>()
        .join("\n");

    let export = if !func.name.starts_with("_") && module_name != "data" {
        "\n@export"
    } else {
        ""
    };

    format!(
        r#"{description}
{concept}{doc_args}{ret_arg}{examples}{export}"#,
        concept = concept,
        description = description,
        doc_args = doc_args,
        ret_arg = generate_docstring_return_arg(&func.ret, hierarchy),
        examples = generate_docstring_examples(module_name, func),
    )
    .split("\n")
    .map(|l| format!("#' {}", l).trim().to_string())
    .collect::<Vec<_>>()
    .join("\n")
}

/// generate a documentation block for a then_* partial constructor, with the function description, args, and return
/// in Roxygen format: https://mpn.metworx.com/packages/roxygen2/7.1.1/articles/rd-formatting.html
fn generate_then_doc_block(
    module_name: &str,
    func: &Function,
    hierarchy: &HashMap<String, Vec<String>>,
) -> String {
    let title = generate_constructor_title(&func.name);
    let offset = if func.supports_partial { 2 } else { 0 };

    let doc_args = (func.args[offset..].iter())
        .map(|v| generate_docstring_arg(v))
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        r#"partial {title}See documentation for [{func_name}()] for details.

@concept {module_name}
@param lhs The prior transformation or metric space.
{doc_args}{ret_arg}{examples}
@export"#,
        func_name = func.name,
        ret_arg = generate_docstring_return_arg(&func.ret, hierarchy),
        examples = generate_docstring_examples(module_name, func),
    )
    .split("\n")
    .map(|l| format!("#' {}", l).trim().to_string())
    .collect::<Vec<_>>()
    .join("\n")
}

/// generate the part of a docstring corresponding to an argument
fn generate_docstring_arg(arg: &Argument) -> String {
    let name = sanitize_r(arg.name(), arg.is_type);
    format!(
        r#"@param {name} {description}"#,
        name = name,
        description = arg
            .description
            .as_ref()
            .map(|v| escape_latex(v.as_str()))
            .unwrap_or_else(|| "undocumented".to_string())
    )
}

/// generate the part of a docstring corresponding to a return argument
fn generate_docstring_return_arg(
    arg: &Argument,
    hierarchy: &HashMap<String, Vec<String>>,
) -> String {
    let description = if let Some(description) = &arg.description {
        description.clone()
    } else if let Some(type_) = arg.python_type_hint(hierarchy) {
        type_
    } else {
        return String::new();
    };
    format!("\n@return {description}")
}

fn generate_docstring_examples(module_name: &str, func: &Function) -> String {
    let example_path = format!("src/{}/code/{}.R", &module_name, &func.name);
    match fs::read_to_string(example_path) {
        Ok(example) => format!("\n@examples\n{example}"),
        Err(_) => "".to_string(),
    }
}

/// generate the function body, consisting of type args formatters, data converters, and the call
/// - type arg formatters make every type arg a RuntimeType, and construct derived RuntimeTypes
/// - data converters convert from python to c representations according to the formatted type args
/// - the call constructs and retrieves the ffi function name, sets ctypes,
///     makes the call, handles errors, and converts the response to python
fn generate_r_body(module_name: &str, func: &Function) -> String {
    format!(
        r#"{deprecated}{flag_checker}{type_arg_formatter}{logger}{assert_is_similar}
{make_call}
output"#,
        deprecated = generate_deprecated(func),
        flag_checker = generate_flag_check(&func.features),
        type_arg_formatter = generate_type_arg_formatter(func),
        assert_is_similar = generate_assert_is_similar(func),
        logger = generate_logger(module_name, func, false),
        make_call = generate_wrapper_call(module_name, func)
    )
}

/// generate code that provides an example of the type of the type_arg
fn generate_public_example(func: &Function, type_arg: &Argument) -> Option<String> {
    // the json has supplied explicit instructions to find an example
    if let Some(example) = &type_arg.example {
        return Some(example.to_r(Some(&func.type_names())));
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
    args.iter().find_map(|arg| match &arg.rust_type {
        Some(TypeRecipe::Name(name)) => (name == type_name).then(|| arg.name()),
        Some(TypeRecipe::Nest { origin, args }) => {
            if origin != "Vec" {
                return None;
            }
            let TypeRecipe::Name(arg_name) = &args[0] else {
                return None;
            };
            if arg_name != type_name {
                return None;
            }

            Some(format!("get_first({name})", name = arg.name()))
        }
        _ => None,
    })
}

/// the generated code ensures every type arg is a RuntimeType, and constructs derived RuntimeTypes
fn generate_type_arg_formatter(func: &Function) -> String {
    let type_names = func.type_names();
    let type_arg_formatter: String = func.args.iter()
        .filter(|arg| arg.is_type)
        .map(|type_arg| {
            let name = sanitize_r(type_arg.name(), type_arg.is_type);
            let generics = if type_arg.generics.is_empty() {
                "".to_string()
            } else {
                format!(", generics = list({})", type_arg.generics.iter()
                    .map(|v| format!("\"{}\"", sanitize_r(v, true)))
                    .collect::<Vec<_>>().join(", "))
            };
            if let Some(example) = generate_public_example(func, type_arg) {
                format!(r#"{name} <- parse_or_infer(type_name = {name}, public_example = {example}{generics})"#)
            } else {
                format!(r#"{name} <- rt_parse(type_name = {name}{generics})"#)
            }
        })
        // additional types that are constructed by introspecting existing types
        .chain(func.derived_types.iter()
            .map(|type_spec|
                format!("{name} <- {derivation}",
                        name = sanitize_r(type_spec.name(), true),
                        derivation = type_spec.rust_type.as_ref().unwrap().to_r(Some(&type_names)))))
        // substitute concrete types in for generics
        .chain(func.args.iter()
            .filter(|arg| !arg.generics.is_empty())
            .map(|arg|
                format!("{name} <- rt_substitute({name}, {args})",
                        name=sanitize_r(arg.name(), true),
                        args=arg.generics.iter()
                            .map(|generic| format!("{generic} = {generic}", generic = sanitize_r(generic, true)))
                            .collect::<Vec<_>>().join(", "))))
        // determine types of arguments that are not type args
        .chain(func.args.iter().filter(|arg| arg.has_implicit_type())
            .map(|arg| {
                let name = sanitize_r(arg.name(), arg.is_type);
                let converter = arg.rust_type.as_ref().unwrap().to_r(Some(&type_names));
                format!(r#".T.{name} <- {converter}"#)
            }))
        .collect::<Vec<_>>()
        .join("\n");

    if type_arg_formatter.is_empty() {
        "# No type arguments to standardize.".to_string()
    } else {
        format!(
            r#"# Standardize type arguments.
{type_arg_formatter}
"#
        )
    }
}

fn generate_assert_is_similar(func: &Function) -> String {
    let assert_is_similar: String = (func.args.iter())
        .filter(|arg| !arg.is_type)
        .map(|arg| {
            let expected = if arg.has_implicit_type() {
                let name = sanitize_r(arg.name(), arg.is_type);
                format!(".T.{name}")
            } else {
                arg.rust_type
                    .as_ref()
                    .unwrap()
                    .to_r(Some(&func.type_names()))
            };
            let inferred = {
                let name = sanitize_r(arg.name(), arg.is_type);
                let generics = if arg.generics.is_empty() {
                    "".to_string()
                } else {
                    format!(
                        ", generics = list({})",
                        arg.generics
                            .iter()
                            .map(|v| format!("\"{}\"", sanitize_r(v, true)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                format!(
                    "rt_infer({name}{generics})",
                    name = name,
                    generics = generics
                )
            };
            (expected, inferred)
        })
        .filter(|(expected, _)| expected != "NULL")
        .map(|(expected, inferred)| {
            format!("rt_assert_is_similar(expected = {expected}, inferred = {inferred})")
        })
        .collect::<Vec<_>>()
        .join("\n");

    if assert_is_similar.is_empty() {
        "".to_string()
    } else {
        format!(
            r#"
# Assert that arguments are correctly typed.
{assert_is_similar}
"#
        )
    }
}

// generates the `log <- ...` code that tracks arguments
fn generate_logger(module_name: &str, func: &Function, then: bool) -> String {
    let func_name = if then {
        func.name.replacen("make_", "then_", 1)
    } else {
        func.name.clone()
    };
    let offset = if then && func.supports_partial { 2 } else { 0 };
    let keys = func.args[offset..]
        .iter()
        .map(|arg| format!("\"{}\"", arg.name()))
        .collect::<Vec<_>>()
        .join(", ");

    let vals = func.args[offset..]
        .iter()
        .map(|arg| {
            let r_name = sanitize_r(arg.name().clone(), arg.is_type);
            generate_recipe_logger(arg.rust_type.clone().unwrap_or(TypeRecipe::None), r_name)
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"
log <- new_constructor_log("{func_name}", "{module_name}", new_hashtab(
  list({keys}),
  list({vals})
))
"#
    )
}

// adds unboxes wherever possible to `name`, based on its type `recipe`
fn generate_recipe_logger(recipe: TypeRecipe, name: String) -> String {
    match recipe {
        TypeRecipe::Name(ty) if ATOM_TYPES.contains(&ty.as_str()) => format!("unbox2({name})"),
        TypeRecipe::Nest { origin, .. } if origin == "Tuple" => {
            format!("lapply({name}, unbox2)")
        }
        _ => name,
    }
}

/// the generated code
/// - constructs and retrieves the ffi function name
/// - calls the C wrapper with arguments *and* extra type info
fn generate_wrapper_call(module_name: &str, func: &Function) -> String {
    let args = (func.args.iter())
        .chain(func.derived_types.iter())
        .map(|arg| sanitize_r(arg.name(), arg.is_type))
        .chain(
            (func.args.iter().filter(|arg| arg.has_implicit_type()))
                .map(|arg| sanitize_r(arg.name(), arg.is_type))
                .map(|name| format!("rt_parse(.T.{name})")),
        )
        .map(|name| format!("{name},"))
        .collect::<Vec<_>>()
        .join(" ");

    let args_str = if args.is_empty() {
        "".to_string()
    } else {
        format!("\n  {args}")
    };

    let call = format!(
        r#".Call(
  "{module_name}__{name}",{args_str}
  log, PACKAGE = "opendp")"#,
        name = func.name
    );
    format!(
        r#"# Call wrapper function.
output <- {call}"#
    )
}

// generate call to ".Deprecated()" if needed
fn generate_deprecated(func: &Function) -> String {
    if let Some(deprecation) = &func.deprecation {
        format!(
            ".Deprecated(msg = \"{}\")\n",
            deprecation.note.replace("\"", "\\\"")
        )
    } else {
        String::default()
    }
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

// ensures that `name` is a valid R variable name, and prefixes types with dots
fn sanitize_r<T: AsRef<str> + ToString>(name: T, is_type: bool) -> String {
    let mut name = name.to_string();
    if is_type {
        name = format!(".{}", name);
    }
    let blacklist = ["function", "T", "F"];
    if blacklist.contains(&name.as_ref()) {
        name = format!("{}_", name)
    }
    name
}

impl Argument {
    // R wants to resolve all types on the R side, before passing them to C
    // some function arguments contain their own nontrivial types/TypeRecipes,
    //     like `Vec<T>` or `measurement_input_carrier_type(this)`
    pub fn has_implicit_type(&self) -> bool {
        !self.is_type && !matches!(self.rust_type, Some(TypeRecipe::Name(_) | TypeRecipe::None))
    }
}

impl Function {
    pub fn type_names(&self) -> Vec<String> {
        (self.args.iter())
            .filter(|arg| arg.is_type)
            .map(|arg| arg.name())
            .chain(self.derived_types.iter().map(Argument::name))
            .collect()
    }
}

impl TypeRecipe {
    /// translate the abstract derived_types info into R RuntimeType constructors
    pub fn to_r(&self, sanitize_types: Option<&[String]>) -> String {
        match self {
            Self::Name(name) => sanitize_types
                .map(|types| sanitize_r(name, types.contains(name)))
                .unwrap_or_else(|| name.to_string()),
            Self::Function { function, params } => format!(
                "{function}({params})",
                function = function,
                params = params
                    .iter()
                    .map(|v| v.to_r(sanitize_types))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Nest { origin, args } => format!(
                "new_runtime_type(origin = \"{origin}\", args = list({args}))",
                origin = origin,
                args = args
                    .iter()
                    .map(|arg| arg.to_r(sanitize_types))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::None => "NULL".to_string(),
        }
    }
}
