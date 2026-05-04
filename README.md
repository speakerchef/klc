# Knobc: Knob-Compiler
>(This is a Rust re-design & re-write of my original; [Original C++ Version](https://github.com/speakerchef/klc-compiler))
>Re-write also includes and will include a great deal of architectural changes both internally and language-definition wise.

- Knobc is a compiler for the **KNOB** (**K**ompiled **NOB**) programming language —
`Knob` is a statically typed, AOT-compiled language I'm creating that emits a custom defined IR (Intermediate Representation) called `klir` - emitting assembly for a few backends: Namely AArch64 (Apple Silicon & Linux) & x86_64(eventually):
- As of now, only Apple Silicon Arm64 assembly is emitted. Linux Arm64 will be implemented as the compiler matures.

> [!NOTE]
> The original C++ version of the compiler emitted raw AArch64 assembly. The new re-write with an MIR level will allow for optimization passes, multiple backends, and other cool stuff!

Uses `.knv` as the file extension.
> *Why `.knv` and not `.knb`?*
> Everyone knows the true perfect language prioritizes ergonomics over sensible standards. `v` is easier to hit from the home row than `b`. You're welcome.
---
## Architecture

```
Source → Lexer → Parser → AST → Type-Checking / Semantic Analysis → Typed-AST → MIR → Optimization Pass(es) → Backend → Assembly Codegen → Link Runtime → Executable
```

- **Lexer/Tokenizer** — tokenizes `.knv` source into a stream of typed tokens
- **Parser** — Pratt-Parsing with precedence climbing for expressions and Recursive-Descent parsing for the rest, producing an untyped-AST.
- Type-Checking and Semantic Analysis that resolves types and mutates the untyped-AST into a typed-AST. Semantic errors are also evaluated here.
- Typed-AST is walked and Knob-MIR is emitted for each node/operation/etc...
- Optimization (Later scope): Will analyze the IR for patterns to exploit and optimize
- **Codegen** — Currently targeting only AArch64 (Apple Silicon / macOS Darwin ABI). (x86_64 support in the future). No LLVM or other backends/deps.
---
## Language features
>KNOB is a fun project of mine still under active construction.

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

### Operators (so far)

| Category | Operators | Notes |
|----------|-----------|-------|
| Arithmetic | `+` `-` `*` `/` `%`| Standard integer arithmetic (fp later)|
| Power | `**` | Right associative, eg. 2**3 = 8 |
| Comparison | `==` `!=` `<` `>` `<=` `>=` | 1 or 0 |
| Logical | `&&` `\|\|` | Boolean logic on truthy/falsy values |
| Bitwise | `&` `\|` `^` | AND, OR, XOR |
| Bit Shift | `<<` `>>` | LSL, LSR |
| Unary | `-` | negation |
| Operate-Assign | `+=` `-=` `*=` `/=` `%=` `**=` `&=` `\|=` `<<=` `>>=` | Combine operation and assignment |


>[!WARNING] 
> The below is stale from the original C++ codebase - will update as things move

### Other Features
- Parenthesized expressions with correct grouping: `(a + b) * (c - d)`
- Scoping with nested blocks and proper variable resolution
- Variables from outer scopes are visible in inner scopes
- Local variables are inaccessible outside their scope
- Nested if/elif/else with arbitrary depth
- While loops with mutable state
- Function declarations and calls with typed arguments
- Calls useable inside expressions and as arguments to other calls

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

### Execute

```bash
./executable

# Or benchmark
time ./executable

# Check exit code
echo $?
```

---

## Roadmap

- [ ] String literals
- [ ] Functions
- [ ] Floating point support (Harder than you think)
- [ ] Standard library functions like print()
- [ ] Loop optimizations
- [ ] Register allocation pass
- [ ] x86_64 generation
- [ ] ...and many more!
