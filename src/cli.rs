use structopt::StructOpt;

use crate::error::{BoxErrors, PublicResult};

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Options {
    pub path: String,
    #[structopt(long, short)]
    pub debug_log_path: Option<String>,
}

pub fn read_program(path: &String) -> PublicResult<Vec<u16>> {
    let bytes = std::fs::read(path).box_error()?;

    let commands = bytes
        .chunks_exact(2)
        .map(|a| (a[0] as u16, a[1] as u16))
        .map(|a| a.1 + (a.0 << 8))
        .collect();
    Ok(commands)
}
