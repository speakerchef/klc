# Knobc: Knob-Compiler
>(This is a Rust re-design & re-write of my original; [Original C++ Version](https://github.com/speakerchef/klc-compiler))
>Re-write also includes and will include a great deal of architectural changes both internally and language-definition wise.

- Knobc is a fun learning project of mine that is a compiler for the **KNOB** (**K**ompiled **NOB**) programming language that I'm creating.
- `Knob` is a statically typed, AOT-compiled language I'm creating. Its syntax is a soup of features I thought were cool from other languages plus some of my personal touches. Knobc emits a custom IR (Intermediate Representation) called `klir` ([SPEC](./KLIR-schema/SPEC.md)) - emitting assembly for a few backends: Namely AArch64 (Apple Silicon & Linux) & x86_64(eventually).

Uses `.knv` as the file extension.
> *Why `.knv` and not `.knb`?*
> Everyone knows the true perfect language prioritizes ergonomics over sensible standards. `v` is easier to hit from the home row than `b`. You're welcome.
---
## Architecture

```
Source → Lexer → Parser → AST → Type-Checking / Semantic Analysis → Typed-AST → MIR → Optimization Pass(es) → Backend → Assembly Codegen → Link Runtime → Executable
```

- **Lexer/Tokenizer**: tokenizes `.knv` source into a stream of typed tokens
- **Parser**: Pratt-Parsing with precedence climbing for expressions and Recursive-Descent parsing for the rest, producing an untyped-AST.
- **Type-Checking** and **Semantic Analysis** that resolves types and mutates the untyped-AST into a typed-AST. Semantic errors are also evaluated here.
- **Diagnostics Handler**: This stage accumulates all errors, warnings, and notes from the above stages and if error-free, proceeds to the below stages.
- **KLIR-Generation**: Typed-AST is walked and KLIR is emitted for each node/operation/etc...
- **Optimization (Later scope)**: Will analyze the IR for patterns to exploit and optimize
- **Codegen** — Currently targeting only AArch64 (Apple Silicon / macOS Darwin ABI). (x86_64 support in the future). No LLVM or other backends/deps.
---
## Language features

### Types (so far)
>[!NOTE] 
> Full type suite is not currently implemented.

| Class | Variants |
|---------|-------------|
| `Integers`   | `u8/i8`, `u16/i16`, `u32/i32`, `u64/i64`, `usize` (semantic alias to `u64/u32`)|
| `Characters`   | `char` (aliased to `u8`)|
| `Floating Point`  | `f32`, `f64`|
| `Strings`    | `string` - likely aliased to a `u8` array of valid UTF-8 (hello rust XD)|
| `Boolean`  | `bool` w/ opts `true` & `false`|

### Keywords (so far)

| Keyword | What it does |
|---------|-------------|
| `let`   | Const-defaulted variable declaration |
| `mut`   | Mutable variable declaration |
| `exit`  | Exit with an exit code |
| `if`    | Conditional branch |
| `elif`  | Alternate branch |
| `else`  | Fallback branch |
| `while` | Loop while condition true |
| `fn` | Function declaration |
| `return` | return to caller from callee |

### Operators & Symbols (so far)

| Category | Operators | Notes |
|----------|-----------|-------|
| Arithmetic | `+` `-` `*` `/` `%`| Standard integer arithmetic (fp later)|
| Power | `**` | Right associative, eg. 2**3 = 8 |
| Comparison | `==` `!=` `<` `>` `<=` `>=` | 1 or 0 |
| Logical | `&&` `\|\|` | Boolean logic on truthy/falsy values |
| Bitwise | `&` `\|` `^` | AND, OR, XOR |
| Bit Shift | `<<` `>>` | LSL, LSR |
| Unary | `-` | negation |
| General | `->` | Denotes an explicit return type for function definitions |
| Comments | `//` `/* */` | Line comments and Block comments |
| Operate-Assign | `+=` `-=` `*=` `/=` `%=` `**=` `&=` `\|=` `<<=` `>>=` | Combine operation and assignment |

---
## Build & Generate Executable

> **Requires:** Cargo, Clang/GCC for linker, AArch64 target (Apple Silicon Mac)

```bash
# Build the compiler
cargo build
cd target/release

# Optional: Alias to use `knob` anywhere on your system
alias knob='path/to/knobc'

# Usages
# Build
[ knob | ./knobc ] build <FILE.knv> <EXEC-NAME> 
# Run
[ knob | ./knobc ] run <FILE.knv>
```

---

## Long-term Roadmap
- [ ] String literals
- [ ] Floating point support (Harder than you think)
- [ ] Standard library functions like print()
- [ ] Loop optimizations
- [ ] Register allocation pass
- [ ] x86_64 generation
- [ ] ...and many more!
