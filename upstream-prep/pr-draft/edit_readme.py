from pathlib import Path
src = Path("README.orig.md").read_text()
src = src.split("\n", 1)[1] if src.startswith("successfully downloaded") else src
t = src

def rep(old, new, count=1):
    global t
    assert t.count(old) == count, (old[:80], t.count(old))
    t = t.replace(old, new)

rep("A lot of people showed up to help. 46 of you have commits in here.",
    "A lot of people showed up to help. 49 of you have commits in here.")
rep("""I'm looking forward to where this project goes, and I wish everyone a lot of luck.
""", """I'm looking forward to where this project goes, and I wish everyone a lot of luck.

To try it: [a build of this branch](SNAPSHOT_LINK), with its mod SDK.
""")
rep('<img alt="46 people and 10,345 commits, by GitHub account"', '<img alt="49 people and 10,848 commits, by GitHub account"')

# Special thanks: three new people, Xmilon's launcher redesign.
rep("""- Recipe viewers got a server-checked transfer""", """- @Joeypc6 gave mod screens clip rectangles, living entities and whole scenes to draw, backed menu slots and recipe transfer into mod menus, integer game rules, a narrator call, and a way to pause a mob's AI while its physics runs on.
- Recipe viewers got a server-checked transfer""")
rep("""- @oloion let content-mod entity types""", """- @Nixotica added `screen_room`, room beside a container screen's window for a mod's own panel, moving the window with it.
- @oloion let content-mod entity types""")
rep("""- @Xmilon brought dimensions, biome modifiers, biome regions and datapack registries to mods, and rebuilt the Mods screen after NeoForge's mod list.""",
    """- @Xmilon brought dimensions, biome modifiers, biome regions and datapack registries to mods, rebuilt the Mods screen after NeoForge's mod list, and redesigned the launcher around mods: its Mods page and native mod builds run on this API.""")
rep("""and client reads of the GUI scale and the hotbar slot.
""", """and client reads of the GUI scale and the hotbar slot.
- @Zenilea made the `PayloadCodec` derive handle recursive types and type macros, and kept a clipped face's colour ramp in mod screens.
""")

# Since the first draft (14 November).
rep("""## What it adds
""", """## Since the first draft

Since 14 November this branch took upstream's main once more (the sync at horizon-rust/horizon 7785026e88), and
nine more rounds of merges came in (arismoko/horizon rollups 109 to 117). Among them:
- @Xmilon's launcher redesign (arismoko/horizon#4450), built around mods: the Mods page and native mod builds run on
  this API, and it presents as Horizon Launcher;
- @Joeypc6's screen drawing, menu slots and AI pause, @Mnntn's entity Part state and attachment sync, @Detimes'
  world and player writes, @Zaccou's command completion and mob paths, @OlaHulleberg's data maps and mob handle
  reads, @hinzee's and @Kerlann's viewed levels, and @levzzz5154's ingredient payloads and registry reads;
- the move of every test that skips without vanilla data to the daily run, and the daily run's own fixes.

## What it adds
""")

rep("""- The mod SDK: the surface manifest, the API crate, the prebuilt game package and the tools, built and signed for
  each nightly and released as `sdk-<sha>`.""", """- The mod SDK: the surface manifest, the API crate, the prebuilt game package and the tools, built by
  `nightly-build.yml` for the Nightly's commit and released as `sdk-<sha>`, signed with the same keys.""")
rep("""- The launcher installs native mods, builds them with consent, reuses its SDK cache, refuses mods that don't fit the
  build, and offers developer builds. Its names, icons, identifiers and Microsoft application are upstream's.""",
    """- The launcher is @Xmilon's redesign. It installs native mods, builds them with consent, reuses its SDK cache,
  refuses mods that don't fit the build, and offers developer builds. Its names, icons, identifiers and Microsoft
  application are upstream's.""")
rep("""- Every pull request and main push runs `checks`: `verify` (format, lint, unit tests, plus the test-tier and mod API
  surface checks) and `launcher`.
- `daily.yml` runs, once a day on main's head: every test that boots the game, needs prepared resources, starts a
  child process or uses the network (`#[ignore = "daily"]`, one row each in `tools/ci/ignored-tests/`), the wasm
  fixture, `fn-replace-check`, the SDK build and its linkage probe. When every leg is green it moves the tag
  `daily-green`. When a test fails, it files one issue for that test.""", """- `ci.yml` is yours as it is, plus one step: the wasm32 check of `horizon-mod-api` when the mod API changes.
- `mod-checks.yml` runs the mod API's own checks (type boundary, vocabulary, test tiers, export-list and stand-in
  guards, the surface report) when a pull request touches crates or those tools, and on demand.
- Tests that boot the game, need prepared resources, start a child process, use the network or take 2 s or more
  are `#[ignore = "daily"]`, one row each in `tools/ci/ignored-tests/`. `daily.yml` runs them once a day on GitHub's
  runners with a time limit, outside the merge gate, with `fn-replace-check` and the SDK linkage probe; the rows
  that need prepared vanilla data wait for a runner that holds it. A failing test files one issue.""")
rep('<img alt="Tests and CI: a quick tier on every push, a daily run on main, and how snapshot, nightly and release builds are cut"',
    '<img alt="Tests and CI: the checks on every push, the mod checks, a daily run on main, and the Nightly that carries the SDK"')
