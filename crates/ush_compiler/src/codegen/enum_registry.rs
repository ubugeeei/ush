use anyhow::{Result, bail};

use crate::types::Set as HashSet;

use super::super::{
    ast::{EnumDef, VariantFields},
    env::EnumRegistry,
};

pub(crate) fn register_enum(def: &EnumDef, enums: &mut EnumRegistry) -> Result<()> {
    if enums.contains_key(&def.name) {
        bail!("duplicate enum: {}", def.name);
    }
    let mut variants = HashSet::with_hasher(Default::default());
    for variant in &def.variants {
        if !variants.insert(variant.name.clone()) {
            bail!("duplicate variant: {}::{}", def.name, variant.name);
        }
        if let VariantFields::Struct(fields) = &variant.fields {
            let mut names = HashSet::with_hasher(Default::default());
            for field in fields {
                if !names.insert(field.name.clone()) {
                    bail!(
                        "duplicate field: {}::{}::{}",
                        def.name,
                        variant.name,
                        field.name
                    );
                }
            }
        }
    }
    enums.insert(def.name.clone(), def.clone());
    Ok(())
}
