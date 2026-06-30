// 中文注释:注册局直接录入公民弹窗。
// 注册局管理员填表后,后端一次性生成身份 CID、护照号和护照有效期。

import { useEffect, useMemo, useState } from 'react';
import { Alert, Button, DatePicker, Form, Input, Modal, Select, Switch, Typography } from 'antd';
import type { Dayjs } from 'dayjs';

import type { AdminAuth } from '../auth/types';
import { createCitizen, type CitizenSex, type CreateCitizenInput } from './api';
import {
  getCidMeta,
  listCidCities,
  listCidTowns,
  type CidCityItem,
  type CidProvinceItem,
  type CidTownItem,
} from '../china/api';
import { notice } from '../utils/notice';

interface Props {
  auth: AdminAuth | null;
  open: boolean;
  onClose: () => void;
  /** 录入成功后回填新身份 CID 并刷新列表。 */
  onCreated: (cidNumber: string) => Promise<void> | void;
}

const DATE_FORMAT = 'YYYY-MM-DD';

interface FormValues {
  citizen_full_name: string;
  citizen_sex: CitizenSex;
  citizen_birth_date: Dayjs;
  residence_town_code: string;
  birth_province_code: string;
  birth_city_code: string;
  birth_town_code: string;
  voting_eligible: boolean;
  wallet_account: string;
}

function trimRequired(value?: string): string {
  return value?.trim() ?? '';
}

function ageAtToday(birth?: Dayjs): number | null {
  if (!birth) return null;
  const today = new Date();
  let age = today.getFullYear() - birth.year();
  const month = today.getMonth() + 1;
  const day = today.getDate();
  if (month < birth.month() + 1 || (month === birth.month() + 1 && day < birth.date())) {
    age -= 1;
  }
  return age;
}

