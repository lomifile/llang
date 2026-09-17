# jsl

A small statically-typed programming language implemented in Rust: source is
lexed, parsed, type-checked, compiled to bytecode, and run on a stack VM.

## Build

```sh
cargo build --release
```

## Usage

Run a file:

```sh
cargo run -- test/test.jsl
```

Start the REPL (no arguments):

```sh
cargo run
```

## Example

```
let num1: Number = 1;
let num2: Number = 2;

function add(a: Number, b: Number): Number {
  return a + b;
}
```

Types: `Number`, `String`, `Boolean`, `Void`, `Null`.

## Layout

The workspace is a pipeline, one crate per stage:

| Crate      | Role                           |
| ---------- | ------------------------------ |
| `token`    | token kinds and spans          |
| `lexer`    | source text to tokens          |
| `ast`      | expression and statement nodes |
| `parser`   | tokens to AST                  |
| `types`    | type checker and environment   |
| `chunk`    | bytecode chunks and values     |
| `compiler` | AST to bytecode                |
| `vm`       | stack-based interpreter        |

`src/main.rs` is the driver that wires them together.

## Tests

```sh
cargo test
```
