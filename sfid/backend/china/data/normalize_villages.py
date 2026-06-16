#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Phase-2 行政区命名统一：村级统一为 村/路 + 兵团残余与镇级未明确项分类。

承接 normalize_towns.py（镇级已统一）。本脚本处理：
  #1 村级机械改名（villages 表，~71 万行）
       村系(村民委员会/村委会/X村) + 嘎查        -> 「X村」
       居系(居民委员会/居委会/社区)              -> 「X路」
  #2 村级残余 6,162（兵团/生活区类，非村非居非社区非嘎查）-> 村 / 路 / 无法区分
  #3 镇级残余 1,284（normalize_towns 保留的非「镇」）   -> 镇 / 不能确认

权威依据：国家统计局《统计用区划代码和城乡划分代码编制规则》
  - 乡级行政区（民政部认可）：镇、乡、民族乡、苏木、民族苏木、街道办事处、区公所
  - 类似乡级单位（统计编码、民政未认可）：开发区、工矿区、农场、林场 等
        —— 有辖区 + 乡级管理职能 -> 本系统三级模型里归「镇」
  - 村级：村民委员会(村)/社区居民委员会(社区)/牧民委员会(牧区)/嘎查(内蒙古=村)
  - 纯设施/机构（监狱/水库/风景区/科研中心/管理处局站）非territorial -> 不能确认
  - 裸名/退化名（无地名前缀，如「经济技术开发区」）无法形成 X镇 -> 不能确认

唯一键：villages=(province_code,city_code,town_code,code)，towns=(province_code,city_code,code)。
改名仅展示层，不影响 SFID 号（只编省+市 R5），不动链。幂等：已是目标后缀则跳过。

用法：
    python3 normalize_villages.py --scan
        只读：打印各桶统计 + 样例 + 最终「无法分别」总数；
        写审计 CSV：villages_undecided.csv / towns_unconfirmed.csv / *_preview.csv
    python3 normalize_villages.py --apply
        套全部规则写回 china.sqlite（villages #1+#2，towns #3），打印账目。
