use crate::device_message::{InterDim, InterSwitch, LampRGB};
use crate::message_enum::MessageEnum::{INTER_DIMMER, INTER_SWITCH, LAMP_RGB};

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    LAMP_RGB((String, LampRGB)),
    INTER_DIMMER((String, InterDim)),
    INTER_SWITCH((String, InterSwitch)),
}

impl MessageEnum {

    pub (crate) fn query_for_state(&self) -> String {
        match self {
            LAMP_RGB((_, _)) => {
                let msg =  r#"{"color":{"x":"","y":""}}"#;
                msg.to_string()
            }
            INTER_DIMMER((_, _)) => {
                let msg = r#"{"color":{"x":"","y":""}}"#;
                msg.to_string()
            }
            INTER_SWITCH((_, _)) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    pub (crate) fn raw_message(&self) -> String {
        match self {
            LAMP_RGB((msg, _)) => {
                msg.to_owned()
            }
            INTER_DIMMER((msg, _)) => {
                msg.to_owned()
            }
            INTER_SWITCH((msg, _)) => {
                msg.to_owned()
            }
        }
    }
    pub (crate) fn json_to_local(&self) -> Result<MessageEnum, String> {
        match self {
            LAMP_RGB((msg, _)) => {
                Ok(LAMP_RGB((msg.clone(), LampRGB::from_json(msg)?)))
            }
            INTER_DIMMER((msg, _)) => {
                Ok(INTER_DIMMER((msg.clone(), InterDim::from_json(msg)?)))
            }
            INTER_SWITCH((msg, _)) => {
                Ok(INTER_SWITCH((msg.clone(), InterSwitch::from_json(msg)?)))
            }
        }
    }

    /// Convert the original message to the type of the current Self
    pub (crate) fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            LAMP_RGB((_, _)) => {
                original_message.to_lamp_rgb(&last_message)
            }
            INTER_DIMMER((_, _)) => {
                original_message.to_inter_dim(&last_message)
            }
            INTER_SWITCH((_, _)) => {
                original_message.to_inter_switch(&last_message)
            }
        }
    }

    /// Convert the current type of message to LampRGB
    fn to_lamp_rgb(&self, last_message: &MessageEnum) -> Self {
        // We know the "last_message" is of type LAMP_RGB
        let rgb = match last_message {
            LAMP_RGB((_, rgb)) => {
                rgb
            }
            _ => {
                panic!("last message must be of type LAMP_RGB")
            }
        };

        match self {
            LAMP_RGB((msg, o)) => {
                LAMP_RGB((msg.clone(), o.clone()))
            }
            INTER_DIMMER((msg, o)) => {
                LAMP_RGB((msg.clone(), LampRGB {
                    color_temp: rgb.color_temp,
                    brightness: o.brightness,
                    state: o.state.clone(),
                }))
            }
            INTER_SWITCH((msg, o)) => {
                LAMP_RGB((msg.clone(), LampRGB {
                    color_temp: rgb.color_temp,
                    brightness: rgb.brightness,
                    state: o.state.clone(),
                }))
            }
        }
    }

    /// Convert the current type of message to InterSwitch
    fn to_inter_switch(&self, _last_message: &MessageEnum) -> Self {
        match self {
            LAMP_RGB((msg, o)) => {
                INTER_SWITCH((msg.clone(), InterSwitch {
                    state: o.state.clone(),
                }))
            }
            INTER_DIMMER((msg, o)) => {
                INTER_SWITCH((msg.clone(), InterSwitch {
                    state: o.state.clone(),
                }))
            }
            INTER_SWITCH((msg, o)) => {
                INTER_SWITCH((msg.clone(), o.clone()))
            }
        }
    }


    /// Convert the current type of message to InterSwitch
    fn to_inter_dim(&self, last_message: &MessageEnum) -> Self {
        // We know the "last_message" is of type INTER_DIMMER
        let inter = match last_message {
            INTER_DIMMER((_, inter)) => {
                inter
            }
            _ => {
                panic!("last message must be of type LAMP_RGB")
            }
        };

        match self {
            LAMP_RGB((msg, o)) => {
                INTER_DIMMER((msg.clone(), InterDim {
                    brightness: o.brightness,
                    state: o.state.clone(),
                }))
            }
            INTER_DIMMER((msg, o)) => {
                INTER_DIMMER((msg.clone(), o.clone()))
            }
            INTER_SWITCH((msg, o)) => {
                INTER_DIMMER((msg.clone(), InterDim {
                    brightness: inter.brightness,
                    state: o.state.clone(),
                }))
            }
        }
    }

}