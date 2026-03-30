# Typed Errors and `?`

This guide walks through `.ush` error handling step by step.

The short version is:

- function signatures expose failures as `Error!T`, in a Zig-like style
- `raise` emits a typed script error
- the compiler infers which errors can flow out of each function
- postfix `?` propagates failures to the current caller
- external shell commands are treated as `unknown` in the inferred error stream

## Step 1: Declare an Error Type

Use an enum-style ADT for your script errors:

```text
enum Problem {
  MissingConfig,
}
```

This gives the compiler a named error variant to track through the script.

## Step 2: Put the Error Set in the Signature

Functions that can fail should say so in the signature:

```text
fn load_config() -> Problem!String {
  raise Problem::MissingConfig
}
```

This is intentionally Zig-like:

- `Problem` is the error set
- `String` is the success value
- `Problem!String` means "returns a `String`, or fails with `Problem`"

If a function returns no meaningful value, use `()`, for example `unknown!()`.

## Step 3: Raise a Typed Error

Use `raise` when a function wants to fail with one of those variants:

```text
fn load_config() -> Problem!String {
  raise Problem::MissingConfig
}
```

At runtime, `raise` writes `ush raise: Problem` to stderr and exits with status `1`.

## Step 4: Let the Compiler Infer the Error Stream

When a function can fail, the compiler records that in the generated `sh` as a comment:

```text
# raises: Problem
ush_fn_load_config() {
  ...
}
```

This inferred stream is built from the functions and shell operations that can fail inside the body.
The compiler also checks that the inferred stream fits inside the declared signature.

## Step 5: Propagate Failures with `?`

Postfix `?` works like Rust's `?`: it stops the current scope and passes the failure upward.

In a value expression:

```text
fn read_message() -> Problem!String {
  let value = load_config()?
  return "<" + value + ">"
}
```

In a call statement:

```text
fn run() -> Problem!() {
  load_config()?
}
```

At the top level:

```text
read_message()?
```

That top-level form exits the script with the propagated status.

## Step 6: Propagate Shell Failures Too

`?` also works for shell statements:

```text
fn check_shell() -> unknown!() {
  $ false?
}
```

and for dynamic shell commands:

```text
fn check_dynamic() -> unknown!() {
  let command = "printf '%s\n' dynamic"
  shell command?
}
```

This propagates the shell exit status upward instead of relying only on `set -e`.
Because external commands are not statically typed, they contribute `unknown`.

## Step 7: Understand `unknown`

External commands cannot be typechecked ahead of time.
Because of that, `$ command ...` and `shell expr` contribute `unknown` to the inferred error stream.

If you combine typed `.ush` failures and external shell failures, both appear:

```text
# raises: Problem | unknown
ush_fn_pipeline() {
  ...
}
```

That is the intended behavior:

- `.ush`-native `raise` values stay typed
- external process failures are tracked, but only as `unknown`

For mixed failures, use a signature like:

```text
fn pipeline() -> (Problem | unknown)!String {
  ...
}
```

## Step 8: Inspect the Generated Shell

You can see the inferred comments and propagation lowering directly:

```bash
cargo run -p ush -- compile examples/error_streams.ush
```

Look for output in this shape:

```text
# raises: Problem
ush_fn_load_config() {
  ...
}

__ush_value_0="$(__ush_capture_return='1' ush_fn_load_config)" || return "$?"
ush_fn_run || exit "$?"
```

The important part is:

- `# raises:` comments preserve the inferred error stream
- `|| return "$?"` is used when `?` propagates from inside a function
- `|| exit "$?"` is used when top-level `?` propagates out of the script

## Step 9: Run the Full Example

The repository includes a complete sample:

```bash
cargo run -p ush -- examples/error_streams.ush
cargo run -p ush -- compile examples/error_streams.ush
```

You can use that file as a starting point for your own scripts.

## Complete Example

```text
enum Problem {
  MissingConfig,
}

fn load_config() -> Problem!String {
  raise Problem::MissingConfig
}

fn wrap(message: String) -> String {
  return "<" + message + ">"
}

fn run() -> Problem!String {
  let value = load_config()?
  return wrap(value)
}

run()?
print "unreachable"
```
