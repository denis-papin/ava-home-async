use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;
use log::{error, info};
use rumqttc::v5::{AsyncClient, Event, EventLoop, Incoming};
use crate::dyn_device::DynDevice;
use crate::loops::HardLoop;


pub (crate) fn find_loops(topic: &str, all_loops: &mut Vec<HardLoop>) -> (Vec<HardLoop>, Option<Arc<RefCell<dyn DynDevice>>>)  {
    let mut eligible_loops : Vec<HardLoop> = vec![];
    let mut output_dev : Option<Arc<RefCell<dyn DynDevice>>> = None;

    for lp in all_loops {
        match lp.find_device_by_topic(topic) {
            None => {}
            Some(dev) => {
                info!("Found topic in [{}] loop, topic=[{}]", & lp.get_name(), topic);
                eligible_loops.push(lp.clone());
                output_dev = Some(dev.clone());
            }
        }

    }
    (eligible_loops, output_dev)
}


///
///
///
pub async fn process_incoming_message(mut client: &mut AsyncClient, mut eventloop: &mut EventLoop, mut all_loops: &mut Vec<HardLoop>)  {
    // let delay = time::Duration::from_millis(10);

    info!(">>> loop 0");

    while let Ok(notification) = eventloop.poll().await {
        info!(">>> loop 1");
        match notification {
            Event::Incoming(Incoming::Publish(publish)) => {
                // Votre logique de traitement des messages ici

                let msg = std::str::from_utf8(&publish.payload).unwrap();
                let topic = std::str::from_utf8(publish.topic.as_ref()).unwrap(); // TODO

                info!("🧶 Publish on topic: [{}], message: <{}>", topic, msg);

                let (loops, opt_device) = find_loops(&topic, &mut all_loops);

                match opt_device {
                    None => {
                        info!("No device to process the message");
                    }
                    Some(dev) => {
                        info!("Receiver device found !");
                        let dd1 = dev.as_ref().borrow();
                        let dd = dd1.deref();
                        for lp in loops {
                            info!("Before Looping");

                            // Change the msg into the DeviceMessage box of the ad hoc device (the original device)
                            let original_message = match dd.from_json_to_local(msg) {
                                Ok(om) => {om}
                                Err(e) => {
                                    error!("💀 Cannot parse the message locally for device {}, msg=<{}>, \n e={}", &dd.get_topic().to_uppercase(), msg, e);
                                    continue
                                }
                            };

                            if dd.process_and_continue(&original_message) {
                                lp.loop_devices(&topic, &original_message, &mut client).await;
                            }
                        }
                    }
                }
            }
            Event::Incoming(Incoming::ConnAck(connack)) => {

            }
            Event::Incoming(Incoming::PubAck(pubAck)) => {

            }
            _ => {}
        }
    }


    // loop {
    //     info!("🔁 New Round");
    //     let packet = match VariablePacket::decode(&mut stream) {
    //         Ok(pk) => pk,
    //         Err(err) => {
    //             error!("Error in receiving packet {:?}", err);
    //             continue;
    //         }
    //     };
    //     info!("PACKET {:?}", packet);
    //
    //     match packet {
    //         VariablePacket::PingrespPacket(..) => {
    //             info!("Receiving PINGRESP from broker ...");
    //         }
    //         VariablePacket::PublishPacket(ref publ) => {
    //             let msg = match str::from_utf8(publ.payload()) {
    //                 Ok(msg) => msg,
    //                 Err(err) => {
    //                     error!("Failed to decode publish message {:?}", err);
    //                     continue;
    //                 }
    //             };
    //             info!("🧶 Publish on topic: [{}], message: <{}>", publ.topic_name(), msg);
    //
    //             let (loops, opt_device) = find_loops(&publ.topic_name(), &mut all_loops);
    //
    //             match opt_device {
    //                 None => {
    //                     info!("No device to process the message");
    //                 }
    //                 Some(dev) => {
    //                     info!("Receiver device found !");
    //                     let dd1 = dev.as_ref().borrow();
    //                     let dd = dd1.deref();
    //                     for lp in loops {
    //                         info!("Before Looping");
    //
    //                         // Change the msg into the DeviceMessage box of the ad hoc device (the original device)
    //                         let original_message = match dd.from_json_to_local(msg) {
    //                             Ok(om) => {om}
    //                             Err(e) => {
    //                                 error!("💀 Cannot parse the message locally for device {}, msg=<{}>, \n e={}", &dd.get_topic().to_uppercase(), msg, e);
    //                                 continue
    //                             }
    //                         };
    //
    //                         if dd.process_and_continue(&original_message) {
    //                             lp.loop_devices(&publ.topic_name(), &original_message, &mut pub_stream);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
        // thread::sleep(delay);
        // thread::yield_now();
    // }
}