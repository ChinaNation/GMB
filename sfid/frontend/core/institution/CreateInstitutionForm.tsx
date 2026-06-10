// 中文注释:机构新增弹窗共享表单。private/education 只传入各自 API 函数,不在公共组件里越过业务边界。

import React, { useEffect, useMemo, useState } from 'react';
import { AutoComplete, Button, Form, Input, Modal, Select, Spin, Upload } from 'antd';
import { SearchOutlined, UploadOutlined } from '@ant-design/icons';
import type { AdminAuth } from '../../auth/types';
import type { SfidCityItem } from '../../china/api';
import { loadCachedSfidCities } from '../../china/metaCache';
import type {
  CreateInstitutionInput,
  CreateInstitutionOutput,
  LegalRepresentativePhoto,
} from '../../subjects/api';
import { searchLegalRepresentativeCitizens } from '../../citizens/api';
import {
  dynamicLocksForSubjectProperty,
  educationP1Locks,
  locksForCategory,
  type CreateFormCategory,
} from '../../subjects/labels';
import { notice } from '../../utils/notice';

interface FormValues {
  subject_property: string;
  p1: string;
  province: string;
  city: string;
  institution: string;
  /** 私权两步式不填;教育机构(JY 学校)必填学校名称。 */
  institution_name?: string;
  /** 教育机构 F(分校)专用:上级法人属性(G/S),仅用于推导 p1,不进提交 payload。 */
  parent_subject_property?: string;
  /** 教育机构 F(分校)且上级=S 时:上级盈利属性(0/1),仅用于推导 p1,不进提交 payload。 */
  parent_p1?: string;
  legal_rep_name: string;
  legal_rep_sfid_number: string;
  legal_rep_photo_path: string;
  legal_rep_photo_name: string;
  legal_rep_photo_mime: string;
  legal_rep_photo_size?: number;
}

type CheckInstitutionName = (
  auth: AdminAuth,
  name: string,
  subject_property?: string,
  city?: string,
) => Promise<{ exists: boolean }>;

type CreateInstitution = (
  auth: AdminAuth,
  input: CreateInstitutionInput,
) => Promise<CreateInstitutionOutput>;

type UploadLegalRepresentativePhoto = (
  auth: AdminAuth,
  file: File,
) => Promise<LegalRepresentativePhoto>;

export interface CreateInstitutionFormProps {
  auth: AdminAuth;
  category: CreateFormCategory;
  open: boolean;
  lockedProvince: string | null;
  lockedCity: string | null;
  checkInstitutionName: CheckInstitutionName;
  createInstitution: CreateInstitution;
  uploadLegalRepresentativePhoto: UploadLegalRepresentativePhoto;
  onCancel: () => void;
  onCreated: (result: CreateInstitutionOutput) => void;
}

