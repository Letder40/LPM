use lpm::utils::exit;
use lpm::config::read_config;
use lpm::home::home;
use std::io::{stdout, stdin, Write};
use std::fs::File;
use std::path::Path;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::fs::PermissionsExt;

fn main(){
    let path: String = String::from("passfile.lpm");
    let passfile_path: &Path = Path::new(&path);
    read_config();
    let mut input = String::new();
    
    if !passfile_path.exists() {
        //code
        loop {
            print!("\npassword file not exists, do you want to create a new one [Y/n] : ");
            stdout().flush().unwrap();

            stdin().read_line(&mut input).unwrap();
            input.make_ascii_lowercase(); 

            if input.trim() == "y" || input.trim() == "" {
                let _passfile = File::create(passfile_path).unwrap();
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                unix_permisions(&passfile_path, &_passfile);
                break;

            }else if input.trim() == "n" {
                eprintln!("password file hasn't been created");
                exit(1);
                break;

            }else{
                println!("invalid input -> it must be [ y ] or [ n ]");
            }

        }

        home();
    }

}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_permisions(path:&Path, passfile:&File){
    let metadata = passfile.metadata().unwrap();
    let mut permisions = metadata.permissions();
    permisions.set_mode(0o600);
    std::fs::set_permissions(path, permisions).unwrap();
}
