# Sourcemaps

`ush compile --sourcemap` writes a JSON sidecar that explains how generated
`/bin/sh` lines relate back to `.ush` source.

This is useful for:

- debugging lowering output
- understanding runtime failures in generated shell
- building editor and tooling integrations on top of `ush`

## Generate a sourcemap

```bash
cargo run -p ush -- compile examples/hello.ush -o /tmp/hello.sh --sourcemap /tmp/hello.sh.map.json
```

The generated shell can still run on its own. The sourcemap file is extra
metadata for humans and tools.

## JSON format

Current sourcemaps use `version: 2`.

Top-level fields:

- `source`: original `.ush` file path
- `generated`: output shell path when `-o` is used
- `summary`: counts for generated, mapped, unmapped, and per-section lines
- `sources`: reverse index from one source line to every generated shell line
- `lines`: per-generated-line mapping entries

Each line entry includes:

- `generated_line`
- `section`
- `source_line`
- `generated_text`
- `source_text`

Current sections are:

- `runtime-support`: emitted helpers and runtime scaffolding
- `doc-support`: generated help/man/completion support
- `user-code`: lowered code that came from the user program

Example:

```json
{
  "version": 2,
  "summary": {
    "mapped_line_count": 2,
    "source_line_count": 2
  },
  "sources": [
    {
      "source_line": 1,
      "source_text": "let greeting = \"hello\"",
      "generated_lines": [449]
    }
  ],
  "lines": [
    {
      "generated_line": 449,
      "section": "user-code",
      "source_line": 1,
      "generated_text": "greeting='hello'",
      "source_text": "let greeting = \"hello\""
    }
  ]
}
```

## Runtime failure mapping

When `.ush` scripts run through `ush`, the generated shell is instrumented so
runtime failures print a sourcemap report to `stderr`.

Example:

```text
ush runtime map: script.ush:10
  section: user-code
  shell  : G0456 | printf '%s\n' "$(printf '%s' "${greeting}" '!')"
  source : print greeting + "!"
  mapped : G0456
```

For source lines that lower into multiple shell lines, `mapped` shows the whole
generated group, not just the failing line. That makes control-flow lowering
much easier to inspect.

The JSON sourcemap and mapped listings still include `runtime-support` and
`doc-support` sections, so tooling can inspect generated support code even when
runtime diagnostics focus on user-originated lines.
