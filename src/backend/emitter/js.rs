use crate::frontend::parser::ast::{TiExpr, TiExprType as E, TiProg, TiStmt, TiStmtType};

use super::{emitter::TiEmit, stream::Stream};

macro_rules! expr_proc {
    ($self: expr, $a: expr, $b: expr, $c: expr, $d: ident $e: literal $f: ident) => {{
        if $a > $b {
            $c.write("(");
        }
        $self.emit_expr_proc($d, $c, $b);
        $c.write(" ");
        $c.write($e);
        $c.write(" ");
        $self.emit_expr_proc($f, $c, $b);
        if $a > $b {
            $c.write(")");
        }
    }};
}

pub struct JSEmitter {
    ident: usize,
    ident_str: String,
}

impl JSEmitter {
    pub fn new(ident_str: &str) -> Self {
        Self {
            ident: 0,
            ident_str: ident_str.to_string(),
        }
    }

    fn begin(&mut self) {
        self.ident += 1;
    }

    fn close(&mut self) {
        self.ident = self.ident.checked_sub(1).unwrap();
    }

    fn emit_ident(&self, stream: &mut Stream) {
        for _ in 0..self.ident {
            stream.write(&self.ident_str);
        }
    }

    fn emit_expr_proc(&mut self, expr: &TiExpr, stream: &mut Stream, proc: u8) {
        match &expr.ti_expr {
            E::Assign(lhs, rhs) => expr_proc!(self, proc, 0x10, stream, lhs "=" rhs),

            E::Or(lhs, rhs) => expr_proc!(self, proc, 0x11, stream, lhs "||" rhs),

            E::And(lhs, rhs) => expr_proc!(self, proc, 0x12, stream, lhs "&&" rhs),

            E::Neq(lhs, rhs) => expr_proc!(self, proc, 0x13, stream, lhs "!==" rhs),
            E::Eq(lhs, rhs) => expr_proc!(self, proc, 0x13, stream, lhs "===" rhs),

            E::Grt(lhs, rhs) => expr_proc!(self, proc, 0x14, stream, lhs ">" rhs),
            E::Geq(lhs, rhs) => expr_proc!(self, proc, 0x14, stream, lhs ">=" rhs),
            E::Les(lhs, rhs) => expr_proc!(self, proc, 0x14, stream, lhs "<" rhs),
            E::Leq(lhs, rhs) => expr_proc!(self, proc, 0x14, stream, lhs "<=" rhs),

            E::Add(lhs, rhs) => expr_proc!(self, proc, 0x15, stream, lhs "+" rhs),
            E::Sub(lhs, rhs) => expr_proc!(self, proc, 0x15, stream, lhs "-" rhs),

            E::Mul(lhs, rhs) => expr_proc!(self, proc, 0x16, stream, lhs "*" rhs),
            E::Div(lhs, rhs) => expr_proc!(self, proc, 0x16, stream, lhs "/" rhs),

            E::Var(n) => {
                stream.write(n);
            }
            E::LlNum(x) => {
                stream.write(&x.to_string());
            }

            E::Block(block) => {
                self.emit_block(block, stream);
            }

            E::IfElse(cond, tcase, fcase) => {
                stream.write("(() => { if (");
                self.emit_expr(cond, stream);
                stream.write(") { return ");
                self.emit_block(tcase, stream);
                if let Some(vfcase) = fcase {
                    stream.write("; } else { return ");
                    self.emit_block(vfcase, stream);
                }
                stream.write("; }})()");
            }

            E::Fn(fname, fargs, _, fbody) => {
                stream.write("function ");
                stream.write(fname);
                stream.write("(");
                if fargs.len() > 0 {
                    stream.write(&fargs[0].0);
                    for (an, _) in &fargs[1..] {
                        stream.write(", ");
                        stream.write(an);
                    }
                }
                stream.write(") {\r\n");
                self.emit_block_internal(fbody, stream);
                self.emit_ident(stream);
                stream.write("}");
            }

            E::Call(callable, fargs) => {
                self.emit_expr(callable, stream);
                stream.write("(");
                if fargs.len() > 0 {
                    self.emit_expr(&fargs[0], stream);
                    for arg in &fargs[1..] {
                        stream.write(", ");
                        self.emit_expr(arg, stream);
                    }
                }
                stream.write(")");
            }

            _ => {}
        }
    }

    fn emit_expr(&mut self, expr: &TiExpr, stream: &mut Stream) {
        self.emit_expr_proc(expr, stream, 0);
    }

    fn emit_stmt(&mut self, stmt: &TiStmt, stream: &mut Stream, ret: bool) {
        self.emit_ident(stream);
        match &stmt.ti_stmt {
            TiStmtType::Let(pattern, value) => {
                stream.write("let ");
                stream.write(pattern);
                if let Some(vvalue) = value {
                    stream.write(" = ");
                    self.emit_expr(vvalue, stream);
                }
                stream.write(";\r\n")
            }
            TiStmtType::Expr(ti_expr) => {
                if ret {
                    stream.write("return ");
                }
                self.emit_expr(ti_expr, stream);
                stream.write(";\r\n")
            }
            TiStmtType::Break => stream.write("break;\r\n"),
            TiStmtType::Continue => stream.write("continue;\r\n"),
            TiStmtType::Return(value) => {
                stream.write("return");
                if let Some(vv) = value {
                    stream.write(" ");
                    self.emit_expr(vv, stream);
                }
                stream.write(";\r\n");
            }
        }
    }

    fn emit_block_internal(&mut self, block: &Vec<TiStmt>, stream: &mut Stream) {
        let last = block.last();
        if let Some(vlast) = last {
            self.begin();
            for stmt in block[0..(block.len() - 1)].iter() {
                self.emit_stmt(stmt, stream, false);
            }
            self.emit_stmt(vlast, stream, true);
            self.close();
        }
    }

    fn emit_block(&mut self, block: &Vec<TiStmt>, stream: &mut Stream) {
        stream.write("(() => {\r\n");
        self.emit_block_internal(block, stream);
        self.emit_ident(stream);
        stream.write("})()");
    }
}

impl TiEmit for JSEmitter {
    fn emit(&mut self, ast: &TiProg, stream: &mut Stream) {
        self.emit_block(&ast.ti_children, stream);
    }
}
