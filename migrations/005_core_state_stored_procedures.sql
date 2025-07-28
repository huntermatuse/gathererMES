/*
===========================================
Author:        hunter
Created:       2025-07-28
Schema:        core
Version:       1.0.0
Description:   State Group and State Comments, Procedures and Constraints 
Change Log:
    2025-07-28  hunter  init
===========================================
*/

COMMENT ON TABLE core.state_group IS 'Groups of equipment states, typically organized by equipment type or PLC system';
COMMENT ON TABLE core.state IS 'Individual equipment states with numeric codes and descriptions';

ALTER TABLE core.state_group 
ADD CONSTRAINT chk_state_group_name_not_empty 
CHECK (length(trim(state_group_name)) > 0);

ALTER TABLE core.state_group 
ADD CONSTRAINT chk_state_group_name_length 
CHECK (length(state_group_name) BETWEEN 2 AND 255);

ALTER TABLE core.state_group 
ADD CONSTRAINT chk_state_group_description_not_empty 
CHECK (length(trim(state_group_description)) > 0);

ALTER TABLE core.state_group 
ADD CONSTRAINT chk_state_group_description_length 
CHECK (length(state_group_description) BETWEEN 5 AND 2048);

ALTER TABLE core.state 
ADD CONSTRAINT chk_state_code_range 
CHECK (state_code >= 0 AND state_code <= 9999);

ALTER TABLE core.state 
ADD CONSTRAINT chk_state_description_not_empty 
CHECK (length(trim(state_description)) > 0);

ALTER TABLE core.state 
ADD CONSTRAINT chk_state_description_length 
CHECK (length(state_description) BETWEEN 2 AND 2048);

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getAllStateGroups
Version:       1.0.0
Description:   Retrieves all state groups with state counts and equipment associations
Parameters:
    None
