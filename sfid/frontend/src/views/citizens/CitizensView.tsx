// 中文注释:从 App.tsx 迁移(任务卡 20260408-sfid-frontend-app-tsx-split 步 4)
// 注册局顶层视图 —— activeView === 'citizens' 分支。
// 包含:citizen 列表 + 搜索栏 + 表格 + 绑定/解绑按钮 + BindModal/UnbindModal/操作扫码 Modal。

import { useEffect, useRef, useState } from 'react';
import { Button, Card, Form, Input, Modal, Space, Table, Typography, message } from 'antd';
import { QrcodeOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import type { CitizenRow } from '../../api/client';
import { listCitizens, scanCpmsStatusQr } from '../../api/client';
import { decodeSs58, tryEncodeSs58 } from '../../utils/ss58';
import { startCameraScanner } from '../../utils/cameraScanner';
import { useAuth } from '../../hooks/useAuth';
import { glassCardStyle, glassCardHeadStyle } from '../../components/App';
import { BindModal } from './BindModal';
import { UnbindModal } from './UnbindModal';

export function CitizensView() {
  const { auth, capabilities } = useAuth();
  const [rows, setRows] = useState<CitizenRow[]>([]);
  const [loading, setLoading] = useState(false);

  // 绑定/解绑弹窗控制(state 仅持有 open + 当前 record,其它细节在 Modal 组件内)
  const [bindModalOpen, setBindModalOpen] = useState(false);
  const [bindTargetRecord, setBindTargetRecord] = useState<CitizenRow | null>(null);
  const [unbindModalOpen, setUnbindModalOpen] = useState(false);
  const [unbindTarget, setUnbindTarget] = useState<CitizenRow | null>(null);

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
      setUnbindModalOpen(false);
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

  const openBindModal = (record: CitizenRow) => {
    setBindTargetRecord(record);
    setBindModalOpen(true);
  };

  const openUnbindModal = (record: CitizenRow) => {
    setUnbindTarget(record);
    setUnbindModalOpen(true);
  };

  const citizenColumns: ColumnsType<CitizenRow> = [
    {
      title: '序号',
      width: 80,
      align: 'center',
      render: (_v: unknown, _r: CitizenRow, idx: number) => idx + 1,
    },
    {
      title: '账户',
      dataIndex: 'account_pubkey',
      align: 'center',
      render: (v: string | undefined) => (v ? tryEncodeSs58(v) : '-'),
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
        if (v === 'BOUND') return '已绑定';
        if (v === 'UNLINKED') return '已解绑';
        return '未绑定';
      },
    },
  ];
  if (capabilities.canBusinessWrite) {
    citizenColumns.push({
      title: '操作',
      width: 200,
      align: 'center',
      render: (_v: unknown, row: CitizenRow) => (
        <Space size={8}>
          {row.status === 'BOUND' ? (
            <Button danger onClick={() => openUnbindModal(row)}>
              解绑
            </Button>
          ) : (
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

      <UnbindModal
        auth={auth}
        open={unbindModalOpen}
        target={unbindTarget}
        onClose={() => setUnbindModalOpen(false)}
        onUnbound={() => refreshList(undefined, true)}
      />

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
              <QrcodeOutlined style={{ fontSize: 32, color: 'rgba(255,255,255,0.25)' }} />
              <Typography.Text style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12 }}>摄像头初始化中...</Typography.Text>
            </div>
          )}
        </div>
      </Modal>
    </>
  );
}
