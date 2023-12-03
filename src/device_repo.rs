use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use log::info;
use crate::dyn_device::DynDevice;
use crate::hall_lamp::{HALL_LAMP, HallLampDevice};
use crate::kitchen_inter_dim::{KITCHEN_INTER_DIM, KitchenInterDimDevice};
use crate::kitchen_lamp::{KITCHEN_LAMP, KitchenLampDevice};
use crate::kitchen_switch::{KITCHEN_SWITCH, KitchenSwitchDevice};

pub (crate) fn build_device_repo() -> HashMap<String, Arc<RefCell<dyn DynDevice>>> {
    info!("Inside the Repo Builder");
    let mut device_repo : HashMap<String, Arc<RefCell<dyn DynDevice>>> = HashMap::new();
    device_repo.insert(KITCHEN_SWITCH.to_owned(), Arc::new(RefCell::new(KitchenSwitchDevice::new())));
    device_repo.insert(KITCHEN_INTER_DIM.to_owned(), Arc::new(RefCell::new(KitchenInterDimDevice::new())));
    device_repo.insert(KITCHEN_LAMP.to_owned(), Arc::new(RefCell::new(KitchenLampDevice::new())));
    device_repo.insert(HALL_LAMP.to_owned(), Arc::new(RefCell::new(HallLampDevice::new())));
    //device_repo.insert(TEMP_BAIE_VITREE.to_owned(), Arc::new(RefCell::new(InsideTempSensorDevice::new())));
    // device_repo.insert(TEMP_MEUBLE_TV.to_owned(), Arc::new(RefCell::new(OutdoorTempSensorDevice::new())));
    device_repo
}

pub (crate) fn device_to_listen(device_repo: &HashMap<String, Arc<RefCell<dyn DynDevice>>>) -> Vec<Arc<RefCell<dyn DynDevice>>> {
    vec![
        device_repo.get(KITCHEN_INTER_DIM).unwrap().clone(),
        device_repo.get(KITCHEN_LAMP).unwrap().clone(),
        device_repo.get(HALL_LAMP).unwrap().clone(),
        //device_repo.get(TEMP_BAIE_VITREE).unwrap().clone(),
        // device_repo.get(TEMP_MEUBLE_TV).unwrap().clone()
        device_repo.get(KITCHEN_SWITCH).unwrap().clone(),
    ]
}