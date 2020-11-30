//! Codegen. utility functions.

use proc_macro2::{TokenStream, Delimiter};
use proc_macro2::TokenTree;

/// Prints a token tree recursively for debugging purpose.
/// Also useful to understand how to traverse a token tree.
#[allow(dead_code)]
pub fn print_expanded_token_stream(stream: &TokenStream) {
    for tree in stream.clone() {
        match tree {
            TokenTree::Group(ref group) => {
                println!("group:");
                print_expanded_token_stream(&group.stream());
            },
            TokenTree::Ident(ref ident) => {
                println!("ident: '{}', ", ident);
            },
            TokenTree::Punct(ref punct) => {
                println!("punct: '{}', ", punct);
            },
            TokenTree::Literal(ref literal) => {
                println!("literal: '{}', ", literal);
            }
        }
    }
}

/// Unwraps a {}-code-block as it wasn't there in the first place.
/// This is useful if you want to achieve the following:
/// ```
/// // do_something_with_vars_proc_macro-proc macro unwraps the block after gaining some
/// // meta info; in the end it looks like the {}-block was never there and only the inner
/// // block remains there.
/// #[do_something_with_vars_proc_macro] {
///     let a = 5;
/// }
/// let b = a;
/// ```
pub fn unwrap_block(stream: TokenStream) -> TokenStream {
    let mut group = None;
    for token_tree in stream {
        match token_tree {
            TokenTree::Group(ref gr) => { group = Some(gr.clone())}
            _ => {}
        }
    }

    if group.is_none() {
        panic!("No group found! #[purtel_tasks] must be around {{}} block");
    }

    let group = group.unwrap();
    if group.delimiter() != Delimiter::Brace {
        panic!("unwrap_block() only works for {{}}-blocks!");
    }

    // we return the stream to the inner block; i.e. skip the surrounding block/group
    group.stream()
}
