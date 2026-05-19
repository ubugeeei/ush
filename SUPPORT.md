# Support

Where to go for what.

## Bugs and feature requests

Open a GitHub issue using the templates under
[`.github/ISSUE_TEMPLATE/`](./.github/ISSUE_TEMPLATE/). The
templates pre-populate the right shape — please use them rather
than blank issues.

## Security vulnerabilities

**Do not open a public issue.** Follow
[SECURITY.md](./SECURITY.md):

1. Use GitHub's Private Vulnerability Reporting
   ([Security tab → "Report a vulnerability"](https://github.com/ubugeeei/ush/security/advisories/new)),
   **or**
2. Email the maintainer at the address listed on their GitHub
   profile.

SLA, supported versions, and scope are all documented in
[SECURITY.md](./SECURITY.md).

## "How do I…" / usage questions

The README's [Usage](./README.md#usage),
[Interactive Editing](./README.md#interactive-editing),
[Structured Helpers](./README.md#structured-helpers), and
[Ush Scripts](./README.md#ush-scripts) sections cover the supported
flows. If something is missing or confusing, a doc improvement is
welcome as a PR — that is usually faster than waiting on a support
reply.

For deeper reading:

- [docs/architecture.md](./docs/architecture.md)
- [docs/language-vision.md](./docs/language-vision.md)
- [docs/lowering.md](./docs/lowering.md)
- [docs/typed-errors.md](./docs/typed-errors.md)
- [docs/lsp.md](./docs/lsp.md)
- [docs/release-process.md](./docs/release-process.md)
- [docs/source-docs.md](./docs/source-docs.md)
- [docs/sourcemaps.md](./docs/sourcemaps.md)

## Supported versions

Only the most recent `0.x` minor release line is supported with
patches. Older releases are not actively patched; please upgrade
before reporting issues.

| Release line | Status                         |
| ------------ | ------------------------------ |
| 0.7.x        | Actively supported             |
| 0.6.x        | Security fixes only            |
| < 0.6        | Prototype, no longer supported |

`ush` is pre-1.0 and may introduce breaking changes between minor
releases. Each release's user-visible changes are tracked in
[CHANGELOG.md](./CHANGELOG.md).
