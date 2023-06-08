use crate::{commands::{clear, help, author_table, gc}, config::read_config, utils::{print_in_color, exit, read_pass, print_info, print_err, print_input}};
use crossterm::{execute, terminal::SetTitle, style::Color};
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
        //cp
        //rm

        match input.as_str().trim() {
            "help"                        => { help() }
            "list"               |  "lp"  => { send("lp".to_string(), &mut socket, &server_pubkey); println!("{}", get(&mut socket, &privkey)) }
            "new password"       |  "np"  => { send(ask_password(), &mut socket, &server_pubkey) }
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
    let password_table_bytes = privkey.decrypt(Pkcs1v15Encrypt, read_buf[0..n].to_vec().as_ref()).unwrap();
    String::from_utf8(password_table_bytes).unwrap()
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
