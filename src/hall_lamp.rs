use std::cell::RefCell;
use std::sync::Arc;

use log::info;

use crate::device_lock::DeviceLock;
use crate::device_message::{DeviceMessage, LampRGB};
use crate::dyn_device::DynDevice;

pub(crate) const HALL_LAMP : &str = "hall_lamp";

#[derive(Debug)]
pub(crate) struct HallLampDevice {
    pub lock : Arc<RefCell<DeviceLock<String>>>,
    pub setup : bool,
}

// TODO generalise the struct to handle all the "Lamp" family, pass the name in the constructor.
impl HallLampDevice {
    pub(crate) fn new() -> Self {
        info!("🌟🌟🌟🌟🌟 NEW HallLampDevice");
        let dl = DeviceLock::new( String::new());
        Self {
            lock : Arc::new(RefCell::new( dl )),
            setup: false,
        }
    }
    pub fn get_name() -> &'static str {
        HALL_LAMP
    }
}

impl DynDevice for HallLampDevice {

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<String>>> {
        self.lock.clone()
    }

    fn setup(&mut self, setup: bool) {
        self.setup = setup;
    }

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        self.setup
    }

    fn from_json_to_local(&self, msg: &str) -> Result<Box<dyn DeviceMessage>, String> {
        Ok(Box::new( LampRGB::from_json(msg)? ))
    }

    fn trigger_info(&self) -> Vec<u8> {
        let msg = r#"{"color":{"x":"","y":""}}"#;
        msg.as_bytes().to_vec()
        // client.publish(&format!("{}/get", &self.get_topic()), QoS::AtLeastOnce, false,  octets).await.unwrap();
        // publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), );
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        info!("HallLamp tries to build its LambRGB message");
        origin_message.to_lamp_rgb(last_message)
    }

}
