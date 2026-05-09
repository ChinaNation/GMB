import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const frontendRoot = path.resolve(scriptDir, '..');
const repoRoot = path.resolve(frontendRoot, '../../..');
const outputPath = path.resolve(frontendRoot, 'generated/local-docs.generated.ts');

const sources = [
  {
    key: 'whitepaper',
    title: '白皮书',
    sourcePath: 'docs/《白皮书》.md',
  },
  {
    key: 'constitution',
    title: '公民宪法',
    sourcePath: 'docs/《公民宪法》.md',
  },
];

const docs = sources.map((item) => {
  const abs = path.resolve(repoRoot, item.sourcePath);
  const markdown = fs.readFileSync(abs, 'utf8');
  return {
    ...item,
    markdown,
    sha256: crypto.createHash('sha256').update(markdown).digest('hex'),
  };
});

fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(
  outputPath,
  [
    '// 本文件由 scripts/generate-local-docs.mjs 自动生成。',
    '// 中文注释：白皮书和公民宪法唯一真源位于仓库根目录 docs/，构建前会内置到桌面端 bundle。',
    '',
    'export type LocalDocKey = "whitepaper" | "constitution";',
    '',
    'export type LocalDoc = {',
    '  key: LocalDocKey;',
    '  title: string;',
    '  sourcePath: string;',
    '  sha256: string;',
    '  markdown: string;',
    '};',
    '',
    `export const LOCAL_DOCS = ${JSON.stringify(docs, null, 2)} as const satisfies readonly LocalDoc[];`,
    '',
  ].join('\n'),
  'utf8',
);

console.log(`generated ${path.relative(repoRoot, outputPath)}`);
