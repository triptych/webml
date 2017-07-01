use hir::*;



pub trait Traverse {
    fn traverse_hir(&mut self, hir: &mut HIR) {
        for val in hir.0.iter_mut() {
            self.traverse_val(val)
        }
    }

    fn traverse_val(&mut self, val: &mut Val) {
        self.traverse_expr(&mut val.expr)
    }

    fn traverse_expr(&mut self, expr: &mut Expr) {
        use hir::Expr::*;
        match *expr {
            Binds {
                ref mut ty,
                ref mut binds,
                ref mut ret,
            } => self.traverse_binds(ty, binds, ret),
            Op {
                ref mut ty,
                ref mut name,
                ref mut l,
                ref mut r,
            } => self.traverse_op(ty, name, l, r),
            PrimFun {
                ref mut param_ty,
                ref mut ret_ty,
                ref mut name,
            } => self.traverse_primfun(param_ty, ret_ty, name),
            Fun {
                ref mut param,
                ref mut body_ty,
                ref mut body,
                ref mut captures,
                ref mut make_closure,
            } => self.traverse_fun(param, body_ty, body, captures, make_closure),
            Closure {
                ref mut envs,
                ref mut param_ty,
                ref mut body_ty,
                ref mut fname,
            } => self.traverse_closure(envs, param_ty, body_ty, fname),
            App {
                ref mut ty,
                ref mut fun,
                ref mut arg,
            } => self.traverse_app(ty, fun, arg),
            If {
                ref mut ty,
                ref mut cond,
                ref mut then,
                ref mut else_,
            } => self.traverse_if(ty, cond, then, else_),
            Tuple {
                ref mut tys,
                ref mut tuple,
            } => self.traverse_tuple(tys, tuple),

            Sym {
                ref mut ty,
                ref mut name,
            } => self.traverse_sym(ty, name),
            Lit {
                ref mut ty,
                ref mut value,
            } => self.traverse_lit(ty, value),

        }
    }
    fn traverse_binds(&mut self, _ty: &mut HTy, binds: &mut Vec<Val>, ret: &mut Box<Expr>) {
        for val in binds.iter_mut() {
            self.traverse_val(val)
        }
        self.traverse_expr(ret)
    }

    fn traverse_op(
        &mut self,
        _ty: &mut HTy,
        _name: &mut Symbol,
        l: &mut Box<Expr>,
        r: &mut Box<Expr>,
    ) {
        self.traverse_expr(l);
        self.traverse_expr(r)
    }

    fn traverse_primfun(&mut self, _param_ty: &mut HTy, _ret_ty: &mut HTy, _name: &mut Symbol) {}

    fn traverse_fun(
        &mut self,
        _param: &mut (HTy, Symbol),
        _body_ty: &mut HTy,
        body: &mut Box<Expr>,
        _captures: &mut Vec<(HTy, Symbol)>,
        _make_closure: &mut Option<bool>,
    ) {
        self.traverse_expr(body)
    }

    fn traverse_closure(
        &mut self,
        _envs: &mut Vec<(HTy, Symbol)>,
        _param_ty: &mut HTy,
        _body_ty: &mut HTy,
        _fname: &mut Symbol,
    ) {

    }

    fn traverse_app(&mut self, _ty: &mut HTy, fun: &mut Box<Expr>, arg: &mut Box<Expr>) {
        self.traverse_expr(fun);
        self.traverse_expr(arg);
    }

    fn traverse_if(
        &mut self,
        _ty: &mut HTy,
        cond: &mut Box<Expr>,
        then: &mut Box<Expr>,
        else_: &mut Box<Expr>,
    ) {
        self.traverse_expr(cond);
        self.traverse_expr(then);
        self.traverse_expr(else_);
    }

    fn traverse_tuple(&mut self, _tys: &mut Vec<HTy>, tuple: &mut Vec<Expr>) {
        for t in tuple.iter_mut() {
            self.traverse_expr(t)
        }
    }

    fn traverse_sym(&mut self, _ty: &mut HTy, _name: &mut Symbol) {}

    fn traverse_lit(&mut self, _ty: &mut HTy, _value: &mut Literal) {}
}