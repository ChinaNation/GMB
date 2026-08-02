import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

// 跨端契约锁。
//
// 背景(真实事故):Worker 与 Flutter 各自的单测都只对齐**自己这一侧**的字段名,
// 于是 `device_public_key`(客户端) vs `device_public_key_hex`(Worker)这种键名
// 漂移两边测试全绿、线上 100% 400。本文件直接读 Flutter 源码文本做断言,把
// 两端共享的 JSON 键名钉死在一处,任一侧改名而另一侧没跟,这里必红。
//
// 只锁**跨端 JSON 键名**,不锁实现细节;新增跨端字段时在此补一条。

const FLUTTER_ROOT = join(import.meta.dirname, '../..');

function readFlutter(relativePath: string): string {
  return readFileSync(join(FLUTTER_ROOT, relativePath), 'utf8');
}

describe('跨端 JSON 契约(Worker ⇔ Flutter 键名一致)', () => {
  const transport = readFlutter('lib/chat/transport/chat_cloud_transport.dart');
  const workerChat = readFileSync(
    join(import.meta.dirname, '../src/chat/service.ts'),
    'utf8',
  );

  it('设备注册 / KeyPackage 发布用同一个设备公钥键 device_public_key_hex', () => {
    // Worker 是 D1 列名真源(chat_devices.device_public_key_hex)。
    expect(workerChat).toContain('body.device_public_key_hex');
    expect(transport).toContain("'device_public_key_hex'");
    // 旧键名不得复活(客户端曾发 device_public_key 导致注册全 400)。
    expect(transport).not.toContain("'device_public_key':");
  });

  it('聊天收件人按身份主键 recipient_cid_number 寻址', () => {
    expect(workerChat).toContain('recipient_cid_number');
    expect(transport).toContain("'recipient_cid_number'");
    // 收件人绝不再用钱包账户寻址(proto envelope 内嵌的 account 归属字段不在此列)。
    expect(transport).not.toContain("'recipient_account_id':");
  });

  it('KeyPackage 发布 / 领取按身份主键 cid_number', () => {
    expect(workerChat).toContain('body.cid_number');
    expect(transport).toContain("'cid_number'");
    // 领取不再传 requester(会话即领取者)。
    expect(transport).not.toContain('requester_account_id');
    expect(workerChat).not.toContain('requester_account_id');
  });

  it('推送唤醒发件人按 sender_cid_number', () => {
    const workerPush = readFileSync(
      join(import.meta.dirname, '../src/chat/push.ts'),
      'utf8',
    );
    const flutterPush = readFlutter('lib/chat/chat_push_service.dart');
    expect(workerPush).toContain('sender_cid_number');
    expect(flutterPush).toContain("'sender_cid_number'");
  });

  it('关注/取关按 followed_cid_number,关注列表响应用 entries', () => {
    const workerFollows = readFileSync(
      join(import.meta.dirname, '../src/feeds/follows.ts'),
      'utf8',
    );
    const workerProfiles = readFileSync(
      join(import.meta.dirname, '../src/profiles/service.ts'),
      'utf8',
    );
    const api = readFlutter('lib/8964/services/square_api_client.dart');
    expect(workerFollows).toContain('followed_cid_number');
    expect(api).toContain("'followed_cid_number'");
    expect(workerProfiles).toContain('entries');
    expect(api).toContain("data['entries']");
    expect(api).not.toContain("data['accounts']");
  });

  it('登录响应下发身份主键 cid_number,客户端必解析', () => {
    const workerAuth = readFileSync(
      join(import.meta.dirname, '../src/auth/service.ts'),
      'utf8',
    );
    const api = readFlutter('lib/8964/services/square_api_client.dart');
    expect(workerAuth).toContain('cid_number: cidNumber');
    expect(api).toContain("session['cid_number']");
  });
});

describe('生产 API 路径契约(Worker ⇔ Flutter 无版本路由一致)', () => {
  const squareApi = readFlutter('lib/8964/services/square_api_client.dart');
  const creatorApi = readFlutter('lib/my/creator/creator_api.dart');
  const chatTransport = readFlutter('lib/chat/transport/chat_cloud_transport.dart');
  const workerRoutes = readFileSync(
    join(import.meta.dirname, '../src/routes.ts'),
    'utf8',
  );
  const routeCatalog = readFileSync(
    join(import.meta.dirname, '../src/limits/catalog.ts'),
    'utf8',
  );

  it('App 只使用同域 /api 部署根，业务路径不携带版本段', () => {
    expect(squareApi).toContain("prodBaseUrl = 'https://www.crcfrcn.com/api'");
    expect(squareApi).toContain("'/square/membership'");
    expect(squareApi).toContain("'/square/contacts?");
    expect(creatorApi).toContain("'/square/creator/plan'");
    expect(chatTransport).toContain("'/chat/devices/register'");
    expect(`${squareApi}\n${creatorApi}\n${chatTransport}`).not.toMatch(/['"]\/v\d+\//);
  });

  it('Worker 路由分发与资源白名单只登记无版本业务路径', () => {
    expect(workerRoutes).toContain('path === "/chain/bootstrap"');
    expect(workerRoutes).toContain('path === "/square/creator/plan"');
    expect(routeCatalog).toContain('^\\/square\\/contacts$');
    expect(routeCatalog).toContain('^\\/chat\\/devices\\/register$');
    expect(`${workerRoutes}\n${routeCatalog}`).not.toMatch(/['"]\/v\d+\//);
  });
});

describe('链上 storage 项名锁(Worker ⇔ citizenchain pallet)', () => {
  // 真实事故:citizenchain 把 WalletAccountByCid/CidByWalletAccount 改名为
  // AccountIdByCid/CidByAccountId,Flutter 跟了、Worker 没跟 —— storage key 拼错
  // 后 state_getStorage 返回 null(不是报错),表现为"所有人都未绑定 CID",
  // 登录/鉴权/主页全线失效且软降级掩盖。此处把名字钉死在 pallet 源码上。
  const identity = readFileSync(
    join(import.meta.dirname, '../src/chain/identity.ts'),
    'utf8',
  );
  const palletPath = join(
    FLUTTER_ROOT,
    '../citizenchain/runtime/misc/citizen-identity/src/lib.rs',
  );

  it('Worker 用的 storage 项名必须存在于 citizen-identity pallet', () => {
    const pallet = readFileSync(palletPath, 'utf8');
    for (const storageName of [
      'AccountIdByCid',
      'CidByAccountId',
      'CidRegistry',
      'VotingIdentityByCid',
      'CandidateIdentityByCid',
    ]) {
      expect(identity).toContain(`"${storageName}"`);
      expect(pallet).toContain(`pub type ${storageName}`);
    }
    // 改名前的旧项名不得复活。
    expect(identity).not.toContain('WalletAccountByCid');
    expect(identity).not.toContain('CidByWalletAccount');
  });
});
