-- 产品规格库表
CREATE TABLE product_specifications (
    product_id INT PRIMARY KEY AUTO_INCREMENT,
    product_name VARCHAR(100) NOT NULL,
    model VARCHAR(50),
    material_type VARCHAR(50),
    color VARCHAR(30),
    dimensions VARCHAR(100),
    created_by INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(user_id)
);

-- 材料规格库表
CREATE TABLE materials (
    material_id INT PRIMARY KEY AUTO_INCREMENT,
    material_name VARCHAR(100) NOT NULL,
    category VARCHAR(50),
    type VARCHAR(50),
    supplier VARCHAR(100),
    created_by INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(user_id)
);

-- 材料领用表
CREATE TABLE material_requests (
    request_id INT PRIMARY KEY AUTO_INCREMENT,
    material_id INT,
    quantity INT,
    requested_by INT,
    warehouse_id INT,
    request_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status ENUM('pending', 'approved', 'rejected') DEFAULT 'pending',
    FOREIGN KEY (material_id) REFERENCES materials(material_id),
    FOREIGN KEY (requested_by) REFERENCES users(user_id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(warehouse_id)
);
