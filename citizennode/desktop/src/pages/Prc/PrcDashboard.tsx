import { Card, Col, Row, Space, Typography } from 'antd';

export function PrcDashboard() {
  return (
    <Space direction="vertical" size={12} style={{ width: '100%' }}>
      <Card>
        <Space direction="vertical" size={6}>
          <Typography.Title level={5} style={{ margin: 0 }}>
            CPMS 档案审核工作台
          </Typography.Title>
          <Typography.Text type="secondary">
            审核档案完整性与索引号规范，处理退回、补录、通过等流程。
          </Typography.Text>
        </Space>
      </Card>

      <Row gutter={[12, 12]}>
        <Col xs={24} md={12}>
          <Card title="待审核档案">
            <Typography.Text type="secondary">查看待审核列表与关键字段差异。</Typography.Text>
          </Card>
        </Col>
        <Col xs={24} md={12}>
          <Card title="审核记录">
            <Typography.Text type="secondary">查询审核操作日志与责任人。</Typography.Text>
          </Card>
        </Col>
      </Row>
    </Space>
  );
}
