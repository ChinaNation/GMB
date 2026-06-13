// 新增市管理员 Modal
// 当 selectedCity 有值时，城市字段预填并锁定（已在某市详情页内新增）

import { useEffect } from 'react';
import { Button, Form, Input, Modal, Select } from 'antd';
import { decodeSs58 } from '../utils/ss58';
import { ScanAccountModal } from '../core/ScanAccountModal';
import { SFID_MODAL_Z_INDEX } from '../core/modalStack';
import { MAX_CITY_ADMINS_PER_CITY, type FederalAdminSharedState } from './adminUtils';

interface AddCityAdminModalProps {
  state: FederalAdminSharedState;
}

export function AddCityAdminModal({ state }: AddCityAdminModalProps) {
  const {
    addCityAdminOpen,
    setAddCityAdminOpen,
    addCityAdminLoading,
    addCityAdminForm,
    cityAdmins,
    cityAdminCities,
    cityAdminCitiesLoading,
    selectedCity,
    onCreateCityAdmin,
    accountScanTarget,
    setAccountScanTarget,
  } = state;
  const selectedCityAdminCity = Form.useWatch('city_admin_city', addCityAdminForm);
  // 中文注释:新增弹窗只做提前拦截,单市 30 人上限最终以后端校验为准。
  const cityCityAdminCount = (city: string) => cityAdmins.filter((item) => item.city === city).length;
  const selectedCityLimitReached = selectedCityAdminCity
    ? cityCityAdminCount(selectedCityAdminCity) >= MAX_CITY_ADMINS_PER_CITY
    : false;

  // selectedCity 有值时预填城市字段
  useEffect(() => {
    if (addCityAdminOpen && selectedCity) {
      addCityAdminForm.setFieldsValue({ city_admin_city: selectedCity });
    }
  }, [addCityAdminOpen, selectedCity, addCityAdminForm]);

  return (
    <>
      <Modal
        title={<div style={{ textAlign: 'center', width: '100%' }}>新增市管理员</div>}
        open={addCityAdminOpen}
        onCancel={() => {
          if (addCityAdminLoading) return;
          addCityAdminForm.resetFields();
          setAddCityAdminOpen(false);
        }}
        footer={[
          <Button
            key="cancel"
            disabled={addCityAdminLoading}
            onClick={() => {
              addCityAdminForm.resetFields();
              setAddCityAdminOpen(false);
            }}
          >
            取消新增
          </Button>,
          <Button
            key="submit"
            type="primary"
            loading={addCityAdminLoading}
            disabled={selectedCityLimitReached}
            title={selectedCityLimitReached ? `本市市管理员已满 ${MAX_CITY_ADMINS_PER_CITY} 人` : undefined}
            onClick={() => addCityAdminForm.submit()}
          >
            确认新增
          </Button>,
        ]}
        destroyOnClose
        closable={!addCityAdminLoading}
        maskClosable={!addCityAdminLoading}
        zIndex={SFID_MODAL_Z_INDEX.business}
      >
        <Form
          form={addCityAdminForm}
          layout="vertical"
          onFinish={(values: { city_admin_name: string; city_admin_pubkey: string; city_admin_city: string }) =>
            onCreateCityAdmin({
              city_admin_name: values.city_admin_name,
              city_admin_pubkey: values.city_admin_pubkey,
              city: values.city_admin_city,
            })
          }
        >
          <Form.Item
            label="姓名"
            name="city_admin_name"
            rules={[{ required: true, message: '请输入市管理员姓名' }]}
          >
            <Input placeholder="请输入市管理员姓名" />
          </Form.Item>
          <Form.Item
            label="市"
            name="city_admin_city"
            rules={[
              { required: true, message: '请选择市' },
              {
                validator: async (_rule, value) => {
                  if (!value) return;
                  if (cityCityAdminCount(String(value)) >= MAX_CITY_ADMINS_PER_CITY) {
                    throw new Error(`本市市管理员已满 ${MAX_CITY_ADMINS_PER_CITY} 人`);
                  }
                },
              },
            ]}
          >
            <Select
              placeholder="请选择市"
              loading={cityAdminCitiesLoading}
              disabled={selectedCity !== null}
              options={cityAdminCities
                .filter((c) => c.code !== '000')
                .map((c) => {
                  const count = cityCityAdminCount(c.name);
                  return {
                    label: `${c.name} (${c.code}) ${count}/${MAX_CITY_ADMINS_PER_CITY}`,
                    value: c.name,
                    disabled: count >= MAX_CITY_ADMINS_PER_CITY,
                  };
                })}
            />
          </Form.Item>
          <Form.Item
            label="账户"
            name="city_admin_pubkey"
            rules={[
              { required: true, message: '请输入市管理员账户' },
              {
                validator: async (_rule, value) => {
                  if (!value) return;
                  try {
                    decodeSs58(String(value));
                  } catch (err) {
                    throw new Error(err instanceof Error ? err.message : '账户格式无效');
                  }
                },
              },
            ]}
          >
            <Input
              placeholder="请输入市管理员账户(SS58)"
              suffix={
                <span
                  title="扫码识别用户码"
                  style={{ cursor: 'pointer', display: 'inline-flex', color: '#0d9488' }}
                  onClick={() => setAccountScanTarget('city_admin')}
                >
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M3 7V5a2 2 0 0 1 2-2h2" />
                    <path d="M17 3h2a2 2 0 0 1 2 2v2" />
                    <path d="M21 17v2a2 2 0 0 1-2 2h-2" />
                    <path d="M7 21H5a2 2 0 0 1-2-2v-2" />
                    <rect x="7" y="7" width="10" height="10" rx="1" />
                  </svg>
                </span>
              }
            />
          </Form.Item>
        </Form>
      </Modal>

      <ScanAccountModal
        open={accountScanTarget !== null}
        onClose={() => setAccountScanTarget(null)}
        onResolved={(addr) => {
          if (accountScanTarget === 'city_admin') {
            addCityAdminForm.setFieldsValue({ city_admin_pubkey: addr });
          }
          setAccountScanTarget(null);
        }}
      />
    </>
  );
}
