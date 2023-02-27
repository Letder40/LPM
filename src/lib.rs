use std::io;
use std::io::Write;
use aes::Aes256;

//Leer la contraseña intrducida por el usuario
pub fn read_pass() -> String {
    print!("master key: ");
    //Vaciamos el buffer de salida para garantizar que el print anterior se vea antes que el input
    io::stdout().flush().unwrap();
    //variable mutable porque el input es incierto
    let mut input:String = String::new();
    io::stdin()
        //referencia a input porque queremos que read_line modifique el valor
        .read_line(&mut input)
        .expect("\nError mientras se leia la entrada del usuario");
    return input;
}


