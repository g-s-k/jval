#![cfg(test)]

use super::*;

#[test]
fn basic() {
    assert_eq!(
        try_get_string("\"foobar\"").unwrap().0 .0,
        Token::StringLiteral("foobar".to_string())
    );
}

#[test]
fn spaces() {
    assert_eq!(
        try_get_string("\"this is a string with spaces\"")
            .unwrap()
            .0
             .0,
        Token::StringLiteral("this is a string with spaces".to_string())
    );
}

#[test]
fn control_chars() {
    assert_eq!(
        try_get_string(
            "\"i \\n have \\b every \\t control \\r character \\f type \\u1234 inside \\\\ me!\""
        )
        .unwrap()
        .0
         .0,
        Token::StringLiteral(
            "i \n have \u{8} every \t control \r character \u{c} type \u{1234} inside \\ me!"
                .to_string()
        )
    );
}

#[test]
fn trailing_content() {
    assert_eq!(
        try_get_string("\"foo\" bar baz ok there is more stuff here after the closing quote")
            .unwrap(),
        (
            (Token::StringLiteral("foo".to_string()), "\"foo\""),
            " bar baz ok there is more stuff here after the closing quote"
        )
    );
}
