┌─────┬─────────────────────────────────────┬────────────────────────────────────────────────────────┐
│     │                                     │                                                        │
├─────┼─────────────────────────────────────┼────────────────────────────────────────────────────────┤
│ 5   │ member offsets                      │ done                                                   │
├─────┼─────────────────────────────────────┼────────────────────────────────────────────────────────┤
│ 4   │ reject incomplete/void object types │ done                                                   │
├─────┼─────────────────────────────────────┼────────────────────────────────────────────────────────┤
│ 3   │ linkage & definitions               │ steps 1–4 done; step 5 (stubs) left                    │
├─────┼─────────────────────────────────────┼────────────────────────────────────────────────────────┤
│ 2   │ initializers 6.5.7                  │ not started — 5 mismatches, 3 of them false rejections │
├─────┼─────────────────────────────────────┼────────────────────────────────────────────────────────┤
│ 1   │ statement constraints 6.6           │ not started — 10 mismatches                            │
└─────┴─────────────────────────────────────┴────────────────────────────────────────────────────────┘

One correction to the original plan's framing: it treated items 1 and 2 as pure constraint-checking. They aren't. Each is also the pass that produces data codegen consumes — the initializer image and the switch/loop target links. Merging those concerns changes the order and the shape of the work, so the plan below is reorganised around that.

Measured gap (cc1 vs gcc -m32 -std=iso9899:1990 -pedantic-errors …)

Statements — all accepted by cc1, all rejected by gcc:
switch on double · duplicate case · case outside switch · default outside switch · two defaults · break outside loop · continue outside loop · goto to undefined label · if (struct) · while (struct)

Initializers:
- false rejections (valid C90 cc1 refuses): struct S s = {1,2} · union U u = {1} · char s[4] = "abc"
- missed rejections: int x = g; at file scope · static int x = g; in a block · extern int x = 1; in a block

Already correct: duplicate labels, return 1; in void, scalar excess braces, nested array init, brace elision, int a[]={1,2,3} sizing, char a[]="abc" sizing, string-to-non-char-array rejection.

---

Revised plan

Step A — Initializers (6.5.7). Do this first.

Highest priority: three valid programs are rejected today, so struct S s = {1,2} can't reach codegen at all, and there is no initializer representation for codegen to emit.

The controlling text, §6.5.7 Constraints (line 2724):

▎ All the expressions in an initializer for an object that has static storage duration or in an initializer list for an object that has aggregate or union type shall be constant expressions.

Verified against gcc: void f(int n) { struct S s = { n, 1 }; } is a constraint violation in C90 (initializer for aggregate is not a compile-time constant, flagged -Wc99-extensions). This is the single most useful fact for codegen: every aggregate initializer is a compile-time constant image. Only scalar automatics can have a runtime initializer. No aggregate ever needs a store sequence — it's always one LLVM constant.

A1. Model. New model/init.rs. Flattened by byte offset, which is what both @g = global and a scalar store want:

Init = { ty, entries: [(offset, ty, Value | ExpressionId)] }
Aggregates carry only Value (constant); a scalar automatic carries the one ExpressionId. Hang it off Symbol or a HashMap<SymbolId, InitId> on Sema.

A2. Type-driven walker. Replace resolver.rs:263 resolve_initializer. Its Initializer::List arm collapses any non-array to (ty, Some(1)), which is the direct cause of all three false rejections. Pseudo:

walk(ty, cursor):
    if ty is struct/union: targets = members (union: first member only)
    elif ty is array:      targets = elements
    else:                  targets = [ty]           # scalar
    for each target:
        if next initializer is a brace list -> recurse into it, must not overflow
        else -> consume flat items from the cursor (brace elision)
    leftover initializers -> ExcessInitializers      # 6.5.7 "no more initializers than objects"

A3. String literals. char a[4] = "abc" and char a[3] = "abc" are both legal (gcc-confirmed; the NUL is dropped when the array is exactly the character count); char a[2] = "abc" is not. Unsized char a[] = "abc" → 4.

A4. Constant enforcement. Apply the §6.5.7 rule above via the existing ice::eval_constant. Catches int x = g; and static int x = g;.

A5. Block-scope linkage. §6.5.7 line 2726 — extern int x = 1; inside a block is a constraint violation. One-line check in visit_init_declarator: initializer present && linkage != None && scope is Block.

