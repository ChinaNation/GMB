// CPMS 地址模块类型：省市由后端安装信息决定，前端只选择镇村。

export interface Town {
  town_code: string;
  town_name: string;
}

export interface Village {
  village_id: string;
  town_code: string;
  village_name: string;
}
