
/*
===========================================
Author:        hunter
Created:       2025-07-28
Schema:        core
Version:       1.0.0
Description:   Equipment Type Comments, Procedures and Constraints 
Change Log:
    2025-07-28  hunter  init
===========================================
*/

COMMENT ON TABLE core.equipment_type IS 'Defines the types of equipment in the MES hierarchy (enterprise, site, area, line, cell)';
COMMENT ON COLUMN core.equipment_type.type_name IS 'Human-readable name for the equipment type, case-insensitive unique';

ALTER TABLE core.equipment_type 
ADD CONSTRAINT chk_equipment_type_name_not_empty 
CHECK (length(trim(type_name)) > 0);

ALTER TABLE core.equipment_type 
ADD CONSTRAINT chk_equipment_type_name_length 
CHECK (length(type_name) BETWEEN 2 AND 255);

-- ALTER TABLE core.equipment_type 
-- ADD CONSTRAINT chk_equipment_type_name_format 
-- CHECK (type_name ~ '^[a-zA-Z0-9][a-zA-Z0-9 _-]*[a-zA-Z0-9]$|^[a-zA-Z0-9]$')


/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getAllEquipmentTypes
Version:       1.0.0
Description:   Retrieves all equipment types with usage statistics
Parameters:
    None
Returns:       table with type_id, type_name, equipment_count, created_at, updated_at
Change Log:
    2025-07-28  hunter  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getAllEquipmentTypes()
