# ush

`ush` is an experimental Rust shell with a simple idea:

- Be POSIX-first. Existing shell scripts and one-liners should keep working.
- Keep stdio Unix-simple by default, but allow richer human-facing output with `--stylish`.
- Add a modern `.ush` language, but compile it to portable `sh` instead of inventing a separate runtime.
- Make the interactive shell feel fast, lightweight, and pleasant to edit in.

> [!WARNING]
> `ush` is still an early WIP / prototype.
> Expect rough edges, missing POSIX coverage, incomplete language features, and breaking changes.
> It is not yet a production shell replacement.

## Concept

`ush` is split into two layers:

- `ush_shell`: an interactive shell runtime with POSIX-friendly execution, REPL UX, stylish mode, and structured helper pipelines
- `ush_compiler`: a `.ush` to `sh` compiler, where scripts stay portable because execution still happens through POSIX `sh`

That means the project can explore a more modern shell language without giving up the portability and compatibility of `sh`.
The `no_std` target is only the compiler core. The app binary and interactive shell runtime are still intentionally `std`-based because they need processes, terminals, files, and OS integration.

## Status

This repository is an MVP focused on architecture and interaction experiments.

Implemented today:

- Interactive shell REPL in Rust
- `-c` command execution
- `.sh` / POSIX scripts executed through `/bin/sh`
- `.ush` scripts compiled to `sh` and then executed by `/bin/sh`
- Generated `.ush` output stays within POSIX `sh` syntax and POSIX command usage
- Prototype typed language features: `type { ... }`, enums, traits, marker `impl`, `match`, and typed `fn`
- Labeled function arguments plus parameter attributes such as `#[default(...)]` and `#[alias("n")]`
- `alias name = "..."` declarations in `.ush`
- `bin.ush` as a generated CLI entrypoint, with flags/defaults/completion derived from the `bin(...)` signature
- `crates/ush_compiler` builds as `no_std + alloc` with CompactString, SmallVec, bumpalo, memchr, phf, and Fx-hashed maps in the core path
- `apps/ush` and `crates/ush_shell` remain `std`-based by design
- Installer patterns such as `curl -fsSL https://... | sh` are detected from the parsed pipeline and executed through POSIX `/bin/sh`
- Builtins: `cd`, `pwd`, `alias`, `unalias`, `history`, `export`, `help`, `source`, `rm`
- Safety prompt for dangerous `rm -rf` unless `--yes` or `USH_INTERACTION=false`
- Stylish renderers for `pwd`, `ls`, `cat`, `ps`, and `kill`
- Structured helpers: `len`, `lines`, `json`, `xml`, `html`, `map`, `each`, `filter`, `any`, `some`
- Environment-variable expansion, `~` expansion, and simple glob expansion
- Criterion benchmark skeleton for parser/profiling work
- `curl`, `nix`, and Docker base-image packaging entry points
- Emacs-style and macOS-friendly cursor bindings for word jumps, line jumps, and history search
- `ush format` and `ush check` commands for formatter and typechecking passes
- `ush_lsp` with document formatting, diagnostics, and semantic tokens for editor integration

Not there yet:

- Full native POSIX grammar coverage inside the Rust runtime
- Richer typed structured values beyond text / JSON helpers
- Broader language features such as HM inference, generics, HKT, modules, `yield`, `?`, and real green-thread scheduling
- Inherent `impl Type { ... }` methods and a Rust-complete type system; the current prototype is still a small subset
- A truly finished shell UX; editing, completion, and IME behavior are still being tuned

Workspace layout:

- `apps/ush`: CLI binary
- `crates/ush_config`: config loading and runtime paths
- `crates/ush_compiler`: `.ush` to `sh` compiler core, `no_std + alloc` capable
- `crates/ush_shell`: interactive shell, parser, stylish I/O, helpers
- `crates/ush_tooling`: formatter, diagnostics, and semantic token generation
- `apps/ush_lsp`: stdio LSP server for editors

