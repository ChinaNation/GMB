# 任务卡(分步·逐步确认):公民控制台 协议升级(runtime dev-direct 冷签)

> 状态:**设计已确认,分步实现**。用户已拍板 5 点(见下)。工作流:每步先出方案→确认→执行→更新文档/清残留→出下一步。runtime 升级为链上不可逆高危操作,每步谨慎。

## 需求
在公民控制台 **CitizenChain WASM** 卡片、`运行 WASM CI` 按钮**右侧**加「协议升级」按钮:点击对**本机链**做 **runtime 升级**,用 **GitHub 上最新且 CI 成功**的 WASM;签名走**公民钱包扫码冷签**(与节点软件"发协议升级"同一套签名方式,只在控制台再实现一遍),控制台只存 NRC 管理员**公钥**,私钥永不进控制台。

## 用户拍板(锁定)
1. 控制台配置管理员**公钥**;点协议升级弹**二维码→公民钱包扫码签名**(冷签,同节点软件)。
2. **开发期直升**(`developer_direct_upgrade`);链进**运行期后此功能下线**(读链 `DeveloperUpgradeEnabled`,false 即隐藏/禁用)。
3. 协议升级=**仅 runtime 升级**,不做别的。
4. 签名带 **NRC(国储会)CID**;**仅国储会管理员**可直升 runtime。
5. WASM **两个条件都必须**:GitHub 上**最新** + **CI 成功**。

## 机制(节点 as-built,必须逐字节对齐钱包解码器 + 节点 Rust)
- 源:`citizenchain/node/src/governance/runtime_upgrade/{signing,commands,call_data}.rs` + `governance/signing.rs`;pallet `runtime/governance/runtime-upgrade`(`developer_direct_upgrade` call_index=2,`DeveloperUpgradeOrigin`=NRC 管理员,`DeveloperUpgradeCheck::is_enabled()` 关则永久失效)。
- 前端仅 `invoke('build_developer_upgrade_request')` / `invoke('submit_developer_upgrade')`(`node/frontend/governance/runtime-upgrade/api.ts`)。
- QR:`developer_direct_upgrade(cid, wasm, pow_params)` SCALE call → `build_signing_payloads(call, genesis, nonce, spec, tx)` → **WASM 过大是 QR_V1 唯一 hash-only 例外**,QR `body.payload` 发 `signing_bytes`(sr25519 实际签名输入,>256B 为 blake2_256)→ `QrSignRequest{proto,kind,id,expires_at,body{action=chain_action_code(call),sig_alg:1,pubkey(b64),payload(b64)}}`;`expected_payload_hash=sha256(payload_for_qr)`;`immortal` 签块。
- CI 产物:GitHub Actions artifact `citizenchain-wasm`(`.compact.compressed.wasm`),`citizenchain-wasm.yml`,retention 30d。

## 移植风险(最高优先级)
跨语言(Rust→Node/@polkadot)复刻冷签,必须与**钱包解码器**(否则两色 decodeFailed 红拒,见 [[project_qr_signing_two_color]]、[[project_citizenwallet_call_registration_three_points]])+ 节点 Rust 逐字段对齐:action code、QR envelope、call SCALE 编码、ExtrinsicPayload(genesis/nonce/spec/tx/immortal)、签名输入(>256 blake2)、response 格式、signed extrinsic 组装。签名协议单源 [[project_unified_signing_protocol_adr026]] `primitives::sign`。

