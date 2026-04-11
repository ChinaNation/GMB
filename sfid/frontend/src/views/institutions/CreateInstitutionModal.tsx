// 中文注释:通用"新增机构"弹窗,按 category 锁字段。
// 交互规则:
//   1. 机构名称查重:私权机构全国唯一;公权机构同城唯一(不同市可重名)
//   2. A3 切换联动:SFR → P1=盈利+企业类型必选; FFR → P1=非盈利+无企业类型+机构去掉 CH
//   3. 企业类型联动:仅股份公司(JOINT_STOCK)可选储备银行(CH),其余三种不可
//   4. 所有必填项已填 + 名称查重通过 → "生成"按钮变绿可点击

import React, { useEffect, useMemo, useState } from 'react';
import { Button, Form, Input, message, Modal, Select, Spin } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import {
  checkInstitutionName,
  createInstitution,
  type CreateInstitutionOutput,
  type InstitutionCategory,
} from '../../api/institution';
import { listSfidCities, type AdminAuth, type SfidCityItem } from '../../api/client';
import { dynamicLocksForA3, institutionChoicesForSubType, locksForCategory } from './locks';

interface Props {
  auth: AdminAuth;
  category: InstitutionCategory;
  open: boolean;
  /** 父视图锁定的省份(ShengAdmin/ShiAdmin 会有值,KeyAdmin 在省详情页也会有值) */
  lockedProvince: string | null;
  /** 父视图锁定的市(ShiAdmin 会有值,其他需要用户在弹窗内选) */
  lockedCity: string | null;
  onCancel: () => void;
  onCreated: (result: CreateInstitutionOutput) => void;
}

interface FormValues {
  a3: string;
  p1: string;
  province: string;
  city: string;
  institution: string;
  institution_name: string;
  sub_type?: string;
}

