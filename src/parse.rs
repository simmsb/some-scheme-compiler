use nom::character::complete::multispace0;
use nom::complete;
use nom::{alt, char};
use nom::{delimited, escaped, is_not, map, map_res};
use nom::{many0, one_of, opt, pair, tag, take_while1};
use std::rc::Rc;

use crate::literals::Literal;
use crate::base_expr::BExpr;

fn ident_char(chr: char) -> bool {
    !" ()\n\r\"\'".contains(chr)
}

macro_rules! ws {
    ($i:expr, $($args:tt)*) => {
        delimited!($i, multispace0, $($args)*, multispace0)
    };
}

pub fn parse_int(input: &str) -> nom::IResult<&str, BExpr> {
    map_res!(
        input,
        pair!(opt!(char!('-')), take_while1!(|c: char| c.is_digit(10))),
        |(sign, num): (Option<char>, &str)| num
            .parse::<i64>()
            .map(|n| if sign.is_some() { -n } else { n })
            .map(|n| BExpr::Lit(Literal::Int(n)))
    )
}

pub fn parse_ident(input: &str) -> nom::IResult<&str, String> {
    map!(input, take_while1!(ident_char), String::from)
}

pub fn parse_var(input: &str) -> nom::IResult<&str, BExpr> {
    map!(input, ws!(parse_ident), BExpr::Var)
}

pub fn parse_builtin(input: &str) -> nom::IResult<&str, BExpr> {
    let builtin_names = [
        "to_string",
        "println",
        "+",
        "-",
        "*",
        "/",
    ];

    for &name in &builtin_names {
        if let Ok((i, _n)) = nom::bytes::complete::tag::<_, _, (_, nom::error::ErrorKind)>(name)(input) {
            return Ok((i, BExpr::BuiltinIdent(name.to_owned())));
        }
    }

    if let Ok((i, _n)) = nom::bytes::complete::tag::<_, _, (_, nom::error::ErrorKind)>("void")(input) {
        return Ok((i, BExpr::Lit(Literal::Void)))
    }

    Err(nom::Err::Error((input, nom::error::ErrorKind::Tag)))
}

pub fn parse_str(input: &str) -> nom::IResult<&str, BExpr> {
    map!(
        input,
        delimited!(
            tag!("\""),
            escaped!(is_not!("\""), '\\', one_of!("\"n\\")),
            tag!("\"")
        ),
        |s| BExpr::Lit(Literal::String(s.to_owned()))
    )
}

fn parse_lam(input: &str) -> nom::IResult<&str, BExpr> {
    let (i, _) = pair!(input, char!('('), ws!(tag!("lambda")))?;
    let (i, plist) = delimited!(i, char!('('), many0!(ws!(parse_ident)), char!(')'))?;
    let (i, body) = ws!(i, many0!(ws!(parse_exp)))?;
    let (i, _) = char!(i, ')')?;

    Ok((i, BExpr::Lam(plist, body)))
}

pub fn parse_app(input: &str) -> nom::IResult<&str, BExpr> {
    let (i, _) = char!(input, '(')?;
    let (i, (function, params)) = pair!(i, parse_exp, many0!(ws!(parse_exp)))?;
    let (i, _) = char!(i, ')')?;

    Ok((i, BExpr::App(Rc::new(function), params)))
}

pub fn parse_exp(input: &str) -> nom::IResult<&str, BExpr> {
    alt!(
        input,
        complete!(parse_int)
            | complete!(parse_str)
            | complete!(parse_builtin)
            | complete!(parse_var)
            | complete!(parse_lam)
            | complete!(parse_app)
    )
}
