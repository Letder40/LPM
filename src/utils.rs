use std::io::stdout;

//stop the program with a specific error code
pub fn exit(code :i32){
    crossterm::execute!(stdout(), crossterm::terminal::LeaveAlternateScreen).unwrap();
    std::process::exit(code);
}