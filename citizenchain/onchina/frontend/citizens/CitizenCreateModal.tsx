// 中文注释:注册局直接录入公民弹窗。
// 注册局管理员填表 -> 提交 POST /api/v1/admin/citizens -> 成功即已发护照(NORMAL + 有效期)。
// 中文注释:公民护照身份即 cid_number。
//
// 行政区取数:省/市走现有 china API(getCidMeta 提供省 + listCidCities 提供市,均含 code);
// 镇无后端选择端点,DTO 接受可选镇代码,故镇用可选文本输入。

import { useEffect, useState } from 'react';
import { Button, DatePicker, Form, Input, Modal, Select, Switch } from 'antd';
import type { Dayjs } from 'dayjs';

import type { AdminAuth } from '../auth/types';
import {
  createCitizen,
  type CreateCitizenInput,
  type ElectionScopeLevel,
} from './api';
import {
  getCidMeta,
  listCidCities,
  type CidCityItem,
  type CidProvinceItem,
} from '../china/api';
import { notice } from '../utils/notice';

interface Props {
  auth: AdminAuth | null;
  open: boolean;
  onClose: () => void;
  /** 录入成功后回填新身份ID并刷新列表。 */
  onCreated: (cidNumber: string) => Promise<void> | void;
}

const DATE_FORMAT = 'YYYY-MM-DD';

const ELECTION_SCOPE_OPTIONS: Array<{ value: ElectionScopeLevel; label: string }> = [
  { value: 'PROVINCE', label: '省级' },
  { value: 'CITY', label: '市级' },
  { value: 'TOWN', label: '镇级' },
];

interface FormValues {
  cid_number: string;
  residence_province_code: string;
  residence_city_code?: string;
  residence_town_code?: string;
  birth_province_code: string;
  birth_city_code?: string;
  birth_town_code?: string;
  voting_eligible: boolean;
  election_scope_level: ElectionScopeLevel;
  valid_range: [Dayjs, Dayjs];
  wallet_pubkey?: string;
  wallet_address?: string;
}

