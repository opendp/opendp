use std::collections::HashMap;

pub mod parse;

// metadata for each function in a module
#[derive(Debug)]
pub struct Function {
    // plaintext description of the function used to generate documentation
    pub description: Option<String>,
    // URL pointing to the location of the DP proof for the function
    pub proof: Option<String>,
    // required feature flags to execute function
    pub features: Vec<String>,
    // arguments and generics
    pub args: Vec<Argument>,
    // metadata for constructing new types based on existing types or introspection
    pub derived_types: Vec<Argument>,
    // metadata for return type
    pub ret: Argument,
}

// Metadata for function arguments, derived types and returns.
#[derive(Debug, Default, Clone)]
pub struct Argument {
    // argument name. Optional for return types
    pub name: Option<String>,
    // c type to translate to/from for FFI. Optional for derived types
    pub c_type: Option<String>,
    // RuntimeType expressed in terms of rust types with generics.
    // Includes various RuntimeType constructors
    pub rust_type: Option<RuntimeType>,
    // type hint- a more abstract type that all potential arguments inherit from
    pub hint: Option<String>,
    // plaintext description of the argument used to generate documentation
    pub description: Option<String>,
    // default value for the argument
    pub default: Option<Value>,
    // a list of names in the default that should be considered generics
    pub generics: Vec<String>,
    // set to true if the argument represents a type
    pub is_type: bool,
    // most functions convert c_to_py or py_to_c. Set to true to leave the value as-is
    // an example usage is slice_as_object,
    //  to prevent the returned AnyObject from getting converted back to python
    pub do_not_convert: bool,
    // when is_type, use this as an example to infer the type
    pub example: Option<RuntimeType>
}

#[derive(Debug, Default, Clone)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    String(String),
    Integer(i64),
    Float(f64),
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
        if let Some(RuntimeType::Nest { origin, args }) = &self.rust_type {
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
#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeType {
    // reference an existing RuntimeType
    Name(String),
    // build a higher level RuntimeType
    Nest { origin: String, args: Vec<RuntimeType> },
    // explicitly absent
    None,
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
            Self::Function { function, params } =>
                format!("{function}({params})", function = function, params = params.iter()
                    .map(|v| v.to_python())
                    .collect::<Vec<_>>().join(", ")),
            Self::Nest { origin, args } =>
                format!("RuntimeType(origin='{origin}', args=[{args}])",
                        origin = origin,
                        args = args.iter().map(|arg| arg.to_python()).collect::<Vec<_>>().join(", ")),
            Self::None => "None".to_string()
        }
    }
}
