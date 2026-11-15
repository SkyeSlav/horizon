# upstream-prep: plan for kacpherek's answers (arismoko/horizon#4498)

Status: plan only. Nothing is rebuilt or pushed. The figures below were measured on 2026-11-14 or 15 against these
commits:
- `upstream/main` = `8c383a5dcb5`
- fork `main` = `0dbf78e35f0` (sync 105 merged)
- `upstream-prep` = `dde1fa4f9a2`, built on main `77b62ef6b60`

"prep step" means a new step in `align.py`: one commit each, with a script commit followed by a hand-fix commit where
the change is mechanical.

## 0. One conflict with a standing rule (decision for root or the owner)

He asks for upstream's current main to be merged into `upstream-prep` before the PR opens. AGENTS.md says the
opposite: "No upstream sync runs before that pull request is open" and "the last sync happens after it is open".
Merging only into `upstream-prep` would make the branch differ from the fork's main.

The clean way to do it is one more ordinary sync PR on main. `prep.sh` would then rebuild on top of that main
unchanged. That needs the owner's word.

Sections 6 and 1 assume the sync happens. Without it, the PR carries every collision in section 6 for the
maintainers to resolve.

## 1. Keep ci.yml in upstream's current shape; our additions become separate workflows

**Today.**
- The branch's `ci.yml` is the fork's version: 740 lines, with jobs verify, launcher, parity-scope, parity,
  vanilla-parity and jfr-analysis.
- Upstream's is 627 lines. It has `merge_group`, verify split into verify-tree, verify-lint and verify-test, a
  "queued" skip, sccache and mold, and `test_scope.py`.

**Change.**
- A prep step `ci-upstream` writes `upstream/main:.github/workflows/ci.yml` verbatim, plus one step his point 6
  accepts: the wasm32 check of `horizon-mod-api` in verify-lint. It runs only when mod API paths change; a
  `git diff --name-only` filter sets it, about 15 lines.
- A new workflow, `mod-checks.yml`, takes everything else. Triggers:
  - `pull_request` with paths `crates/horizon-mod-*/**`, `crates/cargo-horizon-mod/**`, `game/modding/**`,
    `game/native_hooks/**`, `tools/ci/ignored-tests/**`, `tools/hotspots/**`, `tools/ci/surface*`, and the files
    each check reads;
  - `workflow_dispatch`.
- The fork's lint lines inside verify are left out, because ci.yml goes back to upstream's.
- `test_rust_pin.py` already exempts `ci.yml`. These tests read ci.yml and take upstream's versions in the sync:
  `test_ci_runners`, `test_ci_rust_changes`, `test_frame_identity`, `test_pr_ready`, `test_release_train` and
  `test_repository_hygiene`.

| Check now in our ci.yml | Goes to | Trigger |
| --- | --- | --- |
| `check_api_boundary.py --source-only`, `check_mod_vocabulary_state.py` | mod-checks / `tree` (Python, seconds) | paths |
| `test_prepared_*.py` (prepared-data tool tests) | mod-checks / `tree` | paths `tools/ci/prepare*`, `crates/horizon-prepare/**` |
| `check_ignored_test_rows`, `check_game_boot_tests`, `check_process_tests`, `reexec_include_ignored --check` | mod-checks / `tree` | paths incl. `tools/ci/ignored-tests/**`, `crates/**/tests/**` |
| `hotspots/check_lists`, `check_stand_ins`, `check_wire_loopback` | mod-checks / `tree` | paths |
| mod-host map tables (main pushes, an artifact) | mod-checks / `tree`, `workflow_dispatch` only | on demand |
| `check_api_boundary.py` (generated signatures), build `horizon-stage`, `surface_report.py`, `test_stage_catalog.py` | mod-checks / `surface` (Rust build) | paths on the mod API and vocab crates |
| extra `rustfmt` passes (mod-api and vocab `skip_children`, `game/`, dispatch-tests fixtures, wire_loopback tests) | mod-checks / `lint` | paths |
| `clippy -p horizon-client -p horizon-game -p horizon-mod-host --no-default-features` | mod-checks / `lint` | paths `crates/horizon-mod-host/**`, `game/modding/**`, `game/native_hooks/**` |
| holder_actor build before the tests | dropped: the only non-daily holder test, `windows_controller_quietly_declines_managed_mode_without_spawning`, starts no actor | – |
| `check_layers` and `check_addresses` with `--pull-request-merge` | upstream's form (`--base HEAD^1`) | – |
| `vanilla-parity` job (dispatch, horizon-game) | dropped: upstream removed it in horizon-rust/horizon#14209 and the fork kept it | – |
| wasm32 check of `horizon-mod-api` (not run today) | ci.yml verify-lint, when mod API files change (his point 6) | paths |
| Windows `cargo check` of the SDK, `check_warnings.py` wrapper, network-guarded runner (`run_test_binaries.py`) (none run today) | `cargo-horizon-mod.yml` or a dispatch-only `sdk-check.yml` (his point 6) | paths and dispatch |

