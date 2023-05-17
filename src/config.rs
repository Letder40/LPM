use std::collections::{HashMap};
use std::io::{Write, Read, stdin, stdout};
use std::path::{PathBuf};
use std::fs::{File, create_dir_all};
use crate::utils::exit;

//Reading the file and transform the content in a legible format


pub fn read_config() -> HashMap<String, String> {

    let mut path:PathBuf = config_path(); // Getting path from private function config_path(), it get the config path widnows/linux/macos
    check_config(&mut path); // Checking if config file exists if not create a new one and all the parents folders

    let mut config_file = File::open(&path).unwrap();
    let mut read_buffer:Vec<u8> = vec![];
    config_file.read_to_end(&mut read_buffer).expect(" [!] wasn't been posible to read the file ");
    let data = String::from_utf8(read_buffer).unwrap();
    let mut data_vec:Vec<&str> = data.trim().split(['\n', ':']).collect();

    //checking that the config file is a valid one, if not create a new one
    if  data_vec.len() < 6 || data_vec[0] != "passfile_path" || data_vec[2] != "lpm_prompt" || data_vec[4] != "lpm_remote_server" {
        
        let mut input = String::new();
        loop {
            print!("Seems that your config file is corrupted, do you like to create a new one? [N/y] : ");
            stdout().flush().unwrap();
            stdin().read_line(&mut input).unwrap();
            if input.trim().to_lowercase() == "" ||  input.trim().to_lowercase() == "n" {
                eprintln!("Your config file is corrupted try to fix it or create a new one");
                exit(1);
                break;
            }else if input.trim().to_lowercase() == "y" {
                let mut write_config_file = File::create(&path).unwrap();
                recreate_file(&mut write_config_file);
                break;
            }
        }

        data_vec = vec!["passfile_path", "default", "lpm_prompt", "default", "lpm_remote_server", "none"];

    }

    let mut data_hashmap:HashMap<&str, &str> = HashMap::new();
    
    //splitting the vector in chunks of size 2 and adding it to the hashmap
    for chunk in data_vec.chunks(2) {
        let key = chunk[0];
        let value = chunk[1];
        data_hashmap.insert(key, value);
    }

    //Getting a hashmap of type String because &str is borrowed
    let mut conf_hashmap:HashMap<String, String> = HashMap::new();
    for (key, value) in data_hashmap {
        let new_key = String::from(key.trim());
        let new_value = String::from(value.trim());
        conf_hashmap.insert(new_key, new_value);
    }

    return conf_hashmap;
}


//check if config does not exits create a new one and the full path of the config file
fn check_config(path:&mut PathBuf){
    if !path.exists() {
        let mut folders = path.clone();
        folders.pop();

        match create_dir_all(folders) {
            Ok(_) => { 
                println!(" [?] {} has been created",path.display() );
            },
            Err(_) => { 
                eprintln!(" [!] The config folder doesn't exists and it can't be create in {}, possibely a permision error \n exiting ...",path.display() );
                exit(1);
            },
        }            
        
        match File::create(&path)  {
            Ok(mut config_file) => { 
                config_file.write_all(b"passfile_path: default\nlpm_prompt: default\nlpm_remote_server: none").expect("failed to write in the config file");
                println!(" [?] {} has been created",path.display() );
            },
            Err(_) => { 
                eprintln!(" [!] The config file can't be create in {}, possibely a permision error \n exiting ...",path.display() );
                exit(1);
            },
        }
      
    }
}


//recreate the file that has been corrupted
fn recreate_file(config_file:&mut File) {
    eprintln!("corrupted conf file restoring...");
    config_file.set_len(0).expect("failed to clear the content of the file");
    config_file.write_all(b"passfile_path: default\nlpm_prompt: default\nlpm_remote_server: default").expect("failed to write in the config file");
}

// Configs files from linux|macos and windows determined by compiler

#[cfg(any(target_os = "linux", target_os = "macos"))]
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

