#!/usr/bin/env python3
"""公民宪法 HTML → 立法院模块 SCALE 字节(ADR-027 宪法迁移,一次性迁移工具)。

【状态】迁移已完成。产物 `constitution.scale` 已入库并随创世注入立法院模块(law_id=0);
创世宪法种子以 `CitizenConstitution.html` 维护中英双语原文,由本脚本复算为
`constitution.scale`;链上生效后的宪法修订仍必须走立法投票上链。本脚本保留作为
迁移溯源与创世种子复算依据。

逻辑:读块状 HTML(chapter-block / section-block / article-block,article 内含
article-paragraph),解析为 章>节>条>款 + 中英双语,SCALE 编码为 `Vec<Chapter>`
(= legislation-yuan::ChaptersOf 的底层布局),写 constitution.scale。

字段序必须与 legislation-yuan 链端类型一致:
  Chapter: number(u32) title(Vec<u8>) title_en(Option) sections(Vec<Section>)
  Section: number title title_en articles(Vec<Article>)
  Article: number title title_en body(Vec<u8>) body_en(Option) clauses(Vec<Clause>)
  Clause : number text text_en
顶层 = Vec<Chapter>。

用法:python3 citizenchain/scripts/parse_constitution.py
"""
import os
import re
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
HTML = os.path.join(HERE, "../runtime/primitives/src/CitizenConstitution.html")
OUT = os.path.join(HERE, "../runtime/governance/legislation-yuan/src/constitution.scale")

# ───────── 中文数字 → int(章号/款号用;条号走 EN 阿拉伯)─────────
_ZH = {"零": 0, "一": 1, "二": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}


def zh2int(s: str) -> int:
    s = s.strip()
    if not s:
        return 0
    total, section, num = 0, 0, 0
    for ch in s:
        if ch in _ZH:
            num = _ZH[ch]
        elif ch == "十":
            section += (num or 1) * 10
            num = 0
        elif ch == "百":
            section += (num or 1) * 100
            num = 0
        elif ch == "千":
            section += (num or 1) * 1000
            num = 0
    return total + section + num


# ───────── HTML 解析 ─────────
def clean(t: str) -> str:
    # 去标签残留 + 实体 + 归一空白(保留正文,trim 两端)
    t = re.sub(r"<[^>]+>", "", t)
    t = t.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&#39;", "'")
    return t.strip()


def split_zh_heading(cn: str, marker: str):
    """『第一章 总则』→ (1, '总则');『第一节 国家的定义』→ (1, '国家的定义')。"""
    m = re.match(rf"第([零一二三四五六七八九十百千]+){marker}\s*(.*)", cn)
    if not m:
        return 0, cn
    return zh2int(m.group(1)), m.group(2).strip()


def parse_clause(cn: str):
    """『第一款\t常规案…』→ (1, '常规案…');无款标记 → (0, 原文)。"""
    m = re.match(r"第([零一二三四五六七八九十百千]+)款[\s　\t]*(.*)", cn, re.S)
    if not m:
        return 0, cn
    return zh2int(m.group(1)), m.group(2).strip()


