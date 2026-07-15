let catalog = [];
let secretComments = {};
let citizenwebLocalRunning = false;
let statusById = new Map();
let chainNodes = [];
let displayedRunId = null;
let runEvents = null;
const cards = document.querySelector('#cards');
const overview = document.querySelector('#overview');
const dialog = document.querySelector('#moduleDialog');
const detail = document.querySelector('#moduleDetail');
const logs = document.querySelector('#logs');

function configured(values) {
  const entries = Object.values(values || {});
  return entries.length === 0 || entries.every(Boolean);
}

function statusPill(ok, text) {
  return `<span class="pill ${ok ? 'ok' : 'missing'}">${ok ? '●' : '○'} ${text}</span>`;
}

function escapeHtml(value) {
  return String(value ?? '').replace(/[&<>'"]/g, (character) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' })[character]);
}

function nodeReady(node) {
  return ['SERVER_IP', 'BOOTNODE_KEY', 'VALIDATOR_KEY', 'SSH_KEY'].every((name) => node.configured[name]);
}

const nodeFieldLabels = {
  SERVER_IP: '服务器IP',
  BOOTNODE_KEY: '引导节点密钥',
  VALIDATOR_KEY: '验证节点密钥',
  SSH_KEY: 'SSH私钥',
};

function missingNodeFields(node) {
  // 中文注释：可部署状态必须与后端真实所需四项配置一致，不用绿色按钮误导用户。
  return Object.entries(nodeFieldLabels)
    .filter(([name]) => !node.configured[name])
    .map(([, label]) => label);
}

function renderChainNodes() {
  return `<section id="chainNodeManager" class="node-manager"><div class="node-manager-head"><div><h3>44 个权威引导节点</h3><p>选择节点、维护 IP 与节点密钥；保存配置和部署均需要 Touch ID。</p></div><span>${chainNodes.filter(nodeReady).length}/44 就绪</span></div>
    <div class="node-list">${chainNodes.map((node) => {
      const ready = nodeReady(node);
      const missing = missingNodeFields(node);
      return `<details class="node-item ${ready ? 'ready' : ''}" data-node="${node.id}">
      <summary><span><strong>${String(node.number).padStart(2, '0')} · ${escapeHtml(node.label)}</strong><small>${escapeHtml(node.domain)}</small>${ready ? '' : `<small class="node-missing">缺少：${escapeHtml(missing.join('、'))}</small>`}</span><span class="node-summary-status">${statusPill(ready, ready ? '可部署' : '待配置')}</span></summary>
      <div class="node-form">
        <label>服务器 IP<input data-field="serverIp" data-original="${escapeHtml(node.serverIp || '')}" value="${escapeHtml(node.serverIp || '')}" placeholder="请输入该节点服务器 IP" inputmode="decimal" /></label>
        <label>引导节点密钥<input data-field="bootnodeKey" type="password" autocomplete="new-password" placeholder="${node.configured.BOOTNODE_KEY ? '已安全保存；留空则不修改' : '请输入64位十六进制私钥'}" /></label>
        <label>验证节点密钥<input data-field="validatorKey" type="password" autocomplete="new-password" placeholder="${node.configured.VALIDATOR_KEY ? '已安全保存；留空则不修改' : '请输入 GRANDPA 私钥'}" /></label>
        <label class="node-ssh">服务器 SSH 私钥<textarea data-field="sshKey" rows="3" autocomplete="off" placeholder="${node.configured.SSH_KEY ? '已安全保存；留空则不修改' : '请输入该服务器 SSH 私钥'}"></textarea></label>
        <div class="node-public"><span>PeerId：${escapeHtml(node.peerId)}</span><span>GRANDPA 公钥：${escapeHtml(node.grandpaPubkeyHex)}</span></div>
        <p class="node-message" role="status"></p>
        <div class="node-buttons"><button type="button" class="test" data-save-node="${node.id}">保存已更改项<small>可单项或多项 · 需要 Touch ID</small></button><button type="button" class="deploy-node ${ready ? 'deploy-ready' : 'deploy-disabled'}" data-deploy-node="${node.id}" ${ready ? '' : 'disabled'}>部署该节点<small>${ready ? '配置齐全 · 需要 Touch ID' : `缺少${missing.length}项配置`}</small></button></div>
      </div>
    </details>`; }).join('')}</div></section>`;
}

