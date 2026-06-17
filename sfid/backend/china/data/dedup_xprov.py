#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""跨省同地复制清理:每组同名市保留"真身省"(共享镇归属地),非真身省删掉抄来的镇;
非真身省删完若空=纯复制整市删,若还有自己独有镇=保留(真不同地)。
真身省由共享镇地名判定(distinctive 镇名锁定真实城市)。--apply 落库。"""
import argparse, os, sqlite3
from collections import defaultdict
HERE = os.path.dirname(os.path.abspath(__file__))

# 组名 -> 真身省 code(共享镇真实归属);置信度: H高 / M中(已注)
OWNER = {
 "东城市":"BP","东安市":"HJ","东山市":"LJ","东港市":"LI","云龙市":"JS","五华市":"GD","兴宁市":"GD",
 "兴安市":"LJ","华容市":"HI","南丰市":"JX","南城市":"JX","南山市":"LJ","台江市":"FJ","向阳市":"HJ",
 "和平市":"HA","城中市":"GX","城关市":"XZ","大安市":"JL","大通市":"AH","太和市":"LI","安宁市":"YN",
 "宝山市":"HJ","山阳市":"HE","市中市":"SD","常山市":"ZJ","平山市":"HB","开平市":"HA","新兴市":"HJ",
 "新华市":"HB","新城市":"HT","昌江市":"JX","昌邑市":"JL","普陀市":"SJ","朝阳市":"BP","栖霞市":"JS",
 "梨树市":"JL","永定市":"FJ","江北市":"ZJ","江城市":"GD","沙湾市":"SC","河东市":"HA","河口市":"SD",
 "海城市":"LI","海州市":"LI","清河市":"HB","港口市":"GX","白云市":"GD","襄城市":"HE","西安市":"JL",
 "西湖市":"ZJ","象山市":"ZJ","连山市":"LI","通州市":"BP","郊市":"SX","金川市":"SC","金平市":"GD",
 "钟山市":"GX","铁东市":"LI","铁西市":"LI","长宁市":"SJ","长安市":"HB","青山市":"HT","鹤山市":"HE",
 "鼓楼市":"JS","龙华市":"GD","龙山市":"JL","龙港市":"ZJ",
}
# 双湖:用户指定移到昆仑省(KL)

def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--db", default=os.path.join(HERE, "china.sqlite"))
    ap.add_argument("--apply", action="store_true"); a = ap.parse_args()
    conn = sqlite3.connect(a.db); cur = conn.cursor()
    fa = lambda s, x=(): cur.execute(s, x).fetchall()
    pn = {p: n for p, n in fa("SELECT code,name FROM provinces")}
    if a.apply: cur.execute("BEGIN")
    rows = []
    for nm, owner_pc in OWNER.items():
        insts = fa("SELECT province_code,code FROM cities WHERE name=?", (nm,))
        provs = {pc for pc, _ in insts}
        if owner_pc not in provs:
            rows.append((nm, f"⚠真身省{pn.get(owner_pc,owner_pc)}不在组内{[(pn[p]) for p in provs]} 跳过")); continue
        owner_inst = [(pc, cd) for pc, cd in insts if pc == owner_pc][0]
        owner_towns = set(t[0] for t in fa("SELECT name FROM towns WHERE province_code=? AND city_code=?", owner_inst))
        keepers = [pn.get(owner_pc, owner_pc)]; del_cities = []; kept_own = []
        for pc, cd in insts:
            if (pc, cd) == owner_inst: continue
            my = fa("SELECT code,name FROM towns WHERE province_code=? AND city_code=?", (pc, cd))
            copied = [c for c, n2 in my if n2 in owner_towns]
            own = [n2 for c, n2 in my if n2 not in owner_towns]
            if a.apply:
                for c in copied:
                    cur.execute("DELETE FROM villages WHERE province_code=? AND city_code=? AND town_code=?", (pc, cd, c))
                    cur.execute("DELETE FROM towns WHERE province_code=? AND city_code=? AND code=?", (pc, cd, c))
            if not own:  # 纯复制 -> 整市删
                if a.apply:
                    cur.execute("DELETE FROM villages WHERE province_code=? AND city_code=?", (pc, cd))
                    cur.execute("DELETE FROM cities WHERE province_code=? AND code=?", (pc, cd))
                del_cities.append(pn.get(pc, pc))
            else:        # 真不同地,删抄来的镇,保留自己的
                kept_own.append(f"{pn.get(pc,pc)}(留{len(own)}自有)")
        rows.append((nm, f"真身={keepers[0]} | 删复制市:{del_cities or '无'} | 真不同地保留:{kept_own or '无'}"))
    # 双湖 -> 昆仑
    sh = fa("SELECT province_code,code FROM cities WHERE name='双湖市'")
    if sh:
        has_kl = any(pc == "KL" for pc, _ in sh)
        if a.apply and not has_kl:
            # 仅阿里有 -> 改省到昆仑(code 取昆仑 max+1)
            al = [(pc, cd) for pc, cd in sh if pc == "AL"]
            if al:
                pc, cd = al[0]
                nc = "%03d" % ((fa("SELECT MAX(CAST(code AS INTEGER)) FROM cities WHERE province_code='KL'")[0][0] or 0) + 1)
                so = (fa("SELECT MAX(sort_order) FROM cities WHERE province_code='KL'")[0][0] or 0) + 1
                cur.execute("UPDATE cities SET province_code='KL',code=?,sort_order=? WHERE province_code=? AND code=?", (nc, so, pc, cd))
                cur.execute("UPDATE towns SET province_code='KL',city_code=? WHERE province_code=? AND city_code=?", (nc, pc, cd))
                cur.execute("UPDATE villages SET province_code='KL',city_code=? WHERE province_code=? AND city_code=?", (nc, pc, cd))
        rows.append(("双湖市", "移动阿里->昆仑省"))
    if a.apply:
        cur.execute("REINDEX towns"); conn.commit()
    for nm, r in rows:
        print(f"  {nm}: {r}")
    print(f"\n共处理 {len(rows)} 组")
    conn.close()

if __name__ == "__main__":
    main()
