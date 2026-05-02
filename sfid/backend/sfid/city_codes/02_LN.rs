use super::{CityCode, TownCode, VillageCode};

static TOWNS_LN_001: [TownCode; 1] = [TownCode {
    name: "香港岛",
    code: "001",
    villages: &[
        VillageCode {
            name: "中西区",
            code: "001",
        },
        VillageCode {
            name: "湾仔区",
            code: "002",
        },
        VillageCode {
            name: "东区",
            code: "003",
        },
        VillageCode {
            name: "南区",
            code: "004",
        },
    ],
}];

static TOWNS_LN_002: [TownCode; 1] = [TownCode {
    name: "新界",
    code: "001",
    villages: &[
        VillageCode {
            name: "葵青区",
            code: "001",
        },
        VillageCode {
            name: "荃湾区",
            code: "002",
        },
        VillageCode {
            name: "屯门区",
            code: "003",
        },
        VillageCode {
            name: "元朗区",
            code: "004",
        },
        VillageCode {
            name: "北区",
            code: "005",
        },
        VillageCode {
            name: "大埔区",
            code: "006",
        },
        VillageCode {
            name: "沙田区",
            code: "007",
        },
        VillageCode {
            name: "西贡区",
            code: "008",
        },
        VillageCode {
            name: "离岛区",
            code: "009",
        },
    ],
}];

static TOWNS_LN_003: [TownCode; 1] = [TownCode {
    name: "澳门半岛",
    code: "001",
    villages: &[
        VillageCode {
            name: "大堂区",
            code: "001",
        },
        VillageCode {
            name: "望德堂区",
            code: "002",
        },
        VillageCode {
            name: "风顺堂区",
            code: "003",
        },
        VillageCode {
            name: "花地玛堂区",
            code: "004",
        },
        VillageCode {
            name: "圣安多尼堂区",
            code: "005",
        },
    ],
}];

