-- create the db
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT FROM pg_database WHERE datname = 'mes'
    ) THEN
        PERFORM dblink_exec('dbname=postgres', 'CREATE DATABASE mes');
    END IF;
END $$;

-- schemas
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT schema_name FROM information_schema.schemata WHERE schema_name = 'eq'
    ) THEN
        EXECUTE 'CREATE SCHEMA eq';
    END IF;

    IF NOT EXISTS (
        SELECT schema_name FROM information_schema.schemata WHERE schema_name = 'app'
    ) THEN
        EXECUTE 'CREATE SCHEMA app';
    END IF;

    IF NOT EXISTS (
        SELECT schema_name FROM information_schema.schemata WHERE schema_name = 'operations'
    ) THEN
        EXECUTE 'CREATE SCHEMA operations';
    END IF;
END $$;

-- my normal user
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT FROM pg_roles WHERE rolname = 'ignition' -- for my container
    ) THEN
        CREATE ROLE ignition LOGIN PASSWORD 'P@ssword1!';
    END IF;
END $$;

-- Grant privileges (equivalent to db_owner)
GRANT ALL PRIVILEGES ON DATABASE default_db TO ignition;
