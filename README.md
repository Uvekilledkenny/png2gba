# PNG2GBA Rust Macro

## What's this ?
This is a group of macros to include images in BGR 15 bits on Rust GBA project.
Everything is done at compile time!

## How to use it ?
```
#![feature(plugin)]
#![plugin(png2gba)]

const x: &[u16] = include_image!("test.png", "t");
const e: (&[u8], &[u16) = include_image_palette!("test.png");

fn main() {
    println!("array_tiled: {}", x);
    println!("array_data: {}", e.0);
    println!("array_palette: {}", e.1);
}
```

## What now ?
- Compression?

## Docs used
- https://github.com/rust-lang/rust/blob/master/src/libsyntax/ext/source_util.rs#L173
- https://github.com/rust-lang/rust/blob/master/src/libsyntax_ext/format.rs
- https://github.com/rust-lang/rust/blob/master/src/libsyntax/test.rs
- https://manishearth.github.io/rust-internals-docs/syntax/ast/struct.Expr.html
- https://doc.rust-lang.org/1.1.0/syntax/ext/base/struct.ExtCtxt.html
- https://github.com/rust-lang/rust/blob/master/src/libsyntax/ast.rs
- https://github.com/rust-lang/rust/blob/master/src/libsyntax_ext/env.rs
