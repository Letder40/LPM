use crate::{utils::{print_err, print_in_color, read_passfile}, commands::*, config::read_config};
use crossterm::{execute, terminal::SetTitle, style::Color};
use std::{io::{stdout, Write, stdin}};

pub fn home(){

    let read_passfile_tuple = read_passfile();
    let mut passfile_data = read_passfile_tuple.0;
    let key = read_passfile_tuple.1;

    clear();
    //Setting new title
    let title:SetTitle<String> = SetTitle(String::from("| LPM | Letder's password manager |"));
    execute!(
        stdout(), 
        title,
    ).unwrap();
    

    let ascii_art = [
        "     ___       ","________  _____ ______      \n",
        "    |\\  \\     ","|\\   __  \\|\\   _ \\  _   \\    \n", 
        "    \\ \\  \\    ","\\ \\  \\|\\  \\ \\  \\\\\\__\\ \\  \\   \n", 
        "     \\ \\  \\    ","\\ \\   ____\\ \\  \\\\|__| \\  \\  \n", 
        "      \\ \\  \\____","\\ \\  \\___|\\ \\  \\    \\ \\  \\ \n", 
        "       \\|_______|","\\|__|     \\|__|     \\|__|\n", 
        "\n", 
        "\n"
    ];
    
    println!();
    let mut index = 0;
    for line in ascii_art.iter() {
        if index % 2 == 0 {
            print!("{line}");
            index+=1;
            continue;
        }
        print_in_color(Color::Yellow,line);
        index+=1
    }


    // APP LOOP
        let config = read_config();

    loop {

        let prompt = format!("{} ", config.lpm_prompt);

        print_in_color(Color::Green, &prompt);

        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
                
        if input.trim().starts_with("cp") || input.trim().starts_with("copy") {
            copy(&passfile_data, input.clone());
            continue
        }

       if input.trim().starts_with("np") || input.trim().starts_with("new password")  {
            
            np(&mut passfile_data, &key, input.trim().to_owned());
            continue
       }

       if input.trim().starts_with("rm") || input.trim().starts_with("rem") || input.trim().starts_with("del") {
            if input.split(' ').collect::<Vec<&str>>().len() > 2 {
                print_err("You can only remove password one by one")
            }
            rm(&mut passfile_data, input.clone(), &key);      
            continue; 

        }

        match input.as_str().trim() {
            "help"                        => { help() }
            "list"               |  "lp"  => { lp(&passfile_data) }
            "new password"       |  "np"  => { np(&mut passfile_data, &key, input.trim().to_owned()) }
            "get configuration"  |  "gc"  => { gc() }
            "author"             |  "lpm" => { author_table() }
            "exit"         | "w" |  "q"   => { std::process::exit(0); }  
            "clear"                       => { clear() }
            ""                            => {}
            _                             => { print_err("Invalid Command, you can use help to list all commands");}
        }
        
    };
}
