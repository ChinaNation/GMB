// 中文注释:新建账户弹窗。
//
// SFID 账户创建规则:
//   - 创建后只登记 `account_name`,**不立即上链**
//   - 链上注册由区块链软件发起,成功后同步状态回 SFID
//   - 链上派生公式由链端按 `account_name` 路由(Role::Main / Role::Fee / Role::Named),
//     sfid 前端不做地址预览(避免和链端公式漂移)
//
// 约束:
//   - 同一 sfid_id 下 account_name 唯一(后端硬校验,前端做一次即时预校验)
//   - "主账户" / "费用账户" 是默认账户(创建机构时已自动生成),这里手工建名不能重复

import React, { useEffect, useState } from 'react';
import { Button, Form, Input, message, Modal, Typography } from 'antd';
import { createAccount, type MultisigAccount } from '../api/institution';
import type { AdminAuth } from '../api/client';

interface Props {
  auth: AdminAuth;
  sfidId: string;
  institutionName: string;
  /** 当前已有的账户列表(用于前端唯一性预校验) */
  existingAccounts: MultisigAccount[];
  open: boolean;
  onCancel: () => void;
  onCreated: () => void;
}

interface FormValues {
  account_name: string;
}

export const CreateAccountModal: React.FC<Props> = ({
  auth,
  sfidId,
  institutionName,
  existingAccounts,
  open,
  onCancel,
  onCreated,
}) => {
  const [form] = Form.useForm<FormValues>();
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (open) {
      form.resetFields();
    }
  }, [open]);

  const onSubmit = async (values: FormValues) => {
    const name = values.account_name.trim();
    if (!name) {
      message.error('账户名称不能为空');
      return;
    }
    if (name.length > 30) {
      message.error('账户名称最多 30 字');
      return;
    }
    // 前端预校验:同 sfid 下不能重名(含已自动生成的主账户/费用账户)
    if (existingAccounts.some((a) => a.account_name === name)) {
      message.error(`账户名称"${name}"在本机构下已存在`);
      return;
    }
    setSubmitting(true);
    try {
      await createAccount(auth, sfidId, name);
      message.success('账户名称已创建,链上注册后会自动同步状态');
      onCreated();
    } catch (err) {
      const raw = err instanceof Error ? err.message : '创建账户失败';
      message.error(raw);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      title={<div style={{ textAlign: 'center', width: '100%' }}>新建账户</div>}
      open={open}
      onCancel={onCancel}
      footer={[
        <Button key="cancel" onClick={onCancel}>
          取消
        </Button>,
        <Button key="submit" type="primary" loading={submitting} onClick={() => form.submit()}>
          {submitting ? '创建中...' : '创建'}
        </Button>,
      ]}
      destroyOnClose
    >
      <div style={{ marginBottom: 12 }}>
        <Typography.Text type="secondary">机构:{institutionName}</Typography.Text>
        <br />
        <Typography.Text type="secondary" code style={{ fontSize: 11 }}>
          {sfidId}
        </Typography.Text>
      </div>
      <Form form={form} layout="vertical" onFinish={onSubmit}>
        <Form.Item
          label="账户名称"
          name="account_name"
          rules={[
            { required: true, message: '请输入账户名称' },
            { max: 30, message: '最多 30 个字' },
          ]}
          extra={'创建后账户状态为"未上链";链上注册由区块链软件完成后同步回来。'}
        >
          <Input
            placeholder="如:办案账户、工资账户、采购账户..."
            maxLength={30}
          />
        </Form.Item>
      </Form>
    </Modal>
  );
};
