#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate walkdir;

use std::env;
use std::fs;
use std::io::{stderr, stdout, Write};
use std::path;
use std::process::exit;
use arg_parser::ArgParser;
use extra::io::fail;
use extra::option::OptionalExt;
use walkdir::WalkDir;

const MAN_PAGE: &'static str = /* @MANSTART{mv} */ r#"
NAME
    mv - move files

SYNOPSIS
    mv [ -h | --help ] SOURCE_FILE(S) DESTINATION_FILE

DESCRIPTION
    The mv utility renames the file named by the SOURCE_FILE operand to the destination path
    named by the DESTINATION_FILE operand. Otherwise moves files to new destination.

OPTIONS
    --help, -h
        print this message
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["r", "recursive"])
        .add_flag(&["n", "no-action"])
        .add_flag(&["v", "verbose"])
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        exit(0);
    }

    let recurse = parser.found("recursive");
    let verbose = parser.found("verbose");
    let execute = ! parser.found("no-action");

    if parser.args.is_empty() {
        fail("No source argument. Use --help to see the usage.", &mut stderr);
    } else if parser.args.len() == 1 {
        fail("No destination argument. Use --help to see the usage.", &mut stderr);
    } else {
        // This unwrap won't panic since it's been verified not to be empty
        let dst = parser.args.pop().unwrap();
        let dst = path::PathBuf::from(dst);
        if dst.is_file() {
            if parser.args.len() == 2 {
                let src = path::Path::new(&parser.args[0]);
                if src.is_file() {
                    if execute {
                        fs::rename(src, dst).try(&mut stderr);
                    }
                    if verbose {
                        println!("{}", src.display());
                    }
                } else {
                    fail("Cannot move directory unto a file. Use --help to see the usage.", &mut stderr)
                }
            } else {
                fail("When moving multiple objects, destination should be to a path, not a file. Use --help to see the usage.", &mut stderr);
            }
        } else if dst.is_dir() {
            for ref arg in parser.args {
                let src = path::Path::new(arg);
                if src.is_dir() && !recurse {
                    fail("Can not move directories non-recursive", &mut stderr);
                }
                if recurse {
                    for entry in WalkDir::new(arg) {
                        let entry = entry.unwrap();
                        let src = path::Path::new(entry.path());
                        if execute {
                            if src.is_dir() {
                                fs::create_dir(dst.join(src)).try(&mut stderr);
                            } else if src.is_file() {
                                fs::rename(src, dst.join(src)).try(&mut stderr);
                            } // here we might also want to check for symlink, hardlink, socket, block, char, ...?
                        }
                        if verbose {
                            println!("{}", src.display());
                        }
                    }
                } else {
                    if execute {
                        fs::rename(src, dst.join(src.file_name().try(&mut stderr))).try(&mut stderr);
                    }
                    if verbose {
                        println!("{}", src.display());
                    }
                }
            }
        } else {
            fail("No destination found. Use --help to see the usage.", &mut stderr);
        }
    }
}
