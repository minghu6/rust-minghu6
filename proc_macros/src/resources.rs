use convert_case::{Case, Casing};
use either::{Left, Right};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Ident, LitStr};

use crate::{
    EitherResourceFileOrResourceDir, ResourceDir, ResourceFile, Resources,
};


macro_rules! ident {
    ($name:expr) => {
        Ident::new($name.as_str(), Span::call_site())
    };
}

macro_rules! litstr {
    ($name:expr) => {
        LitStr::new($name, Span::call_site())
    };
}

impl ToTokens for Resources {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut braced_content = quote! {};
        let prefix_name = "Res".to_string();

        for either_file_or_dir in self.items.iter() {
            let def_method = match &either_file_or_dir.value {
                Left(resfile) => {
                    tokens.extend(resource_file(prefix_name.clone(), resfile));
                    def_resource_file_method(prefix_name.clone(), resfile)
                }
                Right(resdir) => {
                    tokens.extend(resource_dir(prefix_name.clone(), resdir));
                    def_resource_dir_method(prefix_name.clone(), resdir)
                }
            };

            braced_content.extend(def_method);
        }

        tokens.extend(quote! {
            pub struct Res {
                path: std::path::PathBuf
            }

            impl Res {
                fn new() -> Self {
                    let config_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

                    Self {
                        path: config_dir.with_file_name("res")
                    }
                }

                #braced_content
            }
        })
    }
}

fn resource_dir(prefix_name: String, resdir: &ResourceDir) -> TokenStream {
    let mut tokens = quote! {};
    let mut braced_content = quote! {};

    for either_file_or_dir in &resdir.fields {
        let def_method = match &either_file_or_dir.value {
            Left(resfile) => {
                tokens.extend(resource_file(prefix_name.clone(), resfile));
                def_resource_file_method(prefix_name.clone(), resfile)
            }
            Right(resdir) => {
                tokens.extend(resource_dir(prefix_name.clone(), resdir));
                def_resource_dir_method(prefix_name.clone(), resdir)
            }
        };

        braced_content.extend(def_method);
    }

    let ResourceDir { name, .. } = &resdir;
    let (long_name_ident, .. )= assemble_name(prefix_name, name);

    tokens.extend(quote! {
        pub struct #long_name_ident {
            path: std::path::PathBuf
        }

        impl #long_name_ident {
            #braced_content
        }
    });

    tokens
}

fn resource_file(prefix_name: String, resfile: &ResourceFile) -> TokenStream {
    let ResourceFile { name, .. } = &resfile;
    let (long_name_ident, .. )= assemble_name(prefix_name, name);

    quote!{
        pub struct #long_name_ident {
            path: std::path::PathBuf
        }

        impl #long_name_ident {
            pub fn path(&self) -> &std::path::Path {
                self.path.as_path()
            }

            pub fn open(&self) -> std::fs::File {
                assert!(self.path.is_file(), "{:#?}", self.path);

                std::fs::File::open(&self.path).unwrap()
            }

            pub fn load(&self) -> Vec<u8> {
                assert!(self.path.is_file(), "{:#?}", self.path);

                std::fs::read(&self.path).unwrap()
            }

            pub fn load_to_string(&self) -> String {
                assert!(self.path.is_file(), "{:#?}", self.path);

                std::fs::read_to_string(&self.path).unwrap()
            }
        }
    }
}

fn def_resource_dir_method(prefix_name: String, resdir: &ResourceDir) -> TokenStream {
    let ResourceDir { name, .. } = &resdir;

    let (long_name_ident, name_litstr, .. )= assemble_name(prefix_name, name);

    quote! {
        pub fn #name(&self) -> #long_name_ident {
            #long_name_ident {
                path: self.path.join(#name_litstr)
            }
        }
    }
}

fn def_resource_file_method(prefix_name: String, resfile: &ResourceFile) -> TokenStream {
    let ResourceFile { name, path, .. } = &resfile;

    let (long_name_ident, .. )= assemble_name(prefix_name, name);

    quote! {
        pub fn #name(&self) -> #long_name_ident {
            #long_name_ident {
                path: self.path.join(#path)
            }
        }
    }
}

/// (long_name_ident, name_litstr, next_prefix_name = long_name_str)
fn assemble_name(mut prefix_name: String, name: &Ident) -> (Ident, LitStr, String) {
    // snake style name
    let name_str = name.to_string();
    assert_eq!(name_str, name_str.to_case(Case::Snake));

    // upper camel style prefix name
    assert_eq!(prefix_name, prefix_name.to_case(Case::UpperCamel));
    prefix_name.push_str(&name_str.to_case(Case::UpperCamel));

    (ident!(&prefix_name), litstr!(&name_str), prefix_name)
}


#[cfg(test)]
mod tests {

    use convert_case::{Case, Casing};


    #[test]
    fn verify_name_convert_case() {
        assert_eq!("AbcDefDd", "abc_def_dd".to_case(Case::UpperCamel));
        assert_eq!("abc_def_dd", "abc_def_dd".to_case(Case::Snake));
    }
}
