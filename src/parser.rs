use crate::ast::*;
use crate::prim::*;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, digit1, multispace0, multispace1};
use nom::combinator::{all_consuming, complete, map, opt, recognize, value, verify};
use nom::multi::{many1, separated_list, separated_nonempty_list};
use nom::number::complete::recognize_float;
use nom::sequence::tuple;
use nom::IResult;

static KEYWORDS: &[&str] = &[
    "val", "fun", "fn", "let", "in", "end", "if", "then", "else", "case", "of", "_", "datatype",
    "op", "=>",
];

static RESERVED: &[&str] = &["|", "=", "#"];

static INFIX9: &[&str] = &[];
static INFIX8: &[&str] = &[];
static INFIX7: &[&str] = &["*", "/", "div", "mod"];
static INFIX6: &[&str] = &["+", "-"];
static INFIX5: &[&str] = &[];
static INFIX4: &[&str] = &["=", "<>", "<=", "<", ">=", ">"];
static INFIX3: &[&str] = &[];
static INFIX2: &[&str] = &[];
static INFIX1: &[&str] = &[];

static FIXTY_TABLE: [&[&str]; 9] = [
    INFIX1, INFIX2, INFIX3, INFIX4, INFIX5, INFIX6, INFIX7, INFIX8, INFIX9,
];

fn top(i: &str) -> IResult<&str, UntypedAst> {
    let (i, _) = multispace0(i)?;
    let (i, tops) = separated_list(multispace1, bind)(i)?;
    let (i, _) = multispace0(i)?;
    Ok((i, AST(tops)))
}

fn bind(i: &str) -> IResult<&str, Statement<()>> {
    alt((bind_datatype, bind_val, bind_fun))(i)
}

fn bind_datatype(i: &str) -> IResult<&str, Statement<()>> {
    let (i, _) = tag("datatype")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, name) = symbol(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("=")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, constructors) =
        separated_nonempty_list(tuple((multispace0, tag("|"), multispace0)), constructor_def)(i)?;
    Ok((i, Statement::Datatype { name, constructors }))
}

fn bind_val(i: &str) -> IResult<&str, Statement<()>> {
    let (i, _) = tag("val")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, pattern) = pattern(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("=")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, expr) = expr(i)?;
    Ok((
        i,
        Statement::Val {
            rec: false,
            pattern,
            expr,
        },
    ))
}

fn bind_fun(i: &str) -> IResult<&str, Statement<()>> {
    let (i, _) = tag("fun")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, cs) = separated_nonempty_list(
        tuple((multispace0, tag("|"), multispace0)),
        map(
            tuple((
                symbol,
                multispace0,
                separated_nonempty_list(multispace1, pattern_atmic),
                multispace0,
                tag("="),
                multispace0,
                expr,
            )),
            |(name, _, params, _, _, _, e)| (name, params, e),
        ),
    )(i)?;
    let mut cs = cs.into_iter();
    let (name, params, expr) = cs.next().expect("nonempty list empty");
    let mut clauses = vec![(params, expr)];
    for (new_name, params, expr) in cs {
        if name != new_name {
            return Err(nom::Err::Error((i, nom::error::ErrorKind::Tag)));
        }
        clauses.push((params, expr))
    }
    Ok((i, Statement::D(DerivedStatement::Fun { name, clauses })))
}

fn constructor_def(i: &str) -> IResult<&str, (Symbol, Option<Type>)> {
    let (i, name) = symbol(i)?;
    let (i, param) = opt(complete(map(
        tuple((multispace1, tag("of"), multispace1, typename)),
        |(_, _, _, ty)| ty,
    )))(i)?;

    Ok((i, (name, param)))
}
fn expr(i: &str) -> IResult<&str, Expr<()>> {
    alt((expr_bind, expr_fun, expr_if, expr_case, expr_infix_and_app))(i)
}

fn expr1(i: &str) -> IResult<&str, Expr<()>> {
    alt((
        expr1_tuple,
        expr1_paren,
        expr1_float,
        expr1_int,
        expr1_bool,
        expr1_sym,
        expr1_builtincall,
    ))(i)
}

fn expr_bind(i: &str) -> IResult<&str, Expr<()>> {
    let (i, _) = tag("let")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, binds) = separated_list(multispace1, bind)(i)?;
    let (i, _) = multispace1(i)?;
    let (i, _) = tag("in")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, ret) = expr(i)?;
    let (i, _) = multispace1(i)?;
    let (i, _) = tag("end")(i)?;
    Ok((
        i,
        Expr::Binds {
            ty: (),
            binds: binds,
            ret: ret.boxed(),
        },
    ))
}

