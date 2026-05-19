# Maintainers

This file documents who is responsible for the project and how to
reach them. For day-to-day contributor information, see
[CONTRIBUTING.md](./CONTRIBUTING.md).

## Active maintainers

| Handle | Areas |
| --- | --- |
| [@ubugeeei](https://github.com/ubugeeei) | Compiler, shell runtime, LSP, CI, releases — everything |

The project is currently single-maintainer. New maintainers will be
added here when the situation changes.

## How to reach a maintainer

- **Bug reports, feature requests, general discussion** — open a
  GitHub issue using the templates under
  [`.github/ISSUE_TEMPLATE/`](./.github/ISSUE_TEMPLATE/).
- **Security reports** — follow the private channel in
  [SECURITY.md](./SECURITY.md). Do **not** open a public issue for
  security matters.
- **Pull request review** — the [CODEOWNERS](./.github/CODEOWNERS)
  file auto-requests review from the maintainer on every PR. Pings
  in the PR description / comments are welcome if it's been more
  than a few days without a response.

## Release authority

Only a maintainer may:

- push a `v*` tag,
- dispatch the `Release` or `Cut Release` workflow,
- publish a GitHub Release.

Anyone can prepare a release PR (version bump + CHANGELOG move) and
hand it off; the actual tag push is gated by the protected `main`
branch and the `Cut Release` workflow's permissions.

The release procedure itself is documented in
[`docs/release-process.md`](./docs/release-process.md).
