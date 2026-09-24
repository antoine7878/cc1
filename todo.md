Compiler defects:

1. Old-style void parameters are accepted and can produce invalid LLVM.
2. Duplicate names in prototype-only declarations are accepted.
3. Named void prototype parameters receive no diagnostic.
4. Invalid old-style parameter declarations are accepted:
    - initialized parameters;
    - empty declarations.

5. Incompatible external redeclarations across scopes are accepted.
6. An inner bare tag declaration does not hide the outer tag.
7. Tags declared in parameter lists leak outside prototype scope.
8. Empty and unknown escape sequences such as '\x' and '\q' are accepted.
9. Oversized hexadecimal escapes use wrapping arithmetic and can evade diagnostics.
