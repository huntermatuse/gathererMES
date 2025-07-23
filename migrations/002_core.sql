-- core tables of the mes, used through of rest of the system

-- equipment types table
CREATE TABLE core.equipment_type (
    type_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    type_name text collate "case_insensitive" unique not null,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

SELECT trigger_updated_at('core.equipment_type');

-- Insert the default equipment types that come with gathererMES
INSERT INTO core.equipment_type (type_name) VALUES ('enterprise') RETURNING type_id;
INSERT INTO core.equipment_type (type_name) VALUES ('site') RETURNING type_id;
INSERT INTO core.equipment_type (type_name) VALUES ('area') RETURNING type_id;
INSERT INTO core.equipment_type (type_name) VALUES ('line') RETURNING type_id;
INSERT INTO core.equipment_type (type_name) VALUES ('cell') RETURNING type_id;

-- equipment table
CREATE TABLE core.equipment (
    equipment_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    -- friendly name of the equipment being created i.e. 'Oven 1'
    equipment_name VARCHAR(255) NOT NULL,
    -- type of equipment that is being created i.e. enterpise, site, area, line, cell
    equipment_type_id uuid NOT NULL REFERENCES core.equipment_type(type_id),
    -- parent to the equipment being created, but could be null (in the case of enterprise)
    equipment_parent_id uuid REFERENCES core.equipment(equipment_id),
    -- is the equipment enabled, defaults to true if not specified
    equipment_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    -- metadata could be used to stored the equipment config, i.e. the json for the ignition tag if we wanted
    equipment_metadata JSONB DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

SELECT trigger_updated_at('core.equipment');

-- mode groups table
-- mode group can be applied to many lines, some lines may be more manual or dynamic. 
-- allows for different operating desciptions for a line or higher level view for the mes
CREATE TABLE core.mode_group (
    mode_group_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    -- should be used to describe a grouping of mes modes that can be applied to different <better word, but places> the mes is tracking.
    -- could be a group of lines (i.e. 'Ovens 1-3', or 'Soft Bake Ovens') where operating mode of the overall group could be the same
    mode_group_name VARCHAR(255) NOT NULL UNIQUE,
    mode_group_description text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

SELECT trigger_updated_at('core.mode_group');

-- modes table
-- i think that modes are the overall <SOME WORD HERE> that describes what the line or mes is doing
-- gathererMES defines modes as being button events triggered within the mes itself. 
-- (i.e. 'change over', 'running', 'mainatance called', 'engineering run')
-- really only used to track tags in the system and do the math required for KPIs and Reporting
-- every cell state would fall under production (minus if the machine is down?) if we are doing this correctly
-- we will need to refine this definition
CREATE TABLE core.mode (
    mode_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    mode_group_id uuid NOT NULL REFERENCES core.mode_group(mode_group_id),
    -- describe the high level <SOME WORD HERE, could be description> the overall line is in.
    mode_description text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz,
    UNIQUE(mode_group_id, mode_description)
);

SELECT trigger_updated_at('core.mode');

DO $$
DECLARE
    default_mode_group uuid;
BEGIN
    INSERT INTO core.mode_group (mode_group_name, mode_group_description)
    VALUES ('Default MES Mode Group', 'Default MES Mode Group')
    RETURNING mode_group_id INTO default_mode_group;

    -- insert default modes
    INSERT INTO core.mode (mode_group_id, mode_description) VALUES (default_mode_group, 'disabled');
    INSERT INTO core.mode (mode_group_id, mode_description) VALUES (default_mode_group, 'production');
    INSERT INTO core.mode (mode_group_id, mode_description) VALUES (default_mode_group, 'idle');
    INSERT INTO core.mode (mode_group_id, mode_description) VALUES (default_mode_group, 'change over');
END $$;

-- state comes from the machine itself, weather from a plc or computer controlling the machine
-- state could also refer to status
-- the mes uses these states to understand the overall health or status of cells that make up a line
-- state may be used for downtime tracking but used for event tracking in realtime calculation system and historian
-- state group table
-- state group could be a grouping of similar cells where a plc throws similar error codes
-- i.e. Capper cell could use similar plc codes for state between 10
-- or could be a AGV uses state codes to communicate what AGV is doing
CREATE TABLE core.state_group (
    state_group_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    -- name of grouping used for similar states (i.e. 'AVG Group 1')
    state_group_name VARCHAR(255) NOT NULL UNIQUE,
    -- a description of what the state group is (i.e. 'AVG Group installed in 2024 for Lines 2-7')
    state_group_description text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

SELECT trigger_updated_at('core.state_group');

-- states table
-- these would be the individual state // status codes descibing what the cell is doing.
-- each state must have a unique code and description within its state group.
-- this ensures that:
--   - no two states in the same group share the same code (state_code)
--   - no two states in the same group have the same description (state_description)
-- this avoids ambiguity when interpreting state codes or descriptions within a group.
CREATE TABLE core.state (
    state_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    state_group_id uuid NOT NULL REFERENCES core.state_group(state_group_id),
    -- state or status code reported to the system
    state_code INTEGER NOT NULL,
    -- human friendly name for the state code being reported by the cell
    state_description text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz,
    
    -- state_code is unique within each group
    UNIQUE(state_group_id, state_code),

    -- state_description is unique within each group
    UNIQUE(state_group_id, state_description)
);

SELECT trigger_updated_at('core.state');

DO $$
DECLARE
    default_state_group uuid;
BEGIN
    INSERT INTO core.state_group (state_group_name, state_group_description)
    VALUES ('Default MES State Group', 'Default MES State Group')
    RETURNING state_group_id INTO default_state_group;

    -- insert default states
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 0, 'disabled');
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 1, 'running');
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 2, 'change over');
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 3, 'idle');
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 4, 'e-stop');
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 5, 'blocked');
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 6, 'starved');
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 7, 'planned downtime');
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 8, 'unplanned downtime');
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 9, 'user planned downtime');
    INSERT INTO core.state (state_group_id, state_code, state_description) VALUES (default_state_group, 10, 'user unplanned downtime');
END $$;


-- i want to rework these but i think they need to stay for now...
-- equipment to mode groups mapping (many-to-many)
CREATE TABLE core.equipment_mode_group_mapping (
    equipment_id uuid NOT NULL REFERENCES core.equipment(equipment_id),
    mode_group_id uuid NOT NULL REFERENCES core.mode_group(mode_group_id),
    PRIMARY KEY (equipment_id, mode_group_id)
);

-- equipment to state groups mapping (many-to-many)
CREATE TABLE core.equipment_state_group_mapping (
    equipment_id uuid NOT NULL REFERENCES core.equipment(equipment_id),
    state_group_id uuid NOT NULL REFERENCES core.state_group(state_group_id),
    PRIMARY KEY (equipment_id, state_group_id)
);

-- generic indexes that i thought about after making the tables
-- indexes
CREATE INDEX idx_equipment_type ON core.equipment(equipment_type_id);
CREATE INDEX idx_equipment_enabled ON core.equipment(equipment_enabled);
CREATE INDEX idx_equipment_metadata ON core.equipment USING GIN(equipment_metadata);
CREATE INDEX idx_mode_group ON core.mode(mode_group_id);
CREATE INDEX idx_state_group ON core.state(state_group_id);