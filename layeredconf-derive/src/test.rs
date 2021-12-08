mod test_util;

#[cfg(test)]
use std::io::Write;

use darling::FromDeriveInput;
use goldenfile::Mint;
use quote::quote;

use crate::LayeredConfStruct;
use test_util::rustfmt_ext;

#[test]
fn test() {
    let mut mint = Mint::new("tests/goldenfiles");
    let mut file = mint.new_goldenfile("test.rs").unwrap();

    let good_input = r#"
#[derive(LayeredConf)]
struct Test {
    boolean: bool,
    integer: u64,
}
"#;
    let parsed = syn::parse_str(good_input).unwrap();
    let conf_struct = LayeredConfStruct::from_derive_input(&parsed).unwrap();

    file.write_all(rustfmt_ext(quote!(#conf_struct)).unwrap().as_bytes())
        .unwrap();
}

#[test]
fn test_option() {
    let mut mint = Mint::new("tests/goldenfiles");
    let mut file = mint.new_goldenfile("test_option.rs").unwrap();

    let good_input = r#"
#[derive(LayeredConf)]
struct Test {
    boolean: bool,
    integer: u64,
    optional: Option<String>,
}
"#;
    let parsed = syn::parse_str(good_input).unwrap();
    let conf_struct = LayeredConfStruct::from_derive_input(&parsed).unwrap();

    file.write_all(rustfmt_ext(quote!(#conf_struct)).unwrap().as_bytes())
        .unwrap();
}

#[test]
fn test_passes_serde_attributes() {
    let mut mint = Mint::new("tests/goldenfiles");
    let mut file = mint
        .new_goldenfile("test_passes_serde_attributes.rs")
        .unwrap();

    let good_input = r#"
#[derive(LayeredConf, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Test {
    #[serde(rename = "bool")]
    boolean: bool,
    integer: u64,
    optional: Option<String>,
}
"#;
    let parsed = syn::parse_str(good_input).unwrap();
    let conf_struct = LayeredConfStruct::from_derive_input(&parsed).unwrap();

    file.write_all(rustfmt_ext(quote!(#conf_struct)).unwrap().as_bytes())
        .unwrap();
}

#[test]
fn test_subconfig_field() {
    let mut mint = Mint::new("tests/goldenfiles");
    let mut file = mint.new_goldenfile("test_subconfig_field.rs").unwrap();

    let input = r#"
#[derive(LayeredConf, serde::Deserialize)]
struct Test {
    name: String,
    #[layered(subconfig)]
    subconfig: TestSubConfig,
}
"#;
    let parsed = syn::parse_str(input).unwrap();
    let conf_struct = LayeredConfStruct::from_derive_input(&parsed).unwrap();

    file.write_all(rustfmt_ext(quote!(#conf_struct)).unwrap().as_bytes())
        .unwrap();
}

#[test]
fn test_subconfig_struct() {
    let mut mint = Mint::new("tests/goldenfiles");
    let mut file = mint.new_goldenfile("test_subconfig_struct.rs").unwrap();

    let input = r#"
#[derive(LayeredConf, serde::Deserialize)]
#[layered(subconfig)]
struct TestSubConfig {
    test: String,
}
"#;
    let parsed = syn::parse_str(input).unwrap();
    let conf_struct = LayeredConfStruct::from_derive_input(&parsed).unwrap();

    file.write_all(rustfmt_ext(quote!(#conf_struct)).unwrap().as_bytes())
        .unwrap();
}
