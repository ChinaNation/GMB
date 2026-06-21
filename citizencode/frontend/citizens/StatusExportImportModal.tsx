import { useState } from 'react';
import { Button, Descriptions, Modal, Space, Typography, Upload } from 'antd';

import type { AdminAuth } from '../auth/types';
import {
  importCpmsStatusExport,
  type CpmsStatusExportFile,
  type CpmsStatusExportImportResult,
} from './api';
import { notice } from '../utils/notice';

type Props = {
  auth: AdminAuth | null;
  open: boolean;
  onClose: () => void;
  onImported: () => void;
};

export function StatusExportImportModal({ auth, open, onClose, onImported }: Props) {
  const [exportFile, setExportFile] = useState<CpmsStatusExportFile | null>(null);
  const [result, setResult] = useState<CpmsStatusExportImportResult | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const reset = () => {
    setExportFile(null);
    setResult(null);
    setSubmitting(false);
  };

  const close = () => {
    reset();
    onClose();
  };

  const onSelectFile = async (file: File) => {
    try {
      const parsed = JSON.parse(await file.text()) as CpmsStatusExportFile;
      if (parsed.proto !== 'CID_CPMS_V1' || parsed.type !== 'CPMS_STATUS_EXPORT') {
        notice.error('年度报告格式不正确');
        return Upload.LIST_IGNORE;
      }
      setExportFile(parsed);
      setResult(null);
      notice.success('年度报告已读取');
    } catch {
      notice.error('年度报告 JSON 解析失败');
    }
    return Upload.LIST_IGNORE;
  };

  const onImport = async () => {
    if (!auth || !exportFile) return;
    setSubmitting(true);
    try {
      const imported = await importCpmsStatusExport(auth, exportFile);
      setResult(imported);
      onImported();
      notice.success(imported.already_imported ? '该年度报告已导入' : '年度报告导入完成');
    } catch (err) {
      notice.error(err, '');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      title="导入年度报告"
      open={open}
      onCancel={close}
      destroyOnClose
      footer={
        <Space>
          <Button onClick={close}>关闭</Button>
          <Button type="primary" disabled={!exportFile} loading={submitting} onClick={onImport}>
            导入
          </Button>
        </Space>
      }
    >
      <Upload accept="application/json,.json" beforeUpload={onSelectFile} showUploadList={false}>
        <Button>选择年度报告 JSON</Button>
      </Upload>

      {exportFile && (
        <Descriptions column={1} size="small" bordered style={{ marginTop: 16 }}>
          <Descriptions.Item label="年度">{exportFile.export_year}</Descriptions.Item>
          <Descriptions.Item label="CPMS">{exportFile.cid_number}</Descriptions.Item>
          <Descriptions.Item label="绑定记录">{exportFile.citizen_binding_records_count}</Descriptions.Item>
          <Descriptions.Item label="释放记录">{exportFile.binding_release_records_count}</Descriptions.Item>
          <Descriptions.Item label="批次">{exportFile.export_batch_id}</Descriptions.Item>
        </Descriptions>
      )}

      {result && (
        <Typography.Paragraph style={{ marginTop: 16, marginBottom: 0 }}>
          已更新 {result.updated_binding_records} 条，已更换钱包 {result.wallet_replaced_records} 条，
          已删除无资格公民 {result.deleted_ineligible_records} 条，已释放绑定 {result.released_binding_records} 条。
        </Typography.Paragraph>
      )}
    </Modal>
  );
}
