// 中文注释:新建账户弹窗。
// 账户名自由输入(≤ 30 字),同 sfid 下不能重名。
// 提交调 createAccount → 上链 register_sfid_institution。
// 铁律:feedback_institutions_two_layer.md

import React, { useEffect, useMemo, useState } from 'react';
import { Button, Form, Input, message, Modal, Typography } from 'antd';
import { createAccount, type MultisigAccount } from '../../api/institution';
import type { AdminAuth } from '../../api/client';
import { deriveDuoqianAddress } from '../../utils/deriveDuoqianAddress';

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
  const [inputName, setInputName] = useState('');

  const previewAddress = useMemo(
    () => deriveDuoqianAddress(sfidId, inputName),
    [sfidId, inputName],
  );

  useEffect(() => {
    if (open) {
      form.resetFields();
      setInputName('');
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
      // 任务卡 20260409 Phase 1.C：识别后端针对省签名密钥缺失的 503 错误，给出友好提示
      const raw = err instanceof Error ? err.message : '创建账户失败';
      if (raw.includes('本省') && raw.includes('未在线')) {
        message.error('本省登录管理员未在线,请联系省管理员登录后重试');
      } else if (raw.includes('密钥管理员不能直接推送')) {
        message.error('请以省或市管理员身份操作');
      } else {
        message.error(raw);
      }
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
          <Input
            placeholder="如:办案账户、工资账户、采购账户..."
            maxLength={30}
            onChange={(e) => setInputName(e.target.value.trim())}
          />
        </Form.Item>
        {previewAddress && (
          <div style={{ marginTop: -8, marginBottom: 16 }}>
            <Typography.Text type="secondary" style={{ fontSize: 12 }}>派生地址：</Typography.Text>
            <Typography.Text code style={{ fontSize: 12, wordBreak: 'break-all' }}>
              {previewAddress}
            </Typography.Text>
          </div>
        )}
      </Form>
    </Modal>
  );
};
