#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
行政区去重清理迁移(ADR-021 续作 · 任务卡 20260616-admin-district-dedup-cleanup)。

默认 DRY-RUN。--apply 执行(调用方须先备份 china.sqlite)。

策略:
  local   : #6 幽灵地级市 / #1#5 纯重复功能区 —— 按 code 顺序用兄弟区市镇数分块,
            独有镇归入所在块(块边界用镇数精确切);开发区/尾部镇标 ambiguous(先落+记审查)。
            然后删本市重复镇+城本身,独有镇已迁出。
  twin:XX : #4(BP)/#7西康(XK)/#7热河(RH) —— 整市删,但每镇须在孪生省验到同名孪生才删(防误删/丢)。
  #2      : 攀枝花(东029->攀枝花市+并西030,仁和031留)/南县(HU075南市->南县市)/北镇(LI058北市->北镇市);
            其余方位市=复制,验孪生(对应规范市)后删;中市 TW093 暂留(未识别)。
  #5 改名 : 浦东新市->浦东市(真行政区)。
铁律:code 不可变不复用(改名只改 name);删市留 code 空档不顺移;独有镇不丢,归户。
"""
import argparse, os, shutil, sqlite3, sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
TWIN_MIN = 0.80
ZONE = ("开发区", "经济技术", "工业园", "产业园", "高新区", "保税", "新区管")
PUREDUP_ZONE = {"高新市", "兰州新市", "沈北新市", "工业园市"}
# #2 规范市处理
PANZHIHUA = ("SC", "029", "030", "攀枝花市")  # 东市029->攀枝花市, 并西市030
CANON_RENAME = {("HU", "075"): "南县市", ("LI", "058"): "北镇市"}  # 南市/北市 真县
PUDONG = ("SJ", "浦东新市", "浦东市")
DIRECTIONAL = ("东市", "西市", "南市", "北市", "中市")
HOLD = {("TW", "093")}  # 中市 暂不处理


def fa(c, sql, a=()): return c.execute(sql, a).fetchall()
def one(c, sql, a=()):
    r = c.execute(sql, a).fetchone(); return r[0] if r else None
def cname(c, pc, code): return one(c, "SELECT name FROM cities WHERE province_code=? AND code=?", (pc, code))
def tnames(c, pc, code): return [n for (n,) in fa(c, "SELECT name FROM towns WHERE province_code=? AND city_code=? ORDER BY code", (pc, code))]
def prov_tnames(c, pc): return {n for (n,) in fa(c, "SELECT DISTINCT name FROM towns WHERE province_code=?", (pc,))}


def name_to_sib(c, pc, code):
    m = defaultdict(set)
    for nm, cc in fa(c, "SELECT name, city_code FROM towns WHERE province_code=? AND city_code!=?", (pc, code)):
        m[nm].add(cc)
    return m


def detect_phantoms(c):
    rows = fa(c, """
        WITH pair AS (SELECT t1.province_code p,t1.city_code X,t2.city_code Y,COUNT(*) shared
          FROM towns t1 JOIN towns t2 ON t1.province_code=t2.province_code AND t1.name=t2.name AND t1.city_code<>t2.city_code
          GROUP BY t1.province_code,t1.city_code,t2.city_code),
        ys AS (SELECT province_code p,city_code Y,COUNT(*) yt FROM towns GROUP BY province_code,city_code),
        xs AS (SELECT province_code p,city_code X,COUNT(*) xt FROM towns GROUP BY province_code,city_code)
        SELECT pair.p,pair.X,COUNT(*) ch FROM pair JOIN ys ON ys.p=pair.p AND ys.Y=pair.Y
          JOIN xs ON xs.p=pair.p AND xs.X=pair.X
        WHERE pair.shared*1.0/ys.yt>=0.8 AND xs.xt>=30 GROUP BY pair.p,pair.X HAVING ch>=3""")
    return [(p, x) for p, x, _ in rows]


def plan_local(c, pc, code):
    """返回 (dup_towns[name], rehome[(name,target)], ambiguous[(name,guess,reason)], unassigned[name])。"""
    towns = fa(c, "SELECT name FROM towns WHERE province_code=? AND city_code=? ORDER BY code", (pc, code))
    sib = name_to_sib(c, pc, code)
    sibsize = dict(fa(c, "SELECT city_code,COUNT(*) FROM towns WHERE province_code=? AND city_code!=? GROUP BY city_code", (pc, code)))
    lab = [(n, sib.get(n)) for (n,) in towns]
    dup = [n for n, s in lab if s]
    rehome, ambiguous, unassigned = [], [], []
    cur, remain, n = None, 0, len(lab)
    for i, (nm, sibs) in enumerate(lab):
        if sibs:
            if cur in sibs and remain > 0:
                chosen = cur
            else:
                chosen = next((s for s in sorted(sibs) if sibsize.get(s, 0) > 0), sorted(sibs)[0])
                cur, remain = chosen, sibsize.get(chosen, 0)
            remain -= 1
        else:
            if cur is not None and remain > 0:
                guess, ok = cur, True
            else:
                nxt = next((lab[j][1] for j in range(i + 1, n) if lab[j][1]), None)
                guess, ok = (sorted(nxt)[0] if nxt else cur), bool(nxt)
            if guess is None:
                unassigned.append(nm)
            elif any(z in nm for z in ZONE):
                ambiguous.append((nm, guess, "开发区镇"))
            elif not ok:
                ambiguous.append((nm, guess, "尾部无后继锚点"))
            else:
                rehome.append((nm, guess))
    return dup, rehome, ambiguous, unassigned


def twin_check(c, pc, code, twin_names):
    tns = tnames(c, pc, code)
    missing = [t for t in tns if t not in twin_names]
    return len(tns) - len(missing), len(tns), missing


def build_jobs(c):
    jobs = []  # dict(pc,code,name,cat,policy)
    for pc, code, nm in fa(c, "SELECT province_code,code,name FROM cities WHERE name='市辖市'"):
        jobs.append(dict(pc=pc, code=code, name=nm, cat="#4甘肃市辖市", pol=("twin", "BP")))
    xk = {n for (n,) in fa(c, "SELECT name FROM cities WHERE province_code='XK'")}
    if xk:
        for pc, code, nm in fa(c, "SELECT province_code,code,name FROM cities WHERE province_code IN('SC','XZ','YN') AND name IN (%s)" % ",".join("?"*len(xk)), tuple(xk)):
            jobs.append(dict(pc=pc, code=code, name=nm, cat="#7西康原省删", pol=("twin", "XK")))
    rh = {n for (n,) in fa(c, "SELECT name FROM cities WHERE province_code='RH'")}
    if rh:
        for pc, code, nm in fa(c, "SELECT province_code,code,name FROM cities WHERE province_code IN('HB','LN','XA','JL') AND name IN (%s)" % ",".join("?"*len(rh)), tuple(rh)):
            jobs.append(dict(pc=pc, code=code, name=nm, cat="#7热河原省删", pol=("twin", "RH")))
    for nm in PUREDUP_ZONE:
        for pc, code in fa(c, "SELECT province_code,code FROM cities WHERE name=?", (nm,)):
            jobs.append(dict(pc=pc, code=code, name=nm, cat="#1/#5纯重复功能区", pol="local"))
    for pc, code in detect_phantoms(c):
        jobs.append(dict(pc=pc, code=code, name=cname(c, pc, code), cat="#6幽灵地级市", pol="local"))
    seen, uniq = set(), []
    for j in jobs:
        k = (j["pc"], j["code"])
        if k in seen: continue
        seen.add(k); uniq.append(j)
    return uniq


# ---------------- APPLY ----------------
def next_town_code(cur, pc, city):
    mx = one(cur, "SELECT MAX(CAST(code AS INTEGER)) FROM towns WHERE province_code=? AND city_code=?", (pc, city)) or 0
    return "%03d" % (mx + 1)


def next_sort(cur, pc, city):
    return (one(cur, "SELECT MAX(sort_order) FROM towns WHERE province_code=? AND city_code=?", (pc, city)) or 0) + 1


def rehome_town(cur, pc, frm, town_name, to):
    row = cur.execute("SELECT code FROM towns WHERE province_code=? AND city_code=? AND name=? LIMIT 1", (pc, frm, town_name)).fetchone()
    if not row: return
    old = row[0]
    nc = next_town_code(cur, pc, to)
    cur.execute("INSERT INTO towns(province_code,city_code,code,name,sort_order) VALUES(?,?,?,?,?)",
                (pc, to, nc, town_name, next_sort(cur, pc, to)))
    cur.execute("UPDATE villages SET city_code=?, town_code=? WHERE province_code=? AND city_code=? AND town_code=?",
                (to, nc, pc, frm, old))


def delete_city(cur, pc, code):
    cur.execute("DELETE FROM villages WHERE province_code=? AND city_code=?", (pc, code))
    cur.execute("DELETE FROM towns WHERE province_code=? AND city_code=?", (pc, code))
    cur.execute("DELETE FROM cities WHERE province_code=? AND code=?", (pc, code))


def do_apply(conn, review_path, skip_phantoms=False):
    cur = conn.cursor()
    cur.execute("BEGIN")
    review = []
    twincache = {}
    # #2 先做 规范市 改名/合并(让规范市先存在,供复制市验孪生)
    spc, ekeep, ewest, ename = PANZHIHUA
    cur.execute("UPDATE cities SET name=? WHERE province_code=? AND code=?", (ename, spc, ekeep))
    for tn in tnames(cur, spc, ewest):
        rehome_town(cur, spc, ewest, tn, ekeep)
    delete_city(cur, spc, ewest)
    for (pc, code), newname in CANON_RENAME.items():
        cur.execute("UPDATE cities SET name=? WHERE province_code=? AND code=?", (newname, pc, code))
    rpc, roldname, rnewname = PUDONG
    cur.execute("UPDATE cities SET name=? WHERE province_code=? AND name=?", (rnewname, rpc, roldname))
    # #2 复制方位市删除(验孪生:对应规范市)
    canon_sets = {ename: set(tnames(cur, spc, ekeep)),
                  "南县市": set(tnames(cur, "HU", "075")), "北镇市": set(tnames(cur, "LI", "058"))}
    for pc, code, nm in fa(cur, "SELECT province_code,code,name FROM cities WHERE name IN %s" % str(DIRECTIONAL)):
        if (pc, code) in HOLD: continue
        my = tnames(cur, pc, code)
        if not my:
            delete_city(cur, pc, code); continue
        best = max(canon_sets.items(), key=lambda kv: sum(1 for t in my if t in kv[1]))
        ratio = sum(1 for t in my if t in best[1]) / len(my)
        if ratio >= TWIN_MIN:
            delete_city(cur, pc, code)
        else:
            review.append(f"HOLD 方位市 {pc}{code} {nm} 未匹配规范市(best={best[0]} {ratio:.0%})")
    # 主体 jobs
    for j in build_jobs(cur):
        pc, code, pol = j["pc"], j["code"], j["pol"]
        if skip_phantoms and j["cat"] == "#6幽灵地级市":
            continue
        if pol == "local":
            dup, rehome, ambiguous, unassigned = plan_local(cur, pc, code)
            for nm, tgt in rehome:
                rehome_town(cur, pc, code, nm, tgt)
            for nm, guess, reason in ambiguous:
                rehome_town(cur, pc, code, nm, guess)
                review.append(f"AMBIG 归户 {pc}{code}{j['name']} {nm}->{cname(cur,pc,guess)} ({reason})")
            for nm in unassigned:
                review.append(f"UNASSIGNED {pc}{code}{j['name']} 独有镇 {nm} 未归户,随城删")
            delete_city(cur, pc, code)
        else:
            tw = pol[1]
            if tw not in twincache: twincache[tw] = prov_tnames(cur, tw)
            match, total, missing = twin_check(cur, pc, code, twincache[tw])
            if total and match / total >= TWIN_MIN and not missing:
                delete_city(cur, pc, code)
            else:
                review.append(f"SKIP twin {pc}{code}{j['name']} {match}/{total} 无孪生:{missing[:5]}")
    cur.execute("REINDEX towns")
    conn.commit()
    with open(review_path, "w", encoding="utf-8") as f:
        f.write("\n".join(review) + "\n")
    return len(review)


# ---------------- PHASE 2: 台湾大陆复制 + 幽灵地级市 ----------------
def do_phase2(conn, review_path):
    cur = conn.cursor()
    cur.execute("BEGIN")
    review = []
    # 各市镇集
    tset = defaultdict(set)
    for pc, cc, nm in fa(cur, "SELECT province_code,city_code,name FROM towns"):
        tset[(pc, cc)].add(nm)
    prov_towns = defaultdict(set)
    for (pc, cc), s in tset.items():
        prov_towns[pc] |= s
    # 1) 台湾里"大陆复制市":TW 市的全部镇都能在某个非TW省找到(纯复制),且与某非TW市≥5共同镇
    tw_deleted = []
    tw_cities = fa(cur, "SELECT code,name FROM cities WHERE province_code='TW'")
    cityrows = fa(cur, "SELECT province_code,code,name FROM cities WHERE province_code!='TW'")
    byname = defaultdict(list)
    for pc, code, nm in cityrows:
        byname[nm].append((pc, code))
    for code, nm in tw_cities:
        my = tset[("TW", code)]
        if not my:
            continue
        # 必须有同名大陆市且≥5共同镇,且本市全部镇都在该大陆省(无台湾独有镇)
        ok = False
        for pc2, code2 in byname.get(nm, []):
            shared = len(my & tset[(pc2, code2)])
            if shared >= 5 and shared == len(my):
                ok = True; break
        if ok:
            delete_city(cur, "TW", code); tw_deleted.append(nm)
        elif byname.get(nm) and any(len(my & tset[(p2, c2)]) >= 5 for p2, c2 in byname[nm]):
            review.append(f"TW-HOLD {code} {nm} 有大陆同地但含台湾独有镇,未删")
    # 2) 幽灵地级市:重新检测当前库,独有镇归户+删
    for pc, code in detect_phantoms(cur):
        nm = cname(cur, pc, code)
        dup, rehome, ambiguous, unassigned = plan_local(cur, pc, code)
        for tn, tgt in rehome:
            rehome_town(cur, pc, code, tn, tgt)
        for tn, guess, reason in ambiguous:
            rehome_town(cur, pc, code, tn, guess)
            review.append(f"AMBIG {pc}{code}{nm} {tn}->{cname(cur,pc,guess)} ({reason})")
        for tn in unassigned:
            review.append(f"UNASSIGNED {pc}{code}{nm} 独有镇 {tn} 随城删(无归户)")
        delete_city(cur, pc, code)
    cur.execute("REINDEX towns")
    conn.commit()
    with open(review_path, "w", encoding="utf-8") as f:
        f.write(f"# 台湾删除 {len(tw_deleted)} 个大陆复制市: {','.join(tw_deleted)}\n")
        f.write("\n".join(review) + "\n")
    return len(tw_deleted), len(review)


# ---------------- PHASE 3: 同父重名去重(镇合并/村去重/台湾重复块)----------------
def do_phase3(conn, review_path):
    cur = conn.cursor()
    cur.execute("BEGIN")
    rv = []
    tn_set = lambda pc, cc: set(x[0] for x in fa(cur, "SELECT name FROM towns WHERE province_code=? AND city_code=?", (pc, cc)))
    # 1) 台湾(及任意省)同省同名市:仅当镇集完全相同(真重复块)才删多余;镇集不同=真不同地,留待改名
    citydup = fa(cur, "SELECT province_code,name FROM cities GROUP BY province_code,name HAVING COUNT(*)>1")
    tw_del = 0
    for pc, nm in citydup:
        insts = [r[0] for r in fa(cur, "SELECT code FROM cities WHERE province_code=? AND name=? ORDER BY code", (pc, nm))]
        keep = insts[0]; ks = tn_set(pc, keep)
        for other in insts[1:]:
            if tn_set(pc, other) == ks and ks:
                delete_city(cur, pc, other); tw_del += 1
                rv.append(f"CITY-DUP删 {pc}{other} {nm}(镇集与{keep}相同)")
            else:
                rv.append(f"CITY-DUP留 {pc}{other} {nm}(镇集不同=真不同地,待改名)")
    # 2) 同市同名镇:合并(villages 迁入保留村数最多的那个,重名村跳过=去重),删多余镇
    tdup = fa(cur, "SELECT province_code,city_code,name FROM towns GROUP BY province_code,city_code,name HAVING COUNT(*)>1")
    merged = 0
    for pc, cc, nm in tdup:
        codes = [r[0] for r in fa(cur, "SELECT code FROM towns WHERE province_code=? AND city_code=? AND name=?", (pc, cc, nm))]
        codes.sort(key=lambda tc: -fa(cur, "SELECT COUNT(*) FROM villages WHERE province_code=? AND city_code=? AND town_code=?", (pc, cc, tc))[0][0])
        keep = codes[0]
        keepv = set(x[0] for x in fa(cur, "SELECT name FROM villages WHERE province_code=? AND city_code=? AND town_code=?", (pc, cc, keep)))
        keepmax = fa(cur, "SELECT MAX(CAST(code AS INTEGER)) FROM villages WHERE province_code=? AND city_code=? AND town_code=?", (pc, cc, keep))[0][0] or 0
        for other in codes[1:]:
            for vcode, vname in fa(cur, "SELECT code,name FROM villages WHERE province_code=? AND city_code=? AND town_code=? ORDER BY code", (pc, cc, other)):
                if vname in keepv:  # 重名村 -> 去重删
                    cur.execute("DELETE FROM villages WHERE province_code=? AND city_code=? AND town_code=? AND code=?", (pc, cc, other, vcode))
                else:               # 迁入 keep,给新村码避免撞码
                    keepmax += 1
                    cur.execute("UPDATE villages SET town_code=?, code=? WHERE province_code=? AND city_code=? AND town_code=? AND code=?", (keep, "%03d" % keepmax, pc, cc, other, vcode))
                    keepv.add(vname)
            cur.execute("DELETE FROM towns WHERE province_code=? AND city_code=? AND code=?", (pc, cc, other))
            merged += 1
    # 3) 同镇同名村:留一删多余
    vdup = fa(cur, "SELECT province_code,city_code,town_code,name FROM villages GROUP BY province_code,city_code,town_code,name HAVING COUNT(*)>1")
    vdel = 0
    for pc, cc, tc, nm in vdup:
        codes = [r[0] for r in fa(cur, "SELECT code FROM villages WHERE province_code=? AND city_code=? AND town_code=? AND name=? ORDER BY code", (pc, cc, tc, nm))]
        for other in codes[1:]:
            cur.execute("DELETE FROM villages WHERE province_code=? AND city_code=? AND town_code=? AND code=?", (pc, cc, tc, other)); vdel += 1
    cur.execute("REINDEX towns"); cur.execute("REINDEX villages")
    conn.commit()
    with open(review_path, "w", encoding="utf-8") as f:
        f.write("\n".join(rv) + "\n")
    return tw_del, merged, vdel


# ---------------- DRY RUN REPORT ----------------
def dry_run(conn):
    print(f"=== DRY-RUN (db={os.path.basename(args_db)}) ===\n")
    by = defaultdict(lambda: [0, 0, 0, 0, 0, 0])
    skip, unas, ambi, samp, tw = [], [], [], [], {}
    tvil = 0
    for j in build_jobs(conn):
        pc, code, cat, pol = j["pc"], j["code"], j["cat"], j["pol"]
        if pol == "local":
            dup, rh, am, un = plan_local(conn, pc, code)
            by[cat][0]+=1; by[cat][1]+=len(dup); by[cat][2]+=len(rh); by[cat][3]+=len(am); by[cat][4]+=len(un)
            tvil += one(conn, "SELECT COUNT(*) FROM villages WHERE province_code=? AND city_code=?", (pc, code))
            if un: unas.append((pc, j["name"], un))
            if am: ambi.append((pc, j["name"], am))
            if rh and len(samp) < 6: samp.append(f"{pc} {j['name']}: {rh[0][0]}->{cname(conn,pc,rh[0][1])} (+{len(rh)-1})")
        else:
            t = pol[1]
            if t not in tw: tw[t] = prov_tnames(conn, t)
            m, tot, miss = twin_check(conn, pc, code, tw[t])
            if tot and m/tot >= TWIN_MIN and not miss:
                by[cat][0]+=1; by[cat][1]+=tot
                tvil += one(conn, "SELECT COUNT(*) FROM villages WHERE province_code=? AND city_code=?", (pc, code))
            else:
                by[cat][5]+=1; skip.append((pc, j["name"], cat, f"{m}/{tot}", miss[:5]))
    print("【按类】          删市|删重复镇|归户|待核|无法归户|twin跳过")
    for cat,(a,b,d,e,f,g) in sorted(by.items()):
        print(f"  {cat:<18}{a:>4}|{b:>7}|{d:>5}|{e:>4}|{f:>5}|{g:>4}")
    print(f"  连带删村 ~{tvil}")
    for s in samp: print("  归户样本:", s)
    if ambi: print(f"  ⚠️待核 {sum(len(a) for _,_,a in ambi)}镇/{len(ambi)}市")
    if unas: print(f"  ⚠️无法归户: {[(p,n,len(t)) for p,n,t in unas]}")
    if skip: print(f"  ⚠️twin跳过 {len(skip)}: {skip[:5]}")


def main():
    global args_db
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", default=os.path.join(HERE, "china.sqlite"))
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--skip-phantoms", action="store_true", help="跳过 #6 幽灵地级市(归户未定时用)")
    ap.add_argument("--phase2", action="store_true", help="台湾大陆复制删除 + 幽灵地级市归户删除")
    ap.add_argument("--phase3", action="store_true", help="同父重名去重:镇合并/村去重/台湾完全相同重复块删")
    a = ap.parse_args()
    args_db = a.db
    conn = sqlite3.connect(a.db)
    if a.phase3:
        rp = os.path.join(os.path.dirname(a.db), "dedup_review_phase3.txt")
        td, mg, vd = do_phase3(conn, rp)
        print(f"PHASE3 APPLIED. 台湾重复块删 {td} 市,镇合并 {mg},村去重 {vd} -> {rp}")
    elif a.phase2:
        rp = os.path.join(os.path.dirname(a.db), "dedup_review_phase2.txt")
        tw, n = do_phase2(conn, rp)
        print(f"PHASE2 APPLIED. 台湾删 {tw} 市,审查项 {n} 条 -> {rp}")
    elif a.apply:
        rp = os.path.join(os.path.dirname(a.db), "dedup_review.txt")
        n = do_apply(conn, rp, skip_phantoms=a.skip_phantoms)
        print(f"APPLIED (skip_phantoms={a.skip_phantoms}). 审查项 {n} 条 -> {rp}")
    else:
        dry_run(conn)
    conn.close()


if __name__ == "__main__":
    main()
