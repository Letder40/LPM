use crate::{commands::{clear, help, author_table, gc}, config::read_config, utils::{print_in_color, exit, read_pass, print_info, print_err, print_input}, serde::deserialize_passwords};
use aes_gcm::{aead::{generic_array::GenericArray, Aead}, Aes128Gcm, KeyInit};
use typenum::{U16, U12};
use crossterm::{execute, terminal::SetTitle, style::{Color, SetForegroundColor}};
use tabled::{builder::Builder, settings::{Style, Margin}}; 
use std::{io::{stdout, stdin, Read, Write}, net::TcpStream};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey, pkcs8::{EncodePublicKey, DecodePublicKey}};
use zeroize::Zeroize;

pub fn client(){
    let config = read_config();
    let mut rng = rand::thread_rng();
    let bits = 2000;
    let privkey = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let pubkey = RsaPublicKey::from(&privkey);

    let server = config.remote_server.lpm_remote_server_ipaddr;
    if server.is_empty() {
        exit(1, "Connection to a remote server functionality is on but no ip address has been provided")
    }

    print_info(format!("Establishing conection to lpm server in {server} ...").as_str());

    let mut socket = match TcpStream::connect(server + ":9702"){
        Ok(socket) => { socket }
        Err(err) => {
            exit(1, format!("{err}").as_str());
            panic!();
        }
    };

    // Get server's Rsa pubkey

    let mut readbuf:[u8; 1024] = [0; 1024];
    let n = match socket.read(&mut readbuf) {
        Ok(n) => { n }
        Err(err) => {
            exit(1, format!("{err}").as_str());
            panic!();
        }
    };

    let key_der = readbuf[0..n].to_vec();
    let server_pubkey = match RsaPublicKey::from_public_key_der(&key_der) {
        Ok(pubkey) => { pubkey }
        Err(err) => {
            exit(1, format!("{err}").as_str());
            panic!();
        }
    };

    // Send client's Rsa pubkey

    let key_der = pubkey.to_public_key_der().unwrap();
    socket.write_all(key_der.as_bytes()).expect("failled to share public key");
    
    // send Master_key
    let mut password = read_pass();
    let password_encrypted = server_pubkey.encrypt(&mut rng, Pkcs1v15Encrypt, password.as_bytes()).unwrap();
    password.zeroize();
    socket.write_all(&password_encrypted).expect("failled to share master key");

    // read response 
    let mut read_buf: [u8; 1024] = [0; 1024];
    let n = match socket.read(&mut read_buf) {
        Ok(n) => { n }
        Err(_) => { 
            exit(1, "has not been posible to read the server response");
            panic!();
        }
    };

    let response = read_buf[0..n].to_vec();
    let response_str = String::from_utf8(response).unwrap();
    
    if response_str != "correct" {
        exit(1, "Access unauthorized. \n")
    }

    clear();
    //Setting new title
    let title:SetTitle<String> = SetTitle(String::from("| LPM | Letder's password manager |"));
    execute!(
        stdout(), 
        title,
    ).unwrap();
    

    let ascii_art = [
        "     ___       ","________  _____ ______      \n",
        "    |\\  \\     ","|\\   __  \\|\\   _ \\  _   \\    \n", 
        "    \\ \\  \\    ","\\ \\  \\|\\  \\ \\  \\\\\\__\\ \\  \\   \n", 
        "     \\ \\  \\    ","\\ \\   ____\\ \\  \\\\|__| \\  \\  \n", 
        "      \\ \\  \\____","\\ \\  \\___|\\ \\  \\    \\ \\  \\ \n", 
        "       \\|_______|","\\|__|     \\|__|     \\|__|\n", 
        "\n", 
        "\n",
        "\n"
    ];
    
    let mut index = 0;
    for line in ascii_art.iter() {
        if index % 2 == 0 {
            print!("{line}");
            index+=1;
            continue;
        }
        print_in_color(Color::Yellow,line);
        index+=1
    }

    loop {

        let prompt = format!("{} ", config.lpm_prompt);

        print_in_color(Color::Green, &prompt);

        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
                
       

        //np 
        if (input.as_str().trim().starts_with("np") && input.as_str().trim().split(' ').count() == 3 ) || (input.as_str().trim().starts_with("new password") && input.as_str().trim().split(' ').count()  == 4){
            if input.as_str().trim().starts_with("np"){
                send(format!("np {} {}", input.trim().to_string().split(' ').collect::<Vec<&str>>()[1], input.trim().to_string().split(' ').collect::<Vec<&str>>()[2]), &mut socket, &server_pubkey);
                response_np(&mut socket, &privkey)
            }else{
                send(format!("new password {} {}", input.trim().to_string().split(' ').collect::<Vec<&str>>()[2], input.trim().to_string().split(' ').collect::<Vec<&str>>()[3]), &mut socket, &server_pubkey);
                response_np(&mut socket, &privkey)
            }
            continue;
        }
        //cp
        //rm

        match input.as_str().trim() {
            "help"                        => { help() }
            "list"               |  "lp"  => { send("lp".to_string(), &mut socket, &server_pubkey); println!("{}", get(&mut socket, &privkey)) }
            "new password"       |  "np"  => { send(ask_password(), &mut socket, &server_pubkey); response_np(&mut socket, &privkey) }
            "get configuration"  |  "gc"  => { gc() }
            "author"             |  "lpm" => { author_table() }
            "exit"         | "w" |  "q"   => { std::process::exit(0); }  
            "clear"                       => { clear() }
            ""                            => {}
            _                             => { print_err("Invalid Command, you can use help to list all commands");}
        }
        
    };

}

