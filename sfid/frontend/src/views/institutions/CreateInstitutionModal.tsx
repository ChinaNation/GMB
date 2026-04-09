// 中文注释:通用"新增机构"弹窗,按 category 锁字段。
// 提交走任务卡 2 新 API:createInstitution(不上链,只生成 sfid_id)
// 账户创建留给任务卡 5 的机构详情页。

import React, { useEffect, useState } from 'react';
import { Button, Form, Input, message, Modal, Select } from 'antd';
import {
  createInstitution,
  type CreateInstitutionOutput,
  type InstitutionCategory,
} from '../../api/institution';
import { listSfidCities, type AdminAuth, type SfidCityItem } from '../../api/client';
import { locksForCategory } from './locks';

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

  // 打开时预填锁定字段
  useEffect(() => {
    if (!open) return;
    form.setFieldsValue({
      a3: locks.a3Choices[0]?.value,
      p1: locks.p1Choices[0]?.value,
      province: lockedProvince ?? '',
      city: lockedCity ?? '',
      institution: locks.institutionChoices[0]?.value,
      institution_name: locks.lockedInstitutionName ?? '',
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
      .catch(() => {
        if (!cancelled) setCities([]);
      })
      .finally(() => {
        if (!cancelled) setCitiesLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [open, lockedProvince, auth.access_token]);

  const onSubmit = async (values: FormValues) => {
    setSubmitting(true);
    try {
      const result = await createInstitution(auth, {
        a3: values.a3.trim(),
        p1: values.p1?.trim(),
        province: values.province.trim(),
        city: values.city.trim(),
        institution: values.institution.trim(),
        institution_name: values.institution_name.trim(),
      });
      message.success(`机构已创建:${result.sfid_id}`);
      onCreated(result);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '创建机构失败');
    } finally {
      setSubmitting(false);
    }
  };

  const a3Disabled = locks.a3Choices.length === 1;
  const p1Disabled = locks.p1Choices.length === 1;
  const instDisabled = locks.institutionChoices.length === 1;
  const nameDisabled = locks.lockedInstitutionName !== null;

  return (
    <Modal
      title={<div style={{ textAlign: 'center', width: '100%' }}>{locks.modalTitle}</div>}
      open={open}
      onCancel={onCancel}
      footer={[
        <Button key="cancel" onClick={onCancel}>
          取消
        </Button>,
        <Button key="submit" type="primary" loading={submitting} onClick={() => form.submit()}>
          生成
        </Button>,
      ]}
      destroyOnClose
    >
      <Form form={form} layout="vertical" onFinish={onSubmit}>
        <Form.Item label="A3 主体属性" name="a3" rules={[{ required: true }]}>
          <Select options={locks.a3Choices} disabled={a3Disabled} />
        </Form.Item>
        <Form.Item label="P1 盈利属性" name="p1" rules={[{ required: true }]}>
          <Select options={locks.p1Choices} disabled={p1Disabled} />
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
          />
        </Form.Item>
        <Form.Item label="机构" name="institution" rules={[{ required: true }]}>
          <Select options={locks.institutionChoices} disabled={instDisabled} />
        </Form.Item>
        <Form.Item
          label="机构名称"
          name="institution_name"
          rules={[
            { required: true, message: '请输入机构名称' },
            { max: 30, message: '最多 30 个字' },
          ]}
        >
          <Input disabled={nameDisabled} placeholder="请输入机构名称(最多 30 字)" maxLength={30} />
        </Form.Item>
      </Form>
    </Modal>
  );
};
