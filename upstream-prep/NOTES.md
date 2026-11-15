# upstream-prep working notes (session upstream-prep)

Working checkout: /box/work/arismoko/horizon on branch upstream-prep (from origin/main 77b62ef6b60). Upstream remote `upstream`.
Merge base with upstream: see `git merge-base origin/main upstream/main` (d1b6cd286ea at start).
Edit helper: /tmp/rep.py (edit(path, [(old,new)]) exact-once replacements). Recreate if /tmp lost.

## Decisions
- dyna-ci.yml removed; its product-only checks ported into ci.yml verify job (test-tier checks, hotspot guards,
  prepared-data tool tests, surface gate via horizon-stage + surface_report.py + test_stage_catalog.py).
- Kept workflows: route jobs (dyna_ci_runner.sh) removed. Jobs needing vanilla data -> [self-hosted, linux, horizon-game]
  (upstream's own label, ci.yml vanilla-parity) + runner-env.sh + vanilla-data.sh; others -> ubuntu-latest.
  Done: ignored-tests, wasm-mods, wasm-coverage, fn-replace-check, native-dylib-probe, machine-sample, hot-paths,
  daily (schedule-dispatch repo guard dropped), game-build (Dyna runner vars + linux-smoke job removed), nightly-build.
- release_train.py: dyna-ci.yml -> ci.yml, gate jobs -> {verify, launcher}; tests adjusted.
- tools/sync: keep README.md + write_file_size_baseline.py (check_file_size.py, docs/layers.md need them).
- tools/hotspots: keep checks + carry.py (refactors use it); drop measure/open_prs/game_rs via README edit.
- Claim limit: ours (b852d8293bb, dc3216bc2ab, daad83cc838) -> own commit removing it.

## TODO
- release.yml comments; wasm-mods/fn-replace-check "root dispatches"; cargo-horizon-mod/launcher-windows-check/modding-book pr_gate refs
- ci.yml port; tests referencing dyna-ci.yml; pr_ready.py comment
- docs scrub; claim limit; prep script + patches; proofs; report
- 2026-11-14: align.py (19 commits total) resolves the ASK list toward upstream; pr-body.md is the owner's draft. Rerun: CARGO_CHECK=1 PUSH=1 bash prep.sh.
- 2026-11-15: rebuilt on main 1151e8d700 (after upstream sync #4523). New: UPSTREAM_REF = merge-base(main, upstream/main)
  for every take of upstream's files; remove.py keeps upstream's own files; mod-checks.yml lives in patch 0001; new align
  steps ci-upstream, nightly-sdk, daily-github (hand diffs), book-links (book_links.py), issue-refs (rewrite_issue_refs.py
  + roadmap rebuild). proofs.sh runs the Python tests with a plain git identity (no commit signing). PR description
  source: private repo arismoko/horizon-mod-api-pr-draft; draft here in pr-draft/README.md (edit_readme.py from README.orig.md).
  drop_from_diff.py removes a file's section from a hand diff when a later rebuild deletes that file.
