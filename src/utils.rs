use std::{io::{stdout, stderr, stdin, Write}, fs::File, path::Path};

use crate::{config::read_config, crypto::{encrypt, get_key, decrypt}, serde::{PasswordData, deserialize_passwords}};

use aes_gcm::Aes256Gcm;
use crossterm::{style::{SetForegroundColor, Color, ResetColor, Print}, execute};
use zeroize::Zeroize;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::fs::PermissionsExt;


//stop the program with a specific error code
pub fn exit(code :i32, error: &str){
    print_err(error);
    std::process::exit(code);
}

pub fn print_err(text: &str){
    eprint_in_color(Color::Red," [!] ");
    print!("{text}\n");
}

pub fn print_input(text: &str){
    eprint_in_color(Color::Blue," [?] ");
    print!("{text}");
}

pub fn print_info(text: &str){
    eprint_in_color(Color::Yellow," [#] ");
    print!("{text}\n");
}

pub fn print_in_color(color: Color,text: &str){
    execute!(
        stdout(),
        SetForegroundColor(color),
        Print(text),
        ResetColor
    ).unwrap();
    stdout().flush().unwrap();
}

fn eprint_in_color(color: Color,text: &str){
    execute!(
        stderr(),
        SetForegroundColor(color),
        Print(text),
        ResetColor
    ).unwrap();
    stdout().flush().unwrap();
}

pub fn check_filepass(){

    let config = read_config();
    let passfile_path: &Path = Path::new(&config.passfile_path);

    // Creating String type var to use it later as input buffer
    let mut input = String::new();
    
    
    if !passfile_path.exists() {
        loop {
            print_input("password file not exists, do you want to create a new one [Y/n] : ");
            stdout().flush().unwrap();

            stdin().read_line(&mut input).unwrap();
            input.make_ascii_lowercase(); 


            if input.trim() == "y" || input.trim() == "" {
                
                let mut _passfile = File::create(passfile_path).expect(" [!] The creation of the file has failed, maybe the path provided is not valid or you have lack of permisions on that directory");
                print_info(format!("password file created in {:?}", passfile_path).as_str());

                #[cfg(any(target_os = "linux", target_os = "macos"))]
                unix_permisions(&passfile_path, &_passfile);
                
                print_info("provides a password you must remember it, there is no way of change it or recover it");
                
                let mut password = read_pass();
                let key = get_key(&password);
                password.zeroize();
                encrypt(key, "".as_bytes().to_vec());

                stdout().flush().unwrap();

                break;

            }else if input.trim() == "n" {
                exit(1, "password file hasn't been created");
                break;

            }else{
                print_err("invalid input, it must be [ y ] or [ n ]");
            }

        }
        
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_permisions(path:&Path, passfile:&File){
    let metadata = passfile.metadata().unwrap();
    let mut permisions = metadata.permissions();
    permisions.set_mode(0o600);
    std::fs::set_permissions(path, permisions).unwrap();
}


pub fn read_passfile() -> (Vec<PasswordData>, Aes256Gcm) {
    let mut password = read_pass();
    stdout().flush().unwrap();
    let key = get_key(&password);
    password.zeroize();
    let passfile_data_bytes = decrypt(key.clone());
    let passfile_data = deserialize_passwords(&passfile_data_bytes);
    (passfile_data, key)
}

//Read password
pub fn read_pass() -> String {
    print_input("Master key : ");
    stdout().flush().unwrap();
    let password:String = rpassword::read_password().unwrap();

    return password;
}