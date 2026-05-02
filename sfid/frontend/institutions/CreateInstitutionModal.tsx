// 中文注释:通用"新增机构"弹窗。两步式创建(2026-04-19 改造)。
//
// ─── 第一步(本弹窗) ───
//   私权(SFR/FFR):**只生成 SFID**,不输入 institution_name / sub_type。
//     字段:A3、P1、省、市、机构代码。提交后跳转到机构详情页。
//   公权(GFR)公安局/机构:保持原流程(含 institution_name + 同城查重),
//     本次改造范围仅限私权,下一步再做两步式改造。
//
// ─── 第二步(详情页) ───
//   在 PrivateInstitutionLayout "完善机构信息" Card 中设置机构名称、
//   企业类型(SFR)等可变字段。名称全国唯一,保存时后端查重。

import React, { useEffect, useMemo, useState } from 'react';
import { Button, Form, Input, message, Modal, Select, Spin } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import {
  checkInstitutionName,
  createInstitution,
  type CreateInstitutionOutput,
  type InstitutionCategory,
} from './api';
import type { AdminAuth } from '../auth/types';
import { listSfidCities, type SfidCityItem } from '../sfid/api';
import { dynamicLocksForA3, locksForCategory } from './locks';

interface Props {
  auth: AdminAuth;
  category: InstitutionCategory;
  open: boolean;
  /** 父视图锁定的省份(ShengAdmin/ShiAdmin 都有值;ADR-008 起 KeyAdmin 已删) */
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
  /** 仅公权(GFR)使用;私权不填 */
  institution_name?: string;
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

  // 公权机构名称查重状态(私权不再在此弹窗查重)
  const [nameChecking, setNameChecking] = useState(false);
  const [nameAvailable, setNameAvailable] = useState<boolean | null>(null);

  // ── a3 动态联动(仅 PRIVATE_INSTITUTION 有效) ──
  const [currentA3, setCurrentA3] = useState<string>(locks.a3Choices[0]?.value ?? '');
  const dynamicLocks = useMemo(() => dynamicLocksForA3(currentA3), [currentA3]);
  const isPrivate = category === 'PRIVATE_INSTITUTION';
  const isPublicGov = category === 'GOV_INSTITUTION';
  const isPublicSecurity = category === 'PUBLIC_SECURITY';

  // 是否在本弹窗中收集机构名称:仅公权分支(公安局/公权机构)
  const collectNameInModal = !isPrivate;

  // 实际使用的选项:非私权时用 locks 原始值,私权时用 dynamicLocks
  const effectiveP1Choices = isPrivate ? dynamicLocks.p1Choices : locks.p1Choices;
  const effectiveInstChoices = isPrivate ? dynamicLocks.institutionChoices : locks.institutionChoices;

  // 打开时预填锁定字段
  useEffect(() => {
    if (!open) return;
    const defaultA3 = locks.a3Choices[0]?.value ?? '';
    setCurrentA3(defaultA3);
    setNameAvailable(null);
    const dl = dynamicLocksForA3(defaultA3);
    form.setFieldsValue({
      a3: defaultA3,
      p1: isPrivate ? dl.p1Default : locks.p1Choices[0]?.value,
      province: lockedProvince ?? '',
      city: lockedCity ?? '',
      institution: isPrivate ? dl.institutionChoices[0]?.value : locks.institutionChoices[0]?.value,
      institution_name: collectNameInModal ? (locks.lockedInstitutionName ?? '') : undefined,
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

  // ── a3 切换联动(仅私权) ──
  const onA3Change = (a3: string) => {
    setCurrentA3(a3);
    const dl = dynamicLocksForA3(a3);
    form.setFieldsValue({
      p1: dl.p1Default,
      institution: dl.institutionChoices[0]?.value,
    });
  };

  // ── 公权机构名称查重:公安局名称锁定;公权机构同城唯一 ──
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
      const cityVal = isPublicGov ? (form.getFieldValue('city') ?? '').trim() : undefined;
      const { exists } = await checkInstitutionName(auth, name, 'GFR', cityVal);
      if (exists) {
        message.error('该市已存在同名机构，请更换名称');
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

  // ── 提交 ──
  const onSubmit = async (values: FormValues) => {
    // 公权分支:名称必须查重通过(公安局名称锁定视为已通过)
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
        // 私权两步式:第一步不提交 institution_name,由详情页补填
        institution_name: collectNameInModal
          ? (values.institution_name ?? '').trim()
          : undefined,
      });
      if (isPrivate) {
        message.success(`机构 SFID 已生成,请到详情页完善信息:${result.sfid_id}`);
      } else {
        message.success(`机构已创建:${result.sfid_id}`);
      }
      onCreated(result);
    } catch (err) {
      const raw = err instanceof Error ? err.message : '创建机构失败';
      if (raw.includes('本省') && raw.includes('未在线')) {
        message.error('本省登录管理员未在线,请联系省管理员登录后重试');
      } else if (raw.includes('密钥管理员不能直接推送')) {
        message.error('请以省或市管理员身份操作');
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

  // ── 按钮可用判断 ──
  const a3Disabled = locks.a3Choices.length === 1;
  const p1Disabled = effectiveP1Choices.length === 1;
  const instDisabled = effectiveInstChoices.length === 1;
  const nameDisabled = locks.lockedInstitutionName !== null;
  // 私权无需在此查重;公权名称锁定或查重通过即可
  const nameCheckPassed = !collectNameInModal || nameDisabled || nameAvailable === true;

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
            onChange={() => { if (isPublicGov && nameAvailable !== null) setNameAvailable(null); }}
          />
        </Form.Item>
        <Form.Item label="机构" name="institution" rules={[{ required: true }]}>
          <Select options={effectiveInstChoices} disabled={instDisabled} />
        </Form.Item>
        {collectNameInModal && (
          <>
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
        {isPrivate && (
          <div style={{ color: '#888', fontSize: 12, marginTop: -8 }}>
            提示:本步骤仅生成机构 SFID。生成后请在详情页设置机构名称、企业类型等信息。
          </div>
        )}
      </Form>
    </Modal>
  );
};