fn expr_fun(i: &str) -> IResult<&str, Expr<()>> {
    let (i, _) = tag("fn")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, param) = symbol(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("=>")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, body) = expr(i)?;
    Ok((
        i,
        Expr::Fn {
            ty: (),
            param: param,
            body: body.boxed(),
        },
    ))
}

fn expr_if(i: &str) -> IResult<&str, Expr<()>> {
    let (i, _) = tag("if")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, cond) = expr(i)?;
    let (i, _) = multispace1(i)?;
    let (i, _) = tag("then")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, then) = expr(i)?;
    let (i, _) = multispace1(i)?;
    let (i, _) = tag("else")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, else_) = expr(i)?;
    Ok((
        i,
        Expr::D(DerivedExpr::If {
            ty: (),
            cond: cond.boxed(),
            then: then.boxed(),
            else_: else_.boxed(),
        }),
    ))
}

fn expr_case(i: &str) -> IResult<&str, Expr<()>> {
    let (i, _) = tag("case")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, cond) = expr(i)?;
    let (i, _) = multispace1(i)?;
    let (i, _) = tag("of")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, clauses) = separated_nonempty_list(
        tuple((multispace0, tag("|"), multispace0)),
        map(
            tuple((pattern, multispace0, tag("=>"), multispace0, expr)),
            |(pat, _, _, _, expr)| (pat, expr),
        ),
    )(i)?;
    Ok((
        i,
        Expr::Case {
            ty: (),
            cond: cond.boxed(),
            clauses: clauses,
        },
    ))
}

fn map_window2<I>(
    iter: I,
    mut f: impl FnMut(I::Item, I::Item) -> (I::Item, Option<I::Item>),
) -> Vec<I::Item>
where
    I: IntoIterator,
{
    let mut ret = vec![];
    let mut iter = iter.into_iter();
    let mut e = match iter.next() {
        Some(e) => e,
        None => return ret,
    };

    while let Some(e2) = iter.next() {
        let (e1, e2) = f(e, e2);
        match e2 {
            Some(e2) => {
                ret.push(e1);
                e = e2;
            }
            None => e = e1,
        }
    }
    ret.push(e);
    ret
}

fn map_window3<I>(
    iter: I,
    mut f: impl FnMut(I::Item, I::Item, I::Item) -> (I::Item, Option<(I::Item, I::Item)>),
) -> Vec<I::Item>
where
    I: IntoIterator,
{
    let mut ret = vec![];
    let mut iter = iter.into_iter();

    let mut e1 = match iter.next() {
        Some(e) => e,
        None => return ret,
    };

    let mut e2 = match iter.next() {
        Some(e) => e,
        None => {
            ret.push(e1);
            return ret;
        }
    };

    while let Some(e3) = iter.next() {
        let (r1, r2) = f(e1, e2, e3);
        match r2 {
            Some((r2, r3)) => {
                ret.push(r1);
                e1 = r2;
                e2 = r3;
            }
            None => {
                e1 = r1;
                e2 = match iter.next() {
                    Some(e) => e,
                    None => {
                        ret.push(e1);
                        return ret;
                    }
                };
            }
        }
    }
    ret.push(e1);
    ret.push(e2);
    ret
}

// treat all of the infix operators and applications, i.e. sequeces of expressions
fn expr_infix_and_app(i: &str) -> IResult<&str, Expr<()>> {
    // TODO: support 1+1
    let (i, mixed) = separated_nonempty_list(multispace1, expr1)(i)?;
    #[derive(Debug)]
    enum Mixed {
        E(Expr<()>),
        Fix(u8, Symbol),
    }
    use Mixed::*;
    // find infixes
    let mixed = mixed
        .into_iter()
        .map(|e| match e {
            Expr::Symbol { ty, name } => {
                for (f, table) in FIXTY_TABLE.iter().enumerate() {
                    if table.contains(&name.0.as_str()) {
                        return Fix(f as u8 + 1, name);
                    }
                }
                E(Expr::Symbol { ty, name })
            }
            e => E(e),
        })
        .collect::<Vec<_>>();
    // reduce applys
    let rest = map_window2(mixed, |m1, m2| match (m1, m2) {
        (E(e1), E(e2)) => (
            E(Expr::App {
                ty: (),
                fun: e1.boxed(),
                arg: e2.boxed(),
            }),
            None,
        ),
        (m1, m2) => (m1, Some(m2)),
    });

    // reduce infixes
    fn reduce_infixl_n(n: u8, mixed: Vec<Mixed>) -> Vec<Mixed> {
        use Mixed::*;
        map_window3(mixed, |m1, m2, m3| match (m1, m2, m3) {
            (E(l), Fix(fixty, op), E(r)) if fixty == n => (
                E(Expr::BinOp {
                    ty: (),
                    op,
                    l: l.boxed(),
                    r: r.boxed(),
                }),
                None,
            ),
            (m1, m2, m3) => (m1, Some((m2, m3))),
        })
    }
    let mut rest = (1u8..=9)
        .rev()
        .fold(rest, |rest, n| reduce_infixl_n(n, rest));
    assert_eq!(rest.len(), 1);
    let e = match rest.remove(0) {
        E(e) => e,
        Fix(..) => unreachable!("infix alone"),
    };
    Ok((i, e))
}

