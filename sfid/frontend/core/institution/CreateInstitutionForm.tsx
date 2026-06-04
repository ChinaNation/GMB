// 中文注释:机构新增弹窗共享表单。gov/private 只传入各自 API 函数,不在公共组件里越过业务边界。

import React, { useEffect, useMemo, useState } from 'react';
import { Button, Form, Input, message, Modal, Select, Spin } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import type { AdminAuth } from '../../auth/types';
import type { SfidCityItem } from '../../china/api';
import { loadCachedSfidCities } from '../../china/metaCache';
import type {
  CreateInstitutionInput,
  CreateInstitutionOutput,
  InstitutionCategory,
} from '../../subjects/api';
import { dynamicLocksForA3, locksForCategory } from '../../subjects/labels';

interface FormValues {
  a3: string;
  p1: string;
  province: string;
  city: string;
  institution: string;
  /** 普通私权机构不填;教育委员会(JY)学校机构必填学校名称。 */
  institution_name?: string;
}

type CheckInstitutionName = (
  auth: AdminAuth,
  name: string,
  a3?: string,
  city?: string,
) => Promise<{ exists: boolean }>;

type CreateInstitution = (
  auth: AdminAuth,
  input: CreateInstitutionInput,
) => Promise<CreateInstitutionOutput>;

export interface CreateInstitutionFormProps {
  auth: AdminAuth;
  category: InstitutionCategory;
  open: boolean;
  lockedProvince: string | null;
  lockedCity: string | null;
  checkInstitutionName: CheckInstitutionName;
  createInstitution: CreateInstitution;
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

  const [currentA3, setCurrentA3] = useState<string>(locks.a3Choices[0]?.value ?? '');
  const [currentInstitution, setCurrentInstitution] = useState<string>(
    locks.institutionChoices[0]?.value ?? '',
  );
  const dynamicLocks = useMemo(() => dynamicLocksForA3(currentA3), [currentA3]);
  const isPrivate = category === 'PRIVATE_INSTITUTION';
  const isPublicGov = category === 'GOV_INSTITUTION';
  const isPublicSecurity = category === 'PUBLIC_SECURITY';
  const isEducationSchool = currentInstitution === 'JY';

  // 中文注释:普通私权两步式不在弹窗收名称;教育委员会(JY)创建的是学校机构,必须填写学校名称。
  const collectNameInModal = !isPrivate || isEducationSchool;

  const effectiveP1Choices = isPrivate ? dynamicLocks.p1Choices : locks.p1Choices;
  const effectiveInstChoices = isPrivate ? dynamicLocks.institutionChoices : locks.institutionChoices;

