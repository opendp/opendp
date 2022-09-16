use indexmap::map::IndexMap;
use serde::{Deserialize, Serialize, Deserializer};
use serde_json::Value;

// a module contains functions by name
#[allow(dead_code)]
pub type Module = IndexMap<String, Function>;

// metadata for each function in a module
#[derive(Deserialize, Serialize, Debug)]
pub struct Function {
    #[serde(default)]
    pub(crate) args: Vec<Argument>,
    // metadata for return type
    #[serde(default)]
    pub ret: Argument,
    // required feature flags to execute function
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    // metadata for constructing new types based on existing types or introspection
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_types: Vec<Argument>,
    // plaintext description of the function used to generate documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // URL pointing to the location of the DP proof for the function
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>
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
    fn name(&self) -> String {
        self.name.clone().expect("unknown name when parsing argument")
    }
    fn c_type(&self) -> String {
        if self.is_type {
            if self.c_type.is_some() { panic!("c_type should not be specified when is_type") }
            return "char *".to_string()
        }
        self.c_type.clone().expect("unknown c_type when parsing argument")
    }
    fn c_type_origin(&self) -> String {
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
