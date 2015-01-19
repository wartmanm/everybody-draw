// build.rs
extern crate libc;

use std::io::Command;
use std::io::process::StdioContainer;
use libc::consts::os::posix88::STDERR_FILENO;

fn main() {
    // note that there are a number of downsides to this approach, the comments
    // below detail how to improve the portability of these commands.
    Command::new("/usr/bin/env")
        .args(&["mkbindings.py", "--prefix", "./src", "bindings.json", "build"])
        .stdout(StdioContainer::InheritFd(STDERR_FILENO))
        .status().unwrap();
}
