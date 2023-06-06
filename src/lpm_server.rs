use crate::{utils::{read_pass}, crypto::decrypt};
use sha2::{Digest, Sha256};
use aes_gcm::{Aes256Gcm, KeyInit};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use zeroize::Zeroize;

use tokio::{net, io::{AsyncWriteExt, AsyncReadExt}};

// lpm share
#[tokio::main]
pub async fn main(){

    let mut key_as_string = read_pass();
    let key_as_bytes = key_as_string.as_bytes();

    let mut hasher = Sha256::new();
    hasher.update(key_as_bytes);
    let sha_key = hasher.finalize();
    let aes_key:Aes256Gcm = Aes256Gcm::new(&sha_key);

    key_as_bytes.to_owned().zeroize();
    key_as_string.zeroize();

    let passfile_data = decrypt(aes_key);

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

            // generacion de claves
            // obtener claves
            // esperar por claves

            let mut read_buf: [u8; 1024] = [0; 1024];
            let mut rng = rand::rngs::OsRng::default();
            let bits = 2000;
            let privatekey = RsaPrivateKey::new(&mut rng, bits).expect("private key has not been successfuly generated");
            let publickey = RsaPublicKey::from(privatekey);
            
            
            let n = match socket.read(&mut read_buf).await {
                Ok(size) => { size },
                Err(err) => { 
                    eprintln!("error: {err}");
                    return;
                },
            };

            let key = String::from_utf8(read_buf[0..n].to_vec()).unwrap();
            let key_as_bytes = key.trim().as_bytes();
            let mut hasher = Sha256::new();
            hasher.update(key_as_bytes);
            let provided_key = hasher.finalize();
            
            if provided_key == sha_key {
                socket.write_all(b"correcto").await.unwrap()
            }else{
                socket.write_all(b"incorrecto").await.unwrap()
            }

            //   give passfile_data
 
            loop {
                let mut read_buf: [u8; 1024] = [0; 1024];
                let n = match socket.read(&mut read_buf).await {
                    Ok(size) => { size },
                    Err(err) => { 
                        eprintln!("error: {err}");
                        return;
                    },
                };

                //   receive passfile_data && update passfile_data


                // socket.write_all(&read_buf[0..n]).await.unwrap() // testing purposes
            }
        });

    }
}