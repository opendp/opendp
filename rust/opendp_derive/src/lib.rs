use proc_macro::TokenStream;

#[cfg(feature = "full")]
mod full;

/// When the opendp crate is compiled with the "derive" feature,
/// the bootstrap procedural macro is executed on the function it decorates, before the library is compiled.
///
/// We use this to insert a link to the proof in the documentation of the function, if an adjacent proof exists.
///
/// We also use this to extract the rust documentation, so that we can reuse it in external language bindings.
/// The bootstrap macro also looks at the names and types of arguments, the generics used, and the return type
/// to automatically generate additional documentation and technical information needed for language bindings.
///
/// External language bindings (like python) oftentimes need some additional metadata--
/// like the default value of arguments or types, or how to infer type information from arguments.
/// This additional metadata is passed directly to the proc-macro itself.
///
/// # Linking Proofs
///
/// The proc-macro will look for a proof file for your function. If you are proving `fn my_function`,
/// and a file named `my_function.tex` is found on the filesystem,
/// the bootstrap macro will insert a link to that file into the documentation.
/// Depending on the environment, this link will go directly to a versioned docs site, or to a local file.
///
/// you can also specify the location of the proof file directly:
/// ```compile_fail
/// #[bootstrap(
///     proof = "transformations/make_clamp.tex"
/// )]
/// fn my_func() {}
/// ```
/// Expands to:
/// ```no_run
/// /// [(Proof Link)](link/to/proof.pdf)
/// fn my_func() {}
/// ```
///
/// Note that when you specify the location of the proof file, it should be relative to the src/ directory.
///
/// # Features
/// You can indicate a list of features that must be enabled by the user for the function to exist.
/// ```compile_fail
/// #[bootstrap(
///     features("contrib", "floating-point")
/// )]
/// ```
/// It is recommended to specify features through the `bootstrap` function, not via `cfg` attributes,
/// because features listed via `bootstrap` are present in external language bindings.
///
/// # Name
/// In some situations, you want the name of the function in external languages to be different
/// from the name of the function in the rust code.
/// This is useful in situations where the bootstrap macro is invoked directly on extern "C" functions,
/// like in the core and data modules. You can ignore this in most contexts.
/// ```compile_fail
/// #[bootstrap(
///     name = "my_func_renamed"
/// )]
/// fn my_func() {}
/// ```
/// Generates in Python:
/// ```compile_fail
/// def my_func_renamed():
///     pass
/// ```
///
/// # Arguments, Generics and Return
/// You can pass additional metadata that is specific to each argument or generic.
///
/// ```compile_fail
/// #[bootstrap(
///     // can specify multiple. Everything is optional
///     arguments(my_arg1(default = "example"), my_arg2(hint = "int")),
///     // can specify multiple. Everything is optional
///     generics(T(default = "AtomDomain<f64>")),
///     // same syntax as inside an argument or generic. Optional
///     returns(c_type = "FfiResult<AnyTransformation *>")
/// )]
/// ```
///
/// The rest of this doc comment documents how to specify metadata for
/// specific arguments, specific generics and the return value.
///
/// ## Default Value
/// You can set the default value for arguments and generics in bindings languages:
/// ```compile_fail
/// #[bootstrap(
///     arguments(value(default = -1074)),
///     generics(T(default = "i32"))
/// )]
/// fn my_func<T: Number>(value: T) -> T {
///     value
/// }
/// ```
/// This can make the library more accessible by making some arguments optional with sensible defaults.
///
/// When specifying the default value for types, you can also specify "int" or "float" instead of concrete types like "i8" or "f32".
/// "int" refers to the default concrete type for ints, and respectively for floats.
/// The default concrete types for ints and floats can be configured by users.
///
/// ## Generics in Default Values
/// This metadata is specific to default values for generic arguments.
/// It is used when you want to specify a default type,
/// but let the atomic type be filled in based on context.
/// ```compile_fail
/// #[bootstrap(
///     generics(
///         D(default = "AtomDomain<T>", generics = "T"),
///         M(default = "AbsoluteDistance<T>", generics = "T")),
///     derived_types(T = "$get_atom_or_infer(D, constant)")
/// )]
/// fn my_func<D: SomeDomain, M>(constant: D::Atom) {
///     unimplemented!()
/// }
/// ```
/// In the above example, when you pass a constant to the function, the type T is first derived via the "$" macro.
/// It is then substituted in-place of T in the generics `D` and `M`.
/// You can separate multiple generics with commas.
///
/// ## Type Example
/// It is often unnecessary for a user to specify the type T, because it can be inferred from other arguments.
/// In the previous example, the generated bindings for `my_func` will treat the argument `T` as optional,
/// because it can be inferred from `value`.
///
/// This breaks down when the public example is embedded in another data structure, like a tuple.
/// You can provide instructions on how to retrieve an example that can be used to infer a type:
/// ```compile_fail
/// #[bootstrap(
///     generics(T(example = "$get_first(bounds)"))
/// )]
/// fn my_func<T>(bounds: (T, T)) -> T {
///     unimplemented!()
/// }
/// ```
/// In the generated code, the type argument T becomes optional.
/// If T is not set, T is inferred from the output of the "$" macro-- inferred from the first bound.
/// The generated code will run a function, `get_first`, that has been defined locally in the bindings language.
///
/// You can specify a null default value with the byte-string literal `b"null"`.
///
/// ## C Type
/// When used by other languages, data is passed to and from the OpenDP library via C interfaces.
///
/// The bootstrap macro will infer the correct C type in most situations by inspecting the types in the Rust signature.
/// There are some cases that are ambiguous, though.
/// For example, the inferred C type is always "AnyObject *" if the rust type is generic.
/// However, it is not always necessary to construct an AnyObject; a raw pointer will do.
///
/// In the following example, the generated bindings would expect `value` to be passed as an "AnyObject *" because it is generic.
/// ```compile_fail
/// #[bootstrap()]
/// fn my_func<T: Number>(value: T) -> T {
///     value
/// }
/// ```
/// However, we know `value` is a number that we can place behind a simple raw pointer "void *".
/// ```compile_fail
/// #[bootstrap(
///     arguments(value(c_type = "void *"))
/// )]
/// fn my_func<T: Number>(value: T) -> T {
///     value
/// }
/// ```
/// With this annotation, the extern function associated with `my_func` should accept a "value: c_void".
/// The extern function will downcast the pointer to a concrete rust type based on the argument "T".
///
/// Note that it is not meaningful to set the C type on generics, as generics are always passed as a "char *".
///
/// ## Rust Type
/// For generic arguments, when unpacking data behind "void *" or "AnyObject *",
/// it is necessary to know the concrete type to downcast to.
/// This is the purpose of the "rust_type" metadata.
/// Generally speaking, the bootstrap macro infers the rust type by reading the function arguments in the Rust signature.
/// In the previous example, the rust type for `value` is automatically inferred to be "T".
///
/// The rust type for `bounds` in the following function is `(T, T)`.
/// ```compile_fail
/// #[bootstrap()]
/// fn my_func<T>(bounds: (T, T)) -> T {
///     unimplemented!()
/// }
/// ```
/// It is again unnecessary to pass additional metadata about the bounds' rust_type because it is inferred from the Rust signature.
///
/// Now consider if each bound is wrapped in an enum that indicates if the value is inclusive or exclusive.
/// It is unclear what the memory layout of the bound enum is in C.
/// For simplicity, the extern C function slightly changes the interface to be less general by assuming that the bounds are inclusive.
/// We can communicate this to the bindings generation by setting the rust type manually.
/// ```compile_fail
/// #[bootstrap(
///     arguments(bounds(rust_type = "(T, T)")),
///     generics(TA(example = "$get_first(bounds)"))
/// )]
/// fn make_unclamp<T>(bounds: (Bound<T>, Bound<T>)) -> S::Atom {
///     unimplemented!()
/// }
/// ```
/// In this case, the extern C function is written to expect `bounds: AnyObject *`,
/// the default C type for any nontrivial or generic data structure.
/// The extern function downcasts said AnyObject to the rust type `(T, T)` and wraps each bound in Bound::Inclusive.
/// This constructor manually specified the rust_type to smooth over a small difference between the api of the rust function,
/// and the api of the extern function.
///
///
/// In a slightly more complicated example, consider the case where the function is generic over some type `S`.
/// `S` has an associated type `S::Atom`, and the function expects bounds of type `S::Atom`.
/// The bootstrap macro doesn't know how to handle these associated types.
///
/// We can instead derive a type, by saying there exists some type T that is inferred from the first bound.
/// As a developer we have knowledge that our naive tooling doesn't-- that T is the same as S::Atom.
/// We can then use this inferred type to specify the rust type.
/// ```compile_fail
/// #[bootstrap(
///     arguments(bounds(rust_type = "(T, T)")),
///     derived_types(T = "$get_first(bounds)")
/// )]
/// fn my_func<S: Summable>(bounds: (S::Atom, S::Atom)) -> S::Atom {
///     unimplemented!()
/// }
/// ```
/// In the bindings, the type T is inferred from the first bound,
/// and it packs the tuple data structure into an AnyObject of type `(T, T)`.
/// The implementation of the rust extern "C" function downcasts the AnyObject to an `(S::Item, S::Item)`.
/// This pattern is used often in the sum constructors.
///
/// It is also possible to specify the rust type via a macro.
/// Here is an example for the invoke function:
/// ```compile_fail
/// #[bootstrap(
///     name = "measurement_invoke",
///     arguments(
///         this(rust_type = b"null"),
///         arg(rust_type = "$measurement_input_carrier_type(this)")
///     )
/// )]
/// #[no_mangle]
/// pub extern "C" fn opendp_core__measurement_invoke(
///     this: *const AnyMeasurement,
///     arg: *const AnyObject
/// ) -> FfiResult<*mut AnyObject> {
///     unimplemented!()
/// }
/// ```
/// First off, the rust type of `this` is marked as null to indicate that we don't want to perform any conversions between C and rust types.
/// Secondly, the rust type of the argument should always be the input carrier type on the measurement.
/// We use another function `measurement_input_carrier_type`, defined in the bindings, to retrieve this value.
/// Thus `arg` is always an instance of the Rust type that the measurement expects.
///
/// Note that it is not meaningful to set the rust type on generics,
/// as generics are just type information, passed as a string.
///
/// ## Hints
/// This metadata contains a fair amount of Python-only syntax.
/// It is added as a type hint to the argument in Python.
/// ```compile_fail
/// #[bootstrap(
///     generics(MO(hint = "SensitivityMetric"))
/// )]
/// fn my_func<MO: SensitivityMetric>() {}
/// ```
/// The above type MO has a trait bound `SensitivityMetric` that restricts the set of possible types MO can be.
///
/// The generated code downgrades this to just a type hint.
/// ```compile_fail
/// def my_func(
///     MO: SensitivityMetric
/// ):
///     pass
/// ```
/// The extern "C" rust function would throw an error if any type that is not a sensitivity metric were passed.
///
/// ## Suppress
/// In some cases you want generated code not to include a type argument for a particular generic,
/// because it is unambiguously determined by another argument.
/// This comes up often when the type is already captured in the `input_domain` or `input_metric` argument.
/// ```compile_fail
/// #[bootstrap(
///     generics(DI(suppress))
/// )]
/// fn my_func<DI>(input_domain: DI) {}
/// ```
///
/// In this case, the generated code will not include the type argument for DI.
/// The extern "C" function will get a handle on DI by introspecting the received `AnyDomain` struct.
///
/// ## Do Not Convert
/// By default, generated bindings code always calls a function to convert between rust types and C types.
/// This can be disabled on individual arguments by specifying `do_not_convert = true`.
/// This is typically only useful on the innermost structural utilities,
/// like when converting from an FfiSlice to an AnyObject or vice versa.
///
/// ## Dependencies
/// When the return value of the function contains data whose memory has been allocated by a foreign language,
/// the data is at risk of being freed.
/// This is because foreign languages may not know the return value is still holding a reference to the data.
///
/// For example, when you construct a domain that holds a member check function that was allocated in Python.
/// ```compile_fail
/// #[bootstrap(
///     dependencies(member)
/// )]
/// fn example_user_domain<DI>(member: CallbackFn) -> AnyDomain {
///     pack_function_into_domain(member)
/// }
/// ```
///
/// The generated code will add a reference to `member` to the return type, which keeps the refcount up.
/// ```compile_fail
/// def example_user_domain(
///     member: Callable[[Any], bool]
/// ):
///     result = lib.example_user_domain(member)
///     setattr(result, "_dependencies", member)
///     return result
/// ```
///
#[cfg(feature = "full")]
#[proc_macro_attribute]
pub fn bootstrap(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    full::bootstrap(attr_args, input)
}

/// When the "derive" crate feature is not enabled, no work is done, and dependencies are simplified.
///
#[cfg(not(feature = "full"))]
#[proc_macro_attribute]
pub fn bootstrap(_attr_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

/// This shares the same interface as the bootstrap macro, but only accepts the first two arguments:
/// proof_link, and features.
///
/// This macro differs from bootstrap in that it is much simpler,
/// and is meant to be used on internal functions that don't get foreign language bindings.
/// This macro throws a compile error if the proof file cannot be found.
///
/// This macro can also be affixed on trait and struct impls.
/// When used on a trait impl, it looks for `TraitName.tex`,
/// and when used on a struct impl, it looks for `StructName.tex`.
///
#[cfg(feature = "full")]
#[proc_macro_attribute]
pub fn proven(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    full::proven(attr_args, input)
}

#[cfg(not(feature = "full"))]
#[proc_macro_attribute]
pub fn proven(_attr_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}
