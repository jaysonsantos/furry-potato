use std::env::args;

use color_eyre::{
    eyre::{bail, WrapErr},
    Result, Section,
};
use krak_it::Cli;
use tokio::{fs::File, io::stdout};

#[tokio::main]
async fn main() -> Result<()> {
    let input_file = if let Some(input_file) = args().nth(1) {
        input_file
    } else {
        bail!("Supply an input file");
    };

    let section = || format!("Input file: {input_file}");

    let input = File::open(&input_file)
        .await
        .wrap_err("failed to open input file")
        .with_section(section)?;

    let client = Cli::new().wrap_err("failed to create client")?;
    let output = stdout();
    client
        .process_and_print_transactions(input, output)
        .await
        .wrap_err("failed to process transactions")
        .with_section(section)?;
    Ok(())
}
