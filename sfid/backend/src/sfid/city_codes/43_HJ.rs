use super::{CityCode, TownCode, VillageCode};

static TOWNS_HJ_001: [TownCode; 27] = [
    TownCode {
        name: "中心街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "学府社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "城西社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "阳光社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "园林社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "红卫社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "莲花社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "农垦连珠山社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "农垦北大营社区居民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "密山镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "金沙社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "长青村委会",
                code: "002",
            },
            VillageCode {
                name: "双胜村委会",
                code: "003",
            },
            VillageCode {
                name: "牧副村委会",
                code: "004",
            },
            VillageCode {
                name: "双跃村委会",
                code: "005",
            },
            VillageCode {
                name: "铁西村委会",
                code: "006",
            },
            VillageCode {
                name: "新路村委会",
                code: "007",
            },
            VillageCode {
                name: "新农村委会",
                code: "008",
            },
            VillageCode {
                name: "新华村委会",
                code: "009",
            },
            VillageCode {
                name: "新山村委会",
                code: "010",
            },
            VillageCode {
                name: "新丰村委会",
                code: "011",
            },
            VillageCode {
                name: "新林村委会",
                code: "012",
            },
            VillageCode {
                name: "新治村委会",
                code: "013",
            },
            VillageCode {
                name: "新和村委会",
                code: "014",
            },
            VillageCode {
                name: "新鲜村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "连珠山镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "沙岗村委会",
                code: "001",
            },
            VillageCode {
                name: "保安村委会",
                code: "002",
            },
            VillageCode {
                name: "永新村委会",
                code: "003",
            },
            VillageCode {
                name: "发展村委会",
                code: "004",
            },
            VillageCode {
                name: "新发村委会",
                code: "005",
            },
            VillageCode {
                name: "永泉村委会",
                code: "006",
            },
            VillageCode {
                name: "新忠村委会",
                code: "007",
            },
            VillageCode {
                name: "东方红村委会",
                code: "008",
            },
            VillageCode {
                name: "解放村委会",
                code: "009",
            },
            VillageCode {
                name: "红光村委会",
                code: "010",
            },
            VillageCode {
                name: "连珠山村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "当壁镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "实边村委会",
                code: "001",
            },
            VillageCode {
                name: "祥生村委会",
                code: "002",
            },
            VillageCode {
                name: "庆利村委会",
                code: "003",
            },
            VillageCode {
                name: "庆康村委会",
                code: "004",
            },
            VillageCode {
                name: "宁安村委会",
                code: "005",
            },
            VillageCode {
                name: "临河村委会",
                code: "006",
            },
            VillageCode {
                name: "大顶山村委会",
                code: "007",
            },
            VillageCode {
                name: "三梭通村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "知一镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "加禾村委会",
                code: "001",
            },
            VillageCode {
                name: "福兴村委会",
                code: "002",
            },
            VillageCode {
                name: "迎恩村委会",
                code: "003",
            },
            VillageCode {
                name: "知一村委会",
                code: "004",
            },
            VillageCode {
                name: "向化村委会",
                code: "005",
            },
            VillageCode {
                name: "归仁村委会",
                code: "006",
            },
            VillageCode {
                name: "崇实村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "黑台镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "庆先村委会",
                code: "001",
            },
            VillageCode {
                name: "黑台村委会",
                code: "002",
            },
            VillageCode {
                name: "农业村委会",
                code: "003",
            },
            VillageCode {
                name: "塔头村委会",
                code: "004",
            },
            VillageCode {
                name: "新福村委会",
                code: "005",
            },
            VillageCode {
                name: "大城村委会",
                code: "006",
            },
            VillageCode {
                name: "兴盛村委会",
                code: "007",
            },
            VillageCode {
                name: "广新村委会",
                code: "008",
            },
            VillageCode {
                name: "直正村委会",
                code: "009",
            },
            VillageCode {
                name: "榆树村委会",
                code: "010",
            },
            VillageCode {
                name: "共裕村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "兴凯镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "八五一一农场社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "兴凯村委会",
                code: "002",
            },
            VillageCode {
                name: "红岭村委会",
                code: "003",
            },
            VillageCode {
                name: "兴农村委会",
                code: "004",
            },
            VillageCode {
                name: "东光村委会",
                code: "005",
            },
            VillageCode {
                name: "兴旺村委会",
                code: "006",
            },
            VillageCode {
                name: "东发村委会",
                code: "007",
            },
            VillageCode {
                name: "平原村委会",
                code: "008",
            },
            VillageCode {
                name: "星火村委会",
                code: "009",
            },
            VillageCode {
                name: "宏亮村委会",
                code: "010",
            },
            VillageCode {
                name: "鲜新村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "裴德镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "农大社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "双峰社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "裴德村委会",
                code: "003",
            },
            VillageCode {
                name: "东胜村委会",
                code: "004",
            },
            VillageCode {
                name: "德兴村委会",
                code: "005",
            },
            VillageCode {
                name: "兴利村委会",
                code: "006",
            },
            VillageCode {
                name: "跃进村委会",
                code: "007",
            },
            VillageCode {
                name: "平安村委会",
                code: "008",
            },
            VillageCode {
                name: "青年村委会",
                code: "009",
            },
            VillageCode {
                name: "红岩村委会",
                code: "010",
            },
            VillageCode {
                name: "中兴村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "白鱼湾镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "劳动村委会",
                code: "001",
            },
            VillageCode {
                name: "勤农村委会",
                code: "002",
            },
            VillageCode {
                name: "湖沿村委会",
                code: "003",
            },
            VillageCode {
                name: "齐心村委会",
                code: "004",
            },
            VillageCode {
                name: "胜利村委会",
                code: "005",
            },
            VillageCode {
                name: "临湖村委会",
                code: "006",
            },
            VillageCode {
                name: "长林子村委会",
                code: "007",
            },
            VillageCode {
                name: "蜂蜜山村委会",
                code: "008",
            },
            VillageCode {
                name: "白泡子村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "柳毛乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "永胜村委会",
                code: "001",
            },
            VillageCode {
                name: "团结村委会",
                code: "002",
            },
            VillageCode {
                name: "利民村委会",
                code: "003",
            },
            VillageCode {
                name: "富乡村委会",
                code: "004",
            },
            VillageCode {
                name: "同心村委会",
                code: "005",
            },
            VillageCode {
                name: "双合村委会",
                code: "006",
            },
            VillageCode {
                name: "新正村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "杨木乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "八五七农场社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "凌云村委会",
                code: "002",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "003",
            },
            VillageCode {
                name: "杨木村委会",
                code: "004",
            },
            VillageCode {
                name: "创业村委会",
                code: "005",
            },
            VillageCode {
                name: "伊通村委会",
                code: "006",
            },
            VillageCode {
                name: "金星村委会",
                code: "007",
            },
            VillageCode {
                name: "板石村委会",
                code: "008",
            },
            VillageCode {
                name: "红旗村委会",
                code: "009",
            },
            VillageCode {
                name: "育青村委会",
                code: "010",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "兴凯湖乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "兴凯湖农场社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "兴凯湖村委会",
                code: "002",
            },
            VillageCode {
                name: "石嘴子村委会",
                code: "003",
            },
            VillageCode {
                name: "金银库村委会",
                code: "004",
            },
            VillageCode {
                name: "马家岗村委会",
                code: "005",
            },
            VillageCode {
                name: "新民村委会",
                code: "006",
            },
            VillageCode {
                name: "爱民村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "承紫河乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "先锋村委会",
                code: "001",
            },
            VillageCode {
                name: "利湖村委会",
                code: "002",
            },
            VillageCode {
                name: "前进村委会",
                code: "003",
            },
            VillageCode {
                name: "光荣村委会",
                code: "004",
            },
            VillageCode {
                name: "继红村委会",
                code: "005",
            },
            VillageCode {
                name: "承紫河村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "二人班乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "红星村委会",
                code: "001",
            },
            VillageCode {
                name: "边疆村委会",
                code: "002",
            },
            VillageCode {
                name: "前哨村委会",
                code: "003",
            },
            VillageCode {
                name: "爱国村委会",
                code: "004",
            },
            VillageCode {
                name: "安定村委会",
                code: "005",
            },
            VillageCode {
                name: "新兴村委会",
                code: "006",
            },
            VillageCode {
                name: "安太村委会",
                code: "007",
            },
            VillageCode {
                name: "安康村委会",
                code: "008",
            },
            VillageCode {
                name: "二人班村委会",
                code: "009",
            },
            VillageCode {
                name: "尚礼村委会",
                code: "010",
            },
            VillageCode {
                name: "尚德村委会",
                code: "011",
            },
            VillageCode {
                name: "尚志村委会",
                code: "012",
            },
            VillageCode {
                name: "联城村委会",
                code: "013",
            },
            VillageCode {
                name: "正阳村委会",
                code: "014",
            },
            VillageCode {
                name: "集贤村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "太平乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "青松村委会",
                code: "001",
            },
            VillageCode {
                name: "太平村委会",
                code: "002",
            },
            VillageCode {
                name: "农丰村委会",
                code: "003",
            },
            VillageCode {
                name: "民主村委会",
                code: "004",
            },
            VillageCode {
                name: "宏林村委会",
                code: "005",
            },
            VillageCode {
                name: "庄内村委会",
                code: "006",
            },
            VillageCode {
                name: "合心村委会",
                code: "007",
            },
            VillageCode {
                name: "庄兴村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "和平乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "庆余村委会",
                code: "001",
            },
            VillageCode {
                name: "三人班村委会",
                code: "002",
            },
            VillageCode {
                name: "东兴村委会",
                code: "003",
            },
            VillageCode {
                name: "新城村委会",
                code: "004",
            },
            VillageCode {
                name: "东明村委会",
                code: "005",
            },
            VillageCode {
                name: "庆合村委会",
                code: "006",
            },
            VillageCode {
                name: "兴光村委会",
                code: "007",
            },
            VillageCode {
                name: "东鲜村委会",
                code: "008",
            },
            VillageCode {
                name: "新建村委会",
                code: "009",
            },
            VillageCode {
                name: "新田村委会",
                code: "010",
            },
            VillageCode {
                name: "幸福村委会",
                code: "011",
            },
            VillageCode {
                name: "东风村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "富源乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "民富村委会",
                code: "001",
            },
            VillageCode {
                name: "民政村委会",
                code: "002",
            },
            VillageCode {
                name: "民强村委会",
                code: "003",
            },
            VillageCode {
                name: "爱林村委会",
                code: "004",
            },
            VillageCode {
                name: "宝泉村委会",
                code: "005",
            },
            VillageCode {
                name: "金沙村委会",
                code: "006",
            },
            VillageCode {
                name: "珠山村委会",
                code: "007",
            },
            VillageCode {
                name: "富源村委会",
                code: "008",
            },
            VillageCode {
                name: "富强村委会",
                code: "009",
            },
            VillageCode {
                name: "富民村委会",
                code: "010",
            },
            VillageCode {
                name: "富升村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "林草局",
        code: "018",
        villages: &[
            VillageCode {
                name: "连珠山林场生活区",
                code: "001",
            },
            VillageCode {
                name: "蜂蜜山林场生活区",
                code: "002",
            },
            VillageCode {
                name: "大顶山林场生活区",
                code: "003",
            },
            VillageCode {
                name: "三道岭林场生活区",
                code: "004",
            },
            VillageCode {
                name: "二龙山林场生活区",
                code: "005",
            },
            VillageCode {
                name: "青梅山林场生活区",
                code: "006",
            },
            VillageCode {
                name: "金银库林场生活区",
                code: "007",
            },
            VillageCode {
                name: "金沙林场生活区",
                code: "008",
            },
            VillageCode {
                name: "珠山林场生活区",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "青年水库",
        code: "019",
        villages: &[VillageCode {
            name: "青年水库虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "煤管局",
        code: "020",
        villages: &[VillageCode {
            name: "珠山地区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "经济开发区管理委员会",
        code: "021",
        villages: &[VillageCode {
            name: "经济开发区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "水产养殖有限公司",
        code: "022",
        villages: &[VillageCode {
            name: "水产养殖有限公司虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "种畜场",
        code: "023",
        villages: &[VillageCode {
            name: "种畜场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "水田良种场",
        code: "024",
        villages: &[VillageCode {
            name: "水田良种场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "市良种场",
        code: "025",
        villages: &[VillageCode {
            name: "良种场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "校办企业公司",
        code: "026",
        villages: &[VillageCode {
            name: "校办企业公司农场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "蜂蜜山粮库有限公司",
        code: "027",
        villages: &[VillageCode {
            name: "蜂蜜山粮库虚拟生活区",
            code: "001",
        }],
    },
];

static TOWNS_HJ_002: [TownCode; 9] = [
    TownCode {
        name: "向阳街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "向阳社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "北山社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "东山社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "园林社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "南山街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "跃进社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "建设社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "康新社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "新建社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "立新街道",
        code: "003",
        villages: &[VillageCode {
            name: "矿部社区居民委员会",
            code: "001",
        }],
    },
    TownCode {
        name: "东风街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "龙行社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "四海居社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "中山社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "东岸社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "红军路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "月秀社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "红旗社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "赛洛城社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "西鸡西街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "幸福里社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "先锋社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "西山街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "西安社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "电台社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "西山社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "红星乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "红星村委会",
                code: "001",
            },
            VillageCode {
                name: "东太村委会",
                code: "002",
            },
            VillageCode {
                name: "红太村委会",
                code: "003",
            },
            VillageCode {
                name: "红胜村委会",
                code: "004",
            },
            VillageCode {
                name: "鸡兴村委会",
                code: "005",
            },
            VillageCode {
                name: "西太村委会",
                code: "006",
            },
            VillageCode {
                name: "前进村委会",
                code: "007",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "西郊乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "梁家村委会",
                code: "001",
            },
            VillageCode {
                name: "团结村委会",
                code: "002",
            },
            VillageCode {
                name: "三合村委会",
                code: "003",
            },
            VillageCode {
                name: "新发村委会",
                code: "004",
            },
            VillageCode {
                name: "西郊村委会",
                code: "005",
            },
            VillageCode {
                name: "东郊村委会",
                code: "006",
            },
        ],
    },
];

static TOWNS_HJ_003: [TownCode; 8] = [
    TownCode {
        name: "大恒山街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "桦木林社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "高锋社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "优胜社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "小恒山街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "宏伟社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "兴隆社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "二道河子街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "富荣社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "雄关社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "张新街道",
        code: "004",
        villages: &[VillageCode {
            name: "张新社区居民委员会",
            code: "001",
        }],
    },
    TownCode {
        name: "奋斗街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "恒新社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "安全社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "柳毛街道",
        code: "006",
        villages: &[VillageCode {
            name: "中兴社区居民委员会",
            code: "001",
        }],
    },
    TownCode {
        name: "红旗乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "张鲜村委会",
                code: "001",
            },
            VillageCode {
                name: "薛家村委会",
                code: "002",
            },
            VillageCode {
                name: "丰乐村委会",
                code: "003",
            },
            VillageCode {
                name: "民乐村委会",
                code: "004",
            },
            VillageCode {
                name: "丰鲜村委会",
                code: "005",
            },
            VillageCode {
                name: "义安村委会",
                code: "006",
            },
            VillageCode {
                name: "小恒山村委会",
                code: "007",
            },
            VillageCode {
                name: "红旗村委会",
                code: "008",
            },
            VillageCode {
                name: "安乐村委会",
                code: "009",
            },
            VillageCode {
                name: "长胜村委会",
                code: "010",
            },
            VillageCode {
                name: "合作村委会",
                code: "011",
            },
            VillageCode {
                name: "民主村委会",
                code: "012",
            },
            VillageCode {
                name: "胜利村委会",
                code: "013",
            },
            VillageCode {
                name: "艳胜村委会",
                code: "014",
            },
            VillageCode {
                name: "艳丰村委会",
                code: "015",
            },
            VillageCode {
                name: "艳东村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "柳毛乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "中心村委会",
                code: "001",
            },
            VillageCode {
                name: "柳毛村委会",
                code: "002",
            },
            VillageCode {
                name: "新胜村委会",
                code: "003",
            },
            VillageCode {
                name: "莲花村委会",
                code: "004",
            },
            VillageCode {
                name: "安山村委会",
                code: "005",
            },
            VillageCode {
                name: "铅矿村委会",
                code: "006",
            },
            VillageCode {
                name: "裕丰村委会",
                code: "007",
            },
            VillageCode {
                name: "光明村委会",
                code: "008",
            },
        ],
    },
];

static TOWNS_HJ_004: [TownCode; 6] = [
    TownCode {
        name: "东兴街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "新华社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "光华社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "白云社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "矿里街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "大半道社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东风社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "洗煤街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "洗煤社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "电厂社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "大通沟街道",
        code: "004",
        villages: &[VillageCode {
            name: "大通沟社区居民委员会",
            code: "001",
        }],
    },
    TownCode {
        name: "滴道河乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "金刚村委会",
                code: "001",
            },
            VillageCode {
                name: "金铁村委会",
                code: "002",
            },
            VillageCode {
                name: "金山村委会",
                code: "003",
            },
            VillageCode {
                name: "王家村委会",
                code: "004",
            },
            VillageCode {
                name: "同乐村委会",
                code: "005",
            },
            VillageCode {
                name: "南甸子村委会",
                code: "006",
            },
            VillageCode {
                name: "团山子村委会",
                code: "007",
            },
            VillageCode {
                name: "大通沟村委会",
                code: "008",
            },
            VillageCode {
                name: "荣丰村委会",
                code: "009",
            },
            VillageCode {
                name: "新民村委会",
                code: "010",
            },
            VillageCode {
                name: "新兴村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "兰岭乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "兰岭村委会",
                code: "001",
            },
            VillageCode {
                name: "永台村委会",
                code: "002",
            },
            VillageCode {
                name: "永胜村委会",
                code: "003",
            },
            VillageCode {
                name: "大同村委会",
                code: "004",
            },
            VillageCode {
                name: "平安村委会",
                code: "005",
            },
            VillageCode {
                name: "新立村委会",
                code: "006",
            },
            VillageCode {
                name: "河北村委会",
                code: "007",
            },
            VillageCode {
                name: "新建村委会",
                code: "008",
            },
        ],
    },
];

static TOWNS_HJ_005: [TownCode; 24] = [
    TownCode {
        name: "富强街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "树文社区",
                code: "001",
            },
            VillageCode {
                name: "树勤社区",
                code: "002",
            },
            VillageCode {
                name: "朝阳社区",
                code: "003",
            },
            VillageCode {
                name: "春阳社区",
                code: "004",
            },
            VillageCode {
                name: "幸福社区",
                code: "005",
            },
            VillageCode {
                name: "北杏山村",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "康平街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "康平社区",
                code: "001",
            },
            VillageCode {
                name: "奉和社区",
                code: "002",
            },
            VillageCode {
                name: "八里庙子村",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "霍家店街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "奉化社区",
                code: "001",
            },
            VillageCode {
                name: "天阳社区",
                code: "002",
            },
            VillageCode {
                name: "树俭社区",
                code: "003",
            },
            VillageCode {
                name: "兴旺社区",
                code: "004",
            },
            VillageCode {
                name: "园艺村",
                code: "005",
            },
            VillageCode {
                name: "霍家店村",
                code: "006",
            },
            VillageCode {
                name: "东白山村",
                code: "007",
            },
            VillageCode {
                name: "岫岩村",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "梨树镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "南杏山村",
                code: "001",
            },
            VillageCode {
                name: "大烟筒村",
                code: "002",
            },
            VillageCode {
                name: "后房身村",
                code: "003",
            },
            VillageCode {
                name: "北夏家村",
                code: "004",
            },
            VillageCode {
                name: "西平安村",
                code: "005",
            },
            VillageCode {
                name: "东平安村",
                code: "006",
            },
            VillageCode {
                name: "夏家堡村",
                code: "007",
            },
            VillageCode {
                name: "西中安村",
                code: "008",
            },
            VillageCode {
                name: "中安堡村",
                code: "009",
            },
            VillageCode {
                name: "双城子村",
                code: "010",
            },
            VillageCode {
                name: "马地方村",
                code: "011",
            },
            VillageCode {
                name: "北老壕村",
                code: "012",
            },
            VillageCode {
                name: "泉眼沟村",
                code: "013",
            },
            VillageCode {
                name: "郝家村",
                code: "014",
            },
            VillageCode {
                name: "后家巴村",
                code: "015",
            },
            VillageCode {
                name: "前房身村",
                code: "016",
            },
            VillageCode {
                name: "胡家窝堡村",
                code: "017",
            },
            VillageCode {
                name: "苗圃村",
                code: "018",
            },
            VillageCode {
                name: "高家窝圃村",
                code: "019",
            },
            VillageCode {
                name: "杨家窝堡村",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "郭家店镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "站北社区",
                code: "001",
            },
            VillageCode {
                name: "站南社区",
                code: "002",
            },
            VillageCode {
                name: "建材社区",
                code: "003",
            },
            VillageCode {
                name: "河南社区",
                code: "004",
            },
            VillageCode {
                name: "曙光社区",
                code: "005",
            },
            VillageCode {
                name: "晨晖社区",
                code: "006",
            },
            VillageCode {
                name: "大顶子山社区",
                code: "007",
            },
            VillageCode {
                name: "石槽沟村",
                code: "008",
            },
            VillageCode {
                name: "化石山村",
                code: "009",
            },
            VillageCode {
                name: "孙家屯村",
                code: "010",
            },
            VillageCode {
                name: "柴火沟村",
                code: "011",
            },
            VillageCode {
                name: "八家子村",
                code: "012",
            },
            VillageCode {
                name: "八里城村",
                code: "013",
            },
            VillageCode {
                name: "新胜村",
                code: "014",
            },
            VillageCode {
                name: "青堆子村",
                code: "015",
            },
            VillageCode {
                name: "小泉眼村",
                code: "016",
            },
            VillageCode {
                name: "双马架村",
                code: "017",
            },
            VillageCode {
                name: "东青石岭村",
                code: "018",
            },
            VillageCode {
                name: "西青石岭村",
                code: "019",
            },
            VillageCode {
                name: "花城子村",
                code: "020",
            },
            VillageCode {
                name: "四大家村",
                code: "021",
            },
            VillageCode {
                name: "东黑嘴子村",
                code: "022",
            },
            VillageCode {
                name: "镇郊村",
                code: "023",
            },
            VillageCode {
                name: "蔬菜村",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "榆树台镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "光明社区",
                code: "001",
            },
            VillageCode {
                name: "银河社区",
                code: "002",
            },
            VillageCode {
                name: "路明村",
                code: "003",
            },
            VillageCode {
                name: "新合村",
                code: "004",
            },
            VillageCode {
                name: "三合村",
                code: "005",
            },
            VillageCode {
                name: "团结村",
                code: "006",
            },
            VillageCode {
                name: "六合村",
                code: "007",
            },
            VillageCode {
                name: "新兴村",
                code: "008",
            },
            VillageCode {
                name: "徐家村",
                code: "009",
            },
            VillageCode {
                name: "东胜村",
                code: "010",
            },
            VillageCode {
                name: "房身村",
                code: "011",
            },
            VillageCode {
                name: "阎家村",
                code: "012",
            },
            VillageCode {
                name: "厢房村",
                code: "013",
            },
            VillageCode {
                name: "新江村",
                code: "014",
            },
            VillageCode {
                name: "双龙村",
                code: "015",
            },
            VillageCode {
                name: "董家窝堡村",
                code: "016",
            },
            VillageCode {
                name: "袁家岭村",
                code: "017",
            },
            VillageCode {
                name: "周家油坊村",
                code: "018",
            },
            VillageCode {
                name: "兴发卜村",
                code: "019",
            },
            VillageCode {
                name: "大榆树村",
                code: "020",
            },
            VillageCode {
                name: "张家街村",
                code: "021",
            },
            VillageCode {
                name: "张家油坊村",
                code: "022",
            },
            VillageCode {
                name: "潘家村",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "孤家子镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "沈洋社区",
                code: "001",
            },
            VillageCode {
                name: "兴隆社区",
                code: "002",
            },
            VillageCode {
                name: "电塔社区",
                code: "003",
            },
            VillageCode {
                name: "文明社区",
                code: "004",
            },
            VillageCode {
                name: "韩道良社区",
                code: "005",
            },
            VillageCode {
                name: "三塔社区",
                code: "006",
            },
            VillageCode {
                name: "小宽社区",
                code: "007",
            },
            VillageCode {
                name: "茅山社区",
                code: "008",
            },
            VillageCode {
                name: "福宁社区",
                code: "009",
            },
            VillageCode {
                name: "新鲜社区",
                code: "010",
            },
            VillageCode {
                name: "孤家子村",
                code: "011",
            },
            VillageCode {
                name: "大林子村",
                code: "012",
            },
            VillageCode {
                name: "两家子村",
                code: "013",
            },
            VillageCode {
                name: "马家窑村",
                code: "014",
            },
            VillageCode {
                name: "张家街村",
                code: "015",
            },
            VillageCode {
                name: "团山子村",
                code: "016",
            },
            VillageCode {
                name: "七里界村",
                code: "017",
            },
            VillageCode {
                name: "红旗村",
                code: "018",
            },
            VillageCode {
                name: "于家街村",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "小城子镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "新城社区",
                code: "001",
            },
            VillageCode {
                name: "爱德村",
                code: "002",
            },
            VillageCode {
                name: "长山村",
                code: "003",
            },
            VillageCode {
                name: "船口村",
                code: "004",
            },
            VillageCode {
                name: "大桥村",
                code: "005",
            },
            VillageCode {
                name: "东河山村",
                code: "006",
            },
            VillageCode {
                name: "六屋村",
                code: "007",
            },
            VillageCode {
                name: "平庄村",
                code: "008",
            },
            VillageCode {
                name: "亲仁村",
                code: "009",
            },
            VillageCode {
                name: "土龙村",
                code: "010",
            },
            VillageCode {
                name: "围子村",
                code: "011",
            },
            VillageCode {
                name: "西道村",
                code: "012",
            },
            VillageCode {
                name: "西河山村",
                code: "013",
            },
            VillageCode {
                name: "新家村",
                code: "014",
            },
            VillageCode {
                name: "新农村村",
                code: "015",
            },
            VillageCode {
                name: "友贤村",
                code: "016",
            },
            VillageCode {
                name: "中央堡村",
                code: "017",
            },
            VillageCode {
                name: "大房身村",
                code: "018",
            },
            VillageCode {
                name: "二里界村",
                code: "019",
            },
            VillageCode {
                name: "张家窝堡村",
                code: "020",
            },
            VillageCode {
                name: "庆东村",
                code: "021",
            },
            VillageCode {
                name: "同庆村",
                code: "022",
            },
            VillageCode {
                name: "江东道村",
                code: "023",
            },
            VillageCode {
                name: "高山村",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "喇嘛甸镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "昌平社区",
                code: "001",
            },
            VillageCode {
                name: "高家窝堡村",
                code: "002",
            },
            VillageCode {
                name: "喇嘛甸村",
                code: "003",
            },
            VillageCode {
                name: "老程窝堡村",
                code: "004",
            },
            VillageCode {
                name: "梨树贝村",
                code: "005",
            },
            VillageCode {
                name: "柳树营村",
                code: "006",
            },
            VillageCode {
                name: "六家子村",
                code: "007",
            },
            VillageCode {
                name: "牛家窝堡村",
                code: "008",
            },
            VillageCode {
                name: "彭家窝堡村",
                code: "009",
            },
            VillageCode {
                name: "平岭村",
                code: "010",
            },
            VillageCode {
                name: "前加把村",
                code: "011",
            },
            VillageCode {
                name: "申染房村",
                code: "012",
            },
            VillageCode {
                name: "王家园子村",
                code: "013",
            },
            VillageCode {
                name: "一棵树村",
                code: "014",
            },
            VillageCode {
                name: "前胡家村",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "蔡家镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "蔡家小站社区",
                code: "001",
            },
            VillageCode {
                name: "爱国村",
                code: "002",
            },
            VillageCode {
                name: "蔡家村",
                code: "003",
            },
            VillageCode {
                name: "横道子村",
                code: "004",
            },
            VillageCode {
                name: "敬友村",
                code: "005",
            },
            VillageCode {
                name: "拉腰子村",
                code: "006",
            },
            VillageCode {
                name: "马家村",
                code: "007",
            },
            VillageCode {
                name: "孟家村",
                code: "008",
            },
            VillageCode {
                name: "娘娘庙村",
                code: "009",
            },
            VillageCode {
                name: "下坎子村",
                code: "010",
            },
            VillageCode {
                name: "新村村",
                code: "011",
            },
            VillageCode {
                name: "姚家村",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "刘家馆子镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "长安社区",
                code: "001",
            },
            VillageCode {
                name: "北六家子村",
                code: "002",
            },
            VillageCode {
                name: "大力虎村",
                code: "003",
            },
            VillageCode {
                name: "东卡篓村",
                code: "004",
            },
            VillageCode {
                name: "东五家村",
                code: "005",
            },
            VillageCode {
                name: "韩家村",
                code: "006",
            },
            VillageCode {
                name: "纪家村",
                code: "007",
            },
            VillageCode {
                name: "刘家馆子村",
                code: "008",
            },
            VillageCode {
                name: "龙山村",
                code: "009",
            },
            VillageCode {
                name: "南六家子村",
                code: "010",
            },
            VillageCode {
                name: "三家窝堡村",
                code: "011",
            },
            VillageCode {
                name: "炭窑村",
                code: "012",
            },
            VillageCode {
                name: "王河村",
                code: "013",
            },
            VillageCode {
                name: "苇田村",
                code: "014",
            },
            VillageCode {
                name: "吴家坨子村",
                code: "015",
            },
            VillageCode {
                name: "乌兰村",
                code: "016",
            },
            VillageCode {
                name: "西卡篓村",
                code: "017",
            },
            VillageCode {
                name: "西五家村",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "十家堡镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "十家堡社区",
                code: "001",
            },
            VillageCode {
                name: "八棵树村",
                code: "002",
            },
            VillageCode {
                name: "八盘碾子村",
                code: "003",
            },
            VillageCode {
                name: "何家村",
                code: "004",
            },
            VillageCode {
                name: "靠山屯村",
                code: "005",
            },
            VillageCode {
                name: "龙湾村",
                code: "006",
            },
            VillageCode {
                name: "龙王庙村",
                code: "007",
            },
            VillageCode {
                name: "三家子村",
                code: "008",
            },
            VillageCode {
                name: "上三台村",
                code: "009",
            },
            VillageCode {
                name: "十家堡村",
                code: "010",
            },
            VillageCode {
                name: "太阳沟村",
                code: "011",
            },
            VillageCode {
                name: "铁岭窝堡村",
                code: "012",
            },
            VillageCode {
                name: "王相村委会",
                code: "013",
            },
            VillageCode {
                name: "西黑嘴子村",
                code: "014",
            },
            VillageCode {
                name: "小桥子村",
                code: "015",
            },
            VillageCode {
                name: "营城子村",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "孟家岭镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "青山路社区",
                code: "001",
            },
            VillageCode {
                name: "孟家岭村",
                code: "002",
            },
            VillageCode {
                name: "马家油坊村",
                code: "003",
            },
            VillageCode {
                name: "苏家村",
                code: "004",
            },
            VillageCode {
                name: "下安村",
                code: "005",
            },
            VillageCode {
                name: "大河沿村",
                code: "006",
            },
            VillageCode {
                name: "赫尔苏门村",
                code: "007",
            },
            VillageCode {
                name: "二道沟村",
                code: "008",
            },
            VillageCode {
                name: "潘家沟村",
                code: "009",
            },
            VillageCode {
                name: "四台子村",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "万发镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "永发社区",
                code: "001",
            },
            VillageCode {
                name: "长胜村",
                code: "002",
            },
            VillageCode {
                name: "东万发村",
                code: "003",
            },
            VillageCode {
                name: "关家岗子村",
                code: "004",
            },
            VillageCode {
                name: "贾杂铺村",
                code: "005",
            },
            VillageCode {
                name: "李家店村",
                code: "006",
            },
            VillageCode {
                name: "刘家岗子村",
                code: "007",
            },
            VillageCode {
                name: "吕家岗子村",
                code: "008",
            },
            VillageCode {
                name: "孙家店村",
                code: "009",
            },
            VillageCode {
                name: "西万发村",
                code: "010",
            },
            VillageCode {
                name: "幸福村",
                code: "011",
            },
            VillageCode {
                name: "朱家村",
                code: "012",
            },
            VillageCode {
                name: "闫家堡子村",
                code: "013",
            },
            VillageCode {
                name: "北太平村",
                code: "014",
            },
            VillageCode {
                name: "毕家堡子村",
                code: "015",
            },
            VillageCode {
                name: "李家村",
                code: "016",
            },
            VillageCode {
                name: "龙母庙村",
                code: "017",
            },
            VillageCode {
                name: "牟家村",
                code: "018",
            },
            VillageCode {
                name: "南太平村",
                code: "019",
            },
            VillageCode {
                name: "前梁家村",
                code: "020",
            },
            VillageCode {
                name: "青松村",
                code: "021",
            },
            VillageCode {
                name: "宋家围子村",
                code: "022",
            },
            VillageCode {
                name: "田家洼子村",
                code: "023",
            },
            VillageCode {
                name: "榆树村",
                code: "024",
            },
            VillageCode {
                name: "张家村",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "东河镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "赵家店社区",
                code: "001",
            },
            VillageCode {
                name: "东河村",
                code: "002",
            },
            VillageCode {
                name: "梁家岗子村",
                code: "003",
            },
            VillageCode {
                name: "胜利村",
                code: "004",
            },
            VillageCode {
                name: "双城子村",
                code: "005",
            },
            VillageCode {
                name: "双树子村",
                code: "006",
            },
            VillageCode {
                name: "松树村",
                code: "007",
            },
            VillageCode {
                name: "王平房村",
                code: "008",
            },
            VillageCode {
                name: "五业村",
                code: "009",
            },
            VillageCode {
                name: "新发村",
                code: "010",
            },
            VillageCode {
                name: "业家村",
                code: "011",
            },
            VillageCode {
                name: "赵家店村",
                code: "012",
            },
            VillageCode {
                name: "周家岗子村",
                code: "013",
            },
            VillageCode {
                name: "良种繁殖场生活区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "沈洋镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "沈新社区",
                code: "001",
            },
            VillageCode {
                name: "后太平村",
                code: "002",
            },
            VillageCode {
                name: "兴无村",
                code: "003",
            },
            VillageCode {
                name: "辽河村",
                code: "004",
            },
            VillageCode {
                name: "丰收村",
                code: "005",
            },
            VillageCode {
                name: "翻身村",
                code: "006",
            },
            VillageCode {
                name: "工农村",
                code: "007",
            },
            VillageCode {
                name: "前太平村",
                code: "008",
            },
            VillageCode {
                name: "白沙坨村",
                code: "009",
            },
            VillageCode {
                name: "张家堡子村",
                code: "010",
            },
            VillageCode {
                name: "大孤山村",
                code: "011",
            },
            VillageCode {
                name: "闫达村",
                code: "012",
            },
            VillageCode {
                name: "李家街村",
                code: "013",
            },
            VillageCode {
                name: "沈洋村",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "林海镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "爱欣社区",
                code: "001",
            },
            VillageCode {
                name: "双山村",
                code: "002",
            },
            VillageCode {
                name: "五家子村",
                code: "003",
            },
            VillageCode {
                name: "绿海村",
                code: "004",
            },
            VillageCode {
                name: "李家围子村",
                code: "005",
            },
            VillageCode {
                name: "大门丁村",
                code: "006",
            },
            VillageCode {
                name: "夏窑村",
                code: "007",
            },
            VillageCode {
                name: "揣家洼子村",
                code: "008",
            },
            VillageCode {
                name: "头道岗村",
                code: "009",
            },
            VillageCode {
                name: "长丰村",
                code: "010",
            },
            VillageCode {
                name: "顺山村",
                code: "011",
            },
            VillageCode {
                name: "老奤村",
                code: "012",
            },
            VillageCode {
                name: "夏甸子村",
                code: "013",
            },
            VillageCode {
                name: "兴开城村",
                code: "014",
            },
            VillageCode {
                name: "王家局子村",
                code: "015",
            },
            VillageCode {
                name: "靠山李村",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "小宽镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "宽城社区",
                code: "001",
            },
            VillageCode {
                name: "小宽村",
                code: "002",
            },
            VillageCode {
                name: "西河村",
                code: "003",
            },
            VillageCode {
                name: "长发村",
                code: "004",
            },
            VillageCode {
                name: "五家户村",
                code: "005",
            },
            VillageCode {
                name: "新风村",
                code: "006",
            },
            VillageCode {
                name: "宏伟村",
                code: "007",
            },
            VillageCode {
                name: "长兴村",
                code: "008",
            },
            VillageCode {
                name: "大宽村",
                code: "009",
            },
            VillageCode {
                name: "陈家村",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "白山乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "裴家村",
                code: "001",
            },
            VillageCode {
                name: "鲍家村",
                code: "002",
            },
            VillageCode {
                name: "老山头村",
                code: "003",
            },
            VillageCode {
                name: "刘家窝堡村",
                code: "004",
            },
            VillageCode {
                name: "大泉眼村",
                code: "005",
            },
            VillageCode {
                name: "四合村",
                code: "006",
            },
            VillageCode {
                name: "西白山村",
                code: "007",
            },
            VillageCode {
                name: "石家堡村",
                code: "008",
            },
            VillageCode {
                name: "隋家村",
                code: "009",
            },
            VillageCode {
                name: "郑家村",
                code: "010",
            },
            VillageCode {
                name: "东风村",
                code: "011",
            },
            VillageCode {
                name: "友谊村",
                code: "012",
            },
            VillageCode {
                name: "平山村",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "泉眼岭乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "西泉村",
                code: "001",
            },
            VillageCode {
                name: "东泉村",
                code: "002",
            },
            VillageCode {
                name: "蒋机房村",
                code: "003",
            },
            VillageCode {
                name: "新发卜村",
                code: "004",
            },
            VillageCode {
                name: "玻璃城子村",
                code: "005",
            },
            VillageCode {
                name: "南泉村",
                code: "006",
            },
            VillageCode {
                name: "小房身村",
                code: "007",
            },
            VillageCode {
                name: "东洼子村",
                code: "008",
            },
            VillageCode {
                name: "常青村",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "胜利乡",
        code: "021",
        villages: &[
            VillageCode {
                name: "关家屯村",
                code: "001",
            },
            VillageCode {
                name: "顺城堡村",
                code: "002",
            },
            VillageCode {
                name: "十家子村",
                code: "003",
            },
            VillageCode {
                name: "九家子村",
                code: "004",
            },
            VillageCode {
                name: "长发堡村",
                code: "005",
            },
            VillageCode {
                name: "郭家窝堡村",
                code: "006",
            },
            VillageCode {
                name: "四家子村",
                code: "007",
            },
            VillageCode {
                name: "代家堡村",
                code: "008",
            },
            VillageCode {
                name: "小城子村",
                code: "009",
            },
            VillageCode {
                name: "羊尾岭村",
                code: "010",
            },
            VillageCode {
                name: "南老奤村",
                code: "011",
            },
            VillageCode {
                name: "石庙子村",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "四棵树乡",
        code: "022",
        villages: &[
            VillageCode {
                name: "四棵树村",
                code: "001",
            },
            VillageCode {
                name: "七家子村",
                code: "002",
            },
            VillageCode {
                name: "小郑屯村",
                code: "003",
            },
            VillageCode {
                name: "三棵树村",
                code: "004",
            },
            VillageCode {
                name: "王家桥村",
                code: "005",
            },
            VillageCode {
                name: "李家桥村",
                code: "006",
            },
            VillageCode {
                name: "十二马架村",
                code: "007",
            },
            VillageCode {
                name: "长山堡村",
                code: "008",
            },
            VillageCode {
                name: "后韩家村",
                code: "009",
            },
            VillageCode {
                name: "安家屯村",
                code: "010",
            },
            VillageCode {
                name: "傅家街村",
                code: "011",
            },
            VillageCode {
                name: "田家庙村",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "双河乡",
        code: "023",
        villages: &[
            VillageCode {
                name: "于大壕村",
                code: "001",
            },
            VillageCode {
                name: "三合堡村",
                code: "002",
            },
            VillageCode {
                name: "范家屯村",
                code: "003",
            },
            VillageCode {
                name: "杨船口村",
                code: "004",
            },
            VillageCode {
                name: "陈大窝堡村",
                code: "005",
            },
            VillageCode {
                name: "刘家炉村",
                code: "006",
            },
            VillageCode {
                name: "腰窝堡村",
                code: "007",
            },
            VillageCode {
                name: "柳家屯村",
                code: "008",
            },
            VillageCode {
                name: "三道岗子村",
                code: "009",
            },
            VillageCode {
                name: "平安村",
                code: "010",
            },
            VillageCode {
                name: "王木铺村",
                code: "011",
            },
            VillageCode {
                name: "双顶子村",
                code: "012",
            },
            VillageCode {
                name: "刘家屯村",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "金山乡",
        code: "024",
        villages: &[
            VillageCode {
                name: "金山村",
                code: "001",
            },
            VillageCode {
                name: "平安堡村",
                code: "002",
            },
            VillageCode {
                name: "大城子村",
                code: "003",
            },
            VillageCode {
                name: "长岭子村",
                code: "004",
            },
            VillageCode {
                name: "旱河村",
                code: "005",
            },
            VillageCode {
                name: "南岗子村",
                code: "006",
            },
            VillageCode {
                name: "朝阳村",
                code: "007",
            },
            VillageCode {
                name: "沿河村",
                code: "008",
            },
            VillageCode {
                name: "三合屯村",
                code: "009",
            },
            VillageCode {
                name: "崔家岗子村",
                code: "010",
            },
        ],
    },
];

static TOWNS_HJ_006: [TownCode; 7] = [
    TownCode {
        name: "城子河街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "老房社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "花园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "长青社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "新立社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "城砖社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "城海社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "三建社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "金三角社区居民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "正阳街道",
        code: "002",
        villages: &[VillageCode {
            name: "正阳社区居民委员会",
            code: "001",
        }],
    },
    TownCode {
        name: "东海街道",
        code: "003",
        villages: &[VillageCode {
            name: "东海社区居民委员会",
            code: "001",
        }],
    },
    TownCode {
        name: "城西街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "晨光社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "白石社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "总厂社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "平安社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "杏花街道",
        code: "005",
        villages: &[VillageCode {
            name: "杏花社区居民委员会",
            code: "001",
        }],
    },
    TownCode {
        name: "长青乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "新阳村委会",
                code: "001",
            },
            VillageCode {
                name: "向阳村委会",
                code: "002",
            },
            VillageCode {
                name: "和平村委会",
                code: "003",
            },
            VillageCode {
                name: "正阳村委会",
                code: "004",
            },
            VillageCode {
                name: "城东村委会",
                code: "005",
            },
            VillageCode {
                name: "红卫村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "永丰乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "新华村委会",
                code: "001",
            },
            VillageCode {
                name: "新兴村委会",
                code: "002",
            },
            VillageCode {
                name: "新城村委会",
                code: "003",
            },
            VillageCode {
                name: "永平村委会",
                code: "004",
            },
            VillageCode {
                name: "丰安村委会",
                code: "005",
            },
            VillageCode {
                name: "永红村委会",
                code: "006",
            },
            VillageCode {
                name: "城子河村委会",
                code: "007",
            },
        ],
    },
];

static TOWNS_HJ_007: [TownCode; 2] = [
    TownCode {
        name: "麻山街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "建国社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "中心社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "前进社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "麻山镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "太和村委会",
                code: "001",
            },
            VillageCode {
                name: "吉祥村委会",
                code: "002",
            },
            VillageCode {
                name: "龙山村委会",
                code: "003",
            },
            VillageCode {
                name: "新光村委会",
                code: "004",
            },
            VillageCode {
                name: "前东新村委会",
                code: "005",
            },
            VillageCode {
                name: "麻山村委会",
                code: "006",
            },
            VillageCode {
                name: "新发村委会",
                code: "007",
            },
            VillageCode {
                name: "和平村委会",
                code: "008",
            },
            VillageCode {
                name: "共荣村委会",
                code: "009",
            },
            VillageCode {
                name: "双岭村委会",
                code: "010",
            },
            VillageCode {
                name: "后东新村委会",
                code: "011",
            },
            VillageCode {
                name: "土顶子村委会",
                code: "012",
            },
            VillageCode {
                name: "五龙村委会",
                code: "013",
            },
            VillageCode {
                name: "青龙村委会",
                code: "014",
            },
            VillageCode {
                name: "西大坡村委会",
                code: "015",
            },
            VillageCode {
                name: "山河村委会",
                code: "016",
            },
        ],
    },
];

static TOWNS_HJ_008: [TownCode; 13] = [
    TownCode {
        name: "鸡东镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "镇中社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "前进社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北华社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "东风社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "城南社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "南华社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "城西社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "新城社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "石河北村委会",
                code: "009",
            },
            VillageCode {
                name: "银峰村委会",
                code: "010",
            },
            VillageCode {
                name: "银河村委会",
                code: "011",
            },
            VillageCode {
                name: "银东村委会",
                code: "012",
            },
            VillageCode {
                name: "红胜村委会",
                code: "013",
            },
            VillageCode {
                name: "荣华村委会",
                code: "014",
            },
            VillageCode {
                name: "保中村委会",
                code: "015",
            },
            VillageCode {
                name: "得胜村委会",
                code: "016",
            },
            VillageCode {
                name: "勇鲜村委会",
                code: "017",
            },
            VillageCode {
                name: "光荣村委会",
                code: "018",
            },
            VillageCode {
                name: "古山子村委会",
                code: "019",
            },
            VillageCode {
                name: "明俊村委会",
                code: "020",
            },
            VillageCode {
                name: "张家村委会",
                code: "021",
            },
            VillageCode {
                name: "新峰村委会",
                code: "022",
            },
            VillageCode {
                name: "鸡东村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "平阳镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "平阳社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "永发村委会",
                code: "002",
            },
            VillageCode {
                name: "平阳村委会",
                code: "003",
            },
            VillageCode {
                name: "牛心山村委会",
                code: "004",
            },
            VillageCode {
                name: "永长村委会",
                code: "005",
            },
            VillageCode {
                name: "河南村委会",
                code: "006",
            },
            VillageCode {
                name: "永隆村委会",
                code: "007",
            },
            VillageCode {
                name: "金城村委会",
                code: "008",
            },
            VillageCode {
                name: "希贤村委会",
                code: "009",
            },
            VillageCode {
                name: "永兴村委会",
                code: "010",
            },
            VillageCode {
                name: "新发村委会",
                code: "011",
            },
            VillageCode {
                name: "新城村委会",
                code: "012",
            },
            VillageCode {
                name: "富国村委会",
                code: "013",
            },
            VillageCode {
                name: "金生村委会",
                code: "014",
            },
            VillageCode {
                name: "前卫村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "向阳镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "向阳社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "联合村委会",
                code: "002",
            },
            VillageCode {
                name: "曲河村委会",
                code: "003",
            },
            VillageCode {
                name: "通街村委会",
                code: "004",
            },
            VillageCode {
                name: "红星村委会",
                code: "005",
            },
            VillageCode {
                name: "东河村委会",
                code: "006",
            },
            VillageCode {
                name: "忠信村委会",
                code: "007",
            },
            VillageCode {
                name: "卫国村委会",
                code: "008",
            },
            VillageCode {
                name: "古城村委会",
                code: "009",
            },
            VillageCode {
                name: "向阳村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "哈达镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "哈达社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "保合社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "哈达村委会",
                code: "003",
            },
            VillageCode {
                name: "先锋村委会",
                code: "004",
            },
            VillageCode {
                name: "太阳村委会",
                code: "005",
            },
            VillageCode {
                name: "东风村委会",
                code: "006",
            },
            VillageCode {
                name: "山河村委会",
                code: "007",
            },
            VillageCode {
                name: "程家村委会",
                code: "008",
            },
            VillageCode {
                name: "杏花村委会",
                code: "009",
            },
            VillageCode {
                name: "双保村委会",
                code: "010",
            },
            VillageCode {
                name: "青山村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "永安镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "永安社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "永丰社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "永安村委会",
                code: "003",
            },
            VillageCode {
                name: "永平村委会",
                code: "004",
            },
            VillageCode {
                name: "永东村委会",
                code: "005",
            },
            VillageCode {
                name: "永红村委会",
                code: "006",
            },
            VillageCode {
                name: "永乐村委会",
                code: "007",
            },
            VillageCode {
                name: "永丰村委会",
                code: "008",
            },
            VillageCode {
                name: "永宁村委会",
                code: "009",
            },
            VillageCode {
                name: "永生村委会",
                code: "010",
            },
            VillageCode {
                name: "永丽村委会",
                code: "011",
            },
            VillageCode {
                name: "永新村委会",
                code: "012",
            },
            VillageCode {
                name: "永政村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "永和镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "永和社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "荣华社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "永和村委会",
                code: "003",
            },
            VillageCode {
                name: "公平村委会",
                code: "004",
            },
            VillageCode {
                name: "永胜村委会",
                code: "005",
            },
            VillageCode {
                name: "东安村委会",
                code: "006",
            },
            VillageCode {
                name: "新乐村委会",
                code: "007",
            },
            VillageCode {
                name: "林安村委会",
                code: "008",
            },
            VillageCode {
                name: "保安村委会",
                code: "009",
            },
            VillageCode {
                name: "东进村委会",
                code: "010",
            },
            VillageCode {
                name: "永庆村委会",
                code: "011",
            },
            VillageCode {
                name: "新和村委会",
                code: "012",
            },
            VillageCode {
                name: "新安村委会",
                code: "013",
            },
            VillageCode {
                name: "长安村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "东海镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "东海社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "兴国村委会",
                code: "002",
            },
            VillageCode {
                name: "幸福村委会",
                code: "003",
            },
            VillageCode {
                name: "长兴村委会",
                code: "004",
            },
            VillageCode {
                name: "东升村委会",
                code: "005",
            },
            VillageCode {
                name: "新华村委会",
                code: "006",
            },
            VillageCode {
                name: "永泉村委会",
                code: "007",
            },
            VillageCode {
                name: "长山村委会",
                code: "008",
            },
            VillageCode {
                name: "建设村委会",
                code: "009",
            },
            VillageCode {
                name: "东海村委会",
                code: "010",
            },
            VillageCode {
                name: "发展村委会",
                code: "011",
            },
            VillageCode {
                name: "新生村委会",
                code: "012",
            },
            VillageCode {
                name: "高峰村委会",
                code: "013",
            },
            VillageCode {
                name: "群英村委会",
                code: "014",
            },
            VillageCode {
                name: "新泉村委会",
                code: "015",
            },
            VillageCode {
                name: "永远村委会",
                code: "016",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "兴农镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "兴农社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "四海社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "兴农镇宝泉社区",
                code: "003",
            },
            VillageCode {
                name: "兴农村委会",
                code: "004",
            },
            VillageCode {
                name: "太平村委会",
                code: "005",
            },
            VillageCode {
                name: "奋斗村委会",
                code: "006",
            },
            VillageCode {
                name: "东保村委会",
                code: "007",
            },
            VillageCode {
                name: "红旗村委会",
                code: "008",
            },
            VillageCode {
                name: "四海村委会",
                code: "009",
            },
            VillageCode {
                name: "双山村委会",
                code: "010",
            },
            VillageCode {
                name: "兴林村委会",
                code: "011",
            },
            VillageCode {
                name: "富强村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "鸡林乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "鸡林村委会",
                code: "001",
            },
            VillageCode {
                name: "东兴村委会",
                code: "002",
            },
            VillageCode {
                name: "东明村委会",
                code: "003",
            },
            VillageCode {
                name: "进兴村委会",
                code: "004",
            },
            VillageCode {
                name: "永光村委会",
                code: "005",
            },
            VillageCode {
                name: "前进村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "明德乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "北河村委会",
                code: "001",
            },
            VillageCode {
                name: "更新村委会",
                code: "002",
            },
            VillageCode {
                name: "立新村委会",
                code: "003",
            },
            VillageCode {
                name: "曙光村委会",
                code: "004",
            },
            VillageCode {
                name: "红火村委会",
                code: "005",
            },
            VillageCode {
                name: "五星村委会",
                code: "006",
            },
            VillageCode {
                name: "明德村委会",
                code: "007",
            },
            VillageCode {
                name: "建政村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "下亮子乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "下亮子村委会",
                code: "001",
            },
            VillageCode {
                name: "裕国村委会",
                code: "002",
            },
            VillageCode {
                name: "裕民村委会",
                code: "003",
            },
            VillageCode {
                name: "复兴村委会",
                code: "004",
            },
            VillageCode {
                name: "久泰村委会",
                code: "005",
            },
            VillageCode {
                name: "新立村委会",
                code: "006",
            },
            VillageCode {
                name: "正乡村委会",
                code: "007",
            },
            VillageCode {
                name: "西庄村委会",
                code: "008",
            },
            VillageCode {
                name: "亮鲜村委会",
                code: "009",
            },
            VillageCode {
                name: "综合村委会",
                code: "010",
            },
            VillageCode {
                name: "三排村委会",
                code: "011",
            },
            VillageCode {
                name: "四排村委会",
                code: "012",
            },
            VillageCode {
                name: "长庆村委会",
                code: "013",
            },
            VillageCode {
                name: "柳河村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "林业局",
        code: "012",
        villages: &[
            VillageCode {
                name: "联合林场生活区",
                code: "001",
            },
            VillageCode {
                name: "凤凰山林场生活区",
                code: "002",
            },
            VillageCode {
                name: "西南岔林场生活区",
                code: "003",
            },
            VillageCode {
                name: "平房林场生活区",
                code: "004",
            },
            VillageCode {
                name: "四山林场生活区",
                code: "005",
            },
            VillageCode {
                name: "宝泉林场生活区",
                code: "006",
            },
            VillageCode {
                name: "曙光林场生活区",
                code: "007",
            },
            VillageCode {
                name: "红旗林场生活区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "八五一零农场",
        code: "013",
        villages: &[
            VillageCode {
                name: "八五一零场直社区",
                code: "001",
            },
            VillageCode {
                name: "八五一０农场杨木林子管理区",
                code: "002",
            },
            VillageCode {
                name: "八五一０农场当壁镇管理区",
                code: "003",
            },
            VillageCode {
                name: "八五一０农场新垦管理区",
                code: "004",
            },
            VillageCode {
                name: "八五一０农场黑背山管理区",
                code: "005",
            },
        ],
    },
];

static TOWNS_HJ_009: [TownCode; 19] = [
    TownCode {
        name: "虎林镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城东社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "中心社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "曙光社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "铁南社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "西苑社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "同和村委会",
                code: "006",
            },
            VillageCode {
                name: "桦树村委会",
                code: "007",
            },
            VillageCode {
                name: "义和村委会",
                code: "008",
            },
            VillageCode {
                name: "西岗村委会",
                code: "009",
            },
            VillageCode {
                name: "东升村委会",
                code: "010",
            },
            VillageCode {
                name: "桦南村委会",
                code: "011",
            },
            VillageCode {
                name: "于林村委会",
                code: "012",
            },
            VillageCode {
                name: "东源村委会",
                code: "013",
            },
            VillageCode {
                name: "安乐村委会",
                code: "014",
            },
            VillageCode {
                name: "镇兴村委会",
                code: "015",
            },
            VillageCode {
                name: "耕农村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "东方红镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "铁路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "粮库社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "兴阳村委会",
                code: "003",
            },
            VillageCode {
                name: "富先村委会",
                code: "004",
            },
            VillageCode {
                name: "东方村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "迎春镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "第一社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "第二社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "第三社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "车站社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "迎春村委会",
                code: "005",
            },
            VillageCode {
                name: "镇西村委会",
                code: "006",
            },
            VillageCode {
                name: "曙光村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "虎头镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "半站村委会",
                code: "001",
            },
            VillageCode {
                name: "月牙村委会",
                code: "002",
            },
            VillageCode {
                name: "卫疆村委会",
                code: "003",
            },
            VillageCode {
                name: "朱德山村委会",
                code: "004",
            },
            VillageCode {
                name: "富路村委会",
                code: "005",
            },
            VillageCode {
                name: "大王家村委会",
                code: "006",
            },
            VillageCode {
                name: "虎头村委会",
                code: "007",
            },
            VillageCode {
                name: "新岗村委会",
                code: "008",
            },
            VillageCode {
                name: "新庆村委会",
                code: "009",
            },
            VillageCode {
                name: "新兴村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "杨岗镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "新建村委会",
                code: "001",
            },
            VillageCode {
                name: "杨树河村委会",
                code: "002",
            },
            VillageCode {
                name: "杨岗村委会",
                code: "003",
            },
            VillageCode {
                name: "六人班村委会",
                code: "004",
            },
            VillageCode {
                name: "富国村委会",
                code: "005",
            },
            VillageCode {
                name: "湖北村委会",
                code: "006",
            },
            VillageCode {
                name: "合民村委会",
                code: "007",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "东诚镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "复兴村委会",
                code: "001",
            },
            VillageCode {
                name: "永丰村委会",
                code: "002",
            },
            VillageCode {
                name: "清和村委会",
                code: "003",
            },
            VillageCode {
                name: "新风村委会",
                code: "004",
            },
            VillageCode {
                name: "东风村委会",
                code: "005",
            },
            VillageCode {
                name: "和平村委会",
                code: "006",
            },
            VillageCode {
                name: "三林村委会",
                code: "007",
            },
            VillageCode {
                name: "仁爱村委会",
                code: "008",
            },
            VillageCode {
                name: "忠信村委会",
                code: "009",
            },
            VillageCode {
                name: "忠诚村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "宝东镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "宝东村委会",
                code: "001",
            },
            VillageCode {
                name: "宝兴村委会",
                code: "002",
            },
            VillageCode {
                name: "共乐村委会",
                code: "003",
            },
            VillageCode {
                name: "凉水泉村委会",
                code: "004",
            },
            VillageCode {
                name: "兴华村委会",
                code: "005",
            },
            VillageCode {
                name: "东兴村委会",
                code: "006",
            },
            VillageCode {
                name: "平原村委会",
                code: "007",
            },
            VillageCode {
                name: "太和村委会",
                code: "008",
            },
            VillageCode {
                name: "太兴村委会",
                code: "009",
            },
            VillageCode {
                name: "太山村委会",
                code: "010",
            },
            VillageCode {
                name: "正义村委会",
                code: "011",
            },
            VillageCode {
                name: "联义村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "新乐乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "兴隆村委会",
                code: "001",
            },
            VillageCode {
                name: "新乐村委会",
                code: "002",
            },
            VillageCode {
                name: "新民村委会",
                code: "003",
            },
            VillageCode {
                name: "富荣村委会",
                code: "004",
            },
            VillageCode {
                name: "连山村委会",
                code: "005",
            },
            VillageCode {
                name: "永平村委会",
                code: "006",
            },
            VillageCode {
                name: "双跃村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "伟光乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "太平村委会",
                code: "001",
            },
            VillageCode {
                name: "胜利村委会",
                code: "002",
            },
            VillageCode {
                name: "伟光村委会",
                code: "003",
            },
            VillageCode {
                name: "吉庆村委会",
                code: "004",
            },
            VillageCode {
                name: "吉安村委会",
                code: "005",
            },
            VillageCode {
                name: "德福村委会",
                code: "006",
            },
            VillageCode {
                name: "永胜村委会",
                code: "007",
            },
            VillageCode {
                name: "幸福村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "珍宝岛乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "永乐村委会",
                code: "001",
            },
            VillageCode {
                name: "永和村委会",
                code: "002",
            },
            VillageCode {
                name: "小木河村委会",
                code: "003",
            },
            VillageCode {
                name: "宝丰村委会",
                code: "004",
            },
            VillageCode {
                name: "独木河村委会",
                code: "005",
            },
            VillageCode {
                name: "新跃村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "阿北乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "新富村委会",
                code: "001",
            },
            VillageCode {
                name: "新路村委会",
                code: "002",
            },
            VillageCode {
                name: "新中村委会",
                code: "003",
            },
            VillageCode {
                name: "阿北村委会",
                code: "004",
            },
            VillageCode {
                name: "阿东村委会",
                code: "005",
            },
            VillageCode {
                name: "新林村委会",
                code: "006",
            },
            VillageCode {
                name: "新政村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "东方红林业局",
        code: "012",
        villages: &[
            VillageCode {
                name: "第一居民委员会社区",
                code: "001",
            },
            VillageCode {
                name: "第二居民委员会社区",
                code: "002",
            },
            VillageCode {
                name: "第三居民委员会社区",
                code: "003",
            },
            VillageCode {
                name: "第五居民委员会社区",
                code: "004",
            },
            VillageCode {
                name: "第六居民委员会社区",
                code: "005",
            },
            VillageCode {
                name: "第七居民委员会社区",
                code: "006",
            },
            VillageCode {
                name: "第八居民委员会社区",
                code: "007",
            },
            VillageCode {
                name: "第九居民委员会社区",
                code: "008",
            },
            VillageCode {
                name: "第十居民委员会社区",
                code: "009",
            },
            VillageCode {
                name: "第十一居民委员会社区",
                code: "010",
            },
            VillageCode {
                name: "第十二居民委员会社区",
                code: "011",
            },
            VillageCode {
                name: "第十三居民委员会社区",
                code: "012",
            },
            VillageCode {
                name: "第十四居民委员会社区",
                code: "013",
            },
            VillageCode {
                name: "第十五居民委员会社区",
                code: "014",
            },
            VillageCode {
                name: "第十六居民委员会社区",
                code: "015",
            },
            VillageCode {
                name: "第十七居民委员会社区",
                code: "016",
            },
            VillageCode {
                name: "第十八居民委员会社区",
                code: "017",
            },
            VillageCode {
                name: "第十九居民委员会社区",
                code: "018",
            },
            VillageCode {
                name: "第二十居民委员会社区",
                code: "019",
            },
            VillageCode {
                name: "东林经营所社区生活区",
                code: "020",
            },
            VillageCode {
                name: "西南岔经营所社区生活区",
                code: "021",
            },
            VillageCode {
                name: "青山经营所社区生活区",
                code: "022",
            },
            VillageCode {
                name: "独木河林场社区生活区",
                code: "023",
            },
            VillageCode {
                name: "马鞍山农场社区生活区",
                code: "024",
            },
            VillageCode {
                name: "大塔山林场社区生活区",
                code: "025",
            },
            VillageCode {
                name: "河口林场社区生活区",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "迎春林业局",
        code: "013",
        villages: &[
            VillageCode {
                name: "第一居民委员会社区",
                code: "001",
            },
            VillageCode {
                name: "第二居民委员会社区",
                code: "002",
            },
            VillageCode {
                name: "第三居民委员会社区",
                code: "003",
            },
            VillageCode {
                name: "第四居民委员会社区",
                code: "004",
            },
            VillageCode {
                name: "第五居民委员会社区",
                code: "005",
            },
            VillageCode {
                name: "方山林场社区生活区",
                code: "006",
            },
            VillageCode {
                name: "皖峰林场社区生活区",
                code: "007",
            },
            VillageCode {
                name: "东风林场社区生活区",
                code: "008",
            },
            VillageCode {
                name: "五泡林场社区生活区",
                code: "009",
            },
            VillageCode {
                name: "尖山林场社区生活区",
                code: "010",
            },
            VillageCode {
                name: "永丰林场社区生活区",
                code: "011",
            },
            VillageCode {
                name: "光明农场社区生活区",
                code: "012",
            },
            VillageCode {
                name: "向阳农场社区生活区",
                code: "013",
            },
            VillageCode {
                name: "曙光农场社区生活区",
                code: "014",
            },
            VillageCode {
                name: "索伦农场社区生活区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "八五〇农场",
        code: "014",
        villages: &[
            VillageCode {
                name: "八五〇场直社区",
                code: "001",
            },
            VillageCode {
                name: "八五〇农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "八五〇农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "八五〇农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "八五〇农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "八五〇农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "八五〇农场第六管理区",
                code: "007",
            },
            VillageCode {
                name: "八五〇农场第七管理区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "八五四农场",
        code: "015",
        villages: &[
            VillageCode {
                name: "八五四场直社区",
                code: "001",
            },
            VillageCode {
                name: "八五四农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "八五四农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "八五四农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "八五四农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "八五四农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "八五四农场第六管理区",
                code: "007",
            },
            VillageCode {
                name: "八五四农场第七管理区",
                code: "008",
            },
            VillageCode {
                name: "八五四农场第八管理区",
                code: "009",
            },
            VillageCode {
                name: "八五四农场第九管理区",
                code: "010",
            },
            VillageCode {
                name: "八五四农场第十管理区",
                code: "011",
            },
            VillageCode {
                name: "八五四农场第十一管理区",
                code: "012",
            },
            VillageCode {
                name: "八五四农场第十二管理区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "八五六农场",
        code: "016",
        villages: &[
            VillageCode {
                name: "八五六场直社区",
                code: "001",
            },
            VillageCode {
                name: "八五六农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "八五六农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "八五六农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "八五六农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "八五六农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "八五六农场第六管理区",
                code: "007",
            },
            VillageCode {
                name: "八五六农场第七管理区",
                code: "008",
            },
            VillageCode {
                name: "八五六农场第八管理区",
                code: "009",
            },
            VillageCode {
                name: "八五六农场第九管理区",
                code: "010",
            },
            VillageCode {
                name: "八五六农场第十管理区",
                code: "011",
            },
            VillageCode {
                name: "八五六农场第十一管理区",
                code: "012",
            },
            VillageCode {
                name: "八五六农场第十二管理区",
                code: "013",
            },
            VillageCode {
                name: "八五六农场第十三管理区",
                code: "014",
            },
            VillageCode {
                name: "八五六农场第十四管理区",
                code: "015",
            },
            VillageCode {
                name: "八五六农场第十五管理区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "八五八农场",
        code: "017",
        villages: &[
            VillageCode {
                name: "八五八场直社区",
                code: "001",
            },
            VillageCode {
                name: "八五八农场第二管理区",
                code: "002",
            },
            VillageCode {
                name: "八五八农场第一管理区",
                code: "003",
            },
            VillageCode {
                name: "八五八农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "八五八农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "八五八农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "八五八农场第六管理区",
                code: "007",
            },
            VillageCode {
                name: "八五八农场第七管理区",
                code: "008",
            },
            VillageCode {
                name: "八五八农场第八管理区",
                code: "009",
            },
            VillageCode {
                name: "八五八农场第十管理区",
                code: "010",
            },
            VillageCode {
                name: "八五八农场第九管理区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "庆丰农场",
        code: "018",
        villages: &[
            VillageCode {
                name: "庆丰场直社区",
                code: "001",
            },
            VillageCode {
                name: "庆丰农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "庆丰农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "庆丰农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "庆丰农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "庆丰农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "庆丰农场第六管理区",
                code: "007",
            },
            VillageCode {
                name: "庆丰农场第七管理区",
                code: "008",
            },
            VillageCode {
                name: "庆丰农场第八管理区",
                code: "009",
            },
            VillageCode {
                name: "庆丰农场第九管理区",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "云山农场",
        code: "019",
        villages: &[
            VillageCode {
                name: "云山场直社区",
                code: "001",
            },
            VillageCode {
                name: "云山农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "云山农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "云山农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "云山农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "云山农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "云山农场第六管理区",
                code: "007",
            },
        ],
    },
];

static TOWNS_HJ_010: [TownCode; 8] = [
    TownCode {
        name: "二马路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "春城社区",
                code: "001",
            },
            VillageCode {
                name: "吉星社区",
                code: "002",
            },
            VillageCode {
                name: "春光社区",
                code: "003",
            },
            VillageCode {
                name: "春晖社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "八马路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "中植社区",
                code: "001",
            },
            VillageCode {
                name: "福园社区",
                code: "002",
            },
            VillageCode {
                name: "建鑫社区",
                code: "003",
            },
            VillageCode {
                name: "向阳社区",
                code: "004",
            },
            VillageCode {
                name: "益人社区",
                code: "005",
            },
            VillageCode {
                name: "朝阳社区",
                code: "006",
            },
            VillageCode {
                name: "光明社区",
                code: "007",
            },
            VillageCode {
                name: "合兴社区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "中心站街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "商贸社区",
                code: "001",
            },
            VillageCode {
                name: "北秀社区",
                code: "002",
            },
            VillageCode {
                name: "鞍山社区",
                code: "003",
            },
            VillageCode {
                name: "兴化社区",
                code: "004",
            },
            VillageCode {
                name: "隆安社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "富安街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "南山社区",
                code: "001",
            },
            VillageCode {
                name: "富东社区",
                code: "002",
            },
            VillageCode {
                name: "窑地社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "长安街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "长安社区",
                code: "001",
            },
            VillageCode {
                name: "方园社区",
                code: "002",
            },
            VillageCode {
                name: "安平社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "铁西街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "河西社区",
                code: "001",
            },
            VillageCode {
                name: "鸿苑社区",
                code: "002",
            },
            VillageCode {
                name: "社保社区",
                code: "003",
            },
            VillageCode {
                name: "铁路社区",
                code: "004",
            },
            VillageCode {
                name: "豫园社区",
                code: "005",
            },
            VillageCode {
                name: "盛苑社区",
                code: "006",
            },
            VillageCode {
                name: "长虹社区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "学府街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "福悦湾社区",
                code: "001",
            },
            VillageCode {
                name: "民生社区",
                code: "002",
            },
            VillageCode {
                name: "银苑社区",
                code: "003",
            },
            VillageCode {
                name: "时代新城社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "安邦乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "双胜村委会",
                code: "001",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "002",
            },
            VillageCode {
                name: "双兴村委会",
                code: "003",
            },
            VillageCode {
                name: "西山村委会",
                code: "004",
            },
            VillageCode {
                name: "双合村委会",
                code: "005",
            },
            VillageCode {
                name: "公立村委会",
                code: "006",
            },
            VillageCode {
                name: "原鲜村委会",
                code: "007",
            },
            VillageCode {
                name: "集东村委会",
                code: "008",
            },
            VillageCode {
                name: "建胜村委会",
                code: "009",
            },
            VillageCode {
                name: "窑地村委会",
                code: "010",
            },
            VillageCode {
                name: "富安村委会",
                code: "011",
            },
            VillageCode {
                name: "长安村委会",
                code: "012",
            },
            VillageCode {
                name: "双富村委会",
                code: "013",
            },
        ],
    },
];

static TOWNS_HJ_011: [TownCode; 7] = [
    TownCode {
        name: "中山街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "安邦社区",
                code: "001",
            },
            VillageCode {
                name: "通达社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "北山街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "东明社区",
                code: "001",
            },
            VillageCode {
                name: "东升社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "南山街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "青山社区",
                code: "001",
            },
            VillageCode {
                name: "东湖社区",
                code: "002",
            },
            VillageCode {
                name: "蓝天社区",
                code: "003",
            },
            VillageCode {
                name: "二站社区",
                code: "004",
            },
            VillageCode {
                name: "安邦河社区",
                code: "005",
            },
            VillageCode {
                name: "青山林场社区",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "东山街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "惠民社区",
                code: "001",
            },
            VillageCode {
                name: "惠安社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "中心街道",
        code: "005",
        villages: &[VillageCode {
            name: "学府社区",
            code: "001",
        }],
    },
    TownCode {
        name: "西山街道",
        code: "006",
        villages: &[VillageCode {
            name: "兴华社区",
            code: "001",
        }],
    },
    TownCode {
        name: "长胜乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "东升村村委会",
                code: "001",
            },
            VillageCode {
                name: "东风村村委会",
                code: "002",
            },
            VillageCode {
                name: "东兴村村委会",
                code: "003",
            },
            VillageCode {
                name: "立新村村委会",
                code: "004",
            },
            VillageCode {
                name: "宏强村村委会",
                code: "005",
            },
            VillageCode {
                name: "团山村村委会",
                code: "006",
            },
            VillageCode {
                name: "新翼村村委会",
                code: "007",
            },
        ],
    },
];

static TOWNS_HJ_012: [TownCode; 5] = [
    TownCode {
        name: "振兴中路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "中心社区",
                code: "001",
            },
            VillageCode {
                name: "广源社区",
                code: "002",
            },
            VillageCode {
                name: "为民社区",
                code: "003",
            },
            VillageCode {
                name: "四井社区",
                code: "004",
            },
            VillageCode {
                name: "秀山社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "振兴东路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "富饶社区",
                code: "001",
            },
            VillageCode {
                name: "满意社区",
                code: "002",
            },
            VillageCode {
                name: "锦程社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "集贤街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "翠园社区",
                code: "001",
            },
            VillageCode {
                name: "北苑社区",
                code: "002",
            },
            VillageCode {
                name: "创业社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "东荣街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "二矿社区",
                code: "001",
            },
            VillageCode {
                name: "三矿社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "太保镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "七一村委会",
                code: "001",
            },
            VillageCode {
                name: "中华村委会",
                code: "002",
            },
            VillageCode {
                name: "开原村委会",
                code: "003",
            },
            VillageCode {
                name: "山河村委会",
                code: "004",
            },
            VillageCode {
                name: "靠山村委会",
                code: "005",
            },
            VillageCode {
                name: "红星村委会",
                code: "006",
            },
            VillageCode {
                name: "长富村委会",
                code: "007",
            },
            VillageCode {
                name: "东胜村委会",
                code: "008",
            },
            VillageCode {
                name: "四合村委会",
                code: "009",
            },
            VillageCode {
                name: "双丰村委会",
                code: "010",
            },
            VillageCode {
                name: "建兴村委会",
                code: "011",
            },
            VillageCode {
                name: "四新村委会",
                code: "012",
            },
            VillageCode {
                name: "东岗村委会",
                code: "013",
            },
            VillageCode {
                name: "永久村委会",
                code: "014",
            },
            VillageCode {
                name: "九三村委会",
                code: "015",
            },
            VillageCode {
                name: "五四村委会",
                code: "016",
            },
        ],
    },
];

static TOWNS_HJ_013: [TownCode; 9] = [
    TownCode {
        name: "红旗街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "红永社区",
                code: "001",
            },
            VillageCode {
                name: "红远社区",
                code: "002",
            },
            VillageCode {
                name: "红升社区",
                code: "003",
            },
            VillageCode {
                name: "红兴社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "跃进街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "宏吉社区",
                code: "001",
            },
            VillageCode {
                name: "宏祥社区",
                code: "002",
            },
            VillageCode {
                name: "宏如社区",
                code: "003",
            },
            VillageCode {
                name: "宏意社区",
                code: "004",
            },
            VillageCode {
                name: "宏顺社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "东保卫街道",
        code: "003",
        villages: &[VillageCode {
            name: "东兴社区",
            code: "001",
        }],
    },
    TownCode {
        name: "七星街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "星河社区",
                code: "001",
            },
            VillageCode {
                name: "星盛社区",
                code: "002",
            },
            VillageCode {
                name: "星隆社区",
                code: "003",
            },
            VillageCode {
                name: "星福社区",
                code: "004",
            },
            VillageCode {
                name: "星吉社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "双阳街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "向阳社区",
                code: "001",
            },
            VillageCode {
                name: "新兴社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "新安街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "东平社区",
                code: "001",
            },
            VillageCode {
                name: "富民社区",
                code: "002",
            },
            VillageCode {
                name: "西平社区",
                code: "003",
            },
            VillageCode {
                name: "仁合社区",
                code: "004",
            },
            VillageCode {
                name: "杨家围社区",
                code: "005",
            },
            VillageCode {
                name: "八营社区",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "电厂街道",
        code: "007",
        villages: &[VillageCode {
            name: "电厂社区",
            code: "001",
        }],
    },
    TownCode {
        name: "农场街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "宝农社区",
                code: "001",
            },
            VillageCode {
                name: "宝民社区",
                code: "002",
            },
            VillageCode {
                name: "宝丰社区",
                code: "003",
            },
            VillageCode {
                name: "宝福社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "七星镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "杨木岗村生活区",
                code: "001",
            },
            VillageCode {
                name: "华新村生活区",
                code: "002",
            },
            VillageCode {
                name: "宝山村生活区",
                code: "003",
            },
            VillageCode {
                name: "新村生活区",
                code: "004",
            },
            VillageCode {
                name: "上游村生活区",
                code: "005",
            },
            VillageCode {
                name: "前进村生活区",
                code: "006",
            },
        ],
    },
];

static TOWNS_HJ_014: [TownCode; 20] = [
    TownCode {
        name: "福利镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "繁荣社区居委会",
                code: "001",
            },
            VillageCode {
                name: "曙光社区居委会",
                code: "002",
            },
            VillageCode {
                name: "亿安社区居委会",
                code: "003",
            },
            VillageCode {
                name: "振业社区居委会",
                code: "004",
            },
            VillageCode {
                name: "前进社区居委会",
                code: "005",
            },
            VillageCode {
                name: "花园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "农丰社区居委会",
                code: "007",
            },
            VillageCode {
                name: "站前社区居委会",
                code: "008",
            },
            VillageCode {
                name: "翠园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "东荣村委会",
                code: "010",
            },
            VillageCode {
                name: "东兴村委会",
                code: "011",
            },
            VillageCode {
                name: "青山村委会",
                code: "012",
            },
            VillageCode {
                name: "双丰村委会",
                code: "013",
            },
            VillageCode {
                name: "清泉村委会",
                code: "014",
            },
            VillageCode {
                name: "先锋村委会",
                code: "015",
            },
            VillageCode {
                name: "长征村委会",
                code: "016",
            },
            VillageCode {
                name: "胜利村委会",
                code: "017",
            },
            VillageCode {
                name: "东发村委会",
                code: "018",
            },
            VillageCode {
                name: "金星村委会",
                code: "019",
            },
            VillageCode {
                name: "新发村委会",
                code: "020",
            },
            VillageCode {
                name: "红联村委会",
                code: "021",
            },
            VillageCode {
                name: "安邦村委会",
                code: "022",
            },
            VillageCode {
                name: "东辉村委会",
                code: "023",
            },
            VillageCode {
                name: "高丰村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "集贤镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "福城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "兴达社区居委会",
                code: "002",
            },
            VillageCode {
                name: "永发村委会",
                code: "003",
            },
            VillageCode {
                name: "双胜村委会",
                code: "004",
            },
            VillageCode {
                name: "中兴村委会",
                code: "005",
            },
            VillageCode {
                name: "七一村委会",
                code: "006",
            },
            VillageCode {
                name: "顺发村委会",
                code: "007",
            },
            VillageCode {
                name: "山河村委会",
                code: "008",
            },
            VillageCode {
                name: "德胜村委会",
                code: "009",
            },
            VillageCode {
                name: "山东村委会",
                code: "010",
            },
            VillageCode {
                name: "国庆村委会",
                code: "011",
            },
            VillageCode {
                name: "红光村委会",
                code: "012",
            },
            VillageCode {
                name: "丰收村委会",
                code: "013",
            },
            VillageCode {
                name: "兆林村委会",
                code: "014",
            },
            VillageCode {
                name: "同意村委会",
                code: "015",
            },
            VillageCode {
                name: "长安村委会",
                code: "016",
            },
            VillageCode {
                name: "永富村委会",
                code: "017",
            },
            VillageCode {
                name: "平原村委会",
                code: "018",
            },
            VillageCode {
                name: "福厚村委会",
                code: "019",
            },
            VillageCode {
                name: "洪仁村委会",
                code: "020",
            },
            VillageCode {
                name: "务正村委会",
                code: "021",
            },
            VillageCode {
                name: "城新村委会",
                code: "022",
            },
            VillageCode {
                name: "五一村委会",
                code: "023",
            },
            VillageCode {
                name: "黎明村委会",
                code: "024",
            },
            VillageCode {
                name: "德祥村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "升昌镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "永昌社区居委会",
                code: "001",
            },
            VillageCode {
                name: "永胜村委会",
                code: "002",
            },
            VillageCode {
                name: "五星村委会",
                code: "003",
            },
            VillageCode {
                name: "保丰村委会",
                code: "004",
            },
            VillageCode {
                name: "华山村委会",
                code: "005",
            },
            VillageCode {
                name: "德兴村委会",
                code: "006",
            },
            VillageCode {
                name: "治安村委会",
                code: "007",
            },
            VillageCode {
                name: "三方村委会",
                code: "008",
            },
            VillageCode {
                name: "大兴村委会",
                code: "009",
            },
            VillageCode {
                name: "友好村委会",
                code: "010",
            },
            VillageCode {
                name: "爱林村委会",
                code: "011",
            },
            VillageCode {
                name: "丰林村委会",
                code: "012",
            },
            VillageCode {
                name: "东方红村委会",
                code: "013",
            },
            VillageCode {
                name: "太升村委会",
                code: "014",
            },
            VillageCode {
                name: "太昌村委会",
                code: "015",
            },
            VillageCode {
                name: "永华村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "丰乐镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "长发社区居委会",
                code: "001",
            },
            VillageCode {
                name: "卫东村委会",
                code: "002",
            },
            VillageCode {
                name: "东升村委会",
                code: "003",
            },
            VillageCode {
                name: "东风村委会",
                code: "004",
            },
            VillageCode {
                name: "奋斗村委会",
                code: "005",
            },
            VillageCode {
                name: "兴发村委会",
                code: "006",
            },
            VillageCode {
                name: "太复村委会",
                code: "007",
            },
            VillageCode {
                name: "永强村委会",
                code: "008",
            },
            VillageCode {
                name: "太联村委会",
                code: "009",
            },
            VillageCode {
                name: "太乐村委会",
                code: "010",
            },
            VillageCode {
                name: "永丰村委会",
                code: "011",
            },
            VillageCode {
                name: "太城村委会",
                code: "012",
            },
            VillageCode {
                name: "太丰村委会",
                code: "013",
            },
            VillageCode {
                name: "庆丰村委会",
                code: "014",
            },
            VillageCode {
                name: "太华村委会",
                code: "015",
            },
            VillageCode {
                name: "新立村委会",
                code: "016",
            },
            VillageCode {
                name: "太源村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "太平镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "永发社区居委会",
                code: "001",
            },
            VillageCode {
                name: "太平村委会",
                code: "002",
            },
            VillageCode {
                name: "太兴村委会",
                code: "003",
            },
            VillageCode {
                name: "太增村委会",
                code: "004",
            },
            VillageCode {
                name: "太山村委会",
                code: "005",
            },
            VillageCode {
                name: "太安村委会",
                code: "006",
            },
            VillageCode {
                name: "太辉村委会",
                code: "007",
            },
            VillageCode {
                name: "太合村委会",
                code: "008",
            },
            VillageCode {
                name: "太忠村委会",
                code: "009",
            },
            VillageCode {
                name: "太恒村委会",
                code: "010",
            },
            VillageCode {
                name: "太洪村委会",
                code: "011",
            },
            VillageCode {
                name: "太玉村委会",
                code: "012",
            },
            VillageCode {
                name: "太林村委会",
                code: "013",
            },
            VillageCode {
                name: "太岩村委会",
                code: "014",
            },
            VillageCode {
                name: "太阳村委会",
                code: "015",
            },
            VillageCode {
                name: "太利村委会",
                code: "016",
            },
            VillageCode {
                name: "太荣村委会",
                code: "017",
            },
            VillageCode {
                name: "太发村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "腰屯乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "兴民社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "昌平村委会",
                code: "002",
            },
            VillageCode {
                name: "八一村委会",
                code: "003",
            },
            VillageCode {
                name: "天兴村委会",
                code: "004",
            },
            VillageCode {
                name: "兴久村委会",
                code: "005",
            },
            VillageCode {
                name: "宏图村委会",
                code: "006",
            },
            VillageCode {
                name: "腰屯村委会",
                code: "007",
            },
            VillageCode {
                name: "明星村委会",
                code: "008",
            },
            VillageCode {
                name: "永红村委会",
                code: "009",
            },
            VillageCode {
                name: "繁荣村委会",
                code: "010",
            },
            VillageCode {
                name: "联丰村委会",
                code: "011",
            },
            VillageCode {
                name: "联合村委会",
                code: "012",
            },
            VillageCode {
                name: "常胜村委会",
                code: "013",
            },
            VillageCode {
                name: "民胜村委会",
                code: "014",
            },
            VillageCode {
                name: "双山村委会",
                code: "015",
            },
            VillageCode {
                name: "福兴村委会",
                code: "016",
            },
            VillageCode {
                name: "万生村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "兴安乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "兴安社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "笔架山村委会",
                code: "002",
            },
            VillageCode {
                name: "精神村委会",
                code: "003",
            },
            VillageCode {
                name: "柳河村委会",
                code: "004",
            },
            VillageCode {
                name: "庆生村委会",
                code: "005",
            },
            VillageCode {
                name: "和平村委会",
                code: "006",
            },
            VillageCode {
                name: "永乐村委会",
                code: "007",
            },
            VillageCode {
                name: "仁德村委会",
                code: "008",
            },
            VillageCode {
                name: "忠厚村委会",
                code: "009",
            },
            VillageCode {
                name: "合发村委会",
                code: "010",
            },
            VillageCode {
                name: "宏德村委会",
                code: "011",
            },
            VillageCode {
                name: "兴业村委会",
                code: "012",
            },
            VillageCode {
                name: "兴一村委会",
                code: "013",
            },
            VillageCode {
                name: "兴二村委会",
                code: "014",
            },
            VillageCode {
                name: "兴三村委会",
                code: "015",
            },
            VillageCode {
                name: "兴四村委会",
                code: "016",
            },
            VillageCode {
                name: "光明村委会",
                code: "017",
            },
            VillageCode {
                name: "保胜村委会",
                code: "018",
            },
            VillageCode {
                name: "鲜兴村委会",
                code: "019",
            },
            VillageCode {
                name: "兴旺村委会",
                code: "020",
            },
            VillageCode {
                name: "双保村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "永安乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "永安社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "德利村委会",
                code: "002",
            },
            VillageCode {
                name: "宏伟村委会",
                code: "003",
            },
            VillageCode {
                name: "兴富村委会",
                code: "004",
            },
            VillageCode {
                name: "兴源村委会",
                code: "005",
            },
            VillageCode {
                name: "永林村委会",
                code: "006",
            },
            VillageCode {
                name: "永兴村委会",
                code: "007",
            },
            VillageCode {
                name: "永吉村委会",
                code: "008",
            },
            VillageCode {
                name: "向阳村委会",
                code: "009",
            },
            VillageCode {
                name: "五七村委会",
                code: "010",
            },
            VillageCode {
                name: "青春村委会",
                code: "011",
            },
            VillageCode {
                name: "长发村委会",
                code: "012",
            },
            VillageCode {
                name: "兴华村委会",
                code: "013",
            },
            VillageCode {
                name: "永革村委会",
                code: "014",
            },
            VillageCode {
                name: "永合村委会",
                code: "015",
            },
            VillageCode {
                name: "永升村委会",
                code: "016",
            },
            VillageCode {
                name: "幸福村委会",
                code: "017",
            },
            VillageCode {
                name: "勤俭村委会",
                code: "018",
            },
            VillageCode {
                name: "永明村委会",
                code: "019",
            },
            VillageCode {
                name: "双跃村委会",
                code: "020",
            },
            VillageCode {
                name: "曙光村委会",
                code: "021",
            },
            VillageCode {
                name: "北星村委会",
                code: "022",
            },
            VillageCode {
                name: "富强村委会",
                code: "023",
            },
            VillageCode {
                name: "向荣村委会",
                code: "024",
            },
            VillageCode {
                name: "富民村委会",
                code: "025",
            },
            VillageCode {
                name: "联明村委会",
                code: "026",
            },
            VillageCode {
                name: "北安村委会",
                code: "027",
            },
            VillageCode {
                name: "洪胜村委会",
                code: "028",
            },
            VillageCode {
                name: "新合村委会",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "太平林场",
        code: "009",
        villages: &[VillageCode {
            name: "太平林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "丰乐林场",
        code: "010",
        villages: &[VillageCode {
            name: "丰乐林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "七星林场",
        code: "011",
        villages: &[VillageCode {
            name: "七星林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "峻山林场",
        code: "012",
        villages: &[VillageCode {
            name: "峻山林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "爱林林场",
        code: "013",
        villages: &[VillageCode {
            name: "爱林林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "腰屯林场",
        code: "014",
        villages: &[VillageCode {
            name: "腰屯林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "升平煤矿",
        code: "015",
        villages: &[VillageCode {
            name: "升平煤矿虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "黑龙江省双鸭山监狱",
        code: "016",
        villages: &[
            VillageCode {
                name: "黑龙江省双鸭山市监狱社区",
                code: "001",
            },
            VillageCode {
                name: "一大队生活区",
                code: "002",
            },
            VillageCode {
                name: "二大队生活区",
                code: "003",
            },
            VillageCode {
                name: "三大队生活区",
                code: "004",
            },
            VillageCode {
                name: "四大队生活区",
                code: "005",
            },
            VillageCode {
                name: "六大队生活区",
                code: "006",
            },
            VillageCode {
                name: "七大队生活区",
                code: "007",
            },
            VillageCode {
                name: "九大队生活区",
                code: "008",
            },
            VillageCode {
                name: "十一大队生活区",
                code: "009",
            },
            VillageCode {
                name: "十二大队生活区",
                code: "010",
            },
            VillageCode {
                name: "五大队生活区",
                code: "011",
            },
            VillageCode {
                name: "第十三大队生活区",
                code: "012",
            },
            VillageCode {
                name: "十四大队生活区",
                code: "013",
            },
            VillageCode {
                name: "科研站生活区",
                code: "014",
            },
            VillageCode {
                name: "水管站生活区",
                code: "015",
            },
            VillageCode {
                name: "八大队生活区",
                code: "016",
            },
            VillageCode {
                name: "十大队生活区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "二九一农场",
        code: "017",
        villages: &[
            VillageCode {
                name: "二九一农场场直社区第一居民委社区",
                code: "001",
            },
            VillageCode {
                name: "二九一农场场直社区第二居民委社区",
                code: "002",
            },
            VillageCode {
                name: "二九一农场场直社区第三居民委社区",
                code: "003",
            },
            VillageCode {
                name: "二九一农场场直社区第四居民委社区",
                code: "004",
            },
            VillageCode {
                name: "二九一农场场直社区第五居民委社区",
                code: "005",
            },
            VillageCode {
                name: "二九一农场第一管理区居民委",
                code: "006",
            },
            VillageCode {
                name: "二九一农场第二管理区居民委",
                code: "007",
            },
            VillageCode {
                name: "二九一农场第三管理区居民委",
                code: "008",
            },
            VillageCode {
                name: "二九一农场第四管理区居民委",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "良种场",
        code: "018",
        villages: &[VillageCode {
            name: "良种场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "种畜场",
        code: "019",
        villages: &[VillageCode {
            name: "种畜场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "果树示范场",
        code: "020",
        villages: &[VillageCode {
            name: "果树示范场虚拟生活区",
            code: "001",
        }],
    },
];

static TOWNS_HJ_015: [TownCode; 13] = [
    TownCode {
        name: "友谊镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "繁荣社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "百兴社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "康乐社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "富强社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "学府社区居民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "兴隆镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "爱林村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "利华村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "邹集村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "和发村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "猴石村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "青年庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "平安村村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "龙山镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "龙山村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "北峰村村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "凤岗镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "凤岗村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "集富村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "六合村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "友利村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "兴隆山村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "幸福村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "兴盛乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "兴盛村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东胜村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "宏伟村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "宏坤村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "农民村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "丰源村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "东建乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "东建村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "发家村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "永林村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "富强村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "靠乡村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "年丰村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "二站村村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "庆丰乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "庆丰村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "康家店村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新欣村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "胜利村村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "建设乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "建设村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "北新发村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "兴华村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "中心村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "富民村村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "友邻乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "友邻村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东明村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "东兴村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "新镇乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "新镇村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "双林村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西邻村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "成富朝鲜族满族乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "套河村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "大成富村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "对面城村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "红兴隆管理局局直",
        code: "012",
        villages: &[VillageCode {
            name: "红兴隆管理局局直虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "友谊农场",
        code: "013",
        villages: &[
            VillageCode {
                name: "友谊场直社区",
                code: "001",
            },
            VillageCode {
                name: "友谊农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "友谊农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "友谊农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "友谊农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "友谊农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "友谊农场第六管理区",
                code: "007",
            },
            VillageCode {
                name: "友谊农场第七管理区",
                code: "008",
            },
            VillageCode {
                name: "友谊农场第八管理区",
                code: "009",
            },
            VillageCode {
                name: "友谊农场第九管理区",
                code: "010",
            },
            VillageCode {
                name: "友谊农场第十管理区",
                code: "011",
            },
            VillageCode {
                name: "友谊农场第十一管理区",
                code: "012",
            },
            VillageCode {
                name: "友谊农场林业管理区",
                code: "013",
            },
        ],
    },
];

static TOWNS_HJ_016: [TownCode; 23] = [
    TownCode {
        name: "宝清镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "东关社区",
                code: "001",
            },
            VillageCode {
                name: "建设社区",
                code: "002",
            },
            VillageCode {
                name: "真理社区",
                code: "003",
            },
            VillageCode {
                name: "幸福社区",
                code: "004",
            },
            VillageCode {
                name: "双胜社区",
                code: "005",
            },
            VillageCode {
                name: "解放社区",
                code: "006",
            },
            VillageCode {
                name: "和平社区",
                code: "007",
            },
            VillageCode {
                name: "亨利社区",
                code: "008",
            },
            VillageCode {
                name: "岚峰社区",
                code: "009",
            },
            VillageCode {
                name: "清河社区",
                code: "010",
            },
            VillageCode {
                name: "建设村委会",
                code: "011",
            },
            VillageCode {
                name: "和平村委会",
                code: "012",
            },
            VillageCode {
                name: "真理村委会",
                code: "013",
            },
            VillageCode {
                name: "双胜村委会",
                code: "014",
            },
            VillageCode {
                name: "东关村委会",
                code: "015",
            },
            VillageCode {
                name: "南元村委会",
                code: "016",
            },
            VillageCode {
                name: "亨利村委会",
                code: "017",
            },
            VillageCode {
                name: "解放村委会",
                code: "018",
            },
            VillageCode {
                name: "十八里村委会",
                code: "019",
            },
            VillageCode {
                name: "十二里村委会",
                code: "020",
            },
            VillageCode {
                name: "连丰村委会",
                code: "021",
            },
            VillageCode {
                name: "庆兰村委会",
                code: "022",
            },
            VillageCode {
                name: "红新村委会",
                code: "023",
            },
            VillageCode {
                name: "庄园村委会",
                code: "024",
            },
            VillageCode {
                name: "郝家村委会",
                code: "025",
            },
            VillageCode {
                name: "报国村委会",
                code: "026",
            },
            VillageCode {
                name: "双泉村委会",
                code: "027",
            },
            VillageCode {
                name: "高家村委会",
                code: "028",
            },
            VillageCode {
                name: "四新村委会",
                code: "029",
            },
            VillageCode {
                name: "靠山村委会",
                code: "030",
            },
            VillageCode {
                name: "北关村委会",
                code: "031",
            },
            VillageCode {
                name: "永宁村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "七星泡镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "第一居委会",
                code: "001",
            },
            VillageCode {
                name: "向华村委会",
                code: "002",
            },
            VillageCode {
                name: "中红村委会",
                code: "003",
            },
            VillageCode {
                name: "红峰村委会",
                code: "004",
            },
            VillageCode {
                name: "平安村委会",
                code: "005",
            },
            VillageCode {
                name: "解放村委会",
                code: "006",
            },
            VillageCode {
                name: "兴华村委会",
                code: "007",
            },
            VillageCode {
                name: "永发村委会",
                code: "008",
            },
            VillageCode {
                name: "永胜村委会",
                code: "009",
            },
            VillageCode {
                name: "永兴村委会",
                code: "010",
            },
            VillageCode {
                name: "德兴村委会",
                code: "011",
            },
            VillageCode {
                name: "新发村委会",
                code: "012",
            },
            VillageCode {
                name: "金沙岗村委会",
                code: "013",
            },
            VillageCode {
                name: "兰凤村委会",
                code: "014",
            },
            VillageCode {
                name: "新民村委会",
                code: "015",
            },
            VillageCode {
                name: "民主村委会",
                code: "016",
            },
            VillageCode {
                name: "义合村委会",
                code: "017",
            },
            VillageCode {
                name: "金沙河村委会",
                code: "018",
            },
            VillageCode {
                name: "永安村委会",
                code: "019",
            },
            VillageCode {
                name: "永泉村委会",
                code: "020",
            },
            VillageCode {
                name: "三合村委会",
                code: "021",
            },
            VillageCode {
                name: "福兴村委会",
                code: "022",
            },
            VillageCode {
                name: "双北村委会",
                code: "023",
            },
            VillageCode {
                name: "巨宝村委会",
                code: "024",
            },
            VillageCode {
                name: "凉水村委会",
                code: "025",
            },
            VillageCode {
                name: "胜利村委会",
                code: "026",
            },
            VillageCode {
                name: "新丰村委会",
                code: "027",
            },
            VillageCode {
                name: "东太村委会",
                code: "028",
            },
            VillageCode {
                name: "西太村委会",
                code: "029",
            },
            VillageCode {
                name: "胜利林场生活区",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "青原镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "第一居委会",
                code: "001",
            },
            VillageCode {
                name: "兴东村委会",
                code: "002",
            },
            VillageCode {
                name: "永乐村委会",
                code: "003",
            },
            VillageCode {
                name: "永红村委会",
                code: "004",
            },
            VillageCode {
                name: "东富村委会",
                code: "005",
            },
            VillageCode {
                name: "兴旺村委会",
                code: "006",
            },
            VillageCode {
                name: "青山村委会",
                code: "007",
            },
            VillageCode {
                name: "庆东村委会",
                code: "008",
            },
            VillageCode {
                name: "复兴村委会",
                code: "009",
            },
            VillageCode {
                name: "兴业村委会",
                code: "010",
            },
            VillageCode {
                name: "东发村委会",
                code: "011",
            },
            VillageCode {
                name: "新城村委会",
                code: "012",
            },
            VillageCode {
                name: "本德村委会",
                code: "013",
            },
            VillageCode {
                name: "本北村委会",
                code: "014",
            },
            VillageCode {
                name: "卫东村委会",
                code: "015",
            },
            VillageCode {
                name: "原种场生活区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "夹信子镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "第一居委会",
                code: "001",
            },
            VillageCode {
                name: "夹信子村委会",
                code: "002",
            },
            VillageCode {
                name: "徐马村委会",
                code: "003",
            },
            VillageCode {
                name: "勇进村委会",
                code: "004",
            },
            VillageCode {
                name: "合作村委会",
                code: "005",
            },
            VillageCode {
                name: "团结村委会",
                code: "006",
            },
            VillageCode {
                name: "七一村委会",
                code: "007",
            },
            VillageCode {
                name: "西沟村委会",
                code: "008",
            },
            VillageCode {
                name: "头道村委会",
                code: "009",
            },
            VillageCode {
                name: "二道村委会",
                code: "010",
            },
            VillageCode {
                name: "三道村委会",
                code: "011",
            },
            VillageCode {
                name: "河泉村委会",
                code: "012",
            },
            VillageCode {
                name: "宏泉村委会",
                code: "013",
            },
            VillageCode {
                name: "林泉村委会",
                code: "014",
            },
            VillageCode {
                name: "向山村委会",
                code: "015",
            },
            VillageCode {
                name: "奋斗村委会",
                code: "016",
            },
            VillageCode {
                name: "光辉村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "龙头镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "龙头村委会",
                code: "001",
            },
            VillageCode {
                name: "红山村委会",
                code: "002",
            },
            VillageCode {
                name: "东龙村委会",
                code: "003",
            },
            VillageCode {
                name: "兰花村委会",
                code: "004",
            },
            VillageCode {
                name: "北龙村委会",
                code: "005",
            },
            VillageCode {
                name: "龙泉村委会",
                code: "006",
            },
            VillageCode {
                name: "农林村委会",
                code: "007",
            },
            VillageCode {
                name: "庆九村委会",
                code: "008",
            },
            VillageCode {
                name: "大泉沟村委会",
                code: "009",
            },
            VillageCode {
                name: "柳毛河村委会",
                code: "010",
            },
            VillageCode {
                name: "龙头桥水库社区",
                code: "011",
            },
            VillageCode {
                name: "龙头林场生活区",
                code: "012",
            },
            VillageCode {
                name: "头道岗林场生活区",
                code: "013",
            },
            VillageCode {
                name: "宝密桥场生活区",
                code: "014",
            },
            VillageCode {
                name: "宝山林场生活区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "小城子镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "小城子村委会",
                code: "001",
            },
            VillageCode {
                name: "青龙山村委会",
                code: "002",
            },
            VillageCode {
                name: "太平村委会",
                code: "003",
            },
            VillageCode {
                name: "梨南村委会",
                code: "004",
            },
            VillageCode {
                name: "梨中村委会",
                code: "005",
            },
            VillageCode {
                name: "梨北村委会",
                code: "006",
            },
            VillageCode {
                name: "富山村委会",
                code: "007",
            },
            VillageCode {
                name: "千山村委会",
                code: "008",
            },
            VillageCode {
                name: "天山村委会",
                code: "009",
            },
            VillageCode {
                name: "梨树林场生活区",
                code: "010",
            },
            VillageCode {
                name: "六道林场生活区",
                code: "011",
            },
            VillageCode {
                name: "种畜场生活区",
                code: "012",
            },
            VillageCode {
                name: "果树场生活区",
                code: "013",
            },
            VillageCode {
                name: "果酒基地生活区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "朝阳镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "朝阳村委会",
                code: "001",
            },
            VillageCode {
                name: "红旗村委会",
                code: "002",
            },
            VillageCode {
                name: "红升村委会",
                code: "003",
            },
            VillageCode {
                name: "红日村委会",
                code: "004",
            },
            VillageCode {
                name: "曙光村委会",
                code: "005",
            },
            VillageCode {
                name: "灯塔村委会",
                code: "006",
            },
            VillageCode {
                name: "丰收村委会",
                code: "007",
            },
            VillageCode {
                name: "合兴村委会",
                code: "008",
            },
            VillageCode {
                name: "东兴村委会",
                code: "009",
            },
            VillageCode {
                name: "东胜村委会",
                code: "010",
            },
            VillageCode {
                name: "东旺村委会",
                code: "011",
            },
            VillageCode {
                name: "东方红林场生活区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "万金山乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "万隆村委会",
                code: "001",
            },
            VillageCode {
                name: "红光村委会",
                code: "002",
            },
            VillageCode {
                name: "兴国村委会",
                code: "003",
            },
            VillageCode {
                name: "方胜村委会",
                code: "004",
            },
            VillageCode {
                name: "农业场村委会",
                code: "005",
            },
            VillageCode {
                name: "三星村委会",
                code: "006",
            },
            VillageCode {
                name: "万中村委会",
                code: "007",
            },
            VillageCode {
                name: "新星村委会",
                code: "008",
            },
            VillageCode {
                name: "金山村委会",
                code: "009",
            },
            VillageCode {
                name: "宝金村委会",
                code: "010",
            },
            VillageCode {
                name: "志强村委会",
                code: "011",
            },
            VillageCode {
                name: "良种场生活区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "尖山子乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "尖东村委会",
                code: "001",
            },
            VillageCode {
                name: "东方村委会",
                code: "002",
            },
            VillageCode {
                name: "东青村委会",
                code: "003",
            },
            VillageCode {
                name: "东红村委会",
                code: "004",
            },
            VillageCode {
                name: "东风村委会",
                code: "005",
            },
            VillageCode {
                name: "索东村委会",
                code: "006",
            },
            VillageCode {
                name: "东明村委会",
                code: "007",
            },
            VillageCode {
                name: "东鑫村委会",
                code: "008",
            },
            VillageCode {
                name: "三道林子村委会",
                code: "009",
            },
            VillageCode {
                name: "头道林子村委会",
                code: "010",
            },
            VillageCode {
                name: "二道林子村委会",
                code: "011",
            },
            VillageCode {
                name: "中岗村委会",
                code: "012",
            },
            VillageCode {
                name: "北岗村委会",
                code: "013",
            },
            VillageCode {
                name: "银龙村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "七星河乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "七星河村委会",
                code: "001",
            },
            VillageCode {
                name: "常张村委会",
                code: "002",
            },
            VillageCode {
                name: "杨树村委会",
                code: "003",
            },
            VillageCode {
                name: "新立村委会",
                code: "004",
            },
            VillageCode {
                name: "东辉村委会",
                code: "005",
            },
            VillageCode {
                name: "东强村委会",
                code: "006",
            },
            VillageCode {
                name: "建平村委会",
                code: "007",
            },
            VillageCode {
                name: "永新村委会",
                code: "008",
            },
            VillageCode {
                name: "兴平村委会",
                code: "009",
            },
            VillageCode {
                name: "北宝村委会",
                code: "010",
            },
            VillageCode {
                name: "芦苇公司生活区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "双鸭山林业局上游经营所",
        code: "011",
        villages: &[VillageCode {
            name: "上游经营所虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "双鸭山林业局南瓮泉经营所",
        code: "012",
        villages: &[VillageCode {
            name: "南瓮泉经营所虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "双鸭山林业局七一林场",
        code: "013",
        villages: &[VillageCode {
            name: "七一林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "双鸭山林业局七星河林场",
        code: "014",
        villages: &[VillageCode {
            name: "七星河林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "双鸭山林业局红旗林场",
        code: "015",
        villages: &[VillageCode {
            name: "红旗林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "双鸭山林业局三岔河林场",
        code: "016",
        villages: &[VillageCode {
            name: "三岔河林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "双鸭山林业局青龙林场",
        code: "017",
        villages: &[VillageCode {
            name: "青龙林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "双鸭山林业局宝石经营所",
        code: "018",
        villages: &[VillageCode {
            name: "宝石经营所虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "双鸭山林业局七星河金矿",
        code: "019",
        villages: &[VillageCode {
            name: "七星河金矿虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "桦南林业局岚峰林场",
        code: "020",
        villages: &[VillageCode {
            name: "桦南林业局岚峰林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "五九七农场",
        code: "021",
        villages: &[
            VillageCode {
                name: "五九七场直社区",
                code: "001",
            },
            VillageCode {
                name: "五九七农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "五九七农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "五九七农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "五九七农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "五九七农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "五九七农场第六管理区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "八五二农场",
        code: "022",
        villages: &[
            VillageCode {
                name: "八五二场直社区",
                code: "001",
            },
            VillageCode {
                name: "八五二农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "八五二农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "八五二农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "八五二农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "八五二农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "八五二农场第六管理区",
                code: "007",
            },
            VillageCode {
                name: "八五二农场第七管理区",
                code: "008",
            },
            VillageCode {
                name: "八五二农场第八管理区",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "八五三农场",
        code: "023",
        villages: &[
            VillageCode {
                name: "八五三场直社区",
                code: "001",
            },
            VillageCode {
                name: "八五三农场一管理区",
                code: "002",
            },
            VillageCode {
                name: "八五三农场二管理区",
                code: "003",
            },
            VillageCode {
                name: "八五三农场三管理区",
                code: "004",
            },
            VillageCode {
                name: "八五三农场四管理区",
                code: "005",
            },
            VillageCode {
                name: "八五三农场五管理区",
                code: "006",
            },
            VillageCode {
                name: "八五三农场六管理区",
                code: "007",
            },
            VillageCode {
                name: "八五三农场七管理区",
                code: "008",
            },
        ],
    },
];

static TOWNS_HJ_017: [TownCode; 25] = [
    TownCode {
        name: "饶河镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "沿江社区",
                code: "001",
            },
            VillageCode {
                name: "团山社区",
                code: "002",
            },
            VillageCode {
                name: "欣民社区",
                code: "003",
            },
            VillageCode {
                name: "荣久社区",
                code: "004",
            },
            VillageCode {
                name: "饶河村委会",
                code: "005",
            },
            VillageCode {
                name: "镇北村委会",
                code: "006",
            },
            VillageCode {
                name: "振兴村委会",
                code: "007",
            },
            VillageCode {
                name: "岭南朝鲜族村委会",
                code: "008",
            },
            VillageCode {
                name: "三义村委会",
                code: "009",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "010",
            },
            VillageCode {
                name: "昌盛村委会",
                code: "011",
            },
            VillageCode {
                name: "森川林场生活区",
                code: "012",
            },
            VillageCode {
                name: "王家店生活区",
                code: "013",
            },
            VillageCode {
                name: "带阳生活区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "小佳河镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "佳平村委会",
                code: "001",
            },
            VillageCode {
                name: "小佳河村委会",
                code: "002",
            },
            VillageCode {
                name: "佳兴村委会",
                code: "003",
            },
            VillageCode {
                name: "蛤蟆河村委会",
                code: "004",
            },
            VillageCode {
                name: "林海村委会",
                code: "005",
            },
            VillageCode {
                name: "新村村委会",
                code: "006",
            },
            VillageCode {
                name: "新风村委会",
                code: "007",
            },
            VillageCode {
                name: "东鲜朝鲜族村委会",
                code: "008",
            },
            VillageCode {
                name: "蜂场村委会",
                code: "009",
            },
            VillageCode {
                name: "永丰朝鲜族村委会",
                code: "010",
            },
            VillageCode {
                name: "富饶村委会",
                code: "011",
            },
            VillageCode {
                name: "创新村委会",
                code: "012",
            },
            VillageCode {
                name: "杏树村委会",
                code: "013",
            },
            VillageCode {
                name: "吉龙村委会",
                code: "014",
            },
            VillageCode {
                name: "河兴社区村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "西丰镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "西丰村委会",
                code: "001",
            },
            VillageCode {
                name: "东林村委会",
                code: "002",
            },
            VillageCode {
                name: "东南村委会",
                code: "003",
            },
            VillageCode {
                name: "五道桥村委会",
                code: "004",
            },
            VillageCode {
                name: "莲花村委会",
                code: "005",
            },
            VillageCode {
                name: "乐山村委会",
                code: "006",
            },
            VillageCode {
                name: "河南村委会",
                code: "007",
            },
            VillageCode {
                name: "河北村委会",
                code: "008",
            },
            VillageCode {
                name: "联合村委会",
                code: "009",
            },
            VillageCode {
                name: "富丰村委会",
                code: "010",
            },
            VillageCode {
                name: "芦源村委会",
                code: "011",
            },
            VillageCode {
                name: "苇子沟村委会",
                code: "012",
            },
            VillageCode {
                name: "长山村委会",
                code: "013",
            },
            VillageCode {
                name: "迎丰村委会",
                code: "014",
            },
            VillageCode {
                name: "连丰村委会",
                code: "015",
            },
            VillageCode {
                name: "果树场生活区",
                code: "016",
            },
            VillageCode {
                name: "良种场生活区",
                code: "017",
            },
            VillageCode {
                name: "渔丰村生活区",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "五林洞镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "鹿山村委会",
                code: "001",
            },
            VillageCode {
                name: "关门村委会",
                code: "002",
            },
            VillageCode {
                name: "西南岔村委会",
                code: "003",
            },
            VillageCode {
                name: "大带村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "西林子乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "西林子村委会",
                code: "001",
            },
            VillageCode {
                name: "北山村委会",
                code: "002",
            },
            VillageCode {
                name: "小南河村委会",
                code: "003",
            },
            VillageCode {
                name: "靠山村委会",
                code: "004",
            },
            VillageCode {
                name: "兰桥村委会",
                code: "005",
            },
            VillageCode {
                name: "三人班村委会",
                code: "006",
            },
            VillageCode {
                name: "柳兰村委会",
                code: "007",
            },
            VillageCode {
                name: "西川河村委会",
                code: "008",
            },
            VillageCode {
                name: "沙河子村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "四排乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "四排赫哲族村委会",
                code: "001",
            },
            VillageCode {
                name: "曙光村委会",
                code: "002",
            },
            VillageCode {
                name: "东河村委会",
                code: "003",
            },
            VillageCode {
                name: "平原村委会",
                code: "004",
            },
            VillageCode {
                name: "马架子林场生活区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "大佳河乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "大佳河村委会",
                code: "001",
            },
            VillageCode {
                name: "东升村委会",
                code: "002",
            },
            VillageCode {
                name: "前唐村委会",
                code: "003",
            },
            VillageCode {
                name: "富山村委会",
                code: "004",
            },
            VillageCode {
                name: "富河村委会",
                code: "005",
            },
            VillageCode {
                name: "桦林村委会",
                code: "006",
            },
            VillageCode {
                name: "永发村委会",
                code: "007",
            },
            VillageCode {
                name: "永前村委会",
                code: "008",
            },
            VillageCode {
                name: "永富村委会",
                code: "009",
            },
            VillageCode {
                name: "永胜村委会",
                code: "010",
            },
            VillageCode {
                name: "种畜场生活区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "山里乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "山里村委会",
                code: "001",
            },
            VillageCode {
                name: "光明村委会",
                code: "002",
            },
            VillageCode {
                name: "双河村委会",
                code: "003",
            },
            VillageCode {
                name: "二林子村委会",
                code: "004",
            },
            VillageCode {
                name: "奋斗村委会",
                code: "005",
            },
            VillageCode {
                name: "山河村委会",
                code: "006",
            },
            VillageCode {
                name: "三道岗村委会",
                code: "007",
            },
            VillageCode {
                name: "新利村委会",
                code: "008",
            },
            VillageCode {
                name: "二道岗村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "大通河乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "青山村委会",
                code: "001",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "002",
            },
            VillageCode {
                name: "镇江村委会",
                code: "003",
            },
            VillageCode {
                name: "太平村委会",
                code: "004",
            },
            VillageCode {
                name: "永利村委会",
                code: "005",
            },
            VillageCode {
                name: "永合村委会",
                code: "006",
            },
            VillageCode {
                name: "永明村委会",
                code: "007",
            },
            VillageCode {
                name: "通河林场生活区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "小佳河林场",
        code: "010",
        villages: &[VillageCode {
            name: "小佳河林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "威山林场",
        code: "011",
        villages: &[VillageCode {
            name: "威山林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "西丰林场",
        code: "012",
        villages: &[VillageCode {
            name: "西丰林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "大牙克林场",
        code: "013",
        villages: &[VillageCode {
            name: "大牙克林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "石场林场",
        code: "014",
        villages: &[VillageCode {
            name: "石场林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "宝马山林场",
        code: "015",
        villages: &[VillageCode {
            name: "宝马山林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "大岱林场",
        code: "016",
        villages: &[VillageCode {
            name: "大岱林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "永幸林场",
        code: "017",
        villages: &[VillageCode {
            name: "永幸林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "奇源林场",
        code: "018",
        villages: &[VillageCode {
            name: "奇源林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "芦源林场",
        code: "019",
        villages: &[VillageCode {
            name: "芦源林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "五林洞林场",
        code: "020",
        villages: &[VillageCode {
            name: "五林洞林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "饶河农场",
        code: "021",
        villages: &[
            VillageCode {
                name: "饶河场直社区",
                code: "001",
            },
            VillageCode {
                name: "饶河农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "饶河农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "饶河农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "饶河农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "饶河农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "饶河农场第六管理区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "红旗岭农场",
        code: "022",
        villages: &[
            VillageCode {
                name: "红旗岭场直社区",
                code: "001",
            },
            VillageCode {
                name: "红旗岭农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "红旗岭农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "红旗岭农场第四管理区",
                code: "004",
            },
            VillageCode {
                name: "红旗岭农场第五管理区",
                code: "005",
            },
            VillageCode {
                name: "红旗岭农场第三管理区",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "八五九农场",
        code: "023",
        villages: &[
            VillageCode {
                name: "八五九场直社区",
                code: "001",
            },
            VillageCode {
                name: "八五九农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "八五九农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "八五九农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "八五九农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "八五九农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "八五九农场第六管理区",
                code: "007",
            },
            VillageCode {
                name: "八五九农场第七管理区",
                code: "008",
            },
            VillageCode {
                name: "八五九农场第八管理区",
                code: "009",
            },
            VillageCode {
                name: "八五九农场第九管理区",
                code: "010",
            },
            VillageCode {
                name: "八五九农场第十二管理区",
                code: "011",
            },
            VillageCode {
                name: "八五九农场第十三管理区",
                code: "012",
            },
            VillageCode {
                name: "八五九农场第十四管理区",
                code: "013",
            },
            VillageCode {
                name: "八五九农场第十五管理区",
                code: "014",
            },
            VillageCode {
                name: "八五九农场第十管理区",
                code: "015",
            },
            VillageCode {
                name: "八五九农场第十一管理区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "胜利农场",
        code: "024",
        villages: &[
            VillageCode {
                name: "胜利场直社区",
                code: "001",
            },
            VillageCode {
                name: "胜利农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "胜利农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "胜利农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "胜利农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "胜利农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "胜利农场第六管理区",
                code: "007",
            },
            VillageCode {
                name: "胜利农场第七管理区",
                code: "008",
            },
            VillageCode {
                name: "胜利农场第八管理区",
                code: "009",
            },
            VillageCode {
                name: "胜利农场第九管理区",
                code: "010",
            },
            VillageCode {
                name: "胜利农场第十管理区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "红卫农场",
        code: "025",
        villages: &[
            VillageCode {
                name: "红卫场直社区",
                code: "001",
            },
            VillageCode {
                name: "红卫农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "红卫农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "红卫农场第四管理区",
                code: "004",
            },
            VillageCode {
                name: "红卫农场第五管理区",
                code: "005",
            },
            VillageCode {
                name: "红卫农场第七管理区",
                code: "006",
            },
            VillageCode {
                name: "红卫农场第八管理区",
                code: "007",
            },
            VillageCode {
                name: "红卫农场第九管理区",
                code: "008",
            },
            VillageCode {
                name: "红卫农场第十管理区",
                code: "009",
            },
            VillageCode {
                name: "红卫农场第三管理区",
                code: "010",
            },
        ],
    },
];

static TOWNS_HJ_018: [TownCode; 5] = [
    TownCode {
        name: "北山街道",
        code: "001",
        villages: &[VillageCode {
            name: "梧桐社区",
            code: "001",
        }],
    },
    TownCode {
        name: "红军街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "友谊社区",
                code: "001",
            },
            VillageCode {
                name: "煤城社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "光明街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "振兴社区",
                code: "001",
            },
            VillageCode {
                name: "和平社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "胜利街道",
        code: "004",
        villages: &[VillageCode {
            name: "红叶社区",
            code: "001",
        }],
    },
    TownCode {
        name: "南翼街道",
        code: "005",
        villages: &[VillageCode {
            name: "南翼社区",
            code: "001",
        }],
    },
];

static TOWNS_HJ_019: [TownCode; 5] = [
    TownCode {
        name: "永安街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "永安社区",
                code: "001",
            },
            VillageCode {
                name: "保卫社区",
                code: "002",
            },
            VillageCode {
                name: "园林社区",
                code: "003",
            },
            VillageCode {
                name: "怡安社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "港湾街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "金港湾社区",
                code: "001",
            },
            VillageCode {
                name: "港务社区",
                code: "002",
            },
            VillageCode {
                name: "粮库社区",
                code: "003",
            },
            VillageCode {
                name: "春光社区",
                code: "004",
            },
            VillageCode {
                name: "亮子河社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "和平街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "站前南社区",
                code: "001",
            },
            VillageCode {
                name: "乐园社区",
                code: "002",
            },
            VillageCode {
                name: "桥南社区",
                code: "003",
            },
            VillageCode {
                name: "林海社区",
                code: "004",
            },
            VillageCode {
                name: "先锋社区",
                code: "005",
            },
            VillageCode {
                name: "云峰社区",
                code: "006",
            },
            VillageCode {
                name: "枫桥社区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "山水街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "山水社区",
                code: "001",
            },
            VillageCode {
                name: "新立社区",
                code: "002",
            },
            VillageCode {
                name: "宏达社区",
                code: "003",
            },
            VillageCode {
                name: "双合社区",
                code: "004",
            },
            VillageCode {
                name: "江口社区",
                code: "005",
            },
            VillageCode {
                name: "佳莲社区",
                code: "006",
            },
            VillageCode {
                name: "南岗村",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "前进区农垦",
        code: "005",
        villages: &[VillageCode {
            name: "前进区农垦虚拟生活区",
            code: "001",
        }],
    },
];

static TOWNS_HJ_020: [TownCode; 6] = [
    TownCode {
        name: "晓云街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "晓云社区居委会",
                code: "001",
            },
            VillageCode {
                name: "机务社区居委会",
                code: "002",
            },
            VillageCode {
                name: "玫瑰园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "长胜社区居委会",
                code: "004",
            },
            VillageCode {
                name: "安庆社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "佳东街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "东兴社区居委会",
                code: "001",
            },
            VillageCode {
                name: "高新社区居委会",
                code: "002",
            },
            VillageCode {
                name: "南兴社区居委会",
                code: "003",
            },
            VillageCode {
                name: "佳东社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东安社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "建国街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "造纸社区居委会",
                code: "001",
            },
            VillageCode {
                name: "电机社区居委会",
                code: "002",
            },
            VillageCode {
                name: "五彩社区居委会",
                code: "003",
            },
            VillageCode {
                name: "永佳社区居委会",
                code: "004",
            },
            VillageCode {
                name: "兴电社区居委会",
                code: "005",
            },
            VillageCode {
                name: "群楼社区居委会",
                code: "006",
            },
            VillageCode {
                name: "警安社区居委会",
                code: "007",
            },
            VillageCode {
                name: "水源社区居委会",
                code: "008",
            },
            VillageCode {
                name: "沿音社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "佳南街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "达贤社区居委会",
                code: "001",
            },
            VillageCode {
                name: "丰登社区居委会",
                code: "002",
            },
            VillageCode {
                name: "胜利社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "建国镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "大堆丰村委会",
                code: "001",
            },
            VillageCode {
                name: "建国村委会",
                code: "002",
            },
            VillageCode {
                name: "建设村委会",
                code: "003",
            },
            VillageCode {
                name: "黎明村委会",
                code: "004",
            },
            VillageCode {
                name: "群利村委会",
                code: "005",
            },
            VillageCode {
                name: "西太平村委会",
                code: "006",
            },
            VillageCode {
                name: "永丰村委会",
                code: "007",
            },
            VillageCode {
                name: "圳江村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "松江乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "东联社区居委会",
                code: "001",
            },
            VillageCode {
                name: "富贵社区居委会",
                code: "002",
            },
            VillageCode {
                name: "东平社区居委会",
                code: "003",
            },
            VillageCode {
                name: "江滨社区居委会",
                code: "004",
            },
            VillageCode {
                name: "长兴村委会",
                code: "005",
            },
            VillageCode {
                name: "恒心村委会",
                code: "006",
            },
            VillageCode {
                name: "红力村委会",
                code: "007",
            },
            VillageCode {
                name: "宏伟村委会",
                code: "008",
            },
            VillageCode {
                name: "江山村委会",
                code: "009",
            },
            VillageCode {
                name: "联合村委会",
                code: "010",
            },
            VillageCode {
                name: "模范村委会",
                code: "011",
            },
            VillageCode {
                name: "农家村委会",
                code: "012",
            },
            VillageCode {
                name: "松江村委会",
                code: "013",
            },
            VillageCode {
                name: "新民村委会",
                code: "014",
            },
            VillageCode {
                name: "兴国村委会",
                code: "015",
            },
            VillageCode {
                name: "双新村委会",
                code: "016",
            },
        ],
    },
];

static TOWNS_HJ_021: [TownCode; 8] = [
    TownCode {
        name: "荫营镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "荫营煤矿社区居委会",
                code: "001",
            },
            VillageCode {
                name: "老虎沟社区居委会",
                code: "002",
            },
            VillageCode {
                name: "下荫营瑞丰社区居委会",
                code: "003",
            },
            VillageCode {
                name: "下荫营文苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "上荫营社区居委会",
                code: "005",
            },
            VillageCode {
                name: "玉泉社区居委会",
                code: "006",
            },
            VillageCode {
                name: "坪上社区居委会",
                code: "007",
            },
            VillageCode {
                name: "桥上社区居委会",
                code: "008",
            },
            VillageCode {
                name: "下荫营社区居委会",
                code: "009",
            },
            VillageCode {
                name: "矾窑村委会",
                code: "010",
            },
            VillageCode {
                name: "上千亩坪村委会",
                code: "011",
            },
            VillageCode {
                name: "下千亩坪村委会",
                code: "012",
            },
            VillageCode {
                name: "上烟村委会",
                code: "013",
            },
            VillageCode {
                name: "下烟村委会",
                code: "014",
            },
            VillageCode {
                name: "三泉村委会",
                code: "015",
            },
            VillageCode {
                name: "三郊村委会",
                code: "016",
            },
            VillageCode {
                name: "三都村委会",
                code: "017",
            },
            VillageCode {
                name: "杨树沟村委会",
                code: "018",
            },
            VillageCode {
                name: "双福村委会",
                code: "019",
            },
            VillageCode {
                name: "垴上村委会",
                code: "020",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "021",
            },
            VillageCode {
                name: "小庄村委会",
                code: "022",
            },
            VillageCode {
                name: "韩庄村委会",
                code: "023",
            },
            VillageCode {
                name: "福洼村委会",
                code: "024",
            },
            VillageCode {
                name: "下白泉村委会",
                code: "025",
            },
            VillageCode {
                name: "上白泉村委会",
                code: "026",
            },
            VillageCode {
                name: "落菇堰村委会",
                code: "027",
            },
            VillageCode {
                name: "东梁庄村委会",
                code: "028",
            },
            VillageCode {
                name: "西梨庄村委会",
                code: "029",
            },
            VillageCode {
                name: "段家庄村委会",
                code: "030",
            },
            VillageCode {
                name: "山头村委会",
                code: "031",
            },
            VillageCode {
                name: "窑沟村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "河底镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "固庄煤矿社区居委会",
                code: "001",
            },
            VillageCode {
                name: "河底村委会",
                code: "002",
            },
            VillageCode {
                name: "中佐村委会",
                code: "003",
            },
            VillageCode {
                name: "龙光峪村委会",
                code: "004",
            },
            VillageCode {
                name: "苇泊村委会",
                code: "005",
            },
            VillageCode {
                name: "上章召村委会",
                code: "006",
            },
            VillageCode {
                name: "下章召村委会",
                code: "007",
            },
            VillageCode {
                name: "任家峪村委会",
                code: "008",
            },
            VillageCode {
                name: "邓家峪村委会",
                code: "009",
            },
            VillageCode {
                name: "关家峪村委会",
                code: "010",
            },
            VillageCode {
                name: "固庄村委会",
                code: "011",
            },
            VillageCode {
                name: "牵牛镇村委会",
                code: "012",
            },
            VillageCode {
                name: "东村村委会",
                code: "013",
            },
            VillageCode {
                name: "东南沟村委会",
                code: "014",
            },
            VillageCode {
                name: "苏家泉村委会",
                code: "015",
            },
            VillageCode {
                name: "大河北村委会",
                code: "016",
            },
            VillageCode {
                name: "小河北村委会",
                code: "017",
            },
            VillageCode {
                name: "五架山村委会",
                code: "018",
            },
            VillageCode {
                name: "北小西庄村委会",
                code: "019",
            },
            VillageCode {
                name: "山底村委会",
                code: "020",
            },
            VillageCode {
                name: "红土岩村委会",
                code: "021",
            },
            VillageCode {
                name: "燕龛村委会",
                code: "022",
            },
            VillageCode {
                name: "曹家掌村委会",
                code: "023",
            },
            VillageCode {
                name: "程庄村委会",
                code: "024",
            },
            VillageCode {
                name: "北庄村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "平坦镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "龙凤沟村委会",
                code: "001",
            },
            VillageCode {
                name: "甘河村委会",
                code: "002",
            },
            VillageCode {
                name: "桃林沟村委会",
                code: "003",
            },
            VillageCode {
                name: "魏家峪村委会",
                code: "004",
            },
            VillageCode {
                name: "辛兴村委会",
                code: "005",
            },
            VillageCode {
                name: "坡头村委会",
                code: "006",
            },
            VillageCode {
                name: "桑掌村委会",
                code: "007",
            },
            VillageCode {
                name: "中庄村委会",
                code: "008",
            },
            VillageCode {
                name: "西上庄村委会",
                code: "009",
            },
            VillageCode {
                name: "后峪村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "西南舁乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "张家井村委会",
                code: "001",
            },
            VillageCode {
                name: "大洼村委会",
                code: "002",
            },
            VillageCode {
                name: "东南舁村委会",
                code: "003",
            },
            VillageCode {
                name: "西南舁村委会",
                code: "004",
            },
            VillageCode {
                name: "五里庄村委会",
                code: "005",
            },
            VillageCode {
                name: "雨下沟村委会",
                code: "006",
            },
            VillageCode {
                name: "霍树头村委会",
                code: "007",
            },
            VillageCode {
                name: "北大西庄村委会",
                code: "008",
            },
            VillageCode {
                name: "北舁村委会",
                code: "009",
            },
            VillageCode {
                name: "孔南庄村委会",
                code: "010",
            },
            VillageCode {
                name: "清石台村委会",
                code: "011",
            },
            VillageCode {
                name: "咀子上村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "杨家庄乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "南杨家庄村委会",
                code: "001",
            },
            VillageCode {
                name: "北杨家庄村委会",
                code: "002",
            },
            VillageCode {
                name: "孙家沟村委会",
                code: "003",
            },
            VillageCode {
                name: "小西庄村委会",
                code: "004",
            },
            VillageCode {
                name: "白家庄村委会",
                code: "005",
            },
            VillageCode {
                name: "黑土岩村委会",
                code: "006",
            },
            VillageCode {
                name: "杏树坡村委会",
                code: "007",
            },
            VillageCode {
                name: "高垴庄村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "李家庄乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "李家庄社区居委会",
                code: "001",
            },
            VillageCode {
                name: "恒大社区居委会",
                code: "002",
            },
            VillageCode {
                name: "冯家庄社区居委会",
                code: "003",
            },
            VillageCode {
                name: "甄家庄社区居委会",
                code: "004",
            },
            VillageCode {
                name: "汉河沟村委会",
                code: "005",
            },
            VillageCode {
                name: "黄沙岩村委会",
                code: "006",
            },
            VillageCode {
                name: "大西庄村委会",
                code: "007",
            },
            VillageCode {
                name: "柳沟村委会",
                code: "008",
            },
            VillageCode {
                name: "余积粮沟村委会",
                code: "009",
            },
            VillageCode {
                name: "桃坡村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "旧街乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "测石村委会",
                code: "001",
            },
            VillageCode {
                name: "南沟村委会",
                code: "002",
            },
            VillageCode {
                name: "旧街村委会",
                code: "003",
            },
            VillageCode {
                name: "新店村委会",
                code: "004",
            },
            VillageCode {
                name: "新庄窝村委会",
                code: "005",
            },
            VillageCode {
                name: "高岺村委会",
                code: "006",
            },
            VillageCode {
                name: "佛洼村委会",
                code: "007",
            },
            VillageCode {
                name: "枣园村委会",
                code: "008",
            },
            VillageCode {
                name: "保安村委会",
                code: "009",
            },
            VillageCode {
                name: "里五村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "开发区",
        code: "008",
        villages: &[
            VillageCode {
                name: "康达社区居委会",
                code: "001",
            },
            VillageCode {
                name: "宏苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "惠泽社区居委会",
                code: "003",
            },
            VillageCode {
                name: "银龙社区居委会",
                code: "004",
            },
            VillageCode {
                name: "大华社区居委会",
                code: "005",
            },
            VillageCode {
                name: "居馨社区居委会",
                code: "006",
            },
            VillageCode {
                name: "桃源社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新澳城社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新泉社区居委会",
                code: "009",
            },
            VillageCode {
                name: "御康社区居委会",
                code: "010",
            },
            VillageCode {
                name: "东城水岸社区居委会",
                code: "011",
            },
            VillageCode {
                name: "盛世新城社区居委会",
                code: "012",
            },
            VillageCode {
                name: "下五渡社区居委会",
                code: "013",
            },
            VillageCode {
                name: "平坦垴社区居委会",
                code: "014",
            },
            VillageCode {
                name: "泉西社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "天峰社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "上五渡村委会",
                code: "017",
            },
            VillageCode {
                name: "河坡村委会",
                code: "018",
            },
            VillageCode {
                name: "王垅村委会",
                code: "019",
            },
            VillageCode {
                name: "侯家沟村委会",
                code: "020",
            },
            VillageCode {
                name: "驼岭头村委会",
                code: "021",
            },
            VillageCode {
                name: "长岭村委会",
                code: "022",
            },
            VillageCode {
                name: "路家山村委会",
                code: "023",
            },
            VillageCode {
                name: "庙堰村委会",
                code: "024",
            },
        ],
    },
];

static TOWNS_HJ_022: [TownCode; 16] = [
    TownCode {
        name: "驼腰子镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "金缸村委会",
                code: "001",
            },
            VillageCode {
                name: "金山村委会",
                code: "002",
            },
            VillageCode {
                name: "金胜村委会",
                code: "003",
            },
            VillageCode {
                name: "上桦村委会",
                code: "004",
            },
            VillageCode {
                name: "东合村委会",
                code: "005",
            },
            VillageCode {
                name: "西合村委会",
                code: "006",
            },
            VillageCode {
                name: "新合村委会",
                code: "007",
            },
            VillageCode {
                name: "光明村委会",
                code: "008",
            },
            VillageCode {
                name: "愚公村委会",
                code: "009",
            },
            VillageCode {
                name: "大兴沟村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "石头河子镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "金矿局居委会",
                code: "001",
            },
            VillageCode {
                name: "石灰石矿居委会",
                code: "002",
            },
            VillageCode {
                name: "八一村委会",
                code: "003",
            },
            VillageCode {
                name: "春富村委会",
                code: "004",
            },
            VillageCode {
                name: "核心村委会",
                code: "005",
            },
            VillageCode {
                name: "林河村委会",
                code: "006",
            },
            VillageCode {
                name: "马家村委会",
                code: "007",
            },
            VillageCode {
                name: "庆丰村委会",
                code: "008",
            },
            VillageCode {
                name: "仁和村委会",
                code: "009",
            },
            VillageCode {
                name: "向阳村委会",
                code: "010",
            },
            VillageCode {
                name: "义和村委会",
                code: "011",
            },
            VillageCode {
                name: "桦阳村委会",
                code: "012",
            },
            VillageCode {
                name: "双丰村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "桦南镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "前进社区居委会",
                code: "001",
            },
            VillageCode {
                name: "铁东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "铁西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "奋斗社区居委会",
                code: "004",
            },
            VillageCode {
                name: "新建社区居委会",
                code: "005",
            },
            VillageCode {
                name: "胜利社区居委会",
                code: "006",
            },
            VillageCode {
                name: "福庆社区居委会",
                code: "007",
            },
            VillageCode {
                name: "育博社区居委会",
                code: "008",
            },
            VillageCode {
                name: "奥林社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新月社区居委会",
                code: "010",
            },
            VillageCode {
                name: "福星社区居委会",
                code: "011",
            },
            VillageCode {
                name: "名和社区居委会",
                code: "012",
            },
            VillageCode {
                name: "金晖社区居委会",
                code: "013",
            },
            VillageCode {
                name: "文政社区居委会",
                code: "014",
            },
            VillageCode {
                name: "吉庆社区居委会",
                code: "015",
            },
            VillageCode {
                name: "文康社区居委会",
                code: "016",
            },
            VillageCode {
                name: "秀北社区居委会",
                code: "017",
            },
            VillageCode {
                name: "新兴社区居委会",
                code: "018",
            },
            VillageCode {
                name: "富强社区居委会",
                code: "019",
            },
            VillageCode {
                name: "教育社区居委会",
                code: "020",
            },
            VillageCode {
                name: "学府社区居委会",
                code: "021",
            },
            VillageCode {
                name: "正南村委会",
                code: "022",
            },
            VillageCode {
                name: "正北村委会",
                code: "023",
            },
            VillageCode {
                name: "正东村委会",
                code: "024",
            },
            VillageCode {
                name: "腰营子村委会",
                code: "025",
            },
            VillageCode {
                name: "隆胜村委会",
                code: "026",
            },
            VillageCode {
                name: "桦丰村委会",
                code: "027",
            },
            VillageCode {
                name: "五一村委会",
                code: "028",
            },
            VillageCode {
                name: "宏昌村委会",
                code: "029",
            },
            VillageCode {
                name: "宏泰村委会",
                code: "030",
            },
            VillageCode {
                name: "民富村委会",
                code: "031",
            },
            VillageCode {
                name: "富荣村委会",
                code: "032",
            },
            VillageCode {
                name: "富贵村委会",
                code: "033",
            },
            VillageCode {
                name: "幸福村委会",
                code: "034",
            },
            VillageCode {
                name: "双合村委会",
                code: "035",
            },
            VillageCode {
                name: "湖南营村委会",
                code: "036",
            },
            VillageCode {
                name: "宏元村委会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "土龙山镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "柴家村委会",
                code: "001",
            },
            VillageCode {
                name: "长青村委会",
                code: "002",
            },
            VillageCode {
                name: "凤岐村委会",
                code: "003",
            },
            VillageCode {
                name: "合力村委会",
                code: "004",
            },
            VillageCode {
                name: "洪林子村委会",
                code: "005",
            },
            VillageCode {
                name: "精勤村委会",
                code: "006",
            },
            VillageCode {
                name: "聚宝村委会",
                code: "007",
            },
            VillageCode {
                name: "三王村委会",
                code: "008",
            },
            VillageCode {
                name: "胜利村委会",
                code: "009",
            },
            VillageCode {
                name: "四合村委会",
                code: "010",
            },
            VillageCode {
                name: "太义村委会",
                code: "011",
            },
            VillageCode {
                name: "新华村委会",
                code: "012",
            },
            VillageCode {
                name: "新颜村委会",
                code: "013",
            },
            VillageCode {
                name: "新源村委会",
                code: "014",
            },
            VillageCode {
                name: "永胜村委会",
                code: "015",
            },
            VillageCode {
                name: "振山村委会",
                code: "016",
            },
            VillageCode {
                name: "战生村委会",
                code: "017",
            },
            VillageCode {
                name: "前进村委会",
                code: "018",
            },
            VillageCode {
                name: "丰收村委会",
                code: "019",
            },
            VillageCode {
                name: "庆发村委会",
                code: "020",
            },
            VillageCode {
                name: "六合村委会",
                code: "021",
            },
            VillageCode {
                name: "新发村委会",
                code: "022",
            },
            VillageCode {
                name: "横岱村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "孟家岗镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "保丰村委会",
                code: "001",
            },
            VillageCode {
                name: "永丰村委会",
                code: "002",
            },
            VillageCode {
                name: "永安村委会",
                code: "003",
            },
            VillageCode {
                name: "太安村委会",
                code: "004",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "005",
            },
            VillageCode {
                name: "楼山村委会",
                code: "006",
            },
            VillageCode {
                name: "秋丰村委会",
                code: "007",
            },
            VillageCode {
                name: "铁岭村委会",
                code: "008",
            },
            VillageCode {
                name: "先进村委会",
                code: "009",
            },
            VillageCode {
                name: "红日村委会",
                code: "010",
            },
            VillageCode {
                name: "长安村委会",
                code: "011",
            },
            VillageCode {
                name: "群英村委会",
                code: "012",
            },
            VillageCode {
                name: "功兴村委会",
                code: "013",
            },
            VillageCode {
                name: "东胜村委会",
                code: "014",
            },
            VillageCode {
                name: "八虎力村委会",
                code: "015",
            },
            VillageCode {
                name: "建国村委会",
                code: "016",
            },
            VillageCode {
                name: "建华村委会",
                code: "017",
            },
            VillageCode {
                name: "黎明村委会",
                code: "018",
            },
            VillageCode {
                name: "平安村委会",
                code: "019",
            },
            VillageCode {
                name: "西平村委会",
                code: "020",
            },
            VillageCode {
                name: "腰梨树村委会",
                code: "021",
            },
            VillageCode {
                name: "中平村委会",
                code: "022",
            },
            VillageCode {
                name: "中心村委会",
                code: "023",
            },
            VillageCode {
                name: "朱家村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "闫家镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "闫家村委会",
                code: "001",
            },
            VillageCode {
                name: "老街基村委会",
                code: "002",
            },
            VillageCode {
                name: "小八浪村委会",
                code: "003",
            },
            VillageCode {
                name: "丰基村委会",
                code: "004",
            },
            VillageCode {
                name: "城子岭村委会",
                code: "005",
            },
            VillageCode {
                name: "大吴家村委会",
                code: "006",
            },
            VillageCode {
                name: "大张家村委会",
                code: "007",
            },
            VillageCode {
                name: "北安村委会",
                code: "008",
            },
            VillageCode {
                name: "公心集村委会",
                code: "009",
            },
            VillageCode {
                name: "桦木岗村委会",
                code: "010",
            },
            VillageCode {
                name: "桦兴村委会",
                code: "011",
            },
            VillageCode {
                name: "宏伟村委会",
                code: "012",
            },
            VillageCode {
                name: "公平村委会",
                code: "013",
            },
            VillageCode {
                name: "荣安村委会",
                code: "014",
            },
            VillageCode {
                name: "中和村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "柳毛河镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "东风村委会",
                code: "001",
            },
            VillageCode {
                name: "东华村委会",
                code: "002",
            },
            VillageCode {
                name: "北柳村委会",
                code: "003",
            },
            VillageCode {
                name: "新庆村委会",
                code: "004",
            },
            VillageCode {
                name: "南柳村委会",
                code: "005",
            },
            VillageCode {
                name: "山春村委会",
                code: "006",
            },
            VillageCode {
                name: "东柳村委会",
                code: "007",
            },
            VillageCode {
                name: "长龙岗村委会",
                code: "008",
            },
            VillageCode {
                name: "向春村委会",
                code: "009",
            },
            VillageCode {
                name: "群力村委会",
                code: "010",
            },
            VillageCode {
                name: "太平村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "金沙乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "红丰村委会",
                code: "001",
            },
            VillageCode {
                name: "红城村委会",
                code: "002",
            },
            VillageCode {
                name: "红权村委会",
                code: "003",
            },
            VillageCode {
                name: "红新村委会",
                code: "004",
            },
            VillageCode {
                name: "前金沙村委会",
                code: "005",
            },
            VillageCode {
                name: "东民主村委会",
                code: "006",
            },
            VillageCode {
                name: "工农村委会",
                code: "007",
            },
            VillageCode {
                name: "治山村委会",
                code: "008",
            },
            VillageCode {
                name: "长征村委会",
                code: "009",
            },
            VillageCode {
                name: "卫东村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "梨树乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "北大村委会",
                code: "001",
            },
            VillageCode {
                name: "大胜村委会",
                code: "002",
            },
            VillageCode {
                name: "东柞村委会",
                code: "003",
            },
            VillageCode {
                name: "和平村委会",
                code: "004",
            },
            VillageCode {
                name: "梨树村委会",
                code: "005",
            },
            VillageCode {
                name: "南大村委会",
                code: "006",
            },
            VillageCode {
                name: "西大村委会",
                code: "007",
            },
            VillageCode {
                name: "西柞村委会",
                code: "008",
            },
            VillageCode {
                name: "永和村委会",
                code: "009",
            },
            VillageCode {
                name: "民主村委会",
                code: "010",
            },
            VillageCode {
                name: "红大村委会",
                code: "011",
            },
            VillageCode {
                name: "长兴村委会",
                code: "012",
            },
            VillageCode {
                name: "福山村委会",
                code: "013",
            },
            VillageCode {
                name: "福兴村委会",
                code: "014",
            },
            VillageCode {
                name: "红光村委会",
                code: "015",
            },
            VillageCode {
                name: "红历村委会",
                code: "016",
            },
            VillageCode {
                name: "红升村委会",
                code: "017",
            },
            VillageCode {
                name: "永久村委会",
                code: "018",
            },
            VillageCode {
                name: "永兴村委会",
                code: "019",
            },
            VillageCode {
                name: "永远村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "明义乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "朝阳村委会",
                code: "001",
            },
            VillageCode {
                name: "东双龙河村委会",
                code: "002",
            },
            VillageCode {
                name: "双龙河村委会",
                code: "003",
            },
            VillageCode {
                name: "油坊村委会",
                code: "004",
            },
            VillageCode {
                name: "奋斗村委会",
                code: "005",
            },
            VillageCode {
                name: "共和村委会",
                code: "006",
            },
            VillageCode {
                name: "东辉村委会",
                code: "007",
            },
            VillageCode {
                name: "永红村委会",
                code: "008",
            },
            VillageCode {
                name: "明义村委会",
                code: "009",
            },
            VillageCode {
                name: "前合发村委会",
                code: "010",
            },
            VillageCode {
                name: "北合发村委会",
                code: "011",
            },
            VillageCode {
                name: "三合村委会",
                code: "012",
            },
            VillageCode {
                name: "新生村委会",
                code: "013",
            },
            VillageCode {
                name: "五分村委会",
                code: "014",
            },
            VillageCode {
                name: "永昌村委会",
                code: "015",
            },
            VillageCode {
                name: "清茶村委会",
                code: "016",
            },
            VillageCode {
                name: "团结村委会",
                code: "017",
            },
            VillageCode {
                name: "立新村委会",
                code: "018",
            },
            VillageCode {
                name: "兴旺村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "大八浪乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "大八浪村委会",
                code: "001",
            },
            VillageCode {
                name: "大鲜村委会",
                code: "002",
            },
            VillageCode {
                name: "宝山村委会",
                code: "003",
            },
            VillageCode {
                name: "铁山村委会",
                code: "004",
            },
            VillageCode {
                name: "新富村委会",
                code: "005",
            },
            VillageCode {
                name: "检草沟村委会",
                code: "006",
            },
            VillageCode {
                name: "九里六村委会",
                code: "007",
            },
            VillageCode {
                name: "达连泡村委会",
                code: "008",
            },
            VillageCode {
                name: "二道沟村委会",
                code: "009",
            },
            VillageCode {
                name: "德荣村委会",
                code: "010",
            },
            VillageCode {
                name: "西太平村委会",
                code: "011",
            },
            VillageCode {
                name: "振兴村委会",
                code: "012",
            },
            VillageCode {
                name: "齐心村委会",
                code: "013",
            },
            VillageCode {
                name: "先锋村委会",
                code: "014",
            },
            VillageCode {
                name: "七一村委会",
                code: "015",
            },
            VillageCode {
                name: "吉兴村委会",
                code: "016",
            },
            VillageCode {
                name: "北太平村委会",
                code: "017",
            },
            VillageCode {
                name: "东安村委会",
                code: "018",
            },
            VillageCode {
                name: "双鸭子村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "五道岗乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "东大村委会",
                code: "001",
            },
            VillageCode {
                name: "林发村委会",
                code: "002",
            },
            VillageCode {
                name: "永发村委会",
                code: "003",
            },
            VillageCode {
                name: "桃源村委会",
                code: "004",
            },
            VillageCode {
                name: "新民村委会",
                code: "005",
            },
            VillageCode {
                name: "五道岗村委会",
                code: "006",
            },
            VillageCode {
                name: "金生村委会",
                code: "007",
            },
            VillageCode {
                name: "福安村委会",
                code: "008",
            },
            VillageCode {
                name: "复兴村委会",
                code: "009",
            },
            VillageCode {
                name: "兴中村委会",
                code: "010",
            },
            VillageCode {
                name: "大木岗村委会",
                code: "011",
            },
            VillageCode {
                name: "东升村委会",
                code: "012",
            },
            VillageCode {
                name: "丰林村委会",
                code: "013",
            },
            VillageCode {
                name: "复胜村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "桦南林业局",
        code: "013",
        villages: &[
            VillageCode {
                name: "东林社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中心社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西林社区居委会",
                code: "003",
            },
            VillageCode {
                name: "福溪社区居委会",
                code: "004",
            },
            VillageCode {
                name: "大肚川社区居委会",
                code: "005",
            },
            VillageCode {
                name: "长青社区居委会",
                code: "006",
            },
            VillageCode {
                name: "红光社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "黑龙江桦南经济开发区",
        code: "014",
        villages: &[VillageCode {
            name: "工业园区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "曙光农场",
        code: "015",
        villages: &[
            VillageCode {
                name: "曙光第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "曙光第二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "曙光农场第一管理区社区",
                code: "003",
            },
            VillageCode {
                name: "曙光农场第二管理区社区",
                code: "004",
            },
            VillageCode {
                name: "曙光农场第三管理区社区",
                code: "005",
            },
            VillageCode {
                name: "曙光农场第四管理区社区",
                code: "006",
            },
            VillageCode {
                name: "曙光农场第五管理区社区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "桦南种畜场",
        code: "016",
        villages: &[
            VillageCode {
                name: "第一分场生活区",
                code: "001",
            },
            VillageCode {
                name: "第二分场生活区",
                code: "002",
            },
            VillageCode {
                name: "第三分场生活区",
                code: "003",
            },
            VillageCode {
                name: "第四分场生活区",
                code: "004",
            },
            VillageCode {
                name: "第五分场生活区",
                code: "005",
            },
            VillageCode {
                name: "第六分场生活区",
                code: "006",
            },
            VillageCode {
                name: "第七分场生活区",
                code: "007",
            },
        ],
    },
];

static TOWNS_HJ_023: [TownCode; 11] = [
    TownCode {
        name: "横头山镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "解放村委会",
                code: "001",
            },
            VillageCode {
                name: "日升村委会",
                code: "002",
            },
            VillageCode {
                name: "向阳堡村委会",
                code: "003",
            },
            VillageCode {
                name: "合乡村委会",
                code: "004",
            },
            VillageCode {
                name: "西朝阳村委会",
                code: "005",
            },
            VillageCode {
                name: "葡萄沟村委会",
                code: "006",
            },
            VillageCode {
                name: "东朝阳村委会",
                code: "007",
            },
            VillageCode {
                name: "万宝村委会",
                code: "008",
            },
            VillageCode {
                name: "国兴村委会",
                code: "009",
            },
            VillageCode {
                name: "申家店村委会",
                code: "010",
            },
            VillageCode {
                name: "横头河子种畜场生活区",
                code: "011",
            },
            VillageCode {
                name: "横头山良种场生活区",
                code: "012",
            },
            VillageCode {
                name: "横头山林场生活区",
                code: "013",
            },
            VillageCode {
                name: "老平岗林场生活区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "苏家店镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "苏家店村委会",
                code: "001",
            },
            VillageCode {
                name: "自新村委会",
                code: "002",
            },
            VillageCode {
                name: "兴光村委会",
                code: "003",
            },
            VillageCode {
                name: "新胜村委会",
                code: "004",
            },
            VillageCode {
                name: "朱家村委会",
                code: "005",
            },
            VillageCode {
                name: "八家子村委会",
                code: "006",
            },
            VillageCode {
                name: "北山村委会",
                code: "007",
            },
            VillageCode {
                name: "中安村委会",
                code: "008",
            },
            VillageCode {
                name: "集贤村委会",
                code: "009",
            },
            VillageCode {
                name: "团结村委会",
                code: "010",
            },
            VillageCode {
                name: "桦树川村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "悦来镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "长新居委会",
                code: "001",
            },
            VillageCode {
                name: "荣安居委会",
                code: "002",
            },
            VillageCode {
                name: "建民居委会",
                code: "003",
            },
            VillageCode {
                name: "团结居委会",
                code: "004",
            },
            VillageCode {
                name: "悦东村委会",
                code: "005",
            },
            VillageCode {
                name: "悦江村委会",
                code: "006",
            },
            VillageCode {
                name: "敬夫村委会",
                code: "007",
            },
            VillageCode {
                name: "冷云村委会",
                code: "008",
            },
            VillageCode {
                name: "悦胜村委会",
                code: "009",
            },
            VillageCode {
                name: "悦强村委会",
                code: "010",
            },
            VillageCode {
                name: "万里河村委会",
                code: "011",
            },
            VillageCode {
                name: "孟家岗村委会",
                code: "012",
            },
            VillageCode {
                name: "马库力村委会",
                code: "013",
            },
            VillageCode {
                name: "腰林子村委会",
                code: "014",
            },
            VillageCode {
                name: "双兴村委会",
                code: "015",
            },
            VillageCode {
                name: "桦树村委会",
                code: "016",
            },
            VillageCode {
                name: "万升村委会",
                code: "017",
            },
            VillageCode {
                name: "中和村委会",
                code: "018",
            },
            VillageCode {
                name: "苏苏村委会",
                code: "019",
            },
            VillageCode {
                name: "东兴村委会",
                code: "020",
            },
            VillageCode {
                name: "汶澄村委会",
                code: "021",
            },
            VillageCode {
                name: "花园居委会",
                code: "022",
            },
            VillageCode {
                name: "水木年华居委会",
                code: "023",
            },
            VillageCode {
                name: "华兴居委会",
                code: "024",
            },
            VillageCode {
                name: "阳光居委会",
                code: "025",
            },
            VillageCode {
                name: "悦绣居委会",
                code: "026",
            },
            VillageCode {
                name: "学府居委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "新城镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "爱国村委会",
                code: "001",
            },
            VillageCode {
                name: "永红村委会",
                code: "002",
            },
            VillageCode {
                name: "四合村委会",
                code: "003",
            },
            VillageCode {
                name: "彻胜村委会",
                code: "004",
            },
            VillageCode {
                name: "仁发村委会",
                code: "005",
            },
            VillageCode {
                name: "前进村委会",
                code: "006",
            },
            VillageCode {
                name: "新华村委会",
                code: "007",
            },
            VillageCode {
                name: "玉丰村委会",
                code: "008",
            },
            VillageCode {
                name: "同力村委会",
                code: "009",
            },
            VillageCode {
                name: "古城村委会",
                code: "010",
            },
            VillageCode {
                name: "宏伟村委会",
                code: "011",
            },
            VillageCode {
                name: "协胜村委会",
                code: "012",
            },
            VillageCode {
                name: "东宝山村委会",
                code: "013",
            },
            VillageCode {
                name: "西宝山村委会",
                code: "014",
            },
            VillageCode {
                name: "七星村委会",
                code: "015",
            },
            VillageCode {
                name: "中伏村委会",
                code: "016",
            },
            VillageCode {
                name: "乌龙村委会",
                code: "017",
            },
            VillageCode {
                name: "中胜村委会",
                code: "018",
            },
            VillageCode {
                name: "东方村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "四马架镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "会龙村委会",
                code: "001",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "002",
            },
            VillageCode {
                name: "红星村委会",
                code: "003",
            },
            VillageCode {
                name: "宝山村委会",
                code: "004",
            },
            VillageCode {
                name: "德庆村委会",
                code: "005",
            },
            VillageCode {
                name: "东华村委会",
                code: "006",
            },
            VillageCode {
                name: "永胜村委会",
                code: "007",
            },
            VillageCode {
                name: "文化村委会",
                code: "008",
            },
            VillageCode {
                name: "四马架村委会",
                code: "009",
            },
            VillageCode {
                name: "六合村委会",
                code: "010",
            },
            VillageCode {
                name: "光复村委会",
                code: "011",
            },
            VillageCode {
                name: "长胜村委会",
                code: "012",
            },
            VillageCode {
                name: "同乐村委会",
                code: "013",
            },
            VillageCode {
                name: "山湾村委会",
                code: "014",
            },
            VillageCode {
                name: "新兴村委会",
                code: "015",
            },
            VillageCode {
                name: "民乐村委会",
                code: "016",
            },
            VillageCode {
                name: "仁合村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "东河乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "九阳村委会",
                code: "001",
            },
            VillageCode {
                name: "兴安村委会",
                code: "002",
            },
            VillageCode {
                name: "东方红村委会",
                code: "003",
            },
            VillageCode {
                name: "东升村委会",
                code: "004",
            },
            VillageCode {
                name: "东宏村委会",
                code: "005",
            },
            VillageCode {
                name: "东河村委会",
                code: "006",
            },
            VillageCode {
                name: "兴国村委会",
                code: "007",
            },
            VillageCode {
                name: "东方红种牛场生活区",
                code: "008",
            },
            VillageCode {
                name: "东方红良种场生活区",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "梨丰乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "南林村委会",
                code: "001",
            },
            VillageCode {
                name: "东兴村委会",
                code: "002",
            },
            VillageCode {
                name: "梨树村委会",
                code: "003",
            },
            VillageCode {
                name: "繁荣村委会",
                code: "004",
            },
            VillageCode {
                name: "东岗村委会",
                code: "005",
            },
            VillageCode {
                name: "昌盛村委会",
                code: "006",
            },
            VillageCode {
                name: "东林村委会",
                code: "007",
            },
            VillageCode {
                name: "黎明村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "创业乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "堆丰里村委会",
                code: "001",
            },
            VillageCode {
                name: "中山村委会",
                code: "002",
            },
            VillageCode {
                name: "西冯村委会",
                code: "003",
            },
            VillageCode {
                name: "拉拉街村委会",
                code: "004",
            },
            VillageCode {
                name: "谷大村委会",
                code: "005",
            },
            VillageCode {
                name: "丰年村委会",
                code: "006",
            },
            VillageCode {
                name: "新发村委会",
                code: "007",
            },
            VillageCode {
                name: "宏图村委会",
                code: "008",
            },
            VillageCode {
                name: "西大村委会",
                code: "009",
            },
            VillageCode {
                name: "小堆峰村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "星火乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "中星村委会",
                code: "001",
            },
            VillageCode {
                name: "燎原村委会",
                code: "002",
            },
            VillageCode {
                name: "星火村委会",
                code: "003",
            },
            VillageCode {
                name: "红光村委会",
                code: "004",
            },
            VillageCode {
                name: "星光村委会",
                code: "005",
            },
            VillageCode {
                name: "燎新村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "江川农场",
        code: "010",
        villages: &[
            VillageCode {
                name: "江川场直社区",
                code: "001",
            },
            VillageCode {
                name: "江川农场第一管理区",
                code: "002",
            },
            VillageCode {
                name: "江川农场第二管理区",
                code: "003",
            },
            VillageCode {
                name: "江川农场第三管理区",
                code: "004",
            },
            VillageCode {
                name: "江川农场第四管理区",
                code: "005",
            },
            VillageCode {
                name: "江川农场第五管理区",
                code: "006",
            },
            VillageCode {
                name: "江川农场第六管理区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "宝山农场",
        code: "011",
        villages: &[
            VillageCode {
                name: "场直第一社区",
                code: "001",
            },
            VillageCode {
                name: "场直第二社区",
                code: "002",
            },
        ],
    },
];

static TOWNS_HJ_024: [TownCode; 14] = [
    TownCode {
        name: "香兰镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "先锋社区",
                code: "001",
            },
            VillageCode {
                name: "兴旺社区",
                code: "002",
            },
            VillageCode {
                name: "香兰村委会",
                code: "003",
            },
            VillageCode {
                name: "红星村委会",
                code: "004",
            },
            VillageCode {
                name: "庆丰村委会",
                code: "005",
            },
            VillageCode {
                name: "双河村委会",
                code: "006",
            },
            VillageCode {
                name: "陶家村委会",
                code: "007",
            },
            VillageCode {
                name: "永久村委会",
                code: "008",
            },
            VillageCode {
                name: "红胜村委会",
                code: "009",
            },
            VillageCode {
                name: "大兴村委会",
                code: "010",
            },
            VillageCode {
                name: "大屯村委会",
                code: "011",
            },
            VillageCode {
                name: "大有村委会",
                code: "012",
            },
            VillageCode {
                name: "新立村委会",
                code: "013",
            },
            VillageCode {
                name: "兴安村委会",
                code: "014",
            },
            VillageCode {
                name: "曾波村委会",
                code: "015",
            },
            VillageCode {
                name: "共和村委会",
                code: "016",
            },
            VillageCode {
                name: "保安村委会",
                code: "017",
            },
            VillageCode {
                name: "双全村委会",
                code: "018",
            },
            VillageCode {
                name: "木良村委会",
                code: "019",
            },
            VillageCode {
                name: "新建村委会",
                code: "020",
            },
            VillageCode {
                name: "庆东村委会",
                code: "021",
            },
            VillageCode {
                name: "汤原木良林场生活区",
                code: "022",
            },
            VillageCode {
                name: "香兰水稻良种场生活区",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "鹤立镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "东鲜村委会",
                code: "001",
            },
            VillageCode {
                name: "新安村委会",
                code: "002",
            },
            VillageCode {
                name: "忠诚村委会",
                code: "003",
            },
            VillageCode {
                name: "和平村委会",
                code: "004",
            },
            VillageCode {
                name: "团结村委会",
                code: "005",
            },
            VillageCode {
                name: "继东村委会",
                code: "006",
            },
            VillageCode {
                name: "建平村委会",
                code: "007",
            },
            VillageCode {
                name: "太平村委会",
                code: "008",
            },
            VillageCode {
                name: "强盛村委会",
                code: "009",
            },
            VillageCode {
                name: "民盛村委会",
                code: "010",
            },
            VillageCode {
                name: "北盛村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "竹帘镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "竹兴社区",
                code: "001",
            },
            VillageCode {
                name: "茨梅村委会",
                code: "002",
            },
            VillageCode {
                name: "新发村委会",
                code: "003",
            },
            VillageCode {
                name: "民主村委会",
                code: "004",
            },
            VillageCode {
                name: "兴顺村委会",
                code: "005",
            },
            VillageCode {
                name: "保全村委会",
                code: "006",
            },
            VillageCode {
                name: "永全村委会",
                code: "007",
            },
            VillageCode {
                name: "吉利村委会",
                code: "008",
            },
            VillageCode {
                name: "龙江村委会",
                code: "009",
            },
            VillageCode {
                name: "兰田村委会",
                code: "010",
            },
            VillageCode {
                name: "竹帘村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "汤原镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "永胜社区",
                code: "001",
            },
            VillageCode {
                name: "建设社区",
                code: "002",
            },
            VillageCode {
                name: "振兴社区",
                code: "003",
            },
            VillageCode {
                name: "中华社区",
                code: "004",
            },
            VillageCode {
                name: "环城社区",
                code: "005",
            },
            VillageCode {
                name: "友谊社区",
                code: "006",
            },
            VillageCode {
                name: "哈肇社区",
                code: "007",
            },
            VillageCode {
                name: "林业社区",
                code: "008",
            },
            VillageCode {
                name: "华胜社区",
                code: "009",
            },
            VillageCode {
                name: "得胜村委会",
                code: "010",
            },
            VillageCode {
                name: "兴华村委会",
                code: "011",
            },
            VillageCode {
                name: "西凤鸣村委会",
                code: "012",
            },
            VillageCode {
                name: "东凤鸣村委会",
                code: "013",
            },
            VillageCode {
                name: "东庆升村委会",
                code: "014",
            },
            VillageCode {
                name: "东江村委会",
                code: "015",
            },
            VillageCode {
                name: "向阳村委会",
                code: "016",
            },
            VillageCode {
                name: "东大桥村委会",
                code: "017",
            },
            VillageCode {
                name: "北靠山村委会",
                code: "018",
            },
            VillageCode {
                name: "正阳村委会",
                code: "019",
            },
            VillageCode {
                name: "福民村委会",
                code: "020",
            },
            VillageCode {
                name: "合作村委会",
                code: "021",
            },
            VillageCode {
                name: "石场村委会",
                code: "022",
            },
            VillageCode {
                name: "长青村委会",
                code: "023",
            },
            VillageCode {
                name: "南向阳村委会",
                code: "024",
            },
            VillageCode {
                name: "新胜村委会",
                code: "025",
            },
            VillageCode {
                name: "红民村委会",
                code: "026",
            },
            VillageCode {
                name: "宝山村委会",
                code: "027",
            },
            VillageCode {
                name: "宝和村委会",
                code: "028",
            },
            VillageCode {
                name: "解放村委会",
                code: "029",
            },
            VillageCode {
                name: "西大桥村委会",
                code: "030",
            },
            VillageCode {
                name: "北向阳村委会",
                code: "031",
            },
            VillageCode {
                name: "仙马村委会",
                code: "032",
            },
            VillageCode {
                name: "汤原县石场沟林场生活区",
                code: "033",
            },
            VillageCode {
                name: "汤原正阳林场生活区",
                code: "034",
            },
            VillageCode {
                name: "汤原县种猪场生活区",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "汤旺乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "金星村委会",
                code: "001",
            },
            VillageCode {
                name: "红旗村委会",
                code: "002",
            },
            VillageCode {
                name: "太阳村委会",
                code: "003",
            },
            VillageCode {
                name: "东光村委会",
                code: "004",
            },
            VillageCode {
                name: "东升村委会",
                code: "005",
            },
            VillageCode {
                name: "曙光村委会",
                code: "006",
            },
            VillageCode {
                name: "红光村委会",
                code: "007",
            },
            VillageCode {
                name: "五星村委会",
                code: "008",
            },
            VillageCode {
                name: "金光村委会",
                code: "009",
            },
            VillageCode {
                name: "裕红村委会",
                code: "010",
            },
            VillageCode {
                name: "星光村委会",
                code: "011",
            },
            VillageCode {
                name: "火星村委会",
                code: "012",
            },
            VillageCode {
                name: "民生村委会",
                code: "013",
            },
            VillageCode {
                name: "永远村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "胜利乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "胜利村委会",
                code: "001",
            },
            VillageCode {
                name: "合力村委会",
                code: "002",
            },
            VillageCode {
                name: "伏胜村委会",
                code: "003",
            },
            VillageCode {
                name: "伏安村委会",
                code: "004",
            },
            VillageCode {
                name: "伏兴村委会",
                code: "005",
            },
            VillageCode {
                name: "福隆村委会",
                code: "006",
            },
            VillageCode {
                name: "连胜村委会",
                code: "007",
            },
            VillageCode {
                name: "荣丰村委会",
                code: "008",
            },
            VillageCode {
                name: "阳光村委会",
                code: "009",
            },
            VillageCode {
                name: "吉城村委会",
                code: "010",
            },
            VillageCode {
                name: "居德村委会",
                code: "011",
            },
            VillageCode {
                name: "新丰村委会",
                code: "012",
            },
            VillageCode {
                name: "荣丰鱼种场生活区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "吉祥乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "吉祥村委会",
                code: "001",
            },
            VillageCode {
                name: "华胜村委会",
                code: "002",
            },
            VillageCode {
                name: "保祥村委会",
                code: "003",
            },
            VillageCode {
                name: "保安村委会",
                code: "004",
            },
            VillageCode {
                name: "华丰村委会",
                code: "005",
            },
            VillageCode {
                name: "双丰村委会",
                code: "006",
            },
            VillageCode {
                name: "守望村委会",
                code: "007",
            },
            VillageCode {
                name: "互助村委会",
                code: "008",
            },
            VillageCode {
                name: "德祥村委会",
                code: "009",
            },
            VillageCode {
                name: "黄花村委会",
                code: "010",
            },
            VillageCode {
                name: "丰祥村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "振兴乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "振兴村委会",
                code: "001",
            },
            VillageCode {
                name: "兴化村委会",
                code: "002",
            },
            VillageCode {
                name: "古城村委会",
                code: "003",
            },
            VillageCode {
                name: "平原村委会",
                code: "004",
            },
            VillageCode {
                name: "民主村委会",
                code: "005",
            },
            VillageCode {
                name: "双兴村委会",
                code: "006",
            },
            VillageCode {
                name: "振江村委会",
                code: "007",
            },
            VillageCode {
                name: "振丰村委会",
                code: "008",
            },
            VillageCode {
                name: "丰兴村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "太平川乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "黑金河村委会",
                code: "001",
            },
            VillageCode {
                name: "太华村委会",
                code: "002",
            },
            VillageCode {
                name: "太安村委会",
                code: "003",
            },
            VillageCode {
                name: "开发村委会",
                code: "004",
            },
            VillageCode {
                name: "庆兴村委会",
                code: "005",
            },
            VillageCode {
                name: "北兴村委会",
                code: "006",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "007",
            },
            VillageCode {
                name: "旭日村委会",
                code: "008",
            },
            VillageCode {
                name: "金川村委会",
                code: "009",
            },
            VillageCode {
                name: "义兴村委会",
                code: "010",
            },
            VillageCode {
                name: "荣春村委会",
                code: "011",
            },
            VillageCode {
                name: "江兴村委会",
                code: "012",
            },
            VillageCode {
                name: "卫星村委会",
                code: "013",
            },
            VillageCode {
                name: "新兴村委会",
                code: "014",
            },
            VillageCode {
                name: "竹青村委会",
                code: "015",
            },
            VillageCode {
                name: "汤原县亮子河林场生活区",
                code: "016",
            },
            VillageCode {
                name: "汤原县黑金河林场生活区",
                code: "017",
            },
            VillageCode {
                name: "汤原团结林场生活区",
                code: "018",
            },
            VillageCode {
                name: "汤原腰营林场生活区",
                code: "019",
            },
            VillageCode {
                name: "汤原东风林场生活区",
                code: "020",
            },
            VillageCode {
                name: "果树示范场生活区",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "永发乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "红丰村委会",
                code: "001",
            },
            VillageCode {
                name: "红庆村委会",
                code: "002",
            },
            VillageCode {
                name: "红泉村委会",
                code: "003",
            },
            VillageCode {
                name: "红卫村委会",
                code: "004",
            },
            VillageCode {
                name: "永发村委会",
                code: "005",
            },
            VillageCode {
                name: "河发村委会",
                code: "006",
            },
            VillageCode {
                name: "跃进村委会",
                code: "007",
            },
            VillageCode {
                name: "裕德村委会",
                code: "008",
            },
            VillageCode {
                name: "加兴村委会",
                code: "009",
            },
            VillageCode {
                name: "前进村委会",
                code: "010",
            },
            VillageCode {
                name: "北华村委会",
                code: "011",
            },
            VillageCode {
                name: "裕新村委会",
                code: "012",
            },
            VillageCode {
                name: "宏图村委会",
                code: "013",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "014",
            },
            VillageCode {
                name: "南华村委会",
                code: "015",
            },
            VillageCode {
                name: "国营良种场生活区",
                code: "016",
            },
            VillageCode {
                name: "国营汤原县种畜场生活区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "鹤立林业局",
        code: "011",
        villages: &[
            VillageCode {
                name: "鹤立林业局山下第一社区",
                code: "001",
            },
            VillageCode {
                name: "鹤立林业局山下第二社区",
                code: "002",
            },
            VillageCode {
                name: "经营所社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "香兰监狱",
        code: "012",
        villages: &[
            VillageCode {
                name: "狱直社区",
                code: "001",
            },
            VillageCode {
                name: "监区大队生活区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "汤原农场",
        code: "013",
        villages: &[
            VillageCode {
                name: "汤原场直社区",
                code: "001",
            },
            VillageCode {
                name: "汤原农场祥和管理区",
                code: "002",
            },
            VillageCode {
                name: "汤原农场如意管理区",
                code: "003",
            },
            VillageCode {
                name: "汤原农场安康管理区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "梧桐河农场",
        code: "014",
        villages: &[
            VillageCode {
                name: "梧桐河场直社区",
                code: "001",
            },
            VillageCode {
                name: "梧桐河农场梧华管理区",
                code: "002",
            },
            VillageCode {
                name: "梧桐河农场丽水管理区",
                code: "003",
            },
            VillageCode {
                name: "梧桐河农场松东管理区",
                code: "004",
            },
            VillageCode {
                name: "梧桐河农场老亮台管理区",
                code: "005",
            },
            VillageCode {
                name: "梧桐河农场兴杉管理区",
                code: "006",
            },
            VillageCode {
                name: "梧桐河农场北片泡管理区",
                code: "007",
            },
        ],
    },
];

static TOWNS_HJ_025: [TownCode; 23] = [
    TownCode {
        name: "繁荣街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "长发社区居委会",
                code: "001",
            },
            VillageCode {
                name: "长安社区居委会",
                code: "002",
            },
            VillageCode {
                name: "东方社区居委会",
                code: "003",
            },
            VillageCode {
                name: "群利社区居委会",
                code: "004",
            },
            VillageCode {
                name: "富民社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "兴华街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "文采社区居委会",
                code: "001",
            },
            VillageCode {
                name: "繁华社区居委会",
                code: "002",
            },
            VillageCode {
                name: "和平社区居委会",
                code: "003",
            },
            VillageCode {
                name: "兴旺社区居委会",
                code: "004",
            },
            VillageCode {
                name: "永庆社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "同江镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "新光村委会",
                code: "001",
            },
            VillageCode {
                name: "新发村委会",
                code: "002",
            },
            VillageCode {
                name: "永胜村委会",
                code: "003",
            },
            VillageCode {
                name: "胜利村委会",
                code: "004",
            },
            VillageCode {
                name: "新街村委会",
                code: "005",
            },
            VillageCode {
                name: "新乐村委会",
                code: "006",
            },
            VillageCode {
                name: "新中村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "乐业镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "乐业村委会",
                code: "001",
            },
            VillageCode {
                name: "东风村委会",
                code: "002",
            },
            VillageCode {
                name: "庆明村委会",
                code: "003",
            },
            VillageCode {
                name: "安卫村委会",
                code: "004",
            },
            VillageCode {
                name: "一庄村委会",
                code: "005",
            },
            VillageCode {
                name: "团发村委会",
                code: "006",
            },
            VillageCode {
                name: "前锋村委会",
                code: "007",
            },
            VillageCode {
                name: "曙平村委会",
                code: "008",
            },
            VillageCode {
                name: "东方红村委会",
                code: "009",
            },
            VillageCode {
                name: "青年庄村委会",
                code: "010",
            },
            VillageCode {
                name: "东胜村委会",
                code: "011",
            },
            VillageCode {
                name: "同胜村委会",
                code: "012",
            },
            VillageCode {
                name: "盛昌村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "三村镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "头村委会",
                code: "001",
            },
            VillageCode {
                name: "二村委会",
                code: "002",
            },
            VillageCode {
                name: "三村委会",
                code: "003",
            },
            VillageCode {
                name: "四村委会",
                code: "004",
            },
            VillageCode {
                name: "红卫村委会",
                code: "005",
            },
            VillageCode {
                name: "新富村委会",
                code: "006",
            },
            VillageCode {
                name: "庆安村委会",
                code: "007",
            },
            VillageCode {
                name: "拉起河村委会",
                code: "008",
            },
            VillageCode {
                name: "华星村委会",
                code: "009",
            },
            VillageCode {
                name: "红建村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "临江镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "临江村委会",
                code: "001",
            },
            VillageCode {
                name: "富江村委会",
                code: "002",
            },
            VillageCode {
                name: "富强村委会",
                code: "003",
            },
            VillageCode {
                name: "富有村委会",
                code: "004",
            },
            VillageCode {
                name: "富国村委会",
                code: "005",
            },
            VillageCode {
                name: "富民村委会",
                code: "006",
            },
            VillageCode {
                name: "富裕村委会",
                code: "007",
            },
            VillageCode {
                name: "富川村委会",
                code: "008",
            },
            VillageCode {
                name: "合兴村委会",
                code: "009",
            },
            VillageCode {
                name: "春华村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "向阳镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "向阳村委会",
                code: "001",
            },
            VillageCode {
                name: "奋斗村委会",
                code: "002",
            },
            VillageCode {
                name: "红旗村委会",
                code: "003",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "004",
            },
            VillageCode {
                name: "东升村委会",
                code: "005",
            },
            VillageCode {
                name: "燎原村委会",
                code: "006",
            },
            VillageCode {
                name: "新兴村委会",
                code: "007",
            },
            VillageCode {
                name: "同富村委会",
                code: "008",
            },
            VillageCode {
                name: "同兴村委会",
                code: "009",
            },
            VillageCode {
                name: "黎明村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "青河镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "东宏村委会",
                code: "001",
            },
            VillageCode {
                name: "红星村委会",
                code: "002",
            },
            VillageCode {
                name: "东明村委会",
                code: "003",
            },
            VillageCode {
                name: "东强村委会",
                code: "004",
            },
            VillageCode {
                name: "东平村委会",
                code: "005",
            },
            VillageCode {
                name: "东原村委会",
                code: "006",
            },
            VillageCode {
                name: "东利村委会",
                code: "007",
            },
            VillageCode {
                name: "东阳村委会",
                code: "008",
            },
            VillageCode {
                name: "永利村委会",
                code: "009",
            },
            VillageCode {
                name: "永丰村委会",
                code: "010",
            },
            VillageCode {
                name: "永恒村委会",
                code: "011",
            },
            VillageCode {
                name: "永存村委会",
                code: "012",
            },
            VillageCode {
                name: "永祥村委会",
                code: "013",
            },
            VillageCode {
                name: "永安村委会",
                code: "014",
            },
            VillageCode {
                name: "永发村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "街津口乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "渔业村村委会",
                code: "001",
            },
            VillageCode {
                name: "卫明村委会",
                code: "002",
            },
            VillageCode {
                name: "卫国村委会",
                code: "003",
            },
            VillageCode {
                name: "卫华村委会",
                code: "004",
            },
            VillageCode {
                name: "卫星村委会",
                code: "005",
            },
            VillageCode {
                name: "卫垦村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "八岔乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "八岔村委会",
                code: "001",
            },
            VillageCode {
                name: "新胜村委会",
                code: "002",
            },
            VillageCode {
                name: "新颜村委会",
                code: "003",
            },
            VillageCode {
                name: "新强村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "金川乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "金江村委会",
                code: "001",
            },
            VillageCode {
                name: "金山村委会",
                code: "002",
            },
            VillageCode {
                name: "金华村委会",
                code: "003",
            },
            VillageCode {
                name: "金河村委会",
                code: "004",
            },
            VillageCode {
                name: "金珠村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "银川乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "银川村委会",
                code: "001",
            },
            VillageCode {
                name: "新民村委会",
                code: "002",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "003",
            },
            VillageCode {
                name: "银河村委会",
                code: "004",
            },
            VillageCode {
                name: "永华村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "街津口林场",
        code: "013",
        villages: &[VillageCode {
            name: "街津口林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "鸭北林场",
        code: "014",
        villages: &[VillageCode {
            name: "鸭北林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "勤得利农场",
        code: "015",
        villages: &[
            VillageCode {
                name: "勤得利第一居委会",
                code: "001",
            },
            VillageCode {
                name: "勤得利第二居委会",
                code: "002",
            },
            VillageCode {
                name: "勤得利第三居委会",
                code: "003",
            },
            VillageCode {
                name: "勤得利第四居委会",
                code: "004",
            },
            VillageCode {
                name: "勤得利第五居委会",
                code: "005",
            },
            VillageCode {
                name: "勤得利第六居委会",
                code: "006",
            },
            VillageCode {
                name: "勤得利第七居委会",
                code: "007",
            },
            VillageCode {
                name: "勤得利农场第一管理区",
                code: "008",
            },
            VillageCode {
                name: "勤得利农场第二管理区",
                code: "009",
            },
            VillageCode {
                name: "勤得利农场第三管理区",
                code: "010",
            },
            VillageCode {
                name: "勤得利农场第四管理区",
                code: "011",
            },
            VillageCode {
                name: "勤得利农场第五管理区",
                code: "012",
            },
            VillageCode {
                name: "勤得利农场第六管理区",
                code: "013",
            },
            VillageCode {
                name: "勤得利农场第七管理区",
                code: "014",
            },
            VillageCode {
                name: "勤得利农场第八管理区",
                code: "015",
            },
            VillageCode {
                name: "勤得利农场第九管理区",
                code: "016",
            },
            VillageCode {
                name: "勤得利农场第十管理区",
                code: "017",
            },
            VillageCode {
                name: "勤得利农场第十一管理区",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "青龙山农场",
        code: "016",
        villages: &[
            VillageCode {
                name: "青龙山第一居委会",
                code: "001",
            },
            VillageCode {
                name: "青龙山第二居委会",
                code: "002",
            },
            VillageCode {
                name: "青龙山第三居委会",
                code: "003",
            },
            VillageCode {
                name: "青龙山第四居委会",
                code: "004",
            },
            VillageCode {
                name: "青龙山农场第一管理区",
                code: "005",
            },
            VillageCode {
                name: "青龙山农场第二管理区",
                code: "006",
            },
            VillageCode {
                name: "青龙山农场第三管理区",
                code: "007",
            },
            VillageCode {
                name: "青龙山农场第四管理区",
                code: "008",
            },
            VillageCode {
                name: "青龙山农场第五管理区",
                code: "009",
            },
            VillageCode {
                name: "青龙山农场第六管理区",
                code: "010",
            },
            VillageCode {
                name: "青龙山农场第七管理区",
                code: "011",
            },
            VillageCode {
                name: "青龙山农场第八管理区",
                code: "012",
            },
            VillageCode {
                name: "青龙山农场第九管理区",
                code: "013",
            },
            VillageCode {
                name: "青龙山农场第十管理区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "前进农场",
        code: "017",
        villages: &[
            VillageCode {
                name: "前进第一居委会",
                code: "001",
            },
            VillageCode {
                name: "前进第二居委会",
                code: "002",
            },
            VillageCode {
                name: "前进第三居委会",
                code: "003",
            },
            VillageCode {
                name: "前进第四居委会",
                code: "004",
            },
            VillageCode {
                name: "前进第五居委会",
                code: "005",
            },
            VillageCode {
                name: "前进第六居委会",
                code: "006",
            },
            VillageCode {
                name: "前进农场第一管理区",
                code: "007",
            },
            VillageCode {
                name: "前进农场第二管理区",
                code: "008",
            },
            VillageCode {
                name: "前进农场第三管理区",
                code: "009",
            },
            VillageCode {
                name: "前进农场第四管理区",
                code: "010",
            },
            VillageCode {
                name: "前进农场第五管理区",
                code: "011",
            },
            VillageCode {
                name: "前进农场第六管理区",
                code: "012",
            },
            VillageCode {
                name: "前进农场第七管理区",
                code: "013",
            },
            VillageCode {
                name: "前进农场第八管理区",
                code: "014",
            },
            VillageCode {
                name: "前进农场第九管理区",
                code: "015",
            },
            VillageCode {
                name: "前进农场第十管理区",
                code: "016",
            },
            VillageCode {
                name: "前进农场第十一管理区",
                code: "017",
            },
            VillageCode {
                name: "前进农场第十二管理区",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "洪河农场",
        code: "018",
        villages: &[
            VillageCode {
                name: "洪河第一居委会",
                code: "001",
            },
            VillageCode {
                name: "洪河第二居委会",
                code: "002",
            },
            VillageCode {
                name: "洪河第三居委会",
                code: "003",
            },
            VillageCode {
                name: "洪河农场第一管理区",
                code: "004",
            },
            VillageCode {
                name: "洪河农场第二管理区",
                code: "005",
            },
            VillageCode {
                name: "洪河农场第三管理区",
                code: "006",
            },
            VillageCode {
                name: "洪河农场第四管理区",
                code: "007",
            },
            VillageCode {
                name: "洪河农场第五管理区",
                code: "008",
            },
            VillageCode {
                name: "洪河农场第六管理区",
                code: "009",
            },
            VillageCode {
                name: "洪河农场第七管理区",
                code: "010",
            },
            VillageCode {
                name: "洪河农场第八管理区",
                code: "011",
            },
            VillageCode {
                name: "洪河农场第九管理区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "鸭绿河农场",
        code: "019",
        villages: &[
            VillageCode {
                name: "鸭绿河第一居委会",
                code: "001",
            },
            VillageCode {
                name: "鸭绿河第二居委会",
                code: "002",
            },
            VillageCode {
                name: "鸭绿河农场第一管理区",
                code: "003",
            },
            VillageCode {
                name: "鸭绿河农场第二管理区",
                code: "004",
            },
            VillageCode {
                name: "鸭绿河农场第三管理区",
                code: "005",
            },
            VillageCode {
                name: "鸭绿河农场第四管理区",
                code: "006",
            },
            VillageCode {
                name: "鸭绿河农场第五管理区",
                code: "007",
            },
            VillageCode {
                name: "鸭绿河农场第六管理区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "浓江农场",
        code: "020",
        villages: &[
            VillageCode {
                name: "浓江第一居委会",
                code: "001",
            },
            VillageCode {
                name: "浓江第二居委会",
                code: "002",
            },
            VillageCode {
                name: "浓江第三居委会",
                code: "003",
            },
            VillageCode {
                name: "浓江农场第一管理区",
                code: "004",
            },
            VillageCode {
                name: "浓江农场第二管理区",
                code: "005",
            },
            VillageCode {
                name: "浓江农场第三管理区",
                code: "006",
            },
            VillageCode {
                name: "浓江农场第四管理区",
                code: "007",
            },
            VillageCode {
                name: "浓江农场第五管理区",
                code: "008",
            },
            VillageCode {
                name: "浓江农场第六管理区",
                code: "009",
            },
            VillageCode {
                name: "浓江农场第七管理区",
                code: "010",
            },
            VillageCode {
                name: "浓江农场第八管理区",
                code: "011",
            },
            VillageCode {
                name: "浓江农场第九管理区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "良种场",
        code: "021",
        villages: &[VillageCode {
            name: "良种场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "畜牧场",
        code: "022",
        villages: &[VillageCode {
            name: "畜牧场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "知青农场",
        code: "023",
        villages: &[VillageCode {
            name: "知青农场虚拟生活区",
            code: "001",
        }],
    },
];

static TOWNS_HJ_026: [TownCode; 27] = [
    TownCode {
        name: "城东街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "福前社区第一居民委员会",
                code: "001",
            },
            VillageCode {
                name: "福前社区第二居民委员会",
                code: "002",
            },
            VillageCode {
                name: "福前社区第三居民委员会",
                code: "003",
            },
            VillageCode {
                name: "福前社区第四居民委员会",
                code: "004",
            },
            VillageCode {
                name: "东平社区第五居民委员会",
                code: "005",
            },
            VillageCode {
                name: "东平社区第六居民委员会",
                code: "006",
            },
            VillageCode {
                name: "东平社区第七居民委员会",
                code: "007",
            },
            VillageCode {
                name: "东平社区第八居民委员会",
                code: "008",
            },
            VillageCode {
                name: "南岗社区第九居民委员会",
                code: "009",
            },
            VillageCode {
                name: "南岗社区第十居民委员会",
                code: "010",
            },
            VillageCode {
                name: "南岗社区第十一居民委员会",
                code: "011",
            },
            VillageCode {
                name: "南岗社区第十二居民委员会",
                code: "012",
            },
            VillageCode {
                name: "南岗社区第十三居民委员会",
                code: "013",
            },
            VillageCode {
                name: "文化社区第二十二居民委员会",
                code: "014",
            },
            VillageCode {
                name: "文化社区第二十三居民委员会",
                code: "015",
            },
            VillageCode {
                name: "文化社区第二十四居民委员会",
                code: "016",
            },
            VillageCode {
                name: "文化社区第二十五居民委员会",
                code: "017",
            },
            VillageCode {
                name: "朝阳社区第二十六居民委员会",
                code: "018",
            },
            VillageCode {
                name: "朝阳社区第二十七居民委员会",
                code: "019",
            },
            VillageCode {
                name: "朝阳社区第二十八居民委员会",
                code: "020",
            },
            VillageCode {
                name: "朝阳社区第二十九居民委员会",
                code: "021",
            },
            VillageCode {
                name: "向阳社区第三十居民委员会",
                code: "022",
            },
            VillageCode {
                name: "向阳社区第三十一居民委员会",
                code: "023",
            },
            VillageCode {
                name: "向阳社区第三十二居民委员会",
                code: "024",
            },
            VillageCode {
                name: "向阳社区第三十三居民委员会",
                code: "025",
            },
            VillageCode {
                name: "向阳社区第三十四居民委员会",
                code: "026",
            },
            VillageCode {
                name: "繁荣社区第三十五居民委员会",
                code: "027",
            },
            VillageCode {
                name: "繁荣社区第三十六居民委员会",
                code: "028",
            },
            VillageCode {
                name: "繁荣社区第三十七居民委员会",
                code: "029",
            },
            VillageCode {
                name: "繁荣社区第三十八居民委员会",
                code: "030",
            },
            VillageCode {
                name: "向阳社区第五十一居民委员会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "城西街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "民主社区第十四居民委员会",
                code: "001",
            },
            VillageCode {
                name: "民主社区第十五居民委员会",
                code: "002",
            },
            VillageCode {
                name: "民主社区第十六居民委员会",
                code: "003",
            },
            VillageCode {
                name: "民主社区第十七居民委员会",
                code: "004",
            },
            VillageCode {
                name: "民主社区第十八居民委员会",
                code: "005",
            },
            VillageCode {
                name: "幸福社区第十九居民委员会",
                code: "006",
            },
            VillageCode {
                name: "幸福社区第二十居民委员会",
                code: "007",
            },
            VillageCode {
                name: "幸福社区第二十一居民委员会",
                code: "008",
            },
            VillageCode {
                name: "临江社区第三十九居民委员会",
                code: "009",
            },
            VillageCode {
                name: "临江社区第四十居民委员会",
                code: "010",
            },
            VillageCode {
                name: "临江社区第四十一居民委员会",
                code: "011",
            },
            VillageCode {
                name: "临江社区第四十二居民委员会",
                code: "012",
            },
            VillageCode {
                name: "新开社区第四十三居民委员会",
                code: "013",
            },
            VillageCode {
                name: "新开社区第四十四居民委员会",
                code: "014",
            },
            VillageCode {
                name: "新开社区第四十五居民委员会",
                code: "015",
            },
            VillageCode {
                name: "西平社区第四十六居民委员会",
                code: "016",
            },
            VillageCode {
                name: "西平社区第四十七居民委员会",
                code: "017",
            },
            VillageCode {
                name: "建设社区第四十八居民委员会",
                code: "018",
            },
            VillageCode {
                name: "建设社区第四十九居民委员会",
                code: "019",
            },
            VillageCode {
                name: "建设社区第五十居民委员会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "富锦镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "红光村委会",
                code: "001",
            },
            VillageCode {
                name: "红星村委会",
                code: "002",
            },
            VillageCode {
                name: "锦兴村委会",
                code: "003",
            },
            VillageCode {
                name: "富华村委会",
                code: "004",
            },
            VillageCode {
                name: "东郊村委会",
                code: "005",
            },
            VillageCode {
                name: "西郊村委会",
                code: "006",
            },
            VillageCode {
                name: "临城村委会",
                code: "007",
            },
            VillageCode {
                name: "林场村委会",
                code: "008",
            },
            VillageCode {
                name: "嘎尔当村委会",
                code: "009",
            },
            VillageCode {
                name: "兴农村委会",
                code: "010",
            },
            VillageCode {
                name: "上街基村委会",
                code: "011",
            },
            VillageCode {
                name: "城东村委会",
                code: "012",
            },
            VillageCode {
                name: "一砖村村委会",
                code: "013",
            },
            VillageCode {
                name: "二砖村村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "长安镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "长安村委会",
                code: "001",
            },
            VillageCode {
                name: "聚贤村委会",
                code: "002",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "003",
            },
            VillageCode {
                name: "新华村委会",
                code: "004",
            },
            VillageCode {
                name: "大安村委会",
                code: "005",
            },
            VillageCode {
                name: "殿文村委会",
                code: "006",
            },
            VillageCode {
                name: "兴本村委会",
                code: "007",
            },
            VillageCode {
                name: "民安村委会",
                code: "008",
            },
            VillageCode {
                name: "新立村委会",
                code: "009",
            },
            VillageCode {
                name: "东北村委会",
                code: "010",
            },
            VillageCode {
                name: "东日新村委会",
                code: "011",
            },
            VillageCode {
                name: "西日新村委会",
                code: "012",
            },
            VillageCode {
                name: "长胜村委会",
                code: "013",
            },
            VillageCode {
                name: "务本村委会",
                code: "014",
            },
            VillageCode {
                name: "永胜村委会",
                code: "015",
            },
            VillageCode {
                name: "德胜村委会",
                code: "016",
            },
            VillageCode {
                name: "漂筏村委会",
                code: "017",
            },
            VillageCode {
                name: "高家村委会",
                code: "018",
            },
            VillageCode {
                name: "长富农场村委会",
                code: "019",
            },
            VillageCode {
                name: "太安村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "砚山镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "恒山村委会",
                code: "001",
            },
            VillageCode {
                name: "东五顶村委会",
                code: "002",
            },
            VillageCode {
                name: "新光村委会",
                code: "003",
            },
            VillageCode {
                name: "东安村委会",
                code: "004",
            },
            VillageCode {
                name: "永发村委会",
                code: "005",
            },
            VillageCode {
                name: "巨福村委会",
                code: "006",
            },
            VillageCode {
                name: "爱国村委会",
                code: "007",
            },
            VillageCode {
                name: "西五顶村委会",
                code: "008",
            },
            VillageCode {
                name: "联合村委会",
                code: "009",
            },
            VillageCode {
                name: "富山村委会",
                code: "010",
            },
            VillageCode {
                name: "平安村委会",
                code: "011",
            },
            VillageCode {
                name: "翻身村委会",
                code: "012",
            },
            VillageCode {
                name: "双发村委会",
                code: "013",
            },
            VillageCode {
                name: "砚山村委会",
                code: "014",
            },
            VillageCode {
                name: "东瑞村委会",
                code: "015",
            },
            VillageCode {
                name: "正阳村委会",
                code: "016",
            },
            VillageCode {
                name: "连生村委会",
                code: "017",
            },
            VillageCode {
                name: "保安村委会",
                code: "018",
            },
            VillageCode {
                name: "长兴村委会",
                code: "019",
            },
            VillageCode {
                name: "福祥村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "头林镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "头林村委会",
                code: "001",
            },
            VillageCode {
                name: "二林村委会",
                code: "002",
            },
            VillageCode {
                name: "庆合村委会",
                code: "003",
            },
            VillageCode {
                name: "兴林村委会",
                code: "004",
            },
            VillageCode {
                name: "建华村委会",
                code: "005",
            },
            VillageCode {
                name: "结合村委会",
                code: "006",
            },
            VillageCode {
                name: "西林村委会",
                code: "007",
            },
            VillageCode {
                name: "双林村委会",
                code: "008",
            },
            VillageCode {
                name: "新胜村委会",
                code: "009",
            },
            VillageCode {
                name: "双丰村委会",
                code: "010",
            },
            VillageCode {
                name: "双福村委会",
                code: "011",
            },
            VillageCode {
                name: "东林村委会",
                code: "012",
            },
            VillageCode {
                name: "永丰村委会",
                code: "013",
            },
            VillageCode {
                name: "解放村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "兴隆岗镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "兴隆村委会",
                code: "001",
            },
            VillageCode {
                name: "河西村委会",
                code: "002",
            },
            VillageCode {
                name: "三林村委会",
                code: "003",
            },
            VillageCode {
                name: "永富村委会",
                code: "004",
            },
            VillageCode {
                name: "宏伟村委会",
                code: "005",
            },
            VillageCode {
                name: "幸福村委会",
                code: "006",
            },
            VillageCode {
                name: "福升村委会",
                code: "007",
            },
            VillageCode {
                name: "前富村委会",
                code: "008",
            },
            VillageCode {
                name: "兴胜村委会",
                code: "009",
            },
            VillageCode {
                name: "兴富村委会",
                code: "010",
            },
            VillageCode {
                name: "高台子村委会",
                code: "011",
            },
            VillageCode {
                name: "新林村委会",
                code: "012",
            },
            VillageCode {
                name: "新风村委会",
                code: "013",
            },
            VillageCode {
                name: "金林村委会",
                code: "014",
            },
            VillageCode {
                name: "振兴村委会",
                code: "015",
            },
            VillageCode {
                name: "东升村委会",
                code: "016",
            },
            VillageCode {
                name: "东胜村委会",
                code: "017",
            },
            VillageCode {
                name: "东明村委会",
                code: "018",
            },
            VillageCode {
                name: "东福村委会",
                code: "019",
            },
            VillageCode {
                name: "西岗村委会",
                code: "020",
            },
            VillageCode {
                name: "鹿林村委会",
                code: "021",
            },
            VillageCode {
                name: "兴会村委会",
                code: "022",
            },
            VillageCode {
                name: "东悦村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "宏胜镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "宏胜村委会",
                code: "001",
            },
            VillageCode {
                name: "兴东村委会",
                code: "002",
            },
            VillageCode {
                name: "兴家村委会",
                code: "003",
            },
            VillageCode {
                name: "宏林村委会",
                code: "004",
            },
            VillageCode {
                name: "北河村委会",
                code: "005",
            },
            VillageCode {
                name: "兴国村委会",
                code: "006",
            },
            VillageCode {
                name: "双山村委会",
                code: "007",
            },
            VillageCode {
                name: "同胜村委会",
                code: "008",
            },
            VillageCode {
                name: "久胜村委会",
                code: "009",
            },
            VillageCode {
                name: "东岗村委会",
                code: "010",
            },
            VillageCode {
                name: "隆胜村委会",
                code: "011",
            },
            VillageCode {
                name: "龙华村委会",
                code: "012",
            },
            VillageCode {
                name: "龙江村委会",
                code: "013",
            },
            VillageCode {
                name: "宏胜农场村委会",
                code: "014",
            },
            VillageCode {
                name: "南林村委会",
                code: "015",
            },
            VillageCode {
                name: "育林村委会",
                code: "016",
            },
            VillageCode {
                name: "红山村委会",
                code: "017",
            },
            VillageCode {
                name: "胜利村委会",
                code: "018",
            },
            VillageCode {
                name: "红旗村委会",
                code: "019",
            },
            VillageCode {
                name: "永平村委会",
                code: "020",
            },
            VillageCode {
                name: "永成村委会",
                code: "021",
            },
            VillageCode {
                name: "永林村委会",
                code: "022",
            },
            VillageCode {
                name: "永旺村委会",
                code: "023",
            },
            VillageCode {
                name: "明胜村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "向阳川镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "向阳川村委会",
                code: "001",
            },
            VillageCode {
                name: "丰太村委会",
                code: "002",
            },
            VillageCode {
                name: "福安村委会",
                code: "003",
            },
            VillageCode {
                name: "大兴村委会",
                code: "004",
            },
            VillageCode {
                name: "后山村委会",
                code: "005",
            },
            VillageCode {
                name: "泰山村委会",
                code: "006",
            },
            VillageCode {
                name: "正兴村委会",
                code: "007",
            },
            VillageCode {
                name: "中和村委会",
                code: "008",
            },
            VillageCode {
                name: "建和村委会",
                code: "009",
            },
            VillageCode {
                name: "正和村委会",
                code: "010",
            },
            VillageCode {
                name: "龙安村委会",
                code: "011",
            },
            VillageCode {
                name: "东兴村委会",
                code: "012",
            },
            VillageCode {
                name: "东来村委会",
                code: "013",
            },
            VillageCode {
                name: "择林村委会",
                code: "014",
            },
            VillageCode {
                name: "马鞍山村委会",
                code: "015",
            },
            VillageCode {
                name: "前进村委会",
                code: "016",
            },
            VillageCode {
                name: "福泉村委会",
                code: "017",
            },
            VillageCode {
                name: "六合村委会",
                code: "018",
            },
            VillageCode {
                name: "桂仁村委会",
                code: "019",
            },
            VillageCode {
                name: "友谊村委会",
                code: "020",
            },
            VillageCode {
                name: "永太村委会",
                code: "021",
            },
            VillageCode {
                name: "孟家岗村委会",
                code: "022",
            },
            VillageCode {
                name: "永福村委会",
                code: "023",
            },
            VillageCode {
                name: "长春岭村委会",
                code: "024",
            },
            VillageCode {
                name: "徐家店村委会",
                code: "025",
            },
            VillageCode {
                name: "连山村委会",
                code: "026",
            },
            VillageCode {
                name: "安洪村委会",
                code: "027",
            },
            VillageCode {
                name: "宝山村委会",
                code: "028",
            },
            VillageCode {
                name: "太和村委会",
                code: "029",
            },
            VillageCode {
                name: "龙富村委会",
                code: "030",
            },
            VillageCode {
                name: "东新民村委会",
                code: "031",
            },
            VillageCode {
                name: "福庆村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "二龙山镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "龙山村委会",
                code: "001",
            },
            VillageCode {
                name: "三胜村委会",
                code: "002",
            },
            VillageCode {
                name: "靠山村委会",
                code: "003",
            },
            VillageCode {
                name: "西凤阳村委会",
                code: "004",
            },
            VillageCode {
                name: "东凤阳村委会",
                code: "005",
            },
            VillageCode {
                name: "北山村委会",
                code: "006",
            },
            VillageCode {
                name: "康庄村委会",
                code: "007",
            },
            VillageCode {
                name: "龙阳村委会",
                code: "008",
            },
            VillageCode {
                name: "集民村委会",
                code: "009",
            },
            VillageCode {
                name: "永乐村委会",
                code: "010",
            },
            VillageCode {
                name: "庆平村委会",
                code: "011",
            },
            VillageCode {
                name: "新兴村委会",
                code: "012",
            },
            VillageCode {
                name: "太东村委会",
                code: "013",
            },
            VillageCode {
                name: "向前村委会",
                code: "014",
            },
            VillageCode {
                name: "春光村委会",
                code: "015",
            },
            VillageCode {
                name: "长发村委会",
                code: "016",
            },
            VillageCode {
                name: "永善村委会",
                code: "017",
            },
            VillageCode {
                name: "德林村委会",
                code: "018",
            },
            VillageCode {
                name: "双合村委会",
                code: "019",
            },
            VillageCode {
                name: "新安村委会",
                code: "020",
            },
            VillageCode {
                name: "莲花村委会",
                code: "021",
            },
            VillageCode {
                name: "共荣村委会",
                code: "022",
            },
            VillageCode {
                name: "永佳村委会",
                code: "023",
            },
            VillageCode {
                name: "新富村委会",
                code: "024",
            },
            VillageCode {
                name: "吉良村委会",
                code: "025",
            },
            VillageCode {
                name: "北地界村委会",
                code: "026",
            },
            VillageCode {
                name: "新卫村委会",
                code: "027",
            },
            VillageCode {
                name: "新龙村委会",
                code: "028",
            },
            VillageCode {
                name: "新宏村委会",
                code: "029",
            },
            VillageCode {
                name: "新民村委会",
                code: "030",
            },
            VillageCode {
                name: "新合村委会",
                code: "031",
            },
            VillageCode {
                name: "新桥村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "上街基镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "西安村委会",
                code: "001",
            },
            VillageCode {
                name: "诚信村委会",
                code: "002",
            },
            VillageCode {
                name: "宋店村委会",
                code: "003",
            },
            VillageCode {
                name: "林河村委会",
                code: "004",
            },
            VillageCode {
                name: "治安村委会",
                code: "005",
            },
            VillageCode {
                name: "福民村委会",
                code: "006",
            },
            VillageCode {
                name: "东立村委会",
                code: "007",
            },
            VillageCode {
                name: "永升村委会",
                code: "008",
            },
            VillageCode {
                name: "德福村委会",
                code: "009",
            },
            VillageCode {
                name: "万发村委会",
                code: "010",
            },
            VillageCode {
                name: "德安村委会",
                code: "011",
            },
            VillageCode {
                name: "振永村委会",
                code: "012",
            },
            VillageCode {
                name: "万宝村委会",
                code: "013",
            },
            VillageCode {
                name: "宏甸村委会",
                code: "014",
            },
            VillageCode {
                name: "和悦陆村委会",
                code: "015",
            },
            VillageCode {
                name: "鲜丰村委会",
                code: "016",
            },
            VillageCode {
                name: "清化村委会",
                code: "017",
            },
            VillageCode {
                name: "万有村委会",
                code: "018",
            },
            VillageCode {
                name: "三合村委会",
                code: "019",
            },
            VillageCode {
                name: "西福山村委会",
                code: "020",
            },
            VillageCode {
                name: "西富乡村委会",
                code: "021",
            },
            VillageCode {
                name: "东富乡村委会",
                code: "022",
            },
            VillageCode {
                name: "合发村委会",
                code: "023",
            },
            VillageCode {
                name: "四合村委会",
                code: "024",
            },
            VillageCode {
                name: "大户村委会",
                code: "025",
            },
            VillageCode {
                name: "明朗村委会",
                code: "026",
            },
            VillageCode {
                name: "希贤村委会",
                code: "027",
            },
            VillageCode {
                name: "天安村委会",
                code: "028",
            },
            VillageCode {
                name: "忠胜村委会",
                code: "029",
            },
            VillageCode {
                name: "大屯村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "锦山镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "锦山村委会",
                code: "001",
            },
            VillageCode {
                name: "南化村委会",
                code: "002",
            },
            VillageCode {
                name: "仁义村委会",
                code: "003",
            },
            VillageCode {
                name: "后贾村委会",
                code: "004",
            },
            VillageCode {
                name: "山北村委会",
                code: "005",
            },
            VillageCode {
                name: "王贵村委会",
                code: "006",
            },
            VillageCode {
                name: "德祥村委会",
                code: "007",
            },
            VillageCode {
                name: "永庆村委会",
                code: "008",
            },
            VillageCode {
                name: "重兴村委会",
                code: "009",
            },
            VillageCode {
                name: "二砖村委会",
                code: "010",
            },
            VillageCode {
                name: "黑鱼泡村委会",
                code: "011",
            },
            VillageCode {
                name: "富廷村委会",
                code: "012",
            },
            VillageCode {
                name: "近山村委会",
                code: "013",
            },
            VillageCode {
                name: "永阳村委会",
                code: "014",
            },
            VillageCode {
                name: "仁和村委会",
                code: "015",
            },
            VillageCode {
                name: "民利村委会",
                code: "016",
            },
            VillageCode {
                name: "兴利村委会",
                code: "017",
            },
            VillageCode {
                name: "跃进村委会",
                code: "018",
            },
            VillageCode {
                name: "二道村委会",
                code: "019",
            },
            VillageCode {
                name: "继承村委会",
                code: "020",
            },
            VillageCode {
                name: "信安村委会",
                code: "021",
            },
            VillageCode {
                name: "世一村委会",
                code: "022",
            },
            VillageCode {
                name: "建设村委会",
                code: "023",
            },
            VillageCode {
                name: "富国村委会",
                code: "024",
            },
            VillageCode {
                name: "强盛村委会",
                code: "025",
            },
            VillageCode {
                name: "公安村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "大榆树镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "大榆树村委会",
                code: "001",
            },
            VillageCode {
                name: "腰中村委会",
                code: "002",
            },
            VillageCode {
                name: "邵店村委会",
                code: "003",
            },
            VillageCode {
                name: "福合村委会",
                code: "004",
            },
            VillageCode {
                name: "拾房村委会",
                code: "005",
            },
            VillageCode {
                name: "保林村委会",
                code: "006",
            },
            VillageCode {
                name: "庆胜村委会",
                code: "007",
            },
            VillageCode {
                name: "兴达村委会",
                code: "008",
            },
            VillageCode {
                name: "健康村委会",
                code: "009",
            },
            VillageCode {
                name: "福来村委会",
                code: "010",
            },
            VillageCode {
                name: "华胜村委会",
                code: "011",
            },
            VillageCode {
                name: "长发岗村委会",
                code: "012",
            },
            VillageCode {
                name: "富民村委会",
                code: "013",
            },
            VillageCode {
                name: "沙岗村委会",
                code: "014",
            },
            VillageCode {
                name: "富士村委会",
                code: "015",
            },
            VillageCode {
                name: "永东村委会",
                code: "016",
            },
            VillageCode {
                name: "富林村委会",
                code: "017",
            },
            VillageCode {
                name: "富海村委会",
                code: "018",
            },
            VillageCode {
                name: "富珍村委会",
                code: "019",
            },
            VillageCode {
                name: "正东村委会",
                code: "020",
            },
            VillageCode {
                name: "福胜村委会",
                code: "021",
            },
            VillageCode {
                name: "七桥村委会",
                code: "022",
            },
            VillageCode {
                name: "新旭村委会",
                code: "023",
            },
            VillageCode {
                name: "隆川村委会",
                code: "024",
            },
            VillageCode {
                name: "隆兴村委会",
                code: "025",
            },
            VillageCode {
                name: "盛田村委会",
                code: "026",
            },
            VillageCode {
                name: "安山村委会",
                code: "027",
            },
            VillageCode {
                name: "金山村委会",
                code: "028",
            },
            VillageCode {
                name: "茂盛村委会",
                code: "029",
            },
            VillageCode {
                name: "吉祥村委会",
                code: "030",
            },
            VillageCode {
                name: "仁安村委会",
                code: "031",
            },
            VillageCode {
                name: "海沟村委会",
                code: "032",
            },
            VillageCode {
                name: "太平村委会",
                code: "033",
            },
            VillageCode {
                name: "向阳村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "石砬山林场",
        code: "014",
        villages: &[VillageCode {
            name: "石砬山林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "东风岗林场",
        code: "015",
        villages: &[VillageCode {
            name: "东风岗林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "太东林场",
        code: "016",
        villages: &[VillageCode {
            name: "太东林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "工农林场",
        code: "017",
        villages: &[VillageCode {
            name: "工农林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "富锦市国营原种场",
        code: "018",
        villages: &[VillageCode {
            name: "富锦市国营原种场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "富锦市国营果树示范场",
        code: "019",
        villages: &[VillageCode {
            name: "富锦市国营果树示范场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "富锦市科研所",
        code: "020",
        villages: &[VillageCode {
            name: "富锦市科研所虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "工业园区",
        code: "021",
        villages: &[VillageCode {
            name: "工业园区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "建三江管理局局直",
        code: "022",
        villages: &[
            VillageCode {
                name: "建三江管理局街道胜利社区",
                code: "001",
            },
            VillageCode {
                name: "建三江管理局街道欣园社区",
                code: "002",
            },
            VillageCode {
                name: "建三江管理局街道中央大街社区",
                code: "003",
            },
            VillageCode {
                name: "建三江管理局街道怡园社区",
                code: "004",
            },
            VillageCode {
                name: "建三江管理局街道鑫泽社区",
                code: "005",
            },
            VillageCode {
                name: "建三江管理局街道市场街社区",
                code: "006",
            },
            VillageCode {
                name: "建三江管理局街道星河社区",
                code: "007",
            },
            VillageCode {
                name: "建三江管理局街道学苑街社区",
                code: "008",
            },
            VillageCode {
                name: "建三江管理局街道宜和社区",
                code: "009",
            },
            VillageCode {
                name: "建三江管理局街道铁南社区",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "七星农场",
        code: "023",
        villages: &[
            VillageCode {
                name: "黑龙江省七星街道办事处第一居委会",
                code: "001",
            },
            VillageCode {
                name: "黑龙江省七星街道办事处第二居委会",
                code: "002",
            },
            VillageCode {
                name: "黑龙江省七星街道办事处第三居委会",
                code: "003",
            },
            VillageCode {
                name: "黑龙江省七星街道办事处第四居委会",
                code: "004",
            },
            VillageCode {
                name: "黑龙江省七星街道办事处第五居委会",
                code: "005",
            },
            VillageCode {
                name: "七星农场第一管理区",
                code: "006",
            },
            VillageCode {
                name: "七星农场第二管理区",
                code: "007",
            },
            VillageCode {
                name: "七星农场第三管理区",
                code: "008",
            },
            VillageCode {
                name: "七星农场第四管理区",
                code: "009",
            },
            VillageCode {
                name: "七星农场第五管理区",
                code: "010",
            },
            VillageCode {
                name: "七星农场第六管理区",
                code: "011",
            },
            VillageCode {
                name: "七星农场第七管理区",
                code: "012",
            },
            VillageCode {
                name: "七星农场第八管理区",
                code: "013",
            },
            VillageCode {
                name: "七星农场第九管理区",
                code: "014",
            },
            VillageCode {
                name: "七星农场第十管理区",
                code: "015",
            },
            VillageCode {
                name: "七星农场第十一管理区",
                code: "016",
            },
            VillageCode {
                name: "七星农场第十二管理区",
                code: "017",
            },
            VillageCode {
                name: "七星农场第十三管理区",
                code: "018",
            },
            VillageCode {
                name: "七星农场第十四管理区",
                code: "019",
            },
            VillageCode {
                name: "七星农场第十五管理区",
                code: "020",
            },
            VillageCode {
                name: "七星农场第十六管理区",
                code: "021",
            },
            VillageCode {
                name: "七星农场第十七管理区",
                code: "022",
            },
            VillageCode {
                name: "七星农场第十八管理区",
                code: "023",
            },
            VillageCode {
                name: "七星农场第十九管理区",
                code: "024",
            },
            VillageCode {
                name: "七星农场第二十管理区",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "大兴农场",
        code: "024",
        villages: &[
            VillageCode {
                name: "黑龙江省大兴街道办事处第一居委会",
                code: "001",
            },
            VillageCode {
                name: "黑龙江省大兴街道办事处第二居委会",
                code: "002",
            },
            VillageCode {
                name: "黑龙江省大兴街道办事处第三居委会",
                code: "003",
            },
            VillageCode {
                name: "黑龙江省大兴街道办事处第四居委会",
                code: "004",
            },
            VillageCode {
                name: "黑龙江省大兴街道办事处第五居委会",
                code: "005",
            },
            VillageCode {
                name: "大兴农场第一管理区",
                code: "006",
            },
            VillageCode {
                name: "大兴农场第二管理区",
                code: "007",
            },
            VillageCode {
                name: "大兴农场第三管理区",
                code: "008",
            },
            VillageCode {
                name: "大兴农场第四管理区",
                code: "009",
            },
            VillageCode {
                name: "大兴农场第五管理区",
                code: "010",
            },
            VillageCode {
                name: "大兴农场第六管理区",
                code: "011",
            },
            VillageCode {
                name: "大兴农场第七管理区",
                code: "012",
            },
            VillageCode {
                name: "大兴农场第八管理区",
                code: "013",
            },
            VillageCode {
                name: "大兴农场第九管理区",
                code: "014",
            },
            VillageCode {
                name: "大兴农场第十管理区",
                code: "015",
            },
            VillageCode {
                name: "大兴农场第十一管理区",
                code: "016",
            },
            VillageCode {
                name: "大兴农场第十二管理区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "创业农场",
        code: "025",
        villages: &[
            VillageCode {
                name: "黑龙江省创业街道办事处第一居委会",
                code: "001",
            },
            VillageCode {
                name: "黑龙江省创业街道办事处第二居委会",
                code: "002",
            },
            VillageCode {
                name: "黑龙江省创业街道办事处第三居委会",
                code: "003",
            },
            VillageCode {
                name: "黑龙江省创业街道办事处第四居委会",
                code: "004",
            },
            VillageCode {
                name: "创业农场第一管理区",
                code: "005",
            },
            VillageCode {
                name: "创业农场第二管理区",
                code: "006",
            },
            VillageCode {
                name: "创业农场第三管理区",
                code: "007",
            },
            VillageCode {
                name: "创业农场第四管理区",
                code: "008",
            },
            VillageCode {
                name: "创业农场第五管理区",
                code: "009",
            },
            VillageCode {
                name: "创业农场第六管理区",
                code: "010",
            },
            VillageCode {
                name: "创业农场第七管理区",
                code: "011",
            },
            VillageCode {
                name: "创业农场第八管理区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "种猪场",
        code: "026",
        villages: &[VillageCode {
            name: "种猪场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "种畜场",
        code: "027",
        villages: &[VillageCode {
            name: "种畜场虚拟生活区",
            code: "001",
        }],
    },
];

static TOWNS_HJ_027: [TownCode; 13] = [
    TownCode {
        name: "抚远镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "西山社区居委会",
                code: "001",
            },
            VillageCode {
                name: "临江社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新兴社区居委会",
                code: "003",
            },
            VillageCode {
                name: "中心社区居委会",
                code: "004",
            },
            VillageCode {
                name: "城南社区居委会",
                code: "005",
            },
            VillageCode {
                name: "新建社区居委会",
                code: "006",
            },
            VillageCode {
                name: "幸福社区居委会",
                code: "007",
            },
            VillageCode {
                name: "红光赫哲族村委会",
                code: "008",
            },
            VillageCode {
                name: "河西村委会",
                code: "009",
            },
            VillageCode {
                name: "石头卧子队",
                code: "010",
            },
            VillageCode {
                name: "亮子队",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "寒葱沟镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "曙光社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "红旗村委会",
                code: "002",
            },
            VillageCode {
                name: "东岗村委会",
                code: "003",
            },
            VillageCode {
                name: "新兴村委会",
                code: "004",
            },
            VillageCode {
                name: "红星村委会",
                code: "005",
            },
            VillageCode {
                name: "红卫村委会",
                code: "006",
            },
            VillageCode {
                name: "红丰村委会",
                code: "007",
            },
            VillageCode {
                name: "农富村委会",
                code: "008",
            },
            VillageCode {
                name: "良种场生活区",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "浓桥镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "和谐社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东方红村委会",
                code: "002",
            },
            VillageCode {
                name: "长征村委会",
                code: "003",
            },
            VillageCode {
                name: "建国村委会",
                code: "004",
            },
            VillageCode {
                name: "建设村委会",
                code: "005",
            },
            VillageCode {
                name: "东极村委会",
                code: "006",
            },
            VillageCode {
                name: "建兴村委会",
                code: "007",
            },
            VillageCode {
                name: "新海村委会",
                code: "008",
            },
            VillageCode {
                name: "新江村委会",
                code: "009",
            },
            VillageCode {
                name: "新远村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "乌苏镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "赫哲社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "赫哲族村委会",
                code: "002",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "003",
            },
            VillageCode {
                name: "万里村委会",
                code: "004",
            },
            VillageCode {
                name: "东胜村委会",
                code: "005",
            },
            VillageCode {
                name: "东兴村委会",
                code: "006",
            },
            VillageCode {
                name: "永胜村委会",
                code: "007",
            },
            VillageCode {
                name: "永丰村委会",
                code: "008",
            },
            VillageCode {
                name: "八盖村委会",
                code: "009",
            },
            VillageCode {
                name: "北岗队",
                code: "010",
            },
            VillageCode {
                name: "东河队",
                code: "011",
            },
            VillageCode {
                name: "别拉洪队",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "黑瞎子岛镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "南岗村委会",
                code: "001",
            },
            VillageCode {
                name: "东安村委会",
                code: "002",
            },
            VillageCode {
                name: "东富村委会",
                code: "003",
            },
            VillageCode {
                name: "东升村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "通江镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "东红村委会",
                code: "001",
            },
            VillageCode {
                name: "东发村委会",
                code: "002",
            },
            VillageCode {
                name: "东辉畜牧场生活区",
                code: "003",
            },
            VillageCode {
                name: "小河子队",
                code: "004",
            },
            VillageCode {
                name: "东风队",
                code: "005",
            },
            VillageCode {
                name: "团结队",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "海青镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "海林村委会",
                code: "001",
            },
            VillageCode {
                name: "永安村委会",
                code: "002",
            },
            VillageCode {
                name: "永发村委会",
                code: "003",
            },
            VillageCode {
                name: "永富村委会",
                code: "004",
            },
            VillageCode {
                name: "海旺村委会",
                code: "005",
            },
            VillageCode {
                name: "海兴村委会",
                code: "006",
            },
            VillageCode {
                name: "海滨村委会",
                code: "007",
            },
            VillageCode {
                name: "海宏村委会",
                code: "008",
            },
            VillageCode {
                name: "海源村委会",
                code: "009",
            },
            VillageCode {
                name: "海青队",
                code: "010",
            },
            VillageCode {
                name: "四合队",
                code: "011",
            },
            VillageCode {
                name: "亮子里队",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "浓江乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "创业村委会",
                code: "001",
            },
            VillageCode {
                name: "双胜村委会",
                code: "002",
            },
            VillageCode {
                name: "生德库村委会",
                code: "003",
            },
            VillageCode {
                name: "浓江队",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "别拉洪乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "民丰村委会",
                code: "001",
            },
            VillageCode {
                name: "民富村委会",
                code: "002",
            },
            VillageCode {
                name: "向阳一队",
                code: "003",
            },
            VillageCode {
                name: "向阳二队",
                code: "004",
            },
            VillageCode {
                name: "向阳三队",
                code: "005",
            },
            VillageCode {
                name: "向阳四队",
                code: "006",
            },
            VillageCode {
                name: "向阳五队",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "鸭南乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "鸭南村委会",
                code: "001",
            },
            VillageCode {
                name: "镇西村委会",
                code: "002",
            },
            VillageCode {
                name: "富兴村委会",
                code: "003",
            },
            VillageCode {
                name: "新胜村委会",
                code: "004",
            },
            VillageCode {
                name: "四排村委会",
                code: "005",
            },
            VillageCode {
                name: "平原村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "前哨农场",
        code: "011",
        villages: &[
            VillageCode {
                name: "黑龙江省前哨街道办事处第一居委会",
                code: "001",
            },
            VillageCode {
                name: "黑龙江省前哨街道办事处第二居委会",
                code: "002",
            },
            VillageCode {
                name: "黑龙江省前哨街道办事处第三居委会",
                code: "003",
            },
            VillageCode {
                name: "黑龙江省前哨街道办事处第四居委会",
                code: "004",
            },
            VillageCode {
                name: "黑龙江省前哨街道办事处第五居委会",
                code: "005",
            },
            VillageCode {
                name: "前哨农场第一管理区",
                code: "006",
            },
            VillageCode {
                name: "前哨农场第二管理区",
                code: "007",
            },
            VillageCode {
                name: "前哨农场第三管理区",
                code: "008",
            },
            VillageCode {
                name: "前哨农场第四管理区",
                code: "009",
            },
            VillageCode {
                name: "前哨农场第五管理区",
                code: "010",
            },
            VillageCode {
                name: "前哨农场第六管理区",
                code: "011",
            },
            VillageCode {
                name: "前哨农场第七管理区",
                code: "012",
            },
            VillageCode {
                name: "前哨农场第八管理区",
                code: "013",
            },
            VillageCode {
                name: "前哨农场第九管理区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "前锋农场",
        code: "012",
        villages: &[
            VillageCode {
                name: "黑龙江省前锋街道办事处第一居委会",
                code: "001",
            },
            VillageCode {
                name: "黑龙江省前锋街道办事处第二居委会",
                code: "002",
            },
            VillageCode {
                name: "黑龙江省前锋街道办事处第三居委会",
                code: "003",
            },
            VillageCode {
                name: "黑龙江省前锋街道办事处第四居委会",
                code: "004",
            },
            VillageCode {
                name: "前锋农场第一管理区",
                code: "005",
            },
            VillageCode {
                name: "前锋农场第二管理区",
                code: "006",
            },
            VillageCode {
                name: "前锋农场第三管理区",
                code: "007",
            },
            VillageCode {
                name: "前锋农场第四管理区",
                code: "008",
            },
            VillageCode {
                name: "前锋农场第五管理区",
                code: "009",
            },
            VillageCode {
                name: "前锋农场第六管理区",
                code: "010",
            },
            VillageCode {
                name: "前锋农场第七管理区",
                code: "011",
            },
            VillageCode {
                name: "前锋农场第八管理区",
                code: "012",
            },
            VillageCode {
                name: "前锋农场第九管理区",
                code: "013",
            },
            VillageCode {
                name: "前锋农场第十管理区",
                code: "014",
            },
            VillageCode {
                name: "前锋农场第十一管理区",
                code: "015",
            },
            VillageCode {
                name: "前锋农场第十二管理区",
                code: "016",
            },
            VillageCode {
                name: "前锋农场第十三管理区",
                code: "017",
            },
            VillageCode {
                name: "前锋农场第十四管理区",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "二道河农场",
        code: "013",
        villages: &[
            VillageCode {
                name: "黑龙江省二道河街道办事处第一居委会",
                code: "001",
            },
            VillageCode {
                name: "黑龙江省二道河街道办事处第二居委会",
                code: "002",
            },
            VillageCode {
                name: "二道河农场第一管理区",
                code: "003",
            },
            VillageCode {
                name: "二道河农场第二管理区",
                code: "004",
            },
            VillageCode {
                name: "二道河农场第三管理区",
                code: "005",
            },
            VillageCode {
                name: "二道河农场第四管理区",
                code: "006",
            },
            VillageCode {
                name: "二道河农场第五管理区",
                code: "007",
            },
            VillageCode {
                name: "二道河农场第六管理区",
                code: "008",
            },
            VillageCode {
                name: "二道河农场第八管理区",
                code: "009",
            },
        ],
    },
];

static TOWNS_HJ_028: [TownCode; 11] = [
    TownCode {
        name: "兴安街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "兴民社区居委会",
                code: "001",
            },
            VillageCode {
                name: "兴乐社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "兴富街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "兴秀社区居委会",
                code: "001",
            },
            VillageCode {
                name: "富强社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "兴和街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "和顺社区居委会",
                code: "001",
            },
            VillageCode {
                name: "兴平社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "兴盛街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "兴业社区居委会",
                code: "001",
            },
            VillageCode {
                name: "兴城社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "欣源街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "春城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "枫叶社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "北山街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "新风社区居委会",
                code: "001",
            },
            VillageCode {
                name: "冬梅社区居委会",
                code: "002",
            },
            VillageCode {
                name: "安居社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "兴华街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "新城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新立社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "金沙街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "福泽社区居委会",
                code: "001",
            },
            VillageCode {
                name: "安民社区居委会",
                code: "002",
            },
            VillageCode {
                name: "利民社区居委会",
                code: "003",
            },
            VillageCode {
                name: "乐民社区居委会",
                code: "004",
            },
            VillageCode {
                name: "新青林场社区居委会",
                code: "005",
            },
            VillageCode {
                name: "向阳林场社区居委会",
                code: "006",
            },
            VillageCode {
                name: "胜利林场社区居委会",
                code: "007",
            },
            VillageCode {
                name: "密林经营所社区居委会",
                code: "008",
            },
            VillageCode {
                name: "金沙经营所社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "红旗镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "红旗村委会",
                code: "001",
            },
            VillageCode {
                name: "红胜村委会",
                code: "002",
            },
            VillageCode {
                name: "红新村委会",
                code: "003",
            },
            VillageCode {
                name: "红光村委会",
                code: "004",
            },
            VillageCode {
                name: "红升村委会",
                code: "005",
            },
            VillageCode {
                name: "红卫村委会",
                code: "006",
            },
            VillageCode {
                name: "红鲜村委会",
                code: "007",
            },
            VillageCode {
                name: "新起村委会",
                code: "008",
            },
            VillageCode {
                name: "新村村委会",
                code: "009",
            },
            VillageCode {
                name: "东升村委会",
                code: "010",
            },
            VillageCode {
                name: "曙光村委会",
                code: "011",
            },
            VillageCode {
                name: "太和村委会",
                code: "012",
            },
            VillageCode {
                name: "东风林场生活区",
                code: "013",
            },
            VillageCode {
                name: "大六林场生活区",
                code: "014",
            },
            VillageCode {
                name: "新兴林场生活区",
                code: "015",
            },
            VillageCode {
                name: "新建林场生活区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "长兴乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "长兴村委会",
                code: "001",
            },
            VillageCode {
                name: "东新村委会",
                code: "002",
            },
            VillageCode {
                name: "中鲜村委会",
                code: "003",
            },
            VillageCode {
                name: "马鞍村委会",
                code: "004",
            },
            VillageCode {
                name: "柳毛河村委会",
                code: "005",
            },
            VillageCode {
                name: "宏志村委会",
                code: "006",
            },
            VillageCode {
                name: "罗泉村委会",
                code: "007",
            },
            VillageCode {
                name: "长发村委会",
                code: "008",
            },
            VillageCode {
                name: "长安村委会",
                code: "009",
            },
            VillageCode {
                name: "红旗村委会",
                code: "010",
            },
            VillageCode {
                name: "新风村委会",
                code: "011",
            },
            VillageCode {
                name: "山星村委会",
                code: "012",
            },
            VillageCode {
                name: "农牧场生活区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "七台河经济开发区管理委员会",
        code: "011",
        villages: &[VillageCode {
            name: "七台河经济开发区管理委员会社区",
            code: "001",
        }],
    },
];

static TOWNS_HJ_029: [TownCode; 7] = [
    TownCode {
        name: "桃东街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "学府社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "同仁社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "东方社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "桃南街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "运销社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "祥和社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "桃西街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "金厦社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "长青社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "桃北街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "旭日社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "银泉社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "花园社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "东正社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "桃源街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "朝阳社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "运管社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "桃山街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "文苑社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "安康社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "万宝河镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "新选社区居委会",
                code: "001",
            },
            VillageCode {
                name: "六井社区居委会",
                code: "002",
            },
            VillageCode {
                name: "桃山村委会",
                code: "003",
            },
            VillageCode {
                name: "桃南村委会",
                code: "004",
            },
            VillageCode {
                name: "良种场村委会",
                code: "005",
            },
            VillageCode {
                name: "八道岗村委会",
                code: "006",
            },
            VillageCode {
                name: "万宝村委会",
                code: "007",
            },
            VillageCode {
                name: "红岩村委会",
                code: "008",
            },
            VillageCode {
                name: "茄子河林场生活区",
                code: "009",
            },
        ],
    },
];

static TOWNS_HJ_030: [TownCode; 11] = [
    TownCode {
        name: "东风街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "惠民社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "安民社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "盛馨社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "欣苑社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "富强街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "富强社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "铁东社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "龙湖街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "龙湖社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "鹿山社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "红岚社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "东胜街道",
        code: "004",
        villages: &[VillageCode {
            name: "东胜社区居民委员会",
            code: "001",
        }],
    },
    TownCode {
        name: "湖东街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "花海社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "康富社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "康乐社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "湖东社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "通达街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "通达社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "永泰社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "茄子河镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "新富村委会",
                code: "001",
            },
            VillageCode {
                name: "茄子河村委会",
                code: "002",
            },
            VillageCode {
                name: "东胜村委会",
                code: "003",
            },
            VillageCode {
                name: "太阳村委会",
                code: "004",
            },
            VillageCode {
                name: "正阳村委会",
                code: "005",
            },
            VillageCode {
                name: "东风村委会",
                code: "006",
            },
            VillageCode {
                name: "中河村委会",
                code: "007",
            },
            VillageCode {
                name: "万龙村委会",
                code: "008",
            },
            VillageCode {
                name: "富强村委会",
                code: "009",
            },
            VillageCode {
                name: "兴龙村委会",
                code: "010",
            },
            VillageCode {
                name: "双阳村委会",
                code: "011",
            },
            VillageCode {
                name: "东河村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "宏伟镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "新明山村委会",
                code: "001",
            },
            VillageCode {
                name: "清泉村委会",
                code: "002",
            },
            VillageCode {
                name: "岚棒山村委会",
                code: "003",
            },
            VillageCode {
                name: "虎山村委会",
                code: "004",
            },
            VillageCode {
                name: "向阳山村委会",
                code: "005",
            },
            VillageCode {
                name: "京石泉村委会",
                code: "006",
            },
            VillageCode {
                name: "建丰村委会",
                code: "007",
            },
            VillageCode {
                name: "桃山村委会",
                code: "008",
            },
            VillageCode {
                name: "向桦村委会",
                code: "009",
            },
            VillageCode {
                name: "钟山村委会",
                code: "010",
            },
            VillageCode {
                name: "三合村委会",
                code: "011",
            },
            VillageCode {
                name: "峻山村委会",
                code: "012",
            },
            VillageCode {
                name: "城山村委会",
                code: "013",
            },
            VillageCode {
                name: "岚峰村委会",
                code: "014",
            },
            VillageCode {
                name: "鹿山村委会",
                code: "015",
            },
            VillageCode {
                name: "春山村委会",
                code: "016",
            },
            VillageCode {
                name: "安山村委会",
                code: "017",
            },
            VillageCode {
                name: "山峰村委会",
                code: "018",
            },
            VillageCode {
                name: "英山村委会",
                code: "019",
            },
            VillageCode {
                name: "云山村委会",
                code: "020",
            },
            VillageCode {
                name: "林山村委会",
                code: "021",
            },
            VillageCode {
                name: "福山村委会",
                code: "022",
            },
            VillageCode {
                name: "前山村委会",
                code: "023",
            },
            VillageCode {
                name: "河东村委会",
                code: "024",
            },
            VillageCode {
                name: "环山村委会",
                code: "025",
            },
            VillageCode {
                name: "双兴村委会",
                code: "026",
            },
            VillageCode {
                name: "富山村委会",
                code: "027",
            },
            VillageCode {
                name: "山泉村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "兴北镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "兴北社区",
                code: "001",
            },
            VillageCode {
                name: "兴安社区",
                code: "002",
            },
            VillageCode {
                name: "金矿社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "铁山乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "新发村委会",
                code: "001",
            },
            VillageCode {
                name: "创新村委会",
                code: "002",
            },
            VillageCode {
                name: "立新村委会",
                code: "003",
            },
            VillageCode {
                name: "红星村委会",
                code: "004",
            },
            VillageCode {
                name: "铁山村委会",
                code: "005",
            },
            VillageCode {
                name: "铁东村委会",
                code: "006",
            },
            VillageCode {
                name: "铁西村委会",
                code: "007",
            },
            VillageCode {
                name: "四新村委会",
                code: "008",
            },
            VillageCode {
                name: "五星村委会",
                code: "009",
            },
            VillageCode {
                name: "铁山林场生活区",
                code: "010",
            },
            VillageCode {
                name: "龙山林场生活区",
                code: "011",
            },
            VillageCode {
                name: "七矿农副业开发基地生活区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "中心河乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "新立村委会",
                code: "001",
            },
            VillageCode {
                name: "更生村委会",
                code: "002",
            },
            VillageCode {
                name: "双利村委会",
                code: "003",
            },
            VillageCode {
                name: "新兴村委会",
                code: "004",
            },
            VillageCode {
                name: "团胜村委会",
                code: "005",
            },
            VillageCode {
                name: "金乡村委会",
                code: "006",
            },
            VillageCode {
                name: "中心河村委会",
                code: "007",
            },
            VillageCode {
                name: "龙湖村委会",
                code: "008",
            },
            VillageCode {
                name: "金沙林场生活区",
                code: "009",
            },
        ],
    },
];

static TOWNS_HJ_031: [TownCode; 15] = [
    TownCode {
        name: "新起街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "太平社区居委会",
                code: "001",
            },
            VillageCode {
                name: "顺天社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "新华街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "长安社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南城社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "元明街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "镇安社区居委会",
                code: "001",
            },
            VillageCode {
                name: "康华社区居委会",
                code: "002",
            },
            VillageCode {
                name: "学府社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "铁西街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "秀岳社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南岳社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "城西街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "友谊社区居委会",
                code: "001",
            },
            VillageCode {
                name: "花园社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "勃利镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "吉祥村委会",
                code: "001",
            },
            VillageCode {
                name: "新华村委会",
                code: "002",
            },
            VillageCode {
                name: "元明村委会",
                code: "003",
            },
            VillageCode {
                name: "镇安村委会",
                code: "004",
            },
            VillageCode {
                name: "城西村委会",
                code: "005",
            },
            VillageCode {
                name: "镇南村委会",
                code: "006",
            },
            VillageCode {
                name: "东岗村委会",
                code: "007",
            },
            VillageCode {
                name: "和平村委会",
                code: "008",
            },
            VillageCode {
                name: "大五村委会",
                code: "009",
            },
            VillageCode {
                name: "蔬菜村委会",
                code: "010",
            },
            VillageCode {
                name: "荣兴村委会",
                code: "011",
            },
            VillageCode {
                name: "太平村委会",
                code: "012",
            },
            VillageCode {
                name: "九龙村委会",
                code: "013",
            },
            VillageCode {
                name: "新起村委会",
                code: "014",
            },
            VillageCode {
                name: "顺天村委会",
                code: "015",
            },
            VillageCode {
                name: "星华村委会",
                code: "016",
            },
            VillageCode {
                name: "全胜村委会",
                code: "017",
            },
            VillageCode {
                name: "通天一林场生活区",
                code: "018",
            },
            VillageCode {
                name: "通天二林场生活区",
                code: "019",
            },
            VillageCode {
                name: "宏伟林场生活区",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "小五站镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "宏图村委会",
                code: "001",
            },
            VillageCode {
                name: "驼腰子村委会",
                code: "002",
            },
            VillageCode {
                name: "大义村委会",
                code: "003",
            },
            VillageCode {
                name: "卫东村委会",
                code: "004",
            },
            VillageCode {
                name: "东丰村委会",
                code: "005",
            },
            VillageCode {
                name: "保丰村委会",
                code: "006",
            },
            VillageCode {
                name: "新民村委会",
                code: "007",
            },
            VillageCode {
                name: "庆云村委会",
                code: "008",
            },
            VillageCode {
                name: "新兴村委会",
                code: "009",
            },
            VillageCode {
                name: "大六村委会",
                code: "010",
            },
            VillageCode {
                name: "东方红林场生活区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "大四站镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "东风村委会",
                code: "001",
            },
            VillageCode {
                name: "玄羊河村委会",
                code: "002",
            },
            VillageCode {
                name: "大连珠河村委会",
                code: "003",
            },
            VillageCode {
                name: "小连珠河村委会",
                code: "004",
            },
            VillageCode {
                name: "古城村委会",
                code: "005",
            },
            VillageCode {
                name: "联合村委会",
                code: "006",
            },
            VillageCode {
                name: "天巨村委会",
                code: "007",
            },
            VillageCode {
                name: "地河子村委会",
                code: "008",
            },
            VillageCode {
                name: "常山村委会",
                code: "009",
            },
            VillageCode {
                name: "仁兴村委会",
                code: "010",
            },
            VillageCode {
                name: "发展村委会",
                code: "011",
            },
            VillageCode {
                name: "开发村委会",
                code: "012",
            },
            VillageCode {
                name: "大祥村委会",
                code: "013",
            },
            VillageCode {
                name: "双兴村委会",
                code: "014",
            },
            VillageCode {
                name: "吉兴河村委会",
                code: "015",
            },
            VillageCode {
                name: "立新村委会",
                code: "016",
            },
            VillageCode {
                name: "福兴村委会",
                code: "017",
            },
            VillageCode {
                name: "养殖场生活区",
                code: "018",
            },
            VillageCode {
                name: "种牛场生活区",
                code: "019",
            },
            VillageCode {
                name: "吉兴河水库生活区",
                code: "020",
            },
            VillageCode {
                name: "福兴林场生活区",
                code: "021",
            },
            VillageCode {
                name: "红旗林场生活区",
                code: "022",
            },
            VillageCode {
                name: "红星林场生活区",
                code: "023",
            },
            VillageCode {
                name: "吉兴河林场生活区",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "双河镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "永峰村委会",
                code: "001",
            },
            VillageCode {
                name: "永平村委会",
                code: "002",
            },
            VillageCode {
                name: "新发村委会",
                code: "003",
            },
            VillageCode {
                name: "治安村委会",
                code: "004",
            },
            VillageCode {
                name: "福安村委会",
                code: "005",
            },
            VillageCode {
                name: "中和村委会",
                code: "006",
            },
            VillageCode {
                name: "兴安村委会",
                code: "007",
            },
            VillageCode {
                name: "太安村委会",
                code: "008",
            },
            VillageCode {
                name: "东方红村委会",
                code: "009",
            },
            VillageCode {
                name: "太阳升村委会",
                code: "010",
            },
            VillageCode {
                name: "中胜村委会",
                code: "011",
            },
            VillageCode {
                name: "民权村委会",
                code: "012",
            },
            VillageCode {
                name: "生产村委会",
                code: "013",
            },
            VillageCode {
                name: "永胜村委会",
                code: "014",
            },
            VillageCode {
                name: "致富村委会",
                code: "015",
            },
            VillageCode {
                name: "双胜村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "倭肯镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "忠义村委会",
                code: "001",
            },
            VillageCode {
                name: "正阳村委会",
                code: "002",
            },
            VillageCode {
                name: "西连村委会",
                code: "003",
            },
            VillageCode {
                name: "东连村委会",
                code: "004",
            },
            VillageCode {
                name: "连峰村委会",
                code: "005",
            },
            VillageCode {
                name: "兴胜村委会",
                code: "006",
            },
            VillageCode {
                name: "东升村委会",
                code: "007",
            },
            VillageCode {
                name: "镇西村委会",
                code: "008",
            },
            VillageCode {
                name: "镇东村委会",
                code: "009",
            },
            VillageCode {
                name: "长福村委会",
                code: "010",
            },
            VillageCode {
                name: "平安村委会",
                code: "011",
            },
            VillageCode {
                name: "民主村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "青山乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "青山村委会",
                code: "001",
            },
            VillageCode {
                name: "青峰村村委会",
                code: "002",
            },
            VillageCode {
                name: "幸福村委会",
                code: "003",
            },
            VillageCode {
                name: "奋斗村委会",
                code: "004",
            },
            VillageCode {
                name: "中原村委会",
                code: "005",
            },
            VillageCode {
                name: "互助村委会",
                code: "006",
            },
            VillageCode {
                name: "建设村委会",
                code: "007",
            },
            VillageCode {
                name: "太升村委会",
                code: "008",
            },
            VillageCode {
                name: "青龙山村委会",
                code: "009",
            },
            VillageCode {
                name: "勃信村委会",
                code: "010",
            },
            VillageCode {
                name: "良种场生活区",
                code: "011",
            },
            VillageCode {
                name: "互助水库生活区",
                code: "012",
            },
            VillageCode {
                name: "长兴林场生活区",
                code: "013",
            },
            VillageCode {
                name: "罗泉林场生活区",
                code: "014",
            },
            VillageCode {
                name: "种羊场生活区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "永恒乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "恒太村委会",
                code: "001",
            },
            VillageCode {
                name: "金山村委会",
                code: "002",
            },
            VillageCode {
                name: "河口村委会",
                code: "003",
            },
            VillageCode {
                name: "荣光村委会",
                code: "004",
            },
            VillageCode {
                name: "解放村委会",
                code: "005",
            },
            VillageCode {
                name: "北兴村委会",
                code: "006",
            },
            VillageCode {
                name: "中江村委会",
                code: "007",
            },
            VillageCode {
                name: "东安村委会",
                code: "008",
            },
            VillageCode {
                name: "丰收村委会",
                code: "009",
            },
            VillageCode {
                name: "景太村委会",
                code: "010",
            },
            VillageCode {
                name: "永顺村委会",
                code: "011",
            },
            VillageCode {
                name: "东辉村委会",
                code: "012",
            },
            VillageCode {
                name: "先峰村委会",
                code: "013",
            },
            VillageCode {
                name: "荣合村委会",
                code: "014",
            },
            VillageCode {
                name: "恒山村委会",
                code: "015",
            },
            VillageCode {
                name: "岱山村委会",
                code: "016",
            },
            VillageCode {
                name: "齐心村委会",
                code: "017",
            },
            VillageCode {
                name: "团结村委会",
                code: "018",
            },
            VillageCode {
                name: "红林村委会",
                code: "019",
            },
            VillageCode {
                name: "河口林场生活区",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "抢垦乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "抢垦村委会",
                code: "001",
            },
            VillageCode {
                name: "原发村委会",
                code: "002",
            },
            VillageCode {
                name: "三兴村委会",
                code: "003",
            },
            VillageCode {
                name: "福利村委会",
                code: "004",
            },
            VillageCode {
                name: "前进村委会",
                code: "005",
            },
            VillageCode {
                name: "前程村委会",
                code: "006",
            },
            VillageCode {
                name: "三合村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "杏树朝鲜族乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "杏树村委会",
                code: "001",
            },
            VillageCode {
                name: "东兴村委会",
                code: "002",
            },
            VillageCode {
                name: "德胜村委会",
                code: "003",
            },
            VillageCode {
                name: "永久村委会",
                code: "004",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "005",
            },
            VillageCode {
                name: "永安村委会",
                code: "006",
            },
            VillageCode {
                name: "原野村委会",
                code: "007",
            },
            VillageCode {
                name: "增产村委会",
                code: "008",
            },
            VillageCode {
                name: "大西村委会",
                code: "009",
            },
            VillageCode {
                name: "杏鲜村委会",
                code: "010",
            },
            VillageCode {
                name: "金刚村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "吉兴朝鲜族满族乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "西吉兴村委会",
                code: "001",
            },
            VillageCode {
                name: "东吉兴村委会",
                code: "002",
            },
            VillageCode {
                name: "永乐村委会",
                code: "003",
            },
            VillageCode {
                name: "和胜村委会",
                code: "004",
            },
            VillageCode {
                name: "合心村委会",
                code: "005",
            },
            VillageCode {
                name: "团山村委会",
                code: "006",
            },
            VillageCode {
                name: "兴耕村委会",
                code: "007",
            },
            VillageCode {
                name: "合庆村委会",
                code: "008",
            },
            VillageCode {
                name: "长太村委会",
                code: "009",
            },
            VillageCode {
                name: "三民村委会",
                code: "010",
            },
            VillageCode {
                name: "心和村委会",
                code: "011",
            },
            VillageCode {
                name: "富兴村委会",
                code: "012",
            },
            VillageCode {
                name: "大阳村委会",
                code: "013",
            },
            VillageCode {
                name: "厚春村委会",
                code: "014",
            },
        ],
    },
];

static TOWNS_HJ_032: [TownCode; 7] = [
    TownCode {
        name: "新安街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "柴市社区居委会",
                code: "001",
            },
            VillageCode {
                name: "牡丹社区居委会",
                code: "002",
            },
            VillageCode {
                name: "市政社区居委会",
                code: "003",
            },
            VillageCode {
                name: "花园社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "长安街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "清福社区居委会",
                code: "001",
            },
            VillageCode {
                name: "福民社区居委会",
                code: "002",
            },
            VillageCode {
                name: "幸福社区居委会",
                code: "003",
            },
            VillageCode {
                name: "平安社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "七星街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "景福社区居委会",
                code: "001",
            },
            VillageCode {
                name: "紫云社区居委会",
                code: "002",
            },
            VillageCode {
                name: "照庆社区居委会",
                code: "003",
            },
            VillageCode {
                name: "积善社区居委会",
                code: "004",
            },
            VillageCode {
                name: "光华社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "五星街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "建福社区居委会",
                code: "001",
            },
            VillageCode {
                name: "热电社区居委会",
                code: "002",
            },
            VillageCode {
                name: "林机社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "东兴街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "学府社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东居华庭社区居委会",
                code: "002",
            },
            VillageCode {
                name: "丽江社区居委会",
                code: "003",
            },
            VillageCode {
                name: "下乜河村委会",
                code: "004",
            },
            VillageCode {
                name: "江南村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "振兴街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "兴隆社区居委会",
                code: "001",
            },
            VillageCode {
                name: "三利社区居委会",
                code: "002",
            },
            VillageCode {
                name: "绿地社区居委会",
                code: "003",
            },
            VillageCode {
                name: "恒大社区居委会",
                code: "004",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "005",
            },
            VillageCode {
                name: "乜河村委会",
                code: "006",
            },
            VillageCode {
                name: "中乜河村委会",
                code: "007",
            },
            VillageCode {
                name: "胜利村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "兴隆镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "牡丹峰社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大湾村委会",
                code: "002",
            },
            VillageCode {
                name: "大团村委会",
                code: "003",
            },
            VillageCode {
                name: "小团村委会",
                code: "004",
            },
            VillageCode {
                name: "跃进村委会",
                code: "005",
            },
            VillageCode {
                name: "东村村委会",
                code: "006",
            },
            VillageCode {
                name: "东胜村委会",
                code: "007",
            },
            VillageCode {
                name: "西村村委会",
                code: "008",
            },
            VillageCode {
                name: "迎门山村委会",
                code: "009",
            },
            VillageCode {
                name: "河西村委会",
                code: "010",
            },
            VillageCode {
                name: "桥头村委会",
                code: "011",
            },
            VillageCode {
                name: "岭东村委会",
                code: "012",
            },
        ],
    },
];

static TOWNS_HJ_033: [TownCode; 8] = [
    TownCode {
        name: "阳明街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "光华社区居委会",
                code: "001",
            },
            VillageCode {
                name: "阳光社区居委会",
                code: "002",
            },
            VillageCode {
                name: "阳明社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东华苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "木材社区居委会",
                code: "005",
            },
            VillageCode {
                name: "公园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "盛世华庭社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "前进街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "机车东社区居委会",
                code: "001",
            },
            VillageCode {
                name: "恒丰居委会",
                code: "002",
            },
            VillageCode {
                name: "机车西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "二纺居委会",
                code: "004",
            },
            VillageCode {
                name: "裕华园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "富江社区居委会",
                code: "006",
            },
            VillageCode {
                name: "阳明村村委会",
                code: "007",
            },
            VillageCode {
                name: "东江村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "新兴街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "东风居委会",
                code: "001",
            },
            VillageCode {
                name: "裕民社区居委会",
                code: "002",
            },
            VillageCode {
                name: "二发电社区居委会",
                code: "003",
            },
            VillageCode {
                name: "莲花居委会",
                code: "004",
            },
            VillageCode {
                name: "镇江村村委会",
                code: "005",
            },
            VillageCode {
                name: "裕民村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "桦林橡胶厂街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "桦橡东社区居委会",
                code: "001",
            },
            VillageCode {
                name: "桦橡西社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "铁岭镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "铁岭社区居委会",
                code: "001",
            },
            VillageCode {
                name: "色织社区居委会",
                code: "002",
            },
            VillageCode {
                name: "爱河社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南山社区居委会",
                code: "004",
            },
            VillageCode {
                name: "青化社区居委会",
                code: "005",
            },
            VillageCode {
                name: "一村村委会",
                code: "006",
            },
            VillageCode {
                name: "二村村委会",
                code: "007",
            },
            VillageCode {
                name: "三村村委会",
                code: "008",
            },
            VillageCode {
                name: "福民村村委会",
                code: "009",
            },
            VillageCode {
                name: "东新村村委会",
                code: "010",
            },
            VillageCode {
                name: "福长村村委会",
                code: "011",
            },
            VillageCode {
                name: "青梅村村委会",
                code: "012",
            },
            VillageCode {
                name: "四道村村委会",
                code: "013",
            },
            VillageCode {
                name: "北岔村村委会",
                code: "014",
            },
            VillageCode {
                name: "苇子沟村村委会",
                code: "015",
            },
            VillageCode {
                name: "南岔村村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "桦林镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "江东社区居委会",
                code: "001",
            },
            VillageCode {
                name: "桦林社区居委会",
                code: "002",
            },
            VillageCode {
                name: "安民社区居委会",
                code: "003",
            },
            VillageCode {
                name: "工农村村委会",
                code: "004",
            },
            VillageCode {
                name: "南城子村村委会",
                code: "005",
            },
            VillageCode {
                name: "安民村村委会",
                code: "006",
            },
            VillageCode {
                name: "桦林村村委会",
                code: "007",
            },
            VillageCode {
                name: "临江村村委会",
                code: "008",
            },
            VillageCode {
                name: "互利村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "磨刀石镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "华街居委会",
                code: "001",
            },
            VillageCode {
                name: "红星居委会",
                code: "002",
            },
            VillageCode {
                name: "新兴居委会",
                code: "003",
            },
            VillageCode {
                name: "奋斗居委会",
                code: "004",
            },
            VillageCode {
                name: "幸福社区居委会",
                code: "005",
            },
            VillageCode {
                name: "前进村委会",
                code: "006",
            },
            VillageCode {
                name: "远景村委会",
                code: "007",
            },
            VillageCode {
                name: "清平村委会",
                code: "008",
            },
            VillageCode {
                name: "金星村委会",
                code: "009",
            },
            VillageCode {
                name: "大甸子村委会",
                code: "010",
            },
            VillageCode {
                name: "红星村委会",
                code: "011",
            },
            VillageCode {
                name: "富强村委会",
                code: "012",
            },
            VillageCode {
                name: "六里地村委会",
                code: "013",
            },
            VillageCode {
                name: "红林村委会",
                code: "014",
            },
            VillageCode {
                name: "代马沟村委会",
                code: "015",
            },
            VillageCode {
                name: "转心湖村委会",
                code: "016",
            },
            VillageCode {
                name: "团山子村委会",
                code: "017",
            },
            VillageCode {
                name: "山底村委会",
                code: "018",
            },
            VillageCode {
                name: "苇子沟村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "五林镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "振兴社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大兴村委会",
                code: "002",
            },
            VillageCode {
                name: "青西村委会",
                code: "003",
            },
            VillageCode {
                name: "五河村委会",
                code: "004",
            },
            VillageCode {
                name: "长兴村委会",
                code: "005",
            },
            VillageCode {
                name: "孔街村委会",
                code: "006",
            },
            VillageCode {
                name: "姚亮村委会",
                code: "007",
            },
            VillageCode {
                name: "青北村委会",
                code: "008",
            },
            VillageCode {
                name: "陈堡村委会",
                code: "009",
            },
            VillageCode {
                name: "长沟村委会",
                code: "010",
            },
            VillageCode {
                name: "马桥村委会",
                code: "011",
            },
            VillageCode {
                name: "马北村委会",
                code: "012",
            },
            VillageCode {
                name: "洪林村委会",
                code: "013",
            },
            VillageCode {
                name: "马西村委会",
                code: "014",
            },
            VillageCode {
                name: "五星村委会",
                code: "015",
            },
            VillageCode {
                name: "四岗村委会",
                code: "016",
            },
            VillageCode {
                name: "西桥村委会",
                code: "017",
            },
            VillageCode {
                name: "庆丰村委会",
                code: "018",
            },
            VillageCode {
                name: "北甸村委会",
                code: "019",
            },
            VillageCode {
                name: "北星村委会",
                code: "020",
            },
            VillageCode {
                name: "西沟村委会",
                code: "021",
            },
            VillageCode {
                name: "板院村委会",
                code: "022",
            },
            VillageCode {
                name: "杏树村委会",
                code: "023",
            },
            VillageCode {
                name: "金场村委会",
                code: "024",
            },
            VillageCode {
                name: "五岗村委会",
                code: "025",
            },
        ],
    },
];

static TOWNS_HJ_034: [TownCode; 8] = [
    TownCode {
        name: "向阳街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "建卫社区居委会",
                code: "001",
            },
            VillageCode {
                name: "兴平社区居委会",
                code: "002",
            },
            VillageCode {
                name: "拥军社区居委会",
                code: "003",
            },
            VillageCode {
                name: "林卫社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "黄花街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "黄花小区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "文化社区居委会",
                code: "002",
            },
            VillageCode {
                name: "韶山社区居委会",
                code: "003",
            },
            VillageCode {
                name: "耐火社区居委会",
                code: "004",
            },
            VillageCode {
                name: "黄花站社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "铁北街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "泰安社区居委会",
                code: "001",
            },
            VillageCode {
                name: "明月社区居委会",
                code: "002",
            },
            VillageCode {
                name: "桥北社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "新华街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "圣林社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中华社区居委会",
                code: "002",
            },
            VillageCode {
                name: "自建社区居委会",
                code: "003",
            },
            VillageCode {
                name: "祥伦社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "大庆街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "庆兴社区居委会",
                code: "001",
            },
            VillageCode {
                name: "庆发社区居委会",
                code: "002",
            },
            VillageCode {
                name: "庆旺社区居委会",
                code: "003",
            },
            VillageCode {
                name: "庆达社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "兴平街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "地明安居小区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "军马社区居委会",
                code: "002",
            },
            VillageCode {
                name: "通乡社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "北山街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "光明社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北山社区居委会",
                code: "002",
            },
            VillageCode {
                name: "牡纺社区居委会",
                code: "003",
            },
            VillageCode {
                name: "银龙社区居委会",
                code: "004",
            },
            VillageCode {
                name: "山水社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "三道关镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "放牛村村委会",
                code: "001",
            },
            VillageCode {
                name: "大砬子村村委会",
                code: "002",
            },
            VillageCode {
                name: "丰收村村委会",
                code: "003",
            },
            VillageCode {
                name: "江西村村委会",
                code: "004",
            },
            VillageCode {
                name: "银龙村村委会",
                code: "005",
            },
            VillageCode {
                name: "八达村村委会",
                code: "006",
            },
            VillageCode {
                name: "北安村村委会",
                code: "007",
            },
            VillageCode {
                name: "三道关村委会",
                code: "008",
            },
            VillageCode {
                name: "前进村村委会",
                code: "009",
            },
            VillageCode {
                name: "金龙村村委会",
                code: "010",
            },
            VillageCode {
                name: "新荣村村委会",
                code: "011",
            },
            VillageCode {
                name: "平安村村委会",
                code: "012",
            },
        ],
    },
];

static TOWNS_HJ_035: [TownCode; 7] = [
    TownCode {
        name: "太安街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "泰安社区",
                code: "001",
            },
            VillageCode {
                name: "新苑社区",
                code: "002",
            },
            VillageCode {
                name: "永泰社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "仙城街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "仙城社区",
                code: "001",
            },
            VillageCode {
                name: "安康社区",
                code: "002",
            },
            VillageCode {
                name: "鸿民社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "东山街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "东山社区",
                code: "001",
            },
            VillageCode {
                name: "红城社区",
                code: "002",
            },
            VillageCode {
                name: "裕明社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "先锋街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "先岭社区",
                code: "001",
            },
            VillageCode {
                name: "福盛社区",
                code: "002",
            },
            VillageCode {
                name: "福先社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "富国街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "富鑫社区",
                code: "001",
            },
            VillageCode {
                name: "富民社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "安家街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "安民社区",
                code: "001",
            },
            VillageCode {
                name: "安仁社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "灯塔镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "丰收村",
                code: "001",
            },
            VillageCode {
                name: "富强村",
                code: "002",
            },
            VillageCode {
                name: "古仙村",
                code: "003",
            },
            VillageCode {
                name: "太阳升村",
                code: "004",
            },
            VillageCode {
                name: "新力村",
                code: "005",
            },
            VillageCode {
                name: "高古村",
                code: "006",
            },
            VillageCode {
                name: "东孟村",
                code: "007",
            },
            VillageCode {
                name: "西孟村",
                code: "008",
            },
            VillageCode {
                name: "胜利村",
                code: "009",
            },
            VillageCode {
                name: "金河村",
                code: "010",
            },
            VillageCode {
                name: "全康村",
                code: "011",
            },
            VillageCode {
                name: "英华村",
                code: "012",
            },
            VillageCode {
                name: "成山村",
                code: "013",
            },
            VillageCode {
                name: "碾山村",
                code: "014",
            },
            VillageCode {
                name: "龙背村",
                code: "015",
            },
            VillageCode {
                name: "古洞村",
                code: "016",
            },
            VillageCode {
                name: "大房村",
                code: "017",
            },
            VillageCode {
                name: "正风村",
                code: "018",
            },
            VillageCode {
                name: "太和村",
                code: "019",
            },
            VillageCode {
                name: "谦和村",
                code: "020",
            },
            VillageCode {
                name: "建国村",
                code: "021",
            },
            VillageCode {
                name: "沐雨村",
                code: "022",
            },
            VillageCode {
                name: "灯塔村",
                code: "023",
            },
        ],
    },
];

static TOWNS_HJ_036: [TownCode; 12] = [
    TownCode {
        name: "林口镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "站前办事处第一居民委员会",
                code: "001",
            },
            VillageCode {
                name: "站前办事处第二居民委员会",
                code: "002",
            },
            VillageCode {
                name: "站前办事处第三居民委员会",
                code: "003",
            },
            VillageCode {
                name: "站前办事处第四居民委员会",
                code: "004",
            },
            VillageCode {
                name: "站前办事处第五居民委员会",
                code: "005",
            },
            VillageCode {
                name: "站前办事处第六居民委员会",
                code: "006",
            },
            VillageCode {
                name: "站前办事处第七居民委员会",
                code: "007",
            },
            VillageCode {
                name: "站前办事处第八居民委员会",
                code: "008",
            },
            VillageCode {
                name: "站前办事处第九居民委员会",
                code: "009",
            },
            VillageCode {
                name: "站前办事处第十居民委员会",
                code: "010",
            },
            VillageCode {
                name: "东街办事处第一居民委员会",
                code: "011",
            },
            VillageCode {
                name: "东街办事处第二居民委员会",
                code: "012",
            },
            VillageCode {
                name: "东街办事处第三居民委员会",
                code: "013",
            },
            VillageCode {
                name: "东街办事处第四居民委员会",
                code: "014",
            },
            VillageCode {
                name: "东街办事处第五居民委员会",
                code: "015",
            },
            VillageCode {
                name: "东街办事处第六居民委员会",
                code: "016",
            },
            VillageCode {
                name: "东街办事处第七居民委员会",
                code: "017",
            },
            VillageCode {
                name: "东街办事处第八居民委员会",
                code: "018",
            },
            VillageCode {
                name: "西街办事处第一居民委员会",
                code: "019",
            },
            VillageCode {
                name: "西街办事处第二居民委员会",
                code: "020",
            },
            VillageCode {
                name: "西街办事处第三居民委员会",
                code: "021",
            },
            VillageCode {
                name: "西街办事处第四居民委员会",
                code: "022",
            },
            VillageCode {
                name: "西街办事处第五居民委员会",
                code: "023",
            },
            VillageCode {
                name: "西街办事处第六居民委员会",
                code: "024",
            },
            VillageCode {
                name: "西街办事处第七居民委员会",
                code: "025",
            },
            VillageCode {
                name: "南山办事处第一居民委员会",
                code: "026",
            },
            VillageCode {
                name: "南山办事处第二居民委员会",
                code: "027",
            },
            VillageCode {
                name: "南山办事处第三居民委员会",
                code: "028",
            },
            VillageCode {
                name: "南山办事处第四居民委员会",
                code: "029",
            },
            VillageCode {
                name: "南山办事处第五居民委员会",
                code: "030",
            },
            VillageCode {
                name: "七星村委会",
                code: "031",
            },
            VillageCode {
                name: "新发村委会",
                code: "032",
            },
            VillageCode {
                name: "振兴村委会",
                code: "033",
            },
            VillageCode {
                name: "红升村委会",
                code: "034",
            },
            VillageCode {
                name: "红旗村委会",
                code: "035",
            },
            VillageCode {
                name: "六合村委会",
                code: "036",
            },
            VillageCode {
                name: "友谊村委会",
                code: "037",
            },
            VillageCode {
                name: "东丰村委会",
                code: "038",
            },
            VillageCode {
                name: "东关村委会",
                code: "039",
            },
            VillageCode {
                name: "团结村委会",
                code: "040",
            },
            VillageCode {
                name: "镇东村委会",
                code: "041",
            },
            VillageCode {
                name: "镇北村委会",
                code: "042",
            },
            VillageCode {
                name: "镇西村委会",
                code: "043",
            },
            VillageCode {
                name: "兴华村委会",
                code: "044",
            },
            VillageCode {
                name: "阜龙村委会",
                code: "045",
            },
        ],
    },
    TownCode {
        name: "古城镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "一村委会",
                code: "001",
            },
            VillageCode {
                name: "二村委会",
                code: "002",
            },
            VillageCode {
                name: "三村委会",
                code: "003",
            },
            VillageCode {
                name: "四村委会",
                code: "004",
            },
            VillageCode {
                name: "五村委会",
                code: "005",
            },
            VillageCode {
                name: "新立村委会",
                code: "006",
            },
            VillageCode {
                name: "乌斯混村委会",
                code: "007",
            },
            VillageCode {
                name: "德安村委会",
                code: "008",
            },
            VillageCode {
                name: "湖一村委会",
                code: "009",
            },
            VillageCode {
                name: "前进村委会",
                code: "010",
            },
            VillageCode {
                name: "马路村委会",
                code: "011",
            },
            VillageCode {
                name: "长安村委会",
                code: "012",
            },
            VillageCode {
                name: "安民村委会",
                code: "013",
            },
            VillageCode {
                name: "河北村委会",
                code: "014",
            },
            VillageCode {
                name: "沿河村委会",
                code: "015",
            },
            VillageCode {
                name: "红石村委会",
                code: "016",
            },
            VillageCode {
                name: "四间房村委会",
                code: "017",
            },
            VillageCode {
                name: "湖水二村委会",
                code: "018",
            },
            VillageCode {
                name: "湖北村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "刁翎镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "长青村委会",
                code: "001",
            },
            VillageCode {
                name: "永安村委会",
                code: "002",
            },
            VillageCode {
                name: "保安村委会",
                code: "003",
            },
            VillageCode {
                name: "治安村委会",
                code: "004",
            },
            VillageCode {
                name: "东沟村委会",
                code: "005",
            },
            VillageCode {
                name: "四合村委会",
                code: "006",
            },
            VillageCode {
                name: "三家子村委会",
                code: "007",
            },
            VillageCode {
                name: "东岗子村委会",
                code: "008",
            },
            VillageCode {
                name: "下马蹄村委会",
                code: "009",
            },
            VillageCode {
                name: "上马蹄村委会",
                code: "010",
            },
            VillageCode {
                name: "样子沟村委会",
                code: "011",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "012",
            },
            VillageCode {
                name: "原发村委会",
                code: "013",
            },
            VillageCode {
                name: "东发村委会",
                code: "014",
            },
            VillageCode {
                name: "德胜村委会",
                code: "015",
            },
            VillageCode {
                name: "胜利村委会",
                code: "016",
            },
            VillageCode {
                name: "双丰村委会",
                code: "017",
            },
            VillageCode {
                name: "二道村委会",
                code: "018",
            },
            VillageCode {
                name: "双发村委会",
                code: "019",
            },
            VillageCode {
                name: "东风村委会",
                code: "020",
            },
            VillageCode {
                name: "黑背村委会",
                code: "021",
            },
            VillageCode {
                name: "生产村委会",
                code: "022",
            },
            VillageCode {
                name: "中合村委会",
                code: "023",
            },
            VillageCode {
                name: "新合村委会",
                code: "024",
            },
            VillageCode {
                name: "五七村委会",
                code: "025",
            },
            VillageCode {
                name: "互利村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "朱家镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "三合村委会",
                code: "001",
            },
            VillageCode {
                name: "万家村委会",
                code: "002",
            },
            VillageCode {
                name: "大碱村委会",
                code: "003",
            },
            VillageCode {
                name: "小碱村委会",
                code: "004",
            },
            VillageCode {
                name: "牛心村委会",
                code: "005",
            },
            VillageCode {
                name: "解放村委会",
                code: "006",
            },
            VillageCode {
                name: "站前村委会",
                code: "007",
            },
            VillageCode {
                name: "兴丰村委会",
                code: "008",
            },
            VillageCode {
                name: "仙洞村委会",
                code: "009",
            },
            VillageCode {
                name: "太安村委会",
                code: "010",
            },
            VillageCode {
                name: "富家村委会",
                code: "011",
            },
            VillageCode {
                name: "朱家村委会",
                code: "012",
            },
            VillageCode {
                name: "新安村委会",
                code: "013",
            },
            VillageCode {
                name: "新兴村委会",
                code: "014",
            },
            VillageCode {
                name: "碱北村委会",
                code: "015",
            },
            VillageCode {
                name: "良种场村委会",
                code: "016",
            },
            VillageCode {
                name: "新胜村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "柳树镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "柞木村委会",
                code: "001",
            },
            VillageCode {
                name: "双河村委会",
                code: "002",
            },
            VillageCode {
                name: "戛库村委会",
                code: "003",
            },
            VillageCode {
                name: "柳西村委会",
                code: "004",
            },
            VillageCode {
                name: "柳新村委会",
                code: "005",
            },
            VillageCode {
                name: "万寿村委会",
                code: "006",
            },
            VillageCode {
                name: "复兴村委会",
                code: "007",
            },
            VillageCode {
                name: "柳树村委会",
                code: "008",
            },
            VillageCode {
                name: "榆树村委会",
                code: "009",
            },
            VillageCode {
                name: "土甸子村委会",
                code: "010",
            },
            VillageCode {
                name: "柳毛村委会",
                code: "011",
            },
            VillageCode {
                name: "柳宝村委会",
                code: "012",
            },
            VillageCode {
                name: "宝山村委会",
                code: "013",
            },
            VillageCode {
                name: "三道村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "三道通镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "江东村委会",
                code: "001",
            },
            VillageCode {
                name: "江南村委会",
                code: "002",
            },
            VillageCode {
                name: "一村委会",
                code: "003",
            },
            VillageCode {
                name: "二村委会",
                code: "004",
            },
            VillageCode {
                name: "长胜村委会",
                code: "005",
            },
            VillageCode {
                name: "四道村委会",
                code: "006",
            },
            VillageCode {
                name: "署光村委会",
                code: "007",
            },
            VillageCode {
                name: "新建村委会",
                code: "008",
            },
            VillageCode {
                name: "新青村委会",
                code: "009",
            },
            VillageCode {
                name: "五道村委会",
                code: "010",
            },
            VillageCode {
                name: "大屯村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "龙爪镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "龙爪村委会",
                code: "001",
            },
            VillageCode {
                name: "暖泉村委会",
                code: "002",
            },
            VillageCode {
                name: "湾龙村委会",
                code: "003",
            },
            VillageCode {
                name: "山东会村委会",
                code: "004",
            },
            VillageCode {
                name: "向阳村委会",
                code: "005",
            },
            VillageCode {
                name: "红林村委会",
                code: "006",
            },
            VillageCode {
                name: "泉眼村委会",
                code: "007",
            },
            VillageCode {
                name: "宝林村委会",
                code: "008",
            },
            VillageCode {
                name: "绿山村委会",
                code: "009",
            },
            VillageCode {
                name: "高云村委会",
                code: "010",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "011",
            },
            VillageCode {
                name: "民主村委会",
                code: "012",
            },
            VillageCode {
                name: "龙丰村委会",
                code: "013",
            },
            VillageCode {
                name: "植场村委会",
                code: "014",
            },
            VillageCode {
                name: "小龙爪村委会",
                code: "015",
            },
            VillageCode {
                name: "保安村委会",
                code: "016",
            },
            VillageCode {
                name: "新龙爪村委会",
                code: "017",
            },
            VillageCode {
                name: "合发村委会",
                code: "018",
            },
            VillageCode {
                name: "楚山村委会",
                code: "019",
            },
            VillageCode {
                name: "龙山村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "莲花镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "江西村委会",
                code: "001",
            },
            VillageCode {
                name: "柳树村委会",
                code: "002",
            },
            VillageCode {
                name: "莲花村委会",
                code: "003",
            },
            VillageCode {
                name: "东河村委会",
                code: "004",
            },
            VillageCode {
                name: "东柳村委会",
                code: "005",
            },
            VillageCode {
                name: "字砬子村委会",
                code: "006",
            },
            VillageCode {
                name: "新富村委会",
                code: "007",
            },
            VillageCode {
                name: "大发村委会",
                code: "008",
            },
            VillageCode {
                name: "新民村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "青山镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "立民村委会",
                code: "001",
            },
            VillageCode {
                name: "亚河村委会",
                code: "002",
            },
            VillageCode {
                name: "亚东村委会",
                code: "003",
            },
            VillageCode {
                name: "永河村委会",
                code: "004",
            },
            VillageCode {
                name: "合乐村委会",
                code: "005",
            },
            VillageCode {
                name: "青山村委会",
                code: "006",
            },
            VillageCode {
                name: "青发村委会",
                code: "007",
            },
            VillageCode {
                name: "青平村委会",
                code: "008",
            },
            VillageCode {
                name: "小二龙村委会",
                code: "009",
            },
            VillageCode {
                name: "大二龙村委会",
                code: "010",
            },
            VillageCode {
                name: "新合村委会",
                code: "011",
            },
            VillageCode {
                name: "虎山村委会",
                code: "012",
            },
            VillageCode {
                name: "联合村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "建堂镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "马桥河村委会",
                code: "001",
            },
            VillageCode {
                name: "西北楞村委会",
                code: "002",
            },
            VillageCode {
                name: "山河村委会",
                code: "003",
            },
            VillageCode {
                name: "小盘道村委会",
                code: "004",
            },
            VillageCode {
                name: "大盘道村委会",
                code: "005",
            },
            VillageCode {
                name: "靠山村委会",
                code: "006",
            },
            VillageCode {
                name: "东兴村委会",
                code: "007",
            },
            VillageCode {
                name: "北兴村委会",
                code: "008",
            },
            VillageCode {
                name: "永进村委会",
                code: "009",
            },
            VillageCode {
                name: "河西村委会",
                code: "010",
            },
            VillageCode {
                name: "通沟村委会",
                code: "011",
            },
            VillageCode {
                name: "土城子村委会",
                code: "012",
            },
            VillageCode {
                name: "河兴村委会",
                code: "013",
            },
            VillageCode {
                name: "大百顺村委会",
                code: "014",
            },
            VillageCode {
                name: "小百顺村委会",
                code: "015",
            },
            VillageCode {
                name: "红旗村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "奎山镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "奎山村委会",
                code: "001",
            },
            VillageCode {
                name: "双龙村委会",
                code: "002",
            },
            VillageCode {
                name: "安乐村委会",
                code: "003",
            },
            VillageCode {
                name: "余庆村委会",
                code: "004",
            },
            VillageCode {
                name: "吉庆村委会",
                code: "005",
            },
            VillageCode {
                name: "安山村委会",
                code: "006",
            },
            VillageCode {
                name: "共禾村委会",
                code: "007",
            },
            VillageCode {
                name: "永安村委会",
                code: "008",
            },
            VillageCode {
                name: "太平村委会",
                code: "009",
            },
            VillageCode {
                name: "马鞍山村委会",
                code: "010",
            },
            VillageCode {
                name: "前杨木村委会",
                code: "011",
            },
            VillageCode {
                name: "后杨木村委会",
                code: "012",
            },
            VillageCode {
                name: "庆岭村委会",
                code: "013",
            },
            VillageCode {
                name: "中山阳村委会",
                code: "014",
            },
            VillageCode {
                name: "上山阳村委会",
                code: "015",
            },
            VillageCode {
                name: "长丰村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "林口林业局",
        code: "012",
        villages: &[
            VillageCode {
                name: "中南区居委会",
                code: "001",
            },
            VillageCode {
                name: "中北区居委会",
                code: "002",
            },
            VillageCode {
                name: "河北区居委会",
                code: "003",
            },
            VillageCode {
                name: "河西区居委会",
                code: "004",
            },
            VillageCode {
                name: "四道经营所居委会",
                code: "005",
            },
            VillageCode {
                name: "胜利经营所居委会",
                code: "006",
            },
            VillageCode {
                name: "向阳生活服务公司居委会",
                code: "007",
            },
            VillageCode {
                name: "湖水经营所居委会",
                code: "008",
            },
            VillageCode {
                name: "青山经营所居委会",
                code: "009",
            },
            VillageCode {
                name: "曙光经营所居委会",
                code: "010",
            },
            VillageCode {
                name: "红石经营所居委会",
                code: "011",
            },
            VillageCode {
                name: "莲花经营所居委会",
                code: "012",
            },
            VillageCode {
                name: "西北楞经营所居委会",
                code: "013",
            },
            VillageCode {
                name: "团结经营所居委会",
                code: "014",
            },
            VillageCode {
                name: "前哨经营所居委会",
                code: "015",
            },
            VillageCode {
                name: "刁翎经营所居委会",
                code: "016",
            },
            VillageCode {
                name: "战斗林场生活区",
                code: "017",
            },
            VillageCode {
                name: "奋斗林场生活区",
                code: "018",
            },
            VillageCode {
                name: "东升林场生活区",
                code: "019",
            },
            VillageCode {
                name: "朝阳林场生活区",
                code: "020",
            },
            VillageCode {
                name: "先锋林场生活区",
                code: "021",
            },
        ],
    },
];

static TOWNS_HJ_037: [TownCode; 2] = [
    TownCode {
        name: "绥芬河镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "友谊社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新华社区居委会",
                code: "002",
            },
            VillageCode {
                name: "永安社区居委会",
                code: "003",
            },
            VillageCode {
                name: "山城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "前进社区居委会",
                code: "005",
            },
            VillageCode {
                name: "北海社区居委会",
                code: "006",
            },
            VillageCode {
                name: "光华社区居委会",
                code: "007",
            },
            VillageCode {
                name: "三合林社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新兴社区居委会",
                code: "009",
            },
            VillageCode {
                name: "绥兴社区",
                code: "010",
            },
            VillageCode {
                name: "铁西社区居委会",
                code: "011",
            },
            VillageCode {
                name: "前进村委会",
                code: "012",
            },
            VillageCode {
                name: "绥东村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "阜宁镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "建西社区居委会",
                code: "001",
            },
            VillageCode {
                name: "阜华社区居委会",
                code: "002",
            },
            VillageCode {
                name: "建南社区居委会",
                code: "003",
            },
            VillageCode {
                name: "旗苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "谷盈社区",
                code: "005",
            },
            VillageCode {
                name: "建西村委会",
                code: "006",
            },
            VillageCode {
                name: "建东村委会",
                code: "007",
            },
            VillageCode {
                name: "建华村委会",
                code: "008",
            },
            VillageCode {
                name: "建新村委会",
                code: "009",
            },
            VillageCode {
                name: "南寒村委会",
                code: "010",
            },
            VillageCode {
                name: "北寒村委会",
                code: "011",
            },
            VillageCode {
                name: "永胜村委会",
                code: "012",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "013",
            },
            VillageCode {
                name: "宽沟村委会",
                code: "014",
            },
        ],
    },
];

static TOWNS_HJ_038: [TownCode; 15] = [
    TownCode {
        name: "海林镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "丽海社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新兴社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "子荣社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "朝阳社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "英雄社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "海浪社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "开发区（城北新区）社区居委会",
                code: "007",
            },
            VillageCode {
                name: "林海社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "方兴社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "广场社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "海丰朝鲜族社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "团结社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "模范村委会",
                code: "013",
            },
            VillageCode {
                name: "新海村委会",
                code: "014",
            },
            VillageCode {
                name: "光荣村委会",
                code: "015",
            },
            VillageCode {
                name: "共和村委会",
                code: "016",
            },
            VillageCode {
                name: "蔬菜村委会",
                code: "017",
            },
            VillageCode {
                name: "江头村委会",
                code: "018",
            },
            VillageCode {
                name: "富强村委会",
                code: "019",
            },
            VillageCode {
                name: "秦家村委会",
                code: "020",
            },
            VillageCode {
                name: "永安村委会",
                code: "021",
            },
            VillageCode {
                name: "泡子村委会",
                code: "022",
            },
            VillageCode {
                name: "林山村委会",
                code: "023",
            },
            VillageCode {
                name: "东德家村委会",
                code: "024",
            },
            VillageCode {
                name: "斗银村委会",
                code: "025",
            },
            VillageCode {
                name: "新民村委会",
                code: "026",
            },
            VillageCode {
                name: "红光村委会",
                code: "027",
            },
            VillageCode {
                name: "三合村委会",
                code: "028",
            },
            VillageCode {
                name: "安乐村委会",
                code: "029",
            },
            VillageCode {
                name: "马北村委会",
                code: "030",
            },
            VillageCode {
                name: "大石头村委会",
                code: "031",
            },
            VillageCode {
                name: "永丰村委会",
                code: "032",
            },
            VillageCode {
                name: "新合村委会",
                code: "033",
            },
            VillageCode {
                name: "平和村委会",
                code: "034",
            },
            VillageCode {
                name: "五星村委会",
                code: "035",
            },
            VillageCode {
                name: "江北村委会",
                code: "036",
            },
            VillageCode {
                name: "石河村委会",
                code: "037",
            },
            VillageCode {
                name: "石东村委会",
                code: "038",
            },
            VillageCode {
                name: "文明村委会",
                code: "039",
            },
            VillageCode {
                name: "大岭村委会",
                code: "040",
            },
            VillageCode {
                name: "西德家村委会",
                code: "041",
            },
            VillageCode {
                name: "密南村委会",
                code: "042",
            },
            VillageCode {
                name: "庙山村委会",
                code: "043",
            },
            VillageCode {
                name: "卢家村委会",
                code: "044",
            },
            VillageCode {
                name: "德家林场生活区",
                code: "045",
            },
            VillageCode {
                name: "新海林场生活区",
                code: "046",
            },
            VillageCode {
                name: "海林林场生活区",
                code: "047",
            },
        ],
    },
    TownCode {
        name: "长汀镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "长青社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "长兴社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "双桥村委会",
                code: "003",
            },
            VillageCode {
                name: "火龙沟村委会",
                code: "004",
            },
            VillageCode {
                name: "平安村委会",
                code: "005",
            },
            VillageCode {
                name: "河北村委会",
                code: "006",
            },
            VillageCode {
                name: "七山村委会",
                code: "007",
            },
            VillageCode {
                name: "猴石村委会",
                code: "008",
            },
            VillageCode {
                name: "万丈村委会",
                code: "009",
            },
            VillageCode {
                name: "马场村委会",
                code: "010",
            },
            VillageCode {
                name: "双丰村委会",
                code: "011",
            },
            VillageCode {
                name: "哈达村委会",
                code: "012",
            },
            VillageCode {
                name: "杨林村委会",
                code: "013",
            },
            VillageCode {
                name: "张明村委会",
                code: "014",
            },
            VillageCode {
                name: "古塔村委会",
                code: "015",
            },
            VillageCode {
                name: "宁古村委会",
                code: "016",
            },
            VillageCode {
                name: "南沟村委会",
                code: "017",
            },
            VillageCode {
                name: "卜家村委会",
                code: "018",
            },
            VillageCode {
                name: "满城村委会",
                code: "019",
            },
            VillageCode {
                name: "古城村委会",
                code: "020",
            },
            VillageCode {
                name: "红海林林场生活区",
                code: "021",
            },
            VillageCode {
                name: "火龙沟林场生活区",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "横道镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "佛手社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "老街社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "二十二村委会",
                code: "003",
            },
            VillageCode {
                name: "顺桥村委会",
                code: "004",
            },
            VillageCode {
                name: "七里地村委会",
                code: "005",
            },
            VillageCode {
                name: "正南村委会",
                code: "006",
            },
            VillageCode {
                name: "道林村委会",
                code: "007",
            },
            VillageCode {
                name: "柳树村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "山市镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "山市社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东街村委会",
                code: "002",
            },
            VillageCode {
                name: "西街村委会",
                code: "003",
            },
            VillageCode {
                name: "道南村委会",
                code: "004",
            },
            VillageCode {
                name: "东光村委会",
                code: "005",
            },
            VillageCode {
                name: "二洼村委会",
                code: "006",
            },
            VillageCode {
                name: "长胜村委会",
                code: "007",
            },
            VillageCode {
                name: "洋草村委会",
                code: "008",
            },
            VillageCode {
                name: "锦山村委会",
                code: "009",
            },
            VillageCode {
                name: "奇峰村委会",
                code: "010",
            },
            VillageCode {
                name: "胜利村委会",
                code: "011",
            },
            VillageCode {
                name: "新兴村委会",
                code: "012",
            },
            VillageCode {
                name: "青岭子村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "柴河镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "友谊社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "光明社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "江滨社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "长石村委会",
                code: "004",
            },
            VillageCode {
                name: "柴河村委会",
                code: "005",
            },
            VillageCode {
                name: "佛塔密村委会",
                code: "006",
            },
            VillageCode {
                name: "东风村委会",
                code: "007",
            },
            VillageCode {
                name: "临江村委会",
                code: "008",
            },
            VillageCode {
                name: "头道河子村委会",
                code: "009",
            },
            VillageCode {
                name: "北站村委会",
                code: "010",
            },
            VillageCode {
                name: "群力村委会",
                code: "011",
            },
            VillageCode {
                name: "黑牛背村委会",
                code: "012",
            },
            VillageCode {
                name: "阳光村委会",
                code: "013",
            },
            VillageCode {
                name: "柴河国营林场生活区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "二道镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "二道社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "西沙村委会",
                code: "002",
            },
            VillageCode {
                name: "东沙村委会",
                code: "003",
            },
            VillageCode {
                name: "二站村委会",
                code: "004",
            },
            VillageCode {
                name: "三站村委会",
                code: "005",
            },
            VillageCode {
                name: "向日村委会",
                code: "006",
            },
            VillageCode {
                name: "老家村委会",
                code: "007",
            },
            VillageCode {
                name: "北宁村委会",
                code: "008",
            },
            VillageCode {
                name: "永兴村委会",
                code: "009",
            },
            VillageCode {
                name: "钓鱼台村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "新安朝鲜族镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "新安社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "和平村委会",
                code: "002",
            },
            VillageCode {
                name: "永乐村委会",
                code: "003",
            },
            VillageCode {
                name: "再兴村委会",
                code: "004",
            },
            VillageCode {
                name: "西安村委会",
                code: "005",
            },
            VillageCode {
                name: "中和村委会",
                code: "006",
            },
            VillageCode {
                name: "三家子村委会",
                code: "007",
            },
            VillageCode {
                name: "共济村委会",
                code: "008",
            },
            VillageCode {
                name: "密江村委会",
                code: "009",
            },
            VillageCode {
                name: "山咀子村委会",
                code: "010",
            },
            VillageCode {
                name: "东和村委会",
                code: "011",
            },
            VillageCode {
                name: "新安村委会",
                code: "012",
            },
            VillageCode {
                name: "复兴村委会",
                code: "013",
            },
            VillageCode {
                name: "岭后村委会",
                code: "014",
            },
            VillageCode {
                name: "北崴子村委会",
                code: "015",
            },
            VillageCode {
                name: "北沟村委会",
                code: "016",
            },
            VillageCode {
                name: "友谊村委会",
                code: "017",
            },
            VillageCode {
                name: "小三家子村委会",
                code: "018",
            },
            VillageCode {
                name: "海林市农委良种场生活区",
                code: "019",
            },
            VillageCode {
                name: "海林市农委种畜场生活区",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "三道镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "三道社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "边安村委会",
                code: "002",
            },
            VillageCode {
                name: "振兴村委会",
                code: "003",
            },
            VillageCode {
                name: "东升村委会",
                code: "004",
            },
            VillageCode {
                name: "兴家村委会",
                code: "005",
            },
            VillageCode {
                name: "双兴村委会",
                code: "006",
            },
            VillageCode {
                name: "木兰村委会",
                code: "007",
            },
            VillageCode {
                name: "春光村委会",
                code: "008",
            },
            VillageCode {
                name: "工农村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "牡林工程公司街道办事处",
        code: "009",
        villages: &[VillageCode {
            name: "新华虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "柴河林机厂街道办事处",
        code: "010",
        villages: &[VillageCode {
            name: "林机虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "大海林林业局",
        code: "011",
        villages: &[
            VillageCode {
                name: "林北社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "岭上社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "林园社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "塔东社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "东川社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "海源林场生活区",
                code: "006",
            },
            VillageCode {
                name: "海浪林场生活区",
                code: "007",
            },
            VillageCode {
                name: "前进林场生活区",
                code: "008",
            },
            VillageCode {
                name: "双峰景区生活区",
                code: "009",
            },
            VillageCode {
                name: "双峰林场生活区",
                code: "010",
            },
            VillageCode {
                name: "杨木沟林场生活区",
                code: "011",
            },
            VillageCode {
                name: "太平沟林场生活区",
                code: "012",
            },
            VillageCode {
                name: "柳河林场生活区",
                code: "013",
            },
            VillageCode {
                name: "七峰林场生活区",
                code: "014",
            },
            VillageCode {
                name: "兴农林场生活区",
                code: "015",
            },
            VillageCode {
                name: "青平林场生活区",
                code: "016",
            },
            VillageCode {
                name: "梨树沟林场生活区",
                code: "017",
            },
            VillageCode {
                name: "二浪河林场生活区",
                code: "018",
            },
            VillageCode {
                name: "西南岔林场生活区",
                code: "019",
            },
            VillageCode {
                name: "新林林场生活区",
                code: "020",
            },
            VillageCode {
                name: "新林村林场生活区",
                code: "021",
            },
            VillageCode {
                name: "青云山林场生活区",
                code: "022",
            },
            VillageCode {
                name: "红旗林场生活区",
                code: "023",
            },
            VillageCode {
                name: "红星林场生活区",
                code: "024",
            },
            VillageCode {
                name: "长汀林场生活区",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "海林林业局",
        code: "012",
        villages: &[
            VillageCode {
                name: "北山社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "林苑社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "兴林社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "三十五林场生活区",
                code: "004",
            },
            VillageCode {
                name: "三部落林场生活区",
                code: "005",
            },
            VillageCode {
                name: "夹皮沟林场生活区",
                code: "006",
            },
            VillageCode {
                name: "五十八林场生活区",
                code: "007",
            },
            VillageCode {
                name: "二十二林场生活区",
                code: "008",
            },
            VillageCode {
                name: "大石沟林场生活区",
                code: "009",
            },
            VillageCode {
                name: "横道林场生活区",
                code: "010",
            },
            VillageCode {
                name: "治山林场生活区",
                code: "011",
            },
            VillageCode {
                name: "道林林场生活区",
                code: "012",
            },
            VillageCode {
                name: "青岭子林场生活区",
                code: "013",
            },
            VillageCode {
                name: "山市林场生活区",
                code: "014",
            },
            VillageCode {
                name: "奋斗林场生活区",
                code: "015",
            },
            VillageCode {
                name: "密江林场生活区",
                code: "016",
            },
            VillageCode {
                name: "石河林场生活区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "柴河林业局",
        code: "013",
        villages: &[
            VillageCode {
                name: "江滨社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "铁东路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "阳明社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "南岗路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "东风林场生活区",
                code: "005",
            },
            VillageCode {
                name: "晨光林场生活区",
                code: "006",
            },
            VillageCode {
                name: "桦木林场生活区",
                code: "007",
            },
            VillageCode {
                name: "大青林场生活区",
                code: "008",
            },
            VillageCode {
                name: "三块石林场生活区",
                code: "009",
            },
            VillageCode {
                name: "板桥子林场生活区",
                code: "010",
            },
            VillageCode {
                name: "临江林场生活区",
                code: "011",
            },
            VillageCode {
                name: "莲花林场生活区",
                code: "012",
            },
            VillageCode {
                name: "向日林场生活区",
                code: "013",
            },
            VillageCode {
                name: "秋皮沟林场生活区",
                code: "014",
            },
            VillageCode {
                name: "细林河林场生活区",
                code: "015",
            },
            VillageCode {
                name: "柳毛河林场生活区",
                code: "016",
            },
            VillageCode {
                name: "红光林场生活区",
                code: "017",
            },
            VillageCode {
                name: "红星林场生活区",
                code: "018",
            },
            VillageCode {
                name: "卫星林场生活区",
                code: "019",
            },
            VillageCode {
                name: "新兴林场生活区",
                code: "020",
            },
            VillageCode {
                name: "宏声林场生活区",
                code: "021",
            },
            VillageCode {
                name: "二道林场生活区",
                code: "022",
            },
            VillageCode {
                name: "群力林场生活区",
                code: "023",
            },
            VillageCode {
                name: "黑牛背林场生活区",
                code: "024",
            },
            VillageCode {
                name: "头道林场生活区",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "海林农场",
        code: "014",
        villages: &[
            VillageCode {
                name: "海林的场直社区",
                code: "001",
            },
            VillageCode {
                name: "海林农场农业管理区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "山市种奶牛场",
        code: "015",
        villages: &[
            VillageCode {
                name: "山市场直社区",
                code: "001",
            },
            VillageCode {
                name: "第一作业区生活区",
                code: "002",
            },
            VillageCode {
                name: "第二作业区生活区",
                code: "003",
            },
            VillageCode {
                name: "第三作业区生活区",
                code: "004",
            },
            VillageCode {
                name: "第四作业区生活区",
                code: "005",
            },
        ],
    },
];

static TOWNS_HJ_039: [TownCode; 15] = [
    TownCode {
        name: "城区街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "文庙社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东关社区居委会",
                code: "002",
            },
            VillageCode {
                name: "乐园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "西关社区居委会",
                code: "004",
            },
            VillageCode {
                name: "虹桥社区居委会",
                code: "005",
            },
            VillageCode {
                name: "铁工社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "宁安镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "利民村委会",
                code: "001",
            },
            VillageCode {
                name: "伊家村委会",
                code: "002",
            },
            VillageCode {
                name: "河西村委会",
                code: "003",
            },
            VillageCode {
                name: "红城村委会",
                code: "004",
            },
            VillageCode {
                name: "临城村委会",
                code: "005",
            },
            VillageCode {
                name: "临江村委会",
                code: "006",
            },
            VillageCode {
                name: "红升村委会",
                code: "007",
            },
            VillageCode {
                name: "新胜村委会",
                code: "008",
            },
            VillageCode {
                name: "向阳村委会",
                code: "009",
            },
            VillageCode {
                name: "兴盛村委会",
                code: "010",
            },
            VillageCode {
                name: "兴林村委会",
                code: "011",
            },
            VillageCode {
                name: "柳林村委会",
                code: "012",
            },
            VillageCode {
                name: "教育村委会",
                code: "013",
            },
            VillageCode {
                name: "三合村委会",
                code: "014",
            },
            VillageCode {
                name: "长江村委会",
                code: "015",
            },
            VillageCode {
                name: "联合村委会",
                code: "016",
            },
            VillageCode {
                name: "双桥子村委会",
                code: "017",
            },
            VillageCode {
                name: "黄旗沟村委会",
                code: "018",
            },
            VillageCode {
                name: "上赊里村委会",
                code: "019",
            },
            VillageCode {
                name: "福荣村委会",
                code: "020",
            },
            VillageCode {
                name: "范家村委会",
                code: "021",
            },
            VillageCode {
                name: "茂盛村委会",
                code: "022",
            },
            VillageCode {
                name: "共和村委会",
                code: "023",
            },
            VillageCode {
                name: "葡萄沟村委会",
                code: "024",
            },
            VillageCode {
                name: "江南村委会",
                code: "025",
            },
            VillageCode {
                name: "新安村委会",
                code: "026",
            },
            VillageCode {
                name: "镇江村委会",
                code: "027",
            },
            VillageCode {
                name: "小唐村委会",
                code: "028",
            },
            VillageCode {
                name: "黄旗村委会",
                code: "029",
            },
            VillageCode {
                name: "张家村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "东京城镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "和平社区居委会",
                code: "001",
            },
            VillageCode {
                name: "铁东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "建园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "于家村委会",
                code: "004",
            },
            VillageCode {
                name: "镇兴村委会",
                code: "005",
            },
            VillageCode {
                name: "糖坊村委会",
                code: "006",
            },
            VillageCode {
                name: "东京村委会",
                code: "007",
            },
            VillageCode {
                name: "中马河村委会",
                code: "008",
            },
            VillageCode {
                name: "牛场村委会",
                code: "009",
            },
            VillageCode {
                name: "兴安村委会",
                code: "010",
            },
            VillageCode {
                name: "光明村委会",
                code: "011",
            },
            VillageCode {
                name: "下窨子村委会",
                code: "012",
            },
            VillageCode {
                name: "东康村委会",
                code: "013",
            },
            VillageCode {
                name: "下马河村委会",
                code: "014",
            },
            VillageCode {
                name: "大荒地村委会",
                code: "015",
            },
            VillageCode {
                name: "哈达村委会",
                code: "016",
            },
            VillageCode {
                name: "烽火村委会",
                code: "017",
            },
            VillageCode {
                name: "红兴村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "渤海镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "新园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "古都社区居委会",
                code: "002",
            },
            VillageCode {
                name: "渤海村委会",
                code: "003",
            },
            VillageCode {
                name: "上京村委会",
                code: "004",
            },
            VillageCode {
                name: "龙泉村委会",
                code: "005",
            },
            VillageCode {
                name: "双庙子村委会",
                code: "006",
            },
            VillageCode {
                name: "西地村委会",
                code: "007",
            },
            VillageCode {
                name: "白庙子村委会",
                code: "008",
            },
            VillageCode {
                name: "拐弯村委会",
                code: "009",
            },
            VillageCode {
                name: "土台子村委会",
                code: "010",
            },
            VillageCode {
                name: "江西村委会",
                code: "011",
            },
            VillageCode {
                name: "响水村委会",
                code: "012",
            },
            VillageCode {
                name: "瀑布村委会",
                code: "013",
            },
            VillageCode {
                name: "上官地村委会",
                code: "014",
            },
            VillageCode {
                name: "东珠村委会",
                code: "015",
            },
            VillageCode {
                name: "莲花一村委会",
                code: "016",
            },
            VillageCode {
                name: "莲花二村委会",
                code: "017",
            },
            VillageCode {
                name: "莲花三村委会",
                code: "018",
            },
            VillageCode {
                name: "杏山村委会",
                code: "019",
            },
            VillageCode {
                name: "富安村委会",
                code: "020",
            },
            VillageCode {
                name: "小三家子村委会",
                code: "021",
            },
            VillageCode {
                name: "大三家子村委会",
                code: "022",
            },
            VillageCode {
                name: "太平沟村委会",
                code: "023",
            },
            VillageCode {
                name: "梁家村委会",
                code: "024",
            },
            VillageCode {
                name: "小朱家村委会",
                code: "025",
            },
            VillageCode {
                name: "上屯村委会",
                code: "026",
            },
            VillageCode {
                name: "湖沿村委会",
                code: "027",
            },
            VillageCode {
                name: "湖北村委会",
                code: "028",
            },
            VillageCode {
                name: "青山村委会",
                code: "029",
            },
            VillageCode {
                name: "繁荣村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "石岩镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "石岩村委会",
                code: "001",
            },
            VillageCode {
                name: "幸福村委会",
                code: "002",
            },
            VillageCode {
                name: "四合村委会",
                code: "003",
            },
            VillageCode {
                name: "爱路村委会",
                code: "004",
            },
            VillageCode {
                name: "前进村委会",
                code: "005",
            },
            VillageCode {
                name: "拥军村委会",
                code: "006",
            },
            VillageCode {
                name: "东和村委会",
                code: "007",
            },
            VillageCode {
                name: "民主村委会",
                code: "008",
            },
            VillageCode {
                name: "团山子村委会",
                code: "009",
            },
            VillageCode {
                name: "腰岭村委会",
                code: "010",
            },
            VillageCode {
                name: "永富村委会",
                code: "011",
            },
            VillageCode {
                name: "平安村委会",
                code: "012",
            },
            VillageCode {
                name: "丰产村委会",
                code: "013",
            },
            VillageCode {
                name: "和平村委会",
                code: "014",
            },
            VillageCode {
                name: "东华村委会",
                code: "015",
            },
            VillageCode {
                name: "建设村委会",
                code: "016",
            },
            VillageCode {
                name: "乐园村委会",
                code: "017",
            },
            VillageCode {
                name: "民安村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "沙兰镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "治安村委会",
                code: "001",
            },
            VillageCode {
                name: "永明村委会",
                code: "002",
            },
            VillageCode {
                name: "长安村委会",
                code: "003",
            },
            VillageCode {
                name: "新富村委会",
                code: "004",
            },
            VillageCode {
                name: "进荣村委会",
                code: "005",
            },
            VillageCode {
                name: "二闾村委会",
                code: "006",
            },
            VillageCode {
                name: "桦树村委会",
                code: "007",
            },
            VillageCode {
                name: "二道沟村委会",
                code: "008",
            },
            VillageCode {
                name: "木其村委会",
                code: "009",
            },
            VillageCode {
                name: "王家村委会",
                code: "010",
            },
            VillageCode {
                name: "和盛村委会",
                code: "011",
            },
            VillageCode {
                name: "卧龙泉村委会",
                code: "012",
            },
            VillageCode {
                name: "三块石村委会",
                code: "013",
            },
            VillageCode {
                name: "阎家村委会",
                code: "014",
            },
            VillageCode {
                name: "王豆坊村委会",
                code: "015",
            },
            VillageCode {
                name: "井城村委会",
                code: "016",
            },
            VillageCode {
                name: "小荒地村委会",
                code: "017",
            },
            VillageCode {
                name: "同心村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "海浪镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "海浪村委会",
                code: "001",
            },
            VillageCode {
                name: "安青村委会",
                code: "002",
            },
            VillageCode {
                name: "高家村委会",
                code: "003",
            },
            VillageCode {
                name: "兰旗村委会",
                code: "004",
            },
            VillageCode {
                name: "牡北村委会",
                code: "005",
            },
            VillageCode {
                name: "太平村委会",
                code: "006",
            },
            VillageCode {
                name: "光荣村委会",
                code: "007",
            },
            VillageCode {
                name: "东炉村委会",
                code: "008",
            },
            VillageCode {
                name: "前阳村委会",
                code: "009",
            },
            VillageCode {
                name: "后地村委会",
                code: "010",
            },
            VillageCode {
                name: "敖东村委会",
                code: "011",
            },
            VillageCode {
                name: "二洼村委会",
                code: "012",
            },
            VillageCode {
                name: "五道梁子村委会",
                code: "013",
            },
            VillageCode {
                name: "羊草村委会",
                code: "014",
            },
            VillageCode {
                name: "安平村委会",
                code: "015",
            },
            VillageCode {
                name: "三道梁子村委会",
                code: "016",
            },
            VillageCode {
                name: "七道梁子村委会",
                code: "017",
            },
            VillageCode {
                name: "八道梁子村委会",
                code: "018",
            },
            VillageCode {
                name: "九道梁子村委会",
                code: "019",
            },
            VillageCode {
                name: "宁西村委会",
                code: "020",
            },
            VillageCode {
                name: "林富村委会",
                code: "021",
            },
            VillageCode {
                name: "盘岭村委会",
                code: "022",
            },
            VillageCode {
                name: "庆城村委会",
                code: "023",
            },
            VillageCode {
                name: "镇北村委会",
                code: "024",
            },
            VillageCode {
                name: "岔路村委会",
                code: "025",
            },
            VillageCode {
                name: "大依兰村委会",
                code: "026",
            },
            VillageCode {
                name: "大牡丹村委会",
                code: "027",
            },
            VillageCode {
                name: "小牡丹村委会",
                code: "028",
            },
            VillageCode {
                name: "长胜村委会",
                code: "029",
            },
            VillageCode {
                name: "大生村委会",
                code: "030",
            },
            VillageCode {
                name: "北山村委会",
                code: "031",
            },
            VillageCode {
                name: "依兰岗满族村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "兰岗镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "兰岗村委会",
                code: "001",
            },
            VillageCode {
                name: "文化村委会",
                code: "002",
            },
            VillageCode {
                name: "东升村委会",
                code: "003",
            },
            VillageCode {
                name: "新农村委会",
                code: "004",
            },
            VillageCode {
                name: "民和村委会",
                code: "005",
            },
            VillageCode {
                name: "牡丹村委会",
                code: "006",
            },
            VillageCode {
                name: "新中村委会",
                code: "007",
            },
            VillageCode {
                name: "依兰村委会",
                code: "008",
            },
            VillageCode {
                name: "自兴村委会",
                code: "009",
            },
            VillageCode {
                name: "永政村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "镜泊镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "镜泊村委会",
                code: "001",
            },
            VillageCode {
                name: "湾沟村委会",
                code: "002",
            },
            VillageCode {
                name: "复兴楼村委会",
                code: "003",
            },
            VillageCode {
                name: "后渔村委会",
                code: "004",
            },
            VillageCode {
                name: "庆丰村委会",
                code: "005",
            },
            VillageCode {
                name: "五峰楼村委会",
                code: "006",
            },
            VillageCode {
                name: "永丰村委会",
                code: "007",
            },
            VillageCode {
                name: "湖南村委会",
                code: "008",
            },
            VillageCode {
                name: "金家村委会",
                code: "009",
            },
            VillageCode {
                name: "褚家村委会",
                code: "010",
            },
            VillageCode {
                name: "北石村委会",
                code: "011",
            },
            VillageCode {
                name: "湖西村委会",
                code: "012",
            },
            VillageCode {
                name: "城子村委会",
                code: "013",
            },
            VillageCode {
                name: "良种场村委会",
                code: "014",
            },
            VillageCode {
                name: "东大泡村委会",
                code: "015",
            },
            VillageCode {
                name: "小夹吉河村委会",
                code: "016",
            },
            VillageCode {
                name: "江北村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "江南朝鲜族满族乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "新兴村委会",
                code: "001",
            },
            VillageCode {
                name: "新城村委会",
                code: "002",
            },
            VillageCode {
                name: "勇进村委会",
                code: "003",
            },
            VillageCode {
                name: "嘎斯村委会",
                code: "004",
            },
            VillageCode {
                name: "榆林村委会",
                code: "005",
            },
            VillageCode {
                name: "大唐村委会",
                code: "006",
            },
            VillageCode {
                name: "宝山村委会",
                code: "007",
            },
            VillageCode {
                name: "东安村委会",
                code: "008",
            },
            VillageCode {
                name: "四方村委会",
                code: "009",
            },
            VillageCode {
                name: "马家村委会",
                code: "010",
            },
            VillageCode {
                name: "东升村委会",
                code: "011",
            },
            VillageCode {
                name: "新顺村委会",
                code: "012",
            },
            VillageCode {
                name: "永胜村委会",
                code: "013",
            },
            VillageCode {
                name: "明星村委会",
                code: "014",
            },
            VillageCode {
                name: "永安村委会",
                code: "015",
            },
            VillageCode {
                name: "解放村委会",
                code: "016",
            },
            VillageCode {
                name: "双富村委会",
                code: "017",
            },
            VillageCode {
                name: "东兴村委会",
                code: "018",
            },
            VillageCode {
                name: "星光村委会",
                code: "019",
            },
            VillageCode {
                name: "缸窑村委会",
                code: "020",
            },
            VillageCode {
                name: "宁东村委会",
                code: "021",
            },
            VillageCode {
                name: "清泉村委会",
                code: "022",
            },
            VillageCode {
                name: "簸箕村委会",
                code: "023",
            },
            VillageCode {
                name: "永乐村委会",
                code: "024",
            },
            VillageCode {
                name: "福兴村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "卧龙朝鲜族乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "卧龙村委会",
                code: "001",
            },
            VillageCode {
                name: "罗城沟村委会",
                code: "002",
            },
            VillageCode {
                name: "前三家子村委会",
                code: "003",
            },
            VillageCode {
                name: "三道湾村委会",
                code: "004",
            },
            VillageCode {
                name: "新政村委会",
                code: "005",
            },
            VillageCode {
                name: "勤劳村委会",
                code: "006",
            },
            VillageCode {
                name: "明泉村委会",
                code: "007",
            },
            VillageCode {
                name: "爱林村委会",
                code: "008",
            },
            VillageCode {
                name: "杏花村委会",
                code: "009",
            },
            VillageCode {
                name: "英山村委会",
                code: "010",
            },
            VillageCode {
                name: "西岗子村委会",
                code: "011",
            },
            VillageCode {
                name: "三道河子村委会",
                code: "012",
            },
            VillageCode {
                name: "农场村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "马河乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "马莲河村委会",
                code: "001",
            },
            VillageCode {
                name: "新立村委会",
                code: "002",
            },
            VillageCode {
                name: "马河村委会",
                code: "003",
            },
            VillageCode {
                name: "红光村委会",
                code: "004",
            },
            VillageCode {
                name: "黎明一村委会",
                code: "005",
            },
            VillageCode {
                name: "金坑村委会",
                code: "006",
            },
            VillageCode {
                name: "跃进村委会",
                code: "007",
            },
            VillageCode {
                name: "东烧锅村委会",
                code: "008",
            },
            VillageCode {
                name: "后斗村委会",
                code: "009",
            },
            VillageCode {
                name: "前斗村委会",
                code: "010",
            },
            VillageCode {
                name: "头道村委会",
                code: "011",
            },
            VillageCode {
                name: "四道村委会",
                code: "012",
            },
            VillageCode {
                name: "鹿道村委会",
                code: "013",
            },
            VillageCode {
                name: "富路村委会",
                code: "014",
            },
            VillageCode {
                name: "松岭村委会",
                code: "015",
            },
            VillageCode {
                name: "五道河子村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "三陵乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "三陵村委会",
                code: "001",
            },
            VillageCode {
                name: "三星村委会",
                code: "002",
            },
            VillageCode {
                name: "南阳村委会",
                code: "003",
            },
            VillageCode {
                name: "西崴子村委会",
                code: "004",
            },
            VillageCode {
                name: "东沟村委会",
                code: "005",
            },
            VillageCode {
                name: "胡家村委会",
                code: "006",
            },
            VillageCode {
                name: "连家村委会",
                code: "007",
            },
            VillageCode {
                name: "贝家村委会",
                code: "008",
            },
            VillageCode {
                name: "爬梨沟村委会",
                code: "009",
            },
            VillageCode {
                name: "八家子村委会",
                code: "010",
            },
            VillageCode {
                name: "北湖村委会",
                code: "011",
            },
            VillageCode {
                name: "兴华村委会",
                code: "012",
            },
            VillageCode {
                name: "兴隆店村委会",
                code: "013",
            },
            VillageCode {
                name: "红旗村委会",
                code: "014",
            },
            VillageCode {
                name: "小兰旗沟村委会",
                code: "015",
            },
            VillageCode {
                name: "红土村委会",
                code: "016",
            },
            VillageCode {
                name: "泉眼沟村委会",
                code: "017",
            },
            VillageCode {
                name: "胡家沟村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "东京城林业局",
        code: "014",
        villages: &[
            VillageCode {
                name: "第一居委会",
                code: "001",
            },
            VillageCode {
                name: "第二居委会",
                code: "002",
            },
            VillageCode {
                name: "第三居委会",
                code: "003",
            },
            VillageCode {
                name: "第四居委会",
                code: "004",
            },
            VillageCode {
                name: "第五居委会",
                code: "005",
            },
            VillageCode {
                name: "第六居委会",
                code: "006",
            },
            VillageCode {
                name: "第七居委会",
                code: "007",
            },
            VillageCode {
                name: "第八居委会",
                code: "008",
            },
            VillageCode {
                name: "第九居委会",
                code: "009",
            },
            VillageCode {
                name: "第十居委会",
                code: "010",
            },
            VillageCode {
                name: "第十一居委会",
                code: "011",
            },
            VillageCode {
                name: "第十二居委会",
                code: "012",
            },
            VillageCode {
                name: "第十三居委会",
                code: "013",
            },
            VillageCode {
                name: "第十四居委会",
                code: "014",
            },
            VillageCode {
                name: "第十五居委会",
                code: "015",
            },
            VillageCode {
                name: "第十六居委会",
                code: "016",
            },
            VillageCode {
                name: "第十七居委会",
                code: "017",
            },
            VillageCode {
                name: "第十八居委会",
                code: "018",
            },
            VillageCode {
                name: "第十九居委会",
                code: "019",
            },
            VillageCode {
                name: "第二十居委会",
                code: "020",
            },
            VillageCode {
                name: "第二十一居委会",
                code: "021",
            },
            VillageCode {
                name: "第二十二居委会",
                code: "022",
            },
            VillageCode {
                name: "第二十三居委会",
                code: "023",
            },
            VillageCode {
                name: "第二十四居委会",
                code: "024",
            },
            VillageCode {
                name: "红旗林场生活区",
                code: "025",
            },
            VillageCode {
                name: "三道林场生活区",
                code: "026",
            },
            VillageCode {
                name: "奋斗林场生活区",
                code: "027",
            },
            VillageCode {
                name: "桦树经营所生活区",
                code: "028",
            },
            VillageCode {
                name: "新城经营所生活区",
                code: "029",
            },
            VillageCode {
                name: "团山子经营所生活区",
                code: "030",
            },
            VillageCode {
                name: "英山经营所生活区",
                code: "031",
            },
            VillageCode {
                name: "尔一林场生活区",
                code: "032",
            },
            VillageCode {
                name: "尔二林场生活区",
                code: "033",
            },
            VillageCode {
                name: "尔三林场生活区",
                code: "034",
            },
            VillageCode {
                name: "小北沟林场生活区",
                code: "035",
            },
            VillageCode {
                name: "苇芦河林场生活区",
                code: "036",
            },
            VillageCode {
                name: "鹿苑岛林场生活区",
                code: "037",
            },
            VillageCode {
                name: "尔站经营所生活区",
                code: "038",
            },
            VillageCode {
                name: "梨树沟经营所生活区",
                code: "039",
            },
            VillageCode {
                name: "抚育站经营所生活区",
                code: "040",
            },
            VillageCode {
                name: "斗沟子林场生活区",
                code: "041",
            },
            VillageCode {
                name: "榆树川林场生活区",
                code: "042",
            },
            VillageCode {
                name: "湾沟林场生活区",
                code: "043",
            },
            VillageCode {
                name: "湖南林场生活区",
                code: "044",
            },
            VillageCode {
                name: "东方红林场生活区",
                code: "045",
            },
            VillageCode {
                name: "南湖头经营所生活区",
                code: "046",
            },
            VillageCode {
                name: "英格岭经营所生活区",
                code: "047",
            },
            VillageCode {
                name: "苇子沟经营所生活区",
                code: "048",
            },
            VillageCode {
                name: "湖北经营所生活区",
                code: "049",
            },
            VillageCode {
                name: "湖西经营所生活区",
                code: "050",
            },
            VillageCode {
                name: "鹿道经营所生活区",
                code: "051",
            },
        ],
    },
    TownCode {
        name: "宁安农场",
        code: "015",
        villages: &[
            VillageCode {
                name: "宁安场直社区",
                code: "001",
            },
            VillageCode {
                name: "宁安农场农业管理区",
                code: "002",
            },
        ],
    },
];

static TOWNS_HJ_040: [TownCode; 10] = [
    TownCode {
        name: "八面通镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "民主社区居委会",
                code: "001",
            },
            VillageCode {
                name: "富家社区居委会",
                code: "002",
            },
            VillageCode {
                name: "头雁社区居委会",
                code: "003",
            },
            VillageCode {
                name: "曙光社区居委会",
                code: "004",
            },
            VillageCode {
                name: "沿河社区居委会",
                code: "005",
            },
            VillageCode {
                name: "和平社区居委会",
                code: "006",
            },
            VillageCode {
                name: "红旗社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新城村委会",
                code: "008",
            },
            VillageCode {
                name: "太和村委会",
                code: "009",
            },
            VillageCode {
                name: "清和村委会",
                code: "010",
            },
            VillageCode {
                name: "中山村委会",
                code: "011",
            },
            VillageCode {
                name: "四平村委会",
                code: "012",
            },
            VillageCode {
                name: "和平村委会",
                code: "013",
            },
            VillageCode {
                name: "农拥村委会",
                code: "014",
            },
            VillageCode {
                name: "民主村委会",
                code: "015",
            },
            VillageCode {
                name: "富家村委会",
                code: "016",
            },
            VillageCode {
                name: "靠河村委会",
                code: "017",
            },
            VillageCode {
                name: "四合村委会",
                code: "018",
            },
            VillageCode {
                name: "莲河村委会",
                code: "019",
            },
            VillageCode {
                name: "秀池村委会",
                code: "020",
            },
            VillageCode {
                name: "民政村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "穆棱镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "光荣居委会",
                code: "001",
            },
            VillageCode {
                name: "中福居委会",
                code: "002",
            },
            VillageCode {
                name: "团结居委会",
                code: "003",
            },
            VillageCode {
                name: "西岗居委会",
                code: "004",
            },
            VillageCode {
                name: "黎明居委会",
                code: "005",
            },
            VillageCode {
                name: "建设居委会",
                code: "006",
            },
            VillageCode {
                name: "先锋居委会",
                code: "007",
            },
            VillageCode {
                name: "道西居委会",
                code: "008",
            },
            VillageCode {
                name: "迎春居委会",
                code: "009",
            },
            VillageCode {
                name: "河南居委会",
                code: "010",
            },
            VillageCode {
                name: "德胜居委会",
                code: "011",
            },
            VillageCode {
                name: "春利居委会",
                code: "012",
            },
            VillageCode {
                name: "中心居委会",
                code: "013",
            },
            VillageCode {
                name: "前进居委会",
                code: "014",
            },
            VillageCode {
                name: "向阳居委会",
                code: "015",
            },
            VillageCode {
                name: "三岔居委会",
                code: "016",
            },
            VillageCode {
                name: "向南居委会",
                code: "017",
            },
            VillageCode {
                name: "新星居委会",
                code: "018",
            },
            VillageCode {
                name: "兴盛村委会",
                code: "019",
            },
            VillageCode {
                name: "石河村委会",
                code: "020",
            },
            VillageCode {
                name: "泉眼河村委会",
                code: "021",
            },
            VillageCode {
                name: "大屯村委会",
                code: "022",
            },
            VillageCode {
                name: "团结村委会",
                code: "023",
            },
            VillageCode {
                name: "大桥村委会",
                code: "024",
            },
            VillageCode {
                name: "红旗村委会",
                code: "025",
            },
            VillageCode {
                name: "西岗村委会",
                code: "026",
            },
            VillageCode {
                name: "河南村委会",
                code: "027",
            },
            VillageCode {
                name: "黎明村委会",
                code: "028",
            },
            VillageCode {
                name: "三岔村委会",
                code: "029",
            },
            VillageCode {
                name: "泉兴村委会",
                code: "030",
            },
            VillageCode {
                name: "振兴村委会",
                code: "031",
            },
            VillageCode {
                name: "北林村委会",
                code: "032",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "033",
            },
            VillageCode {
                name: "河北村委会",
                code: "034",
            },
            VillageCode {
                name: "腰岭村委会",
                code: "035",
            },
            VillageCode {
                name: "岭前村委会",
                code: "036",
            },
            VillageCode {
                name: "明新村委会",
                code: "037",
            },
            VillageCode {
                name: "柳毛村委会",
                code: "038",
            },
        ],
    },
    TownCode {
        name: "下城子镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "义和居委会",
                code: "001",
            },
            VillageCode {
                name: "铁路居委会",
                code: "002",
            },
            VillageCode {
                name: "仁义居委会",
                code: "003",
            },
            VillageCode {
                name: "北安居委会",
                code: "004",
            },
            VillageCode {
                name: "义和村委会",
                code: "005",
            },
            VillageCode {
                name: "南站村委会",
                code: "006",
            },
            VillageCode {
                name: "红桥村委会",
                code: "007",
            },
            VillageCode {
                name: "新利村委会",
                code: "008",
            },
            VillageCode {
                name: "枯榆树村委会",
                code: "009",
            },
            VillageCode {
                name: "北安村委会",
                code: "010",
            },
            VillageCode {
                name: "中新村委会",
                code: "011",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "012",
            },
            VillageCode {
                name: "新建村委会",
                code: "013",
            },
            VillageCode {
                name: "梨树村委会",
                code: "014",
            },
            VillageCode {
                name: "三道河村委会",
                code: "015",
            },
            VillageCode {
                name: "仁里东村委会",
                code: "016",
            },
            VillageCode {
                name: "仁里西村委会",
                code: "017",
            },
            VillageCode {
                name: "新民村委会",
                code: "018",
            },
            VillageCode {
                name: "悬羊村委会",
                code: "019",
            },
            VillageCode {
                name: "保安村委会",
                code: "020",
            },
            VillageCode {
                name: "仁义村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "马桥河镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "西河居委会",
                code: "001",
            },
            VillageCode {
                name: "进步居委会",
                code: "002",
            },
            VillageCode {
                name: "新站居委会",
                code: "003",
            },
            VillageCode {
                name: "新华居委会",
                code: "004",
            },
            VillageCode {
                name: "跃进居委会",
                code: "005",
            },
            VillageCode {
                name: "跃进村委会",
                code: "006",
            },
            VillageCode {
                name: "新华村委会",
                code: "007",
            },
            VillageCode {
                name: "新站村委会",
                code: "008",
            },
            VillageCode {
                name: "西河村委会",
                code: "009",
            },
            VillageCode {
                name: "进步村委会",
                code: "010",
            },
            VillageCode {
                name: "南沟村委会",
                code: "011",
            },
            VillageCode {
                name: "战胜村委会",
                code: "012",
            },
            VillageCode {
                name: "北盛村委会",
                code: "013",
            },
            VillageCode {
                name: "幸福村委会",
                code: "014",
            },
            VillageCode {
                name: "北兴村委会",
                code: "015",
            },
            VillageCode {
                name: "永安村委会",
                code: "016",
            },
            VillageCode {
                name: "杨木村委会",
                code: "017",
            },
            VillageCode {
                name: "石门子村委会",
                code: "018",
            },
            VillageCode {
                name: "太安村委会",
                code: "019",
            },
            VillageCode {
                name: "东风村委会",
                code: "020",
            },
            VillageCode {
                name: "山东村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "兴源镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "镇北居委会",
                code: "001",
            },
            VillageCode {
                name: "镇南居委会",
                code: "002",
            },
            VillageCode {
                name: "车站居委会",
                code: "003",
            },
            VillageCode {
                name: "东村村委会",
                code: "004",
            },
            VillageCode {
                name: "西村村委会",
                code: "005",
            },
            VillageCode {
                name: "北村村委会",
                code: "006",
            },
            VillageCode {
                name: "南村村委会",
                code: "007",
            },
            VillageCode {
                name: "兴源村委会",
                code: "008",
            },
            VillageCode {
                name: "车站村委会",
                code: "009",
            },
            VillageCode {
                name: "兴鲜村委会",
                code: "010",
            },
            VillageCode {
                name: "康吉村委会",
                code: "011",
            },
            VillageCode {
                name: "新丰村委会",
                code: "012",
            },
            VillageCode {
                name: "东兴村委会",
                code: "013",
            },
            VillageCode {
                name: "东胜村委会",
                code: "014",
            },
            VillageCode {
                name: "红岩村委会",
                code: "015",
            },
            VillageCode {
                name: "红盛村委会",
                code: "016",
            },
            VillageCode {
                name: "西崴子村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "河西镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "雷锋村委会",
                code: "001",
            },
            VillageCode {
                name: "普兴村委会",
                code: "002",
            },
            VillageCode {
                name: "双兴村委会",
                code: "003",
            },
            VillageCode {
                name: "红兴村委会",
                code: "004",
            },
            VillageCode {
                name: "光义村委会",
                code: "005",
            },
            VillageCode {
                name: "福兴村委会",
                code: "006",
            },
            VillageCode {
                name: "奇景村委会",
                code: "007",
            },
            VillageCode {
                name: "向阳村委会",
                code: "008",
            },
            VillageCode {
                name: "新兴村委会",
                code: "009",
            },
            VillageCode {
                name: "常兴村委会",
                code: "010",
            },
            VillageCode {
                name: "更新村委会",
                code: "011",
            },
            VillageCode {
                name: "金光村委会",
                code: "012",
            },
            VillageCode {
                name: "朝兴村委会",
                code: "013",
            },
            VillageCode {
                name: "三兴村委会",
                code: "014",
            },
            VillageCode {
                name: "自兴村委会",
                code: "015",
            },
            VillageCode {
                name: "建兴村委会",
                code: "016",
            },
            VillageCode {
                name: "福来村委会",
                code: "017",
            },
            VillageCode {
                name: "龙眼村委会",
                code: "018",
            },
            VillageCode {
                name: "五兴村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "福录乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "福录村委会",
                code: "001",
            },
            VillageCode {
                name: "光明村委会",
                code: "002",
            },
            VillageCode {
                name: "五林村委会",
                code: "003",
            },
            VillageCode {
                name: "高峰村委会",
                code: "004",
            },
            VillageCode {
                name: "四方村委会",
                code: "005",
            },
            VillageCode {
                name: "国光村委会",
                code: "006",
            },
            VillageCode {
                name: "广太村委会",
                code: "007",
            },
            VillageCode {
                name: "康乐村委会",
                code: "008",
            },
            VillageCode {
                name: "自平村委会",
                code: "009",
            },
            VillageCode {
                name: "东新村委会",
                code: "010",
            },
            VillageCode {
                name: "成德村委会",
                code: "011",
            },
            VillageCode {
                name: "巨丰村委会",
                code: "012",
            },
            VillageCode {
                name: "纯盛村委会",
                code: "013",
            },
            VillageCode {
                name: "福生村委会",
                code: "014",
            },
            VillageCode {
                name: "平盛村委会",
                code: "015",
            },
            VillageCode {
                name: "新明村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "共和乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "太平村委会",
                code: "001",
            },
            VillageCode {
                name: "立新村委会",
                code: "002",
            },
            VillageCode {
                name: "北金场村委会",
                code: "003",
            },
            VillageCode {
                name: "牛心村委会",
                code: "004",
            },
            VillageCode {
                name: "靠山村委会",
                code: "005",
            },
            VillageCode {
                name: "金峪村委会",
                code: "006",
            },
            VillageCode {
                name: "六峰村委会",
                code: "007",
            },
            VillageCode {
                name: "共和村委会",
                code: "008",
            },
            VillageCode {
                name: "碱场村委会",
                code: "009",
            },
            VillageCode {
                name: "东升村委会",
                code: "010",
            },
            VillageCode {
                name: "东光村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "八面通林业局",
        code: "009",
        villages: &[
            VillageCode {
                name: "第一居委会",
                code: "001",
            },
            VillageCode {
                name: "第二居委会",
                code: "002",
            },
            VillageCode {
                name: "第三居委会",
                code: "003",
            },
            VillageCode {
                name: "第四居委会",
                code: "004",
            },
            VillageCode {
                name: "第五居委会",
                code: "005",
            },
            VillageCode {
                name: "第六居委会",
                code: "006",
            },
            VillageCode {
                name: "风月桥经营所社区",
                code: "007",
            },
            VillageCode {
                name: "护林经营所社区",
                code: "008",
            },
            VillageCode {
                name: "纯盛经营所社区",
                code: "009",
            },
            VillageCode {
                name: "青沟岭经营所社区",
                code: "010",
            },
            VillageCode {
                name: "自兴经营所社区",
                code: "011",
            },
            VillageCode {
                name: "三兴经营所社区",
                code: "012",
            },
            VillageCode {
                name: "新兴经营所社区",
                code: "013",
            },
            VillageCode {
                name: "红星经营所社区",
                code: "014",
            },
            VillageCode {
                name: "光义经营所社区",
                code: "015",
            },
            VillageCode {
                name: "老黑山经营所社区",
                code: "016",
            },
            VillageCode {
                name: "马桥河经营所社区",
                code: "017",
            },
            VillageCode {
                name: "幸福经营所社区",
                code: "018",
            },
            VillageCode {
                name: "红房子经营所社区",
                code: "019",
            },
            VillageCode {
                name: "柳毛河经营所社区",
                code: "020",
            },
            VillageCode {
                name: "枯河沟经营所社区",
                code: "021",
            },
            VillageCode {
                name: "悬羊经营所社区",
                code: "022",
            },
            VillageCode {
                name: "砍椽沟经营所社区",
                code: "023",
            },
            VillageCode {
                name: "七家子经营所社区",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "穆棱林业局",
        code: "010",
        villages: &[
            VillageCode {
                name: "第一居委会",
                code: "001",
            },
            VillageCode {
                name: "第二居委会",
                code: "002",
            },
            VillageCode {
                name: "第三居委会",
                code: "003",
            },
            VillageCode {
                name: "第四居委会",
                code: "004",
            },
            VillageCode {
                name: "第五居委会",
                code: "005",
            },
            VillageCode {
                name: "第六居委会",
                code: "006",
            },
            VillageCode {
                name: "第七居委会",
                code: "007",
            },
            VillageCode {
                name: "第八居委会",
                code: "008",
            },
            VillageCode {
                name: "第九居委会",
                code: "009",
            },
            VillageCode {
                name: "第十居委会",
                code: "010",
            },
            VillageCode {
                name: "第十一居委会",
                code: "011",
            },
            VillageCode {
                name: "第十二居委会",
                code: "012",
            },
            VillageCode {
                name: "第十三居委会",
                code: "013",
            },
            VillageCode {
                name: "第十四居委会",
                code: "014",
            },
            VillageCode {
                name: "第十五居委会",
                code: "015",
            },
            VillageCode {
                name: "第十六居委会",
                code: "016",
            },
            VillageCode {
                name: "第十七居委会",
                code: "017",
            },
            VillageCode {
                name: "第十八居委会",
                code: "018",
            },
            VillageCode {
                name: "第十九居委会",
                code: "019",
            },
            VillageCode {
                name: "第二十居委会",
                code: "020",
            },
            VillageCode {
                name: "莲河经营所社区",
                code: "021",
            },
            VillageCode {
                name: "桦树河经营所社区",
                code: "022",
            },
            VillageCode {
                name: "杨木桥经营所社区",
                code: "023",
            },
            VillageCode {
                name: "双宁经营所社区",
                code: "024",
            },
            VillageCode {
                name: "和平经营所社区",
                code: "025",
            },
            VillageCode {
                name: "泉眼河经营所社区",
                code: "026",
            },
            VillageCode {
                name: "狮子桥经营所社区",
                code: "027",
            },
            VillageCode {
                name: "红岩经营所社区",
                code: "028",
            },
            VillageCode {
                name: "老道沟经营所社区",
                code: "029",
            },
            VillageCode {
                name: "代马沟经营所社区",
                code: "030",
            },
            VillageCode {
                name: "磨刀石经营所社区",
                code: "031",
            },
            VillageCode {
                name: "西北岔经营所社区",
                code: "032",
            },
            VillageCode {
                name: "共和经营所社区",
                code: "033",
            },
            VillageCode {
                name: "东兴经营所社区",
                code: "034",
            },
            VillageCode {
                name: "牛心山经营所社区",
                code: "035",
            },
            VillageCode {
                name: "龙爪沟经营所社区",
                code: "036",
            },
            VillageCode {
                name: "三新山经营所社区",
                code: "037",
            },
        ],
    },
];

static TOWNS_HJ_041: [TownCode; 7] = [
    TownCode {
        name: "东宁镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "率宾社区居委会",
                code: "001",
            },
            VillageCode {
                name: "光明社区居委会",
                code: "002",
            },
            VillageCode {
                name: "中心社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东兴社区居委会",
                code: "004",
            },
            VillageCode {
                name: "宏源社区居委会",
                code: "005",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "006",
            },
            VillageCode {
                name: "南山社区居委会",
                code: "007",
            },
            VillageCode {
                name: "繁荣社区居委会",
                code: "008",
            },
            VillageCode {
                name: "建国社区居委会",
                code: "009",
            },
            VillageCode {
                name: "西山社区居委会",
                code: "010",
            },
            VillageCode {
                name: "一街村委会",
                code: "011",
            },
            VillageCode {
                name: "二街村委会",
                code: "012",
            },
            VillageCode {
                name: "菜一村委会",
                code: "013",
            },
            VillageCode {
                name: "菜二村委会",
                code: "014",
            },
            VillageCode {
                name: "大城子村委会",
                code: "015",
            },
            VillageCode {
                name: "夹信子村委会",
                code: "016",
            },
            VillageCode {
                name: "北河沿村委会",
                code: "017",
            },
            VillageCode {
                name: "民主村委会",
                code: "018",
            },
            VillageCode {
                name: "南沟村委会",
                code: "019",
            },
            VillageCode {
                name: "东绥村委会",
                code: "020",
            },
            VillageCode {
                name: "万鹿沟村委会",
                code: "021",
            },
            VillageCode {
                name: "转角楼村委会",
                code: "022",
            },
            VillageCode {
                name: "暖一村委会",
                code: "023",
            },
            VillageCode {
                name: "暖二村委会",
                code: "024",
            },
            VillageCode {
                name: "葫罗卜葳村委会",
                code: "025",
            },
            VillageCode {
                name: "太平沟村委会",
                code: "026",
            },
            VillageCode {
                name: "新屯村委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "三岔口镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "东大川村委会",
                code: "001",
            },
            VillageCode {
                name: "新立村委会",
                code: "002",
            },
            VillageCode {
                name: "幸福村委会",
                code: "003",
            },
            VillageCode {
                name: "永和村委会",
                code: "004",
            },
            VillageCode {
                name: "泡子沿村委会",
                code: "005",
            },
            VillageCode {
                name: "三岔口村委会",
                code: "006",
            },
            VillageCode {
                name: "五大队村委会",
                code: "007",
            },
            VillageCode {
                name: "高安村委会",
                code: "008",
            },
            VillageCode {
                name: "南山村委会",
                code: "009",
            },
            VillageCode {
                name: "矿山村委会",
                code: "010",
            },
            VillageCode {
                name: "东星村委会",
                code: "011",
            },
            VillageCode {
                name: "光星二村委会",
                code: "012",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "大肚川镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "大肚川村委会",
                code: "001",
            },
            VillageCode {
                name: "团结村委会",
                code: "002",
            },
            VillageCode {
                name: "浪东沟村委会",
                code: "003",
            },
            VillageCode {
                name: "李家趟子村委会",
                code: "004",
            },
            VillageCode {
                name: "胜利村委会",
                code: "005",
            },
            VillageCode {
                name: "太阳升村委会",
                code: "006",
            },
            VillageCode {
                name: "煤矿村委会",
                code: "007",
            },
            VillageCode {
                name: "新城沟村委会",
                code: "008",
            },
            VillageCode {
                name: "老城沟村委会",
                code: "009",
            },
            VillageCode {
                name: "闹枝沟村委会",
                code: "010",
            },
            VillageCode {
                name: "太平川村委会",
                code: "011",
            },
            VillageCode {
                name: "西沟村委会",
                code: "012",
            },
            VillageCode {
                name: "石门子村委会",
                code: "013",
            },
            VillageCode {
                name: "马营村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "老黑山镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "信号村委会",
                code: "001",
            },
            VillageCode {
                name: "黑瞎沟村委会",
                code: "002",
            },
            VillageCode {
                name: "阳明村委会",
                code: "003",
            },
            VillageCode {
                name: "上碱村委会",
                code: "004",
            },
            VillageCode {
                name: "下碱村委会",
                code: "005",
            },
            VillageCode {
                name: "太平沟村委会",
                code: "006",
            },
            VillageCode {
                name: "和光村委会",
                code: "007",
            },
            VillageCode {
                name: "二道沟村委会",
                code: "008",
            },
            VillageCode {
                name: "奔楼头村委会",
                code: "009",
            },
            VillageCode {
                name: "罗家店村委会",
                code: "010",
            },
            VillageCode {
                name: "永红村委会",
                code: "011",
            },
            VillageCode {
                name: "老黑山村委会",
                code: "012",
            },
            VillageCode {
                name: "万宝湾村委会",
                code: "013",
            },
            VillageCode {
                name: "西葳子村委会",
                code: "014",
            },
            VillageCode {
                name: "南村村委会",
                code: "015",
            },
            VillageCode {
                name: "黄泥河村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "道河镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "道河村委会",
                code: "001",
            },
            VillageCode {
                name: "和平村委会",
                code: "002",
            },
            VillageCode {
                name: "小地营村委会",
                code: "003",
            },
            VillageCode {
                name: "通沟村委会",
                code: "004",
            },
            VillageCode {
                name: "岭后村委会",
                code: "005",
            },
            VillageCode {
                name: "砬子沟村委会",
                code: "006",
            },
            VillageCode {
                name: "洞庭村委会",
                code: "007",
            },
            VillageCode {
                name: "岭西村委会",
                code: "008",
            },
            VillageCode {
                name: "跃进村委会",
                code: "009",
            },
            VillageCode {
                name: "西河村委会",
                code: "010",
            },
            VillageCode {
                name: "东村村委会",
                code: "011",
            },
            VillageCode {
                name: "西村村委会",
                code: "012",
            },
            VillageCode {
                name: "土城子村委会",
                code: "013",
            },
            VillageCode {
                name: "八里坪村委会",
                code: "014",
            },
            VillageCode {
                name: "沙河子村委会",
                code: "015",
            },
            VillageCode {
                name: "奋斗村委会",
                code: "016",
            },
            VillageCode {
                name: "前进村委会",
                code: "017",
            },
            VillageCode {
                name: "兴东村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "绥阳镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "第一居委会",
                code: "001",
            },
            VillageCode {
                name: "第二居委会",
                code: "002",
            },
            VillageCode {
                name: "第三居委会",
                code: "003",
            },
            VillageCode {
                name: "第四居委会",
                code: "004",
            },
            VillageCode {
                name: "第五居委会",
                code: "005",
            },
            VillageCode {
                name: "第六居委会",
                code: "006",
            },
            VillageCode {
                name: "第七居委会",
                code: "007",
            },
            VillageCode {
                name: "第八居委会",
                code: "008",
            },
            VillageCode {
                name: "第九居委会",
                code: "009",
            },
            VillageCode {
                name: "第十居委会",
                code: "010",
            },
            VillageCode {
                name: "先锋村委会",
                code: "011",
            },
            VillageCode {
                name: "爱国村委会",
                code: "012",
            },
            VillageCode {
                name: "红旗村委会",
                code: "013",
            },
            VillageCode {
                name: "柞木村委会",
                code: "014",
            },
            VillageCode {
                name: "柳毛河村委会",
                code: "015",
            },
            VillageCode {
                name: "联兴村委会",
                code: "016",
            },
            VillageCode {
                name: "曙村村委会",
                code: "017",
            },
            VillageCode {
                name: "河南村委会",
                code: "018",
            },
            VillageCode {
                name: "三道村委会",
                code: "019",
            },
            VillageCode {
                name: "太平村委会",
                code: "020",
            },
            VillageCode {
                name: "二道村委会",
                code: "021",
            },
            VillageCode {
                name: "新民村委会",
                code: "022",
            },
            VillageCode {
                name: "北沟村委会",
                code: "023",
            },
            VillageCode {
                name: "菜营村委会",
                code: "024",
            },
            VillageCode {
                name: "蔬菜村委会",
                code: "025",
            },
            VillageCode {
                name: "绥西村委会",
                code: "026",
            },
            VillageCode {
                name: "三道河子村委会",
                code: "027",
            },
            VillageCode {
                name: "太岭村委会",
                code: "028",
            },
            VillageCode {
                name: "九里地村委会",
                code: "029",
            },
            VillageCode {
                name: "细鳞河村委会",
                code: "030",
            },
            VillageCode {
                name: "鸡冠村委会",
                code: "031",
            },
            VillageCode {
                name: "双丰村委会",
                code: "032",
            },
            VillageCode {
                name: "细岭村委会",
                code: "033",
            },
            VillageCode {
                name: "河西村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "绥阳林业局",
        code: "007",
        villages: &[
            VillageCode {
                name: "绥阳林业居委会",
                code: "001",
            },
            VillageCode {
                name: "细鳞河林场生活区",
                code: "002",
            },
            VillageCode {
                name: "双桥子林场生活区",
                code: "003",
            },
            VillageCode {
                name: "万宝湾林场生活区",
                code: "004",
            },
            VillageCode {
                name: "太平川林场生活区",
                code: "005",
            },
            VillageCode {
                name: "三节砬子林场生活区",
                code: "006",
            },
            VillageCode {
                name: "暖泉河林场生活区",
                code: "007",
            },
            VillageCode {
                name: "中股流林场生活区",
                code: "008",
            },
            VillageCode {
                name: "三岔河林场生活区",
                code: "009",
            },
            VillageCode {
                name: "元山林场生活区",
                code: "010",
            },
            VillageCode {
                name: "会川林场生活区",
                code: "011",
            },
            VillageCode {
                name: "道河林场生活区",
                code: "012",
            },
            VillageCode {
                name: "沙洞林场生活区",
                code: "013",
            },
            VillageCode {
                name: "河湾林场生活区",
                code: "014",
            },
            VillageCode {
                name: "黄松林场生活区",
                code: "015",
            },
            VillageCode {
                name: "双丫子林场生活区",
                code: "016",
            },
            VillageCode {
                name: "寒葱河林场生活区",
                code: "017",
            },
            VillageCode {
                name: "青山林场生活区",
                code: "018",
            },
            VillageCode {
                name: "二道岗子林场生活区",
                code: "019",
            },
            VillageCode {
                name: "八里坪林场生活区",
                code: "020",
            },
            VillageCode {
                name: "柳桥沟林场生活区",
                code: "021",
            },
            VillageCode {
                name: "向岭林场生活区",
                code: "022",
            },
            VillageCode {
                name: "新青林场生活区",
                code: "023",
            },
        ],
    },
];

pub const CITIES_HJ: [CityCode; 42] = [
    CityCode {
        name: "省辖市",
        code: "000",
        towns: &[],
    },
    CityCode {
        name: "密山市",
        code: "001",
        towns: &TOWNS_HJ_001,
    },
    CityCode {
        name: "鸡冠市",
        code: "002",
        towns: &TOWNS_HJ_002,
    },
    CityCode {
        name: "恒山市",
        code: "003",
        towns: &TOWNS_HJ_003,
    },
    CityCode {
        name: "滴道市",
        code: "004",
        towns: &TOWNS_HJ_004,
    },
    CityCode {
        name: "梨树市",
        code: "005",
        towns: &TOWNS_HJ_005,
    },
    CityCode {
        name: "城子河市",
        code: "006",
        towns: &TOWNS_HJ_006,
    },
    CityCode {
        name: "麻山市",
        code: "007",
        towns: &TOWNS_HJ_007,
    },
    CityCode {
        name: "鸡东市",
        code: "008",
        towns: &TOWNS_HJ_008,
    },
    CityCode {
        name: "虎林市",
        code: "009",
        towns: &TOWNS_HJ_009,
    },
    CityCode {
        name: "尖山市",
        code: "010",
        towns: &TOWNS_HJ_010,
    },
    CityCode {
        name: "岭东市",
        code: "011",
        towns: &TOWNS_HJ_011,
    },
    CityCode {
        name: "四方台市",
        code: "012",
        towns: &TOWNS_HJ_012,
    },
    CityCode {
        name: "宝山市",
        code: "013",
        towns: &TOWNS_HJ_013,
    },
    CityCode {
        name: "集贤市",
        code: "014",
        towns: &TOWNS_HJ_014,
    },
    CityCode {
        name: "友谊市",
        code: "015",
        towns: &TOWNS_HJ_015,
    },
    CityCode {
        name: "宝清市",
        code: "016",
        towns: &TOWNS_HJ_016,
    },
    CityCode {
        name: "饶河市",
        code: "017",
        towns: &TOWNS_HJ_017,
    },
    CityCode {
        name: "向阳市",
        code: "018",
        towns: &TOWNS_HJ_018,
    },
    CityCode {
        name: "前进市",
        code: "019",
        towns: &TOWNS_HJ_019,
    },
    CityCode {
        name: "东风市",
        code: "020",
        towns: &TOWNS_HJ_020,
    },
    CityCode {
        name: "郊市",
        code: "021",
        towns: &TOWNS_HJ_021,
    },
    CityCode {
        name: "桦南市",
        code: "022",
        towns: &TOWNS_HJ_022,
    },
    CityCode {
        name: "桦川市",
        code: "023",
        towns: &TOWNS_HJ_023,
    },
    CityCode {
        name: "汤原市",
        code: "024",
        towns: &TOWNS_HJ_024,
    },
    CityCode {
        name: "同江市",
        code: "025",
        towns: &TOWNS_HJ_025,
    },
    CityCode {
        name: "富锦市",
        code: "026",
        towns: &TOWNS_HJ_026,
    },
    CityCode {
        name: "抚远市",
        code: "027",
        towns: &TOWNS_HJ_027,
    },
    CityCode {
        name: "新兴市",
        code: "028",
        towns: &TOWNS_HJ_028,
    },
    CityCode {
        name: "桃山市",
        code: "029",
        towns: &TOWNS_HJ_029,
    },
    CityCode {
        name: "茄子河市",
        code: "030",
        towns: &TOWNS_HJ_030,
    },
    CityCode {
        name: "勃利市",
        code: "031",
        towns: &TOWNS_HJ_031,
    },
    CityCode {
        name: "东安市",
        code: "032",
        towns: &TOWNS_HJ_032,
    },
    CityCode {
        name: "阳明市",
        code: "033",
        towns: &TOWNS_HJ_033,
    },
    CityCode {
        name: "爱民市",
        code: "034",
        towns: &TOWNS_HJ_034,
    },
    CityCode {
        name: "西安市",
        code: "035",
        towns: &TOWNS_HJ_035,
    },
    CityCode {
        name: "林口市",
        code: "036",
        towns: &TOWNS_HJ_036,
    },
    CityCode {
        name: "绥芬河市",
        code: "037",
        towns: &TOWNS_HJ_037,
    },
    CityCode {
        name: "海林市",
        code: "038",
        towns: &TOWNS_HJ_038,
    },
    CityCode {
        name: "宁安市",
        code: "039",
        towns: &TOWNS_HJ_039,
    },
    CityCode {
        name: "穆棱市",
        code: "040",
        towns: &TOWNS_HJ_040,
    },
    CityCode {
        name: "东宁市",
        code: "041",
        towns: &TOWNS_HJ_041,
    },
];