static TOWNS_LN_004: [TownCode; 31] = [
    TownCode {
        name: "翠香街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "紫荆社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "翠香社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "为农社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "沿河社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "银桦社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "兴业社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "北园社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "新竹社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "福宁社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "青竹社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "康宁社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "山场社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "新村社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "柠溪社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "安宁社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "钰海社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "香山社区居民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "梅华街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "南虹社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "翠东社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "红山社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "环山社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "鸿运社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "翠前社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "新香社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "富华社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "兴发社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "南村社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "上冲社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "仁恒社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "创业社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "敬业社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "悦城社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "鸿业社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "翠福社区居民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "前山街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "中山亭社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "金钟社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "圆明社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "和晟社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "岱山社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "莲塘社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "翠景社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "前山社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "白石社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "兰埔社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "夏村社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "翠微社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "造贝社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "翠珠社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "逸仙社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "福石社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "南沙湾社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "凤祥社区居民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "吉大街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "白莲社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "吉莲社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "莲花社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "海大社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "九洲社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "竹苑社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "南山社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "景山社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "怡景社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "海湾社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "水湾社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "官村社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "园林社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "海天社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "石花社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "景莲社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "洲仔社区居民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "拱北街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "将军山社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "关闸社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "昌盛社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "侨光社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "迎宾社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "联安社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "北岭社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "岭南社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "婆石社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "华平社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "茂盛社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "夏湾社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "港昌社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "粤华社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "春泽社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "桂花社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "昌平社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "前河社区居民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "香湾街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "朝阳社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "海虹社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "海前社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "北堤社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "碧涛社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "香凤社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "海霞社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "神前社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "水拥社区居民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "狮山街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "胡湾社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "红旗社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "东风社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "桃园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "教育社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "青春社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "幸福社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "南香社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "南坑社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "南美社区居民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "湾仔街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "湾仔社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "桂园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "富兴社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "连屏社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "银坑社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "作物社区居民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "凤山街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "沥溪社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "福溪社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "红荔社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "春晖社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "长沙社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "梅溪社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "东坑社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "南溪社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "界涌社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "新溪社区居民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "唐家湾镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "下栅社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "金峰社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "官塘社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "东岸社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "北沙社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "永丰社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "那洲社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "会同社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "宁堂社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "上栅社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "唐家社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "银星社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "唐乐社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "鸡山社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "后环社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "淇澳社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "前环社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "星湾社区居民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "南屏镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "濂泉社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "南屏社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北山社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "十二村社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "广生社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "广昌社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "洪湾社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "红东社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "东桥社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "华发社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "茂丰社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "永济社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "新城社区居民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "横琴镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "新家园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "荷塘社区居委会",
                code: "002",
            },
            VillageCode {
                name: "小横琴社区居委会",
                code: "003",
            },
            VillageCode {
                name: "莲花社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "桂山镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "桂海村委会",
                code: "001",
            },
            VillageCode {
                name: "桂山村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "万山镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "万山村委会",
                code: "001",
            },
            VillageCode {
                name: "东澳村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "担杆镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "伶仃村委会",
                code: "001",
            },
            VillageCode {
                name: "新村村委会",
                code: "002",
            },
            VillageCode {
                name: "担杆村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "南屏科技园",
        code: "016",
        villages: &[VillageCode {
            name: "南屏科技园虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "保税区",
        code: "017",
        villages: &[VillageCode {
            name: "保税区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "三溪科创小镇发展中心",
        code: "018",
        villages: &[VillageCode {
            name: "三溪科创小镇发展中心虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "洪湾商贸物流中心",
        code: "019",
        villages: &[VillageCode {
            name: "洪湾商贸物流中心虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "白藤街道",
        code: "020",
        villages: &[
            VillageCode {
                name: "好景社区居委会",
                code: "001",
            },
            VillageCode {
                name: "群兴社区居委会",
                code: "002",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "白藤社区居委会",
                code: "005",
            },
            VillageCode {
                name: "鹤洲社区居委会",
                code: "006",
            },
            VillageCode {
                name: "湖滨社区居委会",
                code: "007",
            },
            VillageCode {
                name: "家和社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "莲洲镇",
        code: "021",
        villages: &[
            VillageCode {
                name: "横山社区居委会",
                code: "001",
            },
            VillageCode {
                name: "莲溪社区居委会",
                code: "002",
            },
            VillageCode {
                name: "大沙社区居委会",
                code: "003",
            },
            VillageCode {
                name: "耕管村委会",
                code: "004",
            },
            VillageCode {
                name: "广丰村委会",
                code: "005",
            },
            VillageCode {
                name: "福安村委会",
                code: "006",
            },
            VillageCode {
                name: "三角村委会",
                code: "007",
            },
            VillageCode {
                name: "三龙村委会",
                code: "008",
            },
            VillageCode {
                name: "二龙村委会",
                code: "009",
            },
            VillageCode {
                name: "獭山村委会",
                code: "010",
            },
            VillageCode {
                name: "三冲村委会",
                code: "011",
            },
            VillageCode {
                name: "大胜村委会",
                code: "012",
            },
            VillageCode {
                name: "三家村委会",
                code: "013",
            },
            VillageCode {
                name: "横山村委会",
                code: "014",
            },
            VillageCode {
                name: "新益村委会",
                code: "015",
            },
            VillageCode {
                name: "粉洲村委会",
                code: "016",
            },
            VillageCode {
                name: "南青村委会",
                code: "017",
            },
            VillageCode {
                name: "新洲村委会",
                code: "018",
            },
            VillageCode {
                name: "西滘村委会",
                code: "019",
            },
            VillageCode {
                name: "东滘村委会",
                code: "020",
            },
            VillageCode {
                name: "红星村委会",
                code: "021",
            },
            VillageCode {
                name: "文锋村委会",
                code: "022",
            },
            VillageCode {
                name: "新丰村委会",
                code: "023",
            },
            VillageCode {
                name: "东安村委会",
                code: "024",
            },
            VillageCode {
                name: "上栏村委会",
                code: "025",
            },
            VillageCode {
                name: "下栏村委会",
                code: "026",
            },
            VillageCode {
                name: "石龙村委会",
                code: "027",
            },
            VillageCode {
                name: "莲江村委会",
                code: "028",
            },
            VillageCode {
                name: "光明村委会",
                code: "029",
            },
            VillageCode {
                name: "东湾村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "斗门镇",
        code: "022",
        villages: &[
            VillageCode {
                name: "斗门社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大赤坎村委会",
                code: "002",
            },
            VillageCode {
                name: "小赤坎村委会",
                code: "003",
            },
            VillageCode {
                name: "上洲村委会",
                code: "004",
            },
            VillageCode {
                name: "下洲村委会",
                code: "005",
            },
            VillageCode {
                name: "新乡村委会",
                code: "006",
            },
            VillageCode {
                name: "斗门村委会",
                code: "007",
            },
            VillageCode {
                name: "南门村委会",
                code: "008",
            },
            VillageCode {
                name: "八甲村委会",
                code: "009",
            },
            VillageCode {
                name: "小濠冲村委会",
                code: "010",
            },
            VillageCode {
                name: "大濠冲村委会",
                code: "011",
            },
            VillageCode {
                name: "龙山工业管理区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "乾务镇",
        code: "023",
        villages: &[
            VillageCode {
                name: "乾南社区居委会",
                code: "001",
            },
            VillageCode {
                name: "沙龙社区居委会",
                code: "002",
            },
            VillageCode {
                name: "乾东村委会",
                code: "003",
            },
            VillageCode {
                name: "乾西村委会",
                code: "004",
            },
            VillageCode {
                name: "乾北村委会",
                code: "005",
            },
            VillageCode {
                name: "东澳村委会",
                code: "006",
            },
            VillageCode {
                name: "狮群村委会",
                code: "007",
            },
            VillageCode {
                name: "湾口村委会",
                code: "008",
            },
            VillageCode {
                name: "石狗村委会",
                code: "009",
            },
            VillageCode {
                name: "大海环村委会",
                code: "010",
            },
            VillageCode {
                name: "虎山村委会",
                code: "011",
            },
            VillageCode {
                name: "荔山村委会",
                code: "012",
            },
            VillageCode {
                name: "马山村委会",
                code: "013",
            },
            VillageCode {
                name: "网山村委会",
                code: "014",
            },
            VillageCode {
                name: "夏村村委会",
                code: "015",
            },
            VillageCode {
                name: "南山村委会",
                code: "016",
            },
            VillageCode {
                name: "新村村委会",
                code: "017",
            },
            VillageCode {
                name: "三里村委会",
                code: "018",
            },
            VillageCode {
                name: "富山工业园管理区",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "白蕉镇",
        code: "024",
        villages: &[
            VillageCode {
                name: "白蕉社区居委会",
                code: "001",
            },
            VillageCode {
                name: "六乡社区居委会",
                code: "002",
            },
            VillageCode {
                name: "城东社区居委会",
                code: "003",
            },
            VillageCode {
                name: "虹桥社区居委会",
                code: "004",
            },
            VillageCode {
                name: "榕益村委会",
                code: "005",
            },
            VillageCode {
                name: "黄家村委会",
                code: "006",
            },
            VillageCode {
                name: "新沙村委会",
                code: "007",
            },
            VillageCode {
                name: "新二村委会",
                code: "008",
            },
            VillageCode {
                name: "新环村委会",
                code: "009",
            },
            VillageCode {
                name: "南环村委会",
                code: "010",
            },
            VillageCode {
                name: "泗喜村委会",
                code: "011",
            },
            VillageCode {
                name: "东围村委会",
                code: "012",
            },
            VillageCode {
                name: "白石村委会",
                code: "013",
            },
            VillageCode {
                name: "大托村委会",
                code: "014",
            },
            VillageCode {
                name: "灯一村委会",
                code: "015",
            },
            VillageCode {
                name: "灯笼村委会",
                code: "016",
            },
            VillageCode {
                name: "灯三村委会",
                code: "017",
            },
            VillageCode {
                name: "桅夹村委会",
                code: "018",
            },
            VillageCode {
                name: "昭信村委会",
                code: "019",
            },
            VillageCode {
                name: "东湖村委会",
                code: "020",
            },
            VillageCode {
                name: "成裕村委会",
                code: "021",
            },
            VillageCode {
                name: "赖家村委会",
                code: "022",
            },
            VillageCode {
                name: "白蕉村委会",
                code: "023",
            },
            VillageCode {
                name: "东岸村委会",
                code: "024",
            },
            VillageCode {
                name: "沙石村委会",
                code: "025",
            },
            VillageCode {
                name: "小托村委会",
                code: "026",
            },
            VillageCode {
                name: "冲口村委会",
                code: "027",
            },
            VillageCode {
                name: "八顷村委会",
                code: "028",
            },
            VillageCode {
                name: "办冲村委会",
                code: "029",
            },
            VillageCode {
                name: "月坑村委会",
                code: "030",
            },
            VillageCode {
                name: "盖山村委会",
                code: "031",
            },
            VillageCode {
                name: "鳘鱼沙村委会",
                code: "032",
            },
            VillageCode {
                name: "虾山村委会",
                code: "033",
            },
            VillageCode {
                name: "南澳村委会",
                code: "034",
            },
            VillageCode {
                name: "孖湾村委会",
                code: "035",
            },
            VillageCode {
                name: "丰洲村委会",
                code: "036",
            },
            VillageCode {
                name: "新马墩村委会",
                code: "037",
            },
            VillageCode {
                name: "白蕉工业开发区居委会",
                code: "038",
            },
        ],
    },
    TownCode {
        name: "井岸镇",
        code: "025",
        villages: &[
            VillageCode {
                name: "红旗社区居委会",
                code: "001",
            },
            VillageCode {
                name: "朝阳社区居委会",
                code: "002",
            },
            VillageCode {
                name: "红卫社区居委会",
                code: "003",
            },
            VillageCode {
                name: "统建社区居委会",
                code: "004",
            },
            VillageCode {
                name: "长亨社区居委会",
                code: "005",
            },
            VillageCode {
                name: "飞龙社区居委会",
                code: "006",
            },
            VillageCode {
                name: "美湾社区居委会",
                code: "007",
            },
            VillageCode {
                name: "南湾社区居委会",
                code: "008",
            },
            VillageCode {
                name: "南峰社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新伟社区居委会",
                code: "010",
            },
            VillageCode {
                name: "南潮村委会",
                code: "011",
            },
            VillageCode {
                name: "坭湾村委会",
                code: "012",
            },
            VillageCode {
                name: "尖峰村委会",
                code: "013",
            },
            VillageCode {
                name: "东风村委会",
                code: "014",
            },
            VillageCode {
                name: "五福村委会",
                code: "015",
            },
            VillageCode {
                name: "新堂村委会",
                code: "016",
            },
            VillageCode {
                name: "新青村委会",
                code: "017",
            },
            VillageCode {
                name: "西埔村委会",
                code: "018",
            },
            VillageCode {
                name: "草朗村委会",
                code: "019",
            },
            VillageCode {
                name: "鸡咀村委会",
                code: "020",
            },
            VillageCode {
                name: "黄金村委会",
                code: "021",
            },
            VillageCode {
                name: "北澳村委会",
                code: "022",
            },
            VillageCode {
                name: "龙西村委会",
                code: "023",
            },
            VillageCode {
                name: "西湾村委会",
                code: "024",
            },
            VillageCode {
                name: "黄杨村委会",
                code: "025",
            },
            VillageCode {
                name: "新青科技工业园管理区",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "三灶镇",
        code: "026",
        villages: &[
            VillageCode {
                name: "三灶社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "金海岸社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "草堂湾社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "西城社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "滨海社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "鱼月村民委员会",
                code: "006",
            },
            VillageCode {
                name: "鱼林村民委员会",
                code: "007",
            },
            VillageCode {
                name: "中心村民委员会",
                code: "008",
            },
            VillageCode {
                name: "海澄村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "南水镇",
        code: "027",
        villages: &[
            VillageCode {
                name: "南水社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金洲社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金龙社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南场村委会",
                code: "004",
            },
            VillageCode {
                name: "荷包村委会",
                code: "005",
            },
            VillageCode {
                name: "高栏村委会",
                code: "006",
            },
            VillageCode {
                name: "沙白石村委会",
                code: "007",
            },
            VillageCode {
                name: "飞沙村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "红旗镇",
        code: "028",
        villages: &[
            VillageCode {
                name: "藤山社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "广安社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "大林社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "湖东社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "矿山社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "三板社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "八一社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "小林社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "双湖社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "小林村民委员会",
                code: "010",
            },
            VillageCode {
                name: "广益村民委员会",
                code: "011",
            },
            VillageCode {
                name: "广发村民委员会",
                code: "012",
            },
            VillageCode {
                name: "沙脊村民委员会",
                code: "013",
            },
            VillageCode {
                name: "三板村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "平沙镇",
        code: "029",
        villages: &[
            VillageCode {
                name: "立新社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "美平社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "南新社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "沙美社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "平塘社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "大虎社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "前进社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "前锋社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "前西社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "大海环社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "连湾社区居民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "联港工业区",
        code: "030",
        villages: &[VillageCode {
            name: "联港工业区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "航空产业园",
        code: "031",
        villages: &[VillageCode {
            name: "航空产业园虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_LN_005: [TownCode; 6] = [
    TownCode {
        name: "三灶镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "三灶社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "金海岸社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "草堂湾社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "西城社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "滨海社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "鱼月村民委员会",
                code: "006",
            },
            VillageCode {
                name: "鱼林村民委员会",
                code: "007",
            },
            VillageCode {
                name: "中心村民委员会",
                code: "008",
            },
            VillageCode {
                name: "海澄村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "南水镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "南水社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金洲社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金龙社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南场村委会",
                code: "004",
            },
            VillageCode {
                name: "荷包村委会",
                code: "005",
            },
            VillageCode {
                name: "高栏村委会",
                code: "006",
            },
            VillageCode {
                name: "沙白石村委会",
                code: "007",
            },
            VillageCode {
                name: "飞沙村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "红旗镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "藤山社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "广安社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "大林社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "湖东社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "矿山社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "三板社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "八一社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "小林社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "双湖社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "小林村民委员会",
                code: "010",
            },
            VillageCode {
                name: "广益村民委员会",
                code: "011",
            },
            VillageCode {
                name: "广发村民委员会",
                code: "012",
            },
            VillageCode {
                name: "沙脊村民委员会",
                code: "013",
            },
            VillageCode {
                name: "三板村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "平沙镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "立新社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "美平社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "南新社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "沙美社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "平塘社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "大虎社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "前进社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "前锋社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "前西社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "大海环社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "连湾社区居民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "联港工业区",
        code: "005",
        villages: &[VillageCode {
            name: "联港工业区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "航空产业园",
        code: "006",
        villages: &[VillageCode {
            name: "航空产业园虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_LN_006: [TownCode; 6] = [
    TownCode {
        name: "白藤街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "好景社区居委会",
                code: "001",
            },
            VillageCode {
                name: "群兴社区居委会",
                code: "002",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "白藤社区居委会",
                code: "005",
            },
            VillageCode {
                name: "鹤洲社区居委会",
                code: "006",
            },
            VillageCode {
                name: "湖滨社区居委会",
                code: "007",
            },
            VillageCode {
                name: "家和社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "莲洲镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "横山社区居委会",
                code: "001",
            },
            VillageCode {
                name: "莲溪社区居委会",
                code: "002",
            },
            VillageCode {
                name: "大沙社区居委会",
                code: "003",
            },
            VillageCode {
                name: "耕管村委会",
                code: "004",
            },
            VillageCode {
                name: "广丰村委会",
                code: "005",
            },
            VillageCode {
                name: "福安村委会",
                code: "006",
            },
            VillageCode {
                name: "三角村委会",
                code: "007",
            },
            VillageCode {
                name: "三龙村委会",
                code: "008",
            },
            VillageCode {
                name: "二龙村委会",
                code: "009",
            },
            VillageCode {
                name: "獭山村委会",
                code: "010",
            },
            VillageCode {
                name: "三冲村委会",
                code: "011",
            },
            VillageCode {
                name: "大胜村委会",
                code: "012",
            },
            VillageCode {
                name: "三家村委会",
                code: "013",
            },
            VillageCode {
                name: "横山村委会",
                code: "014",
            },
            VillageCode {
                name: "新益村委会",
                code: "015",
            },
            VillageCode {
                name: "粉洲村委会",
                code: "016",
            },
            VillageCode {
                name: "南青村委会",
                code: "017",
            },
            VillageCode {
                name: "新洲村委会",
                code: "018",
            },
            VillageCode {
                name: "西滘村委会",
                code: "019",
            },
            VillageCode {
                name: "东滘村委会",
                code: "020",
            },
            VillageCode {
                name: "红星村委会",
                code: "021",
            },
            VillageCode {
                name: "文锋村委会",
                code: "022",
            },
            VillageCode {
                name: "新丰村委会",
                code: "023",
            },
            VillageCode {
                name: "东安村委会",
                code: "024",
            },
            VillageCode {
                name: "上栏村委会",
                code: "025",
            },
            VillageCode {
                name: "下栏村委会",
                code: "026",
            },
            VillageCode {
                name: "石龙村委会",
                code: "027",
            },
            VillageCode {
                name: "莲江村委会",
                code: "028",
            },
            VillageCode {
                name: "光明村委会",
                code: "029",
            },
            VillageCode {
                name: "东湾村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "斗门镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "斗门社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大赤坎村委会",
                code: "002",
            },
            VillageCode {
                name: "小赤坎村委会",
                code: "003",
            },
            VillageCode {
                name: "上洲村委会",
                code: "004",
            },
            VillageCode {
                name: "下洲村委会",
                code: "005",
            },
            VillageCode {
                name: "新乡村委会",
                code: "006",
            },
            VillageCode {
                name: "斗门村委会",
                code: "007",
            },
            VillageCode {
                name: "南门村委会",
                code: "008",
            },
            VillageCode {
                name: "八甲村委会",
                code: "009",
            },
            VillageCode {
                name: "小濠冲村委会",
                code: "010",
            },
            VillageCode {
                name: "大濠冲村委会",
                code: "011",
            },
            VillageCode {
                name: "龙山工业管理区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "乾务镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "乾南社区居委会",
                code: "001",
            },
            VillageCode {
                name: "沙龙社区居委会",
                code: "002",
            },
            VillageCode {
                name: "乾东村委会",
                code: "003",
            },
            VillageCode {
                name: "乾西村委会",
                code: "004",
            },
            VillageCode {
                name: "乾北村委会",
                code: "005",
            },
            VillageCode {
                name: "东澳村委会",
                code: "006",
            },
            VillageCode {
                name: "狮群村委会",
                code: "007",
            },
            VillageCode {
                name: "湾口村委会",
                code: "008",
            },
            VillageCode {
                name: "石狗村委会",
                code: "009",
            },
            VillageCode {
                name: "大海环村委会",
                code: "010",
            },
            VillageCode {
                name: "虎山村委会",
                code: "011",
            },
            VillageCode {
                name: "荔山村委会",
                code: "012",
            },
            VillageCode {
                name: "马山村委会",
                code: "013",
            },
            VillageCode {
                name: "网山村委会",
                code: "014",
            },
            VillageCode {
                name: "夏村村委会",
                code: "015",
            },
            VillageCode {
                name: "南山村委会",
                code: "016",
            },
            VillageCode {
                name: "新村村委会",
                code: "017",
            },
            VillageCode {
                name: "三里村委会",
                code: "018",
            },
            VillageCode {
                name: "富山工业园管理区",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "白蕉镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "白蕉社区居委会",
                code: "001",
            },
            VillageCode {
                name: "六乡社区居委会",
                code: "002",
            },
            VillageCode {
                name: "城东社区居委会",
                code: "003",
            },
            VillageCode {
                name: "虹桥社区居委会",
                code: "004",
            },
            VillageCode {
                name: "榕益村委会",
                code: "005",
            },
            VillageCode {
                name: "黄家村委会",
                code: "006",
            },
            VillageCode {
                name: "新沙村委会",
                code: "007",
            },
            VillageCode {
                name: "新二村委会",
                code: "008",
            },
            VillageCode {
                name: "新环村委会",
                code: "009",
            },
            VillageCode {
                name: "南环村委会",
                code: "010",
            },
            VillageCode {
                name: "泗喜村委会",
                code: "011",
            },
            VillageCode {
                name: "东围村委会",
                code: "012",
            },
            VillageCode {
                name: "白石村委会",
                code: "013",
            },
            VillageCode {
                name: "大托村委会",
                code: "014",
            },
            VillageCode {
                name: "灯一村委会",
                code: "015",
            },
            VillageCode {
                name: "灯笼村委会",
                code: "016",
            },
            VillageCode {
                name: "灯三村委会",
                code: "017",
            },
            VillageCode {
                name: "桅夹村委会",
                code: "018",
            },
            VillageCode {
                name: "昭信村委会",
                code: "019",
            },
            VillageCode {
                name: "东湖村委会",
                code: "020",
            },
            VillageCode {
                name: "成裕村委会",
                code: "021",
            },
            VillageCode {
                name: "赖家村委会",
                code: "022",
            },
            VillageCode {
                name: "白蕉村委会",
                code: "023",
            },
            VillageCode {
                name: "东岸村委会",
                code: "024",
            },
            VillageCode {
                name: "沙石村委会",
                code: "025",
            },
            VillageCode {
                name: "小托村委会",
                code: "026",
            },
            VillageCode {
                name: "冲口村委会",
                code: "027",
            },
            VillageCode {
                name: "八顷村委会",
                code: "028",
            },
            VillageCode {
                name: "办冲村委会",
                code: "029",
            },
            VillageCode {
                name: "月坑村委会",
                code: "030",
            },
            VillageCode {
                name: "盖山村委会",
                code: "031",
            },
            VillageCode {
                name: "鳘鱼沙村委会",
                code: "032",
            },
            VillageCode {
                name: "虾山村委会",
                code: "033",
            },
            VillageCode {
                name: "南澳村委会",
                code: "034",
            },
            VillageCode {
                name: "孖湾村委会",
                code: "035",
            },
            VillageCode {
                name: "丰洲村委会",
                code: "036",
            },
            VillageCode {
                name: "新马墩村委会",
                code: "037",
            },
            VillageCode {
                name: "白蕉工业开发区居委会",
                code: "038",
            },
        ],
    },
    TownCode {
        name: "井岸镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "红旗社区居委会",
                code: "001",
            },
            VillageCode {
                name: "朝阳社区居委会",
                code: "002",
            },
            VillageCode {
                name: "红卫社区居委会",
                code: "003",
            },
            VillageCode {
                name: "统建社区居委会",
                code: "004",
            },
            VillageCode {
                name: "长亨社区居委会",
                code: "005",
            },
            VillageCode {
                name: "飞龙社区居委会",
                code: "006",
            },
            VillageCode {
                name: "美湾社区居委会",
                code: "007",
            },
            VillageCode {
                name: "南湾社区居委会",
                code: "008",
            },
            VillageCode {
                name: "南峰社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新伟社区居委会",
                code: "010",
            },
            VillageCode {
                name: "南潮村委会",
                code: "011",
            },
            VillageCode {
                name: "坭湾村委会",
                code: "012",
            },
            VillageCode {
                name: "尖峰村委会",
                code: "013",
            },
            VillageCode {
                name: "东风村委会",
                code: "014",
            },
            VillageCode {
                name: "五福村委会",
                code: "015",
            },
            VillageCode {
                name: "新堂村委会",
                code: "016",
            },
            VillageCode {
                name: "新青村委会",
                code: "017",
            },
            VillageCode {
                name: "西埔村委会",
                code: "018",
            },
            VillageCode {
                name: "草朗村委会",
                code: "019",
            },
            VillageCode {
                name: "鸡咀村委会",
                code: "020",
            },
            VillageCode {
                name: "黄金村委会",
                code: "021",
            },
            VillageCode {
                name: "北澳村委会",
                code: "022",
            },
            VillageCode {
                name: "龙西村委会",
                code: "023",
            },
            VillageCode {
                name: "西湾村委会",
                code: "024",
            },
            VillageCode {
                name: "黄杨村委会",
                code: "025",
            },
            VillageCode {
                name: "新青科技工业园管理区",
                code: "026",
            },
        ],
    },
];

static TOWNS_LN_007: [TownCode; 5] = [
    TownCode {
        name: "梅沙街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "小梅沙居委会",
                code: "001",
            },
            VillageCode {
                name: "滨海居委会",
                code: "002",
            },
            VillageCode {
                name: "大梅沙居委会",
                code: "003",
            },
            VillageCode {
                name: "东海岸居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "盐田街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "永安居委会",
                code: "001",
            },
            VillageCode {
                name: "盐田居委会",
                code: "002",
            },
            VillageCode {
                name: "沿港居委会",
                code: "003",
            },
            VillageCode {
                name: "东海居委会",
                code: "004",
            },
            VillageCode {
                name: "明珠居委会",
                code: "005",
            },
            VillageCode {
                name: "海桐居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "沙头角街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "桥东居委会",
                code: "001",
            },
            VillageCode {
                name: "中英街居委会",
                code: "002",
            },
            VillageCode {
                name: "东和居委会",
                code: "003",
            },
            VillageCode {
                name: "沙头角居委会",
                code: "004",
            },
            VillageCode {
                name: "元墩头居委会",
                code: "005",
            },
            VillageCode {
                name: "凤凰居委会",
                code: "006",
            },
            VillageCode {
                name: "田心居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "海山街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "鹏湾居委会",
                code: "001",
            },
            VillageCode {
                name: "梧桐居委会",
                code: "002",
            },
            VillageCode {
                name: "倚山居委会",
                code: "003",
            },
            VillageCode {
                name: "海月居委会",
                code: "004",
            },
            VillageCode {
                name: "海涛居委会",
                code: "005",
            },
            VillageCode {
                name: "海景居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "深圳盐田综合保税区",
        code: "005",
        villages: &[VillageCode {
            name: "深圳盐田综合保税区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_LN_008: [TownCode; 7] = [
    TownCode {
        name: "坪山街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "六联社区居委会",
                code: "001",
            },
            VillageCode {
                name: "六和社区居委会",
                code: "002",
            },
            VillageCode {
                name: "坪山社区居委会",
                code: "003",
            },
            VillageCode {
                name: "和平社区居委会",
                code: "004",
            },
            VillageCode {
                name: "新和社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "马峦街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "坪环社区居委会",
                code: "001",
            },
            VillageCode {
                name: "江岭社区居委会",
                code: "002",
            },
            VillageCode {
                name: "马峦社区居委会",
                code: "003",
            },
            VillageCode {
                name: "沙坣社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "碧岭街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "碧岭社区居委会",
                code: "001",
            },
            VillageCode {
                name: "汤坑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "沙湖社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "石井街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "金龟社区居委会",
                code: "001",
            },
            VillageCode {
                name: "石井社区居委会",
                code: "002",
            },
            VillageCode {
                name: "田头社区居委会",
                code: "003",
            },
            VillageCode {
                name: "田心社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "坑梓街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "坑梓社区居委会",
                code: "001",
            },
            VillageCode {
                name: "秀新社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金沙社区居委会",
                code: "003",
            },
            VillageCode {
                name: "沙田社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "龙田街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "龙田社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南布社区居委会",
                code: "002",
            },
            VillageCode {
                name: "竹坑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "老坑社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "深圳坪山综合保税区",
        code: "007",
        villages: &[VillageCode {
            name: "深圳坪山综合保税区虚拟社区",
            code: "001",
        }],
    },
];

pub const CITIES_LN: [CityCode; 9] = [
    CityCode {
        name: "省辖市",
        code: "000",
        towns: &[],
    },
    CityCode {
        name: "香港市",
        code: "001",
        towns: &TOWNS_LN_001,
    },
    CityCode {
        name: "新界市",
        code: "002",
        towns: &TOWNS_LN_002,
    },
    CityCode {
        name: "澳门市",
        code: "003",
        towns: &TOWNS_LN_003,
    },
    CityCode {
        name: "珠海市",
        code: "004",
        towns: &TOWNS_LN_004,
    },
    CityCode {
        name: "金湾市",
        code: "005",
        towns: &TOWNS_LN_005,
    },
    CityCode {
        name: "斗门市",
        code: "006",
        towns: &TOWNS_LN_006,
    },
    CityCode {
        name: "盐田市",
        code: "007",
        towns: &TOWNS_LN_007,
    },
    CityCode {
        name: "坪山市",
        code: "008",
        towns: &TOWNS_LN_008,
    },
];
