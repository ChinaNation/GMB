import { invoke } from '../../core/tauri';
import type {
  ActivateRequestResult,
  ActivatedAdmin,
  AdminSubjectState,
  InstitutionDetail,
  VoteSignRequestResult,
  VoteSubmitResult,
} from './types';

// 管理员更换模块前端 API。激活、更换与主体读取统一聚合到 admins_change。
export const adminsChangeApi = {
  getInstitutionDetail: (sfidNumber: string) =>
    invoke<InstitutionDetail>('get_institution_detail', { sfidNumber }),
  buildActivateAdminRequest: (pubkeyHex: string, sfidNumber: string) =>
    invoke<ActivateRequestResult>('build_activate_admin_request', { pubkeyHex, sfidNumber }),
  verifyActivateAdmin: (
    requestId: string,
    pubkeyHex: string,
    expectedPayloadHash: string,
    payloadHex: string,
    responseJson: string,
  ) =>
    invoke<ActivatedAdmin>('verify_activate_admin', {
      requestId,
      pubkeyHex,
      expectedPayloadHash,
      payloadHex,
      responseJson,
    }),
  getActivatedAdmins: (sfidNumber: string) =>
    invoke<ActivatedAdmin[]>('get_activated_admins', { sfidNumber }),
  deactivateAdmin: (pubkeyHex: string, sfidNumber: string, unlockPassword: string) =>
    invoke<void>('deactivate_admin', { pubkeyHex, sfidNumber, unlockPassword }),
  hasAnyActivatedAdmin: () => invoke<boolean>('has_any_activated_admin'),
  getAdminSubjectState: (sfidNumber: string) =>
    invoke<AdminSubjectState | null>('get_admin_subject_state', { sfidNumber, subjectIdHex: null }),
  buildAdminSetChangeRequest: (pubkeyHex: string, sfidNumber: string, newAdmins: string[]) =>
    invoke<VoteSignRequestResult>('build_admin_set_change_request', {
      pubkeyHex,
      sfidNumber,
      subjectIdHex: null,
      newAdmins,
    }),
  submitAdminSetChange: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    sfidNumber: string,
    newAdmins: string[],
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_admin_set_change', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      sfidNumber,
      subjectIdHex: null,
      newAdmins,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
};
