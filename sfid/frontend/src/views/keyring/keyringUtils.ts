// 密钥管理视图共享工具函数与类型

import type { FormInstance } from 'antd';
import type {
  AdminAuth,
  KeyringRotateChallengeResult,
  KeyringStateResult,
} from '../../api/client';

/** 所有子视图共享的状态与回调 */
export interface KeyringSharedState {
  auth: AdminAuth | null;

  keyringState: KeyringStateResult | null;
  keyringLoading: boolean;
  keyringActionLoading: boolean;
  keyringChallenge: KeyringRotateChallengeResult | null;

  keyringScannerActive: boolean;
  keyringScannerReady: boolean;
  keyringScanSubmitting: boolean;

  keyringScanAccountOpen: boolean;
  setKeyringScanAccountOpen: (v: boolean) => void;

  mainAccountBalance: string | null;
  mainAccountBalanceError: string | null;

  /** 主密钥登录时禁用轮换控件 */
  isMainKeySigned: boolean;

  keyringForm: FormInstance<{ new_backup_pubkey: string }>;
  keyringVideoRef: React.MutableRefObject<HTMLVideoElement | null>;

  onRefresh: () => void;
  onCreateRotateChallenge: (values: { new_backup_pubkey: string }) => void;
  onToggleScanner: () => void;
}
