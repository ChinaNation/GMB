import { Link } from 'react-router-dom';

export default function NotFound() {
  return (
    <div className="login-page">
      <div className="login-card login-card--simple" style={{ textAlign: 'center' }}>
        <h1 style={{ fontSize: 48, color: 'var(--color-primary)', marginBottom: 12 }}>404</h1>
        <p style={{ color: 'var(--color-text-secondary)', marginBottom: 24 }}>页面不存在</p>
        <Link to="/" className="btn btn--primary">返回首页</Link>
      </div>
    </div>
  );
}
