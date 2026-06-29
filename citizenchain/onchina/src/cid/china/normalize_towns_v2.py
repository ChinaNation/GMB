#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""china.sqlite 镇名残留清理 v2 —— 任务卡 20260621-143758-china-sqlite

按用户规则清理 towns 表里的残留镇名:
  规则1  省名/老地级市/区 前缀残留        -> 剥离成真实镇名(撞名并入已有镇)
  规则2  纯设施/产业名(无真实地址段)       -> 删除
  规则3a 开发区/保税区/示范园/高新等       -> 有真实地址段改名保留; 无地址段删除
  规则3b 公司/集团/国营/农场/矿/管委会     -> 删除; 有地址段并入周边真实镇
  规则3c 含'区'/'市'                       -> 对照国家统计局2024判定真实区划名

死规则: 凡镇下有"真实居住地址段"(社区/村/居委会/路,且非'虚拟'占位) -> 一律改名保留绝不硬删(用户决策1)。
权威源: 国家统计局 2024 行政区划 reference/area_code_2024.csv.gz(用户决策2)。
桥接:  GMB province_code 是字母(AH/SD),与国家统计局无直接关系;
        唯一桥 = address_units.source_code 的官方 12 位 GB 码(省2市2县2镇3村3)。

保守原则(v2.1,经安徽试点修正):
  - 只对高精度信号动手:省名前缀 / 功能区词 / 企业词 / 官方码(同省约束)对不上;
  - 绝不剥离"地级市名+路/街/道"这种以外地命名的真实道路名(铜陵路镇/延安路镇);
  - 国家统计局官方码与社区反查一律按 GMB 省的 GB 两位前缀地域约束,杜绝跨省误配;
  - 解析不出干净名又带居民的 -> REVIEW(留人工),绝不猜。

