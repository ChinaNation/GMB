#!/usr/bin/env python3
"""统一解析 git stash pop 留下的冲突标记，保留 Stashed (新版本=改名后)。

冲突格式:
  <<<<<<< Updated upstream
  ...HEAD 版本...
  =======
  ...Stashed 版本...
  >>>>>>> Stashed changes

策略:HEAD 是 stash 之前的旧代码,Stashed 是 big-bang 改名后的代码,选 Stashed。
"""

import re
import sys
from pathlib import Path

ROOTS = [
    Path("/Users/rhett/GMB/wuminapp"),
    Path("/Users/rhett/GMB/wumin"),
    Path("/Users/rhett/GMB/citizenchain"),
    Path("/Users/rhett/GMB/sfid"),
]
EXCLUDE_DIR_PARTS = {"target", "node_modules", "dist", "build", ".dart_tool"}

# 一次匹配:<<<<<<< 段 ... ======= 段 ... >>>>>>> 段
PATTERN = re.compile(
    r"^<<<<<<<[^\n]*\n(.*?)^=======\s*\n(.*?)^>>>>>>>[^\n]*\n",
    re.DOTALL | re.MULTILINE,
)

def resolve(text: str) -> tuple[str, int]:
    count = [0]

    def keep_stashed(m: re.Match) -> str:
        count[0] += 1
        # m.group(2) = Stashed 段(改名后)
        return m.group(2)

    new = PATTERN.sub(keep_stashed, text)
    return new, count[0]


def main():
    total_files = 0
    total_blocks = 0
    for root in ROOTS:
        for p in root.rglob("*"):
            if not p.is_file():
                continue
            if set(p.parts) & EXCLUDE_DIR_PARTS:
                continue
            try:
                text = p.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            if "<<<<<<< " not in text and "<<<<<<<" not in text:
                continue
            new_text, blocks = resolve(text)
            if blocks > 0:
                p.write_text(new_text, encoding="utf-8")
                total_files += 1
                total_blocks += blocks
                print(f"  {p}: 解析 {blocks} 个冲突段")
    print(f"\n共处理 {total_files} 文件,解析 {total_blocks} 个冲突段(全部保留 Stashed = 改名后版本)")


if __name__ == "__main__":
    main()
