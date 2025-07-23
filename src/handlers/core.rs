use crate::models::core::{EquipmentTypes, ModeGroups, Modes, State, StateGroup};
use std::sync::{Arc, Mutex};

pub type EquipmentTypeStore = Arc<Mutex<Vec<EquipmentTypes>>>;
pub type ModeGroupStore = Arc<Mutex<Vec<ModeGroups>>>;
pub type ModesStore = Arc<Mutex<Vec<Modes>>>;
pub type StateGroupStore = Arc<Mutex<Vec<StateGroup>>>;
pub type StateStore = Arc<Mutex<Vec<State>>>;

// eq types crud and more?
impl EquipmentTypes {
    // create
    pub fn create(store: &EquipmentTypeStore, type_name: String) -> Result<EquipmentTypes, String> {
        let mut data = store.lock().map_err(|_| "Failed to acquire lock")?;

        // generating mock next id (simple auto-increment)
        let next_id = data.iter().map(|item| item.type_id).max().unwrap_or(0) + 1;

        let new_equipment = EquipmentTypes {
            type_id: next_id,
            type_name,
        };

        data.push(new_equipment.clone());
        Ok(new_equipment)
    }

    // read
    pub fn get_by_id(
        store: &EquipmentTypeStore,
        type_id: i8,
    ) -> Result<Option<EquipmentTypes>, String> {
        let data = store.lock().map_err(|_| "Failed to acquire lock")?;
        Ok(data.iter().find(|item| item.type_id == type_id).cloned())
    }

    // read all
    pub fn get_all(store: &EquipmentTypeStore) -> Result<Vec<EquipmentTypes>, String> {
        let data = store.lock().map_err(|_| "Failed to acquire lock")?;
        Ok(data.clone())
    }

    // update
    pub fn update(
        store: &EquipmentTypeStore,
        type_id: i8,
        new_name: String,
    ) -> Result<Option<EquipmentTypes>, String> {
        let mut data = store.lock().map_err(|_| "Failed to acquire lock")?;

        if let Some(item) = data.iter_mut().find(|item| item.type_id == type_id) {
            item.type_name = new_name;
            Ok(Some(item.clone()))
        } else {
            Ok(None)
        }
    }

    // delete
    pub fn delete(store: &EquipmentTypeStore, type_id: i8) -> Result<bool, String> {
        let mut data = store.lock().map_err(|_| "Failed to acquire lock")?;

        if let Some(pos) = data.iter().position(|item| item.type_id == type_id) {
            data.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl ModeGroups {
    // create
    // read
    // update
    // delete
}

impl Modes {
    // create
    // read
    // update
    // delete
}

pub fn init_equipment_type_store() -> EquipmentTypeStore {
    let initial_data = vec![
        EquipmentTypes {
            type_id: 1,
            type_name: "Enterprise".to_string(),
        },
        EquipmentTypes {
            type_id: 2,
            type_name: "Site".to_string(),
        },
        EquipmentTypes {
            type_id: 3,
            type_name: "Area".to_string(),
        },
        EquipmentTypes {
            type_id: 4,
            type_name: "Line".to_string(),
        },
        EquipmentTypes {
            type_id: 5,
            type_name: "Cell".to_string(),
        },
    ];

    Arc::new(Mutex::new(initial_data))
}

pub fn init_mode_group_store() -> ModeGroupStore {
    let initial_data = vec![ModeGroups {
        mode_group_id: 1,
        mode_group_name: "Default Mode Group".to_string(),
        mode_group_description: Some("Default mode group for the MES system.".to_string()),
    }];

    Arc::new(Mutex::new(initial_data))
}

pub fn init_mode_store() -> ModesStore {
    let initial_data = vec![
        Modes {
            mode_id: 1,
            mode_group_id: 1,
            mode_code: 0,
            mode_description: Some("Disabled".to_string()),
        },
        Modes {
            mode_id: 2,
            mode_group_id: 1,
            mode_code: 1,
            mode_description: Some("Running".to_string()),
        },
        Modes {
            mode_id: 3,
            mode_group_id: 1,
            mode_code: 2,
            mode_description: Some("Change Over".to_string()),
        },
        Modes {
            mode_id: 3,
            mode_group_id: 1,
            mode_code: 4,
            mode_description: Some("Idle".to_string()),
        },
        Modes {
            mode_id: 4,
            mode_group_id: 1,
            mode_code: 5,
            mode_description: Some("E-Stop".to_string()),
        },
        Modes {
            mode_id: 5,
            mode_group_id: 1,
            mode_code: 6,
            mode_description: Some("Blocked".to_string()),
        },
        Modes {
            mode_id: 6,
            mode_group_id: 1,
            mode_code: 7,
            mode_description: Some("Starved".to_string()),
        },
        Modes {
            mode_id: 7,
            mode_group_id: 1,
            mode_code: 8,
            mode_description: Some("Planned Downtime".to_string()),
        },
        Modes {
            mode_id: 8,
            mode_group_id: 1,
            mode_code: 9,
            mode_description: Some("Unplanned Downtime".to_string()),
        },
        Modes {
            mode_id: 9,
            mode_group_id: 1,
            mode_code: 10,
            mode_description: Some("User Planned Downtime".to_string()),
        },
        Modes {
            mode_id: 10,
            mode_group_id: 1,
            mode_code: 11,
            mode_description: Some("User Unplanned Downtime".to_string()),
        },
    ];

    Arc::new(Mutex::new(initial_data))
}

pub fn init_state_groups_store() -> StateGroupStore {
    let initial_value = vec![StateGroup {
        state_group_id: 1,
        state_group_name: "Default State Group".to_string(),
        state_group_description: Some("Default state group for the MES system.".to_string()),
    }];

    Arc::new(Mutex::new(initial_value))
}

pub fn init_state_store() -> StateStore {
    let inital_value = vec![
        State {
            state_id: 1,
            state_group_id: 1,
            state_code: 0,
            state_description: Some("Disabled".to_string()),
        },
        State {
            state_id: 2,
            state_group_id: 1,
            state_code: 1,
            state_description: Some("Running".to_string()),
        },
        State {
            state_id: 3,
            state_group_id: 1,
            state_code: 2,
            state_description: Some("Change Over".to_string()),
        },
        State {
            state_id: 3,
            state_group_id: 1,
            state_code: 4,
            state_description: Some("Idle".to_string()),
        },
        State {
            state_id: 4,
            state_group_id: 1,
            state_code: 5,
            state_description: Some("E-Stop".to_string()),
        },
        State {
            state_id: 5,
            state_group_id: 1,
            state_code: 6,
            state_description: Some("Blocked".to_string()),
        },
        State {
            state_id: 6,
            state_group_id: 1,
            state_code: 7,
            state_description: Some("Starved".to_string()),
        },
        State {
            state_id: 7,
            state_group_id: 1,
            state_code: 8,
            state_description: Some("Planned Downtime".to_string()),
        },
        State {
            state_id: 8,
            state_group_id: 1,
            state_code: 9,
            state_description: Some("Unplanned Downtime".to_string()),
        },
        State {
            state_id: 9,
            state_group_id: 1,
            state_code: 10,
            state_description: Some("User Planned Downtime".to_string()),
        },
        State {
            state_id: 10,
            state_group_id: 1,
            state_code: 11,
            state_description: Some("User Unplanned Downtime".to_string()),
        },
    ];

    Arc::new(Mutex::new(inital_value))
}
