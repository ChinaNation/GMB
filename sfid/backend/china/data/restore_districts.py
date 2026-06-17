#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""第1步:恢复 12 个被误删的真实区/县,带联网核实的真实镇/街道(统一 镇 后缀)。
兰州城关/安宁的真街道误挂在七里河/红古,迁回(保留村);其余按真实数据新建镇。
code 不复用:新市取该省 max+1,镇 001..。--apply 落库。"""
import argparse, os, sqlite3
HERE = os.path.dirname(os.path.abspath(__file__))

# (省code, 市名, 误挂来源市名 or None, [镇/街道词干(自动加镇)])
RESTORE = [
 ("CQ","江北市",None,["华新街","观音桥","石马河","大石坝","江北城","五里店","寸滩","铁山坪","郭家沱","鱼嘴","复盛"]),
 ("GS","城关市","七里河市",["酒泉路","张掖路","雁南","临夏路","雁北","五泉","白银路","皋兰路","广武门","伏龙坪","靖远路","草场街","火车站","拱星墩","东岗","团结新村","东岗西路","铁路东村","铁路西村","渭源路","盐场路","嘉峪关路","焦家湾","青白石","雁园","高新区"]),
 ("GS","安宁市","红古市",["培黎","安宁西路","沙井驿","十里店","孔家崖","银滩路","刘家堡","安宁堡","忠和","九合"]),
 ("YN","五华市",None,["华山","护国","大观","龙翔","丰宁","莲华","红云","黑林铺","普吉","西翥"]),
 ("GD","鹤山市",None,["沙坪","龙口","雅瑶","古劳","桃源","鹤城","共和","址山","宅梧","双合"]),
 ("HI","襄城市",None,["真武山","古城","庞公","檀溪","隆中","余家湖","欧庙","卧龙","尹集"]),
 ("HU","华容市",None,["三封寺","治河渡","北景港","鲇鱼须","万庾","插旗","注滋口","操军","东山","梅田湖","章华","胜峰","新河","团洲"]),
 ("JS","海州市",None,["海州","幸福路","朐阳","洪门","云台","新浦","浦西","新东","新南","路南","新海","花果山","南城","宁海","郁洲","新坝","锦屏","板浦","浦南"]),
 ("TS","沙湾市",None,["书香","桃园","团结","四道河子","老沙湾","乌兰乌苏","安集海","东湾","西戈壁","柳毛湾","金沟河","商户地","大泉","博尔通古"]),
 ("HX","金川市",None,["滨河路","桂林路","北京路","金川路","新华路","广州路","宁远堡","双湾"]),
 ("GZ","钟山市",None,["黄土坡","红岩","荷泉","荷城","杨柳","凤凰","德坞","月照","双戛","大河","汪家寨","大湾","木果","保华","青林","南开","金盆"]),
 ("SI","长安市",None,["韦曲","郭杜","马王","滦镇","子午","太乙宫","引镇","斗门","王寺","东大","王曲","杜曲","鸣犊","细柳","黄良"]),
]


def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--db", default=os.path.join(HERE, "china.sqlite"))
    ap.add_argument("--apply", action="store_true"); a = ap.parse_args()
    conn = sqlite3.connect(a.db); cur = conn.cursor()
    fa = lambda s, x=(): cur.execute(s, x).fetchall()
    one = lambda s, x=(): (fa(s, x)[0][0] if fa(s, x) else None)
    if a.apply: cur.execute("BEGIN")
    for prov, cityname, mishome, stems in RESTORE:
        if one("SELECT COUNT(*) FROM cities WHERE province_code=? AND name=?", (prov, cityname)):
            print(f"  ⚠ {prov} 已存在 {cityname},跳过(避免同省重名)"); continue
        new_code = "%03d" % ((one("SELECT MAX(CAST(code AS INTEGER)) FROM cities WHERE province_code=?", (prov,)) or 0) + 1)
        new_so = (one("SELECT MAX(sort_order) FROM cities WHERE province_code=?", (prov,)) or 0) + 1
        if a.apply:
            cur.execute("INSERT INTO cities(province_code,code,name,sort_order) VALUES(?,?,?,?)", (prov, new_code, cityname, new_so))
        # 误挂来源市的镇名集合(迁回用)
        src_code = one("SELECT code FROM cities WHERE province_code=? AND name=?", (prov, mishome)) if mishome else None
        moved = added = 0
        for i, stem in enumerate(stems, 1):
            tname = stem + "镇"
            tcode = "%03d" % i
            src_town = None
            if src_code:
                r = fa("SELECT code FROM towns WHERE province_code=? AND city_code=? AND name=?", (prov, src_code, tname))
                if r: src_town = r[0][0]
            if a.apply:
                if src_town:  # 迁回(连村)
                    cur.execute("UPDATE villages SET city_code=?, town_code=? WHERE province_code=? AND city_code=? AND town_code=?", (new_code, tcode, prov, src_code, src_town))
                    cur.execute("UPDATE towns SET city_code=?, code=? WHERE province_code=? AND city_code=? AND code=?", (new_code, tcode, prov, src_code, src_town))
                    moved += 1
                else:        # 新建
                    cur.execute("INSERT INTO towns(province_code,city_code,code,name,sort_order) VALUES(?,?,?,?,?)", (prov, new_code, tcode, tname, i))
                    added += 1
            else:
                (moved := moved + 1) if src_town else (added := added + 1)
        print(f"  {prov} {cityname}(code={new_code}): 迁回{moved} 新建{added} 共{len(stems)}镇" + (f" <-误挂在{mishome}" if mishome else ""))
    if a.apply:
        cur.execute("REINDEX towns"); conn.commit()
    conn.close()


if __name__ == "__main__":
    main()
