use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct EquipmentTypes {
    pub type_id: i8,
    pub type_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct EquipmentMetadata {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Equipment {
    pub equipment_id: i32,
    pub equipment_name: String,
    pub equipment_type_id: i8,
    pub equipment_parent_id: Option<i32>, // Optional << Equipment.equipment_id >>
    pub equipment_enabled: bool,
    pub equipment_metadata: Option<EquipmentMetadata>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct EquipmentModeGroupMapping {
    pub equipment_id: i32,
    pub mode_group_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ModeGroups {
    pub mode_group_id: i32,
    pub mode_group_name: String,
    pub mode_group_description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Modes {
    pub mode_id: i32,
    pub mode_group_id: i32,
    pub mode_description: String,
}

#[derive(Serialize, Deserialize)]
pub struct EquipmentStateGroupMapping {
    pub equipment_id: i32,
    pub state_group_id: i8,
}

#[derive(Serialize, Deserialize)]
pub struct StateGroup {
    pub state_group_id: i32,
    pub state_group_name: String,
    pub state_group_description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub state_id: i32,
    pub state_group_id: i32,
    pub state_code: i32,
    pub state_description: Option<String>,
    // pub state_mode_map: i32, // we will sort this out later
}

#[derive(Serialize, Deserialize)]
struct EquipmentWithRelations {
    equipment_id: i32,
    equipment_name: String,
    equipment_type_id: i8, // 1=Enterprise, 2=Site, 3=Area, 4=Line, 5=Cell
    equipment_parent_id: Option<i32>,
    equipment_enabled: bool,
    equipment_metadata: EquipmentMetadata,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,

    // Related data
    equipment_type: EquipmentTypes,
    child_equipment: Vec<Equipment>,

    // Only present for Line level and above (equipment_type_id <= 4)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    mode_groups: Vec<ModeGroupWithModes>,

    // Only present for Cell level (equipment_type_id == 5)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    state_groups: Vec<StateGroupWithStates>,
}

#[derive(Serialize, Deserialize)]
struct ModeGroupWithModes {
    mode_group_id: i32,
    mode_group_name: String,
    mode_group_description: String,
    modes: Vec<Modes>,
}

#[derive(Serialize, Deserialize)]
struct StateGroupWithStates {
    state_group_id: i32,
    state_group_name: String,
    state_group_description: String,
    states: Vec<State>,
}

#[derive(Serialize, Deserialize)]
struct CurrentEquipmentMode {
    equipment_id: i32,
    mode_id: i32,
    set_at: DateTime<Utc>,
    set_by: Option<String>, // user who set it
}

#[derive(Serialize, Deserialize)]
struct CurrentEquipmentState {
    equipment_id: i32,
    state_id: i32,
    state_value: Option<Value>, // actual state data/value
    updated_at: DateTime<Utc>,
    updated_by: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct EquipmentPath {
    equipment_id: i32,
    path: Vec<Equipment>, // from root to current equipment
    depth: i32,
}

// theorectical eq path finder
/*
impl EquipmentPath {
    pub fn get_enterprise(&self) -> Option<&Equipment> {
        self.path.iter().find(|e| e.equipment_type_id == 1)
    }

    pub fn get_site(&self) -> Option<&Equipment> {
        self.path.iter().find(|e| e.equipment_type_id == 2)
    }

    pub fn get_area(&self) -> Option<&Equipment> {
        self.path.iter().find(|e| e.equipment_type_id == 3)
    }

    pub fn get_line(&self) -> Option<&Equipment> {
        self.path.iter().find(|e| e.equipment_type_id == 4)
    }

    pub fn get_cell(&self) -> Option<&Equipment> {
        self.path.iter().find(|e| e.equipment_type_id == 5)
    }

    pub fn get_parent(&self) -> Option<&Equipment> {
        if self.path.len() > 1 {
            self.path.get(self.path.len() - 2)
        } else {
            None
        }
    }
}
*/
