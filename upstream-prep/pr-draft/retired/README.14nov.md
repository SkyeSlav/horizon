successfully downloaded text file (SHA: 10d14cae4243e8de21f33343f6a11bf474e1af0f)
> Draft of the pull request description. Edit this file directly or open a pull request against it; the final text is copied into the real PR.

# The mod API

This brings the mod API from arismoko/horizon upstream.

I started this fork because I want Horizon to have massive mods that everyone can play. Horizon could already run mods: content mods, and native mods that patch the game's source and rebuild it. What it didn't have was an API that something the size of a real Java mod could be built on. This adds one: a mod API with its own types, named after NeoForge wherever NeoForge has the same thing, shared by native and WebAssembly mods, and built against a prebuilt game so nobody has to compile Horizon to play a mod. My hope is that it grows into something like NeoForge and can stand next to the Java modding experience.

A lot of people showed up to help. 46 of you have commits in here. You ported actual mods to find out what was missing, and most of what's good in the API came out of that. Thank you.

This is by no means close to done. It's the very start of a mod API. Things are broken right now, and things will keep breaking as it changes, so a mod written against it today should expect to be updated.

I'm asking for a standard merge commit so everyone's commits stay theirs. If you want to change something before it goes in, push to this branch or carry the PR with your changes on top.

This will show conflicts when it opens. They go away once upstream freezes and I sync this branch one more time, or they get resolved in a carry PR on top.

I'm looking forward to where this project goes, and I wish everyone a lot of luck.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/01-overview-dark.png">
  <img alt="What this branch adds: one mod API, native and WebAssembly mods, the SDK, the launcher's Mods page and the modding book" src="images/01-overview-light.png" width="800">
</picture>

The tree is kept in step with horizon-rust/horizon: upstream's main has been merged in regularly, and the Horizon layer (telemetry, cosmetics, shop, sponsors, Discord, the official executable, the launcher's names and its Microsoft sign-in) is upstream's. It is far too large to review line by line, so the sections below say what is in it, what it changes in existing workflows, and what I need the maintainers to decide.

## Who built it

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/07-contributors-dark.png">
  <img alt="46 people and 10,345 commits, by GitHub account" src="images/07-contributors-light.png" width="800">
</picture>

### Special thanks

