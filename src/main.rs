use lpm::utils::exit;
use lpm::config::read_config;
use lpm::home::home;
use std::io::{stdout, stdin, Write};
use std::fs::File;
use std::path::Path;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::fs::PermissionsExt;

//     ___       ________  _____ ______      
//    |\  \     |\   __  \|\   _ \  _   \    
//    \ \  \    \ \  \|\  \ \  \\\__\ \  \   
//     \ \  \    \ \   ____\ \  \\|__| \  \  
//      \ \  \____\ \  \___|\ \  \    \ \  \ 
//       \ \_______\ \__\    \ \__\    \ \__\
//        \|_______|\|__|     \|__|     \|__|


fn main(){
    // Handler for user keyboard interruptions ctrlc
    ctrlc::set_handler(move || {
        exit(1);
    }).expect("Error setting Ctrl-C handler");

    let config = read_config();
    let passfile_path: &Path = Path::new(&config["passfile_path"]);

    // Creating String type var to use it later as input buffer
    let mut input = String::new();
    

    if !passfile_path.exists() {
        //code rustico.
        loop {
            print!("\npassword file not exists, do you want to create a new one [Y/n] : ");
            stdout().flush().unwrap();

            stdin().read_line(&mut input).unwrap();
            input.make_ascii_lowercase(); 

            if input.trim() == "y" || input.trim() == "" {
                // passfile creation
                let mut _passfile = File::create(passfile_path).expect(" [!] The creation of the file has failed, maybe the path provided is not valid or you have lack of permisions on that directory");
                // If unix gives secure privileges
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                unix_permisions(&passfile_path, &_passfile);
                // write {CHECK LINE} (This first line will be used to check the validity of the password)
                _passfile.write_all(String::from("Access!\n").as_bytes()).expect(" [!] Error in the initialization of the passfile");
                // Getting password
                println!("\n [!] provaid a password you must remember it, there is no way of change it or recover it \n");
                let password = lpm::home::read_pass();
                lpm::crypto::encrypt(password);

                stdout().flush().unwrap();
                // Encrypting passfile

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

    // LPM INIT 
    home();


}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_permisions(path:&Path, passfile:&File){
    let metadata = passfile.metadata().unwrap();
    let mut permisions = metadata.permissions();
    permisions.set_mode(0o600);
    std::fs::set_permissions(path, permisions).unwrap();
}


