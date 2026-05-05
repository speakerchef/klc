# Deferred error tests

## scope visibility: child scope leak into parent
- [ ] Fix in Sema by adding scopes. Currently global cached_ty HashMap
- [ ] errors/scope.knv should fail; Currently compiles as valid
Fix sketch: replace `cached_ty: HashMap` with a stack of HashMaps and
push/pop in `visit_scope` / `visit_stmt_fn`.

## type annotation mismatch: non-convertible declared vs inferred
- [ ] Add failure mode checks for `is_digit_convertible_to()` once stable
