use std::{
    io::Write,
    process::{Command, Stdio},
};

use proc_macro2::TokenStream;

pub fn rustfmt_ext(content: TokenStream) -> anyhow::Result<String> {
    let string = content.to_string();

    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(string.as_bytes())?;
    drop(stdin);

    let output = child.wait_with_output()?;

    Ok(String::from_utf8(output.stdout)?)
}
