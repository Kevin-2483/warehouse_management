-- 仓库信息表
CREATE TABLE warehouses (
    warehouse_id INTEGER PRIMARY KEY AUTOINCREMENT,
    localkey TEXT DEFAULT NULL,
    warehouse_name TEXT NOT NULL,
    location TEXT NOT NULL,
    capacity INTEGER,
    current_stock INTEGER DEFAULT 0,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 仓库库存表
CREATE TABLE warehouse_stock (
    warehouse_id INTEGER,
    material_id INTEGER,
    quantity INTEGER,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(warehouse_id),
    FOREIGN KEY (material_id) REFERENCES materials(material_id),
    PRIMARY KEY (warehouse_id, material_id)
);

-- 用户表
CREATE TABLE users (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    full_name TEXT,
    position TEXT,
    contact_info TEXT,
    status TEXT CHECK(status IN ('active', 'inactive')) DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 角色表
CREATE TABLE roles (
    role_id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_name TEXT NOT NULL UNIQUE,
    description TEXT
);

-- 用户角色关联表
CREATE TABLE user_roles (
    user_id INTEGER,
    role_id INTEGER,
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (role_id) REFERENCES roles(role_id),
    PRIMARY KEY (user_id, role_id)
);

-- 权限表
CREATE TABLE permissions (
    permission_id INTEGER PRIMARY KEY AUTOINCREMENT,
    permission_name TEXT NOT NULL UNIQUE,
    description TEXT
);

-- 角色权限关联表
CREATE TABLE role_permissions (
    role_id INTEGER,
    permission_id INTEGER,
    FOREIGN KEY (role_id) REFERENCES roles(role_id),
    FOREIGN KEY (permission_id) REFERENCES permissions(permission_id),
    PRIMARY KEY (role_id, permission_id)
);

-- 操作日志表
CREATE TABLE operation_logs (
    log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    action TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- 生产任务表
CREATE TABLE production_tasks (
    task_id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER,
    quantity INTEGER NOT NULL,
    due_date DATE,
    created_by INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status TEXT CHECK(status IN ('not_started', 'in_progress', 'completed')) DEFAULT 'not_started',
    FOREIGN KEY (product_id) REFERENCES product_specifications(product_id),
    FOREIGN KEY (created_by) REFERENCES users(user_id)
);

-- 费用设定表
CREATE TABLE production_costs (
    cost_id INTEGER PRIMARY KEY AUTOINCREMENT,
    process_type TEXT NOT NULL,
    cost_per_unit DECIMAL(10, 2) NOT NULL,
    created_by INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(user_id)
);

-- 价格预算公式表
CREATE TABLE price_formulas (
    formula_id INTEGER PRIMARY KEY AUTOINCREMENT,
    formula_name TEXT,
    base_material_cost DECIMAL(10, 2),
    additional_material_cost DECIMAL(10, 2),
    galvanization_cost DECIMAL(10, 2),
    labor_cost DECIMAL(10, 2),
    management_fee DECIMAL(10, 2),
    sales_fee DECIMAL(10, 2),
    manufacturing_fee DECIMAL(10, 2),
    vat DECIMAL(10, 2),
    profit DECIMAL(10, 2),
    created_by INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(user_id)
);

-- 产品规格库表
CREATE TABLE product_specifications (
    product_id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_name TEXT NOT NULL,
    model TEXT,
    material_type TEXT,
    color TEXT,
    dimensions TEXT,
    created_by INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(user_id)
);

-- 材料规格库表
CREATE TABLE materials (
    material_id INTEGER PRIMARY KEY AUTOINCREMENT,
    material_name TEXT NOT NULL,
    category TEXT,
    type TEXT,
    supplier TEXT,
    created_by INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(user_id)
);

-- 材料领用表
CREATE TABLE material_requests (
    request_id INTEGER PRIMARY KEY AUTOINCREMENT,
    material_id INTEGER,
    quantity INTEGER,
    requested_by INTEGER,
    warehouse_id INTEGER,
    request_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status TEXT CHECK(status IN ('pending', 'approved', 'rejected')) DEFAULT 'pending',
    FOREIGN KEY (material_id) REFERENCES materials(material_id),
    FOREIGN KEY (requested_by) REFERENCES users(user_id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(warehouse_id)
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
