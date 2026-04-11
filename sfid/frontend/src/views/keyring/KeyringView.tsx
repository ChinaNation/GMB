// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 3)
// 密钥管理顶层视图 —— 调度器:持有所有状态和副作用,
// 按职责分派到 KeyringRotatePanel。
//
// 本步骤迁移内容:
//   - state: keyringState / keyringLoading / keyringActionLoading / keyringChallenge /
//     keyringSignedPayload / keyringScannerActive / keyringScannerReady /
//     keyringScanSubmitting / keyringCommitLoading / keyringScanAccountOpen /
//     mainAccountBalance / mainAccountBalanceError
//   - ref: keyringVideoRef / keyringScanCleanupRef(整体迁出 App.tsx)
//   - handler: refreshKeyringState / stopKeyringScanner /
//     onCreateKeyringRotateChallenge / onCompleteKeyringRotate / onToggleKeyringScanner
//   - useEffect: 挂载刷新状态 + 摄像头扫码启动
//
// 摄像头扫码使用 src/utils/cameraScanner.ts 共享工具,
// 组件 unmount 时通过 useEffect cleanup 强制释放 MediaStream。

import { useEffect, useRef, useState } from 'react';
import { Form, message } from 'antd';
import { useAuth } from '../../hooks/useAuth';
import type {
  AdminAuth,
  KeyringRotateChallengeResult,
  KeyringStateResult,
} from '../../api/client';
import {
  commitKeyringRotate,
  createKeyringRotateChallenge,
  getAttestorKeyring,
  getChainBalance,
  verifyKeyringRotateSignature,
} from '../../api/client';
import { decodeSs58 } from '../../utils/ss58';
import { startCameraScanner } from '../../utils/cameraScanner';
import { parseKeyringSignedPayload, type KeyringSignedPayload } from '../../utils/parseSignedPayload';
import { KeyringRotatePanel } from './KeyringRotatePanel';
import type { KeyringSharedState } from './keyringUtils';

// 中文注释:步 4 — parseKeyringSignedPayload 已抽到 utils/parseSignedPayload.ts,统一复用。

