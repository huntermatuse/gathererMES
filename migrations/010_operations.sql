-- operation tables 
-- these tables include products, work orders, jobs

-- product table
create table operation.product {
    product_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    product_name text collate "case_insensitive" unique not null,
    product_metadata JSONB DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
}

SELECT trigger_updated_at('operation.product');


-- work order table
create table operation.work_order {
    work_order_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    work_order_name text collate "case_insensitive" unique not null,
    work_order_metadata DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz

}

SELECT trigger_updated_at('operation.work_order');

-- job table
create table operation.job {
    job_id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    work_order_id uuid NOT NULL REFERENCES operation.work_order(work_order_id),
    job_metadata DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
}

SELECT trigger_updated_at('operation.job');
