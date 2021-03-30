use lc3rs::cli:: {Options, read_program};
use lc3rs::error::{BoxErrors, PublicResult};
use lc3rs::vm::VM;
use std::fs::File;
use lc3rs::plugin::debuglogger::DebugLogger;
use structopt::StructOpt;

fn main() -> PublicResult<()> {
    let options = Options::from_args();
    let program = read_program(&options.path)?;

    let mut vm = VM::new();

    if let Some(path) = options.debug_log_path {
        let debug_file = File::create(path)?;
        let logger = DebugLogger::new(debug_file);
        vm.add_plugin(Box::new(logger));
    }

    vm.load_program(&program)?;

    vm.run().box_error()
}
