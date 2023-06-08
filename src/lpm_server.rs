use crate::{utils::{read_pass, print_info}, crypto::decrypt, serde::{PasswordData, deserialize_passwords, serialize_passwords}};
use rand::RngCore;
use sha2::{Digest, Sha256};
use aes_gcm::{Aes256Gcm, Aes128Gcm, KeyInit, aead::{generic_array::GenericArray, Aead}};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey, pkcs8::{EncodePublicKey, DecodePublicKey}};
use tokio::{net::{self, TcpStream}, io::{AsyncWriteExt, AsyncReadExt}};


// lpm share
#[tokio::main]
pub async fn main(){

    
     //passfile decryption
     let key_as_string = read_pass();
     let key_as_bytes = key_as_string.as_bytes();

     let mut hasher = Sha256::new();
     hasher.update(key_as_bytes);
     let sha_key = hasher.finalize();
     let aes_key:Aes256Gcm = Aes256Gcm::new(&sha_key);
     decrypt(aes_key);

    let listerner = net::TcpListener::bind("0.0.0.0:9702").await.unwrap();
    print_info("Listenning connections on port 9702...");
    
    loop {
        let (mut socket, _ )= match listerner.accept().await {
            Ok(socket) => { socket },
            Err(err) => { 
                eprintln!("error: {err}");
                return;
            },
        };
        
        tokio::spawn(async move {  

            let aes_key:Aes256Gcm = Aes256Gcm::new(&sha_key);
            let serialized_passfile_data = decrypt(aes_key);
            let passfile_data = deserialize_passwords(&serialized_passfile_data);

            // keys generation
            let mut read_buf: [u8; 1024] = [0; 1024];
            let mut rng = rand::rngs::OsRng::default();
            let bits = 2000;
            let privatekey = RsaPrivateKey::new(&mut rng, bits).expect("private key has not been successfuly generated");
            let publickey = RsaPublicKey::from(privatekey.clone());
           
            // send publickey to client
            let pubkey_serialize = publickey.to_public_key_der().unwrap();
            socket.write_all(pubkey_serialize.as_bytes()).await.unwrap();
            
            //get client's public key
            let n = match socket.read(&mut read_buf).await {
                Ok(size) => { size },
                Err(err) => { 
                    eprintln!("error: {err}");
                    return;
                },
            };

            let bytes = read_buf[0..n].to_vec();
            let client_publickey = match RsaPublicKey::from_public_key_der(&bytes) {
                Ok(pubkey) => { pubkey }
                Err(_) => {
                    socket.shutdown().await.unwrap();
                    return;
                }
            };

            // read master key
            let mut read_buf: [u8; 1024] = [0; 1024];
            let n = match socket.read(&mut read_buf).await {
                Ok(size) => { size },
                Err(err) => { 
                    eprintln!("error: {err}");
                    return;
                },
            };

            let key_as_bytes = match privatekey.decrypt(Pkcs1v15Encrypt, read_buf[0..n].to_vec().as_ref()){
                Ok(key) => { key }
                Err(_) => {
                    socket.shutdown().await.unwrap();
                    return;
                } 
            };
            
            let mut hasher = Sha256::new();
            hasher.update(key_as_bytes);
            let provided_key = hasher.finalize();
            
            if provided_key == sha_key {
                socket.write_all(b"correct").await.unwrap();
            }else{
                socket.write_all(b"incorrect").await.unwrap();
                return;
            }

            // Connection Stablished nad password correct
            loop {
                let mut read_buf: [u8; 1024] = [0; 1024];
                let n = match socket.read(&mut read_buf).await {
                    Ok(size) => { size },
                    Err(err) => { 
                        eprintln!("error: {err}");
                        return;
                    },
                };
                let action_encrypted = read_buf[0..n].to_vec();
                let action_decrypted = privatekey.decrypt(Pkcs1v15Encrypt, &action_encrypted).unwrap();
                let action_str = String::from_utf8(action_decrypted).unwrap();
                match action_str.as_str().trim() {
                    "lp" => { lp(&passfile_data, &mut socket, &client_publickey).await }
                    _ => {}
                }
            }  
            
        });

    }
}

async fn lp(passfile_data:&Vec<PasswordData>, socket: &mut TcpStream, client_pubkey: &RsaPublicKey){

    if passfile_data.is_empty() {
        //print_err("You don't have any saved password");
        let mut rng = rand::rngs::OsRng::default();
        let encrypted_message = client_pubkey.encrypt(&mut rng, Pkcs1v15Encrypt, b"empty").unwrap();
        socket.write_all(&encrypted_message).await.unwrap();
        return;
    }
    

    let mut key_bytes: [u8; 16] = [0; 16];
    rand::rngs::OsRng::default().fill_bytes(&mut key_bytes);
    let key_array = GenericArray::from(key_bytes);
    let key = Aes128Gcm::new(&key_array);
    let mut nonce: [u8; 12] = [0; 12];
    rand::rngs::OsRng::default().fill_bytes(&mut nonce);
    let nonce_array = GenericArray::from(nonce);

    let passfile_data_serialized = serialize_passwords(passfile_data);
    let encrypted_table = key.encrypt(&nonce_array, passfile_data_serialized.as_slice()).unwrap();
    
    let mut message_vec:Vec<u8> = vec![];
    message_vec.append(&mut key_bytes.to_vec());
    message_vec.append(&mut nonce.to_vec());
    message_vec.append(&mut encrypted_table.to_vec());

    let mut rng = rand::rngs::OsRng::default();
    println!("{}", encrypted_table.len());
    println!("{}", message_vec.len());
    let encrypted_message = client_pubkey.encrypt(&mut rng, Pkcs1v15Encrypt, &message_vec).unwrap();
    socket.write_all(&encrypted_message).await.unwrap();

}
