use nom::character::complete::multispace0;
use nom::complete;
use nom::{alt, char};
use nom::{delimited, escaped, is_not, map, map_res};
use nom::{many0, one_of, opt, pair, tag, take_while1};
use std::borrow::Cow;

use crate::nodes::{ExprLit, LExpr};

fn ident_char(chr: char) -> bool {
    !" ()\n\r\"\'".contains(chr)
}

macro_rules! ws {
    ($i:expr, $($args:tt)*) => {
        delimited!($i, multispace0, $($args)*, multispace0)
    };
}

pub fn parse_int(input: &str) -> nom::IResult<&str, LExpr<'_>> {
    map_res!(
        input,
        pair!(opt!(char!('-')), take_while1!(|c: char| c.is_digit(10))),
        |(sign, num): (Option<char>, &str)| num
            .parse::<i64>()
            .map(|n| if sign.is_some() { -n } else { n })
            .map(|n| LExpr::Lit(ExprLit::NumLit(n)))
    )
}

pub fn parse_ident(input: &str) -> nom::IResult<&str, Cow<'_, str>> {
    map!(input, take_while1!(ident_char), Cow::from)
}

pub fn parse_var(input: &str) -> nom::IResult<&str, LExpr<'_>> {
    map!(input, ws!(parse_ident), LExpr::Var)
}

pub fn parse_str(input: &str) -> nom::IResult<&str, LExpr<'_>> {
    map!(
        input,
        delimited!(
            tag!("\""),
            escaped!(is_not!("\""), '\\', one_of!("\"n\\")),
            tag!("\"")
        ),
        |s| LExpr::Lit(ExprLit::StringLit(s.into()))
    )
}

#[allow(clippy::cyclomatic_complexity)]
fn parse_lam(input: &str) -> nom::IResult<&str, LExpr<'_>> {
    let (i, _) = pair!(input, char!('('), ws!(tag!("lambda")))?;
    let (i, plist) = delimited!(i, char!('('), many0!(ws!(parse_ident)), char!(')'))?;
    let (i, body) = ws!(i, many0!(ws!(parse_exp)))?;
    let (i, _) = char!(i, ')')?;

    Ok((i, LExpr::Lam(plist, body)))
}

pub fn parse_app(input: &str) -> nom::IResult<&str, LExpr<'_>> {
    let (i, _) = char!(input, '(')?;
    let (i, (rand, rator)) = pair!(i, parse_exp, many0!(ws!(parse_exp)))?;
    let (i, _) = char!(i, ')')?;

    Ok((i, LExpr::App(box rand, rator)))
}

pub fn parse_exp(input: &str) -> nom::IResult<&str, LExpr<'_>> {
    alt!(
        input,
        complete!(parse_int)
            | complete!(parse_str)
            | complete!(parse_var)
            | complete!(parse_lam)
            | complete!(parse_app)
    )
}