export const CreateInstitutionForm: React.FC<CreateInstitutionFormProps> = ({
  auth,
  category,
  open,
  lockedProvince,
  lockedCity,
  checkInstitutionName,
  createInstitution,
  uploadLegalRepresentativePhoto,
  onCancel,
  onCreated,
}) => {
  const locks = locksForCategory(category);
  const [form] = Form.useForm<FormValues>();
  const [cities, setCities] = useState<SfidCityItem[]>([]);
  const [citiesLoading, setCitiesLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [nameChecking, setNameChecking] = useState(false);
  const [nameAvailable, setNameAvailable] = useState<boolean | null>(null);
  const [legalRepSearching, setLegalRepSearching] = useState(false);
  const [legalRepOptions, setLegalRepOptions] = useState<string[]>([]);
  const [photoUploading, setPhotoUploading] = useState(false);
  const [photoName, setPhotoName] = useState<string>('');

  const [currentSubjectProperty, setCurrentSubjectProperty] = useState<string>(locks.subjectPropertyChoices[0]?.value ?? '');
  // 教育机构 F(分校)专用:上级法人属性 + 上级盈利属性,仅用于推导 p1
  const [parentSubjectProperty, setParentSubjectProperty] = useState<string>('G');
  const [parentP1, setParentP1] = useState<string>('0');
  const dynamicLocks = useMemo(() => dynamicLocksForSubjectProperty(currentSubjectProperty), [currentSubjectProperty]);
  const isPrivate = category === 'PRIVATE_INSTITUTION';
  const isEducation = category === 'EDUCATION_INSTITUTION';
  const educationLocks = useMemo(
    () => educationP1Locks(currentSubjectProperty, parentSubjectProperty, parentP1),
    [currentSubjectProperty, parentSubjectProperty, parentP1],
  );

  // 中文注释:私权两步式不在弹窗收名称;教育机构创建的是学校机构,必须填写学校名称。
  const collectNameInModal = isEducation;

  const effectiveP1Choices = isEducation ? educationLocks.p1Choices : dynamicLocks.p1Choices;
  const effectiveInstChoices = isEducation ? locks.institutionChoices : dynamicLocks.institutionChoices;

  useEffect(() => {
    if (!open) return;
    const defaultSubjectProperty = locks.subjectPropertyChoices[0]?.value ?? '';
    setCurrentSubjectProperty(defaultSubjectProperty);
    setParentSubjectProperty('G');
    setParentP1('0');
    setNameAvailable(null);
    const dl = dynamicLocksForSubjectProperty(defaultSubjectProperty);
    const defaultInstitution = isEducation
      ? locks.institutionChoices[0]?.value
      : dl.institutionChoices[0]?.value;
    form.setFieldsValue({
      subject_property: defaultSubjectProperty,
      p1: isEducation
        ? educationP1Locks(defaultSubjectProperty, 'G', '0').p1Value
        : dl.p1Default,
      province: lockedProvince ?? '',
      city: lockedCity ?? '',
      institution: defaultInstitution,
      institution_name: isEducation ? '' : undefined,
      parent_subject_property: 'G',
      parent_p1: '0',
      legal_rep_name: '',
      legal_rep_sfid_number: '',
      legal_rep_photo_path: '',
      legal_rep_photo_name: '',
      legal_rep_photo_mime: '',
      legal_rep_photo_size: undefined,
    });
    setLegalRepOptions([]);
    setPhotoName('');
  }, [open, category, lockedProvince, lockedCity]);

  useEffect(() => {
    if (!open || !lockedProvince) return;
    let cancelled = false;
    setCitiesLoading(true);
    loadCachedSfidCities(auth, lockedProvince)
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
  }, [open, lockedProvince, auth.access_token]);

  const onSubjectPropertyChange = (subject_property: string) => {
    setCurrentSubjectProperty(subject_property);
    setNameAvailable(null);
    if (isEducation) {
      // 中文注释:F 的 p1 是隐藏推导值,切主体属性时上级两字段一并重置,p1 同步重算。
      setParentSubjectProperty('G');
      setParentP1('0');
      form.setFieldsValue({
        p1: educationP1Locks(subject_property, 'G', '0').p1Value,
        parent_subject_property: 'G',
        parent_p1: '0',
      });
      return;
    }
    const dl = dynamicLocksForSubjectProperty(subject_property);
    const nextInstitution = dl.institutionChoices[0]?.value;
    form.setFieldsValue({
      p1: dl.p1Default,
      institution: nextInstitution,
    });
  };

  const onParentSubjectPropertyChange = (value: string) => {
    setParentSubjectProperty(value);
    setParentP1('0');
    form.setFieldsValue({
      parent_p1: '0',
      p1: educationP1Locks(currentSubjectProperty, value, '0').p1Value,
    });
  };

  const onParentP1Change = (value: string) => {
    setParentP1(value);
    form.setFieldsValue({
      p1: educationP1Locks(currentSubjectProperty, parentSubjectProperty, value).p1Value,
    });
  };

  const onCheckName = async () => {
    const name = (form.getFieldValue('institution_name') ?? '').trim();
    if (!name) {
      notice.warning('请先输入学校名称');
      return;
    }
    // 中文注释:G(公立学校)查重是同市同名(后端 check-name G 分支要求 city),S/F 全国查重。
    const isPublicSchool = currentSubjectProperty === 'G';
    if (isPublicSchool) {
      const city = (form.getFieldValue('city') ?? '').trim();
      if (!city) {
        notice.warning('学校名称查重需要先选择市');
        return;
      }
    }
    setNameChecking(true);
    try {
      const cityVal = isPublicSchool ? (form.getFieldValue('city') ?? '').trim() : undefined;
      const { exists } = await checkInstitutionName(
        auth,
        name,
        form.getFieldValue('subject_property'),
        cityVal,
      );
      if (exists) {
        notice.error(isPublicSchool ? '该市已存在同名机构，请更换名称' : '该机构名称已被使用');
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

  const triggerLegalRepSearch = async () => {
    const q = (form.getFieldValue('legal_rep_sfid_number') ?? '').trim();
    if (!q) {
      notice.warning('请先输入法定代表人身份ID关键字');
      return;
    }
    setLegalRepSearching(true);
    try {
      const rows = await searchLegalRepresentativeCitizens(auth, q);
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

  const onSubmit = async (values: FormValues) => {
    if (collectNameInModal && nameAvailable !== true) {
      notice.warning('请先点击搜索图标检查名称是否可用');
      return;
    }
    setSubmitting(true);
    try {
      const result = await createInstitution(auth, {
        subject_property: values.subject_property.trim(),
        p1: values.p1?.trim(),
        province: values.province.trim(),
        city: values.city.trim(),
        institution: values.institution.trim(),
        institution_name: collectNameInModal
          ? (values.institution_name ?? '').trim()
          : undefined,
        legal_rep_name: values.legal_rep_name.trim(),
        legal_rep_sfid_number: values.legal_rep_sfid_number.trim(),
        legal_rep_photo_path: values.legal_rep_photo_path,
        legal_rep_photo_name: values.legal_rep_photo_name,
        legal_rep_photo_mime: values.legal_rep_photo_mime,
        legal_rep_photo_size: values.legal_rep_photo_size,
      });
      if (isPrivate) {
        notice.success(`身份ID 已生成,请到详情页完善信息:${result.sfid_number}`);
      } else {
        notice.success(`学校机构已创建:${result.sfid_number}`);
      }
      onCreated(result);
    } catch (err) {
      const raw = err instanceof Error ? err.message : '创建机构失败';
      if (raw.includes('本省') && raw.includes('未在线')) {
        notice.error('本省登录管理员未在线,请联系联邦管理员登录后重试');
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

  const subjectPropertyDisabled = locks.subjectPropertyChoices.length === 1;
  const p1Disabled = isEducation ? educationLocks.p1Locked : effectiveP1Choices.length === 1;
  const instDisabled = effectiveInstChoices.length === 1;
  const nameCheckPassed = !collectNameInModal || nameAvailable === true;
  const nameLabel = '学校名称';
  const isBranchSchool = isEducation && currentSubjectProperty === 'F';

  return (
    <Modal
      title={<div style={{ textAlign: 'center', width: '100%' }}>{locks.modalTitle}</div>}
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
        <Form.Item label="SubjectProperty 主体属性" name="subject_property" rules={[{ required: true }]}>
          <Select options={locks.subjectPropertyChoices} disabled={subjectPropertyDisabled} onChange={onSubjectPropertyChange} />
        </Form.Item>
        {isBranchSchool && (
          <>
            <Form.Item
              label="上级法人属性"
              name="parent_subject_property"
              rules={[{ required: true, message: '请选择上级法人属性' }]}
            >
              <Select
                options={[
                  { value: 'G', label: '公法人 (G)' },
                  { value: 'S', label: '私法人 (S)' },
                ]}
                onChange={onParentSubjectPropertyChange}
              />
            </Form.Item>
            {parentSubjectProperty === 'S' && (
              <Form.Item
                label="上级盈利属性"
                name="parent_p1"
                rules={[{ required: true, message: '请选择上级盈利属性' }]}
              >
                <Select
                  options={[
                    { value: '1', label: '盈利 (1)' },
                    { value: '0', label: '非盈利 (0)' },
                  ]}
                  onChange={onParentP1Change}
                />
              </Form.Item>
            )}
            <div style={{ color: '#888', fontSize: 12, marginTop: -16, marginBottom: 12 }}>
              上级法人属性仅用于推导盈利属性,创建后请在详情页关联所属法人。
            </div>
          </>
        )}
        <Form.Item label="P1 盈利属性" name="p1" rules={[{ required: true }]}>
          <Select options={effectiveP1Choices} disabled={p1Disabled} />
        </Form.Item>
        <Form.Item label="省" name="province" rules={[{ required: true }]}>
          <Input disabled />
        </Form.Item>
        <Form.Item label="市" name="city" rules={[{ required: true, message: '请选择市' }]}>
          <Select
            loading={citiesLoading}
            disabled={lockedCity !== null}
            options={cities.map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }))}
            placeholder="请选择市"
            onChange={() => {
              // 中文注释:G(公立学校)查重按市,换市后必须重新查重。
              if (isEducation && currentSubjectProperty === 'G' && nameAvailable !== null) {
                setNameAvailable(null);
              }
            }}
          />
        </Form.Item>
        <Form.Item label="机构" name="institution" rules={[{ required: true }]}>
          <Select options={effectiveInstChoices} disabled={instDisabled} />
        </Form.Item>
        {collectNameInModal && (
          <>
            <Form.Item
              label={nameLabel}
              name="institution_name"
              rules={[
                { required: true, message: `请输入${nameLabel}` },
                { max: 30, message: '最多 30 个字' },
              ]}
            >
              <Input
                placeholder={`请输入${nameLabel}(最多 30 字)`}
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
          </>
        )}
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
        <Form.Item
          label="法定代表人身份ID"
          name="legal_rep_sfid_number"
          rules={[{ required: true, message: '请选择法定代表人身份ID' }]}
        >
          <AutoComplete
            filterOption={false}
            options={legalRepOptions.map((sfidNumber) => ({
              value: sfidNumber,
              label: sfidNumber,
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
        {isPrivate && (
          <div style={{ color: '#888', fontSize: 12, marginTop: -8 }}>
            提示:本步骤仅生成身份ID。生成后请在详情页设置机构名称、企业类型等信息。
          </div>
        )}
      </Form>
    </Modal>
  );
};