- @a2z453 taught the launcher to import Prism and MultiMC ZIP packs, bounded the ZIP reader before it parses an archive, and fixed lock contention in WebAssembly mod groups.
- World views began with @AfrobamaYT's work, the world drawn again from a mod's own camera into its own target, alongside a way to hide single parts of a vanilla mob model.
- @arslanarm fixed batched mod model copies under a shader pack, so they are shaded as block entities like the other model route.
- Because of @bct8925, enchantments, their tags and their names load from every data namespace, so a content mod's own enchantments reach the game.
- @BPMichon gave spawn checks the level, the spawner and the pending mob, let `finalize_spawn` lend the unspawned mob so mod goals survive it, and added mod skull models.
- @bvironn moved the mod registry to SQLite 3.54, closing a security advisory against the older copy it bundled.
- A client mod can hear chat as it arrives and send chat or a command to the server through @Cheezer1656's two hooks.
- @CocofireHD put each hook's notes on its own declaration and generated the book's hook page from them, and gave `level_can_survive` a form that crosses into WebAssembly.
- @d3xc0 added a run of `BlockPart` answers (explosion resistance and drops, which entities may destroy a block, connections, rotation and state changes) and gave mods `player_disconnect` with a reason.
- Mods store items by kind in 64-bit amounts and poll a block capability in constant time through `STORAGE` and `BlockCapabilityCache`, both from @Dawnforger, who also added tameable content-mod mobs.
- @Detimes worked through what a community server's mods need: a player's inventory, ender chest and saved data, raw plugin channels, datafixing older saved NBT, and a vetoable `player_move`.
- Client mods can register their own block tint sources, coloured by vanilla's rules for each block, thanks to @ErikKalikas.
- @foudretbr made action keybinds follow the player's current binding, stopped a server-side item use when the item in hand changes, and let title splashes come from every resource namespace.
- From @Giantcookie392 came client reads such as latency, the local player's rotation and screen projection, and a fix that keeps a mob on a valid player target across ticks.
- @hinzee built viewed levels and passages, from the server keeping a player's areas in other levels to the client keeping a world for each one, and gave world views clip planes, mirroring and culling.
- @Hiuoy laid the first stages of typed mod payloads (capacities, a bounded value codec and its derive) and made the game ask before opening a world whose saved mods are missing.
- @iseaprojects contributed the fix that makes a native build stop when a mixin helper breaks the format, instead of carrying on.
- Recipe viewers got a server-checked transfer that moves nothing and a JEI-style item panel from @J5Casey, who also kept synced vanilla recipes in the recipe book.
- @JuanLuci gave mod screens buttons and text fields, gave client mods their own sounds through `SoundRef`, and added camera and tooltip hooks.
- @KBroneZ let client mods add geometry to chunk meshes and bring their own texture atlases, fixed depth copies on OpenGL, and kept LAN guest turns intact when a listener panics.
- Attribute bounds now come from the attribute catalog and mods can widen them, both by @Kerlann, who also added runtime levels a mod can open and unload.
- @levzzz5154 wrote dozens of the NeoForge-named hooks and handle methods the API grew from, such as `server_chat`, `living_damage_pre`, `MerchantRef` and `BossBarRef`, and later decoded item component payloads for mod ingredients.
- Native mods can register their own dimensions and chunk generators because of @LoneStarrr, who also made dimension arguments suggest every server level.
- @Mayonesu0 made the Rust tests and the self-check build and run on Windows, and added a long run of Item and Block Part answers such as `can_equip`, bounce restitution and `damage_item`.
- @merenotnoonereal went through the hook catalog against Fabric and Paper batch by batch, recording each counterpart and gap, then built the Paper parity harness and the engine behind menu decisions.
- @Mnntn brought registered entity renderers and their layers, fluid container helpers for pickup and placement, and block entity Part state on the server.
- Behind many parts of the API are @OlaHulleberg's design notes, among them Part instance state, decide-fire-apply for client hooks and typed payloads, together with the hooks and fixes that followed, from fake-player actions to a cache of compiled WebAssembly modules.
- @oloion let content-mod entity types declare their passenger attachment points.
- @Qwerner88880 built custom mob goals and brain behaviours for mods, the general entity type registry from saved rows to ticking, and the single move gate behind `entity_move`.
- @R4INN fixed mods' language files not loading when a windowed start hands off from the title menu.
- Early on, @reservedbytes added block state ids and item tags to the host, the hand and hit on block-use hooks, item entities and experience orbs on level handles, and tamed and owner reads.
- @RLC248 fixed a saddled mod mob being driven as a mount: as in vanilla, only a horse or camel now takes a player rider's input.
- Mod authors can unit-test against a composable `FakeHost`, set listener priority and receive cancelled events, all thanks to @sirrayi.
- @Solenopsisbot reworked screens so each layout is built once on open, resize and close, documented NeoForge extension-method coverage in the modding book, and added custom command argument types.
- @Sylviromi gave inventory and recipe mods the reads they rely on: the item under the pointer, burn times and compost chances, drawing a fluid, and a screen's exclusion areas.
- @TheApple05 added world preset editors for mods and trimmed the CI shard that tests the client library against prepared data.
- @TheLion102009 made a mod's world text keep its styles and settled how spinning parts and block sprites are drawn.
- With @TiggerTakkyTTT's fixes, entity section crossings are reported at the write that causes them, and client mods have typed combat targeting controls.
- A mod that simulates a fluid itself can refuse vanilla's scheduled fluid tick through @Ufuk-a's cancellable hook.
- @umbx-dot gave mods typed settings that the game draws for them, with a `setting_changed` notice, and let `player_submit` hide single parts of the player model.
- @valkdzx split `player_tick` into pre and post, let mods move the in-game chat and clear client particles, and added a player heart type hook.
- Client mods can register their own numeric and categorical item model properties, the two pieces @Vaspyyy added.
- @vnsmoreira opened model and animation registration to mods: sprite sources, model loaders, item model types, and keyframe animations on the game's own sampling path.
- @Xmilon brought dimensions, biome modifiers, biome regions and datapack registries to mods, and rebuilt the Mods screen after NeoForge's mod list.
- @Zaccou added entity writes for portal cooldown and no-physics, exact-total experience orb merging, and client reads of the GUI scale and the hotbar slot.