## Usage

If you installed `ush` with `install.sh`, the binary is placed in `"$USH_PREFIX/bin"` and `USH_PREFIX` defaults to `~/.local`.
In the default case, make sure `~/.local/bin` is on your `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

For a persistent setup:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

If you installed with a custom prefix, use that prefix instead:

```bash
export PATH="$USH_PREFIX/bin:$PATH"
```

You can confirm the binary is visible with:

```bash
command -v ush
ush -c 'printf "ok\n"'
```

`nix profile install ...` and the Docker image already expose `ush` on `PATH`.

```bash
cargo run -p ush
```

Run a one-liner:

```bash
cargo run -p ush -- -c 'printf "a\nb\n" | len'
```

Enable stylish mode:

```bash
cargo run -p ush -- -s -c 'ls crates'
```

Force stylish mode globally:

```bash
export USH_STYLISH=true
```

Disable interactive confirmations:

```bash
export USH_INTERACTION=false
```

## Interactive Editing

The REPL is tuned around `rustyline`'s Emacs mode with extra bindings for shell-heavy navigation:

- `Up` / `Down`: previous and next history entry
- `Shift-Left` / `Shift-Right`: extend a visible character selection
- `Shift-Up` / `Shift-Down`: behaves like normal history movement when the terminal forwards those keys
- `Option-Up` / `Option-Down`: prefix history search
- `Option-Left` / `Option-Right`: word-wise cursor movement
- `Option-Shift-Left` / `Option-Shift-Right`: extend selection word-by-word
- `Ctrl-Left` / `Ctrl-Right`: word-wise movement on terminals that send control-arrow escapes
- `Ctrl-Alt-Shift-Left` / `Ctrl-Alt-Shift-Right`: extend selection across big shell tokens
- `Home` / `End`: jump to line start/end, and `Shift-Home` / `Shift-End` selects to the edge
- `Cmd-Left` / `Cmd-Right`: works when your terminal maps them to `Home` / `End`

When a selection is active, typing replaces it and `Backspace` / `Delete` removes it, so keyboard-only editing feels closer to a native text field even inside the terminal.

## Structured Helpers

`ush` keeps normal Unix pipes, but helper stages can operate on structured values:

```bash
printf "alpha\nbeta\ngamma\n" | filter(\it -> contains(it, "a")) | len
printf "hello\nworld\n" | map(\it -> upper(it))
printf "hello\n" | map(\line -> { upper(line) })
cat package.json | json | len
cat feed.xml | xml
curl -fsSL https://example.com | html
```

Currently supported helper forms:

- `len`
- `length` (compatibility alias)
- `lines`
- `json`
- `xml`
- `html`
- `map(\it -> upper(it))`
- `map(\it -> lower(it))`
- `map(\it -> trim(it))`
- `map(\it -> replace(it, "from", "to"))`
- `each(\it -> print(it))`
- `filter(\it -> contains(it, "foo"))`
- `filter(\it -> starts_with(it, "foo"))`
- `filter(\it -> ends_with(it, "foo"))`
- `any(\it -> contains(it, "foo"))`
- `some(\it -> contains(it, "foo"))`

`html` writes the current stream into a temporary HTML file and opens it in your default browser.
If `json` cannot parse the stream, `ush` falls back to this browser flow instead of failing immediately.
`xml` pretty-prints valid XML and falls back to the same browser flow if the input is not valid XML.
Helper lambdas also accept `\name -> expr`, `\name -> { expr }`, and zero-arg forms like `\-> { "ok" }`.

## Stylish Mode

The shell-level `-s` / `--stylish` flag swaps some Unix commands into richer output without changing their names:

- `pwd`
- `ls`
- `cat`
- `ps`
- `kill`

When stylish mode is off, `ush` stays close to classic Unix text output.
Compiled `.ush` scripts keep POSIX stdio and run under `/bin/sh`; stylish rendering is only an interactive shell feature.

## Config

Config is resolved from:

- `~/.config/ush/config.pkl`
- `~/.config/ush/config.json`

Example `config.pkl`:

```pkl
{
  shell {
    stylishDefault = true
    interaction = true
    historySize = 10000
    prompt = "ush> "
  }

  aliases {
    ll = "ls -la"
    gs = "git status -sb"
  }
}
```

Because Pkl tooling differs by version, `ush` tries a few `pkl eval` JSON output flag variants before falling back to JSON config.
Legacy `~/.config/ubsh` config files and `UBSHELL_*` env vars are still accepted for compatibility.

## Ush Scripts

`.ush` files are compiled to `sh`; execution still happens in `/bin/sh`.

Supported prototype syntax:

```text
let greeting = "hello"
print greeting + " world"
shell "printf '%s\n' from-ush"
match greeting {
  "hello" => print "matched"
  _ => print "fallback"
}
```

Function calls, task spawning, and `await`-based result retrieval are also supported:

```text
fn worker(message: String) -> String {
  return message
}