export function CitizenCreateModal({ auth, open, onClose, onCreated }: Props) {
  const [form] = Form.useForm<FormValues>();
  const [submitting, setSubmitting] = useState(false);
  const [birthProvinces, setBirthProvinces] = useState<CidProvinceItem[]>([]);
  const [birthCities, setBirthCities] = useState<CidCityItem[]>([]);
  const [birthTowns, setBirthTowns] = useState<CidTownItem[]>([]);
  const [residenceTowns, setResidenceTowns] = useState<CidTownItem[]>([]);

  const birthProvinceCode = Form.useWatch('birth_province_code', form);
  const birthCityCode = Form.useWatch('birth_city_code', form);
  const birthDate = Form.useWatch('citizen_birth_date', form);
  const age = useMemo(() => ageAtToday(birthDate), [birthDate]);
  const autoValidityYears = age === null ? null : age >= 16 ? 10 : 5;
  const scopeReady = Boolean(auth?.scope_province_name && auth?.scope_city_name);

  useEffect(() => {
    if (!open || !auth) return;
    form.resetFields();
    setBirthCities([]);
    setBirthTowns([]);
    setResidenceTowns([]);
    getCidMeta(auth)
      .then(async (meta) => {
        setBirthProvinces(meta.all_provinces?.length ? meta.all_provinces : meta.provinces);
        const scopeProvince = auth.scope_province_name?.trim();
        const scopeCity = auth.scope_city_name?.trim();
        if (!scopeProvince || !scopeCity) return;
        const cities = await listCidCities(auth, scopeProvince);
        const currentCity = cities.find((city) => city.city_name === scopeCity);
        if (!currentCity) return;
        const towns = await listCidTowns(auth, scopeProvince, currentCity.city_code);
        setResidenceTowns(towns);
      })
      .catch((err) => notice.error(err, '行政区加载失败'));
  }, [open, auth, form]);

  useEffect(() => {
    if (!auth || !birthProvinceCode) {
      setBirthCities([]);
      setBirthTowns([]);
      return;
    }
    const province = birthProvinces.find((p) => p.province_code === birthProvinceCode);
    if (!province) return;
    listCidCities(auth, province.province_name)
      .then((rows) => {
        setBirthCities(rows);
        setBirthTowns([]);
        form.setFieldsValue({ birth_city_code: undefined, birth_town_code: undefined });
      })
      .catch((err) => notice.error(err, '出生城市加载失败'));
  }, [auth, birthProvinceCode, birthProvinces, form]);

  useEffect(() => {
    if (!auth || !birthProvinceCode || !birthCityCode) {
      setBirthTowns([]);
      return;
    }
    const province = birthProvinces.find((p) => p.province_code === birthProvinceCode);
    if (!province) return;
    listCidTowns(auth, province.province_name, birthCityCode)
      .then((rows) => {
        setBirthTowns(rows);
        form.setFieldsValue({ birth_town_code: undefined });
      })
      .catch((err) => notice.error(err, '出生镇加载失败'));
  }, [auth, birthProvinceCode, birthCityCode, birthProvinces, form]);

  useEffect(() => {
    if (age !== null && age < 16) {
      form.setFieldsValue({ voting_eligible: false });
    }
  }, [age, form]);

  const onSubmit = async (values: FormValues) => {
    if (!auth) {
      notice.error('请先登录');
      return;
    }
    if (!scopeReady) {
      notice.error('当前登录缺少办理城市');
      return;
    }
    const payload: CreateCitizenInput = {
      citizen_full_name: trimRequired(values.citizen_full_name),
      citizen_sex: values.citizen_sex,
      citizen_birth_date: values.citizen_birth_date.format(DATE_FORMAT),
      residence_town_code: values.residence_town_code,
      birth_province_code: values.birth_province_code,
      birth_city_code: values.birth_city_code,
      birth_town_code: values.birth_town_code,
      voting_eligible: Boolean(values.voting_eligible),
      wallet_account: trimRequired(values.wallet_account),
    };
    setSubmitting(true);
    try {
      const result = await createCitizen(auth, payload);
      notice.success(`公民录入成功,护照号：${result.passport_no}`);
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
      width={680}
      footer={[
        <Button key="cancel" onClick={onClose}>
          取消
        </Button>,
        <Button
          key="submit"
          type="primary"
          loading={submitting}
          disabled={!scopeReady}
          onClick={() => form.submit()}
        >
          {submitting ? '提交中...' : '录入并发护照'}
        </Button>,
      ]}
    >
      {!scopeReady && (
        <Alert
          type="warning"
          showIcon
          style={{ marginBottom: 16 }}
          message="当前登录未绑定办理城市，不能新增公民"
        />
      )}
      <Form
        form={form}
        layout="vertical"
        onFinish={onSubmit}
        initialValues={{ voting_eligible: true }}
      >
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', columnGap: 16 }}>
          <Form.Item
            label="姓名"
            name="citizen_full_name"
            rules={[{ required: true, message: '请输入姓名' }]}
          >
            <Input placeholder="姓名" allowClear />
          </Form.Item>
          <Form.Item
            label="性别"
            name="citizen_sex"
            rules={[{ required: true, message: '请选择性别' }]}
          >
            <Select
              placeholder="性别"
              options={[
                { value: 'MALE', label: '男' },
                { value: 'FEMALE', label: '女' },
              ]}
            />
          </Form.Item>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', columnGap: 16 }}>
          <Form.Item
            label="出生日期"
            name="citizen_birth_date"
            rules={[{ required: true, message: '请选择出生日期' }]}
          >
            <DatePicker style={{ width: '100%' }} format={DATE_FORMAT} />
          </Form.Item>
          <Form.Item label="护照有效期">
            <Input
              readOnly
              value={autoValidityYears ? `${autoValidityYears}年` : ''}
              placeholder="自动生成"
            />
          </Form.Item>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', columnGap: 16 }}>
          <Form.Item label="办理省市">
            <Input
              readOnly
              value={[auth?.scope_province_name, auth?.scope_city_name].filter(Boolean).join(' / ')}
            />
          </Form.Item>
          <Form.Item
            label="居住镇"
            name="residence_town_code"
            rules={[{ required: true, message: '请选择居住镇' }]}
          >
            <Select
              placeholder="居住镇"
              showSearch
              optionFilterProp="label"
              disabled={!scopeReady}
              options={residenceTowns.map((town) => ({
                value: town.town_code,
                label: town.town_name,
              }))}
            />
          </Form.Item>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', columnGap: 16 }}>
          <Form.Item
            label="出生省"
            name="birth_province_code"
            rules={[{ required: true, message: '请选择出生省' }]}
          >
            <Select
              placeholder="出生省"
              showSearch
              optionFilterProp="label"
              options={birthProvinces.map((p) => ({ value: p.province_code, label: p.province_name }))}
            />
          </Form.Item>
          <Form.Item
            label="出生市"
            name="birth_city_code"
            rules={[{ required: true, message: '请选择出生市' }]}
          >
            <Select
              placeholder="出生市"
              showSearch
              optionFilterProp="label"
              disabled={!birthProvinceCode}
              options={birthCities.map((city) => ({ value: city.city_code, label: city.city_name }))}
            />
          </Form.Item>
          <Form.Item
            label="出生镇"
            name="birth_town_code"
            rules={[{ required: true, message: '请选择出生镇' }]}
          >
            <Select
              placeholder="出生镇"
              showSearch
              optionFilterProp="label"
              disabled={!birthCityCode}
              options={birthTowns.map((town) => ({ value: town.town_code, label: town.town_name }))}
            />
          </Form.Item>
        </div>

        <Form.Item
          label="投票账户"
          name="wallet_account"
          rules={[{ required: true, message: '请输入或扫码填入投票账户' }]}
        >
          <Input placeholder="SS58 地址" allowClear />
        </Form.Item>

        <Form.Item label="选举资格" name="voting_eligible" valuePropName="checked">
          <Switch checkedChildren="有" unCheckedChildren="无" disabled={age !== null && age < 16} />
        </Form.Item>
        <Typography.Text type="secondary">
          {age !== null && age < 16 ? '未满16周岁，选举资格自动为无。' : ''}
        </Typography.Text>
      </Form>
    </Modal>
  );
}
