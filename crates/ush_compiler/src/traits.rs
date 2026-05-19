use anyhow::{Result, anyhow, bail};

use super::ast::{FunctionDef, TraitDef, TraitImpl, Type};
use crate::types::{AstString as String, Map as HashMap, Set as HashSet};

pub(crate) type TraitRegistry = HashSet<String>;

#[derive(Debug, Clone, Default)]
pub(crate) struct TraitImplRegistry {
    markers: HashSet<(String, Type)>,
    methods: HashMap<(Type, String), FunctionDef>,
}

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
    if let Some(trait_name) = &item.trait_name {
        if !is_builtin_trait(trait_name) && !traits.contains(trait_name) {
            bail!("unknown trait: {trait_name}");
        }
        if !impls.markers.insert((trait_name.clone(), item.ty.clone())) {
            bail!("duplicate impl: {trait_name} for {:?}", item.ty);
        }
    }
    for method in &item.methods {
        if method.receiver.as_ref() != Some(&item.ty) {
            bail!("method `{}` must use `self` for {:?}", method.name, item.ty);
        }
        let key = (item.ty.clone(), method.name.clone());
        if impls.methods.insert(key.clone(), method.clone()).is_some() {
            bail!("duplicate method `{}` for {:?}", key.1, key.0);
        }
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
    builtin_trait_impl(trait_name, ty) || impls.markers.contains(&(trait_name.into(), ty.clone()))
}

pub(crate) fn lookup_method<'a>(
    ty: &Type,
    method: &str,
    impls: &'a TraitImplRegistry,
) -> Result<&'a FunctionDef> {
    impls
        .methods
        .get(&(ty.clone(), method.into()))
        .ok_or_else(|| anyhow!("unknown method `{method}` for `{ty:?}`"))
}

fn is_builtin_trait(name: &str) -> bool {
    matches!(name, "Eq" | "Ord" | "Add" | "Display")
}

fn builtin_trait_impl(name: &str, ty: &Type) -> bool {
    match name {
        "Eq" => matches!(ty, Type::String | Type::Int | Type::Bool | Type::Unit),
        "Ord" => matches!(ty, Type::String | Type::Int | Type::Bool | Type::Unit),
        "Add" => matches!(ty, Type::String | Type::Int),
        "Display" => matches!(ty, Type::String | Type::Int | Type::Bool | Type::Unit),
        _ => false,
    }
}
