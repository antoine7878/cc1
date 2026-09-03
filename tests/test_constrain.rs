use cc1::ast::declaration::DeclaratorId;
use cc1::ast::{
    DeclarationSpecifier, InitDeclaratorNode, Initializer, InitializerNode, Name, Qualifier, Storage, StringId,
    TypeSpecifier, Value,
};
use cc1::ast::{DeclaratorNode, Node};
use cc1::parser::Span;
use cc1::semantic::constrain::declaration::{
    basic_type, check_bit_width, check_complete_object, check_element_type, check_member_type, check_qualifier,
    extern_function_only, get_qualifier, get_storage,
};
use cc1::semantic::constrain::external::{
    check_complete_parameter, check_definition_return, check_external_specifiers, check_function_storage,
    is_tentative_definition, is_valid_old_style, param_storage_only_register,
};
use cc1::semantic::{Diag, QualifiedType, ResolvedType, ScopeKind, Sema};

fn reported<T>(diag: &Diag<T>) -> String {
    match &diag.diagnosis {
        Some(diagnosis) => format!("{diagnosis:?}"),
        None => "None".to_string(),
    }
}

fn name(index: usize) -> StringId {
    StringId::from(index)
}

fn resolve(types: &[TypeSpecifier]) -> Diag<Option<ResolvedType>> {
    basic_type(&types.iter().collect::<Vec<_>>())
}

#[test]
fn no_storage_specifier_is_not_an_error() {
    let diag = get_storage(&[DeclarationSpecifier::Type(TypeSpecifier::Int)]);
    assert_eq!(diag.res, None);
    assert_eq!(reported(&diag), "None");
}

#[test]
fn one_storage_specifier_is_returned() {
    let diag = get_storage(&[
        DeclarationSpecifier::Storage(Storage::Static),
        DeclarationSpecifier::Type(TypeSpecifier::Int),
    ]);
    assert_eq!(diag.res, Some(Storage::Static));
    assert_eq!(reported(&diag), "None");
}

#[test]
fn two_storage_specifiers_are_rejected() {
    let diag = get_storage(&[
        DeclarationSpecifier::Storage(Storage::Static),
        DeclarationSpecifier::Storage(Storage::Extern),
    ]);
    assert_eq!(diag.res, Some(Storage::Static));
    assert_eq!(reported(&diag), "MultipleStorageSpecifiers");
}

#[test]
fn a_function_declared_in_a_block_must_be_extern() {
    assert_eq!(
        reported(&extern_function_only(ScopeKind::Block, Storage::Extern)),
        "None"
    );
    assert_eq!(
        reported(&extern_function_only(ScopeKind::Block, Storage::Static)),
        "BlockScopeNotExtern"
    );
    assert_eq!(
        reported(&extern_function_only(ScopeKind::Block, Storage::Auto)),
        "BlockScopeNotExtern"
    );
    assert_eq!(
        reported(&extern_function_only(ScopeKind::Function, Storage::Auto)),
        "BlockScopeNotExtern"
    );
    assert_eq!(
        reported(&extern_function_only(ScopeKind::Function, Storage::Extern)),
        "None"
    );
    assert_eq!(
        reported(&extern_function_only(ScopeKind::File, Storage::Static)),
        "None"
    );
    assert_eq!(reported(&extern_function_only(ScopeKind::File, Storage::Auto)), "None");
    assert_eq!(
        reported(&extern_function_only(ScopeKind::Prototype, Storage::Auto)),
        "None"
    );
}

#[test]
fn no_qualifier_leaves_the_type_unqualified() {
    let diag = check_qualifier([]);
    assert_eq!(diag.res, (false, false));
    assert_eq!(reported(&diag), "None");
}

#[test]
fn each_qualifier_is_reported_once() {
    assert_eq!(check_qualifier([Qualifier::Const]).res, (true, false));
    assert_eq!(check_qualifier([Qualifier::Volatile]).res, (false, true));
    assert_eq!(
        check_qualifier([Qualifier::Const, Qualifier::Volatile]).res,
        (true, true)
    );
    assert_eq!(
        reported(&check_qualifier([Qualifier::Const, Qualifier::Volatile])),
        "None"
    );
}

