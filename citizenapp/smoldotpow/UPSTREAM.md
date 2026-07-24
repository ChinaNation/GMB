# smoldot-pow 快照来源说明

## 1. 目录用途

本目录保存 `citizenapp` 当前使用的 PoW 轻节点内核固定快照。

约束：

- 目录内不保留 `.git`
- 不提交 `target/` 等编译产物
- Flutter / Rust 构建只依赖此目录，不再回指仓库根目录临时 `smoldot/`

## 2. 上游来源

- 上游项目仓库字段：`https://github.com/smol-dot/smoldot`
- 本次收编前本地来源：`file:///Users/rhett/.cargo/git/db/smoldot-df5fc45614b6d921`
- 收编基线提交：`f471baac1f0fa821569c42ebb14c4f8533ba77ad`

## 3. 收编时已存在的本地 PoW 改动

以下文件在收编前已经带有本地修改：

- `lib/src/chain/blocks_tree.rs`
- `lib/src/chain/blocks_tree/finality.rs`
- `lib/src/chain/blocks_tree/verify.rs`
- `lib/src/chain/chain_information.rs`
- `lib/src/chain/chain_information/build.rs`
- `lib/src/header.rs`
- `lib/src/sync/warp_sync.rs`
- `lib/src/verify.rs`
- `lib/src/verify/header_only.rs`
- `light-base/src/sync_service/standalone.rs`
- `lib/src/verify/pow.rs`（新增）

说明：

- 这些改动是当前 `PoW + GRANDPA` 轻节点实验基线的一部分
- 后续必须整理进独立 GitHub fork，再按显式同步流程回灌到本目录

## 4. 后续同步规则（可执行流程）

> 目标：补上「上游追踪能力」——本目录不保留 `.git`，但通过下列流程随时能看清本快照
> 相对上游基线改了什么、并把本地 PoW 改动 rebase 到上游新版本后再回灌。

### 4.0 一次性：建立独立 fork（**用户 GitHub 动作，AI 不代建仓库**）

```bash
# 在 GitHub 上 fork smol-dot/smoldot 到你的账户,然后:
git clone git@github.com:<你的账户>/smoldot.git && cd smoldot
git remote add upstream https://github.com/smol-dot/smoldot
# 基于收编基线开 PoW 分支,把本目录本地改动(见 4.1 补丁)作为一次提交灌入:
git checkout -b pow-grandpa f471baac1f0fa821569c42ebb14c4f8533ba77ad
```

### 4.1 生成「本快照 vs 上游基线」补丁（无需 fork，随时可查的追踪能力）

```bash
cd citizenapp/smoldotpow
tmp=$(mktemp -d)
git clone --no-checkout https://github.com/smol-dot/smoldot "$tmp/up"
git -C "$tmp/up" checkout f471baac1f0fa821569c42ebb14c4f8533ba77ad
for d in lib light-base full-node; do
  diff -ruN --exclude=target --exclude=.git "$tmp/up/$d" "./$d"
done > local-vs-upstream.patch
rm -rf "$tmp"
# local-vs-upstream.patch = 当前 PoW+GRANDPA 全部本地改动。
# 其覆盖的文件集必须与 §3 清单一致;多出的文件 = 未登记的漂移,必须登记或回滚。
```

### 4.2 同步上游新版本（在 fork 上做，**绝不在本目录直接手改上游代码**）

```bash
# 在 fork 的 pow-grandpa 分支:
git fetch upstream
git rebase <上游新 tag 或 commit>   # 把本地 PoW 补丁 rebase 到新上游
# 解决冲突 → cargo test + citizenapp 轻端真机联调通过后,再进 4.3。
```

### 4.3 回灌本目录

- 从 fork 新固定提交，把 `lib/` `light-base/` `full-node/` 快照覆盖回本目录（**不带 `.git` / `target`**）。
- 用 4.1 重新生成 `local-vs-upstream.patch`，据此更新 §2 基线提交与 §3 改动文件清单。

### 4.4 硬规则

1. 每次同步都必须更新 §2 基线提交与 §3 改动清单。
2. §3 清单必须与 4.1 补丁覆盖的文件集**逐一对齐**；不一致即视为未登记漂移，必须登记或回滚。
3. `local-vs-upstream.patch` 是临时产物，不入库（`.gitignore` 忽略或用后即删）。
