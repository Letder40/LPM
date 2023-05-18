use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, generic_array::GenericArray};
use sha2::{Digest, Sha256};
use rand::Rng;
use typenum::U32;
use crate::config::read_config;
use std::fs::File;
use std::io::{Write, Read};
use std::path::PathBuf;
use zeroize::{self, Zeroize};

// Getting a hash of the provaided password in order to have a password with fixed lenght.
fn get_key(key_as_string: &String ) -> GenericArray<u8, U32> {
    let key_as_bytes = key_as_string.as_bytes();
    let mut hasher = Sha256::new();
    hasher.update(key_as_bytes);
    let key = hasher.finalize();
    GenericArray::from(key)
}

pub fn encrypt(mut passwd:String)  {
    let key = get_key(&passwd);
    passwd.zeroize();
    let cipher = Aes256Gcm::new(&key);
    
    //Getting all bytes of passfile
    let passfile_string = &read_config()["passfile_path"];
    let passfile_path = PathBuf::from(passfile_string);
    let mut passfile = File::open(&passfile_path).expect(" [!] passfile could not be opened, check permission issues");
    let mut data:Vec<u8> = Vec::new();
    passfile.read_to_end(&mut data).expect( "[!] File could not be readed, check permission issues.");

    let mut nonce_buff: Vec<u8> = Vec::new();
    for _ in 0..12 {
        let byte = rand::thread_rng().gen();
        nonce_buff.push(byte);
    }

    let nonce = GenericArray::from_slice(nonce_buff.as_slice());
    let mut cipher_data = cipher.encrypt(nonce, data.as_ref()).expect(" [!] The encryption has failed");
    
    //println!("Ciphering...\n{:?} of {} bytes", nonce, nonce.len());   // for testing purposes
    //println!("{:?} of {} bytes\n\n", cipher_data, cipher_data.len());    // for testing purposes

    //Populating passfile.lpm with 12bytes(64bits) of nonce and the cipher data
    let mut passfile = File::create(passfile_path).unwrap();
    let mut data_buf: Vec<u8> = Vec::new();
    data_buf.append(&mut nonce_buff);
    data_buf.append(&mut cipher_data);
    passfile.write_all(data_buf.as_slice()).expect(" [!] File could not be populated, check permission issues.");
}


pub fn decrypt(mut passwd: String) {
    let key = get_key(&passwd);
    let decipher = Aes256Gcm::new(&key);
    passwd.zeroize();
    
    let mut buffer:Vec<u8> = Vec::new();

    // Getting nonce from first 12 bytes(96 bits) of .passfile.lpm
    let passfile_string = &read_config()["passfile_path"];
    let passfile_path = PathBuf::from(passfile_string);
    let mut passfile = File::open(passfile_path).unwrap();
    passfile.read_to_end(&mut buffer).expect( "[!] File could not be readed, check permission issues.");

    let nonce = Nonce::from_slice(buffer.split_at(12).0);    
    let cipher_text = buffer.split_at(12).1;
    
    let text_buf = decipher.decrypt(nonce, cipher_text).unwrap();
    let plain_text = String::from_utf8(text_buf).unwrap();

    //println!("Decrypting...\n{:?} of {} bytes", nonce, nonce.len());   // for testing purposes
    //println!("{:?} of {} bytes -> {}",cipher_text, cipher_text.len(), plain_text );    // for testing purposes
}


