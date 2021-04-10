# lc3rs

lc3rs is an lc3 virtual machine. If you just want to run lc3 binaries, all you need to do is build lc3rs and point it at your binary through the command line (see the usage section below) The library can also be imported and embedded in a larger Rust application. It offers hooks for extension through the Plugin trait and redirection of input/output streams through the IOHandle trait.

Many thanks to Justin Meiners for his [fantastic walkthrough of writing an LC3 virtual machine in C](https://justinmeiners.github.io/lc3-vm/), which made the process of implementing the VM very straightforward.

## Command Line Usage

Basic usage:

```
/path/to/lc3rs/binary/lc3rs /path/to/your/lc3/program/
```

By default, lc3rs assumes that your program is big-endian. If you're passing it a little-endian binary you can use the -l / --little-endian flag to make sure the bytes are flipped the correct way for execution on the way in.

The command line can also write a debug log to a separate file during execution using the -d/--debug-log-path argument:

```
/path/to/lc3rs/binary/lc3rs --debug-log-path ~/debug_log.txt /path/to/your/lc3/program
```

If you do use a debug log, be aware that it can chew through disk space very fast since it logs every event (command execution, memory read, register read etc.) that occurs during execution.

## Installation Notes

lc3rs depends on [device query](https://github.com/ostrosco/device_query). On Windows and MacOS it should work out of the box but on Linux you'll also need to install the X11 development libraries (libx11-dev on Debian or xorg-x11-server-devel on Fedora).
