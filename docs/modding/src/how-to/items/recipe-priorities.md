# Recipe priorities

Server data packs and content roots can supply `data/<namespace>/recipe_priorities.json`:

```json
{"replace": false, "entries": {"example:specific": 5, "example:broad": -1}}
```

Files apply lowest pack first (namespaces within a pack in name order). Passive
content mods keep dependency order, with each manifest's jars in declared order
below its own directory or `.hmod` files. Singleton layers survive path overrides,
including identical paths and replacements from another namespace; other passive
resources retain their usual last-file-wins behavior. Entries override earlier
values; `replace: true` clears all earlier priorities, even when `entries` is
absent or empty. A file's `neoforge:conditions` is evaluated before
merge or replace, so an omitted layer changes nothing. Priorities are signed
32-bit integers; unspecified recipes default to zero. Unknown recipe IDs are
retained quietly, including recipes absent because their conditions failed.
A layer that is not valid (its root is not an object, `replace` is not a boolean,
a priority is not a 32-bit integer) is refused whole, and the log names the pack
or path it came from.

The shared recipe sources are sorted once per server data load by descending
priority, then vanilla's existing registration key (path, then namespace).
Crafting, every cooking type, smithing, the stonecutter list and mod-type recipe
maps inherit that order without lookup-time sorting. With no file, registration
order is unchanged. The loader is `crafting/priorities.rs`; behavior fixtures are
in its `priorities/tests.rs`, including literal overlapping shapeless recipes.

Format references: [NeoForge recipe-priorities documentation](https://docs.neoforged.net/docs/resources/server/recipes/#recipe-priorities)
and the public [26.3 `RecipePriorityManager` Javadoc](https://aldak0.ru/javadoc/26.3.x/net/neoforged/neoforge/common/crafting/RecipePriorityManager.html).
