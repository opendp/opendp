use std::collections::HashMap;
use syn::{AttributeArgs, ItemFn};

use crate::{Argument, Function, TypeRecipe};

pub mod arguments;
pub mod docstring;
pub mod signature;

pub mod partial;

use darling::{Error, Result};

use crate::bootstrap::{arguments::BootstrapArguments, docstring::BootstrapDocstring};

use self::{
    arguments::{BootType, DerivedTypes},
    signature::{BootSigArgType, BootstrapSignature},
};

#[cfg(test)]
mod test;

impl Function {
    pub fn from_ast(
        attr_args: AttributeArgs,
        item_fn: ItemFn,
        module: Option<&str>,
    ) -> Result<Function> {
        // Parse the proc bootstrap macro args
        let arguments = BootstrapArguments::from_attribute_args(&attr_args)?;

        // Parse the signature
        let signature = BootstrapSignature::from_syn(item_fn.sig.clone())?;

        // Parse the docstring
        let path = if arguments.name.is_none() {
            module.map(|module| {
                let name = arguments.name.as_ref().unwrap_or(&signature.name).as_str();
                (module, name)
            })
        } else {
            None
        };

        let name = arguments.name.clone().unwrap_or(signature.name.clone());
        let docstring = BootstrapDocstring::from_attrs(
            &name,
            item_fn.attrs,
            &item_fn.sig.output,
            path,
            arguments.features.0.clone(),
        )?;

        // aggregate info from all sources
        reconcile_function(arguments, docstring, signature)
    }
}

pub fn reconcile_function(
    mut bootstrap: BootstrapArguments,
    mut doc_comments: BootstrapDocstring,
    signature: BootstrapSignature,
) -> Result<Function> {
    Ok(Function {
        name: bootstrap.name.unwrap_or(signature.name),
        features: bootstrap.features.0,
        description: doc_comments.description,
        args: reconcile_arguments(
            &mut bootstrap.arguments.0,
            &mut doc_comments.arguments,
            signature.arguments,
            signature.supports_partial,
        )?
        .into_iter()
        .chain(reconcile_generics(
            &mut bootstrap.generics.0,
            &mut doc_comments.generics,
            signature.generics,
        )?)
        .collect(),
        ret: reconcile_return(
            bootstrap.returns,
            doc_comments.returns,
            signature.output_c_type,
        )?,
        derived_types: reconcile_derived_types(bootstrap.derived_types),
        dependencies: bootstrap.dependencies.0,
        supports_partial: signature.supports_partial,
        has_ffi: bootstrap.has_ffi.unwrap_or(true),
        deprecation: doc_comments.deprecated,
    })
}

fn reconcile_arguments(
    bootstrap_args: &mut HashMap<String, BootType>,
    doc_comments: &mut HashMap<String, String>,
    arguments: Vec<(String, BootSigArgType)>,
    supports_partial: bool,
) -> Result<Vec<Argument>> {
    (arguments.into_iter().enumerate())
        .map(|(i, (name, arg_type))| {
            // struct of additional metadata for this argument supplied by bootstrap macro
            let boot_type = bootstrap_args.remove(&name).unwrap_or_default();

            Ok(Argument {
                name: Some(name.clone()),
                // if C type is given in boot_type, use it. Otherwise use the rust type on the function
                c_type: Some(match boot_type.c_type {
                    Some(v) => v,
                    None => {
                        // if supports_partial, then the first two c types are AnyDomain and AnyMetric
                        if supports_partial && i < 2 {
                            if i == 0 { "AnyDomain *" } else { "AnyMetric *" }.to_string()
                        } else {
                            arg_type.c_type?
                        }
                    }
                }),
                // if Rust type is given, use it. Otherwise parse the rust type on the function
                rust_type: Some(match boot_type.rust_type {
                    Some(v) => v,
                    None => {
                        // if supports_partial, then the first two rust types are un-set
                        if supports_partial && i < 2 {
                            TypeRecipe::None
                        } else {
                            arg_type.rust_type?
                        }
                    }
                }),
                generics: Vec::new(),
                description: doc_comments.remove(&name),
                hint: boot_type.hint,
                default: boot_type.default,
                is_type: false,
                do_not_convert: boot_type.do_not_convert,
                example: None,
            })
        })
        .collect()
}

fn reconcile_generics(
    bootstrap_args: &mut HashMap<String, BootType>,
    doc_comments: &mut HashMap<String, String>,
    generics: Vec<String>,
) -> Result<Vec<Argument>> {
    (generics.into_iter())
        .filter_map(|name| {
            let boot_type = bootstrap_args.remove(&name).unwrap_or_default();
            if boot_type.suppress {
                return None;
            }
            if boot_type.c_type.is_some() {
                return Some(Err(Error::custom(
                    "c_type should not be specified on generics",
                )));
            }
            Some(Ok(Argument {
                name: Some(name.clone()),
                description: doc_comments.remove(&name),
                hint: boot_type.hint,
                default: boot_type.default,
                generics: boot_type.generics.0,
                is_type: true,
                example: boot_type.example,
                ..Default::default()
            }))
        })
        .collect()
}

fn reconcile_return(
    boot_type: Option<BootType>,
    doc_comment: Option<String>,
    output_c_type: Result<String>,
) -> Result<Argument> {
    let boot_type = boot_type.unwrap_or_default();

    Ok(Argument {
        c_type: Some(match boot_type.c_type {
            Some(v) => v,
            None => output_c_type?,
        }),
        rust_type: boot_type.rust_type,
        description: doc_comment,
        do_not_convert: boot_type.do_not_convert,
        ..Default::default()
    })
}

fn reconcile_derived_types(derived_types: Option<DerivedTypes>) -> Vec<Argument> {
    derived_types
        .map(|dt| dt.0)
        .unwrap_or_default()
        .into_iter()
        .map(|(name, rt)| Argument {
            name: Some(name),
            rust_type: Some(rt),
            is_type: true,
            ..Default::default()
        })
        .collect()
}
