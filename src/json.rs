#![cfg(test)]

use super::*;

#[test]
fn null() {
    assert_eq!("null".parse::<Json>().unwrap(), Json::Null);
}

#[test]
fn r#true() {
    assert_eq!("true".parse::<Json>().unwrap(), Json::Boolean(true));
}

#[test]
fn r#false() {
    assert_eq!("false".parse::<Json>().unwrap(), Json::Boolean(false));
}

#[test]
fn numbers() {
    assert_eq!("0".parse::<Json>().unwrap(), Json::Number(0.));
    assert_eq!(" 6 ".parse::<Json>().unwrap(), Json::Number(6.));
    assert_eq!("-3.5".parse::<Json>().unwrap(), Json::Number(-3.5));
    assert_eq!(
        "6.626E-34".parse::<Json>().unwrap(),
        Json::Number(6.626e-34)
    );
}

#[test]
fn array() {
    assert_eq!(
        "[false, 3, \"hello\", {}]".parse::<Json>().unwrap(),
        Json::Array(vec![
            Json::Boolean(false),
            Json::Number(3.),
            Json::String("hello".into()),
            Json::Object(BTreeMap::new())
        ])
    );
}

#[test]
fn object() {
    assert_eq!(
        r#"{"three": 3, "null": null, "string": "foo bar baz"}"#
            .parse::<Json>()
            .unwrap(),
        Json::Object(
            vec![
                ("three".into(), Json::Number(3.)),
                ("null".into(), Json::Null),
                ("string".into(), Json::String("foo bar baz".into()))
            ]
            .into_iter()
            .collect()
        )
    );
}

#[test]
fn reject_invalid() {
    assert!("[".parse::<Json>().is_err());
    assert!("]".parse::<Json>().is_err());
    assert!("{".parse::<Json>().is_err());
    assert!("}".parse::<Json>().is_err());
    assert!("[ ,]".parse::<Json>().is_err());
    assert!("{ , }".parse::<Json>().is_err());
    // trailing commas
    assert!("[null, true, \"wow\",]".parse::<Json>().is_err());
    assert!(r#"{ "yes": null, "no": true, "wow": 333, }"#.parse::<Json>().is_err());

    // un-quoted keys
    assert!("{ yes: null, no: true, wow: 333 }".parse::<Json>().is_err());
}
