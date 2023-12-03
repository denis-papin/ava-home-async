
use std::cell::{RefCell};
use std::net::TcpStream;

use std::sync::Arc;
use log::info;
use rumqttc::v5::AsyncClient;
use rumqttc::v5::mqttbytes::QoS;
use crate::device_lock;
use crate::device_lock::DeviceLock;
use crate::device_message::{DeviceMessage, LampRGB};
use crate::dyn_device::DynDevice;
use crate::mqtt::publish;


pub (crate) const KITCHEN_LAMP: &str = "kitchen_lamp";

#[derive(Debug)]
pub (crate) struct KitchenLampDevice {
    pub lock : Arc<RefCell<DeviceLock<String>>>,
    pub setup : bool,
}

impl KitchenLampDevice {
    pub(crate) fn new() -> Self {
        info!("🌟🌟🌟🌟🌟 NEW KitchenLampDevice");
        let dl = DeviceLock::new( String::new());
        Self {
            lock : Arc::new(RefCell::new( dl )),
            setup: false,
        }
    }

    pub fn get_name() -> &'static str {
        KITCHEN_LAMP
    }
}


impl DynDevice for KitchenLampDevice {

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

    fn trigger_info(&self) -> Vec<u8> {
        let msg = r#"{"color":{"x":"","y":""}}"#;
        msg.as_bytes().to_vec()
        // client.publish(&format!("{}/get", &self.get_topic()), QoS::AtLeastOnce, false,  octets).await.unwrap();
        // publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"color":{"x":"","y":""}}"#);
    }

    fn from_json_to_local(&self, msg: &str) -> Result<Box<dyn DeviceMessage>, String> {
        Ok(Box::new( LampRGB::from_json(msg)? ))
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        info!("KitchenLamp tries to build its LambRGB message");
        origin_message.to_lamp_rgb(&last_message)
    }
}

