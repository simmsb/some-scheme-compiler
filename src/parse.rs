use std::boxed::Box;
use std::borrow::Cow;
use std::fmt;

use nom;
use itertools::Itertools;

#[derive(Debug)]
pub enum LExpr<'a> {
    Lam(Vec<Cow<'a, str>>, Vec<LExpr<'a>>),
    App(Box<LExpr<'a>>, Vec<LExpr<'a>>),
    Var(Cow<'a, str>),

    /// A lambda with no parameters
    LamNone(Vec<LExpr<'a>>),
    LamNoneOne(Box<LExpr<'a>>),
    LamNoneNone, // for (lambda ())

    /// A lambda expanded to one parameter
    LamOne(Cow<'a, str>, Vec<LExpr<'a>>),
    LamOneOne(Cow<'a, str>, Box<LExpr<'a>>),
    LamOneNone(Cow<'a, str>), // for (lambda (_))

    /// An application with zero arguments
    AppNone(Box<LExpr<'a>>),

    /// An application expanded to only one argument
    AppOne(Box<LExpr<'a>>, Box<LExpr<'a>>),
}

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


impl<'a> fmt::Display for LExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use parse::LExpr::*;

        match self {
            Lam(args, body) => {
                write!(f, "(lambda ({})", args.iter().join(" "))?;
                for expr in body {
                    write!(f, " {}", expr)?;
                }
                write!(f, ")")
            },
            App(box operator, operands) => {
                write!(f, "({}", operator)?;
                for operand in operands {
                    write!(f, " {}", operand)?;
                }
                write!(f, ")")
            },
            Var(name) =>
                write!(f, "{}", name),
            LamNone(body) => {
                write!(f, "(lambda ()")?;
                for expr in body {
                    write!(f, " {}", expr)?;
                }
                write!(f, ")")
            },
            LamNoneOne(box expr) =>
                write!(f, "(lambda () {})", expr),
            LamNoneNone =>
                write!(f, "(lambda ())"),
            LamOne(arg, body) => {
                write!(f, "(lambda ({})", arg)?;
                for expr in body {
                    write!(f, " {}", expr)?;
                }
                write!(f, ")")
            },
            LamOneOne(arg, box expr) =>
                write!(f, "(lambda ({}) {})", arg, expr),
            LamOneNone(arg) =>
                write!(f, "(lambda ({}))", arg),
            AppNone(box operator) =>
                write!(f, "({})", operator),
            AppOne(box operator, box operands) =>
                write!(f, "({} {})", operator, operands),
        }
    }
}
