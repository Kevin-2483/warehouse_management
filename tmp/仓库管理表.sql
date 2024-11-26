-- 仓库信息表
CREATE TABLE warehouses (
    warehouse_id INT PRIMARY KEY AUTO_INCREMENT,
    warehouse_name VARCHAR(50) NOT NULL,
    location VARCHAR(100),
    capacity INT,
    current_stock INT DEFAULT 0,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 仓库库存表
CREATE TABLE warehouse_stock (
    warehouse_id INT,
    material_id INT,
    quantity INT,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(warehouse_id),
    FOREIGN KEY (material_id) REFERENCES materials(material_id),
    PRIMARY KEY (warehouse_id, material_id)
);
