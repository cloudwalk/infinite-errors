#![doc = include_str!("../README.md")]
#![recursion_limit = "256"]

use syn::*;
use crate::parameters::MacroArgs;

mod parameters;
mod tracing_impl;


#[macro_use]
extern crate proc_macro_error;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn instrument(
    macro_args_tokens: proc_macro::TokenStream,
    fn_tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {

    let fn_item = syn::parse_macro_input!(fn_tokens as ItemFn);
    let parameters = syn::parse_macro_input!(macro_args_tokens as MacroArgs);

    let function_definition_tokens = tracing_impl::instrument(parameters, fn_item);
//panic!("FN is defined AS:\n{}", function_definition_tokens.to_string());
    function_definition_tokens.into()

}