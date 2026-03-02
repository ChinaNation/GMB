import { Alert, Space, Tag, Typography } from 'antd';
import { useSessionStore } from '../../stores/session';

export function NodeStatusBanner() {
  const { state, endpoint, error } = useSessionStore();
  const color = state === 'connected' ? 'success' : state === 'error' ? 'error' : 'default';

  return (
    <Space direction="vertical" size={4} style={{ width: '100%' }}>
      <Space wrap>
        <Typography.Text>本机节点状态</Typography.Text>
        <Tag color={color}>{state}</Tag>
        <Typography.Text type="secondary">{endpoint}</Typography.Text>
      </Space>
      {state === 'error' ? <Alert type="error" showIcon message={`自动连接失败：${error}`} /> : null}
    </Space>
  );
}
