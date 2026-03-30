use anyhow::{Result, bail};

use super::ast::{TraitDef, TraitImpl, Type};
use crate::types::{AstString as String, Set as HashSet};

pub(crate) type TraitRegistry = HashSet<String>;
pub(crate) type TraitImplRegistry = HashSet<(String, Type)>;

pub(crate) fn register_trait(def: &TraitDef, traits: &mut TraitRegistry) -> Result<()> {
    if !traits.insert(def.name.clone()) {
        bail!("duplicate trait: {}", def.name);
    }
    Ok(())
}

pub(crate) fn register_trait_impl(
    item: &TraitImpl,
    traits: &TraitRegistry,
    impls: &mut TraitImplRegistry,
) -> Result<()> {
    if !is_builtin_trait(&item.trait_name) && !traits.contains(&item.trait_name) {
        bail!("unknown trait: {}", item.trait_name);
    }
    if !impls.insert((item.trait_name.clone(), item.ty.clone())) {
        bail!("duplicate impl: {} for {:?}", item.trait_name, item.ty);
    }
    Ok(())
}

pub(crate) fn ensure_trait(ty: &Type, trait_name: &str, impls: &TraitImplRegistry) -> Result<()> {
    if supports_trait(ty, trait_name, impls) {
        return Ok(());
    }
    bail!("`{ty:?}` does not implement `{trait_name}`");
}

pub(crate) fn supports_trait(ty: &Type, trait_name: &str, impls: &TraitImplRegistry) -> bool {
    builtin_trait_impl(trait_name, ty) || impls.contains(&(trait_name.into(), ty.clone()))
}

fn is_builtin_trait(name: &str) -> bool {
    matches!(name, "Eq" | "Ord" | "Add")
}

fn builtin_trait_impl(name: &str, ty: &Type) -> bool {
    match name {
        "Eq" => matches!(ty, Type::String | Type::Int | Type::Bool | Type::Unit),
        "Ord" => matches!(ty, Type::String | Type::Int | Type::Bool | Type::Unit),
        "Add" => matches!(ty, Type::String | Type::Int),
        _ => false,
    }
}
