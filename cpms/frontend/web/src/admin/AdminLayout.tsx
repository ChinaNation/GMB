import { Outlet } from 'react-router-dom';
import Sidebar from '../common/Sidebar';
import Header from '../common/Header';

const NAV = [
  { label: '操作员管理', to: '/admin' },
  { label: '站点密钥', to: '/admin/site-keys' },
  { label: '公民状态', to: '/admin/citizen-status' },
];

export default function AdminLayout() {
  return (
    <div className="layout">
      <Sidebar items={NAV} />
      <div className="layout__main">
        <Header title="超级管理员后台" />
        <div className="layout__content"><Outlet /></div>
      </div>
    </div>
  );
}
