- [ ] Parenthesized expressions with `sub` & `add` op `(1 + 2) - 4` fail
- [ ] No unary negation
- [ ] No working comments


- [x] Impl `return` and default-emit `ret` after every function; values are optionally provided
- [x] figure out using functions in expressions (if not void, load values from x/w0)

- [ ] check `return` type against func declared return type
- [ ] Emit prologue and epilogue for every function 
- [ ] Infinite loop when function definition args are in reverse pair order (i.e. \<T>: \<name>)
- [ ] panics when no return type is given to functions; issue with default-void setting logic
- [ ] panics when void return function is used in expressions at sema

- [x] Allow fwd decls and assign missing values at discovery
- [x] Refactor to function-like state where main is a function and not implicit. i.e. codegenerator structs for each individual function.
- [ ] Generate default behavior and expect main to be defined for program to start
- [ ] Impl some version of a global scope (rn function-based scope)
