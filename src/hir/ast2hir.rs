use ::ast;
use pass::Pass;
use prim::*;
use hir::{HIR, Expr, Val};

pub struct AST2HIR;

impl AST2HIR {
    fn conv_ast(&self, ast: ast::AST) -> HIR {
        HIR(ast.0.into_iter().map(|val| self.conv_val(val)).collect())
    }

    fn conv_val(&self, val: ast::Val) -> Val {
        Val {
            ty: val.ty.force("internal typing error"),
            rec: val.rec,
            name: val.name,
            expr: self.conv_expr(val.expr),
        }
    }

    fn conv_expr(&self, expr: ast::Expr) -> Expr {
        use ast::Expr as E;
        match expr {
            E::Binds { ty, binds, ret } => {
                Expr::Binds {
                    ty: ty.force("internal typing error"),
                    binds: binds.into_iter().map(|b| self.conv_val(b)).collect(),
                    ret: Box::new(self.conv_expr(*ret)),
                }
            }
            E::Add { ty, l, r } => {
                Expr::Op {
                    ty: ty.force("internal typing error"),
                    name: Symbol("+".to_string()),
                    l: Box::new(self.conv_expr(*l)),
                    r: Box::new(self.conv_expr(*r)),
                }
            }
            E::Mul { ty, l, r } => {
                Expr::Op {
                    ty: ty.force("internal typing error"),
                    name: Symbol("*".to_string()),
                    l: Box::new(self.conv_expr(*l)),
                    r: Box::new(self.conv_expr(*r)),
                }
            }
            E::Fun { param_ty, param, body_ty, body } => {
                Expr::Fun {
                    param: (param_ty.force("internal typing error"), param),
                    body_ty: body_ty.force("internal typing error"),
                    body: Box::new(self.conv_expr(*body)),
                    captures: Vec::new(),
                }
            }
            E::App { ty, fun, arg } => {
                self.conv_expr(*fun).app1(ty.force("internal typing error"), self.conv_expr(*arg))
            }
            E::If { ty, cond, then, else_ } => {
                Expr::If {
                    ty: ty.force("internal typing error"),
                    cond: Box::new(self.conv_expr(*cond)),
                    then: Box::new(self.conv_expr(*then)),
                    else_: Box::new(self.conv_expr(*else_)),
                }
            }
            E::Sym { ty, name } => {
                Expr::Sym {
                    ty: ty.force("internal typing error"),
                    name: name,
                }
            }
            E::Lit { ty, value } => {
                Expr::Lit {
                    ty: ty.force("internal typing error"),
                    value: value,
                }
            }
        }
    }
}

impl Pass<ast::AST> for AST2HIR {
    type Target = HIR;
    type Err = TypeError;

    fn trans(&mut self, ast: ast::AST) -> ::std::result::Result<Self::Target, Self::Err> {
        Ok(self.conv_ast(ast))
    }
}