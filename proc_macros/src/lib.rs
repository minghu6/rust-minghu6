#![feature(macro_metavar_expr)]
#![feature(log_syntax)]
#![allow(unused)]

use std::{any::Any, convert::TryFrom, iter::FromIterator};

use derive_quote_to_tokens::ToTokens;
use derive_syn_parse::Parse;
use either::*;
use proc_macro::TokenStream;
use proc_macro2::{extra::DelimSpan, Span};
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{discouraged::Speculative, Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Bracket, Paren, Token, Underscore},
    AngleBracketedGenericArguments, GenericArgument, GenericParam, Generics,
    Ident, LitInt, LitStr, Path, Token, Type, TypeTuple, WhereClause,
};

#[macro_use]
mod duck_typing_impl;
mod resources;

use duck_typing_impl::*;

////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! either_wrapper {
    ($left:ident, $right:ident) => {
        paste::paste! {
            #[derive(Clone)]
            #[repr(transparent)]
            struct [<Either $left Or $right>] {
                value: Either<$left, $right>
            }

            impl Parse for [<Either $left Or $right>] {
                fn parse(input: ParseStream) -> Result<Self> {
                    let forked = input.fork();

                    Ok(Self {
                        value: if let Ok(left) = forked.parse() {
                            input.advance_to(&forked);

                            Left(left)
                        }
                        else {
                            Right(input.parse()?)
                        }
                    })
                }
            }
        }
    };
    ($left:ident, $right:ident $(,)? +ToTokens) => {
        either_wrapper!($left, $right);

        paste::paste! {
            impl ToTokens for [<Either $left Or $right>] {
                fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                    tokens.extend(match &self.value {
                        Left(left) => quote! {
                            #left
                        },
                        Right(right) => quote! {
                            #right
                        }
                    });
                }
            }
        }
    }
}

macro_rules! maybe_wrapper {
    ($it:ident) => {
        paste::paste! {
            #[derive(Clone)]
            #[repr(transparent)]
            struct [<Maybe $it>] {
                value: Option<$it>
            }

            impl Parse for [<Maybe $it>] {
                fn parse(input: ParseStream) -> Result<Self> {
                    let forked = input.fork();

                    Ok(Self {
                        value: if forked.parse::<$it>().is_ok() {
                            input.parse().ok()
                        } else {
                            None
                        },
                    })
                }
            }
        }
    };
    ($it:ident +ToTokens) => {
        maybe_wrapper!($it);

        paste::paste! {
            impl ToTokens for [<Maybe $it>] {
                fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                    if let Some(value) = &self.value {
                        tokens.extend(quote!{
                            #value
                        })
                    }
                }
            }
        }
    }
}

macro_rules! _bracket_punctuated {
    (@squarebracket $it:ident, $sub:path, $punc:ty) => {
        #[derive(Clone)]
        struct $it {
            bracket_token: syn::token::Bracket,
            args: syn::punctuated::Punctuated<$sub, $punc>,
        }

        impl Parse for $it {
            fn parse(input: ParseStream) -> Result<Self> {
                let content;

                Ok(Self {
                    bracket_token: syn::bracketed!(content in input),
                    args: syn::punctuated::Punctuated::<$sub, $punc>::parse_terminated(
                        &content,
                    )?,
                })
            }
        }

        impl ToTokens for $it {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                tokens.extend(quote!{
                    [ #self.args ]
                })
            }
        }
    };
    (@bracebracket $it:ident, $sub:path, $punc:ty) => {
        #[derive(Clone)]
        struct $it {
            bracket_token: syn::token::Brace,
            args: syn::punctuated::Punctuated<$sub, $punc>,
        }

        impl Parse for $it {
            fn parse(input: ParseStream) -> Result<Self> {
                let content;

                Ok(Self {
                    bracket_token: syn::braced!(content in input),
                    args: syn::punctuated::Punctuated::<$sub, $punc>::parse_terminated(
                        &content,
                    )?,
                })
            }
        }

        impl ToTokens for $it {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                tokens.extend(quote!{
                    [ #self.args ]
                })
            }
        }
    };
    (@anglebracket $it:ident, $sub:path, $punc:ty) => {
        #[derive(Clone)]
        struct $it {
            lt_token: Token![<],
            args: syn::punctuated::Punctuated<$sub, $punc>,
            gt_token: Token![>],
        }

        impl Parse for $it {
            fn parse(input: ParseStream) -> Result<Self> {
                let lt_token = input.parse()?;

                let args = if input.peek(Token![>]) {
                    syn::punctuated::Punctuated::<$sub, $punc>::new()
                } else {
                    syn::punctuated::Punctuated::<$sub, $punc>::parse_separated_nonempty(
                        &input,
                    )?
                };

                let gt_token = input.parse()?;

                Ok(Self {
                    lt_token,
                    args,
                    gt_token
                })
            }
        }

        impl ToTokens for $it {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                tokens.extend(quote!{
                    < #self.args >
                })
            }
        }
    }
}

