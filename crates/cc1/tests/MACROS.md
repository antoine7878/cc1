# Test macros

Every macro expands to one `#[test] fn <name>`. Shared macros live in `common/macros.rs`.
"Clean" means: parses, no diagnostics, `missing_facts()` empty.

## Accept / reject

- `accept!(name, src)`: compiles clean.
- `reject!(name, src)`: `!unit.accepts()` (at least one error, any kind).
- `syntax!([ignore "why",] name, src)`: parses only, no semantic check.
- `facts!(name, src)`: compiles clean; same check as `accept!`, used in `test_facts`.
- `recover!(name, src, [Diag::A, Diag::B(_)], &["x"])`: diagnostics match the patterns exactly and in order; no diagnostic names an identifier from the last list (cascade check).
- `reports!([ignore "why",] name, src, ["<test>:1:16: error: ..."])`: rendered messages equal the list.

## Execution (checked against gcc)

- `exits!(name, src, 42)`: compiles clean; the exit status of both the cc1 IR and gcc's build equals the value. gcc results are cached in `target/gcc-oracle`.
  - `exits!(ignore "why", name, src, n)`: ignored test.
  - `exits!(warns name, src, n)`: warnings allowed, no errors.
- `exits_linked!(name, src, helper, n)`: `src` compiled by cc1, `helper` by gcc, linked and run; tests the ABI across the gcc boundary.
- `emits!(name, src, "needle")`: IR contains the needle.
  - `emits!(not name, ...)`: IR does not contain it.
  - `emits!(warns name, ...)`, `emits!(warns not name, ...)`: same, warnings allowed.

## Constants and folding

- `value!(name, src, &[("A", "3")])`: compiles clean; each enumerator has the value.
- `folds!([ignore "why",] name, src, ["Int(1)", ...])`: parses; folded constant-expression contexts (enumerators, array sizes, `sizeof`).
- `folded!(name, src, ["Int(42)", ...])`: accepted; fold result of every non-literal expression.
- `constant!(name, "0x10", "Int(16)")`: `ConstValue::parse` gives the value, no diagnostic.
- `too_large!(name, lit, repr)`: same, diagnostic `IntegerConstantTooLarge`.
- `escape_out_of_range!(name, lit, repr)`: same, diagnostic `EscapeOutOfRange`.
- `escape_invalid!(name, lit, repr, Diag::X)`: same, diagnostic matches the pattern.

## Layout

- `size!(name, decl, "struct S", 12)`: after `decl`, `sizeof(ty)` equals the value (appends `enum { PROBE = sizeof(ty) }`).
- `bits!(name, decl, "S", &[("m0", 0), ...])`: member bit offsets of tag `S`.
- `offsets!(name, decl, "S", &[("a", 0, 0), ...])`: (member, byte offset, bit) tuples of tag `S`.

## Semantic side tables

- `uses!(name, src, &[("x", true), ...])`: used flag of each symbol, in declaration order.
- `inits!(name, src, &[("s", "...")])`: rendered initializer of each symbol.
- `member_refs!(name, src, &[("m", "struct S", 0)])`: member reference resolves to that type and member index.
- `placements!(name, src, &[(name, linkage, duration, ...)])`: linkage and storage placement of each symbol.
- `stmts!(name, src, &["..."])`: rendered statement list.
- `labels!(name, src, &[("f", &["L1"])])`: labels of each function.
- `tree!(name, src, "sym", "...")`: accepted; type tree of `sym` renders as given.

## Expression shapes (value category and implicit conversions)

- `shaped!([ignore "why",] name, src, vec![lv(Ty::Int).then(LValueToRValue, Ty::Int), rv(..)])`: no errors; expression shapes match exactly.
- `rejects_shaped!(name, src, Diag::X, vec![...])`: exactly one diagnostic matching the pattern; shapes still match.
  - Guarded form: `rejects_shaped!(name, src, |u| Diag::X(n) if cond, vec![...])`.

## Strings (lexer/parser only)

- `literal!([ignore "why",] name, src, "value")`: the single string literal has this decoded value.
- `pool!(name, src, &["a", "b"])`: string pool contents.

## Local to one test file

- `unnamed_size!` (`test_layout`): same as `size!`.
- `renders!(name, src, "sym", "desc")` (`test_resolution`): `unit.describe(sym)` equals the description.
- `op!(name, src, "Addr")` (`test_operator`): parses; expressions include the label.
- `fold!(name, method(args..), "repr")` (`test_value`): calls `ConstFolder.method(args)` directly.
- `fold_overflow!(name, method(args..), "repr")` (`test_value`): same, diagnostic `ArithmeticOverflow`.

## Base

- `test_case!([ignore "why",] name, { body })`: plain `#[test] fn`; all shared macros are built on it.
