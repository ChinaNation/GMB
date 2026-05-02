// 中文注释:省管理员 3-tier 名册管理页(ADR-008)。
// Main 可加/删 Backup1/Backup2;Backup* 与 SHI_ADMIN 仅可读。
// 后端 endpoint:GET /api/v1/admin/sheng-admin/roster + add-backup / remove-backup,
// 推链段当前为 mock(phase45 实现,phase7 切真)。

import React, { useCallback, useEffect, useState } from 'react';
import { Card, Button, Tag, Space, Modal, Form, Input, message, Spin } from 'antd';
import type { AdminAuth } from '../../api/client';
import { getRoster, addBackup, removeBackup, type RosterEntry, type ShengAdminRoster } from './api';
import type { ShengSlot } from './types';
import { ShengSlotLabel } from './types';
import { glassCardStyle, glassCardHeadStyle } from '../../App';

interface Props {
  auth: AdminAuth;
}

export const RosterPage: React.FC<Props> = ({ auth }) => {
  const [roster, setRoster] = useState<ShengAdminRoster | null>(null);
  const [loading, setLoading] = useState(false);
  const [addModalOpen, setAddModalOpen] = useState(false);
  const [addingSlot, setAddingSlot] = useState<Exclude<ShengSlot, 'Main'> | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [form] = Form.useForm<{ new_pubkey: string; new_name?: string }>();

  // 仅 Main 槽可以操作名册;Backup* 与 SHI_ADMIN 进入此页只读
  const isMain = auth.role === 'SHENG_ADMIN' && auth.unlocked_slot === 'Main';

  const reload = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getRoster(auth, auth.admin_province ?? undefined);
      setRoster(data);
    } catch (e) {
      message.error(e instanceof Error ? e.message : '名册加载失败');
    } finally {
      setLoading(false);
    }
  }, [auth]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const handleAdd = async () => {
    if (!addingSlot) return;
    try {
      const values = await form.validateFields();
      setSubmitting(true);
      await addBackup(auth, { slot: addingSlot, new_pubkey: values.new_pubkey.trim(), new_name: values.new_name?.trim() });
      message.success(`${ShengSlotLabel[addingSlot]} 添加成功(链上推送 mock,phase7 切真)`);
      setAddModalOpen(false);
      form.resetFields();
      setAddingSlot(null);
      await reload();
    } catch (e) {
      if (e instanceof Error) message.error(e.message);
    } finally {
      setSubmitting(false);
    }
  };

  const handleRemove = (slot: Exclude<ShengSlot, 'Main'>) => {
    Modal.confirm({
      title: `确认移除 ${ShengSlotLabel[slot]}?`,
      content: '该操作会推链 remove_sheng_admin_backup,移除后该槽签名密钥失效。',
      okType: 'danger',
      okText: '移除',
      cancelText: '取消',
      onOk: async () => {
        try {
          await removeBackup(auth, { slot });
          message.success(`${ShengSlotLabel[slot]} 已移除`);
          await reload();
        } catch (e) {
          if (e instanceof Error) message.error(e.message);
        }
      },
    });
  };

  if (loading && !roster) {
    return <Spin tip="加载名册..." />;
  }

  return (
    <Card
      title="省管理员名册(3-tier)"
      style={glassCardStyle}
      headStyle={glassCardHeadStyle}
      extra={<Button onClick={() => void reload()}>刷新</Button>}
    >
      {roster && (
        <Space direction="vertical" size={12} style={{ width: '100%' }}>
          <div style={{ color: '#6b7280' }}>
            省份:<strong>{roster.province}</strong>
            {auth.unlocked_slot && <Tag color="cyan" style={{ marginLeft: 12 }}>当前槽:{ShengSlotLabel[auth.unlocked_slot]}</Tag>}
          </div>
          {roster.entries.map((entry) => (
            <RosterRow
              key={entry.slot}
              entry={entry}
              canEdit={isMain}
              onAdd={() => {
                if (entry.slot === 'Main') return;
                setAddingSlot(entry.slot);
                setAddModalOpen(true);
              }}
              onRemove={() => entry.slot !== 'Main' && handleRemove(entry.slot)}
            />
          ))}
        </Space>
      )}

      <Modal
        title={addingSlot ? `添加 ${ShengSlotLabel[addingSlot]}` : '添加备份槽'}
        open={addModalOpen}
        onCancel={() => {
          setAddModalOpen(false);
          form.resetFields();
          setAddingSlot(null);
        }}
        onOk={handleAdd}
        confirmLoading={submitting}
        okText="提交并推链(mock)"
      >
        <Form form={form} layout="vertical">
          <Form.Item
            name="new_pubkey"
            label="新管理员公钥(0x 小写 hex,32 字节)"
            rules={[
              { required: true, message: '请输入公钥' },
              { pattern: /^0x[0-9a-f]{64}$/, message: '需为 0x 开头 64 位小写 hex' },
            ]}
          >
            <Input placeholder="0x..." />
          </Form.Item>
          <Form.Item name="new_name" label="备注名(可选)">
            <Input placeholder="如:王二备份" />
          </Form.Item>
        </Form>
      </Modal>
    </Card>
  );
};

const RosterRow: React.FC<{
  entry: RosterEntry;
  canEdit: boolean;
  onAdd: () => void;
  onRemove: () => void;
}> = ({ entry, canEdit, onAdd, onRemove }) => {
  const occupied = entry.admin_pubkey && entry.admin_pubkey.length > 0;
  return (
    <div style={{ padding: 12, border: '1px solid #e5e7eb', borderRadius: 8 }}>
      <Space size={12} align="center">
        <Tag color={entry.slot === 'Main' ? 'gold' : 'blue'}>{ShengSlotLabel[entry.slot]}</Tag>
        {occupied ? (
          <>
            <code style={{ fontSize: 12 }}>{entry.admin_pubkey}</code>
            {entry.admin_name && <span>{entry.admin_name}</span>}
            <Tag color={entry.signing_status === 'ACTIVATED' ? 'green' : 'default'}>
              {entry.signing_status === 'ACTIVATED' ? '签名已激活' : '签名未激活'}
            </Tag>
          </>
        ) : (
          <span style={{ color: '#9ca3af' }}>(空槽)</span>
        )}
        {canEdit && entry.slot !== 'Main' && (
          occupied ? (
            <Button danger size="small" onClick={onRemove}>移除</Button>
          ) : (
            <Button type="primary" size="small" onClick={onAdd}>添加</Button>
          )
        )}
      </Space>
    </div>
  );
};
