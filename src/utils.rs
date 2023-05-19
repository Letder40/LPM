use std::io::stdout;
use std::io::{stdin, Write};
use std::fs::File;
use std::path::Path;
use crate::{config::read_config, crypto::{encrypt, get_key}};
use zeroize::Zeroize;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::fs::PermissionsExt;


//stop the program with a specific error code
pub fn exit(code :i32, error: &str){
    eprint!("{}", error);
    std::process::exit(code);
}

pub fn check_filepass(){

    let config = read_config();
    let passfile_path: &Path = Path::new(&config["passfile_path"]);

    // Creating String type var to use it later as input buffer
    let mut input = String::new();
    
    
    if !passfile_path.exists() {
        loop {
            print!("\npassword file not exists, do you want to create a new one [Y/n] : ");
            stdout().flush().unwrap();

            stdin().read_line(&mut input).unwrap();
            input.make_ascii_lowercase(); 

            if input.trim() == "y" || input.trim() == "" {
                
                let mut _passfile = File::create(passfile_path).expect(" [!] The creation of the file has failed, maybe the path provided is not valid or you have lack of permisions on that directory");

                #[cfg(any(target_os = "linux", target_os = "macos"))]
                unix_permisions(&passfile_path, &_passfile);
                
                println!("\n [!] provaid a password you must remember it, there is no way of change it or recover it \n");
                
                let mut password = crate::home::read_pass();
                let key = get_key(&password);
                password.zeroize();
                encrypt(key, "".as_bytes().to_vec());

                stdout().flush().unwrap();

                break;

            }else if input.trim() == "n" {
                eprintln!("password file hasn't been created");
                exit(1, "");
                break;

            }else{
                println!("invalid input -> it must be [ y ] or [ n ]");
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