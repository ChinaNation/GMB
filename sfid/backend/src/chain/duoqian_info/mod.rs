//! 机构信息查询(chain pull)。
//!
//! - 调用方:`citizenchain/node` 桌面"添加清算行"页 + wuminapp 钱包绑定/支付前查询
//! - 端点:
//!   - `GET /api/v1/app/institutions/search`            通用机构搜索
//!   - `GET /api/v1/app/institutions/:sfid_id`          单个机构详情
//!   - `GET /api/v1/app/institutions/:sfid_id/accounts` 机构账户列表
//!   - `GET /api/v1/app/clearing-banks/search`          已激活清算行搜索(分页)
//!   - `GET /api/v1/app/clearing-banks/eligible-search` 候选清算行搜索(无分页,候选含未激活)
//!
//! 机构数据由 SFID 独立维护,链端按需 pull;不再依赖 SFID 推链或 watcher 反向读链。
//! 清算行只是众多机构的一个子集,凡链上能注册多签的机构都从这里查。

pub(crate) mod dto;
pub(crate) mod handler;

pub(crate) use handler::{
    app_get_institution, app_list_accounts, app_search_clearing_banks,
    app_search_eligible_clearing_banks, app_search_institutions,
};
