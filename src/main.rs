#![allow(dead_code)]

extern crate encoding;
#[macro_use]
extern crate json;

mod yeelight;

//use std::env;
use std::net::TcpStream;

const HOST: &'static str = "192.168.1.83:55443";

fn main() {/*
    let _command = match env::args().nth(1) {
        Some(cmd) => cmd,
        None => {
            let my_name = env::args().nth(0).unwrap();
            panic!("Usage: {} [command]", my_name)
        }
    };*/

    let cmd = yeelight::CommandMessage::new_toggle(0);

    let mut stream = TcpStream::connect(HOST).unwrap();
    match cmd.send(&mut stream) {
        Ok(response) => println!("Got response: {}", response),
        Err(err) => println!("An error occurred: {}", err),
    }
}
