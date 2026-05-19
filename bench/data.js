window.BENCHMARK_DATA = {
  "lastUpdate": 1779160928113,
  "repoUrl": "https://github.com/ubugeeei/ush",
  "entries": {
    "Criterion microbenchmarks": [
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "538a0563412cb831d5503bf56edc958ea844faf2",
          "message": "ci: stop cancelling main-branch runs on every subsequent push (#106)\n\nEvery workflow used `group: <name>-<workflow>-${{ github.ref }}`\nwhich means rapid pushes to `main` all land in the same concurrency\ngroup. Even with `cancel-in-progress: false` set for non-PR events,\nthe GitHub Actions queue ends up cancelling earlier runs when\nseveral newer ones pile up behind them, because the runner pool\ntreats stale queue heads as superseded.\n\nSwitch every workflow's concurrency group expression to:\n\n    <name>-<workflow>-${{ github.ref == 'refs/heads/main'\n                          && github.run_id\n                          || github.ref }}\n\nSo each main-branch run gets its own unique group keyed on\n`run_id` (and therefore cannot be superseded by anything), while\nPR pushes keep the existing \"cancel the previous run on the same\nbranch\" semantics keyed on `github.ref`.\n\nTouched workflows: ci.yml, codeql.yml, secret-scan.yml,\nshellcheck.yml, dependencies.yml.\n\nCo-authored-by: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-19T04:16:21+09:00",
          "tree_id": "d240a5070d5e58ed958cd184545a034894c8ef24",
          "url": "https://github.com/ubugeeei/ush/commit/538a0563412cb831d5503bf56edc958ea844faf2"
        },
        "date": 1779133043696,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3206,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 84444,
            "range": "± 2949",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 111491,
            "range": "± 673",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d057251b90b38fbc5eb3e37d043256d952a0e00c",
          "message": "chore: add scripts/preflight.sh as a one-command CI mirror (#107)\n\nRuns every gate CI runs (rustfmt, clippy, workspace tests, release\ntests, no_std check/tests, rustdoc, bench build, line limit,\nrustyline drift, installer, shellcheck if available, and\n`ush check`/`ush format --check` on every example) in the same\norder CI runs them, with a banner for each section so the failing\ngate is obvious in plain `sh -e` output.\n\nCONTRIBUTING.md gets a one-line pointer.\n\nCo-authored-by: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-19T04:18:38+09:00",
          "tree_id": "7b72cfee9b9798b9c318c58625afecdf17bf4187",
          "url": "https://github.com/ubugeeei/ush/commit/d057251b90b38fbc5eb3e37d043256d952a0e00c"
        },
        "date": 1779133078590,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3042,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 81909,
            "range": "± 1742",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 114893,
            "range": "± 724",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b11b35fffaa7abe48d522ea350094d27f5b7bab1",
          "message": "test(cli): extend smoke suite with `ush compile` and `ush check` (#105)\n\nPins two more outer-CLI guarantees that downstream automation\nrelies on but that nothing was previously asserting:\n\n- `ush compile <file.ush>` lowers a trivial program to POSIX `sh`\n  (output begins with `#!/bin/sh`, and `print \"hi\"` is lowered to\n  the expected `printf` invocation).\n- `ush check <file.ush>` exits 0 for a well-typed program.\n\nThe tests use the existing `tempfile` dev-dependency to materialise\ninputs, so they remain hermetic.\n\nCo-authored-by: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-19T04:09:42+09:00",
          "tree_id": "552f55225600b145ec2abc4b2fcee2fc0256c262",
          "url": "https://github.com/ubugeeei/ush/commit/b11b35fffaa7abe48d522ea350094d27f5b7bab1"
        },
        "date": 1779133109947,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3018,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 81180,
            "range": "± 7459",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 101637,
            "range": "± 1165",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8a3172649442e9afa8d2aacfd92e5e61b5446d8f",
          "message": "feat(ush): install a user-facing panic hook on the CLI entrypoint (#108)\n\nWith `panic = \"abort\"` set in the release profile, the default\npanic message ends up looking like a raw rustc diagnostic to anyone\nwhose shell session just died. The custom hook replaces it with a\nconsistent, end-user-friendly message that:\n\n1. clearly identifies which binary panicked,\n2. surfaces the panic payload + source location, and\n3. points at the bug tracker with the version string already in\n   the line, so a copy-paste-into-issue actually contains enough\n   information to be triaged.\n\n`main()` calls `panic_hook::install()` first, before any other\nwork runs.\n\nCo-authored-by: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-19T10:17:14+09:00",
          "tree_id": "31fd927442bf3176d3bba8ff3d1307636fd5e2a8",
          "url": "https://github.com/ubugeeei/ush/commit/8a3172649442e9afa8d2aacfd92e5e61b5446d8f"
        },
        "date": 1779153714680,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3053,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 82824,
            "range": "± 504",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 112549,
            "range": "± 1098",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "42fb7512205fc422b08c1051b07a630f6423094c",
          "message": "chore(release): bump version to 0.7.0 (#109)\n\nA production-readiness release. Folds the contents of `[Unreleased]`\ninto a fresh `## [0.7.0] — 2026-05-19` section in CHANGELOG.md and\nbumps `[workspace.package].version` from 0.6.1 → 0.7.0.\n\nHighlights of 0.7.0 (see CHANGELOG for the full list):\n\n- Compiler enforces match exhaustiveness in the effects pass; no\n  more silent fall-through on uncovered ADT variants.\n- Shell signal helpers move to sigaction(2) + checked pid_t casts.\n- install.sh hardens its trust surface (umask, pipefail, TLS pin).\n- CI matrix across Ubuntu + macOS, MSRV gate (now 1.88),\n  cargo-audit, cargo-deny, CodeQL, Gitleaks, Shellcheck, benchmark\n  regression gate, rustdoc -D warnings, `ush check` / `ush format\n  --check` over examples.\n- Linux aarch64 release archive.\n- Release profile uses LTO + strip + panic=abort, halving binary\n  sizes; user-facing panic hook on the CLI entrypoint.\n\nCo-authored-by: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-19T10:22:09+09:00",
          "tree_id": "55abad44ef8c36c596d02b87ee42a055586d3fdb",
          "url": "https://github.com/ubugeeei/ush/commit/42fb7512205fc422b08c1051b07a630f6423094c"
        },
        "date": 1779154124927,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3229,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 83477,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 104414,
            "range": "± 1655",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0a6c9b57884081c64bb8e5d0b76a219e94467421",
          "message": "ci(deps): bump the actions group with 2 updates (#92)\n\nBumps the actions group with 2 updates: [github/codeql-action](https://github.com/github/codeql-action) and [actions/labeler](https://github.com/actions/labeler).\n\n\nUpdates `github/codeql-action` from 3 to 4\n- [Release notes](https://github.com/github/codeql-action/releases)\n- [Changelog](https://github.com/github/codeql-action/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/github/codeql-action/compare/v3...v4)\n\nUpdates `actions/labeler` from 5 to 6\n- [Release notes](https://github.com/actions/labeler/releases)\n- [Commits](https://github.com/actions/labeler/compare/v5...v6)\n\n---\nupdated-dependencies:\n- dependency-name: github/codeql-action\n  dependency-version: '4'\n  dependency-type: direct:production\n  update-type: version-update:semver-major\n  dependency-group: actions\n- dependency-name: actions/labeler\n  dependency-version: '6'\n  dependency-type: direct:production\n  update-type: version-update:semver-major\n  dependency-group: actions\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-19T12:09:12+09:00",
          "tree_id": "41c13a3feadcdcbe6c873a63fe6ad708c78d8fe3",
          "url": "https://github.com/ubugeeei/ush/commit/0a6c9b57884081c64bb8e5d0b76a219e94467421"
        },
        "date": 1779160261308,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3028,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 80292,
            "range": "± 1560",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 98754,
            "range": "± 1379",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82c17aa63e7756826ddb55e0d5f08e79480d871a",
          "message": "docs(readme): add CodeQL and Secret-scan badges (#110)\n\nThe two security workflows added in 0.7.0 had no surface presence\nin the README. Adds their badges next to the existing CI /\nShellcheck / Dependencies / License row so the security posture is\nvisible at first glance.",
          "timestamp": "2026-05-19T12:10:29+09:00",
          "tree_id": "998729d132ab3c684f5a0802f9e9cc5a2a0ac6bc",
          "url": "https://github.com/ubugeeei/ush/commit/82c17aa63e7756826ddb55e0d5f08e79480d871a"
        },
        "date": 1779160349301,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3038,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 81267,
            "range": "± 1885",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 99833,
            "range": "± 5017",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "db8af1fb46cf63a9d8a09168df3de467e4d1aca5",
          "message": "docs: link release-process.md from the docs/ index (#111)\n\nThe doc landed in 0.7.0 but only README.md pointed at it; the docs/\nindex had no entry. Adds the entry so it discoverable from inside\nthe `docs/` tree.",
          "timestamp": "2026-05-19T12:11:46+09:00",
          "tree_id": "bdab7995e2966e42db9702ca4a06b5812e627a61",
          "url": "https://github.com/ubugeeei/ush/commit/db8af1fb46cf63a9d8a09168df3de467e4d1aca5"
        },
        "date": 1779160448818,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3026,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 80295,
            "range": "± 3553",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 100137,
            "range": "± 731",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "59e35cdafb13cd430b3ba2a05e7c36ee33d77799",
          "message": "docs(readme): add a Production readiness section (#112)\n\nSummarises the CI / supply-chain / runtime hardening posture that\nlanded across the 0.7.0 cycle (correctness, static analysis, supply\nchain, runtime, release pipeline, performance gate) so a downstream\noperator can read it in one place without spelunking the workflow\nfiles.",
          "timestamp": "2026-05-19T12:13:10+09:00",
          "tree_id": "51f557e4b150098b30fa435e2d1433760165fe59",
          "url": "https://github.com/ubugeeei/ush/commit/59e35cdafb13cd430b3ba2a05e7c36ee33d77799"
        },
        "date": 1779160514827,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3021,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 80387,
            "range": "± 988",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 98312,
            "range": "± 297",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5d3eda631164188dbe3e282009e55f9634b7d295",
          "message": "test(lsp): exercise a real LSP initialize/shutdown/exit handshake (#113)\n\nThe existing smoke tests only cover the CLI surface (--version,\n--help, unknown flag). They never actually spoke LSP, which means\na regression that broke initialize parsing, exit handling, or the\nJSON-RPC framing would slip through.\n\nThe new integration test:\n\n  - launches the real `ush_lsp` binary over a stdio pipe pair,\n  - sends `initialize`, asserts the response is shape-correct,\n  - sends `initialized`,\n  - sends `shutdown`, asserts the response,\n  - sends `exit` and asserts the process terminates cleanly,\n  - aborts the whole flow with a 5s deadline if any read blocks.\n\nAdds `serde_json` as a dev-dependency in `apps/ush_lsp/Cargo.toml`\n(it was already a workspace dep) so the test can parse responses.",
          "timestamp": "2026-05-19T12:15:30+09:00",
          "tree_id": "f0ff18c004e572b4f5b37acbd5cc8be7abbeba1c",
          "url": "https://github.com/ubugeeei/ush/commit/5d3eda631164188dbe3e282009e55f9634b7d295"
        },
        "date": 1779160658140,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3021,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 79930,
            "range": "± 4552",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 99104,
            "range": "± 564",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "64d13e60ae27fd46938ecfcdcf85dc5ad2635096",
          "message": "docs: add MAINTAINERS.md (#114)\n\nDocuments who owns the project, how to reach them (issue templates,\nSECURITY.md, CODEOWNERS), and who has release authority. Companion\nto CONTRIBUTING.md / SECURITY.md / CODE-of-conduct conventions that\nproduction-grade GitHub projects ship.",
          "timestamp": "2026-05-19T12:16:29+09:00",
          "tree_id": "eaef0da10dd9ca1e85073ae0d4086b1c26284cfc",
          "url": "https://github.com/ubugeeei/ush/commit/64d13e60ae27fd46938ecfcdcf85dc5ad2635096"
        },
        "date": 1779160710155,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3239,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 83204,
            "range": "± 409",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 103104,
            "range": "± 2551",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ubuge1122@gmail.com",
            "name": "ubugeeei",
            "username": "ubugeeei"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "29f8db61ea33a24c7c51d16fda01910d91a9e0d2",
          "message": "ci: pin every third-party action to a full commit SHA (#115)\n\nMutable major-version tags (e.g. `actions/checkout@v6`) are\ntrust-on-first-use: the same ref can be quietly retargeted to a\ndifferent commit at any time, so a compromised maintainer account\nupstream would silently rotate the action under us. Pinning to the\nfull commit SHA removes that class of supply-chain risk; Dependabot\nalready understands the format and will continue to open bump PRs\nfor each pinned SHA.\n\nPinned with the latest stable resolved SHA, with the human-readable\nversion preserved as a trailing comment so review diffs are still\nlegible:\n\n  - actions/checkout                 -> v6.0.2\n  - actions/labeler                  -> v6.1.0\n  - actions/upload-artifact          -> v7.0.1\n  - actions/download-artifact        -> v8.0.1\n  - actions/attest-build-provenance  -> v4.1.0\n  - Swatinem/rust-cache              -> v2.9.1\n  - softprops/action-gh-release      -> v3.0.0\n  - benchmark-action/github-action-benchmark -> v1.22.1\n  - github/codeql-action/{init,autobuild,analyze} -> v4.0.0\n\nThe two `dtolnay/rust-toolchain@stable` / `@master` references are\nintentionally left as moving refs: they exist precisely so the\ntoolchain rolls forward with upstream rustc releases, and pinning\nthem would require a fresh PR every few days for no real security\ngain (the action is install-only, not a privileged step).",
          "timestamp": "2026-05-19T12:20:01+09:00",
          "tree_id": "0a3ee018b4b8a46d28e50ddf4590120702decd9c",
          "url": "https://github.com/ubugeeei/ush/commit/29f8db61ea33a24c7c51d16fda01910d91a9e0d2"
        },
        "date": 1779160927359,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse pipeline with helper",
            "value": 3010,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "compile small ush program",
            "value": 80334,
            "range": "± 978",
            "unit": "ns/iter"
          },
          {
            "name": "compile adt ush program",
            "value": 99027,
            "range": "± 466",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}