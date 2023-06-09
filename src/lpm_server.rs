use crate::{utils::{read_pass, print_info}, crypto::{decrypt, encrypt}, serde::{PasswordData, deserialize_passwords, serialize_passwords}, commands::random_password};
use bincode::serialize;
use serde_derive::Serialize;
use sha2::{Digest, Sha256};
use aes_gcm::{Aes256Gcm, KeyInit};
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

        let connection = match socket.peer_addr() { 
            Ok(addr) => { addr }
            Err(_) => { continue; }
        };

        print_info(format!("connection from: {:?}", connection).as_str());

        
        tokio::spawn(async move {  
            let aes_key:Aes256Gcm = Aes256Gcm::new(&sha_key);
            let serialized_passfile_data = decrypt(aes_key.clone());
            let mut passfile_data = deserialize_passwords(&serialized_passfile_data);

            // keys generation
            let mut read_buf: [u8; 1024] = [0; 1024];
            let mut rng = rand::rngs::OsRng::default();
            let bits = 2048;
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

            // Connection Stablished and password correct
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
                let action_decrypted = match privatekey.decrypt(Pkcs1v15Encrypt, &action_encrypted){
                    Ok(action) => {action}
                    Err(_) => { 
                        print_info(format!("client: {:?} has disconnected", connection).as_str());
                        return;
                    }
                };
                let action_str = String::from_utf8(action_decrypted).unwrap();

                if action_str.starts_with("np") || action_str.starts_with("new password"){
                    np(action_str.clone(), &mut passfile_data, &client_publickey, &mut socket, aes_key.clone()).await;
                }

                if action_str.starts_with("rm") || action_str.starts_with("rem") || action_str.starts_with("del"){
                    let id = action_str.trim().to_string().split(' ').collect::<Vec<&str>>()[1].to_string();
                    println!("removing {id}");
                    let mut count = 0;
                    for password in passfile_data.clone(){
                        if password.id == id{
                            passfile_data.remove(count);
                            encrypt(&aes_key, serialize_passwords(&passfile_data));
                            println!("removed {id}");
                        }
                        count += 1;
                    }
                }

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
        get_ack(socket).await;
        return;

    }else{
        let mut rng = rand::rngs::OsRng::default();
        let encrypted_message = client_pubkey.encrypt(&mut rng, Pkcs1v15Encrypt, b"sending").unwrap();
        socket.write_all(&encrypted_message).await.unwrap();
        get_ack(socket).await;

    }

    let mut passfile_data_serialized = serialize_passwords(passfile_data);
    let blocks = (passfile_data_serialized.len() as f32 / 256.0 as f32).ceil() as u32 * 2;

    #[derive(Serialize)]
    struct BlocksData { value: u32 }
    let blocks_data = BlocksData { value: blocks };
    let blocks_data_serialized = serialize(&blocks_data).unwrap();
    
    let mut rng = rand::rngs::OsRng::default();
    let encrypted_blocks_data_serialized = client_pubkey.encrypt(&mut rng, Pkcs1v15Encrypt, &blocks_data_serialized).unwrap();
    socket.write_all(&encrypted_blocks_data_serialized).await.unwrap();
    get_ack(socket).await;

    if blocks <= 2 {
        let encrypted_block = client_pubkey.encrypt(&mut rng, Pkcs1v15Encrypt, &passfile_data_serialized).unwrap();
        socket.write_all(&encrypted_block).await.unwrap();
        get_ack(socket).await;
        return;
    }

    for _ in 0..blocks {
        let mut _passfile_data_block = vec![];

        if passfile_data_serialized.len() > 128{
            _passfile_data_block = passfile_data_serialized[0..128].to_vec();
        }else{
            _passfile_data_block = passfile_data_serialized.clone()
        }

        let encrypted_block = match client_pubkey.encrypt(&mut rng, Pkcs1v15Encrypt, &_passfile_data_block) {
            Ok(encrypted_block) => {encrypted_block},
            Err(err) => {
                println!("{}", _passfile_data_block.len());
                println!("{err}");
                return;
            },
        };
        socket.write_all(&encrypted_block).await.unwrap();
        get_ack(socket).await;

        if passfile_data_serialized.len() > 128{
            passfile_data_serialized = passfile_data_serialized[128..].to_vec();
        }

    }

}

async fn np(action_str: String, passfile_data:&mut Vec<PasswordData>, client_pubkey: &RsaPublicKey, socket: &mut TcpStream, aes_key: Aes256Gcm){
    let mut _id = String::new();
    let mut _password = String::new();

    if action_str.as_str().trim().starts_with("np"){
        _id = action_str.trim().to_string().split(' ').collect::<Vec<&str>>()[1].to_string();
        _password = action_str.trim().to_string().split(' ').collect::<Vec<&str>>()[2].to_string();
    }else{
        _id = action_str.trim().to_string().split(' ').collect::<Vec<&str>>()[2].to_string();
        _password = action_str.trim().to_string().split(' ').collect::<Vec<&str>>()[3].to_string();
    }
                    
    let mut password_data = PasswordData {
        id: _id,
        value: _password,
    };
    
    for password in passfile_data.clone() {
        if password.id == password_data.id{
            let mut rng = rand::rngs::OsRng::default();
            let message_encrypted = client_pubkey.encrypt(&mut rng, Pkcs1v15Encrypt, b"reused").unwrap();
            socket.write_all(&message_encrypted).await.unwrap();
            get_ack(socket).await;
            return;
        }
    }


    if password_data.value == "r" || password_data.value == "random"{
        password_data.value = random_password();
    } 

    passfile_data.push(password_data);
    let passfile_data_bytes = serialize_passwords(&passfile_data);

    encrypt(&aes_key, passfile_data_bytes);

    let mut rng = rand::rngs::OsRng::default();
    let message_encrypted = client_pubkey.encrypt(&mut rng, Pkcs1v15Encrypt, b"ok").unwrap();
    socket.write_all(&message_encrypted).await.unwrap();
    get_ack(socket).await;

}
async fn get_ack(socket: &mut TcpStream){
    let mut read_buf: [u8; 256] = [0; 256];
    socket.read(&mut read_buf).await.unwrap();
}