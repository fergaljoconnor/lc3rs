## lc3rs: A simple, extensible LC3 virtual machine

lc3rs is an lc3 virtual machine. If you just want to run lc3 binaries, all you need to do is build lc3rs and point it at your binary through the command line (see the usage section below) The library can also be imported and embedded in a larger Rust application. It offers hooks for extension through the Plugin trait and redirection of input/output streams through the IOHandle trait.

Many thanks to Justin Meiners for his [fantastic walkthrough of writing an LC3 virtual machine in C](https://justinmeiners.github.io/lc3-vm/), which made the process of implementing the VM very straightforward.

### Installation Notes

lc3rs depends on [device query](https://github.com/ostrosco/device_query). On Windows and MacOS it should work out of the box but on Linux you'll also need to install the X11 development libraries (libx11-dev on Debian or xorg-x11-server-devel on Fedora).

### Command Line Usage

Basic Usage:

```
/path/to/lc3rs /path/to/your/lc3/program.obj
```

By default, lc3rs assumes that your program is big-endian. If you're passing it a little-endian binary you can use the -l / --little-endian flag to flip the bytes on the way in.

The command line can also write a debug log to a separate file during execution using the -d/--debug-log-path argument:

```
/path/to/lc3rs --debug-log-path ~/debug_log.txt /path/to/your/lc3/program.obj
```

If you do use a debug log, be aware that it can eat disk space very fast since it logs every event (command execution, memory read, register read etc.) that occurs during execution.

### Embedded Usage

Basic Example:

```Rust
use lc3rs::{VM, LC3Error};

fn main() -> Result<(), LC3Error> {

    // Build a quick hello world program.
    let mut program: Vec<u16> = vec![
        // Write (incremented program counter + 2) into RR0
        0b1110_0000_0000_0010,
        // Print the string starting at the address in RR0
        0xF022,
        // Halt
        0xF025,
    ];
    let out_string = "Hello world!";
    let char_vals = out_string.chars().map(|ch| ch as u16);
    program.extend(char_vals);

    // Execute the program
    let mut vm = VM::new();
    vm.load_program(&program)?;
    vm.run()
}
```

Implementing A Custom IO Handle:

```Rust
use lc3rs::{IOHandle, LC3Result};

struct MyIOHandle {}

impl IOHandle for MyIOHandle {
    fn getchar(&self) -> LC3Result<char> {
       Ok('a')
    }

    fn putchar(&self, ch: char) -> LC3Result<()> {
        Ok(())
    }

    fn is_key_down(&self) -> LC3Result<bool> {
        Ok(true)
    }
}
```

Using your custom IO Handle:

```Rust
let io_handle = MyIOHandle{};
let mut vm = VM::new_with_io(io_handle);
```

Implementing a Custom Plugin

```Rust
use lc3rs::{Event, IOHandle, LC3Result, Plugin};

struct MyPlugin {}

impl<IOType: IOHandle> Plugin<IOType> for MyPlugin {
    fn handle_event(&mut self, _vm: &mut VM<IOType>, event: &Event) -> LC3Result<()> {
        println!("Event received");
        Ok(())
    }
}
```

Using Your Custom Plugin:

```Rust
let mut vm = VM::new();
let plugin = MyPlugin {};
vm.add_plugin(Box::new(plugin));
```
