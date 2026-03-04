import { Card, Space, Typography } from 'antd';

export function FullDashboard() {
  return (
    <Space direction="vertical" size={12} style={{ width: '100%' }}>
      <Card>
        <Space direction="vertical" size={6}>
          <Typography.Title level={5} style={{ margin: 0 }}>
            节点运行面板
          </Typography.Title>
          <Typography.Text type="secondary">节点已启动，可在本机持续同步与出块。</Typography.Text>
        </Space>
      </Card>
    </Space>
  );
}
