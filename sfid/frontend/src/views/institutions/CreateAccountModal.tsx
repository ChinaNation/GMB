// 中文注释:新建账户弹窗。
// 账户名自由输入(≤ 30 字),同 sfid 下不能重名。
// 提交调 createAccount → 上链 register_sfid_institution。
// 铁律:feedback_institutions_two_layer.md

import React, { useEffect, useState } from 'react';
import { Button, Form, Input, message, Modal, Typography } from 'antd';
import { createAccount, type MultisigAccount } from '../../api/institution';
import type { AdminAuth } from '../../api/client';

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
    // 前端预校验:同 sfid 下不能重名
    if (existingAccounts.some((a) => a.account_name === name)) {
      message.error(`账户名称"${name}"在本机构下已存在`);
      return;
    }
    setSubmitting(true);
    try {
      await createAccount(auth, sfidId, name);
      message.success(`账户已创建并上链:${name}`);
      onCreated();
    } catch (err) {
      message.error(err instanceof Error ? err.message : '创建账户失败');
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
          {submitting ? '上链中...' : '创建并上链'}
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
          extra="同一机构下账户名称不能重复。账户名称将作为链上派生多签地址的 name 参数。"
        >
          <Input placeholder="如:办案账户、工资账户、采购账户..." maxLength={30} />
        </Form.Item>
      </Form>
    </Modal>
  );
};
