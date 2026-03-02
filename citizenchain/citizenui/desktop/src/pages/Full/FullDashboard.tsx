import { Card, Col, Row, Space, Typography } from 'antd';
import { NodeConnectionCard } from '../../features/chain/NodeConnectionCard';
import { AccountBalanceCard } from '../../features/monitor/AccountBalanceCard';
import { RecentTransactionsCard } from '../../features/monitor/RecentTransactionsCard';
import { DeveloperSettingsCard } from '../../features/settings/DeveloperSettingsCard';

export function FullDashboard() {
  return (
    <Space direction="vertical" size={12} style={{ width: '100%' }}>
      <Card>
        <Space direction="vertical" size={6}>
          <Typography.Title level={5} style={{ margin: 0 }}>
            SFID 超级管理员工作台
          </Typography.Title>
          <Typography.Text type="secondary">
            管理管理员账户、系统参数、数据备份恢复与局域网共享配置。
          </Typography.Text>
        </Space>
      </Card>

      <Row gutter={[12, 12]}>
        <Col xs={24} md={12}>
          <NodeConnectionCard />
        </Col>
        <Col xs={24} md={12}>
          <AccountBalanceCard />
        </Col>
        <Col xs={24}>
          <RecentTransactionsCard />
        </Col>
        <Col xs={24}>
          <DeveloperSettingsCard />
        </Col>
      </Row>
    </Space>
  );
}
