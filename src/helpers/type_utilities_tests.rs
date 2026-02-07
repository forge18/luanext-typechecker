use crate::helpers::type_utilities::{
    is_boolean_type, operator_kind_name, type_to_string, widen_type,
};
use typedlua_parser::ast::expression::OperatorKind;
use typedlua_parser::ast::types::{Literal, PrimitiveType, Type, TypeKind};
use typedlua_parser::span::Span;

fn default_span() -> Span {
    Span::new(0, 0, 0, 0)
}

fn create_type(kind: TypeKind) -> Type {
    Type::new(kind, default_span())
}

#[test]
fn test_is_boolean_type_true() {
    let bool_type = create_type(TypeKind::Primitive(PrimitiveType::Boolean));
    assert!(is_boolean_type(&bool_type));
}

#[test]
fn test_is_boolean_type_false() {
    let number_type = create_type(TypeKind::Primitive(PrimitiveType::Number));
    assert!(!is_boolean_type(&number_type));

    let string_type = create_type(TypeKind::Primitive(PrimitiveType::String));
    assert!(!is_boolean_type(&string_type));
}

#[test]
fn test_widen_type_literal_number() {
    let literal = Type::new(TypeKind::Literal(Literal::Number(42.0)), default_span());
    let widened = widen_type(literal);
    assert!(matches!(
        widened.kind,
        TypeKind::Primitive(PrimitiveType::Number)
    ));
}

#[test]
fn test_widen_type_literal_integer() {
    let literal = Type::new(TypeKind::Literal(Literal::Integer(42)), default_span());
    let widened = widen_type(literal);
    assert!(matches!(
        widened.kind,
        TypeKind::Primitive(PrimitiveType::Number)
    ));
}

#[test]
fn test_widen_type_literal_string() {
    let literal = Type::new(
        TypeKind::Literal(Literal::String("test".into())),
        default_span(),
    );
    let widened = widen_type(literal);
    assert!(matches!(
        widened.kind,
        TypeKind::Primitive(PrimitiveType::String)
    ));
}

#[test]
fn test_widen_type_literal_boolean() {
    let literal = Type::new(TypeKind::Literal(Literal::Boolean(true)), default_span());
    let widened = widen_type(literal);
    assert!(matches!(
        widened.kind,
        TypeKind::Primitive(PrimitiveType::Boolean)
    ));
}

#[test]
fn test_widen_type_literal_nil() {
    let literal = Type::new(TypeKind::Literal(Literal::Nil), default_span());
    let widened = widen_type(literal);
    assert!(matches!(
        widened.kind,
        TypeKind::Primitive(PrimitiveType::Nil)
    ));
}

#[test]
fn test_widen_type_primitive_unchanged() {
    let number_type = create_type(TypeKind::Primitive(PrimitiveType::Number));
    let widened = widen_type(number_type.clone());
    assert_eq!(widened.kind, number_type.kind);
}

#[test]
fn test_operator_kind_name_add() {
    assert_eq!(operator_kind_name(&OperatorKind::Add), "__add");
}

#[test]
fn test_operator_kind_name_subtract() {
    assert_eq!(operator_kind_name(&OperatorKind::Subtract), "__sub");
}

#[test]
fn test_operator_kind_name_multiply() {
    assert_eq!(operator_kind_name(&OperatorKind::Multiply), "__mul");
}

#[test]
fn test_operator_kind_name_divide() {
    assert_eq!(operator_kind_name(&OperatorKind::Divide), "__div");
}

#[test]
fn test_operator_kind_name_index() {
    assert_eq!(operator_kind_name(&OperatorKind::Index), "__index");
}

#[test]
fn test_operator_kind_name_call() {
    assert_eq!(operator_kind_name(&OperatorKind::Call), "__call");
}

#[test]
fn test_operator_kind_name_all_operators() {
    use OperatorKind::*;

    assert_eq!(operator_kind_name(&Add), "__add");
    assert_eq!(operator_kind_name(&Subtract), "__sub");
    assert_eq!(operator_kind_name(&Multiply), "__mul");
    assert_eq!(operator_kind_name(&Divide), "__div");
    assert_eq!(operator_kind_name(&FloorDivide), "__idiv");
    assert_eq!(operator_kind_name(&Modulo), "__mod");
    assert_eq!(operator_kind_name(&Power), "__pow");
    assert_eq!(operator_kind_name(&Concatenate), "__concat");
    assert_eq!(operator_kind_name(&Equal), "__eq");
    assert_eq!(operator_kind_name(&NotEqual), "__ne");
    assert_eq!(operator_kind_name(&LessThan), "__lt");
    assert_eq!(operator_kind_name(&LessThanOrEqual), "__le");
    assert_eq!(operator_kind_name(&GreaterThan), "__gt");
    assert_eq!(operator_kind_name(&GreaterThanOrEqual), "__ge");
    assert_eq!(operator_kind_name(&Length), "__len");
    assert_eq!(operator_kind_name(&UnaryMinus), "__unm");
    assert_eq!(operator_kind_name(&BitwiseAnd), "__band");
    assert_eq!(operator_kind_name(&BitwiseOr), "__bor");
    assert_eq!(operator_kind_name(&BitwiseXor), "__bxor");
    assert_eq!(operator_kind_name(&ShiftLeft), "__shl");
    assert_eq!(operator_kind_name(&ShiftRight), "__shr");
    assert_eq!(operator_kind_name(&Index), "__index");
    assert_eq!(operator_kind_name(&NewIndex), "__newindex");
    assert_eq!(operator_kind_name(&Call), "__call");
}

#[test]
fn test_type_to_string_primitive() {
    let number_type = create_type(TypeKind::Primitive(PrimitiveType::Number));
    let result = type_to_string(&number_type);
    assert!(!result.is_empty());
}

#[test]
fn test_type_to_string_nil() {
    let nil_type = create_type(TypeKind::Primitive(PrimitiveType::Nil));
    let result = type_to_string(&nil_type);
    assert!(!result.is_empty());
}
