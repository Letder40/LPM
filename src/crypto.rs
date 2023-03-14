use aes_gcm::{Aes256Gcm, KeyInit};
use aes_gcm::aead::{Aead, generic_array::GenericArray};
use sha2::{Digest, Sha256};
use rand::Rng;

pub fn get_key(key_as_string: String ) -> [u8; 256] {
    let key_as_bytes = key_as_string.as_bytes();
    let mut hasher = Sha256::new();
    hasher.update(key_as_bytes);
    let key = hasher.finalize();
    GenericArray::from(key);
    
}

pub fn encrypt(text: String, passwd:Vec<u8>) -> Vec<u8> {
    let key = GenericArray::from_slice(passwd.as_slice());
    let cipher = Aes256Gcm::new(key);
    let mut nonce_buff: Vec<u8> = Vec::new();
    for _ in 0..96 {
        let bit:u8 = rand::thread_rng().gen();
        nonce_buff.push(bit);
    }
    let nonce = GenericArray::from_slice(nonce_buff.as_slice());

    let cipher_data = cipher.encrypt(nonce, text.as_ref()).expect("[!] The encryption has failed, this is an important error");
    println!("{:?} of {} bytes ->  {}", nonce, nonce.len(), text);
    println!("{:?} of {} bytes", cipher_data, cipher_data.len());
    cipher_data 
}

