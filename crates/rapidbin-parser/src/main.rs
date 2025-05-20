use std::{io::Error, path::PathBuf, str::FromStr};

use clap::Parser;
use rapidbin_parser::{emit_text_format, parse_from_file};


#[derive(Parser)]
struct Cli {
    path: String
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let path = match PathBuf::from_str(&cli.path) {
        Ok(buf) => buf.into_boxed_path(),
        Err(e) => panic!("Error: {e}"),
    };
    
    let trace = parse_from_file(&path)?;
    let std_trace = emit_text_format(trace);

    println!("Trace:\n{}", std_trace.join("\n"));

    Ok(())
}
