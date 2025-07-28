
/*
===========================================
Author:        hunter
Created:       2025-07-28
Schema:        core
Version:       1.0.0
Description:   Mode Group and Mode Comments, Procedures and Constraints 
Change Log:
    2025-07-28  hunter  init
===========================================
*/

COMMENT ON TABLE core.mode_group IS 'Groups of operational modes that can be applied to equipment';
COMMENT ON TABLE core.mode IS 'Individual operational modes within a mode group';

ALTER TABLE core.mode_group 
ADD CONSTRAINT chk_mode_group_name_not_empty 
CHECK (length(trim(mode_group_name)) > 0);

ALTER TABLE core.mode_group 
ADD CONSTRAINT chk_mode_group_name_length 
CHECK (length(mode_group_name) BETWEEN 2 AND 255);

ALTER TABLE core.mode_group 
ADD CONSTRAINT chk_mode_group_description_not_empty 
CHECK (length(trim(mode_group_description)) > 0);

ALTER TABLE core.mode_group 
ADD CONSTRAINT chk_mode_group_description_length 
CHECK (length(mode_group_description) BETWEEN 5 AND 2048);

ALTER TABLE core.mode 
ADD CONSTRAINT chk_mode_description_not_empty 
CHECK (length(trim(mode_description)) > 0);

ALTER TABLE core.mode 
ADD CONSTRAINT chk_mode_description_length 
CHECK (length(mode_description) BETWEEN 2 AND 2048);

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getAllModeGroups
Version:       1.0.0
Description:   Retrieves all mode groups with mode counts and equipment associations
Parameters:
    None
