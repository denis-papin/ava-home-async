use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;

use log::{error, info};
use rumqttc::v5::AsyncClient;
use rumqttc::v5::mqttbytes::QoS;
use tokio::runtime;

use crate::device_lock::DeviceLock;
use crate::device_message::DeviceMessage;

///
pub (crate) trait DynDevice {

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<String>>> {
        todo!()
    }

    fn setup(&mut self, _setup: bool) {
        todo!()
    }

    fn get_topic(&self) -> String;
    fn is_init(&self) -> bool;

    fn init(&mut self, topic : &str, msg : &str) {
        let new_lock = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();
            if topic == &self.get_topic() {
                info!("✨ Init device [{}], with message <{}>",  &self.get_topic().to_uppercase(), &msg);
                self.setup(true);
                dev_lock.replace(msg.to_string());
                info!("Init done");
            }
            dev_lock
        };
        self.get_lock().replace(new_lock.clone());
    }

    /// Send the message on the right end point (/get) to trigger the device properties on the bus
    fn trigger_info(&self) -> Vec<u8>;

    fn from_json_to_local(&self, msg: &str) -> Result<Box<dyn DeviceMessage>, String>;


    fn allowed_to_process(&self, object_message : &Box<dyn DeviceMessage>) -> (bool, bool) {
        let lk = self.get_lock();
        let borr = lk.as_ref().borrow();
        let dev_lock = borr.deref().clone();

        let incoming_message = object_message.to_json().unwrap();
        let is_locked = dev_lock.count_locks > 0;
        let is_same = *incoming_message == dev_lock.last_object_message;
        (is_locked, is_same)
    }

    ///
    /// Specific processing for the device that emits the message
    ///
    fn process(&self,  _original_message : &Box<dyn DeviceMessage>) {
        // Nothing by defaut
        info!("Default empty process for device {}.", & self.get_topic());
    }

    ///
    /// Run the local specific processing if allowed.
    ///
    fn process_and_continue(&self, original_message : &Box<dyn DeviceMessage>) -> bool {

        info!("process_and_continue");
        let (new_lock, allowed) = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();
            let allowed: bool;
            match self.allowed_to_process(&original_message) {
                (true, _) => {
                    info!("❌ Device {} is locked.", & self.get_topic().to_uppercase());
                    // self.unlock(&mut locks);
                    dev_lock.dec();
                    allowed = false;
                }
                (false, true) => {
                    info!("❌ Device {}, same message.", & self.get_topic().to_uppercase());
                    allowed = false;
                }
                (false, false) => {
                    info!("👍 Device {}, allowed to process the message.", & self.get_topic().to_uppercase());
                    self.process(&original_message);
                    allowed = true;
                }
            }
            let json_message = original_message.to_json().unwrap().clone();
            dev_lock.replace(json_message);
            (dev_lock, allowed)
        };
        self.get_lock().replace(new_lock);
        allowed
    }

    ///
    /// Make the device consume the current message
    ///
    fn consume_message(&self, original_message : &Box<dyn DeviceMessage>, mut client: &mut AsyncClient) {
        info!("The device is consuming the message");
        let new_lock = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();

            info!("Execute device {}", & self.get_topic().to_uppercase());

            // Convert the incoming message to the format the device needs. Last message est du même format que le message du device. Il permet de récupérer certaines informations.
            // Ex : Incoming inter dim message + last (LampRGB) ---> hall_lamp message (LampRGB)
            let last_message = match self.from_json_to_local(&dev_lock.last_object_message)  {
                Err(e) => {
                    error!("💀 Cannot parse the message for device {}, message=<{}>, \n e={}", &self.get_topic().to_uppercase(), &dev_lock.last_object_message, e);
                    return;
                }
                Ok(lm) => lm
            };
            let object_message = self.to_local(&original_message, &last_message);

            match self.allowed_to_process(&object_message) {
                (true, _) => {
                    info!("⛔ Device {} is locked.", & self.get_topic().to_uppercase());
                    info!("Incoming message : {:?}, last message : {:?}", &object_message.to_json(), &dev_lock.last_object_message);
                    dev_lock.dec();
                    // self.unlock(&mut locks);
                }
                (false, true) => {
                    info!("⛔ Device {}, same message.", & self.get_topic().to_uppercase());
                    info!("Incoming message : {:?}, last message : {:?}", &object_message.to_json(), &dev_lock.last_object_message);
                }
                (false, false) => {
                    info!("🍺 Device {}, process the message.", & self.get_topic().to_uppercase());
                    info!("Incoming message : {:?}, last message : {:?}", &object_message.to_json(), &dev_lock.last_object_message);
                    dev_lock.inc();
                    self.publish_message(&mut client, &object_message);
                }
            }
            let json_message = object_message.to_json().unwrap().clone();
            dev_lock.replace(json_message);

            let message_locked = &dev_lock.last_object_message;
            info!("Now last : {:?}", &message_locked);
            dev_lock
        };
        self.get_lock().replace(new_lock);
    }

    fn publish_message(&self, mut client: &mut AsyncClient, object_message : &Box<dyn DeviceMessage>) {
        match object_message.to_json() {
            Ok(message) => {
                info!("➡ Prepare to be sent to the {}, {:?} ", &self.get_topic().to_uppercase(), &message);
                // Créer un runtime asynchrone avec tokio
                let rt = runtime::Runtime::new().unwrap();
                // Appeler la fonction asynchrone à l'intérieur du runtime
                rt.block_on(async {
                    let data = message.as_bytes().to_vec();
                    client.publish(&format!("{}/set", &self.get_topic()), QoS::AtLeastOnce, false, data).await.unwrap(); // TODO unwrap handle
                });
            }
            Err(e) => {
                error!("💣 Impossible to parse the message : e={:?}", e);
            }
        }
    }

    // Could be a method of a receiver trait
    // fn receive(&self, mut pub_stream: &mut TcpStream, object_message : Box<dyn DeviceMessage>) {
    //     match object_message.to_json() {
    //         Ok(message) => {
    //             info!("➡ Prepare to be sent to the {}, {:?} ", &self.get_topic().to_uppercase(), &message);
    //             publish(&mut pub_stream, &format!("{}/set", &self.get_topic()), &message);
    //         }
    //         Err(e) => {
    //             error!("💣 Impossible to parse the message : e={:?}", e);
    //         }
    //     }
    // }

    // Convert any message (origin_message) into a local message type needed by the device
    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage>;
}
