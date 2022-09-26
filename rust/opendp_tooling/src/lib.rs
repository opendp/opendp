pub mod bootstrap;
pub mod proof;
pub mod codegen;

// metadata for each function in a module
#[derive(Debug)]
pub struct Function {
    pub name: String,
    // plaintext description of the function used to generate documentation
    pub description: Option<String>,
    // relative path to the location of the DP proof for the function
    pub proof_path: Option<String>,
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

// holds literal values, like for default
#[derive(Debug, Default, Clone)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    String(String),
    Integer(i64),
    Float(f64),
}
