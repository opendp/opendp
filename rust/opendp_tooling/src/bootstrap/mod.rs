use std::collections::HashMap;
use syn::{AttributeArgs, ItemFn};

use crate::{Argument, Function};

pub mod arguments;
pub mod docstring;
pub mod signature;

use darling::{Error, Result};

use crate::bootstrap::{arguments::BootstrapArguments, docstring::BootstrapDocstring};

use self::{
    arguments::BootType,
    signature::{BootstrapSignature, BootSigArgType},
};

impl Function {
    pub fn from_ast(
        attr_args: AttributeArgs,
        item_fn: ItemFn,
        proof_paths: &HashMap<String, Option<String>>,
    ) -> Result<Function> {
        // Parse the proc bootstrap macro args
        let arguments = BootstrapArguments::from_attribute_args(&attr_args)?;

        // Parse the docstring
        let docstring = BootstrapDocstring::from_attrs(item_fn.attrs, &item_fn.sig.output)?;

        let signature = BootstrapSignature::from_syn(item_fn.sig)?;

        // aggregate info from all sources
        reconcile_function(arguments, docstring, signature, proof_paths)
    }
}

pub fn reconcile_function(
    mut bootstrap: BootstrapArguments,
    mut doc_comments: BootstrapDocstring,
    signature: BootstrapSignature,
    proof_paths: &HashMap<String, Option<String>>,
) -> Result<Function> {
    let name = bootstrap.name.unwrap_or(signature.name);
    Ok(Function {
        name: name.clone(),
        features: bootstrap.features.0,
        description: doc_comments.description,
        proof_path: reconcile_proof_path(bootstrap.proof_path, &name, proof_paths)?,
        args: reconcile_arguments(
            &mut bootstrap.arguments.0,
            &mut doc_comments.arguments,
            signature.arguments,
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
        derived_types: bootstrap
            .derived_types
            .map(|dt| dt.0)
            .unwrap_or_default()
            .into_iter()
            .map(|(name, rt)| Argument {
                name: Some(name),
                rust_type: Some(rt),
                ..Default::default()
            })
            .collect(),
    })
}

fn reconcile_proof_path(
    bootstrap: Option<String>,
    name: &str,
    proof_paths: &HashMap<String, Option<String>>,
) -> Result<Option<String>> {
    Ok(match bootstrap {
        Some(proof_path) => Some(proof_path),
        None => match proof_paths.get(name) {
            Some(None) => return Err(Error::custom(format!("more than one file named {name}.tex. Please specify `proof_path = \"{{module}}/path/to/proof.tex\"` in the macro attributes."))),
            Some(proof_path) => proof_path.clone(),
            None => None
        }
    })
}

fn reconcile_arguments(
    bootstrap_args: &mut HashMap<String, BootType>,
    doc_comments: &mut HashMap<String, String>,
    arguments: Vec<(String, BootSigArgType)>,
) -> Result<Vec<Argument>> {
    (arguments.into_iter())
        .map(|(name, arg_type)| {
            // struct of additional metadata for this argument supplied by bootstrap macro
            let boot_type = bootstrap_args.remove(&name).unwrap_or_default();

            Ok(Argument {
                name: Some(name.clone()),
                // if C type is given in boot_type, use it. Otherwise use the rust type on the function
                c_type: Some(match boot_type.c_type {
                    Some(v) => v,
                    None => arg_type.c_type?,
                }),
                // if Rust type is given, use it. Otherwise parse the rust type on the function
                rust_type: Some(match boot_type.rust_type {
                    Some(v) => v,
                    None => arg_type.rust_type?,
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
        .map(|name| {
            let boot_type = bootstrap_args.remove(&name).unwrap_or_default();
            Ok(Argument {
                name: Some(name.clone()),
                description: doc_comments.remove(&name),
                hint: boot_type.hint,
                default: boot_type.default,
                generics: boot_type.generics.0,
                is_type: true,
                example: boot_type.example,
                ..Default::default()
            })
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