def parse(html: str):
    content = html[html.find('class="content"'):]
    block_re = re.compile(
        r'<(section|article)[^>]*class="block (chapter-block|section-block|article-block)"[^>]*>(.*?)</\1>',
        re.S,
    )
    heading_re = re.compile(
        r'<span class="cn heading-cn">(.*?)</span>\s*<span class="en heading-en">(.*?)</span>', re.S
    )
    para_re = re.compile(
        r'<span class="cn body-cn">(.*?)</span>\s*<span class="en body-en">(.*?)</span>', re.S
    )

    chapters = []
    cur_chapter = cur_section = None
    for m in block_re.finditer(content):
        kind, body = m.group(2), m.group(3)
        if kind == "chapter-block":
            h = heading_re.search(body)
            # 标题存完整(「第一章 总则」/「Chapter I General Principles」),number 仅供锚点排序。
            num, _ = split_zh_heading(clean(h.group(1)), "章")
            cur_chapter = {
                "number": num,
                "title": clean(h.group(1)),
                "title_en": clean(h.group(2)),
                "sections": [],
            }
            chapters.append(cur_chapter)
            cur_section = None
        elif kind == "section-block":
            h = heading_re.search(body)
            num, _ = split_zh_heading(clean(h.group(1)), "节")
            cur_section = {
                "number": num,
                "title": clean(h.group(1)),
                "title_en": clean(h.group(2)),
                "articles": [],
            }
            cur_chapter["sections"].append(cur_section)
        else:  # article-block
            h = heading_re.search(body)
            title_cn, title_en = clean(h.group(1)), clean(h.group(2))
            am = re.search(r"Article\s+(\d+)", title_en)
            anum = int(am.group(1)) if am else 0
            paras = [(clean(a), clean(b)) for a, b in para_re.findall(body)]
            if not paras:
                paras = [("", "")]
            body_cn, body_en = paras[0]
            clauses = []
            for i, (ccn, cen) in enumerate(paras[1:], start=1):
                # 款正文存完整原文(含「第N款」前缀,匹配原样式);number 仅供排序参考。
                knum, _ = parse_clause(ccn)
                clauses.append({"number": knum or i, "text": ccn, "text_en": cen})
            cur_section["articles"].append(
                {
                    "number": anum,
                    "title": title_cn,
                    "title_en": title_en,
                    "body": body_cn,
                    "body_en": body_en,
                    "clauses": clauses,
                }
            )
    return chapters


# ───────── SCALE 编码 ─────────
def compact(n: int) -> bytes:
    if n < 0:
        raise ValueError
    if n < 64:
        return bytes([n << 2])
    if n < 2**14:
        return struct.pack("<H", (n << 2) | 0b01)
    if n < 2**30:
        return struct.pack("<I", (n << 2) | 0b10)
    out = bytearray([0b11])
    v = n
    while v:
        out.append(v & 0xFF)
        v >>= 8
    out[0] |= (len(out) - 1 - 4) << 2
    return bytes(out)


def enc_str(s: str) -> bytes:
    b = s.encode("utf-8")
    return compact(len(b)) + b


def enc_opt_str(s: str) -> bytes:
    return b"\x01" + enc_str(s) if s else b"\x00"


def enc_u32(n: int) -> bytes:
    return struct.pack("<I", n)


def enc_vec(items, f) -> bytes:
    return compact(len(items)) + b"".join(f(x) for x in items)


def enc_clause(c):
    return enc_u32(c["number"]) + enc_str(c["text"]) + enc_opt_str(c["text_en"])


def enc_article(a):
    return (
        enc_u32(a["number"])
        + enc_str(a["title"])
        + enc_opt_str(a["title_en"])
        + enc_str(a["body"])
        + enc_opt_str(a["body_en"])
        + enc_vec(a["clauses"], enc_clause)
    )


def enc_section(s):
    return (
        enc_u32(s["number"])
        + enc_str(s["title"])
        + enc_opt_str(s["title_en"])
        + enc_vec(s["articles"], enc_article)
    )


def enc_chapter(c):
    return (
        enc_u32(c["number"])
        + enc_str(c["title"])
        + enc_opt_str(c["title_en"])
        + enc_vec(c["sections"], enc_section)
    )


def main():
    html = open(HTML, encoding="utf-8").read()
    chapters = parse(html)
    n_sec = sum(len(c["sections"]) for c in chapters)
    n_art = sum(len(s["articles"]) for c in chapters for s in c["sections"])
    n_clause = sum(len(a["clauses"]) for c in chapters for s in c["sections"] for a in s["articles"])
    scale = enc_vec(chapters, enc_chapter)
    open(OUT, "wb").write(scale)
    print(f"章 {len(chapters)} / 节 {n_sec} / 条 {n_art} / 款 {n_clause}")
    print(f"SCALE 字节 {len(scale)} → {OUT}")
    # 抽样自检
    empty_body = [a["number"] for c in chapters for s in c["sections"] for a in s["articles"] if not a["body"]]
    if empty_body:
        print(f"!!! 空 body 条号(违反 body 必填): {empty_body}", file=sys.stderr)
    nums = [a["number"] for c in chapters for s in c["sections"] for a in s["articles"]]
    print(f"条号范围 {min(nums)}..{max(nums)},去重 {len(set(nums))} 个")


if __name__ == "__main__":
    main()
