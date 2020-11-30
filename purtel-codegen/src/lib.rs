//! Codegen. Definition and export of procedural macros.

use darling::FromMeta;
use syn::AttributeArgs;
// See https://crates.io/crates/proc-macro2:
// it's recommended to always use proc_macro2-exports
// proc_macro: rust internal library
// proc_macro2: library from crates.io
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{TokenStream as TokenStream2, TokenTree as TokenTree2, Group as Group2};
use syn::parse_macro_input;

mod util;
mod data;
use data::PurtelTaskAttributes;
use crate::util::unwrap_block;

/// This macro should be around a block that contains all purtel task definitions
/// (or at least `[purtel_task]`).
#[proc_macro_attribute]
pub fn purtel_tasks(_attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    // as recommended: always use "proc_macro2" in prodecural macros
    let mut item = TokenStream2::from(item);

    // print_expanded_token_stream(&item);

    /////////////////////////////////////////////////////////////////
    // Part 1/2: unwrap block

    item = unwrap_block(item);


    /////////////////////////////////////////////////////////////////
    // Part 2/2: generate dependencies vector from metadata annotations

    // we get all token streams for the #[purtel_task()]-macros
    // parse_macro_input!()- works only when called in the proc macro function itself
    let mut token_streams = vec![];
    purtel_task_traverse_token_stream(&item, &mut token_streams);

    // we parse each token stream
    let mut attributes = vec![];
    for ts in token_streams {
        let ts = ts.into();
        // parse_macro_input!()- works only when called in the proc macro function itself,
        // otherwise weird errors :/
        let attr_args = parse_macro_input!(ts as AttributeArgs);
        // from_list() gets generated from derive-annotation on the struct
        let attrs: PurtelTaskAttributes = PurtelTaskAttributes::from_list(&attr_args)
            .expect("#[purtel_task] could not be parsed!");
        attributes.push(attrs);
    }

    // prints out param usage per task (index)
    /*for (i, x) in attributes.iter().enumerate() {
        println!("task {} uses: read={:?}, write={:?}", i, x.read, x.write);
    }*/

    // TODO check how this whole section can be make less ugly
    // with quote!{} macro

    let mut str = String::from("let param_usages = vec![\n");
    for (_task_i, param_attr) in attributes.iter().enumerate() {
        str.push_str("  vec![\n");
        for write_param in &param_attr.write_params() {
            str.push_str(
                &format!("    purtel::PurtelParamUsage::new(\
                        \"{}\", purtel::PurtelParamUsageKind::WRITE),", write_param)
            )
        };
        for read_param in &param_attr.read_params() {
            str.push_str(
                &format!("    purtel::PurtelParamUsage::new(\
                        \"{}\", purtel::PurtelParamUsageKind::READ),", read_param)
            )
        }
        str.push_str("  ],\n");
    }
    str.push_str("];");
    // println!("{}", str);

    item.extend(str.parse::<TokenStream2>().unwrap());

    item.into() // Transform TokenStream2 back to TokenSteam1
}

/// Constructs a TokenStream out of the stream of a group.
/// Must be called like that: `&group.stream()`.
fn group_to_token_stream(group: &TokenStream2) -> TokenStream2 {
    let mut nts = TokenStream2::new();

    for x in group.clone() {
        match x {
            TokenTree2::Group(ref group) => {
                // print!("group:");
                let group_ts = group_to_token_stream(&group.stream());
                let group = Group2::new(group.delimiter(), group_ts);
                nts.extend(vec![TokenTree2::Group(group)]);
            },
            TokenTree2::Ident(ref _ident) => {
                nts.extend(vec![x]);
            },
            TokenTree2::Punct(ref _punct) => {
                nts.extend(vec![x]);
            },
            TokenTree2::Literal(ref _literal) => {
                nts.extend(vec![x]);
            }
        }
    }

    nts
}

/// Traverses the token stream and searches for all #[purtel_task]-Annotations. It extracts
/// their TokenStream and stores it into a vector. It's a recursive function.
fn purtel_task_traverse_token_stream(stream: &TokenStream2, token_streams: &mut Vec<TokenStream2>) {
    let mut next_group_is_purtel_task = false;
    for tree in stream.clone() {
        match tree {
            TokenTree2::Group(ref group) => {
                // println!("group:");
                if next_group_is_purtel_task {
                    let ts = group_to_token_stream(&group.stream());
                    token_streams.push(ts);
                    next_group_is_purtel_task = false;
                }
                purtel_task_traverse_token_stream(&group.stream(), token_streams);
            },
            TokenTree2::Ident(ref ident) => {
                // println!("ident: '{}', ", ident);
                if ident.to_string() == "purtel_task" {
                    next_group_is_purtel_task = true;
                }
            },
            _ => {}
        }
    }
}

/// Invoked like this:
/// - `#[purtel_task(write = "data1, data2")]`
/// - `#[purtel_task(write = "data1", read = "data2")]`
/// - `#[purtel_task(read = "data2")]`
/// If a parameter is write it is automatically also read.
#[proc_macro_attribute]
pub fn purtel_task(_args: TokenStream1, item: TokenStream1) -> TokenStream1 {
    // we do nothing here; just a marker for the super macro
    /*let attr_args = parse_macro_input!(args as AttributeArgs);
    let attrs: PurtelTaskAttributes = PurtelTaskAttributes::from_list(&attr_args).expect("#[purtel_task] could not be parsed!");
    println!("read_params: {:#?}", attrs.read_params());
    println!("write_params: {:#?}", attrs.write_params());*/
    item
}
