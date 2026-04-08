-- 003_drop_runtime_meta.sql
-- 移除 runtime_meta 空壳表。
-- 该表历史上用于持久化主签名人状态，后被弃用为 {version: 2} 占位空壳，
-- 现由代码侧一并移除（见任务卡 20260407-sfid-runtime-meta-清理.md）。
-- 部署顺序：先发布不再访问 runtime_meta 的新代码，确认稳定后再执行本 migration。

DROP TABLE IF EXISTS runtime_meta;
