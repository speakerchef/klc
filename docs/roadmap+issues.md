

> IN PROGRESS

> ISSUES
- [ ] Parenthesized expressions with `sub` & `add` op `(1 + 2) - 4` fail
- [ ] Infinite loop when function definition args are in reverse pair order (i.e. \<T>: \<name>)
- [ ] panics when no return type is given to functions; issue with default-void setting logic
- [ ] panics when void return function is used in expressions at sema

>TODOS
- [ ] check `return` type against func declared return type
- [ ] Allow expressions in function calls
- [ ] No unary negation
- [ ] No comment syntax
- [ ] Allow for more than 8 function arguments; store on stack.
- [ ] check caller passed argument types against function definition types
- [ ] add errors for too little arguments, too many arguments, expected arguments, unexpected arguments, in sema
- [ ] Generate default behavior and expect main to be defined for program to start
- [ ] Impl some version of a global scope (rn function-based scope)
- [x] Recursive functions result in segfault
- [x] Impl `return` and default-emit `ret` after every function; values are optionally provided
- [x] figure out using functions in expressions (if not void, load values from [x | w]0)
- [x] emit `ret` by default even if no return stmt exists
- [x] Emit prologue and epilogue for every function
- [x] Emit epilogue before function return
- [x] Allow fwd decls and assign missing values at discovery
- [x] Refactor to function-like state where main is a function and not implicit. i.e. codegenerator structs for each individual function.


