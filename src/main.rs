extern crate encoding;
#[macro_use]
extern crate json;

//use std::env;
use std::error;
use std::io::prelude::*;
use std::net::TcpStream;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::ASCII;

const HOST: &'static str = "192.168.1.83:55443";

fn send_one_command(command: &str) -> Result<String, Box<error::Error + Send + Sync>> {
    let mut command_bytes = try!(ASCII.encode(command, EncoderTrap::Strict).map_err(|x| x.into_owned()));
    command_bytes.push('\r' as u8);
    command_bytes.push('\n' as u8);

    let mut stream = try!(TcpStream::connect(HOST));
    try!(stream.write_all(&command_bytes));

    let mut response = [0; 1024];
    try!(stream.read(&mut response[..]));

    Ok(ASCII.decode(&response, DecoderTrap::Strict).unwrap())
}

fn main() {/*
    let _command = match env::args().nth(1) {
        Some(cmd) => cmd,
        None => {
            let my_name = env::args().nth(0).unwrap();
            panic!("Usage: {} [command]", my_name)
        }
    };*/

    let json_command = object!{"id" => 1, "method" => "set_power", "params" => array!["on", "smooth", 500]};
    let _command = json_command.dump();

    match send_one_command(&json_command.dump()) {
        Ok(response) => println!("Got response: {}", response),
        Err(err) => println!("An error occurred: {}", err),
    }
}
