import { Card, Col, Row, Space, Typography } from 'antd';

export function PrbDashboard() {
  return (
    <Space direction="vertical" size={12} style={{ width: '100%' }}>
      <Card>
        <Space direction="vertical" size={6}>
          <Typography.Title level={5} style={{ margin: 0 }}>
            CPMS 档案检索工作台
          </Typography.Title>
          <Typography.Text type="secondary">
            按档案索引号、护照号、姓名等条件快速检索并查看档案详情。
          </Typography.Text>
        </Space>
      </Card>

      <Row gutter={[12, 12]}>
        <Col xs={24} md={12}>
          <Card title="快速检索">
            <Typography.Text type="secondary">支持复合条件与最近检索历史。</Typography.Text>
          </Card>
        </Col>
        <Col xs={24} md={12}>
          <Card title="档案导出">
            <Typography.Text type="secondary">导出 PDF/打印预览，供线下归档。</Typography.Text>
          </Card>
        </Col>
      </Row>
    </Space>
  );
}
