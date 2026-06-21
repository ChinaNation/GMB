// 中文注释:机构新增弹窗共享表单(私权/公权/教育三入口共用)。
// 各入口只传入本模块 API 函数,不在公共组件里越过业务边界。
//
// 主体属性统一联动(与后端号码生成器/subjects/uninorg 同源):
//   G → 盈利属性锁死非盈利;公权入口建公权机构(ZF/LF/SF/JC)、教育入口建公立学校(JY),名称必填
//   S → 私权类型规则锁定 T2/P1;教育私立学校名称必填
//   F → 个体经营/无限合伙是独立非法人;分校等从属非法人必须选择所属法人,
//       搜索范围由后端按地域规则预过滤(分校→本市学校本部;公权→本市市级/本省省级/国家级)

import React, { useEffect, useMemo, useState } from 'react';
import { AutoComplete, Button, Col, Form, Input, Modal, Row, Select, Spin, Upload } from 'antd';
import { SearchOutlined, UploadOutlined } from '@ant-design/icons';
import type { AdminAuth } from '../../auth/types';
import type { CidCityItem } from '../../china/api';
import { loadCachedCidCities } from '../../china/metaCache';
import type {
  CreateInstitutionInput,
  CreateInstitutionOutput,
  EducationType,
  LegalRepresentativePhoto,
  ParentInstitutionRow,
  PartnershipKind,
  PrivateType,
  SearchParentsOptions,
} from '../../subjects/api';
import { searchLegalRepresentativeCitizens } from '../../citizens/api';
import {
  inheritedP1,
  institutionChoicesFor,
  locksForCategory,
  p1LocksForSubject,
  privateRuleFor,
  PRIVATE_TYPE_LABEL,
  SCHOOL_EDUCATION_TYPE_OPTIONS,
  SUBJECT_PROPERTY_LABEL,
  type CreateFormCategory,
} from '../../subjects/labels';
import { notice } from '../../utils/notice';

interface FormValues {
  subject_property: string;
  p1: string;
  province_name: string;
  city_name: string;
  institution: string;
  education_type?: EducationType;
  private_type?: PrivateType;
  partnership_kind?: PartnershipKind;
  /** 私权/教育机构/手动公权机构创建时必填名称。 */
  cid_full_name?: string;
  /** 需挂靠的非法人必填;个体经营/无限合伙不接受所属法人。 */
  parent_cid_number?: string;
  legal_rep_name: string;
  legal_rep_cid_number: string;
  legal_rep_photo_path: string;
  legal_rep_photo_name: string;
  legal_rep_photo_mime: string;
  legal_rep_photo_size?: number;
}

type CheckInstitutionName = (
  auth: AdminAuth,
  name: string,
  subject_property?: string,
  city_name?: string,
) => Promise<{ exists: boolean }>;

type CreateInstitution = (
  auth: AdminAuth,
  input: CreateInstitutionInput,
) => Promise<CreateInstitutionOutput>;

type UploadLegalRepresentativePhoto = (
  auth: AdminAuth,
  file: File,
) => Promise<LegalRepresentativePhoto>;

type SearchParentInstitutions = (
  auth: AdminAuth,
  q: string,
  opts: SearchParentsOptions,
) => Promise<ParentInstitutionRow[]>;

export interface CreateInstitutionFormProps {
  auth: AdminAuth;
  category: CreateFormCategory;
  privateType?: PrivateType;
  open: boolean;
  lockedProvinceName: string | null;
  lockedCityName: string | null;
  checkInstitutionName: CheckInstitutionName;
  createInstitution: CreateInstitution;
  uploadLegalRepresentativePhoto: UploadLegalRepresentativePhoto;
  searchParentInstitutions: SearchParentInstitutions;
  onCancel: () => void;
  onCreated: (result: CreateInstitutionOutput) => void;
}

