# Writing Source Docs with `#|`

This guide shows how to write `.ush` source comments in a more std/rustdoc-like
style.

The idea is simple:

- keep the first line short and summary-like
- use blank `#|` lines to split longer explanations into paragraphs
- add explicit sections such as notes, warnings, errors, and examples
- let `ush` generate `--help`, `--man`, and `--complete` from those comments

## Step 1: Start with a Summary Line

The first prose line becomes the short summary.

```text
#| Greet a user and return a message.
fn greet(name: String) -> String {
  return "hello " + name
}
```

That summary shows up in compact places such as:

- the top line of `--help`
- the `NAME` / item listings in `--man`

## Step 2: Add Paragraphs with Blank `#|` Lines

Use an empty `#|` line when you want a second paragraph.

```text
#| Greet a user and return a message.
#|
#| This function is intentionally tiny, but the docs block is written in a
#| std/rustdoc-like style with a short summary followed by more detail.
fn greet(name: String) -> String {
  return "hello " + name
}
```

The first line stays the summary. The following prose becomes the longer
description shown in `--help` and `--man`.

## Step 3: Document Parameters and Returns

Use `@param` and `@return` for function signatures.

```text
#| Greet a user and return a message.
#| @param name user name to greet
#| @return greeting text
fn greet(name: String) -> String {
  return "hello " + name
}
```

These render into `PARAMETERS` and `RETURNS` sections in `--man greet`.

## Step 4: Add Notes, Warnings, and Errors

For std-like detail, use explicit section tags:

```text
#| Greet a user and return a message.
#| @note Safe for plain user-facing names.
#| @warning Escape the result before embedding it into HTML.
#| @error This function does not raise typed `.ush` errors today.
fn greet(name: String) -> String {
  return "hello " + name
}
```

Available section tags today:

- `@note ...`
- `@warning ...`
- `@error ...`
- `@raises ...`
- `@see ...`
- `@example ...`

`@error` and `@raises` are treated the same way and render into an `ERRORS`
section.

## Step 5: Add Script-Level Usage and Examples

The same `#|` style also works at the top of the script.

```text
#| Documented ush example.
#| @usage docs.ush --man greet
#| @example docs.ush --help
#| @example docs.ush --man greet
```

Useful top-level tags:

- `@usage ...`
- `@note ...`
- `@warning ...`
- `@error ...`
- `@see ...`
- `@example ...`

These show up in script-level `--help` and `--man`.

## Step 6: See What Gets Generated

You can inspect the rendered docs in two ways.

Run the script helpers directly:

```bash
cargo run -p ush -- examples/docs.ush --help
cargo run -p ush -- examples/docs.ush --man greet
cargo run -p ush -- examples/docs.ush --complete gr
```

Or inspect the compiled shell:

```bash
cargo run -p ush -- compile examples/docs.ush
```

That generated `sh` will include `__ush_print_help`, `__ush_print_man`, and
`__ush_complete`.

## Complete Example

```text
#| Documented ush example.
#|
#| Shows generated help, man output, and completion candidates in a more
#| std/rustdoc-like style.
#| @usage docs.ush --man greet
#| @note `--help` stays compact, while `--man` expands the full doc shape.
#| @warning Generated docs are still an evolving language feature.
#| @see docs/README.md
#| @example docs.ush --help

#| Greet a user and return a message.
#|
#| This function is intentionally tiny, but the docs block is written to show
#| how source comments can carry a summary, detail paragraphs, and structured
#| sections similar to standard library docs.
#| @param name user name to greet
#| @return greeting text
#| @note Safe for plain user-facing names and shell-generated strings.
#| @warning Escape the result before embedding it into HTML.
#| @error This function does not raise typed `.ush` errors today.
#| @see Status
#| @example greet "ush"
fn greet(name: String) -> String {
  return "hello " + name
}
```

For a runnable version, see [`../examples/docs.ush`](../examples/docs.ush).