rep("""- The modding book (`docs/modding/`, built by `modding-book.yml` and published to Pages by `modding-book-pages.yml`),
  `docs/mod-api/`, the design records under `docs/design/mod-api/`, and `AGENTS.md`'s mod API rules.""",
    """- The modding book, Markdown in `docs/modding/` (`modding-book.yml` checks that it builds), `docs/mod-api/`, the
  design records under `docs/design/mod-api/`, and `AGENTS.md`'s mod API rules.""")

# Existing mods.
rep("""## Upstream workflows this changes, and why""", """## Mods published before this

The registry holds 137 native mods (`.hnmod`) and 12 content mods (`.hmod`) built for Horizon's current mod system.
None of them loads after this merge: the mod API replaces that system, with no compatibility path. Each author
rebuilds against the new SDK and publishes one `.hmod` per mod. The modding book's
[native mods](docs/modding/src/how-to/native-mods.md) and [content mods](docs/modding/src/how-to/content-mods.md)
pages and `cargo horizon-mod new` cover the port.

## Upstream workflows this changes, and why""")
a = t.index("## Upstream workflows this changes, and why")
b = t.index("## Notes for maintainers")
t = t[:a] + """## Upstream workflows this changes, and why

| Workflow | Change |
| --- | --- |
| `ci.yml` | One step in `verify`: the wasm32 check of `horizon-mod-api` when the mod API or a crate it builds changes |
| `nightly-build.yml` | Builds the mod SDK of the Nightly's commit, releases it as `sdk-<sha>` with the same keys before the Nightly, and signs its index (`sdk.json`) in the Nightly's sums |
| `game-build.yml` | The pinned Rust (`tools/ci/rust-version`), the shared wasm standard library as an artifact, the glibc baseline and smoke checks run from the workflow's own copy of the tools, and a refusal to run a trusted build without a dedicated `MAC_RUNNER` |
| `launcher.yml` | A manual `base` input, test isolation from the runner's real folders, and a check that the release binary keeps its capability marker |
| `cargo-horizon-mod.yml`, `parity.yml`, `perf-wasm.yml`, `telemetry-server.yml` | The pinned Rust, the toolchain the launcher builds native mods with; `perf-wasm` runs on main and by hand, not on pull requests |
| `mod-registry.yml` | The package crate's new path (`horizon-mods` is now `horizon-mod-package`) |
| `native-hook-stamp.yml` | Removed: it numbered the JSON hook fragments, which are now trait-method stubs, so there is nothing to stamp |

`release.yml`, `nightly.yml`, `parity-main.yml` and the other workflows are unchanged.

New workflows:
- `mod-checks.yml`: the mod API's own checks.
- `daily.yml` and its legs: `ignored-tests`, `wasm-mods`, `fn-replace-check`, `fn-replace-windows`,
  `native-dylib-probe` and `launcher-windows-check`.
- The SDK's builds: `prebuilt-game-build`, `sdk-publish`, `prebuilt-game` and `prebuilt-mixins-tool`.
- `modding-book`, which checks that the book builds.
- Run by hand only: `wasm-coverage`, `hot-paths` (a performance measurement that never gates) and `paper-parity`
  (informational).

""" + t[b:]
i = t.index("## Notes for maintainers")
t = t[:i] + """## Notes for maintainers

Thanks for the answers in arismoko/horizon#4498; this is how the branch follows them.

1. **Nightly and SDK.** `nightly-build.yml` builds the mod SDK for the same commit as the Nightly, and its publish
   job releases it as `sdk-<sha>`, signed with `RELEASE_SIGNING_KEY` and `RELEASE_SIGNING_KEY_NEXT`, before the
   Nightly. The Nightly carries the SDK's index in its signed sums, because a native mod builds against the SDK of
   the exact game build. A Nightly builds main's commit alone, without picked pull requests, so the Nightly and its
   SDK are one commit of main; picks can come back if you want them. There is no snapshot flow and no `daily-green` tag.
2. **`ci.yml`** is yours as it is, plus the wasm32 check of `horizon-mod-api` in `verify` when the mod API changes.
   The mod API's own checks are in `mod-checks.yml`, which runs when a pull request touches crates or those tools,
   and on demand. Being path-filtered, it can't be a required check; if you want it required, it can run on every
   pull request and decide inside, as `verify` does.
3. **Daily tests** run in `daily.yml` on GitHub's runners with a time limit, outside the merge gate. GitHub's
   runners have no vanilla data, so the rows that need it or boot the game are left out there
   (`ignored_tests.py --without-data`) until a runner holds the data. The workflows that need it
   (the wasm fixture, `hot-paths`, `paper-parity`, `wasm-coverage`) run only when started by hand.
4. **On-demand builds.** `cargo-horizon-mod`, `modding-book` and `launcher-windows-check` run when a pull request
   touches their files; a maintainer can start any of them by hand.
5. **The book** is Markdown in `docs/modding/`, linked by its path in this repository. No Pages deploy.
6. **Release addresses.** The launcher downloads the SDK, the prebuilt game and the horizon-mixins tool from this
   repository's releases. Native mod builds work from the first Nightly after the merge.
7. **Old issue references** are written as `arismoko/horizon#N`, so they never point at an issue here with the
   same number. The example native mod stays in arismoko/horizon-machine-sample, and the book links it.
"""
Path("README.md").write_text(t)
print("written", len(t))