export const CreateInstitutionForm: React.FC<CreateInstitutionFormProps> = ({
  auth,
  category,
  privateType,
  open,
  lockedProvinceName,
  lockedCityName,
  checkInstitutionName,
  createInstitution,
  uploadLegalRepresentativePhoto,
  searchParentInstitutions,
  onCancel,
  onCreated,
}) => {
  const locks = locksForCategory(category);
  const [form] = Form.useForm<FormValues>();
  const [cities, setCities] = useState<CidCityItem[]>([]);
  const [citiesLoading, setCitiesLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [nameChecking, setNameChecking] = useState(false);
  const [nameAvailable, setNameAvailable] = useState<boolean | null>(null);
  const [legalRepSearching, setLegalRepSearching] = useState(false);
  const [legalRepOptions, setLegalRepOptions] = useState<string[]>([]);
  const [photoUploading, setPhotoUploading] = useState(false);
  const [photoName, setPhotoName] = useState<string>('');

  const [currentSubjectProperty, setCurrentSubjectProperty] = useState<string>(
    locks.subjectPropertyChoices[0]?.value ?? '',
  );
  // 非法人(F)所属法人选择器状态:必须从搜索结果中选定真实父级
  const [selectedParent, setSelectedParent] = useState<ParentInstitutionRow | null>(null);
  const [parentOptions, setParentOptions] = useState<ParentInstitutionRow[]>([]);
  const [parentSearching, setParentSearching] = useState(false);

  const isPrivate = category === 'PRIVATE_INSTITUTION';
  const isGov = category === 'GOV_INSTITUTION';
  const isEducation = category === 'EDUCATION_INSTITUTION';
  const watchedPartnershipKind = Form.useWatch('partnership_kind', form) as PartnershipKind | undefined;
  const privateRule = isPrivate && privateType
    ? privateRuleFor(privateType, watchedPartnershipKind)
    : null;
  const isF = currentSubjectProperty === 'F';
  const requiresParent = isF && !isPrivate;
  const showEducationType = isEducation && !isF;

  // 中文注释:私权目标态创建阶段直接写入名称;教育学校和手动公权机构也在弹窗内查重。
  const collectNameInModal = isPrivate || isEducation || (isGov && !isF);
  const nameLabel = isEducation ? '学校名称' : '机构名称';

  const subjectPropertyChoices = privateRule
    ? [{
        value: privateRule.subjectProperty,
        label: SUBJECT_PROPERTY_LABEL[privateRule.subjectProperty] ?? privateRule.subjectProperty,
      }]
    : locks.subjectPropertyChoices;
  const instChoices = useMemo(() => {
    if (privateRule) {
      return [{ value: privateRule.institution, label: PRIVATE_TYPE_LABEL[privateRule.privateType] }];
    }
    return institutionChoicesFor(category, currentSubjectProperty);
  }, [category, currentSubjectProperty, privateRule]);
  const p1Locks = useMemo(() => {
    if (privateRule) {
      return {
        choices: [
          privateRule.p1 === '1'
            ? { value: '1', label: '盈利' }
            : { value: '0', label: '非盈利' },
        ],
        value: privateRule.p1,
        locked: true,
      };
    }
    return p1LocksForSubject(currentSubjectProperty, selectedParent);
  }, [currentSubjectProperty, selectedParent, privateRule]);

  const resetParentState = () => {
    setSelectedParent(null);
    setParentOptions([]);
  };

  useEffect(() => {
    if (!open) return;
    const defaultPartnershipKind: PartnershipKind | undefined =
      privateType === 'PARTNERSHIP' ? 'GENERAL' : undefined;
    const defaultRule = privateType ? privateRuleFor(privateType, defaultPartnershipKind) : null;
    const defaultSubjectProperty = defaultRule?.subjectProperty ?? locks.subjectPropertyChoices[0]?.value ?? '';
    setCurrentSubjectProperty(defaultSubjectProperty);
    setNameAvailable(null);
    resetParentState();
    const defaultInstitution = defaultRule?.institution ?? institutionChoicesFor(category, defaultSubjectProperty)[0]?.value;
    const defaultEducationType = category === 'EDUCATION_INSTITUTION' && defaultSubjectProperty !== 'F'
      ? SCHOOL_EDUCATION_TYPE_OPTIONS[0]?.value as EducationType
      : undefined;
    const defaultCollectName = isPrivate || isEducation || (isGov && defaultSubjectProperty === 'G');
    form.setFieldsValue({
      subject_property: defaultSubjectProperty,
      p1: defaultRule?.p1 ?? p1LocksForSubject(defaultSubjectProperty, null).value,
      province_name: lockedProvinceName ?? '',
      city_name: lockedCityName ?? '',
      institution: defaultInstitution,
      education_type: defaultEducationType,
      private_type: privateType,
      partnership_kind: defaultPartnershipKind,
      cid_full_name: defaultCollectName ? '' : undefined,
      parent_cid_number: undefined,
      legal_rep_name: '',
      legal_rep_cid_number: '',
      legal_rep_photo_path: '',
      legal_rep_photo_name: '',
      legal_rep_photo_mime: '',
      legal_rep_photo_size: undefined,
    });
    setLegalRepOptions([]);
    setPhotoName('');
  }, [open, category, privateType, lockedProvinceName, lockedCityName]);

  useEffect(() => {
    if (!open || !lockedProvinceName) return;
    let cancelled = false;
    setCitiesLoading(true);
    loadCachedCidCities(auth, lockedProvinceName)
      .then((rows) => {
        if (!cancelled) setCities(rows.filter((c) => c.code !== '000'));
      })
      .catch((err) => {
        if (!cancelled) {
          setCities([]);
          notice.error(err, '');
        }
      })
      .finally(() => {
        if (!cancelled) setCitiesLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [open, lockedProvinceName, auth.access_token]);

  const onSubjectPropertyChange = (subject_property: string) => {
    setCurrentSubjectProperty(subject_property);
    setNameAvailable(null);
    // 中文注释:切主体属性必须重置所属法人与 p1(F 的 p1 是父级继承值,残留会提交旧值)。
    resetParentState();
    setLegalRepOptions([]);
    const nextInstitution = institutionChoicesFor(category, subject_property)[0]?.value;
    const nextEducationType = isEducation && subject_property !== 'F'
      ? (form.getFieldValue('education_type') ?? SCHOOL_EDUCATION_TYPE_OPTIONS[0]?.value)
      : undefined;
    const collectName =
      isEducation || (isGov && subject_property === 'G');
    form.setFieldsValue({
      institution: nextInstitution,
      education_type: nextEducationType,
      p1: p1LocksForSubject(subject_property, null).value,
      parent_cid_number: undefined,
      cid_full_name: collectName ? (form.getFieldValue('cid_full_name') ?? '') : undefined,
    });
  };

  const onPartnershipKindChange = (kind: PartnershipKind) => {
    if (!privateType) return;
    const rule = privateRuleFor(privateType, kind);
    setCurrentSubjectProperty(rule.subjectProperty);
    form.setFieldsValue({
      subject_property: rule.subjectProperty,
      p1: rule.p1,
      institution: rule.institution,
      partnership_kind: kind,
      parent_cid_number: undefined,
    });
    resetParentState();
    setLegalRepOptions([]);
  };

  // ── 所属法人搜索/选定(仅 F)────────────────────────────────

  const parentSearchOptions = (): SearchParentsOptions | null => {
    const province_name = (form.getFieldValue('province_name') ?? '').trim();
    const city_name = (form.getFieldValue('city_name') ?? '').trim();
    if (!province_name || !city_name) {
      notice.warning('请先选择市,所属法人按落位省市过滤');
      return null;
    }
    return {
      fInstitution: (form.getFieldValue('institution') ?? '').trim(),
      province_name:  province_name,
      city_name:  city_name,
      // 公权入口只挂公法人;教育入口(分校)由后端按学校本部过滤。
      parentProperty: isGov ? 'G' : undefined,
    };
  };

  const triggerParentSearch = async () => {
    const q = (form.getFieldValue('parent_cid_number') ?? '').trim();
    if (!q) {
      notice.warning('请先输入所属法人名称或身份ID关键字');
      return;
    }
    const opts = parentSearchOptions();
    if (!opts) return;
    setParentSearching(true);
    try {
      const rows = await searchParentInstitutions(auth, q, opts);
      setParentOptions(rows);
      if (rows.length === 0) {
        notice.info(isEducation ? '本市未找到可选的学校本部' : '未找到可选的所属法人');
      }
    } catch (err) {
      notice.error(err, '');
      setParentOptions([]);
    } finally {
      setParentSearching(false);
    }
  };

  const onParentSelect = (value: string) => {
    const row = parentOptions.find((r) => r.cid_number === value);
    if (!row) return;
    setSelectedParent(row);
    // 盈利属性附属于所属法人:选定父级即重算 p1(后端 uninorg 同规则复核)
    form.setFieldsValue({ p1: inheritedP1(row.subject_property, row.p1) });
    setLegalRepOptions([]);
  };

  const onParentInputChange = (value: string) => {
    if (selectedParent && value !== selectedParent.cid_number) {
      setSelectedParent(null);
      form.setFieldsValue({ p1: undefined });
      setLegalRepOptions([]);
    }
  };

  // ── 名称查重 ─────────────────────────────────────────────

  const onCheckName = async () => {
    const name = (form.getFieldValue('cid_full_name') ?? '').trim();
    if (!name) {
      notice.warning(`请先输入${nameLabel}`);
      return;
    }
    // 中文注释:G(公立学校/公权机构)查重是同市同名(后端 check-name G 分支要求 city_name),S/F 全国查重。
    const isGovName = currentSubjectProperty === 'G';
    if (isGovName) {
      const city_name = (form.getFieldValue('city_name') ?? '').trim();
      if (!city_name) {
        notice.warning(`${nameLabel}查重需要先选择市`);
        return;
      }
    }
    setNameChecking(true);
    try {
      const cityVal = isGovName ? (form.getFieldValue('city_name') ?? '').trim() : undefined;
      const { exists } = await checkInstitutionName(
        auth,
        name,
        form.getFieldValue('subject_property'),
        cityVal,
      );
      if (exists) {
        notice.error(isGovName ? '该市已存在同名机构，请更换名称' : '该机构名称已被使用');
        setNameAvailable(false);
      } else {
        notice.success('名称可用');
        setNameAvailable(true);
      }
    } catch (err) {
      notice.error(err, '');
      setNameAvailable(null);
    } finally {
      setNameChecking(false);
    }
  };

  const onNameChange = () => {
    if (nameAvailable !== null) setNameAvailable(null);
  };

  // ── 法定代表人 ───────────────────────────────────────────

  const triggerLegalRepSearch = async () => {
    const q = (form.getFieldValue('legal_rep_cid_number') ?? '').trim();
    if (!q) {
      notice.warning('请先输入法定代表人身份ID关键字');
      return;
    }
    const province_name = (form.getFieldValue('province_name') ?? '').trim();
    const city_name = (form.getFieldValue('city_name') ?? '').trim();
    const subjectProperty = (form.getFieldValue('subject_property') ?? '').trim();
    const institution = (form.getFieldValue('institution') ?? '').trim();
    if (!province_name || !city_name || !subjectProperty || !institution) {
      notice.warning('请先选择省、市、主体属性和机构');
      return;
    }
    if (requiresParent && !selectedParent) {
      notice.warning('请先从搜索结果中选择所属法人');
      return;
    }
    setLegalRepSearching(true);
    try {
      const rows = await searchLegalRepresentativeCitizens(auth, q, {
        province_name:  province_name,
        city_name:  city_name,
        subject_property: subjectProperty,
        institution,
        education_type: showEducationType ? form.getFieldValue('education_type') : undefined,
        parent_cid_number: requiresParent ? selectedParent?.cid_number : undefined,
      });
      setLegalRepOptions(rows);
      if (rows.length === 0) {
        notice.info('未找到正常状态公民');
      }
    } catch (err) {
      notice.error(err, '');
      setLegalRepOptions([]);
    } finally {
      setLegalRepSearching(false);
    }
  };

  const handlePhotoUpload = async (file: File) => {
    setPhotoUploading(true);
    try {
      const photo = await uploadLegalRepresentativePhoto(auth, file);
      form.setFieldsValue({
        legal_rep_photo_path: photo.file_path,
        legal_rep_photo_name: photo.file_name,
        legal_rep_photo_mime: photo.mime_type,
        legal_rep_photo_size: photo.file_size,
      });
      setPhotoName(photo.file_name);
      notice.success('证件照已上传');
    } catch (err) {
      notice.error(err, '证件照上传失败');
    } finally {
      setPhotoUploading(false);
    }
    return false;
  };

  // ── 提交 ─────────────────────────────────────────────────

  const onSubmit = async (values: FormValues) => {
    if (collectNameInModal && nameAvailable !== true) {
      notice.warning('请先点击搜索图标检查名称是否可用');
      return;
    }
    if (requiresParent) {
      // 非法人必须从搜索结果中选定所属法人,手填未选定的不放行
      if (!selectedParent || selectedParent.cid_number !== (values.parent_cid_number ?? '').trim()) {
        notice.warning('请从搜索结果中选择所属法人');
        return;
      }
    }
    setSubmitting(true);
    try {
      const result = await createInstitution(auth, {
        subject_property: values.subject_property.trim(),
        p1: values.p1?.trim(),
        province_name: values.province_name.trim(),
        city_name: values.city_name.trim(),
        institution: values.institution.trim(),
        education_type: showEducationType ? values.education_type : undefined,
        cid_full_name: collectNameInModal
          ? (values.cid_full_name ?? '').trim()
          : undefined,
        parent_cid_number: requiresParent ? (values.parent_cid_number ?? '').trim() : undefined,
        private_type: isPrivate ? privateType : undefined,
        partnership_kind: isPrivate && privateType === 'PARTNERSHIP'
          ? values.partnership_kind
          : undefined,
        legal_rep_name: values.legal_rep_name.trim(),
        legal_rep_cid_number: values.legal_rep_cid_number.trim(),
        legal_rep_photo_path: values.legal_rep_photo_path,
        legal_rep_photo_name: values.legal_rep_photo_name,
        legal_rep_photo_mime: values.legal_rep_photo_mime,
        legal_rep_photo_size: values.legal_rep_photo_size,
      });
      if (isPrivate && privateType) {
        notice.success(`${PRIVATE_TYPE_LABEL[privateType]}已创建:${result.cid_number}`);
      } else if (isEducation) {
        notice.success(`学校机构已创建:${result.cid_number}`);
      } else if (collectNameInModal) {
        notice.success(`公权机构已创建:${result.cid_number}`);
      } else {
        notice.success(`身份ID 已生成:${result.cid_number}`);
      }
      onCreated(result);
    } catch (err) {
      const raw = err instanceof Error ? err.message : '创建机构失败';
      if (raw.includes('本省') && raw.includes('未在线')) {
        notice.error('本省登录管理员未在线,请联系联邦注册局机构管理员登录后重试');
      } else if (raw.includes('已被使用') || raw.includes('同名机构')) {
        notice.error('该市已存在同名机构，请更换名称');
        setNameAvailable(false);
      } else {
        notice.error(err, '创建机构失败');
      }
    } finally {
      setSubmitting(false);
    }
  };

  const subjectPropertyDisabled = isPrivate || subjectPropertyChoices.length === 1;
  const instDisabled = instChoices.length === 1;
  const nameCheckPassed = !collectNameInModal || nameAvailable === true;

  return (
    <Modal
      title={
        <div style={{ textAlign: 'center', width: '100%' }}>
          {isPrivate && privateType ? `新增${PRIVATE_TYPE_LABEL[privateType]}` : locks.modalTitle}
        </div>
      }
      open={open}
      onCancel={onCancel}
      footer={[
        <Button key="cancel" onClick={onCancel}>
          取消
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={submitting}
          disabled={!nameCheckPassed}
          style={
            nameCheckPassed
              ? { backgroundColor: '#52c41a', borderColor: '#52c41a' }
              : undefined
          }
          onClick={() => form.submit()}
        >
          生成
        </Button>,
      ]}
      destroyOnClose
    >
      <Form form={form} layout="vertical" onFinish={onSubmit}>
        {/* 中文注释:短选项字段双列排布压低弹窗高度;所属法人/法定代表人身份ID 内容长,保持整行。 */}
        <Row gutter={16}>
          {isPrivate && privateType === 'PARTNERSHIP' && (
            <Col span={24}>
              <Form.Item label="合伙类型" name="partnership_kind" rules={[{ required: true }]}>
                <Select
                  options={[
                    { value: 'GENERAL', label: '无限合伙' },
                    { value: 'LIMITED', label: '有限合伙' },
                  ]}
                  onChange={onPartnershipKindChange}
                />
              </Form.Item>
            </Col>
          )}
          <Col span={12}>
            <Form.Item label="主体属性" name="subject_property" rules={[{ required: true }]}>
              <Select options={subjectPropertyChoices} disabled={subjectPropertyDisabled} onChange={onSubjectPropertyChange} />
            </Form.Item>
          </Col>
          <Col span={12}>
            <Form.Item
              label="盈利属性"
              name="p1"
              rules={[
                {
                  required: true,
                  message: isF ? '盈利属性继承所属法人,请先选择所属法人' : '请选择盈利属性',
                },
              ]}
            >
              <Select
                options={p1Locks.choices}
                disabled={p1Locks.locked}
                placeholder={isF ? '由所属法人决定' : undefined}
              />
            </Form.Item>
          </Col>
        </Row>
        <Row gutter={16}>
          <Col span={12}>
            <Form.Item label="省" name="province_name" rules={[{ required: true }]}>
              <Input disabled />
            </Form.Item>
          </Col>
          <Col span={12}>
            <Form.Item label="市" name="city_name" rules={[{ required: true, message: '请选择市' }]}>
              <Select
                loading={citiesLoading}
                disabled={lockedCityName !== null}
                options={cities.map((c) => ({ label: c.name, value: c.name }))}
                placeholder="请选择市"
                onChange={() => {
                  // 中文注释:G 名称查重按市,所属法人搜索按落位省市;换市后两者都要重来。
                  if (currentSubjectProperty === 'G' && nameAvailable !== null) {
                    setNameAvailable(null);
                  }
                  if (isF && (selectedParent || parentOptions.length > 0)) {
                    resetParentState();
                    form.setFieldsValue({ parent_cid_number: undefined, p1: undefined });
                  }
                  setLegalRepOptions([]);
                }}
              />
            </Form.Item>
          </Col>
        </Row>
        <Row gutter={16}>
          <Col span={12}>
            <Form.Item label="机构" name="institution" rules={[{ required: true }]}>
              <Select options={instChoices} disabled={instDisabled} />
            </Form.Item>
          </Col>
          {showEducationType && (
            <Col span={12}>
              <Form.Item
                label="教育机构类型"
                name="education_type"
                rules={[{ required: true, message: '请选择教育机构类型' }]}
              >
                <Select options={SCHOOL_EDUCATION_TYPE_OPTIONS} />
              </Form.Item>
            </Col>
          )}
        </Row>
        {collectNameInModal && (
          <Row gutter={16}>
            <Col span={24}>
              <Form.Item
                label={nameLabel}
                name="cid_full_name"
                rules={[
                  { required: true, message: `请输入${nameLabel}` },
                  { max: 30, message: '最多 30 个字' },
                ]}
              >
                <Input
                  placeholder={`请输入${nameLabel}`}
                  maxLength={30}
                  onChange={onNameChange}
                  suffix={
                    <span
                      style={{ cursor: 'pointer', color: nameChecking ? '#999' : '#1890ff' }}
                      onClick={nameChecking ? undefined : onCheckName}
                    >
                      {nameChecking ? <Spin size="small" /> : <SearchOutlined />}
                    </span>
                  }
                />
              </Form.Item>
              {nameAvailable === true && (
                <div style={{ color: '#52c41a', marginTop: -16, marginBottom: 12, fontSize: 12 }}>
                  名称可用
                </div>
              )}
              {nameAvailable === false && (
                <div style={{ color: '#ff4d4f', marginTop: -16, marginBottom: 12, fontSize: 12 }}>
                  该名称已被占用，请更换
                </div>
              )}
            </Col>
          </Row>
        )}
        {requiresParent && (
          <>
            <Form.Item
              label={isEducation ? '所属法人(学校本部)' : '所属法人'}
              name="parent_cid_number"
              rules={[{ required: true, message: '请选择所属法人' }]}
            >
              <AutoComplete
                filterOption={false}
                options={parentOptions.map((row) => ({
                  value: row.cid_number,
                  label: `${row.cid_full_name}(${SUBJECT_PROPERTY_LABEL[row.subject_property] ?? row.subject_property}) ${row.province_name}/${row.city_name}`,
                }))}
                onSelect={onParentSelect}
                onChange={onParentInputChange}
              >
                <Input
                  placeholder="输入所属法人名称或身份ID后点击搜索"
                  suffix={
                    <span
                      style={{
                        cursor: parentSearching ? 'default' : 'pointer',
                        color: parentSearching ? '#999' : '#1890ff',
                      }}
                      onClick={parentSearching ? undefined : triggerParentSearch}
                      title="搜索所属法人"
                    >
                      {parentSearching ? <Spin size="small" /> : <SearchOutlined />}
                    </span>
                  }
                />
              </AutoComplete>
            </Form.Item>
            <div style={{ color: '#888', fontSize: 12, marginTop: -16, marginBottom: 12 }}>
              {isEducation
                ? '分校与本部同市,盈利属性继承本部学校。'
                : isGov
                  ? '可选本市市级、本省省级或国家级公权机构,盈利属性锁定非盈利。'
                  : '可选全国私法人机构,盈利属性继承所属法人。'}
            </div>
            {selectedParent && (
              <div style={{ color: '#52c41a', marginTop: -8, marginBottom: 12, fontSize: 12 }}>
                已选:{selectedParent.cid_full_name}(
                {SUBJECT_PROPERTY_LABEL[selectedParent.subject_property] ?? selectedParent.subject_property}
                ,{selectedParent.p1 === '1' ? '盈利' : '非盈利'})
              </div>
            )}
          </>
        )}
        <Row gutter={16}>
          <Col span={12}>
            <Form.Item
              label="法定代表人姓名"
              name="legal_rep_name"
              rules={[
                { required: true, message: '请输入法定代表人姓名' },
                { max: 30, message: '最多 30 个字' },
              ]}
            >
              <Input placeholder="请输入法定代表人姓名" maxLength={30} />
            </Form.Item>
          </Col>
          <Col span={12}>
            <Form.Item label="法定代表人证件照" required>
              <Upload
                accept="image/jpeg,image/png,image/webp"
                showUploadList={false}
                beforeUpload={(file) => handlePhotoUpload(file as File)}
              >
                <Button icon={<UploadOutlined />} loading={photoUploading}>
                  上传证件照
                </Button>
              </Upload>
              {photoName && (
                <div style={{ color: '#52c41a', marginTop: 8, fontSize: 12 }}>
                  {photoName}
                </div>
              )}
            </Form.Item>
          </Col>
        </Row>
        <Form.Item
          label="法定代表人身份ID"
          name="legal_rep_cid_number"
          rules={[{ required: true, message: '请选择法定代表人身份ID' }]}
        >
          <AutoComplete
            filterOption={false}
            options={legalRepOptions.map((cidNumber) => ({
              value: cidNumber,
              label: cidNumber,
            }))}
          >
            <Input
              placeholder="输入身份ID后点击搜索"
              suffix={
                <span
                  style={{
                    cursor: legalRepSearching ? 'default' : 'pointer',
                    color: legalRepSearching ? '#999' : '#1890ff',
                  }}
                  onClick={legalRepSearching ? undefined : triggerLegalRepSearch}
                  title="搜索正常状态公民"
                >
                  {legalRepSearching ? <Spin size="small" /> : <SearchOutlined />}
                </span>
              }
            />
          </AutoComplete>
        </Form.Item>
        <Form.Item
          name="legal_rep_photo_path"
          rules={[{ required: true, message: '请上传法定代表人证件照' }]}
          hidden
        >
          <Input />
        </Form.Item>
        <Form.Item name="legal_rep_photo_name" hidden><Input /></Form.Item>
        <Form.Item name="legal_rep_photo_mime" hidden><Input /></Form.Item>
        <Form.Item name="legal_rep_photo_size" hidden><Input type="number" /></Form.Item>
        {!collectNameInModal && (
          <div style={{ color: '#888', fontSize: 12, marginTop: -8 }}>
            提示:本步骤仅生成身份ID。生成后请在详情页设置机构名称等信息。
          </div>
        )}
      </Form>
    </Modal>
  );
};