"""
from __future__ import annotations

import argparse
import csv
import os
import sqlite3
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
DB_PATH = os.path.join(HERE, "china.sqlite")

CUN = "村"
LU = "路"
ZHEN = "镇"

# ---------------- 村级 #1：委员会后缀（长在前，最长优先 strip） ----------------
CUN_MARKERS = ("村民委员会", "村民委", "村民小组", "村委会", "村委", "行政村")
# 含城镇街道居委会旧称「X街道委员会 / X街委员会」-> 路
LU_MARKERS = (
    "社区居民委员会", "街道居民委员会", "社区居委会", "居民委员会",
    "街道委员会", "街委员会", "居委会", "社区",
)
# 强制路：居系一律以「路」结尾。下列道路尾词整词替换成「路」（长在前，最长优先）。
# 用户决策(2026-06-15)：街/道/巷/弄/里 全部统一改「路」，零例外。
ROAD_REPLACE_TAILS = ("街道", "胡同", "大街", "大道", "街", "道", "巷", "弄", "条", "里")

# ---------------- 村级 #2：残余关键词 ----------------
# 工矿/园区/监管设施 -> 无法区分（虚拟社区或非居住）
VILLAGE_UNDECIDED_KW = (
    "工业", "科技", "高新", "开发区", "保税", "园区", "产业园", "工业园",
    "监狱", "劳教", "戒毒", "看守", "虚拟", "指挥部", "管委会",
)
# 乡村基层生产单位 -> 村
VILLAGE_CUN_KW = (
    "连", "分场", "生产队", "牧业队", "作业区", "大队", "良种场", "种畜场",
    "原种场", "农场", "牧场", "林场", "渔场", "茶场", "盐场", "管理区",
    "生活区", "牧委会", "渔委会", "家委会", "队", "场",
)
# #2 中用于剥离得基底的单位尾词（剥后 + 村）
VILLAGE_UNIT_TAILS = ("生活区", "生产队", "牧业队", "作业区", "管理区", "大队", "队")

# ---------------- 镇级 #3：分类关键词 ----------------
# 正式乡级（民政认可）
TOWN_FORMAL_KW = ("街道", "办事处", "苏木")
# 纯设施/机构/非居住 -> 不能确认（优先于类似乡级判定）
TOWN_FACILITY_KW = (
    "监狱", "劳教", "戒毒", "看守", "水库", "灌区", "水利", "水电",
    "风景", "景区", "度假", "公园", "旅游", "保护区", "保护站", "苗圃",
    "农科", "科研", "试验", "科学院", "研究", "实验", "观测", "气象",
    "指挥部", "管理站", "管理处", "服务", "中心", "基地", "委员会",
    "渔委", "家委", "牧委", "半岛", "群岛", "林业局", "管理局", "哨所",
    "局直",
)
# 类似乡级单位（有辖区+人口）-> 镇
TOWN_ZHEN_KW = (
    "开发区", "经济区", "经济技术", "高新", "工业园", "产业园", "科技园",
    "科技", "科创", "园区", "工业区", "保税", "自贸", "自由贸易", "出口加工",
    "示范区", "新区", "新城", "软件", "智谷", "创意", "商贸园", "农场",
    "牧场", "渔场", "盐场", "茶场", "林场", "林区", "农垦", "垦区", "垦",
    "华侨", "矿区", "煤矿", "矿务局", "矿", "团", "城", "管理区", "特区",
)
# 退化基底：剥掉乡级后缀后若基底只是通用限定词（非地名），无法成具体镇名 -> 不能确认
DEGENERATE_BASES = frozenset({
    "工业", "经济", "经济技术", "高新", "高新技术", "产业", "科技", "综合",
    "现代", "农业", "商贸", "物流", "保税", "出口加工", "自由贸易", "新", "老",
    "旅游", "循环经济", "循环", "生态", "创业", "化工", "合作", "临港", "临空",
    "空港", "高铁", "滨江", "沿江", "城", "园", "服务", "行政", "管理", "示范",
    "自贸", "国际", "中心", "金融", "文化", "科学", "教育", "医药", "汽车",
})
# #3 用于剥离得地名基底的尾词（长在前），剥后 + 镇
TOWN_ZHEN_SUFFIXES = (
    "经济技术开发区", "高新技术产业开发区", "高新技术开发区", "经济开发区",
    "高新技术产业园区", "高新技术园区", "综合保税区", "综合保税港区",
    "保税港区", "出口加工区",
    "自由贸易区", "工业园区", "产业园区", "科技园区", "科技工业园",
    "高新区", "开发区", "工业园", "产业园", "科技园", "科创园", "软件园",
    "创意园", "商贸园", "园区", "工业区", "保税区", "自贸区", "示范区",
    "新区", "新城", "管理区", "矿务局", "煤矿", "矿区", "林场", "林区",
    "农场", "牧场", "渔场", "盐场", "茶场", "农垦", "垦区",
    "旅游经济特区", "经济特区", "特区",
    "街道办事处", "街道",
)


def _strip_one(name: str, suffixes: tuple[str, ...]) -> tuple[str, bool]:
    """剥掉首个命中的尾缀（按给定顺序，应长在前）。返回 (基底, 是否剥到)。"""
    for s in suffixes:
        if name.endswith(s) and len(name) > len(s):
            return name[: -len(s)], True
    return name, False


# ============================ 村级 #1 + #2 ============================
def to_cun(name: str) -> str:
    base, _ = _strip_one(name, CUN_MARKERS)
    if base.endswith("嘎查") and len(base) > 2:
        base = base[:-2]
    if not base:
        return name
    if base.endswith(CUN):
        return base
    return base + CUN


def _ensure_lu(base: str) -> str:
    """把基底变成以「路」结尾：已是路保持；街/道/巷/弄/里/街道等整词换「路」；否则加「路」。"""
    if base.endswith(LU):
        return base
    for t in ROAD_REPLACE_TAILS:
        if base.endswith(t) and len(base) > len(t):
            return base[: -len(t)] + LU
    return base + LU


def to_lu(name: str) -> str:
    base, _ = _strip_one(name, LU_MARKERS)
    if not base:
        return name
    return _ensure_lu(base)


def residual_to_cun(name: str) -> str:
    base, _ = _strip_one(name, VILLAGE_UNIT_TAILS)
    if not base:
        return name
    if base.endswith(CUN):
        return base
    return base + CUN


def classify_village(name: str) -> tuple[str, str]:
    """返回 (bucket, new_name)；bucket ∈ {noop, cun, lu, undecided}。"""
    if name.endswith(CUN) and "社区" not in name and "居" not in name:
        # 已是 X村
        return "noop", name
    if name.endswith(CUN_MARKERS) or name.endswith("嘎查"):
        return "cun", to_cun(name)
    if name.endswith(LU_MARKERS):
        return "lu", to_lu(name)
    if CUN in name and "社区" not in name and "居" not in name:
        return "cun", to_cun(name)
    if "居" in name or "社区" in name:
        return "lu", to_lu(name)
    # ---- #2 残余 ----
    if any(k in name for k in VILLAGE_UNDECIDED_KW):
        return "undecided", name
    if any(k in name for k in VILLAGE_CUN_KW):
        return "cun", residual_to_cun(name)
    return "undecided", name


# ============================ 镇级 #3 ============================
def town_to_zhen(name: str) -> str | None:
    """剥地名基底 + 镇；裸名/退化(基底空)返回 None。"""
    base, hit = _strip_one(name, TOWN_ZHEN_SUFFIXES)
    if not hit:
        # 含关键词但无规整尾缀（如「东莞生态园」）：无法干净成名
        return None
    if not base or len(base) < 1:
        return None
    if base in DEGENERATE_BASES:
        # 裸名/退化（如「工业园区」「经济技术开发区」）：基底是通用限定词非地名
        return None
    if base.endswith(ZHEN):
        return base
    return base + ZHEN


def classify_town3(name: str) -> tuple[str, str]:
    """非镇 town -> (bucket, new_name)；bucket ∈ {noop, zhen, unconfirmed}。"""
    if name.endswith(ZHEN):
        return "noop", name
    # 1) 正式乡级 -> 镇
    if any(k in name for k in TOWN_FORMAL_KW):
        new = town_to_zhen(name)
        return ("zhen", new) if new else ("unconfirmed", name)
    # 2) 纯设施/机构 -> 不能确认
    if any(k in name for k in TOWN_FACILITY_KW):
        return "unconfirmed", name
    # 3) 类似乡级单位 -> 镇（需可成名）
    if any(k in name for k in TOWN_ZHEN_KW):
        new = town_to_zhen(name)
        return ("zhen", new) if new else ("unconfirmed", name)
    # 4) 无法识别 -> 不能确认
    return "unconfirmed", name


# ============================ 扫描 / 应用 ============================
def _samples(rows: list[tuple[str, str]], n: int = 12) -> list[tuple[str, str]]:
    return rows[:n]


def scan(conn: sqlite3.Connection) -> None:
    # ---------- villages ----------
    v_rows = conn.execute("SELECT name FROM villages").fetchall()
    v_stat: dict[str, int] = defaultdict(int)
    v_sample: dict[str, list[tuple[str, str]]] = defaultdict(list)
    undecided_rows: list[str] = []
    for (name,) in v_rows:
        bucket, new = classify_village(name)
        v_stat[bucket] += 1
        if len(v_sample[bucket]) < 12 and new != name:
            v_sample[bucket].append((name, new))
        if bucket == "undecided":
            undecided_rows.append(name)

    # ---------- towns (#3) ----------
    t_rows = conn.execute(
        "SELECT province_code, city_code, code, name FROM towns WHERE name NOT LIKE '%镇'"
    ).fetchall()
    t_stat: dict[str, int] = defaultdict(int)
    t_sample: dict[str, list[tuple[str, str]]] = defaultdict(list)
    unconfirmed_rows: list[tuple[str, str, str, str]] = []
    for pc, cc, code, name in t_rows:
        bucket, new = classify_town3(name)
        t_stat[bucket] += 1
        if len(t_sample[bucket]) < 14:
            t_sample[bucket].append((name, new))
        if bucket == "unconfirmed":
            unconfirmed_rows.append((pc, cc, code, name))

    # ---------- 写审计 CSV ----------
    with open(os.path.join(HERE, "villages_undecided.csv"), "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f)
        w.writerow(["name"])
        for n in undecided_rows:
            w.writerow([n])
    with open(os.path.join(HERE, "towns_unconfirmed.csv"), "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f)
        w.writerow(["province_code", "city_code", "code", "name"])
        w.writerows(unconfirmed_rows)

    # ---------- 打印 ----------
    print("=" * 66)
    print(f"villages 总数: {len(v_rows)}")
    print(f"  #1 村系/嘎查 -> 村 (cun)      : {v_stat['cun']}")
    print(f"  #1 居系/社区 -> 路 (lu)       : {v_stat['lu']}")
    print(f"     已是 X村 (noop)            : {v_stat['noop']}")
    print(f"  #2 残余 无法区分 (undecided)  : {v_stat['undecided']}  -> villages_undecided.csv")
    print("-" * 66)
    for b, label in (("cun", "村样例"), ("lu", "路样例"), ("undecided", "无法区分样例")):
        print(f"  [{label}]")
        if b == "undecided":
            for n in undecided_rows[:14]:
                print(f"      {n}  (保留)")
        else:
            for old, new in v_sample[b]:
                print(f"      {old}  ->  {new}")
    print("=" * 66)
    print(f"towns 非镇残余: {len(t_rows)}")
    print(f"  #3 判定为镇且可成名 (zhen)    : {t_stat['zhen']}")
    print(f"  #3 不能确认/裸名 (unconfirmed): {t_stat['unconfirmed']}  -> towns_unconfirmed.csv")
    print("-" * 66)
    print("  [镇样例]")
    for old, new in t_sample["zhen"]:
        print(f"      {old}  ->  {new}")
    print("  [不能确认样例]")
    for old, _ in t_sample["unconfirmed"]:
        print(f"      {old}  (保留)")
    print("=" * 66)
    total_undef = v_stat["undecided"] + t_stat["unconfirmed"]
    print(">>> 全部分类完成后『无法分别』总数 = "
          f"村级无法区分 {v_stat['undecided']} + 镇级不能确认 {t_stat['unconfirmed']} = {total_undef}")
    print("=" * 66)


def apply_changes(conn: sqlite3.Connection) -> None:
    # ---------- villages ----------
    v_rows = conn.execute("SELECT rowid, name FROM villages").fetchall()
    v_updates: list[tuple[str, int]] = []
    v_stat: dict[str, int] = defaultdict(int)
    for rowid, name in v_rows:
        bucket, new = classify_village(name)
        v_stat[bucket] += 1
        if new != name:
            v_updates.append((new, rowid))
    conn.executemany("UPDATE villages SET name = ? WHERE rowid = ?", v_updates)

    # ---------- towns (#3) ----------
    t_rows = conn.execute(
        "SELECT province_code, city_code, code, name FROM towns WHERE name NOT LIKE '%镇'"
    ).fetchall()
    t_updates: list[tuple[str, str, str, str]] = []
    t_stat: dict[str, int] = defaultdict(int)
    for pc, cc, code, name in t_rows:
        bucket, new = classify_town3(name)
        t_stat[bucket] += 1
        if new != name:
            t_updates.append((new, pc, cc, code))
    conn.executemany(
        "UPDATE towns SET name = ? WHERE province_code = ? AND city_code = ? AND code = ?",
        t_updates,
    )
    conn.commit()

    # ---------- 复核 ----------
    # 注意：路类不止以「路」结尾，街/道/巷/弄/条/胡同/里 等道路名按原样保留也算路类，
    # 故复核须按「非村类且非任何道路名」才是真·无法区分。
    road_like = "(name LIKE '%路' OR name LIKE '%街' OR name LIKE '%道' OR name LIKE '%巷' " \
                "OR name LIKE '%弄' OR name LIKE '%条' OR name LIKE '%胡同' OR name LIKE '%里')"
    v_cun = conn.execute("SELECT COUNT(*) FROM villages WHERE name LIKE '%村'").fetchone()[0]
    v_lu = conn.execute(f"SELECT COUNT(*) FROM villages WHERE name NOT LIKE '%村' AND {road_like}").fetchone()[0]
    v_other = conn.execute(
        f"SELECT COUNT(*) FROM villages WHERE name NOT LIKE '%村' AND NOT {road_like}"
    ).fetchone()[0]
    t_non_zhen = conn.execute("SELECT COUNT(*) FROM towns WHERE name NOT LIKE '%镇'").fetchone()[0]

    print("=" * 66)
    print(f"villages 改名 {len(v_updates)} 条；towns(#3) 改名 {len(t_updates)} 条。")
    print(f"村级现状: 以村结尾 {v_cun} | 以路结尾 {v_lu} | 其它(无法区分) {v_other}")
    print(f"镇级现状: 仍非镇(不能确认) {t_non_zhen}")
    print(f">>> 『无法分别』总数 = 村级 {v_other} + 镇级 {t_non_zhen} = {v_other + t_non_zhen}")
    print("=" * 66)


def force_lu(conn: sqlite3.Connection) -> None:
    """一次性把存量路类(以街/道/巷/弄/条/胡同/里/街道结尾)整词改成以「路」结尾。
    幂等：已是「路」的跳过；只命中道路尾词，绝不碰村类(以村结尾)与无法区分(区/会/县…)。"""
    rows = conn.execute("SELECT rowid, name FROM villages WHERE name NOT LIKE '%路'").fetchall()
    updates: list[tuple[str, int]] = []
    for rowid, name in rows:
        for t in ROAD_REPLACE_TAILS:
            if name.endswith(t):
                base = name[: -len(t)]
                new = (base + LU) if base else LU  # 退化(名=路词,如「胡同」)→「路」
                if new != name:
                    updates.append((new, rowid))
                break
    conn.executemany("UPDATE villages SET name = ? WHERE rowid = ?", updates)
    conn.commit()

    road_left = conn.execute(
        "SELECT COUNT(*) FROM villages WHERE name NOT LIKE '%村' AND name NOT LIKE '%路' AND ("
        "name LIKE '%街' OR name LIKE '%道' OR name LIKE '%巷' OR name LIKE '%弄' "
        "OR name LIKE '%条' OR name LIKE '%胡同' OR name LIKE '%里')"
    ).fetchone()[0]
    v_cun = conn.execute("SELECT COUNT(*) FROM villages WHERE name LIKE '%村'").fetchone()[0]
    v_lu = conn.execute("SELECT COUNT(*) FROM villages WHERE name LIKE '%路'").fetchone()[0]
    v_other = conn.execute(
        "SELECT COUNT(*) FROM villages WHERE name NOT LIKE '%村' AND name NOT LIKE '%路'"
    ).fetchone()[0]
    print("=" * 66)
    print(f"force-lu 改名 {len(updates)} 条。")
    print(f"村级现状: 以村结尾 {v_cun} | 以路结尾 {v_lu} | 其余(无法区分) {v_other}")
    print(f"残留道路尾词(街/道/巷/弄/里…)非路结尾: {road_left}（应为 0）")
    print("=" * 66)


# ============================ 重构(phase 3) ============================
# 用户决策(2026-06-15)：
#  - 「不能确认镇」凡有正常村/路子节点者 = 它就是这些村/路的乡级父级(类似乡级单位)
#    → 改名为「镇」(给村/路一个真实的镇父级),不删。
#  - 香港岛/新界/澳门半岛 = 退化中间层,其下的区/堂区提升为「镇」直挂市,中间层删。
#  - 纯设施/退化单位(监狱/种畜场/良种场/苗圃/试验站/金门县…只挂虚拟子节点)→ 整删。
#  - 所有「无法区分」村(虚拟生活区/管委会/监狱管理区/区/堂区…)→ 删。

# 香港市/新界市/澳门市 的三个退化中间层(其下区/堂区提为镇)
HK_MACAU_DISSOLVE = {("LN", "001", "001"), ("LN", "002", "001"), ("LN", "003", "001")}

# A 类「不能确认镇」改镇用的后缀(长在前，最长优先)
RESTRUCTURE_ZHEN_SUFFIXES = (
    "区直辖村级区划", "国家级森林公园", "国家森林公园", "风景名胜区", "森林公园",
    "社区行政事务管理中心", "行政事务管理中心", "行政管理服务中心", "行政管理中心",
    "社区管理服务中心", "社区服务管理中心", "社区服务中心", "社区管理中心",
    "经济技术开发区", "高新技术产业开发区", "高新技术产业园区", "高新技术开发区",
    "循环经济示范园", "经济技术开发区", "经济开发区", "经济发展区", "经济合作区",
    "生态旅游度假区", "旅游度假区", "生态示范城", "现代农业示范区",
    "产业集聚区", "产业集中区", "工业集中区", "综合保税港区", "综合保税区",
    "保税港区", "出口加工区", "石油管理局", "管理委员会", "经济特区",
    "服务中心", "管理中心", "托管区", "化工园区", "化工园", "物流园区", "物流园",
    "创业园区", "创业园", "农业园区", "商贸园", "集聚区", "合作区",
    "开发区", "发展区", "度假区", "示范城", "示范区", "示范园", "保税区",
    "工业园区", "产业园区", "科技园区", "工业园", "产业园", "科技园", "生态园", "园区",
    "集中区", "工业区", "高新区", "经开区", "新区", "新城", "城区", "片区", "投资区",
    "林业局", "管理局", "管理处", "管委会", "总场", "分场",
    "农场", "牧场", "渔场", "盐场", "茶场", "林场", "林区", "矿务局", "矿区", "煤矿",
    "企业集团", "集团", "特区", "苏木", "街道办事处", "街道", "团",
)


def _strip_parens(name: str) -> str:
    """去掉结尾的（…）/(…) 括注。"""
    for lp, rp in (("（", "）"), ("(", ")")):
        if name.endswith(rp) and lp in name:
            name = name[: name.rfind(lp)]
    return name.strip()


def derive_zhen_name(name: str) -> str | None:
    """A 类不能确认镇 → 「X镇」名；迭代剥尾词直到见底；裸名/退化返回 None。"""
    n = _strip_parens(name)
    if n.startswith("兵团") and len(n) > 2:
        n = n[2:]
    for _ in range(8):  # 迭代剥离，最多 8 轮足够
        n = _strip_parens(n)
        if n.endswith(ZHEN):
            return n
        hit = False
        for suf in RESTRUCTURE_ZHEN_SUFFIXES:
            if n.endswith(suf) and len(n) > len(suf):
                n = n[: -len(suf)]
                hit = True
                break
        if not hit:
            break
    if n.endswith(ZHEN):
        return n
    if not n or n in DEGENERATE_BASES:
        return None
    return n + ZHEN


def district_to_zhen(name: str) -> str:
    """香港/澳门 区/堂区 → 镇(东区→东镇、大堂区→大堂镇、圣安多尼堂区→圣安多尼堂镇)。"""
    if name.endswith("区") and len(name) > 1:
        return name[:-1] + ZHEN
    return name + ZHEN


def _is_legit_village(vn: str) -> bool:
    return vn.endswith(CUN) or vn.endswith(LU)


def restructure(conn: sqlite3.Connection, apply: bool) -> None:
    towns = conn.execute(
        "SELECT province_code, city_code, code, name FROM towns WHERE name NOT LIKE '%镇' "
        "ORDER BY province_code, city_code, code"
    ).fetchall()

    renames: list[tuple[str, str, str, str]] = []       # (new, pc, cc, code)  A类→镇
    unnameable: list[tuple[str, str, str, str, int]] = []  # A类有子但裸名无法成镇
    town_deletes: list[tuple[str, str, str]] = []       # 纯设施 + 港澳新界中间层
    promoted: list[tuple[str, str, str, str]] = []       # (pc, cc, new_code, 镇名) 区/堂区提升

    for pc, cc, code, name in towns:
        children = conn.execute(
            "SELECT name FROM villages WHERE province_code=? AND city_code=? AND town_code=? ORDER BY code",
            (pc, cc, code),
        ).fetchall()
        legit = [vn for (vn,) in children if _is_legit_village(vn)]

        if (pc, cc, code) in HK_MACAU_DISSOLVE:
            for i, (vn,) in enumerate(children, start=1):
                promoted.append((pc, cc, f"{i:03d}", district_to_zhen(vn)))
            town_deletes.append((pc, cc, code))
            continue

        if legit:
            zn = derive_zhen_name(name)
            if not zn:
                # 裸名(无地名)但有子节点：兜底成镇,保证每个 town 都以镇结尾(零例外)
                base = _strip_parens(name)
                for s in ("管理委员会", "管委会", "管理处", "管理局", "管理服务中心"):
                    if base.endswith(s) and len(base) > len(s):
                        base = base[: -len(s)]
                        break
                zn = base if base.endswith(ZHEN) else ((base + ZHEN) if base else name + ZHEN)
                unnameable.append((pc, cc, code, name, len(legit)))
            renames.append((zn, pc, cc, code))
        else:
            town_deletes.append((pc, cc, code))

    # 待删无法区分村(全量):非村且非路
    junk_villages = conn.execute(
        "SELECT COUNT(*) FROM villages WHERE name NOT LIKE '%村' AND name NOT LIKE '%路'"
    ).fetchone()[0]

    print("=" * 70)
    print(f"非镇 town 总数: {len(towns)}")
    print(f"  A 有正常村/路 → 改镇: {len(renames)}（其中裸名兜底通用名 {len(unnameable)}）")
    print(f"  港澳新界中间层删除 + 区/堂区提升为镇: 删 3 镇, 提升 {len(promoted)} 个镇")
    print(f"  纯设施/退化单位整删: {len(town_deletes) - 3}")
    print(f"  无法区分村(全删): {junk_villages}")
    print("-" * 70)
    print("  [A类改镇样例]")
    for zn, pc, cc, code in renames[:16]:
        old = conn.execute("SELECT name FROM towns WHERE province_code=? AND city_code=? AND code=?", (pc, cc, code)).fetchone()[0]
        print(f"      {old}  ->  {zn}")
    print("  [港澳新界 区/堂区 提升为镇]")
    for pc, cc, ncode, zn in promoted:
        print(f"      {pc}-{cc}-{ncode}  {zn}")
    if unnameable:
        print("  [裸名兜底通用名镇,样例]")
        for pc, cc, code, name, n in unnameable[:12]:
            zn = next((r[0] for r in renames if r[1:] == (pc, cc, code)), "?")
            print(f"      {name}  ->  {zn}  (有 {n} 个村/路)")
    print("  [纯设施整删样例]")
    for pc, cc, code in town_deletes[:14]:
        nm = conn.execute("SELECT name FROM towns WHERE province_code=? AND city_code=? AND code=?", (pc, cc, code)).fetchone()
        if nm:
            print(f"      {nm[0]}")
    print("=" * 70)

    if not apply:
        print(">>> dry-run，未写库。加 --go 落库。")
        return

    # 1) 先删纯设施镇 + 港澳新界中间层(必须在 INSERT 提升镇之前,否则 code 001 会被连带删)
    conn.executemany(
        "DELETE FROM towns WHERE province_code=? AND city_code=? AND code=?", town_deletes
    )
    # 2) 提升区/堂区为镇(INSERT 新 town,此时中间层已删,code 001 不再冲突)
    for pc, cc, ncode, zn in promoted:
        so = int(ncode)
        conn.execute(
            "INSERT INTO towns(province_code, city_code, code, name, sort_order) VALUES(?,?,?,?,?)",
            (pc, cc, ncode, zn, so),
        )
    # 3) A类改镇
    conn.executemany(
        "UPDATE towns SET name=? WHERE province_code=? AND city_code=? AND code=?", renames
    )
    # 4) 删无法区分村(含港澳区/堂区原村行、纯设施子节点、A类垃圾子节点)
    conn.execute("DELETE FROM villages WHERE name NOT LIKE '%村' AND name NOT LIKE '%路'")
    conn.commit()

    t_non_zhen = conn.execute("SELECT COUNT(*) FROM towns WHERE name NOT LIKE '%镇'").fetchone()[0]
    t_zhen = conn.execute("SELECT COUNT(*) FROM towns WHERE name LIKE '%镇'").fetchone()[0]
    v_junk = conn.execute("SELECT COUNT(*) FROM villages WHERE name NOT LIKE '%村' AND name NOT LIKE '%路'").fetchone()[0]
    v_total = conn.execute("SELECT COUNT(*) FROM villages").fetchone()[0]
    orphan = conn.execute(
        "SELECT COUNT(*) FROM villages v LEFT JOIN towns t "
        "ON v.province_code=t.province_code AND v.city_code=t.city_code AND v.town_code=t.code "
        "WHERE t.id IS NULL"
    ).fetchone()[0]
    print("已落库。")
    print(f"  镇(以镇结尾): {t_zhen} | 仍非镇(裸名保留): {t_non_zhen}")
    print(f"  村级总数: {v_total} | 无法区分残留: {v_junk}（应0）| 孤儿村(父镇不存在): {orphan}")
    print("=" * 70)


def main() -> None:
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--scan", action="store_true", help="只读扫描 + 审计 CSV")
    g.add_argument("--apply", action="store_true", help="套规则写回 sqlite")
    g.add_argument("--force-lu", action="store_true", help="存量路类强制以「路」结尾")
    g.add_argument("--restructure", action="store_true", help="phase3:A类改镇/港澳提升/纯设施删/无法区分村删")
    ap.add_argument("--go", action="store_true", help="配合 --restructure 真正落库")
    args = ap.parse_args()

    if not os.path.exists(DB_PATH):
        print(f"找不到 {DB_PATH}", file=sys.stderr)
        sys.exit(1)

    conn = sqlite3.connect(DB_PATH)
    try:
        if args.scan:
            scan(conn)
        elif args.force_lu:
            force_lu(conn)
        elif args.restructure:
            restructure(conn, apply=args.go)
        else:
            apply_changes(conn)
    finally:
        conn.close()


if __name__ == "__main__":
    main()
