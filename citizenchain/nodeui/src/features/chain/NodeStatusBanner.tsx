import { Alert, Space, Tag, Typography } from 'antd';
import { useSessionStore } from '../../stores/session';

export function NodeStatusBanner() {
  const { state, error } = useSessionStore();
  const color = state === 'connected' ? 'success' : state === 'error' ? 'error' : 'default';
  const statusText =
    state === 'connected' ? '已连接' : state === 'connecting' ? '连接中' : state === 'error' ? '连接异常' : '未连接';

  return (
    <Space direction="vertical" size={4} style={{ width: '100%' }}>
      <Space wrap>
        <Typography.Text>本机节点状态</Typography.Text>
        <Tag color={color}>{statusText}</Tag>
      </Space>
      {state === 'error' ? <Alert type="error" showIcon message={`自动连接失败：${error}`} /> : null}
    </Space>
  );
}
