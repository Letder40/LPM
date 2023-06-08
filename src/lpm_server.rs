use crate::{utils::{read_pass}, crypto::decrypt, serde::{PasswordData, deserialize_passwords}};
use sha2::{Digest, Sha256};
use aes_gcm::{Aes256Gcm, KeyInit};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey, pkcs8::{EncodePublicKey, DecodePublicKey}};
use zeroize::Zeroize;
use tokio::{net, io::{AsyncWriteExt, AsyncReadExt}};

use tabled::{builder::Builder, settings::{Style, Margin}}; 


// lpm share
#[tokio::main]
pub async fn main(){

    

    let listerner = net::TcpListener::bind("0.0.0.0:9702").await.unwrap();
    println!("Listenning connections on port 9702");
    
    loop {
        let (mut socket, _ )= match listerner.accept().await {
            Ok(socket) => { socket },
            Err(err) => { 
                eprintln!("error: {err}");
                return;
            },
        };
        
        tokio::spawn(async move {    
            //passfile decryption
            let mut key_as_string = read_pass();
            let key_as_bytes = key_as_string.as_bytes();

            let mut hasher = Sha256::new();
            hasher.update(key_as_bytes);
            let sha_key = hasher.finalize();
            let aes_key:Aes256Gcm = Aes256Gcm::new(&sha_key);

            key_as_bytes.to_owned().zeroize();
            key_as_string.zeroize();

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

                match action_str.as_str() {
                    "lp" => { lp(&passfile_data).await }
                    _ => {}
                }
            }  
            
        });

    }
}

async fn lp(passfile_data:&Vec<PasswordData>){

    if passfile_data.len() == 0 {
        //print_err("You don't have any saved password");
        
        return;
    }

    let mut builder = Builder::default();
    let columns = vec!["#".to_owned(), "Id".to_owned(), "Password".to_owned()];
    let mut n = 1;
    builder.set_header(columns);
    
    for password_data in passfile_data.iter(){
        let row = vec![n.to_string(), password_data.id.clone(), password_data.value.clone()];
        builder.push_record(row);
        n += 1
    }

    let table = builder.build()
    .with(Style::rounded())
    .with(Margin::new(2, 0, 1, 1))
    .to_string();

    println!("{}", table);
        
}
