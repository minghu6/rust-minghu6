#![allow(unused_imports)]
#![allow(unused_import_braces)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    token::{Paren, Priv},
    Expr, Ident, Lit, LitInt, LitStr, MacroDelimiter, Path, Token,
};


////////////////////////////////////////////////////////////////////////////////
//// DefCollInit

struct VecMacroRules {
    name: Ident,
    path: Path,
    inc_op: Ident,
}

impl Parse for VecMacroRules {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let path: Path = input.parse()?;

        let inc_op;
        if let Ok(_) = input.parse::<Token![,]>() {
            inc_op = input.parse()?;
        } else {
            inc_op = Ident::new("insert", Span::call_site());
        }

        Ok(Self { name, path, inc_op })
    }
}

#[proc_macro]
pub fn make_vec_macro_rules(input: TokenStream) -> TokenStream {
    let VecMacroRules { name, path, inc_op } =
        parse_macro_input!(input as VecMacroRules);

    TokenStream::from(quote! {
        #[macro_export]
        macro_rules! #name {
            ( $($value:expr),+ ) => {
                {
                    let mut vec_like = #path::new();

                    $(
                        vec_like.#inc_op($value);
                    )+

                    vec_like
                }
            };
            () => {
                {
                    #path::new()
                }
            }
        }
    })
}



////////////////////////////////////////////////////////////////////////////////
//// Define New Custom Error
struct MakeSimpleError {
    name: Ident,
}

impl Parse for MakeSimpleError {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;

        Ok(Self { name })
    }
}

#[proc_macro]
pub fn make_simple_error_rules(input: TokenStream) -> TokenStream {
    let MakeSimpleError { name } =
        parse_macro_input!(input as MakeSimpleError);

    TokenStream::from(quote! {
        pub struct #name {
            msg: String
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.msg)
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self)
            }
        }

        impl std::error::Error for #name {}

        impl #name {
            pub fn new_box_err(msg: &str) -> Box<dyn std::error::Error> {
                Box::new(Self::new(msg))
            }

            pub fn new(msg: &str) -> Self {
                Self {
                    msg: msg.to_string()
                }
            }
        }

    })
}


#[cfg(test)]
mod tests {}
