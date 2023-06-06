use std::env;
use crate::{utils::{exit, read_passfile, print_err}, commands::*, lpm_server};

pub fn check_arguments() {
    let arguments:String = env::args().collect::<Vec<String>>()[1..].join(" ");

    if arguments.is_empty() {
        return;
    }

    let first_argument:String = arguments.split(' ').collect::<Vec<&str>>()[0].trim().to_string();
    match first_argument.trim() {
        "-c"     => {
            let command = arguments.split(' ').collect::<Vec<&str>>()[1..].join(" ").trim().to_string();
            commander(command)
        },
        "--help" => {
            help();
            std::process::exit(0);
        },"--server" => {
            lpm_server::main();
        },
         _       => {
            exit(1, "not a valid argument, use lpm -c {command} or lpm --help, if not arguments are passed it will be turned into TUI mode")
         },
    }
}

fn commander (command: String) {

    let read_passfile_tuple = read_passfile();
    let mut passfile_data = read_passfile_tuple.0;
    let key = read_passfile_tuple.1;

    if command.starts_with("np") || command.starts_with("new password")  {
        np(&mut passfile_data, &key, command.to_owned());
        std::process::exit(0);
    }

    if command.starts_with("rm") || command.trim().starts_with("rem") || command.starts_with("del") {
        if command.split(' ').collect::<Vec<&str>>().len() > 2 {
            print_err("You can only remove password one by one");
            std::process::exit(0);
        }
        rm(&mut passfile_data, command.clone().trim().to_string(), &key);      
        std::process::exit(0);
    }

    match command.as_str().trim() {
        "help"                        => { help(); std::process::exit(0) }
        "list"               |  "lp"  => { lp(&passfile_data); std::process::exit(0) }
        "new password"       |  "np"  => { np(&mut passfile_data, &key, command.trim().to_owned()); std::process::exit(0) }
        "get configuration"  |  "gc"  => { gc(); std::process::exit(0); }
        "author"             |  "lpm" => { author_table(); std::process::exit(0) }
        "exit"         | "w" |  "q"   => { std::process::exit(0) }  
        "clear"                       => { clear(); std::process::exit(0); }
        ""                            => {}
        _                             => { print_err("Invalid Command, you can use help to list all commands"); std::process::exit(0) }
    }
}
