use proc_macro2::Span;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Ident, Meta, Token};

#[derive(Debug, PartialEq)]
pub struct MacroArgs {
    pub log_return: ReturnLogOptions,
    pub log_parameters: bool,
    pub parameters_to_skip: Vec<String>,
    pub custom_params: Vec<(String, String)>,
}
impl Default for MacroArgs {
    fn default() -> Self {
        MacroArgs {
            log_return: ReturnLogOptions::Skip,
            log_parameters: true,
            parameters_to_skip: vec![],
            custom_params: vec![],
        }
    }
}

/// Options for logging the returned value of a function
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReturnLogOptions {
    /// Ignores the return value (simply do not log it) even if the function is configured to emmit a log event at its egress
    Skip,
    /// Provided the function returns a `Result`, logs only if it is `Err`
    LogErrOnly,
    /// Provided the function returns a `Result`, logs only if it is `Ok`
    LogOkOnly,
    /// Logs any variant of the function -- that must return a `Result`
    LogResult,
    /// Logs the return value of the function -- that is not `Result`
    LogNonFallible,
}

impl Parse for MacroArgs {
    /// Our proc-macro parameters are inspired in https://docs.rs/tracing/0.1.40/tracing/attr.instrument.html
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const INVALID_IDENTIFIER_COMBINATION: &str =
            "Invalid identifier combination: only one of `err`, `ok` and `ret` may be specified";
        const INVALID_SKIP_COMBINATION: &str =
            "Invalid `skip` / `skip_all` combination: only one them may specified -- and just once";

        // variables used to enforce the combination rules above
        let mut log_return_set: Option<()> = None;
        let mut skip_set: Option<()> = None;

