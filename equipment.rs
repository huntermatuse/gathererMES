use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
struct EquipmentTypes {
    type_id: i8,
    type_name: String, 
}

#[derive(Serialize, Deserialize)]
struct EquipmentMetadata {
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct Equipment {
    equipment_id: i32,
    equipment_name: String, 
    equipment_type_id: i8,
    equipment_parent_id: Option<i32>, // Equipment.equipment_id
    equipment_enabled: bool,
    equipment_metadata: Option<EquipmentMetadata>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>
}

#[derive(Serialize, Deserialize)]
struct EquipmentModeGroupMapping {
    equipment_id: i32,
    mode_group_id: i32,
}

#[derive(Serialize, Deserialize)]
struct ModeGroups {
    mode_group_id: i32,
    mode_group_name: String,
    mode_group_description: String,
}

#[derive(Serialize, Deserialize)]
struct Modes {
    mode_id: i32,
    mode_group_id: i32,
    mode_code: i32,
    mode_description: String, 
}

#[derive(Serialize, Deserialize)]
struct EquipmentStateGroupMapping {
    equipment_id: i32,
    state_group_id: i8,
}

#[derive(Serialize, Deserialize)]
struct StateGroup {
    state_group_id: i32,
    state_group_name: String, 
    state_group_description: String,
}

#[derive(Serialize, Deserialize)]
struct State {
    state_id: i32,
    state_group_id: i8,
    state_name: String, 
    state_description: String, 
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