print "main"
let task = async worker "worker"
print "after"
let result = task.await
print result
```

Compile explicitly:

```bash
cargo run -p ush -- compile examples/hello.ush
```

Format a script:

```bash
cargo run -p ush -- format examples/hello.ush --stdout
```

Typecheck a script:

```bash
cargo run -p ush -- check examples/hello.ush
```

Run the LSP server over stdio:

```bash
cargo run -p ush_lsp
```

A larger example catalog lives in `examples/README.md`.

`bin.ush` files can act like small CLI tools. The `bin(...)` function becomes the entrypoint, and parameter names drive `--long-flags`, `#[alias("x")]` adds a short flag, and `#[default(...)]` provides default values:

```text
fn bin(#[alias("n")] name: String, #[default(2)] count: Int, verbose: Bool) {
  print name + ":" + count
  print verbose
}
```

Run it like:

```bash
cargo run -p ush -- examples/bin.ush --name ush --count 4 --verbose
```

Struct-like `type` declarations are also available as a prototype:

```text
type User {
  name: String,
  age: Int,
}

let user = User { name: "ush", age: 7 }
match user {
  User { name, age } => print name + ":" + age
  _ => print "fallback"
}
```

Execute directly:

```bash
cargo run -p ush -- examples/hello.ush
```

`#|` doc comments can be attached to the script itself and to top-level `fn`, `enum`, and `trait` declarations.
Those comments are used to auto-generate help, manual text, and completion candidates:

```text
#| Documented ush example.
#| @usage docs.ush --man greet
#|
#| Greet a user and return a message.
#| @param name user name to greet
#| @return greeting text
fn greet(name: String) -> String {
  return "hello " + name
}
```

```bash
cargo run -p ush -- examples/docs.ush --help
cargo run -p ush -- examples/docs.ush --man greet
cargo run -p ush -- examples/docs.ush --complete gr
```

## Install

### curl

```bash
curl -fsSL https://raw.githubusercontent.com/ubugeeei/ubshell/main/install.sh | sh
```

### nix

```bash
nix profile install github:ubugeeei/ubshell
```

### Docker

The Docker image is meant to be a distribution target and base image: a small environment where `ush` is already installed and ready to use.

```bash
docker build -t ush .
docker run --rm -it ush
```

Use it as a base image:

```dockerfile
FROM ush

RUN printf "a\nb\n" | len
CMD ["ush"]
```

## Benchmarks

```bash
cargo bench -p ush_shell
```

The initial benchmark targets parser + helper-pipeline shapes and is intended as a seed for more aggressive profiling.

## CI

GitHub Actions runs formatting, the Rust 250-line file limit check, workspace tests, release tests, bench build checks, and the `.ush` async / ADT examples on every pull request and push to `main`.

## License

MIT