A6. Confirm int a[] = {1,2,3} records length 3 (the exemption added during item 4 only suppressed the diagnostic).

Step B — Statements (6.6) and control-flow context

One pass, two outputs: the constraints, and the enclosing-construct links codegen needs for branch targets.

B1. New constrain/statement.rs.

B2. Give SymbolResolver a context stack:

CfContext = Loop
          | Switch { control_ty, cases: [(value, span)], default: Option<span> }
pushed by visit_iteration_statement / the switch arm of visit_selection_statement.

B3. Checks, each falling straight out of that stack:
- break — stack non-empty · continue — a Loop on the stack
- case/default — innermost Switch exists; default not already set; case value not already present
- switch controlling expression has integral type after promotion (6.6.4.2)
- if/while/do/for-condition controlling expression is scalar (6.6.4.1, 6.6.5)

B4. goto: end-of-function check that every referenced label was defined — the label analogue of finish. add_label_symbol already records both sides via is_init; nothing consumes it yet.

B5. Codegen payload: keep the completed Switch (case values, default, and the statement each targets) keyed by statement id. Without it, codegen re-derives the case list from scratch.

Step C — Retire the stubs (original item 3, step 5)

In constrain/external.rs, all returning Diag::ok and none reachable from src/:
- check_unique_internal_linkage, check_one_external → now enforced by register_external + finish. Delete.
- tentative_defintion_init_zero, no_internal_incomplete_type → now finish. Delete.
- check_typedef → returns true unconditionally, never called. Delete.
- is_tentative_definition → used only by tests; superseded by Symbol::definition_of. Delete and retarget the tests at placements!.

Sequencing

A → B → C. A unblocks the most programs and settles the initializer representation that codegen is built against; B is independent of A and could be parallelised into a worktree if you want two agents; C is a cleanup that only makes sense after A and B stop referencing anything there.

What is not on the list

I checked and these are already in place for codegen: per-expression ResolvedExpression { ty, kind, casts, result_cast }, binding, folded constant, Layout with member offsets and bit offsets, FunctionDef with parameter symbols, and linkage/duration/definition on every symbol. After A and B, the missing pieces are the two representations those steps produce — nothing else in the semantic layer blocks starting the LLVM backend.

✻ Churned for 2m 51s · done 9:46 AM

※ recap: Goal is getting cc1's semantic analysis ready for LLVM codegen. Step 4 of the linkage plan is now fixed and tested (1360 passing), and I've laid out the remaining plan. Next action is Step A: initializers, 6.5.7.
        |

  Ran 2 shell commands

Right — real GCC warns where Apple clang errors. Both conform, and the choice is yours.

§6.7 is a Constraints paragraph, so §5.1.1.3 requires a diagnostic message — it does not require that translation fail. A warning discharges the obligation. GCC picks warning and defers to the linker (undefined reference to 'sf'); clang picks error. Neither is more correct.

Two things about your invocation:omote it.

Recommendation: keep Severity::Error. Not because gcc is wrong, but because of what comes next — codegen. If cc1 only warns, it reaches LLVM IR emission holding a symbol with Linkage::Internal and no body, which is:

declare internal i32 @sf()

The LLVM verifier rejects that outright (a declaration may not have internal linkage). Your options at that point are to emit it as declare i32 @sf(), silently dropping the static, or to drop the call. GCC gets away with warning because its assembler backend turns an undefined local label into an undefined external symbol; you don't have that escape hatch. Erroring in the frontend is the cheapest way to guarantee codegen only ever sees well-formed input.

If you'd rather match gcc exactly, the mechanism is already there — add one arm next to the existing precedent:

Diagnosis::MixedWideStringConcat | Diagnosis::InternalNeverDefined(_) => Severity::Warning,

Note the knock-on: is_error() drives the exit code, so the test would become accept! (or recover!) rather than reject!.

