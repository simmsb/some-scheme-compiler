use std::borrow::Cow;
use nom;

use nodes::{LExpr, ExprLit};

fn ident_char(chr: char) -> bool {
    return !" ()\n\r".contains(chr);
}

pub fn parse_int<'a>(input: &'a str) -> nom::IResult<&'a str, LExpr<'a>> {
    map_res!(input,
        pair!(opt!(char!('-')), nom::digit),
        | (sign, num) : (Option<char>, &str) | num.parse::<i64>()
             .map(|n| if sign.is_some() { -n } else { n })
             .map(|n| LExpr::Lit(ExprLit::NumLit(n))))
}

pub fn parse_ident<'a>(input: &'a str) -> nom::IResult<&'a str, Cow<'a, str>> {
    map!(input, take_while1!(ident_char), Cow::from)
}

pub fn parse_var<'a>(input: &'a str) -> nom::IResult<&'a str, LExpr<'a>> {
    map!(input, ws!(parse_ident), LExpr::Var)
}

fn parse_lam<'a>(input: &'a str) -> nom::IResult<&'a str, LExpr<'a>> {
    do_parse!(input,
        char!('(') >>
        ws!(tag!("lambda")) >>
        char!('(') >>
        plist: ws!(many0!(parse_ident)) >>
        char!(')') >>
        body: ws!(many0!(parse_exp)) >>
        char!(')') >>

        (LExpr::Lam(plist, body))
    )
}

pub fn parse_app<'a>(input: &'a str) -> nom::IResult<&'a str, LExpr<'a>> {
    do_parse!(input,
        char!('(') >>
        rand: parse_exp >>
        rator: ws!(many0!(parse_exp)) >>
        char!(')') >>
        (LExpr::App(box rand, rator))
    )
}

pub fn parse_exp<'a>(input: &'a str) -> nom::IResult<&'a str, LExpr<'a>> {
    alt_complete!(input, parse_int | parse_var | parse_lam | parse_app)
}


