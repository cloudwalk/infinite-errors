extern crate proc_macro;

#[cfg(test)]
mod test;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, ItemFn, TypePath};

/// Add a context when this function returns an error.
///
/// This macro takes a single argument: the error kind which serves as
/// context. This macro is compatible with both sync and async functions. The
/// function must return a `Result<T, infinite_errors::Error<K>>` for some `T`
/// and `K`.
///
/// ```ignore
/// #[err_context(ErrorKind::Parse)]
/// fn parse(s: &str) -> Result<SomeStruct, Error<ErrorKind>> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn err_context(attributes: TokenStream, item: TokenStream) -> TokenStream {
    err_context_impl(attributes.into(), item.into()).into()
}

fn err_context_impl(
    attributes: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let context_error_kind = unwrap_parse("attributes", syn::parse2::<TypePath>(attributes));
    let mut item_fn = unwrap_parse("item", syn::parse2::<ItemFn>(item));
    let (left, right) = if item_fn.sig.asyncness.is_some() {
        (quote! {async move}, quote! {.await})
    } else {
        (quote! {move | |}, quote! {()})
    };
    let block = item_fn.block;
    let body = parse_quote! {{
        ::infinite_errors::ErrorContext::err_context(
            (#left
                #block
            )#right,
            #context_error_kind
        )
    }};
    item_fn.block = Box::new(body);

    item_fn.into_token_stream()
}

fn unwrap_parse<T>(name: &str, res: syn::parse::Result<T>) -> T {
    match res {
        Ok(x) => x,
        Err(err) => panic!("failed to parse {name}: {err}"),
    }
}
