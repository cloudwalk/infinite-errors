use proc_macro2::{Ident, Span, TokenStream};
use syn::{Expr, ExprLit, ItemFn, Lit, LitStr, MetaNameValue};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use crate::parameters::{MacroArgs, ReturnLogOptions};
use crate::tracing_impl;

pub fn instrument(
    parameters: MacroArgs,
    fn_item: ItemFn,
) -> proc_macro2::TokenStream {

    let ingress_log_level: Option<LitStr> = None; //Some(LitStr::new("info", Span::call_site()));
    let ok_result_log_level = LitStr::new("info", Span::call_site());
    let err_result_log_level = LitStr::new("error", Span::call_site());

    let minitrace_annotation_tokens = quote!(
        #[::minitrace::trace]
    );

    // IMPORTANT: tricky var ahead:
    //   1) the special value of `None` means "do not log parameters at all"
    //   2) if `Some`, it contains the list of parameters to skip
    let mut skip_options: Option<Vec<String>> = parameters.log_parameters.then(|| parameters.parameters_to_skip);
    let ingress_params = if let Some(ingress_log_level) = ingress_log_level {
        quote!(ingress=#ingress_log_level)
    } else {
        quote!()
    };
    let degress_params = match parameters.log_return {
        ReturnLogOptions::Skip => {
            quote!()
        },
        ReturnLogOptions::LogErrOnly => {
            quote!(err=#err_result_log_level,)
        },
        ReturnLogOptions::LogOkOnly => {
            quote!(ok=#ok_result_log_level,)
        },
        ReturnLogOptions::LogResult => {
            quote!(err=#err_result_log_level, ok=#ok_result_log_level,)
        },
        ReturnLogOptions::LogNonFallible => {
            quote!(egress=#ok_result_log_level,)
        },
    };
    let skip_params = if let Some(to_skip) = skip_options {
        let skip_punctuated_list: Punctuated<Ident, Comma> = to_skip.into_iter()
            .map(|param_name| Ident::new(&param_name, Span::call_site()))
            .collect();
        quote!(skip=[#skip_punctuated_list], )
    } else {
        // no `skip_options` list present means that no input params will be logged for the function call
        quote!()
    };

    let logcall_annotation_tokens = if degress_params.is_empty() && ingress_params.is_empty() {
        quote!()
    } else {
        quote!(
            #[::logcall::logcall(#degress_params #ingress_params #skip_params)]
        )
    };

    quote!(
        #minitrace_annotation_tokens
        #logcall_annotation_tokens
        #fn_item
    )
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use syn::{Block, ReturnType, Signature, Visibility};
    use super::*;


    /// If no parameters are provided, the `minitrace` annotation should be issued,
    /// but `logcall`s don't, as the latter doesn't accept parameterless invocations.
    /// ```nocompile
    /// #[instrument()]
    #[test]
    fn standard_usage() {
        let expected_fn_def = r#"# [:: minitrace :: trace] fn a () { }"#;
        let parameters = MacroArgs {
            ..MacroArgs::default()
        };
        let fn_tokens = test_fn_item();
        let observed_fn_def = instrument(parameters, fn_tokens).to_string();
        assert_eq!(observed_fn_def, expected_fn_def, "Function definition attributes mismatch");
    }

    /// By specifying "Log Err Results", the logging of params is also activated for `logcall`.
    /// ```nocompile
    /// #[instrument(err)]
    #[test]
    fn log_result_and_params_on_err() {
        let expected_fn_def = r#"# [:: minitrace :: trace] # [:: logcall :: logcall (err = "error" , skip = [] ,)] fn a () { }"#;
        let parameters = MacroArgs {
            log_return: ReturnLogOptions::LogErrOnly,
            log_parameters: true,
            ..MacroArgs::default()
        };
        let fn_tokens = test_fn_item();
        let observed_fn_def = instrument(parameters, fn_tokens).to_string();
        assert_eq!(observed_fn_def, expected_fn_def, "Function definition attributes mismatch");
    }

    /// By specifying "skipall", the logging of params is deactivated for `logcall`.
    /// ```nocompile
    /// #[instrument(err, skip_all)]
    #[test]
    fn log_result_but_no_params_on_err() {
        let expected_fn_def = r#"# [:: minitrace :: trace] # [:: logcall :: logcall (err = "error" ,)] fn a () { }"#;
        let parameters = MacroArgs {
            log_return: ReturnLogOptions::LogErrOnly,
            log_parameters: false,
            ..MacroArgs::default()
        };
        let fn_tokens = test_fn_item();
        let observed_fn_def = instrument(parameters, fn_tokens).to_string();
        assert_eq!(observed_fn_def, expected_fn_def, "Function definition attributes mismatch");
    }

    /// By specifying a `parameters_to_skip` list, `logcall` should be informed of it.
    /// ```nocompile
    /// #[instrument(err, skip(password,secret))]
    #[test]
    fn log_result_and_some_params_on_err() {
        let expected_fn_def = r#"# [:: minitrace :: trace] # [:: logcall :: logcall (err = "error" , skip = [password , secret] ,)] fn a () { }"#;
        let parameters = MacroArgs {
            log_return: ReturnLogOptions::LogErrOnly,
            log_parameters: true,
            parameters_to_skip: vec!["password", "secret"].into_iter().map(|e| e.to_string()).collect(),
            ..MacroArgs::default()
        };
        let fn_tokens = test_fn_item();
        let observed_fn_def = instrument(parameters, fn_tokens).to_string();
        assert_eq!(observed_fn_def, expected_fn_def, "Function definition attributes mismatch");
    }


    // helper functions
    ///////////////////

    /// Returns a `ItemFn` that corresponds to the following function declaration:
    /// ```nocompile
    ///     fn a () {}
    fn test_fn_item() -> ItemFn {
        ItemFn {
            attrs: vec![],
            vis: Visibility::Inherited,
            sig: Signature {
                constness: None,
                asyncness: None,
                unsafety: None,
                abi: None,
                fn_token: Default::default(),
                ident: Ident::new("a", Span::call_site()),
                generics: Default::default(),
                paren_token: Default::default(),
                inputs: Default::default(),
                variadic: None,
                output: ReturnType::Default,
            },
            block: Box::new(Block { brace_token: Default::default(), stmts: vec![] }),
        }
    }
}