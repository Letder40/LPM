use crate::{crypto::{decrypt, encrypt, get_key}, serde::{PasswordData, deserialize_passwords, serialize_passwords}, utils::{print_err, print_input, print_in_color}, config::read_config};
use crossterm::{execute, terminal::{SetTitle}, style::Color};
use rand::Rng;
use tabled::{builder::{Builder}, settings::{Modify, object::Rows, Alignment, Style, Margin, Width, Padding}}; 

#[cfg(target_os = "linux") ]
use copypasta_ext::{prelude::*, x11_fork::ClipboardContext};
#[cfg(any(target_os = "windows", target_os = "macos"))]
use cli_clipboard::{ClipboardContext, ClipboardProvider};

use std::{io::{stdout, Write, stdin}};

use aes_gcm::aead::{generic_array::GenericArray};
use typenum::U32;
use zeroize::Zeroize;

//CAPABILITY OF RANDOMS PASSWORDS IN USER INPUT; 
//copy|cp function to copy a password to clipboard by PasswordId or NumericId    

pub fn home(){
    let mut password = read_pass();
    stdout().flush().unwrap();
    
    //Setting new title
    let title:SetTitle<String> = SetTitle(String::from("| LPM | Letder's password manager |"));
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

    clear();

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
            
            np(&mut passfile_data, key, input.trim().to_owned());
            continue
       }

       if input.trim().starts_with("rm") || input.trim().starts_with("rem") || input.trim().starts_with("del") {
            if input.split(' ').collect::<Vec<&str>>().len() < 2 {
                print_err("You can only remove password one by one")
            }
            rm(&mut passfile_data, input.clone());      
            continue; 

        }

        match input.as_str().trim() {
            "help"                        => { help() }
            "list"               |  "lp"  => { lp(&passfile_data) }
            "new password"       |  "np"  => { np(&mut passfile_data, key, input.trim().to_owned()) }
            "get configuration"  |  "gc"  => { gc() }
            "author"             |  "lpm" => { author_table() }
            "exit"         | "w" |  "q"   => { std::process::exit(0); }  
            "clear"                       => { clear() }
            ""                            => {}
            _                             => { print_err("Invalid Command, you can use help to list all commands");}
        }
        
    }
}

//Read password
pub fn read_pass() -> String {
    print_input("Master key : ");
    stdout().flush().unwrap();
    let password:String = rpassword::read_password().unwrap();

    return password;
}


// ------------- 
// CLI FUNCTIONS
// -------------


fn help(){
    let mut builder = Builder::default();
    let headers = vec!["Command", "functionality"];
    let allrows = vec![
        vec!["help", "prints this help"],
        vec!["list | lp", "prints all saved passwords"],
        vec!["new password | np", "save a new password, type r or random in the password input to generate a randow password"],
        vec!["rm | del | rem", "remove a password by the password id as argument"],
        vec!["copy | cp", "copy a password to clipboard by [ Password Id ] or [ Numeric Id displayed ] on list or lp"],
        vec!["get configuration | gc   ", "Prints the path of the config file and its content"],
        vec!["author | lpm", "information about the author of the program also known as me"],
        vec!["exit | wq | q", "closes lpm"],
        vec!["clear", "Clear the screen buffer as clear or cls"],
    ];

    builder.set_header(headers);

    for row in allrows.into_iter() { builder.push_record(row); }

    let table = builder.build()
    .with(Style::rounded())
    .with(Modify::new(Rows::new(1..)).with(Alignment::left()))
    .with(Margin::new(2, 0, 1, 1))
    .with(Modify::new(Rows::new(1..)).with(Width::wrap(50).keep_words()))
    .with(Modify::new(Rows::new(1..)).with(Padding::new(0, 0, 0, 0)))
    .to_string();
    println!("{}", table);
}

