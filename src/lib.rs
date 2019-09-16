#![warn(clippy::pedantic)]

use std::collections::BTreeMap;
use std::fmt;
use std::io;
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
    TrailingComma,
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
                        0..=2 => u.push(d),
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
            ('1'..='9', Sign) | ('0'..='9', Whole) => Whole,
            ('.', Point) | ('.', Whole) | ('0'..='9', Fract) => Fract,
            ('-', ExpSign) | ('+', ExpSign) | ('0'..='9', Exp) | ('0'..='9', ExpSign) => Exp,
            ('e', Fract)
            | ('E', Fract)
            | ('e', Point)
            | ('E', Point)
            | ('e', Whole)
            | ('E', Whole) => ExpSign,
            ('-', _) | ('+', _) | ('0'..='9', _) | ('.', _) | ('e', _) | ('E', _) => {
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

type MoreToParse<'a> = (Option<Json>, &'a [TokenRecord]);
type ParseError<'a> = (Error, &'a [TokenRecord]);
type ParseResult<'a> = Result<MoreToParse<'a>, ParseError<'a>>;

fn next_value(tokens: &[TokenRecord]) -> ParseResult<'_> {
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

                let mut last_comma = (tok_len, tok_rest);
                while let Some((token, more)) = rest.split_first() {
                    match token {
                        (Token::StringLiteral(key), token_len, token_rest) => {
                            if let Some(((Token::Colon, _, f), even_more)) = more.split_first() {
                                match next_value(even_more) {
                                    Err((e, mut still_more)) => {
                                        // find closing curly brace
                                        while let Some((head, tail)) = still_more.split_first() {
                                            if let (Token::CloseCurly, _, _) = head {
                                                return Err((e, tail));
                                            }

                                            still_more = tail;
                                        }

                                        // if it's not there, return no more tokens to continue
                                        return Err((e, &[]));
                                    }
                                    Ok((Some(value), still_more)) => {
                                        map.insert(key.to_string(), value);

                                        match still_more.split_first() {
                                            Some((
                                                (Token::Comma, comma_len, comma_start),
                                                please_stop,
                                            )) => {
                                                last_comma = (comma_len, comma_start);
                                                rest = please_stop
                                            }
                                            Some(((Token::CloseCurly, _, _), please_stop)) => {
                                                return Ok((Some(Json::Object(map)), please_stop));
                                            }
                                            Some(((_, l, f), more_rest)) => {
                                                return Err((
                                                    (ErrorKind::UnexpectedToken, *l, *f),
                                                    more_rest,
                                                ))
                                            }
                                            None => {
                                                return Err((
                                                    (ErrorKind::UnterminatedObject, 0, *f),
                                                    still_more,
                                                ))
                                            }
                                        }
                                    }
                                    Ok((None, still_more)) => {
                                        return Err((
                                            (ErrorKind::UnterminatedObject, 0, *f),
                                            still_more,
                                        ));
                                    }
                                }
                            } else {
                                return Err((
                                    (ErrorKind::UnexpectedToken, *token_len, *token_rest),
                                    more,
                                ));
                            }
                        }
                        (Token::CloseCurly, _, _) => {
                            return Err((
                                (ErrorKind::TrailingComma, *last_comma.0, *last_comma.1),
                                more,
                            ))
                        }
                        _ => return Err(((ErrorKind::UnexpectedToken, *tok_len, *tok_rest), rest)),
                    }
                }

                Err(((ErrorKind::UnexpectedToken, *tok_len, *tok_rest), rest))
            }
            ((Token::OpenSquare, _, f), mut rest) => {
                let mut vec = Vec::new();

                if let Some(((Token::CloseSquare, _, _), more)) = rest.split_first() {
                    return Ok((Some(Json::Array(vec)), more));
                }

                while !rest.is_empty() {
                    match next_value(rest) {
                        Err((e, mut more)) => {
                            // find closing square brace
                            while let Some((head, tail)) = more.split_first() {
                                if let (Token::CloseSquare, _, _) = head {
                                    return Err((e, tail));
                                }

                                more = tail;
                            }

                            // if it's not there, return no more tokens to continue
                            return Err((e, &[]));
                        }
                        Ok((Some(value), more)) => {
                            vec.push(value);

                            match more.split_first() {
                                Some(((Token::Comma, _, _), even_more)) => rest = even_more,
                                Some(((Token::CloseSquare, _, _), even_more)) => {
                                    return Ok((Some(Json::Array(vec)), even_more))
                                }
                                Some(((_, l, f), even_more)) => {
                                    return Err(((ErrorKind::UnexpectedToken, *l, *f), even_more))
                                }
                                None => return Err(((ErrorKind::UnterminatedArray, 0, *f), more)),
                            }
                        }
                        Ok((None, more)) => {
                            return Err(((ErrorKind::UnterminatedArray, 0, *f), more));
                        }
                    }
                }

                Err(((ErrorKind::UnterminatedArray, 0, *f), rest))
            }
            ((_, l, f), rest) => Err(((ErrorKind::UnexpectedToken, *l, *f), rest)),
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
    type Err = Vec<(ErrorKind, Range<usize>)>;

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
        let mut errvec = Vec::new();

        while !s.is_empty() {
            match next_token(s) {
                Err(e) => {
                    s = &s[s.len() - e.2 + e.1..];
                    errvec.push(calc_range(e));
                }
                Ok((Some(t), s_)) => {
                    tokens.push(t);
                    s = s_;
                }
                Ok((None, _)) => break,
            }
        }

        if !errvec.is_empty() {
            return Err(errvec);
        }

        let mut toks = &tokens[..];
        let mut values = Vec::new();

        while !toks.is_empty() {
            match next_value(toks) {
                Err((e, t_)) => {
                    errvec.push(calc_range(e));
                    toks = t_;
                }
                Ok((Some(v), t_)) => {
                    values.push(v);
                    toks = t_;
                }
                Ok((None, _)) => break,
            }
        }

        if !errvec.is_empty() {
            return Err(errvec);
        }

        if values.len() == 1 {
            Ok(values.remove(0))
        } else {
            Err(vec![calc_range((
                ErrorKind::UnexpectedToken,
                s.len(),
                s.len(),
            ))])
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
    pub fn print<W: io::Write>(&self, spacing: &Spacing, f: &mut W) -> io::Result<()> {
        match spacing {
            Spacing::None => f.write_fmt(format_args!("{}", self)),
            Spacing::Tab => self.print_tabs(0, f),
            Spacing::Space(n) => self.print_indented(*n, 0, f),
        }
    }

    fn print_indented<W: io::Write>(
        &self,
        spacing: usize,
        start: usize,
        f: &mut W,
    ) -> io::Result<()> {
        match self {
            Json::Array(a) if a.is_empty() => f.write_fmt(format_args!("[]")),
            Json::Array(a) if a.len() == 1 => f.write_fmt(format_args!("[ {} ]", a[0])),
            Json::Object(o) if o.is_empty() => f.write_fmt(format_args!("{{}}")),

            Json::Array(a) => {
                f.write_fmt(format_args!("[\n"))?;

                for (i, el) in a.iter().enumerate() {
                    for _ in 0..start + spacing {
                        f.write_fmt(format_args!(" ",))?;
                    }

                    el.print_indented(spacing, start + spacing, f)?;

                    if i != a.len() - 1 {
                        f.write_fmt(format_args!(","))?;
                    }

                    writeln!(f)?;
                }

                for _ in 0..start {
                    f.write_fmt(format_args!(" ",))?;
                }
                f.write_fmt(format_args!("]"))
            }
            Json::Object(o) => {
                f.write_fmt(format_args!("{{\n"))?;

                for (i, (k, v)) in o.iter().enumerate() {
                    for _ in 0..start + spacing {
                        f.write_fmt(format_args!(" ",))?;
                    }

                    f.write_fmt(format_args!("\"{}\": ", k))?;
                    v.print_indented(spacing, start + spacing, f)?;

                    if i != o.len() - 1 {
                        f.write_fmt(format_args!(","))?;
                    }
                    writeln!(f)?;
                }

                for _ in 0..start {
                    f.write_fmt(format_args!(" ",))?;
                }
                f.write_fmt(format_args!("}}"))
            }

            _ => f.write_fmt(format_args!("{}", self)),
        }
    }

    fn print_tabs<W: io::Write>(&self, start: usize, f: &mut W) -> io::Result<()> {
        match self {
            Json::Array(a) if a.is_empty() => f.write_fmt(format_args!("[]")),
            Json::Array(a) if a.len() == 1 => f.write_fmt(format_args!("[ {} ]", a[0])),
            Json::Object(o) if o.is_empty() => f.write_fmt(format_args!("{{}}")),

            Json::Array(a) => {
                f.write_fmt(format_args!("[\n"))?;

                for (i, el) in a.iter().enumerate() {
                    for _ in 0..=start {
                        f.write_fmt(format_args!("\t",))?;
                    }

                    el.print_tabs(start + 1, f)?;

                    if i != a.len() - 1 {
                        f.write_fmt(format_args!(","))?;
                    }

                    f.write_fmt(format_args!("\n"))?;
                }

                for _ in 0..start {
                    f.write_fmt(format_args!("\t",))?;
                }
                f.write_fmt(format_args!("]"))
            }
            Json::Object(o) => {
                f.write_fmt(format_args!("{{\n"))?;

                for (i, (k, v)) in o.iter().enumerate() {
                    for _ in 0..=start {
                        f.write_fmt(format_args!("\t",))?;
                    }

                    f.write_fmt(format_args!("\"{}\": ", k))?;
                    v.print_tabs(start + 1, f)?;

                    if i != o.len() - 1 {
                        f.write_fmt(format_args!(","))?;
                    }

                    f.write_fmt(format_args!("\n"))?;
                }

                for _ in 0..start {
                    f.write_fmt(format_args!("\t",))?;
                }
                f.write_fmt(format_args!("}}"))
            }

            _ => f.write_fmt(format_args!("{}", self)),
        }
    }
}
