-- 仓库信息表
CREATE TABLE warehouses (
    warehouse_id INT PRIMARY KEY AUTO_INCREMENT,
    localkey Text DEFAULT NULL,
    warehouse_name VARCHAR(50) NOT NULL,
    location TEXT NOT NULL,
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

-- 用户表
CREATE TABLE users (
    user_id INT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(100),
    position VARCHAR(50),
    contact_info VARCHAR(100),
    status ENUM('active', 'inactive') DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 角色表
CREATE TABLE roles (
    role_id INT PRIMARY KEY AUTO_INCREMENT,
    role_name VARCHAR(50) NOT NULL UNIQUE,
    description TEXT
);

-- 用户角色关联表
CREATE TABLE user_roles (
    user_id INT,
    role_id INT,
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (role_id) REFERENCES roles(role_id),
    PRIMARY KEY (user_id, role_id)
);

-- 权限表
CREATE TABLE permissions (
    permission_id INT PRIMARY KEY AUTO_INCREMENT,
    permission_name VARCHAR(50) NOT NULL UNIQUE,
    description TEXT
);

-- 角色权限关联表
CREATE TABLE role_permissions (
    role_id INT,
    permission_id INT,
    FOREIGN KEY (role_id) REFERENCES roles(role_id),
    FOREIGN KEY (permission_id) REFERENCES permissions(permission_id),
    PRIMARY KEY (role_id, permission_id)
);

-- 操作日志表
CREATE TABLE operation_logs (
    log_id INT PRIMARY KEY AUTO_INCREMENT,
    user_id INT,
    action VARCHAR(255),
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- 报表数据生成视图 (例如：材料领用记录汇总)
CREATE VIEW material_request_summary AS
SELECT 
    r.request_id,
    m.material_name,
    r.quantity,
    r.request_date,
    u.full_name AS requested_by,
    w.warehouse_name
FROM 
    material_requests r
JOIN materials m ON r.material_id = m.material_id
JOIN users u ON r.requested_by = u.user_id
JOIN warehouses w ON r.warehouse_id = w.warehouse_id;

-- 生产任务表
CREATE TABLE production_tasks (
    task_id INT PRIMARY KEY AUTO_INCREMENT,
    product_id INT,
    quantity INT NOT NULL,
    due_date DATE,
    created_by INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status ENUM('not_started', 'in_progress', 'completed') DEFAULT 'not_started',
    FOREIGN KEY (product_id) REFERENCES product_specifications(product_id),
    FOREIGN KEY (created_by) REFERENCES users(user_id)
);

-- 费用设定表
CREATE TABLE production_costs (
    cost_id INT PRIMARY KEY AUTO_INCREMENT,
    process_type VARCHAR(50) NOT NULL,
    cost_per_unit DECIMAL(10, 2) NOT NULL,
    created_by INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(user_id)
);

-- 价格预算公式表
CREATE TABLE price_formulas (
    formula_id INT PRIMARY KEY AUTO_INCREMENT,
    formula_name VARCHAR(100),
    base_material_cost DECIMAL(10, 2),
    additional_material_cost DECIMAL(10, 2),
    galvanization_cost DECIMAL(10, 2),
    labor_cost DECIMAL(10, 2),
    management_fee DECIMAL(10, 2),
    sales_fee DECIMAL(10, 2),
    manufacturing_fee DECIMAL(10, 2),
    vat DECIMAL(10, 2),
    profit DECIMAL(10, 2),
    created_by INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(user_id)
);

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