  useEffect(() => {
    if (!open) return;
    const defaultA3 = locks.a3Choices[0]?.value ?? '';
    setCurrentA3(defaultA3);
    setNameAvailable(null);
    const dl = dynamicLocksForA3(defaultA3);
    const defaultInstitution = isPrivate
      ? dl.institutionChoices[0]?.value
      : locks.institutionChoices[0]?.value;
    setCurrentInstitution(defaultInstitution ?? '');
    form.setFieldsValue({
      a3: defaultA3,
      p1: isPrivate ? dl.p1Default : locks.p1Choices[0]?.value,
      province: lockedProvince ?? '',
      city: lockedCity ?? '',
      institution: defaultInstitution,
      institution_name: (!isPrivate || defaultInstitution === 'JY')
        ? (locks.lockedInstitutionName ?? '')
        : undefined,
    });
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
          message.error(err instanceof Error ? err.message : '加载城市列表失败');
        }
      })
      .finally(() => {
        if (!cancelled) setCitiesLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [open, lockedProvince, auth.access_token]);

  const onA3Change = (a3: string) => {
    setCurrentA3(a3);
    const dl = dynamicLocksForA3(a3);
    const nextInstitution = dl.institutionChoices[0]?.value;
    setCurrentInstitution(nextInstitution ?? '');
    setNameAvailable(null);
    form.setFieldsValue({
      p1: dl.p1Default,
      institution: nextInstitution,
      institution_name: nextInstitution === 'JY' ? '' : undefined,
    });
  };

  const onInstitutionChange = (institution: string) => {
    setCurrentInstitution(institution);
    setNameAvailable(null);
    if (institution !== 'JY' && isPrivate) {
      form.setFieldsValue({
        institution_name: undefined,
      });
    }
  };

  const onCheckName = async () => {
    const name = (form.getFieldValue('institution_name') ?? '').trim();
    if (!name) {
      message.warning(isEducationSchool ? '请先输入学校名称' : '请先输入机构名称');
      return;
    }
    if (isPublicGov) {
      const city = (form.getFieldValue('city') ?? '').trim();
      if (!city) {
        message.warning('学校名称查重需要先选择市');
        return;
      }
    }
    setNameChecking(true);
    try {
      const cityVal = isPublicGov ? (form.getFieldValue('city') ?? '').trim() : undefined;
      const { exists } = await checkInstitutionName(
        auth,
        name,
        isPublicGov ? 'GFR' : form.getFieldValue('a3'),
        cityVal,
      );
      if (exists) {
        message.error(isPublicGov ? '该市已存在同名机构，请更换名称' : '该机构名称已被使用');
        setNameAvailable(false);
      } else {
        message.success('名称可用');
        setNameAvailable(true);
      }
    } catch (err) {
      message.error(err instanceof Error ? err.message : '查重失败');
      setNameAvailable(null);
    } finally {
      setNameChecking(false);
    }
  };

  const onNameChange = () => {
    if (nameAvailable !== null) setNameAvailable(null);
  };

  const onSubmit = async (values: FormValues) => {
    if (collectNameInModal && !locks.lockedInstitutionName && nameAvailable !== true) {
      message.warning('请先点击搜索图标检查名称是否可用');
      return;
    }
    setSubmitting(true);
    try {
      const result = await createInstitution(auth, {
        a3: values.a3.trim(),
        p1: values.p1?.trim(),
        province: values.province.trim(),
        city: values.city.trim(),
        institution: values.institution.trim(),
        institution_name: collectNameInModal
          ? (values.institution_name ?? '').trim()
          : undefined,
      });
      if (isPrivate && !isEducationSchool) {
        message.success(`机构 SFID 已生成,请到详情页完善信息:${result.sfid_number}`);
      } else {
        message.success(`学校机构已创建:${result.sfid_number}`);
      }
      onCreated(result);
    } catch (err) {
      const raw = err instanceof Error ? err.message : '创建机构失败';
      if (raw.includes('本省') && raw.includes('未在线')) {
        message.error('本省登录管理员未在线,请联系省管理员登录后重试');
      } else if (raw.includes('已被使用') || raw.includes('同名机构')) {
        message.error('该市已存在同名机构，请更换名称');
        setNameAvailable(false);
      } else {
        message.error(raw);
      }
    } finally {
      setSubmitting(false);
    }
  };

  const a3Disabled = locks.a3Choices.length === 1;
  const p1Disabled = effectiveP1Choices.length === 1;
  const instDisabled = effectiveInstChoices.length === 1;
  const nameDisabled = locks.lockedInstitutionName !== null;
  const nameCheckPassed = !collectNameInModal || nameDisabled || nameAvailable === true;
  const nameLabel = isEducationSchool ? '学校名称' : '机构名称';

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
        <Form.Item label="A3 主体属性" name="a3" rules={[{ required: true }]}>
          <Select options={locks.a3Choices} disabled={a3Disabled} onChange={onA3Change} />
        </Form.Item>
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
              if (isPublicGov && nameAvailable !== null) setNameAvailable(null);
            }}
          />
        </Form.Item>
        <Form.Item label="机构" name="institution" rules={[{ required: true }]}>
          <Select
            options={effectiveInstChoices}
            disabled={instDisabled}
            onChange={onInstitutionChange}
          />
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
                disabled={nameDisabled}
                placeholder={`请输入${nameLabel}(最多 30 字)`}
                maxLength={30}
                onChange={onNameChange}
                suffix={
                  nameDisabled || isPublicSecurity ? null : (
                    <span
                      style={{ cursor: 'pointer', color: nameChecking ? '#999' : '#1890ff' }}
                      onClick={nameChecking ? undefined : onCheckName}
                    >
                      {nameChecking ? <Spin size="small" /> : <SearchOutlined />}
                    </span>
                  )
                }
              />
            </Form.Item>
            {!nameDisabled && nameAvailable === true && (
              <div style={{ color: '#52c41a', marginTop: -16, marginBottom: 12, fontSize: 12 }}>
                名称可用
              </div>
            )}
            {!nameDisabled && nameAvailable === false && (
              <div style={{ color: '#ff4d4f', marginTop: -16, marginBottom: 12, fontSize: 12 }}>
                该名称已被占用，请更换
              </div>
            )}
          </>
        )}
        {isPrivate && !isEducationSchool && (
          <div style={{ color: '#888', fontSize: 12, marginTop: -8 }}>
            提示:本步骤仅生成机构 SFID。生成后请在详情页设置机构名称、企业类型等信息。
          </div>
        )}
      </Form>
    </Modal>
  );
};
