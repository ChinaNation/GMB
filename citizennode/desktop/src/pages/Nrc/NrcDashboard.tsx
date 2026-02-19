import { Card, Col, Row, Space, Typography } from 'antd';

export function NrcDashboard() {
  return (
    <Space direction="vertical" size={12} style={{ width: '100%' }}>
      <Card>
        <Space direction="vertical" size={6}>
          <Typography.Title level={5} style={{ margin: 0 }}>
            CPMS 档案录入工作台
          </Typography.Title>
          <Typography.Text type="secondary">
            新建公民档案，录入姓名、出生日期、性别、身高、护照号与档案索引号。
          </Typography.Text>
        </Space>
      </Card>

      <Row gutter={[12, 12]}>
        <Col xs={24} md={12}>
          <Card title="照片采集">
            <Typography.Text type="secondary">支持拍照上传与本地文件导入。</Typography.Text>
          </Card>
        </Col>
        <Col xs={24} md={12}>
          <Card title="指纹采集">
            <Typography.Text type="secondary">支持设备采集与模板文件导入。</Typography.Text>
          </Card>
        </Col>
      </Row>
    </Space>
  );
}
