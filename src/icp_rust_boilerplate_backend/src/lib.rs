#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};


type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct MobileDevice {
    id: u64,
    model: String,
    brand: String,
    specifications: String,
    price: f64,
    quantity: i32,
    ram: String,
    rom: String,
    battery_power: String,
    camera_quality: String, // Added camera_quality field
    created_at: u64,
    updated_at: Option<u64>,
}

// Implementing Storable and BoundedStorable traits for MobileDevice
impl Storable for MobileDevice {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for MobileDevice {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// ... (existing thread-local variables and payload structure)

// New thread-local variables for our Mobile Store Management app

thread_local! {
    static MOBILE_MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static MOBILE_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MOBILE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter for mobile devices")
    );

    static MOBILE_STORAGE: RefCell<StableBTreeMap<u64, MobileDevice, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MOBILE_MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

// Helper method to perform insert for MobileDevice
fn do_insert_mobile_device(device: &MobileDevice) {
    MOBILE_STORAGE.with(|service| service.borrow_mut().insert(device.id, device.clone()));
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct MobileDevicePayload {
    model: String,
    brand: String,
    specifications: String,
    price: f64,
    quantity: i32,
    ram: String,
    rom: String,
    battery_power: String,
    camera_quality: String, // Added camera_quality field
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct MobilePurchasePayload {
    device_id: u64,
    quantity: i32,
}

// 2.7.1 get_mobile_device Function:
#[ic_cdk::query]
fn get_mobile_device(id: u64) -> Result<MobileDevice, Error> {
    match _get_mobile_device(&id) {
        Some(device) => Ok(device),
        None => Err(Error::NotFound {
            msg: format!("a mobile device with id={} not found", id),
        }),
    }
}

// 2.7.2 _get_mobile_device Function:
fn _get_mobile_device(id: &u64) -> Option<MobileDevice> {
    MOBILE_STORAGE.with(|s| s.borrow().get(id))
}

// 2.7.3 add_mobile_device Function:
#[ic_cdk::update]
fn add_mobile_device(payload: MobileDevicePayload) -> Option<MobileDevice> {
    let id = MOBILE_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter for mobile devices");

    let timestamp = time();

    let mobile_device = MobileDevice {
        id,
        model: payload.model,
        brand: payload.brand,
        specifications: payload.specifications,
        price: payload.price,
        quantity: payload.quantity,
        ram: payload.ram,
        rom: payload.rom,
        battery_power: payload.battery_power,
        camera_quality: payload.camera_quality, // Added camera_quality field
        created_at: timestamp,
        updated_at: None,
    };

    do_insert_mobile_device(&mobile_device);
    Some(mobile_device)
}

// 2.7.4 update_mobile_device Function:
#[ic_cdk::update]
fn update_mobile_device(id: u64, payload: MobileDevicePayload) -> Result<MobileDevice, Error> {
    match MOBILE_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut mobile_device) => {
            mobile_device.model = payload.model;
            mobile_device.brand = payload.brand;
            mobile_device.specifications = payload.specifications;
            mobile_device.price = payload.price;
            mobile_device.quantity = payload.quantity;
            mobile_device.ram = payload.ram;
            mobile_device.rom = payload.rom;
            mobile_device.battery_power = payload.battery_power;
            mobile_device.camera_quality = payload.camera_quality; // Added camera_quality field
            mobile_device.updated_at = Some(time());
            do_insert_mobile_device(&mobile_device);
            Ok(mobile_device)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a mobile device with id={}. device not found",
                id
            ),
        }),
    }
}

// 2.7.5 delete_mobile_device Function:
#[ic_cdk::update]
fn delete_mobile_device(id: u64) -> Result<MobileDevice, Error> {
    match MOBILE_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(mobile_device) => Ok(mobile_device),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a mobile device with id={}. device not found.",
                id
            ),
        }),
    }
}

// 2.7.6 get_all_mobile_devices Function:
#[ic_cdk::query]
fn get_all_mobile_devices() -> Vec<MobileDevice> {
    MOBILE_STORAGE.with(|service| {
        let storage = service.borrow_mut();
        storage.iter().map(|(_, item)| item.clone()).collect()
    })
}

#[ic_cdk::query]
fn search_mobile_devices_by_price_range(min_price: f64, max_price: f64) -> Vec<MobileDevice> {
    MOBILE_STORAGE.with(|service| {
        let borrow = service.borrow();
        borrow
            .iter()
            .filter_map(|(_, device)| {
                if device.price >= min_price && device.price <= max_price {
                    Some(device.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}

#[ic_cdk::query]
fn search_mobile_devices_by_quantity(quantity: i32) -> Vec<MobileDevice> {
    MOBILE_STORAGE.with(|service| {
        let borrow = service.borrow();
        borrow
            .iter()
            .filter_map(|(_, device)| {
                if device.quantity == quantity {
                    Some(device.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}


#[ic_cdk::query]
fn search_mobile_devices_by_battery_power(battery_power: String) -> Vec<MobileDevice> {
    MOBILE_STORAGE.with(|service| {
        let borrow = service.borrow();
        borrow
            .iter()
            .filter_map(|(_, device)| {
                if device.battery_power == battery_power {
                    Some(device.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}

#[ic_cdk::query]
fn search_mobile_devices_by_ram_size(ram_size: String) -> Vec<MobileDevice> {
    MOBILE_STORAGE.with(|service| {
        let borrow = service.borrow();
        borrow
            .iter()
            .filter_map(|(_, device)| {
                if device.ram == ram_size {
                    Some(device.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}


#[ic_cdk::query]
fn get_mobile_devices_by_brand(brand: String) -> Vec<MobileDevice> {
    MOBILE_STORAGE.with(|service| {
        let borrow = service.borrow();
        borrow
            .iter()
            .filter_map(|(_, device)| {
                if device.brand == brand {
                    Some(device.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}

#[ic_cdk::query]
fn get_mobile_devices_by_camera_quality(camera_quality: String) -> Vec<MobileDevice> {
    MOBILE_STORAGE.with(|service| {
        let borrow = service.borrow();
        borrow
            .iter()
            .filter_map(|(_, device)| {
                if device.camera_quality == camera_quality {
                    Some(device.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}




// 2.7.7 enum Error:
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InsufficientStock { msg: String },
}

// To generate the Candid interface definitions for our canister
ic_cdk::export_candid!();
