// 左侧导航栏

import { NavLink } from 'react-router-dom';

interface NavItem {
  label: string;
  to: string;
}

export default function Sidebar({ items }: { items: NavItem[] }) {
  return (
    <aside className="layout__sidebar">
      <div className="sidebar__brand">CPMS<br /><span style={{ fontSize: 11, fontWeight: 400, opacity: 0.7 }}>公民护照管理系统</span></div>
      <nav className="sidebar__nav">
        {items.map(item => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) => `sidebar__item${isActive ? ' sidebar__item--active' : ''}`}
          >
            {item.label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
