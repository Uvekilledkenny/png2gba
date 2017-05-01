#![feature(plugin_registrar, quote, rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;
mod process;

use syntax::codemap::Span;
use syntax::tokenstream::TokenTree;
use syntax::ext::base::{expr_to_string, ExtCtxt, MacResult, DummyResult, MacEager, get_exprs_from_tts};
use syntax::ext::build::AstBuilder;
use syntax::symbol::{keywords};
use rustc_plugin::Registry;
use syntax::ast;

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use process::*;

fn expand_include_image(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut exprs = match get_exprs_from_tts(cx, sp, tts) {
        Some(ref exprs) if exprs.is_empty() => {
            cx.span_err(sp, "include_image! takes 1 or 2 arguments");
            return DummyResult::expr(sp);
        }
        None => return DummyResult::expr(sp),
        Some(exprs) => exprs.into_iter(),
    };

   let var = match expr_to_string(cx, exprs.next().unwrap(), "expected string literal") {
        None => return DummyResult::expr(sp),
        Some((v, _style)) => v.to_string(),
    };

    let tile = match exprs.next() {
        None => false,
        Some(second) => {
            match expr_to_string(cx, second, "expected string literal") {
                None => return DummyResult::expr(sp),
                Some((s, _style)) => {
                    if  s.to_string() == "tile" || 
                        s.to_string() == "t" {
                        true
                    } else {
                        return DummyResult::expr(sp)
                    }
                },
            }
        }
    };

    if let Some(_) = exprs.next() {
        cx.span_err(sp, "include_image! takes 1 or 2 arguments");
        return DummyResult::expr(sp);
    }

    let file = res_rel_file(cx, sp, Path::new(&var));
    let mut bytes = Vec::new();

    match File::open(&file).and_then(|mut f| f.read_to_end(&mut bytes)) {
        Err(e) => {
            cx.span_err(sp, &format!("couldn't read {}: {}", file.display(), e));
            return DummyResult::expr(sp);
        }
        Ok(..) => {
            let filename = format!("{}", file.display());

            let name = match file.file_stem() {
                Some(f) => format!("{:?}", f),
                None => return DummyResult::expr(sp),
            };

            cx.codemap()
                .new_filemap_and_lines(&filename, file.to_str(), "");

            let data = to_data(bytes.as_slice(), tile);
            let mut token_trees = vec![];

            for i in data.clone() {
                token_trees.push(quote_expr!(cx, $i));
            }

            let vec = cx.expr_vec_slice(sp, token_trees.clone());
            let length = token_trees.len();

            let ty = cx.ty_rptr(sp,
                                cx.ty(sp,
                                      ast::TyKind::Array(quote_ty!(cx, u16),
                                                         quote_expr!(cx, $length))),
                                Some(cx.lifetime(sp, keywords::StaticLifetime.name())),
                                ast::Mutability::Immutable);
            let st = ast::ItemKind::Const(ty, vec);

            let name = cx.ident_of(&name.to_uppercase());
            let item = cx.item(sp, name, vec![], st);
            let stmt = ast::Stmt {
                id: ast::DUMMY_NODE_ID,
                node: ast::StmtKind::Item(item),
                span: sp,
            };

            MacEager::expr(cx.expr_block(cx.block(sp,
                                                  vec![stmt,
                                                       cx.stmt_expr(cx.expr_ident(sp, name))])))
        }
    }
}

fn res_rel_file(cx: &mut ExtCtxt, sp: Span, arg: &Path) -> PathBuf {
    if !arg.is_absolute() {
        let callsite = sp.source_callsite();
        let mut cu = PathBuf::from(&cx.codemap().span_to_filename(callsite));
        cu.pop();
        cu.push(arg);
        cu
    } else {
        arg.to_path_buf()
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("include_image", expand_include_image);
}
