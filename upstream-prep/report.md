# upstream-prep: report

Branch `upstream-prep` on arismoko/horizon, head `b8b91dcd1af`, = main `4dc6455d4a6` + 23 commits (no trailers).
Built by `prep.sh` from the inputs in this folder; the same folder is the orphan branch `upstream-prep-tools`.
Fork main is untouched. Upstream's files come from the upstream commit main contains (`7785026e88`, prep.sh's
UPSTREAM_REF). The pull request description's source is the private draft repository; `pr-body.md` is a copy.

| Commit | Files | + / − |
| --- | --- | --- |
| CI: run the kept workflows without the fork's runner pool and gate | 38 | +254 / −584 |
| Docs and comments: drop the fork's own process and names | 18 | +29 / −38 |
| Remove the fork's CI gate, runner pool and boat plumbing | 27 | −4189 |
| Remove the fork's board, roadmap and Discord automation | 8 | −602 |
| Remove the fork's upstream-sync tools and records | 36 | −2967 |
| Remove the conflict-measurement scripts | 9 | −1307 |
| Remove the shm-scene bench scratch crate | 3 | −189 |
| README: upstream's text, without the fork notice | 1 | −11 |
| Launcher: build only upstream's identity (scripted) | 19 | +85 / −412 |
| Launcher: drop the fork's names, icons, keys and packaging | 44 | +144 / −1135 |
| CI: upstream's telemetry-server and mod-registry workflows | 2 | +32 / −35 |
| Claims: upstream's claim command and limit | 4 | +73 / −484 |
| CI: upstream's runner watch, nightly audit, parity-main, pack-bench and run-parity | 5 | +55 / −12 |
| CI: stop building the external machine sample | 16 | +21 / −214 |
| Docs and tools: drop the remaining local paths and agent-runner names | 8 | +25 / −22 |
| CI: upstream's ci.yml, plus the mod API's wasm32 check | 1 | +249 / −393 |
| Release: the SDK built and published by the Nightly; no snapshot or daily-green flow | 25 | +416 / −1527 |
| Daily tests on GitHub's runners, outside the gate | 15 | +116 / −153 |
| Point every repository reference at horizon-rust/horizon (scripted) | 50 | +120 / −118 |
| Repository references: corrections after the rename | 7 | +10 / −11 |
| Modding book: read in the repository, not published to Pages | 11 | +35 / −77 |
| Write the fork's issue references as arismoko/horizon#N (scripted) | 524 | +1265 / −1263 |
| Issue references: corrections after the rewrite | 1 | +1 / −1 |
| Total | 759 | +2799 / −15613 |

Item 4 (Microsoft application id) needed no commit: the launcher already reads `HORIZON_MSA_CLIENT_ID` exactly as
upstream's does (`vars.HORIZON_MSA_CLIENT_ID` in launcher.yml, `account::client_id()`); the only fork handling was the
README and launcher README text, gone with items 2 and 3.

## Proofs (`proofs.sh`, log in `proofs.log`)

