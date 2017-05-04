#![feature(plugin_registrar, quote, rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;
mod process;

use syntax::codemap::Span;
use syntax::tokenstream::TokenTree;
use syntax::ext::base::{expr_to_string, ExtCtxt, MacResult, DummyResult, MacEager,
                        get_exprs_from_tts};
use syntax::ext::build::AstBuilder;
use syntax::symbol::keywords;
use rustc_plugin::Registry;
use syntax::ast;
use syntax::ptr::P;

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::u32;

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
                    if s.to_string() == "tile" || s.to_string() == "t" {
                        true
                    } else {
                        cx.span_err(sp, &format!("expected t or tile, found: {}", s));
                        return DummyResult::expr(sp);
                    }
                }
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
            cx.span_err(sp,
                        &format!("couldn't read {}: {}", file.display(), e.to_string()));
            return DummyResult::expr(sp);
        }
        Ok(..) => {
            let filename = format!("{}", file.display());

            let name = match file.file_stem() {
                Some(f) => format!("{:?}", f),
                None => return DummyResult::expr(sp),
            };

            cx.codemap()
                .new_filemap_and_lines(&filename, "");

            let data = to_data(bytes.as_slice(), tile);
            let mut token_trees = vec![];

            for i in data.clone() {
                token_trees.push(quote_expr!(cx, $i));
            }

            let ty = quote_ty!(cx, u16);
            let array = static_array(cx, &name, ty, sp, token_trees);

            MacEager::expr(array)
        }
    }
}

fn expand_include_imagepalette(cx: &mut ExtCtxt,
                               sp: Span,
                               tts: &[TokenTree])
                               -> Box<MacResult + 'static> {
    let mut exprs = match get_exprs_from_tts(cx, sp, tts) {
        Some(ref exprs) if exprs.is_empty() => {
            cx.span_err(sp, "include_image_palette! takes 2 or 3 arguments");
            return DummyResult::expr(sp);
        }
        None => return DummyResult::expr(sp),
        Some(exprs) => exprs.into_iter(),
    };

    let var = match expr_to_string(cx, exprs.next().unwrap(), "expected string literal") {
        None => return DummyResult::expr(sp),
        Some((v, _style)) => v.to_string(),
    };

    let alpha = match exprs.next() {
        None => return DummyResult::expr(sp),
        Some(second) => {
            match expr_to_string(cx, second, "expected string literal") {
                None => return DummyResult::expr(sp),
                Some((s, _style)) => {
                    let x = s.to_string();
                    if !x.starts_with("#") || x.len() != 7 {
                        cx.span_err(sp, &format!("wrong hexadecimal format: {}", x));
                        return DummyResult::expr(sp);
                    }

                    match u32::from_str_radix(&x[1..], 16) {
                        Ok(i) => i,
                        Err(e) => {
                            cx.span_err(sp,
                                        &format!("can't parse string to hexadecimal {}: {}", x, e));
                            return DummyResult::expr(sp);
                        }
                    }
                }
            }
        }
    };

    let tile = match exprs.next() {
        None => false,
        Some(second) => {
            match expr_to_string(cx, second, "expected string literal") {
                None => return DummyResult::expr(sp),
                Some((s, _style)) => {
                    if s.to_string() == "tile" || s.to_string() == "t" {
                        true
                    } else {
                        return DummyResult::expr(sp);
                    }
                }
            }
        }
    };

    if let Some(_) = exprs.next() {
        cx.span_err(sp, "include_image_palette! takes 2 or 3 arguments");
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

            let (name, name_b) = match file.file_stem() {
                Some(f) => (format!("{:?}", f), format!("{:?}_palette", f)),
                None => return DummyResult::expr(sp),
            };

            cx.codemap()
                .new_filemap_and_lines(&filename, "");

            let (data, palette) = match to_data_palette(bytes.as_slice(), alpha, tile) {
                Ok((e, f)) => (e, f),
                Err(e) => {
                    cx.span_err(sp, &format!("can't process file {}: {}", name, e));
                    return DummyResult::expr(sp);
                }
            };

            let mut token_trees_a = vec![];
            let mut token_trees_b = vec![];

            for i in data.clone() {
                token_trees_a.push(quote_expr!(cx, $i));
            }

            for i in palette.clone() {
                token_trees_b.push(quote_expr!(cx, $i));
            }

            let ty_a = quote_ty!(cx, u8);
            let a = static_array(cx, &name, ty_a, sp, token_trees_a);

            let ty_b = quote_ty!(cx, u16);
            let b = static_array(cx, &name_b, ty_b, sp, token_trees_b);

            MacEager::expr(cx.expr_tuple(sp, vec![a, b]))
        }
    }
}

fn static_array(ecx: &mut ExtCtxt,
                name: &str,
                piece_ty: P<ast::Ty>,
                sp: Span,
                pieces: Vec<P<ast::Expr>>)
                -> P<ast::Expr> {

    let len = pieces.clone().len();
    let ty = ecx.ty_rptr(sp,
                         ecx.ty(sp, ast::TyKind::Array(piece_ty, quote_expr!(ecx, $len))),
                         Some(ecx.lifetime(sp, keywords::StaticLifetime.name())),
                         ast::Mutability::Immutable);
    let slice = ecx.expr_vec_slice(sp, pieces);
    let st = ast::ItemKind::Const(ty, slice);

    let name = ecx.ident_of(&name.to_uppercase());
    let item = ecx.item(sp, name, vec![], st);
    let stmt = ast::Stmt {
        id: ast::DUMMY_NODE_ID,
        node: ast::StmtKind::Item(item),
        span: sp,
    };

    ecx.expr_block(ecx.block(sp, vec![stmt, ecx.stmt_expr(ecx.expr_ident(sp, name))]))
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
    reg.register_macro("include_image_palette", expand_include_imagepalette);
}
