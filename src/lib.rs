use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle}};
use std::io::{stdin, Read, stdout, Write};
//use aes::Aes256;

//Leer la contraseña intrducida por el usuario
pub fn read_pass(){
    
    //change to alternative buffer screen
    execute!(stdout(), EnterAlternateScreen).unwrap();
    let title:SetTitle<String> = SetTitle(String::from("Hola mundo"));
    
    execute!(stdout(), title).unwrap();

    // Enable raw mode
    print!("Master key : ");
    stdout().flush().unwrap();
    enable_raw_mode().unwrap();

    // Read a single character
    let mut input = [0u8; 1];
    stdin().read_exact(&mut input).unwrap();

    // Disable raw mode
    disable_raw_mode().unwrap(); 

    //change to alternative buffer screen
    execute!(stdout(), LeaveAlternateScreen).unwrap();  

    // Print the character as ASCII code
    println!("Has introducido el carácter {}", input[0]); 
}
