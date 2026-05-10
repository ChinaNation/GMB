import { invoke } from '../../core/tauri';
import type {
  ActivateRequestResult,
  ActivatedAdmin,
  AdminSubjectRef,
  AdminSubjectState,
  InstitutionDetail,
  VoteSignRequestResult,
  VoteSubmitResult,
} from './types';

// 管理员更换模块前端 API。激活、更换与主体读取统一聚合到 admins_change。
const subjectRefParams = (subjectRef: AdminSubjectRef) => ({
  sfidNumber: subjectRef.sfidNumber ?? null,
  subjectIdHex: subjectRef.subjectIdHex ?? null,
  expectedOrg: subjectRef.org ?? null,
});

export const adminsChangeApi = {
  getInstitutionDetail: (sfidNumber: string) =>
    invoke<InstitutionDetail>('get_institution_detail', { sfidNumber }),
  buildActivateAdminRequest: (pubkeyHex: string, sfidNumber: string, subjectRef?: AdminSubjectRef) =>
    invoke<ActivateRequestResult>('build_activate_admin_request', {
      pubkeyHex,
      sfidNumber,
      subjectIdHex: subjectRef?.subjectIdHex ?? null,
      expectedOrg: subjectRef?.org ?? null,
    }),
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
  getActivatedAdmins: (sfidNumber: string, subjectRef?: AdminSubjectRef) =>
    invoke<ActivatedAdmin[]>('get_activated_admins', {
      sfidNumber,
      subjectIdHex: subjectRef?.subjectIdHex ?? null,
      expectedOrg: subjectRef?.org ?? null,
    }),
  deactivateAdmin: (pubkeyHex: string, sfidNumber: string, unlockPassword: string) =>
    invoke<void>('deactivate_admin', { pubkeyHex, sfidNumber, unlockPassword }),
  hasAnyActivatedAdmin: () => invoke<boolean>('has_any_activated_admin'),
  getAdminSubjectState: (subjectRef: AdminSubjectRef) =>
    invoke<AdminSubjectState | null>('get_admin_subject_state', subjectRefParams(subjectRef)),
  buildAdminSetChangeRequest: (pubkeyHex: string, subjectRef: AdminSubjectRef, newAdmins: string[]) =>
    invoke<VoteSignRequestResult>('build_admin_set_change_request', {
      pubkeyHex,
      ...subjectRefParams(subjectRef),
      newAdmins,
    }),
  submitAdminSetChange: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    subjectRef: AdminSubjectRef,
    newAdmins: string[],
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_admin_set_change', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      ...subjectRefParams(subjectRef),
      newAdmins,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
};
