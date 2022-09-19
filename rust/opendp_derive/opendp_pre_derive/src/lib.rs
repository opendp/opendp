use std::collections::HashMap;

pub mod target;

use serde::{Deserialize, Serialize, Deserializer};
use serde_json::Value;


// metadata for each function in a module
#[derive(Deserialize, Serialize, Debug)]
pub struct Function {
    // plaintext description of the function used to generate documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // URL pointing to the location of the DP proof for the function
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>,
    // required feature flags to execute function
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    // arguments and generics
    #[serde(default)]
    pub args: Vec<Argument>,
    // metadata for constructing new types based on existing types or introspection
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_types: Vec<Argument>,
    // metadata for return type
    #[serde(default)]
    pub ret: Argument,
}

// Metadata for function arguments, derived types and returns.
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Argument {
    // argument name. Optional for return types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    // c type to translate to/from for FFI. Optional for derived types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_type: Option<String>,
    // RuntimeType expressed in terms of rust types with generics.
    // Includes various RuntimeType constructors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_type: Option<RuntimeType>,
    // a list of names in the rust_type that should be considered generics
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generics: Vec<String>,
    // type hint- a more abstract type that all potential arguments inherit from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    // plaintext description of the argument used to generate documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // default value for the argument
    #[serde(default, deserialize_with = "deserialize_some", skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    // set to true if the argument represents a type
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_type: bool,
    // most functions convert c_to_py or py_to_c. Set to true to leave the value as-is
    // an example usage is slice_as_object,
    //  to prevent the returned AnyObject from getting converted back to python
    #[serde(default, skip_serializing_if = "is_false")]
    pub do_not_convert: bool,
    // when is_type, use this as an example to infer the type
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<RuntimeType>
}


impl Argument {
    /// retrieve the python ctype corresponding to the type inside FfiResult<*>
    pub fn python_unwrapped_ctype(&self, typemap: &HashMap<String, String>) -> String {
        assert_eq!(&self.c_type()[..9], "FfiResult");
        typemap.get(&self.c_type()[10..self.c_type().len() - 1]).unwrap().clone()
    }
    /// retrieve the python ctypes corresponding to the origin of a type (subtypes/args omitted)
    pub fn python_origin_ctype(&self, typemap: &HashMap<String, String>) -> String {
        typemap.get(&self.c_type_origin()).cloned().expect("ctype not recognized in typemap")
    }
    pub fn python_type_hint(&self, hierarchy: &HashMap<String, Vec<String>>) -> Option<String> {
        if self.hint.is_some() {
            return self.hint.clone()
        }
        if self.is_type {
            return Some("RuntimeTypeDescriptor".to_string())
        }
        if let Some(RuntimeType::Raise { origin, args }) = &self.rust_type {
            if origin == "Tuple" {
                return Some(format!("Tuple[{}]", vec!["Any"; args.len()].join(", ")))
            }
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


fn is_false(v: &bool) -> bool {
    !v
}

// deserialize "k": null as `Some(Value::Null)` and no key as `None`.
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where T: Deserialize<'de>, D: Deserializer<'de> {
    Deserialize::deserialize(deserializer).map(Some)
}

#[allow(dead_code)]
impl Argument {
    pub fn name(&self) -> String {
        self.name.clone().expect("unknown name when parsing argument")
    }
    pub fn c_type(&self) -> String {
        if self.is_type {
            if self.c_type.is_some() { panic!("c_type should not be specified when is_type") }
            return "char *".to_string()
        }
        self.c_type.clone().expect("unknown c_type when parsing argument")
    }
    pub fn c_type_origin(&self) -> String {
        self.c_type().split('<').next().unwrap().to_string()
    }
}

// RuntimeType contains the metadata to generate code that evaluates to a rust type name
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum RuntimeType {
    // reference an existing RuntimeType
    Name(String),
    // get the ith subtype of an existing RuntimeType
    Lower { root: Box<RuntimeType>, index: i32 },
    // build a higher level RuntimeType
    Raise { origin: String, args: Vec<RuntimeType> },
    // construct the RuntimeType via function call
    Function { function: String, params: Vec<RuntimeType> },
}

impl<S: Into<String>> From<S> for RuntimeType {
    fn from(name: S) -> Self {
        RuntimeType::Name(name.into())
    }
}


impl RuntimeType {
    /// translate the abstract derived_types info into python RuntimeType constructors
    pub fn to_python(&self) -> String {
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