Returns:       Table with state_group_id, state_group_name, state_group_description, state_count, equipment_count, created_at, updated_at
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getAllStateGroups()
RETURNS TABLE (
    state_group_id uuid,
    state_group_name varchar(255),
    state_group_description text,
    state_count bigint,
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
        sg.state_group_id,
        sg.state_group_name,
        sg.state_group_description,
        COALESCE(COUNT(DISTINCT s.state_id), 0) as state_count,
        COALESCE(COUNT(DISTINCT esgm.equipment_id), 0) as equipment_count,
        sg.created_at,
        sg.updated_at
    FROM core.state_group sg
    LEFT JOIN core.state s ON sg.state_group_id = s.state_group_id
    LEFT JOIN core.equipment_state_group_mapping esgm ON sg.state_group_id = esgm.state_group_id
    GROUP BY sg.state_group_id, sg.state_group_name, sg.state_group_description, sg.created_at, sg.updated_at
    ORDER BY sg.state_group_name;
    
EXCEPTION WHEN OTHERS THEN
    RAISE EXCEPTION 'Error retrieving state groups: %', SQLERRM;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getStateGroupById
Version:       1.0.0
Description:   Retrieves specific state group by ID
Parameters:
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- State group data with states and equipment as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getStateGroupById(
    p_state_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_group RECORD;
    v_states jsonb;
    v_equipment jsonb;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'State group retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- basic state group info
    SELECT 
        sg.state_group_id,
        sg.state_group_name,
        sg.state_group_description,
        sg.created_at,
        sg.updated_at
    INTO v_state_group
    FROM core.state_group sg
    WHERE sg.state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- response
    "Data" := jsonb_build_object(
        'state_group_id', v_state_group.state_group_id,
        'state_group_name', v_state_group.state_group_name,
        'state_group_description', v_state_group.state_group_description,
        'created_at', v_state_group.created_at,
        'updated_at', v_state_group.updated_at
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
Procedure:     core.getStateGroupByIdContext
Version:       1.0.0
Description:   Retrieves specific state group by ID with associated states and equipment
Parameters:
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- State group data with states and equipment as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getStateGroupByIdContext(
    p_state_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_group RECORD;
    v_states jsonb;
    v_equipment jsonb;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'State group retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- basic state group info
    SELECT 
        sg.state_group_id,
        sg.state_group_name,
        sg.state_group_description,
        sg.created_at,
        sg.updated_at
    INTO v_state_group
    FROM core.state_group sg
    WHERE sg.state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- associated states
    SELECT COALESCE(jsonb_agg(
        jsonb_build_object(
            'state_id', s.state_id,
            'state_code', s.state_code,
            'state_description', s.state_description,
            'created_at', s.created_at,
            'updated_at', s.updated_at
        ) ORDER BY s.state_code
    ), '[]'::jsonb)
    INTO v_states
    FROM core.state s
    WHERE s.state_group_id = p_state_group_id;
    
    -- equipment
    SELECT COALESCE(jsonb_agg(
        jsonb_build_object(
            'equipment_id', e.equipment_id,
            'equipment_name', e.equipment_name,
            'equipment_type', et.type_name,
            'equipment_enabled', e.equipment_enabled
        ) ORDER BY e.equipment_name
    ), '[]'::jsonb)
    INTO v_equipment
    FROM core.equipment_state_group_mapping esgm
    JOIN core.equipment e ON esgm.equipment_id = e.equipment_id
    JOIN core.equipment_type et ON e.equipment_type_id = et.type_id
    WHERE esgm.state_group_id = p_state_group_id;
    
    -- a complete response
    "Data" := jsonb_build_object(
        'state_group_id', v_state_group.state_group_id,
        'state_group_name', v_state_group.state_group_name,
        'state_group_description', v_state_group.state_group_description,
        'created_at', v_state_group.created_at,
        'updated_at', v_state_group.updated_at,
        'states', v_states,
        'equipment', v_equipment,
        'state_count', jsonb_array_length(v_states),
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
Procedure:     core.getStateGroupByName
Version:       1.0.0
Description:   Retrieves specific state group by name
Parameters:
    p_state_group_name VARCHAR(255)     -- Required state group name
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- State group data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getStateGroupByName(
    p_state_group_name varchar(255),
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_group_id uuid;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'State group retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_group_name IS NULL OR trim(p_state_group_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'State group name cannot be empty';
        RETURN;
    END IF;
    
    -- get state by id
    SELECT state_group_id INTO v_state_group_id
    FROM core.state_group
    WHERE state_group_name = trim(p_state_group_name);
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- Delegate to getStateGroupById
    SELECT sg."Status", sg."Message", sg."Data"
    INTO "Status", "Message", "Data"
    FROM core.getStateGroupById(v_state_group_id) sg;
    
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
Procedure:     core.getStateGroupByNameContext
Version:       1.0.0
Description:   Retrieves specific state group by name with context 
Parameters:
    p_state_group_name VARCHAR(255)     -- Required state group name
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- State group data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getStateGroupByNameContext(
    p_state_group_name varchar(255),
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_group_id uuid;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'State group retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_group_name IS NULL OR trim(p_state_group_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'State group name cannot be empty';
        RETURN;
    END IF;
    
    -- get state by id
    SELECT state_group_id INTO v_state_group_id
    FROM core.state_group
    WHERE state_group_name = trim(p_state_group_name);
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- sends to getStateGroupByIdContext
    SELECT sg."Status", sg."Message", sg."Data"
    INTO "Status", "Message", "Data"
    FROM core.getStateGroupByIdContext(v_state_group_id) sg;
    
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
Procedure:     core.insertStateGroup
Version:       1.0.0
Description:   Creates new state group with validation
Parameters:
    p_state_group_name VARCHAR(255)     -- Required state group name
    p_state_group_description TEXT      -- Required state group description
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- New state group data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.insertStateGroup(
    p_state_group_name varchar(255),
    p_state_group_description text,
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
    "Message" := 'State group created successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_group_name IS NULL OR trim(p_state_group_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'State group name cannot be empty';
        RETURN;
    END IF;
    
    IF p_state_group_description IS NULL OR trim(p_state_group_description) = '' THEN
        "Status" := 'Error';
        "Message" := 'State group description cannot be empty';
        RETURN;
    END IF;
    
    v_trimmed_name := trim(p_state_group_name);
    v_trimmed_description := trim(p_state_group_description);
    
    -- validation sizes
    IF length(v_trimmed_name) < 2 OR length(v_trimmed_name) > 255 THEN
        "Status" := 'Error';
        "Message" := 'State group name must be between 2 and 255 characters';
        RETURN;
    END IF;
    
    IF length(v_trimmed_description) < 5 OR length(v_trimmed_description) > 2048 THEN
        "Status" := 'Error';
        "Message" := 'State group description must be between 5 and 2048 characters';
        RETURN;
    END IF;
    
    -- state group exist
    IF EXISTS (SELECT 1 FROM core.state_group WHERE state_group_name = v_trimmed_name) THEN
        "Status" := 'Error';
        "Message" := 'State group name already exists';
        RETURN;
    END IF;
    
    -- insert new state group
    INSERT INTO core.state_group (state_group_name, state_group_description)
    VALUES (v_trimmed_name, v_trimmed_description)
    RETURNING state_group_id INTO v_new_id;
    
    -- data
    "Data" := jsonb_build_object(
        'state_group_id', v_new_id,
        'state_group_name', v_trimmed_name,
        'state_group_description', v_trimmed_description,
        'state_count', 0,
        'equipment_count', 0,
        'created_at', now()
    );
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'State group name already exists';
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
Procedure:     core.updateStateGroup
Version:       1.0.0
Description:   Updates existing state group with validation
Parameters:
    p_state_group_id UUID               -- Required state group ID
    p_state_group_name VARCHAR(255)     -- Optional new state group name (null = no change)
    p_state_group_description TEXT      -- Optional new state group description (null = no change)
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Updated state group data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.updateStateGroup(
    p_state_group_id uuid,
    p_state_group_name varchar(255) DEFAULT NULL,
    p_state_group_description text DEFAULT NULL,
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
    "Message" := 'State group updated successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- current values
    SELECT state_group_name, state_group_description
    INTO v_current_record
    FROM core.state_group
    WHERE state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- what to update
    v_new_name := COALESCE(trim(p_state_group_name), v_current_record.state_group_name);
    v_new_description := COALESCE(trim(p_state_group_description), v_current_record.state_group_description);
    
    -- validate the new values with if to check provided
    IF p_state_group_name IS NOT NULL THEN
        IF trim(p_state_group_name) = '' THEN
            "Status" := 'Error';
            "Message" := 'State group name cannot be empty';
            RETURN;
        END IF;
        
        IF length(v_new_name) < 2 OR length(v_new_name) > 255 THEN
            "Status" := 'Error';
            "Message" := 'State group name must be between 2 and 255 characters';
            RETURN;
        END IF;
        
        -- name conflicts
        IF EXISTS (
            SELECT 1 FROM core.state_group 
            WHERE state_group_name = v_new_name AND state_group_id != p_state_group_id
        ) THEN
            "Status" := 'Error';
            "Message" := 'State group name already exists';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    IF p_state_group_description IS NOT NULL THEN
        IF trim(p_state_group_description) = '' THEN
            "Status" := 'Error';
            "Message" := 'State group description cannot be empty';
            RETURN;
        END IF;
        
        IF length(v_new_description) < 5 OR length(v_new_description) > 2048 THEN
            "Status" := 'Error';
            "Message" := 'State group description must be between 5 and 2048 characters';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    -- update if changes were made
    IF v_changes_made THEN
        UPDATE core.state_group
        SET state_group_name = v_new_name,
            state_group_description = v_new_description,
            updated_at = now()
        WHERE state_group_id = p_state_group_id;
    ELSE
        "Message" := 'No changes were made to state group';
    END IF;
    
    -- return updated data using getStateGroupById
    SELECT sg."Data" INTO "Data"
    FROM core.getStateGroupById(p_state_group_id) sg
    WHERE sg."Status" = 'Success';
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'State group name already exists';
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
Procedure:     core.deleteStateGroup
Version:       1.0.0
Description:   Safely deletes state group with dependency checks
Parameters:
    p_state_group_id UUID               -- Required state group ID
    p_force_delete BOOLEAN=FALSE        -- Optional force delete flag
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Operation result data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.deleteStateGroup(
    p_state_group_id uuid,
    p_force_delete boolean DEFAULT FALSE,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_count integer;
    v_equipment_count integer;
    v_state_group_name varchar(255);
    v_deleted_count integer;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'State group deleted successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- state group info and dependency counts
    SELECT 
        sg.state_group_name,
        COALESCE(COUNT(DISTINCT s.state_id), 0),
        COALESCE(COUNT(DISTINCT esgm.equipment_id), 0)
    INTO v_state_group_name, v_state_count, v_equipment_count
    FROM core.state_group sg
    LEFT JOIN core.state s ON sg.state_group_id = s.state_group_id
    LEFT JOIN core.equipment_state_group_mapping esgm ON sg.state_group_id = esgm.state_group_id
    WHERE sg.state_group_id = p_state_group_id
    GROUP BY sg.state_group_name;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- default state group will not be deleted
    IF v_state_group_name = 'Default MES State Group' THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete default MES state group';
        RETURN;
    END IF;
    
    -- check for dependencies
    IF (v_state_count > 0 OR v_equipment_count > 0) AND NOT p_force_delete THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete state group: ' || v_state_count || ' state(s) and ' || v_equipment_count || ' equipment association(s) exist. Use force_delete=true to override.';
        RETURN;
    END IF;
    
    -- force delete remove dependencies first
    IF p_force_delete THEN
        -- equipment many to many
        DELETE FROM core.equipment_state_group_mapping 
        WHERE state_group_id = p_state_group_id;
        
        -- remove states
        DELETE FROM core.state 
        WHERE state_group_id = p_state_group_id;
    END IF;
    
    -- delete the state group
    DELETE FROM core.state_group WHERE state_group_id = p_state_group_id;
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    IF v_deleted_count = 0 THEN
        "Status" := 'Error';
        "Message" := 'State group could not be deleted';
        RETURN;
    END IF;
    
    "Data" := jsonb_build_object(
        'deleted_state_group_id', p_state_group_id,
        'deleted_state_group_name', v_state_group_name,
        'affected_state_count', v_state_count,
        'affected_equipment_count', v_equipment_count,
        'force_delete_used', p_force_delete
    );
    
EXCEPTION 
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete state group: dependencies still exist';
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
Procedure:     core.searchStateGroups
Version:       1.0.0
Description:   Searches state groups by name or description pattern
Parameters:
    p_search_term TEXT                  -- Optional search term (null returns all)
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Search results as JSON array
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.searchStateGroups(
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
        -- all state groups returned
        SELECT jsonb_agg(
            jsonb_build_object(
                'state_group_id', sg.state_group_id,
                'state_group_name', sg.state_group_name,
                'state_group_description', sg.state_group_description,
                'state_count', COALESCE(COUNT(DISTINCT s.state_id), 0),
                'equipment_count', COALESCE(COUNT(DISTINCT esgm.equipment_id), 0),
                'created_at', sg.created_at,
                'updated_at', sg.updated_at
            ) ORDER BY sg.state_group_name
        )
        INTO v_results
        FROM core.state_group sg
        LEFT JOIN core.state s ON sg.state_group_id = s.state_group_id
        LEFT JOIN core.equipment_state_group_mapping esgm ON sg.state_group_id = esgm.state_group_id
        GROUP BY sg.state_group_id, sg.state_group_name, sg.state_group_description, sg.created_at, sg.updated_at;
    ELSE
        -- by pattern in name or description
        SELECT jsonb_agg(
            jsonb_build_object(
                'state_group_id', sg.state_group_id,
                'state_group_name', sg.state_group_name,
                'state_group_description', sg.state_group_description,
                'state_count', COALESCE(COUNT(DISTINCT s.state_id), 0),
                'equipment_count', COALESCE(COUNT(DISTINCT esgm.equipment_id), 0),
                'created_at', sg.created_at,
                'updated_at', sg.updated_at
            ) ORDER BY sg.state_group_name
        )
        INTO v_results
        FROM core.state_group sg
        LEFT JOIN core.state s ON sg.state_group_id = s.state_group_id
        LEFT JOIN core.equipment_state_group_mapping esgm ON sg.state_group_id = esgm.state_group_id
        WHERE sg.state_group_name ILIKE '%' || trim(p_search_term) || '%'
           OR sg.state_group_description ILIKE '%' || trim(p_search_term) || '%'
        GROUP BY sg.state_group_id, sg.state_group_name, sg.state_group_description, sg.created_at, sg.updated_at;
    END IF;
    
    "Data" := COALESCE(v_results, '[]'::jsonb);
    
    IF jsonb_array_length("Data") = 0 THEN
        "Message" := 'No state groups found matching search criteria';
    ELSE
        "Message" := 'Found ' || jsonb_array_length("Data") || ' state group(s)';
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
Procedure:     core.getStateGroupUsageStats
Version:       1.0.0
Description:   Retrieves detailed usage stats for all state groups
Parameters:
    None
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Usage statistics as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getStateGroupUsageStats(
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_results jsonb;
    v_total_state_groups integer;
    v_active_state_groups integer;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Usage statistics retrieved successfully';
    "Data" := NULL;
    
    -- stats
    SELECT jsonb_agg(
        jsonb_build_object(
            'state_group_id', sg.state_group_id,
            'state_group_name', sg.state_group_name,
            'state_group_description', sg.state_group_description,
            'state_count', COALESCE(COUNT(DISTINCT s.state_id), 0),
            'equipment_count', COALESCE(COUNT(DISTINCT esgm.equipment_id), 0),
            'enabled_equipment_count', COALESCE(COUNT(DISTINCT CASE WHEN e.equipment_enabled THEN esgm.equipment_id END), 0),
            'state_code_range', CASE 
                WHEN COUNT(s.state_id) > 0 THEN 
                    jsonb_build_object(
                        'min_code', MIN(s.state_code),
                        'max_code', MAX(s.state_code)
                    )
                ELSE NULL
            END,
            'is_default_group', (sg.state_group_name = 'Default MES State Group'),
            'created_at', sg.created_at,
            'updated_at', sg.updated_at
        ) ORDER BY sg.state_group_name
    )
    INTO v_results
    FROM core.state_group sg
    LEFT JOIN core.state s ON sg.state_group_id = s.state_group_id
    LEFT JOIN core.equipment_state_group_mapping esgm ON sg.state_group_id = esgm.state_group_id
    LEFT JOIN core.equipment e ON esgm.equipment_id = e.equipment_id
    GROUP BY sg.state_group_id, sg.state_group_name, sg.state_group_description, sg.created_at, sg.updated_at;
    
    -- counts
    SELECT 
        COUNT(*),
        COUNT(CASE WHEN sg.state_group_id IN (
            SELECT DISTINCT esgm.state_group_id 
            FROM core.equipment_state_group_mapping esgm
        ) THEN 1 END)
    INTO v_total_state_groups, v_active_state_groups
    FROM core.state_group sg;
    
    "Data" := jsonb_build_object(
        'state_groups', COALESCE(v_results, '[]'::jsonb),
        'summary', jsonb_build_object(
            'total_state_groups', v_total_state_groups,
            'active_state_groups', v_active_state_groups,
            'unused_state_groups', v_total_state_groups - v_active_state_groups
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
Procedure:     core.assignEquipmentToStateGroup
Version:       1.0.0
Description:   Assigns equipment to a state group (creates mapping)
Parameters:
    p_equipment_id UUID                 -- Required equipment ID
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Assignment result data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.assignEquipmentToStateGroup(
    p_equipment_id uuid,
    p_state_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_equipment_name varchar(255);
    v_state_group_name varchar(255);
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment assigned to state group successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_equipment_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment ID cannot be null';
        RETURN;
    END IF;
    
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- does equipment exists
    SELECT equipment_name INTO v_equipment_name
    FROM core.equipment
    WHERE equipment_id = p_equipment_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Equipment not found';
        RETURN;
    END IF;
    
    -- does state group exists
    SELECT state_group_name INTO v_state_group_name
    FROM core.state_group
    WHERE state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    --does the assignment already exist
    IF EXISTS (
        SELECT 1 FROM core.equipment_state_group_mapping 
        WHERE equipment_id = p_equipment_id AND state_group_id = p_state_group_id
    ) THEN
        "Status" := 'Error';
        "Message" := 'Equipment is already assigned to this state group';
        RETURN;
    END IF;
    
    -- new assignment
    INSERT INTO core.equipment_state_group_mapping (equipment_id, state_group_id)
    VALUES (p_equipment_id, p_state_group_id);
    
    "Data" := jsonb_build_object(
        'equipment_id', p_equipment_id,
        'equipment_name', v_equipment_name,
        'state_group_id', p_state_group_id,
        'state_group_name', v_state_group_name,
        'assigned_at', now()
    );
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'Equipment is already assigned to this state group';
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
Procedure:     core.unassignEquipmentFromStateGroup
Version:       1.0.0
Description:   Removes equipment assignment from a state group
Parameters:
    p_equipment_id UUID                 -- Required equipment ID
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Unassignment result data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.unassignEquipmentFromStateGroup(
    p_equipment_id uuid,
    p_state_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_equipment_name varchar(255);
    v_state_group_name varchar(255);
    v_deleted_count integer;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment unassigned from state group successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_equipment_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment ID cannot be null';
        RETURN;
    END IF;
    
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- get names
    SELECT equipment_name INTO v_equipment_name
    FROM core.equipment
    WHERE equipment_id = p_equipment_id;
    
    SELECT state_group_name INTO v_state_group_name
    FROM core.state_group
    WHERE state_group_id = p_state_group_id;
    
    -- remove the assignment
    DELETE FROM core.equipment_state_group_mapping
    WHERE equipment_id = p_equipment_id AND state_group_id = p_state_group_id;
    
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    IF v_deleted_count = 0 THEN
        "Status" := 'Error';
        "Message" := 'Equipment assignment not found';
        RETURN;
    END IF;
    
    "Data" := jsonb_build_object(
        'equipment_id', p_equipment_id,
        'equipment_name', COALESCE(v_equipment_name, 'Unknown'),
        'state_group_id', p_state_group_id,
        'state_group_name', COALESCE(v_state_group_name, 'Unknown'),
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
Procedure:     core.getEquipmentForStateGroup
Version:       1.0.0
Description:   Retrieves all equipment assigned to a specific state group
Parameters:
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Equipment list as JSON array
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getEquipmentForStateGroup(
    p_state_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_results jsonb;
    v_state_group_name varchar(255);
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment list retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- state group exists
    SELECT state_group_name INTO v_state_group_name
    FROM core.state_group
    WHERE state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- equipment list
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
    FROM core.equipment_state_group_mapping esgm
    JOIN core.equipment e ON esgm.equipment_id = e.equipment_id
    JOIN core.equipment_type et ON e.equipment_type_id = et.type_id
    WHERE esgm.state_group_id = p_state_group_id;
    
    "Data" := jsonb_build_object(
        'state_group_id', p_state_group_id,
        'state_group_name', v_state_group_name,
        'equipment', v_results,
        'equipment_count', jsonb_array_length(v_results)
    );
    
    "Message" := 'Found ' || jsonb_array_length(v_results) || ' equipment item(s) for state group: ' || v_state_group_name;
    
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
Procedure:     core.bulkAssignEquipmentToStateGroup
Version:       1.0.0
Description:   Assigns multiple equipment items to a state group in batch
Parameters:
    p_equipment_ids UUID[]              -- Required array of equipment IDs
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Bulk assignment results as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.bulkAssignEquipmentToStateGroup(
    p_equipment_ids uuid[],
    p_state_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_group_name varchar(255);
    v_valid_equipment_ids uuid[];
    v_existing_assignments uuid[];
    v_new_assignments uuid[];
    v_assigned_count integer := 0;
    v_skipped_count integer := 0;
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
    
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- state group exists
    SELECT state_group_name INTO v_state_group_name
    FROM core.state_group
    WHERE state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- valid eq_ids
    SELECT array_agg(equipment_id) INTO v_valid_equipment_ids
    FROM core.equipment
    WHERE equipment_id = ANY(p_equipment_ids);
    
    IF v_valid_equipment_ids IS NULL THEN
        "Status" := 'Error';
        "Message" := 'No valid equipment IDs found';
        RETURN;
    END IF;
    
    -- existing
    SELECT array_agg(equipment_id) INTO v_existing_assignments
    FROM core.equipment_state_group_mapping
    WHERE equipment_id = ANY(v_valid_equipment_ids) 
    AND state_group_id = p_state_group_id;
    
    -- new assignments - existing ones
    IF v_existing_assignments IS NULL THEN
        v_new_assignments := v_valid_equipment_ids;
    ELSE
        SELECT array_agg(eq_id) INTO v_new_assignments
        FROM unnest(v_valid_equipment_ids) AS eq_id
        WHERE eq_id != ALL(v_existing_assignments);
    END IF;
    
    -- perform bulk insert for new assignments
    IF v_new_assignments IS NOT NULL AND array_length(v_new_assignments, 1) > 0 THEN
        INSERT INTO core.equipment_state_group_mapping (equipment_id, state_group_id)
        SELECT unnest(v_new_assignments), p_state_group_id;
        
        GET DIAGNOSTICS v_assigned_count = ROW_COUNT;
    END IF;
    
    -- skipped count
    v_skipped_count := COALESCE(array_length(v_existing_assignments, 1), 0);
    
    "Data" := jsonb_build_object(
        'state_group_id', p_state_group_id,
        'state_group_name', v_state_group_name,
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
Procedure:     core.stateGroupExists
Version:       1.0.0
Description:   Checks if state group exists by name
Parameters:
    p_state_group_name VARCHAR(255)     -- Required state group name
Returns:       BOOLEAN                  -- TRUE if exists, FALSE otherwise
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.stateGroupExists(
    p_state_group_name varchar(255)
)
RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    IF p_state_group_name IS NULL OR trim(p_state_group_name) = '' THEN
        RETURN FALSE;
    END IF;
    
    RETURN EXISTS (
        SELECT 1 FROM core.state_group 
        WHERE state_group_name = trim(p_state_group_name)
    );
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getAllStates
Version:       1.0.0
Description:   Retrieves all states with their state group information
Parameters:
    None
Returns:       Table with state_id, state_code, state_description, state_group_id, state_group_name, created_at, updated_at
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getAllStates()
RETURNS TABLE (
    state_id uuid,
    state_code integer,
    state_description text,
    state_group_id uuid,
    state_group_name varchar(255),
    state_group_description text,
    created_at timestamptz,
    updated_at timestamptz
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    RETURN QUERY
    SELECT 
        s.state_id,
        s.state_code,
        s.state_description,
        s.state_group_id,
        sg.state_group_name,
        sg.state_group_description,
        s.created_at,
        s.updated_at
    FROM core.state s
    JOIN core.state_group sg ON s.state_group_id = sg.state_group_id
    ORDER BY sg.state_group_name, s.state_code;
    
EXCEPTION WHEN OTHERS THEN
    RAISE EXCEPTION 'Error retrieving states: %', SQLERRM;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getStateById
Version:       1.0.0
Description:   Retrieves specific state by ID with state group details
Parameters:
    p_state_id UUID                     -- Required state ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- State data with state group info as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getStateById(
    p_state_id uuid,
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
    "Message" := 'State retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State ID cannot be null';
        RETURN;
    END IF;
    
    -- get state data with state group info
    SELECT 
        s.state_id,
        s.state_code,
        s.state_description,
        s.state_group_id,
        sg.state_group_name,
        sg.state_group_description,
        s.created_at,
        s.updated_at
    INTO v_result
    FROM core.state s
    JOIN core.state_group sg ON s.state_group_id = sg.state_group_id
    WHERE s.state_id = p_state_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State not found';
        RETURN;
    END IF;
    
    -- to json
    "Data" := jsonb_build_object(
        'state_id', v_result.state_id,
        'state_code', v_result.state_code,
        'state_description', v_result.state_description,
        'state_group', jsonb_build_object(
            'state_group_id', v_result.state_group_id,
            'state_group_name', v_result.state_group_name,
            'state_group_description', v_result.state_group_description
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
Procedure:     core.getStatesByStateGroupId
Version:       1.0.0
Description:   Retrieves all states for a specific state group
Parameters:
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- States array with state group info as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getStatesByStateGroupId(
    p_state_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_group RECORD;
    v_states jsonb;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'States retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- does state group exists and get info
    SELECT state_group_name, state_group_description
    INTO v_state_group
    FROM core.state_group
    WHERE state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- states for this state group
    SELECT COALESCE(jsonb_agg(
        jsonb_build_object(
            'state_id', s.state_id,
            'state_code', s.state_code,
            'state_description', s.state_description,
            'created_at', s.created_at,
            'updated_at', s.updated_at
        ) ORDER BY s.state_code
    ), '[]'::jsonb)
    INTO v_states
    FROM core.state s
    WHERE s.state_group_id = p_state_group_id;
    
    -- data response
    "Data" := jsonb_build_object(
        'state_group_id', p_state_group_id,
        'state_group_name', v_state_group.state_group_name,
        'state_group_description', v_state_group.state_group_description,
        'states', v_states,
        'state_count', jsonb_array_length(v_states)
    );
    
    "Message" := 'Found ' || jsonb_array_length(v_states) || ' state(s) for state group: ' || v_state_group.state_group_name;
    
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
Procedure:     core.getStateByCode
Version:       1.0.0
Description:   Retrieves state by code within a specific state group
Parameters:
    p_state_code INTEGER                -- Required state code
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- State data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getStateByCode(
    p_state_code integer,
    p_state_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_id uuid;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'State retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_code IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State code cannot be null';
        RETURN;
    END IF;
    
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- get the state id by code and state group
    SELECT state_id INTO v_state_id
    FROM core.state
    WHERE state_code = p_state_code 
    AND state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State not found in specified state group';
        RETURN;
    END IF;
    
    -- getStateById
    SELECT s."Status", s."Message", s."Data"
    INTO "Status", "Message", "Data"
    FROM core.getStateById(v_state_id) s;
    
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
Procedure:     core.getStateByDescription
Version:       1.0.0
Description:   Retrieves state by description within a specific state group
Parameters:
    p_state_description TEXT            -- Required state description
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- State data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getStateByDescription(
    p_state_description text,
    p_state_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_id uuid;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'State retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_description IS NULL OR trim(p_state_description) = '' THEN
        "Status" := 'Error';
        "Message" := 'State description cannot be empty';
        RETURN;
    END IF;
    
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- get state ID by description and state group
    SELECT state_id INTO v_state_id
    FROM core.state
    WHERE state_description = trim(p_state_description) 
    AND state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State not found in specified state group';
        RETURN;
    END IF;
    
    -- getStateById
    SELECT s."Status", s."Message", s."Data"
    INTO "Status", "Message", "Data"
    FROM core.getStateById(v_state_id) s;
    
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
Procedure:     core.insertState
Version:       1.0.0
Description:   Creates new state with validation
Parameters:
    p_state_code INTEGER                -- Required state code
    p_state_description TEXT            -- Required state description
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- New state data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.insertState(
    p_state_code integer,
    p_state_description text,
    p_state_group_id uuid,
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
    v_state_group_name varchar(255);
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'State created successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_code IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State code cannot be null';
        RETURN;
    END IF;
    
    IF p_state_description IS NULL OR trim(p_state_description) = '' THEN
        "Status" := 'Error';
        "Message" := 'State description cannot be empty';
        RETURN;
    END IF;
    
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    v_trimmed_description := trim(p_state_description);
    
    -- validation code range
    IF p_state_code < 0 OR p_state_code > 9999 THEN
        "Status" := 'Error';
        "Message" := 'State code must be between 0 and 9999';
        RETURN;
    END IF;
    
    -- validation description size
    IF length(v_trimmed_description) < 2 OR length(v_trimmed_description) > 2048 THEN
        "Status" := 'Error';
        "Message" := 'State description must be between 2 and 2048 characters';
        RETURN;
    END IF;
    
    -- group existant
    SELECT state_group_name INTO v_state_group_name
    FROM core.state_group
    WHERE state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- someone already has state code in the same state group
    IF EXISTS (
        SELECT 1 FROM core.state 
        WHERE state_code = p_state_code 
        AND state_group_id = p_state_group_id
    ) THEN
        "Status" := 'Error';
        "Message" := 'State code already exists in this state group';
        RETURN;
    END IF;
    
    -- check for existing descriptions
    IF EXISTS (
        SELECT 1 FROM core.state 
        WHERE state_description = v_trimmed_description 
        AND state_group_id = p_state_group_id
    ) THEN
        "Status" := 'Error';
        "Message" := 'State description already exists in this state group';
        RETURN;
    END IF;
    
    -- create new
    INSERT INTO core.state (state_code, state_description, state_group_id)
    VALUES (p_state_code, v_trimmed_description, p_state_group_id)
    RETURNING state_id INTO v_new_id;
    
    "Data" := jsonb_build_object(
        'state_id', v_new_id,
        'state_code', p_state_code,
        'state_description', v_trimmed_description,
        'state_group', jsonb_build_object(
            'state_group_id', p_state_group_id,
            'state_group_name', v_state_group_name
        ),
        'created_at', now()
    );
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'State code or description already exists in this state group';
        "Data" := NULL;
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Invalid state group ID';
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
Procedure:     core.updateState
Version:       1.0.0
Description:   Updates existing state with validation
Parameters:
    p_state_id UUID                     -- Required state ID
    p_state_code INTEGER                -- Optional new state code (null = no change)
    p_state_description TEXT            -- Optional new state description (null = no change)
    p_state_group_id UUID               -- Optional new state group ID (null = no change)
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Updated state data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.updateState(
    p_state_id uuid,
    p_state_code integer DEFAULT NULL,
    p_state_description text DEFAULT NULL,
    p_state_group_id uuid DEFAULT NULL,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_current_record RECORD;
    v_new_code integer;
    v_new_description text;
    v_new_state_group_id uuid;
    v_changes_made boolean := false;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'State updated successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State ID cannot be null';
        RETURN;
    END IF;
    
    -- get the record
    SELECT state_code, state_description, state_group_id
    INTO v_current_record
    FROM core.state
    WHERE state_id = p_state_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State not found';
        RETURN;
    END IF;
    
    -- who is getting updated COALESCE
    v_new_code := COALESCE(p_state_code, v_current_record.state_code);
    v_new_description := COALESCE(trim(p_state_description), v_current_record.state_description);
    v_new_state_group_id := COALESCE(p_state_group_id, v_current_record.state_group_id);
    
    -- new code validation
    IF p_state_code IS NOT NULL THEN
        IF p_state_code < 0 OR p_state_code > 9999 THEN
            "Status" := 'Error';
            "Message" := 'State code must be between 0 and 9999';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    -- new description validation
    IF p_state_description IS NOT NULL THEN
        IF trim(p_state_description) = '' THEN
            "Status" := 'Error';
            "Message" := 'State description cannot be empty';
            RETURN;
        END IF;
        
        IF length(v_new_description) < 2 OR length(v_new_description) > 2048 THEN
            "Status" := 'Error';
            "Message" := 'State description must be between 2 and 2048 characters';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    -- validate the state group
    IF p_state_group_id IS NOT NULL THEN
        IF NOT EXISTS (SELECT 1 FROM core.state_group WHERE state_group_id = p_state_group_id) THEN
            "Status" := 'Error';
            "Message" := 'Invalid state group ID';
            RETURN;
        END IF;
        
        v_changes_made := true;
    END IF;
    
    -- conflicts in the state code per group
    IF EXISTS (
        SELECT 1 FROM core.state 
        WHERE state_code = v_new_code 
        AND state_group_id = v_new_state_group_id 
        AND state_id != p_state_id
    ) THEN
        "Status" := 'Error';
        "Message" := 'State code already exists in the target state group';
        RETURN;
    END IF;
    
    -- conflicts in the state description per group
    IF EXISTS (
        SELECT 1 FROM core.state 
        WHERE state_description = v_new_description 
        AND state_group_id = v_new_state_group_id 
        AND state_id != p_state_id
    ) THEN
        "Status" := 'Error';
        "Message" := 'State description already exists in the target state group';
        RETURN;
    END IF;
    
    -- update if changes were made
    IF v_changes_made THEN
        UPDATE core.state
        SET state_code = v_new_code,
            state_description = v_new_description,
            state_group_id = v_new_state_group_id,
            updated_at = now()
        WHERE state_id = p_state_id;
    ELSE
        "Message" := 'No changes were made to state';
    END IF;
    
    -- data by using getStateById
    SELECT s."Data" INTO "Data"
    FROM core.getStateById(p_state_id) s
    WHERE s."Status" = 'Success';
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'State code or description already exists in the target state group';
        "Data" := NULL;
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Invalid state group ID';
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
Procedure:     core.deleteState
Version:       1.0.0
Description:   Safely deletes state with validation
Parameters:
    p_state_id UUID                     -- Required state ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Operation result data as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.deleteState(
    p_state_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_info RECORD;
    v_deleted_count integer;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'State deleted successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State ID cannot be null';
        RETURN;
    END IF;
    
    -- get state info
    SELECT 
        s.state_code,
        s.state_description,
        sg.state_group_name,
        sg.state_group_id
    INTO v_state_info
    FROM core.state s
    JOIN core.state_group sg ON s.state_group_id = sg.state_group_id
    WHERE s.state_id = p_state_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State not found';
        RETURN;
    END IF;
    
    -- default state group name 
    IF v_state_info.state_group_name = 'Default MES State Group' 
       AND v_state_info.state_description IN ('disabled', 'running', 'change over', 'idle', 'e-stop', 'blocked', 'starved', 'planned downtime', 'unplanned downtime', 'user planned downtime', 'user unplanned downtime') THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete default MES state: ' || v_state_info.state_description;
        RETURN;
    END IF;

    -- default state codes 
    IF v_state_info.state_description IN ('disabled', 'running', 'change over', 'idle', 'e-stop', 'blocked', 'starved', 'planned downtime', 'unplanned downtime', 'user planned downtime', 'user unplanned downtime') THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete default MES state: ' || v_state_info.state_description;
        RETURN;
    END IF;
    
    -- delete the state
    DELETE FROM core.state WHERE state_id = p_state_id;
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    IF v_deleted_count = 0 THEN
        "Status" := 'Error';
        "Message" := 'State could not be deleted';
        RETURN;
    END IF;
    
    "Data" := jsonb_build_object(
        'deleted_state_id', p_state_id,
        'deleted_state_code', v_state_info.state_code,
        'deleted_state_description', v_state_info.state_description,
        'state_group_id', v_state_info.state_group_id,
        'state_group_name', v_state_info.state_group_name
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
Procedure:     core.searchStates
Version:       1.0.0
Description:   Searches states by description pattern across all state groups
Parameters:
    p_search_term TEXT                  -- Optional search term (null returns all)
    p_state_group_id UUID               -- Optional state group filter (null = all groups)
    p_state_code_min INTEGER            -- Optional minimum state code filter
    p_state_code_max INTEGER            -- Optional maximum state code filter
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Search results as JSON array
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.searchStates(
    p_search_term text DEFAULT NULL,
    p_state_group_id uuid DEFAULT NULL,
    p_state_code_min integer DEFAULT NULL,
    p_state_code_max integer DEFAULT NULL,
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
    
    -- search queries
    IF p_search_term IS NOT NULL AND trim(p_search_term) != '' THEN
        v_where_clause := v_where_clause || ' AND s.state_description ILIKE ''%' || trim(p_search_term) || '%''';
    END IF;
    
    IF p_state_group_id IS NOT NULL THEN
        v_where_clause := v_where_clause || ' AND s.state_group_id = ''' || p_state_group_id || '''';
    END IF;
    
    IF p_state_code_min IS NOT NULL THEN
        v_where_clause := v_where_clause || ' AND s.state_code >= ' || p_state_code_min;
    END IF;
    
    IF p_state_code_max IS NOT NULL THEN
        v_where_clause := v_where_clause || ' AND s.state_code <= ' || p_state_code_max;
    END IF;
    
    -- searching
    EXECUTE format('
        SELECT COALESCE(jsonb_agg(
            jsonb_build_object(
                ''state_id'', s.state_id,
                ''state_code'', s.state_code,
                ''state_description'', s.state_description,
                ''state_group'', jsonb_build_object(
                    ''state_group_id'', sg.state_group_id,
                    ''state_group_name'', sg.state_group_name,
                    ''state_group_description'', sg.state_group_description
                ),
                ''created_at'', s.created_at,
                ''updated_at'', s.updated_at
            ) ORDER BY sg.state_group_name, s.state_code
        ), ''[]''::jsonb)
        FROM core.state s
        JOIN core.state_group sg ON s.state_group_id = sg.state_group_id
        WHERE %s', v_where_clause)
    INTO v_results;
    
    "Data" := v_results;
    
    IF jsonb_array_length("Data") = 0 THEN
        "Message" := 'No states found matching search criteria';
    ELSE
        "Message" := 'Found ' || jsonb_array_length("Data") || ' state(s)';
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
Procedure:     core.bulkInsertStates
Version:       1.0.0
Description:   Creates multiple states for a state group in batch
Parameters:
    p_states JSONB                      -- Required array of state objects [{code, description}, ...]
    p_state_group_id UUID               -- Required state group ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Bulk insert results as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.bulkInsertStates(
    p_states jsonb,
    p_state_group_id uuid,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_group_name varchar(255);
    v_created_states jsonb := '[]'::jsonb;
    v_failed_states text[] := ARRAY[]::text[];
    v_created_count integer := 0;
    state_obj jsonb;
    v_state_code integer;
    v_state_description text;
    v_new_state_id uuid;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Bulk state creation completed';
    "Data" := NULL;
    
    -- p_validation
    IF p_states IS NULL OR jsonb_array_length(p_states) = 0 THEN
        "Status" := 'Error';
        "Message" := 'No states provided';
        RETURN;
    END IF;
    
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    -- does state group exists
    SELECT state_group_name INTO v_state_group_name
    FROM core.state_group
    WHERE state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- running through the states
    FOR state_obj IN SELECT jsonb_array_elements(p_states) LOOP
        BEGIN
            -- extract values from json object
            v_state_code := (state_obj->>'code')::integer;
            v_state_description := trim(state_obj->>'description');
            
            -- validation of code
            IF v_state_code IS NULL THEN
                v_failed_states := array_append(v_failed_states, 'Missing or invalid state code in: ' || state_obj::text);
                CONTINUE;
            END IF;
            
            IF v_state_code < 0 OR v_state_code > 9999 THEN
                v_failed_states := array_append(v_failed_states, 'Code ' || v_state_code || ': Must be between 0 and 9999');
                CONTINUE;
            END IF;
            
            -- validation of description
            IF v_state_description IS NULL OR v_state_description = '' THEN
                v_failed_states := array_append(v_failed_states, 'Code ' || v_state_code || ': Description cannot be empty');
                CONTINUE;
            END IF;
            
            IF length(v_state_description) < 2 OR length(v_state_description) > 2048 THEN
                v_failed_states := array_append(v_failed_states, 'Code ' || v_state_code || ': Description must be between 2 and 2048 characters');
                CONTINUE;
            END IF;
            
            -- checking for existing state code
            IF EXISTS (
                SELECT 1 FROM core.state 
                WHERE state_code = v_state_code AND state_group_id = p_state_group_id
            ) THEN
                v_failed_states := array_append(v_failed_states, 'Code ' || v_state_code || ': Already exists in this state group');
                CONTINUE;
            END IF;
            
            -- state description
            IF EXISTS (
                SELECT 1 FROM core.state 
                WHERE state_description = v_state_description AND state_group_id = p_state_group_id
            ) THEN
                v_failed_states := array_append(v_failed_states, 'Code ' || v_state_code || ': Description already exists in this state group');
                CONTINUE;
            END IF;
            
            -- create the state
            INSERT INTO core.state (state_code, state_description, state_group_id)
            VALUES (v_state_code, v_state_description, p_state_group_id)
            RETURNING state_id INTO v_new_state_id;
            
            -- add to created states
            v_created_states := v_created_states || jsonb_build_object(
                'state_id', v_new_state_id,
                'state_code', v_state_code,
                'state_description', v_state_description
            );
            
            v_created_count := v_created_count + 1;
            
        EXCEPTION WHEN OTHERS THEN
            v_failed_states := array_append(v_failed_states, 'Code ' || COALESCE(v_state_code::text, 'unknown') || ': ' || SQLERRM);
        END;
    END LOOP;
    
    "Data" := jsonb_build_object(
        'state_group_id', p_state_group_id,
        'state_group_name', v_state_group_name,
        'requested_count', jsonb_array_length(p_states),
        'created_count', v_created_count,
        'failed_count', array_length(v_failed_states, 1),
        'created_states', v_created_states,
        'failed_states', v_failed_states
    );
    
    -- set overall status
    IF v_created_count = 0 THEN
        "Status" := 'Error';
        "Message" := 'No states were created';
    ELSIF array_length(v_failed_states, 1) > 0 THEN
        "Message" := 'Partial success: ' || v_created_count || ' created, ' || array_length(v_failed_states, 1) || ' failed';
    ELSE
        "Message" := 'Successfully created ' || v_created_count || ' state(s)';
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
Procedure:     core.stateExists
Version:       1.0.0
Description:   Checks if state exists by code and state group
Parameters:
    p_state_code INTEGER                -- Required state code
    p_state_group_id UUID               -- Required state group ID
Returns:       BOOLEAN                  -- TRUE if exists, FALSE otherwise
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.stateExists(
    p_state_code integer,
    p_state_group_id uuid
)
RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    IF p_state_code IS NULL OR p_state_group_id IS NULL THEN
        RETURN FALSE;
    END IF;
    
    RETURN EXISTS (
        SELECT 1 FROM core.state 
        WHERE state_code = p_state_code
        AND state_group_id = p_state_group_id
    );
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getAvailableStateCodes
Version:       1.0.0
Description:   Returns available state codes for a state group within a range
Parameters:
    p_state_group_id UUID               -- Required state group ID
    p_min_code INTEGER=0                -- Optional minimum code range
    p_max_code INTEGER=100              -- Optional maximum code range
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Available codes and usage info as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getAvailableStateCodes(
    p_state_group_id uuid,
    p_min_code integer DEFAULT 0,
    p_max_code integer DEFAULT 100,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_state_group_name varchar(255);
    v_used_codes integer[];
    v_available_codes integer[];
    v_code integer;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Available state codes retrieved successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_state_group_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'State group ID cannot be null';
        RETURN;
    END IF;
    
    IF p_min_code < 0 OR p_max_code > 9999 OR p_min_code >= p_max_code THEN
        "Status" := 'Error';
        "Message" := 'Invalid code range: min must be >= 0, max must be <= 9999, and min < max';
        RETURN;
    END IF;
    
    -- state group verfication
    SELECT state_group_name INTO v_state_group_name
    FROM core.state_group
    WHERE state_group_id = p_state_group_id;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'State group not found';
        RETURN;
    END IF;
    
    -- find me the used
    SELECT array_agg(state_code ORDER BY state_code) INTO v_used_codes
    FROM core.state
    WHERE state_group_id = p_state_group_id
    AND state_code BETWEEN p_min_code AND p_max_code;
    
    -- available codes
    v_available_codes := ARRAY[]::integer[];
    FOR v_code IN p_min_code..p_max_code LOOP
        IF v_used_codes IS NULL OR v_code != ALL(v_used_codes) THEN
            v_available_codes := array_append(v_available_codes, v_code);
        END IF;
    END LOOP;
    
    "Data" := jsonb_build_object(
        'state_group_id', p_state_group_id,
        'state_group_name', v_state_group_name,
        'code_range', jsonb_build_object(
            'min_code', p_min_code,
            'max_code', p_max_code,
            'total_range', p_max_code - p_min_code + 1
        ),
        'used_codes', COALESCE(v_used_codes, ARRAY[]::integer[]),
        'available_codes', v_available_codes,
        'usage_stats', jsonb_build_object(
            'used_count', COALESCE(array_length(v_used_codes, 1), 0),
            'available_count', array_length(v_available_codes, 1),
            'usage_percentage', CASE 
                WHEN p_max_code - p_min_code + 1 > 0 THEN 
                    ROUND((COALESCE(array_length(v_used_codes, 1), 0)::numeric / (p_max_code - p_min_code + 1)) * 100, 2)
                ELSE 0 
            END
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
Procedure:     core.getStateUsageStats
Version:       1.0.0
Description:   Retrieves detailed usage stats for all states
Parameters:
    None
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Usage stats as JSON
Change Log:
    2025-07-28  hunter init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getStateUsageStats(
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
    "Message" := 'State usage statistics retrieved successfully';
    "Data" := NULL;
    
    SELECT jsonb_agg(
        jsonb_build_object(
            'state_group_id', sg.state_group_id,
            'state_group_name', sg.state_group_name,
            'state_group_description', sg.state_group_description,
            'state_count', COALESCE(COUNT(s.state_id), 0),
            'code_range', CASE 
                WHEN COUNT(s.state_id) > 0 THEN 
                    jsonb_build_object(
                        'min_code', MIN(s.state_code),
                        'max_code', MAX(s.state_code),
                        'code_span', MAX(s.state_code) - MIN(s.state_code) + 1
                    )
                ELSE NULL
            END,
            'states', COALESCE(jsonb_agg(
                jsonb_build_object(
                    'state_id', s.state_id,
                    'state_code', s.state_code,
                    'state_description', s.state_description,
                    'created_at', s.created_at,
                    'updated_at', s.updated_at
                ) ORDER BY s.state_code
            ) FILTER (WHERE s.state_id IS NOT NULL), '[]'::jsonb),
            'is_default_group', (sg.state_group_name = 'Default MES State Group'),
            'created_at', sg.created_at,
            'updated_at', sg.updated_at
        ) ORDER BY sg.state_group_name
    )
    INTO v_results
    FROM core.state_group sg
    LEFT JOIN core.state s ON sg.state_group_id = s.state_group_id
    GROUP BY sg.state_group_id, sg.state_group_name, sg.state_group_description, sg.created_at, sg.updated_at;

    SELECT jsonb_build_object(
        'total_states', (SELECT COUNT(*) FROM core.state),
        'total_state_groups', (SELECT COUNT(*) FROM core.state_group),
        'state_groups_with_states', (SELECT COUNT(DISTINCT state_group_id) FROM core.state),
        'default_states', (
            SELECT COUNT(*) 
            FROM core.state s 
            JOIN core.state_group sg ON s.state_group_id = sg.state_group_id 
            WHERE sg.state_group_name = 'Default MES State Group'
        ),
        'custom_states', (
            SELECT COUNT(*) 
            FROM core.state s 
            JOIN core.state_group sg ON s.state_group_id = sg.state_group_id 
            WHERE sg.state_group_name != 'Default MES State Group'
        ),
        'code_range_overall', (
            SELECT jsonb_build_object(
                'min_code', MIN(state_code),
                'max_code', MAX(state_code),
                'total_span', MAX(state_code) - MIN(state_code) + 1
            )
            FROM core.state
        )
    )
    INTO v_summary;
    
    "Data" := jsonb_build_object(
        'state_groups', COALESCE(v_results, '[]'::jsonb),
        'summary', v_summary
    );
    
EXCEPTION WHEN OTHERS THEN
    "Status" := 'Error';
    "Message" := 'Unexpected error: ' || SQLERRM;
    "Data" := NULL;
END;
$$;
