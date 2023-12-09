use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;
use log::{error, info};
use rumqttc::v5::AsyncClient;
use rumqttc::v5::mqttbytes::QoS;
use tokio::runtime;
use crate::device_lock::DeviceLock;
use crate::device_message::DeviceMessage;
use crate::message_enum::MessageEnum;

#[derive(Debug)]
pub(crate) struct GenericDevice {
    pub topic: String,
    //pub state: MessageEnum, // This is the current device state, aka the last message
    pub lock: Arc<RefCell<DeviceLock<MessageEnum>>>,
    pub setup: bool,
}

impl GenericDevice {

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<MessageEnum>>> {
        self.lock.clone()
    }

    fn setup(&mut self, setup: bool) {
        self.setup = setup;
    }

    // better use the attribute directly
    fn get_topic(&self) -> String {
        self.topic.clone()
    }
    fn is_init(&self) -> bool {
        self.setup
    }

    fn init(&mut self, topic : &str, msg: MessageEnum) {
        let new_lock = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();
            if topic == &self.get_topic() {
                info!("✨ Init device [{}], with message <{:?}>",  &self.get_topic().to_uppercase(), &msg);
                self.topic = topic.to_string();
                self.setup(true);
                dev_lock.replace(msg.clone());
                info!("Init done");
            }
            dev_lock
        };
        self.get_lock().replace(new_lock.clone());
    }

    /// Send the message on the right end point (/get) to trigger the device properties on the bus
    fn trigger_info(&self) -> Vec<u8> {
        let lk = self.get_lock();
        let borr = lk.as_ref().borrow();
        let dev_lock = borr.deref().clone();
        dev_lock.last_object_message.query_for_state().as_bytes().to_vec()
    }

    /// Transform a json message to a Message of the same type of &self
    ///  @deprecated - the device knows the last message (state) in the correct format.
    // fn from_json_to_local(&self, msg: &str) -> Result<MessageEnum, String> {
    //     self.state.json_to_local()
    // }


    // Convert any message (origin_message) into a local message type needed by the device
    fn to_local(&self, origin_message : &MessageEnum, last_message: &MessageEnum) -> MessageEnum {
        let lk = self.get_lock();
        let borr = lk.as_ref().borrow();
        let dev_lock = borr.deref().clone();
        dev_lock.last_object_message.to_local(&origin_message, &last_message)
    }

    fn allowed_to_process(&self, object_message: &MessageEnum) -> (bool, bool) {
        let lk = self.get_lock();
        let borr = lk.as_ref().borrow();
        let dev_lock = borr.deref().clone();

        // let incoming_message = object_message.to_json().unwrap();
        let incoming_message = object_message.raw_message();
        let is_locked = dev_lock.count_locks > 0;
        let is_same = *incoming_message == dev_lock.last_object_message;
        (is_locked, is_same)
    }

    ///
    /// Specific processing for the device that emits the message
    ///
    fn process(&self,  _original_message : &MessageEnum) {
        // Nothing by defaut
        info!("Default empty process for device {}.", & self.get_topic());
    }

    ///
    /// Run the local specific processing if allowed.
    ///
    fn process_and_continue(&self, original_message : &MessageEnum) -> bool {

        info!("process_and_continue");
        let (new_lock, allowed) = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();
            let allowed: bool;
            match self.allowed_to_process(&original_message) {
                (true, _) => {
                    info!("❌ Device {} is locked.", & self.get_topic().to_uppercase());
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
            //let json_message= original_message.raw_message();
            dev_lock.replace(original_message.clone());
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

            // Convert the incoming message to the format the device needs.
            // Last message est du même format que le message du device.
            // Il permet de récupérer certaines informations.
            // Ex : Incoming inter dim message + last (LampRGB) ---> hall_lamp message (LampRGB)

            // In Generic Mode it's much simplier, we have the last message in the correct format.
            let last_message = &self.state;

            // let last_message = match self.from_json_to_local(&dev_lock.last_object_message)  {
            //     Err(e) => {
            //         error!("💀 Cannot parse the message for device {}, message=<{}>, \n e={}", &self.get_topic().to_uppercase(), &dev_lock.last_object_message, e);
            //         return;
            //     }
            //     Ok(lm) => lm
            // };

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
            let json_message = object_message.raw_message();
            self.state = object_message.clone(); // TODO use the object into the Locker instead
            dev_lock.replace(json_message);

            let message_locked = &dev_lock.last_object_message;
            info!("Now last : {:?}", &message_locked);
            dev_lock
        };
        self.get_lock().replace(new_lock);
    }

    fn publish_message(&self, mut client: &mut AsyncClient, object_message : &MessageEnum) {

        let message = object_message.raw_message();
        let data = message.as_bytes().to_vec();
        client.publish(&format!("{}/set", &self.get_topic()), QoS::AtLeastOnce, false, data).await.unwrap(); // TODO unwrap handle

        // match object_message.to_json() {
        //     Ok(message) => {
        //         info!("➡ Prepare to be sent to the {}, {:?} ", &self.get_topic().to_uppercase(), &message);
        //         // Créer un runtime asynchrone avec tokio
        //         let rt = runtime::Runtime::new().unwrap();
        //         // Appeler la fonction asynchrone à l'intérieur du runtime
        //         rt.block_on(async {
        //             let data = message.as_bytes().to_vec();
        //             client.publish(&format!("{}/set", &self.get_topic()), QoS::AtLeastOnce, false, data).await.unwrap(); // TODO unwrap handle
        //         });
        //     }
        //     Err(e) => {
        //         error!("💣 Impossible to parse the message : e={:?}", e);
        //     }
        // }
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

}
