import { Card, Col, Row, Space, Typography } from 'antd';

export function FullDashboard() {
  return (
    <Space direction="vertical" size={12} style={{ width: '100%' }}>
      <Card>
        <Space direction="vertical" size={6}>
          <Typography.Title level={5} style={{ margin: 0 }}>
            CPMS 超级管理员工作台
          </Typography.Title>
          <Typography.Text type="secondary">
            管理管理员账户、系统参数、数据备份恢复与局域网共享配置。
          </Typography.Text>
        </Space>
      </Card>

      <Row gutter={[12, 12]}>
        <Col xs={24} md={12}>
          <Card title="管理员账户管理">
            <Typography.Text type="secondary">创建、禁用、重置普通管理员账号。</Typography.Text>
          </Card>
        </Col>
        <Col xs={24} md={12}>
          <Card title="备份与恢复">
            <Typography.Text type="secondary">本地备份可拷贝到其他电脑继续使用。</Typography.Text>
          </Card>
        </Col>
      </Row>
    </Space>
  );
}
