//! Pack-layered `data/<namespace>/recipe_priorities.json` (NeoForge's public
//! recipe-priorities format). Order the shared sources before any table is built.
use super::{ResolvedSources, registration_key};
use crate::Result;
use horizon_items::recipe_sources::PriorityLayer;
use horizon_world::{
    conditions::LoadConditions, loot::PackOverlay, native_load_context::LoadContext,
};
use serde_json::Value;
use std::{cmp::Reverse, collections::HashMap, fs, path::Path};

#[derive(Default)]
struct Priorities(HashMap<String, i32>);

impl Priorities {
    /// Applies the layer `bytes` of `source`, or refuses it whole: its error
    /// names `source`, and nothing of it is kept.
    fn layer(
        &mut self,
        source: &str,
        bytes: &[u8],
        conditions: &LoadConditions,
    ) -> std::result::Result<(), String> {
        let file: Value = serde_json::from_slice(bytes).map_err(|e| format!("{source}: {e}"))?;
        if !conditions.allows_file(&file, "recipe_priorities.json") {
            return Ok(());
        }
        let layer = PriorityLayer::decode(source, file)?;
        if layer.replace {
            self.0.clear();
        }
        // Unknown ids are intentionally retained, including conditioned-out recipes.
        self.0.extend(
            layer
                .entries
                .into_iter()
                .map(|(id, priority)| (horizon_items::recipe_sources::qualified(&id), priority)),
        );
        Ok(())
    }

    fn load(root: &Path, overlay: &PackOverlay) -> Result<Self> {
        let inventory = horizon_mod_package::enabled::EnabledMods::read(root)?;
        let conditions = overlay
            .conditions
            .with_mod_inventory(inventory.server_ids());
        let mut priorities = Self::default();
        let mut namespaces: Vec<_> = match fs::read_dir(root.join("data")) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e.into()),
        };
        namespaces.sort();
        for namespace in namespaces {
            let path = namespace.join("recipe_priorities.json");
            match fs::read(&path) {
                Ok(bytes) => {
                    let source = path.display().to_string();
                    if let Err(e) = priorities.layer(&source, &bytes, &conditions) {
                        log::warn!("recipe priorities layer refused, {e}");
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => log::warn!("{} not loaded: {e}", path.display()),
            }
        }
        for (_, layers) in overlay
            .files
            .iter()
            .filter(|(key, _)| key == "recipe_priorities")
        {
            for bytes in layers {
                if let Err(e) =
                    priorities.layer("a data pack's recipe_priorities.json", bytes, &conditions)
                {
                    log::warn!("recipe priorities layer refused, {e}");
                }
            }
        }
        Ok(priorities)
    }

    fn get(&self, name: &str) -> i32 {
        if name.contains(':') {
            self.0.get(name).copied().unwrap_or(0)
        } else {
            self.0
                .get(&format!("minecraft:{name}"))
                .copied()
                .unwrap_or(0)
        }
    }
}

impl ResolvedSources {
    /// Resolve supplied recipes in vanilla's default registration order.
    pub fn resolve(mut sources: Vec<(String, Value)>, context: &LoadContext) -> Result<Self> {
        sources.sort_by_cached_key(|(name, _)| registration_key(name));
        Self::resolve_sorted(sources, context)
    }

    /// Resolve one owning data load, then order all its projections once.
    pub fn load(root: &Path, overlay: &PackOverlay, context: &LoadContext) -> Result<Self> {
        let context = context.with_fluid_tags(horizon_items::items::fluid_tag_list(root, overlay)?);
        Self::resolve_prioritized(
            root,
            overlay,
            horizon_items::recipe_sources::sources(root, overlay)?,
            &context,
        )
    }

    pub(super) fn resolve_prioritized(
        root: &Path,
        overlay: &PackOverlay,
        mut sources: Vec<(String, Value)>,
        context: &LoadContext,
    ) -> Result<Self> {
        let priorities = Priorities::load(root, overlay)?;
        sources.sort_by_cached_key(|(name, _)| {
            (Reverse(priorities.get(name)), registration_key(name))
        });
        Self::resolve_sorted(sources, context)
    }
}

#[cfg(test)]
mod tests;