#[test]
fn a_repeated_qualifier_is_rejected() {
    let diag = check_qualifier([Qualifier::Const, Qualifier::Const]);
    assert_eq!(diag.res, (true, false));
    assert_eq!(reported(&diag), "DuplicateTypeQualifiers");
    assert_eq!(
        reported(&check_qualifier([Qualifier::Volatile, Qualifier::Volatile])),
        "DuplicateTypeQualifiers"
    );
}

#[test]
fn qualifiers_are_picked_out_of_the_specifier_list() {
    let specifiers = [
        DeclarationSpecifier::Storage(Storage::Static),
        DeclarationSpecifier::Qualifier(Qualifier::Const),
        DeclarationSpecifier::Type(TypeSpecifier::Int),
    ];
    let diag = get_qualifier(&specifiers);
    assert_eq!(diag.res, (true, false));
    assert_eq!(reported(&diag), "None");
    assert_eq!(
        reported(&get_qualifier(&[
            DeclarationSpecifier::Qualifier(Qualifier::Const),
            DeclarationSpecifier::Qualifier(Qualifier::Const)
        ])),
        "DuplicateTypeQualifiers"
    );
}

#[test]
fn a_bit_field_must_have_int_type() {
    let diag = check_bit_width(&ResolvedType::Int, Some(Value::Int(3)));
    assert_eq!(diag.res, Some(3));
    assert_eq!(reported(&diag), "None");

    let diag = check_bit_width(&ResolvedType::Char, Some(Value::Int(3)));
    assert_eq!(diag.res, None);
    assert_eq!(reported(&diag), "NonIntBitFieldType");
}

#[test]
fn a_bit_field_width_must_be_an_integer_constant() {
    let diag = check_bit_width(&ResolvedType::Int, Some(Value::Double(1.0)));
    assert_eq!(diag.res, None);
    assert_eq!(reported(&diag), "NonIntegerConstantExpression");
}

#[test]
fn a_member_without_a_width_is_not_a_bit_field() {
    let diag = check_bit_width(&ResolvedType::Int, None);
    assert_eq!(diag.res, None);
    assert_eq!(reported(&diag), "None");
}

#[test]
fn auto_and_register_are_rejected_at_file_scope() {
    assert_eq!(
        reported(&check_external_specifiers(&[DeclarationSpecifier::Storage(
            Storage::Auto
        )])),
        "AutoRegisterExternal"
    );
    assert_eq!(
        reported(&check_external_specifiers(&[DeclarationSpecifier::Storage(
            Storage::Register
        )])),
        "AutoRegisterExternal"
    );
    assert_eq!(
        reported(&check_external_specifiers(&[DeclarationSpecifier::Storage(
            Storage::Static
        )])),
        "None"
    );
    assert_eq!(
        reported(&check_external_specifiers(&[DeclarationSpecifier::Storage(
            Storage::Extern
        )])),
        "None"
    );
    assert_eq!(
        reported(&check_external_specifiers(&[DeclarationSpecifier::Storage(
            Storage::Typedef
        )])),
        "None"
    );
    assert_eq!(reported(&check_external_specifiers(&[])), "None");
}

#[test]
fn a_function_definition_is_static_or_extern() {
    assert_eq!(reported(&check_function_storage(Storage::Static)), "None");
    assert_eq!(reported(&check_function_storage(Storage::Extern)), "None");
    assert_eq!(reported(&check_function_storage(Storage::Auto)), "FunctionAutoExtern");
    assert_eq!(
        reported(&check_function_storage(Storage::Register)),
        "FunctionAutoExtern"
    );
    assert_eq!(
        reported(&check_function_storage(Storage::Typedef)),
        "FunctionAutoExtern"
    );
}

#[test]
fn an_old_style_parameter_declaration_is_register_or_nothing() {
    let diag = param_storage_only_register(Storage::Register);
    assert_eq!(diag.res, Some(()));
    assert_eq!(reported(&diag), "None");

    let diag = param_storage_only_register(Storage::Static);
    assert_eq!(diag.res, None);
    assert_eq!(reported(&diag), "ParameterNotRegister");
    assert_eq!(
        reported(&param_storage_only_register(Storage::Auto)),
        "ParameterNotRegister"
    );
}

