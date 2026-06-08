#!/usr/bin/env python3
"""把白皮书 docs/《白皮书》.md 里的内联 base64 图片外链成 docs/assets/ 下的文件。

- 匹配 <img ... src="data:image/<ext>;base64,<DATA>" ...>，仅替换 src，保留 alt/width 等其余属性。
- 按 alt 文本映射成语义化英文文件名，未知 alt 回退 whitepaper-<N>.<ext>。
- 幂等：已外链（src="./assets/...")的 img 不再处理；可重复运行。

用法：
  python3 tools/extract_whitepaper_images.py            # dry-run，仅打印将要做的改动
  python3 tools/extract_whitepaper_images.py --apply    # 解码写盘 + 回写 .md
"""

import argparse
import base64
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MD_PATH = ROOT / "docs" / "《白皮书》.md"
ASSETS_DIR = ROOT / "docs" / "assets"

# alt 文本 -> 语义化文件名（不含扩展名）
ALT_TO_NAME = {
    "节点图": "whitepaper-node-diagram",
    "联储会架构图": "whitepaper-reserve-architecture",
}

DATA_IMG_RE = re.compile(
    r'<img\b([^>]*?)src="data:image/([a-zA-Z0-9.+-]+);base64,([^"]+)"([^>]*)>',
    flags=re.S,
)
ALT_RE = re.compile(r'alt="([^"]*)"')


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true", help="实际写盘，否则仅 dry-run")
    args = ap.parse_args()

    if not MD_PATH.exists():
        print(f"未找到白皮书: {MD_PATH}", file=sys.stderr)
        return 1

    md = MD_PATH.read_text(encoding="utf-8")
    counter = {"n": 0}

    def replace(m: re.Match) -> str:
        counter["n"] += 1
        pre, ext, data_b64, post = m.group(1), m.group(2), m.group(3), m.group(4)
        ext = "jpg" if ext.lower() in ("jpeg", "jpg") else ext.lower()
        alt_m = ALT_RE.search(pre + post)
        alt = alt_m.group(1) if alt_m else ""
        stem = ALT_TO_NAME.get(alt, f"whitepaper-{counter['n']}")
        fname = f"{stem}.{ext}"
        raw = base64.b64decode(data_b64)
        print(f"  img#{counter['n']}: alt={alt!r} -> docs/assets/{fname} ({len(raw):,} bytes)")
        if args.apply:
            ASSETS_DIR.mkdir(parents=True, exist_ok=True)
            (ASSETS_DIR / fname).write_bytes(raw)
        return f'<img{pre}src="./assets/{fname}"{post}>'

    new_md = DATA_IMG_RE.sub(replace, md)

    if counter["n"] == 0:
        print("没有发现内联 base64 图片（可能已外链）。")
        return 0

    print(f"\n原 .md: {len(md.encode('utf-8')):,} bytes -> 新 .md: {len(new_md.encode('utf-8')):,} bytes")
    if args.apply:
        MD_PATH.write_text(new_md, encoding="utf-8")
        print("已回写 docs/《白皮书》.md 并写出 docs/assets/ 图片。")
    else:
        print("dry-run（未写盘）。加 --apply 执行。")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
