use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use log::info;
use rumqttc::v5::{AsyncClient, Event, Incoming};
use rumqttc::v5::EventLoop;
use rumqttc::v5::mqttbytes::QoS;

use crate::dyn_device::DynDevice;
use crate::hall_lamp::HALL_LAMP;
use crate::kitchen_lamp::KITCHEN_LAMP;

pub (crate) fn build_init_list(device_repo : &HashMap<String, Arc<RefCell<dyn DynDevice>>>) -> Vec<Arc<RefCell<dyn DynDevice>>> {
    vec![
        device_repo.get(KITCHEN_LAMP).unwrap().clone(),
        device_repo.get(HALL_LAMP).unwrap().clone()
    ]
}

///
/// Send an information message for all the device we want to init
/// Read the responses from mosquitto and run the init routine for the devices.
///
pub (crate) async fn process_initialization_message(mut client: &mut AsyncClient, mut eventloop: &mut EventLoop, device_to_init: &Vec<Arc<RefCell<dyn DynDevice>>>) -> Result<(), String> {

    info!("Initialisation stage starts");

    if !device_to_init.is_empty() {
        for dev in device_to_init {
            let borr = dev.as_ref().borrow();
            let dd = borr.deref().clone();

            dbg!("Topic", &dd.get_topic());
            let data = dd.trigger_info();
            client.publish(&format!("{}/get", &dd.get_topic()), QoS::AtLeastOnce, false,  data).await.unwrap(); // TODO
        }

        while let Ok(notification) = eventloop.poll().await {
            let mut end_loop = true;
            handle_event(notification, device_to_init).await;
            for dev in device_to_init {
                info!("Devices before check ----------");
                let borr = dev.as_ref().borrow();
                let dd = borr.deref().clone();
                if !dd.is_init() {
                    end_loop = false;
                }
            }
            if end_loop {
                break;
            }
        }
    } // device is empty

    info!("Initialisation stage finished");

    Ok(())
}

async fn handle_event(event: Event, device_to_init: &Vec<Arc<RefCell<dyn DynDevice>>>) {
    println!("Message reçu = {:?}", &event);
    match event {
        Event::Incoming(Incoming::Publish(publish)) => {
            // Votre logique de traitement des messages ici

            let msg = std::str::from_utf8(&publish.payload).unwrap();
            let topic = std::str::from_utf8(publish.topic.as_ref()).unwrap(); // TODO

            if let Some(properties) = publish.properties {
                // Vous pouvez accéder à différentes propriétés ici
                println!(">>>> Propriétés de la réponse de publish: {:?}", properties);
            } else {
                println!("No properties on publish")
            }

            println!( "Message reçu sur le topic {:?}: {:?}",topic  , msg    );

            info!("PUBLISH ({}): {}", topic, msg);

            // TODO is it necessary to loop over all the devices ?
            for dev in device_to_init {
                let mut borr = dev.as_ref().borrow_mut();
                let dd = borr.deref_mut();
                dd.init(topic, msg);
            }

        }
        Event::Incoming(Incoming::ConnAck(connack)) => {
            // Accéder aux métadonnées de la réponse de connexion (Connack)
            if let Some(properties) = connack.properties {
                // Vous pouvez accéder à différentes propriétés ici
                println!("Propriétés de la réponse de connexion: {:?}", properties);
            }
        }
        Event::Incoming(Incoming::PubAck(pubAck)) => {
            // Accéder aux métadonnées de la réponse de connexion (Connack)
            if let Some(properties) = pubAck.properties {
                // Vous pouvez accéder à différentes propriétés ici
                println!("Propriétés de la réponse de connexion: {:?}", properties);
            } else {
                println!("No properties on puback")
            }
        }
        _ => {}
    }
}