export function KeyringView() {
  const { auth, capabilities } = useAuth();
  const [keyringState, setKeyringState] = useState<KeyringStateResult | null>(null);
  const [keyringLoading, setKeyringLoading] = useState(false);
  const [keyringActionLoading, setKeyringActionLoading] = useState(false);
  const [keyringChallenge, setKeyringChallenge] = useState<KeyringRotateChallengeResult | null>(null);
  // 中文注释:keyringSignedPayload 仅作过程态保存,UI 没有直接读它,但保留语义不变。
  const [, setKeyringSignedPayload] = useState<KeyringSignedPayload | null>(null);
  const [keyringScannerActive, setKeyringScannerActive] = useState(false);
  const [keyringScannerReady, setKeyringScannerReady] = useState(false);
  const [keyringScanSubmitting, setKeyringScanSubmitting] = useState(false);
  const [, setKeyringCommitLoading] = useState(false);
  const [mainAccountBalance, setMainAccountBalance] = useState<string | null>(null);
  const [mainAccountBalanceError, setMainAccountBalanceError] = useState<string | null>(null);
  const [keyringScanAccountOpen, setKeyringScanAccountOpen] = useState(false);
  const [keyringForm] = Form.useForm<{ new_backup_name: string; new_backup_pubkey: string }>();
  const keyringVideoRef = useRef<HTMLVideoElement | null>(null);
  const keyringScanCleanupRef = useRef<(() => void) | null>(null);

  const stopKeyringScanner = () => {
    if (keyringScanCleanupRef.current) {
      keyringScanCleanupRef.current();
      keyringScanCleanupRef.current = null;
    }
    setKeyringScannerReady(false);
  };

  const refreshKeyringState = async (currentAuth: AdminAuth) => {
    setKeyringLoading(true);
    try {
      const state = await getAttestorKeyring(currentAuth);
      setKeyringState(state);
      // 拉到主账户后立即查链上余额(每次进入密钥管理页都查一次,不缓存)
      if (state?.main_pubkey) {
        setMainAccountBalance(null);
        setMainAccountBalanceError(null);
        try {
          const bal = await getChainBalance(currentAuth, state.main_pubkey);
          setMainAccountBalance(bal.balance_text);
        } catch (err) {
          setMainAccountBalanceError(err instanceof Error ? err.message : String(err));
        }
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '加载密钥状态失败';
      message.error(msg);
    } finally {
      setKeyringLoading(false);
    }
  };

  const onCreateKeyringRotateChallenge = async (values: { new_backup_name: string; new_backup_pubkey: string }) => {
    if (!auth) return;
    void values;
    // 主公钥不能发起轮换
    if (
      keyringState &&
      auth.admin_pubkey.replace(/^0x/i, '').toLowerCase() ===
        keyringState.main_pubkey.replace(/^0x/i, '').toLowerCase()
    ) {
      message.error('主密钥不能发起轮换,请用备用密钥登录');
      return;
    }
    setKeyringActionLoading(true);
    try {
      const challenge = await createKeyringRotateChallenge(auth, {
        initiator_pubkey: auth.admin_pubkey,
      });
      setKeyringChallenge(challenge);
      setKeyringSignedPayload(null);
      setKeyringScannerActive(false);
      stopKeyringScanner();
      message.success('轮换签名二维码已生成,请用备用私钥钱包扫码签名');
    } catch (err) {
      const msg = err instanceof Error ? err.message : '生成轮换挑战失败';
      message.error(msg);
      setKeyringChallenge(null);
    } finally {
      setKeyringActionLoading(false);
    }
  };

  const onCompleteKeyringRotate = async (raw: string) => {
    if (!auth || !keyringChallenge) {
      message.error('请先生成轮换二维码');
      return;
    }
    const newBackupAddress = keyringForm.getFieldValue('new_backup_pubkey')?.trim();
    if (!newBackupAddress) {
      message.error('新备用账户不能为空');
      return;
    }
    let newBackupPubkey: string;
    try {
      newBackupPubkey = decodeSs58(newBackupAddress);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '账户格式无效');
      return;
    }
    setKeyringScanSubmitting(true);
    try {
      const payload = parseKeyringSignedPayload(raw, keyringChallenge.challenge_id);
      await verifyKeyringRotateSignature(auth, {
        challenge_id: payload.challenge_id,
        signature: payload.signature,
      });
      setKeyringSignedPayload(payload);
      setKeyringScannerActive(false);
      stopKeyringScanner();
      message.success('签名校验通过,正在提交轮换...');
      setKeyringCommitLoading(true);
      try {
        const newBackupName = keyringForm.getFieldValue('new_backup_name')?.trim();
        const result = await commitKeyringRotate(auth, {
          challenge_id: payload.challenge_id,
          signature: payload.signature,
          new_backup_pubkey: newBackupPubkey,
          new_backup_name: newBackupName || undefined,
        });
        if (result.chain_submit_ok) {
          message.success(`主密钥轮换成功,新版本:${result.version}`);
        } else {
          message.warning(
            `主密钥已本地轮换为版本 ${result.version},但上链提交失败:${result.chain_submit_error || '未知错误'}`,
          );
        }
        setKeyringChallenge(null);
        setKeyringSignedPayload(null);
        keyringForm.resetFields();
        await refreshKeyringState(auth);
      } catch (commitErr) {
        const commitMsg = commitErr instanceof Error ? commitErr.message : '提交轮换失败';
        message.error(commitMsg);
      } finally {
        setKeyringCommitLoading(false);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '提交轮换签名失败';
      message.error(msg);
    } finally {
      setKeyringScanSubmitting(false);
    }
  };

  const onToggleKeyringScanner = () => {
    if (!keyringChallenge) {
      message.warning('请先生成轮换二维码');
      return;
    }
    setKeyringScannerActive((v) => !v);
  };

  // 挂载时拉一次密钥状态;auth 切换也重新拉
  useEffect(() => {
    if (!auth) return;
    void refreshKeyringState(auth);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth?.access_token]);

  // 摄像头扫码 effect:迁自 App.tsx,unmount 时强制释放
  useEffect(() => {
    if (!keyringScannerActive || !keyringChallenge || !keyringVideoRef.current) {
      stopKeyringScanner();
      return;
    }
    keyringScanCleanupRef.current = startCameraScanner(
      keyringVideoRef.current,
      (raw) => {
        setKeyringScannerActive(false);
        stopKeyringScanner();
        void onCompleteKeyringRotate(raw);
      },
      () => setKeyringScannerReady(true),
      (msg) => message.error(msg),
    );
    return () => stopKeyringScanner();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [keyringScannerActive, keyringChallenge]);

  // 组件 unmount 时兜底清理摄像头(防止 view 切走时摄像头还在跑)
  useEffect(() => {
    return () => {
      if (keyringScanCleanupRef.current) {
        keyringScanCleanupRef.current();
        keyringScanCleanupRef.current = null;
      }
    };
  }, []);

  if (!capabilities.canManageKeyring) {
    return null;
  }

  // 主密钥登录时,一切轮换相关控件(输入框/按钮/扫码图标)都禁用
  const isMainKeySigned = Boolean(
    keyringState &&
      auth &&
      auth.admin_pubkey.replace(/^0x/i, '').toLowerCase() ===
        keyringState.main_pubkey.replace(/^0x/i, '').toLowerCase(),
  );

  // ── 组装共享状态 ──
  const shared: KeyringSharedState = {
    auth,
    keyringState,
    keyringLoading,
    keyringActionLoading,
    keyringChallenge,
    keyringScannerActive,
    keyringScannerReady,
    keyringScanSubmitting,
    keyringScanAccountOpen,
    setKeyringScanAccountOpen,
    mainAccountBalance,
    mainAccountBalanceError,
    isMainKeySigned,
    keyringForm,
    keyringVideoRef,
    onRefresh: () => {
      if (auth) {
        void refreshKeyringState(auth);
      }
    },
    onCreateRotateChallenge: onCreateKeyringRotateChallenge,
    onToggleScanner: onToggleKeyringScanner,
  };

  return <KeyringRotatePanel state={shared} />;
}
