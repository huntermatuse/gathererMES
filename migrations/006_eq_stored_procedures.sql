/*
===========================================
Author:        hunter
Created:       2025-07-28
Schema:        core
Version:       1.0.0
Description:   Equipment Comments, Procedures and Constraints 
Change Log:
    2025-07-28  hunter  init
===========================================
*/

COMMENT ON TABLE core.equipment IS 'Main equipment hierarchy table representing physical or logical manufacturing assets';
COMMENT ON COLUMN core.equipment.equipment_name IS 'Friendly name for the equipment instance';
COMMENT ON COLUMN core.equipment.equipment_parent_id IS 'Self-referencing foreign key creating the equipment hierarchy';
COMMENT ON COLUMN core.equipment.equipment_metadata IS 'JSON configuration and settings for the equipment';

ALTER TABLE core.equipment 
ADD CONSTRAINT chk_equipment_name_not_empty 
CHECK (length(trim(equipment_name)) > 0);

ALTER TABLE core.equipment 
ADD CONSTRAINT chk_equipment_name_length 
CHECK (length(equipment_name) BETWEEN 1 AND 255);

-- equipment can not  be its own parent
ALTER TABLE core.equipment 
ADD CONSTRAINT chk_equipment_no_self_reference 
CHECK (equipment_id != equipment_parent_id);

-- unique equipment names within the same parent and type
-- prevents duplicate names at the same hierarchy level
CREATE UNIQUE INDEX idx_equipment_unique_name_per_parent_type 
ON core.equipment (equipment_parent_id, equipment_type_id, equipment_name)
WHERE equipment_parent_id IS NOT NULL;

CREATE UNIQUE INDEX idx_equipment_unique_root_name_per_type 
ON core.equipment (equipment_type_id, equipment_name)
WHERE equipment_parent_id IS NULL;

-- equipment_metadata is valid json if not null
ALTER TABLE core.equipment 
ADD CONSTRAINT chk_equipment_metadata_valid_json 
CHECK (equipment_metadata IS NULL OR (equipment_metadata::text != 'null'::text));


/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getAllEquipment
Version:       1.0.0
Description:   Retrieves all equipment with type and parent information
Parameters:
    None
Returns:       Table with equipment details
Change Log:
    2025-07-28  hm  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getAllEquipment()
