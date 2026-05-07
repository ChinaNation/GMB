#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""把 7 个 china_*.rs 中 sfid_number 与 sfid_name 的相对顺序交换:
旧:  sfid_number 在前 / sfid_name 在后
新:  sfid_name   在前 / sfid_number 在后

覆盖两类位置:
  1. struct 定义的字段声明(7 处)
  2. struct literal 初始化器(277 处)
"""

import re
from pathlib import Path

CHINA_DIR = Path("/Users/rhett/GMB/citizenchain/runtime/primitives/china")
FILES = [
    "china_cb.rs",
    "china_ch.rs",
    "china_lf.rs",
    "china_sf.rs",
    "china_jc.rs",
    "china_jy.rs",
    "china_zf.rs",
]

# struct 定义:`    pub sfid_number: TYPE,\n    pub sfid_name: TYPE,`
PATTERN_DEF = re.compile(
    r"^(?P<indent>[ \t]+)pub sfid_number:(?P<num_rest>[^\n]+)\n"
    r"(?P=indent)pub sfid_name:(?P<name_rest>[^\n]+)$",
    re.MULTILINE,
)

# literal:`        sfid_number: "...",\n        sfid_name: "...",`
PATTERN_LIT = re.compile(
    r"^(?P<indent>[ \t]+)sfid_number:(?P<num_rest>[^\n]+)\n"
    r"(?P=indent)sfid_name:(?P<name_rest>[^\n]+)$",
    re.MULTILINE,
)


def swap_def(m: re.Match) -> str:
    indent = m.group("indent")
    return f"{indent}pub sfid_name:{m.group('name_rest')}\n{indent}pub sfid_number:{m.group('num_rest')}"


def swap_lit(m: re.Match) -> str:
    indent = m.group("indent")
    return f"{indent}sfid_name:{m.group('name_rest')}\n{indent}sfid_number:{m.group('num_rest')}"


def main():
    total_def = 0
    total_lit = 0
    for fname in FILES:
        fp = CHINA_DIR / fname
        text = fp.read_text(encoding="utf-8")
        new, n_def = PATTERN_DEF.subn(swap_def, text)
        new, n_lit = PATTERN_LIT.subn(swap_lit, new)
        if new != text:
            fp.write_text(new, encoding="utf-8")
            print(f"  {fname}: struct def {n_def} | literal {n_lit}")
            total_def += n_def
            total_lit += n_lit

    print(f"\n合计:struct 定义交换 {total_def} 处 + literal 初始化交换 {total_lit} 处 = {total_def + total_lit} 处")


if __name__ == "__main__":
    main()
