//!# [LayeredConf](https://crates.io/crates/layeredconf) Derive Macro
//!
//!## Yet Another Config Package

use std::vec;

use darling::{
    ast,
    util::{Ignored, Override},
    FromDeriveInput, FromField, FromMeta, ToTokens,
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, GenericArgument, Ident, Path, PathArguments, Type};

#[proc_macro_derive(LayeredConf, attributes(layered, clap))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let conf_struct = LayeredConfStruct::from_derive_input(&input).expect("Wrong options");

    let tokens = quote! { #conf_struct };

    tokens.into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(layered),
    forward_attrs(clap, serde),
    supports(struct_named)
)]
struct LayeredConfStruct {
    ident: Ident,
    data: ast::Data<Ignored, LayeredConfField>,
    attrs: Vec<syn::Attribute>,

    #[darling(default)]
    subconfig: bool,
    #[darling(default)]
    default: bool,
}

impl LayeredConfStruct {
    fn is_option(&self, ty: &Type) -> bool {
        match ty {
            Type::Path(path) => match path.path.segments.first() {
                Some(seg) => seg.ident == "Option",
                _ => false,
            },
            _ => false,
        }
    }

    fn extract_type(&self, ty: &Type) -> Option<Type> {
        fn path_is_option(path: &Path) -> bool {
            path.leading_colon.is_none()
                && path.segments.len() == 1
                && path.segments.first().unwrap().ident == "Option"
        }
        match ty {
            Type::Path(tp) if tp.qself.is_none() && path_is_option(&tp.path) => {
                if let Some(seg) = tp.path.segments.first() {
                    match &seg.arguments {
                        PathArguments::AngleBracketed(params) => match params.args.first() {
                            Some(GenericArgument::Type(ty)) => Some(ty.clone()),
                            _ => None,
                        },
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn layer_ident(&self) -> Ident {
        format_ident!("{}Layer", self.ident)
    }

    fn fields(&self) -> Vec<&LayeredConfField> {
        self.data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields
    }

    fn to_layer_tokens(&self) -> proc_macro2::TokenStream {
        let layer_ident = self.layer_ident();

        let fields = self.fields();

        let option_field_list = fields
            .into_iter()
            .map(|f| {
                let name = &f.ident;
                let ty = &f.ty;

                let attrs = f
                    .attrs
                    .iter()
                    .map(|a| a.into_token_stream())
                    .collect::<Vec<_>>();

                let option = self.is_option(ty);
                let subtype = if option { self.extract_type(ty) } else { None };
                let subconfig = f.subconfig;
                match (option, subconfig, subtype) {
                    (true, false, _) => {
                        quote! {
                            #[serde(default, skip_serializing_if = "Option::is_none")]
                            #(#attrs)*
                            #name: #ty,
                        }
                    }
                    (true, true, Some(_)) => {
                        panic!("layered(subconfig) should not be wrapped in Option")
                    }
                    (true, true, None) => {
                        panic!("Subtype not extracted {:?} {:?}", name, ty);
                    }
                    (false, false, _) => {
                        quote! {
                            #[serde(default, skip_serializing_if = "Option::is_none")]
                            #(#attrs)*
                            #name: Option<#ty>,
                        }
                    }
                    (false, true, None) => {
                        let ty_id = match &ty {
                            Type::Path(path) => match path.path.segments.first() {
                                Some(seg) => &seg.ident,
                                _ => panic!("Can't find ident"),
                            },
                            _ => panic!("Can't find ident"),
                        };
                        let layer_ty = format_ident!("{}Layer", ty_id);

                        let skip_serializing_if = quote! { #layer_ty::empty }.to_string();

                        quote! {
                            #[serde(default, skip_serializing_if = #skip_serializing_if)]
                            #[clap(flatten)]
                            #(#attrs)*
                            #name: #layer_ty,
                        }
                    }
                    (false, true, Some(..)) => {
                        panic!("Subtype not extracted {:?} {:?}", name, ty);
                    }
                }
            })
            .collect::<Vec<_>>();

        let container_attrs = self
            .attrs
            .iter()
            .map(|a| a.into_token_stream())
            .collect::<Vec<_>>();

        let clap_derive = if self.subconfig {
            quote! { clap::Args }
        } else {
            quote! { clap::Parser }
        };

        quote! {
            #[derive(serde::Deserialize, serde::Serialize, #clap_derive, Clone, Debug)]
            #(#container_attrs)*
            struct #layer_ident {
                #(#option_field_list)*
            }
        }
    }

    fn to_impl_layered_conf_tokens(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let layer_ident = self.layer_ident();

        let fields = self.fields();

        let load_config_field_list = fields
            .clone()
            .into_iter()
            .filter(|f| f.load_config)
            .map(|f| {
                let ident = &f.ident;

                quote! {
                    if let Some(load_config) = &self.#ident {
                        load_configs.push(load_config.clone());
                    }
                }
            })
            .collect::<Vec<_>>();

        let default_layer = match self.default {
            true => Some(quote! {
                let default = #ident::default();
            }),
            false => None,
        };

        let default_layer_field_list = fields
            .into_iter()
            .map(|f| {
                let name = &f.ident;
                let ty = &f.ty;
                let subconfig = f.subconfig;
                let default = &f.default;

                let ty_id = match &ty {
                    Type::Path(path) => match path.path.segments.first() {
                        Some(seg) => &seg.ident,
                        _ => panic!("Can't find ident"),
                    },
                    _ => panic!("Can't find ident"),
                };
                let layer_ident = format_ident!("{}Layer", ty_id);

                if subconfig {
                    quote! {
                        #name: #layer_ident::default_layer(),
                    }
                } else {
                    match default {
                        Some(Override::Explicit(path)) => quote! {
                            #name: Some(#path()),
                        },
                        Some(Override::Inherit) => quote! {
                            #name: Some(std::default::Default::default()),
                        },
                        None => match self.default {
                            true => quote! {
                                #name: Some(default.#name),
                            },
                            false => quote! {
                                #name: None,
                            },
                        },
                    }
                }
            })
            .collect::<Vec<_>>();

        quote! {
            impl layeredconf::LayeredConfSolid for #ident {
                type Layer = #layer_ident;
            }
            impl layeredconf::LayeredConfLayer for #layer_ident {
                type Config = #ident;

                fn load_configs(&self) -> Vec<std::path::PathBuf> {
                    let mut load_configs = vec![];

                    #(#load_config_field_list)*

                    load_configs
                }

                fn default_layer() -> Self {
                    #default_layer

                    Self {
                        #(#default_layer_field_list)*
                    }
                }
            }
        }
    }

    fn to_layer_impl_tokens(&self) -> proc_macro2::TokenStream {
        let layer_ident = self.layer_ident();

        let fields = self.fields();

        let empty_field_list = fields
            .into_iter()
            .map(|f| {
                let ident = &f.ident;
                let subconfig = f.subconfig;

                if subconfig {
                    quote! {
                        empty.push(self.#ident.empty());
                    }
                } else {
                    quote! {
                        empty.push(self.#ident.is_none());
                    }
                }
            })
            .collect::<Vec<_>>();

        quote! {
            impl #layer_ident {
                fn empty(&self) -> bool {
                    let mut empty = vec![];

                    #(#empty_field_list)*

                    empty.iter().all(|v| *v)
                }
            }
        }
    }

    fn to_layer_default_tokens(&self) -> proc_macro2::TokenStream {
        let layer_ident = self.layer_ident();

        let fields = self.fields();

        let std_default_field_list = fields
            .into_iter()
            .map(|f| {
                let name = &f.ident;
                let ty = &f.ty;
                let subconfig = f.subconfig;

                let ty_id = match &ty {
                    Type::Path(path) => match path.path.segments.first() {
                        Some(seg) => &seg.ident,
                        _ => panic!("Can't find ident"),
                    },
                    _ => panic!("Can't find ident"),
                };
                let layer_ident = format_ident!("{}Layer", ty_id);

                if subconfig {
                    quote! {
                        #name: #layer_ident::default(),
                    }
                } else {
                    quote! {
                        #name: None,
                    }
                }
            })
            .collect::<Vec<_>>();

        quote! {
            impl std::default::Default for #layer_ident {
                fn default() -> Self {
                    Self {
                        #(#std_default_field_list)*
                    }
                }
            }
        }
    }

    fn to_merge_tokens(&self) -> proc_macro2::TokenStream {
        let layer_ident = self.layer_ident();

        let fields = self.fields();

        let field_list = fields
            .into_iter()
            .map(|f| {
                let ident = &f.ident;
                if f.subconfig {
                    quote! {
                        self.#ident.merge_from(&other.#ident);
                    }
                } else {
                    quote! {
                        if self.#ident.is_none() {
                            self.#ident = other.#ident.clone();
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

        quote! {
            impl layeredconf::LayeredConfMerge<#layer_ident> for #layer_ident {
                fn merge_from(&mut self, other: &#layer_ident) {
                    #(#field_list)*
                }
            }
        }
    }

    fn to_solidify_tokens(&self) -> proc_macro2::TokenStream {
        let layer_ident = self.layer_ident();

        let fields = self.fields();

        let field_list = fields
            .clone()
            .into_iter()
            .map(|f| {
                let name = &f.ident;
                let name_str = name.as_ref().map(|id| id.to_string());
                let ty = &f.ty;

                let option = self.is_option(ty);

                if option {
                    if f.subconfig {
                        panic!("layered(subconfig) shouldn't be wrapped in Option");
                    } else {
                        quote! {
                            let #name = self.#name.clone();
                        }
                    }
                } else if f.subconfig {
                    quote! {
                        let #name = self.#name.solidify()?;
                    }
                } else {
                    quote! {
                        let #name;
                        if let Some(val) = &self.#name {
                            #name = Some(val.clone());
                        } else {
                            #name = None;
                            missing.push(#name_str.to_string());
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

        let field_destructure = fields
            .into_iter()
            .map(|f| {
                let name = &f.ident;
                let ty = &f.ty;
                let option = self.is_option(ty);

                if option || f.subconfig {
                    quote! {
                        #name,
                    }
                } else {
                    quote! {
                        #name: #name.unwrap(),
                    }
                }
            })
            .collect::<Vec<_>>();

        let ident = &self.ident;

        quote! {
            impl layeredconf::LayeredConfSolidify<#ident> for #layer_ident {
                fn solidify(&self) -> layeredconf::Result<#ident> {
                    let mut missing = vec![];

                    #(#field_list)*

                    if !missing.is_empty() {
                        return Err(layeredconf::Error::SolidifyFailedMissing {
                            missing,
                        });
                    }

                    Ok(#ident {
                        #(#field_destructure)*
                    })
                }
            }
        }
    }
}

impl ToTokens for LayeredConfStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.to_layer_tokens());
        tokens.extend(self.to_impl_layered_conf_tokens());
        tokens.extend(self.to_layer_impl_tokens());
        tokens.extend(self.to_layer_default_tokens());
        tokens.extend(self.to_merge_tokens());
        tokens.extend(self.to_solidify_tokens());
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(layered), forward_attrs(clap, serde))]
struct LayeredConfField {
    ident: Option<Ident>,
    ty: Type,
    attrs: Vec<syn::Attribute>,

    // Our attrs
    #[darling(default)]
    subconfig: bool,
    #[darling(default)]
    load_config: bool,
    #[darling(default)]
    default: Option<Override<Path>>,
}

#[derive(Debug, FromMeta)]
enum LayeredConfFieldSkip {
    None,
    Args,
}

impl std::default::Default for LayeredConfFieldSkip {
    fn default() -> Self {
        LayeredConfFieldSkip::None
    }
}

#[cfg(test)]
mod test;