1. Nothing kept names a path the branch deleted (every `D` against main, by path and file name): 0 hits.
2. `actionlint` on every workflow: clean. 2b. `tools/check_addresses.py` (ci.yml verify): passed.
3. `python3 -m unittest discover -s tests` + `tools/ci` prepared-data tests, Python 3.12 (CI's): OK, 1661 + 2 tests.
4. Rust/Cargo files changed, by commit: naming 3, shm-scene 2, launcher identity script 18 and hand pass 12,
   repository rename 10 (strings and doc comments only).
5. Scrub: portizon, `/box/`, Dyna (word and identifiers), host names and the agent-swarm line are 0 (`scrub.md`).
6. `arismoko` left: 1316 lines (mostly `arismoko/horizon#N` references), all allowed kinds; unexpected: 0.
7. `cargo check --workspace --all-targets --locked` and the launcher workspace's `cargo check --all-targets`: OK.
   Compile only; no clippy or tests were run on the Rust changes.

## Resolved toward upstream (the former ASK list)

1. **Repository references** (`rename_repository.py`, then a hand pass): `arismoko/horizon` → `horizon-rust/horizon`
   in the launcher's `REPO` and README address, the SDK, mixins-tool and prebuilt release URLs, `PREBUILT_REPO`, the
   book links and its pages address (`horizon-rust.github.io/horizon`), workflow conditions (nightly-build's SDK
   steps), `nightly.py` and `release_train.py`. Left as they are, by rule:
   - `arismoko/horizon-machine-sample` (no upstream twin; see pr-body question 7);
   - references to one issue or pull request (`arismoko/horizon#N`, `.../issues/N`, `.../pull/N`): about 170 lines,
     mostly design records and doc comments. The same number on horizon-rust/horizon is a different issue, so a
     rewrite would link to the wrong one;
   - lines that upstream's own files have word for word (GOVERNANCE.md, CONTRIBUTING.md's "Mod-related pull requests
     go to arismoko/horizon", docs/layers.md, docs/mod-hotload.md): upstream's text about the fork, for the
     maintainers to reword at the merge.
   `arismoko/horizon-mixins` occurs nowhere (the tool comes from this repository's releases; horizon-rust/horizon-mixins
   exists). Repository switches that selected the fork's *build flow* (the SDK-paired nightly from `daily-green`,
   `release_train`'s gate jobs, nightly-build's SDK steps) now select it for horizon-rust/horizon, since the CI tiers
   on this branch need it; pr-body question 1 asks the maintainers. Tests that compared the two repositories now
   compare horizon-rust/horizon with another one.
2. **README:** upstream's README; the one kept change is `cargo run --release -p horizon-game` (the game's executable
   is built by horizon-game here).
3. **Launcher identity:** `strip_fork_launcher.py` evaluates `cfg!(horizon_fork_launcher)` as off and writes upstream's
   literals where `launcher/src/identity.rs` stood; the hand pass takes upstream's `build.rs`, `desktop.rs`,
   `installer.rs` (keeping `horizon_addresses::LAUNCHER_ABOUT` for the address check), `setup_args.rs`,
   `setup_page.rs`, `titlebar.rs`, design-system and security docs, macOS packaging README; launcher.yml back to
   upstream's names, matrix, signing and ARM64 steps (kept: the `base` input, test isolation, the capability-marker
   check, `verify --repository`); deletes the fork icons, release notes, staging and matrix tools, the nightly banner,
   `fork-release-keys.txt` (release trust is now `release-keys.txt` for horizon-rust/horizon, every other repository
   none) and their tests; `links.json` loses the fork board link. Native-mod and SDK features untouched.
4. **Microsoft application id:** nothing to change (above).
5. **telemetry-server.yml, mod-registry.yml:** upstream's files, keeping two product changes: the package crate's new
   path, and the pinned Rust (`tools/ci/rustup_install.sh`; `tests/test_rust_pin.py` requires it in every workflow).
6. **Claim limit:** upstream's claim workflow, `claim_command.py`, its test and CONTRIBUTING text (8 open issues).
7. **Workflows:** `runner-watch.yml` restored from upstream (its script already matched). `native-hook-stamp.yml`
   stays deleted: it numbered `tools/native-hook-fragments/*.json`, which no longer exist (hooks are trait-method stubs
   since 570922b0dc9).
8. **Dyna and `/box/`:** the persistent prepared-data cache is `HORIZON_PERSISTENT_CACHE`, apt retries
   `APT_RETRY_TRIES`/`APT_RETRY_PAUSE`; the read-only vanilla copy guarded by `tools/datafix_paths.py` and the
   row-4 mutants script is `HORIZON_SHARED_SOURCE` (vanilla-data.sh's name, same default) instead of a fixed local
   path; `tools/refactors/api-pr0/move.py` writes into `.local/`; the two refactor scripts for the removed gate
   workflow are deleted.
9. **Machine sample:** `machine-sample.yml`, the daily leg, `check_machine_sample.py` (the pin) and its test,
   `runner-packages.txt` (only that workflow read it), daily_summary's action-issue mechanism (only the sample used
   it), `rust_changes.py`'s unused `sample=` output, and the AGENTS, CONTRIBUTING and release-flow lines. The book keeps
   "An example mod lives at <url>" in its introduction and the recipes page; the modding-book review line no longer
   says the reviewer runs the sample. Design records under docs/design are left as written.
