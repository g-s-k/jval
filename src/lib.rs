#![warn(clippy::pedantic)]

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Range;
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

type TokenRecord = (Token, usize, usize);
type Error = (ErrorKind, usize, usize);

fn try_get_string(s: &str) -> Result<(TokenRecord, &str), Error> {
    let s_len = s.len();
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
            '"' if !escape => return Ok(((Token::StringLiteral(out), i + 1, s_len), &s[i + 1..])),
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
                                return Err((ErrorKind::InvalidUnicode, u.len(), s_len - i + 3));
                            }
                        }
                        _ => return Err((ErrorKind::InvalidUnicode, u.len(), s_len - i + u.len())),
                    }
                }
            }
            _ => out.push(c),
        }
    }

    Err((ErrorKind::UnterminatedString, s_len, s_len))
}

fn try_get_number(s: &str) -> Result<(TokenRecord, &str), Error> {
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
            ('.', Point) | ('.', Whole) | ('0'...'9', Fract) => Fract,
            ('-', ExpSign) | ('+', ExpSign) | ('0'...'9', Exp) | ('0'...'9', ExpSign) => Exp,
            ('e', Fract)
            | ('E', Fract)
            | ('e', Point)
            | ('E', Point)
            | ('e', Whole)
            | ('E', Whole) => ExpSign,
            ('-', _) | ('+', _) | ('0'...'9', _) | ('.', _) | ('e', _) | ('E', _) => {
                return Err((ErrorKind::InvalidNumber, 1, s.len() - i))
            }
            _ => break,
        };

        current = i;
    }

    // no lookahead!
    if s[..=current].ends_with('.') {
        return Err((ErrorKind::InvalidNumber, 1, s.len() - current));
    }

    s[..=current]
        .parse()
        .map(|n| {
            (
                (Token::NumberLiteral(n), current + 1, s.len()),
                &s[current + 1..],
            )
        })
        .map_err(|_| (ErrorKind::InvalidNumber, s.len(), s.len()))
}

type TokenResult<'a> = Result<(Option<TokenRecord>, &'a str), Error>;

