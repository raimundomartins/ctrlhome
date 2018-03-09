extern crate encoding;
extern crate json;

use std::error;
use std::io::prelude::*;
use std::net::TcpStream;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::ASCII;

pub struct CommandMessage {
    id: i32,
    method: &'static str,
    params: json::JsonValue,
}

pub enum ResultMessage {
    Ok(i32),
    Error {id:i32, code:i32, message:String},
    Response(i32, Vec<String>),
}

pub struct NotificationMessage {
    method: String, // Can only be "props"
    params: Vec<String/*"key":"val"*/>,
}

pub enum TransitionEffect {
    Sudden,
    Smooth{duration: i32},
}

pub enum PowerOnMode {
    Normal,
    CT,
    RGB,
    HSV,
    Flow,
    Night,
}

trait AppendableToJsonArray {
    fn append_to(&self, jarray: json::Array) -> json::Array;
}

impl AppendableToJsonArray for TransitionEffect  {
    fn append_to(&self, mut jarray: json::Array) -> json::Array {
        match self {
            &TransitionEffect::Sudden => {
                jarray.push("sudden".into());
                jarray.push(0.into());
            },
            &TransitionEffect::Smooth{duration} => {
                jarray.push("smooth".into());
                jarray.push(duration.into());
            }
        }
        jarray
    }
}

impl AppendableToJsonArray for Option<PowerOnMode> {
    fn append_to(&self, mut jarray: json::Array) -> json::Array {
        if let &Some(ref pom) = self {
            jarray.push(match pom {
                &PowerOnMode::Normal => 0,
                &PowerOnMode::CT => 1,
                &PowerOnMode::RGB => 2,
                &PowerOnMode::HSV => 3,
                &PowerOnMode::Flow => 4,
                &PowerOnMode::Night => 5,
            }.into());
        }
        jarray
    }
}

impl CommandMessage {
    pub fn new_get_prop(id: i32, params: &Vec<&str>) -> CommandMessage {
        let params: Vec<json::JsonValue> = params.iter().map(|&x| json::JsonValue::from(x)).collect();
        CommandMessage { id, method: "get_prop", params: params.into() }
    }

    pub fn new_set_color_temp(id: i32, color_temp: i32, effect: TransitionEffect) -> CommandMessage {
        CommandMessage { id, method: "set_ct_abx", params: effect.append_to(vec![color_temp.into()]).into() }
    }

    pub fn new_set_rgb(id: i32, r: i8, g: i8, b: i8, effect: TransitionEffect) -> CommandMessage {
        let rgb = ((r as i32)<<16)+((g as i32)<<8)+(b as i32);
        CommandMessage { id, method: "set_rgb", params: effect.append_to(vec![rgb.into()]).into() }
    }

    pub fn new_set_hsv(id: i32, h: i16, s: i8, effect: TransitionEffect) -> CommandMessage {
        CommandMessage { id, method: "set_hsv", params: effect.append_to(vec![h.into(), s.into()]).into() }
    }

    pub fn new_set_brightness(id: i32, b: i8, effect: TransitionEffect) -> CommandMessage {
        CommandMessage { id, method: "set_bright", params: effect.append_to(vec![b.into()]).into() }
    }

    pub fn new_set_power(id: i32, on: bool, mode: Option<PowerOnMode>, effect: TransitionEffect) -> CommandMessage {
        let on: json::JsonValue = {if on { "on" } else { "off" }}.into();
        CommandMessage { id, method: "set_power", params: mode.append_to(effect.append_to(vec![on])).into() }
    }

    pub fn new_toggle(id: i32) -> CommandMessage {
        CommandMessage { id, method: "toggle", params: array![] }
    }

    pub fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<String, Box<error::Error+Send+Sync>> {
        let command = object!{ "id" => self.id, "method" => self.method, "params" => self.params.clone() }.dump()+"\r\n";
        let command_bytes = ASCII.encode(&command, EncoderTrap::Strict).map_err(|x| x.into_owned())?;

        stream.write_all(&command_bytes)?;
        println!("Sent: {:?}", command_bytes);

        let mut response = [0; 1024];
        stream.read(&mut response[..])?;

        Ok(ASCII.decode(&response, DecoderTrap::Strict)?)
    }
}

