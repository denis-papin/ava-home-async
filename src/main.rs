use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use log::info;
use rumqttc::v5::{AsyncClient, MqttOptions};
use rumqttc::v5::mqttbytes::QoS;

use crate::device_repo::{build_device_repo, device_to_listen};
use crate::dyn_device::DynDevice;
use crate::init_loop::{build_init_list, process_initialization_message};
use crate::loops::build_loops;
use crate::processing::process_incoming_message;

mod hall_lamp;
mod kitchen_lamp;
mod kitchen_switch;
mod device_lock;
mod dyn_device;
mod device_message;
mod mqtt;
mod loops;
mod kitchen_inter_dim;
mod device_repo;
mod init_loop;
mod processing;
mod message_enum;
mod generic_device;

const CLIENT_ID: &str = "ava-0.5.0";

#[derive(Debug, Clone)]
pub struct Params {
    pub server_addr : String,
    pub client_id : String,
    pub channel_filters: Vec<(String, QoS)>,
    pub keep_alive :  u16,
}

/// Build the list of channel to listen
fn parse_params(device_repo: &HashMap<String, Arc<RefCell<dyn DynDevice>>>) -> Params {
    let client_id = CLIENT_ID.to_string();

    let mut channel_filters: Vec<(String, QoS)> = vec![];
    for dev in device_to_listen(&device_repo) {
        let dd = dev.as_ref().borrow();
        let topic = dd.get_topic();
        channel_filters.push((topic, QoS::AtMostOnce));
    }

    Params {
        server_addr : "raspberrypi.local".to_string(),
        client_id,
        channel_filters,
        keep_alive : 30_000,
    }
}


#[tokio::main]
async fn main() {

    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    info!("Starting AVA 0.5.0");

    info!("Building the device repository");
    let device_repo = build_device_repo();
    let params = parse_params(&device_repo);

    ///

    let mut mqttoptions = MqttOptions::new(&params.client_id, &params.server_addr, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(params.keep_alive as u64));
    mqttoptions.set_clean_start(true);
    mqttoptions.set_credentials("ava", "avatece3.X");

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    for p in &params.channel_filters {
        info!("Subscribe to [{}]", p.0);
        client.subscribe(p.0.clone(), QoS::AtMostOnce).await.unwrap();
    }

    // task::spawn(async move {
    //     for i in 0..10 {
    //
    //         let msg : &str = "{\"brightness\":200, \"state\":\"ON\"}";
    //         // let toto = String::from("mes données");
    //         let octets = msg.as_bytes().to_vec();
    //
    //         let user_properties = vec![("a".to_string(), "b".to_string())];
    //
    //         let properties = PublishProperties {
    //            // payload_format_indicator: Some(true),  // Indicateur de format de charge utile (UTF-8)
    //             user_properties,
    //             // Ajoutez d'autres propriétés selon vos besoins
    //             ..Default::default()  // Utiliser les valeurs par défaut pour les autres propriétés
    //         };
    //
    //         // Création de la structure Publish avec les propriétés
    //         // let publish = Publish::new("zigbee2mqtt/denis", QoS::AtLeastOnce,
    //         //                            octets, Some(properties));
    //
    //         client.publish_with_properties("zigbee2mqtt/kitchen_lamp/set", QoS::AtLeastOnce, false,
    //                                        octets, properties).await.unwrap();
    //         time::sleep(Duration::from_millis(100)).await;
    //     }
    // });

    let mut init_list = build_init_list(&device_repo);
    let mut all_loops = build_loops(&device_repo);

    match process_initialization_message(&mut client, &mut eventloop, &mut init_list).await {
        Ok(_) => {
            info!("Process incoming messages");
            let _ = process_incoming_message(&mut client, &mut eventloop, &mut all_loops).await;
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
    println!("Done!");
}



