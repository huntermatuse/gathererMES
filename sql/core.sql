-- core tables of the mes, used through of rest of the system

-- equipment types table
CREATE TABLE equipment_types (
    type_id SERIAL PRIMARY KEY,
    type_name VARCHAR(255) NOT NULL UNIQUE
);

-- equipment table
CREATE TABLE equipment (
    equipment_id SERIAL PRIMARY KEY,
    equipment_name VARCHAR(255) NOT NULL,
    equipment_type_id INTEGER NOT NULL REFERENCES equipment_types(type_id),
    equipment_parent_id INTEGER REFERENCES equipment(equipment_id),
    equipment_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    equipment_metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- mode groups table
CREATE TABLE mode_groups (
    mode_group_id SERIAL PRIMARY KEY,
    mode_group_name VARCHAR(255) NOT NULL UNIQUE,
    mode_group_description TEXT
);

-- modes table
CREATE TABLE modes (
    mode_id SERIAL PRIMARY KEY,
    mode_group_id INTEGER NOT NULL REFERENCES mode_groups(mode_group_id),
    mode_code INTEGER NOT NULL,
    mode_description VARCHAR(500),
    UNIQUE(mode_group_id, mode_code)
);

-- state groups table
CREATE TABLE state_groups (
    state_group_id SERIAL PRIMARY KEY,
    state_group_name VARCHAR(255) NOT NULL UNIQUE,
    state_group_description TEXT
);

-- states table
CREATE TABLE states (
    state_id SERIAL PRIMARY KEY,
    state_group_id INTEGER NOT NULL REFERENCES state_groups(state_group_id),
    state_name VARCHAR(255) NOT NULL,
    state_description TEXT,
    UNIQUE(state_group_id, state_name)
);

-- equipment to mode groups mapping (many-to-many)
CREATE TABLE equipment_mode_group_mapping (
    equipment_id INTEGER NOT NULL REFERENCES equipment(equipment_id) ON DELETE CASCADE,
    mode_group_id INTEGER NOT NULL REFERENCES mode_groups(mode_group_id) ON DELETE CASCADE,
    PRIMARY KEY (equipment_id, mode_group_id)
);

-- equipment to state groups mapping (many-to-many)
CREATE TABLE equipment_state_group_mapping (
    equipment_id INTEGER NOT NULL REFERENCES equipment(equipment_id) ON DELETE CASCADE,
    state_group_id INTEGER NOT NULL REFERENCES state_groups(state_group_id) ON DELETE CASCADE,
    PRIMARY KEY (equipment_id, state_group_id)
);

-- Indexes
CREATE INDEX idx_equipment_type ON equipment(equipment_type_id);
CREATE INDEX idx_equipment_enabled ON equipment(equipment_enabled);
CREATE INDEX idx_equipment_metadata ON equipment USING GIN(equipment_metadata);
CREATE INDEX idx_modes_group ON modes(mode_group_id);
CREATE INDEX idx_states_group ON states(state_group_id);