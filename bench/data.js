window.BENCHMARK_DATA = {
  "lastUpdate": 1779160350081,
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
      }
    ]
  }
}