- **Also:** the changelog digest (`changelog.yml`, `changelog-test.yml`, `tools/changelog.py`, test, fixture) is
  removed: it summarised the fork's PRs around its rollups for a branch only the fork's README linked.

Left as they are: 14 lines naming rollups (the hotspots carry tool and README, the sync file-size baseline docs,
the `rollup-*.txt` ignored-test row files' names); design records under docs/design.

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
  workflows, modding book, the surface gate, fn-replace-check, wasm workflows, the tiers, and `tools/refactors/`
  except the two scripts for the removed gate workflow (step 8 below).

### Where the fork gate's product checks went

`dyna-ci.yml` held product checks besides its routing. Ported as plain steps into `ci.yml`'s `verify` (upstream's
`checks`): the test-tier checks (`check_ignored_test_rows`, `check_game_boot_tests`, `check_process_tests`,
`reexec_include_ignored --check`), the hotspot guards (`check_lists`, `check_stand_ins`, `check_wire_loopback`), the
prepared-data tool tests, the mod-host map tables (main pushes), and the surface gate (build `horizon-stage`, then
`surface_report.py` on pull requests and `test_stage_catalog.py`). Not ported: dyna-ci's own lint extras (the wasm32
check of `horizon-mod-api`, the SDK host/Windows `cargo check`, `check_warnings.py` wrapping), the network-guarded test
runner, `check_file_size.py`, `check_repository_hygiene.py`, `format_scan.py` (pr_ready still runs the last two).
Upstream's reviewers may want some of these in ci.yml.

## Rerunning after a sync (or any main change)

```sh
cd /box/shared/upstream-prep            # or check out branch upstream-prep-tools
bash prep.sh                            # rebuilds upstream-prep from origin/main in ../horizon-upstream-prep, runs proofs.sh
PUSH=1 bash prep.sh                     # same, then force-pushes upstream-prep
```

- `prep.sh` fetches origin and upstream, resets the work tree's branch to the base (keeping its `target/`), applies
  `patches/0001…0003` with `git am --3way` (a patch already on the base is skipped), runs `remove.py` (closure + one
  commit per category), `align.py` (the steps above: scripts `strip_fork_launcher.py` and `rename_repository.py`,
  upstream's files from `upstream/main`, and the hand diffs in `patches/align/*.diff`), then `proofs.sh`.
  `CARGO_CHECK=1` adds the compile-only cargo checks.
- An align diff that no longer applies stops `align.py` naming it: fix it in the work tree with
  `python3 align.py --until <step before>`, edit, then `git add -A && git diff --cached --binary > patches/align/<step>.diff`
  and `python3 align.py --from <step>` (align.py's docstring).
- A patch that conflicts stops `git am`: resolve in `../horizon-upstream-prep`, `git am --continue`, run
  `python3 /box/shared/upstream-prep/remove.py && BASE=origin/main bash /box/shared/upstream-prep/proofs.sh`, then refresh
  the patches: `git format-patch -o /box/shared/upstream-prep/patches origin/main..HEAD~5` (the three patch commits).
- `remove.py` stops before deleting anything when the closure finds a removable file no category claims (a sync brought a
  new fork-operations file): add a rule in `CATEGORIES`, or keep it by fixing what refers to it.
- New references the sync adds to the gate or the pool (a workflow naming `dyna_ci_runner.sh`, a doc naming
  `dyna-checks`) surface in proof 1 and keep that file's target in the closure; edit patch 0001/0002 for them.
- Requirements: `actionlint` in `~/.local/bin`, Python 3.12 via `uv python install 3.12` (proofs use it when present).