export const CreateInstitutionModal: React.FC<Props> = ({
  auth,
  category,
  open,
  lockedProvince,
  lockedCity,
  onCancel,
  onCreated,
}) => {
  const locks = locksForCategory(category);
  const [form] = Form.useForm<FormValues>();
  const [cities, setCities] = useState<SfidCityItem[]>([]);
  const [citiesLoading, setCitiesLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  // ── 名称查重状态 ──
  const [nameChecking, setNameChecking] = useState(false);
  // null=未查, true=可用, false=已占用
  const [nameAvailable, setNameAvailable] = useState<boolean | null>(null);

  // ── a3 动态联动(仅 PRIVATE_INSTITUTION 有效) ──
  const [currentA3, setCurrentA3] = useState<string>(locks.a3Choices[0]?.value ?? '');
  const [currentSubType, setCurrentSubType] = useState<string | undefined>(undefined);
  const dynamicLocks = useMemo(() => dynamicLocksForA3(currentA3), [currentA3]);
  const isPrivate = category === 'PRIVATE_INSTITUTION';
  const isPublicGov = category === 'GOV_INSTITUTION';

  // 实际使用的选项:非私权时用 locks 原始值,私权时用 dynamicLocks
  const effectiveP1Choices = isPrivate ? dynamicLocks.p1Choices : locks.p1Choices;
  // SFR 机构代码还需按企业类型(sub_type)细化:仅股份公司可选储备银行
  const effectiveInstChoices = isPrivate
    ? (currentA3 === 'SFR' ? institutionChoicesForSubType(currentSubType) : dynamicLocks.institutionChoices)
    : locks.institutionChoices;
  const effectiveSubTypeChoices = isPrivate ? dynamicLocks.subTypeChoices : [];
  const showSubType = effectiveSubTypeChoices.length > 0;

  // 打开时预填锁定字段
  useEffect(() => {
    if (!open) return;
    const defaultA3 = locks.a3Choices[0]?.value ?? '';
    setCurrentA3(defaultA3);
    setCurrentSubType(undefined);
    setNameAvailable(null);
    const dl = dynamicLocksForA3(defaultA3);
    form.setFieldsValue({
      a3: defaultA3,
      p1: isPrivate ? dl.p1Default : locks.p1Choices[0]?.value,
      province: lockedProvince ?? '',
      city: lockedCity ?? '',
      institution: isPrivate ? dl.institutionChoices[0]?.value : locks.institutionChoices[0]?.value,
      institution_name: locks.lockedInstitutionName ?? '',
      sub_type: isPrivate && defaultA3 === 'SFR' ? undefined : undefined,
    });
  }, [open, category, lockedProvince, lockedCity]);

  // 加载城市列表
  useEffect(() => {
    if (!open || !lockedProvince) return;
    let cancelled = false;
    setCitiesLoading(true);
    listSfidCities(auth, lockedProvince)
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

  // ── a3 切换联动 ──
  const onA3Change = (a3: string) => {
    setCurrentA3(a3);
    setCurrentSubType(undefined);
    const dl = dynamicLocksForA3(a3);
    form.setFieldsValue({
      p1: dl.p1Default,
      institution: dl.institutionChoices[0]?.value,
      sub_type: undefined,
    });
  };

  // ── 企业类型切换联动:更新可选机构代码 ──
  const onSubTypeChange = (subType: string) => {
    setCurrentSubType(subType);
    const choices = institutionChoicesForSubType(subType);
    const currentInst = form.getFieldValue('institution');
    // 如果当前选中的机构代码不在新列表中,回退到第一个
    if (!choices.some((c) => c.value === currentInst)) {
      form.setFieldsValue({ institution: choices[0]?.value });
    }
  };

  // ── 名称查重 ──
  // 公权机构(GFR)按同城查重,私权机构按全国查重
  const onCheckName = async () => {
    const name = (form.getFieldValue('institution_name') ?? '').trim();
    if (!name) {
      message.warning('请先输入机构名称');
      return;
    }
    if (isPublicGov) {
      const city = (form.getFieldValue('city') ?? '').trim();
      if (!city) {
        message.warning('公权机构查重需要先选择市');
        return;
      }
    }
    setNameChecking(true);
    try {
      const a3Val = isPublicGov ? 'GFR' : undefined;
      const cityVal = isPublicGov ? (form.getFieldValue('city') ?? '').trim() : undefined;
      const { exists } = await checkInstitutionName(auth, name, a3Val, cityVal);
      if (exists) {
        message.error(isPublicGov ? '该市已存在同名机构，请更换名称' : '该机构名称已被使用，请更换名称');
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

  // 名称变化时重置查重状态
  const onNameChange = () => {
    if (nameAvailable !== null) {
      setNameAvailable(null);
    }
  };

  // ── 提交 ──
  const onSubmit = async (values: FormValues) => {
    // 非锁定名称的,必须先通过查重
    if (!locks.lockedInstitutionName && nameAvailable !== true) {
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
        institution_name: values.institution_name.trim(),
        sub_type: values.sub_type?.trim() || undefined,
      });
      message.success(`机构已创建:${result.sfid_id}`);
      onCreated(result);
    } catch (err) {
      const raw = err instanceof Error ? err.message : '创建机构失败';
      if (raw.includes('本省') && raw.includes('未在线')) {
        message.error('本省登录管理员未在线,请联系省管理员登录后重试');
      } else if (raw.includes('密钥管理员不能直接推送')) {
        message.error('请以省或市管理员身份操作');
      } else if (raw.includes('已被使用') || raw.includes('同名机构')) {
        message.error(isPublicGov ? '该市已存在同名机构，请更换名称' : '该机构名称已被使用，请更换名称');
        setNameAvailable(false);
      } else {
        message.error(raw);
      }
    } finally {
      setSubmitting(false);
    }
  };

  // ── 按钮可用判断 ──
  const a3Disabled = locks.a3Choices.length === 1;
  const p1Disabled = effectiveP1Choices.length === 1;
  const instDisabled = effectiveInstChoices.length === 1;
  const nameDisabled = locks.lockedInstitutionName !== null;
  // 公安局名称锁定,不需要查重;其他必须查重通过
  const nameCheckPassed = nameDisabled || nameAvailable === true;

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
        {showSubType && (
          <Form.Item
            label="企业类型"
            name="sub_type"
            rules={[{ required: true, message: '请选择企业类型' }]}
          >
            <Select options={effectiveSubTypeChoices} placeholder="请选择企业类型" onChange={onSubTypeChange} />
          </Form.Item>
        )}
        <Form.Item label="省" name="province" rules={[{ required: true }]}>
          <Input disabled />
        </Form.Item>
        <Form.Item label="市" name="city" rules={[{ required: true, message: '请选择市' }]}>
          <Select
            loading={citiesLoading}
            disabled={lockedCity !== null}
            options={cities.map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }))}
            placeholder="请选择市"
            onChange={() => { if (isPublicGov && nameAvailable !== null) setNameAvailable(null); }}
          />
        </Form.Item>
        <Form.Item label="机构" name="institution" rules={[{ required: true }]}>
          <Select options={effectiveInstChoices} disabled={instDisabled} />
        </Form.Item>
        <Form.Item
          label="机构名称"
          name="institution_name"
          rules={[
            { required: true, message: '请输入机构名称' },
            { max: 30, message: '最多 30 个字' },
          ]}
        >
          <Input
            disabled={nameDisabled}
            placeholder="请输入机构名称(最多 30 字)"
            maxLength={30}
            onChange={onNameChange}
            suffix={
              nameDisabled ? null : (
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
      </Form>
    </Modal>
  );
};
