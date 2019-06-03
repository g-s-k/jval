use std::collections::HashMap;
use std::str::FromStr;

mod json;
mod number;
mod string;

#[derive(Debug)]
pub enum ErrorKind {
    InvalidUnicode,
    UnterminatedString,
    InvalidNumber,
    UnexpectedToken,
    UnterminatedObject,
    UnterminatedArray,
}

#[derive(Debug, PartialEq)]
enum Token {
    OpenCurly,
    CloseCurly,
    OpenSquare,
    CloseSquare,
    Comma,
    Colon,
    Null,
    True,
    False,
    NumberLiteral(f64),
    StringLiteral(String),
}

type Error<'a> = (ErrorKind, &'a str);

fn try_get_string(s: &str) -> Result<((Token, &str), &str), Error> {
    let mut out = String::new();
    let mut escape = false;
    let mut unic = None;

    for (i, c) in s.char_indices().skip(1) {
        match c {
            '\\' if !escape => escape = true,
            '\\' => {
                out.push(c);
                escape = false;
            }
            '"' if !escape => return Ok(((Token::StringLiteral(out), &s[..=i]), &s[i + 1..])),
            'b' if escape => {
                out.push(8 as char);
                escape = false;
            }
            'f' if escape => {
                out.push(12 as char);
                escape = false;
            }
            'n' if escape => {
                out.push('\n');
                escape = false;
            }
            'r' if escape => {
                out.push('\r');
                escape = false;
            }
            't' if escape => {
                out.push('\t');
                escape = false;
            }
            'u' if escape => {
                unic = Some(String::new());
                escape = false;
            }
            d if d.is_ascii_hexdigit() && unic.is_some() => {
                if let Some(u) = unic.iter_mut().next() {
                    match u.len() {
                        0...2 => u.push(d),
                        3 => {
                            u.push(d);

                            if let Some(uc) = std::char::from_u32(
                                u32::from_str_radix(&u, 16)
                                    .expect("failed to parse code point from hex number"),
                            ) {
                                out.push(uc);
                                unic = None;
                            } else {
                                return Err((ErrorKind::InvalidUnicode, &s[i..]));
                            }
                        }
                        _ => return Err((ErrorKind::InvalidUnicode, &s[i..])),
                    }
                }
            }
            _ => out.push(c),
        }
    }

    Err((ErrorKind::UnterminatedString, s))
}

fn try_get_number(s: &str) -> Result<((Token, &str), &str), Error> {
    enum State {
        Sign,
        Whole,
        Point,
        Fract,
        ExpSign,
        Exp,
    }

    use State::*;

    let mut state = State::Sign;
    let mut current = 0;

    for (i, c) in s.char_indices() {
        state = match (c, state) {
            ('-', Sign) => Sign,
            ('0', Sign) => Point,
            ('1'...'9', Sign) | ('0'...'9', Whole) => Whole,
            ('.', Point) | ('.', Whole) => Fract,
            ('0'...'9', Fract) => Fract,
            ('-', ExpSign) => Exp,
            ('+', ExpSign) => Exp,
            ('e', Fract)
            | ('E', Fract)
            | ('e', Point)
            | ('E', Point)
            | ('e', Whole)
            | ('E', Whole) => ExpSign,
            ('0'...'9', Exp) | ('0'...'9', ExpSign) => Exp,
            ('-', _) | ('+', _) | ('0'...'9', _) | ('.', _) | ('e', _) | ('E', _) => {
                return Err((ErrorKind::InvalidNumber, &s[i..]))
            }
            _ => break,
        };

        current = i;
    }

    // no lookahead!
    if s[..=current].ends_with('.') {
        return Err((ErrorKind::InvalidNumber, &s[current..]));
    }

    s[..=current]
        .parse()
        .map(|n| ((Token::NumberLiteral(n), &s[..=current]), &s[current + 1..]))
        .map_err(|_| (ErrorKind::InvalidNumber, s))
}

type TokenResult<'a> = Result<(Option<(Token, &'a str)>, &'a str), Error<'a>>;

