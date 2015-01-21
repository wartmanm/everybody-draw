// build.rs
//
#![allow(unstable)]

extern crate libc;

use std::io::Command;
use std::io::process::StdioContainer;
use libc::consts::os::posix88::STDERR_FILENO;

fn main() {
    let result = Command::new("/usr/bin/env")
        .args(&["python", "mkbindings.py", "-v", "--prefix", "./src", "bindings.json", "build"])
        .stdout(StdioContainer::InheritFd(STDERR_FILENO))
        .stderr(StdioContainer::InheritFd(STDERR_FILENO))
        .status().unwrap()
        .success();
    if !result {
        panic!("failed to generate bindings!");
    }
}
