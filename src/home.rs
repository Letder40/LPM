use crossterm::{execute, terminal::{EnterAlternateScreen, SetTitle}};
use std::io::{stdout, Write, stdin};


use crate::crypto::{decrypt};
//use crate::utils::exit;

pub fn home(){
    let password = read_pass();
    stdout().flush().unwrap();

    //change to alternative buffer screen
    execute!(stdout(), EnterAlternateScreen).unwrap();
    
    //Setting new title
    let title:SetTitle<String> = SetTitle(String::from("| LPM | Letder's password manager |"));
    execute!(stdout(), title).unwrap();

    decrypt(password);
    
    let config = crate::config::read_config();
    
    let ascii_art = ["", "     ___       ________  _____ ______      ","    |\\  \\     |\\   __  \\|\\   _ \\  _   \\    ", "    \\ \\  \\    \\ \\  \\|\\  \\ \\  \\\\\\__\\ \\  \\   ", "     \\ \\  \\    \\ \\   ____\\ \\  \\\\|__| \\  \\  ", "      \\ \\  \\____\\ \\  \\___|\\ \\  \\    \\ \\  \\ ", "        \\|_______|\\|__|     \\|__|     \\|__|", "", ""];
    for line in ascii_art.iter() {
        println!("{}", line)
    }
    loop {
        

        if config["lpm_prompt"] == "default" {
            print!("LPM > ");
        }else{
            print!("{} > ", config["lpm_prompt"]);
        }
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let input_splited: Vec<&str> = input.split(" ").collect();
        let input_len: usize = input_splited.len();
        
        if input_len > 2 {
            println!(" [!] Invalid Command -> [ help ] to list all commands");
            stdout().flush().unwrap();
        }

        match input.as_str().trim() {
            "help" =>  { println!("help") }
            "list passwords"     | "lp"  => { println!("listing passwords") }
            "new password"       | "np"  => { println!("writting password") }
            "get configuration"  | "gc"  => { println!("getting configuration") }
            "author"             | "lpm" => { println!("\n\t+-------------------------------+\n\t|  https://github.com/Letder40  |\n\t+-------------------------------+\n")}
            ""                           => {}
            _                            => { println!(" [!] Invalid Command -> [ help ] to list all commands")}
        }
        
        
    }
}

//Read password
pub fn read_pass() -> String {
    print!("Master key : ");
    stdout().flush().unwrap();
    let password:String = rpassword::read_password().unwrap();

    return password;
}

// Home functions
fn np(){
    let mut input_buffer = String::new();
    print!("Password id: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut input_buffer).unwrap();
    let id = input_buffer.trim();

    let mut input_buffer = String::new();
    print!("Password value: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut input_buffer).unwrap();
    let value = input_buffer.trim();
}
fn lp(){

}