#[test]
fn test_expr_infix_and_app() {
    let input = "true";
    let ret = expr_infix_and_app(input).unwrap();
    assert_eq!(
        ret,
        (
            "",
            Expr::Constructor {
                ty: (),
                arg: None,
                name: Symbol::new("true")
            }
        )
    )
}

#[test]
fn test_expr_infix_and_app2() {
    let input = "f arg";
    let ret = expr_infix_and_app(input).unwrap();
    assert_eq!(
        ret,
        (
            "",
            Expr::App {
                ty: (),
                fun: Expr::Symbol {
                    ty: (),
                    name: Symbol::new("f"),
                }
                .boxed(),
                arg: Expr::Symbol {
                    ty: (),
                    name: Symbol::new("arg"),
                }
                .boxed()
            }
        )
    )
}

fn expr1_sym(i: &str) -> IResult<&str, Expr<()>> {
    // = is allowed to be used in expression exceptionally
    map(alt((symbol, map(tag("="), Symbol::new))), |s| {
        Expr::Symbol { ty: (), name: s }
    })(i)
}

fn expr1_int(i: &str) -> IResult<&str, Expr<()>> {
    map(digit1, |s: &str| Expr::Literal {
        ty: (),
        value: Literal::Int(s.parse().unwrap()),
    })(i)
}

fn expr1_float(i: &str) -> IResult<&str, Expr<()>> {
    let not_int = verify(recognize_float, |s: &&str| s.contains('.'));

    map(not_int, |s: &str| Expr::Literal {
        ty: (),
        value: Literal::Real(s.parse().unwrap()),
    })(i)
}

fn expr1_bool(i: &str) -> IResult<&str, Expr<()>> {
    alt((
        value(
            Expr::Constructor {
                name: Symbol::new("true"),
                arg: None,
                ty: (),
            },
            tag("true"),
        ),
        value(
            Expr::Constructor {
                name: Symbol::new("false"),
                arg: None,
                ty: (),
            },
            tag("false"),
        ),
    ))(i)
}

fn expr1_paren(i: &str) -> IResult<&str, Expr<()>> {
    let (i, _) = tag("(")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, e) = expr(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(")")(i)?;
    Ok((i, e))
}