## What it adds

**Mod API and vocabulary**
- `horizon-mod-api`: what a mod sees. Hooks, a small shared `Host` with `ServerHost` and `ClientHost` per side,
  handles, Parts, attachments and transactions. It exposes only its own types and builds for wasm32.
- `horizon-mod-vocab`: the engine-layer interface that the API re-exports (events, `HookId`, `Hooks`, the `Host`
  traits, value types). Hook declarations are attributed trait-method stubs that `horizon-mod-api-gen` reads with
  `syn`. Where NeoForge 26.3 has a hook, the hook takes its name and contract, with one hook per moment.
- `horizon-mod-host` and `horizon-mod-points`: loading, `Host` implementations, registries and dispatch above the
  engine. `horizon-mod-runtime` holds attachment storage, query indexes and registry slots.
- `horizon-mod-surface`: the surface manifest (what an API-only mod can name, with a contract hash per item), which
  CI checks on every change.
- The mod-facing host code in the game: `game/modding/` and `game/native_hooks/`.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/03-api-numbers-dark.png">
  <img alt="The mod API in numbers: 736 hooks, 284 Host methods, 1,678 handle methods, 1,583 public types" src="images/03-api-numbers-light.png" width="800">
</picture>

**Native route and function replacement**
- Native mods build on the player's machine against a pinned prebuilt SDK of the exact game build, with the pinned
  Rust toolchain. The game is never rebuilt for a mod.
- The `[native.*]` tables (`mixin`, `bridge`, `module`, `loop`) replace functions on the prebuilt game through the
  horizon-mixins tool, which ships prebuilt per pin and platform. `[[inject]]` never compiles game code.
- A native mod may use crates.io dependencies, pinned by its own lockfile. Git and path dependencies outside the mod
  are refused.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/02-native-build-dark.png">
  <img alt="How a native mod reaches a player: install, check, download the SDK parts, build and play" src="images/02-native-build-light.png" width="800">
</picture>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/04-native-route-dark.png">
  <img alt="The API route and the native route, and how a mixin runs without a game rebuild" src="images/04-native-route-light.png" width="800">
</picture>

**Wasm route**
- `horizon-mod-wasm`: wasm mods with the same API and dependency model as native mods, across the shared-memory
  `Wire` layout (`#[derive(Wire)]`). A capability that can't cross is refused at load with a reason.
- Libraries shared between mods, groups ordered by dependencies, compile budgets, and a guest-stack crate.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/05-wasm-route-dark.png">
  <img alt="The WebAssembly route: the same API over one shared encoding, with refusals named at load" src="images/05-wasm-route-light.png" width="800">
</picture>

**SDK and launcher mod support**
- The mod SDK: the surface manifest, the API crate, the prebuilt game package and the tools, built and signed for
  each nightly and released as `sdk-<sha>`.
- `cargo horizon-mod` (`new`, `build`, `run`, `check`, `registry`) and `.hmod` packages. `horizon-mods` is renamed to
  `horizon-mod-package`.
- The launcher installs native mods, builds them with consent, reuses its SDK cache, refuses mods that don't fit the
  build, and offers developer builds. Its names, icons, identifiers and Microsoft application are upstream's.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/08-screens-dark.png">
  <img alt="The launcher's Mods page and a native mod's machine screen in the game" src="images/08-screens-light.png" width="800">
</picture>

**Tests and CI tiers**
- Every pull request and main push runs `checks`: `verify` (format, lint, unit tests, plus the test-tier and mod API
  surface checks) and `launcher`.
- `daily.yml` runs, once a day on main's head: every test that boots the game, needs prepared resources, starts a
  child process or uses the network (`#[ignore = "daily"]`, one row each in `tools/ci/ignored-tests/`), the wasm
  fixture, `fn-replace-check`, the SDK build and its linkage probe. When every leg is green it moves the tag
  `daily-green`. When a test fails, it files one issue for that test.
- Parity harnesses are kept, and new ones cover the hooks.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/06-tests-ci-dark.png">
  <img alt="Tests and CI: a quick tier on every push, a daily run on main, and how snapshot, nightly and release builds are cut" src="images/06-tests-ci-light.png" width="800">
</picture>

