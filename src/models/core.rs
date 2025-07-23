use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct EquipmentType {
    pub type_id: Uuid, 
    pub type_name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EquipmentMetadata {
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Equipment {
    pub equipment_id: Uuid, 
    pub equipment_name: String,
    pub equipment_type_id: Uuid,           
    pub equipment_parent_id: Option<Uuid>,
    pub equipment_enabled: bool,
    pub equipment_metadata: Option<EquipmentMetadata>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EquipmentModeGroupMapping {
    pub equipment_id: Uuid,  
    pub mode_group_id: Uuid, 
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModeGroups {
    pub mode_group_id: Uuid, 
    pub mode_group_name: String,
    pub mode_group_description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Modes {
    pub mode_id: Uuid,       
    pub mode_group_id: Uuid, 
    pub mode_description: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EquipmentStateGroupMapping {
    pub equipment_id: Uuid,   
    pub state_group_id: Uuid, 
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StateGroup {
    pub state_group_id: Uuid, 
    pub state_group_name: String,
    pub state_group_description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    pub state_id: Uuid,       
    pub state_group_id: Uuid, 
    pub state_code: i32,
    pub state_description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EquipmentWithRelations {
    pub equipment_id: Uuid,
    pub equipment_name: String,
    pub equipment_type_id: Uuid,
    pub equipment_parent_id: Option<Uuid>,
    pub equipment_enabled: bool,
    pub equipment_metadata: EquipmentMetadata,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,

    // Related data
    pub equipment_type: EquipmentType,
    pub child_equipment: Vec<Equipment>,

    // Only present for certain equipment types
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mode_groups: Vec<ModeGroupWithModes>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub state_groups: Vec<StateGroupWithStates>,
}

#[derive(Serialize, Deserialize)]
pub struct ModeGroupWithModes {
    pub mode_group_id: Uuid,
    pub mode_group_name: String,
    pub mode_group_description: String,
    pub modes: Vec<Modes>,
}

#[derive(Serialize, Deserialize)]
pub struct StateGroupWithStates {
    pub state_group_id: Uuid,
    pub state_group_name: String,
    pub state_group_description: String,
    pub states: Vec<State>,
}

#[derive(Serialize, Deserialize)]
pub struct CurrentEquipmentMode {
    pub equipment_id: Uuid,
    pub mode_id: Uuid,
    pub set_at: DateTime<Utc>,
    pub set_by: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CurrentEquipmentState {
    pub equipment_id: Uuid,
    pub state_id: Uuid,
    pub state_value: Option<Value>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EquipmentPath {
    pub equipment_id: Uuid,
    pub path: Vec<Equipment>,
    pub depth: i32,
}