#[cfg(test)]
mod test_util;

use std::vec;

use darling::{
    ast::{self},
    util::{self, Ignored},
    FromDeriveInput, FromField, FromMeta, ToTokens,
};
use proc_macro::{self, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, GenericArgument, Ident, Path, PathArguments, Type};

#[proc_macro_derive(LayeredConf, attributes(layered))]
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
    data: ast::Data<util::Ignored, LayeredConfField>,
    attrs: Vec<syn::Attribute>,
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

    fn to_layer_tokens(
        &self,
        ident: &Ident,
        data: &ast::Data<Ignored, LayeredConfField>,
        attrs: &[Attribute],
    ) -> proc_macro2::TokenStream {
        let layer_ident = format_ident!("{}Layer", ident);

        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let option_field_list = fields
            .clone()
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
                    (true, true, Some(subtype)) => {
                        let subtype_id = match &subtype {
                            Type::Path(path) => match path.path.segments.first() {
                                Some(seg) => &seg.ident,
                                _ => panic!("Can't find ident"),
                            },
                            _ => panic!("Can't find ident"),
                        };
                        let layer_subtype = format_ident!("{}Layer", subtype_id);
                        quote! {
                            #[serde(default, skip_serializing_if = "Option::is_none")]
                            #(#attrs)*
                            #name: Option<#layer_subtype>,
                        }
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

                        quote! {
                            #[serde(default, skip_serializing_if = "Option::is_none")]
                            #(#attrs)*
                            #name: Option<#layer_ty>,
                        }
                    }
                    (false, true, Some(..)) => {
                        panic!("Subtype not extracted {:?} {:?}", name, ty);
                    }
                }
            })
            .collect::<Vec<_>>();

        let default_field_list = fields
            .into_iter()
            .map(|f| {
                let name = &f.ident;

                quote! {
                    #name: None,
                }
            })
            .collect::<Vec<_>>();

        let container_attrs = attrs
            .iter()
            .map(|a| a.into_token_stream())
            .collect::<Vec<_>>();

        quote! {
            impl layeredconf::LayeredConfSolid for #ident {
                type Layer = #layer_ident;
            }
            impl layeredconf::LayeredConfLayer for #layer_ident {
                type Config = #ident;
            }
            #[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
            #(#container_attrs)*
            struct #layer_ident {
                #(#option_field_list)*
            }
            impl std::default::Default for #layer_ident {
                fn default() -> Self {
                    Self {
                        #(#default_field_list)*
                    }
                }
            }
        }
    }

    fn to_merge_tokens(
        &self,
        ident: &Ident,
        data: &ast::Data<Ignored, LayeredConfField>,
    ) -> proc_macro2::TokenStream {
        let layer_ident = format_ident!("{}Layer", ident);

        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        let field_list = fields
            .into_iter()
            .map(|f| {
                let name = &f.ident;
                if f.subconfig {
                    quote! {
                        if self.#name.is_none() {
                            self.#name = other.#name.clone();
                        } else if let Some(other) = &other.#name {
                            self.#name.as_mut().unwrap().merge_from(other);
                        }
                    }
                } else {
                    quote! {
                        if self.#name.is_none() {
                            self.#name = other.#name.clone();
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

    fn to_solidify_tokens(
        &self,
        ident: &Ident,
        data: &ast::Data<Ignored, LayeredConfField>,
    ) -> proc_macro2::TokenStream {
        let layer_ident = format_ident!("{}Layer", ident);

        let fields = data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

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
                        quote! {
                            let #name;
                            if let Some(v) = &self.#name {
                                #name = Some(v.solidify());
                            } else {
                                #name = None;
                            }
                        }
                    } else {
                        quote! {
                            let #name = self.#name.clone();
                        }
                    }
                } else if f.subconfig {
                    quote! {
                        let #name;
                        if let Some(val) = &self.#name {
                            #name = Some(val.solidify()?);
                        } else {
                            #name = None;
                            missing.push(#name_str.to_string());
                        }
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

                if option {
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
        let LayeredConfStruct {
            ref ident,
            ref data,
            ref attrs,
            ..
        } = *self;

        tokens.extend(self.to_layer_tokens(ident, data, attrs));
        tokens.extend(self.to_merge_tokens(ident, data));
        tokens.extend(self.to_solidify_tokens(ident, data));
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
mod test {
    use std::io::Write;

    use darling::FromDeriveInput;
    use goldenfile::Mint;
    use quote::quote;

    use crate::{test_util::rustfmt_ext, LayeredConfStruct};

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut mint = Mint::new("tests/goldenfiles");
        let mut file = mint.new_goldenfile("test.rs")?;

        let good_input = r#"
#[derive(LayeredConf)]
struct Test{
    boolean: bool,
    integer: u64,
}
"#;
        let parsed = syn::parse_str(good_input)?;
        let conf_struct = LayeredConfStruct::from_derive_input(&parsed).unwrap();

        file.write_all(rustfmt_ext(quote!(#conf_struct))?.as_bytes())?;

        Ok(())
    }

    #[test]
    fn test_option() -> anyhow::Result<()> {
        let mut mint = Mint::new("tests/goldenfiles");
        let mut file = mint.new_goldenfile("test_option.rs")?;

        let good_input = r#"
#[derive(LayeredConf)]
struct Test{
    boolean: bool,
    integer: u64,
    optional: Option<String>,
}
"#;
        let parsed = syn::parse_str(good_input)?;
        let conf_struct = LayeredConfStruct::from_derive_input(&parsed).unwrap();

        file.write_all(rustfmt_ext(quote!(#conf_struct))?.as_bytes())?;

        Ok(())
    }

    #[test]
    fn test_passes_serde_attributes() -> anyhow::Result<()> {
        let mut mint = Mint::new("tests/goldenfiles");
        let mut file = mint.new_goldenfile("test_passes_serde_attributes.rs")?;

        let good_input = r#"
#[derive(LayeredConf, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Test{
    #[serde(rename = "bool")]
    boolean: bool,
    integer: u64,
    optional: Option<String>,
}
"#;
        let parsed = syn::parse_str(good_input)?;
        let conf_struct = LayeredConfStruct::from_derive_input(&parsed).unwrap();

        file.write_all(rustfmt_ext(quote!(#conf_struct))?.as_bytes())?;

        Ok(())
    }
}