RETURNS TABLE (
    equipment_id uuid,
    equipment_name varchar(255),
    equipment_type_id uuid,
    equipment_type_name text,
    equipment_parent_id uuid,
    equipment_parent_name varchar(255),
    equipment_enabled boolean,
    equipment_metadata jsonb,
    created_at timestamptz,
    updated_at timestamptz
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    RETURN QUERY
    SELECT 
        e.equipment_id,
        e.equipment_name,
        e.equipment_type_id,
        et.type_name as equipment_type_name,
        e.equipment_parent_id,
        pe.equipment_name as equipment_parent_name,
        e.equipment_enabled,
        e.equipment_metadata,
        e.created_at,
        e.updated_at
    FROM core.equipment e
    JOIN core.equipment_type et ON e.equipment_type_id = et.type_id
    LEFT JOIN core.equipment pe ON e.equipment_parent_id = pe.equipment_id
    ORDER BY et.type_name, e.equipment_name;
    
EXCEPTION WHEN OTHERS THEN
    RAISE EXCEPTION 'Error retrieving equipment: %', SQLERRM;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Function:      core.getAllEquipmentJson
Version:       1.0.0
Description:   Retrieves all equipment from the equipment hierarchy view
Parameters:
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Equipment hierarchy as JSON array
Returns:       Record with Status, Message, and Data fields
Change Log:
    2025-07-28  hm  Iinit
===========================================
*/
CREATE OR REPLACE FUNCTION core.getAllEquipmentJson(
    OUT "Status" VARCHAR(50),
    OUT "Message" TEXT,
    OUT "Data" JSONB
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment hierarchy retrieved successfully';
    "Data" := NULL;
    
    SELECT jsonb_agg(
        jsonb_build_object(
            'equipment_id', e.equipment_id,
            'equipment_name', e.equipment_name,
            'equipment_type_id', e.equipment_type_id,
            'equipment_type_name', et.type_name,
            'equipment_parent_id', e.equipment_parent_id,
            'equipment_enabled', e.equipment_enabled,
            'equipment_metadata', e.equipment_metadata,
            'created_at', e.created_at,
            'updated_at', e.updated_at
        ) ORDER BY et.type_name, e.equipment_name
    ) INTO "Data"
    FROM core.equipment e
    INNER JOIN core.equipment_type et ON e.equipment_type_id = et.type_id
    WHERE e.equipment_enabled = true;
    
    IF "Data" IS NULL THEN
        "Data" := '[]'::jsonb;
        "Message" := 'No equipment found';
    END IF;
    
EXCEPTION 
    WHEN OTHERS THEN
        "Status" := 'Error';
        "Message" := 'Unexpected error retrieving equipment: ' || SQLERRM;
        "Data" := NULL;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getEquipmentById
Version:       1.0.0
Description:   Retrieves specific equipment by ID with details
Parameters:
    p_equipment_id UUID                 -- Required equipment ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Equipment data as JSON
Change Log:
    2025-07-28  hm  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getEquipmentById(
    p_equipment_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_result RECORD;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment retrieved successfully';
    "Data" := NULL;
    
    -- p-validation
    IF p_equipment_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment ID cannot be null';
        RETURN;
    END IF;
    
    -- Get equipment with type and parent details
    SELECT 
        e.equipment_id,
        e.equipment_name,
        e.equipment_type_id,
        et.type_name as equipment_type_name,
        e.equipment_parent_id,
        pe.equipment_name as equipment_parent_name,
        e.equipment_enabled,
        e.equipment_metadata,
        e.created_at,
        e.updated_at
    INTO v_result
    FROM core.equipment e
    JOIN core.equipment_type et ON e.equipment_type_id = et.type_id
    LEFT JOIN core.equipment pe ON e.equipment_parent_id = pe.equipment_id
    WHERE e.equipment_id = p_equipment_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Equipment not found';
        RETURN;
    END IF;
    
    -- json output
    "Data" := jsonb_build_object(
        'equipment_id', v_result.equipment_id,
        'equipment_name', v_result.equipment_name,
        'equipment_enabled', v_result.equipment_enabled,
        'equipment_metadata', v_result.equipment_metadata,
        'equipment_type', jsonb_build_object(
            'equipment_type_id', v_result.equipment_type_id,
            'equipment_type_name', v_result.equipment_type_name
        ),
        'parent_equipment', CASE 
            WHEN v_result.equipment_parent_id IS NOT NULL THEN
                jsonb_build_object(
                    'equipment_parent_id', v_result.equipment_parent_id,
                    'equipment_parent_name', v_result.equipment_parent_name
                )
            ELSE NULL
        END,
        'created_at', v_result.created_at,
        'updated_at', v_result.updated_at
    );
    
EXCEPTION WHEN OTHERS THEN
    "Status" := 'Error';
    "Message" := 'Unexpected error: ' || SQLERRM;
    "Data" := NULL;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.insertEquipment
Version:       1.0.0
Description:   Creates new equipment with validation
Parameters:
    p_equipment_name VARCHAR(255)       -- Required equipment name
    p_equipment_type_id UUID            -- Required equipment type ID
    p_equipment_parent_id UUID          -- Optional parent equipment ID
    p_equipment_enabled BOOLEAN=TRUE    -- Optional enabled flag
    p_equipment_metadata JSONB          -- Optional metadata
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- New equipment data as JSON
Change Log:
    2025-07-28  hm  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.insertEquipment(
    p_equipment_name varchar(255),
    p_equipment_type_id uuid,
    p_equipment_parent_id uuid DEFAULT NULL,
    p_equipment_enabled boolean DEFAULT TRUE,
    p_equipment_metadata jsonb DEFAULT NULL,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_new_id uuid;
    v_trimmed_name varchar(255);
    v_equipment_type_name text;
    v_parent_name varchar(255);
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment created successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_equipment_name IS NULL OR trim(p_equipment_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'Equipment name cannot be empty';
        RETURN;
    END IF;
    
    IF p_equipment_type_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment type ID cannot be null';
        RETURN;
    END IF;
    
    v_trimmed_name := trim(p_equipment_name);
    
    -- validation
    IF length(v_trimmed_name) < 1 OR length(v_trimmed_name) > 255 THEN
        "Status" := 'Error';
        "Message" := 'Equipment name must be between 1 and 255 characters';
        RETURN;
    END IF;
    
    -- eq found?
    SELECT type_name INTO v_equipment_type_name
    FROM core.equipment_type
    WHERE type_id = p_equipment_type_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Equipment type not found';
        RETURN;
    END IF;
    
    -- verify parent exists if provided
    IF p_equipment_parent_id IS NOT NULL THEN
        SELECT equipment_name INTO v_parent_name
        FROM core.equipment
        WHERE equipment_id = p_equipment_parent_id;
        
        IF NOT FOUND THEN
            "Status" := 'Error';
            "Message" := 'Parent equipment not found';
            RETURN;
        END IF;
    END IF;
    
    -- create new equipment
    INSERT INTO core.equipment (
        equipment_name, 
        equipment_type_id, 
        equipment_parent_id, 
        equipment_enabled, 
        equipment_metadata
    )
    VALUES (
        v_trimmed_name, 
        p_equipment_type_id, 
        p_equipment_parent_id, 
        COALESCE(p_equipment_enabled, TRUE), 
        COALESCE(p_equipment_metadata, '{}'::jsonb)
    )
    RETURNING equipment_id INTO v_new_id;
    
    -- return created data
    "Data" := jsonb_build_object(
        'equipment_id', v_new_id,
        'equipment_name', v_trimmed_name,
        'equipment_enabled', COALESCE(p_equipment_enabled, TRUE),
        'equipment_metadata', COALESCE(p_equipment_metadata, '{}'::jsonb),
        'equipment_type', jsonb_build_object(
            'equipment_type_id', p_equipment_type_id,
            'equipment_type_name', v_equipment_type_name
        ),
        'parent_equipment', CASE 
            WHEN p_equipment_parent_id IS NOT NULL THEN
                jsonb_build_object(
                    'equipment_parent_id', p_equipment_parent_id,
                    'equipment_parent_name', v_parent_name
                )
            ELSE NULL
        END,
        'created_at', now()
    );
    
EXCEPTION 
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Invalid equipment type or parent equipment ID';
        "Data" := NULL;
    WHEN OTHERS THEN
        "Status" := 'Error';
        "Message" := 'Unexpected error: ' || SQLERRM;
        "Data" := NULL;
END;
$$;

/*
===========================================
Author:        hunter 
Created:       2025-01-28
Procedure:     core.updateEquipment
Version:       1.0.0
Description:   Updates existing equipment with validation
Parameters:
    p_equipment_id UUID                 -- Required equipment ID
    p_equipment_name VARCHAR(255)       -- Optional new equipment name (null = no change)
    p_equipment_type_id UUID            -- Optional new equipment type ID (null = no change)
    p_equipment_parent_id UUID          -- Optional new parent equipment ID (null = no change)
    p_equipment_enabled BOOLEAN         -- Optional new enabled flag (null = no change)
    p_equipment_metadata JSONB          -- Optional new metadata (null = no change)
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Updated equipment data as JSON
Change Log:
    2025-07-28 hm init
===========================================
*/
CREATE OR REPLACE FUNCTION core.updateEquipment(
    p_equipment_id uuid,
    p_equipment_name varchar(255) DEFAULT NULL,
    p_equipment_type_id uuid DEFAULT NULL,
    p_equipment_parent_id uuid DEFAULT NULL,
    p_equipment_enabled boolean DEFAULT NULL,
    p_equipment_metadata jsonb DEFAULT NULL,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_current_record RECORD;
    v_new_name varchar(255);
    v_new_type_id uuid;
    v_new_parent_id uuid;
    v_new_enabled boolean;
    v_new_metadata jsonb;
    v_changes_made boolean := false;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment updated successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_equipment_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment ID cannot be null';
        RETURN;
    END IF;
    
    -- get current record
    SELECT 
        equipment_name, 
        equipment_type_id, 
        equipment_parent_id, 
        equipment_enabled, 
        equipment_metadata
    INTO v_current_record
    FROM core.equipment
    WHERE equipment_id = p_equipment_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Equipment not found';
        RETURN;
    END IF;
    
    -- determine what to update using COALESCE pattern for partials
    v_new_name := COALESCE(trim(p_equipment_name), v_current_record.equipment_name);
    v_new_type_id := COALESCE(p_equipment_type_id, v_current_record.equipment_type_id);
    v_new_parent_id := COALESCE(p_equipment_parent_id, v_current_record.equipment_parent_id);
    v_new_enabled := COALESCE(p_equipment_enabled, v_current_record.equipment_enabled);
    v_new_metadata := COALESCE(p_equipment_metadata, v_current_record.equipment_metadata);
    
    -- validate new name if provided
    IF p_equipment_name IS NOT NULL THEN
        IF trim(p_equipment_name) = '' THEN
            "Status" := 'Error';
            "Message" := 'Equipment name cannot be empty';
            RETURN;
        END IF;
        
        IF length(v_new_name) < 1 OR length(v_new_name) > 255 THEN
            "Status" := 'Error';
            "Message" := 'Equipment name must be between 1 and 255 characters';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    -- validate new equipment type if provided
    IF p_equipment_type_id IS NOT NULL THEN
        IF NOT EXISTS (SELECT 1 FROM core.equipment_type WHERE type_id = p_equipment_type_id) THEN
            "Status" := 'Error';
            "Message" := 'Invalid equipment type ID';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    -- validate new parent if provided
    IF p_equipment_parent_id IS NOT NULL THEN
        -- parent found?
        IF NOT EXISTS (SELECT 1 FROM core.equipment WHERE equipment_id = p_equipment_parent_id) THEN
            "Status" := 'Error';
            "Message" := 'Invalid parent equipment ID';
            RETURN;
        END IF;
        
        -- self-reference
        IF p_equipment_parent_id = p_equipment_id THEN
            "Status" := 'Error';
            "Message" := 'Equipment cannot be its own parent';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    -- check other field changes
    IF p_equipment_enabled IS NOT NULL THEN
        v_changes_made := true;
    END IF;
    
    IF p_equipment_metadata IS NOT NULL THEN
        v_changes_made := true;
    END IF;
    
    -- update if changes
    IF v_changes_made THEN
        UPDATE core.equipment
        SET equipment_name = v_new_name,
            equipment_type_id = v_new_type_id,
            equipment_parent_id = v_new_parent_id,
            equipment_enabled = v_new_enabled,
            equipment_metadata = v_new_metadata,
            updated_at = now()
        WHERE equipment_id = p_equipment_id;
    ELSE
        "Message" := 'No changes were made to equipment';
    END IF;
    
    -- lazy return getEquipmentById
    SELECT e."Data" INTO "Data"
    FROM core.getEquipmentById(p_equipment_id) e
    WHERE e."Status" = 'Success';
    
EXCEPTION 
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Invalid equipment type or parent equipment ID';
        "Data" := NULL;
    WHEN OTHERS THEN
        "Status" := 'Error';
        "Message" := 'Unexpected error: ' || SQLERRM;
        "Data" := NULL;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-01-28
Procedure:     core.deleteEquipment
Version:       1.0.0
Description:   Safely deletes equipment with dependency checks
Parameters:
    p_equipment_id UUID                 -- Required equipment ID
    p_force_delete BOOLEAN=FALSE        -- Optional force delete flag
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Operation result data as JSON
Change Log:
    2025-07-28  hm  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.deleteEquipment(
    p_equipment_id uuid,
    p_force_delete boolean DEFAULT FALSE,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_equipment_info RECORD;
    v_children_count integer;
    v_deleted_count integer;
BEGIN
    -- init validation
    "Status" := 'Success';
    "Message" := 'Equipment deleted successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_equipment_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment ID cannot be null';
        RETURN;
    END IF;
    
    -- get eq info
    SELECT 
        e.equipment_name,
        et.type_name as equipment_type_name,
        e.equipment_parent_id
    INTO v_equipment_info
    FROM core.equipment e
    JOIN core.equipment_type et ON e.equipment_type_id = et.type_id
    WHERE e.equipment_id = p_equipment_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Equipment not found';
        RETURN;
    END IF;
    
    -- count the children
    SELECT COUNT(*) INTO v_children_count
    FROM core.equipment
    WHERE equipment_parent_id = p_equipment_id;
    
    -- does anything rely on this?
    IF v_children_count > 0 AND NOT p_force_delete THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete equipment: ' || v_children_count || ' child equipment exist. Use force_delete=true to override.';
        RETURN;
    END IF;
    
    -- if force delete orphan the children
    IF p_force_delete AND v_children_count > 0 THEN
        UPDATE core.equipment 
        SET equipment_parent_id = NULL,
            updated_at = now()
        WHERE equipment_parent_id = p_equipment_id;
    END IF;
    
    -- delete the equipment
    DELETE FROM core.equipment WHERE equipment_id = p_equipment_id;
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    IF v_deleted_count = 0 THEN
        "Status" := 'Error';
        "Message" := 'Equipment could not be deleted';
        RETURN;
    END IF;
    
    "Data" := jsonb_build_object(
        'deleted_equipment_id', p_equipment_id,
        'deleted_equipment_name', v_equipment_info.equipment_name,
        'deleted_equipment_type', v_equipment_info.equipment_type_name,
        'affected_children_count', v_children_count,
        'force_delete_used', p_force_delete
    );
    
EXCEPTION 
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete equipment: dependencies still exist';
        "Data" := NULL;
    WHEN OTHERS THEN
        "Status" := 'Error';
        "Message" := 'Unexpected error: ' || SQLERRM;
        "Data" := NULL;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.equipmentExists
Version:       1.0.0
Description:   Checks if equipment exists by name and optional parent/type
Parameters:
    p_equipment_name VARCHAR(255)       -- Required equipment name
    p_equipment_parent_id UUID          -- Optional parent equipment ID
    p_equipment_type_id UUID            -- Optional equipment type ID
Returns:       BOOLEAN                  -- TRUE if exists, FALSE otherwise
Change Log:
    2025-07-28  hm  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.equipmentExists(
    p_equipment_name varchar(255),
    p_equipment_parent_id uuid DEFAULT NULL,
    p_equipment_type_id uuid DEFAULT NULL
)
RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    IF p_equipment_name IS NULL OR trim(p_equipment_name) = '' THEN
        RETURN FALSE;
    END IF;
    
    RETURN EXISTS (
        SELECT 1 FROM core.equipment 
        WHERE equipment_name = trim(p_equipment_name)
        AND (p_equipment_parent_id IS NULL OR equipment_parent_id = p_equipment_parent_id)
        AND (p_equipment_type_id IS NULL OR equipment_type_id = p_equipment_type_id)
    );
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Function:      core.setEquipmentConfig
Version:       1.0.0
Description:   Sets/updates configuration data for specified equipment (upsert operation)
Parameters:
    p_equipmentId UUID                  -- Required equipment ID
    p_configData JSONB                  -- Required configuration data as JSON
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Updated configuration data
Returns:       Record with Status, Message, and Data fields
Change Log:
    2025-07-28  hm  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.setEquipmentConfig(
    p_equipmentId UUID,
    p_configData JSONB,
    OUT "Status" VARCHAR(50),
    OUT "Message" TEXT,
    OUT "Data" JSONB
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment configuration updated successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_equipmentId IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment ID cannot be null';
        RETURN;
    END IF;
    
    IF p_configData IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Configuration data cannot be null';
        RETURN;
    END IF;
    
    -- if equipment exists
    IF NOT EXISTS (SELECT 1 FROM core.equipment WHERE equipment_id = p_equipmentId) THEN
        "Status" := 'Error';
        "Message" := 'Equipment not found';
        RETURN;
    END IF;
    
    -- update metatdata
    UPDATE core.equipment 
    SET equipment_metadata = p_configData,
        updated_at = now()
    WHERE equipment_id = p_equipmentId;
    
    -- return updated config
    "Data" := p_configData;
    
EXCEPTION 
    WHEN OTHERS THEN
        "Status" := 'Error';
        "Message" := 'Unexpected error updating equipment configuration: ' || SQLERRM;
        "Data" := NULL;
END;
$$;
