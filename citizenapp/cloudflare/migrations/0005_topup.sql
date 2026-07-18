-- 稳定币充值购买公民币 · 三态台账。
-- 一行 = 一笔「已确认收到稳定币」的入金订单;未支付订单不入表(无第四态)。
-- status 三态与用户口径一一对应:pending=待支付 / paid=已支付 / exception=异常。
CREATE TABLE IF NOT EXISTS topup_orders (
  order_id TEXT PRIMARY KEY,
  chain_id INTEGER NOT NULL,
  token TEXT NOT NULL,                 -- 'USDC' | 'USDT'
  token_contract TEXT NOT NULL,        -- ERC-20 合约地址(小写)
  evm_tx_hash TEXT NOT NULL,           -- 用户付款的 EVM 交易哈希(小写)
  payer_address TEXT,                  -- 付款方 EVM 地址(小写);可空
  recv_address TEXT NOT NULL,          -- 收款 EVM 地址(小写)
  pay_amount TEXT NOT NULL,            -- 应付稳定币最小单位(字符串,防溢出)
  gmb_address TEXT NOT NULL,           -- 收公民币的公民链钱包地址
  coin_fen TEXT NOT NULL,              -- 应发公民币分额(字符串)
  package_id TEXT NOT NULL,            -- 套餐标识
  status TEXT NOT NULL,                -- 'pending' | 'paid' | 'exception'
  gmb_tx_hash TEXT,                    -- 公民币发币交易哈希(paid 时写入)
  exception_reason TEXT,               -- 异常原因(exception 时写入)
  confirmed_at INTEGER NOT NULL,       -- 稳定币到账确认落库时刻(ms)
  settled_at INTEGER                   -- 结算(已支付/异常)时刻(ms)
);

-- 同一笔链上付款只入账一次:幂等键,杜绝重复发币。
CREATE UNIQUE INDEX IF NOT EXISTS idx_topup_orders_txhash ON topup_orders (chain_id, evm_tx_hash);

-- 待发币队列扫描(status='pending' 按到账时间升序)。
CREATE INDEX IF NOT EXISTS idx_topup_orders_status ON topup_orders (status, confirmed_at);
