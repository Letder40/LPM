use lpm::{utils::{exit, check_filepass}, home::home, arguments::check_arguments, config::read_config, lpm_client::client} ;

//     ___       ________  _____ ______      
//    |\  \     |\   __  \|\   _ \  _   \    
//    \ \  \    \ \  \|\  \ \  \\\__\ \  \   
//     \ \  \    \ \   ____\ \  \\|__| \  \  
//      \ \  \____\ \  \___|\ \  \    \ \  \ 
//       \ \_______\ \__\    \ \__\    \ \__\
//        \|_______|\|__|     \|__|     \|__|

fn main(){
    // Handler to handle user keyboard interruptions {ctrlc}
    ctrlc::set_handler(move || {
        println!("\n");
        exit(1, "Exited by user...\n");
    }).expect("Error setting Ctrl-C handler");
    check_filepass();
    check_arguments();
    // LPM INIT
    
    if !read_config().remote_server.lpm_remote_server {
        home();
    }else{
        client()
    }

}