-- 删除报表视图
DROP VIEW IF EXISTS material_request_summary;

-- 删除操作日志表
DROP TABLE IF EXISTS operation_logs;

-- 删除角色权限关联表
DROP TABLE IF EXISTS role_permissions;

-- 删除权限表
DROP TABLE IF EXISTS permissions;

-- 删除用户角色关联表
DROP TABLE IF EXISTS user_roles;

-- 删除角色表
DROP TABLE IF EXISTS roles;

-- 删除用户表
DROP TABLE IF EXISTS users;

-- 删除仓库库存表
DROP TABLE IF EXISTS warehouse_stock;

-- 删除仓库信息表
DROP TABLE IF EXISTS warehouses;

-- 删除生产任务表
DROP TABLE IF EXISTS production_tasks;

-- 删除费用设定表
DROP TABLE IF EXISTS production_costs;

-- 删除价格预算公式表
DROP TABLE IF EXISTS price_formulas;

-- 删除产品规格库表
DROP TABLE IF EXISTS product_specifications;

-- 删除材料规格库表
DROP TABLE IF EXISTS materials;

-- 删除材料领用表
DROP TABLE IF EXISTS material_requests;
