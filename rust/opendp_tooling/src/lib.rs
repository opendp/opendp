// logic for the bootstrap macro
pub mod bootstrap;

// logic for the proven macro
pub mod proven;

// logic for generating bindings from the core structures below
pub mod codegen;

#[derive(Debug)]
pub struct Deprecation {
    since: String,
    note: String,
}

// metadata for each function in a module
#[derive(Debug)]
pub struct Function {
    pub name: String,
    // plaintext description of the function used to generate documentation
    pub description: Option<String>,
    // required feature flags to execute function
    pub features: Vec<String>,
    // arguments and generics
    pub args: Vec<Argument>,
    // metadata for constructing new types based on existing types or introspection
    pub derived_types: Vec<Argument>,
    // metadata for return type
    pub ret: Argument,
    // references to values that should share the same lifetime
    pub dependencies: Vec<TypeRecipe>,
    // set to true if the first two arguments are input domain and input metric
    pub supports_partial: bool,
    // whether to generate FFI
    pub has_ffi: bool,
    // deprecation warning if applicable
    pub deprecation: Option<Deprecation>,
}

// Metadata for function arguments, derived types and returns.
#[derive(Debug, Default, Clone)]
pub struct Argument {
    // argument name. Optional for return types
    pub name: Option<String>,
    // c type to translate to/from for FFI. Optional for derived types
    pub c_type: Option<String>,
    // directions to construct a Rust type with generics
    pub rust_type: Option<TypeRecipe>,
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
    pub example: Option<TypeRecipe>,
}

// TypeRecipe contains the metadata to generate code that evaluates to a rust type name
#[derive(Debug, PartialEq, Clone)]
pub enum TypeRecipe {
    // reference an existing type
    Name(String),
    // build up a rust type from other rust types
    Nest {
        origin: String,
        args: Vec<TypeRecipe>,
    },
    // explicitly absent
    None,
    // construct the rust type via function call
    Function {
        function: String,
        params: Vec<TypeRecipe>,
    },
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