fn next_token(mut s: &str) -> TokenResult {
    fn split_slice(slice: &str, idx: usize, token: Token) -> TokenResult {
        Ok((Some((token, idx, slice.len())), &slice[idx..]))
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

fn next_value(tokens: &[TokenRecord]) -> Result<(Option<Json>, &[TokenRecord]), Error> {
    if let Some(tup) = tokens.split_first() {
        match tup {
            ((Token::Null, _, _), rest) => Ok((Some(Json::Null), rest)),
            ((Token::True, _, _), rest) => Ok((Some(Json::Boolean(true)), rest)),
            ((Token::False, _, _), rest) => Ok((Some(Json::Boolean(false)), rest)),
            ((Token::NumberLiteral(n), _, _), rest) => Ok((Some(Json::Number(*n)), rest)),
            ((Token::StringLiteral(n), _, _), rest) => {
                Ok((Some(Json::String(n.to_string())), rest))
            }
            ((Token::OpenCurly, tok_len, tok_rest), mut rest) => {
                let mut map = BTreeMap::new();

                if let Some(((Token::CloseCurly, _, _), more)) = rest.split_first() {
                    return Ok((Some(Json::Object(map)), more));
                }

                while let Some(((Token::StringLiteral(key), tok_len, tok_rest), more)) =
                    rest.split_first()
                {
                    if let Some(((Token::Colon, _, f), even_more)) = more.split_first() {
                        if let (Some(value), still_more) = next_value(even_more)? {
                            map.insert(key.to_string(), value);

                            match still_more.split_first() {
                                Some(((Token::Comma, _, _), please_stop)) => rest = please_stop,
                                Some(((Token::CloseCurly, _, _), please_stop)) => {
                                    return Ok((Some(Json::Object(map)), please_stop));
                                }
                                Some(((_, l, f), _)) => {
                                    return Err((ErrorKind::UnexpectedToken, *l, *f))
                                }
                                None => return Err((ErrorKind::UnterminatedObject, 0, *f)),
                            }
                        } else {
                            return Err((ErrorKind::UnterminatedObject, 0, *f));
                        }
                    } else {
                        return Err((ErrorKind::UnexpectedToken, *tok_len, *tok_rest));
                    }
                }

                Err((ErrorKind::UnexpectedToken, *tok_len, *tok_rest))
            }
            ((Token::OpenSquare, _, f), mut rest) => {
                let mut vec = Vec::new();

                if let Some(((Token::CloseSquare, _, _), more)) = rest.split_first() {
                    return Ok((Some(Json::Array(vec)), more));
                }

                while let (Some(value), more) = next_value(rest)? {
                    vec.push(value);

                    match more.split_first() {
                        Some(((Token::Comma, _, _), even_more)) => rest = even_more,
                        Some(((Token::CloseSquare, _, _), even_more)) => {
                            return Ok((Some(Json::Array(vec)), even_more))
                        }
                        Some(((_, l, f), _)) => return Err((ErrorKind::UnexpectedToken, *l, *f)),
                        None => return Err((ErrorKind::UnterminatedArray, 0, *f)),
                    }
                }

                Err((ErrorKind::UnterminatedArray, 0, *f))
            }
            ((_, l, f), _) => return Err((ErrorKind::UnexpectedToken, *l, *f)),
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
    Object(BTreeMap<String, Self>),
}

impl FromStr for Json {
    type Err = (ErrorKind, Range<usize>);

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let s_len = s.len();
        let calc_range = |(e, l, f)| {
            (
                e,
                Range {
                    start: s_len - f,
                    end: s_len - f + l,
                },
            )
        };

        let mut tokens = Vec::new();

        while let (Some(t), s_) = next_token(s).map_err(calc_range)? {
            tokens.push(t);
            s = s_;
        }

        let mut toks = &tokens[..];
        let mut values = Vec::new();

        while let (Some(v), t_) = next_value(toks).map_err(calc_range)? {
            values.push(v);
            toks = t_;
        }

        if values.len() == 1 {
            Ok(values.remove(0))
        } else {
            Err(calc_range((ErrorKind::UnexpectedToken, s.len(), s.len())))
        }
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Json::Null => write!(f, "null"),
            Json::Boolean(b) => write!(f, "{}", b),
            Json::Number(n) => write!(f, "{}", n),
            Json::String(s) => write!(f, "\"{}\"", s),
            Json::Array(a) => write!(
                f,
                "[{}]",
                a.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Json::Object(o) => write!(
                f,
                "{{{}}}",
                o.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, v))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

pub enum Spacing {
    None,
    Tab,
    Space(usize),
}

impl Json {
    pub fn print<W: fmt::Write>(&self, spacing: &Spacing, f: &mut W) -> fmt::Result {
        match spacing {
            Spacing::None => write!(f, "{}", self),
            Spacing::Tab => self.print_tabs(0, f),
            Spacing::Space(n) => self.print_indented(*n, 0, f),
        }
    }

    fn print_indented<W: fmt::Write>(
        &self,
        spacing: usize,
        start: usize,
        f: &mut W,
    ) -> fmt::Result {
        match self {
            Json::Array(a) if a.is_empty() => write!(f, "[]"),
            Json::Array(a) if a.len() == 1 => write!(f, "[ {} ]", a[0]),
            Json::Object(o) if o.is_empty() => write!(f, "{{}}"),

            Json::Array(a) => {
                writeln!(f, "[")?;

                for (i, el) in a.iter().enumerate() {
                    for _ in 0..start + spacing {
                        write!(f, " ",)?;
                    }

                    el.print_indented(spacing, start + spacing, f)?;

                    if i != a.len() - 1 {
                        write!(f, ",")?;
                    }

                    writeln!(f)?;
                }

                for _ in 0..start {
                    write!(f, " ",)?;
                }
                write!(f, "]")
            }
            Json::Object(o) => {
                writeln!(f, "{{")?;

                for (i, (k, v)) in o.iter().enumerate() {
                    for _ in 0..start + spacing {
                        write!(f, " ",)?;
                    }

                    write!(f, "\"{}\": ", k)?;
                    v.print_indented(spacing, start + spacing, f)?;

                    if i != o.len() - 1 {
                        write!(f, ",")?;
                    }
                    writeln!(f)?;
                }

                for _ in 0..start {
                    write!(f, " ",)?;
                }
                write!(f, "}}")
            }

            _ => write!(f, "{}", self),
        }
    }

    fn print_tabs<W: fmt::Write>(&self, start: usize, f: &mut W) -> fmt::Result {
        match self {
            Json::Array(a) if a.is_empty() => write!(f, "[]"),
            Json::Array(a) if a.len() == 1 => write!(f, "[ {} ]", a[0]),
            Json::Object(o) if o.is_empty() => write!(f, "{{}}"),

            Json::Array(a) => {
                writeln!(f, "[")?;

                for (i, el) in a.iter().enumerate() {
                    for _ in 0..=start {
                        write!(f, "\t",)?;
                    }

                    el.print_tabs(start + 1, f)?;

                    if i != a.len() - 1 {
                        write!(f, ",")?;
                    }

                    writeln!(f)?;
                }

                for _ in 0..start {
                    write!(f, "\t",)?;
                }
                write!(f, "]")
            }
            Json::Object(o) => {
                writeln!(f, "{{")?;

                for (i, (k, v)) in o.iter().enumerate() {
                    for _ in 0..=start {
                        write!(f, "\t",)?;
                    }

                    write!(f, "\"{}\": ", k)?;
                    v.print_tabs(start + 1, f)?;

                    if i != o.len() - 1 {
                        write!(f, ",")?;
                    }
                    writeln!(f)?;
                }

                for _ in 0..start {
                    write!(f, "\t",)?;
                }
                write!(f, "}}")
            }

            _ => write!(f, "{}", self),
        }
    }
}
