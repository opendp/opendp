use darling::{Error, FromMeta, Result};
use syn::{AttributeArgs, Item, Type, TypePath};

use crate::bootstrap::arguments::Features;

use self::filesystem::load_proof_paths;

pub mod filesystem;

#[derive(FromMeta)]
pub struct Proven {
    pub proof_path: Option<String>,
    #[darling(default)]
    pub features: Features,
}

impl Proven {
    // assumes that proof paths have already been written in the lib's build script
    pub fn from_ast(attr_args: AttributeArgs, item: Item) -> Result<Self> {
        let mut proven = Proven::from_list(&attr_args)?;

        if proven.proof_path.is_some() {
            return Ok(proven);
        }

        // parse function
        let name = match item {
            Item::Fn(func) => func.sig.ident.to_string(),
            Item::Impl(imp) => {
                let path = match &imp.trait_ {
                    Some(v) => &v.1,
                    None => match &*imp.self_ty {
                        Type::Path(TypePath { path, .. }) => path,
                        ty => return Err(Error::custom("failed to parse type").with_span(&ty)),
                    },
                };
                (path.segments.last())
                    .ok_or_else(|| {
                        Error::custom("path must have at least one segment").with_span(&imp.self_ty)
                    })?
                    .ident
                    .to_string()
            }

            input => {
                return Err(Error::custom("only functions or impls can be proven").with_span(&input))
            }
        };

        let help = "You can specify a path instead: `#[proven(proof_path = \"{{module}}/path/to/proof.tex\")]`";

        // we have a hashmap of options. So retrieving a the value at a key gives you an Option<Option<String>>:
        //    None              no path was found
        //    Some(None) means  more than one path was found
        //    Some(Some(path))  one unique path was found

        proven.proof_path = Some(
            load_proof_paths()?
                .remove(&name)
                .ok_or_else(|| Error::custom(format!("failed to find {name}.tex. {help}")))?
                .ok_or_else(|| {
                    Error::custom(format!("more than one file named {name}.tex. {help}"))
                })?,
        );

        Ok(proven)
    }
}