#[test]
fn a_fully_declared_old_style_list_leaves_no_parameter_implicit() {
    let diag = is_valid_old_style(&[name(1), name(2)], vec![Some(name(1)), Some(name(2))]);
    assert_eq!(diag.res, Some(Vec::new()));
    assert_eq!(reported(&diag), "None");
}

#[test]
fn an_undeclared_old_style_parameter_is_implicitly_int() {
    let diag = is_valid_old_style(&[name(1), name(2)], vec![Some(name(1))]);
    assert_eq!(diag.res, Some(vec![name(2)]));
    assert_eq!(reported(&diag), "None");
}

#[test]
fn an_old_style_parameter_declared_twice_is_not_flagged_here() {
    let diag = is_valid_old_style(&[name(1)], vec![Some(name(1)), Some(name(1))]);
    assert_eq!(diag.res, Some(Vec::new()));
    assert_eq!(reported(&diag), "None");
}

#[test]
fn a_duplicate_old_style_parameter_name_is_rejected() {
    let diag = is_valid_old_style(&[name(1), name(1)], vec![]);
    assert_eq!(diag.res, None);
    assert_eq!(reported(&diag), "DuplicateParameterName");
}

#[test]
fn an_old_style_declaration_of_an_unlisted_name_is_rejected() {
    let diag = is_valid_old_style(&[name(1)], vec![Some(name(2))]);
    assert_eq!(diag.res, None);
    assert_eq!(reported(&diag), "MissingParameterInOldStyle");
}

#[test]
fn an_unresolved_old_style_declaration_falls_back_to_implicit_int() {
    let diag = is_valid_old_style(&[name(1)], vec![None]);
    assert_eq!(diag.res, Some(vec![name(1)]));
    assert_eq!(reported(&diag), "None");
}

#[test]
fn no_type_specifier_denotes_int() {
    let diag = resolve(&[]);
    assert_eq!(diag.res, Some(ResolvedType::Int));
    assert_eq!(reported(&diag), "None");
}

#[test]
fn each_single_keyword_names_its_type() {
    use TypeSpecifier as T;
    assert_eq!(resolve(&[T::Void]).res, Some(ResolvedType::Void));
    assert_eq!(resolve(&[T::Char]).res, Some(ResolvedType::Char));
    assert_eq!(resolve(&[T::Short]).res, Some(ResolvedType::Short));
    assert_eq!(resolve(&[T::Int]).res, Some(ResolvedType::Int));
    assert_eq!(resolve(&[T::Long]).res, Some(ResolvedType::Long));
    assert_eq!(resolve(&[T::Float]).res, Some(ResolvedType::Float));
    assert_eq!(resolve(&[T::Double]).res, Some(ResolvedType::Double));
    assert_eq!(resolve(&[T::Signed]).res, Some(ResolvedType::Int));
    assert_eq!(resolve(&[T::Unsigned]).res, Some(ResolvedType::UnsignedInt));
}

#[test]
fn a_combination_resolves_regardless_of_order() {
    use TypeSpecifier as T;
    let want = Some(ResolvedType::UnsignedLong);
    assert_eq!(resolve(&[T::Unsigned, T::Long, T::Int]).res, want);
    assert_eq!(resolve(&[T::Int, T::Unsigned, T::Long]).res, want);
    assert_eq!(resolve(&[T::Long, T::Int, T::Unsigned]).res, want);
}

#[test]
fn equivalent_spellings_resolve_to_the_same_type() {
    use TypeSpecifier as T;
    assert_eq!(
        resolve(&[T::Short]).res,
        resolve(&[T::Signed, T::Short, T::Int]).res
    );
    assert_eq!(resolve(&[T::Signed, T::Char]).res, Some(ResolvedType::SignedChar));
    assert_eq!(resolve(&[T::Long, T::Double]).res, Some(ResolvedType::LongDouble));
}

#[test]
fn a_repeated_specifier_is_rejected() {
    use TypeSpecifier as T;
    for types in [
        vec![T::Long, T::Long],
        vec![T::Int, T::Int],
        vec![T::Signed, T::Int, T::Signed],
    ] {
        let diag = resolve(&types);
        assert_eq!(diag.res, None);
        assert_eq!(reported(&diag), "InvalidTypeSpecifier");
    }
}