**Open point to state in the PR.** A path-filtered workflow cannot be a required check, because a skipped workflow
leaves the check pending. So mod-checks is advisory, unless upstream wants it required. In that case it runs on
every PR and on `merge_group`, and decides inside, as verify does with `rust_changes.py`.

**Size.** ci.yml goes back to upstream's exactly (+15 lines). `mod-checks.yml` is new, about 150 lines. Tests change
by about 50 lines. This is medium-sized, written by hand.

## 2. The SDK built inside nightly-build.yml for the Nightly's commit, published with it; no snapshot or daily-green flow

**Today.**
- `daily.yml` builds the SDK, publishes `sdk-<sha>`, moves `daily-green` when every leg is green, and calls
  `release.yml`, which runs `nightly-build` with the SDK index.
- `release.yml` also has `kind=snapshot`.
- On the branch, `nightly-build.yml` differs from upstream's by +163 lines and `release.yml` by +143.

**Change (prep step `nightly-sdk`, hand diff).**
- `nightly-build.yml`: upstream's current file, which upstream only starts by hand since 28321687063, plus:
  - job `sdk` (`uses: ./.github/workflows/prebuilt-game-build.yml`, commit = `pick.outputs.sha`);
  - job `sdk-index`, the index step of `sdk-publish.yml`;
  - `sign` includes `sdk.json` in SHA256SUMS;
  - `publish` creates `sdk-<sha>` with the same release keys before the Nightly (`require-sdk`).
  - About +70 lines over upstream's.
- `sdk-publish.yml` is either folded into nightly-build's publish job (his wording) or kept as the reusable
  workflow it is. I'd fold it.
- `release.yml`: upstream's file (−143 lines). `game-build.yml`: the `kind` input and the snapshot `continue-on-error`
  are removed. The other fork changes stay: pinned Rust, wasm-std, the glibc baseline, the macOS checks.
- `tools/ci/nightly.py`:
  - removed: `paired`, `green_base` (daily-green), `pick_snapshot`, `prune_snapshots`, `--kind`;
  - kept: `sdk-index`, `require-sdk`, `sdk-previous`, `sdk_ready`.
- `tools/release_train.py`: upstream's `green_main_head` (checks + parity-main) and no repository switch.
- `tools/ci/sign_release.sh`: drop the snapshot channel.
- `daily.yml`: drop the `sdk`, `sdk-publish`, `sdk-publish-report` and `nightly` jobs and the tag move.
  `daily_summary.py`: drop the tag move.
- `tests/test_snapshot.py`: delete. `test_nightly` and `test_release_train`: trim.
- `docs/release-flow.md` and `docs/operations.md`: rewrite the Nightly and snapshot sections.
- The launcher's `snapshot-*` handling stays as it is (no such tags will exist). That is harmless; I'd remove it after
  the merge.

