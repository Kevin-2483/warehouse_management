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