fn next_token(mut s: &str) -> TokenResult {
    fn split_slice(slice: &str, idx: usize, token: Token) -> TokenResult {
        let (head, tail) = slice.split_at(idx);
        Ok((Some((token, head)), tail))
    }

    s = s.trim_start();

    if s.is_empty() {
        return Ok((None, s));
    }

    match &s[..1] {
        "{" => split_slice(s, 1, Token::OpenCurly),
        "}" => split_slice(s, 1, Token::CloseCurly),
        "[" => split_slice(s, 1, Token::OpenSquare),
        "]" => split_slice(s, 1, Token::CloseSquare),
        "," => split_slice(s, 1, Token::Comma),
        ":" => split_slice(s, 1, Token::Colon),
        "\"" => try_get_string(s).map(|(t, s)| (Some(t), s)),
        _ if s.starts_with("null") => split_slice(s, 4, Token::Null),
        _ if s.starts_with("true") => split_slice(s, 4, Token::True),
        _ if s.starts_with("false") => split_slice(s, 5, Token::False),
        _ => try_get_number(s).map(|(t, s)| (Some(t), s)),
    }
}

type ValueResult<'a> = Result<(Option<Json>, &'a [(Token, &'a str)]), Error<'a>>;

fn next_value<'a>(tokens: &'a [(Token, &str)]) -> ValueResult<'a> {
    if let Some(tup) = tokens.split_first() {
        match tup {
            ((Token::Null, _), rest) => Ok((Some(Json::Null), rest)),
            ((Token::True, _), rest) => Ok((Some(Json::Boolean(true)), rest)),
            ((Token::False, _), rest) => Ok((Some(Json::Boolean(false)), rest)),
            ((Token::NumberLiteral(n), _), rest) => Ok((Some(Json::Number(*n)), rest)),
            ((Token::StringLiteral(n), _), rest) => Ok((Some(Json::String(n.to_string())), rest)),
            ((Token::OpenCurly, tok_str), mut rest) => {
                let mut map = HashMap::new();

                if let Some(((Token::CloseCurly, _), more)) = rest.split_first() {
                    return Ok((Some(Json::Object(map)), more));
                }

                while let Some(((Token::StringLiteral(key), tok_str), more)) = rest.split_first() {
                    if let Some(((Token::Colon, _), even_more)) = more.split_first() {
                        if let (Some(value), still_more) = next_value(even_more)? {
                            map.insert(key.to_string(), value);

                            match still_more.split_first() {
                                Some(((Token::Comma, _), please_stop)) => rest = please_stop,
                                Some(((Token::CloseCurly, _), please_stop)) => {
                                    return Ok((Some(Json::Object(map)), please_stop));
                                }
                                Some(((_, s), _)) => return Err((ErrorKind::UnexpectedToken, s)),
                                None => return Err((ErrorKind::UnterminatedObject, "")),
                            }
                        } else {
                            return Err((ErrorKind::UnterminatedObject, ""));
                        }
                    } else {
                        return Err((ErrorKind::UnexpectedToken, tok_str));
                    }
                }

                Err((ErrorKind::UnexpectedToken, tok_str))
            }
            ((Token::OpenSquare, _), mut rest) => {
                let mut vec = Vec::new();

                if let Some(((Token::CloseSquare, _), more)) = rest.split_first() {
                    return Ok((Some(Json::Array(vec)), more));
                }

                while let (Some(value), more) = next_value(rest)? {
                    vec.push(value);

                    match more.split_first() {
                        Some(((Token::Comma, _), even_more)) => rest = even_more,
                        Some(((Token::CloseSquare, _), even_more)) => {
                            return Ok((Some(Json::Array(vec)), even_more))
                        }
                        Some(((_, s), _)) => return Err((ErrorKind::UnexpectedToken, s)),
                        None => return Err((ErrorKind::UnterminatedArray, "")),
                    }
                }

                Err((ErrorKind::UnterminatedArray, ""))
            }
            ((_, s), _) => return Err((ErrorKind::UnexpectedToken, s)),
        }
    } else {
        Ok((None, tokens))
    }
}

#[derive(Debug, PartialEq)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Self>),
    Object(HashMap<String, Self>),
}

impl FromStr for Json {
    type Err = (ErrorKind, String);

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut tokens = Vec::new();

        while let (Some(t), s_) = next_token(s).map_err(|(e, s)| (e, s.into()))? {
            tokens.push(t);
            s = s_;
        }

        let mut toks = &tokens[..];
        let mut values = Vec::new();

        while let (Some(v), t_) = next_value(toks).map_err(|(e, s)| (e, s.into()))? {
            values.push(v);
            toks = t_;
        }

        if values.len() == 1 {
            Ok(values.remove(0))
        } else {
            Err((ErrorKind::UnexpectedToken, s.into()))
        }
    }
}