///
/// `@anglebracket` => `AngleBracketed$name`
macro_rules! bracket_punctuated {
    (@anglebracket @comma $name:ident) => {
        paste::paste! {
            _bracket_punctuated!(@anglebracket [<AngleBracketed $name s>], $name, syn::Token![,]);
        }
    };
    (@squarebracket @comma $name:ident) => {
        paste::paste! {
            _bracket_punctuated!(@squarebracket [<SquareBracketed $name s>], $name, syn::Token![,]);
        }
    };
    (@bracebracket @comma $name:ident) => {
        paste::paste! {
            _bracket_punctuated!(@bracebracket [<BraceBracketed $name s>], $name, syn::Token![,]);
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Structures

struct Resources {
    items: Punctuated<EitherResourceFileOrResourceDir, Token![,]>,
}

#[derive(Clone)]
struct ResourceDir {
    name: Ident,
    colon_token: Option<Token![:]>,
    brace_token: Brace,
    fields: Punctuated<Box<EitherResourceFileOrResourceDir>, Token![,]>,
}

#[derive(Parse, Clone)]
struct ResourceFile {
    name: Ident,
    maybe_colon_token: Token![:],
    path: LitStr,
}

either_wrapper!(ResourceFile, ResourceDir);

#[derive(Parse)]
struct TraitImplHeader {
    generics: Generics,
    either_underscope_or_path: EitherUnderscoreOrPath,
    maybe_args: MaybeAngleBracketedGenericArguments,
    maybe_type_bounds: MaybeSquareBracketedAssocArguments,
    for_token: Token![for],
    self_: Path,
    maybe_where_clause: Option<WhereClause>,
}

#[derive(Parse, Clone)]
struct PositionalReference {
    dollar_token: Token![$],
    idx: LitInt,
}

struct GenericArguments {}

#[derive(Parse, ToTokens, Clone)]
struct AssocArgument {
    ident: Ident,
    generics: MaybeAngleBracketedGenericArguments,
    eq_token: Token![=],
    ty: Type,
}

/// Alias of ToTokens
#[repr(transparent)]
struct Quoted(proc_macro2::TokenStream);

bracket_punctuated!(@anglebracket @comma PositionalReference);
bracket_punctuated!(@squarebracket @comma AssocArgument);

maybe_wrapper!(AngleBracketedPositionalReferences +ToTokens);
maybe_wrapper!(SquareBracketedAssocArguments +ToTokens);
maybe_wrapper!(AngleBracketedGenericArguments +ToTokens);

either_wrapper!(Underscore, Path, +ToTokens);

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl Parse for Resources {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Punctuated::new();

        while !input.is_empty() {
            items.push_value(input.parse()?);

            if input.is_empty() {
                break;
            }

            items.push_punct(input.parse()?);
        }

        Ok(Self { items })
    }
}

impl Parse for ResourceDir {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let colon_token = input.parse()?;

        let content;
        let brace_token = braced!(content in input);

        let mut fields = Punctuated::new();

        while !content.is_empty() {
            let either_resource_file_or_dir = Box::new(content.parse()?);
            fields.push_value(either_resource_file_or_dir);

            if content.is_empty() {
                break;
            }

            fields.push_punct(content.parse()?);
        }

        Ok(Self {
            name,
            colon_token,
            brace_token,
            fields,
        })
    }
}

impl ToTokens for Quoted {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.clone());
    }
}

impl TraitImplHeader {
    pub fn bound_type(&self, name: &str) -> Type {
        self.maybe_type_bounds
            .value
            .as_ref()
            .map(|bounds| bounds.bound_type(name))
            .unwrap()
    }

    // pub fn resolve_argument_references(
    //     &self,
    // ) -> MaybeAngleBracketedGenericArguments {
    //     MaybeAngleBracketedGenericArguments {
    //         value: self.maybe_args.value.clone().map(
    //             |AngleBracketedPositionalReferences {
    //                  lt_token,
    //                  mut args,
    //                  gt_token,
    //              }| {

    //                 AngleBracketedGenericArguments {
    //                     colon2_token: None,
    //                     lt_token,
    //                     args: args.into_iter().map(|pos| {
    //                        match self.generics.params.get(pos.int()).unwrap() {
    //                         GenericParam::Lifetime(lifetime_param) => GenericArgument::Lifetime(lifetime_param.lifetime.clone()),
    //                         GenericParam::Type(type_param) => GenericArgument::Type(Type),
    //                         GenericParam::Const(const_param) => todo!(),
    //                     }
    //                     }).collect(),
    //                     gt_token,
    //                 }
    //             },
    //         ),
    //     }
    // }
}

impl PositionalReference {
    pub fn int(&self) -> usize {
        self.idx.base10_parse().unwrap()
    }
}

impl SquareBracketedAssocArguments {
    pub fn bound_type(&self, name: &str) -> Type {
        self.args
            .iter()
            .find(|arg| &arg.ident.to_string() == name)
            .map(|arg| arg.ty.clone())
            .unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Procedural Macros

#[proc_macro]
pub fn resources(input: TokenStream) -> TokenStream {
    let resources = parse_macro_input!(input as Resources);

    TokenStream::from(quote! { #resources })
}
