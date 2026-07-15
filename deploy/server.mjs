import { createServer } from 'node:http';
import { createPrivateKey, createPublicKey, randomBytes, randomUUID } from 'node:crypto';
import { spawn, spawnSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { isIP } from 'node:net';

const deployDir = dirname(fileURLToPath(import.meta.url));
const rootDir = dirname(deployDir);
const host = '127.0.0.1';
const port = Number(process.env.GMB_DEPLOY_PORT || 41731);
const inheritedListenFd = process.env.GMB_DEPLOY_FD ? Number(process.env.GMB_DEPLOY_FD) : null;
const idleExitMs = 15 * 60 * 1000;
const sessionToken = randomBytes(32).toString('hex');
const institutionCatalog = JSON.parse(await readFile(join(rootDir, 'citizenchain', 'node', 'src', 'settings', 'institution-catalog.json'), 'utf8'));
const chainNodes = institutionCatalog.map((entry, index) => ({
  id: `node-${String(index + 1).padStart(2, '0')}`,
  number: index + 1,
  label: entry.authorityNodeLabel,
  role: entry.role,
  domain: entry.domain,
  peerId: entry.peerId,
  grandpaPubkeyHex: entry.grandpaPubkeyHex.toLowerCase(),
}));
const nodeSecretNames = ['SERVER_IP', 'BOOTNODE_KEY', 'VALIDATOR_KEY', 'SSH_KEY'];
const secretNames = [
  'CF_ACCOUNT_ID', 'CF_API_TOKEN', 'CHAIN_ID', 'CHAIN_SECRET', 'CHAIN_URL',
  'FCM_EMAIL', 'FCM_KEY', 'FCM_PROJECT', 'HASH_KEY', 'IMAGES_SIGNING_KEY',
  'R2_ACCESS_ID', 'R2_SECRET_KEY', 'STREAM_HOOK_SECRET', 'STRIPE_API_KEY',
  'STRIPE_HOOK_SECRET', 'TURNSTILE_SECRET',
];

// 中文注释：注释只解释用途，不包含格式示例或任何密钥片段。
const secretComments = {
  CF_ACCOUNT_ID: 'Cloudflare 账户编号',
  CF_API_TOKEN: 'Worker 调用 Images 与 Stream 的低权限令牌',
  CHAIN_ID: '受保护链 RPC 的 Access 客户端 ID',
  CHAIN_SECRET: '受保护链 RPC 的 Access 客户端密钥',
  CHAIN_URL: 'CitizenChain 受保护 HTTPS RPC 地址',
  FCM_EMAIL: 'Firebase 推送服务账号邮箱',
  FCM_KEY: 'Firebase 推送服务账号私钥',
  FCM_PROJECT: 'Firebase 推送项目编号',
  HASH_KEY: 'Worker 敏感标识哈希密钥',
  IMAGES_SIGNING_KEY: '图片上传与访问签名密钥',
  R2_ACCESS_ID: 'R2 对象存储访问 ID',
  R2_SECRET_KEY: 'R2 对象存储访问密钥',
  STREAM_HOOK_SECRET: 'Cloudflare Stream 回调验签密钥',
  STRIPE_API_KEY: 'Stripe 订阅与支付 API 密钥',
  STRIPE_HOOK_SECRET: 'Stripe Webhook 回调验签密钥',
  TURNSTILE_SECRET: 'Cloudflare Turnstile 服务端校验密钥',
  GMB_APP_KEY: 'CitizenApp 与 CitizenWallet Android 正式签名材料',
  GMB_SSH_KEY: 'CitizenChain 服务器部署 SSH 私钥',
  GMB_TOP_KEY: 'CitizenChain 正式版与更新包签名私钥',
  GMB_TOP_PUBKEY: 'CitizenChain 更新包签名公钥',
};

const modules = [
  {
    id: 'cloudflare', icon: '☁️', title: 'CitizenApp Cloudflare',
    description: '聊天、广场、会员和 D1 Worker',
    secrets: { keychain: secretNames, github: [] },
    actions: [
      { id: 'staging', title: '测试部署', mode: 'staging', production: false },
      {
        id: 'membership-test', title: '会员逻辑真实测试', mode: 'membership-test', production: false,
        keychain: ['CF_ACCOUNT_ID', 'CF_API_TOKEN', 'STRIPE_API_KEY'],
      },
      {
        id: 'membership-e2e', title: 'Stripe Sandbox 全链路验收', mode: 'membership-e2e', production: false,
        keychain: ['CF_ACCOUNT_ID', 'CF_API_TOKEN', 'STRIPE_API_KEY', 'STRIPE_HOOK_SECRET'],
      },
      { id: 'production', title: '生产部署', mode: 'production', production: true },
    ],
  },
  {
    id: 'citizenweb', icon: '🌐', title: 'CitizenWeb', description: '官方网站 Cloudflare Pages',
    secrets: { keychain: ['CF_ACCOUNT_ID'], github: [] },
    actions: [
      { id: 'local-start', title: '测试部署', mode: 'local-start', production: false, localOnly: true, keychain: [] },
      { id: 'local-stop', title: '关闭测试部署', mode: 'local-stop', production: false, localOnly: true, keychain: [] },
      { id: 'production', title: '生产部署', mode: 'production', production: true },
    ],
  },
  {
    id: 'citizenapp', icon: '📱', title: 'CitizenApp', description: '公民 Android CI 与正式版',
    secrets: { keychain: [], github: ['GMB_APP_KEY'] },
    actions: [
      { id: 'ci', title: '运行 CI', mode: 'ci', production: false },
      { id: 'release', title: '正式 Release', mode: 'release', production: true },
    ],
  },
  {
    id: 'citizenwallet', icon: '🔐', title: 'CitizenWallet', description: '公民钱包 Android CI 与正式版',
    secrets: { keychain: [], github: ['GMB_APP_KEY'] },
    actions: [
      { id: 'ci', title: '运行 CI', mode: 'ci', production: false },
      { id: 'release', title: '正式 Release', mode: 'release', production: true },
    ],
  },
  {
    id: 'citizenchain', icon: '⛓️', title: 'CitizenChain', description: '节点 CI、正式版与服务器部署',
    secrets: { keychain: [], github: ['GMB_SSH_KEY', 'GMB_TOP_KEY', 'GMB_TOP_PUBKEY'] },
    actions: [
      { id: 'ci', title: '运行 CI', mode: 'ci', production: false },
      { id: 'release', title: '正式 Release', mode: 'release', production: true },
      { id: 'deploy-all', title: '部署服务器', mode: 'deploy-all', production: true },
      // 中文注释：节点卡片单独调用此动作；顶部批量按钮使用 deploy-all，不重复展示。
      { id: 'deploy', title: '部署该节点', mode: 'deploy', production: true, hidden: true },
    ],
  },
  {
    id: 'citizenchainwasm', icon: '🧬', title: 'CitizenChain WASM', description: 'Runtime WASM 构建与校验',
    secrets: { keychain: [], github: ['GMB_SSH_KEY'] },
    actions: [{ id: 'ci', title: '运行 WASM CI', mode: 'ci', production: false }],
  },
];

const runs = new Map();
let activeRunId = null;

function json(res, status, value) {
  res.writeHead(status, { 'content-type': 'application/json; charset=utf-8', 'cache-control': 'no-store' });
  res.end(JSON.stringify(value));
}

function hasSession(req) {
  return (req.headers.cookie || '').split(';').some((item) => item.trim() === `gmb_deploy=${sessionToken}`);
}

function validOrigin(req) {
  const origin = req.headers.origin;
  return !origin || origin === `http://${host}:${port}`;
}

function keychainExists(environment, secretName) {
  return spawnSync(join(deployDir, 'keychain.sh'), ['exists', environment, secretName], { stdio: 'ignore' }).status === 0;
}

function keychainGet(environment, secretName) {
  const command = secretName === 'SSH_KEY' ? 'get-multiline' : 'get';
  const result = spawnSync(join(deployDir, 'keychain.sh'), [command, environment, secretName], { encoding: 'utf8' });
  if (result.status !== 0) throw new Error(`Keychain 缺少 ${environment}:${secretName}`);
  return result.stdout.replace(/\n$/, '');
}

function keychainPut(environment, secretName, value) {
  const command = secretName === 'SSH_KEY' ? 'put-multiline' : 'put';
  const result = spawnSync(join(deployDir, 'keychain.sh'), [command, environment, secretName], {
    input: `${value}\n`, encoding: 'utf8', stdio: ['pipe', 'ignore', 'pipe'],
  });
  if (result.status !== 0) throw new Error(`Keychain 写入失败：${environment}:${secretName}`);
}

function keychainDelete(environment, secretName) {
  if (!keychainExists(environment, secretName)) return;
  const result = spawnSync(join(deployDir, 'keychain.sh'), ['delete', environment, secretName], {
    encoding: 'utf8', stdio: ['ignore', 'ignore', 'pipe'],
  });
  if (result.status !== 0) throw new Error(`Keychain 删除失败：${environment}:${secretName}`);
}

function authorizeProduction(reason) {
  const auth = spawnSync(join(deployDir, '.runtime', 'touchid-auth'), [], { stdio: 'inherit' });
  if (auth.status !== 0) throw new Error(`Touch ID 验证失败，未执行${reason}`);
}

function normalizePrivateHex(value, label) {
  const normalized = String(value || '').trim().replace(/^0x/i, '').toLowerCase();
  if (!/^[0-9a-f]{64}$/.test(normalized)) throw new Error(`${label}必须是64位十六进制私钥`);
  return normalized;
}

function ed25519PublicKey(privateHex) {
  const prefix = Buffer.from('302e020100300506032b657004220420', 'hex');
  const privateKey = createPrivateKey({ key: Buffer.concat([prefix, Buffer.from(privateHex, 'hex')]), format: 'der', type: 'pkcs8' });
  const publicDer = createPublicKey(privateKey).export({ format: 'der', type: 'spki' });
  return Buffer.from(publicDer).subarray(-32);
}

function base58Encode(bytes) {
  const alphabet = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  let number = BigInt(`0x${Buffer.from(bytes).toString('hex') || '0'}`);
  let output = '';
  while (number > 0n) { output = alphabet[Number(number % 58n)] + output; number /= 58n; }
  for (const byte of bytes) { if (byte !== 0) break; output = `1${output}`; }
  return output || '1';
}

function peerIdFromBootnodeKey(privateHex) {
  const publicKey = ed25519PublicKey(privateHex);
  // 中文注释：libp2p Ed25519 PeerId 是 identity multihash 包裹的 protobuf 公钥。
  return base58Encode(Buffer.concat([Buffer.from([0x00, 0x24, 0x08, 0x01, 0x12, 0x20]), publicKey]));
}

function nodeStatus(node) {
  const configured = Object.fromEntries(nodeSecretNames.map((name) => [name, keychainExists(node.id, name)]));
  let serverIp = null;
  if (configured.SERVER_IP) serverIp = keychainGet(node.id, 'SERVER_IP');
  return { ...node, serverIp, configured };
}

function validateNodeConfiguration(node, body) {
  const writes = [];
  if (body.serverIp !== undefined) {
    const serverIp = String(body.serverIp).trim();
    if (!isIP(serverIp)) throw new Error('服务器 IP 必须是有效的 IPv4 或 IPv6 地址');
    writes.push(['SERVER_IP', serverIp]);
  }
  if (String(body.bootnodeKey || '').trim()) {
    const privateHex = normalizePrivateHex(body.bootnodeKey, '引导节点密钥');
    const derivedPeerId = peerIdFromBootnodeKey(privateHex);
    if (derivedPeerId !== node.peerId) {
      // 中文注释：只返回由私钥派生的公开身份，便于发现节点填错，绝不回显私钥。
      const owner = chainNodes.find((item) => item.peerId === derivedPeerId);
      const detail = owner
        ? `该密钥属于「${owner.label}」，不属于「${node.label}」`
        : `派生 PeerId 为 ${derivedPeerId}，不属于44个登记节点`;
      throw new Error(`引导节点密钥与该节点公开 PeerId 不匹配；${detail}`);
    }
    writes.push(['BOOTNODE_KEY', privateHex]);
  }
  if (String(body.validatorKey || '').trim()) {
    const privateHex = normalizePrivateHex(body.validatorKey, '验证节点密钥');
    if (privateHex === node.grandpaPubkeyHex) {
      throw new Error('输入的是该节点 GRANDPA 公钥，请输入32字节验证节点私钥 seed');
    }
    const derivedPubkey = ed25519PublicKey(privateHex).toString('hex');
    if (derivedPubkey !== node.grandpaPubkeyHex) {
      // 中文注释：GRANDPA 公钥属于公开节点身份，可用于说明填错哪个节点而不泄露私密材料。
      const owner = chainNodes.find((item) => item.grandpaPubkeyHex === derivedPubkey);
      const detail = owner
        ? `该密钥属于「${owner.label}」，不属于「${node.label}」`
        : `派生 GRANDPA 公钥为 ${derivedPubkey}，不属于44个登记节点`;
      throw new Error(`验证节点密钥与该节点 GRANDPA 公钥不匹配；${detail}`);
    }
    writes.push(['VALIDATOR_KEY', privateHex]);
  }
  if (String(body.sshKey || '').trim()) {
    const sshKey = String(body.sshKey).trim();
    const lines = sshKey.split(/\r?\n/);
    const begin = lines[0]?.match(/^-----BEGIN ((?:OPENSSH |RSA |EC )?PRIVATE KEY)-----$/);
    if (!begin || lines.length < 3 || lines.at(-1) !== `-----END ${begin[1]}-----`) {
      throw new Error('服务器 SSH 私钥不完整，必须包含开始行、密钥正文和匹配的结束行');
    }
    writes.push(['SSH_KEY', `${sshKey}\n`]);
  }
  if (writes.length === 0) throw new Error('没有需要保存的节点配置');
  return writes;
}

function replaceNodeConfiguration(node, writes) {
  // 中文注释：指纹通过后先快照本次字段，再清除旧值并写入新值；任一步失败就恢复原状态。
  const previous = new Map(writes.map(([name]) => [name, keychainExists(node.id, name) ? keychainGet(node.id, name) : null]));
  try {
    for (const [name] of writes) keychainDelete(node.id, name);
    for (const [name, value] of writes) keychainPut(node.id, name, value);
  } catch (error) {
    for (const [name] of writes) {
      try { keychainDelete(node.id, name); } catch {}
      const oldValue = previous.get(name);
      if (oldValue !== null) {
        try { keychainPut(node.id, name, oldValue); } catch {}
      }
    }
    throw new Error(`节点配置写入失败，已尝试恢复旧配置：${error.message}`);
  }
}

function githubSecretNames() {
  const result = spawnSync('gh', ['secret', 'list', '--json', 'name', '--jq', '.[].name'], {
    cwd: rootDir, encoding: 'utf8', timeout: 15000,
  });
  return result.status === 0 ? new Set(result.stdout.trim().split('\n').filter(Boolean)) : new Set();
}

function citizenwebLocalRunning() {
  const result = spawnSync('curl', ['--fail', '--silent', '--max-time', '1', 'http://127.0.0.1:41732'], { stdio: 'ignore' });
  return result.status === 0;
}

function redact(text, secretValues = []) {
  let safe = String(text);
  for (const value of secretValues) {
    if (value && value.length >= 6) safe = safe.split(value).join('[已脱敏]');
  }
  return safe
    .replace(/\b(?:sk|rk)_(?:test|live)_[A-Za-z0-9]+\b/g, '[Stripe 密钥已脱敏]')
    .replace(/\bwhsec_[A-Za-z0-9]+\b/g, '[Webhook 密钥已脱敏]')
    .replace(/-----BEGIN [^-]+-----[\s\S]*?-----END [^-]+-----/g, '[私钥已脱敏]');
}

function emit(run, event, data) {
  const item = { event, data, at: new Date().toISOString() };
  run.events.push(item);
  for (const listener of run.listeners) listener(item);
}

function runSnapshot(run) {
  if (!run) return null;
  // 中文注释：任务输出在进入事件列表前已经脱敏；状态接口只返回可恢复控制台所需字段。
  return {
    id: run.id,
    moduleId: run.moduleId,
    actionId: run.actionId,
    state: run.state,
    startedAt: run.startedAt,
    finishedAt: run.finishedAt ?? null,
    exitCode: run.exitCode,
    events: run.events,
  };
}

function finishRun(run, code) {
  run.state = code === 0 ? 'success' : 'failed';
  run.exitCode = code;
  run.finishedAt = new Date().toISOString();
  activeRunId = null;
  const successMessage = run.actionId === 'membership-test'
    ? '[完成] 会员真实测试已执行完毕，请以上方逐项报告为准。'
    : '[完成] 部署任务执行成功。';
  emit(run, 'log', code === 0 ? successMessage : '[失败] 任务已停止，请查看上方步骤。');
  emit(run, 'done', { state: run.state, exitCode: code });
}

function createRun(module, action) {
  const run = {
    id: randomUUID(), moduleId: module.id, actionId: action.id, state: 'starting',
    startedAt: new Date().toISOString(), events: [], listeners: new Set(), exitCode: null,
  };
  runs.set(run.id, run);
  activeRunId = run.id;
  emit(run, 'log', `[开始] ${module.title} · ${action.title}`);
  return run;
}

function baseChildEnv() {
  return {
    ...process.env,
    PATH: `${dirname(process.execPath)}:${process.env.PATH ?? ''}`,
    GMB_ROOT: rootDir,
    GMB_NODE_BIN: process.execPath,
    GMB_NPX_BIN: join(dirname(process.execPath), 'npx'),
  };
}

function nodeChildEnv(node, secretValues) {
  const childEnv = baseChildEnv();
  const serverIp = keychainGet(node.id, 'SERVER_IP');
  const bootnodeKey = keychainGet(node.id, 'BOOTNODE_KEY');
  const validatorKey = keychainGet(node.id, 'VALIDATOR_KEY');
  const sshKey = keychainGet(node.id, 'SSH_KEY');
  Object.assign(childEnv, {
    GMB_NODE_ID: node.id,
    GMB_NODE_LABEL: node.label,
    GMB_NODE_IP: serverIp,
    GMB_NODE_PEER_ID: node.peerId,
    GMB_NODE_GRANDPA_PUBKEY: node.grandpaPubkeyHex,
    GMB_NODE_BOOTNODE_KEY: bootnodeKey,
    GMB_NODE_VALIDATOR_KEY: validatorKey,
    GMB_NODE_SSH_KEY: sshKey,
  });
  secretValues.push(bootnodeKey, validatorKey, sshKey);
  return childEnv;
}

function startBatchRun(module, action) {
  if (activeRunId) throw new Error('已有部署任务正在运行');
  const run = createRun(module, action);
  const script = join(deployDir, 'actions', `${module.id}.sh`);
  const results = [];
  try {
    authorizeProduction('批量部署全部配置齐全节点');
    const readyNodes = chainNodes.filter((node) => nodeSecretNames.every((name) => keychainExists(node.id, name)));
    const skipped = chainNodes.length - readyNodes.length;
    if (readyNodes.length === 0) {
      emit(run, 'log', `[汇总] 成功 0，失败 0，跳过 ${skipped}：没有配置齐全的节点。`);
      finishRun(run, 0);
      return run;
    }
    emit(run, 'log', `[批量] 已授权，同时部署 ${readyNodes.length} 个配置齐全节点；成功节点不输出过程日志。`);
    run.state = 'running';
    const jobs = readyNodes.map((node) => new Promise((resolve) => {
      const secretValues = [];
      let output = '';
      let child;
      try {
        child = spawn('bash', [script, 'deploy'], { cwd: rootDir, env: nodeChildEnv(node, secretValues) });
        child.stdout.on('data', (chunk) => { output += chunk.toString(); });
        child.stderr.on('data', (chunk) => { output += chunk.toString(); });
        child.on('error', (error) => {
          results.push({ node, success: false, output: redact(error.message, secretValues) });
          resolve();
        });
        child.on('close', (code) => {
          const success = code === 0;
          results.push({ node, success, output: redact(output, secretValues) });
          if (!success) emit(run, 'log', `[失败] ${node.label}（${node.id}）\n${redact(output, secretValues).trim()}\n`);
          resolve();
        });
      } catch (error) {
        results.push({ node, success: false, output: redact(error.message, secretValues) });
        resolve();
      }
    }));
    Promise.all(jobs).then(() => {
      const failed = results.filter((item) => !item.success);
      const succeeded = results.length - failed.length;
      const failedLabels = failed.map((item) => item.node.label).join('、') || '无';
      emit(run, 'log', `[汇总] 成功 ${succeeded}，失败 ${failed.length}，跳过 ${skipped}。失败节点：${failedLabels}`);
      finishRun(run, failed.length === 0 ? 0 : 1);
    });
  } catch (error) {
    emit(run, 'log', redact(error.message));
    finishRun(run, 1);
  }
  return run;
}

function startRun(module, action, options = {}) {
  if (activeRunId) throw new Error('已有部署任务正在运行');
  if (module.id === 'citizenchain' && action.id === 'deploy-all') return startBatchRun(module, action);
  const run = createRun(module, action);

  const environment = action.mode === 'production' ? 'production' : 'staging';
  const secretValues = [];
  // 中文注释：launchd 的 PATH 很精简，动作脚本必须复用当前控制台的 Node 工具链绝对路径。
  const childEnv = baseChildEnv();
  try {
    if (action.production) {
      emit(run, 'log', '正在请求 Touch ID 指纹授权…');
      authorizeProduction('任何生产命令');
      emit(run, 'log', 'Touch ID 验证通过。');
    }
    const requiredKeychain = action.keychain ?? module.secrets.keychain;
    for (const secretName of requiredKeychain) {
      const value = keychainGet(environment, secretName);
      childEnv[secretName] = value;
      secretValues.push(value);
    }
    if (module.id === 'citizenchain' && action.id === 'deploy') {
      const node = chainNodes.find((item) => item.id === options.nodeId);
      if (!node) throw new Error('必须选择一个有效的权威引导节点');
      const serverIp = keychainGet(node.id, 'SERVER_IP');
      const bootnodeKey = keychainGet(node.id, 'BOOTNODE_KEY');
      const validatorKey = keychainGet(node.id, 'VALIDATOR_KEY');
      const sshKey = keychainGet(node.id, 'SSH_KEY');
      Object.assign(childEnv, {
        GMB_NODE_ID: node.id,
        GMB_NODE_LABEL: node.label,
        GMB_NODE_IP: serverIp,
        GMB_NODE_PEER_ID: node.peerId,
        GMB_NODE_GRANDPA_PUBKEY: node.grandpaPubkeyHex,
        GMB_NODE_BOOTNODE_KEY: bootnodeKey,
        GMB_NODE_VALIDATOR_KEY: validatorKey,
        GMB_NODE_SSH_KEY: sshKey,
      });
      secretValues.push(bootnodeKey, validatorKey, sshKey);
      emit(run, 'log', `[步骤 1] 已选择 ${node.number} 号节点：${node.label}（${serverIp}）`);
    }
  } catch (error) {
    emit(run, 'log', redact(error.message));
    finishRun(run, 1);
    return run;
  }

  // 中文注释：仅供本地验收使用；正式启动默认关闭，绝不会伪装成真实部署结果。
  if (process.env.GMB_DEPLOY_DRY_RUN === '1' && !action.localOnly) {
    emit(run, 'log', `验收模式：已通过 ${module.title} / ${action.title} 的全部前置门禁，未执行远端命令。`);
    finishRun(run, 0);
    return run;
  }

  run.state = 'running';
  const script = join(deployDir, 'actions', `${module.id}.sh`);
  const child = spawn('bash', [script, action.mode], { cwd: rootDir, env: childEnv });
  const onData = (chunk) => emit(run, 'log', redact(chunk.toString(), secretValues));
  child.stdout.on('data', onData);
  child.stderr.on('data', onData);
  child.on('error', (error) => { emit(run, 'log', redact(error.message)); finishRun(run, 1); });
  child.on('close', (code) => finishRun(run, code ?? 1));
  return run;
}

async function readBody(req) {
  const chunks = [];
  for await (const chunk of req) chunks.push(chunk);
  return JSON.parse(Buffer.concat(chunks).toString('utf8') || '{}');
}

async function serveStatic(pathname, res) {
  const files = { '/': ['index.html', 'text/html'], '/app.js': ['app.js', 'text/javascript'], '/styles.css': ['styles.css', 'text/css'] };
  const entry = files[pathname];
  if (!entry) return false;
  const body = await readFile(join(deployDir, 'web', entry[0]));
  res.writeHead(200, { 'content-type': `${entry[1]}; charset=utf-8`, 'cache-control': 'no-store' });
  res.end(body);
  return true;
}

const server = createServer(async (req, res) => {
  const url = new URL(req.url, `http://${host}:${port}`);
  try {
    if (req.method === 'GET' && url.pathname === '/') {
      res.setHeader('set-cookie', `gmb_deploy=${sessionToken}; HttpOnly; SameSite=Strict; Path=/`);
      await serveStatic('/', res);
      return;
    }
    if (!hasSession(req)) return json(res, 403, { error: '无效本机会话' });
    if (req.method === 'GET' && await serveStatic(url.pathname, res)) return;
    if (req.method === 'GET' && url.pathname === '/api/catalog') return json(res, 200, { modules, secretComments });
    if (req.method === 'GET' && url.pathname === '/api/status') {
      const github = githubSecretNames();
      const statusModules = modules.map((module) => ({
        id: module.id,
        keychain: Object.fromEntries(['staging', 'production'].map((environment) => [
          environment,
          Object.fromEntries(module.secrets.keychain.map((name) => [name, keychainExists(environment, name)])),
        ])),
        github: Object.fromEntries(module.secrets.github.map((name) => [name, github.has(name)])),
      }));
      const latestRun = [...runs.values()].at(-1);
      return json(res, 200, {
        modules: statusModules,
        activeRunId,
        latestRun: runSnapshot(latestRun),
        citizenwebLocalRunning: citizenwebLocalRunning(),
      });
    }
    if (req.method === 'POST' && url.pathname === '/api/run') {
      if (!validOrigin(req)) return json(res, 403, { error: '来源校验失败' });
      const body = await readBody(req);
      const module = modules.find((item) => item.id === body.moduleId);
      const action = module?.actions.find((item) => item.id === body.actionId);
      if (!module || !action) return json(res, 404, { error: '部署动作不存在' });
      const run = startRun(module, action, body);
      return json(res, 202, { runId: run.id });
    }
    if (req.method === 'GET' && url.pathname === '/api/chain-nodes') {
      return json(res, 200, { nodes: chainNodes.map(nodeStatus) });
    }
    const nodeConfigMatch = url.pathname.match(/^\/api\/chain-nodes\/(node-[0-9]{2})\/config$/);
    if (req.method === 'POST' && nodeConfigMatch) {
      if (!validOrigin(req)) return json(res, 403, { error: '来源校验失败' });
      const node = chainNodes.find((item) => item.id === nodeConfigMatch[1]);
      if (!node) return json(res, 404, { error: '节点不存在' });
      const body = await readBody(req);
      const writes = validateNodeConfiguration(node, body);
      authorizeProduction('节点配置变更');
      replaceNodeConfiguration(node, writes);
      return json(res, 200, { node: nodeStatus(node) });
    }
    const eventMatch = url.pathname.match(/^\/api\/runs\/([^/]+)\/events$/);
    if (req.method === 'GET' && eventMatch) {
      const run = runs.get(eventMatch[1]);
      if (!run) return json(res, 404, { error: '任务不存在' });
      res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-store', connection: 'keep-alive' });
      const send = (item) => res.write(`event: ${item.event}\ndata: ${JSON.stringify(item)}\n\n`);
      run.events.forEach(send);
      run.listeners.add(send);
      req.on('close', () => run.listeners.delete(send));
      return;
    }
    json(res, 404, { error: '页面不存在' });
  } catch (error) {
    json(res, 500, { error: redact(error.message) });
  }
});

let idleTimer = null;
function armIdleExit() {
  if (process.env.GMB_DEPLOY_LAUNCHD !== '1') return;
  if (idleTimer) clearTimeout(idleTimer);
  idleTimer = setTimeout(() => {
    if (activeRunId) return armIdleExit();
    server.close(() => process.exit(0));
  }, idleExitMs);
  idleTimer.unref();
}
server.on('request', armIdleExit);

const listenOptions = inheritedListenFd === null ? { port, host } : { fd: inheritedListenFd };
server.listen(listenOptions, () => {
  console.log(`GMB 本地部署控制台：http://${host}:${port}`);
  armIdleExit();
  // 中文注释：浏览器已经触发按需启动时不重复打开新标签；手动启动仍自动打开控制台。
  if (process.env.GMB_DEPLOY_LAUNCHD !== '1') {
    spawn('open', [`http://${host}:${port}`], { stdio: 'ignore', detached: true }).unref();
  }
});
