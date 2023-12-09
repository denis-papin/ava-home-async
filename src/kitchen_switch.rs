use std::cell::RefCell;
use std::sync::Arc;

use log::info;

use crate::device_lock::DeviceLock;
use crate::device_message::{DeviceMessage, InterSwitch};
use crate::dyn_device::DynDevice;

pub (crate) const KITCHEN_SWITCH : &str = "kitchen_switch";

#[derive(Debug)]
pub (crate) struct KitchenSwitchDevice {
    pub setup : bool,
    pub lock : Arc<RefCell<DeviceLock<String>>>
}

impl KitchenSwitchDevice {
    pub(crate) fn new() -> Self {
        info!("🌟🌟🌟🌟🌟 NEW KitchenSwitchDevice");
        let dl = DeviceLock::new( String::new());
        Self {setup: false,lock : Arc::new(RefCell::new( dl )) }
    }

    pub fn get_name() -> &'static str {
        KITCHEN_SWITCH
    }
}

impl DynDevice for KitchenSwitchDevice {

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<String>>> {
        self.lock.clone()
    }

    fn setup(&mut self, _setup: bool) {
        // Nothing to do
    }

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        self.setup
    }

    fn trigger_info(&self) -> Vec<u8> {
        let msg = r#"{"state":""}"#;
        msg.as_bytes().to_vec()
        // client.publish(&format!("{}/get", &self.get_topic()), QoS::AtLeastOnce, false,  octets).await.unwrap();
        // publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"state":""}"#);
    }

    fn from_json_to_local(&self, msg: &str) -> Result<Box<dyn DeviceMessage>, String> {
        Ok(Box::new( InterSwitch::from_json(msg)? ))
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, _last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        origin_message.to_inter_switch()
    }

}
