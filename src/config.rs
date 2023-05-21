use std::{io::{Write, Read},path::{PathBuf}, fs::{File, create_dir_all}};
use toml;
use serde_derive::{Deserialize, Serialize};
use crate::utils::{exit, print_info};


//Reading the file and transform the content in a legible format
#[derive(Deserialize, Serialize)]
pub struct ConfigFile {
    pub passfile_path: String,
    pub lpm_prompt: String,
    pub remote_server: RemoteServer, 
}   

#[derive(Deserialize, Serialize)]
pub struct RemoteServer {
    pub lpm_remote_server: bool,
    pub lpm_remote_server_type: String,
    pub lpm_remote_server_path: String,
}


pub fn read_config() -> ConfigFile {
    check_config(&mut config_path());
    let mut config_file_content:Vec<u8> = Vec::new();
    let mut config_file = File::open(config_path().as_path()).expect(" [!] config file could not be open, check permission issues\n");
    config_file.read_to_end(&mut config_file_content).expect(" [!] config file could not be readed, check permission issues\n");
    let config_file_content = String::from_utf8(config_file_content).expect("[!] there is non utf8 characters in the config file\n");
    let config = toml::from_str(&config_file_content).expect("[!] has not been posible to read the config file, check toml syntax\n");
    config

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
                create_conf_file(&mut config_file);
            },
            Err(_) => { 
                let error = format!("The config file can't be create in {}, possibely a permision error \n",path.display() );
                exit(1, error.as_str());
            },
        }
      
    }
}


//recreate the file that has been corrupted
fn create_conf_file(config_file:&mut File) {    
    let default_config_file:ConfigFile = ConfigFile { 
        passfile_path: lpm_default_path(),
        lpm_prompt: "lpm >".to_string(),
        remote_server: RemoteServer{
            lpm_remote_server: false,
            lpm_remote_server_type: "".to_string(),
            lpm_remote_server_path: "".to_string()
        }

    };

    let toml = toml::to_string(&default_config_file).unwrap();
    config_file.write_all(toml.as_bytes()).unwrap();
    print_info( format!("Created config in {}", config_path().display()).as_str() );

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