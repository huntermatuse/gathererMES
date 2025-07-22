CREATE SEQUENCE products_productid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE TABLE products (
    productId INTEGER PRIMARY KEY DEFAULT nextval('products_productid_seq'),
    productCode VARCHAR(100) NOT NULL,
    productName VARCHAR(255) NOT NULL,
    productDescription TEXT NOT NULL,
    isActive BOOLEAN NOT NULL DEFAULT true,
    createDate TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updatedDate TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_products_code ON products(productCode);
CREATE INDEX idx_products_name ON products(productName);
CREATE INDEX idx_products_active ON products(isActive);
CREATE INDEX idx_products_created ON products(createDate);
CREATE INDEX idx_products_metadata ON products USING GIN(metadata);

COMMENT ON TABLE products IS 'Product table';
COMMENT ON COLUMN products.productId IS 'Unique identifier for the product';
COMMENT ON COLUMN products.productCode IS 'Product code identifier';
COMMENT ON COLUMN products.productName IS 'Name of the product';
COMMENT ON COLUMN products.productDescription IS 'Detailed description of the product';
COMMENT ON COLUMN products.isActive IS 'Whether the product is currently active';
COMMENT ON COLUMN products.createDate IS 'Timestamp when the product was created';
COMMENT ON COLUMN products.updatedDate IS 'Timestamp when the product was last updated';
COMMENT ON COLUMN products.metadata IS 'Additional metadata stored as JSONB';