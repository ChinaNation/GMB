// 注册局直接录入公民弹窗。
// 注册局管理员填表后,后端一次性生成身份 CID、护照号和护照有效期。

import { useEffect, useMemo, useState } from 'react';
import { Alert, Button, DatePicker, Form, Input, Modal, Select, Switch } from 'antd';
import type { Dayjs } from 'dayjs';

import type { AdminAuth } from '../auth/types';
import { submitChainSign, useChainSign } from '../core/useChainSign';
import {
  prepareCitizenOccupy,
  type CreateCitizenResult,
  type CitizenSex,
  type CreateCitizenInput,
} from './api';
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
  provinceName: string | null;
  cityName: string | null;
  onClose: () => void;
  /** 录入成功后回填新身份 CID 并刷新列表。 */
  onCreated: (cidNumber: string) => Promise<void> | void;
}

const DATE_FORMAT = 'YYYY-MM-DD';

interface FormValues {
  actor_role_code: string;
  family_name: string;
  given_name: string;
  citizen_sex: CitizenSex;
  citizen_birth_date: Dayjs;
  town_code: string;
  birth_province_code: string;
  birth_city_code: string;
  birth_town_code: string;
  voting_eligible: boolean;
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

export function CitizenCreateModal({
  auth,
  open,
  provinceName,
  cityName,
  onClose,
  onCreated,
}: Props) {
  const [form] = Form.useForm<FormValues>();
  const [submitting, setSubmitting] = useState(false);
  const { signChain, chainSignModal } = useChainSign('注册局占号签名');
  const [birthProvinces, setBirthProvinces] = useState<CidProvinceItem[]>([]);
  const [birthCities, setBirthCities] = useState<CidCityItem[]>([]);
  const [birthTowns, setBirthTowns] = useState<CidTownItem[]>([]);
  const [towns, setTowns] = useState<CidTownItem[]>([]);

  const birthProvinceCode = Form.useWatch('birth_province_code', form);
  const birthCityCode = Form.useWatch('birth_city_code', form);
  const birthDate = Form.useWatch('citizen_birth_date', form);
  const age = useMemo(() => ageAtToday(birthDate), [birthDate]);
  const autoValidityYears = age === null ? null : age >= 16 ? 10 : 5;
  const scopeReady = Boolean(auth && provinceName && cityName);

  useEffect(() => {
    if (!open || !auth) return;
    form.resetFields();
    setBirthCities([]);
    setBirthTowns([]);
    setTowns([]);
    getCidMeta(auth)
      .then(async (meta) => {
        setBirthProvinces(meta.all_provinces?.length ? meta.all_provinces : meta.provinces);
        if (!provinceName || !cityName) return;
        const cities = await listCidCities(auth, provinceName);
        const currentCity = cities.find((city) => city.city_name === cityName);
        if (!currentCity) return;
        const rows = await listCidTowns(auth, provinceName, currentCity.city_code);
        setTowns(rows);
      })
      .catch((err) => notice.error(err, '行政区加载失败'));
  }, [open, auth, provinceName, cityName, form]);

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
      actor_role_code: trimRequired(values.actor_role_code),
      family_name: trimRequired(values.family_name),
      given_name: trimRequired(values.given_name),
      citizen_sex: values.citizen_sex,
      citizen_birth_date: values.citizen_birth_date.format(DATE_FORMAT),
      province_name: provinceName!,
      city_name: cityName!,
      town_code: values.town_code,
      birth_province_code: values.birth_province_code,
      birth_city_code: values.birth_city_code,
      birth_town_code: values.birth_town_code,
      voting_eligible: Boolean(values.voting_eligible),
    };
    setSubmitting(true);
    try {
      const prepared = await prepareCitizenOccupy(auth, payload);
      // 占号先行：管理员 CitizenWallet 只签名一次并展示响应二维码，OnChina 回扫提交，进块后才落档案。
      const signed = await signChain(prepared.request_id, prepared.sign_request);
      const submitted = await submitChainSign<CreateCitizenResult>(
        auth,
        prepared.request_id,
        signed.signer_public_key,
        signed.signature,
      );
      const result = submitted.citizen;
      if (!result) {
        throw new Error('占号已上链,但档案落库结果缺失');
      }
      notice.success(`占号上链成功,护照号：${result.passport_no}`);
      onClose();
      await onCreated(result.cid_number);
    } catch (err) {
      notice.error(err, '公民录入失败');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <>
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
          message="请先选择办理城市后再新增公民"
        />
      )}
      <Form
        form={form}
        layout="vertical"
        onFinish={onSubmit}
        initialValues={{ voting_eligible: true }}
      >
        <Form.Item
          label="注册局岗位码"
          name="actor_role_code"
          rules={[{ required: true, message: '请输入当前任职岗位码' }]}
        >
          <Input placeholder="岗位码" allowClear maxLength={64} />
        </Form.Item>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', columnGap: 16 }}>
          <Form.Item
            label="姓"
            name="family_name"
            rules={[{ required: true, message: '请输入姓' }]}
          >
            <Input placeholder="姓" allowClear />
          </Form.Item>
          <Form.Item
            label="名"
            name="given_name"
            rules={[{ required: true, message: '请输入名' }]}
          >
            <Input placeholder="名" allowClear />
          </Form.Item>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', columnGap: 16 }}>
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

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', columnGap: 16 }}>
          <Form.Item label="居住省">
            <Input readOnly value={provinceName ?? ''} />
          </Form.Item>
          <Form.Item label="居住市">
            <Input readOnly value={cityName ?? ''} />
          </Form.Item>
          <Form.Item
            label="居住镇"
            name="town_code"
            rules={[{ required: true, message: '请选择居住镇' }]}
          >
            <Select
              placeholder="居住镇"
              showSearch
              optionFilterProp="label"
              disabled={!scopeReady}
              options={towns.map((town) => ({
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

        <Form.Item label="选举资格" name="voting_eligible" valuePropName="checked">
          <Switch checkedChildren="有" unCheckedChildren="无" disabled={age !== null && age < 16} />
        </Form.Item>
      </Form>
    </Modal>
    {chainSignModal}
    </>
  );
}