#[test]
fn an_unlisted_combination_is_rejected() {
    use TypeSpecifier as T;
    for types in [
        vec![T::Signed, T::Unsigned],
        vec![T::Short, T::Long],
        vec![T::Long, T::Long, T::Int],
    ] {
        assert_eq!(reported(&resolve(&types)), "InvalidTypeSpecifier");
    }
}

#[test]
fn a_tag_or_typedef_name_is_not_a_basic_type() {
    let name = Name::new(StringId::from(0usize), Span::default());
    assert_eq!(reported(&resolve(&[TypeSpecifier::TypedefName(name)])), "InvalidTypeSpecifier");
    assert_eq!(
        reported(&resolve(&[TypeSpecifier::Int, TypeSpecifier::TypedefName(name)])),
        "InvalidTypeSpecifier"
    );
}

fn init_declarator(initializer: Option<InitializerNode>) -> InitDeclaratorNode {
    let declarator = DeclaratorNode::new(DeclaratorId::from(0usize), Span::default());
    InitDeclaratorNode::new(declarator, initializer, Span::default())
}

fn initializer() -> InitializerNode {
    InitializerNode::new(Initializer::List(Vec::new()), Span::default())
}

#[test]
fn a_tentative_definition_has_no_initializer() {
    assert!(is_tentative_definition(&init_declarator(None), Some(Storage::Static)));
    assert!(!is_tentative_definition(
        &init_declarator(Some(initializer())),
        Some(Storage::Static)
    ));
}

#[test]
fn a_tentative_definition_is_static_or_unqualified() {
    assert!(is_tentative_definition(&init_declarator(None), None));
    assert!(!is_tentative_definition(&init_declarator(Some(initializer())), None));
    assert!(!is_tentative_definition(&init_declarator(None), Some(Storage::Extern)));
    assert!(!is_tentative_definition(
        &init_declarator(Some(initializer())),
        Some(Storage::Extern)
    ));
    assert_eq!(init_declarator(None).span(), Span::default());
}

#[test]
fn a_tentative_definition_is_not_typedef_auto_or_register() {
    assert!(!is_tentative_definition(&init_declarator(None), Some(Storage::Typedef)));
    assert!(!is_tentative_definition(&init_declarator(None), Some(Storage::Auto)));
    assert!(!is_tentative_definition(
        &init_declarator(None),
        Some(Storage::Register)
    ));
}

fn int_type() -> QualifiedType {
    let sema = Sema::default();
    QualifiedType::new(sema.builtins.int, false, false)
}

#[test]
fn an_object_with_a_complete_type_is_accepted() {
    assert_eq!(reported(&check_complete_object(true, int_type())), "None");
}

#[test]
fn an_object_with_an_incomplete_type_is_rejected() {
    assert!(reported(&check_complete_object(false, int_type())).starts_with("IncompleteVariable("));
}

#[test]
fn a_member_with_an_object_type_is_accepted() {
    assert_eq!(reported(&check_member_type(true, int_type())), "None");
}

#[test]
fn a_member_without_an_object_type_is_rejected() {
    assert!(reported(&check_member_type(false, int_type())).starts_with("InvalidMemberType("));
}

#[test]
fn an_array_element_with_an_object_type_is_accepted() {
    assert_eq!(reported(&check_element_type(true, int_type())), "None");
}

#[test]
fn an_array_element_without_an_object_type_is_rejected() {
    assert!(reported(&check_element_type(false, int_type())).starts_with("InvalidElementType("));
}

#[test]
fn a_complete_parameter_is_accepted() {
    assert_eq!(reported(&check_complete_parameter(true, int_type())), "None");
}

#[test]
fn an_incomplete_parameter_is_rejected() {
    assert!(reported(&check_complete_parameter(false, int_type())).starts_with("IncompleteParameter("));
}

#[test]
fn a_valid_definition_return_type_is_accepted() {
    assert_eq!(reported(&check_definition_return(true, int_type())), "None");
}

#[test]
fn an_invalid_definition_return_type_is_rejected() {
    assert!(reported(&check_definition_return(false, int_type())).starts_with("IncompleteReturn("));
}
