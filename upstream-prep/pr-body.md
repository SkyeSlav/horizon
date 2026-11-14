# The mod API

This pull request brings in the mod API built on arismoko/horizon. It merges with a standard merge commit, so every
contributor's commits stay as they are. The tree is kept in step with horizon-rust/horizon: upstream's main has been
merged in regularly, and the Horizon layer (telemetry, cosmetics, shop, sponsors, Discord, the official executable,
the launcher's names and its Microsoft sign-in) is upstream's.

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

**Native route and function replacement**
- Native mods build on the player's machine against a pinned prebuilt SDK of the exact game build, with the pinned
  Rust toolchain. The game is never rebuilt for a mod.
- The `[native.*]` tables (`mixin`, `bridge`, `module`, `loop`) replace functions on the prebuilt game through the
  horizon-mixins tool, which ships prebuilt per pin and platform. `[[inject]]` never compiles game code.
- A native mod may use crates.io dependencies, pinned by its own lockfile. Git and path dependencies outside the mod
  are refused.

**Wasm route**
- `horizon-mod-wasm`: wasm mods with the same API and dependency model as native mods, across the shared-memory
  `Wire` layout (`#[derive(Wire)]`). A capability that can't cross is refused at load with a reason.
- Libraries shared between mods, groups ordered by dependencies, compile budgets, and a guest-stack crate.

**SDK and launcher mod support**
- The mod SDK: the surface manifest, the API crate, the prebuilt game package and the tools, built and signed for
  each nightly and released as `sdk-<sha>`.
- `cargo horizon-mod` (`new`, `build`, `run`, `check`, `registry`) and `.hmod` packages. `horizon-mods` is renamed to
  `horizon-mod-package`.
- The launcher installs native mods, builds them with consent, reuses its SDK cache, refuses mods that don't fit the
  build, and offers developer builds. Its names, icons, identifiers and Microsoft application are upstream's.

**Tests and CI tiers**
- Every pull request and main push runs `checks`: `verify` (format, lint, unit tests, plus the test-tier and mod API
  surface checks) and `launcher`.
- `daily.yml` runs, once a day on main's head: every test that boots the game, needs prepared resources, starts a
  child process or uses the network (`#[ignore = "daily"]`, one row each in `tools/ci/ignored-tests/`), the wasm
  fixture, `fn-replace-check`, the SDK build and its linkage probe. When every leg is green it moves the tag
  `daily-green`. When a test fails, it files one issue for that test.
- Parity harnesses are kept, and new ones cover the hooks.

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

## Questions for maintainers

1. **Snapshot and nightly flow.** A nightly here is built only from `daily-green`, together with the SDK of that
   same commit. `release.yml` can also build a snapshot of a commit whose `checks` passed. Keep this flow, or move
   the SDK onto upstream's nightly selection?
2. **Daily legs on `horizon-game`.** `ignored-tests`, the wasm fixture job and `hot-paths` ask for
   `[self-hosted, linux, horizon-game]` and prepare the vanilla data with `tools/ci/vanilla-data.sh`, as `parity.yml`
   does. Can that runner carry them? The ignored-test shards are long, up to a few hours in total.
3. **Secrets and variables.** No new ones are needed: the SDK and nightlies sign with `RELEASE_SIGNING_KEY` and
   `RELEASE_SIGNING_KEY_NEXT`. What the workflow token must be allowed to write:
   - the `daily-green` tag;
   - `sdk-<sha>` releases;
   - the `perf-history` branch;
   - GitHub Pages (for the book at horizon-rust.github.io/horizon).
4. **Release addresses.** The launcher downloads the SDK, the prebuilt game and the horizon-mixins tool from this
   repository's releases. Native mod builds work once the first daily run has published `sdk-<sha>`.
5. **Builds skipped for pull requests from forks.** `cargo-horizon-mod`, `modding-book` and
   `launcher-windows-check` skip compiling for them, and `perf-wasm` has no pull-request trigger. A maintainer can
   dispatch them. Should they run on every pull request instead?
6. **Lint extras not in `ci.yml`.** These are not in `ci.yml` today:
   - the wasm32 check of `horizon-mod-api`;
   - a host and Windows `cargo check` of the SDK;
   - the warnings wrapper (`check_warnings.py`);
   - the network-guarded test runner;
   - `check_file_size.py`, `check_repository_hygiene.py` and `format_scan.py` (`pr_ready.py` still runs the last
     two).

   Which of these should `verify` run?
7. **The machine sample.** The example native mod (arismoko/horizon-machine-sample) lives in its own repository and
   CI no longer builds it. The book names it as an example only. Should it move into horizon-rust, or stay outside?
8. **Old issue references.** Links to one issue or pull request of arismoko/horizon (`arismoko/horizon#N`) are kept,
   because the same number here is a different issue. Every other repository address points here.
