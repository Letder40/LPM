use lpm::utils::exit;
use lpm::config::read_config;
use lpm::home::home;
use std::io::{stdout, stdin, Write};
use std::fs::File;
use std::path::Path;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::fs::PermissionsExt;

fn main(){
    ctrlc::set_handler(move || {
        exit(1);
    }).expect("Error setting Ctrl-C handler");

    let config = read_config();
    let mut _path: String = String::new();
    
    if config["passfile_path"] == "default" {
        _path = lpm_default_path();
    }else{
        _path = config["passfile_path"].clone() + "/passfile.lpm";
    }

    let passfile_path: &Path = Path::new(&_path);
    let mut input = String::new();
    
    if !passfile_path.exists() {
        //code
        loop {
            print!("\npassword file not exists, do you want to create a new one [Y/n] : ");
            stdout().flush().unwrap();

            stdin().read_line(&mut input).unwrap();
            input.make_ascii_lowercase(); 

            if input.trim() == "y" || input.trim() == "" {
                let _passfile = File::create(passfile_path).expect("the creation of the file has failed, maybe the path provided is not valid or you have lack of permisions on that directory");
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                unix_permisions(&passfile_path, &_passfile);
                println!("\n [!] provaid a password you must remember it, there is no way of change it or recover it \n");
                stdout().flush().unwrap();
                break;

            }else if input.trim() == "n" {
                eprintln!("password file hasn't been created");
                exit(1);
                break;

            }else{
                println!("invalid input -> it must be [ y ] or [ n ]");
            }

        }
        
    }

    home();

}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_permisions(path:&Path, passfile:&File){
    let metadata = passfile.metadata().unwrap();
    let mut permisions = metadata.permissions();
    permisions.set_mode(0o600);
    std::fs::set_permissions(path, permisions).unwrap();
}

fn lpm_default_path() -> String {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let mut config_path_str = std::env::var("HOME").unwrap();
    
    #[cfg(target_os = "windows")]
    let mut config_path_str = std::env::var("USERPROFILE").unwrap();

    config_path_str.push_str("/.passfile.lpm");
    return config_path_str;
}


