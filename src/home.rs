use crossterm::{execute, terminal::{EnterAlternateScreen, SetTitle}};
use std::io::{stdout, Write, stdin};
//use crate::utils::exit;


pub fn home(){

    //change to alternative buffer screen
    execute!(stdout(), EnterAlternateScreen).unwrap();
    
    //Setting new title
    let title:SetTitle<String> = SetTitle(String::from("| LPM | Letder's password manager |"));
    execute!(stdout(), title).unwrap();

    //let credentials = crate::utils::read_pass();
    
    let mut input = String::new();
    

    read_pass();

    loop {
        print!("LPM > ");
        stdout().flush().unwrap();

        stdin().read_line(&mut input).unwrap();

        let input_splited: Vec<&str> = input.split(" ").collect();
        let input_len: usize = input_splited.len();
        if input_len > 2 {
            println!(" [!] Invalid Command -> [ help ] to list all commands");
            stdout().flush().unwrap();
        }
         
    }
}

//Read password
fn read_pass() -> [String; 2] {

    print!("Master key : ");
    stdout().flush().unwrap();

    let password:String = rpassword::read_password().unwrap();

    print!("IV : ");
    stdout().flush().unwrap();
    let iv:String = rpassword::read_password().unwrap();

    let credentials:[String; 2] = [
        password,
        iv,
    ];

    return credentials;
}