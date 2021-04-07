use structopt::StructOpt;

use crate::error::{BoxErrors, PublicResult};

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Options {
    pub path: String,
    #[structopt(long, short)]
    pub debug_log_path: Option<String>,
    #[structopt(short, long)]
    pub little_endian: bool, 
}

pub fn read_program(path: &String, little_endian: bool) -> PublicResult<Vec<u16>> {
    let bytes = std::fs::read(path).box_error()?;

    let mut commands: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|a| (a[0] as u16, a[1] as u16))
        .map(|a| a.1 + (a.0 << 8))
        .collect();

    if little_endian {
        commands  = commands.iter().map(|a| a.swap_bytes()).collect()
    }

    Ok(commands)
}
