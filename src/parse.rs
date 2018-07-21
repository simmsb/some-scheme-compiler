use std::borrow::Cow;
use nom;

use nodes::{LExpr, ExprLit};

fn ident_char(chr: char) -> bool {
    !" ()\n\r".contains(chr)
}

pub fn parse_int(input: &str) -> nom::IResult<&str, LExpr> {
    map_res!(input,
        pair!(opt!(char!('-')), nom::digit),
        | (sign, num) : (Option<char>, &str) | num.parse::<i64>()
             .map(|n| if sign.is_some() { -n } else { n })
             .map(|n| LExpr::Lit(ExprLit::NumLit(n))))
}

pub fn parse_ident(input: &str) -> nom::IResult<&str, Cow<str>> {
    map!(input, take_while1!(ident_char), Cow::from)
}

pub fn parse_var(input: &str) -> nom::IResult<&str, LExpr> {
    map!(input, ws!(parse_ident), LExpr::Var)
}

#[allow(cyclomatic_complexity)]
fn parse_lam(input: &str) -> nom::IResult<&str, LExpr> {
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

pub fn parse_app(input: &str) -> nom::IResult<&str, LExpr> {
    do_parse!(input,
        char!('(') >>
        rand: parse_exp >>
        rator: ws!(many0!(parse_exp)) >>
        char!(')') >>
        (LExpr::App(box rand, rator))
    )
}

pub fn parse_exp(input: &str) -> nom::IResult<&str, LExpr> {
    alt_complete!(input, parse_int | parse_var | parse_lam | parse_app)
}