export function CitizenCreateModal({ auth, open, onClose, onCreated }: Props) {
  const [form] = Form.useForm<FormValues>();
  const [submitting, setSubmitting] = useState(false);

  const [provinces, setProvinces] = useState<CidProvinceItem[]>([]);
  const [residenceCities, setResidenceCities] = useState<CidCityItem[]>([]);
  const [birthCities, setBirthCities] = useState<CidCityItem[]>([]);

  const residenceProvinceCode = Form.useWatch('residence_province_code', form);
  const birthProvinceCode = Form.useWatch('birth_province_code', form);

  // 弹窗打开:重置表单并加载省份。
  useEffect(() => {
    if (!open || !auth) return;
    form.resetFields();
    setResidenceCities([]);
    setBirthCities([]);
    getCidMeta(auth)
      .then((meta) => setProvinces(meta.provinces))
      .catch((err) => notice.error(err, '省份加载失败'));
  }, [open, auth, form]);

  // 居住地省份变化:按 province_name 拉取城市(含 city_code)。
  useEffect(() => {
    if (!auth || !residenceProvinceCode) {
      setResidenceCities([]);
      return;
    }
    const province = provinces.find((p) => p.province_code === residenceProvinceCode);
    if (!province) return;
    listCidCities(auth, province.province_name)
      .then(setResidenceCities)
      .catch((err) => notice.error(err, '居住地城市加载失败'));
  }, [auth, residenceProvinceCode, provinces]);

  // 出生地省份变化:同上。
  useEffect(() => {
    if (!auth || !birthProvinceCode) {
      setBirthCities([]);
      return;
    }
    const province = provinces.find((p) => p.province_code === birthProvinceCode);
    if (!province) return;
    listCidCities(auth, province.province_name)
      .then(setBirthCities)
      .catch((err) => notice.error(err, '出生地城市加载失败'));
  }, [auth, birthProvinceCode, provinces]);

  const onResidenceProvinceChange = () => {
    form.setFieldsValue({ residence_city_code: undefined });
  };
  const onBirthProvinceChange = () => {
    form.setFieldsValue({ birth_city_code: undefined });
  };

  const trimOptional = (value?: string): string | undefined => {
    const trimmed = value?.trim();
    return trimmed ? trimmed : undefined;
  };

  const onSubmit = async (values: FormValues) => {
    if (!auth) {
      notice.error('请先登录');
      return;
    }
    const [from, until] = values.valid_range;
    const payload: CreateCitizenInput = {
      cid_number: values.cid_number.trim(),
      residence_province_code: values.residence_province_code,
      residence_city_code: trimOptional(values.residence_city_code),
      residence_town_code: trimOptional(values.residence_town_code),
      birth_province_code: values.birth_province_code,
      birth_city_code: trimOptional(values.birth_city_code),
      birth_town_code: trimOptional(values.birth_town_code),
      voting_eligible: values.voting_eligible,
      election_scope_level: values.election_scope_level,
      valid_from: from.format(DATE_FORMAT),
      valid_until: until.format(DATE_FORMAT),
      wallet_pubkey: trimOptional(values.wallet_pubkey),
      wallet_address: trimOptional(values.wallet_address),
    };
    setSubmitting(true);
    try {
      const result = await createCitizen(auth, payload);
      notice.success(`公民录入成功,已发护照,身份ID：${result.cid_number}`);
      onClose();
      await onCreated(result.cid_number);
    } catch (err) {
      notice.error(err, '公民录入失败');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      title={<div style={{ textAlign: 'center', width: '100%' }}>新增公民</div>}
      open={open}
      onCancel={onClose}
      destroyOnClose
      width={640}
      footer={[
        <Button key="cancel" onClick={onClose}>
          取消
        </Button>,
        <Button key="submit" type="primary" loading={submitting} onClick={() => form.submit()}>
          {submitting ? '提交中...' : '录入并发护照'}
        </Button>,
      ]}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={onSubmit}
        initialValues={{ voting_eligible: true, election_scope_level: 'PROVINCE' }}
      >
        <Form.Item
          label="身份ID"
          name="cid_number"
          rules={[{ required: true, message: '请输入公民身份ID' }]}
        >
          <Input placeholder="公民身份ID(护照身份)" allowClear />
        </Form.Item>

        <Form.Item
          label="居住地省份"
          name="residence_province_code"
          rules={[{ required: true, message: '请选择居住地省份' }]}
        >
          <Select
            placeholder="选择省份"
            showSearch
            optionFilterProp="label"
            onChange={onResidenceProvinceChange}
            options={provinces.map((p) => ({ value: p.province_code, label: p.province_name }))}
          />
        </Form.Item>
        <Form.Item label="居住地城市(可选)" name="residence_city_code">
          <Select
            placeholder="选择城市"
            allowClear
            showSearch
            optionFilterProp="label"
            disabled={!residenceProvinceCode}
            options={residenceCities.map((c) => ({ value: c.city_code, label: c.city_name }))}
          />
        </Form.Item>
        <Form.Item label="居住地镇代码(可选)" name="residence_town_code">
          <Input placeholder="镇级行政区代码" allowClear />
        </Form.Item>

        <Form.Item
          label="出生地省份"
          name="birth_province_code"
          rules={[{ required: true, message: '请选择出生地省份' }]}
        >
          <Select
            placeholder="选择省份"
            showSearch
            optionFilterProp="label"
            onChange={onBirthProvinceChange}
            options={provinces.map((p) => ({ value: p.province_code, label: p.province_name }))}
          />
        </Form.Item>
        <Form.Item label="出生地城市(可选)" name="birth_city_code">
          <Select
            placeholder="选择城市"
            allowClear
            showSearch
            optionFilterProp="label"
            disabled={!birthProvinceCode}
            options={birthCities.map((c) => ({ value: c.city_code, label: c.city_name }))}
          />
        </Form.Item>
        <Form.Item label="出生地镇代码(可选)" name="birth_town_code">
          <Input placeholder="镇级行政区代码" allowClear />
        </Form.Item>

        <Form.Item label="选举资格" name="voting_eligible" valuePropName="checked">
          <Switch checkedChildren="有" unCheckedChildren="无" />
        </Form.Item>
        <Form.Item
          label="选举范围层级"
          name="election_scope_level"
          rules={[{ required: true, message: '请选择选举范围层级' }]}
        >
          <Select options={ELECTION_SCOPE_OPTIONS} />
        </Form.Item>

        <Form.Item
          label="护照有效期"
          name="valid_range"
          rules={[{ required: true, message: '请选择护照有效期' }]}
        >
          <DatePicker.RangePicker style={{ width: '100%' }} format={DATE_FORMAT} />
        </Form.Item>

        <Form.Item label="投票账户地址(可选)" name="wallet_address">
          <Input placeholder="SS58 地址,prefix=2027" allowClear />
        </Form.Item>
        <Form.Item label="投票账户公钥(可选)" name="wallet_pubkey">
          <Input placeholder="0x 开头的公钥 hex" allowClear />
        </Form.Item>
      </Form>
    </Modal>
  );
}
