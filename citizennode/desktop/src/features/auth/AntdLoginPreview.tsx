import { LockOutlined, UserOutlined } from '@ant-design/icons';
import { Alert, Button, Card, Checkbox, Form, Input, Space, Typography } from 'antd';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '../../stores/auth';
import './AntdLoginPreview.css';

type LoginValues = {
  username: string;
  password: string;
  remember?: boolean;
};

export function AntdLoginPreview() {
  const [form] = Form.useForm<LoginValues>();
  const navigate = useNavigate();
  const login = useAuthStore((state) => state.login);
  const [loginError, setLoginError] = useState<string | null>(null);

  const DEV_SUPER_ADMIN = {
    username: 'superadmin',
    password: 'SFID@123456'
  };

  const onFinish = (values: LoginValues) => {
    if (values.username !== DEV_SUPER_ADMIN.username || values.password !== DEV_SUPER_ADMIN.password) {
      setLoginError('账号或密码错误');
      return;
    }

    setLoginError(null);
    login({
      role: 'full',
      publicKey: '0xSFID_LOCAL_SUPER_ADMIN',
      organizationName: 'SFID 本地超级管理员'
    });
    navigate('/');
  };

  return (
    <div className="sfid-cartoon-page">
      <div className="sfid-cartoon-frame">
        <Card bordered={false} className="sfid-cartoon-card">
          <Space direction="vertical" size={20} style={{ width: '100%' }}>
            <div>
              <Typography.Title level={3} style={{ margin: 0 }}>
                SFID 管理端登录
              </Typography.Title>
              <Typography.Text type="secondary">公民护照管理系统（局域网离线版）</Typography.Text>
            </div>

            <Alert
              type="info"
              showIcon
              className="sfid-cartoon-alert"
              message="请使用系统管理员账号登录"
              description="开发账号：superadmin / SFID@123456"
            />

            {loginError ? <Alert type="error" showIcon message={loginError} /> : null}

            <Form<LoginValues> form={form} layout="vertical" onFinish={onFinish} initialValues={{ remember: true }}>
              <Form.Item label="账号" name="username" rules={[{ required: true, message: '请输入账号' }]}>
                <Input placeholder="请输入管理员账号" prefix={<UserOutlined />} size="large" />
              </Form.Item>

              <Form.Item label="密码" name="password" rules={[{ required: true, message: '请输入密码' }]}>
                <Input.Password placeholder="请输入密码" prefix={<LockOutlined />} size="large" />
              </Form.Item>

              <Form.Item name="remember" valuePropName="checked" style={{ marginBottom: 12 }}>
                <Checkbox>记住本机账号</Checkbox>
              </Form.Item>

              <Button type="primary" htmlType="submit" size="large" block>
                登录系统
              </Button>
            </Form>
          </Space>
        </Card>
      </div>
    </div>
  );
}
