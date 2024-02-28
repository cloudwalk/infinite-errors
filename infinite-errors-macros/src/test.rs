use pretty_assertions::assert_eq;
use quote::quote;
use syn::{parse_quote, ItemFn};

#[test]
fn err_context_empty_fn() {
    let output = err_context(quote! {fn f() {}});

    assert_err_context_output(output, quote! {fn f()}, quote! {});
}

#[test]
fn err_context_empty_async_fn() {
    let output = err_context(quote! {async fn f() {}});

    assert_err_context_output(output, quote! {async fn f()}, quote! {});
}

#[test]
fn err_context_empty_generic_fn() {
    let output = err_context(quote! {fn f<T>() {}});

    assert_err_context_output(output, quote! {fn f<T>()}, quote! {});
}

#[test]
fn err_context_empty_async_generic_fn() {
    let output = err_context(quote! {async fn f<T>() {}});

    assert_err_context_output(output, quote! {async fn f<T>()}, quote! {});
}

#[test]
fn err_context_empty_fn_with_attribute() {
    let output = err_context(quote! {#[attr] fn f() {}});

    assert_err_context_output(output, quote! {#[attr] fn f()}, quote! {});
}

#[test]
fn err_context_fn() {
    let output = err_context(
        quote! {fn try_something(s: &str) -> Result<String> { try_try(s.to_string()) }},
    );

    assert_err_context_output(
        output,
        quote! {fn try_something(s: &str) -> Result<String>},
        quote! {try_try(s.to_string())},
    );
}

fn err_context(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    super::err_context_impl(quote! {ErrorKind::SomeContext}, item)
}

fn assert_err_context_output(
    output: proc_macro2::TokenStream,
    decl: proc_macro2::TokenStream,
    body: proc_macro2::TokenStream,
) {
    let parsed_decl: ItemFn = parse_quote! {#decl {}};
    let (left, right) = if parsed_decl.sig.asyncness.is_some() {
        (quote! {async move}, quote! {.await})
    } else {
        (quote! {move | |}, quote! {()})
    };

    assert_eq!(
        output.to_string(),
        quote! {
                #decl {
                ::infinite_errors::ErrorContext::err_context(
                    (#left {
                        #body
                    })#right,
                    ErrorKind::SomeContext
                )
            }
        }
        .to_string()
    );
}
