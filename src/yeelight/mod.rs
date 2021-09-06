/* Copyright © 2018 Raimundo Martins

   This file is part of CtrlHome

   CtrlHome is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   CtrlHome is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with History.  If not, see <http://www.gnu.org/licenses/>.
*/

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

pub enum PowerOnMode { Normal = 0, CT, RGB, HSV, Flow, Night }

pub enum ColorMode { RGB = 1, Temperature, HSV }

pub enum Property {
    Power(bool),
    Brightness(u8),              // 0 ~ 100
    Temperature(u16),            // 1700 ~ 6500 (K)
    RGB(u32),                    // 1 ~ 16777215
    Hue(u16),                    // 0 ~ 359
    Sat(u8),                     // 0 ~ 100
    ColorMode(ColorMode),
    Flowing(bool),
    DelayOff(u8),                // 0 ~ 60 (minutes): Remaining time of a sleep timer
    FlowParameters(Vec<String>), // ??
    MusicOn(bool),
    Name(String),
}

impl Property {
    fn type_name(&self) -> &'static str {
        match *self {
            Property::Power(_v) => "power",
            Property::Brightness(_v) => "bright",
            Property::Temperature(_v) => "ct",
            Property::RGB(_v) => "rgb",
            Property::Hue(_v) => "hue",
            Property::Sat(_v) => "sat",
            Property::ColorMode(ref _v) => "color_mode",
            Property::Flowing(_v) => "flowing",
            Property::DelayOff(_v) => "delayoff",
            Property::FlowParameters(ref _v) => "flow_params",
            Property::MusicOn(_v) => "music_on",
            Property::Name(ref _v) => "name",
        }
    }
}

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
    pub fn new_get_prop(id: i32, params: &Vec<Property>) -> CommandMessage {
        let params: Vec<_> = params.iter().map(|x| x.type_name().into()).collect();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_toggle() {
        let cmd = CommandMessage::new_toggle(5);
        assert_eq!(serde_json::to_string(&cmd).unwrap(), r#"{"id":5,"method":"toggle","params":[]}"#);
    }

    #[test]
    fn test_rgb() {
        let cmd = CommandMessage::new_set_rgb(7, 50, 20, 10, TransitionEffect::Sudden);
        assert_eq!(serde_json::to_string(&cmd).unwrap(), r#"{"id":7,"method":"set_rgb","params":[3281930,"sudden",0]}"#);
    }
}
