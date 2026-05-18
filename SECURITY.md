# Security Policy

`ush` is a shell that executes user-supplied scripts directly, ships
its own installer (`curl … | sh`), and compiles a `.ush` source
language down to POSIX `sh`. The security boundary therefore spans:

- the interactive REPL and its readline integration,
- the POSIX runtime (signal handling, job control, builtin commands,
  redirections, alias / dangerous-command guards),
- the `.ush → sh` compiler (shell-injection at codegen time,
  source-map and CLI-argument handling),
- the installer (`install.sh`) and the released archives.

## Supported versions

Only the most recent `0.6.x` release line receives security fixes.
Earlier versions are considered prototypes and are not supported.

| Version | Status                |
| ------- | --------------------- |
| 0.6.x   | Supported             |
| < 0.6   | Not supported (WIP)   |

## Reporting a vulnerability

**Please do not open a public issue or PR for a security report.**

Use one of the following private channels:

1. **GitHub Private Vulnerability Reporting (preferred).** Go to the
   repository's "Security" tab and choose
   _"Report a vulnerability"_. This opens a private advisory only
   visible to the maintainers.
2. **Email.** Send a report to the maintainer's GitHub-listed email
   address with the subject `[ush][security] …`.

Please include:

- The version of `ush` (`ush --version`) and the OS / architecture.
- A clear description of the issue and the impact you observed.
- A minimal reproducer (a `.ush` snippet, a script, or an installer
  invocation) — or, if you cannot share one publicly, a description
  of what would reproduce it.
- Whether you intend to publish your own write-up, and on what
  timeline.

## Response process

- **Acknowledgement** within 7 days.
- **Triage** (confirmed / not-applicable / needs more info) within
  14 days.
- **Fix or mitigation** communicated within 30 days of triage. For
  more complex issues we will agree on a timeline with you in the
  advisory thread.
- **Coordinated disclosure window** of up to 90 days after the
  initial report, with the option to extend by mutual agreement if a
  fix is in progress.
- Credit in the advisory and the changelog if you wish.

## Scope

In scope:

- Shell-injection or command-injection via `.ush` codegen, alias
  rendering, dangerous-command bypass, or runtime helpers.
- TOCTOU and privilege issues in `install.sh`.
- Memory-safety issues in the Rust code (any `unsafe` block, signal
  handling, FFI boundary).
- Issues in vendored dependencies (`vendor/rustyline`) **when
  exploitable through `ush`'s usage**.

Out of scope:

- Issues that require an already-compromised local account or root
  access.
- Theoretical issues in `vendor/rustyline` that have not been
  demonstrated against `ush`.
- Denial of service against tools that explicitly take untrusted
  input (e.g. `ush format`) by feeding pathological input.
- Bugs in third-party plugins / scripts written in `.ush`.

## Hardening reports

Hardening suggestions that do not constitute a vulnerability are also
welcome — please open a regular issue with the `security` label.
