//! Shared recipe resource merge, conditions and filters.
use crate::Result;
use horizon_world::loot::PackOverlay;
use serde_json::Value;
use std::{collections::HashMap, path::Path};

/// The overlay id of the `recipe_priorities.json` layers of the packs, lowest
/// first, and of the list that names, one UTF-8 source per layer in the same
/// order, what supplied each.
pub const PRIORITY_LAYERS: &str = "recipe_priorities";
pub const PRIORITY_SOURCES: &str = "recipe_priorities.source";

/// One `recipe_priorities.json` layer: `replace` clears the priorities of the
/// layers below, `entries` gives recipe ids their signed priority.
#[derive(serde::Deserialize)]
pub struct PriorityLayer {
    #[serde(default)]
    pub replace: bool,
    #[serde(default)]
    pub entries: HashMap<String, i32>,
}
impl PriorityLayer {
    /// The layer `file` holds, which `source` (a pack or a path) supplied: a
    /// JSON object whose `replace` is a boolean and whose `entries` map recipe
    /// ids to 32-bit integers. The error names `source`.
    pub fn decode(source: &str, file: Value) -> std::result::Result<Self, String> {
        // A positional array would also fill the struct's fields.
        if !file.is_object() {
            return Err(format!("{source}: the root must be an object"));
        }
        serde_json::from_value(file).map_err(|e| format!("{source}: {e}"))
    }
}

/// A recipe's whole id from its name in the tables: vanilla's are kept bare
/// (`oak_planks`), everyone else's whole (`mymod:crusher/iron`). The book,
/// the network copy, the advancements' JSON and brewing know a recipe by
/// this id.
pub fn qualified(id: &str) -> String {
    if id.contains(':') {
        id.to_owned()
    } else {
        format!("minecraft:{id}")
    }
}

/// The recipe files the tables are built from, each id once: vanilla's by
/// bare name, then the content mods' as `<ns>:<name>`, the world's packs'
/// over them (`overlay`), parsed, and only those whose load conditions
/// hold. A server sends some of them to its clients (`recipe_content`).
pub fn sources(root: &Path, overlay: &PackOverlay) -> Result<Vec<(String, Value)>> {
    // Vanilla's recipes by bare name, then content mods' as `<ns>:<name>`.
    let mut sources: Vec<(String, Vec<u8>)> = Vec::new();
    // Load conditions: `mod_loaded` asks which mods' data is
    // in this root (`minecraft`, always, and the content mods').
    // `mod_loaded` asks which mods are loaded, not which namespaces
    // they own: a mod that also owns a namespace doesn't make that
    // namespace another mod.
    let inventory = horizon_mod_package::enabled::EnabledMods::read(root)?;
    let conditions = overlay
        .conditions
        .with_mod_inventory(inventory.server_ids())
        .with_diagnostics();
    for file in &horizon_world::json_folder::read(&root.join("data/minecraft/recipe"))?.files {
        sources.push((file.name.clone(), file.bytes()?.to_vec()));
    }
    for namespace in crate::content_namespaces::namespaces(root) {
        let dir = root.join(format!("data/{namespace}/recipe"));
        if dir.is_dir() {
            for file in &horizon_world::json_folder::read(&dir)?.files {
                sources.push((format!("{namespace}:{}", file.name), file.bytes()?.to_vec()));
            }
        }
    }
    // The world's packs' recipes over the game's own: the top pack's
    // file for each id replaces it (`SimpleJsonResourceReloadListener`
    // sees one resource per id, the resource stack's top).
    for (id, files) in overlay.kind("recipe") {
        let Some((ns, path)) = id.split_once(':') else {
            continue;
        };
        let name = if ns == "minecraft" {
            path.to_owned()
        } else {
            id.to_owned()
        };
        let bytes = PackOverlay::top(files).to_vec();
        match sources.iter_mut().find(|(n, _)| *n == name) {
            Some(slot) => slot.1 = bytes,
            None => sources.push((name, bytes)),
        }
    }
    let filters = crate::recipe_filters::RecipeFilters::load(root, overlay);
    let mut parsed = Vec::with_capacity(sources.len());
    for (name, bytes) in sources {
        // Vanilla's RecipeManager logs a file that does not parse and
        // goes on ("Couldn't parse data file").
        let mut v: Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Horizon: recipe {name} not loaded: {e}");
                continue;
            }
        };
        // A file whose load conditions don't hold is silently absent.
        if !conditions.allows_id(&v, "recipe", &qualified(&name)) {
            continue;
        }
        // The data's recipe filters, over the final set.
        if !filters.apply(&qualified(&name), &mut v) {
            continue;
        }
        parsed.push((name, v));
    }
    conditions.check_resource_errors("recipe")?;
    Ok(parsed)
}
