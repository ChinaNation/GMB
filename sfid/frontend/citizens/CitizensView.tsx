// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 4)
// 注册局顶层视图 —— activeView === 'citizens' 分支。
// 包含:citizen 列表 + 搜索栏 + 表格 + 绑定/推链绑定/推链解绑按钮 + BindModal/操作扫码 Modal。

import { useEffect, useRef, useState } from 'react';
import { Button, Card, Form, Input, Modal, Space, Table, Typography, message } from 'antd';

import type { ColumnsType } from 'antd/es/table';
import type { CitizenRow } from '../api/client';
import { listCitizens, scanCpmsStatusQr, citizenPushChainBind, citizenPushChainUnbind } from '../api/client';
import { decodeSs58 } from '../utils/ss58';
import { startCameraScanner } from '../utils/cameraScanner';
import { useAuth } from '../hooks/useAuth';
import { glassCardStyle, glassCardHeadStyle } from '../App';
import { BindModal } from './BindModal';


export function CitizensView() {
  const { auth, capabilities } = useAuth();
  const [rows, setRows] = useState<CitizenRow[]>([]);
  const [loading, setLoading] = useState(false);

  // 绑定/解绑弹窗控制(state 仅持有 open + 当前 record,其它细节在 Modal 组件内)
  const [bindModalOpen, setBindModalOpen] = useState(false);
  const [bindTargetRecord, setBindTargetRecord] = useState<CitizenRow | null>(null);

  // 操作扫码(QR4 citizen 状态扫描)—— 原 opScan 系列
  const [opScanOpen, setOpScanOpen] = useState(false);
  const [opScannerReady, setOpScannerReady] = useState(false);
  const [, setOpScanSubmitting] = useState(false);
  const opVideoRef = useRef<HTMLVideoElement | null>(null);
  const opScanCleanupRef = useRef<(() => void) | null>(null);

  const refreshList = async (keyword?: string, silent?: boolean) => {
    if (!auth) return;
    setLoading(true);
    try {
      const raw = await listCitizens(auth, keyword);
      const list = Array.isArray(raw) ? raw : [];
      setRows(list);
      if (keyword && list.length === 0) {
        Modal.warning({
          title: '查询结果',
          content: '没有的公民信息',
        });
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : '查询失败';
      if (!silent) {
        message.error(msg);
      }
    } finally {
      setLoading(false);
    }
  };

  // 挂载时自动加载;auth 变化时(登录/登出)重新加载
  useEffect(() => {
    if (!auth) {
      setRows([]);
      setBindModalOpen(false);
      setOpScanOpen(false);
      stopOpScanner();
      return;
    }
    void refreshList(undefined, true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth]);

  const onSearch = async (values: { keyword: string }) => {
    if (!auth) return;
    let keyword = values.keyword?.trim() || '';
    if (keyword) {
      try {
        keyword = decodeSs58(keyword);
      } catch {
        // 非 SS58 格式,保留原值
      }
    }
    await refreshList(keyword);
  };

  const stopOpScanner = () => {
    if (opScanCleanupRef.current) {
      opScanCleanupRef.current();
      opScanCleanupRef.current = null;
    }
    setOpScannerReady(false);
  };

  const onHandleOperationQr = async (raw: string) => {
    if (!auth) return;
    setOpScanSubmitting(true);
    try {
      const result = await scanCpmsStatusQr(auth, { qr_payload: raw });
      message.success(`状态已更新：${result.archive_no} -> ${result.status}`);
      await refreshList(undefined, true);
      setOpScanOpen(false);
      stopOpScanner();
    } catch (err) {
      const msg = err instanceof Error ? err.message : '扫码处理失败';
      message.error(msg);
    } finally {
      setOpScanSubmitting(false);
    }
  };

  useEffect(() => {
    if (!opScanOpen || !opVideoRef.current) {
      stopOpScanner();
      return;
    }
    opScanCleanupRef.current = startCameraScanner(
      opVideoRef.current,
      (raw) => void onHandleOperationQr(raw),
      () => setOpScannerReady(true),
      (msg) => message.error(msg),
    );
    return () => stopOpScanner();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [opScanOpen, auth]);

  const onPushChainBind = async (record: CitizenRow) => {
    if (!auth) return;
    try {
      setLoading(true);
      await citizenPushChainBind(auth, { citizen_id: record.id });
      message.success('推链绑定成功');
      await refreshList(undefined, true);
    } catch (err) {
      message.error(err instanceof Error ? err.message : '推链绑定失败');
    } finally {
      setLoading(false);
    }
  };

  const onPushChainUnbind = async (record: CitizenRow) => {
    if (!auth) return;
    Modal.confirm({
      title: '确认推链解绑',
      content: `确定要解绑账户 ${record.account_address ?? record.account_pubkey ?? ''} 吗？解绑后链上绑定关系将被移除。`,
      okText: '确认解绑',
      okButtonProps: { danger: true },
      cancelText: '取消',
      onOk: async () => {
        try {
          setLoading(true);
          await citizenPushChainUnbind(auth, { citizen_id: record.id });
          message.success('推链解绑成功');
          await refreshList(undefined, true);
        } catch (err) {
          message.error(err instanceof Error ? err.message : '推链解绑失败');
        } finally {
          setLoading(false);
        }
      },
    });
  };

  const openBindModal = (record: CitizenRow) => {
    setBindTargetRecord(record);
    setBindModalOpen(true);
  };

  const citizenColumns: ColumnsType<CitizenRow> = [
    {
      title: '序号',
      width: 80,
      align: 'center',
      render: (_v: unknown, _r: CitizenRow, idx: number) => idx + 1,
    },
    {
      title: '账户地址',
      dataIndex: 'account_address',
      align: 'center',
      render: (v: string | undefined) => v ?? '-',
    },
    {
      title: '档案号',
      dataIndex: 'archive_no',
      align: 'center',
      render: (v: string | undefined) => v ?? '-',
    },
    {
      title: 'SFID码',
      dataIndex: 'sfid_code',
      align: 'center',
      render: (v: string | undefined) => v ?? '-',
    },
    {
      title: '状态',
      dataIndex: 'status',
      width: 100,
      align: 'center',
      render: (v: string) => {
        if (v === 'PENDING') return '待绑定';
        if (v === 'BINDABLE') return '待推链';
        if (v === 'BOUND') return '已绑定';
        if (v === 'UNLINKED') return '已解绑';
        return v;
      },
    },
  ];
  if (capabilities.canBusinessWrite) {
    citizenColumns.push({
      title: '操作',
      width: 280,
      align: 'center',
      render: (_v: unknown, row: CitizenRow) => (
        <Space size={8}>
          {row.status === 'BOUND' && (
            <Button danger onClick={() => onPushChainUnbind(row)}>
              解绑
            </Button>
          )}
          {row.status === 'BINDABLE' && (
            <Button type="primary" onClick={() => onPushChainBind(row)}>
              确认
            </Button>
          )}
          {(row.status === 'UNLINKED' || row.status === 'PENDING') && (
            <Button type="primary" onClick={() => openBindModal(row)}>
              绑定
            </Button>
          )}
        </Space>
      ),
    });
  }

  return (
    <>
      <Card
        title={'公民身份列表'}
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
        extra={
          <Form layout="inline" onFinish={onSearch}>
            <Form.Item name="keyword" style={{ marginBottom: 0 }}>
              <Input style={{ width: 420 }} placeholder="请输入账户、档案号或SFID号" allowClear />
            </Form.Item>
            <Form.Item style={{ marginBottom: 0 }}>
              <Button htmlType="submit" type="primary" loading={loading}>
                查询
              </Button>
            </Form.Item>
          </Form>
        }
      >
        <Table<CitizenRow>
          rowKey={(r) => `${r.id}`}
          dataSource={rows}
          loading={loading}
          pagination={{ pageSize: 10 }}
          columns={citizenColumns}
        />
      </Card>

      {capabilities.canBusinessWrite && (
        <BindModal
          auth={auth}
          open={bindModalOpen}
          record={bindTargetRecord}
          onClose={() => setBindModalOpen(false)}
          onBound={() => refreshList(undefined, true)}
        />
      )}

      <Modal
        title="状态变更扫码"
        open={opScanOpen}
        footer={null}
        onCancel={() => {
          setOpScanOpen(false);
          stopOpScanner();
        }}
        destroyOnClose
      >
        <Typography.Paragraph type="secondary">
          请使用本机摄像头扫描二维码。
        </Typography.Paragraph>
        <div
          style={{
            width: '100%',
            aspectRatio: '1 / 1',
            background: 'linear-gradient(145deg, #0f172a, #1e293b)',
            borderRadius: 16,
            overflow: 'hidden',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            position: 'relative',
            border: '2px solid #334155',
            boxShadow: 'inset 0 2px 8px rgba(0,0,0,0.3)',
          }}
        >
          <div style={{ position: 'absolute', top: 8, left: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderTopLeftRadius: 4, zIndex: 2 }} />
          <div style={{ position: 'absolute', top: 8, right: 8, width: 16, height: 16, borderTop: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderTopRightRadius: 4, zIndex: 2 }} />
          <div style={{ position: 'absolute', bottom: 8, left: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderLeft: '2px solid #0d9488', borderBottomLeftRadius: 4, zIndex: 2 }} />
          <div style={{ position: 'absolute', bottom: 8, right: 8, width: 16, height: 16, borderBottom: '2px solid #0d9488', borderRight: '2px solid #0d9488', borderBottomRightRadius: 4, zIndex: 2 }} />
          <video ref={opVideoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
          {!opScannerReady && (
            <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 8 }}>
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.25)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 7V5a2 2 0 0 1 2-2h2"/><path d="M17 3h2a2 2 0 0 1 2 2v2"/><path d="M21 17v2a2 2 0 0 1-2 2h-2"/><path d="M7 21H5a2 2 0 0 1-2-2v-2"/><rect x="7" y="7" width="10" height="10" rx="1"/></svg>
              <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>摄像头初始化中...</Typography.Text>
            </div>
          )}
        </div>
      </Modal>
    </>
  );
}