function renderCards() {
  cards.innerHTML = catalog.map((module) => {
    const status = statusById.get(module.id) || { keychain: {}, github: {} };
    const testOk = configured(status.keychain.staging) && configured(status.github);
    const productionOk = configured(status.keychain.production) && configured(status.github);
    return `<button class="card" data-module="${module.id}">
      <span class="icon">${module.icon}</span>
      <span class="card-copy"><strong>${module.title}</strong><small>${module.description}</small></span>
      <span class="status-row">${statusPill(testOk, '测试')}${statusPill(productionOk, '生产')}</span>
    </button>`;
  }).join('');
  document.querySelectorAll('[data-module]').forEach((button) => {
    button.addEventListener('click', () => openModule(button.dataset.module));
  });
}

function openModule(moduleId) {
  const module = catalog.find((item) => item.id === moduleId);
  const status = statusById.get(moduleId) || { keychain: {}, github: {} };
  const keyRows = ['staging', 'production'].flatMap((environment) =>
    Object.entries(status.keychain[environment] || {}).map(([name, ok]) =>
      `<li><span class="secret-copy"><code>${environment}:${name}</code><small>${secretComments[name] || '部署密钥'}</small></span>${statusPill(ok, ok ? '已配置' : '缺失')}</li>`));
  const githubRows = Object.entries(status.github || {}).map(([name, ok]) =>
    `<li><span class="secret-copy"><code>GitHub:${name}</code><small>${secretComments[name] || 'GitHub 部署密钥'}</small></span>${statusPill(ok, ok ? '已配置' : '缺失')}</li>`);
  const localSite = module.id === 'citizenweb'
    ? `<p class="local-site">本地测试网站 ${statusPill(citizenwebLocalRunning, citizenwebLocalRunning ? '运行中 · 127.0.0.1:41732' : '已关闭')}</p>`
    : '';
  const chainNodeManager = module.id === 'citizenchain' ? renderChainNodes() : '';
  detail.innerHTML = `<div class="detail-title"><span class="icon">${module.icon}</span><div><h2>${module.title}</h2><p>${module.description}</p></div></div>
    ${localSite}<h3>可执行操作</h3><div class="actions ${module.id === 'citizenchain' ? 'actions-three' : ''}">${module.actions.map((action) =>
      `<button data-action="${action.id}" class="${action.production ? 'production' : 'test'}">${action.title}${action.production ? '<small>需要 Touch ID</small>' : '<small>无需密码</small>'}</button>`).join('')}</div>
    ${chainNodeManager}<h3>密钥状态</h3><ul class="secret-list">${[...keyRows, ...githubRows].join('') || '<li>此操作不需要部署密钥</li>'}</ul>`;
  detail.querySelectorAll('[data-action]').forEach((button) => {
    button.addEventListener('click', () => {
      if (module.id === 'citizenchain' && button.dataset.action === 'deploy') {
        detail.querySelector('#chainNodeManager').scrollIntoView({ behavior: 'smooth', block: 'start' });
        return;
      }
      runAction(module.id, button.dataset.action);
    });
  });
  detail.querySelectorAll('[data-save-node]').forEach((button) => button.addEventListener('click', () => saveNode(button.dataset.saveNode)));
  detail.querySelectorAll('[data-deploy-node]').forEach((button) => button.addEventListener('click', () => runAction('citizenchain', 'deploy', { nodeId: button.dataset.deployNode })));
  dialog.showModal();
}

