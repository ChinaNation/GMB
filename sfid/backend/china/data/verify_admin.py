#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""china.sqlite 全量错误体检:同父重名 / 跨省同地(拆省残留) / 残留幽灵地级市。只读。"""
import os, sqlite3
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
c = sqlite3.connect(os.path.join(HERE, "china.sqlite"))
def fa(s, a=()): return c.execute(s, a).fetchall()

print("=== A. 同父重名(同一上级下不得重名)===")
cd = fa("SELECT province_code,name,COUNT(*) FROM cities GROUP BY province_code,name HAVING COUNT(*)>1")
td = fa("SELECT province_code,city_code,name,COUNT(*) n FROM towns GROUP BY province_code,city_code,name HAVING n>1")
vd = fa("SELECT province_code,city_code,town_code,name,COUNT(*) n FROM villages GROUP BY province_code,city_code,town_code,name HAVING n>1")
print(f"  市级同省重名: {len(cd)} 组 / 多 {sum(x[2]-1 for x in cd)} 条")
print(f"  镇级同市重名: {len(td)} 组 / 多 {sum(x[3]-1 for x in td)} 条")
print(f"  村级同镇重名: {len(vd)} 组 / 多 {sum(x[4]-1 for x in vd)} 条")

# ---- B. 跨省同地(同名+镇集高度重合 = 同一物理地,拆省没删干净)----
print("\n=== B. 跨省同地(同名市 且 ≥5个共同镇 且 ≥70%重合 = 真同地复制)===")
MIN_SHARED = 5
tset = defaultdict(set)
for pc, cc, nm in fa("SELECT province_code,city_code,name FROM towns"):
    tset[(pc, cc)].add(nm)
cities = fa("SELECT province_code,code,name FROM cities")
byname = defaultdict(list)
for pc, code, nm in cities:
    byname[nm].append((pc, code))
pairs = defaultdict(list)
prov_dupcities = defaultdict(set)
for nm, locs in byname.items():
    if len({p for p, _ in locs}) < 2:
        continue
    for i in range(len(locs)):
        for j in range(i + 1, len(locs)):
            (pa, ca), (pb, cb) = locs[i], locs[j]
            if pa == pb:
                continue
            A, B = tset[(pa, ca)], tset[(pb, cb)]
            shared = len(A & B)
            if shared >= MIN_SHARED and A and B and shared / min(len(A), len(B)) >= 0.70:
                pairs[tuple(sorted([pa, pb]))].append(nm)
                prov_dupcities[pa].add(nm); prov_dupcities[pb].add(nm)
tot = sum(len(v) for v in pairs.values())
print(f"  真同地市对: {tot} 对,分布:")
for (pa, pb), items in sorted(pairs.items(), key=lambda kv: -len(kv[1])):
    print(f"    {pa}<->{pb}: {len(items)}  例:{','.join(items[:10])}")
print("\n  按省:各省有多少市在别省有同地双胞胎(=该省被复制/复制别省):")
pn = {p: n for p, n in fa("SELECT code,name FROM provinces")}
for p, s in sorted(prov_dupcities.items(), key=lambda kv: -len(kv[1])):
    print(f"    {pn.get(p,p)}({p}): {len(s)}")

# ---- C. 残留幽灵地级市(完整包含≥3个其它市的≥80%镇)----
print("\n=== C. 残留幽灵地级市(包含≥3个同省其它市)===")
rows = fa("""
  WITH pair AS (SELECT t1.province_code p,t1.city_code X,t2.city_code Y,COUNT(*) sh
    FROM towns t1 JOIN towns t2 ON t1.province_code=t2.province_code AND t1.name=t2.name AND t1.city_code<>t2.city_code
    GROUP BY t1.province_code,t1.city_code,t2.city_code),
  ys AS (SELECT province_code p,city_code Y,COUNT(*) yt FROM towns GROUP BY province_code,city_code),
  xs AS (SELECT province_code p,city_code X,COUNT(*) xt FROM towns GROUP BY province_code,city_code)
  SELECT pair.p,pair.X,xs.xt,COUNT(*) ch FROM pair JOIN ys ON ys.p=pair.p AND ys.Y=pair.Y
    JOIN xs ON xs.p=pair.p AND xs.X=pair.X
  WHERE pair.sh*1.0/ys.yt>=0.8 GROUP BY pair.p,pair.X HAVING ch>=3 ORDER BY ch DESC""")
print(f"  共 {len(rows)} 个:")
for p, x, xt, ch in rows:
    nm = c.execute("SELECT name FROM cities WHERE province_code=? AND code=?", (p, x)).fetchone()[0]
    print(f"    {p} {nm}({xt}镇,含{ch}个子市)")

# ---- D. 省份总览 ----
print("\n=== D. 43省市数 ===")
pv = fa("SELECT p.code,p.name,(SELECT COUNT(*) FROM cities WHERE province_code=p.code) FROM provinces p ORDER BY p.sort_order")
print("  " + " ".join(f"{name}{n}" for _, name, n in pv))
c.close()