// Function for new password
fn np(passfile_data: &mut Vec<PasswordData>, key: GenericArray<u8, U32>, input: String){
    let mut _new_password:PasswordData = PasswordData { id: "".to_string(), value: "".to_string() };

    if input == "np" || input == "new password" {
        let mut input_buffer = String::new();
        print_input("Password id: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input_buffer).unwrap();
        let id = input_buffer.trim();

        _new_password = PasswordData{
            id: id.to_owned(),
            value: "".to_owned()
        };

        //Checking if ID is unique
        let mut password_ids: Vec<String> = Vec::<String>::new(); 
        for password_data in passfile_data.iter() {
            let id = password_data.id.clone();
            password_ids.push(id);
        }

        if password_ids.contains(&_new_password.id) {
            print_err("Password Identifier must be unique\n");
            return
        } 

        let mut input_buffer = String::new();
        print_input("Password value: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input_buffer).unwrap();
        let value:String = if input_buffer.trim() == "r" || input_buffer.trim() == "random" { random_password() }else{ input_buffer.trim().to_string() };

        _new_password = PasswordData{
            id: _new_password.id,
            value: value,
        };
        
        println!()
    
    }else{
        let command:String = if input.starts_with("np"){ "np".to_string() }else{ "new password".to_string() } ;

        if input.trim().split(" ").count() != 3 && input.trim().starts_with("np") || input.trim().split(" ").count() != 4 &&  input.trim().starts_with("new password") {
            print_err(format!("usage: {command} [password ID] [password value or random | r], if no arguments both are requested in user inputs").as_str());
            return
        }

        if command == "np" {
            _new_password = PasswordData{
                id: input.split(" ").collect::<Vec<&str>>()[1].to_string(),
                value: input.split(" ").collect::<Vec<&str>>()[2].to_string(),
            };
        }else {
            _new_password = PasswordData{
                id: input.split(" ").collect::<Vec<&str>>()[2].to_string(),
                value: input.split(" ").collect::<Vec<&str>>()[3].to_string(),
            };
        }

        if _new_password.value == "r" || _new_password.value == "random" {
            _new_password.value = random_password();
        }

         //Checking if ID is unique
         let mut password_ids: Vec<String> = Vec::<String>::new(); 
         for password_data in passfile_data.iter() {
             let id = password_data.id.clone();
             password_ids.push(id);
         }
 
         if password_ids.contains(&_new_password.id) {
             print_err("Password Identifier must be unique\n");
             return
         } 

    }

    passfile_data.push(_new_password);
    save(passfile_data, key);
}

// Function for list
fn lp(passfile_data:&Vec<PasswordData>){
    
    if passfile_data.len() == 0 {
        print_err("You don't have any saved password");
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

// Ccopy functionaity
fn copy(passfile_data: &Vec<PasswordData>, input: String){
    let input_splitted:Vec<&str> = input.split(" ").collect();
    
    if input_splitted.len() != 2 {
        print_err("You have to provide a valid identifier");
        return;
    }
    
    let identifier = input_splitted[1].trim();
    let mut copied = false; 

    for password_struct in passfile_data.iter() {
        if identifier == password_struct.id {   
            #[cfg(target_os = "linux")]
            {
            let mut clipboard = ClipboardContext::new().unwrap();
            clipboard.set_contents(password_struct.value.clone()).unwrap();
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
            let mut clipboard = ClipboardContext::new().unwrap();
            clipboard.set_contents(password_struct.value.clone()).unwrap();
            }
            copied = true
        }
    }

    if copied != true {
        print_err("There is not such identifier")
    }
}


fn rm(passfile_data: &mut Vec<PasswordData>, input: String){

    let password_requested_id = input.trim().split(' ').collect::<Vec<&str>>()[1];

    let mut removed = false; 
    let  mut counter = 0;
    for password in  passfile_data.clone() {
        if password.id == password_requested_id {
            passfile_data.remove(counter);
            removed = true
        } 
        counter += 1
    }

    if removed != true {
        print_err("Not a valid password identifier")
    }
    
}

// Function for get configuration
fn gc(){
    let config_path = crate::config::config_path().as_os_str().to_owned().into_string().unwrap();
    let configuration = read_config();
    let mut builder = Builder::default();
    let headers = vec!["Configuration name", "Value"];
    builder.set_header(headers);

    let remote_server = if configuration.remote_server.lpm_remote_server { "True" } else { "False" };
    let rows = vec![
        vec!["LPM config file path", config_path.as_str()],
        vec!["passfile.lpm path", configuration.passfile_path.as_str() ],
        vec!["Prompt", configuration.lpm_prompt.as_str() ],
        vec!["Remote Server", remote_server],
        vec!["Remote Server type", configuration.remote_server.lpm_remote_server_type.as_str() ],
        vec!["Remote Server path", configuration.remote_server.lpm_remote_server_path.as_str() ],
    ];

    for row in rows.into_iter() { builder.push_record(row); }

    let table = builder.build()
    .with(Style::rounded())
    .with(Modify::new(Rows::new(1..)).with(Alignment::left()))
    .with(Margin::new(2, 0, 1, 1))
    .with(Modify::new(Rows::new(1..)).with(Width::wrap(40).keep_words()))
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
    #[cfg(any(target_os = "linux", target_os = "macos"))]
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

fn random_password() -> String {
    let char_candidates = vec!['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','0','1','2','3','4','5','6','7','8','9','!','@','#','$','%','^','&','*','(',')','-','_','+','=','~','`','[',']','{','}','|',':',';','"','\'','<','>',',','.','?','/',' '];
    let mut password = String::new();

    for _ in 0..30 {
        let mut rng = rand::thread_rng();
        let rand_index = rng.gen_range(0..93);
        password.push(char_candidates[rand_index])
    } 

    password
}