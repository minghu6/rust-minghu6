
use either::try_right;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Type};

use crate::{Quoted, TraitImplHeader};


macro_rules! quoted {
    ($($args:tt)*) => {
        Box::new(Quoted(quote! { $($args)* }))
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Functions

pub fn duck_typing_impl_countable_(
    implhdr: &TraitImplHeader,
) -> proc_macro2::TokenStream {
    let TraitImplHeader {
        generics,
        either_underscope_or_path,
        for_token,
        self_,
        maybe_where_clause,
        ..
    } = implhdr;

    let traitname = quote! { Countable };

    quote! {
        impl #generics #traitname  #for_token #self_ #maybe_where_clause {
            fn len(&self) -> usize {
                self.len()
            }
        }
    }
}

pub fn duck_typing_impl_iterable_(
    implhdr: &TraitImplHeader,
) -> proc_macro2::TokenStream {
    let TraitImplHeader {
        generics,
        maybe_type_bounds,
        maybe_args,
        for_token,
        self_,
        maybe_where_clause,
        ..
    } = implhdr;

    let bound_type_item = implhdr.bound_type("Item");

    quote! {
        impl #generics Iterable<'a> #for_token #self_ #maybe_args #maybe_where_clause {
            type Item = #bound_type_item;

            fn iter(&'a self) -> impl Iterator<Item = &Self::Item> {
                self.iter()
            }
        }
    }
}

pub fn duck_typing_impl_collection_(
    implhdr: &TraitImplHeader,
) -> proc_macro2::TokenStream {
    let TraitImplHeader {
        generics,
        maybe_type_bounds,
        for_token,
        self_,
        maybe_where_clause,
        ..
    } = implhdr;

    quote! {
        impl #generics Collection #for_token #self_ #maybe_where_clause {
            fn validate(&self) {
                self.validate()
            }
        }
    }
}

pub fn duck_typing_impl_mapping_(
    implhdr: &TraitImplHeader
) -> proc_macro2::TokenStream {
    let TraitImplHeader {
        generics,
        maybe_type_bounds,
        for_token,
        self_,
        maybe_where_clause,
        ..
    } = implhdr;

    let bound_type_key = implhdr.bound_type("Key");
    let bound_type_value = implhdr.bound_type("Value");

    quote! {
        impl #generics Mapping #for_token #self_ #maybe_where_clause {
            type Key = #bound_type_key;
            type Value = #bound_type_value;

            fn get<Q>(&self, key: &Q) -> Option<&Self::Value>
            where
                Q: Ord + ?Sized,
                Self::Key: Borrow<Q>
            {
                self.get(key)
            }
        }
    }
}

pub fn duck_typing_impl_mutmapping_(
    implhdr: &TraitImplHeader,
) -> proc_macro2::TokenStream {
    let TraitImplHeader {
        generics,
        maybe_type_bounds,
        for_token,
        self_,
        maybe_where_clause,
        ..
    } = implhdr;

    quote! {
        impl #generics MutableMapping #for_token #self_ #maybe_where_clause {
            fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
                self.insert(key, value)
            }

            fn remove<Q>(&mut self, key: &Q) -> Option<Self::Value>
            where
                Q: Ord + ?Sized,
                Self::Key: Borrow<Q>
            {
                self.remove(key)
            }
        }
    }
}

pub fn duck_typing_recursive_impl_mutablemapping_(
    implhdr: &TraitImplHeader,
) -> proc_macro2::TokenStream {
    let TraitImplHeader {
        generics,
        maybe_type_bounds,
        for_token,
        self_,
        maybe_where_clause,
        ..
    } = implhdr;

    let bound_type_key = implhdr.bound_type("Key");
    let bound_type_value = implhdr.bound_type("Value");

    quote! {
        impl #generics Countable #for_token #self_ #maybe_where_clause {
            fn len(&self) -> usize {
                self.len()
            }
        }

        impl #generics Iterable #for_token #self_ #maybe_where_clause {
            type Item = (#bound_type_key, #bound_type_value);

            fn iter(&self) -> impl Iterator<Item = &Self::Item> {
                self.iter()
            }
        }

        impl #generics Collection #for_token #self_ #maybe_where_clause {
            fn validate(&self) {
                self.validate()
            }
        }

        impl #generics Mapping #for_token #self_ #maybe_where_clause {
            type Key = #bound_type_key;
            type Value = #bound_type_value;

            fn get<Q>(&self, key: &Q) -> Option<&Self::Value>
            where
                Q: Ord + ?Sized,
                Self::Key: Borrow<Q>
            {
                self.get(key)
            }
        }

        impl #generics MutableMapping #for_token #self_ #maybe_where_clause {
            fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value> {
                self.insert(key, value)
            }

            fn remove<Q>(&mut self, key: &Q) -> Option<Self::Value>
            where
                Q: Ord + ?Sized,
                Self::Key: Borrow<Q>
            {
                self.remove(key)
            }
        }
    }
}
