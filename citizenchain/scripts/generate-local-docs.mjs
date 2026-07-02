import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url)); // citizenchain/scripts
const chainRoot = path.resolve(scriptDir, '..');                // citizenchain
const frontendRoot = path.resolve(chainRoot, 'node/frontend');  // citizenchain/node/frontend
const repoRoot = path.resolve(chainRoot, '..');                 // 仓库根
const outputPath = path.resolve(frontendRoot, 'generated/local-docs.generated.ts');

const sources = [
  {
    key: 'whitepaper',
    title: '白皮书',
    sourcePath: 'website/src/whitepaper.md',
    // 白皮书正文迁入官网源码目录后，图片仍沿用 docs/assets 资源目录。
    assetFallbackRoots: ['docs'],
  }
];

const mimeTypes = new Map([
  ['.png', 'image/png'],
  ['.jpg', 'image/jpeg'],
  ['.jpeg', 'image/jpeg'],
  ['.webp', 'image/webp'],
  ['.gif', 'image/gif'],
  ['.svg', 'image/svg+xml'],
]);

function toDataUri(absPath) {
  const ext = path.extname(absPath).toLowerCase();
  const mime = mimeTypes.get(ext);
  if (!mime || !fs.existsSync(absPath)) return null;
  return `data:${mime};base64,${fs.readFileSync(absPath).toString('base64')}`;
}

function resolveRelativeAsset(sourceAbs, item, assetPath) {
  if (/^(?:[a-z]+:|#|\/)/i.test(assetPath)) return null;
  const decodedPath = decodeURIComponent(assetPath);
  const primary = path.resolve(path.dirname(sourceAbs), decodedPath);
  if (fs.existsSync(primary)) return primary;

  for (const fallbackRoot of item.assetFallbackRoots ?? []) {
    const fallback = path.resolve(repoRoot, fallbackRoot, decodedPath);
    if (fs.existsSync(fallback)) return fallback;
  }

  return primary;
}

function embedLocalImages(markdown, sourceAbs, item) {
  const htmlImgPattern = /(<img\b[^>]*\bsrc=["'])([^"']+)(["'][^>]*>)/gi;
  const markdownImgPattern = /(!\[[^\]]*\]\()([^)]+)(\))/g;

  // 白皮书会被内置进前端 bundle，相对图片必须转成 data URI 才能在桌面端显示。
  return markdown
    .replace(htmlImgPattern, (match, prefix, assetPath, suffix) => {
      const absAssetPath = resolveRelativeAsset(sourceAbs, item, assetPath);
      if (!absAssetPath) return match;
      const dataUri = toDataUri(absAssetPath);
      return dataUri ? `${prefix}${dataUri}${suffix}` : match;
    })
    .replace(markdownImgPattern, (match, prefix, assetPath, suffix) => {
      const absAssetPath = resolveRelativeAsset(sourceAbs, item, assetPath.trim());
      if (!absAssetPath) return match;
      const dataUri = toDataUri(absAssetPath);
      return dataUri ? `${prefix}${dataUri}${suffix}` : match;
    });
}

const docs = sources.map((item) => {
  const abs = path.resolve(repoRoot, item.sourcePath);
  const markdown = embedLocalImages(fs.readFileSync(abs, 'utf8'), abs, item);
  return {
    key: item.key,
    title: item.title,
    sourcePath: item.sourcePath,
    markdown,
    sha256: crypto.createHash('sha256').update(markdown).digest('hex'),
  };
});

fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(
  outputPath,
  [
    '// 本文件由 citizenchain/scripts/generate-local-docs.mjs 自动生成。',
    '// 本文件只内置白皮书；公民宪法由链上 runtime API 返回。',
    '',
    'export type LocalDocKey = "whitepaper";',
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