默认 dry-run(只出审查表,不动库)。落库在审查通过后单独实现。
"""
import argparse
import csv
import gzip
import os
import re
import sqlite3
from collections import Counter, defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
DB_PATH = os.path.join(HERE, "china.sqlite")
REF_GZ = os.path.join(HERE, "reference", "area_code_2024.csv.gz")

# 功能区 token(规则3a)
FACILITY_TOKENS = [
    "高新技术产业开发区", "经济技术开发区", "循环经济示范园", "综合保税区",
    "经济开发区", "工业集中区", "产业集聚区", "高新区", "开发区", "保税区",
    "示范园", "示范区", "产业园", "工业园", "高新", "综合",
]
# 企业/事业单位 token(规则3b)
ENTERPRISE_TOKENS = [
    "有限责任公司", "有限公司", "集团", "公司", "国营", "农场", "林场", "牧场",
    "渔场", "盐场", "茶场", "园艺场", "管委会", "管理委员会", "管理区",
    "矿务局", "矿业", "煤矿", "铁矿", "基地",
]
RESIDENTIAL_HINTS = ("社区", "村", "居委会", "居民委员会", "村民委员会", "路", "街", "弄", "组")
ROAD_NEXT = ("路", "街", "道", "桥", "巷", "大")  # 地级市名后紧跟这些字 -> 是道路名,不可剥
STREET_NEXT = ("街", "路", "道", "桥", "巷")  # 功能区词后紧跟这些字 -> 其实是街/路名(高新街/高新路),不算功能区
# 功能区/企业"单位类型"后缀:权威名以这些结尾才算真功能区单位(高新街街道 不算,城区/矿区 不算)
UNIT_SUFFIX = (
    "经济技术开发区", "高新技术产业开发区", "循环经济示范园", "综合保税区", "经济开发区",
    "工业集中区", "产业集聚区", "高新区", "开发区", "保税区", "示范园", "示范区",
    "产业园", "工业园区", "工业园", "园区", "农场", "林场", "牧场", "渔场", "盐场",
    "茶场", "园艺场", "管委会", "管理委员会", "管理区", "矿务局", "矿业", "集团", "基地",
)

COMM_SUFFIX_RE = re.compile(
    r"(社区村民委员会|社区居民委员会|村民委员会|居民委员会|社区居委会|村委会|居委会|社区|村)$"
)


def comm_core(name):
    """反复剥社区/村后缀,统一 GMB(义和社区村) 与国家统计局(义和社区村民委员会) 到同一核(义和)。"""
    s = name
    while True:
        n = COMM_SUFFIX_RE.sub("", s)
        if n == s:
            return s
        s = n


def strip_suffix(name, suffixes):
    for s in suffixes:
        if name.endswith(s) and len(name) > len(s):
            return name[: -len(s)]
    return name


def load_reference(path):
    """解析国家统计局 2024。"""
    prov_full, prov_short = set(), set()
    prefecture_short = set()
    town_by_code9 = {}
    comm_to_code9 = defaultdict(Counter)

    with gzip.open(path, "rt", encoding="utf-8") as fh:
        for row in csv.reader(fh):
            if len(row) < 4:
                continue
            code, name, level = row[0], row[1], row[2]
            if level == "1":
                prov_full.add(name)
                prov_short.add(strip_suffix(name, [
                    "壮族自治区", "维吾尔自治区", "回族自治区", "自治区",
                    "特别行政区", "省", "市",
                ]))
            elif level == "2" and name not in ("市辖区", "县"):
                prefecture_short.add(strip_suffix(name, ["地区", "盟", "市"]))
            elif level == "4":
                town_by_code9[code[:9]] = name
            elif level == "5":
                core = comm_core(name)
                if core:
                    comm_to_code9[core][code[:9]] += 1

    prefecture_short = {p for p in prefecture_short if len(p) >= 2}
    return {
        "prov_full": prov_full, "prov_short": prov_short,
        "prefecture_short": prefecture_short,
        "town_by_code9": town_by_code9, "comm_to_code9": comm_to_code9,
    }


def to_zhen(name):
    """国家统计局后缀归一为 GMB 的 镇。"""
    n = re.sub(r"(街道办事处|街道|民族乡|苏木|乡)$", "镇", name)
    if not n.endswith("镇"):
        n += "镇"
    return n


def has_real_residents(units):
    """用户铁律:有地址段就保留。任何非"虚拟"占位的地址段都算真实内容,绝不硬删。"""
    return any("虚拟" not in nm for nm, _sc in units)


def resolve_authoritative(units, ref, gb2):
    """同省(gb2)约束下:官方码定位国家统计局镇名;否则社区反查(需≥2票且同省)。"""
    code9 = Counter()
    for _nm, sc in units:
        if sc and not sc.startswith("LOCAL-") and len(sc) >= 9 and sc[:9].isdigit() and sc[:2] == gb2:
            code9[sc[:9]] += 1
    if code9:
        c9 = code9.most_common(1)[0][0]
        if c9 in ref["town_by_code9"]:
            return ref["town_by_code9"][c9], f"官方码{c9}", "official"
    votes = Counter()
    for nm, _sc in units:
        core = comm_core(nm)
        if core and core in ref["comm_to_code9"]:
            for c9, n in ref["comm_to_code9"][core].items():
                if c9[:2] == gb2:
                    votes[c9] += n
    if votes:
        c9, v = votes.most_common(1)[0]
        if v >= 2 and c9 in ref["town_by_code9"]:
            return ref["town_by_code9"][c9], f"社区反查{c9}", "community"
    return None, "", ""


def strip_admin_prefix(name, prov_all, prefecture_short):
    """剥离开头 省名 + 至多一个紧邻地级市名;地级市名后紧跟 路/街/道 等(道路名)不剥。"""
    s = name
    for p in sorted(prov_all, key=len, reverse=True):
        if p and len(s) > len(p) + 1 and s.startswith(p):
            s = s[len(p):]
            break
    for pf in sorted(prefecture_short, key=len, reverse=True):
        if len(s) > len(pf) + 1 and s.startswith(pf) and s[len(pf):len(pf) + 1] not in ROAD_NEXT:
            s = s[len(pf):]
            # 吃掉地级市级别后缀:锦州市小东镇 -> (剥锦州)市小东镇 -> 小东镇
            if len(s) > 2 and s[0] in ("市", "区", "县", "盟", "州"):
                s = s[1:]
            break
    return s


def strip_facility_suffix(core):
    changed = True
    while changed:
        changed = False
        for t in sorted(FACILITY_TOKENS, key=len, reverse=True):
            if core.endswith(t) and len(core) > len(t):
                core = core[: -len(t)]
                changed = True
                break
    return core


def make_target(src_name, prov_all, prefecture_short):
    """从一个名字剥前缀+去功能区尾,得到 xx镇;剥空或剥不净(残留行政级别字)返回 ''。"""
    t = strip_admin_prefix(src_name, prov_all, prefecture_short)
    core = t[:-1] if t.endswith("镇") else t
    core = strip_facility_suffix(core)
    if not core or core[0] in ("市", "区", "县", "盟", "州"):  # 兴安盟->盟 这类剥不干净,交退化处理
        return ""
    return core + "镇"


def _has_token(name, tokens):
    """name 含某 token 且其后不是 街/路/道(排除 高新街/经济路 这类街路名)。"""
    for t in tokens:
        idx = name.find(t)
        while idx != -1:
            if name[idx + len(t): idx + len(t) + 1] not in STREET_NEXT:
                return True
            idx = name.find(t, idx + 1)
    return False


def classify(name, units, ref, prov_all, prefecture_short, gb2):
    real = has_real_residents(units)
    is_ent = _has_token(name, ENTERPRISE_TOKENS)
    is_fac = _has_token(name, FACILITY_TOKENS)
    starts_prov = any(name.startswith(p) for p in prov_all if p)
    core = name[:-1] if name.endswith("镇") else name
    has_qu_shi = ("区" in core) or ("市" in core)
    # 退化残留:镇名核心恰好是某地级市/省名(拍平地级市留下的"合肥镇/池州镇"),非道路名
    core_is_admin = (core in prefecture_short) or (core in prov_all)

    auth, basis, src = resolve_authoritative(units, ref, gb2)
    auth_zhen = to_zhen(auth) if auth else None
    # 权威名以"单位类型"后缀结尾才算真功能区/企业(高新街街道/城区/矿区 不算)
    auth_special = bool(auth and auth.endswith(UNIT_SUFFIX))

    def out(action, target="", sig="", conf="-"):
        return dict(action=action, target=target, signal=sig, confidence=conf, auth=auth or "")

    # 0. 国家统计局确认现名即为正解 -> 干净保留(杀掉"与外地地级市同名"的误报:山南/白山/唐山/海口…)
    if auth_zhen and auth_zhen == name and not auth_special:
        return out("KEEP", name, "国家统计局确认", "-")

    # 规则3c:含区/市 且 官方权威名(同省安全)与现名不一致
    s4 = bool(has_qu_shi and auth_zhen and auth_zhen != name and not auth_special)
    if not (starts_prov or is_ent or is_fac or s4 or core_is_admin):
        return out("KEEP", name, "clean", "-")

    # 1. 企业/事业名(规则3b):无居民删;带居民强行并入驻地镇删壳
    if is_ent:
        return out("DELETE", "", "企业/事业名·无居民", "HIGH") if not real \
            else out("FORCE_MERGE", "", "企业名带居民·并入驻地", "MED")

    # 2. 官方身份其实是功能区/企业(规则3a):无居民删;带居民并入驻地删壳
    if auth_special and src in ("official", "community"):
        return out("DELETE", "", f"功能区·无居民·官方[{auth}]", "HIGH") if not real \
            else out("FORCE_MERGE", "", f"功能区带居民·并入驻地·官方[{auth}]", "MED")

    # 3. 名字带功能区词(规则3a):能剥出干净真实核(龙岗)->改名;否则无居民删/带居民并入驻地
    if is_fac:
        cand = make_target(name, prov_all, prefecture_short)
        if cand and cand != name and not _facility_ish(cand):
            return out("RENAME", cand, "功能区剥离", "MED")
        return out("DELETE", "", "功能区·无居民", "HIGH") if not real \
            else out("FORCE_MERGE", "", "功能区带居民·并入驻地", "MED")

    # 4. 同省官方码给出干净真名(规则3c等)
    if src == "official" and auth_zhen and auth_zhen != name:
        return out("RENAME", to_zhen(strip_admin_prefix(auth, prov_all, prefecture_short)),
                   f"官方权威({basis})", "HIGH")

    # 5. 省名前缀 / 退化核(core=老地级市名)-> 前缀剥离
    if starts_prov or core_is_admin or s4:
        cand = make_target(name, prov_all, prefecture_short)
        if cand and cand != name and not _facility_ish(cand):
            return out("RENAME", cand, "前缀剥离", "MED")
        # 剥不出干净名(合肥镇/池州镇/淮南镇 退化老地级市城区):决策2 有居民保留,无居民删
        tip = f"·官方[{auth}]" if auth else ""
        return out("KEEP", name, "退化城区·有居民保留" + tip, "MED") if real \
            else out("DELETE", "", "退化名·无居民" + tip, "HIGH")

    return out("KEEP", name, "clean", "-")


# 功能区残核判定:剥前缀去尾后仍是"高新/经济/产业/园区"等无地理实义的词
_FACILITY_FRAG = ("高新", "经济", "产业", "工业", "开发区", "示范", "园区", "保税", "循环", "综合")


def _facility_ish(name):
    core = name[:-1] if name.endswith("镇") else name
    return (not core) or any(f in core for f in _FACILITY_FRAG)


def build_plan(con, province=None, ref_path=REF_GZ):
    """分类全部镇,解析 MERGE/FORCE_MERGE 目标码;返回 results 列表。"""
    ref = load_reference(ref_path)
    prov_rows = {r["code"]: r["name"] for r in con.execute("SELECT code,name FROM provinces")}

    gb2_map = {}
    tmp = defaultdict(Counter)
    for r in con.execute(
        "SELECT province_code, substr(source_code,1,2) g, count(*) c FROM address_units "
        "WHERE source_code NOT LIKE 'LOCAL-%' AND source_code<>'' GROUP BY province_code, g"
    ):
        if r["g"].isdigit():
            tmp[r["province_code"]][r["g"]] += r["c"]
    for pc, cnt in tmp.items():
        gb2_map[pc] = cnt.most_common(1)[0][0]

    where = "WHERE province_code=?" if province else ""
    params = (province,) if province else ()
    towns = list(con.execute(
        f"SELECT province_code,city_code,code,name,sort_order FROM towns {where} ORDER BY province_code,city_code,sort_order",
        params))

    au = defaultdict(list)
    for r in con.execute(
        f"SELECT province_code,city_code,town_code,name,source_code FROM address_units {where}", params):
        au[(r["province_code"], r["city_code"], r["town_code"])].append((r["name"], r["source_code"]))

    name_to_code = defaultdict(dict)
    for t in towns:
        name_to_code[(t["province_code"], t["city_code"])][t["name"]] = t["code"]

    results = []
    for t in towns:
        key = (t["province_code"], t["city_code"], t["code"])
        prov_name = prov_rows.get(t["province_code"], "")
        prov_short = re.sub(r"(省|市|壮族自治区|维吾尔自治区|回族自治区|自治区|特别行政区)$", "", prov_name)
        gb2 = gb2_map.get(t["province_code"], "")
        res = classify(t["name"], au.get(key, []), ref, {prov_name, prov_short}, ref["prefecture_short"], gb2)
        if res["action"] == "RENAME" and res["target"] in name_to_code[(t["province_code"], t["city_code"])] \
                and res["target"] != t["name"]:
            res["action"] = "MERGE"
        res.update(pc=t["province_code"], cc=t["city_code"], tc=t["code"], cur=t["name"],
                   sort=t["sort_order"], nseg=len(au.get(key, [])), target_code="")
        results.append(res)

    # 每市驻地镇 = 按 sort_order 第一个 KEEP 的镇
    seat = {}
    for r in results:
        if r["action"] == "KEEP":
            seat.setdefault((r["pc"], r["cc"]), (r["tc"], r["cur"]))
    for r in results:
        if r["action"] == "FORCE_MERGE":
            s = seat.get((r["pc"], r["cc"]))
            if s and s[0] != r["tc"]:
                r["target"], r["target_code"] = s[1], s[0]
            else:
                r["action"], r["signal"] = "REVIEW", r["signal"] + "·无驻地镇"
        elif r["action"] == "MERGE":
            r["target_code"] = name_to_code[(r["pc"], r["cc"])].get(r["target"], "")
            if not r["target_code"]:
                r["action"] = "RENAME"  # 目标其实不存在,降级为改名
        elif r["action"] == "RENAME":
            r["target_code"] = r["tc"]
    return results


def apply_plan(con, results, today):
    """事务内落库:RENAME/MERGE/FORCE_MERGE/DELETE + tombstone + change_log + version bump。"""
    cur = con.cursor()
    cur.execute("BEGIN")
    log = Counter()

    def names_in_town(pc, cc, tc):
        return {row[0] for row in cur.execute(
            "SELECT name FROM address_units WHERE province_code=? AND city_code=? AND town_code=?",
            (pc, cc, tc))}

    def move_units(pc, cc, src, dst):
        existing = names_in_town(pc, cc, dst)
        for aid, nm in cur.execute(
            "SELECT address_unit_id,name FROM address_units WHERE province_code=? AND city_code=? AND town_code=?",
            (pc, cc, src)).fetchall():
            new_nm = nm
            if new_nm in existing:  # 同镇名称唯一铁律:撞名加来源后缀
                k = 2
                while f"{nm}{k}" in existing:
                    k += 1
                new_nm = f"{nm}{k}"
            existing.add(new_nm)
            cur.execute(
                "UPDATE address_units SET town_code=?, name=? WHERE address_unit_id=?",
                (dst, new_nm, aid))

    def changelog(action, pc, cc, tc, old, new, reason):
        cur.execute(
            "INSERT INTO admin_division_change_log(version,action,level,province_code,city_code,town_code,old_name,new_name,reason,changed_at)"
            " VALUES(2,?,?,?,?,?,?,?,?,?)",
            (action, "town", pc, cc, tc, old, new, reason, today))

    def tombstone(pc, cc, tc, retired_name, reason):
        cur.execute(
            "INSERT OR REPLACE INTO town_tombstones(province_code,city_code,code,retired_name,retired_at,reason)"
            " VALUES(?,?,?,?,?,?)", (pc, cc, tc, retired_name, today, reason))

    for r in results:
        a, pc, cc, tc = r["action"], r["pc"], r["cc"], r["tc"]
        if a == "RENAME":
            cur.execute("UPDATE towns SET name=? WHERE province_code=? AND city_code=? AND code=?",
                        (r["target"], pc, cc, tc))
            changelog("rename", pc, cc, tc, r["cur"], r["target"], r["signal"])
        elif a in ("MERGE", "FORCE_MERGE"):
            move_units(pc, cc, tc, r["target_code"])
            cur.execute("DELETE FROM towns WHERE province_code=? AND city_code=? AND code=?", (pc, cc, tc))
            tombstone(pc, cc, tc, r["cur"], f"{a}->{r['target']}")
            changelog("merge", pc, cc, tc, r["cur"], r["target"], r["signal"])
        elif a == "DELETE":
            cur.execute("DELETE FROM address_units WHERE province_code=? AND city_code=? AND town_code=?", (pc, cc, tc))
            cur.execute("DELETE FROM towns WHERE province_code=? AND city_code=? AND code=?", (pc, cc, tc))
            tombstone(pc, cc, tc, r["cur"], r["signal"])
            changelog("delete", pc, cc, tc, r["cur"], "", r["signal"])
        else:
            continue
        log[a] += 1

    # 版本与计数
    town_count = cur.execute("SELECT COUNT(*) FROM towns").fetchone()[0]
    cur.execute("UPDATE metadata SET value=? WHERE key='admin_division_version'", ("2",))
    cur.execute("UPDATE metadata SET value=? WHERE key='admin_division_published_at'", (today,))
    cur.execute("UPDATE metadata SET value=? WHERE key='town_count'", (str(town_count),))
    cur.execute(
        "INSERT OR REPLACE INTO admin_division_versions(version,published_at,reason) VALUES(2,?,?)",
        (today, "镇名残留清理:剥省/老地级市前缀、功能区企业名删除或并入驻地镇、对照国家统计局2024"))
    con.commit()
    return log


def print_table(results, show):
    print(f"\n===== 审查表 =====")
    for r in sorted(results, key=lambda x: (x["action"], x["pc"], x["cc"], x["tc"])):
        if r["action"] in show:
            loc = f"{r['pc']}/{r['cc']}/{r['tc']}"
            authtxt = f"[{r['auth']}]" if r["auth"] else ""
            tgt = f"{r['target']}({r['target_code']})" if r.get("target_code") and r["action"] in ("MERGE", "FORCE_MERGE") else r["target"]
            print(f"{loc:13} {r['cur']:24} {r['action']:11} -> {tgt:16} 段{r['nseg']:>3} {r['confidence']:4} {r['signal']} {authtxt}")
    print("----- 统计 -----")
    c = Counter(r["action"] for r in results)
    for k, v in c.most_common():
        print(f"  {k:12} {v}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--province", default=None, help="只跑某省 GMB 字母码,如 AH")
    ap.add_argument("--db", default=DB_PATH)
    ap.add_argument("--ref", default=REF_GZ)
    ap.add_argument("--show", default="RENAME,MERGE,FORCE_MERGE,DELETE,REVIEW")
    ap.add_argument("--apply", action="store_true", help="落库(默认 dry-run)")
    ap.add_argument("--date", default="2026-06-21", help="变更日期(确定性)")
    args = ap.parse_args()

    con = sqlite3.connect(args.db)
    con.row_factory = sqlite3.Row
    results = build_plan(con, args.province, args.ref)
    print_table(results, set(args.show.split(",")))

    if args.apply:
        if args.province:
            print("\n[!] --apply 必须全国一次性执行(不带 --province),已中止。")
            return
        log = apply_plan(con, results, args.date)
        print("\n===== 已落库 =====")
        for k, v in log.most_common():
            print(f"  {k:12} {v}")
    con.close()


if __name__ == "__main__":
    main()
