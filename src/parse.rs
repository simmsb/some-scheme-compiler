use std::borrow::Cow;
use nom;

use nodes::LExpr;

fn ident_char(chr: char) -> bool {
    use nom::*;

    return is_alphanumeric(chr as u8); // | r"-_|/\".contains(chr);
}

pub fn parse_ident<'a>(input: &'a str) -> nom::IResult<&'a str, Cow<'a, str>> {
    map!(input, take_while1!(ident_char), Cow::Borrowed)
}

fn parse_var<'a>(input: &'a str) -> nom::IResult<&'a str, LExpr<'a>> {
    do_parse!(input,
        name: parse_ident >>
        (LExpr::Var(name))
    )
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
    alt!(input, parse_lam | parse_app | parse_var)
}


