// 新增市级管理员 Modal
// 当 selectedCity 有值时，城市字段预填并锁定（已在某市详情页内新增）

import { useEffect } from 'react';
import { Button, Form, Input, Modal, Select } from 'antd';
import { decodeSs58 } from '../utils/ss58';
import { ScanAccountModal } from '../common/ScanAccountModal';
import type { ShengAdminSharedState } from './shengAdminUtils';

interface AddOperatorModalProps {
  state: ShengAdminSharedState;
}

export function AddOperatorModal({ state }: AddOperatorModalProps) {
  const {
    addOperatorOpen,
    setAddOperatorOpen,
    addOperatorLoading,
    addOperatorForm,
    operatorCities,
    operatorCitiesLoading,
    selectedShengAdmin,
    selectedCity,
    onCreateOperator,
    accountScanTarget,
    setAccountScanTarget,
  } = state;

  // selectedCity 有值时预填城市字段
  useEffect(() => {
    if (addOperatorOpen && selectedCity) {
      addOperatorForm.setFieldsValue({ operator_city: selectedCity });
    }
  }, [addOperatorOpen, selectedCity, addOperatorForm]);

  return (
    <>
      <Modal
        title={<div style={{ textAlign: 'center', width: '100%' }}>新增市级管理员</div>}
        open={addOperatorOpen}
        onCancel={() => {
          addOperatorForm.resetFields();
          setAddOperatorOpen(false);
        }}
        footer={[
          <Button
            key="cancel"
            onClick={() => {
              addOperatorForm.resetFields();
              setAddOperatorOpen(false);
            }}
          >
            取消新增
          </Button>,
          <Button
            key="submit"
            type="primary"
            loading={addOperatorLoading}
            onClick={() => addOperatorForm.submit()}
          >
            确认新增
          </Button>,
        ]}
        destroyOnClose
      >
        <Form
          form={addOperatorForm}
          layout="vertical"
          onFinish={(values: { operator_name: string; operator_pubkey: string; operator_city: string }) =>
            onCreateOperator({
              operator_name: values.operator_name,
              operator_pubkey: values.operator_pubkey,
              city: values.operator_city,
              created_by: selectedShengAdmin?.admin_pubkey,
            })
          }
        >
          <Form.Item
            label="姓名"
            name="operator_name"
            rules={[{ required: true, message: '请输入市级管理员姓名' }]}
          >
            <Input placeholder="请输入市级管理员姓名" />
          </Form.Item>
          <Form.Item
            label="市"
            name="operator_city"
            rules={[{ required: true, message: '请选择市' }]}
          >
            <Select
              placeholder="请选择市"
              loading={operatorCitiesLoading}
              disabled={selectedCity !== null}
              options={operatorCities
                .filter((c) => c.code !== '000')
                .map((c) => ({ label: `${c.name} (${c.code})`, value: c.name }))}
            />
          </Form.Item>
          <Form.Item
            label="账户"
            name="operator_pubkey"
            rules={[
              { required: true, message: '请输入市级管理员账户' },
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
              placeholder="请输入市级管理员账户(SS58)"
              suffix={
                <span
                  title="扫码识别用户码"
                  style={{ cursor: 'pointer', display: 'inline-flex', color: '#0d9488' }}
                  onClick={() => setAccountScanTarget('operator')}
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
          if (accountScanTarget === 'operator') {
            addOperatorForm.setFieldsValue({ operator_pubkey: addr });
          }
          setAccountScanTarget(null);
        }}
      />
    </>
  );
}
