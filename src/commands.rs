use std::{io::{stdout, Write, stdin}};
use rand::Rng;
use tabled::{builder::{Builder}, settings::{Modify, object::Rows, Alignment, Style, Margin, Width}}; 
use crate::{crypto::encrypt, config::read_config, serde::{serialize_passwords, PasswordData}, utils::{print_input, print_err}};

#[cfg(target_os = "linux") ]
use copypasta_ext::{prelude::*, x11_fork::ClipboardContext};
#[cfg(any(target_os = "windows", target_os = "macos"))]
use cli_clipboard::{ClipboardContext, ClipboardProvider};

use aes_gcm::Aes256Gcm;

pub fn help(){
    let mut builder = Builder::default();
    let headers = ["Command", "functionality"];
    let allrows = [
        ["help", "[+] prints this help"],
        ["list | lp", "[+] prints all saved passwords"],
        ["new password | np", "[+] save a new password, type r or random in the password input to generate a randow password"],
        ["rm | del | rem", "[+] remove a password by the password id as argument"],
        ["copy | cp", "[+] copy a password to clipboard by Password Id on list or lp"],
        ["get configuration | gc   ", "[+] Prints the path of the config file and its content"],
        ["author | lpm", "[+] information about the author of the program also known as me"],
        ["exit | wq | q", "[+] closes lpm"],
        ["clear", "[+] Clear the screen buffer as clear or cls"],
    ];

    builder.set_header(headers);

    for row in allrows.into_iter() { builder.push_record(row); }

    let table = builder.build()
    .with(Style::rounded())
    .with(Modify::new(Rows::new(1..)).with(Width::wrap(50).keep_words()))
    .with(Margin::new(2, 0, 1, 1))
    .to_string();
    println!("{}", table);
}

pub fn np(passfile_data: &mut Vec<PasswordData>, key: &Aes256Gcm, input: String){
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
    save(passfile_data, key.clone());

}

// Function for list
pub fn lp(passfile_data:&Vec<PasswordData>){
    
    if passfile_data.len() == 0 {
        print_err("You don't have any saved password");
        return;
    }

    let mut builder = Builder::default();
    let columns = vec!["#".to_owned(), "Id".to_owned(), "Password".to_owned()];
    let mut n = 1;
    builder.set_header(columns);
    
    for password_data in passfile_data.iter(){
        let row = vec![n.to_string(), password_data.id.clone(), password_data.value.clone()];
        builder.push_record(row);
        n += 1
    }

    let table = builder.build()
    .with(Style::rounded())
    .with(Margin::new(2, 0, 1, 1))
    .to_string();

    println!("{}", table);

}

// Ccopy functionaity
pub fn copy(passfile_data: &Vec<PasswordData>, input: String){
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


pub fn rm(passfile_data: &mut Vec<PasswordData>, input: String){

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
pub fn gc(){
    let config_path = crate::config::config_path().as_os_str().to_owned().into_string().unwrap();
    let configuration = read_config();
    let mut builder = Builder::default();
    let headers = vec!["Configuration name", "Value"];
    builder.set_header(headers);

    let remote_server = if configuration.remote_server.lpm_remote_server { "True" } else { "False" };
    let rows = [
        ["LPM config file path", config_path.as_str()],
        ["passfile.lpm path", configuration.passfile_path.as_str() ],
        ["Prompt", configuration.lpm_prompt.as_str() ],
        ["Remote Server", remote_server],
        ["Remote Server type", configuration.remote_server.lpm_remote_server_type.as_str() ],
        ["Remote Server path", configuration.remote_server.lpm_remote_server_path.as_str() ],
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

pub fn author_table(){
    let mut builder = Builder::default();
    let headers = ["Author", "github"];
    let row = ["Letder", "https://github.com/Letder40"];
    builder.set_header(headers);
    builder.push_record(row);
    let table = builder.build()
    .with(Style::rounded())
    .with(Modify::new(Rows::new(1..)).with(Alignment::left()))
    .with(Margin::new(2, 0, 1, 1))
    .to_string();
    println!("{}", table);
}

pub fn clear(){
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    #[cfg(target_os = "windows")]
    {
    print!("{esc}[2H", esc = 27 as char);
    print!("{esc}[2J", esc = 27 as char);
    }
}

pub fn save(passfile_data: &Vec<PasswordData>, key: Aes256Gcm){
    let passfile_data_bytes = serialize_passwords(passfile_data);
    encrypt(key, passfile_data_bytes)
}

fn random_password() -> String {
    let char_candidates = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','0','1','2','3','4','5','6','7','8','9','!','@','#','$','%','^','&','*','(',')','-','_','+','=','~','`','[',']','{','}','|',':',';','"','\'','<','>',',','.','?','/',' '];
    let mut password = String::new();

    for _ in 0..30 {
        let mut rng = rand::thread_rng();
        let rand_index = rng.gen_range(0..93);
        password.push(char_candidates[rand_index])
    } 

    password
}