# Docs

Step-by-step guides for `ush` live here.

Available today:

- [`language-vision.md`](./language-vision.md): the long-term direction for `.ush`
  as a small language, aiming for roughly MoonBit-level expressiveness with a
  feel between Rust, Zig, and Go
- [`lowering.md`](./lowering.md): representative `.ush` to `sh` lowering for major
  language features, runtime scaffolding, Rust-like tail expressions, `bin(...)`,
  and typed error propagation
- [`source-docs.md`](./source-docs.md): how to write std-like `#|` source docs with
  summaries, paragraphs, notes, warnings, errors, examples, and see-also links
- [`sourcemaps.md`](./sourcemaps.md): the sourcemap JSON format, section model,
  reverse line index, and runtime failure mapping output
- [`typed-errors.md`](./typed-errors.md): how `.ush` typed errors, Zig-style `Problem!T`
  signatures, `raise`, inferred `# raises:`, Rust-like `?`, and tail expressions fit together
- [`lsp.md`](./lsp.md): how to wire `ush_lsp` into editors over stdio
- [`release-process.md`](./release-process.md): pre-flight checklist, published
  artefacts, post-flight verification, and rollback procedure for cutting a
  release

For runnable scripts, see [`../examples/README.md`](../examples/README.md).
