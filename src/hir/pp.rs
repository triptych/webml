use std::io;

use util::PP;
use hir::*;

impl PP for HIR {
    fn pp(&self, mut w: &mut io::Write, indent: usize) -> io::Result<()> {
        for val in &self.0 {
            val.pp(w, indent)?;
            write!(w, "\n")?;
        }
        Ok(())
    }
}


impl PP for Val {
    fn pp(&self, mut w: &mut io::Write, indent: usize) -> io::Result<()> {
        write!(w, "{}val {}: ", Self::nspaces(indent), self.name.0)?;
        self.ty.pp(w, indent)?;
        write!(w, " = ")?;
        self.expr.pp(w, indent + 4)?;
        Ok(())
    }

}

impl PP for Expr {
    fn pp(&self, mut w: &mut io::Write, indent: usize) -> io::Result<()> {
        use hir::Expr::*;
        match self {
            &Binds{ref binds, ref ret, ..} => {
                let ind = Self::nspaces(indent);
                let nextind = Self::nspaces(indent+4);
                write!(w, "let\n")?;
                for val in binds {
                    val.pp(w, indent + 4)?;
                    write!(w, "\n")?;
                }
                write!(w, "{}in\n{}", ind, nextind)?;
                ret.pp(w, indent + 4)?;
                write!(w, "\n{}end", ind)?;
            },
            &Fun{ref body, ref param, ..} => {
                write!(w, "fun ")?;
                param.pp(w, indent)?;
                write!(w, " => ")?;
                body.pp(&mut w, indent + 4)?;
            }
            &App{ref fun, ref arg, ..} => {
                write!(w, "(")?;
                fun.pp(w, indent)?;
                write!(w, ") ")?;
                arg.pp(w, indent + 4)?;
            }
            &If {ref cond, ref then, ref else_, ..} => {
                let ind = Self::nspaces(indent);
                write!(w, "if ")?;
                cond.pp(w, indent + 4)?;
                write!(w, "\n{}then ", ind)?;
                then.pp(w, indent + 4)?;
                write!(w, "\n{}else ", ind)?;
                else_.pp(w, indent + 4)?;
            }
            &PrimFun{ref name, ..} => {
                name.pp(w, indent)?;
            }
            &Sym{ref name, ..} => {
                name.pp(w, indent)?;
            }
            &Lit{ref value, ..} => {
                value.pp(w, indent)?;
            }

        }
        Ok(())
    }

}