# upstream-prep: report

Branch `upstream-prep` on arismoko/horizon, head `dc39907bf95`, = main `77b62ef6b60` + 8 commits (one per category, no
trailers). Built by `prep.sh` from the inputs in this folder, also committed on branch `upstream-prep-tools` (an
orphan branch holding only this folder). Fork main is untouched.

| Commit | Files | + / − |
| --- | --- | --- |
| CI: run the kept workflows without the fork's runner pool and gate | 38 | +163 / −584 |
| Docs and comments: drop the fork's own process and names | 19 | +30 / −39 |
| Reword comments and a test mod id (also PR #4432 to main; skipped by the script once main has it) | 3 | +10 / −9 |
| Remove the fork's CI gate, runner pool and boat plumbing | 26 | −4158 |
| Remove the fork's board, roadmap and Discord automation | 8 | −602 |
| Remove the fork's upstream-sync tools and records | 34 | −2849 |
| Remove the conflict-measurement scripts | 9 | −1307 |
| Remove the shm-scene bench scratch crate | 3 | −189 |
| Total | 137 | +203 / −9737 |

## Proofs (static, `proofs.sh`, log in `proofs.log`)

1. Nothing kept names a removed path: 0 hits. Accepted and listed: `tools/refactors/centralize_ci_selection.py` and
   `split_verify_static.py` name `dyna-ci.yml` (one-off refactor scripts; `tools/refactors/` is kept whole by the brief).
2. `actionlint` 1.7.7 on every remaining workflow: clean. (Main itself fails it: 13 findings, all in the removed
   workflows plus `horizon-parity-main`, upstream's own Mac label, which is now listed in `.github/actionlint.yaml`.
   Upstream's own main fails it too, 4 findings.)
3. `python3 -m unittest discover -s tests` and `discover -s tools/ci -p 'test_prepared_*.py'` (what ci.yml verify
   runs) under Python 3.12 (CI's): OK, 1747 tests. Under this box's Python 3.11 one test fails on main too
   (`test_prebuilt_game`, `tarfile.extractall(filter=)` is 3.12).
4. Rust/Cargo files changed: only the three files of the naming commit and the deleted `tools/shm-scene/` (both
   approved by root). The naming commit passed `cargo check --tests` on its three crates and CI on PR #4432.

## Removed, with reasons

All read first and confirmed as the fork's own. The `tools/ci`, `tools/hotspots` and `tools/sync` files were removed only
when outside the reference closure (`closure.py`: a file stays when any kept workflow, test, tool, doc, Cargo or Rust file
names it, transitively; tests of removed tools share their fate).

- **CI gate, runner pool, boat (26):** `dyna-ci.yml` (the fork's gate: lint/verify/verify-static/launcher with Dyna/boat
  routing), `dyna-automerge-guard.yml` (disarms auto-merge for the fork account), `boat-recovery.yml`, `boat-smoke.yml`
  (boat runners), `ci-cache.yml` (warms caches for dyna-ci); `tools/ci/` boat_controller, boat_health, boat_watch,
  dyna_ci_deps.sh, dyna_ci_runner.sh, dyna_ci_pool_watch.py, dyna_parity_python.sh, dyna-slot-caps.txt (host names),
  pr_gate.py (rollup/sync heavy-check rule), gate_summary.py, check_runner_packages.py (compares with the Dyna image),
  the five `daily-move-2026-11*.tsv` one-off migration records; their tests (test_dyna_ci, test_ci_boat,
  test_gate_failures, test_ci_history_fetch, test_pr_gate).
- **Board, roadmap, Discord (8):** `board.yml` + `.github/scripts/board-sync.js` (arismoko project board),
  `roadmap-progress.yml` + `roadmap-progress.js` (issue #771 bars), `merge-updates.yml` + `tools/ci/merge_updates.py`
  (rollup/sync posts to our Discord); tests.
- **Sync tools and records (34):** `tools/sync/*` except the two kept below, `tools/sync_upstream.py`,
  `tools/convert_sync_fragment.py`, `docs/sync-27…38-decisions.md`, `docs/sync-fragments-r2.md`; tests. Lost with it:
  dyna-ci's "duplicate public native functions" step (`sync_upstream.py --check`).
- **Conflict measurement (9):** `tools/hotspots/measure.py`, `open_prs.py`, `game_rs.py`/`.txt` (rollup and sync
  churn measures), `check_guards.py` (a guard keyed on our rollup/sync branch names); tests.
- **shm-scene (3):** a bench crate with absolute `/box/` paths, not a workspace member.

## Kept, with reasons

- **Kept by reference despite the folder:** `tools/sync/README.md` and `tools/sync/write_file_size_baseline.py`
  (`tools/check_file_size.py` and `docs/layers.md` name them; the README lost its sync-audit section);
  `tools/hotspots/check_lists.py`, `check_stand_ins.py`, `check_wire_loopback.py` (product guards, now in ci.yml verify)
  and `carry.py` (used by `tools/refactors/split_dispatch_stand_ins.py`); `docs/sync/` (named by Rust comments and docs).
- **tools/ci:** 70 files/folders stay in the closure (`last-kept.json` lists each with its referrers): the daily tier
  (ignored_tests, ignored_shards, ignored-tests/, check_* tier checks, move_to_daily, daily_summary, run_test_binaries,
  network_guard), the SDK and release chain (nightly, nightly_set, sign_*, legacy_releases, rust-version,
  rustup_install), fn-replace-check fixtures, surface_report + surface.toml, upstream's bots (claim, triage, overlap,
  labels, mojang), runner scripts upstream uses (runner-env, vanilla-data, linux-runner, checks-runners, runner_watch,
  runner-gc, git-slim), and generic helpers (cargo_jobs, source_mtimes, compiler-cache, apt_retry, with_xvfb).
- **Brief's KEEP list:** daily.yml and every `#[ignore = "daily"]` row, sdk-publish, prebuilt game and mixins-tool
  workflows, modding book, the surface gate, fn-replace-check, wasm workflows, all of `tools/refactors/`, the tiers.
- **Claim limit:** kept (root agreed): upstream already refused past 8 open claims; ours is 5 with exemptions. On the ASK list.
- **Changelog:** `changelog.yml`/`tools/changelog.py` kept (not in the brief's list); disclosure line neutralised. ASK.

### Where the fork gate's product checks went

`dyna-ci.yml` held product checks besides its routing. Ported as plain steps into `ci.yml`'s `verify` (upstream's
`checks`): the test-tier checks (`check_ignored_test_rows`, `check_game_boot_tests`, `check_process_tests`,
`reexec_include_ignored --check`), the hotspot guards (`check_lists`, `check_stand_ins`, `check_wire_loopback`), the
prepared-data tool tests, the mod-host map tables (main pushes), and the surface gate (build `horizon-stage`, then
`surface_report.py` on pull requests and `test_stage_catalog.py`). Not ported: dyna-ci's own lint extras (the wasm32
check of `horizon-mod-api`, the SDK host/Windows `cargo check`, `check_warnings.py` wrapping), the network-guarded test
runner, `check_file_size.py`, `check_repository_hygiene.py`, `format_scan.py` (pr_ready still runs the last two).
Upstream's reviewers may want some of these in ci.yml.

## Workflows changed on this branch (for upstream's reviewers)

Upstream workflows the fork had already modified (16 kept, vs the merge base `d1b6cd286ea`): cargo-horizon-mod, ci,
claim-command, game-build, launcher, mod-registry, nightly-build, nightly, pack-bench, parity-main, parity, perf-wasm,
pr-labels, release, run-parity, telemetry-server. Kept as ours; this branch changes only:

| Workflow | What this branch changed |
| --- | --- |
| ci.yml | verify gains the product checks listed above (ported from the fork's gate) |
| game-build.yml | dropped the `DYNA_CI_SELF_HOSTED`/`NIGHTLY_*_RUNNER` runner branches and `DYNA_CI_BUILD` data paths; removed the `linux-smoke` job (ran only on our pool); macOS smoke uploads the report again |
| nightly-build.yml | bookkeeping job's runner: dropped the `NIGHTLY_BOOKKEEPING_RUNNER` branch |
| release.yml | comments: snapshot is a maintainer's dispatch, gated on `checks`' verify and launcher |
| cargo-horizon-mod.yml | comment only (no `pr_gate.py`) |
| others | unchanged |

Fork-only workflows kept and changed: ignored-tests, wasm-mods, hot-paths (data jobs now `[self-hosted, linux,
horizon-game]`, upstream's label for machines holding the maintainer's Minecraft files, with `runner-env.sh` +
`vanilla-data.sh` as parity.yml does); wasm-coverage, fn-replace-check, native-dylib-probe, machine-sample (route job
removed, GitHub's runners); daily.yml (the schedule's self-dispatch no longer limited to arismoko/horizon);
launcher-windows-check (pr_gate job replaced by the same fork-PR skip cargo-horizon-mod and modding-book use);
modding-book, sdk-publish (comments). `.github/actionlint.yaml` lists `horizon-parity-main`.
Tools changed to match: `release_train.py` reads ci.yml's `verify`+`launcher` for snapshots instead of dyna-ci's jobs;
`rust_changes.py` drops dyna-ci from its trigger set; 13 tests drop or adjust assertions on dyna-ci and the pool.

## Scrub (before = main, after = branch; third_party excluded; `scrub.md` lists every remaining file)

| Term | Before (lines / files) | After |
| --- | --- | --- |
| portizon | 11 / 5 | 0 / 0 |
| /box/ paths | 18 / 9 | 4 / 4 |
| ses_ / room_ ids | 0 | 0 |
| hermeslab, ovh-rise5 | 2 / 2 | 0 / 0 |
| arch as a host | 0 | 0 |
| Dyna (word) | 74 / 31 | 5 / 3 |
| dyna identifiers (dyna-ci, DYNA_…) | 246 / 59 | 19 / 7 |
| agent-swarm disclosure | 6 / 5 | 1 / 1 |
| rollup | 164 / 36 | 28 / 13 |

Remaining, listed not fixed: `/box/` in `tools/datafix_paths.py`, `tools/test_block_entity_tick_items_mutants.py`
(code defaults to our vanilla copy), `tests/test_hot_paths.py` (a sample bad path), `tools/refactors/api-pr0/move.py`.
Dyna: `README.md` fork notice (ASK 1), `tools/ci/vanilla-data.sh` slot-cache mode (`HORIZON_DYNA_CACHE`, its tests,
`apt_retry.sh`'s `DYNA_CI_APT_*` variables: code, off unless set), `tools/refactors/` (2 scripts). Rollup: the changelog
tool and its fixture (ASK), `tools/hotspots/carry.py` + README "In a rollup" section, `check_stand_ins.py` docstring,
`tools/sync` baseline docs and `check_file_size.py`/`docs/layers.md` (sync-PR file-size caps), the
`tools/ci/ignored-tests/rollup-*.txt` row files (names only), 2 refactor scripts. Process text fixed: AGENTS.md CI tiers,
CONTRIBUTING (tiers table, the "we carry it in a rollup" bullet, fork parity note), docs/release-flow.md (rollup carry,
"root" dispatch, dyna-checks), PR template's rollup note, docs/ci-runners.md's "Cheap checks" section, sdk-publish's
design path, the daily run's and changelog's disclosure lines.

## ASK list (decisions for the owner and upstream; this branch changes none) — 13 items

1. **README fork notice** (first block): "This is arismoko/horizon, a fork…", Modrizon Discord, fork launcher
   downloads, "maintained by Aria's agent swarm, Dyna". Replace, keep or drop for upstream.
2. **Hardcoded `arismoko/`**: 345 references in 147 files (`arismoko-refs.txt` has file:count). By kind:
   launcher `REPO` and `README_URL` (`launcher/src/backend.rs`), update/SDK/mixins-tool release URLs
   (`crates/horizon-addresses/src/lib.rs`: FORK_LAUNCHER_ABOUT, MIXINS_TOOL_RELEASES, SDK_RELEASES, the releases API),
   `PREBUILT_REPO` (`crates/horizon-mod-package/src/index/native.rs`), release keys by repo
   (`crates/horizon-release-sign/src/lib.rs`, `fork-release-keys.txt`), `arismoko/horizon-machine-sample` (7: daily.yml,
   machine-sample.yml, check_machine_sample.py, daily_summary.py, 3 book pages), book links (`docs/modding/`: 30 in 21
   files, `book.toml`), issue references `arismoko/horizon#N` in docs/design (132 in 47 files) and Rust doc comments
   (crates/horizon-mod-vocab 11, horizon-client 8), `github.repository == 'arismoko/horizon'` branches in launcher.yml
   (10), nightly-build.yml (5), game-build.yml timeout, `tools/ci/nightly.py` (10) and `nightly-arismoko.md`,
   `tools/release_train.py`, `tools/changelog.py`, tests (43 in 11). `arismoko/horizon-mixins` is not hardcoded
   (the mixins tool is fetched from this repository's releases).
3. **Launcher naming as the fork's:** `LAUNCHER_PREFIX horizon-fork-launcher`, "Horizon Fork Launcher (arismoko)",
   `tools/fork_launcher_identity.py`, `configure_fork_launcher_workflow.py`, `generate_fork_launcher_icons.py`,
   `packaging/fork-launcher-release-notes.md`, `launcher/src/identity.rs`.
4. **Microsoft application id:** the launcher reads `vars.HORIZON_MSA_CLIENT_ID` (upstream's variable); the fork builds
   with upstream's app id by agreement. Confirm upstream keeps that arrangement.
5. **telemetry-server.yml:** our change disabled its automatic runs ("pending root CI-capacity approval"); restore
   upstream's triggers or keep dispatch-only.
6. **mod-registry.yml:** our change dropped upstream's fork-PR guard on `horizon-checks` and renamed the crate path.
7. **Fork-PR skips:** cargo-horizon-mod, modding-book and launcher-windows-check skip compiling for pull requests
   from forks (our tier policy); upstream may want them on every PR.
8. **Snapshot channel and `daily-green`:** release.yml `kind=snapshot`, release_train `--channel`, daily.yml's
   tag and nightly publish are the fork's build flow; upstream decides whether to adopt it.
9. **Self-hosted data runner:** ignored-tests, wasm-mods' fixture job and hot-paths now ask for
   `[self-hosted, linux, horizon-game]` and the prepared data `vanilla-data.sh` makes there; upstream confirms that
   runner can carry the daily legs (ignored-test shards ran up to 180 min on our pool).
10. **Secrets and variables a kept workflow needs that upstream lacks:** `vars.CLAIM_LIMIT_EXEMPT` (optional,
    claim-command.yml); `secrets: inherit` into sdk-publish/release from daily.yml (uses upstream's existing
    `RELEASE_SIGNING_KEY`/`_NEXT`); `GH_TOKEN` writes for `daily-green`, `perf-history` (hot-paths) and the `changelog`
    branch (workflow token, `contents: write`). Everything else they read upstream already has (`SELF_HOSTED_CI`,
    `MAC_RUNNER`, `MACOS_SIGN_IDENTITY`, `RELEASES_PAUSED`, `HORIZON_MSA_CLIENT_ID`, `HORIZON_CURSEFORGE_API_KEY`).
11. **Claim limit:** upstream refused past 8 open claims; ours refuses past 5 claimed issues without an open PR,
    exempting OWNER/MEMBER/COLLABORATOR, trusted reviewers and `CLAIM_LIMIT_EXEMPT` (b852d8293bb, dc3216bc2ab).
12. **`native-hook-stamp.yml` (deleted, not restored):** removed by 570922b0dc9 (2026-11-04, "Declare hooks as
    attributed trait-method stubs the generator reads with syn"): the hook catalog left the JSON fragments this
    workflow numbered, so it had nothing to stamp.
13. **`runner-watch.yml` (absent, not restored):** added upstream in 9dc8b78a8f4 (2026-11-10); the fork's syncs since
    did not take it (no merge message says why; it watches upstream's self-hosted runners, which the fork did not use).
    `tools/ci/runner_watch.py` and its test are in the tree.

Also for the owner: the changelog workflow and digest (`changelog.yml`, `tools/changelog.py`: rollup-aware, publishes
to a `changelog` branch the README links) — keep for upstream or drop.

## Rerunning after a sync (or any main change)

```sh
cd /box/shared/upstream-prep            # or check out branch upstream-prep-tools
bash prep.sh                            # rebuilds upstream-prep from origin/main in ../horizon-upstream-prep, runs proofs.sh
PUSH=1 bash prep.sh                     # same, then force-pushes upstream-prep
```

- `prep.sh` recreates the branch from the base, applies `patches/0001…0003` with `git am --3way` (a patch already on the
  base, such as the naming commit after #4432 merges, is skipped), runs `remove.py` (closure + one commit per category),
  then `proofs.sh` (writes `proofs.log`, `scrub.md`, `last-removed.json`, `last-kept.json`).
- A patch that conflicts stops `git am`: resolve in `../horizon-upstream-prep`, `git am --continue`, run
  `python3 /box/shared/upstream-prep/remove.py && BASE=origin/main bash /box/shared/upstream-prep/proofs.sh`, then refresh
  the patches: `git format-patch -o /box/shared/upstream-prep/patches origin/main..HEAD~5` (the three patch commits).
- `remove.py` stops before deleting anything when the closure finds a removable file no category claims (a sync brought a
  new fork-operations file): add a rule in `CATEGORIES`, or keep it by fixing what refers to it.
- New references the sync adds to the gate or the pool (a workflow naming `dyna_ci_runner.sh`, a doc naming
  `dyna-checks`) surface in proof 1 and keep that file's target in the closure; edit patch 0001/0002 for them.
- Requirements: `actionlint` in `~/.local/bin`, Python 3.12 via `uv python install 3.12` (proofs use it when present).
