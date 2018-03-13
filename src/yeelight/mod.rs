use std::error;
use std::io::prelude::{Read,Write};
use std::net::TcpStream;
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::ASCII;
use serde_json;

#[derive(Serialize)]
#[serde(untagged)]
enum CommandParameter {
    Integer(i32),
    String(String),
}

macro_rules! impl_into_int_command_parameter { ($($ty:ty)*) => {
    $( impl From<$ty> for CommandParameter {
        #[inline] fn from(v: $ty) -> Self { CommandParameter::Integer(v as i32) }
    } )*
} }
impl_into_int_command_parameter!(i8 i16 i32 i64);

impl<'a> From<&'a str> for CommandParameter {
    fn from(string: &'a str) -> Self { CommandParameter::String(string.to_string()) }
}

#[derive(Serialize)]
pub struct CommandMessage {
    id: i32,
    method: &'static str,
    params: Vec<CommandParameter>,
}

pub enum ResultMessage {
    Ok(i32),
    Error {id:i32, code:i32, message:String},
    Response(i32, Vec<String>),
}

pub struct NotificationMessage {
    method: String, // Can only be "props"
    params: Vec<(String, String)>, /*"key":"val"*/
}

pub enum TransitionEffect {
    Sudden,
    Smooth{duration: i32},
}

pub enum PowerOnMode { Normal, CT, RGB, HSV, Flow, Night }

impl Into<Vec<CommandParameter>> for TransitionEffect {
    fn into(self) -> Vec<CommandParameter> {
        match self {
            TransitionEffect::Sudden => vec!["sudden".into(), 0.into()],
            TransitionEffect::Smooth{duration} => vec!["smooth".into(), duration.into()],
        }
    }
}

impl From<Option<PowerOnMode>> for CommandParameter {
    fn from(pom: Option<PowerOnMode>) -> Self {
        CommandParameter::Integer(match pom {
            Some(pom_i) => pom_i as i32,
            None => 0
        })
    }
}


impl CommandMessage {
    pub fn new_get_prop(id: i32, params: &Vec<&str>) -> CommandMessage {
        let params: Vec<_> = params.iter().map(|&x| x.into()).collect();
        CommandMessage { id, method: "get_prop", params }
    }

    pub fn new_set_color_temp(id: i32, color_temp: i32, effect: TransitionEffect) -> CommandMessage {
        let mut params = Vec::<CommandParameter>::with_capacity(3);
        params.push(color_temp.into());
        params.append(&mut effect.into());
        CommandMessage { id, method: "set_ct_abx", params }
    }

    pub fn new_set_rgb(id: i32, r: i8, g: i8, b: i8, effect: TransitionEffect) -> CommandMessage {
        let mut params = Vec::<CommandParameter>::with_capacity(3);
        params.push({((r as i32)<<16)+((g as i32)<<8)+(b as i32)}.into());
        params.append(&mut effect.into());
        CommandMessage { id, method: "set_rgb", params }
    }

    pub fn new_set_hsv(id: i32, h: i16, s: i8, effect: TransitionEffect) -> CommandMessage {
        let mut params = Vec::<CommandParameter>::with_capacity(4);
        params.push(h.into());
        params.push(s.into());
        params.append(&mut effect.into());
        CommandMessage { id, method: "set_hsv", params }
    }

    pub fn new_set_brightness(id: i32, b: i8, effect: TransitionEffect) -> CommandMessage {
        let mut params = Vec::<CommandParameter>::with_capacity(3);
        params.push(b.into());
        params.append(&mut effect.into());
        CommandMessage { id, method: "set_bright", params }
    }

    pub fn new_set_power(id: i32, on: bool, mode: Option<PowerOnMode>, effect: TransitionEffect) -> CommandMessage {
        let mut params = Vec::<_>::with_capacity(4);
        params.push({if on { "on" } else { "off" }}.into());
        params.append(&mut effect.into());
        params.push(mode.into());
        CommandMessage { id, method: "set_power", params }
    }

    pub fn new_toggle(id: i32) -> CommandMessage {
        CommandMessage { id, method: "toggle", params: vec![] }
    }

    pub fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<String, Box<error::Error+Send+Sync>> {
        let command = serde_json::to_string(&self).map(|v| v+"\r\n")?;
        let command_bytes = ASCII.encode(&command, EncoderTrap::Strict).map_err(|x| x.into_owned())?;

        stream.write_all(&command_bytes)?;
        println!("Sent: {:?}", command_bytes);

        let mut response = [0; 1024];
        stream.read(&mut response[..])?;

        Ok(ASCII.decode(&response, DecoderTrap::Strict)?)
    }
}