        let mut macro_args = MacroArgs::default();

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Ident) {
                let ident: Ident = input.parse()?;
                if input.is_empty() || input.parse::<Token![,]>().is_ok() {
                    // stand-alone identifiers parsing
                    match ident.to_string().as_str() {
                        "err" => {
                            macro_args.log_return = log_return_set
                                .replace(())
                                .map(|_| Err(syn::Error::new(Span::call_site(), String::from(INVALID_IDENTIFIER_COMBINATION) )))
                                .unwrap_or(Ok(ReturnLogOptions::LogErrOnly))?;
                        },
                        "ok" => {
                            macro_args.log_return = log_return_set
                                .replace(())
                                .map(|_| Err(syn::Error::new(Span::call_site(), String::from(INVALID_IDENTIFIER_COMBINATION) )))
                                .unwrap_or(Ok(ReturnLogOptions::LogResult))?;
                        },
                        "ret" => {
                            macro_args.log_return = log_return_set
                                .replace(())
                                .map(|_| Err(syn::Error::new(Span::call_site(), String::from(INVALID_IDENTIFIER_COMBINATION) )))
                                .unwrap_or(Ok(ReturnLogOptions::LogNonFallible))?;
                        },
                        "skip_all" => {
                            macro_args.log_parameters = skip_set.replace(())
                                .map(|_| Err(syn::Error::new(Span::call_site(), String::from(INVALID_SKIP_COMBINATION) )))
                                .unwrap_or(Ok(false))?;

                        }
                        _ => return Err(syn::Error::new(Span::call_site(), format!("Unknown identifier parameter '{}' -- known ones are: 'err', 'ok', 'ret', 'skip_all'", ident))),
                    }
                } else if input.lookahead1().peek(token::Paren) {
                    // name(identifier list) parsing -- "func"
                    match ident.to_string().as_str() {
                        "skip" => {
                            skip_set
                                .replace(())
                                .map(|_| {
                                    Err(syn::Error::new(
                                        Span::call_site(),
                                        String::from(INVALID_SKIP_COMBINATION),
                                    ))
                                })
                                .unwrap_or(Ok(()))?;
                            let content;
                            parenthesized!(content in input);
                            let punc = Punctuated::<Meta, Token![,]>::parse_terminated(&content)
                                .map_err(|err| {
                                    syn::Error::new(
                                        Span::call_site(),
                                        format!("While parsing `skip`: {:?}", err),
                                    )
                                })?;
                            macro_args.parameters_to_skip = punc
                                .into_iter()
                                .map(|s| s.to_token_stream().to_string())
                                .collect();
                            macro_args.log_parameters = true;
                        }
                        _ => {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                format!(
                                    "Unknown func parameter '{}' -- known one is: 'skip'",
                                    ident
                                ),
                            ))
                        }
                    }
                } else if input.parse::<Token![=]>().is_ok() {
                    // name=value parsing
                    match ident.to_string().as_str() {
                        /* FILL HERE MORE OPTIONS IN THE FUTURE */
                        _ => {
                            // custom name=val parameters
                            let name = ident.to_string();
                            let val = input.parse::<Ident>()
                                .map_err(|err| syn::Error::new(Span::call_site(), format!("Can't parse `val` for custom `name`=`val` parameter for name '{}': {:?}", name, err)))?
                                .to_string();
                            macro_args.custom_params.push((name, val));
                        }
                    }
                } else {
                    return Err(syn::Error::new(Span::call_site(), format!("Can't parse parameter {:?} -- parameters should either be in the forms name=value, name(values_list) or name (standalone identifiers)", input.to_string())));
                }
            } else {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!(
                        "Can't the reminder of the parameters. An Identifier was expected next: {}",
                        input
                    ),
                ));
            }
            // consume any unconsumed ','
            input.parse::<Token![,]>().ok();
        }

        Ok(macro_args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn default_parameters() {
        let parsed_args: MacroArgs = parse_str(r#""#).unwrap();
        assert_eq!(parsed_args, MacroArgs::default());
    }

    #[test]
    fn typical_usage() {
        let parsed_args: MacroArgs = parse_str(r#"err, skip(hello, world), a=b"#).unwrap();
        assert_eq!(parsed_args.log_return, ReturnLogOptions::LogErrOnly);
        assert_eq!(parsed_args.log_parameters, true);
        assert_eq!(parsed_args.parameters_to_skip, vec!["hello", "world"]);
        assert_eq!(
            parsed_args.custom_params,
            vec![("a".to_string(), "b".to_string())]
        );
    }

    #[test]
    fn identifier_parsing_scenarios() {
        let parsed_args: MacroArgs = parse_str(r#"err"#).unwrap();
        assert_eq!(
            parsed_args.log_return,
            ReturnLogOptions::LogErrOnly,
            "Unexpected `log_return`"
        );

        let parsed_args: MacroArgs = parse_str(r#"skip_all, err"#).unwrap();
        assert_eq!(
            parsed_args.log_return,
            ReturnLogOptions::LogErrOnly,
            "Unexpected `log_return`"
        );
        assert!(
            !parsed_args.log_parameters,
            "`skip_all` should simply cause the logging of parameters to be disabled"
        );

        let parsed_args: MacroArgs = parse_str(r#"err, skip_all"#).unwrap();
        assert_eq!(
            parsed_args.log_return,
            ReturnLogOptions::LogErrOnly,
            "Unexpected `log_return`"
        );
        assert!(!parsed_args.log_parameters);

        let parsed_args: MacroArgs = parse_str(r#"skip_all"#).unwrap();
        assert_eq!(
            parsed_args.log_return,
            ReturnLogOptions::Skip,
            "Unexpected `log_return`"
        );
        assert!(!parsed_args.log_parameters);

        let parsed_args: MacroArgs = parse_str(r#"err,skip()"#).unwrap();
        assert_eq!(
            parsed_args.log_return,
            ReturnLogOptions::LogErrOnly,
            "Unexpected `log_return`"
        );
        assert!(parsed_args.log_parameters);

        let parsed_args: MacroArgs = parse_str(r#"ret"#).unwrap();
        assert_eq!(
            parsed_args.log_return,
            ReturnLogOptions::LogNonFallible,
            "Unexpected `log_return`"
        );
        assert!(parsed_args.log_parameters);

        let parsed_args: MacroArgs = parse_str(r#"ok"#).unwrap();
        assert_eq!(
            parsed_args.log_return,
            ReturnLogOptions::LogResult,
            "Unexpected `log_return`"
        );
        assert!(parsed_args.log_parameters);
    }

    #[test]
    fn invalid_identifier_combinations() {
        fn assert(params: &str) {
            let parsed_args: Result<MacroArgs, syn::Error> = parse_str(params);
            assert!(
                parsed_args.is_err(),
                "Parsing the parameter combination '{params}' should not have succeeded"
            );
            assert_eq!(parsed_args.unwrap_err().to_string(), "Invalid identifier combination: only one of `err`, `ok` and `ret` may be specified", "Unexpected error message");
        }

        assert(r#"ok,err"#);
        assert(r#"err,ok"#);
        assert(r#"ret,ok"#);
        assert(r#"ok,ret"#);
        assert(r#"err,ret"#);
        assert(r#"ret,err"#);
    }

    #[test]
    fn invalid_skip_combinations() {
        fn assert(params: &str) {
            let parsed_args: Result<MacroArgs, syn::Error> = parse_str(params);
            assert!(
                parsed_args.is_err(),
                "Parsing the parameter combination '{params}' should not have succeeded"
            );
            assert_eq!(parsed_args.unwrap_err().to_string(), "Invalid `skip` / `skip_all` combination: only one them may specified -- and just once", "Unexpected error message");
        }

        assert(r#"skip(),skip_all"#);
        assert(r#"skip_all,skip()"#);
        assert(r#"skip(a),skip_all"#);
        assert(r#"skip_all,skip(a)"#);
        assert(r#"skip_all,skip_all"#);
        assert(r#"skip(a),skip()"#);
    }

    #[test]
    fn invalid_standalone_identifier() {
        let parsed_args: Result<MacroArgs, syn::Error> = parse_str(r#"nonexistingparam"#);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err().to_string(),
            r#"Unknown identifier parameter 'nonexistingparam' -- known ones are: 'err', 'ok', 'ret', 'skip_all'"#
        );
    }

    #[test]
    fn invalid_func_identifier() {
        let parsed_args: Result<MacroArgs, syn::Error> = parse_str(r#"nonexistingfunc()"#);
        assert!(parsed_args.is_err());
        assert_eq!(
            parsed_args.unwrap_err().to_string(),
            r#"Unknown func parameter 'nonexistingfunc' -- known one is: 'skip'"#
        );
    }
}
