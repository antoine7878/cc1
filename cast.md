
Here's how I'd shape it, given what's already in src/semantic/model/resolved_expression.rs.

Don't put it in the AST arena

The tempting move is Expression::ImplicitCast(kind, ExpressionNode). It costs more than it looks in this codebase:

- The Expression arena is filled by yacc actions; parents hold their children by value (Expression::Add(lhs, rhs)), so inserting a node means get_mut on every parent and patching the field — every operator arm becomes a rewrite site.
- ExpressionId is currently 1:1 with source text. ctx.expressions, ctx.declarations, spans and diagnostics all key off that. Synthetic ids break the invariant (what span does a promotion have?).
- visit.rs and print.rs grow a variant that can never come out of the parser.

You already have the right place: sema.expressions is a per-id side table, i.e. a parallel resolved tree. A cast recorded there is "written in the tree" for every consumer that goes through it — which is all of them, since codegen needs the table for types anyway.

Shape

// resolved_expression.rs
# [derive(Clone, Copy, Debug)]
pub struct ImplicitCast {
    pub kind: CastKind,
    pub to: QualifiedType,
}

# [derive(Clone, Copy, Debug, PartialEq)]
pub enum CastKind {
    LValueToRValue,     // 6.2.2.1
    ArrayToPointer,     // 6.2.2.1
    FunctionToPointer,  // 6.2.2.1
    IntegerPromotion,   // 6.2.1.1
    IntegerConversion,  // 6.2.1.2
    IntegerToFloating,  // 6.2.1.3
    FloatingToInteger,  // 6.2.1.4
    FloatingConversion, // 6.2.1.4
    PointerConversion,  // 6.3.16.1
    NullPointer,        // 6.2.2.3
    ToVoid,             // 6.3.4
}

pub struct ResolvedExpression {
    pub sym: Option<SymbolId>,
    pub ty: Option<QualifiedType>,        // type as written, before conversion
    pub const_value: Option<Option<Value>>,
    pub kind: Option<ExpressionKind>,
    pub casts: Vec<ImplicitCast>,         // applied on the way to the parent
}

impl ResolvedExpression {
    pub fn value_ty(&self) -> Option<QualifiedType> {
        self.casts.last().map(|c| c.to).or(self.ty)
    }
}

Three points that matter:

A chain, not one cast. char c; c + 1 is LValueToRValue → IntegerPromotion(int); char a[10]; f(a) is ArrayToPointer(char*) with no lvalue conversion. Each element maps to one numbered
paragraph, which fits your comment rule (// 6.2.1.1) exactly. In practice the chne.

Carry to even though it's derivable. The chain is the only thing codegen reads tosi; making it re-derive the target type is how the two passes drift apart.

One slot can need two conversions. a += b is a = a op b with a evaluated once: t operation, and the result converts back to the lhs type. Same for theassignment's rhs. Put the write-back on the parent:

pub result_cast: Option<ImplicitCast>,   // value → type of the assignment target

Ternary branches, call arguments, and return don't need this — each is its own node with its own chain.

Where the chains get built

One module, src/semantic/conversion.rs (not mod.rs), holding the five rules — every operator arm calls into it and never writes a CastKind itself:

┌─────────────────────────────┬───────────────────────────────────────────────────────────┐
│             fn              │                         standard
├─────────────────────────────┼───────────────────────────────────────────────────────────┤
│ lvalue_conversion           │ 6.2.2.1 — lvalue→rvalue, array→pointer, function
├─────────────────────────────┼───────────────────────────────────────────────────────────┤
│ promote                     │ 6.2.1.1
├─────────────────────────────┼───────────────────────────────────────────────────────────┤
│ usual_arithmetic            │ 6.2.1.5 — returns the common type, pushes onto b
├─────────────────────────────┼───────────────────────────────────────────────────────────┤
│ assignment_conversion       │ 6.3.16.1 — reused by =, init, prototype args, re
├─────────────────────────────┼───────────────────────────────────────────────────────────┤
│ default_argument_promotions │ 6.3.2.2 — old-style calls and the variadic tail
└─────────────────────────────┴───────────────────────────────────────────────────────────┘

Note ExpressionKind becomes redundant after a chain: anything with LValueToRValue is an rvalue at the parent. Keep kind as the node's own category (constraint checks need "is this a
modifiable lvalue" before conversion) and let casts answer what the parent sees.

Folding

const_value should hold the value after the chain — (char)300 and an enum E { A ffer, and ICE checking in eval/ice.rs reads the converted value. Simplest rule:fold the node, then apply each cast's truncation in order as you push it.

Testing

describe in display.rs extends naturally — print the chain after the type, e.g. int [lvalue, promote]. Then a converts! macro next to folds!/describes! in tests/test_resolution.rs asserts
the chain for char c; c + 1, char a[10]; a + 1, f(c) under prototype vs old-styl 6.2.1.5. Those assert internal structure, so gcc doesn't validate them directly — cross-check the observable consequences instead (sizeof(c + 1) = 4, 1u < -1 = 1) and let the chain assertions ride on top.