**Docs and book**
- The modding book (`docs/modding/`, built by `modding-book.yml` and published to Pages by `modding-book-pages.yml`),
  `docs/mod-api/`, the design records under `docs/design/mod-api/`, and `AGENTS.md`'s mod API rules.

## Upstream workflows this changes, and why

| Workflow | Change |
| --- | --- |
| `ci.yml` | Tiered: `verify` holds unit tests and the mod API checks; slow and resource-backed tests move to `daily.yml` |
| `game-build.yml`, `nightly-build.yml`, `release.yml` | Each nightly carries its SDK index in its signed sums and is built from `daily-green`; `release.yml` can also build a snapshot of a green main commit |
| `launcher.yml` | A manual `base` input, test isolation from the runner's real folders, and a check that the release binary keeps its capability marker |
| `cargo-horizon-mod.yml`, `parity.yml`, `perf-wasm.yml` | Build with the pinned Rust in `tools/ci/rust-version`, the same toolchain the launcher builds mods with |
| `mod-registry.yml`, `launcher.yml` paths | The package crate's new name |
| `parity-main.yml`, `pr-labels.yml`, `nightly.yml`, `pack-bench.yml`, `run-parity.yml` | Small differences that upstream's next commits change again; a merge from upstream takes them |
| `native-hook-stamp.yml` | Removed: it numbered the JSON hook fragments, which are now the trait-method stubs, so it has nothing to stamp |

New workflows: `daily.yml` and its legs (`ignored-tests`, `wasm-mods`, `wasm-coverage`, `fn-replace-check`,
`fn-replace-windows`, `native-dylib-probe`, `launcher-windows-check`, `prebuilt-game-build`, `sdk-publish`), plus
`prebuilt-game`, `prebuilt-mixins-tool`, `modding-book`, `modding-book-pages`, `hot-paths` (a performance trend that
never gates) and `paper-parity` (informational).

## Notes for maintainers

1. **Snapshot and nightly flow.** A nightly here is built only from `daily-green`, together with the SDK of that
   same commit. `release.yml` can also build a snapshot of a commit whose `checks` passed. I recommend keeping this
   flow: a native mod builds against the SDK of the exact game build, so the two have to come from one commit.
2. **Daily legs on `horizon-game`.** `ignored-tests`, the wasm fixture job and `hot-paths` ask for
   `[self-hosted, linux, horizon-game]` and prepare the vanilla data with `tools/ci/vanilla-data.sh`, as `parity.yml`
   does. The ignored-test shards are long, up to a few hours in total. I recommend scheduling `daily.yml` for when
   that runner is otherwise idle.
3. **Secrets and variables.** No new ones are needed: the SDK and nightlies sign with `RELEASE_SIGNING_KEY` and
   `RELEASE_SIGNING_KEY_NEXT`. The workflow token needs to be allowed to write:
   - the `daily-green` tag;
   - `sdk-<sha>` releases;
   - the `perf-history` branch;
   - GitHub Pages (for the book at horizon-rust.github.io/horizon).
4. **Release addresses.** The launcher downloads the SDK, the prebuilt game and the horizon-mixins tool from this
   repository's releases, so native mod builds work once the first `sdk-<sha>` is published. I recommend dispatching
   one snapshot right after the merge, so the first SDK exists before anyone tries to build a mod.
5. **Builds skipped for pull requests from forks.** `cargo-horizon-mod`, `modding-book` and
   `launcher-windows-check` skip compiling for them, and `perf-wasm` has no pull-request trigger. I recommend
   keeping that and having a maintainer dispatch them on a pull request that touches those areas.
6. **Lint extras not in `ci.yml`.** These checks exist in the tree but `verify` does not run them:
   - the wasm32 check of `horizon-mod-api`;
   - a host and Windows `cargo check` of the SDK;
   - the warnings wrapper (`check_warnings.py`);
   - the network-guarded test runner;
   - `check_file_size.py`, `check_repository_hygiene.py` and `format_scan.py` (`pr_ready.py` still runs the last
     two).

   I recommend adding the wasm32 check of `horizon-mod-api` to `verify` first, since it is what keeps the API
   buildable for WebAssembly mods, and taking the others as you see fit.
7. **Old issue references.** Links to one issue or pull request of arismoko/horizon (`arismoko/horizon#N`) are kept,
   because the same number here is a different issue. Every other repository address points here.
