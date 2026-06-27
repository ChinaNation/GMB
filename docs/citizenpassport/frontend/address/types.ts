// CPMS 地址模块类型：居住省市由安装授权决定；出生地省市镇来自随包 CID 行政区真源只读拷贝。

export interface Province {
  province_code: string;
  province_name: string;
}

export interface City {
  city_code: string;
  city_name: string;
}

export interface Town {
  town_code: string;
  town_name: string;
}

export interface AddressUnit {
  address_unit_id: string;
  town_code: string;
  address_unit_name: string;
}
