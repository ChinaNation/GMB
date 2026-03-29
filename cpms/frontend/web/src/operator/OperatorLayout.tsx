import { Outlet } from 'react-router-dom';
import Sidebar from '../common/Sidebar';
import Header from '../common/Header';

const NAV = [
  { label: '档案管理', to: '/operator' },
  { label: '新建档案', to: '/operator/create' },
];

export default function OperatorLayout() {
  return (
    <div className="layout">
      <Sidebar items={NAV} />
      <div className="layout__main">
        <Header title="操作员工作台" />
        <div className="layout__content"><Outlet /></div>
      </div>
    </div>
  );
}
