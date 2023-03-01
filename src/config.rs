use std::path::{PathBuf};
use std::fs::{File, create_dir_all};
use crate::utils::exit;

pub fn read_config() {
    let path:PathBuf = config_path();
    
    if !path.exists() {
        
        let folders= path.iter().collect::< Vec<_> >();

        #[cfg(target_os = "linux")]
        let path_tocheck: PathBuf = folders[0..5].iter().collect();
        #[cfg(target_os = "windows")]
        let path_tocheck: PathBuf = folders[0..7].iter().collect();
        
        if !path_tocheck.exists() {
            match create_dir_all(&path_tocheck) {
                Ok(_) => { 
                    println!(" [?] {} has been created",path_tocheck.display() );
                },
                Err(_) => { 
                    eprintln!(" [!] The config folder doesn't exists and it can't be create in {}, possibely a permision error \n exiting ...",path_tocheck.display() );
                    exit(1);
                },
            }            
        }

        let path_tocheck: PathBuf = folders[0..].iter().collect();

        if !path_tocheck.exists() {
            match File::create(&path_tocheck) {
                Ok(_) => { 
                    println!(" [?] {} has been created",path_tocheck.display() );
                },
                Err(_) => { 
                    eprintln!(" [!] The config file can't be create in {}, possibely a permision error \n exiting ...",path_tocheck.display() );
                    exit(1);
                },
            }


        }   
    }
}


#[cfg(target_os = "linux")]
fn config_path() -> PathBuf {
    let mut home = std::env::var("HOME").unwrap();
    home.push_str("/.config/lpm/lpm.conf");
    let config_path = PathBuf::from(home);
    return config_path;
}

#[cfg(target_os = "windows")]
fn config_path() -> PathBuf {
    let mut home = std::env::var("USERPROFILE").unwrap();
    home.push_str("/AppData/Roaming/lpm/lpm.conf");
    let config_path = PathBuf::from(home);
    return config_path;
}

