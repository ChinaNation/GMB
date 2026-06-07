# 任务卡：CPMS 初始化页恢复「上传二维码」按钮

## 任务需求

CPMS 初始化第 1 步「扫描安装码」当前只有「开启扫码」一个按钮，恢复其右侧的「上传二维码」按钮：用户选本地图片，前端识别图片中的安装码二维码，复用现有 `handleQr1Scanned` 初始化流程。

## 背景结论（分析已确认）

- CPMS 扫码引擎是浏览器原生 `BarcodeDetector`（`qr/cameraScanner.ts`），`detect()` 接受图片源 → 摄像头与上传图片共用同一引擎，无需第三方库。
- SFID 已有同款先例 `memory/08-tasks/done/20260525-sfid-bind-upload-qr.md`：`utils/cameraScanner.ts` 导出 `decodeQrImageFile(file)`，BindModal 在「开启扫码」右侧加「上传二维码」，本地解码、**不传图片到后端**、复用原流程。CPMS 照搬此 pattern。
- 范围用户确认：**仅初始化「扫描安装码」步骤**（与 SFID 先例只给档案码加上传、不扩到钱包码一致）；绑定管理员步骤（公民钱包码）不加。
- 冲突已知：`CPMS_TECHNICAL.md:182` 原写「只保留摄像头扫码模式」，本次推翻，文档同步改。

## 建议模块

- CPMS 前端 `qr`（解码工具 + 统一扫码组件）
- CPMS 前端 `initialize`
- CPMS 技术文档

## 影响范围

- `cpms/frontend/qr/cameraScanner.ts`：新增 `createQrDetector` helper + `decodeQrImageFile(file): Promise<string>`，与摄像头共用 `BarcodeDetector`，镜像 SFID 命名。涉及代码。
- `cpms/frontend/qr/CameraQrScanner.tsx`：加可选 `allowUpload` / `uploadLabel`，在统一组件内部渲染「上传二维码」按钮 + 隐藏 `<input type=file accept=image/*>`，选图后停摄像头、本地解码、复用 `onDetected`/`onError`。把上传入口收进统一组件，守住「不分散第二套扫码入口」。涉及代码。
- `cpms/frontend/initialize/InstallPage.tsx`：第 1 步 `CameraQrScanner` 传 `allowUpload`；第 2 步不传。涉及代码。
- `cpms/CPMS_TECHNICAL.md:182`：规则改为「摄像头 + 上传图片本地解码两种入口，统一走 CameraQrScanner；上传图片只在前端本地解析，不传后端」。涉及文档。

## 主要风险点

- 必须复用 `handleQr1Scanned`，不新起第二套初始化逻辑。
- 选图前先停摄像头，上传解码与摄像头不能同时推进步骤状态。
- 图片只在前端本地解码，绝不上传图片文件到后端。
- 浏览器不支持 `BarcodeDetector` / 非图片 / 未识别到二维码 → 明确报错，不改步骤状态。
- 推翻已记录决策（camera-only），文档不同步改即残留。
- file input 选同一文件需可重复触发（onChange 后清空 value）。

## 是否需要先沟通

- 范围与是否推翻 camera-only 规则已确认。

## 执行清单

- [x] `cameraScanner.ts` 加 `createQrDetector` + `decodeQrImageFile`（镜像 SFID 命名，消息改通用「二维码」）。
- [x] `CameraQrScanner.tsx` 加 `allowUpload`/`uploadLabel`：组件内渲染「上传二维码」按钮 + 隐藏 file input，选图停摄像头、本地解码、复用 `onDetected`/`onError`，onChange 后清空 value。
- [x] `InstallPage.tsx` 第 1 步 `CameraQrScanner` 开 `allowUpload`；第 2 步未传。
- [x] `CPMS_TECHNICAL.md:182` 规则改为摄像头 + 上传本地解码两种入口、图片不传后端。
- [x] `npm run build` 通过（55 modules，tsc + vite 全绿）；旧规则「只保留摄像头扫码模式」在代码/文档归零。

## 完成记录

- 2026-06-06：创建任务卡，开始执行。
- 2026-06-06：执行完成。照搬 SFID `decodeQrImageFile` 先例，把上传入口收进统一组件 `CameraQrScanner`（守住「不分散第二套扫码入口」），仅初始化「扫描安装码」步骤开启；摄像头与上传共用 `BarcodeDetector` 与同一 `onDetected`，图片只在前端本地解析不传后端。推翻并更新了 CPMS camera-only 文档规则。npm run build 全绿。