**Size: second largest.** About −1,100 to −1,300 lines and +150 across about 15 files, by hand. The fork's own
snapshot flow ends with this branch only; main keeps it until the merge.

## 3. Daily ignored tests out of the merge gate, in their own workflow on GitHub's runners with a time limit

**Today.**
- Our `ci.yml` already has no daily tests: the gate holds unit tests only, so "out of the merge gate" holds already.
- The daily legs that need `[self-hosted, linux, horizon-game]` are the ignored-tests shards, the wasm fixture,
  hot-paths and paper-parity (macOS). His point: those machines are gone.

**Change (prep step `daily-github`).**
- `ignored-tests.yml`:
  - `runs-on: ubuntu-latest`;
  - `timeout-minutes` 120, a guess: the client shards took 180 on the self-hosted runner, and GitHub's runners are
    not measured;
  - no `vanilla-data.sh` step.
- GitHub's runners have no vanilla data, and Horizon never downloads it. A shard there can run only the rows that
  need none. Of the 881 daily rows:
  - 518 can run there: `slow` 143, `child-process` 230, `network` 145;
  - 209 cannot: `resources` 154 and `game-boot` 55;
  - also out are the 22 `harness`, 108 `needs` and 3 `gpu` policy rows.
- This needs a new `--without-data` selection in `ignored_tests.py` and `ignored_shards.py`, about 60 lines of code
  and 40 of tests. Without it, the data rows fail, because a listed row that does not run fails.
- The data rows run nowhere until the new server exists. The PR says so.
- `wasm-mods.yml`: the fixture job needs vanilla data, so it is dispatch only; `guest-stack` stays in the daily run.
- `hot-paths.yml`: schedule removed and the `record` job (the `perf-history` branch) removed. It stays as a dispatch
  that prints its measurement. `paper-parity.yml`: schedule removed.
- `wasm-coverage.yml` runs 150 minutes on GitHub's runners on a schedule. I'd make it dispatch only, for his
  Actions-usage point.
- `daily.yml` keeps its schedule, with these legs:
  - shards without data;
  - guest-stack;
  - fn-replace-check;
  - linkage (native-dylib-probe);
  - Windows runtime leases.
- The summary files issues only; its permission drops to `issues: write`.

**Size.** About 200 lines of workflow and code, by hand. Not measured: how long the data-free shards take on
GitHub's 4-core runners, so the time limit may need one adjustment after the first run.

## 4. The book as Markdown in docs/modding, no Pages deploy; daily-green and perf-history dropped

**Change.**
- Delete `modding-book-pages.yml` (40 lines).
- `modding-book.yml` stays as the build check on pull requests that touch the book (his point 5), without the
  fork-PR skip.
- About 49 links to `horizon-rust.github.io/horizon/<path>` become
  `https://github.com/horizon-rust/horizon/blob/main/docs/modding/src/<path>.md`. They sit in 20 files:
  - `horizon-mod-api` doc comments;
  - docs/README, layers, modpacks, native-mods and modding-book.md;
  - `llms.txt` and the book's reference page;
  - `launcher/links.json`;
  - a refactor script.

  A prep step `book-links` extends `rename_repository.py`. It maps `concepts/`, `reference/`, `how-to/`, `start/`
  and `api` paths to their source file, and the root to `docs/modding/src/introduction.md`.
- The `horizon-mods` registry links (11, `horizon-rust.github.io/horizon-mods/...`) are upstream's and are not
  touched.
- `daily-green` (50 lines in 12 files) goes with section 2. `perf-history` (8 lines: `hot-paths.yml` and
  `docs/hot-path-trends.md`) goes with section 3.

**Size.** Small: about 60 lines, scripted.

## 5. Old issue references as full arismoko/horizon#N links

