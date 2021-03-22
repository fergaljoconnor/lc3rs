use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Options {
    pub path: String,
    #[structopt(long, short)]
    pub debug_log_path: Option<String>,
}

pub fn read_program(path: &String) -> Vec<u16> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => panic!("{}", e),
    };

    bytes
        .chunks_exact(2)
        .map(|a| (a[0] as u16, a[1] as u16))
        .map(|a| a.1 + (a.0 << 8))
        .collect()
}
