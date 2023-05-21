use std::{fs::File, io::{Write, Read}, path::PathBuf};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, generic_array::GenericArray};
use sha2::{Digest, Sha256};
use rand::Rng;
use typenum::U32;
use crate::{config::read_config, utils::{exit, print_err}};
use zeroize::{self, Zeroize};

// Getting a hash of the provaided password in order to have a password with fixed lenght.
pub fn get_key(key_as_string: &String ) -> GenericArray<u8, U32> {
    let key_as_bytes = key_as_string.as_bytes();
    let mut hasher = Sha256::new();
    hasher.update(key_as_bytes);
    let key = hasher.finalize();
    GenericArray::from(key)
}

pub fn encrypt(mut key:GenericArray<u8, U32>, passfile_data_bytes: Vec<u8>)  {
    let cipher = Aes256Gcm::new(&key);
    key.zeroize();
    
    let mut nonce_buff: Vec<u8> = Vec::new();
    for _ in 0..12 {
        let byte = rand::thread_rng().gen();
        nonce_buff.push(byte);
    }

    let nonce = Nonce::from_slice(nonce_buff.as_slice());
    let mut cipher_data = match cipher.encrypt(nonce, passfile_data_bytes.as_ref()) {
        Ok(ok) => {
            ok 
        }
        Err(_) => {
            print_err("The encryption has failed");
            panic!()
        }
        
    };
    //println!("Ciphering...\n{:?} of {} bytes", nonce, nonce.len());   // for testing purposes
    //println!("{:?} of {} bytes\n\n", cipher_data, cipher_data.len());    // for testing purposes

    //Populating passfile.lpm with 12bytes(64bits) of nonce and the cipher data
    let passfile_path_string = &read_config().passfile_path;
    let passfile_path = PathBuf::from(passfile_path_string);
    let mut passfile = File::create(passfile_path).unwrap();
    let mut data_buf: Vec<u8> = Vec::new();
    data_buf.append(&mut nonce_buff);
    data_buf.append(&mut cipher_data);
    match passfile.write_all(data_buf.as_slice()) {
        Ok(_) => { },
        Err(_) => {
            print_err("Has not been posible to write data in passfile.lpm, check permission issues");
            panic!()
        }
    }
}


pub fn decrypt(key: GenericArray<u8, U32>) -> Vec<u8> {
    
    let decipher = Aes256Gcm::new(&key);
    
    let mut buffer:Vec<u8> = Vec::new();

    // Getting nonce from first 12 bytes(96 bits) of .passfile.lpm
    let passfile_string = &read_config().passfile_path;
    let passfile_path = PathBuf::from(passfile_string);
    let mut passfile = File::open(passfile_path).unwrap();
    match passfile.read_to_end(&mut buffer) {
        Ok(_) => {  },
        Err(_) => {
            print_err("Has not been posible to read data in passfile.lpm, check permission issues");
            panic!()
        }
    };

    let nonce = Nonce::from_slice(buffer.split_at(12).0);    
    let cipher_text = buffer.split_at(12).1;
    
    let mut textbuf: Vec<u8> =  Vec::new();

    match decipher.decrypt(nonce, cipher_text) {
        Ok(returned_buf) => {
            textbuf = returned_buf;
        },
        Err(_) => {
            exit(1, "Access unauthorized. \n")
        },
    };
    
    //let plain_text = String::from_utf8(textbuf.clone()).unwrap(); // testing purposes
    //println!("Decrypting...\n{:?} of {} bytes", nonce, nonce.len());   // for testing purposes
    //println!("{:?} of {} bytes -> {}",cipher_text, cipher_text.len(), plain_text );    // for testing purposes

    textbuf

}