TentativeNeverCompleted is unaffected either way — GCC errors there too (storage size of 'x' isn't known), so it stays Severity::Error.

✻ Baked for 53s · done 9:05 AM

※ recap: Goal: finish C90 semantic analysis in cc1 before codegen. Current task is step 4, the end-of-unit pass in analyzer.rs, with both diagnoses still unreached
  because finish is never called. Next: wire finish into analyze and d's severity. (disable recaps in /config)
nt file (you've since wired it at line 15). Six problems, one of which disables the whole first half.

1. Line 37 — is_object gates out exactly the cases the block exists for

pub fn is_object(&self, sema: &Sema) -> bool {
    self.is_complete(sema) && !self.is_function()
}

is_object requires completeness. An incomplete struct S is not complete; an Array { len: None } is not complete (resolved_type.rs:52). So for struct S; struct S x; and for static int a[]; the condition is false and the block is skipped entirely. TentativeNeverCompleted is unreachable, and no unsized array is ever completed.

You want "identifier for an object" in the §6.7.2 sense — not a function — regardless of completeness:

if !qty.is_function(sema) && ext.defined.is_none() && ext.tentative

2. Line 15 — the Result is dropped

finish(&mut sema); discards the Err, so even once (1) is fixed nothing reaches ctx.diagnosis. This is the sole remaining build warning.

3. False positive: static int sv; int f(void) { return sv; }accepts this program; every static object is a tentative definition by §6.7.2 and ends up defined.

Fixing the mark-defined step fixes this as a side effect, and you need it for codegen anyway. Gating check 2 on SymbolKind::Function would also work but leaves codegen unable to distinguish tentative from defined.

4. Line 44 reads the stale type

Lines 41–42 write the completed array into the symbol, but qty is still the local copy holding the old unsized-array id. Line 44's sema.types.get(qty.id) re-fetches Array { len: None }, so line 46 reports TentativeNeverCompleted on the array it just repaired. Re-read from the symbol, or thread the new QualifiedType through a local.

5. Line 42 drops qualifiers

QualifiedType::new(ty, false, false) — static const int a[]; silently loses its const. Carry qty.is_const, qty.is_volatile.

6. Result + HashMap = one diagnosis, no span, random order

Early return Err stops at the first bad symbol, so a TU with two fanosis carries no Span, unlike every other diagnostic in the codebase — yet ext.tentative and ext.defined are exactly the spans you want. And externals is a HashMap, so which error you report varies between runs; that will make tests flaky.

Shape
 (symbol_id, defined, tentative) from externals   # own the data first
    diags = []
    for (sym, defined, tentative) in entries:
        qty = sym.ty or skip
        if not qty.is_function and defined is None and tentative is Some(span):
            if qty is Array{len: None}:
                qty = array(elem, 1) keeping qty's qualifiers
                sym.ty = qty
            if not qty.is_complete:
                diags.push(TentativeNeverCompleted(qty) at span)
                continue
            sym.definition = Definition                 # implicit all-zero initialiser
            externals[name].defined = Some(span)
        if sym.used and sym not defined and sym.linkage == Internal:
            diags.push(InternalNeverDefined(sym.name) at sym's decl span)
    sort diags by span
    append to sema.diagnosis

Two mechanical notes on that snapshot: it is what lets you call sema.add_diag inside the loop. The current code only compiles because sema.externals, sema.types and sema.symbols are disjoint fields — the moment you call a &mut self method on Sema, the borrow of externals.values() conflicts.

Also, InternalNeverDefined needs a declaration span, which External doesn't currently store — it has defined and tentative, and a pure declaration sets neither. You'll want a third declared: Span on External, set on the first-insert path in register_external.


  Searched for 1 pattern, ran 1 shell command

External is already Clone and Span is Copy, so snapshot the whole entry — it survives you adding a declared field later without touching this line.

let mut entries: Vec<(StringId, External)> = sema
    .externals
    .iter()
    .map(|(&name, ext)| (name, ext.clone()))
    .collect();

Imports: StringId from crate::ast, External from crate::semantic.

For the determinism half, neither Span nor Position derives Ord, so key on the fields:

entries.sort_by_key(|(_, ext)| {
    let p = ext.tentative.or(ext.defined).unwrap_or_default().start;
    (p.file, p.line, p.col)
});

Then the loop reads for (name, ext) in entries, and the write-back is sema.externals.get_mut(&name).unwrap().defined = Some(span); with sema fully free for add_diag, types.array, and symbols.get_mut.

One gap: an entry that is a pure declaration has neither span set, so unwrap_or_default() collapses all of them to (0,0,0). sort_by_key is stable, so those keep HashMap order — still nondeterministic among themselves. That's precisely the set InternalNeverDefined fires on. Adding declared: Span to External and keying the sort on it makes the order total; until then, expect the relative order of two undefined static functions to vary between runs.

✻ Cooked for 24s · done 9:19 AM
───────────────────┼──────────┼────────────────────────────────┤
│ struct S; static struct S x;              │ error    │ ✅ error                       │
├───────────────────────────────────────────┼──────────┼────────────────────────────────┤
│ …; static struct S x; struct S{int i;};   │ accept   │ ✅ accept                      │
├───────────────────────────────────────────┼──────────┼────────────────────────────────┤
│ static int sf(void); …sf();               │ warn     │ ✅ warn                        │
├───────────────────────────────────────────┼──────────┼────────────────────────────────┤
│ static int sf(void); …sizeof(int);        │ accept   │ ✅ accept                      │
├───────────────────────────────────────────┼──────────┼────────────────────────────────┤
│ static int sf(void){return 1;} …sf();     │ accept   │ ✅ accept                      │
├───────────────────────────────────────────┼──────────┼────────────────────────────────┤
│ static int sv; int f(void){return sv;}    │ accept   │ ❌ 'sv' used but never defined │
├───────────────────────────────────────────┼──────────┼────────────────────────────────┤
│ static int a[]; int f(void){return a[0];} │ accept   │ ❌ 'a' used but never defined  │
└───────────────────────────────────────────┴──────────┴────────────────────────────────┘

Both failures are the same line. Smallest fix — carry it in a local

let mut defined = ext.defined;

set it alongside the map write,

defined = Some(span);
if let Some(e) = sema.externals.get_mut(&i) {


and test that instead of the snapshot:

if sym.used && defined.is_none() && sym.linkage == Linkage::Internal {

Two cosmetic leftovers: line 55's let Some(qty) = sym.ty else { continue } is unreachable (line 43 already proved it Some, and line 52 only writes Some), and its continue would wrongly skip check 2 if it ever did fire; line 74's continue is the last statement of the loop body. The commented-out lines 61 and 66–68 and the trailing block at 79–86 are now redundant with the code.

Once that's in, static int sf(void); + a defined static object is the only shape reaching check 2 — which matches the claim that InternalNeverDefined is function-only.

✻ Churned for 57s · done 9:30 AM

※ recap: Goal is finishing step 4 of the linkage work in cc1's semantic analyzer. The finish function is nearly correct, but check 2 reads a stale snapshot, so two valid programs get false warnings. Next: carry defined in a local and test that instead.

❯ implement the fix

  Read 1 file

⏺ Update(src/semantic/analyzer.rs)
Added 8 lines, removed 19 lines
ty) = sema.symbols.get(sym_id).ty else { continue };
        let Some(mut qty) = sema.symbols.get(sym_id).ty else { continue };
        let mut defined = ext.defined;
        if !qty.is_function(sema)
            && ext.defined.is_none()
            && defined.is_none()
            && let Some(span) = ext.tentative
        {
            let ty = sema.types.get(qty.id);
            if let ResolvedType::Array { elem, len: None } = ty {
                let ty = sema.types.array(*elem, Some(1));
                let t = QualifiedType::new(ty, qty.is_const, qty.is_volatile);
                sema.symbols.get_mut(sym_id).ty = Some(t);
                qty = QualifiedType::new(ty, qty.is_const, qty.is_volatile);
                sema.symbols.get_mut(sym_id).ty = Some(qty);
            }
            let sym = sema.symbols.get(sym_id);
            let Some(qty) = sym.ty else { continue };
an 7 shell commands

❯ add tests

  Ran 3 shell commands

Now the test edits.

⏺ Update(tests/test_resolution.rs)
Added 9 lines, removed 2 lines
    "a",
    Ty::arr(Ty::arr(Ty::arr(Ty::Int, 7), 5), 3)
);
tree!(describe_incomplete_array, "int a[];", "a", Ty::flex(Ty::Int));
tree!(describe_incomplete_array, "extern int a[];", "a", Ty::flex(Ty::Int));
tree!(
    describe_incomplete_array_of_array,
    "int a[][5];",
    "extern int a[][5];",
    "a",
    Ty::flex(Ty::arr(Ty::Int, 5))
);
tree!(describe_tentative_array_completed, "int a[];", "a", Ty::arr(Ty::Int, 1));
tree!(
    describe_tentative_array_of_array_completed,
    "int a[][5];",
ay_of_pointer,
    "int *a[3];",
    "a",

⏺ Update(tests/test_linkage.rs)
Added 6 lines, removed 2 lines
use crate::common::accepted;

placements!(placement_tentative_external, "int x;", &[("x", "external", "static", "tentative")]);
placements!(
    placement_tentative_external,
    "int x;",
    &[("x", "external", "static", "definition")]
);

placements!(
    placement_tentative_internal,
    "static int x;",
    &[("x", "internal", "static", "tentative")]
    &[("x", "internal", "static", "definition")]
);

placements!(

commands

⏺ Update(tests/test_linkage.rs)
Added 3 lines, removed 3 lines
    &[]
);
recover!(
    internal_array_never_defined,
    "struct S; static struct S a[3]; int use(void) { return sizeof a; }",
    [Diagnosis::SizeofIncomplete(_)],
    end_of_unit_diagnosis_order,
    "static int sf(void); struct S; static struct S x; int use(void) { return sf(); }",
    [Diagnosis::InternalNeverDefined(_), Diagnosis::TentativeNeverCompleted(_)],
    &[]
);

  Ran 3 shell commands

⏺ Update(tests/test_linkage.rs)
Added 1 line, removed 1 line
accept!(external_function_never_defined, "int ef(void); int use(void) { return ef(); }");
accept!(external_incomplete_array_declaration, "extern int a[]; int *p(void) { return a; }");

size!(tentative_array_completes_to_one_element, "int a[];"
e_array_sizeof_before_completion, "int a[]; enum e { P = sizeof(a) };");

recover!(
    internal_function_never_defined,

  Ran 2 shell commands

Fixed and verified. 1360 tests pass, build clean, zero warnings.

The fix (src/semantic/analyzer.rs)

ext is a pre-loop clone, so writing sema.externals didn't update it. Now a local carries the state across both checks:

let Some(mut qty) = sema.symbols.get(sym_id).ty else { continue };
let mut defined = ext.defined;

The array completion assigns back into qty instead of a throwaway t, which removed the stale re-read and the dead let Some(qty) = sym.ty else { continue }. On
the success path:

sema.symbols.get_mut(sym_id).definition = Definition::Definition;
defined = Some(span);
if let Some(e) = sema.externals.get_mut(&i) { e.defined = Some(span); }

and check 2 tests defined.is_none(). Both false positives are gone;match gcc.

Four tests were already failing before this fix

They're consequences of finish being wired at all, not of the bug. Both changes are norm-driven, but flag them if you disagree:

- test_resolution.rs — tree!(describe_incomplete_array, "int a[];", …, Ty::flex(Ty::Int)) asserted the type stays unsized, but §6.7.2 completes it to one element at end of unit. Retargeted the two printer tests at extern int a[]; / extern int a[][5];, which stay declarations and stay incomplete, and added describe_tentative_array_completed + the array-of-array variant asserting int[1][5].
- test_linkage.rs — placement_tentative_external / _internal expected "tentative". After the end-of-unit pass, int x; and static int x; are definitions. Expectations updated to "definition". Note this makes Definition::Tentative unobservable through placements! — it only survives on the error path now.

Tests added (16)

test_linkage.rs: three reject! for TentativeNeverCompleted (external, internal, and one where the object is also used), plus accepts for the completed-later tag, the completed array, the two former false positives (static int sv; used, static int a[]; used), a defined-then-used static function, an unused static function, the §6.7 sizeof exemption (sizeof(&sf)), and the two external-linkage cases that are the linker's problem. Two recover!: the warning itself, and a two-diagnosis case pinning source order.

test_resolution.rs: two tree! for the completion.

Every accept/reject was run through gcc -m32 -std=iso9899:1990 -pednotes from that:

- tentative_array_sizeof_before_completion replaced a size! test I'd written wrong. int a[]; enum e { P = sizeof(a) }; is a constraint violation — completion happens at end of unit, sizeof is folded before it, so the type is genuinely incomplete at that point. gcc agrees: invalid application of 'sizeof' to an incomplete type 'int[]'. cc1 already got this right.
- On tentative_completed_later and tentative_array_completed, clang errors under -Wtentative-definition-incomplete-type. That is the C99 6.9.2p3 constraint, which C90's §6.7.2 does not contain — C90 has no Constraints section there at all. The tests encode the C90 behaviour deliberately.

make a plan for what is left to do on the orgininal plan (update this orginal plan if necessary, goal is to be ready for starting llvm code generation)

  Searched for 3 patterns, read 2 files, listed 2 directories, ran 8 shell commands

Surveyed the tree and probed cc1 against gcc on 30 cases. Here's where the original plan stands and what's actually left.
