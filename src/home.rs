use crossterm::{execute, terminal::{SetTitle}, style::{SetForegroundColor, Color, ResetColor, Print}};
use tabled::{builder::{Builder}, settings::{Modify, object::Rows, Alignment, Style, Margin, Width}};

use std::{io::{stdout, Write, stdin}};

use aes_gcm::aead::{generic_array::GenericArray};
use typenum::U32;
use crate::{crypto::{decrypt, encrypt, get_key}, serde::{PasswordData, deserialize_passwords, serialize_passwords}, utils::exit};
use zeroize::Zeroize;

//TODO IMPROVE THE NPASSWORD USER INPUT; 
//CAPABILITY OF RANDOMS PASSWORDS IN USER INPUT; 
//copy|cp function to copy a password to clipboard by PasswordId or NumericId    

pub fn home(){
    let mut password = read_pass();
    stdout().flush().unwrap();
    
    //Setting new title
    let title:SetTitle<String> = SetTitle(String::from("| LPM | Letder's password manager |"));
    clear();
    execute!(
        stdout(), 
        title,
    ).unwrap();

    
    let config = crate::config::read_config();
    let key = get_key(&password);
    password.zeroize();

    let passfile_data_bytes: Vec<u8> = decrypt(key);

    let mut passfile_data: Vec<PasswordData> = Vec::new();

    if passfile_data_bytes.len() != 0 {
        passfile_data = deserialize_passwords(&passfile_data_bytes)
    }

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
            "help" =>                        { help() }
            "list"               |  "lp"  => { lp(&passfile_data) }
            "new password"       |  "np"  => { np(&mut passfile_data, key) }
            "get configuration"  |  "gc"  => { println!("getting configuration") }
            "author"             |  "lpm" => { author_table() }
            "exit"               |  "q"   => { exit(0, "")}  
            "clear"                       => { clear() }
            ""                            => {}
            _                             => { println!(" [!] Invalid Command -> [ help ] to list all commands")}
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

fn help(){
    let mut builder = Builder::default();
    let headers = vec!["Command", "functionality"];
    let row0 = vec!["help", "prints this help"];
    let row1 = vec!["list | lp", "prints all saved passwords"];
    let row2 = vec!["new password | np", "save a new password"];
    let row3 = vec!["get configuration | gc", "Prints the path of the config file and its content"];
    let row4 = vec!["author | lpm", "information about the author of the program also known as me"];
    let row5 = vec!["clear", "Clear the screen buffer as clear or cls"];
    builder.set_header(headers);
    builder.push_record(row0);
    builder.push_record(row1);
    builder.push_record(row2);
    builder.push_record(row3);
    builder.push_record(row4);
    builder.push_record(row5);

    let table = builder.build()
    .with(Style::rounded())
    .with(Modify::new(Rows::new(1..)).with(Alignment::left()))
    .with(Margin::new(2, 0, 1, 1))
    .to_string();
    println!("{}", table);
}

// Function for new password
fn np(passfile_data: &mut Vec<PasswordData>, key: GenericArray<u8, U32>){
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
    save(passfile_data, key)
}


// Function for list
fn lp(passfile_data:&Vec<PasswordData>){
    
    if passfile_data.len() == 0 {
        eprintln!(" [?] You don't have any saved password");
        return;
    }

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
    .with(Margin::new(2, 0, 1, 1))
    .with(Modify::new(Rows::new(1..)).with(Width::wrap(30).keep_words()))
    .to_string();

    println!("{}", table);

}

fn author_table(){
    let mut builder = Builder::default();
    let headers = vec!["Author", "github"];
    let row = vec!["Letder", "https://github.com/Letder40"];
    builder.set_header(headers);
    builder.push_record(row);
    let table = builder.build()
    .with(Style::rounded())
    .with(Modify::new(Rows::new(1..)).with(Alignment::left()))
    .with(Margin::new(2, 0, 1, 1))
    .to_string();
    println!("{}", table);
}

fn clear(){
    #[cfg(target_os = "linux")]
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    #[cfg(target_os = "windows")]
    {
    print!("{esc}[2H", esc = 27 as char);
    print!("{esc}[2J", esc = 27 as char);
    }
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
    stdout().flush().unwrap();
}