**Counts.** These cover the fork's own lines: the diff from the last synced upstream commit `d1b6cd286ea` to
`upstream-prep`, with rename detection, so upstream text that only moved does not count.
- Already full: 23 `arismoko/horizon#N` and 148 `github.com/arismoko/horizon/issues|pull/N`. They stay.
- Bare `#N` with N ≤ 4600 (the fork's range; its newest is about #4500): **about 1,334 references on about 1,195
  lines in about 476 files**:
  - by area: crates 614, docs 533, tools 111, ROADMAP and roadmap 34, tests 16, .github 12, clippy.toml 7,
    AGENTS and CONTRIBUTING 5;
  - by kind: Rust comments 611, Markdown 582, Python 63, TOML 29, TSV 28, YAML 12, txt 9.
- Bare `#N` with N > 4600 (68): upstream's numbers, left as they are.

**Script (`rewrite_issue_refs.py`, then a hand-fix commit).**
- Only lines that differ from the merge base with upstream (`git diff -M`) are touched, so upstream's own `#N` are
  never touched.
- `(?<![\w/&#-])#(\d{1,4})\b` becomes `arismoko/horizon#\1` when N ≤ the fork's highest number.
- Skipped: URLs, `owner/repo#N` already written, `.json`, `.css`, `.svg`, `.wgsl` and `.html`.
- In Rust and Python only comment and doc lines (`//`, `///`, `//!`, `#`, docstrings) are touched, never string
  literals, so test data such as chat text is safe.
- `tools/ci/ignored-tests/*.txt` REFERENCE tokens (`#2177`) are rewritten too. The parser takes one token, but the
  rows read through `ignored_tests.py`'s tests.
- The script writes a review list (file, line, number) for the hand pass, to catch an upstream number at or below
  4600 that a sync conflict brought into a fork line.
- Comments grow by 16 characters. Rustfmt does not wrap comments, so nothing reflows.
- A proof counts what is left: bare `#N` at or below 4600 in fork lines must be 0, apart from the reviewed list.

**Size: largest by line count**, but mechanical: about 1,200 lines in about 475 files, one scripted commit.

## 6. Upstream main moved: how far behind, and what collides

**Behind.**

| Commit | Commits behind `upstream/main` (`8c383a5dcb5`) | Upstream first-parent commits behind |
| --- | --- | --- |
| fork main `0dbf78e35f0` (after sync 105) | 386 | 134 |
| `upstream-prep` base `77b62ef6b60` | 635 | 229 |

His "621" matches the branch's base when he wrote. Main is now 386 behind.

**Trial merge** (`git merge-tree`; nothing written) of fork main with upstream main: **169 conflicted paths**.

| Kind | Count | Where |
| --- | --- | --- |
| content | 93 | 48 `horizon-client/src/game/`, 7 mobs, `ci.yml`, `nightly-build.yml`, `release.yml`, AGENTS/CONTRIBUTING/README, `pr_ready.py` + test, `rust_changes.py` + test, 6 `tools/port_map/*.tsv`, a few world and items files |
| rename/rename | 51 | **the integration-test layout**: horizon-world 33, horizon-shaders 17, horizon-client 1 |
| modify/delete | 15 | horizon-client files the fork moved into `game/<area>/` and upstream edited |
| file location | 10 | new upstream tests in `game/commands/` and `game/functions/`, which the fork moved |

`upstream-prep` itself against upstream main: 354 conflicted paths. Once main is synced it is rebuilt, never merged.

**Each of his changes against the fork's tree.**
- **One `tests/it` binary per crate** (horizon-rust/horizon#16504, `ec85e818d88`): 11 crates moved 112 files.
  - The fork had already merged horizon-shaders and horizon-world into `tests/integration/`. That gives the 51
    rename/rename conflicts. The fix is to take upstream's name: `integration/` → `it/`, with the union of both
    `main.rs` module lists.
  - Fork-only test files in those crates move into `it/` with a `mod` line: horizon-client 4, client-render 2,
    core 1, plus the top-level files left in shaders 2 and world 1.
  - `horizon-client/Cargo.toml`'s per-file `[[test]]` entries (`datapack_biome_sync`, `mod_biome_sync`,
    `mod_api_hooks`, `skull_models`, ...) collapse into upstream's `[[test]] name = "it"`.
  - **51 daily rows** naming `test:<file>` in these crates become `test:it` with a `file::` prefix: relay 22,
    perf 8, release-sign 8, shader-api 4, world 4, client-render 1, perf-format 1. The 7 `test:integration` rows
    become `test:it`. The row format already supports the prefixed form.
  - A script `merge_test_binaries.py` (git mv, `main.rs`, Cargo, rows) does it in the sync.
  - The fork's own crates (mod-api 21 files, mod-wasm 20, mod-points 7, cargo-horizon-mod 7, about 70 in all) can
    follow the same rule. No upstream check requires it; I'd do it after the merge, since it touches about 60 more
    rows.
- **Merge queue with squashes.** Taking upstream's `ci.yml` brings `merge_group` and the "queued" skip. Upstream's
  `parity-main.yml` already reads squash titles (horizon-rust/horizon#16593). `mod-checks.yml` needs `merge_group`
  only if it is required (section 1). This PR itself merges with a merge commit (his answer).
- **3000-line PR limit** (horizon-rust/horizon#16586, a `pr_ready.py` warning). It conflicts with `pr_ready.py`,
  `tests/test_pr_ready.py`, and the fork's CONTRIBUTING and AGENTS "no line limit" lines; take upstream's wording.
  - Correction to his "the file size check is already in place": upstream **deleted** `tools/check_file_size.py`
    on 2026-11-14 (`a48b460d9fa`, "Remove the 3000-line file size rule").
  - So the fork's `check_file_size.py`, `file_size_baseline.txt` and `tools/sync/write_file_size_baseline.py` go
    with it. The sync takes the deletion, and `tools/sync/README.md`'s section goes.
- **Nightly by hand** (horizon-rust/horizon#16599). It conflicts in `nightly-build.yml` and `release.yml`, and is
  covered by section 2: the fork's daily run no longer publishes a Nightly.

**Size: the largest piece of work** (169 conflicts), but it is a sync on main by the usual procedure, not a prep
step. `prep.sh` needs no change for it.

## prep.sh and the tool folder: the full list

New `align.py` steps, after the current ones:
1. `ci-upstream`
2. `mod-checks`
3. `nightly-sdk`
4. `daily-github`
5. `book-links` (in `rename_repository.py`)
6. `issue-refs` (new `rewrite_issue_refs.py`, then a hand pass)

Each is one commit; the script steps get a separate hand-fix commit. The existing `telemetry-registry`, `claim-limit`
and `runner-watch` steps should become no-ops after the sync, since main will already match upstream there; this gets
checked on rebuild.

New proofs:
- `ci.yml` equals upstream's apart from the wasm32 step;
- no `daily-green`, `perf-history`, `kind: snapshot` or `modding-book-pages` left;
- bare fork `#N` count is 0;
- actionlint, Python tests and cargo check as now.

`pr-body.md` and `PR-BODY-DRAFT.md`: the questions become statements (proposed diff in
`/box/shared/root/pr-visuals/PR-BODY-DRAFT.md`, at the end).

## Three largest items

1. **Section 6, the sync with upstream main:** 386 commits, 169 conflicted paths, 51 of them the integration-test
   layout, plus 51 daily rows. It needs the owner's word, because it runs before the PR opens.
2. **Section 2, the SDK inside nightly-build and the snapshot/daily-green flow removed:** about −1,200 / +150 lines in
   about 15 files, by hand.
3. **Section 5, old issue references:** about 1,334 bare `#N` on about 1,195 lines in about 476 files. It is
   scripted, with a review list for the hand pass.
