// `error_chain!` can recurse
#![recursion_limit = "64"]

#[macro_use]
extern crate error_chain;
extern crate encoding;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod yeelight;

//use std::env;
use std::net::TcpStream;
use std::io::prelude::{Read, Write};
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::ASCII;

const HOST: &'static str = "192.168.1.83:55443";

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        errors {
            InvalidEncodingConversion(msg: String) {
                description("Invalid encoding conversion")
                display("Invalid encoding conversion: {}", msg)
            }
        }
        foreign_links {
            Fmt(::std::fmt::Error);
            Io(::std::io::Error) #[cfg(unix)];
        }
    }
}

use errors::*;

fn main() {
    if let Err(ref e) = run() {
        println!("Error: {}", e);

        for e in e.iter().skip(1) {
            println!("Caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            println!("Backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

fn send_command(stream: &mut TcpStream, command: yeelight::CommandMessage) -> Result<String> {
    let cmd = serde_json::to_string(&command)
                        .chain_err(|| "Failed to create JSON command")?;
    let cmd_bytes = ASCII.encode(&cmd, EncoderTrap::Strict)
             .map_err(|e| Error::from_kind(ErrorKind::InvalidEncodingConversion(e.into())))
             .chain_err(|| "Failed to ASCII encode command")?;

    stream.write_all(&cmd_bytes).chain_err(|| "Failed to send command to server")?;
    if cfg!(feature="debug") {
        println!("Sent: {:?}", cmd_bytes);
    }

    let mut response = [0; 1024];
    stream.read(&mut response[..]).chain_err(|| "Failed to read server response")?;

    ASCII.decode(&response, DecoderTrap::Strict)
         .map_err(|e| Error::from_kind(ErrorKind::InvalidEncodingConversion(e.into())))
         .chain_err(|| "Failed to ASCII decode server response")
}

fn run() -> Result<()> {/*
    let _command = match env::args().nth(1) {
        Some(cmd) => cmd,
        None => {
            let my_name = env::args().nth(0).unwrap();
            panic!("Usage: {} [command]", my_name)
        }
    };*/

    let mut stream = TcpStream::connect(HOST).chain_err(|| format!("Failed to connect to {}", HOST))?;
    send_command(&mut stream, yeelight::CommandMessage::new_toggle(0))
                 .chain_err(|| "Failed to send command to server")?;
    Ok(())
}
