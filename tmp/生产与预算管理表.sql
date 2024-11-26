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
