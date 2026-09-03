Plan — item 3: linkage, storage duration, and definitions

The shape of the problem

Symbol carries storage: Option<Storage> (the keyword as written) and is_init (doubling as "is a definition").
Neither linkage nor storage duration exists, and both are derived properties the keyword alone doesn't determine:
extern int x; is external or internal depending on what came before it, and a file-scope object with no keyword at all has external linkage and static duration.

The second gap is that every check in 6.7 is translation-unit-wide, not lexical.
void f(void){ extern int x; } followed by static int x; is an error, but the two declarations live in different scopes and never meet in the scope chain.
So the core of this work is a TU-wide table keyed by identifier, sitting alongside Scopes.

Step 1 — the derived properties

Add to Symbol:
linkage (External | Internal | None),
duration (Static | Automatic),
and replace the is_init overload with an explicit definition: Definition (Declaration | Tentative | Definition).
Compute them in the resolver where scope, storage and initializer are all in hand.

linkage_of(scope, storage, kind):
    typedef, parameter, label, member          -> None
    block scope, object without extern         -> None
    storage is static at file scope            -> Internal
    storage is extern (any scope)              -> the linkage of the visible file-scope
                                                  declaration of that name, else External
    everything else (file-scope object, any function) -> External

duration_of(scope, storage):
    file scope, or storage is static or extern -> Static
    otherwise                                  -> Automatic

definition_of(scope, storage, has_initializer, has_body):
    function     -> Definition if it has a body, else Declaration
    initializer  -> Definition
    file scope, storage none or static -> Tentative
    block scope, storage not extern    -> Definition
    otherwise    -> Declaration

Land this step with no new diagnostics and add the three columns to Context::dump_symbols — that gives you an eyeball check on the whole model before anything depends on it.

Step 2 — the translation-unit table

Sema.externals : map StringId -> Entry
Entry: linkage, symbol, kind(Object|Function),
       defined_at: Option<Span>, tentative_at: Option<Span>, used: bool

Register from the resolver every time a declaration with linkage is declared — file-scope declarations and block-scope extern, which is what makes the l4 case reachable:

register(name, linkage, declaration):
    if name is new -> insert and return
    e = entry
    if e.linkage != linkage:
        error ConflictingLinkage(name)     # keep e.linkage and carry on
    if types are not compatible -> error DuplicateDeclaration   # exists today
    else e.symbol.type = composite(e.symbol.type, declaration.type)
    if declaration is a Definition:
        if e.defined_at is set -> error DuplicateDeclaration
        e.defined_at = span
    if declaration is Tentative and e.tentative_at is unset:
        e.tentative_at = span

Note the asymmetry the standard requires and gcc enforces: static int b; extern int b; is legal (the later extern inherits internal linkage), while extern int a; static int a; is an error. The rule falls out of computing extern's linkage from the prior declaration before comparing.

Sema::dedup keeps doing lexical same-scope redeclaration; the cross-scope and whole-unit questions move here. Layer the table on top first and only then delete the overlap in dedup, so the 1310 existing tests keep telling you something.

Step 3 — use marking

6.7 exempts the operand of sizeof, so this cannot be a flag set during resolution — it has to be a walk that refuses to descend into one node kind:

mark_uses():
    walk all expressions, but do not descend into the operand of SizeofExpr
    at an identifier expression:
        sym = sema.binding(expression id)
        if sym has linkage -> externals[sym.name].used = true

sizeof(type-name) has no identifier operand, so only SizeofExpr needs the guard. visit.rs:289 currently lumps SizeofExpr in with Unary and ConstantExpression — that arm has to be split.

Step 4 — the end-of-unit pass

A third step in Analyzer::analyze, after resolve_names and check_constants:

finish_unit():
    for each entry:
        if Object and defined_at is unset and tentative_at is set:
            if type is an array of unknown size -> complete it with one element
            if type is still incomplete -> error TentativeNeverCompleted(name, type)
            else mark defined, with an implicit all-zero initialiser
        if used and defined_at is unset and linkage is Internal:
            error InternalNeverDefined(kind, name)

6.7.2 line 3193 for the collapse; 6.7 line 3124 for the "exactly one external definition" rule.

Worth knowing before you write it: the second check only ever fires for functions. A static int x; is itself a tentative definition, so an internal-linkage object cannot be declared-but-undefined. That also means no_internal_incomplete_type (6.7.2 line 3195) is fully subsumed by the tentative branch above — it is not a separate check.

Step 5 — the five stubs

- check_unique_internal_linkage → delete; step 2's defined_at check is the rule, and it applies to external linkage too.
- check_one_external → becomes the InternalNeverDefined branch of step 4.
- no_internal_incomplete_type → delete, subsumed as above.
- tentative_defintion_init_zero → not a constraint; becomes the tentative-collapse action in step 4 (and it is codegen's cue to emit into .bss).
- check_typedef (6.7.1, "an identifier declared as a typedef name shall not be redeclared as a parameter") → unrelated to linkage; call it from param_old_style. Be aware the parser already diverges here: int g(T) int T; {} is parsed as a prototype, and cc1 reports ParameterTypeListWithList where gcc reports the typedef clash.

is_tentative_definition is the one stub that survives unchanged — give it the caller it has never had.

Diagnoses to add

- ConflictingLinkage(Name) — "static declaration of 'x' follows non-static declaration"
- TentativeNeverCompleted(Name, QualifiedType) — "tentative definition has type 'struct S' that is never completed"
- InternalNeverDefined(SymbolKind, Name) — "function 'sf' has internal linkage but is not defined"

Redefinition reuses DuplicateDeclaration.

Test matrix

All verified against gcc -m32 -std=iso9899:1990 -pedantic-errors just now. Ticks are what cc1 already gets right.

reject: extern int a; static int a; · int c; static int c; · static int d; int d; · void f(void){extern int x;} static int x; · static int sf(void); int use(void){return sf();} · struct S; struct S a; · struct S; static struct S d; · int f=1; int f=2; ✓ · static void sd(void){} static void sd(void){} ✓

accept: static int b; extern int b; · int g; int g; · int h; int h=1; · int i=1; int i; · static int j; static int j=1; · static int su; int use(void){return sizeof su;} · static int sf(void); static int sf(void){return 0;} int use(void){return sf();} · struct S; struct S late; struct S {int a;};

The last accept case is the one that keeps you honest: item 4 deliberately skips file-scope objects precisely so this stays legal, and step 4 is what finally decides it.

What it unlocks

Codegen's emission decision (.globl vs local, .bss vs .data vs stack slot) is exactly linkage × duration × definition, so this is the last piece of sema that codegen cannot fake. It also makes the 6.5.7 rule at line 2724 — "all the expressions in an initializer for an object that has static storage duration shall be constant expressions" — implementable, since that constraint is keyed on duration, which does not exist today.
