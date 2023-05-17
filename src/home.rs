use aes_gcm::aead::generic_array::GenericArray;
use crossterm::{execute, terminal::{EnterAlternateScreen, SetTitle}};
use std::io::{stdout, Write, stdin};

use crate::crypto::encrypt;
//use crate::utils::exit;

pub fn home(){
    
    let password = read_pass();
    let key: GenericArray<u8, _> = crate::crypto::get_key(password);

    //change to alternative buffer screen
    execute!(stdout(), EnterAlternateScreen).unwrap();
    
    //Setting new title
    let title:SetTitle<String> = SetTitle(String::from("| LPM | Letder's password manager |"));
    execute!(stdout(), title).unwrap();

    encrypt(String::from("prueba !end"), key.to_vec());
   
    let mut input = String::new();

    let config = crate::config::read_config();

    loop {
        if config["lpm_prompt"] == "default" {
            print!("LPM > ");
        }else{
            print!("{} > ", config["lpm_prompt"]);
        }
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
pub fn read_pass() -> String {

    print!("Master key : ");
    stdout().flush().unwrap();

    let password:String = rpassword::read_password().unwrap();

    return password;
}