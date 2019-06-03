#![cfg(test)]

use super::*;

#[test]
fn int() {
    assert_eq!(
        try_get_number("12345").unwrap().0 .0,
        Token::NumberLiteral(12345.)
    );
}

#[test]
fn signed_int() {
    assert_eq!(
        try_get_number("-98765").unwrap().0 .0,
        Token::NumberLiteral(-98765.)
    );
}

#[test]
fn only_fract() {
    assert_eq!(
        try_get_number("0.111").unwrap().0 .0,
        Token::NumberLiteral(0.111)
    );
}

#[test]
fn signed_only_fract() {
    assert_eq!(
        try_get_number("-0.9").unwrap().0 .0,
        Token::NumberLiteral(-0.9)
    );
}

#[test]
fn int_fract() {
    assert_eq!(
        try_get_number("3.14159").unwrap().0 .0,
        Token::NumberLiteral(3.14159)
    );
}

#[test]
fn signed_int_fract() {
    assert_eq!(
        try_get_number("-444.55555678").unwrap().0 .0,
        Token::NumberLiteral(-444.55555678)
    );
}

#[test]
fn exp() {
    assert_eq!(
        try_get_number("0e67").unwrap().0 .0,
        Token::NumberLiteral(0e67)
    );
    assert_eq!(
        try_get_number("0E67").unwrap().0 .0,
        Token::NumberLiteral(0e67)
    );
    assert_eq!(
        try_get_number("123e4").unwrap().0 .0,
        Token::NumberLiteral(123e4)
    );
    assert_eq!(
        try_get_number("0.5e0").unwrap().0 .0,
        Token::NumberLiteral(0.5)
    );
    assert_eq!(
        try_get_number("6.67e-11").unwrap().0 .0,
        Token::NumberLiteral(6.67e-11)
    );
}

#[test]
fn trailing_content() {
    assert_eq!(
        try_get_number("3.14159, null").unwrap().0 .0,
        Token::NumberLiteral(3.14159)
    );
    assert_eq!(
        try_get_number("3.14159 , \"what\"").unwrap().0 .0,
        Token::NumberLiteral(3.14159)
    );
}

#[test]
fn reject_invalid() {
    assert!(try_get_number("+3").is_err());
    assert!(try_get_number("03.2").is_err());
    assert!(try_get_number("-01").is_err());
    assert!(try_get_number(".5").is_err());
    assert!(try_get_number("5.").is_err());
    assert!(try_get_number("e123").is_err());
    assert!(try_get_number("2.78E").is_err());
    assert!(try_get_number("3e-").is_err());
}