RETURNS TABLE (
    type_id uuid,
    type_name text,
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
        et.type_id,
        et.type_name,
        COALESCE(COUNT(e.equipment_id), 0) as equipment_count,
        et.created_at,
        et.updated_at
    FROM core.equipment_type et
    LEFT JOIN core.equipment e ON et.type_id = e.equipment_type_id
    GROUP BY et.type_id, et.type_name, et.created_at, et.updated_at
    ORDER BY et.type_name;
    
EXCEPTION WHEN OTHERS THEN
    RAISE EXCEPTION 'Error retrieving equipment types: %', SQLERRM;
END;
$$;

/*
===========================================
Author:        hunter
Created:       2025-07-28
Procedure:     core.getEquipmentTypeById
Version:       1.0.0
Description:   Retrieves specific equipment type by ID with usage count
Parameters:
    p_type_id UUID                      -- Required equipment type ID
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Equipment type data as JSON
Change Log:
    2025-07-28  hunter  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getEquipmentTypeById(
    p_type_id uuid,
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
    -- output parameters
    "Status" := 'Success';
    "Message" := 'Equipment type retrieved successfully';
    "Data" := NULL;
    
    -- input validation
    IF p_type_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment type ID cannot be null';
        RETURN;
    END IF;
    
    -- get equipment type data
    SELECT 
        et.type_id,
        et.type_name,
        COALESCE(COUNT(e.equipment_id), 0) as equipment_count,
        et.created_at,
        et.updated_at
    INTO v_result
    FROM core.equipment_type et
    LEFT JOIN core.equipment e ON et.type_id = e.equipment_type_id
    WHERE et.type_id = p_type_id
    GROUP BY et.type_id, et.type_name, et.created_at, et.updated_at;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Equipment type not found';
        RETURN;
    END IF;
    
    -- convert result to json
    "Data" := to_jsonb(v_result);
    
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
Procedure:     core.getEquipmentTypeByName
Version:       1.0.0
Description:   Retrieves specific equipment type by name with usage count
Parameters:
    p_type_name TEXT                    -- Required equipment type name
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Equipment type data as JSON
Change Log:
    2025-07-28  hunter  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.getEquipmentTypeByName(
    p_type_name text,
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
    -- output
    "Status" := 'Success';
    "Message" := 'Equipment type retrieved successfully';
    "Data" := NULL;
    
    -- input validation
    IF p_type_name IS NULL OR trim(p_type_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'Equipment type name cannot be empty';
        RETURN;
    END IF;
    
    -- get equipment type data
    SELECT 
        et.type_id,
        et.type_name,
        COALESCE(COUNT(e.equipment_id), 0) as equipment_count,
        et.created_at,
        et.updated_at
    INTO v_result
    FROM core.equipment_type et
    LEFT JOIN core.equipment e ON et.type_id = e.equipment_type_id
    WHERE et.type_name = trim(p_type_name)
    GROUP BY et.type_id, et.type_name, et.created_at, et.updated_at;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Equipment type not found';
        RETURN;
    END IF;
    
    -- convert result to json
    "Data" := to_jsonb(v_result);
    
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
Procedure:     core.insertEquipmentType
Version:       1.0.0
Description:   Creates new equipment type with validation
Parameters:
    p_type_name TEXT                    -- Required equipment type name
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- New equipment type data as JSON
Change Log:
    2025-07-28  hunter  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.insertEquipmentType(
    p_type_name text,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_new_id uuid;
    v_trimmed_name text;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment type created successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_type_name IS NULL OR trim(p_type_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'Equipment type name cannot be empty';
        RETURN;
    END IF;
    
    v_trimmed_name := trim(p_type_name);
    
    -- validate length
    IF length(v_trimmed_name) < 2 OR length(v_trimmed_name) > 255 THEN
        "Status" := 'Error';
        "Message" := 'Equipment type name must be between 2 and 255 characters';
        RETURN;
    END IF;
    
    -- check for existing type
    IF EXISTS (SELECT 1 FROM core.equipment_type WHERE type_name = v_trimmed_name) THEN
        "Status" := 'Error';
        "Message" := 'Equipment type already exists';
        RETURN;
    END IF;
    
    -- insert new equipment type
    INSERT INTO core.equipment_type (type_name)
    VALUES (v_trimmed_name)
    RETURNING type_id INTO v_new_id;
    
    -- return created data
    "Data" := jsonb_build_object(
        'type_id', v_new_id,
        'type_name', v_trimmed_name,
        'equipment_count', 0,
        'created_at', now()
    );
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'Equipment type already exists';
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
Procedure:     core.updateEquipmentType
Version:       1.0.0
Description:   Updates existing equipment type with validation
Parameters:
    p_type_id UUID                      -- Required equipment type ID
    p_type_name TEXT                    -- Required new equipment type name
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Updated equipment type data as JSON
Change Log:
    2025-07-28  hunter  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.updateEquipmentType(
    p_type_id uuid,
    p_type_name text,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_trimmed_name text;
    v_result RECORD;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment type updated successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_type_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment type ID cannot be null';
        RETURN;
    END IF;
    
    IF p_type_name IS NULL OR trim(p_type_name) = '' THEN
        "Status" := 'Error';
        "Message" := 'Equipment type name cannot be empty';
        RETURN;
    END IF;
    
    v_trimmed_name := trim(p_type_name);
    
    -- validate length
    IF length(v_trimmed_name) < 2 OR length(v_trimmed_name) > 255 THEN
        "Status" := 'Error';
        "Message" := 'Equipment type name must be between 2 and 255 characters';
        RETURN;
    END IF;
    
    -- check if equipment type id exists
    IF NOT EXISTS (SELECT 1 FROM core.equipment_type WHERE type_id = p_type_id) THEN
        "Status" := 'Error';
        "Message" := 'Equipment type not found';
        RETURN;
    END IF;
    
    -- check for conflicts (excluding current type)
    IF EXISTS (
        SELECT 1 FROM core.equipment_type 
        WHERE type_name = v_trimmed_name AND type_id != p_type_id
    ) THEN
        "Status" := 'Error';
        "Message" := 'Equipment type name already exists';
        RETURN;
    END IF;
    
    -- update the equipment type
    UPDATE core.equipment_type
    SET type_name = v_trimmed_name,
        updated_at = now()
    WHERE type_id = p_type_id;
    
    -- get updated data with equipment count
    SELECT 
        et.type_id,
        et.type_name,
        COALESCE(COUNT(e.equipment_id), 0) as equipment_count,
        et.created_at,
        et.updated_at
    INTO v_result
    FROM core.equipment_type et
    LEFT JOIN core.equipment e ON et.type_id = e.equipment_type_id
    WHERE et.type_id = p_type_id
    GROUP BY et.type_id, et.type_name, et.created_at, et.updated_at;
    
    "Data" := to_jsonb(v_result);
    
EXCEPTION 
    WHEN unique_violation THEN
        "Status" := 'Error';
        "Message" := 'Equipment type name already exists';
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
Procedure:     core.deleteEquipmentType
Version:       1.0.0
Description:   Safely deletes equipment type with dependency checks
Parameters:
    p_type_id UUID                      -- Required equipment type ID
    p_force_delete BOOLEAN=FALSE        -- Optional force delete flag
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Operation result data as JSON
Change Log:
    2025-07-28  hunter  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.deleteEquipmentType(
    p_type_id uuid,
    p_force_delete boolean DEFAULT FALSE,
    OUT "Status" varchar(50),
    OUT "Message" text,
    OUT "Data" jsonb
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_equipment_count integer;
    v_type_name text;
    v_deleted_count integer;
BEGIN
    -- init out
    "Status" := 'Success';
    "Message" := 'Equipment type deleted successfully';
    "Data" := NULL;
    
    -- p_validation
    IF p_type_id IS NULL THEN
        "Status" := 'Error';
        "Message" := 'Equipment type ID cannot be null';
        RETURN;
    END IF;
    
    -- get equipment type info and usage count
    SELECT 
        et.type_name, 
        COALESCE(COUNT(e.equipment_id), 0)
    INTO v_type_name, v_equipment_count
    FROM core.equipment_type et
    LEFT JOIN core.equipment e ON et.type_id = e.equipment_type_id
    WHERE et.type_id = p_type_id
    GROUP BY et.type_name;
    
    IF NOT FOUND THEN
        "Status" := 'Error';
        "Message" := 'Equipment type not found';
        RETURN;
    END IF;
    
    -- check if it's a default type
    IF v_type_name IN ('enterprise', 'site', 'area', 'line', 'cell') THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete default equipment type: ' || v_type_name;
        RETURN;
    END IF;
    
    -- check for dependent equipment
    IF v_equipment_count > 0 AND NOT p_force_delete THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete equipment type: ' || v_equipment_count || ' equipment items are using this type. Use force_delete=true to override.';
        RETURN;
    END IF;
    
    -- delete the equipment type
    DELETE FROM core.equipment_type WHERE type_id = p_type_id;
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    IF v_deleted_count = 0 THEN
        "Status" := 'Error';
        "Message" := 'Equipment type could not be deleted';
        RETURN;
    END IF;
    
    "Data" := jsonb_build_object(
        'deleted_type_id', p_type_id,
        'deleted_type_name', v_type_name,
        'affected_equipment_count', v_equipment_count,
        'force_delete_used', p_force_delete
    );
    
EXCEPTION 
    WHEN foreign_key_violation THEN
        "Status" := 'Error';
        "Message" := 'Cannot delete equipment type: equipment items are still using this type';
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
Procedure:     core.searchEquipmentTypes
Version:       1.0.0
Description:   Searches equipment types by name pattern
Parameters:
    p_search_term TEXT                  -- Optional search term (null returns all)
    OUT Status VARCHAR(50)              -- 'Success' or 'Error'
    OUT Message TEXT                    -- Details or error text
    OUT Data JSONB                      -- Search results as JSON array
Change Log:
    2025-07-28  hunter  init
===========================================
*/
CREATE OR REPLACE FUNCTION core.searchEquipmentTypes(
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
    
    -- perform search
    IF p_search_term IS NULL OR trim(p_search_term) = '' THEN
        -- return all equipment types
        SELECT jsonb_agg(
            jsonb_build_object(
                'type_id', et.type_id,
                'type_name', et.type_name,
                'equipment_count', COALESCE(COUNT(e.equipment_id), 0),
                'created_at', et.created_at,
                'updated_at', et.updated_at
            )
        )
        INTO v_results
        FROM core.equipment_type et
        LEFT JOIN core.equipment e ON et.type_id = e.equipment_type_id
        GROUP BY et.type_id, et.type_name, et.created_at, et.updated_at
        ORDER BY et.type_name;
    ELSE
        -- search by pattern
        SELECT jsonb_agg(
            jsonb_build_object(
                'type_id', et.type_id,
                'type_name', et.type_name,
                'equipment_count', COALESCE(COUNT(e.equipment_id), 0),
                'created_at', et.created_at,
                'updated_at', et.updated_at
            )
        )
        INTO v_results
        FROM core.equipment_type et
        LEFT JOIN core.equipment e ON et.type_id = e.equipment_type_id
        WHERE et.type_name ILIKE '%' || trim(p_search_term) || '%'
        GROUP BY et.type_id, et.type_name, et.created_at, et.updated_at
        ORDER BY et.type_name;
    END IF;
    
    "Data" := COALESCE(v_results, '[]'::jsonb);
    
    IF jsonb_array_length("Data") = 0 THEN
        "Message" := 'No equipment types found matching search criteria';
    ELSE
        "Message" := 'Found ' || jsonb_array_length("Data") || ' equipment type(s)';
    END IF;
    
EXCEPTION WHEN OTHERS THEN
    "Status" := 'Error';
    "Message" := 'Unexpected error: ' || SQLERRM;
    "Data" := '[]'::jsonb;
END;
$$;