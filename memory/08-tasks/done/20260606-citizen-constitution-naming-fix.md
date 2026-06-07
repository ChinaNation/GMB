# 任务卡：公民宪法正文命名/译名/字形统一修订

## 任务需求

对 runtime 内置《公民宪法》原文做一轮命名、中英译名、字形与大小写统一修订（只动文字，不改条文结构与法律含义）。来源为前序只读检查报告中用户逐条确认的 10 项。

## 修改范围

- `citizenchain/runtime/primitives/src/CitizenConstitution.html`（唯一改动文件，被 `include_str!` 编入 WASM）

逐项：

1. 储备银行命名统一：全称「公民储备银行 / Citizen Reserve Bank」，简称「省储行 / Provincial Reserve Bank」。非标准的「省储备银行 / provincial reserve bank」全部改全称；第137条简称英文保留 Provincial Reserve Bank。
2. 部/厅/局英译分级：部=Department/Secretary（不动）；厅=Provincial Department/Director-General（改第57条厅名、第58条厅长头衔、第11/139条财税厅）；局=Bureau/Director（不动，已与 Department 天然区分；第34/39条泛指公民安全局不改，避免引入新不一致）。
3. （撤销）监察官 2/2/4 为有意设计，不改。
4. 第55条「对外代表中国 / represent China」→「对外代表中华民族联邦共和国 / represent the Federal Republic of the China Nation」。
5. （撤销）第93条「联邦司法院」措辞正确（省司法院属联邦司法院），不改。
6. 注册局英译统一为 Registration：第52条 Federal Registry Bureau → Federal Registration Bureau（第43条通指仍为 Registration Bureau）。
7. 字形：第52条第四款「剩馀」→「剩余」。
8. 国语=国家官方语言：第5条「中文普通话」→「中文普通话（国语）/ Standard Chinese (the national language)」；第10条第五款英文「the national language」→「Standard Chinese」。
9. 英文专名大小写：省/市级 Legislative Yuan、Senate、House of Representatives、Legislative Council、Judicial Yuan、Control Yuan 统一 Title Case。本轮不动 government / administrative region / 各 committee（多为通名用法，留作可选后续）。
11. 第3条「民治/民主/民权/民生/民族」英文加拼音 mínzhì/mínzhǔ/mínquán/mínshēng/mínzú。

## 目标状态

见“验证记录”，全部达成。

## 执行记录

- 全局 perl 替换（byte-literal）：
  - CN：省储备银行→公民储备银行(8)、对外代表中国→对外代表中华民族联邦共和国(1)、剩馀→剩余(1)。
  - EN：provincial reserve bank(s)→Citizen Reserve Bank(s)(9)、represent China externally→represent the Federal Republic of the China Nation externally(1)、Federal Registry Bureau→Federal Registration Bureau(1)、Department of Finance and Taxation of that province→Provincial Department of Finance and Taxation(2)。
  - #9 大小写：provincial/municipal 的 Legislative Yuan/Senate/House(es) of Representatives/Judicial Yuan/Control Yuan/Legislative Council 统一 Title Case（含单复数，约 180 处）。
- 定点 Edit 11 处：
  - 第137条简称 EN→Provincial Reserve Bank。
  - 第3条 EN 五民加拼音。
  - 第5条 CN 中文普通话（国语）+ EN Standard Chinese (the national language)。
  - 第10条第五款 EN→Standard Chinese。
  - 第57条 9 个厅名→Provincial Department of X（用“nine ... department-level”前缀定位，避免误伤第53条国家部清单）。
  - 第58条厅长/副厅长 5 句→Director-General/Deputy Director-General（复数 Directors-General）。
- 按裁定保持不变：第53条国家“部”仍 Department/Secretary；第64/65条市“局”仍 Bureau/Director。

## 验证记录

- 残留旧形态全 0：`省储备银行`/`对外代表中国`/`剩馀`/`provincial reserve bank`/`Federal Registry Bureau`/`represent China externally`/`abbreviated as a Citizen Reserve Bank`/`the national language shall be a compulsory language`/`Department of Finance and Taxation of that province`/小写院会名（grep 全 0）。
- 新形态计数：公民储备银行 22、Citizen Reserve Bank 22、Provincial Reserve Bank 1、Provincial Department of 11、Director-General 18、Federal Registration Bureau 1、`中文普通话（国语）` 1、`Standard Chinese (the national language)` 1、mínzhì/mínzú 各 1、对外代表中华民族联邦共和国 1。
- 未误伤：第53条国家 10 部仍 `Department of`（Department of Foreign Affairs 保留）；第64/65条市局块仅 `Director`（无 Director-General）。
- 结构完好：article 锚点 140、`</main>` 1、`<script>` 1，内联脚本与 base64 国徽未触碰。
- 未执行 cargo build / 未 bump spec_version（纯 HTML 文案变更，必编译通过）。正式链生效需后续单独发布 runtime 升级（setCode）；`citizen_constitution_blake2_256` 摘要将随之变化。

## 后续（非本卡范围）

- runtime 升级发布（setCode）使主网生效。
- 可选：government / administrative region / 各 committee 的省市级 Title Case 二次统一；局是否统一加 Municipal 前缀。
