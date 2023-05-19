use crossterm::{execute, terminal::{EnterAlternateScreen, SetTitle}, style::{SetForegroundColor, Color, ResetColor, Print}};
use tabled::{builder::{Builder}, settings::{Modify, object::Rows, Alignment, Style}};

use std::{io::{stdout, Write, stdin}};

use aes_gcm::aead::{generic_array::GenericArray};
use typenum::U32;
use crate::{crypto::{decrypt, encrypt, get_key}, serde::{PasswordData, deserialize_passwords, serialize_passwords}, utils::exit};
use zeroize::Zeroize;
// TODO DATA SERIALIZATION / DESERALIZATION -> ONLY 1 TIME (OPEN/CLOSE FILE) -> KEEP PASSWORDS PROVIDED BY USERS AS Vec<PasswordData>    

pub fn home(){
    let mut password = read_pass();
    stdout().flush().unwrap();

    //change to alternative buffer screen
    execute!(stdout(), EnterAlternateScreen).unwrap();
    
    //Setting new title
    let title:SetTitle<String> = SetTitle(String::from("| LPM | Letder's password manager |"));
    execute!(stdout(), title).unwrap();
    
    let config = crate::config::read_config();
    let key = get_key(&password);
    password.zeroize();

    let passfile_data_bytes: Vec<u8> = decrypt(key);

    let mut passfile_data: Vec<PasswordData> = Vec::new();

    if passfile_data_bytes.len() != 0 {
        passfile_data = deserialize_passwords(&passfile_data_bytes)
    }

    let ascii_art = ["", "     ___       ________  _____ ______      ","    |\\  \\     |\\   __  \\|\\   _ \\  _   \\    ", "    \\ \\  \\    \\ \\  \\|\\  \\ \\  \\\\\\__\\ \\  \\   ", "     \\ \\  \\    \\ \\   ____\\ \\  \\\\|__| \\  \\  ", "      \\ \\  \\____\\ \\  \\___|\\ \\  \\    \\ \\  \\ ", "        \\|_______|\\|__|     \\|__|     \\|__|", "", ""];
    for line in ascii_art.iter() {
        println!("{}", line)
    }

    // APP LOOP
    loop {
        
        if config["lpm_prompt"] == "default" {
            print_in_color(Color::Green, "LPM > ")
        }else{
            let prompt = format!("{} > ", config["lpm_prompt"]);
            print_in_color(Color::Green, &prompt)
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
            "list"                            | "lp"  => { lp(&passfile_data) }
            "new password"                    | "np"  => { np(&mut passfile_data) }
            "get configuration"               | "gc"  => { println!("getting configuration") }
            "author"                          | "lpm" => { println!("\n\t+-------------------------------+\n\t|  https://github.com/Letder40  |\n\t+-------------------------------+\n")}
            "save"       |    "write"         | "w"   => { save(&passfile_data, key) }
            "save exit"  |    "write exit"    | "wq"  => { save(&passfile_data, key); exit(0, "") } 
            "exit"       |                      "q"   => { exit(0, "")}  
            ""                                        => {}
            _                                         => { println!(" [!] Invalid Command -> [ help ] to list all commands")}
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

// Function for new password
fn np(passfile_data: &mut Vec<PasswordData>){
    let mut input_buffer = String::new();
    print!("Password id: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut input_buffer).unwrap();
    let id = input_buffer.trim();

    let mut new_password = PasswordData{
        id: id.to_owned(),
        value: "".to_owned()
    };

    //Checking if ID is uniq
    let mut password_ids: Vec<String> = Vec::<String>::new(); 
    for password_data in passfile_data.iter() {
        let id = password_data.id.clone();
        password_ids.push(id);
    }

    if password_ids.contains(&new_password.id) {
        print_in_color(Color::Red, " [!] Password Identifier must be unique\n");
        return
    } 
    
    let mut input_buffer = String::new();
    print!("Password value: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut input_buffer).unwrap();
    let value = input_buffer.trim();

    new_password = PasswordData{
        id: new_password.id,
        value: value.to_owned(),
    };

    passfile_data.push(new_password);
}
// Function for list passwords
fn lp(passfile_data:&Vec<PasswordData>){
    
    if passfile_data.len() == 0 {
        eprintln!(" [?] You don't have any saved password");
        return;
    }

    println!("");
    let mut builder = Builder::default();
    let columns = vec!["#".to_owned(), "Id".to_owned(), "Password".to_owned()];
    let mut n = 1;
    builder.set_header(columns);
    for password_data in passfile_data.iter(){
        let row: Vec<String> = vec![n.to_string(), password_data.id.clone(), password_data.value.clone()];
        builder.push_record(row);
        n += 1
    }

    let table = builder.build()
    .with(Style::rounded())
    .with(Modify::new(Rows::new(1..)).with(Alignment::left()))
    .to_string();

    println!("{}", table);
    println!("");

}

pub fn save(passfile_data: &Vec<PasswordData>, key: GenericArray<u8, U32>){
    let passfile_data_bytes = serialize_passwords(passfile_data);
    encrypt(key, passfile_data_bytes)
}

fn print_in_color(color: Color,text: &str){
    execute!(
        stdout(), 
        SetForegroundColor(color),
        Print(text),
        ResetColor
    ).unwrap();
}