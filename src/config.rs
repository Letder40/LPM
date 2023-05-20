use std::{collections::{HashMap}, io::{Write, Read, stdin, stdout},path::PathBuf, fs::{File, create_dir_all}};

use crate::utils::{exit, print_info};


//Reading the file and transform the content in a legible format

pub fn read_config() -> HashMap<String, String> {

    let mut path:PathBuf = config_path(); // Getting path from private function config_path(), it get the config path widnows/linux/macos
    check_config(&mut path); // Checking if config file exists if not create a new one and all the parents folders

    let mut config_file = File::open(&path).unwrap();
    let mut read_buffer:Vec<u8> = vec![];
    match config_file.read_to_end(&mut read_buffer) {
        Ok(_) => {},
        Err(_) => {
            eprint!(" [!] wasn't been posible to read the file ")
        },
    }

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
                exit(1, "Your config file is corrupted try to fix it or create a new one\n");
                break;
            }else if input.trim().to_lowercase() == "y" {
                let mut write_config_file = File::create(&path).unwrap();
                recreate_file(&mut write_config_file);
                break;
            }
        }

        data_vec = vec!["passfile_path", "default", "lpm_prompt", "default", "lpm_remote_server", "none"];

    }

    let mut data_hashmap:HashMap<String, String> = HashMap::new();
    
    //splitting the vector in chunks of size 2 and adding it to the hashmap then ...
    //Getting a hashmap of type String because &str is borrowed

    for chunk in data_vec.chunks(2) {
        let key = chunk[0].to_owned();
        
        let value = if key.as_str() == "passfile_path" && chunk[1].trim() == "default" {
            lpm_default_path()
        } else {
            chunk[1].trim().to_owned()
        };
        
        data_hashmap.insert(key, value);
    }

    return data_hashmap;
}


//check if config does not exits create a new one and the full path of the config file
fn check_config(path:&mut PathBuf){
    if !path.exists() {
        let mut folders = path.clone();
        folders.pop();

        match create_dir_all(folders) {
            Ok(_) => { 
                print_info(format!("{} has been created",path.display()).as_str());
            },
            Err(_) => { 
                let error = format!("The config folder doesn't exists and it can't be create in {}, possibly a permision error \n",path.display() );
                exit(1, error.as_str());
            },
        }            
        
        match File::create(&path)  {
            Ok(mut config_file) => { 
                config_file.write_all(b"passfile_path: default\nlpm_prompt: default\nlpm_remote_server: none").expect("failed to write in the config file");
            },
            Err(_) => { 
                let error = format!("The config file can't be create in {}, possibely a permision error \n",path.display() );
                exit(1, error.as_str());
            },
        }
      
    }
}


//recreate the file that has been corrupted
fn recreate_file(config_file:&mut File) {
    print_info("corrupted conf file restoring...");
    config_file.set_len(0).expect("failed to clear the content of the file");
    config_file.write_all(b"passfile_path: default\nlpm_prompt: default\nlpm_remote_server: default").expect("failed to write in the config file");
}

// Configs files from linux|macos and windows determined by compiler

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn config_path() -> PathBuf {
    let mut home = std::env::var("HOME").unwrap();
    home.push_str("/.config/lpm/lpm.conf");
    let config_path = PathBuf::from(home);
    return config_path;
}

#[cfg(target_os = "windows")]
pub fn config_path() -> PathBuf {
    let mut home = std::env::var("USERPROFILE").unwrap();
    home.push_str("/AppData/Roaming/lpm/lpm.conf");
    let config_path = PathBuf::from(home);
    return config_path;
}

fn lpm_default_path() -> String {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let mut config_path_str = std::env::var("HOME").unwrap();
    
    #[cfg(target_os = "windows")]
    let mut config_path_str = std::env::var("USERPROFILE").unwrap();

    config_path_str.push_str("/.passfile.lpm");
    return config_path_str;
}