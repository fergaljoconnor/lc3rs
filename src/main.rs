use lc3rs::cli:: {Options, read_program};
use lc3rs::vm::VM;
use structopt::StructOpt;

fn main() {
    let options = Options::from_args();
    let program = read_program(&options.path);

    let mut vm = VM::new();
    vm.load_program(&program);
    vm.run();
}