fn expr1_tuple(i: &str) -> IResult<&str, Expr<()>> {
    let (i, _) = tag("(")(i)?;
    let (i, _) = multispace0(i)?;
    let sep = tuple((multispace0, tag(","), multispace0));
    let (i, es) = many1(map(tuple((expr, sep)), |(e, _)| e))(i)?;
    let (i, e) = expr(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(")")(i)?;

    let mut es = es;
    es.push(e);
    Ok((i, Expr::Tuple { ty: (), tuple: es }))
}

fn expr1_builtincall(i: &str) -> IResult<&str, Expr<()>> {
    let (i, _) = tag("_builtincall")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("\"")(i)?;
    let (i, name) = alphanumeric1(i)?;
    let (i, _) = tag("\"")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("(")(i)?;
    let (i, args) = separated_nonempty_list(tuple((multispace0, tag(","), multispace0)), expr)(i)?;
    let (i, _) = tag(")")(i)?;
    Ok((
        i,
        Expr::BuiltinCall {
            ty: (),
            name: name.to_string(),
            args,
        },
    ))
}

fn typename(i: &str) -> IResult<&str, Type> {
    typename0(i)
}

fn typename0(i: &str) -> IResult<&str, Type> {
    alt((complete(typename0_fun), typename1))(i)
}

fn typename1(i: &str) -> IResult<&str, Type> {
    alt((typename1_tuple, typename2))(i)
}

fn typename2(i: &str) -> IResult<&str, Type> {
    alt((
        typename2_int,
        typename2_real,
        typename2_unit,
        typename2_paren,
        typename2_datatype,
    ))(i)
}

fn typename0_fun(i: &str) -> IResult<&str, Type> {
    let (i, arg) = typename1(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("->")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, ret) = typename(i)?;
    Ok((i, Type::Fun(Box::new(arg), Box::new(ret))))
}

fn typename1_tuple(i: &str) -> IResult<&str, Type> {
    let sep = tuple((multispace0, tag("*"), multispace0));

    let (i, tys) = many1(map(tuple((typename2, sep)), |(ty, _)| ty))(i)?;
    let (i, ty) = typename2(i)?;

    let mut tys = tys;
    tys.push(ty);
    Ok((i, Type::Tuple(tys)))
}

fn typename2_int(i: &str) -> IResult<&str, Type> {
    map(tag("int"), |_| Type::Int)(i)
}

fn typename2_real(i: &str) -> IResult<&str, Type> {
    map(tag("real"), |_| Type::Real)(i)
}

fn typename2_unit(i: &str) -> IResult<&str, Type> {
    map(tag("()"), |_| Type::Tuple(vec![]))(i)
}

fn typename2_paren(i: &str) -> IResult<&str, Type> {
    let (i, _) = tag("(")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, ty) = typename(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(")")(i)?;
    Ok((i, ty))
}

fn typename2_datatype(i: &str) -> IResult<&str, Type> {
    map(symbol, |name| Type::Datatype(name))(i)
}

fn symbol(i: &str) -> IResult<&str, Symbol> {
    alt((symbol_alphanumeric, symbol_symbolic))(i)
}

fn symbol_alphanumeric(i: &str) -> IResult<&str, Symbol> {
    // FIXME: collect syntax is [a-zA-Z'_][a-zA-Z'_0-9]*
    let (i, sym) = verify(alphanumeric1, |s: &str| !KEYWORDS.contains(&s))(i)?;
    Ok((i, Symbol::new(sym.to_string())))
}

fn symbol_symbolic(i: &str) -> IResult<&str, Symbol> {
    let symbolic1 = recognize(many1(nom::character::complete::one_of(
        "!%&$#+-/:<=>?@\\~'^|*",
    )));

    let (i, sym) = verify(symbolic1, |s: &str| {
        !KEYWORDS.contains(&s) && !RESERVED.contains(&s)
    })(i)?;
    Ok((i, Symbol::new(sym.to_string())))
}

fn pattern(i: &str) -> IResult<&str, Pattern<()>> {
    alt((pattern_constructor, pattern_atmic))(i)
}

fn pattern_atmic(i: &str) -> IResult<&str, Pattern<()>> {
    alt((
        pattern_bool,
        pattern_int,
        pattern_tuple,
        pattern_var,
        pattern_wildcard,
        pattern_paren,
    ))(i)
}

fn pattern_bool(i: &str) -> IResult<&str, Pattern<()>> {
    alt((
        map(tag("true"), |_| Pattern::Constructor {
            name: Symbol::new("true"),
            arg: None,
            ty: (),
        }),
        map(tag("false"), |_| Pattern::Constructor {
            name: Symbol::new("false"),
            arg: None,
            ty: (),
        }),
    ))(i)
}

fn pattern_int(i: &str) -> IResult<&str, Pattern<()>> {
    map(digit1, |s: &str| Pattern::Constant {
        ty: (),
        value: s.parse().unwrap(),
    })(i)
}

fn pattern_tuple(i: &str) -> IResult<&str, Pattern<()>> {
    let (i, _) = tag("(")(i)?;
    let (i, _) = multispace0(i)?;
    let sep = tuple((multispace0, tag(","), multispace0));
    let (i, es) = many1(map(tuple((pattern, sep)), |(e, _)| e))(i)?;
    let (i, e) = pattern(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(")")(i)?;

    let mut es = es;
    es.push(e);
    Ok((i, Pattern::Tuple { tuple: es, ty: () }))
}

// require constructor to have arg for now.
// it will be converted in later phases
fn pattern_constructor(i: &str) -> IResult<&str, Pattern<()>> {
    let (i, name) = symbol(i)?;
    let (i, _) = multispace1(i)?;
    let (i, arg) = pattern_atmic(i)?;
    Ok((
        i,
        Pattern::Constructor {
            name,
            arg: Some(Box::new(arg)),
            ty: (),
        },
    ))
}

fn pattern_var(i: &str) -> IResult<&str, Pattern<()>> {
    map(symbol, |name| Pattern::Variable { name: name, ty: () })(i)
}

fn pattern_wildcard(i: &str) -> IResult<&str, Pattern<()>> {
    value(Pattern::Wildcard { ty: () }, tag("_"))(i)
}

fn pattern_paren(i: &str) -> IResult<&str, Pattern<()>> {
    let (i, _) = tag("(")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, e) = pattern(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(")")(i)?;

    Ok((i, e))
}

pub fn parse(
    input: &str,
) -> ::std::result::Result<UntypedAst, nom::Err<(&str, nom::error::ErrorKind)>> {
    let (_, iresult) = all_consuming(top)(input)?;
    Ok(iresult)
}