## 分步
1. **WASM CI 只推 runtime**(✅ 已完成,2026-07-18 收敛):`citizenchainwasm.sh` 调 `ensure_runtime_pushed`(原 `ensure_pushed_commit` 的 `git add -A` 推全部→已废)。规则:WASM CI 只编 runtime,故只提交/推送 `citizenchain/runtime/ + Cargo.toml + Cargo.lock`(其余工作区文件一律不动);有 runtime 改动→只 `git add -- 这三处`+commit+push(head_sha=推后 HEAD);无 runtime 改动→不提交、直接触发 CI 出最新(head_sha=origin 分支 tip,供 run_workflow 匹配)。仅 WASM CI 用此函数,不影响 citizenapp/citizenwallet/citizenchain 的 `require_clean_remote_commit`。
2. **控制台配置 NRC 管理员公钥 + 协议升级按钮**(✅ 已完成 as-built):
   - 位置定稿(用户纠正):在 **CitizenChain WASM** 模块——「运行 WASM CI」按钮**右侧**加「协议升级」按钮(红·副标题「钱包扫码冷签」,`action.dialog=true`,前端特判弹窗不走脚本);**密钥状态表**加「管理员公钥 `NRC_ADMIN_PUBKEY`」行。
   - 后端 `server.mjs`:citizenchainwasm 模块加 `localKeys:[{name:'NRC_ADMIN_PUBKEY',env:'rtupg',desc}]` + `protocol-upgrade` dialog 动作;`secretComments` 加 NRC_ADMIN_PUBKEY;`/api/status` 返回 `localKeys` 存在态;`resolveSecretTarget` + `/api/secret/set|delete` 支持 `store:'local'`(keychainPut/Delete,过 Touch ID)。
   - 前端 `app.js`:`secretRows` 纳入 `localKeys`(store local,密钥表出行);`openSecretEditor` local 用明文 text(公钥公开);动作渲染支持 `dialog`(副标题「钱包扫码冷签」);点击 dialog 动作 → `openProtocolUpgrade`(**当前占位**:未配公钥则提示先配、已配则提示第 3–7 步冷签流程建设中,零链上动作)。
   - NRC CID:不做单独配置项,后续第 4 步从机构目录/链上取 NRC 的 CID(单一已知机构)。
   - 实测(8888):CI 右侧现「协议升级」按钮、密钥表现 NRC_ADMIN_PUBKEY 行、无 console error。
3. **WASM 获取 + 表单**(✅ 已完成):
   - 新增 `rtupg/fetch_wasm.mjs` `fetchLatestSuccessfulWasm`:`gh run list --status success --limit 1`(最新且成功)→ `gh run download <id> --name citizenchain-wasm` → 取 `citizenchain.compact.compressed.wasm` → 缓存 `.runtime/rtupg-latest.wasm` + 返回 {run/commit/时间/大小/sha256/路径}。
   - `server.mjs` `POST /api/rtupg/latest-wasm`(需已配公钥,回 wasm 元数据 + adminPubkey);`app.js` `openProtocolUpgrade`→ 拉取 → 弹表单 `<dialog id=rtupgDialog>`(最新 WASM 只读 + 来源 CI + 时间 + SHA-256 + 管理员公钥 + 取消/确认);取消关闭,确认 → 第 4–5 步(当前占位)。`styles.css` 加 `.rtupg-*`。
   - **顺带修真 bug**:控制台在 launchd 下 PATH 精简(`/usr/bin:/bin:…`)找不到 gh(在 `/opt/homebrew/bin`)→ 之前 `githubSecretNames` 失败致 GitHub 密钥全显未配置。已给所有 `spawnSync('gh')`(list/set/delete)+ fetch 传 `env: baseChildEnv()`(带登录 shell 完整 PATH)。GMB_SSH_KEY 等现应正确显示。
   - 实测(8888,临时占位公钥):点协议升级→控制台进程 gh 成功拉取 run #29636312704(commit 03bf213dc0,1.07MB)→ 表单渲染齐全 → 确认转第 4–5 步占位 → 取消/关闭正常;无 console error;临时公钥+测试 WASM 已清。

  (原第 3 步)~~`gh` 取最新成功 WASM~~ 已并入上文。