async function saveNode(nodeId) {
  const item = detail.querySelector(`[data-node="${nodeId}"]`);
  const message = item.querySelector('.node-message');
  message.className = 'node-message';
  message.textContent = '正在校验配置…';
  // 中文注释：IP 只在发生变化时提交，私密字段只提交已填写项；留空不会覆盖 Keychain 旧值。
  const payload = {};
  for (const field of item.querySelectorAll('[data-field]')) {
    const value = field.value;
    if (field.dataset.field === 'serverIp') {
      if (value.trim() !== String(field.dataset.original || '').trim()) payload.serverIp = value;
    } else if (value.trim()) {
      payload[field.dataset.field] = value;
    }
  }
  if (Object.keys(payload).length === 0) {
    message.className = 'node-message error';
    message.textContent = '没有已更改或已填写的配置项';
    logs.textContent = `${nodeId}：未提交任何变更，未请求 Touch ID。\n`;
    return;
  }
  logs.textContent = `正在校验 ${nodeId} 的${Object.keys(payload).length}项配置；通过后请求 Touch ID…\n`;
  const response = await fetch(`/api/chain-nodes/${nodeId}/config`, {
    method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(payload),
  });
  const result = await response.json();
  if (!response.ok) {
    message.className = 'node-message error';
    message.textContent = result.error;
    item.querySelectorAll('[data-field]:not([data-field="serverIp"])').forEach((field) => { field.value = ''; });
    logs.textContent += `${nodeId}：${result.error}\n`;
    return;
  }
  logs.textContent += `已通过 Touch ID，${Object.keys(payload).length}项节点配置已独立写入 macOS Keychain，未提交项保持不变。\n`;
  await loadStatus();
  dialog.close();
  openModule('citizenchain');
}

async function runAction(moduleId, actionId, options = {}) {
  const response = await fetch('/api/run', {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ moduleId, actionId, ...options }),
  });
  const result = await response.json();
  if (!response.ok) {
    // 中文注释：启动冲突不能覆盖正在执行或最近一次任务的验收证据。
    logs.textContent += `\n${result.error}\n`;
    return;
  }
  dialog.close();
  logs.textContent = '';
  displayedRunId = result.runId;
  connectRunEvents(result.runId);
}

function connectRunEvents(runId) {
  if (runEvents) runEvents.close();
  runEvents = new EventSource(`/api/runs/${runId}/events`);
  runEvents.addEventListener('log', (event) => {
    const item = JSON.parse(event.data);
    logs.textContent += item.data;
    if (!String(item.data).endsWith('\n')) logs.textContent += '\n';
    logs.scrollTop = logs.scrollHeight;
  });
  runEvents.addEventListener('done', async (event) => {
    const item = JSON.parse(event.data);
    logs.textContent += `\n任务结束：${item.data.state}\n`;
    runEvents.close();
    runEvents = null;
    await loadStatus();
  });
}

async function loadStatus() {
  const [catalogResponse, statusResponse, chainNodesResponse] = await Promise.all([fetch('/api/catalog'), fetch('/api/status'), fetch('/api/chain-nodes')]);
  const catalogResult = await catalogResponse.json();
  catalog = catalogResult.modules;
  secretComments = catalogResult.secretComments;
  const status = await statusResponse.json();
  const chainNodesResult = await chainNodesResponse.json();
  chainNodes = chainNodesResult.nodes;
  citizenwebLocalRunning = status.citizenwebLocalRunning;
  statusById = new Map(status.modules.map((item) => [item.id, item]));
  const ready = status.modules.filter((item) => configured(item.keychain.production) && configured(item.github)).length;
  overview.innerHTML = `<article><strong>${catalog.length}</strong><span>部署模块</span></article><article><strong>${ready}</strong><span>生产就绪</span></article><article><strong>${status.activeRunId ? '运行中' : '空闲'}</strong><span>执行状态</span></article>`;
  if (status.latestRun) {
    const running = status.latestRun.state === 'running' || status.latestRun.state === 'starting';
    if (running && status.latestRun.id !== displayedRunId) {
      displayedRunId = status.latestRun.id;
      // 中文注释：SSE 会重放当前任务的完整事件，先清空可避免恢复连接后出现重复日志。
      logs.textContent = '';
      connectRunEvents(status.latestRun.id);
    } else if (!running) {
      // 中文注释：标签交接可能中断 SSE；已结束任务每次刷新都以服务端完整事件重建结果。
      displayedRunId = status.latestRun.id;
      if (runEvents) runEvents.close();
      runEvents = null;
      logs.textContent = status.latestRun.events
        .filter((item) => item.event === 'log')
        .map((item) => item.data)
        .join('\n');
      logs.textContent += `\n\n任务结束：${status.latestRun.state}\n`;
    }
  }
  renderCards();
}

document.querySelector('#closeDialog').addEventListener('click', () => dialog.close());
document.querySelector('#refresh').addEventListener('click', loadStatus);
loadStatus().catch((error) => { logs.textContent = `控制台加载失败：${error.message}`; });