Returns:       Table with mode_group_id, mode_group_name, mode_group_description, mode_count, equipment_count, created_at, updated_at
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getAllModeGroups()
RETURNS TABLE (
    mode_group_id uuid,
    mode_group_name varchar(255),
    mode_group_description text,
    mode_count bigint,
    equipment_count bigint,
    created_at timestamptz,
    updated_at timestamptz
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    RETURN QUERY
    SELECT 
        mg.mode_group_id,
        mg.mode_group_name,
        mg.mode_group_description,
        COALESCE(COUNT(DISTINCT m.mode_id), 0) as mode_count,
        COALESCE(COUNT(DISTINCT emgm.equipment_id), 0) as equipment_count,
        mg.created_at,
        mg.updated_at
    FROM core.mode_group mg
    LEFT JOIN core.mode m ON mg.mode_group_id = m.mode_group_id
    LEFT JOIN core.equipment_mode_group_mapping emgm ON mg.mode_group_id = emgm.mode_group_id
    GROUP BY mg.mode_group_id, mg.mode_group_name, mg.mode_group_description, mg.created_at, mg.updated_at
    ORDER BY mg.mode_group_name;
    
EXCEPTION WHEN OTHERS THEN
    RAISE EXCEPTION 'Error retrieving mode groups: %', SQLERRM;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getModeGroupById
Version:       1.0.0
Description:   Retrieves specific mode group by ID
Parameters:
    p_mode_group_id UUID                -- Required mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Mode group data with modes and equipment as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getModeGroupById(
    p_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_group RECORD;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode group retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- get mode group basic info
    SELECT 
        mg.mode_group_id,
        mg.mode_group_name,
        mg.mode_group_description,
        mg.created_at,
        mg.updated_at
    INTO v_mode_group
    FROM core.mode_group mg
    WHERE mg.mode_group_id = p_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- build complete response
    "Data" := jsonb_build_object(
        'mode_group_id', v_mode_group.mode_group_id,
        'mode_group_name', v_mode_group.mode_group_name,
        'mode_group_description', v_mode_group.mode_group_description,
        'created_at', v_mode_group.created_at,
        'updated_at', v_mode_group.updated_at
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
Procedure:     core.getModeGroupByName
Version:       1.0.0
Description:   Retrieves specific mode group by name
Parameters:
    p_mode_group_name VARCHAR(255)      -- Required mode group name
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Mode group data with modes and equipment as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getModeGroupByName(
    p_mode_group_name varchar(255),
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_group RECORD;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode group retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_group_name IS NULL OR trim(p_mode_group_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'Mode group name cannot be empty';
        RETURN;
    END IF;
    
    -- get mode group ID by name
    SELECT mode_group_id INTO v_mode_group
    FROM core.mode_group
    WHERE mode_group_name = trim(p_mode_group_name);
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- delegate to getModeGroupById
    SELECT mg."Status", mg."Message", mg."Data"
    INTO "Status", "Message", "Data"
    FROM core.getModeGroupById(v_mode_group) mg;
    
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
Procedure:     core.getModeGroupByIdContext
Version:       1.0.0
Description:   Retrieves specific mode group by ID with associated modes and equipment
Parameters:
    p_mode_group_id UUID                -- Required mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Mode group data with modes and equipment as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getModeGroupByIdContext(
    p_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_group RECORD;
    v_modes jsonb;
    v_equipment jsonb;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode group retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- get mode group basic info
    SELECT 
        mg.mode_group_id,
        mg.mode_group_name,
        mg.mode_group_description,
        mg.created_at,
        mg.updated_at
    INTO v_mode_group
    FROM core.mode_group mg
    WHERE mg.mode_group_id = p_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- get associated modes
    SELECT COALESCE(jsonb_agg(
        jsonb_build_object(
            'mode_id', m.mode_id,
            'mode_description', m.mode_description,
            'created_at', m.created_at,
            'updated_at', m.updated_at
        ) ORDER BY m.mode_description
    ), '[]'::jsonb)
    INTO v_modes
    FROM core.mode m
    WHERE m.mode_group_id = p_mode_group_id;
    
    -- get associated equipment
    SELECT COALESCE(jsonb_agg(
        jsonb_build_object(
            'equipment_id', e.equipment_id,
            'equipment_name', e.equipment_name,
            'equipment_type', et.type_name,
            'equipment_enabled', e.equipment_enabled
        ) ORDER BY e.equipment_name
    ), '[]'::jsonb)
    INTO v_equipment
    FROM core.equipment_mode_group_mapping emgm
    JOIN core.equipment e ON emgm.equipment_id = e.equipment_id
    JOIN core.equipment_type et ON e.equipment_type_id = et.type_id
    WHERE emgm.mode_group_id = p_mode_group_id;
    
    -- build complete response
    "Data" := jsonb_build_object(
        'mode_group_id', v_mode_group.mode_group_id,
        'mode_group_name', v_mode_group.mode_group_name,
        'mode_group_description', v_mode_group.mode_group_description,
        'created_at', v_mode_group.created_at,
        'updated_at', v_mode_group.updated_at,
        'modes', v_modes,
        'equipment', v_equipment,
        'mode_count', jsonb_array_length(v_modes),
        'equipment_count', jsonb_array_length(v_equipment)
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
Procedure:     core.getModeGroupByName
Version:       1.0.0
Description:   Retrieves specific mode group by name
Parameters:
    p_mode_group_name VARCHAR(255)      -- Required mode group name
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Mode group data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getModeGroupByName(
    p_mode_group_name varchar(255),
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_group_id uuid;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode group retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_group_name IS NULL OR trim(p_mode_group_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'Mode group name cannot be empty';
        RETURN;
    END IF;
    
    -- get mode group ID by name
    SELECT mode_group_id INTO v_mode_group_id
    FROM core.mode_group
    WHERE mode_group_name = trim(p_mode_group_name);
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- delegate to getModeGroupById
    SELECT mg."Status", mg."Message", mg."Data"
    INTO "Status", "Message", "Data"
    FROM core.getModeGroupById(v_mode_group_id) mg;
    
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
Procedure:     core.insertModeGroup
Version:       1.0.0
Description:   Creates new mode group with validation
Parameters:
    p_mode_group_name VARCHAR(255)      -- Required mode group name
    p_mode_group_description TEXT       -- Required mode group description
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- New mode group data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.insertModeGroup(
    p_mode_group_name varchar(255),
    p_mode_group_description text,
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
    v_trimmed_description text;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode group created successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_group_name IS NULL OR trim(p_mode_group_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'Mode group name cannot be empty';
        RETURN;
    END IF;
    
    IF p_mode_group_description IS NULL OR trim(p_mode_group_description) = '' THEN
        "Status" := 'Error';
        "Message" := 'Mode group description cannot be empty';
        RETURN;
    END IF;
    
    v_trimmed_name := trim(p_mode_group_name);
    v_trimmed_description := trim(p_mode_group_description);
    
    -- validate lengths
    IF length(v_trimmed_name) < 2 OR length(v_trimmed_name) > 255 THEN
        "Status" := 'Error';
        "Message" := 'Mode group name must be between 2 and 255 characters';
        RETURN;
    END IF;
    
    IF length(v_trimmed_description) < 5 OR length(v_trimmed_description) > 1000 THEN
        "Status" := 'Error';
        "Message" := 'Mode group description must be between 5 and 1000 characters';
        RETURN;
    END IF;
    
    -- check for lengh
    IF EXISTS (SELECT 1 FROM core.mode_group WHERE mode_group_name = v_trimmed_name) THEN
        "Status" := 'Error';
        "Message" := 'Mode group name already exists';
        RETURN;
    END IF;
    
    -- add new mode group
    INSERT INTO core.mode_group (mode_group_name, mode_group_description)
    VALUES (v_trimmed_name, v_trimmed_description)
    RETURNING mode_group_id INTO v_new_id;
    
    -- created data
    "Data" := jsonb_build_object(
        'mode_group_id', v_new_id,
        'mode_group_name', v_trimmed_name,
        'mode_group_description', v_trimmed_description,
        'mode_count', 0,
        'equipment_count', 0,
        'created_at', now()
    );
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'Mode group name already exists';
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
Procedure:     core.updateModeGroup
Version:       1.0.0
Description:   Updates existing mode group with validation
Parameters:
    p_mode_group_id UUID                -- Required mode group ID
    p_mode_group_name VARCHAR(255)      -- Optional new mode group name (null = no change)
    p_mode_group_description TEXT       -- Optional new mode group description (null = no change)
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Updated mode group data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.updateModeGroup(
    p_mode_group_id uuid,
    p_mode_group_name varchar(255) DEFAULT NULL,
    p_mode_group_description text DEFAULT NULL,
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
    v_new_description text;
    v_changes_made boolean := false;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode group updated successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- get the current counts
    SELECT mode_group_name, mode_group_description
    INTO v_current_record
    FROM core.mode_group
    WHERE mode_group_id = p_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- determine what to update (COALESCE pattern for partial updates)
    v_new_name := COALESCE(trim(p_mode_group_name), v_current_record.mode_group_name);
    v_new_description := COALESCE(trim(p_mode_group_description), v_current_record.mode_group_description);
    
    -- validate new values if provided
    IF p_mode_group_name IS NOT NULL THEN
        IF trim(p_mode_group_name) = '' THEN
            "Status" := 'Error';
            "Message" := 'Mode group name cannot be empty';
            RETURN;
        END IF;
        
        IF length(v_new_name) < 2 OR length(v_new_name) > 255 THEN
            "Status" := 'Error';
            "Message" := 'Mode group name must be between 2 and 255 characters';
            RETURN;
        END IF;
        
        -- check for conflicts
        IF EXISTS (
            SELECT 1 FROM core.mode_group 
            WHERE mode_group_name = v_new_name AND mode_group_id != p_mode_group_id
        ) THEN
            "Status" := 'Error';
            "Message" := 'Mode group name already exists';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    IF p_mode_group_description IS NOT NULL THEN
        IF trim(p_mode_group_description) = '' THEN
            "Status" := 'Error';
            "Message" := 'Mode group description cannot be empty';
            RETURN;
        END IF;
        
        IF length(v_new_description) < 5 OR length(v_new_description) > 1000 THEN
            "Status" := 'Error';
            "Message" := 'Mode group description must be between 5 and 1000 characters';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    -- Update if changes were made
    IF v_changes_made THEN
        UPDATE core.mode_group
        SET mode_group_name = v_new_name,
            mode_group_description = v_new_description,
            updated_at = now()
        WHERE mode_group_id = p_mode_group_id;
    ELSE
        "Message" := 'No changes were made to mode group';
    END IF;
    
    -- Return updated data using getModeGroupById
    SELECT mg."Data" INTO "Data"
    FROM core.getModeGroupById(p_mode_group_id) mg
    WHERE mg."Status" = 'Success';
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'Mode group name already exists';
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
Procedure:     core.deleteModeGroup
Version:       1.0.0
Description:   Safely deletes mode group with dependency checks
Parameters:
    p_mode_group_id UUID                -- Required mode group ID
    p_force_delete BOOLEAN=FALSE        -- Optional force delete flag
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Operation result data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.deleteModeGroup(
    p_mode_group_id uuid,
    p_force_delete boolean DEFAULT FALSE,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_count integer;
    v_equipment_count integer;
    v_mode_group_name varchar(255);
    v_deleted_count integer;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode group deleted successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- get mode group info and dependency counts
    SELECT 
        mg.mode_group_name,
        COALESCE(COUNT(DISTINCT m.mode_id), 0),
        COALESCE(COUNT(DISTINCT emgm.equipment_id), 0)
    INTO v_mode_group_name, v_mode_count, v_equipment_count
    FROM core.mode_group mg
    LEFT JOIN core.mode m ON mg.mode_group_id = m.mode_group_id
    LEFT JOIN core.equipment_mode_group_mapping emgm ON mg.mode_group_id = emgm.mode_group_id
    WHERE mg.mode_group_id = p_mode_group_id
    GROUP BY mg.mode_group_name;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- check if it's the default mode group even if force is true
    IF v_mode_group_name = 'Default MES Mode Group' THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete default MES mode group';
        RETURN;
    END IF;
    
    -- check for deps
    IF (v_mode_count > 0 OR v_equipment_count > 0) AND NOT p_force_delete THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete mode group: ' || v_mode_count || ' mode(s) and ' || v_equipment_count || ' equipment association(s) exist. Use force_delete=true to override.';
        RETURN;
    END IF;
    
    -- for delete, start with associations
    IF p_force_delete THEN
        -- remove eq many to many (leave the eq around)
        DELETE FROM core.equipment_mode_group_mapping 
        WHERE mode_group_id = p_mode_group_id;
        
        -- removes modes (should cascade properly due to foreign key)
        DELETE FROM core.mode 
        WHERE mode_group_id = p_mode_group_id;
    END IF;
    
    -- delete the mode group
    DELETE FROM core.mode_group WHERE mode_group_id = p_mode_group_id;
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    IF v_deleted_count = 0 THEN
        "Status" := 'Error';
        "Message" := 'Mode group could not be deleted';
        RETURN;
    END IF;
    
    "Data" := jsonb_build_object(
        'deleted_mode_group_id', p_mode_group_id,
        'deleted_mode_group_name', v_mode_group_name,
        'affected_mode_count', v_mode_count,
        'affected_equipment_count', v_equipment_count,
        'force_delete_used', p_force_delete
    );
    
EXCEPTION 
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete mode group: dependencies still exist';
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
Procedure:     core.searchModeGroups
Version:       1.0.0
Description:   Searches mode groups by name or description pattern
Parameters:
    p_search_term TEXT                  -- Optional search term (null returns all)
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Search results as JSON array
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.searchModeGroups(
    p_search_term text DEFAULT NULL,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_results jsonb;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Search completed successfully';
    "Data" := NULL;
    
    -- search
    IF p_search_term IS NULL OR trim(p_search_term) = '' THEN
        -- 
        SELECT jsonb_agg(
            jsonb_build_object(
                'mode_group_id', mg.mode_group_id,
                'mode_group_name', mg.mode_group_name,
                'mode_group_description', mg.mode_group_description,
                'mode_count', COALESCE(COUNT(DISTINCT m.mode_id), 0),
                'equipment_count', COALESCE(COUNT(DISTINCT emgm.equipment_id), 0),
                'created_at', mg.created_at,
                'updated_at', mg.updated_at
            ) ORDER BY mg.mode_group_name
        )
        INTO v_results
        FROM core.mode_group mg
        LEFT JOIN core.mode m ON mg.mode_group_id = m.mode_group_id
        LEFT JOIN core.equipment_mode_group_mapping emgm ON mg.mode_group_id = emgm.mode_group_id
        GROUP BY mg.mode_group_id, mg.mode_group_name, mg.mode_group_description, mg.created_at, mg.updated_at;
    ELSE
        -- search by pattern in name or description
        SELECT jsonb_agg(
            jsonb_build_object(
                'mode_group_id', mg.mode_group_id,
                'mode_group_name', mg.mode_group_name,
                'mode_group_description', mg.mode_group_description,
                'mode_count', COALESCE(COUNT(DISTINCT m.mode_id), 0),
                'equipment_count', COALESCE(COUNT(DISTINCT emgm.equipment_id), 0),
                'created_at', mg.created_at,
                'updated_at', mg.updated_at
            ) ORDER BY mg.mode_group_name
        )
        INTO v_results
        FROM core.mode_group mg
        LEFT JOIN core.mode m ON mg.mode_group_id = m.mode_group_id
        LEFT JOIN core.equipment_mode_group_mapping emgm ON mg.mode_group_id = emgm.mode_group_id
        WHERE mg.mode_group_name ILIKE '%' || trim(p_search_term) || '%'
           OR mg.mode_group_description ILIKE '%' || trim(p_search_term) || '%'
        GROUP BY mg.mode_group_id, mg.mode_group_name, mg.mode_group_description, mg.created_at, mg.updated_at;
    END IF;
    
    "Data" := COALESCE(v_results, '[]'::jsonb);
    
    IF jsonb_array_length("Data") = 0 THEN
        "Message" := 'No mode groups found matching search criteria';
    ELSE
        "Message" := 'Found ' || jsonb_array_length("Data") || ' mode group(s)';
    END IF;
    
EXCEPTION WHEN OTHERS THEN
    "Status" := 'Error';
    "Message" := 'Unexpected error: ' || SQLERRM;
    "Data" := '[]'::jsonb;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getModeGroupUsageStats
Version:       1.0.0
Description:   Retrieves detailed usage statistics for all mode groups
Parameters:
    None
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Usage statistics as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getModeGroupUsageStats(
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_results jsonb;
    v_total_mode_groups integer;
    v_active_mode_groups integer;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Usage statistics retrieved successfully';
    "Data" := NULL;
    
    -- get stats
    SELECT jsonb_agg(
        jsonb_build_object(
            'mode_group_id', mg.mode_group_id,
            'mode_group_name', mg.mode_group_name,
            'mode_group_description', mg.mode_group_description,
            'mode_count', COALESCE(COUNT(DISTINCT m.mode_id), 0),
            'equipment_count', COALESCE(COUNT(DISTINCT emgm.equipment_id), 0),
            'enabled_equipment_count', COALESCE(COUNT(DISTINCT CASE WHEN e.equipment_enabled THEN emgm.equipment_id END), 0),
            'is_default_group', (mg.mode_group_name = 'Default MES Mode Group'),
            'created_at', mg.created_at,
            'updated_at', mg.updated_at
        ) ORDER BY mg.mode_group_name
    )
    INTO v_results
    FROM core.mode_group mg
    LEFT JOIN core.mode m ON mg.mode_group_id = m.mode_group_id
    LEFT JOIN core.equipment_mode_group_mapping emgm ON mg.mode_group_id = emgm.mode_group_id
    LEFT JOIN core.equipment e ON emgm.equipment_id = e.equipment_id
    GROUP BY mg.mode_group_id, mg.mode_group_name, mg.mode_group_description, mg.created_at, mg.updated_at;
    
    -- get summary counts
    SELECT 
        COUNT(*),
        COUNT(CASE WHEN mg.mode_group_id IN (
            SELECT DISTINCT emgm.mode_group_id 
            FROM core.equipment_mode_group_mapping emgm
        ) THEN 1 END)
    INTO v_total_mode_groups, v_active_mode_groups
    FROM core.mode_group mg;
    
    "Data" := jsonb_build_object(
        'mode_groups', COALESCE(v_results, '[]'::jsonb),
        'summary', jsonb_build_object(
            'total_mode_groups', v_total_mode_groups,
            'active_mode_groups', v_active_mode_groups,
            'unused_mode_groups', v_total_mode_groups - v_active_mode_groups
        )
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
Procedure:     core.assignEquipmentToModeGroup
Version:       1.0.0
Description:   Assigns equipment to a mode group (creates mapping)
Parameters:
    p_equipment_id UUID                 -- Required equipment ID
    p_mode_group_id UUID                -- Required mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Assignment result data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.assignEquipmentToModeGroup(
    p_equipment_id uuid,
    p_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_equipment_name varchar(255);
    v_mode_group_name varchar(255);
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment assigned to mode group successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_equipment_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment ID cannot be null';
        RETURN;
    END IF;
    
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- verify equipment exists
    SELECT equipment_name INTO v_equipment_name
    FROM core.equipment
    WHERE equipment_id = p_equipment_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Equipment not found';
        RETURN;
    END IF;
    
    -- verify mode group exists
    SELECT mode_group_name INTO v_mode_group_name
    FROM core.mode_group
    WHERE mode_group_id = p_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- check if assignment already exists
    IF EXISTS (
        SELECT 1 FROM core.equipment_mode_group_mapping 
        WHERE equipment_id = p_equipment_id AND mode_group_id = p_mode_group_id
    ) THEN
        "Status" := 'Error';
        "Message" := 'Equipment is already assigned to this mode group';
        RETURN;
    END IF;
    
    -- create the assignment
    INSERT INTO core.equipment_mode_group_mapping (equipment_id, mode_group_id)
    VALUES (p_equipment_id, p_mode_group_id);
    
    "Data" := jsonb_build_object(
        'equipment_id', p_equipment_id,
        'equipment_name', v_equipment_name,
        'mode_group_id', p_mode_group_id,
        'mode_group_name', v_mode_group_name,
        'assigned_at', now()
    );
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'Equipment is already assigned to this mode group';
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
Procedure:     core.unassignEquipmentFromModeGroup
Version:       1.0.0
Description:   Removes equipment assignment from a mode group
Parameters:
    p_equipment_id UUID                 -- Required equipment ID
    p_mode_group_id UUID                -- Required mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Unassignment result data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.unassignEquipmentFromModeGroup(
    p_equipment_id uuid,
    p_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_equipment_name varchar(255);
    v_mode_group_name varchar(255);
    v_deleted_count integer;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment unassigned from mode group successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_equipment_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment ID cannot be null';
        RETURN;
    END IF;
    
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- get names for response (even if assignment doesn't exist)
    SELECT equipment_name INTO v_equipment_name
    FROM core.equipment
    WHERE equipment_id = p_equipment_id;
    
    SELECT mode_group_name INTO v_mode_group_name
    FROM core.mode_group
    WHERE mode_group_id = p_mode_group_id;
    
    -- remove the assignment
    DELETE FROM core.equipment_mode_group_mapping
    WHERE equipment_id = p_equipment_id AND mode_group_id = p_mode_group_id;
    
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    IF v_deleted_count = 0 THEN
        "Status" := 'Error';
        "Message" := 'Equipment assignment not found';
        RETURN;
    END IF;
    
    "Data" := jsonb_build_object(
        'equipment_id', p_equipment_id,
        'equipment_name', COALESCE(v_equipment_name, 'Unknown'),
        'mode_group_id', p_mode_group_id,
        'mode_group_name', COALESCE(v_mode_group_name, 'Unknown'),
        'unassigned_at', now()
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
Procedure:     core.getEquipmentForModeGroup
Version:       1.0.0
Description:   Retrieves all equipment assigned to a specific mode group
Parameters:
    p_mode_group_id UUID                -- Required mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Equipment list as JSON array
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getEquipmentForModeGroup(
    p_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_results jsonb;
    v_mode_group_name varchar(255);
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment list retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- verify mode group exists
    SELECT mode_group_name INTO v_mode_group_name
    FROM core.mode_group
    WHERE mode_group_id = p_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- get equipment list
    SELECT COALESCE(jsonb_agg(
        jsonb_build_object(
            'equipment_id', e.equipment_id,
            'equipment_name', e.equipment_name,
            'equipment_type_id', e.equipment_type_id,
            'equipment_type_name', et.type_name,
            'equipment_enabled', e.equipment_enabled,
            'equipment_parent_id', e.equipment_parent_id,
            'created_at', e.created_at,
            'updated_at', e.updated_at
        ) ORDER BY e.equipment_name
    ), '[]'::jsonb)
    INTO v_results
    FROM core.equipment_mode_group_mapping emgm
    JOIN core.equipment e ON emgm.equipment_id = e.equipment_id
    JOIN core.equipment_type et ON e.equipment_type_id = et.type_id
    WHERE emgm.mode_group_id = p_mode_group_id;
    
    "Data" := jsonb_build_object(
        'mode_group_id', p_mode_group_id,
        'mode_group_name', v_mode_group_name,
        'equipment', v_results,
        'equipment_count', jsonb_array_length(v_results)
    );
    
    "Message" := 'Found ' || jsonb_array_length(v_results) || ' equipment item(s) for mode group: ' || v_mode_group_name;
    
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
Procedure:     core.bulkAssignEquipmentToModeGroup
Version:       1.0.0
Description:   Assigns multiple equipment items to a mode group in batch
Parameters:
    p_equipment_ids UUID[]              -- Required array of equipment IDs
    p_mode_group_id UUID                -- Required mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Bulk assignment results as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.bulkAssignEquipmentToModeGroup(
    p_equipment_ids uuid[],
    p_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_group_name varchar(255);
    v_valid_equipment_ids uuid[];
    v_existing_assignments uuid[];
    v_new_assignments uuid[];
    v_assigned_count integer := 0;
    v_skipped_count integer := 0;
    equipment_id uuid;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Bulk assignment completed';
    "Data" := NULL;
    
    -- p_validation
    IF p_equipment_ids IS NULL OR array_length(p_equipment_ids, 1) IS NULL THEN
        "Status" := 'Error';
        "Message" := 'No equipment IDs provided';
        RETURN;
    END IF;
    
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- verify mode group exists
    SELECT mode_group_name INTO v_mode_group_name
    FROM core.mode_group
    WHERE mode_group_id = p_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- get valid equipment ids (ones that actually exist)
    SELECT array_agg(equipment_id) INTO v_valid_equipment_ids
    FROM core.equipment
    WHERE equipment_id = ANY(p_equipment_ids);
    
    IF v_valid_equipment_ids IS NULL THEN
        "Status" := 'Error';
        "Message" := 'No valid equipment IDs found';
        RETURN;
    END IF;
    
    -- get existing assignments
    SELECT array_agg(equipment_id) INTO v_existing_assignments
    FROM core.equipment_mode_group_mapping
    WHERE equipment_id = ANY(v_valid_equipment_ids) 
    AND mode_group_id = p_mode_group_id;
    
    -- determine new assignments (exclude existing ones)
    IF v_existing_assignments IS NULL THEN
        v_new_assignments := v_valid_equipment_ids;
    ELSE
        SELECT array_agg(eq_id) INTO v_new_assignments
        FROM unnest(v_valid_equipment_ids) AS eq_id
        WHERE eq_id != ALL(v_existing_assignments);
    END IF;
    
    -- perform bulk insert for new assignments
    IF v_new_assignments IS NOT NULL AND array_length(v_new_assignments, 1) > 0 THEN
        INSERT INTO core.equipment_mode_group_mapping (equipment_id, mode_group_id)
        SELECT unnest(v_new_assignments), p_mode_group_id;
        
        GET DIAGNOSTICS v_assigned_count = ROW_COUNT;
    END IF;
    
    -- calculate skipped count
    v_skipped_count := COALESCE(array_length(v_existing_assignments, 1), 0);
    
    "Data" := jsonb_build_object(
        'mode_group_id', p_mode_group_id,
        'mode_group_name', v_mode_group_name,
        'requested_count', array_length(p_equipment_ids, 1),
        'valid_equipment_count', array_length(v_valid_equipment_ids, 1),
        'assigned_count', v_assigned_count,
        'skipped_count', v_skipped_count,
        'assigned_equipment_ids', COALESCE(v_new_assignments, ARRAY[]::uuid[]),
        'skipped_equipment_ids', COALESCE(v_existing_assignments, ARRAY[]::uuid[])
    );
    
    IF v_assigned_count > 0 AND v_skipped_count > 0 THEN
        "Message" := 'Partial success: ' || v_assigned_count || ' assigned, ' || v_skipped_count || ' skipped (already assigned)';
    ELSIF v_assigned_count > 0 THEN
        "Message" := 'Successfully assigned ' || v_assigned_count || ' equipment item(s)';
    ELSIF v_skipped_count > 0 THEN
        "Message" := 'All ' || v_skipped_count || ' equipment item(s) were already assigned';
        "Status" := 'Error';
    END IF;
    
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
Procedure:     core.modeGroupExists
Version:       1.0.0
Description:   Checks if mode group exists by name
Parameters:
    p_mode_group_name VARCHAR(255)      -- Required mode group name
Returns:       BOOLEAN                  -- TRUE if exists, FALSE otherwise
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.modeGroupExists(
    p_mode_group_name varchar(255)
)
RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    IF p_mode_group_name IS NULL OR trim(p_mode_group_name) = '' THEN
        RETURN FALSE;
    END IF;
    
    RETURN EXISTS (
        SELECT 1 FROM core.mode_group 
        WHERE mode_group_name = trim(p_mode_group_name)
    );
END;
$$;

----
-- mode stored procedures
----

/*
===========================================
Author:        hunter
Created:       2025-07-28 
Procedure:     core.getAllModes
Version:       1.0.0
Description:   Retrieves all modes with their mode group information
Parameters:
    None
Returns:       Table with mode_id, mode_description, mode_group_id, mode_group_name, created_at, updated_at
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getAllModes()
RETURNS TABLE (
    mode_id uuid,
    mode_description text,
    mode_group_id uuid,
    mode_group_name varchar(255),
    mode_group_description text,
    created_at timestamptz,
    updated_at timestamptz
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    RETURN QUERY
    SELECT 
        m.mode_id,
        m.mode_description,
        m.mode_group_id,
        mg.mode_group_name,
        mg.mode_group_description,
        m.created_at,
        m.updated_at
    FROM core.mode m
    JOIN core.mode_group mg ON m.mode_group_id = mg.mode_group_id
    ORDER BY mg.mode_group_name, m.mode_description;
    
EXCEPTION WHEN OTHERS THEN
    RAISE EXCEPTION 'Error retrieving modes: %', SQLERRM;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28 
Procedure:     core.getModeById
Version:       1.0.0
Description:   Retrieves specific mode by ID with mode group details
Parameters:
    p_mode_id UUID                      -- Required mode ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Mode data with mode group info as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getModeById(
    p_mode_id uuid,
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
    "Message" := 'Mode retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode ID cannot be null';
        RETURN;
    END IF;
    
    -- retrieve mode data with mode group info
    SELECT 
        m.mode_id,
        m.mode_description,
        m.mode_group_id,
        mg.mode_group_name,
        mg.mode_group_description,
        m.created_at,
        m.updated_at
    INTO v_result
    FROM core.mode m
    JOIN core.mode_group mg ON m.mode_group_id = mg.mode_group_id
    WHERE m.mode_id = p_mode_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode not found';
        RETURN;
    END IF;
    
    -- to json
    "Data" := jsonb_build_object(
        'mode_id', v_result.mode_id,
        'mode_description', v_result.mode_description,
        'mode_group', jsonb_build_object(
            'mode_group_id', v_result.mode_group_id,
            'mode_group_name', v_result.mode_group_name,
            'mode_group_description', v_result.mode_group_description
        ),
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
Procedure:     core.getModesByModeGroupId
Version:       1.0.0
Description:   Retrieves all modes for a specific mode group
Parameters:
    p_mode_group_id UUID                -- Required mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Modes array with mode group info as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getModesByModeGroupId(
    p_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_group RECORD;
    v_modes jsonb;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Modes retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- verify mode group exists and get info
    SELECT mode_group_name, mode_group_description
    INTO v_mode_group
    FROM core.mode_group
    WHERE mode_group_id = p_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- get the modes in the mode_group
    SELECT COALESCE(jsonb_agg(
        jsonb_build_object(
            'mode_id', m.mode_id,
            'mode_description', m.mode_description,
            'created_at', m.created_at,
            'updated_at', m.updated_at
        ) ORDER BY m.mode_description
    ), '[]'::jsonb)
    INTO v_modes
    FROM core.mode m
    WHERE m.mode_group_id = p_mode_group_id;
    
    -- build output
    "Data" := jsonb_build_object(
        'mode_group_id', p_mode_group_id,
        'mode_group_name', v_mode_group.mode_group_name,
        'mode_group_description', v_mode_group.mode_group_description,
        'modes', v_modes,
        'mode_count', jsonb_array_length(v_modes)
    );
    
    "Message" := 'Found ' || jsonb_array_length(v_modes) || ' mode(s) for mode group: ' || v_mode_group.mode_group_name;
    
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
Procedure:     core.getModeByDescription
Version:       1.0.0
Description:   Retrieves mode by description within a specific mode group
Parameters:
    p_mode_description TEXT             -- Required mode description
    p_mode_group_id UUID                -- Required mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Mode data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getModeByDescription(
    p_mode_description text,
    p_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_id uuid;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_description IS NULL OR trim(p_mode_description) = '' THEN
        "Status" := 'Error';
        "Message" := 'Mode description cannot be empty';
        RETURN;
    END IF;
    
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- get mode ID by description and mode group
    SELECT mode_id INTO v_mode_id
    FROM core.mode
    WHERE mode_description = trim(p_mode_description) 
    AND mode_group_id = p_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode not found in specified mode group';
        RETURN;
    END IF;
    
    -- Delegate to getModeById
    SELECT m."Status", m."Message", m."Data"
    INTO "Status", "Message", "Data"
    FROM core.getModeById(v_mode_id) m;
    
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
Procedure:     core.insertMode
Version:       1.0.0
Description:   Creates new mode with validation
Parameters:
    p_mode_description TEXT             -- Required mode description
    p_mode_group_id UUID                -- Required mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- New mode data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.insertMode(
    p_mode_description text,
    p_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_new_id uuid;
    v_trimmed_description text;
    v_mode_group_name varchar(255);
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode created successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_description IS NULL OR trim(p_mode_description) = '' THEN
        "Status" := 'Error';
        "Message" := 'Mode description cannot be empty';
        RETURN;
    END IF;
    
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    v_trimmed_description := trim(p_mode_description);
    
    -- validate length
    IF length(v_trimmed_description) < 2 OR length(v_trimmed_description) > 2048 THEN
        "Status" := 'Error';
        "Message" := 'Mode description must be between 2 and 2048 characters';
        RETURN;
    END IF;
    
    -- verifyy mode group exists
    SELECT mode_group_name INTO v_mode_group_name
    FROM core.mode_group
    WHERE mode_group_id = p_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- check for existing mode in the same mode group
    IF EXISTS (
        SELECT 1 FROM core.mode 
        WHERE mode_description = v_trimmed_description 
        AND mode_group_id = p_mode_group_id
    ) THEN
        "Status" := 'Error';
        "Message" := 'Mode description already exists in this mode group';
        RETURN;
    END IF;
    
    -- insert new mode
    INSERT INTO core.mode (mode_description, mode_group_id)
    VALUES (v_trimmed_description, p_mode_group_id)
    RETURNING mode_id INTO v_new_id;
    
    -- return created data
    "Data" := jsonb_build_object(
        'mode_id', v_new_id,
        'mode_description', v_trimmed_description,
        'mode_group', jsonb_build_object(
            'mode_group_id', p_mode_group_id,
            'mode_group_name', v_mode_group_name
        ),
        'created_at', now()
    );
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'Mode description already exists in this mode group';
        "Data" := NULL;
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Invalid mode group ID';
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
Procedure:     core.updateMode
Version:       1.0.0
Description:   Updates existing mode with validation
Parameters:
    p_mode_id UUID                      -- Required mode ID
    p_mode_description TEXT             -- Optional new mode description (null = no change)
    p_mode_group_id UUID                -- Optional new mode group ID (null = no change)
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Updated mode data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.updateMode(
    p_mode_id uuid,
    p_mode_description text DEFAULT NULL,
    p_mode_group_id uuid DEFAULT NULL,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_current_record RECORD;
    v_new_description text;
    v_new_mode_group_id uuid;
    v_changes_made boolean := false;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode updated successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode ID cannot be null';
        RETURN;
    END IF;
    
    -- get the current record
    SELECT mode_description, mode_group_id
    INTO v_current_record
    FROM core.mode
    WHERE mode_id = p_mode_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode not found';
        RETURN;
    END IF;
    
    -- determine what to update (COALESCE pattern for partial updates)
    v_new_description := COALESCE(trim(p_mode_description), v_current_record.mode_description);
    v_new_mode_group_id := COALESCE(p_mode_group_id, v_current_record.mode_group_id);
    
    -- validate new description if provided
    IF p_mode_description IS NOT NULL THEN
        IF trim(p_mode_description) = '' THEN
            "Status" := 'Error';
            "Message" := 'Mode description cannot be empty';
            RETURN;
        END IF;
        
        IF length(v_new_description) < 2 OR length(v_new_description) > 2048 THEN
            "Status" := 'Error';
            "Message" := 'Mode description must be between 2 and 2048 characters';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    -- validate new mode group if provided
    IF p_mode_group_id IS NOT NULL THEN
        IF NOT EXISTS (SELECT 1 FROM core.mode_group WHERE mode_group_id = p_mode_group_id) THEN
            "Status" := 'Error';
            "Message" := 'Invalid mode group ID';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    -- check for conflicts -- description must be unique within mode group
    IF EXISTS (
        SELECT 1 FROM core.mode 
        WHERE mode_description = v_new_description 
        AND mode_group_id = v_new_mode_group_id 
        AND mode_id != p_mode_id
    ) THEN
        "Status" := 'Error';
        "Message" := 'Mode description already exists in the target mode group';
        RETURN;
    END IF;
    
    -- update if changes were made
    IF v_changes_made THEN
        UPDATE core.mode
        SET mode_description = v_new_description,
            mode_group_id = v_new_mode_group_id,
            updated_at = now()
        WHERE mode_id = p_mode_id;
    ELSE
        "Message" := 'No changes were made to mode';
    END IF;
    
    -- output updated data using getModeById
    SELECT m."Data" INTO "Data"
    FROM core.getModeById(p_mode_id) m
    WHERE m."Status" = 'Success';
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'Mode description already exists in the target mode group';
        "Data" := NULL;
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Invalid mode group ID';
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
Procedure:     core.deleteMode
Version:       1.0.0
Description:   Safely deletes mode with validation
Parameters:
    p_mode_id UUID                      -- Required mode ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Operation result data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.deleteMode(
    p_mode_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_info RECORD;
    v_deleted_count integer;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode deleted successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode ID cannot be null';
        RETURN;
    END IF;
    
    -- mode mode info before deletion
    SELECT 
        m.mode_description,
        mg.mode_group_name,
        mg.mode_group_id
    INTO v_mode_info
    FROM core.mode m
    JOIN core.mode_group mg ON m.mode_group_id = mg.mode_group_id
    WHERE m.mode_id = p_mode_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode not found';
        RETURN;
    END IF;
    
    -- check if it's a default mode that shouldn't be deleted
    IF v_mode_info.mode_group_name = 'Default MES Mode Group' 
       AND v_mode_info.mode_description IN ('disabled', 'production', 'idle', 'change over') THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete default MES mode: ' || v_mode_info.mode_description;
        RETURN;
    END IF;
    
    -- dlete the mode
    DELETE FROM core.mode WHERE mode_id = p_mode_id;
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    IF v_deleted_count = 0 THEN
        "Status" := 'Error';
        "Message" := 'Mode could not be deleted';
        RETURN;
    END IF;
    
    "Data" := jsonb_build_object(
        'deleted_mode_id', p_mode_id,
        'deleted_mode_description', v_mode_info.mode_description,
        'mode_group_id', v_mode_info.mode_group_id,
        'mode_group_name', v_mode_info.mode_group_name
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
Procedure:     core.searchModes
Version:       1.0.0
Description:   Searches modes by description pattern across all mode groups
Parameters:
    p_search_term TEXT                  -- Optional search term (null returns all)
    p_mode_group_id UUID                -- Optional mode group filter (null = all groups)
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Search results as JSON array
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.searchModes(
    p_search_term text DEFAULT NULL,
    p_mode_group_id uuid DEFAULT NULL,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_results jsonb;
    v_where_clause text := '1=1';
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Search completed successfully';
    "Data" := NULL;
    
    -- build search query
    IF p_search_term IS NOT NULL AND trim(p_search_term) != '' THEN
        v_where_clause := v_where_clause || ' AND m.mode_description ILIKE ''%' || trim(p_search_term) || '%''';
    END IF;
    
    IF p_mode_group_id IS NOT NULL THEN
        v_where_clause := v_where_clause || ' AND m.mode_group_id = ''' || p_mode_group_id || '''';
    END IF;
    
    -- excute search
    EXECUTE format('
        SELECT COALESCE(jsonb_agg(
            jsonb_build_object(
                ''mode_id'', m.mode_id,
                ''mode_description'', m.mode_description,
                ''mode_group'', jsonb_build_object(
                    ''mode_group_id'', mg.mode_group_id,
                    ''mode_group_name'', mg.mode_group_name,
                    ''mode_group_description'', mg.mode_group_description
                ),
                ''created_at'', m.created_at,
                ''updated_at'', m.updated_at
            ) ORDER BY mg.mode_group_name, m.mode_description
        ), ''[]''::jsonb)
        FROM core.mode m
        JOIN core.mode_group mg ON m.mode_group_id = mg.mode_group_id
        WHERE %s', v_where_clause)
    INTO v_results;
    
    "Data" := v_results;
    
    IF jsonb_array_length("Data") = 0 THEN
        "Message" := 'No modes found matching search criteria';
    ELSE
        "Message" := 'Found ' || jsonb_array_length("Data") || ' mode(s)';
    END IF;
    
EXCEPTION WHEN OTHERS THEN
    "Status" := 'Error';
    "Message" := 'Unexpected error: ' || SQLERRM;
    "Data" := '[]'::jsonb;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28 
Procedure:     core.bulkInsertModes
Version:       1.0.0
Description:   Creates multiple modes for a mode group in batch
Parameters:
    p_mode_descriptions TEXT[]          -- Required **array** of mode descriptions
    p_mode_group_id UUID                -- Required mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Bulk insert results as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.bulkInsertModes(
    p_mode_descriptions text[],
    p_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_group_name varchar(255);
    v_created_modes jsonb := '[]'::jsonb;
    v_failed_modes text[] := ARRAY[]::text[];
    v_created_count integer := 0;
    mode_desc text;
    v_new_mode_id uuid;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Bulk mode creation completed';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_descriptions IS NULL OR array_length(p_mode_descriptions, 1) IS NULL THEN
        "Status" := 'Error';
        "Message" := 'No mode descriptions provided';
        RETURN;
    END IF;
    
    IF p_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- does the mode group exist
    SELECT mode_group_name INTO v_mode_group_name
    FROM core.mode_group
    WHERE mode_group_id = p_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- process each mode description independently
    FOREACH mode_desc IN ARRAY p_mode_descriptions LOOP
        BEGIN
            -- validate and trim description
            IF mode_desc IS NULL OR trim(mode_desc) = '' THEN
                v_failed_modes := array_append(v_failed_modes, COALESCE(mode_desc, 'NULL') || ': Description cannot be empty');
                CONTINUE;
            END IF;
            
            mode_desc := trim(mode_desc);
            
            -- validate length
            IF length(mode_desc) < 2 OR length(mode_desc) > 2048 THEN
                v_failed_modes := array_append(v_failed_modes, mode_desc || ': Description must be between 2 and 2048 characters');
                CONTINUE;
            END IF;
            
            -- check for existing mode
            IF EXISTS (
                SELECT 1 FROM core.mode 
                WHERE mode_description = mode_desc AND mode_group_id = p_mode_group_id
            ) THEN
                v_failed_modes := array_append(v_failed_modes, mode_desc || ': Already exists in this mode group');
                CONTINUE;
            END IF;
            
            -- insert the mode
            INSERT INTO core.mode (mode_description, mode_group_id)
            VALUES (mode_desc, p_mode_group_id)
            RETURNING mode_id INTO v_new_mode_id;
            
            -- add to created modes
            v_created_modes := v_created_modes || jsonb_build_object(
                'mode_id', v_new_mode_id,
                'mode_description', mode_desc
            );
            
            v_created_count := v_created_count + 1;
            
        EXCEPTION WHEN OTHERS THEN
            v_failed_modes := array_append(v_failed_modes, mode_desc || ': ' || SQLERRM);
        END;
    END LOOP;
    
    "Data" := jsonb_build_object(
        'mode_group_id', p_mode_group_id,
        'mode_group_name', v_mode_group_name,
        'requested_count', array_length(p_mode_descriptions, 1),
        'created_count', v_created_count,
        'failed_count', array_length(v_failed_modes, 1),
        'created_modes', v_created_modes,
        'failed_modes', v_failed_modes
    );
    
    -- overall status
    IF v_created_count = 0 THEN
        "Status" := 'Error';
        "Message" := 'No modes were created';
    ELSIF array_length(v_failed_modes, 1) > 0 THEN
        "Message" := 'Partial success: ' || v_created_count || ' created, ' || array_length(v_failed_modes, 1) || ' failed';
    ELSE
        "Message" := 'Successfully created ' || v_created_count || ' mode(s)';
    END IF;
    
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
Procedure:     core.modeExists
Version:       1.0.0
Description:   Checks if mode exists by description and mode group
Parameters:
    p_mode_description TEXT             -- Required mode description
    p_mode_group_id UUID                -- Required mode group ID
Returns:       BOOLEAN                  -- TRUE if exists, FALSE otherwise
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.modeExists(
    p_mode_description text,
    p_mode_group_id uuid
)
RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    IF p_mode_description IS NULL OR trim(p_mode_description) = '' OR p_mode_group_id IS NULL THEN
        RETURN FALSE;
    END IF;
    
    RETURN EXISTS (
        SELECT 1 FROM core.mode 
        WHERE mode_description = trim(p_mode_description)
        AND mode_group_id = p_mode_group_id
    );
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28 
Procedure:     core.getModeUsageStats
Version:       1.0.0
Description:   Retrieves detailed usage statistics for all modes
Parameters:
    None
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Usage statistics as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getModeUsageStats(
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_results jsonb;
    v_summary jsonb;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode usage statistics retrieved successfully';
    "Data" := NULL;
    
    -- detailed mode statistics
    SELECT jsonb_agg(
        jsonb_build_object(
            'mode_id', m.mode_id,
            'mode_description', m.mode_description,
            'mode_group', jsonb_build_object(
                'mode_group_id', mg.mode_group_id,
                'mode_group_name', mg.mode_group_name,
                'mode_group_description', mg.mode_group_description
            ),
            'is_default_mode', (mg.mode_group_name = 'Default MES Mode Group'),
            'created_at', m.created_at,
            'updated_at', m.updated_at
        ) ORDER BY mg.mode_group_name, m.mode_description
    )
    INTO v_results
    FROM core.mode m
    JOIN core.mode_group mg ON m.mode_group_id = mg.mode_group_id;
    
    -- summart stats
    SELECT jsonb_build_object(
        'total_modes', COUNT(*),
        'default_modes', COUNT(CASE WHEN mg.mode_group_name = 'Default MES Mode Group' THEN 1 END),
        'custom_modes', COUNT(CASE WHEN mg.mode_group_name != 'Default MES Mode Group' THEN 1 END),
        'mode_groups_with_modes', COUNT(DISTINCT m.mode_group_id),
        'total_mode_groups', (SELECT COUNT(*) FROM core.mode_group)
    )
    INTO v_summary
    FROM core.mode m
    JOIN core.mode_group mg ON m.mode_group_id = mg.mode_group_id;
    
    "Data" := jsonb_build_object(
        'modes', COALESCE(v_results, '[]'::jsonb),
        'summary', v_summary
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
Procedure:     core.moveModeToModeGroup
Version:       1.0.0
Description:   Moves a mode from one mode group to another
Parameters:
    p_mode_id UUID                      -- Required mode ID
    p_target_mode_group_id UUID         -- Required target mode group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Move operation result as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.moveModeToModeGroup(
    p_mode_id uuid,
    p_target_mode_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_info RECORD;
    v_target_mode_group_name varchar(255);
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Mode moved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Mode ID cannot be null';
        RETURN;
    END IF;
    
    IF p_target_mode_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Target mode group ID cannot be null';
        RETURN;
    END IF;
    
    -- get current mode info
    SELECT 
        m.mode_description,
        m.mode_group_id,
        mg.mode_group_name
    INTO v_mode_info
    FROM core.mode m
    JOIN core.mode_group mg ON m.mode_group_id = mg.mode_group_id
    WHERE m.mode_id = p_mode_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode not found';
        RETURN;
    END IF;
    
    -- check if already in target mode group
    IF v_mode_info.mode_group_id = p_target_mode_group_id THEN
        "Status" := 'Error';
        "Message" := 'Mode is already in the target mode group';
        RETURN;
    END IF;
    
    -- verify target mode group exists
    SELECT mode_group_name INTO v_target_mode_group_name
    FROM core.mode_group
    WHERE mode_group_id = p_target_mode_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Target mode group not found';
        RETURN;
    END IF;
    
    -- check if mode description exists in target mode group
    IF EXISTS (
        SELECT 1 FROM core.mode 
        WHERE mode_description = v_mode_info.mode_description 
        AND mode_group_id = p_target_mode_group_id
    ) THEN
        "Status" := 'Error';
        "Message" := 'Mode description already exists in target mode group';
        RETURN;
    END IF;
    
    -- prevent moving default modes out of default mode group
    IF v_mode_info.mode_group_name = 'Default MES Mode Group' 
       AND v_mode_info.mode_description IN ('disabled', 'production', 'idle', 'change over') THEN
        "Status" := 'Error';
        "Message" := 'Cannot move default MES mode: ' || v_mode_info.mode_description;
        RETURN;
    END IF;
    
    -- now we move the mode
    UPDATE core.mode
    SET mode_group_id = p_target_mode_group_id,
        updated_at = now()
    WHERE mode_id = p_mode_id;
    
    "Data" := jsonb_build_object(
        'mode_id', p_mode_id,
        'mode_description', v_mode_info.mode_description,
        'source_mode_group', jsonb_build_object(
            'mode_group_id', v_mode_info.mode_group_id,
            'mode_group_name', v_mode_info.mode_group_name
        ),
        'target_mode_group', jsonb_build_object(
            'mode_group_id', p_target_mode_group_id,
            'mode_group_name', v_target_mode_group_name
        ),
        'moved_at', now()
    );
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'Mode description already exists in target mode group';
        "Data" := NULL;
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Invalid target mode group ID';
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
Procedure:     core.getModesByModeGroupName
Version:       1.0.0
Description:   Retrieves all modes for a mode group by name
Parameters:
    p_mode_group_name VARCHAR(255)      -- Required mode group name
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Modes array with mode group info as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getModesByModeGroupName(
    p_mode_group_name varchar(255),
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_mode_group_id uuid;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Modes retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_mode_group_name IS NULL OR trim(p_mode_group_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'Mode group name cannot be empty';
        RETURN;
    END IF;
    
    -- get by mode group id
    SELECT mode_group_id INTO v_mode_group_id
    FROM core.mode_group
    WHERE mode_group_name = trim(p_mode_group_name);
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Mode group not found';
        RETURN;
    END IF;
    
    -- getModesByModeGroupId
    SELECT mg."Status", mg."Message", mg."Data"
    INTO "Status", "Message", "Data"
    FROM core.getModesByModeGroupId(v_mode_group_id) mg;
    
EXCEPTION WHEN OTHERS THEN
    "Status" := 'Error';
    "Message" := 'Unexpected error: ' || SQLERRM;
    "Data" := NULL;
END;
$$;