4. **冷签请求构造(Node/@polkadot)**(🔶 代码已写,待连节点验证):
   - **协议已完全逆向**(读 call_data.rs / governance/signing.rs / crates/chain-signing):call=pallet12/call2 + `compact(cidLen)+cid + compact(wasmLen)+wasm + powParams(u32+u16+u64+u32+u64+u64 LE)`;extra=`era(immortal 0x00)+compact(nonce)+tip(0x00)+CheckMetadataHash.mode(0x00)`;additional=`specVer(u32)+txVer(u32)+genesis(32)+genesis(32)+None(0x00)`;签名输入=SignedPayload>256B→blake2_256(升级恒 32B);QR=`{p:"QR_V1",k:1,i,e,b:{a:3074,g:1,u:b64url(pubkey),d:b64url(signingBytes)}}`;`expected_payload_hash=sha256(signingBytes)`;QR_KIND 请求=1/响应=2;immortal。
   - **NRC CID**=固定常量 `LN001-NRC0G-944805165-2026`(节点 NrcSection 同源)。
   - 已建 `rtupg/build_request.mjs`(@polkadot 连节点从 metadata 构造 call+ExtrinsicPayload→byte-exact;自校验 call 前两字节=12/2)。**语法 OK**。
   - **✅ 已连节点实测逐字节对齐(2026-07-18)**:用户启动节点(ws://127.0.0.1:9944)后实测——call 头 `0c02`、action 3074、cid 长度前缀 `0x68`、extra=`00`(era)`00`(nonce)`00`(tip)`00`(mode)、additional=`specVer+txVer+genesis×2+00(None)`、签名输入 blake2 32B、QR 信封 `{p:QR_V1,k:1,b:{a:3074,g:1,u,d}}`——与 chain_signing 逐字段一致。**最难一步(错一位红拒)已证实正确**。坑:@polkadot 的 Bytes 参数(cid/wasm)必须传 0x hex(传 Uint8Array 会被当已编码去解码);AuthorizeCall/CheckNonStakeSender 被 @polkadot 视为 no-effect(零字节),与 Rust 的 () 对齐。清理:去掉 debug 的 fullPayloadHex 返回,删测试 wasm。
5. **提交路径 + 签名面板**(✅ 代码完成 + 组件验证;真·E2E 待节点+钱包):
   - `rtupg/tx_common.mjs`:抽 `buildTxAndPayload`(build/submit 共用,保证重建逐字节一致)+ `connectApiWithTimeout`(15s 超时,节点没跑快速失败不挂死)。
   - `rtupg/submit_request.mjs`(对应节点 verify_and_submit):解析回执 `{k:2,b:{u,s}}` → 校验 proto/kind/id/expires/pubkey → 用会话同一 signNonce 重建 → payload 哈希一致性(powParams 漂移则拒)→ `sr25519Verify` → `addSignature`(MultiSignature Sr25519 变体前缀 `0x01`)→ `system_dryRun` 预检 → `author_submitExtrinsic`。
   - `server.mjs`:内存会话 Map(TTL 90s)+ `POST /api/rtupg/build-request`(读 rtupg:NRC_ADMIN_PUBKEY + NRC CID + 缓存 WASM + topup:NODE_WS → 存会话回 requestJson)+ `POST /api/rtupg/submit`(会话校验→submit_request)。
   - 前端 `app.js`:确认 → build-request → 复刻 CitizenSignaturePanel 两栏(左请求 QR + 倒计时,右相机 getUserMedia+jsQR 扫回执)→ submit → 显示 txHash。vendor `web/vendor/{qrcode.js,jsQR.js}`(本地不走 CDN,serveStatic 放行,index.html 加载)。
   - **✅ 后端全链路已验(节点在跑时)**:真临时 sr25519 密钥跑 build→sign→submit,dry-run 到达 `{"invalid":{"payment":null}}`(随机账户没钱)=签名验过+组装正确,非 BadProof/解码错。**✅ 前端 QR 客户端验**:vendor qrcode 对 201 字样例生成 53×53 SVG。
   - **未完(节点中途掉线,矿工端我不启)**:走控制台进程的 build-request→真 QR→相机扫→submit 的整链 UI 实测,及真钱包扫码上链 = 第 7 步(需节点在跑 + 真钱包 + 真 NRC 管理员密钥+费账户)。
5. **弹窗 QR + 扫码回签 + 验签 + 提交**:控制台弹 QR(qrcode)→ 公民钱包扫码签 → 回填/回扫 signature → 验 pubkey+payload_hash → 组 signed extrinsic → 提交本机节点 → 等 finalized。
6. **开发期门禁**:读链 `DeveloperUpgradeEnabled`;false 则「协议升级」按钮隐藏/禁用(运行期下线)。
7. **E2E**:真公民钱包扫码签 + 真本机链 dev-direct 升级到 GitHub 最新成功 WASM。

## 验收
一键取 GitHub 最新成功 WASM → 弹 QR → 公民钱包冷签 → 本机链 dev-direct 升级成功(spec_version 提升)、finalized;运行期后按钮下线;私钥全程不进控制台。