fn send(action: String, socket: &mut TcpStream, server_pubkey: &RsaPublicKey){
    let mut rng = rand::thread_rng();
    let action_encripted = server_pubkey.encrypt(&mut rng, Pkcs1v15Encrypt, action.as_bytes()).unwrap();
    socket.write_all(&action_encripted).unwrap();
}

fn get(socket: &mut TcpStream, privkey: &RsaPrivateKey) -> String {
    let mut read_buf: [u8; 1024] = [0; 1024];
    let n = match socket.read(&mut read_buf) {
        Ok(n) => { n }
        Err(err) => {
            exit(1, format!("{err}").as_str());
            panic!();
        }
    };
    
    let messages_bytes = privkey.decrypt(Pkcs1v15Encrypt, read_buf[0..n].to_vec().as_ref()).unwrap();

    if messages_bytes[0..5].to_owned() == b"empty"{
        return format!("{} [!] {}You don't have any saved password", SetForegroundColor(Color::Red), SetForegroundColor(Color::Reset));
    }
    let key = messages_bytes[0..16].to_vec();
    let nonce = messages_bytes[16..28].to_vec();
    let encrypted_table = messages_bytes[28..].to_vec();

    let key_array: GenericArray<u8, U16> = GenericArray::clone_from_slice(key.as_slice());
    let nonce_array: GenericArray<u8, U12> = GenericArray::clone_from_slice(nonce.as_slice());
    let key = Aes128Gcm::new(&key_array);

    let passfile_data_bytes = key.decrypt(&nonce_array, encrypted_table.as_slice()).unwrap();

    let passfile_data = deserialize_passwords(&passfile_data_bytes);

    let mut builder = Builder::default();
    let columns = vec!["#".to_owned(), "Id".to_owned(), "Password".to_owned()];
    let mut n = 1;
    builder.set_header(columns);
    
    for password_data in passfile_data.iter(){
        let row = vec![n.to_string(), password_data.id.clone(), password_data.value.clone()];
        builder.push_record(row);
        n += 1
    }

     builder.build()
    .with(Style::rounded())
    .with(Margin::new(2, 0, 1, 1))
    .to_string()

}

fn ask_password() -> String {
    let mut input_buffer = String::new();
    print_input("Password id: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut input_buffer).unwrap();
    let id = input_buffer.trim();
    
    let mut input_buffer = String::new();
    print_input("Password value: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut input_buffer).unwrap();
    let password = input_buffer.trim();

    format!("np {id} {password}")
}

fn response_np(socket: &mut TcpStream, privkey: &RsaPrivateKey){
    let mut read_buf: [u8; 256] = [0; 256];
    let n = match socket.read(&mut read_buf){
        Ok(n) => {n}
        Err(err) => { 
            print_err(format!("{err}").as_str()); 
            return;
        } 
    };
    let message = privkey.decrypt(Pkcs1v15Encrypt, &read_buf[0..n]).unwrap();
    if message == b"reused"{
        print_err("Password Identifier must be unique\n")
    }
}
