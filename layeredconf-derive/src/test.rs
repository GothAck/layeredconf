mod test_util;

#[cfg(test)]
use std::io::Write;

use darling::FromDeriveInput;
use goldenfile::Mint;
use quote::quote;

use crate::LayeredConfStruct;
use test_util::rustfmt_ext;

#[test]
fn test() -> anyhow::Result<()> {
    let mut mint = Mint::new("tests/goldenfiles");
    let mut file = mint.new_goldenfile("test.rs")?;

    let good_input = r#"
#[derive(LayeredConf)]
struct Test {
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
struct Test {
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
struct Test {
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
