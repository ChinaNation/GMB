use super::{CityCode, TownCode, VillageCode};

static TOWNS_XJ_001: [TownCode; 8] = [
    TownCode {
        name: "清水泉片区管委会街道",
        code: "001",
        villages: &[VillageCode {
            name: "清水泉社区",
            code: "001",
        }],
    },
    TownCode {
        name: "谢家沟片区管委会街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "谢家沟社区",
                code: "001",
            },
            VillageCode {
                name: "硫磺沟社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "水西沟镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "溪水社区",
                code: "001",
            },
            VillageCode {
                name: "南河路社区",
                code: "002",
            },
            VillageCode {
                name: "南滩路社区",
                code: "003",
            },
            VillageCode {
                name: "平西梁村委会",
                code: "004",
            },
            VillageCode {
                name: "小东沟村委会",
                code: "005",
            },
            VillageCode {
                name: "大庙村委会",
                code: "006",
            },
            VillageCode {
                name: "水西沟村委会",
                code: "007",
            },
            VillageCode {
                name: "方家庄村委会",
                code: "008",
            },
            VillageCode {
                name: "东梁村委会",
                code: "009",
            },
            VillageCode {
                name: "庙尔沟村委会",
                code: "010",
            },
            VillageCode {
                name: "闸滩村委会",
                code: "011",
            },
            VillageCode {
                name: "东湾村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "板房沟镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "琴苑社区",
                code: "001",
            },
            VillageCode {
                name: "天峡社区",
                code: "002",
            },
            VillageCode {
                name: "灯草沟村委会",
                code: "003",
            },
            VillageCode {
                name: "板房沟村委会",
                code: "004",
            },
            VillageCode {
                name: "七工村委会",
                code: "005",
            },
            VillageCode {
                name: "八家户村委会",
                code: "006",
            },
            VillageCode {
                name: "合胜村委会",
                code: "007",
            },
            VillageCode {
                name: "东湾村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "永丰镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "亚新社区",
                code: "001",
            },
            VillageCode {
                name: "永盛村委会",
                code: "002",
            },
            VillageCode {
                name: "公盛村委会",
                code: "003",
            },
            VillageCode {
                name: "上寺村委会",
                code: "004",
            },
            VillageCode {
                name: "下寺村委会",
                code: "005",
            },
            VillageCode {
                name: "永丰村委会",
                code: "006",
            },
            VillageCode {
                name: "永新村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "萨尔达坂乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "雪莲谷社区",
                code: "001",
            },
            VillageCode {
                name: "赵家庄子村委会",
                code: "002",
            },
            VillageCode {
                name: "萨尔乔克村委会",
                code: "003",
            },
            VillageCode {
                name: "中梁村委会",
                code: "004",
            },
            VillageCode {
                name: "白杨沟村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "甘沟乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "菊花台社区",
                code: "001",
            },
            VillageCode {
                name: "白杨沟村委会",
                code: "002",
            },
            VillageCode {
                name: "前进村委会",
                code: "003",
            },
            VillageCode {
                name: "东风村委会",
                code: "004",
            },
            VillageCode {
                name: "团结村委会",
                code: "005",
            },
            VillageCode {
                name: "高潮村委会",
                code: "006",
            },
            VillageCode {
                name: "土圈村委会",
                code: "007",
            },
            VillageCode {
                name: "小渠子村委会",
                code: "008",
            },
            VillageCode {
                name: "天山村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "托里乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "苜蓿台社区",
                code: "001",
            },
            VillageCode {
                name: "白建沟村委会",
                code: "002",
            },
            VillageCode {
                name: "乌什城村委会",
                code: "003",
            },
            VillageCode {
                name: "羊圈沟村委会",
                code: "004",
            },
            VillageCode {
                name: "乌拉泊村委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_XJ_002: [TownCode; 22] = [
    TownCode {
        name: "燕儿窝街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "燕儿窝南社区居委会",
                code: "001",
            },
            VillageCode {
                name: "红雁池社区居委会",
                code: "002",
            },
            VillageCode {
                name: "燕儿窝北路西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "燕儿窝北路东社区居委会",
                code: "004",
            },
            VillageCode {
                name: "青水社区居委会",
                code: "005",
            },
            VillageCode {
                name: "三泰路社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "胜利路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "三甬碑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "胜利路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "南梁社区居委会",
                code: "003",
            },
            VillageCode {
                name: "河坝巷社区居委会",
                code: "004",
            },
            VillageCode {
                name: "湖源巷社区居委会",
                code: "005",
            },
            VillageCode {
                name: "羊毛湖社区居委会",
                code: "006",
            },
            VillageCode {
                name: "多斯鲁克社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新疆大学社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新东街社区居委会",
                code: "009",
            },
            VillageCode {
                name: "边瑞社区居委会",
                code: "010",
            },
            VillageCode {
                name: "湖源巷西社区居委会",
                code: "011",
            },
            VillageCode {
                name: "新华南路东社区居委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "团结路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "领馆巷北社区居委会",
                code: "001",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "002",
            },
            VillageCode {
                name: "八户梁社区居委会",
                code: "003",
            },
            VillageCode {
                name: "皇城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "瓷厂社区居委会",
                code: "005",
            },
            VillageCode {
                name: "后泉路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "领馆巷南社区居委会",
                code: "007",
            },
            VillageCode {
                name: "文新社区居委会",
                code: "008",
            },
            VillageCode {
                name: "中泉街南社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "解放南路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "建中路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "永和巷社区居委会",
                code: "002",
            },
            VillageCode {
                name: "山西巷社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新市路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "育才巷社区居委会",
                code: "005",
            },
            VillageCode {
                name: "天池路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "龙泉社区居委会",
                code: "007",
            },
            VillageCode {
                name: "马市小区社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "新华南路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "南公园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北国春城社区居委会",
                code: "002",
            },
            VillageCode {
                name: "五桥社区居委会",
                code: "003",
            },
            VillageCode {
                name: "团结西路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "四桥社区居委会",
                code: "005",
            },
            VillageCode {
                name: "三桥社区居委会",
                code: "006",
            },
            VillageCode {
                name: "西河坝前街社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西河坝后街社区居委会",
                code: "008",
            },
            VillageCode {
                name: "河滩南路东社区居委会",
                code: "009",
            },
            VillageCode {
                name: "南郊客运站社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "和平路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "二道湾社区居委会",
                code: "001",
            },
            VillageCode {
                name: "康泰社区居委会",
                code: "002",
            },
            VillageCode {
                name: "三山社区居委会",
                code: "003",
            },
            VillageCode {
                name: "体育馆路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东菜园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "国际城社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新湾社区居委会",
                code: "007",
            },
            VillageCode {
                name: "人民路南社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "解放北路街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "南大街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "和平北路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "天山路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "青年路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "广场社区居委会",
                code: "005",
            },
            VillageCode {
                name: "健康路社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "幸福路街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "幸福路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "幸福集社区居委会",
                code: "002",
            },
            VillageCode {
                name: "天福花园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "四道巷社区居委会",
                code: "004",
            },
            VillageCode {
                name: "北三巷社区居委会",
                code: "005",
            },
            VillageCode {
                name: "百信花园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "职大社区居委会",
                code: "007",
            },
            VillageCode {
                name: "建国南路南社区居委会",
                code: "008",
            },
            VillageCode {
                name: "中环路北社区居委会",
                code: "009",
            },
            VillageCode {
                name: "宏大社区居委会",
                code: "010",
            },
            VillageCode {
                name: "湖东社区居委会",
                code: "011",
            },
            VillageCode {
                name: "幸福城市花园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "幸福园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "幸福路南社区居委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "东门街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "建国南路北社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东后南街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "东风路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东后北街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "五星南路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "西后街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "前进东路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "前进西路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "建国北路社区居委会",
                code: "009",
            },
            VillageCode {
                name: "光华路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "东环路西社区居委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "新华北路街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "人民路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西河坝社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西河街南社区居委会",
                code: "003",
            },
            VillageCode {
                name: "红旗路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西河街北社区居委会",
                code: "005",
            },
            VillageCode {
                name: "民主西路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "建设西路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "建设路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "小西门社区居委会",
                code: "009",
            },
            VillageCode {
                name: "光明路南社区居委会",
                code: "010",
            },
            VillageCode {
                name: "文化路社区居委会",
                code: "011",
            },
            VillageCode {
                name: "春风巷社区居委会",
                code: "012",
            },
            VillageCode {
                name: "中山路社区居委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "青年路街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "光明路北社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新民路西社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新民路东社区居委会",
                code: "003",
            },
            VillageCode {
                name: "建工大院社区居委会",
                code: "004",
            },
            VillageCode {
                name: "五星路西社区居委会",
                code: "005",
            },
            VillageCode {
                name: "五星路东社区居委会",
                code: "006",
            },
            VillageCode {
                name: "红山路南社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "碱泉街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "向阳社区居委会",
                code: "001",
            },
            VillageCode {
                name: "碱泉东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "碱泉中社区居委会",
                code: "003",
            },
            VillageCode {
                name: "碱泉西社区居委会",
                code: "004",
            },
            VillageCode {
                name: "月光小区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "方圆社区居委会",
                code: "006",
            },
            VillageCode {
                name: "红桥社区居委会",
                code: "007",
            },
            VillageCode {
                name: "日光小区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "青年路东社区居委会",
                code: "009",
            },
            VillageCode {
                name: "翠泉路东社区居委会",
                code: "010",
            },
            VillageCode {
                name: "青年路南社区居委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "延安路街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "团结东路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "晨光社区居委会",
                code: "002",
            },
            VillageCode {
                name: "东湾社区居委会",
                code: "003",
            },
            VillageCode {
                name: "富泉街北社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西域轻工社区居委会",
                code: "005",
            },
            VillageCode {
                name: "吉顺路东社区居委会",
                code: "006",
            },
            VillageCode {
                name: "富泉街社区居委会",
                code: "007",
            },
            VillageCode {
                name: "中湾街南社区居委会",
                code: "008",
            },
            VillageCode {
                name: "富康街北社区居委会",
                code: "009",
            },
            VillageCode {
                name: "吉顺路北社区居委会",
                code: "010",
            },
            VillageCode {
                name: "中湾街北社区居委会",
                code: "011",
            },
            VillageCode {
                name: "虹湾巷社区居委会",
                code: "012",
            },
            VillageCode {
                name: "富北社区居委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "红雁街道",
        code: "014",
        villages: &[
            VillageCode {
                name: "东大梁社区居委会",
                code: "001",
            },
            VillageCode {
                name: "乌拉泊社区居委会",
                code: "002",
            },
            VillageCode {
                name: "红雁池东社区居委会",
                code: "003",
            },
            VillageCode {
                name: "红雁池北社区居委会",
                code: "004",
            },
            VillageCode {
                name: "乌拉泊村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "南草滩街道",
        code: "015",
        villages: &[
            VillageCode {
                name: "翠青社区居委会",
                code: "001",
            },
            VillageCode {
                name: "翠林社区居委会",
                code: "002",
            },
            VillageCode {
                name: "翠园社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "东泉路街道",
        code: "016",
        villages: &[
            VillageCode {
                name: "翠泉南路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "翠泉北路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "东泉路中社区居委会",
                code: "003",
            },
            VillageCode {
                name: "天府社区居委会",
                code: "004",
            },
            VillageCode {
                name: "翠泉中社区居委会",
                code: "005",
            },
            VillageCode {
                name: "碱泉一街南社区居委会",
                code: "006",
            },
            VillageCode {
                name: "翠泉路西社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "二道桥片区",
        code: "017",
        villages: &[
            VillageCode {
                name: "二道桥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "宽北巷社区居委会",
                code: "002",
            },
            VillageCode {
                name: "福寿巷社区居委会",
                code: "003",
            },
            VillageCode {
                name: "跃进街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "药王庙社区居委会",
                code: "005",
            },
            VillageCode {
                name: "固原巷社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "黑甲山片区",
        code: "018",
        villages: &[
            VillageCode {
                name: "黑甲山前街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "黑甲山后街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "二道湾东社区居委会",
                code: "003",
            },
            VillageCode {
                name: "大湾北路西社区居委会",
                code: "004",
            },
            VillageCode {
                name: "富康街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "金银路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "二道湾北社区居委会",
                code: "007",
            },
            VillageCode {
                name: "北湾街社区居委会",
                code: "008",
            },
            VillageCode {
                name: "富康街南社区居委会",
                code: "009",
            },
            VillageCode {
                name: "大湾北路东社区居委会",
                code: "010",
            },
            VillageCode {
                name: "后泉路北社区居委会",
                code: "011",
            },
            VillageCode {
                name: "跃进街南社区居委会",
                code: "012",
            },
            VillageCode {
                name: "北湾街北社区居委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "大湾片区",
        code: "019",
        villages: &[
            VillageCode {
                name: "延安大道社区居委会",
                code: "001",
            },
            VillageCode {
                name: "盐化社区居委会",
                code: "002",
            },
            VillageCode {
                name: "延安新村社区居委会",
                code: "003",
            },
            VillageCode {
                name: "明华街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "延安东路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "新翠社区居委会",
                code: "006",
            },
            VillageCode {
                name: "希望街北社区居委会",
                code: "007",
            },
            VillageCode {
                name: "金悦巷社区居委会",
                code: "008",
            },
            VillageCode {
                name: "花儿沟社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "赛马场片区",
        code: "020",
        villages: &[
            VillageCode {
                name: "大湾南社区居委会",
                code: "001",
            },
            VillageCode {
                name: "红旗社区居委会",
                code: "002",
            },
            VillageCode {
                name: "赛马场西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "十七户社区居委会",
                code: "004",
            },
            VillageCode {
                name: "赛马场东社区居委会",
                code: "005",
            },
            VillageCode {
                name: "水上乐园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "榆园路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "十七户路东社区居委会",
                code: "008",
            },
            VillageCode {
                name: "巴哈尔路南社区居委会",
                code: "009",
            },
            VillageCode {
                name: "榆园路北社区居委会",
                code: "010",
            },
            VillageCode {
                name: "水上乐园南社区居委会",
                code: "011",
            },
            VillageCode {
                name: "赛马场北社区居委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "南湾街片区",
        code: "021",
        villages: &[
            VillageCode {
                name: "延安路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "夏玛勒巴克巷社区居委会",
                code: "002",
            },
            VillageCode {
                name: "波斯坦巷社区居委会",
                code: "003",
            },
            VillageCode {
                name: "广电社区居委会",
                code: "004",
            },
            VillageCode {
                name: "昌乐园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "中环路南社区居委会",
                code: "006",
            },
            VillageCode {
                name: "大湾北社区居委会",
                code: "007",
            },
            VillageCode {
                name: "南湾街南社区居委会",
                code: "008",
            },
            VillageCode {
                name: "南湾街北社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "大巴扎片区",
        code: "022",
        villages: &[
            VillageCode {
                name: "大巴扎社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新市路南社区居委会",
                code: "002",
            },
            VillageCode {
                name: "双庆巷社区居委会",
                code: "003",
            },
        ],
    },
];

static TOWNS_XJ_003: [TownCode; 20] = [
    TownCode {
        name: "长江路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "经一路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "伊宁路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "经二路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "长江南路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "长江北路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "碾子沟社区居委会",
                code: "006",
            },
            VillageCode {
                name: "冷库社区居委会",
                code: "007",
            },
            VillageCode {
                name: "牛奶巷社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新兴巷社区居委会",
                code: "009",
            },
            VillageCode {
                name: "奇台北路社区",
                code: "010",
            },
            VillageCode {
                name: "棉花北街社区居委会",
                code: "011",
            },
            VillageCode {
                name: "棉花南街社区居委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "和田街街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "建机社区居委会",
                code: "001",
            },
            VillageCode {
                name: "和田街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "交通社区居委会",
                code: "003",
            },
            VillageCode {
                name: "公安社区居委会",
                code: "004",
            },
            VillageCode {
                name: "水利社区居委会",
                code: "005",
            },
            VillageCode {
                name: "东一街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "东二街社区居委会",
                code: "007",
            },
            VillageCode {
                name: "黄河社区居委会",
                code: "008",
            },
            VillageCode {
                name: "珠江路北社区",
                code: "009",
            },
            VillageCode {
                name: "河滩南路西社区",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "扬子江路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "人民公园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "青松苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "扬子江社区居委会",
                code: "003",
            },
            VillageCode {
                name: "邮政社区居委会",
                code: "004",
            },
            VillageCode {
                name: "新华社区居委会",
                code: "005",
            },
            VillageCode {
                name: "揽秀园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "孔雀社区居委会",
                code: "007",
            },
            VillageCode {
                name: "腾威社区居委会",
                code: "008",
            },
            VillageCode {
                name: "十月社区居委会",
                code: "009",
            },
            VillageCode {
                name: "黑龙江路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "汇月社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "宝山东社区",
                code: "012",
            },
            VillageCode {
                name: "扬子江路东社区",
                code: "013",
            },
            VillageCode {
                name: "虹桥南社区居委会",
                code: "014",
            },
            VillageCode {
                name: "西北路南社区居委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "友好南路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "友好社区居委会",
                code: "001",
            },
            VillageCode {
                name: "明园有色社区居委会",
                code: "002",
            },
            VillageCode {
                name: "明园石油社区居委会",
                code: "003",
            },
            VillageCode {
                name: "石油学院社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西北路北社区居委会",
                code: "005",
            },
            VillageCode {
                name: "虹桥北社区居委会",
                code: "006",
            },
            VillageCode {
                name: "金色花苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "鸿雁社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "友好北路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "友好新村东社区居委会",
                code: "001",
            },
            VillageCode {
                name: "友好新村西社区居委会",
                code: "002",
            },
            VillageCode {
                name: "师大社区居委会",
                code: "003",
            },
            VillageCode {
                name: "八楼社区居委会",
                code: "004",
            },
            VillageCode {
                name: "军区医院社区居委会",
                code: "005",
            },
            VillageCode {
                name: "友谊社区居委会",
                code: "006",
            },
            VillageCode {
                name: "宝地社区居委会",
                code: "007",
            },
            VillageCode {
                name: "石家园子社区居委会",
                code: "008",
            },
            VillageCode {
                name: "利民社区居委会",
                code: "009",
            },
            VillageCode {
                name: "泰华社区居委会",
                code: "010",
            },
            VillageCode {
                name: "巴州北路社区居委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "八一街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "国道社区居委会",
                code: "001",
            },
            VillageCode {
                name: "瑞安社区居委会",
                code: "002",
            },
            VillageCode {
                name: "农大社区居委会",
                code: "003",
            },
            VillageCode {
                name: "电信社区居委会",
                code: "004",
            },
            VillageCode {
                name: "农科院社区居委会",
                code: "005",
            },
            VillageCode {
                name: "金阳社区居委会",
                code: "006",
            },
            VillageCode {
                name: "锦华苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新北社区居委会",
                code: "008",
            },
            VillageCode {
                name: "哈密西路社区居委会",
                code: "009",
            },
            VillageCode {
                name: "老满城社区居委会",
                code: "010",
            },
            VillageCode {
                name: "博物馆社区居委会",
                code: "011",
            },
            VillageCode {
                name: "南昌路社区居委会",
                code: "012",
            },
            VillageCode {
                name: "南昌北路社区居委会",
                code: "013",
            },
            VillageCode {
                name: "古城南社区",
                code: "014",
            },
            VillageCode {
                name: "古城北社区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "炉院街街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "炉院街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "和平桥社区居委会",
                code: "002",
            },
            VillageCode {
                name: "炉院北街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "炉院东街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "长征新村社区居委会",
                code: "005",
            },
            VillageCode {
                name: "变电站社区居委会",
                code: "006",
            },
            VillageCode {
                name: "仓房沟南路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "仓房沟北路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "天山建材社区居委会",
                code: "009",
            },
            VillageCode {
                name: "乌拉泊社区居委会",
                code: "010",
            },
            VillageCode {
                name: "后峡社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "双和社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "仓房沟中路西社区",
                code: "013",
            },
            VillageCode {
                name: "珠江路南社区",
                code: "014",
            },
            VillageCode {
                name: "双龙社区",
                code: "015",
            },
            VillageCode {
                name: "茶街社区",
                code: "016",
            },
            VillageCode {
                name: "和悦社区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "西山街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "新标社区居委会",
                code: "001",
            },
            VillageCode {
                name: "骑马山社区居委会",
                code: "002",
            },
            VillageCode {
                name: "马料地街南社区居委会",
                code: "003",
            },
            VillageCode {
                name: "永康一巷东社区居委会",
                code: "004",
            },
            VillageCode {
                name: "永乐一巷社区居委会",
                code: "005",
            },
            VillageCode {
                name: "永平巷社区居委会",
                code: "006",
            },
            VillageCode {
                name: "马料地街西社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西山东街北社区居委会",
                code: "008",
            },
            VillageCode {
                name: "平川路社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "雅玛里克山街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "青年社区居委会",
                code: "001",
            },
            VillageCode {
                name: "校园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "铁东社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南站社区居委会",
                code: "004",
            },
            VillageCode {
                name: "泰裕社区居委会",
                code: "005",
            },
            VillageCode {
                name: "青峰社区居委会",
                code: "006",
            },
            VillageCode {
                name: "铁西社区居委会",
                code: "007",
            },
            VillageCode {
                name: "秀园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "古丽斯坦社区居委会",
                code: "009",
            },
            VillageCode {
                name: "光明社区居委会",
                code: "010",
            },
            VillageCode {
                name: "冷库山社区居委会",
                code: "011",
            },
            VillageCode {
                name: "雪莲社区居委会",
                code: "012",
            },
            VillageCode {
                name: "宝山社区居委会",
                code: "013",
            },
            VillageCode {
                name: "西虹社区居委会",
                code: "014",
            },
            VillageCode {
                name: "南梁坡社区居委会",
                code: "015",
            },
            VillageCode {
                name: "宝山路西社区",
                code: "016",
            },
            VillageCode {
                name: "新雅社区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "红庙子街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "嘉和园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新通社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金泰社区居委会",
                code: "003",
            },
            VillageCode {
                name: "天海社区居委会",
                code: "004",
            },
            VillageCode {
                name: "泰秀社区居委会",
                code: "005",
            },
            VillageCode {
                name: "汇珊园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "汇芙园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西城街西社区",
                code: "008",
            },
            VillageCode {
                name: "西城街北社区",
                code: "009",
            },
            VillageCode {
                name: "福地社区",
                code: "010",
            },
            VillageCode {
                name: "新居社区",
                code: "011",
            },
            VillageCode {
                name: "西环中路东社区",
                code: "012",
            },
            VillageCode {
                name: "金地社区居委会",
                code: "013",
            },
            VillageCode {
                name: "康悦社区居委会",
                code: "014",
            },
            VillageCode {
                name: "新乐社区居委会",
                code: "015",
            },
            VillageCode {
                name: "福星社区居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "长胜东街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "二道泉子社区",
                code: "001",
            },
            VillageCode {
                name: "二十里店社区",
                code: "002",
            },
            VillageCode {
                name: "泉台子社区",
                code: "003",
            },
            VillageCode {
                name: "西梁社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "长胜西街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "林榆台社区",
                code: "001",
            },
            VillageCode {
                name: "苏家庄社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "长胜南街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "惠达社区",
                code: "001",
            },
            VillageCode {
                name: "荣泽社区",
                code: "002",
            },
            VillageCode {
                name: "安泰社区",
                code: "003",
            },
            VillageCode {
                name: "薛家槽北社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "火车南站街道",
        code: "014",
        villages: &[VillageCode {
            name: "火车南站片区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "仓房沟片区街道",
        code: "015",
        villages: &[
            VillageCode {
                name: "仓盛社区",
                code: "001",
            },
            VillageCode {
                name: "仓欣社区",
                code: "002",
            },
            VillageCode {
                name: "仓泉社区",
                code: "003",
            },
            VillageCode {
                name: "仓泊社区",
                code: "004",
            },
            VillageCode {
                name: "仓郁社区",
                code: "005",
            },
            VillageCode {
                name: "仓园社区",
                code: "006",
            },
            VillageCode {
                name: "仓谷社区",
                code: "007",
            },
            VillageCode {
                name: "仓荣社区",
                code: "008",
            },
            VillageCode {
                name: "仓祥社区",
                code: "009",
            },
            VillageCode {
                name: "明月社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "环卫路街道",
        code: "016",
        villages: &[
            VillageCode {
                name: "马料地社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中泰社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新运社区居委会",
                code: "003",
            },
            VillageCode {
                name: "环卫路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "四道岔社区居委会",
                code: "005",
            },
            VillageCode {
                name: "建陶社区居委会",
                code: "006",
            },
            VillageCode {
                name: "大浦沟社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "环卫路南社区",
                code: "008",
            },
            VillageCode {
                name: "华南巷社区",
                code: "009",
            },
            VillageCode {
                name: "西山东街南社区",
                code: "010",
            },
            VillageCode {
                name: "辛福屯社区",
                code: "011",
            },
            VillageCode {
                name: "芙蓉巷社区居委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "骑马山街道",
        code: "017",
        villages: &[
            VillageCode {
                name: "儿童村社区居委会",
                code: "001",
            },
            VillageCode {
                name: "骑马山路西社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西盛社区居委会",
                code: "003",
            },
            VillageCode {
                name: "西源社区",
                code: "004",
            },
            VillageCode {
                name: "水库街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "兴荣巷社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "平顶山街道",
        code: "018",
        villages: &[
            VillageCode {
                name: "平顶山社区居委会",
                code: "001",
            },
            VillageCode {
                name: "克西路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北园春社区居委会",
                code: "003",
            },
            VillageCode {
                name: "公交社区居委会",
                code: "004",
            },
            VillageCode {
                name: "头宫社区居委会",
                code: "005",
            },
            VillageCode {
                name: "阿勒泰路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "锦福苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "汇嘉园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "南苑社区",
                code: "009",
            },
            VillageCode {
                name: "春熙街南社区",
                code: "010",
            },
            VillageCode {
                name: "清河路社区",
                code: "011",
            },
            VillageCode {
                name: "金沙江路西社区",
                code: "012",
            },
            VillageCode {
                name: "汇南社区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "兵团农十二师一零四团",
        code: "019",
        villages: &[
            VillageCode {
                name: "西城北社区",
                code: "001",
            },
            VillageCode {
                name: "西城南社区",
                code: "002",
            },
            VillageCode {
                name: "西城东社区",
                code: "003",
            },
            VillageCode {
                name: "西城西社区",
                code: "004",
            },
            VillageCode {
                name: "桃园社区",
                code: "005",
            },
            VillageCode {
                name: "紫金城南社区",
                code: "006",
            },
            VillageCode {
                name: "紫金城北社区",
                code: "007",
            },
            VillageCode {
                name: "悦府山水社区",
                code: "008",
            },
            VillageCode {
                name: "连兴社区",
                code: "009",
            },
            VillageCode {
                name: "苜蓿沟社区",
                code: "010",
            },
            VillageCode {
                name: "四道岔社区",
                code: "011",
            },
            VillageCode {
                name: "新建生产建设兵团第十二师一〇四团润安社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "新建生产建设兵团第十二师一〇四团桌子山社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "新疆生产建设兵团第十二师一〇四团兴业路社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "新疆生产建设兵团第十二师一〇四团新河社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "牧一场生活区",
                code: "016",
            },
            VillageCode {
                name: "牧二场生活区",
                code: "017",
            },
            VillageCode {
                name: "牧三场生活区",
                code: "018",
            },
            VillageCode {
                name: "一连生活区",
                code: "019",
            },
            VillageCode {
                name: "二连生活区",
                code: "020",
            },
            VillageCode {
                name: "三连生活区",
                code: "021",
            },
            VillageCode {
                name: "畜牧连生活区",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "兵团十二师西山农场",
        code: "020",
        villages: &[
            VillageCode {
                name: "安康社区",
                code: "001",
            },
            VillageCode {
                name: "锦绣家园社区",
                code: "002",
            },
            VillageCode {
                name: "第三社区",
                code: "003",
            },
            VillageCode {
                name: "一连生活区",
                code: "004",
            },
            VillageCode {
                name: "二连社区生活区",
                code: "005",
            },
        ],
    },
];

static TOWNS_XJ_004: [TownCode; 23] = [
    TownCode {
        name: "北京路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "北京路口社区居委会",
                code: "001",
            },
            VillageCode {
                name: "联建社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北京南路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "中营工社区居委会",
                code: "004",
            },
            VillageCode {
                name: "蜘蛛山社区居委会",
                code: "005",
            },
            VillageCode {
                name: "呈祥社区居委会",
                code: "006",
            },
            VillageCode {
                name: "阳光社区居委会",
                code: "007",
            },
            VillageCode {
                name: "锦海巷社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "二工街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "天津北路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新科社区居委会",
                code: "002",
            },
            VillageCode {
                name: "江苏东路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "河南东路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "长青社区居委会",
                code: "005",
            },
            VillageCode {
                name: "科学北路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "小西沟社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新体社区居委会",
                code: "008",
            },
            VillageCode {
                name: "北京中路社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "三工街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "和平桥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "百园路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "祥和社区居委会",
                code: "003",
            },
            VillageCode {
                name: "三工社区居委会",
                code: "004",
            },
            VillageCode {
                name: "花都社区居委会",
                code: "005",
            },
            VillageCode {
                name: "经环社区居委会",
                code: "006",
            },
            VillageCode {
                name: "景苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "汇轩园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新联路社区居委会",
                code: "009",
            },
            VillageCode {
                name: "北京北路社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "石油新村街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "石油新村社区居委会",
                code: "001",
            },
            VillageCode {
                name: "阿勒泰路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "工运司社区居委会",
                code: "003",
            },
            VillageCode {
                name: "红庙社区居委会",
                code: "004",
            },
            VillageCode {
                name: "九家湾社区居委会",
                code: "005",
            },
            VillageCode {
                name: "商运司社区居委会",
                code: "006",
            },
            VillageCode {
                name: "外运司社区居委会",
                code: "007",
            },
            VillageCode {
                name: "九家湾路北社区居委会",
                code: "008",
            },
            VillageCode {
                name: "锦峰社区居委会",
                code: "009",
            },
            VillageCode {
                name: "景宁社区居委会",
                code: "010",
            },
            VillageCode {
                name: "锦山社区居委会",
                code: "011",
            },
            VillageCode {
                name: "锦域社区",
                code: "012",
            },
            VillageCode {
                name: "明山社区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "迎宾路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "迎宾北二路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "三友社区居委会",
                code: "002",
            },
            VillageCode {
                name: "友谊路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新明社区居委会",
                code: "004",
            },
            VillageCode {
                name: "迎宾北一路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "迎宾路西社区居委会",
                code: "006",
            },
            VillageCode {
                name: "兰亭社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "喀什东路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "铁桥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "五建社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新兴社区居委会",
                code: "003",
            },
            VillageCode {
                name: "四平路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "汇园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "喀什东路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "晨光社区居委会",
                code: "007",
            },
            VillageCode {
                name: "乌东站社区居委会",
                code: "008",
            },
            VillageCode {
                name: "文轩社区居委会",
                code: "009",
            },
            VillageCode {
                name: "京疆路社区",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "八家户街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "东方社区居委会",
                code: "001",
            },
            VillageCode {
                name: "建新社区居委会",
                code: "002",
            },
            VillageCode {
                name: "鸿阳社区",
                code: "003",
            },
            VillageCode {
                name: "新慧社区",
                code: "004",
            },
            VillageCode {
                name: "八家户社区居委会",
                code: "005",
            },
            VillageCode {
                name: "河滩北路西社区居委会",
                code: "006",
            },
            VillageCode {
                name: "鲤鱼山南路社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "银川路街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "鲤鱼山社区居委会",
                code: "001",
            },
            VillageCode {
                name: "银川路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西八家户社区居委会",
                code: "003",
            },
            VillageCode {
                name: "上八家户社区居委会",
                code: "004",
            },
            VillageCode {
                name: "小红桥社区居委会",
                code: "005",
            },
            VillageCode {
                name: "锦苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "汇展园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "天山花园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "华源社区居委会",
                code: "009",
            },
            VillageCode {
                name: "泉州街社区居委会",
                code: "010",
            },
            VillageCode {
                name: "西八家户路东社区居委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "南纬路街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "青海路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南一路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "南二路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南三路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "北二路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "太原路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "北一路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "北纬一路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "北纬三路社区居委会",
                code: "009",
            },
            VillageCode {
                name: "河南西路社区",
                code: "010",
            },
            VillageCode {
                name: "锦江社区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "杭州路街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "金谷社区居委会",
                code: "001",
            },
            VillageCode {
                name: "秀林园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "振兴社区居委会",
                code: "003",
            },
            VillageCode {
                name: "河北西路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "崇文社区居委会",
                code: "005",
            },
            VillageCode {
                name: "兴奥社区居委会",
                code: "006",
            },
            VillageCode {
                name: "杭州东街社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "鲤鱼山街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "府友路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新医社区居委会",
                code: "002",
            },
            VillageCode {
                name: "贵州东路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "京都小区社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "百园路街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "兴安社区居委会",
                code: "001",
            },
            VillageCode {
                name: "唐山路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "通安南路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "友兴街南社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "正扬路街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "冬融街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "抚顺街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "正扬社区居委会",
                code: "003",
            },
            VillageCode {
                name: "金藤社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东站北社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "机场街道",
        code: "014",
        villages: &[
            VillageCode {
                name: "飞机场社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东风社区居委会",
                code: "002",
            },
            VillageCode {
                name: "安新社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "友谊路街道",
        code: "015",
        villages: &[
            VillageCode {
                name: "迎宾北路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "永昌社区居委会",
                code: "002",
            },
            VillageCode {
                name: "永盛社区居委会",
                code: "003",
            },
            VillageCode {
                name: "永睦社区居委会",
                code: "004",
            },
            VillageCode {
                name: "永泰社区",
                code: "005",
            },
            VillageCode {
                name: "迎亚社区居委会",
                code: "006",
            },
            VillageCode {
                name: "澳华社区居委会",
                code: "007",
            },
            VillageCode {
                name: "地窝堡社区居委会",
                code: "008",
            },
            VillageCode {
                name: "迎宾路北社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "高新街街道",
        code: "016",
        villages: &[
            VillageCode {
                name: "天津南路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "昆明路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "桂林路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "新洲社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "美林社区居委会",
                code: "005",
            },
            VillageCode {
                name: "长沙路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "长春南路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "苏州路社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "长春中路街道",
        code: "017",
        villages: &[
            VillageCode {
                name: "长春北路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新盛社区居委会",
                code: "002",
            },
            VillageCode {
                name: "锦秀社区居委会",
                code: "003",
            },
            VillageCode {
                name: "锦程社区居委会",
                code: "004",
            },
            VillageCode {
                name: "海宝社区居委会",
                code: "005",
            },
            VillageCode {
                name: "万盛社区居委会",
                code: "006",
            },
            VillageCode {
                name: "天津路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新和社区居委会",
                code: "008",
            },
            VillageCode {
                name: "长河社区居委会",
                code: "009",
            },
            VillageCode {
                name: "长治路社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "安宁渠镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "宁华社区居委会",
                code: "001",
            },
            VillageCode {
                name: "安馨社区居委会",
                code: "002",
            },
            VillageCode {
                name: "安泰社区",
                code: "003",
            },
            VillageCode {
                name: "安宁渠村委会",
                code: "004",
            },
            VillageCode {
                name: "北大路村委会",
                code: "005",
            },
            VillageCode {
                name: "东戈壁村委会",
                code: "006",
            },
            VillageCode {
                name: "河西村委会",
                code: "007",
            },
            VillageCode {
                name: "保昌堡村委会",
                code: "008",
            },
            VillageCode {
                name: "东村村委会",
                code: "009",
            },
            VillageCode {
                name: "西村村委会",
                code: "010",
            },
            VillageCode {
                name: "广东庄子村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "二工乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "京轩社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金华路社区",
                code: "002",
            },
            VillageCode {
                name: "唐山路东社区",
                code: "003",
            },
            VillageCode {
                name: "宁波街社区",
                code: "004",
            },
            VillageCode {
                name: "湖州路西社区",
                code: "005",
            },
            VillageCode {
                name: "湖州路东社区",
                code: "006",
            },
            VillageCode {
                name: "三工村委会",
                code: "007",
            },
            VillageCode {
                name: "百园路新村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "地窝堡乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "宣仁墩北街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "小地窝堡东街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "小地窝堡西街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "宣仁墩南街社区",
                code: "004",
            },
            VillageCode {
                name: "小地窝堡中街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "小地窝堡村委会",
                code: "006",
            },
            VillageCode {
                name: "宣仁墩村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "青格达湖乡",
        code: "021",
        villages: &[
            VillageCode {
                name: "新联村委会",
                code: "001",
            },
            VillageCode {
                name: "青格达湖村委会",
                code: "002",
            },
            VillageCode {
                name: "联合村委会",
                code: "003",
            },
            VillageCode {
                name: "天山村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "六十户乡",
        code: "022",
        villages: &[
            VillageCode {
                name: "星火村委会",
                code: "001",
            },
            VillageCode {
                name: "三宫梁村委会",
                code: "002",
            },
            VillageCode {
                name: "六十户村委会",
                code: "003",
            },
            VillageCode {
                name: "八段村委会",
                code: "004",
            },
            VillageCode {
                name: "大梁村委会",
                code: "005",
            },
            VillageCode {
                name: "哈族新村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "兵团第十二师养禽场",
        code: "023",
        villages: &[
            VillageCode {
                name: "新疆生产建设兵团第十二师一〇四团南通路东社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "通嘉世纪城社区",
                code: "002",
            },
            VillageCode {
                name: "一〇四团常州街南社区",
                code: "003",
            },
            VillageCode {
                name: "一〇四团常州街北社区",
                code: "004",
            },
            VillageCode {
                name: "新疆生产建设兵团第十二师一〇四团南通路西社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "新天润社区",
                code: "006",
            },
        ],
    },
];

static TOWNS_XJ_005: [TownCode; 15] = [
    TownCode {
        name: "七纺街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "北山社区居委会",
                code: "001",
            },
            VillageCode {
                name: "沿河路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "风景社区居委会",
                code: "003",
            },
            VillageCode {
                name: "田园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "众泰社区居委会",
                code: "005",
            },
            VillageCode {
                name: "温泉社区居委会",
                code: "006",
            },
            VillageCode {
                name: "康居社区居委会",
                code: "007",
            },
            VillageCode {
                name: "沿河北社区居委会",
                code: "008",
            },
            VillageCode {
                name: "温泉北社区居委会",
                code: "009",
            },
            VillageCode {
                name: "景泉社区居委会",
                code: "010",
            },
            VillageCode {
                name: "水磨社区居委会",
                code: "011",
            },
            VillageCode {
                name: "秀泉社区居委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "六道湾街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "五星北路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西虹东路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "德裕社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南湖花苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "克拉玛依东路社区居委",
                code: "005",
            },
            VillageCode {
                name: "北山坡社区居委会",
                code: "006",
            },
            VillageCode {
                name: "双拥社区居委会",
                code: "007",
            },
            VillageCode {
                name: "六道湾社区居委会",
                code: "008",
            },
            VillageCode {
                name: "天平社区居委会",
                code: "009",
            },
            VillageCode {
                name: "融鑫社区居委会",
                code: "010",
            },
            VillageCode {
                name: "沁园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "红星社区居委会",
                code: "012",
            },
            VillageCode {
                name: "康苑社区居委会",
                code: "013",
            },
            VillageCode {
                name: "青翠社区居委会",
                code: "014",
            },
            VillageCode {
                name: "青玉社区居委会",
                code: "015",
            },
            VillageCode {
                name: "斜井西社区居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "苇湖梁街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "龙翔路南社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新光社区居委会",
                code: "002",
            },
            VillageCode {
                name: "昌盛祥社区居委会",
                code: "003",
            },
            VillageCode {
                name: "融合社区居委会",
                code: "004",
            },
            VillageCode {
                name: "立井南社区居委会",
                code: "005",
            },
            VillageCode {
                name: "立井北社区居委会",
                code: "006",
            },
            VillageCode {
                name: "立井东社区居委会",
                code: "007",
            },
            VillageCode {
                name: "立井西社区居委会",
                code: "008",
            },
            VillageCode {
                name: "融睦社区居委会",
                code: "009",
            },
            VillageCode {
                name: "龙盛街南社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "八道湾街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "丰华社区居委会",
                code: "001",
            },
            VillageCode {
                name: "绿洲社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新建社区居委会",
                code: "003",
            },
            VillageCode {
                name: "八道湾西社区居委会",
                code: "004",
            },
            VillageCode {
                name: "九道湾路社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "新民路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "新民西街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "红塔社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新民西街北社区居委会",
                code: "003",
            },
            VillageCode {
                name: "犁铧街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "爱心社区居委会",
                code: "005",
            },
            VillageCode {
                name: "红山路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新城社区居委会",
                code: "007",
            },
            VillageCode {
                name: "荣惠社区居委会",
                code: "008",
            },
            VillageCode {
                name: "银河社区居委会",
                code: "009",
            },
            VillageCode {
                name: "成功社区居委会",
                code: "010",
            },
            VillageCode {
                name: "新民社区居委会",
                code: "011",
            },
            VillageCode {
                name: "新大地社区居委会",
                code: "012",
            },
            VillageCode {
                name: "南大湖社区居委会",
                code: "013",
            },
            VillageCode {
                name: "成功东社区居委会",
                code: "014",
            },
            VillageCode {
                name: "祥泰社区居委会",
                code: "015",
            },
            VillageCode {
                name: "兴惠社区居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "南湖南路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "克东路北社区居委会",
                code: "001",
            },
            VillageCode {
                name: "安居北路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "旭东社区居委会",
                code: "003",
            },
            VillageCode {
                name: "安居社区居委会",
                code: "004",
            },
            VillageCode {
                name: "华祥社区居委会",
                code: "005",
            },
            VillageCode {
                name: "林科院社区居委会",
                code: "006",
            },
            VillageCode {
                name: "劳动街社区居委会",
                code: "007",
            },
            VillageCode {
                name: "友谊社区居委会",
                code: "008",
            },
            VillageCode {
                name: "绿苑社区委会",
                code: "009",
            },
            VillageCode {
                name: "南湖广场社区居委会",
                code: "010",
            },
            VillageCode {
                name: "南湖西路社区居委会",
                code: "011",
            },
            VillageCode {
                name: "宁苑社区居委会",
                code: "012",
            },
            VillageCode {
                name: "华清社区居委会",
                code: "013",
            },
            VillageCode {
                name: "宁安社区居委会",
                code: "014",
            },
            VillageCode {
                name: "南湖南路西社区居委会",
                code: "015",
            },
            VillageCode {
                name: "绿荫社区居委会",
                code: "016",
            },
            VillageCode {
                name: "昆仑路南社区居委会",
                code: "017",
            },
            VillageCode {
                name: "宁峰社区居委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "南湖北路街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "河滩社区居委会",
                code: "001",
            },
            VillageCode {
                name: "宏怡花园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "友好花园西居委会",
                code: "003",
            },
            VillageCode {
                name: "王家梁社区居委会",
                code: "004",
            },
            VillageCode {
                name: "昆仑路北社区居委会",
                code: "005",
            },
            VillageCode {
                name: "王家梁东社区居委会",
                code: "006",
            },
            VillageCode {
                name: "宏瑞社区居委会",
                code: "007",
            },
            VillageCode {
                name: "康和社区居委会",
                code: "008",
            },
            VillageCode {
                name: "百万庄社区居委会",
                code: "009",
            },
            VillageCode {
                name: "碧园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "顺和苑社区居委会",
                code: "011",
            },
            VillageCode {
                name: "南湖北路东社区居委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "七道湾街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "新兴社区居委会",
                code: "001",
            },
            VillageCode {
                name: "龙瑞街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "七道湾东街南社区居委会",
                code: "003",
            },
            VillageCode {
                name: "会展社区居委会",
                code: "004",
            },
            VillageCode {
                name: "和居社区居委会",
                code: "005",
            },
            VillageCode {
                name: "红光社区居委会",
                code: "006",
            },
            VillageCode {
                name: "文源社区居委会",
                code: "007",
            },
            VillageCode {
                name: "文汇社区居委会",
                code: "008",
            },
            VillageCode {
                name: "七道湾北社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新丰社区居委会",
                code: "010",
            },
            VillageCode {
                name: "会展中心社区居委会",
                code: "011",
            },
            VillageCode {
                name: "和锦社区居委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "榆树沟街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "温泉东路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "温泉南社区居委会",
                code: "002",
            },
            VillageCode {
                name: "水磨园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "学府社区居委会",
                code: "004",
            },
            VillageCode {
                name: "安平社区居委会",
                code: "005",
            },
            VillageCode {
                name: "榆树沟社区居委会",
                code: "006",
            },
            VillageCode {
                name: "温康社区居委会",
                code: "007",
            },
            VillageCode {
                name: "安东社区居委会",
                code: "008",
            },
            VillageCode {
                name: "宜丰社区居委会",
                code: "009",
            },
            VillageCode {
                name: "水磨沟村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "石人子沟街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "蝴蝶谷社区居委会",
                code: "001",
            },
            VillageCode {
                name: "石人子沟村委会",
                code: "002",
            },
            VillageCode {
                name: "葛家沟村委会",
                code: "003",
            },
            VillageCode {
                name: "涝坝沟村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "水塔山街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "花苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新居巷社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新纺社区居委会",
                code: "003",
            },
            VillageCode {
                name: "温泉西路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "清泉社区居委会",
                code: "005",
            },
            VillageCode {
                name: "南山社区居委会",
                code: "006",
            },
            VillageCode {
                name: "文苑巷社区居委会",
                code: "007",
            },
            VillageCode {
                name: "水塔山社区居委会",
                code: "008",
            },
            VillageCode {
                name: "怡和社区居委",
                code: "009",
            },
            VillageCode {
                name: "苇湖庄社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "华光街街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "和谐园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "斜井东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "斜井南社区居委会",
                code: "003",
            },
            VillageCode {
                name: "昆仑东街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "昆仑东街北社区居委会",
                code: "005",
            },
            VillageCode {
                name: "美丰社区居委会",
                code: "006",
            },
            VillageCode {
                name: "运成社区居委会",
                code: "007",
            },
            VillageCode {
                name: "芙蓉社区居委会",
                code: "008",
            },
            VillageCode {
                name: "苇湖庄西社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "龙盛街街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "会盛巷社区居委会",
                code: "001",
            },
            VillageCode {
                name: "苏州路立交桥居委会",
                code: "002",
            },
            VillageCode {
                name: "新跃社区居委会",
                code: "003",
            },
            VillageCode {
                name: "八家户社区居委会",
                code: "004",
            },
            VillageCode {
                name: "红光山南社区居委会",
                code: "005",
            },
            VillageCode {
                name: "康宁社区居委会",
                code: "006",
            },
            VillageCode {
                name: "康辉社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "振安街街道",
        code: "014",
        villages: &[
            VillageCode {
                name: "春祥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "振安街南社区居委会",
                code: "002",
            },
            VillageCode {
                name: "振安街北社区居委会",
                code: "003",
            },
            VillageCode {
                name: "七道湾东街北社区居委会",
                code: "004",
            },
            VillageCode {
                name: "鸿园南路东社区居委会",
                code: "005",
            },
            VillageCode {
                name: "翼翔社区居委会",
                code: "006",
            },
            VillageCode {
                name: "山润社区居委会",
                code: "007",
            },
            VillageCode {
                name: "青润社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "河马泉街道",
        code: "015",
        villages: &[
            VillageCode {
                name: "观园路北社区居委会",
                code: "001",
            },
            VillageCode {
                name: "观园路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "瑞景社区居委会",
                code: "003",
            },
            VillageCode {
                name: "和奕社区居委会",
                code: "004",
            },
            VillageCode {
                name: "雪莲山社区居委会",
                code: "005",
            },
            VillageCode {
                name: "八道湾东社区居委会",
                code: "006",
            },
            VillageCode {
                name: "葛家沟西社区居委会",
                code: "007",
            },
        ],
    },
];

static TOWNS_XJ_006: [TownCode; 18] = [
    TownCode {
        name: "钢城片区街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "魏户滩路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "洛克伦街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新村街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新立社区居委会",
                code: "004",
            },
            VillageCode {
                name: "柯坪路北社区居委会",
                code: "005",
            },
            VillageCode {
                name: "西域社区居委会",
                code: "006",
            },
            VillageCode {
                name: "东干渠社区",
                code: "007",
            },
            VillageCode {
                name: "顺河路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新风路社区居委会",
                code: "009",
            },
            VillageCode {
                name: "八一路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "柯坪路西社区居委会",
                code: "011",
            },
            VillageCode {
                name: "滨河社区居委会",
                code: "012",
            },
            VillageCode {
                name: "新村东街社区居委会",
                code: "013",
            },
            VillageCode {
                name: "顺河北路社区居委会",
                code: "014",
            },
            VillageCode {
                name: "平安社区居委会",
                code: "015",
            },
            VillageCode {
                name: "顺河路西社区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "火车西站片区街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "车辆段路东社区居委会",
                code: "001",
            },
            VillageCode {
                name: "三十五户路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "中枢路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东林街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "沟西路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "西园街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "机务段路西社区居委会",
                code: "007",
            },
            VillageCode {
                name: "景明社区居委会",
                code: "008",
            },
            VillageCode {
                name: "秀丽社区居委会",
                code: "009",
            },
            VillageCode {
                name: "乾园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "西林社区",
                code: "011",
            },
            VillageCode {
                name: "北站五路社区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "王家沟片区街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "王家沟储运新村社区居委会",
                code: "001",
            },
            VillageCode {
                name: "王家沟新园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西坪社区",
                code: "003",
            },
            VillageCode {
                name: "百园社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "乌昌路片区街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "永红社区居委会",
                code: "001",
            },
            VillageCode {
                name: "孝感路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "楼兰社区居委会",
                code: "003",
            },
            VillageCode {
                name: "沙坪路社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "北站西路片区街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "北站路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北站路北社区居委会",
                code: "002",
            },
            VillageCode {
                name: "丝路社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "中亚北路片区街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "科技园路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "文苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "上海路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "北彩门社区居委会",
                code: "004",
            },
            VillageCode {
                name: "喀什西路社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "迎宾桥社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "尚北社区",
                code: "007",
            },
            VillageCode {
                name: "桥东社区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "中亚南路片区街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "海滨社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中亚南路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "浦东街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "白石桥社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "广州街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "卫星路社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "团结新村社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "南彩门社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "青水湾社区居委会",
                code: "009",
            },
            VillageCode {
                name: "铁路花园社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "嵩山街片区街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "泰山街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "黄山街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西湖社区居委会",
                code: "003",
            },
            VillageCode {
                name: "紫阳湖社区居委会",
                code: "004",
            },
            VillageCode {
                name: "融北社区居委会",
                code: "005",
            },
            VillageCode {
                name: "融南社区居委会",
                code: "006",
            },
            VillageCode {
                name: "庐山街社区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "高铁片区街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "绿谷社区",
                code: "001",
            },
            VillageCode {
                name: "香山街社区",
                code: "002",
            },
            VillageCode {
                name: "天鹅湖社区居委会",
                code: "003",
            },
            VillageCode {
                name: "澎湖路社区",
                code: "004",
            },
            VillageCode {
                name: "玄武湖社区居委会",
                code: "005",
            },
            VillageCode {
                name: "荣泰社区",
                code: "006",
            },
            VillageCode {
                name: "锦霞社区",
                code: "007",
            },
            VillageCode {
                name: "绿岭社区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "白鸟湖片区街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "馨园社区",
                code: "001",
            },
            VillageCode {
                name: "祥云社区",
                code: "002",
            },
            VillageCode {
                name: "金桥社区",
                code: "003",
            },
            VillageCode {
                name: "城缘社区",
                code: "004",
            },
            VillageCode {
                name: "红岩社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "西湖片区街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "九华山街社区",
                code: "001",
            },
            VillageCode {
                name: "东山坡社区",
                code: "002",
            },
            VillageCode {
                name: "火山南社区",
                code: "003",
            },
            VillageCode {
                name: "红沙滩路社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "北站东路片区街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "北站二路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "瑞昌街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "丰北社区居委会",
                code: "003",
            },
            VillageCode {
                name: "兴丰社区",
                code: "004",
            },
            VillageCode {
                name: "丰田村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "两河片区街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "东南沟村委会",
                code: "001",
            },
            VillageCode {
                name: "萨尔达坂村委会",
                code: "002",
            },
            VillageCode {
                name: "马家庄子村委会",
                code: "003",
            },
            VillageCode {
                name: "大泉村委会",
                code: "004",
            },
            VillageCode {
                name: "阿合乔克村",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "乌鲁木齐站片区街道",
        code: "014",
        villages: &[VillageCode {
            name: "乌鲁木齐站虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "区直辖村级区划",
        code: "015",
        villages: &[VillageCode {
            name: "河南庄村委会",
            code: "001",
        }],
    },
    TownCode {
        name: "兵团十二师三坪农场",
        code: "016",
        villages: &[
            VillageCode {
                name: "屯坪社区",
                code: "001",
            },
            VillageCode {
                name: "祥和社区",
                code: "002",
            },
            VillageCode {
                name: "融合社区",
                code: "003",
            },
            VillageCode {
                name: "恒汇社区",
                code: "004",
            },
            VillageCode {
                name: "三坪一连队",
                code: "005",
            },
            VillageCode {
                name: "三坪二连队",
                code: "006",
            },
            VillageCode {
                name: "三坪三连队",
                code: "007",
            },
            VillageCode {
                name: "三坪六连队",
                code: "008",
            },
            VillageCode {
                name: "三坪四连队",
                code: "009",
            },
            VillageCode {
                name: "三坪五连队",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "兵团十二师五一农场",
        code: "017",
        villages: &[
            VillageCode {
                name: "怡和园社区",
                code: "001",
            },
            VillageCode {
                name: "怡丰园南社区",
                code: "002",
            },
            VillageCode {
                name: "怡丰园北社区",
                code: "003",
            },
            VillageCode {
                name: "兴业街社区",
                code: "004",
            },
            VillageCode {
                name: "一连队",
                code: "005",
            },
            VillageCode {
                name: "二连队",
                code: "006",
            },
            VillageCode {
                name: "三连队",
                code: "007",
            },
            VillageCode {
                name: "四连队",
                code: "008",
            },
            VillageCode {
                name: "五连队",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "新疆兵团十二师头屯河农场",
        code: "018",
        villages: &[
            VillageCode {
                name: "同和幸福城一社区",
                code: "001",
            },
            VillageCode {
                name: "同和幸福城二社区",
                code: "002",
            },
            VillageCode {
                name: "大门院社区",
                code: "003",
            },
            VillageCode {
                name: "绿洲街南社区",
                code: "004",
            },
            VillageCode {
                name: "绿洲街北社区",
                code: "005",
            },
            VillageCode {
                name: "一连队",
                code: "006",
            },
            VillageCode {
                name: "二连队",
                code: "007",
            },
            VillageCode {
                name: "三连队",
                code: "008",
            },
        ],
    },
];

static TOWNS_XJ_007: [TownCode; 8] = [
    TownCode {
        name: "艾维尔沟街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "工人新村社区居委会",
                code: "001",
            },
            VillageCode {
                name: "七一平峒社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新街社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "乌拉泊街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "福利路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "同心路东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "同心路西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "同心路南社区居委会",
                code: "004",
            },
            VillageCode {
                name: "窝尔图社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "达坂城区盐湖街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "盐湖社区居委会",
                code: "001",
            },
            VillageCode {
                name: "盐湖北社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "达坂城镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "达坂村村委会",
                code: "001",
            },
            VillageCode {
                name: "八家户村委会",
                code: "002",
            },
            VillageCode {
                name: "红山嘴子村委会",
                code: "003",
            },
            VillageCode {
                name: "达坂城社区村委会",
                code: "004",
            },
            VillageCode {
                name: "洛宾社区村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "东沟乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "苇子村村委会",
                code: "001",
            },
            VillageCode {
                name: "王家庄村村委会",
                code: "002",
            },
            VillageCode {
                name: "兰洲湾村村委会",
                code: "003",
            },
            VillageCode {
                name: "方家沟村村委会",
                code: "004",
            },
            VillageCode {
                name: "东湖村村委会",
                code: "005",
            },
            VillageCode {
                name: "月牙湾村村委会",
                code: "006",
            },
            VillageCode {
                name: "高崖子村村委会",
                code: "007",
            },
            VillageCode {
                name: "苇兰社区村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "西沟乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "沙梁子村村委会",
                code: "001",
            },
            VillageCode {
                name: "水磨村村委会",
                code: "002",
            },
            VillageCode {
                name: "泉泉湖村村委会",
                code: "003",
            },
            VillageCode {
                name: "陈麻子村村委会",
                code: "004",
            },
            VillageCode {
                name: "雷家沟村村委会",
                code: "005",
            },
            VillageCode {
                name: "桦树林社区村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "阿克苏乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "阿克苏村村委会",
                code: "001",
            },
            VillageCode {
                name: "黑沟村村委会",
                code: "002",
            },
            VillageCode {
                name: "黄渠泉村村委会",
                code: "003",
            },
            VillageCode {
                name: "大河沿村村委会",
                code: "004",
            },
            VillageCode {
                name: "牧业一队",
                code: "005",
            },
            VillageCode {
                name: "牧业二队",
                code: "006",
            },
            VillageCode {
                name: "鹰舞社区村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "柴窝堡管委会",
        code: "008",
        villages: &[
            VillageCode {
                name: "柴窝堡社区",
                code: "001",
            },
            VillageCode {
                name: "窝尔图社区",
                code: "002",
            },
            VillageCode {
                name: "白杨沟村委会",
                code: "003",
            },
            VillageCode {
                name: "兴睦社区村委会",
                code: "004",
            },
            VillageCode {
                name: "柴源村委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_XJ_008: [TownCode; 16] = [
    TownCode {
        name: "石化街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "中兴社区居委会",
                code: "001",
            },
            VillageCode {
                name: "朝阳社区居委会",
                code: "002",
            },
            VillageCode {
                name: "安和社区居委会",
                code: "003",
            },
            VillageCode {
                name: "光明社区居委会",
                code: "004",
            },
            VillageCode {
                name: "奋进社区居委会",
                code: "005",
            },
            VillageCode {
                name: "佳瑞社区居委会",
                code: "006",
            },
            VillageCode {
                name: "佳祥社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "地磅街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "东山社区居委会",
                code: "001",
            },
            VillageCode {
                name: "碱沟社区居委会",
                code: "002",
            },
            VillageCode {
                name: "大洪沟社区居委会",
                code: "003",
            },
            VillageCode {
                name: "金河社区居委会",
                code: "004",
            },
            VillageCode {
                name: "健民社区居委会",
                code: "005",
            },
            VillageCode {
                name: "东瑞北路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "东盛社区居委会",
                code: "007",
            },
            VillageCode {
                name: "卡子湾村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "卡子湾街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "育林社区居委会",
                code: "001",
            },
            VillageCode {
                name: "佳园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "利民社区居委会",
                code: "003",
            },
            VillageCode {
                name: "象新社区居委会",
                code: "004",
            },
            VillageCode {
                name: "创业社区居委会",
                code: "005",
            },
            VillageCode {
                name: "文化路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "华兴社区居委会",
                code: "007",
            },
            VillageCode {
                name: "卡子湾社区居委会",
                code: "008",
            },
            VillageCode {
                name: "华盛社区居委会",
                code: "009",
            },
            VillageCode {
                name: "华龙社区居委会",
                code: "010",
            },
            VillageCode {
                name: "红光山社区居委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "古牧地东路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "新星社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "南苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "祥和社区居委会",
                code: "004",
            },
            VillageCode {
                name: "益民社区居委会",
                code: "005",
            },
            VillageCode {
                name: "永乐社区居委会",
                code: "006",
            },
            VillageCode {
                name: "振兴社区居委会",
                code: "007",
            },
            VillageCode {
                name: "祥瑞社区居委会",
                code: "008",
            },
            VillageCode {
                name: "丰源社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "古牧地西路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "新华社区居委会",
                code: "001",
            },
            VillageCode {
                name: "园艺社区居委会",
                code: "002",
            },
            VillageCode {
                name: "八方社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "安居社区居委会",
                code: "005",
            },
            VillageCode {
                name: "明珠社区居委会",
                code: "006",
            },
            VillageCode {
                name: "西营社区居委会",
                code: "007",
            },
            VillageCode {
                name: "佳和社区居委会",
                code: "008",
            },
            VillageCode {
                name: "乐业社区居委会",
                code: "009",
            },
            VillageCode {
                name: "永兴社区居委会",
                code: "010",
            },
            VillageCode {
                name: "汇祥社区居委会",
                code: "011",
            },
            VillageCode {
                name: "汇和社区居委会",
                code: "012",
            },
            VillageCode {
                name: "博苑社区居委会",
                code: "013",
            },
            VillageCode {
                name: "四方社区居委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "南路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "龙泉社区居委会",
                code: "001",
            },
            VillageCode {
                name: "同心社区居委会",
                code: "002",
            },
            VillageCode {
                name: "众和社区居委会",
                code: "003",
            },
            VillageCode {
                name: "常乐社区居委会",
                code: "004",
            },
            VillageCode {
                name: "小水渠社区居委会",
                code: "005",
            },
            VillageCode {
                name: "虹桥社区居委会",
                code: "006",
            },
            VillageCode {
                name: "兴业社区居委会",
                code: "007",
            },
            VillageCode {
                name: "金安社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "永祥街街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "华强社区居委会",
                code: "001",
            },
            VillageCode {
                name: "华瑞社区居委会",
                code: "002",
            },
            VillageCode {
                name: "华丰社区居委会",
                code: "003",
            },
            VillageCode {
                name: "华成社区居委会",
                code: "004",
            },
            VillageCode {
                name: "华裕社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "盛达东路街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "瑞园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "瑞华社区居委会",
                code: "002",
            },
            VillageCode {
                name: "瑞泰社区居委会",
                code: "003",
            },
            VillageCode {
                name: "瑞兴社区居委会",
                code: "004",
            },
            VillageCode {
                name: "瑞康社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "古牧地镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "佳乐社区居委会",
                code: "001",
            },
            VillageCode {
                name: "泰和社区居委会",
                code: "002",
            },
            VillageCode {
                name: "乾惠社区居委会",
                code: "003",
            },
            VillageCode {
                name: "乾和社区居委会",
                code: "004",
            },
            VillageCode {
                name: "上沙河村委会",
                code: "005",
            },
            VillageCode {
                name: "大破城村委会",
                code: "006",
            },
            VillageCode {
                name: "小破城村委会",
                code: "007",
            },
            VillageCode {
                name: "锅底坑村委会",
                code: "008",
            },
            VillageCode {
                name: "下沙河村委会",
                code: "009",
            },
            VillageCode {
                name: "西二渠村委会",
                code: "010",
            },
            VillageCode {
                name: "太平渠村委会",
                code: "011",
            },
            VillageCode {
                name: "西工村委会",
                code: "012",
            },
            VillageCode {
                name: "园艺村委会",
                code: "013",
            },
            VillageCode {
                name: "皇渠沿村委会",
                code: "014",
            },
            VillageCode {
                name: "团结村委会",
                code: "015",
            },
            VillageCode {
                name: "东工村委会",
                code: "016",
            },
            VillageCode {
                name: "振兴村委会",
                code: "017",
            },
            VillageCode {
                name: "下大草滩村委会",
                code: "018",
            },
            VillageCode {
                name: "菜园子村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "铁厂沟镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "天山村委会",
                code: "001",
            },
            VillageCode {
                name: "八家户村委会",
                code: "002",
            },
            VillageCode {
                name: "铁厂沟东村村委会",
                code: "003",
            },
            VillageCode {
                name: "铁厂沟西村村委会",
                code: "004",
            },
            VillageCode {
                name: "曙光上村村委会",
                code: "005",
            },
            VillageCode {
                name: "曙光下村村委会",
                code: "006",
            },
            VillageCode {
                name: "石化新村村委会",
                code: "007",
            },
            VillageCode {
                name: "大草滩村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "长山子镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "马场湖村委会",
                code: "001",
            },
            VillageCode {
                name: "硷梁村委会",
                code: "002",
            },
            VillageCode {
                name: "梁东村委会",
                code: "003",
            },
            VillageCode {
                name: "吉三泉村委会",
                code: "004",
            },
            VillageCode {
                name: "吴家梁村委会",
                code: "005",
            },
            VillageCode {
                name: "万家梁村委会",
                code: "006",
            },
            VillageCode {
                name: "上梁头村委会",
                code: "007",
            },
            VillageCode {
                name: "解放村村委会",
                code: "008",
            },
            VillageCode {
                name: "黑水村委会",
                code: "009",
            },
            VillageCode {
                name: "湖南村村委会",
                code: "010",
            },
            VillageCode {
                name: "六户地村委会",
                code: "011",
            },
            VillageCode {
                name: "土梁村委会",
                code: "012",
            },
            VillageCode {
                name: "高家湖村委会",
                code: "013",
            },
            VillageCode {
                name: "大庄子村委会",
                code: "014",
            },
            VillageCode {
                name: "三个庄村委会",
                code: "015",
            },
            VillageCode {
                name: "土窑子村委会",
                code: "016",
            },
            VillageCode {
                name: "新庄子村委会",
                code: "017",
            },
            VillageCode {
                name: "二湾村委会",
                code: "018",
            },
            VillageCode {
                name: "下梁头村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "羊毛工镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "牛庄子村委会",
                code: "001",
            },
            VillageCode {
                name: "红雁湖村委会",
                code: "002",
            },
            VillageCode {
                name: "协标工村委会",
                code: "003",
            },
            VillageCode {
                name: "羊毛工村委会",
                code: "004",
            },
            VillageCode {
                name: "陕西工村委会",
                code: "005",
            },
            VillageCode {
                name: "雷家塘村委会",
                code: "006",
            },
            VillageCode {
                name: "东方村委会",
                code: "007",
            },
            VillageCode {
                name: "卧龙岗村委会",
                code: "008",
            },
            VillageCode {
                name: "西庄子村委会",
                code: "009",
            },
            VillageCode {
                name: "留子庙村委会",
                code: "010",
            },
            VillageCode {
                name: "新建村委会",
                code: "011",
            },
            VillageCode {
                name: "柳树庄村委会",
                code: "012",
            },
            VillageCode {
                name: "蒋家湾村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "三道坝镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "碱泉子社区居委会",
                code: "001",
            },
            VillageCode {
                name: "塔桥湾村委会",
                code: "002",
            },
            VillageCode {
                name: "天生沟村委会",
                code: "003",
            },
            VillageCode {
                name: "西村村委会",
                code: "004",
            },
            VillageCode {
                name: "东村村委会",
                code: "005",
            },
            VillageCode {
                name: "头道坝村委会",
                code: "006",
            },
            VillageCode {
                name: "新庄子村委会",
                code: "007",
            },
            VillageCode {
                name: "东滩村委会",
                code: "008",
            },
            VillageCode {
                name: "大庄子村委会",
                code: "009",
            },
            VillageCode {
                name: "二道坝村委会",
                code: "010",
            },
            VillageCode {
                name: "上三道坝村委会",
                code: "011",
            },
            VillageCode {
                name: "河南村委会",
                code: "012",
            },
            VillageCode {
                name: "三道坝村委会",
                code: "013",
            },
            VillageCode {
                name: "西阴沟村委会",
                code: "014",
            },
            VillageCode {
                name: "四道坝村委会",
                code: "015",
            },
            VillageCode {
                name: "杜家庄村委会",
                code: "016",
            },
            VillageCode {
                name: "韩家庄村委会",
                code: "017",
            },
            VillageCode {
                name: "皇工村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "柏杨河乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "和瑞社区居委会",
                code: "001",
            },
            VillageCode {
                name: "两园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "梧桐窝子村委会",
                code: "003",
            },
            VillageCode {
                name: "红柳村委会",
                code: "004",
            },
            VillageCode {
                name: "柏杨河村委会",
                code: "005",
            },
            VillageCode {
                name: "独山子村委会",
                code: "006",
            },
            VillageCode {
                name: "玉西布早村委会",
                code: "007",
            },
            VillageCode {
                name: "阿合阿德尔村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "芦草沟乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "集镇区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金戈壁社区居委会",
                code: "002",
            },
            VillageCode {
                name: "东苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "芦草沟村委会",
                code: "004",
            },
            VillageCode {
                name: "人民庄子村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "兵团梧桐镇分部",
        code: "016",
        villages: &[
            VillageCode {
                name: "八连生活区",
                code: "001",
            },
            VillageCode {
                name: "九连生活区",
                code: "002",
            },
        ],
    },
];

static TOWNS_XJ_009: [TownCode; 15] = [
    TownCode {
        name: "老城路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "新城社区",
                code: "001",
            },
            VillageCode {
                name: "新春社区",
                code: "002",
            },
            VillageCode {
                name: "椿树路社区",
                code: "003",
            },
            VillageCode {
                name: "老城路社区",
                code: "004",
            },
            VillageCode {
                name: "苏公塔社区",
                code: "005",
            },
            VillageCode {
                name: "广场社区",
                code: "006",
            },
            VillageCode {
                name: "滨湖社区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "高昌路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "幸福社区",
                code: "001",
            },
            VillageCode {
                name: "绿洲社区",
                code: "002",
            },
            VillageCode {
                name: "广汇社区",
                code: "003",
            },
            VillageCode {
                name: "文化路社区",
                code: "004",
            },
            VillageCode {
                name: "共建路社区",
                code: "005",
            },
            VillageCode {
                name: "新站社区",
                code: "006",
            },
            VillageCode {
                name: "友谊巷社区",
                code: "007",
            },
            VillageCode {
                name: "西环路社区",
                code: "008",
            },
            VillageCode {
                name: "绿园社区",
                code: "009",
            },
            VillageCode {
                name: "光明路社区",
                code: "010",
            },
            VillageCode {
                name: "东环路社区",
                code: "011",
            },
            VillageCode {
                name: "丝绸社区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "葡萄沟街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "葡萄社区",
                code: "001",
            },
            VillageCode {
                name: "布依鲁克社区",
                code: "002",
            },
            VillageCode {
                name: "拜什买里社区",
                code: "003",
            },
            VillageCode {
                name: "达甫散盖社区",
                code: "004",
            },
            VillageCode {
                name: "宜居园社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "红柳河街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "桃园社区",
                code: "001",
            },
            VillageCode {
                name: "红柳社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "七泉湖镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "车站社区居委会",
                code: "001",
            },
            VillageCode {
                name: "红山社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新域社区居委会",
                code: "003",
            },
            VillageCode {
                name: "七泉湖村民委员会",
                code: "004",
            },
            VillageCode {
                name: "煤窑沟村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "大河沿镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "新区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "铁路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "复兴社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "亚尔镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "老城东门社区居委会",
                code: "001",
            },
            VillageCode {
                name: "戈壁社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新城东门社区居委会",
                code: "003",
            },
            VillageCode {
                name: "克孜勒吐尔社区居委会",
                code: "004",
            },
            VillageCode {
                name: "南门社区居委会",
                code: "005",
            },
            VillageCode {
                name: "红星社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新城西门村民委员会",
                code: "007",
            },
            VillageCode {
                name: "英买里村民委员会",
                code: "008",
            },
            VillageCode {
                name: "亚尔果勒村民委员会",
                code: "009",
            },
            VillageCode {
                name: "亚尔村民委员会",
                code: "010",
            },
            VillageCode {
                name: "上湖村民委员会",
                code: "011",
            },
            VillageCode {
                name: "恰章村民委员会",
                code: "012",
            },
            VillageCode {
                name: "塔格托维村民委员会",
                code: "013",
            },
            VillageCode {
                name: "色依迪汗村民委员会",
                code: "014",
            },
            VillageCode {
                name: "吕宗村民委员会",
                code: "015",
            },
            VillageCode {
                name: "加依村民委员会",
                code: "016",
            },
            VillageCode {
                name: "亚尔贝希村民委员会",
                code: "017",
            },
            VillageCode {
                name: "夏勒克村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "艾丁湖镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "也木什村民委员会",
                code: "001",
            },
            VillageCode {
                name: "西然木村民委员会",
                code: "002",
            },
            VillageCode {
                name: "花园村民委员会",
                code: "003",
            },
            VillageCode {
                name: "琼库勒村民委员会",
                code: "004",
            },
            VillageCode {
                name: "干店村民委员会",
                code: "005",
            },
            VillageCode {
                name: "庄子村",
                code: "006",
            },
            VillageCode {
                name: "帕克布拉克村",
                code: "007",
            },
            VillageCode {
                name: "阔西墩村",
                code: "008",
            },
            VillageCode {
                name: "大庄子村",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "葡萄镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "巴格日社区居委会",
                code: "001",
            },
            VillageCode {
                name: "木纳尔社区居委会",
                code: "002",
            },
            VillageCode {
                name: "火焰山社区居委会",
                code: "003",
            },
            VillageCode {
                name: "鸿景园社区",
                code: "004",
            },
            VillageCode {
                name: "布拉克村民委员会",
                code: "005",
            },
            VillageCode {
                name: "古渔村民委员会",
                code: "006",
            },
            VillageCode {
                name: "英萨村民委员会",
                code: "007",
            },
            VillageCode {
                name: "铁提尔村民委员会",
                code: "008",
            },
            VillageCode {
                name: "霍依拉坎儿孜村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "火焰山镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "巴达木村民委员会",
                code: "001",
            },
            VillageCode {
                name: "古城村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西游村民委员会",
                code: "003",
            },
            VillageCode {
                name: "二堡村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "恰特喀勒乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "恰特喀勒村民委员会",
                code: "001",
            },
            VillageCode {
                name: "喀拉霍加坎儿孜村民委员会",
                code: "002",
            },
            VillageCode {
                name: "拜什巴拉坎儿孜村民委员会",
                code: "003",
            },
            VillageCode {
                name: "奥依曼坎儿孜村民委员会",
                code: "004",
            },
            VillageCode {
                name: "吐鲁番克尔村民委员会",
                code: "005",
            },
            VillageCode {
                name: "其盖布拉克村民委员会",
                code: "006",
            },
            VillageCode {
                name: "公相村民委员会",
                code: "007",
            },
            VillageCode {
                name: "阿依库勒村",
                code: "008",
            },
            VillageCode {
                name: "琼坎儿孜村",
                code: "009",
            },
            VillageCode {
                name: "幸福村",
                code: "010",
            },
            VillageCode {
                name: "新光村",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "三堡乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "台藏村民委员会",
                code: "001",
            },
            VillageCode {
                name: "英吐尔村民委员会",
                code: "002",
            },
            VillageCode {
                name: "园艺村民委员会",
                code: "003",
            },
            VillageCode {
                name: "阿瓦提村民委员会",
                code: "004",
            },
            VillageCode {
                name: "曼古布拉克村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "胜金乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "开斯突尔村民委员会",
                code: "001",
            },
            VillageCode {
                name: "加依霍加木村民委员会",
                code: "002",
            },
            VillageCode {
                name: "胜金村民委员会",
                code: "003",
            },
            VillageCode {
                name: "排孜阿瓦提村民委员会",
                code: "004",
            },
            VillageCode {
                name: "色格孜库勒村民委员会",
                code: "005",
            },
            VillageCode {
                name: "阿克塔木村民委员会",
                code: "006",
            },
            VillageCode {
                name: "恰勒坎村民委员会",
                code: "007",
            },
            VillageCode {
                name: "木日吐克村民委员会",
                code: "008",
            },
            VillageCode {
                name: "艾西夏村民委员会",
                code: "009",
            },
            VillageCode {
                name: "乌鲁库勒村",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "原种场",
        code: "014",
        villages: &[VillageCode {
            name: "原种场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "兵团二二一团",
        code: "015",
        villages: &[
            VillageCode {
                name: "一连生产队",
                code: "001",
            },
            VillageCode {
                name: "三连生产队",
                code: "002",
            },
            VillageCode {
                name: "四连生产队",
                code: "003",
            },
            VillageCode {
                name: "交河西社区村委会",
                code: "004",
            },
            VillageCode {
                name: "二连生产队",
                code: "005",
            },
        ],
    },
];

static TOWNS_XJ_010: [TownCode; 12] = [
    TownCode {
        name: "鄯善镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "木卡姆社区",
                code: "001",
            },
            VillageCode {
                name: "鸿雁社区",
                code: "002",
            },
            VillageCode {
                name: "庭子路社区",
                code: "003",
            },
            VillageCode {
                name: "滨沙社区",
                code: "004",
            },
            VillageCode {
                name: "育才路社区",
                code: "005",
            },
            VillageCode {
                name: "沙园路社区",
                code: "006",
            },
            VillageCode {
                name: "蒲昌路社区",
                code: "007",
            },
            VillageCode {
                name: "双水磨社区",
                code: "008",
            },
            VillageCode {
                name: "蝴蝶泉社区",
                code: "009",
            },
            VillageCode {
                name: "新楼兰社区",
                code: "010",
            },
            VillageCode {
                name: "石材园社区",
                code: "011",
            },
            VillageCode {
                name: "铁提尔社区",
                code: "012",
            },
            VillageCode {
                name: "百丽社区",
                code: "013",
            },
            VillageCode {
                name: "苗园路社区",
                code: "014",
            },
            VillageCode {
                name: "太阳岛社区",
                code: "015",
            },
            VillageCode {
                name: "高铁北站社区",
                code: "016",
            },
            VillageCode {
                name: "凌云社区",
                code: "017",
            },
            VillageCode {
                name: "巴扎村",
                code: "018",
            },
            VillageCode {
                name: "牧业队",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "七克台镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "彩云社区",
                code: "001",
            },
            VillageCode {
                name: "彩玉社区",
                code: "002",
            },
            VillageCode {
                name: "巴喀村",
                code: "003",
            },
            VillageCode {
                name: "台孜村",
                code: "004",
            },
            VillageCode {
                name: "热阿运村",
                code: "005",
            },
            VillageCode {
                name: "库木坎儿孜村",
                code: "006",
            },
            VillageCode {
                name: "七克台村",
                code: "007",
            },
            VillageCode {
                name: "南湖村",
                code: "008",
            },
            VillageCode {
                name: "亚喀坎儿孜村",
                code: "009",
            },
            VillageCode {
                name: "黄家坎村",
                code: "010",
            },
            VillageCode {
                name: "牧业队",
                code: "011",
            },
            VillageCode {
                name: "底湖矿区社区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "火车站镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "金桥社区",
                code: "001",
            },
            VillageCode {
                name: "振兴社区",
                code: "002",
            },
            VillageCode {
                name: "兴业社区",
                code: "003",
            },
            VillageCode {
                name: "友好社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "连木沁镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "连心社区",
                code: "001",
            },
            VillageCode {
                name: "同心社区",
                code: "002",
            },
            VillageCode {
                name: "阿克墩村",
                code: "003",
            },
            VillageCode {
                name: "苏克协尔村",
                code: "004",
            },
            VillageCode {
                name: "艾斯力汗都村",
                code: "005",
            },
            VillageCode {
                name: "汗都坎村",
                code: "006",
            },
            VillageCode {
                name: "布拉克阿勒迪村",
                code: "007",
            },
            VillageCode {
                name: "汗都夏村",
                code: "008",
            },
            VillageCode {
                name: "尤库日买里村",
                code: "009",
            },
            VillageCode {
                name: "曲旺克尔村",
                code: "010",
            },
            VillageCode {
                name: "库木买里村",
                code: "011",
            },
            VillageCode {
                name: "连木沁巴扎村",
                code: "012",
            },
            VillageCode {
                name: "连木沁坎村",
                code: "013",
            },
            VillageCode {
                name: "连木沁阿斯坦村",
                code: "014",
            },
            VillageCode {
                name: "牧业队",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "鲁克沁镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "柳城社区",
                code: "001",
            },
            VillageCode {
                name: "阿凡提社区",
                code: "002",
            },
            VillageCode {
                name: "阔纳夏村",
                code: "003",
            },
            VillageCode {
                name: "英夏买里村",
                code: "004",
            },
            VillageCode {
                name: "迪汗苏村",
                code: "005",
            },
            VillageCode {
                name: "吐格曼博依村",
                code: "006",
            },
            VillageCode {
                name: "赛尔克甫村",
                code: "007",
            },
            VillageCode {
                name: "阿曼夏村",
                code: "008",
            },
            VillageCode {
                name: "三个桥村",
                code: "009",
            },
            VillageCode {
                name: "牧业队",
                code: "010",
            },
            VillageCode {
                name: "木卡姆村",
                code: "011",
            },
            VillageCode {
                name: "沙坎村",
                code: "012",
            },
            VillageCode {
                name: "其那尔巴格村",
                code: "013",
            },
            VillageCode {
                name: "赛尔克甫夏村",
                code: "014",
            },
            VillageCode {
                name: "乃再尔巴格村",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "辟展镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "田园社区",
                code: "001",
            },
            VillageCode {
                name: "东湖村",
                code: "002",
            },
            VillageCode {
                name: "小东湖村",
                code: "003",
            },
            VillageCode {
                name: "马场村",
                code: "004",
            },
            VillageCode {
                name: "库尔干村",
                code: "005",
            },
            VillageCode {
                name: "乔克塔木村",
                code: "006",
            },
            VillageCode {
                name: "柯柯亚村",
                code: "007",
            },
            VillageCode {
                name: "英也尔村",
                code: "008",
            },
            VillageCode {
                name: "克其克村",
                code: "009",
            },
            VillageCode {
                name: "树柏沟村",
                code: "010",
            },
            VillageCode {
                name: "兰干村",
                code: "011",
            },
            VillageCode {
                name: "卡格托尔村",
                code: "012",
            },
            VillageCode {
                name: "牧业队",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "迪坎镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "迪坎尔村",
                code: "001",
            },
            VillageCode {
                name: "也扎坎儿孜村",
                code: "002",
            },
            VillageCode {
                name: "玉尔门村",
                code: "003",
            },
            VillageCode {
                name: "托特坎儿孜村",
                code: "004",
            },
            VillageCode {
                name: "坎儿孜库勒村",
                code: "005",
            },
            VillageCode {
                name: "塔什塔盘村",
                code: "006",
            },
            VillageCode {
                name: "牧业队",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "东巴扎回族乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "前街村",
                code: "001",
            },
            VillageCode {
                name: "艾孜拉村",
                code: "002",
            },
            VillageCode {
                name: "后梁村",
                code: "003",
            },
            VillageCode {
                name: "塔乌村",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "吐峪沟乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "洋海湾社区",
                code: "001",
            },
            VillageCode {
                name: "苏贝希夏村",
                code: "002",
            },
            VillageCode {
                name: "吐峪沟村",
                code: "003",
            },
            VillageCode {
                name: "吐峪沟克尔火焰山村",
                code: "004",
            },
            VillageCode {
                name: "团结村",
                code: "005",
            },
            VillageCode {
                name: "潘家坎儿孜村",
                code: "006",
            },
            VillageCode {
                name: "泽日甫坎儿孜村",
                code: "007",
            },
            VillageCode {
                name: "洋海夏村",
                code: "008",
            },
            VillageCode {
                name: "洋海村",
                code: "009",
            },
            VillageCode {
                name: "碱滩坎村",
                code: "010",
            },
            VillageCode {
                name: "幸福村",
                code: "011",
            },
            VillageCode {
                name: "杏花村",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "达朗坎乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "央布拉克村",
                code: "001",
            },
            VillageCode {
                name: "拜什塔木村",
                code: "002",
            },
            VillageCode {
                name: "乔亚村",
                code: "003",
            },
            VillageCode {
                name: "阿扎提村",
                code: "004",
            },
            VillageCode {
                name: "玉旺克尔村",
                code: "005",
            },
            VillageCode {
                name: "牧业队",
                code: "006",
            },
            VillageCode {
                name: "英坎儿孜村",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "南山矿区",
        code: "011",
        villages: &[VillageCode {
            name: "南山矿区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "园艺场",
        code: "012",
        villages: &[
            VillageCode {
                name: "园艺场一队",
                code: "001",
            },
            VillageCode {
                name: "园艺场三队",
                code: "002",
            },
            VillageCode {
                name: "园艺场四队",
                code: "003",
            },
        ],
    },
];

static TOWNS_XJ_011: [TownCode; 8] = [
    TownCode {
        name: "托克逊镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "天泉社区",
                code: "001",
            },
            VillageCode {
                name: "山泉社区",
                code: "002",
            },
            VillageCode {
                name: "龙泉社区",
                code: "003",
            },
            VillageCode {
                name: "玉泉社区",
                code: "004",
            },
            VillageCode {
                name: "银泉社区",
                code: "005",
            },
            VillageCode {
                name: "金泉社区",
                code: "006",
            },
            VillageCode {
                name: "阳光社区",
                code: "007",
            },
            VillageCode {
                name: "滨河社区",
                code: "008",
            },
            VillageCode {
                name: "九龙社区",
                code: "009",
            },
            VillageCode {
                name: "友好社区",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "库米什镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "库米什社区",
                code: "001",
            },
            VillageCode {
                name: "柯尔克孜铁米村委会",
                code: "002",
            },
            VillageCode {
                name: "英博斯坦村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "克尔碱镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "红河谷社区",
                code: "001",
            },
            VillageCode {
                name: "克尔碱村委会",
                code: "002",
            },
            VillageCode {
                name: "英阿瓦提村委会",
                code: "003",
            },
            VillageCode {
                name: "通沟村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "阿乐惠镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "鱼儿沟社区",
                code: "001",
            },
            VillageCode {
                name: "南泉社区",
                code: "002",
            },
            VillageCode {
                name: "阿拉沟社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "伊拉湖镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "幸福社区",
                code: "001",
            },
            VillageCode {
                name: "康喀村委会",
                code: "002",
            },
            VillageCode {
                name: "伊拉湖村委会",
                code: "003",
            },
            VillageCode {
                name: "古勒巴格村委会",
                code: "004",
            },
            VillageCode {
                name: "郭若村委会",
                code: "005",
            },
            VillageCode {
                name: "依提帕克村委会",
                code: "006",
            },
            VillageCode {
                name: "安西村委会",
                code: "007",
            },
            VillageCode {
                name: "阿克塔格村委会",
                code: "008",
            },
            VillageCode {
                name: "布尔加依村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "夏镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "巴扎尔社区",
                code: "001",
            },
            VillageCode {
                name: "布拉克贝希村委会",
                code: "002",
            },
            VillageCode {
                name: "喀拉苏村民委员会",
                code: "003",
            },
            VillageCode {
                name: "奥依曼买里村委会",
                code: "004",
            },
            VillageCode {
                name: "色日克吉勒尕村委会",
                code: "005",
            },
            VillageCode {
                name: "托台村委会",
                code: "006",
            },
            VillageCode {
                name: "铁提尔村委会",
                code: "007",
            },
            VillageCode {
                name: "工尚村委会",
                code: "008",
            },
            VillageCode {
                name: "大地村委会",
                code: "009",
            },
            VillageCode {
                name: "南湖村委会",
                code: "010",
            },
            VillageCode {
                name: "喀格恰克村委会",
                code: "011",
            },
            VillageCode {
                name: "色日克墩村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "博斯坦镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "博孜尤勒贡村委会",
                code: "001",
            },
            VillageCode {
                name: "硝尔坎儿孜村委会",
                code: "002",
            },
            VillageCode {
                name: "伯日布拉克村委会",
                code: "003",
            },
            VillageCode {
                name: "长安村委会",
                code: "004",
            },
            VillageCode {
                name: "博斯坦村委会",
                code: "005",
            },
            VillageCode {
                name: "上湖坎儿孜村委会",
                code: "006",
            },
            VillageCode {
                name: "吉格代村委会",
                code: "007",
            },
            VillageCode {
                name: "琼帕依扎村委会",
                code: "008",
            },
            VillageCode {
                name: "李孟坎儿孜村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "郭勒布依乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "湘泉社区",
                code: "001",
            },
            VillageCode {
                name: "石榴籽社区",
                code: "002",
            },
            VillageCode {
                name: "喀拉布拉克村委会",
                code: "003",
            },
            VillageCode {
                name: "奥依曼布拉克村委会",
                code: "004",
            },
            VillageCode {
                name: "河东村委会",
                code: "005",
            },
            VillageCode {
                name: "开斯克尔村委会",
                code: "006",
            },
            VillageCode {
                name: "切克曼坎儿孜村委会",
                code: "007",
            },
            VillageCode {
                name: "郭勒布依村委会",
                code: "008",
            },
            VillageCode {
                name: "尤库日克喀拉阿什村委会",
                code: "009",
            },
            VillageCode {
                name: "萨依吐格曼村委会",
                code: "010",
            },
            VillageCode {
                name: "硝尔村委会",
                code: "011",
            },
            VillageCode {
                name: "巴格万村委会",
                code: "012",
            },
            VillageCode {
                name: "喀拉阿什村委会",
                code: "013",
            },
        ],
    },
];

static TOWNS_XJ_012: [TownCode; 32] = [
    TownCode {
        name: "东河街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "建国北路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "复兴路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "广场南路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "阿牙社区居委会",
                code: "004",
            },
            VillageCode {
                name: "青年路东社区居委会",
                code: "005",
            },
            VillageCode {
                name: "青年路西社区居委会",
                code: "006",
            },
            VillageCode {
                name: "青年南路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "青年北路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "向阳路社区居委会",
                code: "009",
            },
            VillageCode {
                name: "融合路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "花园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "迎宾社区居委会",
                code: "012",
            },
            VillageCode {
                name: "可园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "高新北社区",
                code: "014",
            },
            VillageCode {
                name: "上阿牙村委会",
                code: "015",
            },
            VillageCode {
                name: "吾尔达村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "西河街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "中山北路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中山南路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "老城社区居委会",
                code: "003",
            },
            VillageCode {
                name: "天山西路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "滨河路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "文化路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "惠康园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "008",
            },
            VillageCode {
                name: "泉水湾社区居委会",
                code: "009",
            },
            VillageCode {
                name: "惠康南社区居委会",
                code: "010",
            },
            VillageCode {
                name: "幸福社区居委会",
                code: "011",
            },
            VillageCode {
                name: "翰林路社区居委会",
                code: "012",
            },
            VillageCode {
                name: "大营门村委会",
                code: "013",
            },
            VillageCode {
                name: "中阿牙村委会",
                code: "014",
            },
            VillageCode {
                name: "东菜园村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "城北街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "光明路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北郊路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "启明新村社区居委会",
                code: "003",
            },
            VillageCode {
                name: "机辆路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "天山北路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "祥和社区居委会",
                code: "006",
            },
            VillageCode {
                name: "田园路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "阳光社区居委会",
                code: "008",
            },
            VillageCode {
                name: "惠泽园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "光辉社区居委会",
                code: "010",
            },
            VillageCode {
                name: "启辰社区居委会",
                code: "011",
            },
            VillageCode {
                name: "黑峰山社区",
                code: "012",
            },
            VillageCode {
                name: "跃进村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "丽园街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "前进西路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "七一路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "丽园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "建设路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "八一路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "友谊社区居委会",
                code: "006",
            },
            VillageCode {
                name: "科苑城社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新丰社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新民社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新华社区居委会",
                code: "010",
            },
            VillageCode {
                name: "新西村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "石油新城街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "支油区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南环路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北环路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "农场社区",
                code: "004",
            },
            VillageCode {
                name: "西环路社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "雅满苏镇",
        code: "006",
        villages: &[VillageCode {
            name: "中心社区居委会",
            code: "001",
        }],
    },
    TownCode {
        name: "七角井镇",
        code: "007",
        villages: &[VillageCode {
            name: "七角井村委会",
            code: "001",
        }],
    },
    TownCode {
        name: "星星峡镇",
        code: "008",
        villages: &[VillageCode {
            name: "星星峡虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "二堡镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "二堡村委会",
                code: "001",
            },
            VillageCode {
                name: "乌拉泉村委会",
                code: "002",
            },
            VillageCode {
                name: "奥尔达坎尔孜村委会",
                code: "003",
            },
            VillageCode {
                name: "宫尚村委会",
                code: "004",
            },
            VillageCode {
                name: "头堡村委会",
                code: "005",
            },
            VillageCode {
                name: "火石泉村委会",
                code: "006",
            },
            VillageCode {
                name: "老嘎克里克村委会",
                code: "007",
            },
            VillageCode {
                name: "托干卡尔尼村委会",
                code: "008",
            },
            VillageCode {
                name: "园林场村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "陶家宫镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "陶家宫村委会",
                code: "001",
            },
            VillageCode {
                name: "卡尔苏村委会",
                code: "002",
            },
            VillageCode {
                name: "黄宫村委会",
                code: "003",
            },
            VillageCode {
                name: "新户村委会",
                code: "004",
            },
            VillageCode {
                name: "上庄子村委会",
                code: "005",
            },
            VillageCode {
                name: "荞麦庄子村委会",
                code: "006",
            },
            VillageCode {
                name: "马场村委会",
                code: "007",
            },
            VillageCode {
                name: "牙吾龙村委会",
                code: "008",
            },
            VillageCode {
                name: "新庄子村委会",
                code: "009",
            },
            VillageCode {
                name: "泉水地村委会",
                code: "010",
            },
            VillageCode {
                name: "幸福村",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "五堡镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "比地力克村委会",
                code: "001",
            },
            VillageCode {
                name: "其格尔提麻克村委会",
                code: "002",
            },
            VillageCode {
                name: "库且提里克村委会",
                code: "003",
            },
            VillageCode {
                name: "阿克吐尔村委会",
                code: "004",
            },
            VillageCode {
                name: "高得格村委会",
                code: "005",
            },
            VillageCode {
                name: "博斯坦村委会",
                code: "006",
            },
            VillageCode {
                name: "吐格曼博依村委会",
                code: "007",
            },
            VillageCode {
                name: "五十里村委会",
                code: "008",
            },
            VillageCode {
                name: "小泉子村委会",
                code: "009",
            },
            VillageCode {
                name: "支边农场村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "三道岭镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "青年路社区",
                code: "001",
            },
            VillageCode {
                name: "西河路社区",
                code: "002",
            },
            VillageCode {
                name: "南泉社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "沁城乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "城东村委会",
                code: "001",
            },
            VillageCode {
                name: "城西村委会",
                code: "002",
            },
            VillageCode {
                name: "白山村委会",
                code: "003",
            },
            VillageCode {
                name: "西路村委会",
                code: "004",
            },
            VillageCode {
                name: "小堡村委会",
                code: "005",
            },
            VillageCode {
                name: "牛毛泉村委会",
                code: "006",
            },
            VillageCode {
                name: "岌芨台村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "乌拉台哈萨克民族乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "乌拉台村委会",
                code: "001",
            },
            VillageCode {
                name: "头宫村村委会",
                code: "002",
            },
            VillageCode {
                name: "二宫村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "双井子乡",
        code: "015",
        villages: &[VillageCode {
            name: "双井子虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "大泉湾乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "圪塔井村委会",
                code: "001",
            },
            VillageCode {
                name: "三道城村委会",
                code: "002",
            },
            VillageCode {
                name: "二道城村委会",
                code: "003",
            },
            VillageCode {
                name: "大泉湾村委会",
                code: "004",
            },
            VillageCode {
                name: "黄芦岗村委会",
                code: "005",
            },
            VillageCode {
                name: "兰新村委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "回城乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "富民社区",
                code: "001",
            },
            VillageCode {
                name: "牧场社区",
                code: "002",
            },
            VillageCode {
                name: "西戈壁社区",
                code: "003",
            },
            VillageCode {
                name: "建国村委会",
                code: "004",
            },
            VillageCode {
                name: "沙枣井村委会",
                code: "005",
            },
            VillageCode {
                name: "九龙树村委会",
                code: "006",
            },
            VillageCode {
                name: "麦盖西村委会",
                code: "007",
            },
            VillageCode {
                name: "阿勒屯村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "花园乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "高新南社区",
                code: "001",
            },
            VillageCode {
                name: "强固村委会",
                code: "002",
            },
            VillageCode {
                name: "布茹村委会",
                code: "003",
            },
            VillageCode {
                name: "下马勒恰瓦克村委会",
                code: "004",
            },
            VillageCode {
                name: "艾里克村委会",
                code: "005",
            },
            VillageCode {
                name: "杜西图尔村委会",
                code: "006",
            },
            VillageCode {
                name: "卡日塔里村委会",
                code: "007",
            },
            VillageCode {
                name: "卡让拉村委会",
                code: "008",
            },
            VillageCode {
                name: "红旗村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "南湖乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "南湖村委会",
                code: "001",
            },
            VillageCode {
                name: "托布塔村委会",
                code: "002",
            },
            VillageCode {
                name: "红山村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "德外里都如克哈萨克乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "恰恰依村委会",
                code: "001",
            },
            VillageCode {
                name: "赛克拉村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "西山乡",
        code: "021",
        villages: &[
            VillageCode {
                name: "园艺场社区",
                code: "001",
            },
            VillageCode {
                name: "乌茹里克村委会",
                code: "002",
            },
            VillageCode {
                name: "塔拉提村委会",
                code: "003",
            },
            VillageCode {
                name: "卡拉卡依提村委会",
                code: "004",
            },
            VillageCode {
                name: "库尔鲁克村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "天山乡",
        code: "022",
        villages: &[
            VillageCode {
                name: "头道沟村委会",
                code: "001",
            },
            VillageCode {
                name: "二道沟村委会",
                code: "002",
            },
            VillageCode {
                name: "三道沟村委会",
                code: "003",
            },
            VillageCode {
                name: "石城子村委会",
                code: "004",
            },
            VillageCode {
                name: "板房沟村委会",
                code: "005",
            },
            VillageCode {
                name: "白杨沟村委会",
                code: "006",
            },
            VillageCode {
                name: "二崖头村委会",
                code: "007",
            },
            VillageCode {
                name: "榆树沟村委会",
                code: "008",
            },
            VillageCode {
                name: "水亭村委会",
                code: "009",
            },
            VillageCode {
                name: "口子村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "白石头乡",
        code: "023",
        villages: &[
            VillageCode {
                name: "松树塘社区",
                code: "001",
            },
            VillageCode {
                name: "白石头村委会",
                code: "002",
            },
            VillageCode {
                name: "口门子村委会",
                code: "003",
            },
            VillageCode {
                name: "塔水村委会",
                code: "004",
            },
            VillageCode {
                name: "牧场村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "柳树沟乡",
        code: "024",
        villages: &[
            VillageCode {
                name: "柳树沟村委会",
                code: "001",
            },
            VillageCode {
                name: "一棵树村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "现代农业园区管理委员会",
        code: "025",
        villages: &[VillageCode {
            name: "现代农业园区管理委员会虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "哈密工业园区",
        code: "026",
        villages: &[VillageCode {
            name: "工业园区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "东郊开发区管理委员会",
        code: "027",
        villages: &[VillageCode {
            name: "东郊开发区管理委员会虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "兵团红星二场",
        code: "028",
        villages: &[
            VillageCode {
                name: "机关社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "九连生活区",
                code: "009",
            },
            VillageCode {
                name: "十连生活区",
                code: "010",
            },
            VillageCode {
                name: "八连生活区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "兵团红星四场",
        code: "029",
        villages: &[VillageCode {
            name: "八连生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "兵团黄田农场",
        code: "030",
        villages: &[
            VillageCode {
                name: "九连生活区",
                code: "001",
            },
            VillageCode {
                name: "十三连生活区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "兵团火箭农场",
        code: "031",
        villages: &[
            VillageCode {
                name: "机关社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
            VillageCode {
                name: "九连生活区",
                code: "010",
            },
            VillageCode {
                name: "十连生活区",
                code: "011",
            },
            VillageCode {
                name: "园艺场生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十二连生活区",
                code: "014",
            },
            VillageCode {
                name: "十三连生活区",
                code: "015",
            },
            VillageCode {
                name: "十四连生活区",
                code: "016",
            },
            VillageCode {
                name: "十五连生活区",
                code: "017",
            },
            VillageCode {
                name: "十六连生活区",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "兵团柳树泉农场",
        code: "032",
        villages: &[
            VillageCode {
                name: "机关社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "四连生活区",
                code: "004",
            },
            VillageCode {
                name: "五连生活区",
                code: "005",
            },
            VillageCode {
                name: "七连生活区",
                code: "006",
            },
            VillageCode {
                name: "八连生活区",
                code: "007",
            },
            VillageCode {
                name: "三连生活区",
                code: "008",
            },
            VillageCode {
                name: "六连生活区",
                code: "009",
            },
        ],
    },
];

static TOWNS_XJ_013: [TownCode; 1] = [TownCode {
    name: "巴里坤镇",
    code: "001",
    villages: &[
        VillageCode {
            name: "广东路社区居民委员会",
            code: "001",
        },
        VillageCode {
            name: "团结路社区居民委员会",
            code: "002",
        },
        VillageCode {
            name: "新市路社区居民委员会",
            code: "003",
        },
        VillageCode {
            name: "东城新区社区居民委员会",
            code: "004",
        },
    ],
}];

static TOWNS_XJ_014: [TownCode; 10] = [
    TownCode {
        name: "伊吾镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "秀水苑社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "振兴社区居民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "淖毛湖镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "淖毛湖镇中心社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "淖毛湖镇工业园区社区",
                code: "002",
            },
            VillageCode {
                name: "克尔赛村委会",
                code: "003",
            },
            VillageCode {
                name: "淖毛湖开发区一村",
                code: "004",
            },
            VillageCode {
                name: "淖毛湖开发区二村",
                code: "005",
            },
            VillageCode {
                name: "淖毛湖开发区三村",
                code: "006",
            },
            VillageCode {
                name: "希望社区村民委员会",
                code: "007",
            },
            VillageCode {
                name: "和顺园社区村民委员会",
                code: "008",
            },
            VillageCode {
                name: "民光社区村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "盐池镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "阿尔通盖村委会",
                code: "001",
            },
            VillageCode {
                name: "幻彩园社区村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "苇子峡乡",
        code: "004",
        villages: &[VillageCode {
            name: "杏花苑社区村民委员会",
            code: "001",
        }],
    },
    TownCode {
        name: "下马崖乡",
        code: "005",
        villages: &[VillageCode {
            name: "新丝路社区村民委员会",
            code: "001",
        }],
    },
    TownCode {
        name: "吐葫芦乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "托背梁村委会",
                code: "001",
            },
            VillageCode {
                name: "甘沟村委会",
                code: "002",
            },
            VillageCode {
                name: "沙梁子村委会",
                code: "003",
            },
            VillageCode {
                name: "泉脑村村委会",
                code: "004",
            },
            VillageCode {
                name: "夏尔吾依来村村委会",
                code: "005",
            },
            VillageCode {
                name: "伊河苑社区村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "前山哈萨克民族乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "塔拉布拉克村委会",
                code: "001",
            },
            VillageCode {
                name: "石磨沟村委会",
                code: "002",
            },
            VillageCode {
                name: "金牧新村",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "伊吾县工业加工区",
        code: "008",
        villages: &[VillageCode {
            name: "工业加工区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "伊吾县山南开发区管委会",
        code: "009",
        villages: &[VillageCode {
            name: "管委会虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "兵团淖毛湖农场",
        code: "010",
        villages: &[
            VillageCode {
                name: "场部社区",
                code: "001",
            },
            VillageCode {
                name: "二连生活区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
        ],
    },
];

static TOWNS_XJ_015: [TownCode; 22] = [
    TownCode {
        name: "宁边路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "西街社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "宁合社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "水电巷社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "北庭新村社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "城关社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "北门村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "延安北路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "柳树巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "园丁社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "广场社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "团结院社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "金陵社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "友联巷社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "康宁社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "天池社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "天山花园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "民乐社区居民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "北京南路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "五彩新城社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "油运基地社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "文化宫社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "丽苑社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "城建社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "地质村社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "毛纺厂社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "亚中社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "金融社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "光明社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "昌建社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "新科园社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "天方社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "南林社区居民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "建国路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "和畅园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "锦绣社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "水林社区居委会",
                code: "003",
            },
            VillageCode {
                name: "晶彩城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "明苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "电力社区居委会",
                code: "006",
            },
            VillageCode {
                name: "星光社区居委会",
                code: "007",
            },
            VillageCode {
                name: "丽景社区居委会",
                code: "008",
            },
            VillageCode {
                name: "尚都社区居委会",
                code: "009",
            },
            VillageCode {
                name: "融锦社区居委会",
                code: "010",
            },
            VillageCode {
                name: "特变社区居委会",
                code: "011",
            },
            VillageCode {
                name: "嘉顺社区居委会",
                code: "012",
            },
            VillageCode {
                name: "南五工一村村委会",
                code: "013",
            },
            VillageCode {
                name: "南五工二村村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "中山路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "田园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "天山社区居委会",
                code: "002",
            },
            VillageCode {
                name: "滨河社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "警苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "牡丹社区居委会",
                code: "006",
            },
            VillageCode {
                name: "翠岸社区居委会",
                code: "007",
            },
            VillageCode {
                name: "丁香社区居委会",
                code: "008",
            },
            VillageCode {
                name: "五星社区居委会",
                code: "009",
            },
            VillageCode {
                name: "御景社区居委会",
                code: "010",
            },
            VillageCode {
                name: "永胜社区居委会",
                code: "011",
            },
            VillageCode {
                name: "夹滩村村委会",
                code: "012",
            },
            VillageCode {
                name: "北沟二村村委会",
                code: "013",
            },
            VillageCode {
                name: "苗圃村村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "绿洲路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "双河社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西域社区居委会",
                code: "002",
            },
            VillageCode {
                name: "绿园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "净化社区居委会",
                code: "004",
            },
            VillageCode {
                name: "庭州社区居委会",
                code: "005",
            },
            VillageCode {
                name: "屯河社区居委会",
                code: "006",
            },
            VillageCode {
                name: "博文社区居委会",
                code: "007",
            },
            VillageCode {
                name: "艺园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "揽翠社区居委会",
                code: "009",
            },
            VillageCode {
                name: "聚合社区居委会",
                code: "010",
            },
            VillageCode {
                name: "农科社区居委会",
                code: "011",
            },
            VillageCode {
                name: "昌化社区居委会",
                code: "012",
            },
            VillageCode {
                name: "园林社区居委会",
                code: "013",
            },
            VillageCode {
                name: "海棠社区居委会",
                code: "014",
            },
            VillageCode {
                name: "香槟社区委员会",
                code: "015",
            },
            VillageCode {
                name: "河畔社区居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "硫磺沟镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "共青团社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "钢花社区",
                code: "002",
            },
            VillageCode {
                name: "楼庄子村村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "三工镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "长丰村村委会",
                code: "001",
            },
            VillageCode {
                name: "庙工村村委会",
                code: "002",
            },
            VillageCode {
                name: "常胜村村委会",
                code: "003",
            },
            VillageCode {
                name: "下营盘村村委会",
                code: "004",
            },
            VillageCode {
                name: "新戽村村委会",
                code: "005",
            },
            VillageCode {
                name: "南头工村村委会",
                code: "006",
            },
            VillageCode {
                name: "二工村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "榆树沟镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "金榆社区居委会",
                code: "001",
            },
            VillageCode {
                name: "前进村村委会",
                code: "002",
            },
            VillageCode {
                name: "曙光村村委会",
                code: "003",
            },
            VillageCode {
                name: "四畦村村委会",
                code: "004",
            },
            VillageCode {
                name: "榆树沟村村委会",
                code: "005",
            },
            VillageCode {
                name: "牧业村村委会",
                code: "006",
            },
            VillageCode {
                name: "勇进村村委会",
                code: "007",
            },
            VillageCode {
                name: "农场生活区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "二六工镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "广东户村村委会",
                code: "001",
            },
            VillageCode {
                name: "十二份村村委会",
                code: "002",
            },
            VillageCode {
                name: "下六工村村委会",
                code: "003",
            },
            VillageCode {
                name: "幸福村村委会",
                code: "004",
            },
            VillageCode {
                name: "光明村村委会",
                code: "005",
            },
            VillageCode {
                name: "红星村村委会",
                code: "006",
            },
            VillageCode {
                name: "农场生活区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "大西渠镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "幸福社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "玉堂村村委会",
                code: "002",
            },
            VillageCode {
                name: "新渠村村委会",
                code: "003",
            },
            VillageCode {
                name: "思源村村委会",
                code: "004",
            },
            VillageCode {
                name: "大西渠村村委会",
                code: "005",
            },
            VillageCode {
                name: "龙河村村委会",
                code: "006",
            },
            VillageCode {
                name: "新戽村村委会",
                code: "007",
            },
            VillageCode {
                name: "幸福村村委会",
                code: "008",
            },
            VillageCode {
                name: "农场生活区",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "六工镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "下六工村村委会",
                code: "001",
            },
            VillageCode {
                name: "西五工村村委会",
                code: "002",
            },
            VillageCode {
                name: "东五工村村委会",
                code: "003",
            },
            VillageCode {
                name: "新庄村村委会",
                code: "004",
            },
            VillageCode {
                name: "四户坝村村委会",
                code: "005",
            },
            VillageCode {
                name: "沙梁子村村委会",
                code: "006",
            },
            VillageCode {
                name: "下三工村村委会",
                code: "007",
            },
            VillageCode {
                name: "十三户村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "滨湖镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "迎丰村村委会",
                code: "001",
            },
            VillageCode {
                name: "滨湖村村委会",
                code: "002",
            },
            VillageCode {
                name: "友丰村村委会",
                code: "003",
            },
            VillageCode {
                name: "下泉子村村委会",
                code: "004",
            },
            VillageCode {
                name: "东沟村村委会",
                code: "005",
            },
            VillageCode {
                name: "五十户村村委会",
                code: "006",
            },
            VillageCode {
                name: "永红村村委会",
                code: "007",
            },
            VillageCode {
                name: "农场生活区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "佃坝镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "土梁村村委会",
                code: "001",
            },
            VillageCode {
                name: "二畦村村委会",
                code: "002",
            },
            VillageCode {
                name: "佃坝村村委会",
                code: "003",
            },
            VillageCode {
                name: "西沟村村委会",
                code: "004",
            },
            VillageCode {
                name: "东沟村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "阿什里哈萨克民族乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "阿维滩村村委会",
                code: "001",
            },
            VillageCode {
                name: "阿什里村村委会",
                code: "002",
            },
            VillageCode {
                name: "二道水村村委会",
                code: "003",
            },
            VillageCode {
                name: "努尔加村村委会",
                code: "004",
            },
            VillageCode {
                name: "胡阿根村村委会",
                code: "005",
            },
            VillageCode {
                name: "金涝坝村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "庙尔沟乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "庙尔沟村村委会",
                code: "001",
            },
            VillageCode {
                name: "阿克旗村村委会",
                code: "002",
            },
            VillageCode {
                name: "和谐二村村委会",
                code: "003",
            },
            VillageCode {
                name: "和谐一村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "新疆昌吉国家农业科技园区管理委员会",
        code: "017",
        villages: &[
            VillageCode {
                name: "农场一生活区",
                code: "001",
            },
            VillageCode {
                name: "农场二生活区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "昌吉市北部荒漠生态保护管理站",
        code: "018",
        villages: &[
            VillageCode {
                name: "农场一生活区",
                code: "001",
            },
            VillageCode {
                name: "农场二生活区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "昌吉国家高新技术产业开发区",
        code: "019",
        villages: &[VillageCode {
            name: "金榆社区居委会虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "兵团蔡家湖镇分部",
        code: "020",
        villages: &[
            VillageCode {
                name: "十三连生活区",
                code: "001",
            },
            VillageCode {
                name: "十四连生活区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "兵团共青团农场",
        code: "021",
        villages: &[
            VillageCode {
                name: "滨河社区",
                code: "001",
            },
            VillageCode {
                name: "青城社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
            VillageCode {
                name: "农业园管理区",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "兵团军户农场",
        code: "022",
        villages: &[
            VillageCode {
                name: "团结社区",
                code: "001",
            },
            VillageCode {
                name: "新民社区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "八连生活区",
                code: "008",
            },
            VillageCode {
                name: "九连生活区",
                code: "009",
            },
            VillageCode {
                name: "十连生活区",
                code: "010",
            },
            VillageCode {
                name: "一连生活区",
                code: "011",
            },
        ],
    },
];

static TOWNS_XJ_016: [TownCode; 13] = [
    TownCode {
        name: "博峰街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "博峰社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "畅岁园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "民主路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "博北路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "有色苑社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "龙祥社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "佳园社区居民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "阜新街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "文化路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "阜兴苑社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "百合村社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "迎宾路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "大桥社区居民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "准东街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "雪莲花路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "阜彩路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "准东矿区社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "南华路社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "甘河子镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "振兴路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "光明路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "天龙社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "城关镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "城北路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "龙王庙社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "鱼儿沟中心村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "四十户村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "黄鸭坑村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "城北村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "张家庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "城南村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "坂干梁村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "河南庄子村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "冰湖村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "山坡中心村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "南湾村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "龙王庙村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "大墩村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "头工南村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "西树窝子村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "丽阳村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "头工中心村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "大西渠村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "石家庄村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "水磨沟口村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "龙王庙西村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "良繁中心村村民委员会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "九运街镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "九龙社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "五工梁中心村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "牧业村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "五运中心村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "八运村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "八运泉村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "丁家湾中心村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "七运村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "十运村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "七运湖村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "雨坡村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "古城中心村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "六运中心村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "黄土梁中心村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "新湖中心村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "九运村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "黄土梁南中心村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "滋泥泉子镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "天山路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "中沟中心村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "街北中心村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "南泉中心村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "东湖中心村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "八户沟中心村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东泉中心村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "树窝子中心村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "二道河子中心村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "何家湾中心村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "上户沟哈萨克族乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "西沟村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "底沟中心村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "黄山中心村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "白杨河中心村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "东湾村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "小泉中心村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "阿克木那拉村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "幸福路村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "水磨沟乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "水磨河社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "水磨沟村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "柳城子西村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "山泉中心村村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "三工河哈萨克族乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "花儿沟村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "拜斯胡木中心村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "大泉中心村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "兵团农六师土墩子农场",
        code: "011",
        villages: &[
            VillageCode {
                name: "鑫龙社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "兵团六运湖农场",
        code: "012",
        villages: &[
            VillageCode {
                name: "柳荫社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "兵团二二二团农场",
        code: "013",
        villages: &[
            VillageCode {
                name: "文幸社区",
                code: "001",
            },
            VillageCode {
                name: "唐坊社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
            VillageCode {
                name: "三连生活区",
                code: "010",
            },
        ],
    },
];

static TOWNS_XJ_017: [TownCode; 12] = [
    TownCode {
        name: "呼图壁镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "熙景社区",
                code: "001",
            },
            VillageCode {
                name: "水晶社区",
                code: "002",
            },
            VillageCode {
                name: "北门社区",
                code: "003",
            },
            VillageCode {
                name: "双元社区",
                code: "004",
            },
            VillageCode {
                name: "双龙社区",
                code: "005",
            },
            VillageCode {
                name: "阿同汗社区",
                code: "006",
            },
            VillageCode {
                name: "西河社区",
                code: "007",
            },
            VillageCode {
                name: "美华社区",
                code: "008",
            },
            VillageCode {
                name: "宝城社区",
                code: "009",
            },
            VillageCode {
                name: "清泉社区",
                code: "010",
            },
            VillageCode {
                name: "幸福社区",
                code: "011",
            },
            VillageCode {
                name: "华安社区",
                code: "012",
            },
            VillageCode {
                name: "花城社区",
                code: "013",
            },
            VillageCode {
                name: "南门社区",
                code: "014",
            },
            VillageCode {
                name: "吉祥社区",
                code: "015",
            },
            VillageCode {
                name: "丽璟社区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "大丰镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "大丰镇社区",
                code: "001",
            },
            VillageCode {
                name: "红山村委会",
                code: "002",
            },
            VillageCode {
                name: "大土古里村委会",
                code: "003",
            },
            VillageCode {
                name: "十八户村委会",
                code: "004",
            },
            VillageCode {
                name: "祁家湖村委会",
                code: "005",
            },
            VillageCode {
                name: "树窝子村委会",
                code: "006",
            },
            VillageCode {
                name: "高桥村委会",
                code: "007",
            },
            VillageCode {
                name: "红柳塘村委会",
                code: "008",
            },
            VillageCode {
                name: "联丰村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "雀尔沟镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "雀尔沟村委会",
                code: "001",
            },
            VillageCode {
                name: "霍斯铁热克村委会",
                code: "002",
            },
            VillageCode {
                name: "克孜勒塔斯村委会",
                code: "003",
            },
            VillageCode {
                name: "西沟村委会",
                code: "004",
            },
            VillageCode {
                name: "独山子村委会",
                code: "005",
            },
            VillageCode {
                name: "南山牧场村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "二十里店镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "二十里店镇社区",
                code: "001",
            },
            VillageCode {
                name: "宁州户村委会",
                code: "002",
            },
            VillageCode {
                name: "小土古里村委会",
                code: "003",
            },
            VillageCode {
                name: "东滩村委会",
                code: "004",
            },
            VillageCode {
                name: "十四户村委会",
                code: "005",
            },
            VillageCode {
                name: "良种场村委会",
                code: "006",
            },
            VillageCode {
                name: "林场村委会",
                code: "007",
            },
            VillageCode {
                name: "四工村委会",
                code: "008",
            },
            VillageCode {
                name: "二十里店村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "园户村镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "北园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "良种场社区居委会",
                code: "002",
            },
            VillageCode {
                name: "上三工村委会",
                code: "003",
            },
            VillageCode {
                name: "上二工村委会",
                code: "004",
            },
            VillageCode {
                name: "和庄村委会",
                code: "005",
            },
            VillageCode {
                name: "园户村村委会",
                code: "006",
            },
            VillageCode {
                name: "三工湖村委会",
                code: "007",
            },
            VillageCode {
                name: "下三工村委会",
                code: "008",
            },
            VillageCode {
                name: "大草滩村委会",
                code: "009",
            },
            VillageCode {
                name: "马场湖村委会",
                code: "010",
            },
            VillageCode {
                name: "广林村委会",
                code: "011",
            },
            VillageCode {
                name: "十三户村委会",
                code: "012",
            },
            VillageCode {
                name: "文昌村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "五工台镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "河西居委会",
                code: "001",
            },
            VillageCode {
                name: "乱山子村委会",
                code: "002",
            },
            VillageCode {
                name: "中渠村委会",
                code: "003",
            },
            VillageCode {
                name: "龙王庙村委会",
                code: "004",
            },
            VillageCode {
                name: "十九户村委会",
                code: "005",
            },
            VillageCode {
                name: "五工台村委会",
                code: "006",
            },
            VillageCode {
                name: "西树窝子村委会",
                code: "007",
            },
            VillageCode {
                name: "十户村委会",
                code: "008",
            },
            VillageCode {
                name: "大泉村委会",
                code: "009",
            },
            VillageCode {
                name: "小泉村委会",
                code: "010",
            },
            VillageCode {
                name: "林场村委会",
                code: "011",
            },
            VillageCode {
                name: "幸福村委会",
                code: "012",
            },
            VillageCode {
                name: "百泉村委会",
                code: "013",
            },
            VillageCode {
                name: "福海新村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "石梯子哈萨克民族乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "白杨河村委会",
                code: "001",
            },
            VillageCode {
                name: "东沟村委会",
                code: "002",
            },
            VillageCode {
                name: "阿苇滩村委会",
                code: "003",
            },
            VillageCode {
                name: "霍斯托别村委会",
                code: "004",
            },
            VillageCode {
                name: "西力克特村委会",
                code: "005",
            },
            VillageCode {
                name: "多斯特克村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "国有林管理中心",
        code: "008",
        villages: &[VillageCode {
            name: "国有林管理中心虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "呼图壁种牛场",
        code: "009",
        villages: &[VillageCode {
            name: "种牛场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "兵团一零五团",
        code: "010",
        villages: &[
            VillageCode {
                name: "枣园香社区",
                code: "001",
            },
            VillageCode {
                name: "新城路社区",
                code: "002",
            },
            VillageCode {
                name: "头道湾社区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
            VillageCode {
                name: "三连生活区",
                code: "010",
            },
            VillageCode {
                name: "五连生活区",
                code: "011",
            },
            VillageCode {
                name: "十一连生活区",
                code: "012",
            },
            VillageCode {
                name: "十二连生活区",
                code: "013",
            },
            VillageCode {
                name: "九连生活区",
                code: "014",
            },
            VillageCode {
                name: "十连生活区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "兵团一零六团",
        code: "011",
        villages: &[
            VillageCode {
                name: "马桥社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "六连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "兵团芳草湖总场",
        code: "012",
        villages: &[
            VillageCode {
                name: "长胜南路社区",
                code: "001",
            },
            VillageCode {
                name: "振兴南路社区",
                code: "002",
            },
            VillageCode {
                name: "希望西街社区",
                code: "003",
            },
            VillageCode {
                name: "芳新东街社区",
                code: "004",
            },
            VillageCode {
                name: "团结北路社区",
                code: "005",
            },
            VillageCode {
                name: "迎宾西街社区",
                code: "006",
            },
            VillageCode {
                name: "呼芳路南社区",
                code: "007",
            },
            VillageCode {
                name: "呼芳路北社区",
                code: "008",
            },
            VillageCode {
                name: "芳新西街社区",
                code: "009",
            },
            VillageCode {
                name: "长征东街社区",
                code: "010",
            },
            VillageCode {
                name: "芳草湖农场一连生活区",
                code: "011",
            },
            VillageCode {
                name: "芳草湖农场二连生活区",
                code: "012",
            },
            VillageCode {
                name: "芳草湖农场三连生活区",
                code: "013",
            },
            VillageCode {
                name: "芳草湖农场四连生活区",
                code: "014",
            },
            VillageCode {
                name: "芳草湖农场五连生活区",
                code: "015",
            },
            VillageCode {
                name: "芳草湖农场六连生活区",
                code: "016",
            },
            VillageCode {
                name: "芳草湖农场七连生活区",
                code: "017",
            },
            VillageCode {
                name: "芳草湖农场八连生活区",
                code: "018",
            },
            VillageCode {
                name: "芳草湖农场九连生活区",
                code: "019",
            },
            VillageCode {
                name: "芳草湖农场十连生活区",
                code: "020",
            },
            VillageCode {
                name: "芳草湖农场十一连生活区",
                code: "021",
            },
            VillageCode {
                name: "芳草湖农场十二连生活区",
                code: "022",
            },
            VillageCode {
                name: "芳草湖农场十三连生活区",
                code: "023",
            },
            VillageCode {
                name: "芳草湖农场十四连生活区",
                code: "024",
            },
            VillageCode {
                name: "芳草湖农场十五连生活区",
                code: "025",
            },
            VillageCode {
                name: "芳草湖农场十六连生活区",
                code: "026",
            },
            VillageCode {
                name: "芳草湖农场十七连生活区",
                code: "027",
            },
            VillageCode {
                name: "芳草湖农场十八连生活区",
                code: "028",
            },
            VillageCode {
                name: "芳草湖农场十九连生活区",
                code: "029",
            },
            VillageCode {
                name: "芳草湖农场二十连生活区",
                code: "030",
            },
            VillageCode {
                name: "芳草湖农场二十一连生活区",
                code: "031",
            },
            VillageCode {
                name: "芳草湖农场二十二连生活区",
                code: "032",
            },
            VillageCode {
                name: "芳草湖农场二十三连生活区",
                code: "033",
            },
            VillageCode {
                name: "芳草湖农场二十四连生活区",
                code: "034",
            },
            VillageCode {
                name: "芳草湖农场二十五连生活区",
                code: "035",
            },
            VillageCode {
                name: "芳草湖农场二十六连生活区",
                code: "036",
            },
            VillageCode {
                name: "芳草湖农场二十七连生活区",
                code: "037",
            },
            VillageCode {
                name: "芳草湖农场二十八连生活区",
                code: "038",
            },
            VillageCode {
                name: "芳草湖农场二十九连生活区",
                code: "039",
            },
            VillageCode {
                name: "芳草湖农场三十连生活区",
                code: "040",
            },
            VillageCode {
                name: "芳草湖农场三十一连生活区",
                code: "041",
            },
            VillageCode {
                name: "芳草湖农场三十二连生活区",
                code: "042",
            },
            VillageCode {
                name: "芳草湖农场三十三连生活区",
                code: "043",
            },
            VillageCode {
                name: "芳草湖农场三十四连生活区",
                code: "044",
            },
            VillageCode {
                name: "芳草湖农场三十五连生活区",
                code: "045",
            },
            VillageCode {
                name: "芳草湖农场三十六连生活区",
                code: "046",
            },
        ],
    },
];

static TOWNS_XJ_018: [TownCode; 19] = [
    TownCode {
        name: "玛纳斯镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "凤城社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "西关社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "玛电家园社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "北城社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "园林社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "东关社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "新街社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "康宁社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "御景苑社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "南城社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "玉缘社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "楼南村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "上二工村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "上三工村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "头工村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "王家庄村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "杨家庄村村民委会",
                code: "017",
            },
            VillageCode {
                name: "三工庙村村民委会",
                code: "018",
            },
            VillageCode {
                name: "二工村村民委会",
                code: "019",
            },
            VillageCode {
                name: "草滩村村民委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "乐土驿镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "郑家庄村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "黑梁村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "赵家庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "文家庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "乐土驿村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "上庄子村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "白杨树庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "东梁村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "柳树庄村村民委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "包家店镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "包家店村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "皇工村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "塔西河村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "柴场村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "油坊村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "冬麦地村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "黑梁湾村村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "凉州户镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "西凉州户村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "庄浪户村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "太阳庙村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "吕家庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "丰益工村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "凉州户新村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "北五岔镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "大庙村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "朱家团庄村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "黑沙窝村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "沙窝道村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "凉州户村村民委员",
                code: "005",
            },
            VillageCode {
                name: "西沟村村民委员",
                code: "006",
            },
            VillageCode {
                name: "油坊庄村村民委员",
                code: "007",
            },
            VillageCode {
                name: "田家井村村民委员",
                code: "008",
            },
            VillageCode {
                name: "魏家场村村民委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "六户地镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "土炮营村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "杨家道村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "六户地村村民委会",
                code: "003",
            },
            VillageCode {
                name: "三岔坪村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "陈家渠村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "闯田地村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "鸭洼坑村村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "兰州湾镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "王家庄村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "二道树窝子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "八家户村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "夹河子村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "头阜梁村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "四阜庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "锦水湾村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "下桥子村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "大湾子村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "兰州湾新村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "广东地乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "繁育场村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "袁家湖村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "袁家庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "新湖坪村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "小海子村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "广丰村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "广东地村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "兵户村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "东兵户村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "清水河子哈萨克民族乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "团庄村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "牙湖村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "乔亚巴斯陶村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "贝母房子村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "坎苏瓦特村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "芦草沟村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "塔西河乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "大草滩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "红沙湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新岸村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东支渠村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "黄台子村村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "旱卡子滩乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "东岸村村民委会",
                code: "001",
            },
            VillageCode {
                name: "头渠村村民委会",
                code: "002",
            },
            VillageCode {
                name: "石灰窑子村村民委会",
                code: "003",
            },
            VillageCode {
                name: "闽玛生态村村民委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "玛电工业区",
        code: "012",
        villages: &[VillageCode {
            name: "玛电虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "自治区林业厅玛纳斯平原林场",
        code: "013",
        villages: &[VillageCode {
            name: "凤凰峪虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "新疆农业科学院玛纳斯县试验站",
        code: "014",
        villages: &[VillageCode {
            name: "试验站虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "兵团农六师新湖农场",
        code: "015",
        villages: &[
            VillageCode {
                name: "青松路社区",
                code: "001",
            },
            VillageCode {
                name: "建设西街社区",
                code: "002",
            },
            VillageCode {
                name: "双庆西路社区",
                code: "003",
            },
            VillageCode {
                name: "胜利西街社区",
                code: "004",
            },
            VillageCode {
                name: "迎宾路社区",
                code: "005",
            },
            VillageCode {
                name: "滨湖社区",
                code: "006",
            },
            VillageCode {
                name: "新湖农场一连生活区",
                code: "007",
            },
            VillageCode {
                name: "新湖农场二连队生活区",
                code: "008",
            },
            VillageCode {
                name: "新湖农场三连生活区",
                code: "009",
            },
            VillageCode {
                name: "新湖农场四连生活区",
                code: "010",
            },
            VillageCode {
                name: "新湖农场五连生活区",
                code: "011",
            },
            VillageCode {
                name: "新湖农场六连生活区",
                code: "012",
            },
            VillageCode {
                name: "新湖农场七连生活区",
                code: "013",
            },
            VillageCode {
                name: "新湖农场八连生活区",
                code: "014",
            },
            VillageCode {
                name: "新湖农场九连生活区",
                code: "015",
            },
            VillageCode {
                name: "新湖农场十连生活区",
                code: "016",
            },
            VillageCode {
                name: "新湖农场十二连生活区",
                code: "017",
            },
            VillageCode {
                name: "新湖农场十三连生活区",
                code: "018",
            },
            VillageCode {
                name: "新湖农场十四连生活区",
                code: "019",
            },
            VillageCode {
                name: "新湖农场三十六连生活区",
                code: "020",
            },
            VillageCode {
                name: "新湖农场十五连生活区",
                code: "021",
            },
            VillageCode {
                name: "新湖农场十六连生活区",
                code: "022",
            },
            VillageCode {
                name: "新湖农场十七连生活区",
                code: "023",
            },
            VillageCode {
                name: "新湖农场十八连生活区",
                code: "024",
            },
            VillageCode {
                name: "新湖农场十九连生活区",
                code: "025",
            },
            VillageCode {
                name: "新湖农场二十一连生活区",
                code: "026",
            },
            VillageCode {
                name: "新湖农场二十二连生活区",
                code: "027",
            },
            VillageCode {
                name: "新湖农场二十三连生活区",
                code: "028",
            },
            VillageCode {
                name: "新湖农场二十四连生活区",
                code: "029",
            },
            VillageCode {
                name: "新湖农场二十五连生活区",
                code: "030",
            },
            VillageCode {
                name: "新湖农场二十六连生活区",
                code: "031",
            },
            VillageCode {
                name: "新湖农场二十七连生活区",
                code: "032",
            },
            VillageCode {
                name: "新湖农场二十八连生活区",
                code: "033",
            },
            VillageCode {
                name: "新湖农场二十九连生活区",
                code: "034",
            },
            VillageCode {
                name: "新湖农场三十连生活区",
                code: "035",
            },
            VillageCode {
                name: "新湖农场三十一连生活区",
                code: "036",
            },
            VillageCode {
                name: "新湖农场三十二连生活区",
                code: "037",
            },
            VillageCode {
                name: "新湖农场三十三连生活区",
                code: "038",
            },
            VillageCode {
                name: "新湖农场三十四连生活区",
                code: "039",
            },
            VillageCode {
                name: "新湖农场三十五连生活区",
                code: "040",
            },
            VillageCode {
                name: "新湖农场十一连生活区",
                code: "041",
            },
            VillageCode {
                name: "新湖农场二十连生活区",
                code: "042",
            },
            VillageCode {
                name: "水管处鸭洼沟生活区",
                code: "043",
            },
        ],
    },
    TownCode {
        name: "兵团一四七团",
        code: "016",
        villages: &[
            VillageCode {
                name: "团部社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "三连生活区",
                code: "003",
            },
            VillageCode {
                name: "四连生活区",
                code: "004",
            },
            VillageCode {
                name: "五连生活区",
                code: "005",
            },
            VillageCode {
                name: "六连生活区",
                code: "006",
            },
            VillageCode {
                name: "七连生活区",
                code: "007",
            },
            VillageCode {
                name: "八连生活区",
                code: "008",
            },
            VillageCode {
                name: "九连生活区",
                code: "009",
            },
            VillageCode {
                name: "十连生活区",
                code: "010",
            },
            VillageCode {
                name: "十一连生活区",
                code: "011",
            },
            VillageCode {
                name: "十三连生活区",
                code: "012",
            },
            VillageCode {
                name: "十四连生活区",
                code: "013",
            },
            VillageCode {
                name: "十五连生活区",
                code: "014",
            },
            VillageCode {
                name: "十六连生活区",
                code: "015",
            },
            VillageCode {
                name: "十七连生活区",
                code: "016",
            },
            VillageCode {
                name: "十八连生活区",
                code: "017",
            },
            VillageCode {
                name: "十九连生活区",
                code: "018",
            },
            VillageCode {
                name: "二十连生活区",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "兵团一四八团",
        code: "017",
        villages: &[
            VillageCode {
                name: "团部社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "四连生活区",
                code: "004",
            },
            VillageCode {
                name: "五连生活区",
                code: "005",
            },
            VillageCode {
                name: "六连生活区",
                code: "006",
            },
            VillageCode {
                name: "七连生活区",
                code: "007",
            },
            VillageCode {
                name: "八连生活区",
                code: "008",
            },
            VillageCode {
                name: "九连生活区",
                code: "009",
            },
            VillageCode {
                name: "十连生活区",
                code: "010",
            },
            VillageCode {
                name: "十一连生活区",
                code: "011",
            },
            VillageCode {
                name: "十二连生活区",
                code: "012",
            },
            VillageCode {
                name: "十三连生活区",
                code: "013",
            },
            VillageCode {
                name: "十四连生活区",
                code: "014",
            },
            VillageCode {
                name: "十五连生活区",
                code: "015",
            },
            VillageCode {
                name: "十六连生活区",
                code: "016",
            },
            VillageCode {
                name: "十七连生活区",
                code: "017",
            },
            VillageCode {
                name: "十八连生活区",
                code: "018",
            },
            VillageCode {
                name: "十九连生活区",
                code: "019",
            },
            VillageCode {
                name: "三厂生活区",
                code: "020",
            },
            VillageCode {
                name: "三连生活区",
                code: "021",
            },
            VillageCode {
                name: "二牛场生活区",
                code: "022",
            },
            VillageCode {
                name: "砖厂生活区",
                code: "023",
            },
            VillageCode {
                name: "二厂生活区",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "兵团一四九团",
        code: "018",
        villages: &[
            VillageCode {
                name: "团部社区",
                code: "001",
            },
            VillageCode {
                name: "二轧花厂生活区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十三连生活区",
                code: "014",
            },
            VillageCode {
                name: "十五连生活区",
                code: "015",
            },
            VillageCode {
                name: "十七连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十六连生活区",
                code: "018",
            },
            VillageCode {
                name: "十二连生活区",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "兵团一五零团",
        code: "019",
        villages: &[
            VillageCode {
                name: "团部社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "三连生活区",
                code: "003",
            },
            VillageCode {
                name: "四连生活区",
                code: "004",
            },
            VillageCode {
                name: "五连生活区",
                code: "005",
            },
            VillageCode {
                name: "六连生活区",
                code: "006",
            },
            VillageCode {
                name: "七连生活区",
                code: "007",
            },
            VillageCode {
                name: "八连生活区",
                code: "008",
            },
            VillageCode {
                name: "九连生活区",
                code: "009",
            },
            VillageCode {
                name: "十一连生活区",
                code: "010",
            },
            VillageCode {
                name: "十二连生活区",
                code: "011",
            },
            VillageCode {
                name: "十三连生活区",
                code: "012",
            },
            VillageCode {
                name: "十四连生活区",
                code: "013",
            },
            VillageCode {
                name: "十五连生活区",
                code: "014",
            },
            VillageCode {
                name: "二十连生活区",
                code: "015",
            },
            VillageCode {
                name: "二十二连生活区",
                code: "016",
            },
            VillageCode {
                name: "二十三连生活区",
                code: "017",
            },
            VillageCode {
                name: "二十五连生活区",
                code: "018",
            },
            VillageCode {
                name: "十六连生活区",
                code: "019",
            },
            VillageCode {
                name: "十七连生活区",
                code: "020",
            },
            VillageCode {
                name: "十八连生活区",
                code: "021",
            },
            VillageCode {
                name: "十九连生活区",
                code: "022",
            },
        ],
    },
];

static TOWNS_XJ_019: [TownCode; 18] = [
    TownCode {
        name: "奇台镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "犁铧尖社区",
                code: "001",
            },
            VillageCode {
                name: "东关社区",
                code: "002",
            },
            VillageCode {
                name: "老满城社区",
                code: "003",
            },
            VillageCode {
                name: "马王庙社区",
                code: "004",
            },
            VillageCode {
                name: "水磨河社区",
                code: "005",
            },
            VillageCode {
                name: "城隍庙社区",
                code: "006",
            },
            VillageCode {
                name: "金山社区",
                code: "007",
            },
            VillageCode {
                name: "三清宫社区",
                code: "008",
            },
            VillageCode {
                name: "天山社区",
                code: "009",
            },
            VillageCode {
                name: "崇文社区",
                code: "010",
            },
            VillageCode {
                name: "景苑社区",
                code: "011",
            },
            VillageCode {
                name: "果果滩社区",
                code: "012",
            },
            VillageCode {
                name: "团结社区",
                code: "013",
            },
            VillageCode {
                name: "康居社区",
                code: "014",
            },
            VillageCode {
                name: "北斗宫社区",
                code: "015",
            },
            VillageCode {
                name: "奇台镇幸福社区",
                code: "016",
            },
            VillageCode {
                name: "奇台镇丽苑社区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "老奇台镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "静宁社区",
                code: "001",
            },
            VillageCode {
                name: "二畦村委会",
                code: "002",
            },
            VillageCode {
                name: "牛王宫村委会",
                code: "003",
            },
            VillageCode {
                name: "榆树沟村委会",
                code: "004",
            },
            VillageCode {
                name: "双大门村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "半截沟镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "中葛根村委会",
                code: "001",
            },
            VillageCode {
                name: "川坝村委会",
                code: "002",
            },
            VillageCode {
                name: "大庄子村委会",
                code: "003",
            },
            VillageCode {
                name: "老葛根村委会",
                code: "004",
            },
            VillageCode {
                name: "江布拉克村委会",
                code: "005",
            },
            VillageCode {
                name: "石河子牧场村委会",
                code: "006",
            },
            VillageCode {
                name: "腰站子村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "吉布库镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "天河村委会",
                code: "001",
            },
            VillageCode {
                name: "上堡子村委会",
                code: "002",
            },
            VillageCode {
                name: "三十户村委会",
                code: "003",
            },
            VillageCode {
                name: "涨坝村委会",
                code: "004",
            },
            VillageCode {
                name: "达板河村委会",
                code: "005",
            },
            VillageCode {
                name: "吉布库牧业村委会",
                code: "006",
            },
            VillageCode {
                name: "达板河牧业村委会",
                code: "007",
            },
            VillageCode {
                name: "二马场村委会",
                code: "008",
            },
            VillageCode {
                name: "西槽子村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "东湾镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "根葛尔村委会",
                code: "001",
            },
            VillageCode {
                name: "墒户村委会",
                code: "002",
            },
            VillageCode {
                name: "中渠村委会",
                code: "003",
            },
            VillageCode {
                name: "大泉村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "西地镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "桥子村委会",
                code: "001",
            },
            VillageCode {
                name: "西地村委会",
                code: "002",
            },
            VillageCode {
                name: "东地村委会",
                code: "003",
            },
            VillageCode {
                name: "旱沟村委会",
                code: "004",
            },
            VillageCode {
                name: "沙山子村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "碧流河镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "西戈壁村委会",
                code: "001",
            },
            VillageCode {
                name: "永丰渠村委会",
                code: "002",
            },
            VillageCode {
                name: "皇宫村委会",
                code: "003",
            },
            VillageCode {
                name: "塘坊门村委会",
                code: "004",
            },
            VillageCode {
                name: "洞子沟村委会",
                code: "005",
            },
            VillageCode {
                name: "东戈壁村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "三个庄子镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "马莲滩村委会",
                code: "001",
            },
            VillageCode {
                name: "三个庄子村委会",
                code: "002",
            },
            VillageCode {
                name: "青年村委会",
                code: "003",
            },
            VillageCode {
                name: "土园仓村委会",
                code: "004",
            },
            VillageCode {
                name: "双涝坝村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "西北湾镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "江布拉克新村",
                code: "001",
            },
            VillageCode {
                name: "小屯村委会",
                code: "002",
            },
            VillageCode {
                name: "二屯村委会",
                code: "003",
            },
            VillageCode {
                name: "头屯村委会",
                code: "004",
            },
            VillageCode {
                name: "西北湾村委会",
                code: "005",
            },
            VillageCode {
                name: "杨柳村委会",
                code: "006",
            },
            VillageCode {
                name: "柳树河子村委会",
                code: "007",
            },
            VillageCode {
                name: "三屯村委会",
                code: "008",
            },
            VillageCode {
                name: "八户地牧业村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "芨芨湖镇",
        code: "010",
        villages: &[VillageCode {
            name: "芨芨湖社区",
            code: "001",
        }],
    },
    TownCode {
        name: "坎尔孜乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "华侨村村委会",
                code: "001",
            },
            VillageCode {
                name: "东坎尔孜村委会",
                code: "002",
            },
            VillageCode {
                name: "西坎尔孜村村委会",
                code: "003",
            },
            VillageCode {
                name: "林场村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "五马场乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "鸿鑫村委会",
                code: "001",
            },
            VillageCode {
                name: "阿哈什乎拉克村委会",
                code: "002",
            },
            VillageCode {
                name: "半截泉村委会",
                code: "003",
            },
            VillageCode {
                name: "铁买克布拉克村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "古城乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "古城村村委会",
                code: "001",
            },
            VillageCode {
                name: "果园村村委会",
                code: "002",
            },
            VillageCode {
                name: "八家户村村委会",
                code: "003",
            },
            VillageCode {
                name: "南湖牧业村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "乔仁乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "乔仁村村委会",
                code: "001",
            },
            VillageCode {
                name: "宽沟村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "七户乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "八户村村委会",
                code: "001",
            },
            VillageCode {
                name: "平顶村村委会",
                code: "002",
            },
            VillageCode {
                name: "东塘村村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "塔塔尔乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "石门泉村村委会",
                code: "001",
            },
            VillageCode {
                name: "大泉湖村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "兵团奇台农场",
        code: "017",
        villages: &[
            VillageCode {
                name: "新奇社区",
                code: "001",
            },
            VillageCode {
                name: "万泉社区",
                code: "002",
            },
            VillageCode {
                name: "三十里大墩社区",
                code: "003",
            },
            VillageCode {
                name: "湖沿镇社区",
                code: "004",
            },
            VillageCode {
                name: "骆驼井社区",
                code: "005",
            },
            VillageCode {
                name: "一零八商贸物流园社区",
                code: "006",
            },
            VillageCode {
                name: "奇台农场十七连生活区",
                code: "007",
            },
            VillageCode {
                name: "奇台农场十八连生活区",
                code: "008",
            },
            VillageCode {
                name: "奇台农场十九连生活区",
                code: "009",
            },
            VillageCode {
                name: "奇台农场二十二连生活区",
                code: "010",
            },
            VillageCode {
                name: "奇台农场二十三连生活区",
                code: "011",
            },
            VillageCode {
                name: "奇台农场二十六连生活区",
                code: "012",
            },
            VillageCode {
                name: "奇台农场二十五连生活区",
                code: "013",
            },
            VillageCode {
                name: "奇台农场二十七连生活区",
                code: "014",
            },
            VillageCode {
                name: "奇台农场二十八连生活区",
                code: "015",
            },
            VillageCode {
                name: "奇台农场二十九连生活区",
                code: "016",
            },
            VillageCode {
                name: "奇台农场三十连生活区",
                code: "017",
            },
            VillageCode {
                name: "奇台农场二十连生活区",
                code: "018",
            },
            VillageCode {
                name: "奇台农场一连生活区",
                code: "019",
            },
            VillageCode {
                name: "奇台农场二十一连生活区",
                code: "020",
            },
            VillageCode {
                name: "奇台农场十六连生活区",
                code: "021",
            },
            VillageCode {
                name: "奇台农场五连生活区",
                code: "022",
            },
            VillageCode {
                name: "奇台农场三连生活区",
                code: "023",
            },
            VillageCode {
                name: "奇台农场四连生活区",
                code: "024",
            },
            VillageCode {
                name: "奇台农场二连生活区",
                code: "025",
            },
            VillageCode {
                name: "奇台农场十四连生活区",
                code: "026",
            },
            VillageCode {
                name: "奇台农场六连生活区",
                code: "027",
            },
            VillageCode {
                name: "奇台农场十二连生活区",
                code: "028",
            },
            VillageCode {
                name: "奇台农场十三连生活区",
                code: "029",
            },
            VillageCode {
                name: "奇台农场十五连生活区",
                code: "030",
            },
            VillageCode {
                name: "奇台农场九连生活区",
                code: "031",
            },
            VillageCode {
                name: "奇台农场七连生活区",
                code: "032",
            },
            VillageCode {
                name: "奇台农场八连生活区",
                code: "033",
            },
            VillageCode {
                name: "奇台农场十连生活区",
                code: "034",
            },
            VillageCode {
                name: "奇台农场十一连生活区",
                code: "035",
            },
            VillageCode {
                name: "奇台农场二十四连生活区",
                code: "036",
            },
        ],
    },
    TownCode {
        name: "兵团农六师北塔山牧场",
        code: "018",
        villages: &[
            VillageCode {
                name: "晋北社区",
                code: "001",
            },
            VillageCode {
                name: "畜牧一队生活区",
                code: "002",
            },
            VillageCode {
                name: "畜牧二队生活区",
                code: "003",
            },
            VillageCode {
                name: "畜牧三队生活区",
                code: "004",
            },
            VillageCode {
                name: "草建连队生活区",
                code: "005",
            },
            VillageCode {
                name: "民族连生活区",
                code: "006",
            },
            VillageCode {
                name: "企业生活区",
                code: "007",
            },
            VillageCode {
                name: "农二连生活区",
                code: "008",
            },
        ],
    },
];

static TOWNS_XJ_020: [TownCode; 11] = [
    TownCode {
        name: "吉木萨尔镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "苇湖巷社区居委会",
                code: "001",
            },
            VillageCode {
                name: "满城路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "文化路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "团结路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "文明路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "中心路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "北庭路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "人民路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "天地园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "北地社区居委会",
                code: "010",
            },
            VillageCode {
                name: "西门村民委员会",
                code: "011",
            },
            VillageCode {
                name: "红畦村民委员会",
                code: "012",
            },
            VillageCode {
                name: "校场湖村民委员会",
                code: "013",
            },
            VillageCode {
                name: "马家槽子村民委员会",
                code: "014",
            },
            VillageCode {
                name: "白泉村民委员会",
                code: "015",
            },
            VillageCode {
                name: "沙河村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "三台镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "三台镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "八家地村民委员会",
                code: "002",
            },
            VillageCode {
                name: "老庄湾村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东地村民委员会",
                code: "004",
            },
            VillageCode {
                name: "喇嘛昭村民委员会",
                code: "005",
            },
            VillageCode {
                name: "羊圈台子村民委员会",
                code: "006",
            },
            VillageCode {
                name: "潘家台子村民委员会",
                code: "007",
            },
            VillageCode {
                name: "黄蒿湾村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "泉子街镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "小西沟村民委员会",
                code: "001",
            },
            VillageCode {
                name: "太平村民委员会",
                code: "002",
            },
            VillageCode {
                name: "公圣村民委员会",
                code: "003",
            },
            VillageCode {
                name: "牧业村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "北庭镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "泉水地村民委员会",
                code: "001",
            },
            VillageCode {
                name: "西上湖村民委员会",
                code: "002",
            },
            VillageCode {
                name: "余家宫村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东二畦村民委员会",
                code: "004",
            },
            VillageCode {
                name: "古城村民委员会",
                code: "005",
            },
            VillageCode {
                name: "三场槽子牧业新村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "二工镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "东台子村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "红山子村民委员会",
                code: "002",
            },
            VillageCode {
                name: "东沟村民委员会",
                code: "003",
            },
            VillageCode {
                name: "大泉湖村民委员会",
                code: "004",
            },
            VillageCode {
                name: "六户地村民委员会",
                code: "005",
            },
            VillageCode {
                name: "八户村民委员会",
                code: "006",
            },
            VillageCode {
                name: "海子沿村民委员会",
                code: "007",
            },
            VillageCode {
                name: "芨芨窝子村民委员会",
                code: "008",
            },
            VillageCode {
                name: "西芦芽湖村民委员会",
                code: "009",
            },
            VillageCode {
                name: "大龙口村民委员会",
                code: "010",
            },
            VillageCode {
                name: "董家湾村民委员会",
                code: "011",
            },
            VillageCode {
                name: "头工街西村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "头工街东村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "十八户村民委员会",
                code: "014",
            },
            VillageCode {
                name: "柳树河子村民委员会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "大有镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "牧业村民委员会",
                code: "001",
            },
            VillageCode {
                name: "大有村民委员会",
                code: "002",
            },
            VillageCode {
                name: "渭户村民委员会",
                code: "003",
            },
            VillageCode {
                name: "韭菜园子村民委员会",
                code: "004",
            },
            VillageCode {
                name: "广泉上村民委员会",
                code: "005",
            },
            VillageCode {
                name: "广泉下村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "五彩湾镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "五彩湾社区居委会",
                code: "001",
            },
            VillageCode {
                name: "火烧山社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金盆湾社区居委会",
                code: "003",
            },
            VillageCode {
                name: "彩南社区居委会",
                code: "004",
            },
            VillageCode {
                name: "彩北社区居委会",
                code: "005",
            },
            VillageCode {
                name: "兵团准东产业园社区",
                code: "006",
            },
            VillageCode {
                name: "乌鲁木齐准东产业园社区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "庆阳湖乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "东庆村民委员会",
                code: "001",
            },
            VillageCode {
                name: "二工梁村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西庆村民委员会",
                code: "003",
            },
            VillageCode {
                name: "大东沟村民委员会",
                code: "004",
            },
            VillageCode {
                name: "双河村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "老台乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "西地村民委员会",
                code: "001",
            },
            VillageCode {
                name: "老湖村民委员会",
                code: "002",
            },
            VillageCode {
                name: "二工河村民委员会",
                code: "003",
            },
            VillageCode {
                name: "老台村民委员会",
                code: "004",
            },
            VillageCode {
                name: "阿克托别村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "新地乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "新地村民委员会",
                code: "001",
            },
            VillageCode {
                name: "新地沟村民委员会",
                code: "002",
            },
            VillageCode {
                name: "小份子村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "兵团农六师红旗农场",
        code: "011",
        villages: &[
            VillageCode {
                name: "育才路社区",
                code: "001",
            },
            VillageCode {
                name: "光明路社区",
                code: "002",
            },
            VillageCode {
                name: "三台南社区",
                code: "003",
            },
            VillageCode {
                name: "十连生活区",
                code: "004",
            },
            VillageCode {
                name: "十二连生活区",
                code: "005",
            },
            VillageCode {
                name: "十一连生活区",
                code: "006",
            },
            VillageCode {
                name: "一连生活区",
                code: "007",
            },
            VillageCode {
                name: "二连生活区",
                code: "008",
            },
            VillageCode {
                name: "三连生活区",
                code: "009",
            },
            VillageCode {
                name: "四连生活区",
                code: "010",
            },
            VillageCode {
                name: "五连生活区",
                code: "011",
            },
            VillageCode {
                name: "六连生活区",
                code: "012",
            },
            VillageCode {
                name: "七连生活区",
                code: "013",
            },
            VillageCode {
                name: "八连生活区",
                code: "014",
            },
            VillageCode {
                name: "九连生活区",
                code: "015",
            },
        ],
    },
];

static TOWNS_XJ_021: [TownCode; 1] = [TownCode {
    name: "木垒镇",
    code: "001",
    villages: &[
        VillageCode {
            name: "园林社区",
            code: "001",
        },
        VillageCode {
            name: "明珠社区",
            code: "002",
        },
        VillageCode {
            name: "迎宾社区",
            code: "003",
        },
        VillageCode {
            name: "西河社区",
            code: "004",
        },
        VillageCode {
            name: "老城社区",
            code: "005",
        },
        VillageCode {
            name: "阿吾勒社区",
            code: "006",
        },
        VillageCode {
            name: "新城社区",
            code: "007",
        },
    ],
}];

static TOWNS_XJ_022: [TownCode; 26] = [
    TownCode {
        name: "团结街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "梨花社区居委会",
                code: "001",
            },
            VillageCode {
                name: "绿园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金梦社区居委会",
                code: "003",
            },
            VillageCode {
                name: "阳光社区居委会",
                code: "004",
            },
            VillageCode {
                name: "龙湖社区居委会",
                code: "005",
            },
            VillageCode {
                name: "永安社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "萨依巴格街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "滨河社区居委会",
                code: "001",
            },
            VillageCode {
                name: "楼兰社区居委会",
                code: "002",
            },
            VillageCode {
                name: "梨香园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "萨依巴格社区居委会",
                code: "004",
            },
            VillageCode {
                name: "孔雀社区居委会",
                code: "005",
            },
            VillageCode {
                name: "瑞祥社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "天山街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "双拥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "蓝天社区居委会",
                code: "002",
            },
            VillageCode {
                name: "电力社区居委会",
                code: "003",
            },
            VillageCode {
                name: "客运社区居委会",
                code: "004",
            },
            VillageCode {
                name: "华凌社区居委会",
                code: "005",
            },
            VillageCode {
                name: "北站社区居委会",
                code: "006",
            },
            VillageCode {
                name: "龙祥社区居委会",
                code: "007",
            },
            VillageCode {
                name: "天山社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "新城街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "新苇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "华星园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "科达社区居委会",
                code: "004",
            },
            VillageCode {
                name: "物探社区居委会",
                code: "005",
            },
            VillageCode {
                name: "南湖社区居委会",
                code: "006",
            },
            VillageCode {
                name: "东湖社区居委会",
                code: "007",
            },
            VillageCode {
                name: "佳德社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "建设街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "龙山社区居委会",
                code: "001",
            },
            VillageCode {
                name: "圣果社区居委会",
                code: "002",
            },
            VillageCode {
                name: "千城社区居委会",
                code: "003",
            },
            VillageCode {
                name: "香梨社区居委会",
                code: "004",
            },
            VillageCode {
                name: "上恰其社区居委会",
                code: "005",
            },
            VillageCode {
                name: "建设社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新华社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "朝阳街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "团结社区居委会",
                code: "001",
            },
            VillageCode {
                name: "华夏名门社区居委会",
                code: "002",
            },
            VillageCode {
                name: "丰源社区居委会",
                code: "003",
            },
            VillageCode {
                name: "阿尔金社区居委会",
                code: "004",
            },
            VillageCode {
                name: "成功社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "梨香街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "凌达社区居委会",
                code: "001",
            },
            VillageCode {
                name: "平安社区居委会",
                code: "002",
            },
            VillageCode {
                name: "德凌社区居委会",
                code: "003",
            },
            VillageCode {
                name: "百合社区居委会",
                code: "004",
            },
            VillageCode {
                name: "美居苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "民生社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新安社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "塔什店镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "落霞湾社区居委会",
                code: "001",
            },
            VillageCode {
                name: "矿山路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "文化社区居委会",
                code: "003",
            },
            VillageCode {
                name: "莲花社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "上户镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "新星社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大二线社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西站社区居委会",
                code: "003",
            },
            VillageCode {
                name: "上户村委会",
                code: "004",
            },
            VillageCode {
                name: "杜尔比村委会",
                code: "005",
            },
            VillageCode {
                name: "大墩子村委会",
                code: "006",
            },
            VillageCode {
                name: "哈拉苏村委会",
                code: "007",
            },
            VillageCode {
                name: "萨依买里村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "西尼尔镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "西尼尔社区居委会",
                code: "001",
            },
            VillageCode {
                name: "红旗社区居委会",
                code: "002",
            },
            VillageCode {
                name: "梨城村委会",
                code: "003",
            },
            VillageCode {
                name: "西尼尔村委会",
                code: "004",
            },
            VillageCode {
                name: "团结村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "铁克其乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "迎宾社区居委会",
                code: "001",
            },
            VillageCode {
                name: "下恰其社区居委会",
                code: "002",
            },
            VillageCode {
                name: "腾飞社区居委会",
                code: "003",
            },
            VillageCode {
                name: "华源社区居委会",
                code: "004",
            },
            VillageCode {
                name: "阿瓦提社区居委会",
                code: "005",
            },
            VillageCode {
                name: "沙南社区居委会",
                code: "006",
            },
            VillageCode {
                name: "铁克其村委会",
                code: "007",
            },
            VillageCode {
                name: "艾兰巴格村委会",
                code: "008",
            },
            VillageCode {
                name: "城康村委员会",
                code: "009",
            },
            VillageCode {
                name: "上恰其村委会",
                code: "010",
            },
            VillageCode {
                name: "中恰其村委会",
                code: "011",
            },
            VillageCode {
                name: "下恰其村委会",
                code: "012",
            },
            VillageCode {
                name: "阿克塔什村委会",
                code: "013",
            },
            VillageCode {
                name: "沙南村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "恰尔巴格乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "恰尔巴格村委会",
                code: "001",
            },
            VillageCode {
                name: "喀赞其村委会",
                code: "002",
            },
            VillageCode {
                name: "萨依巴格村委会",
                code: "003",
            },
            VillageCode {
                name: "喀拉墩村委会",
                code: "004",
            },
            VillageCode {
                name: "博斯坦村委会",
                code: "005",
            },
            VillageCode {
                name: "上阔什巴格村委会",
                code: "006",
            },
            VillageCode {
                name: "下阔什巴格村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "英下乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "幸福社区居委会",
                code: "001",
            },
            VillageCode {
                name: "沙依东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金梨社区居委会",
                code: "003",
            },
            VillageCode {
                name: "英下村委会",
                code: "004",
            },
            VillageCode {
                name: "其兰巴格村委会",
                code: "005",
            },
            VillageCode {
                name: "喀尔巴格村委会",
                code: "006",
            },
            VillageCode {
                name: "阿克东村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "兰干乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "兰干村委会",
                code: "001",
            },
            VillageCode {
                name: "贡拉提村委会",
                code: "002",
            },
            VillageCode {
                name: "结帕尔村委会",
                code: "003",
            },
            VillageCode {
                name: "夏库尔村委会",
                code: "004",
            },
            VillageCode {
                name: "新村",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "和什力克乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "上和什力克村委会",
                code: "001",
            },
            VillageCode {
                name: "下和什力克村委会",
                code: "002",
            },
            VillageCode {
                name: "萨依力克村委会",
                code: "003",
            },
            VillageCode {
                name: "库勒村委会",
                code: "004",
            },
            VillageCode {
                name: "柳林村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "哈拉玉宫乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "中兴社区居委会",
                code: "001",
            },
            VillageCode {
                name: "哈拉玉宫村委会",
                code: "002",
            },
            VillageCode {
                name: "中多尕村委会",
                code: "003",
            },
            VillageCode {
                name: "巴格吉格代村委会",
                code: "004",
            },
            VillageCode {
                name: "下多尕村委会",
                code: "005",
            },
            VillageCode {
                name: "台斯砍村委会",
                code: "006",
            },
            VillageCode {
                name: "阿克吐尔村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "阿瓦提乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "阿瓦提村委会",
                code: "001",
            },
            VillageCode {
                name: "阿克艾日克村委会",
                code: "002",
            },
            VillageCode {
                name: "明昆格尔村委会",
                code: "003",
            },
            VillageCode {
                name: "吾夏克铁热克村委会",
                code: "004",
            },
            VillageCode {
                name: "小兰干村委会",
                code: "005",
            },
            VillageCode {
                name: "喀拉亚尕奇村委会",
                code: "006",
            },
            VillageCode {
                name: "强布勒村委会",
                code: "007",
            },
            VillageCode {
                name: "其盖克其克村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "托布力其乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "湖滨社区居委会",
                code: "001",
            },
            VillageCode {
                name: "托布力其村委会",
                code: "002",
            },
            VillageCode {
                name: "上牙克托格拉克村委会",
                code: "003",
            },
            VillageCode {
                name: "下牙克托格拉克村委会",
                code: "004",
            },
            VillageCode {
                name: "新村",
                code: "005",
            },
            VillageCode {
                name: "艾力坎土曼村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "普惠乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "金胡杨社区居委会",
                code: "001",
            },
            VillageCode {
                name: "润疆社区居委会",
                code: "002",
            },
            VillageCode {
                name: "复兴社区居委会",
                code: "003",
            },
            VillageCode {
                name: "振兴社区居委会",
                code: "004",
            },
            VillageCode {
                name: "普惠村委会",
                code: "005",
            },
            VillageCode {
                name: "雅其克村委会",
                code: "006",
            },
            VillageCode {
                name: "库米石阔坦村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "迎宾街道片区",
        code: "020",
        villages: &[
            VillageCode {
                name: "中恰其社区居委会",
                code: "001",
            },
            VillageCode {
                name: "光明社区居委会",
                code: "002",
            },
            VillageCode {
                name: "蓝湾社区居委会",
                code: "003",
            },
            VillageCode {
                name: "冠农社区居委会",
                code: "004",
            },
            VillageCode {
                name: "雅居社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "丝路街道片区",
        code: "021",
        villages: &[
            VillageCode {
                name: "丝路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "上水城社区居委会",
                code: "002",
            },
            VillageCode {
                name: "朝阳社区居委会",
                code: "003",
            },
            VillageCode {
                name: "索克巴格社区居委会",
                code: "004",
            },
            VillageCode {
                name: "连心社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "希望街道片区",
        code: "022",
        villages: &[
            VillageCode {
                name: "富民社区居委会",
                code: "001",
            },
            VillageCode {
                name: "其兰巴格社区居委会",
                code: "002",
            },
            VillageCode {
                name: "友好社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "永安街道片区",
        code: "023",
        villages: &[
            VillageCode {
                name: "永乐社区居委会",
                code: "001",
            },
            VillageCode {
                name: "育才社区居委会",
                code: "002",
            },
            VillageCode {
                name: "康健社区居委会",
                code: "003",
            },
            VillageCode {
                name: "哈赞其社区居委会",
                code: "004",
            },
            VillageCode {
                name: "恰尔巴格社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "友好街道片区",
        code: "024",
        villages: &[
            VillageCode {
                name: "金冠社区居委会",
                code: "001",
            },
            VillageCode {
                name: "交通路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "博爱社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "楼兰街道片区",
        code: "025",
        villages: &[
            VillageCode {
                name: "辰兴社区居委会",
                code: "001",
            },
            VillageCode {
                name: "巴音社区居委会",
                code: "002",
            },
            VillageCode {
                name: "康乐社区居委会",
                code: "003",
            },
            VillageCode {
                name: "佳鑫社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "康都街道片区",
        code: "026",
        villages: &[
            VillageCode {
                name: "塔里木油田社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东方社区居委会",
                code: "002",
            },
            VillageCode {
                name: "时代社区居委会",
                code: "003",
            },
            VillageCode {
                name: "阿克塔什社区居委会",
                code: "004",
            },
            VillageCode {
                name: "康都社区居委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_XJ_023: [TownCode; 11] = [
    TownCode {
        name: "轮台镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "东苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新城社区居委会",
                code: "002",
            },
            VillageCode {
                name: "青年路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新星路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "迪那路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "西苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新大渠社区居委会",
                code: "007",
            },
            VillageCode {
                name: "红桥社区居委会",
                code: "008",
            },
            VillageCode {
                name: "西域社区居委会",
                code: "009",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "010",
            },
            VillageCode {
                name: "览山社区居委会",
                code: "011",
            },
            VillageCode {
                name: "丘吾克村委会",
                code: "012",
            },
            VillageCode {
                name: "英吾依拉村委会",
                code: "013",
            },
            VillageCode {
                name: "克孜勒村委会",
                code: "014",
            },
            VillageCode {
                name: "巴格布依村委会",
                code: "015",
            },
            VillageCode {
                name: "麦台村委会",
                code: "016",
            },
            VillageCode {
                name: "亚克巴格村委会",
                code: "017",
            },
            VillageCode {
                name: "依更巴格村委会",
                code: "018",
            },
            VillageCode {
                name: "迪哈拉村委会",
                code: "019",
            },
            VillageCode {
                name: "拉帕村委会",
                code: "020",
            },
            VillageCode {
                name: "牧业村委会",
                code: "021",
            },
            VillageCode {
                name: "夏玛勒巴格村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "轮南镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "塔河桥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "牙买提社区居委会",
                code: "002",
            },
            VillageCode {
                name: "轮南小区社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "群巴克镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "拉依苏社区居委会",
                code: "001",
            },
            VillageCode {
                name: "园艺场社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "迪那尔村委会",
                code: "003",
            },
            VillageCode {
                name: "阿拉萨依村委会",
                code: "004",
            },
            VillageCode {
                name: "克西里克阿热勒村委会",
                code: "005",
            },
            VillageCode {
                name: "依格孜吾依村委会",
                code: "006",
            },
            VillageCode {
                name: "诺乔喀村委会",
                code: "007",
            },
            VillageCode {
                name: "恰先拜村委会",
                code: "008",
            },
            VillageCode {
                name: "阿克亚村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "阳霞镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "博斯坦村委会",
                code: "001",
            },
            VillageCode {
                name: "库都克村委会",
                code: "002",
            },
            VillageCode {
                name: "哈尔墩村委会",
                code: "003",
            },
            VillageCode {
                name: "卡尕麦来村委会",
                code: "004",
            },
            VillageCode {
                name: "塔拉布拉克村委会",
                code: "005",
            },
            VillageCode {
                name: "其盖布拉克村委会",
                code: "006",
            },
            VillageCode {
                name: "乌尊布拉克村委会",
                code: "007",
            },
            VillageCode {
                name: "牧业村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "哈尔巴克乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "玉奇托乎拉克村委会",
                code: "001",
            },
            VillageCode {
                name: "库克色格孜村委会",
                code: "002",
            },
            VillageCode {
                name: "吾夏克铁热克村委会",
                code: "003",
            },
            VillageCode {
                name: "哈尔东村委会",
                code: "004",
            },
            VillageCode {
                name: "巴格吉格代村委会",
                code: "005",
            },
            VillageCode {
                name: "卡西比西村委会",
                code: "006",
            },
            VillageCode {
                name: "哈尔巴克村委会",
                code: "007",
            },
            VillageCode {
                name: "阔什吐格曼村委会",
                code: "008",
            },
            VillageCode {
                name: "库台克布拉克村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "野云沟乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "野云沟村委会",
                code: "001",
            },
            VillageCode {
                name: "塔勒克村委会",
                code: "002",
            },
            VillageCode {
                name: "阿克塔木村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "阿克萨来乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "卡塔苏盖提村委会",
                code: "001",
            },
            VillageCode {
                name: "阿克萨来村委会",
                code: "002",
            },
            VillageCode {
                name: "月堂村委会",
                code: "003",
            },
            VillageCode {
                name: "牧业村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "塔尔拉克乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "库木墩村委会",
                code: "001",
            },
            VillageCode {
                name: "塔尔拉克村委会",
                code: "002",
            },
            VillageCode {
                name: "阿克布拉克村委会",
                code: "003",
            },
            VillageCode {
                name: "牧业村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "草湖乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "英苏村委会",
                code: "001",
            },
            VillageCode {
                name: "可可桥村委会",
                code: "002",
            },
            VillageCode {
                name: "解放渠村委会",
                code: "003",
            },
            VillageCode {
                name: "阿克库木村委会",
                code: "004",
            },
            VillageCode {
                name: "阿克提坎村委会",
                code: "005",
            },
            VillageCode {
                name: "博斯坦村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "铁热克巴扎乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "阔西哈曼村委会",
                code: "001",
            },
            VillageCode {
                name: "阔那巴扎村委会",
                code: "002",
            },
            VillageCode {
                name: "曼曲鲁克村委会",
                code: "003",
            },
            VillageCode {
                name: "巴什阔玉克村委会",
                code: "004",
            },
            VillageCode {
                name: "巴格托格拉克村委会",
                code: "005",
            },
            VillageCode {
                name: "萨依麦里村委会",
                code: "006",
            },
            VillageCode {
                name: "托乎拉村委会",
                code: "007",
            },
            VillageCode {
                name: "牧业村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "策达雅乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "多斯买提村委会",
                code: "001",
            },
            VillageCode {
                name: "萨依巴克村委会",
                code: "002",
            },
            VillageCode {
                name: "其格力克村委会",
                code: "003",
            },
            VillageCode {
                name: "艾孜甘村委会",
                code: "004",
            },
            VillageCode {
                name: "牧业村委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_XJ_024: [TownCode; 11] = [
    TownCode {
        name: "尉犁镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "解放社区居委会",
                code: "001",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "002",
            },
            VillageCode {
                name: "孔雀社区居委会",
                code: "003",
            },
            VillageCode {
                name: "和平社区居委会",
                code: "004",
            },
            VillageCode {
                name: "文化社区居委会",
                code: "005",
            },
            VillageCode {
                name: "银华社区居委会",
                code: "006",
            },
            VillageCode {
                name: "五一社区居委会",
                code: "007",
            },
            VillageCode {
                name: "幸福社区居委会",
                code: "008",
            },
            VillageCode {
                name: "金宇社区居委会",
                code: "009",
            },
            VillageCode {
                name: "光明社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "团结镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "富强社区",
                code: "001",
            },
            VillageCode {
                name: "孔畔村委会",
                code: "002",
            },
            VillageCode {
                name: "西海子村委会",
                code: "003",
            },
            VillageCode {
                name: "东海子村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "兴平镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "达西村委会",
                code: "001",
            },
            VillageCode {
                name: "昆其村委会",
                code: "002",
            },
            VillageCode {
                name: "哈拉洪村委会",
                code: "003",
            },
            VillageCode {
                name: "园艺村委会",
                code: "004",
            },
            VillageCode {
                name: "统其克村委会",
                code: "005",
            },
            VillageCode {
                name: "巴西阿瓦提村委会",
                code: "006",
            },
            VillageCode {
                name: "向阳村委会",
                code: "007",
            },
            VillageCode {
                name: "孔雀村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "塔里木乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "塔里木村委会",
                code: "001",
            },
            VillageCode {
                name: "园艺村委会",
                code: "002",
            },
            VillageCode {
                name: "库木库勒村委会",
                code: "003",
            },
            VillageCode {
                name: "库万库勒村委会",
                code: "004",
            },
            VillageCode {
                name: "博斯坦村委会",
                code: "005",
            },
            VillageCode {
                name: "英努尔村委会",
                code: "006",
            },
            VillageCode {
                name: "拜海提村委会",
                code: "007",
            },
            VillageCode {
                name: "琼库勒村委会",
                code: "008",
            },
            VillageCode {
                name: "东海子村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "墩阔坦乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "库木巴格村委会",
                code: "001",
            },
            VillageCode {
                name: "塔特里克村委会",
                code: "002",
            },
            VillageCode {
                name: "墩阔坦村委会",
                code: "003",
            },
            VillageCode {
                name: "琼库勒村委会",
                code: "004",
            },
            VillageCode {
                name: "霍尔加村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "喀尔曲尕乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "喀尔曲尕村委会",
                code: "001",
            },
            VillageCode {
                name: "阿瓦提村委会",
                code: "002",
            },
            VillageCode {
                name: "琼买里村委会",
                code: "003",
            },
            VillageCode {
                name: "阿克牙斯克村委会",
                code: "004",
            },
            VillageCode {
                name: "英买里村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "阿克苏普乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "英巴格村委会",
                code: "001",
            },
            VillageCode {
                name: "喀尔喀提村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "古勒巴格乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "库克喀衣那木村委会",
                code: "001",
            },
            VillageCode {
                name: "古勒巴格村委会",
                code: "002",
            },
            VillageCode {
                name: "阿克其开村委会",
                code: "003",
            },
            VillageCode {
                name: "巴西买里村委会",
                code: "004",
            },
            VillageCode {
                name: "奥曼库勒村委会",
                code: "005",
            },
            VillageCode {
                name: "红光村委会",
                code: "006",
            },
            VillageCode {
                name: "兴地村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "兵团三十一团",
        code: "009",
        villages: &[
            VillageCode {
                name: "朝阳社区",
                code: "001",
            },
            VillageCode {
                name: "阳光社区",
                code: "002",
            },
            VillageCode {
                name: "一连队",
                code: "003",
            },
            VillageCode {
                name: "二连队",
                code: "004",
            },
            VillageCode {
                name: "三连队",
                code: "005",
            },
            VillageCode {
                name: "四连队",
                code: "006",
            },
            VillageCode {
                name: "五连队",
                code: "007",
            },
            VillageCode {
                name: "六连队",
                code: "008",
            },
            VillageCode {
                name: "七连队",
                code: "009",
            },
            VillageCode {
                name: "八连队",
                code: "010",
            },
            VillageCode {
                name: "九连队",
                code: "011",
            },
            VillageCode {
                name: "十连队",
                code: "012",
            },
            VillageCode {
                name: "十一连队",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "兵团三十三团",
        code: "010",
        villages: &[
            VillageCode {
                name: "拥军社区",
                code: "001",
            },
            VillageCode {
                name: "承德社区",
                code: "002",
            },
            VillageCode {
                name: "山水社区",
                code: "003",
            },
            VillageCode {
                name: "一连队",
                code: "004",
            },
            VillageCode {
                name: "二连队",
                code: "005",
            },
            VillageCode {
                name: "三连队",
                code: "006",
            },
            VillageCode {
                name: "五连队",
                code: "007",
            },
            VillageCode {
                name: "六连队",
                code: "008",
            },
            VillageCode {
                name: "八连队",
                code: "009",
            },
            VillageCode {
                name: "九连队",
                code: "010",
            },
            VillageCode {
                name: "十连队",
                code: "011",
            },
            VillageCode {
                name: "十一连队",
                code: "012",
            },
            VillageCode {
                name: "农一连队",
                code: "013",
            },
            VillageCode {
                name: "农三连队",
                code: "014",
            },
            VillageCode {
                name: "十二连队",
                code: "015",
            },
            VillageCode {
                name: "林一连队",
                code: "016",
            },
            VillageCode {
                name: "一十六连队",
                code: "017",
            },
            VillageCode {
                name: "四连队",
                code: "018",
            },
            VillageCode {
                name: "十五连队",
                code: "019",
            },
            VillageCode {
                name: "七连队",
                code: "020",
            },
            VillageCode {
                name: "一十八连队",
                code: "021",
            },
            VillageCode {
                name: "一十九连队",
                code: "022",
            },
            VillageCode {
                name: "二十连队",
                code: "023",
            },
            VillageCode {
                name: "一十七连队",
                code: "024",
            },
            VillageCode {
                name: "蛭石矿生活区",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "兵团三十四团",
        code: "011",
        villages: &[
            VillageCode {
                name: "金鹿社区",
                code: "001",
            },
            VillageCode {
                name: "营盘社区",
                code: "002",
            },
            VillageCode {
                name: "一连队",
                code: "003",
            },
            VillageCode {
                name: "二连队",
                code: "004",
            },
            VillageCode {
                name: "三连队",
                code: "005",
            },
            VillageCode {
                name: "四连队",
                code: "006",
            },
            VillageCode {
                name: "六连队",
                code: "007",
            },
            VillageCode {
                name: "七连队",
                code: "008",
            },
            VillageCode {
                name: "九连队",
                code: "009",
            },
            VillageCode {
                name: "农三连队",
                code: "010",
            },
            VillageCode {
                name: "十二连队",
                code: "011",
            },
            VillageCode {
                name: "畜牧总场生活区",
                code: "012",
            },
            VillageCode {
                name: "十六连队",
                code: "013",
            },
            VillageCode {
                name: "十连队",
                code: "014",
            },
            VillageCode {
                name: "十一连队",
                code: "015",
            },
            VillageCode {
                name: "五连队",
                code: "016",
            },
        ],
    },
];

static TOWNS_XJ_025: [TownCode; 8] = [
    TownCode {
        name: "若羌镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "胜利社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "文化社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "团结社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "新城社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "楼兰社区居民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "依吞布拉克镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "依吞布拉克社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "阿尔金社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "巴什考贡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "昆玉村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "罗布泊镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "罗钾社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "米兰村民委员会",
                code: "002",
            },
            VillageCode {
                name: "红卫村民委员会",
                code: "003",
            },
            VillageCode {
                name: "雅丹村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "瓦石峡镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "瓦石峡社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "乌都勒吾斯塘村民委员会",
                code: "002",
            },
            VillageCode {
                name: "吾塔木村民委员会",
                code: "003",
            },
            VillageCode {
                name: "新建村民委员会",
                code: "004",
            },
            VillageCode {
                name: "牧业村民委员会",
                code: "005",
            },
            VillageCode {
                name: "塔什萨依村民委员会",
                code: "006",
            },
            VillageCode {
                name: "吐格曼塔什萨依村民委员会",
                code: "007",
            },
            VillageCode {
                name: "金胡杨村民委员会",
                code: "008",
            },
            VillageCode {
                name: "羌都村民委员会",
                code: "009",
            },
            VillageCode {
                name: "康土盖村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "铁干里克镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "古力巴格社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "物流园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "果勒吾斯塘村民委员会",
                code: "003",
            },
            VillageCode {
                name: "亚喀吾斯塘村民委员会",
                code: "004",
            },
            VillageCode {
                name: "库尔干村民委员会",
                code: "005",
            },
            VillageCode {
                name: "托格拉克勒克村民委员会",
                code: "006",
            },
            VillageCode {
                name: "英苏牧业村民委员会",
                code: "007",
            },
            VillageCode {
                name: "铁干里克村民委员会",
                code: "008",
            },
            VillageCode {
                name: "努尔巴格村民委员会",
                code: "009",
            },
            VillageCode {
                name: "蒲昌村民委员会",
                code: "010",
            },
            VillageCode {
                name: "阿拉干村民委员会",
                code: "011",
            },
            VillageCode {
                name: "若水村民委员会",
                code: "012",
            },
            VillageCode {
                name: "天泽村民委员会",
                code: "013",
            },
            VillageCode {
                name: "罗布庄村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "吾塔木乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "果勒艾日克村民委员会",
                code: "001",
            },
            VillageCode {
                name: "尤勒滚艾日克村民委员会",
                code: "002",
            },
            VillageCode {
                name: "依格孜吾斯塘村民委员会",
                code: "003",
            },
            VillageCode {
                name: "西塔提让村民委员会",
                code: "004",
            },
            VillageCode {
                name: "牧业村民委员会",
                code: "005",
            },
            VillageCode {
                name: "康拉克村民委员会",
                code: "006",
            },
            VillageCode {
                name: "昆其村民委员会",
                code: "007",
            },
            VillageCode {
                name: "英格里克村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "铁木里克乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "铁木里克村民委员会",
                code: "001",
            },
            VillageCode {
                name: "白干湖村民委员会",
                code: "002",
            },
            VillageCode {
                name: "拉配泉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "玉泉村民委员会",
                code: "004",
            },
            VillageCode {
                name: "丹水村民委员会",
                code: "005",
            },
            VillageCode {
                name: "阳光村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "祁曼塔格乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "祁曼塔格村民委员会",
                code: "001",
            },
            VillageCode {
                name: "喀拉乔卡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "瑶池村民委员会",
                code: "003",
            },
        ],
    },
];

static TOWNS_XJ_026: [TownCode; 13] = [
    TownCode {
        name: "且末镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "却格里买亚社区",
                code: "001",
            },
            VillageCode {
                name: "古路卡吾入克社区",
                code: "002",
            },
            VillageCode {
                name: "加哈巴格社区",
                code: "003",
            },
            VillageCode {
                name: "科台买社区",
                code: "004",
            },
            VillageCode {
                name: "电视新村社区",
                code: "005",
            },
            VillageCode {
                name: "佳园社区",
                code: "006",
            },
            VillageCode {
                name: "菜队村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "奥依亚依拉克镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "奥依亚依拉克村委会",
                code: "001",
            },
            VillageCode {
                name: "布古纳村委会",
                code: "002",
            },
            VillageCode {
                name: "阿尔帕村委会",
                code: "003",
            },
            VillageCode {
                name: "色日克阔勒村委会",
                code: "004",
            },
            VillageCode {
                name: "苏塘村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "塔提让镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "巴什塔提让村委会",
                code: "001",
            },
            VillageCode {
                name: "台吐库勒村委会",
                code: "002",
            },
            VillageCode {
                name: "色日克布央村委会",
                code: "003",
            },
            VillageCode {
                name: "阿亚克塔提让村委会",
                code: "004",
            },
            VillageCode {
                name: "阿德热斯曼村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "塔中镇",
        code: "004",
        villages: &[VillageCode {
            name: "塔中社区",
            code: "001",
        }],
    },
    TownCode {
        name: "阿羌镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "阿羌村委会",
                code: "001",
            },
            VillageCode {
                name: "喀特勒什村委会",
                code: "002",
            },
            VillageCode {
                name: "依山干村委会",
                code: "003",
            },
            VillageCode {
                name: "萨尔干吉村委会",
                code: "004",
            },
            VillageCode {
                name: "昆其布拉克村委会",
                code: "005",
            },
            VillageCode {
                name: "吐拉村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "阿热勒镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "古再勒村委会",
                code: "001",
            },
            VillageCode {
                name: "阿热勒村委会",
                code: "002",
            },
            VillageCode {
                name: "亚喀吾斯塘村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "琼库勒乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "墩买里村委会",
                code: "001",
            },
            VillageCode {
                name: "欧吐拉艾日克村委会",
                code: "002",
            },
            VillageCode {
                name: "克亚克勒克村委会",
                code: "003",
            },
            VillageCode {
                name: "琼库勒村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "托格拉克勒克乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "扎滚鲁克村委会",
                code: "001",
            },
            VillageCode {
                name: "托格拉克勒克村委会",
                code: "002",
            },
            VillageCode {
                name: "兰干村委会",
                code: "003",
            },
            VillageCode {
                name: "阿日希村委会",
                code: "004",
            },
            VillageCode {
                name: "阔什艾日克村委会",
                code: "005",
            },
            VillageCode {
                name: "加瓦艾日克村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "巴格艾日克乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "阿其玛艾日克村委会",
                code: "001",
            },
            VillageCode {
                name: "巴格艾日克村委会",
                code: "002",
            },
            VillageCode {
                name: "江大铁日木村委会",
                code: "003",
            },
            VillageCode {
                name: "科台买艾日克村委会",
                code: "004",
            },
            VillageCode {
                name: "克仁艾日克村委会",
                code: "005",
            },
            VillageCode {
                name: "其盖喀什村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "英吾斯塘乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "英吾斯塘村委会",
                code: "001",
            },
            VillageCode {
                name: "铁热格勒克库勒村委会",
                code: "002",
            },
            VillageCode {
                name: "阿瓦提村委会",
                code: "003",
            },
            VillageCode {
                name: "科台买艾日克村委会",
                code: "004",
            },
            VillageCode {
                name: "塔格艾日克村委会",
                code: "005",
            },
            VillageCode {
                name: "吐排吾斯塘村委会",
                code: "006",
            },
            VillageCode {
                name: "艾盖希铁热木村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "阿克提坎墩乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "阿克提坎墩村委会",
                code: "001",
            },
            VillageCode {
                name: "伊斯克吾塔克村委会",
                code: "002",
            },
            VillageCode {
                name: "托格拉克艾格勒村委会",
                code: "003",
            },
            VillageCode {
                name: "色格孜勒克希庞村委会",
                code: "004",
            },
            VillageCode {
                name: "恰瓦勒墩管委会村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "阔什萨特玛乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "阔什萨特玛村委会",
                code: "001",
            },
            VillageCode {
                name: "苏尕克布拉克村委会",
                code: "002",
            },
            VillageCode {
                name: "阿勒玛铁热木村委会",
                code: "003",
            },
            VillageCode {
                name: "托盖苏拉克村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "库拉木勒克乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "库拉木勒克村委会",
                code: "001",
            },
            VillageCode {
                name: "其木布拉克村委会",
                code: "002",
            },
            VillageCode {
                name: "阿克亚村委会",
                code: "003",
            },
            VillageCode {
                name: "江尕勒萨依村委会",
                code: "004",
            },
            VillageCode {
                name: "巴什克其克村委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_XJ_027: [TownCode; 10] = [
    TownCode {
        name: "焉耆镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "新城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新桥社区居委会",
                code: "002",
            },
            VillageCode {
                name: "解放社区居委会",
                code: "003",
            },
            VillageCode {
                name: "文苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "滨河社区居委会",
                code: "005",
            },
            VillageCode {
                name: "迎宾社区居委会",
                code: "006",
            },
            VillageCode {
                name: "商城社区居委会",
                code: "007",
            },
            VillageCode {
                name: "和平社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新华社区居委会",
                code: "009",
            },
            VillageCode {
                name: "友好社区居委会",
                code: "010",
            },
            VillageCode {
                name: "团结社区",
                code: "011",
            },
            VillageCode {
                name: "佳星社区居委会",
                code: "012",
            },
            VillageCode {
                name: "文化社区居委会",
                code: "013",
            },
            VillageCode {
                name: "金粮社区居委会",
                code: "014",
            },
            VillageCode {
                name: "海都社区居委会",
                code: "015",
            },
            VillageCode {
                name: "团结村居委会",
                code: "016",
            },
            VillageCode {
                name: "上四号渠村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "七个星镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "幸福路居委会",
                code: "001",
            },
            VillageCode {
                name: "桑巴巴格次村委会",
                code: "002",
            },
            VillageCode {
                name: "夏热采开村委会",
                code: "003",
            },
            VillageCode {
                name: "七个星村委会",
                code: "004",
            },
            VillageCode {
                name: "老城村委会",
                code: "005",
            },
            VillageCode {
                name: "哈尔莫墩村委会",
                code: "006",
            },
            VillageCode {
                name: "乎尔东村委会",
                code: "007",
            },
            VillageCode {
                name: "乃明莫墩村委会",
                code: "008",
            },
            VillageCode {
                name: "霍拉山村委会",
                code: "009",
            },
            VillageCode {
                name: "芒日戈拉尔村委会",
                code: "010",
            },
            VillageCode {
                name: "紫泥泉村",
                code: "011",
            },
            VillageCode {
                name: "霍拉哈木墩村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "永宁镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "南河路居委会",
                code: "001",
            },
            VillageCode {
                name: "永新路居委会",
                code: "002",
            },
            VillageCode {
                name: "下岔河村委会",
                code: "003",
            },
            VillageCode {
                name: "马莲滩村委会",
                code: "004",
            },
            VillageCode {
                name: "新户村村委会",
                code: "005",
            },
            VillageCode {
                name: "九号渠村委会",
                code: "006",
            },
            VillageCode {
                name: "黑疙瘩村委会",
                code: "007",
            },
            VillageCode {
                name: "新居户村委会",
                code: "008",
            },
            VillageCode {
                name: "上岔河村委会",
                code: "009",
            },
            VillageCode {
                name: "西大渠村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "四十里城子镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "新城居委会",
                code: "001",
            },
            VillageCode {
                name: "店子村委会",
                code: "002",
            },
            VillageCode {
                name: "阿克墩村委会",
                code: "003",
            },
            VillageCode {
                name: "博格达村委会",
                code: "004",
            },
            VillageCode {
                name: "巴克来村委会",
                code: "005",
            },
            VillageCode {
                name: "新渠村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "北大渠乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "太平渠村委会",
                code: "001",
            },
            VillageCode {
                name: "北大渠村委会",
                code: "002",
            },
            VillageCode {
                name: "八家户村委会",
                code: "003",
            },
            VillageCode {
                name: "北渠村委会",
                code: "004",
            },
            VillageCode {
                name: "六十户村委会",
                code: "005",
            },
            VillageCode {
                name: "十号渠村委会",
                code: "006",
            },
            VillageCode {
                name: "丰达实验农场村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "五号渠乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "光明社区",
                code: "001",
            },
            VillageCode {
                name: "阳光社区",
                code: "002",
            },
            VillageCode {
                name: "查汗渠村委会",
                code: "003",
            },
            VillageCode {
                name: "阿伦渠村委会",
                code: "004",
            },
            VillageCode {
                name: "头号渠村委会",
                code: "005",
            },
            VillageCode {
                name: "四号渠村委会",
                code: "006",
            },
            VillageCode {
                name: "上五号渠村委会",
                code: "007",
            },
            VillageCode {
                name: "中五号渠村委会",
                code: "008",
            },
            VillageCode {
                name: "下五号渠村委会",
                code: "009",
            },
            VillageCode {
                name: "下三号渠村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "查汗采开乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "查汗采开村委会",
                code: "001",
            },
            VillageCode {
                name: "布热村委会",
                code: "002",
            },
            VillageCode {
                name: "阿尔莫墩村委会",
                code: "003",
            },
            VillageCode {
                name: "莫哈尔苏木村委会",
                code: "004",
            },
            VillageCode {
                name: "嘎伦莫墩村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "包尔海乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "查汗布乎村委会",
                code: "001",
            },
            VillageCode {
                name: "开来提村委会",
                code: "002",
            },
            VillageCode {
                name: "岱尔斯村委会",
                code: "003",
            },
            VillageCode {
                name: "包尔海村委会",
                code: "004",
            },
            VillageCode {
                name: "夏热勒代村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "王家庄牧场",
        code: "009",
        villages: &[
            VillageCode {
                name: "第一村委会",
                code: "001",
            },
            VillageCode {
                name: "第二村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "苏海良种场",
        code: "010",
        villages: &[
            VillageCode {
                name: "第一村委会",
                code: "001",
            },
            VillageCode {
                name: "第二村委会",
                code: "002",
            },
        ],
    },
];

static TOWNS_XJ_028: [TownCode; 13] = [
    TownCode {
        name: "和静镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "克再西路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "建设一路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "团结西路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "查汗通古北路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "阿尔夏特路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "乌鲁木齐市和静农牧场社区",
                code: "006",
            },
            VillageCode {
                name: "东风社区居委会",
                code: "007",
            },
            VillageCode {
                name: "吉祥社区居委会",
                code: "008",
            },
            VillageCode {
                name: "兴合社区居委会",
                code: "009",
            },
            VillageCode {
                name: "鸿雁社区居委会",
                code: "010",
            },
            VillageCode {
                name: "田园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "金水湾社区居委会",
                code: "012",
            },
            VillageCode {
                name: "新苑社区居委会",
                code: "013",
            },
            VillageCode {
                name: "东归社区居委会",
                code: "014",
            },
            VillageCode {
                name: "天富社区居委会",
                code: "015",
            },
            VillageCode {
                name: "阳光社区居委会",
                code: "016",
            },
            VillageCode {
                name: "江格尔社区",
                code: "017",
            },
            VillageCode {
                name: "富民社区居委会",
                code: "018",
            },
            VillageCode {
                name: "新希望社区居委会",
                code: "019",
            },
            VillageCode {
                name: "克再村委会",
                code: "020",
            },
            VillageCode {
                name: "夏尔布鲁克村委会",
                code: "021",
            },
            VillageCode {
                name: "巩哈尔村委会",
                code: "022",
            },
            VillageCode {
                name: "查汗通古村委会",
                code: "023",
            },
            VillageCode {
                name: "阿力腾布鲁克村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "巴伦台镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南桥社区居委会",
                code: "002",
            },
            VillageCode {
                name: "乌市牧场村民委员会",
                code: "003",
            },
            VillageCode {
                name: "呼斯台村委会",
                code: "004",
            },
            VillageCode {
                name: "巴伦台村委会",
                code: "005",
            },
            VillageCode {
                name: "乌拉斯台村委会",
                code: "006",
            },
            VillageCode {
                name: "包格旦郭勒村委会",
                code: "007",
            },
            VillageCode {
                name: "夏尔才开村委会",
                code: "008",
            },
            VillageCode {
                name: "古仁郭勒村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "巴润哈尔莫敦镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "文化街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "商业街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "呼青衙门村委会",
                code: "003",
            },
            VillageCode {
                name: "查汗赛尔村委会",
                code: "004",
            },
            VillageCode {
                name: "阿日勒村委会",
                code: "005",
            },
            VillageCode {
                name: "哈尔乌苏村委会",
                code: "006",
            },
            VillageCode {
                name: "阿尔孜尕尔村委会",
                code: "007",
            },
            VillageCode {
                name: "开来村委会",
                code: "008",
            },
            VillageCode {
                name: "拜勒其尔村委会",
                code: "009",
            },
            VillageCode {
                name: "查汗通古村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "哈尔莫敦镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "文化路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "乌兰尕扎尔村委会",
                code: "002",
            },
            VillageCode {
                name: "哈尔莫敦村委会",
                code: "003",
            },
            VillageCode {
                name: "查茨村委会",
                code: "004",
            },
            VillageCode {
                name: "觉伦图尔根村委会",
                code: "005",
            },
            VillageCode {
                name: "萨拉村委会",
                code: "006",
            },
            VillageCode {
                name: "夏尔莫敦村委会",
                code: "007",
            },
            VillageCode {
                name: "乌拉斯台村委会",
                code: "008",
            },
            VillageCode {
                name: "海迪克村",
                code: "009",
            },
            VillageCode {
                name: "乃仁哈尔村",
                code: "010",
            },
            VillageCode {
                name: "才干布鲁克村",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "巴音布鲁克镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "天鹅湖社区居委会",
                code: "001",
            },
            VillageCode {
                name: "艾尔宾社区居委会",
                code: "002",
            },
            VillageCode {
                name: "巴西里格村委会",
                code: "003",
            },
            VillageCode {
                name: "藏德图哈德村委会",
                code: "004",
            },
            VillageCode {
                name: "敖伦布鲁克村委会",
                code: "005",
            },
            VillageCode {
                name: "赛热木村委会",
                code: "006",
            },
            VillageCode {
                name: "查汗赛尔村委会",
                code: "007",
            },
            VillageCode {
                name: "赛罕陶海村委会",
                code: "008",
            },
            VillageCode {
                name: "伊克扎尕斯台村民委员会",
                code: "009",
            },
            VillageCode {
                name: "德尔比勒金村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "巩乃斯镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "巩乃斯社区居委会",
                code: "001",
            },
            VillageCode {
                name: "阿尔先郭勒村委会",
                code: "002",
            },
            VillageCode {
                name: "浩伊特开勒德村委会",
                code: "003",
            },
            VillageCode {
                name: "巩乃斯郭勒村委会",
                code: "004",
            },
            VillageCode {
                name: "巩乃斯林场村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "乃门莫敦镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "友好路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "乃门莫敦村委会",
                code: "002",
            },
            VillageCode {
                name: "包尔尕扎村委会",
                code: "003",
            },
            VillageCode {
                name: "包尔布呼村委会",
                code: "004",
            },
            VillageCode {
                name: "古尔温苏门村委会",
                code: "005",
            },
            VillageCode {
                name: "夏尔乌苏村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "协比乃尔布呼镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "协比乃尔布呼村委会",
                code: "001",
            },
            VillageCode {
                name: "查汗才开村委会",
                code: "002",
            },
            VillageCode {
                name: "开发社区居委会",
                code: "003",
            },
            VillageCode {
                name: "建设社区居委会",
                code: "004",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "005",
            },
            VillageCode {
                name: "兴园社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "克尔古提乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "浩尔哈特村委会",
                code: "001",
            },
            VillageCode {
                name: "那英特村委会",
                code: "002",
            },
            VillageCode {
                name: "克尔古提村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "阿拉沟乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "阿拉沟村委会",
                code: "001",
            },
            VillageCode {
                name: "夏尔尕村委会",
                code: "002",
            },
            VillageCode {
                name: "乌拉斯台查汗村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "额勒再特乌鲁乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "古尔温吐勒尕村委会",
                code: "001",
            },
            VillageCode {
                name: "乌兰布鲁克村委会",
                code: "002",
            },
            VillageCode {
                name: "额勒再特乌鲁村委会",
                code: "003",
            },
            VillageCode {
                name: "哈尔诺尔村委会",
                code: "004",
            },
            VillageCode {
                name: "察汗乌苏村委会",
                code: "005",
            },
            VillageCode {
                name: "哈尔嘎特西里村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "巴音郭楞乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "阿尔夏特村委会",
                code: "001",
            },
            VillageCode {
                name: "巴音郭楞村委会",
                code: "002",
            },
            VillageCode {
                name: "苏力间村委会",
                code: "003",
            },
            VillageCode {
                name: "哈尔萨拉村委会",
                code: "004",
            },
            VillageCode {
                name: "奎克乌苏村委会",
                code: "005",
            },
            VillageCode {
                name: "巴音塔拉村委会",
                code: "006",
            },
            VillageCode {
                name: "巴音布鲁克村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "兵团二十一团",
        code: "013",
        villages: &[
            VillageCode {
                name: "开来社区",
                code: "001",
            },
            VillageCode {
                name: "月牙湖社区",
                code: "002",
            },
            VillageCode {
                name: "一连队",
                code: "003",
            },
            VillageCode {
                name: "二连队",
                code: "004",
            },
            VillageCode {
                name: "三连队",
                code: "005",
            },
            VillageCode {
                name: "四连队",
                code: "006",
            },
            VillageCode {
                name: "五连队",
                code: "007",
            },
            VillageCode {
                name: "六连队",
                code: "008",
            },
            VillageCode {
                name: "七连队",
                code: "009",
            },
            VillageCode {
                name: "八连队",
                code: "010",
            },
            VillageCode {
                name: "十一连队",
                code: "011",
            },
            VillageCode {
                name: "十连队",
                code: "012",
            },
            VillageCode {
                name: "九连队",
                code: "013",
            },
            VillageCode {
                name: "养猪总场生活区",
                code: "014",
            },
            VillageCode {
                name: "鹿场生活区",
                code: "015",
            },
            VillageCode {
                name: "木材厂生活区",
                code: "016",
            },
            VillageCode {
                name: "园林一连队",
                code: "017",
            },
        ],
    },
];

static TOWNS_XJ_029: [TownCode; 9] = [
    TownCode {
        name: "特吾里克镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "光明中心社区",
                code: "001",
            },
            VillageCode {
                name: "团结社区",
                code: "002",
            },
            VillageCode {
                name: "文化社区",
                code: "003",
            },
            VillageCode {
                name: "龙驹社区",
                code: "004",
            },
            VillageCode {
                name: "明珠社区",
                code: "005",
            },
            VillageCode {
                name: "清水河社区",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "塔哈其镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "小康社区",
                code: "001",
            },
            VillageCode {
                name: "古努恩布呼村",
                code: "002",
            },
            VillageCode {
                name: "祖鲁门苏勒村",
                code: "003",
            },
            VillageCode {
                name: "查干布呼村",
                code: "004",
            },
            VillageCode {
                name: "浩尧尔莫墩村",
                code: "005",
            },
            VillageCode {
                name: "阿尔文德尔文村",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "曲惠镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "冬都呼都格村",
                code: "001",
            },
            VillageCode {
                name: "老城村",
                code: "002",
            },
            VillageCode {
                name: "榆树园村",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "乌什塔拉回族民族乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "拥军社区",
                code: "001",
            },
            VillageCode {
                name: "塔拉村",
                code: "002",
            },
            VillageCode {
                name: "大庄子村",
                code: "003",
            },
            VillageCode {
                name: "硝井子村",
                code: "004",
            },
            VillageCode {
                name: "则格德恩呼都格村",
                code: "005",
            },
            VillageCode {
                name: "沙井子村",
                code: "006",
            },
            VillageCode {
                name: "泽斯特村",
                code: "007",
            },
            VillageCode {
                name: "红星村",
                code: "008",
            },
            VillageCode {
                name: "大湾村",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "苏哈特乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "肖然托勒盖村",
                code: "001",
            },
            VillageCode {
                name: "苏哈特村",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "乃仁克尔乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "包尔图村",
                code: "001",
            },
            VillageCode {
                name: "艾迪恩阿门村",
                code: "002",
            },
            VillageCode {
                name: "本布图村",
                code: "003",
            },
            VillageCode {
                name: "艾勒斯特村",
                code: "004",
            },
            VillageCode {
                name: "乌勒泽特村",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "新塔热乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "新塔热村",
                code: "001",
            },
            VillageCode {
                name: "布茨恩查干村",
                code: "002",
            },
            VillageCode {
                name: "则格德恩阿茨村",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "清水河农场",
        code: "008",
        villages: &[VillageCode {
            name: "清水河农场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "和硕县马兰公安管区",
        code: "009",
        villages: &[VillageCode {
            name: "马兰机关虚拟生活区",
            code: "001",
        }],
    },
];

static TOWNS_XJ_030: [TownCode; 8] = [
    TownCode {
        name: "博湖镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "银湖社区居委会",
                code: "001",
            },
            VillageCode {
                name: "阳光社区居委会",
                code: "002",
            },
            VillageCode {
                name: "芦花社区居委会",
                code: "003",
            },
            VillageCode {
                name: "宝浪社区居委会",
                code: "004",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "005",
            },
            VillageCode {
                name: "乌什塔拉渔村社区居委会",
                code: "006",
            },
            VillageCode {
                name: "西海社区居委会",
                code: "007",
            },
            VillageCode {
                name: "幸福社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "本布图镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "新城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "本布图村委会",
                code: "002",
            },
            VillageCode {
                name: "新布呼村委会",
                code: "003",
            },
            VillageCode {
                name: "芒南查干村委会",
                code: "004",
            },
            VillageCode {
                name: "再格森诺尔村委会",
                code: "005",
            },
            VillageCode {
                name: "劳希浩诺尔村委会",
                code: "006",
            },
            VillageCode {
                name: "乔鲁图村委会",
                code: "007",
            },
            VillageCode {
                name: "那音托勒盖村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "塔温觉肯乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "塔温觉肯村委会",
                code: "001",
            },
            VillageCode {
                name: "哈尔恩格村委会",
                code: "002",
            },
            VillageCode {
                name: "科克莫敦村委会",
                code: "003",
            },
            VillageCode {
                name: "东大罕村委会",
                code: "004",
            },
            VillageCode {
                name: "克日木哈尔村委会",
                code: "005",
            },
            VillageCode {
                name: "敖瓦特村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "乌兰再格森乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "乌兰再格森村委会",
                code: "001",
            },
            VillageCode {
                name: "乌图阿热勒村委会",
                code: "002",
            },
            VillageCode {
                name: "席子木呼村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "才坎诺尔乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "哈尔尼敦村委会",
                code: "001",
            },
            VillageCode {
                name: "才坎诺尔村委会",
                code: "002",
            },
            VillageCode {
                name: "拉罕诺尔村委会",
                code: "003",
            },
            VillageCode {
                name: "莫盖图村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "查干诺尔乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "敦都布呼村委会",
                code: "001",
            },
            VillageCode {
                name: "乌腾郭楞村委会",
                code: "002",
            },
            VillageCode {
                name: "查干诺尔村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "博斯腾湖乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "库代力克村委会",
                code: "001",
            },
            VillageCode {
                name: "闹音呼都克村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "兵团二十五团",
        code: "008",
        villages: &[
            VillageCode {
                name: "湖光社区",
                code: "001",
            },
            VillageCode {
                name: "一连队",
                code: "002",
            },
            VillageCode {
                name: "三连队",
                code: "003",
            },
            VillageCode {
                name: "四连队",
                code: "004",
            },
            VillageCode {
                name: "六连队",
                code: "005",
            },
            VillageCode {
                name: "五连队",
                code: "006",
            },
            VillageCode {
                name: "二连队",
                code: "007",
            },
        ],
    },
];

static TOWNS_XJ_031: [TownCode; 18] = [
    TownCode {
        name: "金山路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "园林社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金山路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "文化路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "金山北路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "八道巷社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "解放路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "解放路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "解放南路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "览景社区居委会",
                code: "003",
            },
            VillageCode {
                name: "解放北路社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "团结路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "团结路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "团结南路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "银水路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "滨河路社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "恰秀路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "恰秀社区居委会",
                code: "001",
            },
            VillageCode {
                name: "红石社区居委会",
                code: "002",
            },
            VillageCode {
                name: "额河社区居委会",
                code: "003",
            },
            VillageCode {
                name: "雪都社区居委会",
                code: "004",
            },
            VillageCode {
                name: "克兰社区居委会",
                code: "005",
            },
            VillageCode {
                name: "红墩路社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "北屯镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "团结社区居委会",
                code: "001",
            },
            VillageCode {
                name: "额河社区居委会",
                code: "002",
            },
            VillageCode {
                name: "恩泽社区居委会",
                code: "003",
            },
            VillageCode {
                name: "阿山社区居委会",
                code: "004",
            },
            VillageCode {
                name: "胡杨社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "阿苇滩镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "库布西村委会",
                code: "001",
            },
            VillageCode {
                name: "青格劳村委会",
                code: "002",
            },
            VillageCode {
                name: "阔克塔勒村委会",
                code: "003",
            },
            VillageCode {
                name: "阿克托别村委会",
                code: "004",
            },
            VillageCode {
                name: "阿克库都克村委会",
                code: "005",
            },
            VillageCode {
                name: "萨斯克巴斯陶村委会",
                code: "006",
            },
            VillageCode {
                name: "江阿塔木村委会",
                code: "007",
            },
            VillageCode {
                name: "墩克尔曼村委会",
                code: "008",
            },
            VillageCode {
                name: "阿克喀仁村委会",
                code: "009",
            },
            VillageCode {
                name: "克孜勒乌英克村委会",
                code: "010",
            },
            VillageCode {
                name: "加依勒玛村委会",
                code: "011",
            },
            VillageCode {
                name: "阿苇滩村委会",
                code: "012",
            },
            VillageCode {
                name: "希力克德村委会",
                code: "013",
            },
            VillageCode {
                name: "阿克阔买村委会",
                code: "014",
            },
            VillageCode {
                name: "阿克阿热勒村委会",
                code: "015",
            },
            VillageCode {
                name: "喀拉干德阔拉村委会",
                code: "016",
            },
            VillageCode {
                name: "艾达尔塔勒村委会",
                code: "017",
            },
            VillageCode {
                name: "雪都村委会",
                code: "018",
            },
            VillageCode {
                name: "毕依克哈巴克村委会",
                code: "019",
            },
            VillageCode {
                name: "喀拉铁列克村委会",
                code: "020",
            },
            VillageCode {
                name: "喀拉塔斯村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "红墩镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "阔克萨孜村委会",
                code: "001",
            },
            VillageCode {
                name: "博肯布拉克村委会",
                code: "002",
            },
            VillageCode {
                name: "玛依阔勒特克村委会",
                code: "003",
            },
            VillageCode {
                name: "乌图布拉克村委会",
                code: "004",
            },
            VillageCode {
                name: "萨亚铁热克村委会",
                code: "005",
            },
            VillageCode {
                name: "多拉特村委会",
                code: "006",
            },
            VillageCode {
                name: "萨尔喀木斯村委会",
                code: "007",
            },
            VillageCode {
                name: "克亚乌特开勒村委会",
                code: "008",
            },
            VillageCode {
                name: "阿克塔斯村委会",
                code: "009",
            },
            VillageCode {
                name: "阔克布喀村委会",
                code: "010",
            },
            VillageCode {
                name: "喀木斯特村委会",
                code: "011",
            },
            VillageCode {
                name: "克列铁克依村委会",
                code: "012",
            },
            VillageCode {
                name: "比铁吾尔格村委会",
                code: "013",
            },
            VillageCode {
                name: "锡别特村委会",
                code: "014",
            },
            VillageCode {
                name: "克孜勒加尔村委会",
                code: "015",
            },
            VillageCode {
                name: "吾勒格村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "切木尔切克镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "阔尕勒村委会",
                code: "001",
            },
            VillageCode {
                name: "喀拉塔斯村委会",
                code: "002",
            },
            VillageCode {
                name: "海那尔村委会",
                code: "003",
            },
            VillageCode {
                name: "洪吾尔托呼特村委会",
                code: "004",
            },
            VillageCode {
                name: "别斯巴斯陶村委会",
                code: "005",
            },
            VillageCode {
                name: "拜格托别村委会",
                code: "006",
            },
            VillageCode {
                name: "阔克什木村委会",
                code: "007",
            },
            VillageCode {
                name: "切木尔切克村委会",
                code: "008",
            },
            VillageCode {
                name: "森塔斯村委会",
                code: "009",
            },
            VillageCode {
                name: "希巴尔齐村委会",
                code: "010",
            },
            VillageCode {
                name: "肯迪尔里克村委会",
                code: "011",
            },
            VillageCode {
                name: "巴勒喀木斯村委会",
                code: "012",
            },
            VillageCode {
                name: "也克阿恰村委会",
                code: "013",
            },
            VillageCode {
                name: "多尔根村委会",
                code: "014",
            },
            VillageCode {
                name: "吉特库勒村委会",
                code: "015",
            },
            VillageCode {
                name: "也尔特斯村委会",
                code: "016",
            },
            VillageCode {
                name: "阿克克勒希村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "阿拉哈克镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "托普乌英克村委会",
                code: "001",
            },
            VillageCode {
                name: "阿克齐村委会",
                code: "002",
            },
            VillageCode {
                name: "阿拉哈克村委会",
                code: "003",
            },
            VillageCode {
                name: "喀拉库木村委会",
                code: "004",
            },
            VillageCode {
                name: "铁斯克别依特村委会",
                code: "005",
            },
            VillageCode {
                name: "塔尔浪村委会",
                code: "006",
            },
            VillageCode {
                name: "赛克赛吾勒吐别克村委会",
                code: "007",
            },
            VillageCode {
                name: "阿热勒村委会",
                code: "008",
            },
            VillageCode {
                name: "比铁吾塔勒村委会",
                code: "009",
            },
            VillageCode {
                name: "喀拉塔勒村委会",
                code: "010",
            },
            VillageCode {
                name: "窝依玛克村委会",
                code: "011",
            },
            VillageCode {
                name: "努尔沼村委会",
                code: "012",
            },
            VillageCode {
                name: "阔阔尔图村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "汗德尕特蒙古族乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "汗德尕特村委会",
                code: "001",
            },
            VillageCode {
                name: "霍布勒特村委会",
                code: "002",
            },
            VillageCode {
                name: "乔尔海村委会",
                code: "003",
            },
            VillageCode {
                name: "阿尔恰特村委会",
                code: "004",
            },
            VillageCode {
                name: "角萨特村委会",
                code: "005",
            },
            VillageCode {
                name: "达布勒哈特村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "拉斯特乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "拉斯特村委会",
                code: "001",
            },
            VillageCode {
                name: "喀拉阔里村委会",
                code: "002",
            },
            VillageCode {
                name: "诺改特村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "喀拉希力克乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "喀拉希力克村委会",
                code: "001",
            },
            VillageCode {
                name: "加勒齐村委会",
                code: "002",
            },
            VillageCode {
                name: "比铁吾铁热克村委会",
                code: "003",
            },
            VillageCode {
                name: "恰特别依特村委会",
                code: "004",
            },
            VillageCode {
                name: "阿克布勒根村委会",
                code: "005",
            },
            VillageCode {
                name: "阿克铁列热克村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "萨尔胡松乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "散德克库木村委会",
                code: "001",
            },
            VillageCode {
                name: "库尔尕克托干村委会",
                code: "002",
            },
            VillageCode {
                name: "喀拉铁热克村委会",
                code: "003",
            },
            VillageCode {
                name: "萨尔胡松村委会",
                code: "004",
            },
            VillageCode {
                name: "库尔特村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "巴里巴盖乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "巴里巴盖村委会",
                code: "001",
            },
            VillageCode {
                name: "喀拉尕什村委会",
                code: "002",
            },
            VillageCode {
                name: "萨尔喀仁村委会",
                code: "003",
            },
            VillageCode {
                name: "巴鲁旺塔斯村委会",
                code: "004",
            },
            VillageCode {
                name: "阔克尔图村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "切尔克齐乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "克孜勒希力克村委会",
                code: "001",
            },
            VillageCode {
                name: "库早齐村委会",
                code: "002",
            },
            VillageCode {
                name: "阿克恰普巴村委会",
                code: "003",
            },
            VillageCode {
                name: "克孜勒喀英村委会",
                code: "004",
            },
            VillageCode {
                name: "康格村委会",
                code: "005",
            },
            VillageCode {
                name: "克孜勒乌英克村委会",
                code: "006",
            },
            VillageCode {
                name: "克孜勒喀巴克村委会",
                code: "007",
            },
            VillageCode {
                name: "克孜勒别勒村委会",
                code: "008",
            },
            VillageCode {
                name: "阿克别勒村委会",
                code: "009",
            },
            VillageCode {
                name: "阿勒玛勒村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "喀拉尕什牧场",
        code: "016",
        villages: &[
            VillageCode {
                name: "哈拉托别生活区",
                code: "001",
            },
            VillageCode {
                name: "克什阔布生活区",
                code: "002",
            },
            VillageCode {
                name: "克孜塔斯生活区",
                code: "003",
            },
            VillageCode {
                name: "恰普汗塔斯生活区",
                code: "004",
            },
            VillageCode {
                name: "阿克达拉生活区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "阿克吐木斯克牧场",
        code: "017",
        villages: &[
            VillageCode {
                name: "莫因生活区",
                code: "001",
            },
            VillageCode {
                name: "阿克吐木斯克生活区",
                code: "002",
            },
            VillageCode {
                name: "克依干库都克生活区",
                code: "003",
            },
            VillageCode {
                name: "大哈拉苏生活区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "兵团一八一团",
        code: "018",
        villages: &[
            VillageCode {
                name: "团部社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
        ],
    },
];

static TOWNS_XJ_032: [TownCode; 7] = [
    TownCode {
        name: "布尔津镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "额河居委会",
                code: "001",
            },
            VillageCode {
                name: "七彩河居委会",
                code: "002",
            },
            VillageCode {
                name: "神湖居委会",
                code: "003",
            },
            VillageCode {
                name: "五彩滩居委会",
                code: "004",
            },
            VillageCode {
                name: "友谊峰居委会",
                code: "005",
            },
            VillageCode {
                name: "津河居委会",
                code: "006",
            },
            VillageCode {
                name: "双河居委会",
                code: "007",
            },
            VillageCode {
                name: "切克台村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "冲乎尔镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "喀拉克木尔村委会",
                code: "001",
            },
            VillageCode {
                name: "库须根村委会",
                code: "002",
            },
            VillageCode {
                name: "孔吐汗村委会",
                code: "003",
            },
            VillageCode {
                name: "布拉乃村委会",
                code: "004",
            },
            VillageCode {
                name: "齐巴尔托布勒格村委会",
                code: "005",
            },
            VillageCode {
                name: "阿克阿依日克村委会",
                code: "006",
            },
            VillageCode {
                name: "阿木拉西台村委会",
                code: "007",
            },
            VillageCode {
                name: "阿克齐村委会",
                code: "008",
            },
            VillageCode {
                name: "冲乎尔村委会",
                code: "009",
            },
            VillageCode {
                name: "克孜勒塔斯村委会",
                code: "010",
            },
            VillageCode {
                name: "库克铁热克村委会",
                code: "011",
            },
            VillageCode {
                name: "波尔托别村委会",
                code: "012",
            },
            VillageCode {
                name: "合孜勒哈英村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "窝依莫克镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "强吐别克村委会",
                code: "001",
            },
            VillageCode {
                name: "恰尔巴克奥提克勒村委会",
                code: "002",
            },
            VillageCode {
                name: "阿勒特拜村委会",
                code: "003",
            },
            VillageCode {
                name: "窝依莫克村委会",
                code: "004",
            },
            VillageCode {
                name: "阿克别依特村委会",
                code: "005",
            },
            VillageCode {
                name: "加勒格孜塔勒村委会",
                code: "006",
            },
            VillageCode {
                name: "库尔木斯村委会",
                code: "007",
            },
            VillageCode {
                name: "克孜勒喀巴克村委会",
                code: "008",
            },
            VillageCode {
                name: "窝依阔克别克村委会",
                code: "009",
            },
            VillageCode {
                name: "喀拉加勒村委会",
                code: "010",
            },
            VillageCode {
                name: "托库木特村委会",
                code: "011",
            },
            VillageCode {
                name: "也拉曼村委会",
                code: "012",
            },
            VillageCode {
                name: "阿克布勒根村委会",
                code: "013",
            },
            VillageCode {
                name: "喀拉库勒村委会",
                code: "014",
            },
            VillageCode {
                name: "哈太村委会",
                code: "015",
            },
            VillageCode {
                name: "阿克吐别克村委会",
                code: "016",
            },
            VillageCode {
                name: "蒙艾提巴斯村委会",
                code: "017",
            },
            VillageCode {
                name: "阿克加尔村委会",
                code: "018",
            },
            VillageCode {
                name: "通克村委会",
                code: "019",
            },
            VillageCode {
                name: "阔普巴拉村委会",
                code: "020",
            },
            VillageCode {
                name: "喀克村委会",
                code: "021",
            },
            VillageCode {
                name: "克依克拜村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "阔斯特克镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "阔斯特克村委会",
                code: "001",
            },
            VillageCode {
                name: "克孜勒乌英克村委会",
                code: "002",
            },
            VillageCode {
                name: "阿克铁热克村委会",
                code: "003",
            },
            VillageCode {
                name: "阔克阿尕什村委会",
                code: "004",
            },
            VillageCode {
                name: "阔斯托干村委会",
                code: "005",
            },
            VillageCode {
                name: "喀拉墩村委会",
                code: "006",
            },
            VillageCode {
                name: "萨尔库木村委会",
                code: "007",
            },
            VillageCode {
                name: "吉迭勒村委会",
                code: "008",
            },
            VillageCode {
                name: "什合斯托汗村委会",
                code: "009",
            },
            VillageCode {
                name: "江阿吉尔村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "杜来提乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "杜来提村委会",
                code: "001",
            },
            VillageCode {
                name: "阿肯齐村委会",
                code: "002",
            },
            VillageCode {
                name: "喀拉塔勒村委会",
                code: "003",
            },
            VillageCode {
                name: "草原一村委会",
                code: "004",
            },
            VillageCode {
                name: "草原二村委会",
                code: "005",
            },
            VillageCode {
                name: "草原三村委会",
                code: "006",
            },
            VillageCode {
                name: "草原新村委会",
                code: "007",
            },
            VillageCode {
                name: "库尔吉拉村委会",
                code: "008",
            },
            VillageCode {
                name: "阿合塔木村委会",
                code: "009",
            },
            VillageCode {
                name: "萨尔铁热克村委会",
                code: "010",
            },
            VillageCode {
                name: "额尔齐斯村委会",
                code: "011",
            },
            VillageCode {
                name: "别斯铁列克村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "也格孜托别乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "托普铁热克村委会",
                code: "001",
            },
            VillageCode {
                name: "也格孜托别村委会",
                code: "002",
            },
            VillageCode {
                name: "喀拉阿尕什村委会",
                code: "003",
            },
            VillageCode {
                name: "克孜勒托盖村委会",
                code: "004",
            },
            VillageCode {
                name: "克孜勒加尔村委会",
                code: "005",
            },
            VillageCode {
                name: "阔斯阿尔阿勒村委会",
                code: "006",
            },
            VillageCode {
                name: "吉迭勒村委会",
                code: "007",
            },
            VillageCode {
                name: "尔格尔胡木村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "禾木哈纳斯蒙古族乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "禾木村委会",
                code: "001",
            },
            VillageCode {
                name: "哈纳斯村委会",
                code: "002",
            },
        ],
    },
];

static TOWNS_XJ_033: [TownCode; 10] = [
    TownCode {
        name: "库额尔齐斯镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "赛尔江西路居委会",
                code: "001",
            },
            VillageCode {
                name: "文化西路居委会",
                code: "002",
            },
            VillageCode {
                name: "赛尔江东路居委会",
                code: "003",
            },
            VillageCode {
                name: "文化东路居委会",
                code: "004",
            },
            VillageCode {
                name: "幸福路居委会",
                code: "005",
            },
            VillageCode {
                name: "新藴社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "也尔特斯村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "可可托海镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "团结路居委会",
                code: "001",
            },
            VillageCode {
                name: "文化东路居委会",
                code: "002",
            },
            VillageCode {
                name: "文化西路居委会",
                code: "003",
            },
            VillageCode {
                name: "塔拉特村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "恰库尔图镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "迎宾路居委会",
                code: "001",
            },
            VillageCode {
                name: "哈希翁村委会",
                code: "002",
            },
            VillageCode {
                name: "乔山拜村委会",
                code: "003",
            },
            VillageCode {
                name: "恰库尔图村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "喀拉通克镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "铜镍矿社区居委会",
                code: "001",
            },
            VillageCode {
                name: "塔斯巴斯陶村委会",
                code: "002",
            },
            VillageCode {
                name: "奥尔塔阿尔格勒泰村委会",
                code: "003",
            },
            VillageCode {
                name: "塔斯塔克村委会",
                code: "004",
            },
            VillageCode {
                name: "克孜勒库都克村委会",
                code: "005",
            },
            VillageCode {
                name: "白杨沟村委会",
                code: "006",
            },
            VillageCode {
                name: "喀拉通克村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "杜热镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "阔克布拉克村委会",
                code: "001",
            },
            VillageCode {
                name: "大坝村委会",
                code: "002",
            },
            VillageCode {
                name: "乌亚勒铁热克村委会",
                code: "003",
            },
            VillageCode {
                name: "杜热村委会",
                code: "004",
            },
            VillageCode {
                name: "克孜勒加尔村委会",
                code: "005",
            },
            VillageCode {
                name: "索依勒特村委会",
                code: "006",
            },
            VillageCode {
                name: "玉什克日什村委会",
                code: "007",
            },
            VillageCode {
                name: "胡吉尔特村委会",
                code: "008",
            },
            VillageCode {
                name: "铁斯甫阿坎村委会",
                code: "009",
            },
            VillageCode {
                name: "乌扎合特村委会",
                code: "010",
            },
            VillageCode {
                name: "有色村委会",
                code: "011",
            },
            VillageCode {
                name: "蒙库村委会",
                code: "012",
            },
            VillageCode {
                name: "金宝村委会",
                code: "013",
            },
            VillageCode {
                name: "窝依阔拉移民新村委会",
                code: "014",
            },
            VillageCode {
                name: "阿合库木斯村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "吐尔洪乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "霍孜克村委会",
                code: "001",
            },
            VillageCode {
                name: "达尔肯村委会",
                code: "002",
            },
            VillageCode {
                name: "吉格尔拜村委会",
                code: "003",
            },
            VillageCode {
                name: "喀拉奥依村委会",
                code: "004",
            },
            VillageCode {
                name: "塔斯托别村委会",
                code: "005",
            },
            VillageCode {
                name: "康阔勒特克村委会",
                code: "006",
            },
            VillageCode {
                name: "托普铁列克村委会",
                code: "007",
            },
            VillageCode {
                name: "吐尔洪村委会",
                code: "008",
            },
            VillageCode {
                name: "拜依格托别村委会",
                code: "009",
            },
            VillageCode {
                name: "阔克铁列克村委会",
                code: "010",
            },
            VillageCode {
                name: "克孜勒塔斯村委会",
                code: "011",
            },
            VillageCode {
                name: "喀拉吉拉村委会",
                code: "012",
            },
            VillageCode {
                name: "乌亚拜村委会",
                code: "013",
            },
            VillageCode {
                name: "霍斯阿热勒村委会",
                code: "014",
            },
            VillageCode {
                name: "阿克哈仁村委会",
                code: "015",
            },
            VillageCode {
                name: "阔克塔勒村委会",
                code: "016",
            },
            VillageCode {
                name: "托留拜克孜勒村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "库尔特乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "阿舍勒村委会",
                code: "001",
            },
            VillageCode {
                name: "库尔特村委会",
                code: "002",
            },
            VillageCode {
                name: "喀拉巴盖村委会",
                code: "003",
            },
            VillageCode {
                name: "苏普特村委会",
                code: "004",
            },
            VillageCode {
                name: "达拉阿吾孜村委会",
                code: "005",
            },
            VillageCode {
                name: "萨尔巴斯村委会",
                code: "006",
            },
            VillageCode {
                name: "温都尔哈拉村委会",
                code: "007",
            },
            VillageCode {
                name: "布拉特村委会",
                code: "008",
            },
            VillageCode {
                name: "曲克尔特村委会",
                code: "009",
            },
            VillageCode {
                name: "吉拉特村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "克孜勒希力克乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "萨尔托海村委会",
                code: "001",
            },
            VillageCode {
                name: "江喀拉村委会",
                code: "002",
            },
            VillageCode {
                name: "叶格孜托别村委会",
                code: "003",
            },
            VillageCode {
                name: "阿合加尔村委会",
                code: "004",
            },
            VillageCode {
                name: "喀拉吉格特村委会",
                code: "005",
            },
            VillageCode {
                name: "阔协萨依村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "铁买克乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "铁买克村委会",
                code: "001",
            },
            VillageCode {
                name: "喀依尔特村委会",
                code: "002",
            },
            VillageCode {
                name: "都孜拜村委会",
                code: "003",
            },
            VillageCode {
                name: "萨尔铁列克村委会",
                code: "004",
            },
            VillageCode {
                name: "铁买克新村委会",
                code: "005",
            },
            VillageCode {
                name: "海子口村委会",
                code: "006",
            },
            VillageCode {
                name: "宏泰新村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "喀拉布勒根乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "喀拉塔斯村委会",
                code: "001",
            },
            VillageCode {
                name: "喀拉苏村委会",
                code: "002",
            },
            VillageCode {
                name: "喀拉卓勒村委会",
                code: "003",
            },
            VillageCode {
                name: "阔克铁列克村委会",
                code: "004",
            },
            VillageCode {
                name: "巴拉额尔齐斯村委会",
                code: "005",
            },
            VillageCode {
                name: "吉别特村委会",
                code: "006",
            },
            VillageCode {
                name: "加木克村委会",
                code: "007",
            },
            VillageCode {
                name: "喀勒恰海村委会",
                code: "008",
            },
            VillageCode {
                name: "霍斯阔拉村委会",
                code: "009",
            },
        ],
    },
];

static TOWNS_XJ_034: [TownCode; 11] = [
    TownCode {
        name: "福海镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "人民路居委会",
                code: "001",
            },
            VillageCode {
                name: "永安路居委会",
                code: "002",
            },
            VillageCode {
                name: "济海路居委会",
                code: "003",
            },
            VillageCode {
                name: "建北路居委会",
                code: "004",
            },
            VillageCode {
                name: "环城路居委会",
                code: "005",
            },
            VillageCode {
                name: "赫勒居委会",
                code: "006",
            },
            VillageCode {
                name: "西城区社区",
                code: "007",
            },
            VillageCode {
                name: "东城区社区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "喀拉玛盖镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "萨尔塔勒村委会",
                code: "001",
            },
            VillageCode {
                name: "阿克科热什村委会",
                code: "002",
            },
            VillageCode {
                name: "喀拉霍英村委会",
                code: "003",
            },
            VillageCode {
                name: "阿克阿热勒村委会",
                code: "004",
            },
            VillageCode {
                name: "喀尔乌提克勒村委会",
                code: "005",
            },
            VillageCode {
                name: "克孜勒乌英克村委会",
                code: "006",
            },
            VillageCode {
                name: "别斯铁热克村委会",
                code: "007",
            },
            VillageCode {
                name: "多尔布力金村委会",
                code: "008",
            },
            VillageCode {
                name: "萨尔库萨克村委会",
                code: "009",
            },
            VillageCode {
                name: "吉迭勒村委会",
                code: "010",
            },
            VillageCode {
                name: "唐巴勒村委会",
                code: "011",
            },
            VillageCode {
                name: "开勒铁开村委会",
                code: "012",
            },
            VillageCode {
                name: "窝依阔拉村委会",
                code: "013",
            },
            VillageCode {
                name: "萨尔库木村委会",
                code: "014",
            },
            VillageCode {
                name: "喀拉苏村委会",
                code: "015",
            },
            VillageCode {
                name: "迭恩村委会",
                code: "016",
            },
            VillageCode {
                name: "喀拉布依拉村",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "解特阿热勒镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "别斯朱勒德孜村委会",
                code: "001",
            },
            VillageCode {
                name: "阔聂科买村委会",
                code: "002",
            },
            VillageCode {
                name: "京什开萨尔村委会",
                code: "003",
            },
            VillageCode {
                name: "阔克铁热克村委会",
                code: "004",
            },
            VillageCode {
                name: "桑孜拜阔克铁列克村委会",
                code: "005",
            },
            VillageCode {
                name: "喀拉毕村委会",
                code: "006",
            },
            VillageCode {
                name: "托夏勒乌依村委会",
                code: "007",
            },
            VillageCode {
                name: "林业村委会",
                code: "008",
            },
            VillageCode {
                name: "喀拉塔合尔村委会",
                code: "009",
            },
            VillageCode {
                name: "博孜塔勒村委会",
                code: "010",
            },
            VillageCode {
                name: "萨尔塔合塔依村委会",
                code: "011",
            },
            VillageCode {
                name: "阿勒尕村委会",
                code: "012",
            },
            VillageCode {
                name: "阿勒玛巴克村委会",
                code: "013",
            },
            VillageCode {
                name: "解特阿热勒村委会",
                code: "014",
            },
            VillageCode {
                name: "喀拉铁热斯肯村委会",
                code: "015",
            },
            VillageCode {
                name: "博塔莫因村委会",
                code: "016",
            },
            VillageCode {
                name: "萨尔胡木村委会",
                code: "017",
            },
            VillageCode {
                name: "加勒帕克村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "阔克阿尕什乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "阔甫别尔根村委会",
                code: "001",
            },
            VillageCode {
                name: "阿克乌提克村委会",
                code: "002",
            },
            VillageCode {
                name: "萨尔哈木斯村委会",
                code: "003",
            },
            VillageCode {
                name: "别斯克拜村委会",
                code: "004",
            },
            VillageCode {
                name: "阔克阿尕什村委会",
                code: "005",
            },
            VillageCode {
                name: "卓勒吐斯坎村委会",
                code: "006",
            },
            VillageCode {
                name: "也斯克库木村委会",
                code: "007",
            },
            VillageCode {
                name: "喀拉塔合尔村委会",
                code: "008",
            },
            VillageCode {
                name: "阔普霍拉村委会",
                code: "009",
            },
            VillageCode {
                name: "都孜根德村委会",
                code: "010",
            },
            VillageCode {
                name: "齐巴尔窝依村委会",
                code: "011",
            },
            VillageCode {
                name: "阔克卓尔尕村委会",
                code: "012",
            },
            VillageCode {
                name: "布尔勒克村委会",
                code: "013",
            },
            VillageCode {
                name: "齐勒喀仁村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "齐干吉迭乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "加郎阿什村委会",
                code: "001",
            },
            VillageCode {
                name: "克孜勒乌英克村委会",
                code: "002",
            },
            VillageCode {
                name: "阿克阿热勒村委会",
                code: "003",
            },
            VillageCode {
                name: "博列克托别村委会",
                code: "004",
            },
            VillageCode {
                name: "齐干吉迭村委会",
                code: "005",
            },
            VillageCode {
                name: "赛克露村委会",
                code: "006",
            },
            VillageCode {
                name: "克孜勒克热什村委会",
                code: "007",
            },
            VillageCode {
                name: "阿斯涛夏村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "阿尔达乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "干河子一村委会",
                code: "001",
            },
            VillageCode {
                name: "干河子二村委会",
                code: "002",
            },
            VillageCode {
                name: "干河子三村委会",
                code: "003",
            },
            VillageCode {
                name: "干河子四村委会",
                code: "004",
            },
            VillageCode {
                name: "阿尔达村委会",
                code: "005",
            },
            VillageCode {
                name: "阿克哲拉村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "地区一农场",
        code: "007",
        villages: &[
            VillageCode {
                name: "场部生活区",
                code: "001",
            },
            VillageCode {
                name: "一分场生活区",
                code: "002",
            },
            VillageCode {
                name: "二分场生活区",
                code: "003",
            },
            VillageCode {
                name: "三分场生活区",
                code: "004",
            },
            VillageCode {
                name: "四分场生活区",
                code: "005",
            },
            VillageCode {
                name: "六分场生活区",
                code: "006",
            },
            VillageCode {
                name: "七分场生活区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "福海监狱",
        code: "008",
        villages: &[
            VillageCode {
                name: "场部社区",
                code: "001",
            },
            VillageCode {
                name: "二中队社区",
                code: "002",
            },
            VillageCode {
                name: "三中队社区",
                code: "003",
            },
            VillageCode {
                name: "四中队社区",
                code: "004",
            },
            VillageCode {
                name: "职工管理中心社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "兵团一八二团",
        code: "009",
        villages: &[
            VillageCode {
                name: "光明社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "兵团一八三团分部",
        code: "010",
        villages: &[
            VillageCode {
                name: "独立营社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "兵团一八八团分部",
        code: "011",
        villages: &[VillageCode {
            name: "五连生活区",
            code: "001",
        }],
    },
];

static TOWNS_XJ_035: [TownCode; 8] = [
    TownCode {
        name: "阿克齐镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "民主东路居委会",
                code: "001",
            },
            VillageCode {
                name: "民主中路居委会",
                code: "002",
            },
            VillageCode {
                name: "民主西路居委会",
                code: "003",
            },
            VillageCode {
                name: "解放中路居委会",
                code: "004",
            },
            VillageCode {
                name: "解放西路居委会",
                code: "005",
            },
            VillageCode {
                name: "长白山居委会",
                code: "006",
            },
            VillageCode {
                name: "建业东路居委会",
                code: "007",
            },
            VillageCode {
                name: "阿克齐村委会",
                code: "008",
            },
            VillageCode {
                name: "坎门尔村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "萨尔布拉克镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "萨尔布拉克村委会",
                code: "001",
            },
            VillageCode {
                name: "吐勒克勒村委会",
                code: "002",
            },
            VillageCode {
                name: "别列则克村委会",
                code: "003",
            },
            VillageCode {
                name: "科克托海村委会",
                code: "004",
            },
            VillageCode {
                name: "加郎阿什村委会",
                code: "005",
            },
            VillageCode {
                name: "玉什阿夏村委会",
                code: "006",
            },
            VillageCode {
                name: "喀拉翁格尔村委会",
                code: "007",
            },
            VillageCode {
                name: "加勒格孜阿尕什村委会",
                code: "008",
            },
            VillageCode {
                name: "阿勒喀别克克孜勒喀英村委会",
                code: "009",
            },
            VillageCode {
                name: "克孜勒珠勒都孜村委会",
                code: "010",
            },
            VillageCode {
                name: "农科村委会",
                code: "011",
            },
            VillageCode {
                name: "阿勒喀别克村委会",
                code: "012",
            },
            VillageCode {
                name: "库勒拜吐勒克勒村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "齐巴尔镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "阔克苏村委会",
                code: "001",
            },
            VillageCode {
                name: "阿依达尔乌英克村委会",
                code: "002",
            },
            VillageCode {
                name: "艾林阿克齐村委会",
                code: "003",
            },
            VillageCode {
                name: "喀拉塔勒村委会",
                code: "004",
            },
            VillageCode {
                name: "克孜勒加尔村委会",
                code: "005",
            },
            VillageCode {
                name: "齐巴尔克孜勒喀英村委会",
                code: "006",
            },
            VillageCode {
                name: "阔斯阿热勒村委会",
                code: "007",
            },
            VillageCode {
                name: "齐巴尔村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "库勒拜镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "姜居勒克村委会",
                code: "001",
            },
            VillageCode {
                name: "萨尔塔克太村委会",
                code: "002",
            },
            VillageCode {
                name: "库勒拜喀拉阔布村委会",
                code: "003",
            },
            VillageCode {
                name: "巴勒塔村委会",
                code: "004",
            },
            VillageCode {
                name: "喀拉希力克村委会",
                code: "005",
            },
            VillageCode {
                name: "塔斯喀拉村委会",
                code: "006",
            },
            VillageCode {
                name: "喀尔乌特克勒村委会",
                code: "007",
            },
            VillageCode {
                name: "喀英德阿热勒村委会",
                code: "008",
            },
            VillageCode {
                name: "库勒拜阔斯阿热勒村委会",
                code: "009",
            },
            VillageCode {
                name: "库勒拜阿克齐村委会",
                code: "010",
            },
            VillageCode {
                name: "喀拉布拉克村委会",
                code: "011",
            },
            VillageCode {
                name: "吾什托别村委会",
                code: "012",
            },
            VillageCode {
                name: "那勒村委会",
                code: "013",
            },
            VillageCode {
                name: "四十一公里村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "萨尔塔木乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "萨尔塔木村委会",
                code: "001",
            },
            VillageCode {
                name: "萨尔塔木阿克托别村委会",
                code: "002",
            },
            VillageCode {
                name: "库尔米希村委会",
                code: "003",
            },
            VillageCode {
                name: "铁克吐尔玛斯村委会",
                code: "004",
            },
            VillageCode {
                name: "克依克拜村委会",
                code: "005",
            },
            VillageCode {
                name: "喀拉奥依村委会",
                code: "006",
            },
            VillageCode {
                name: "却限村委会",
                code: "007",
            },
            VillageCode {
                name: "萨尔乌楞村委会",
                code: "008",
            },
            VillageCode {
                name: "阔尔合热玛村委会",
                code: "009",
            },
            VillageCode {
                name: "塔依索依干村委会",
                code: "010",
            },
            VillageCode {
                name: "喀布尔喀塔勒村委会",
                code: "011",
            },
            VillageCode {
                name: "马胡村委会",
                code: "012",
            },
            VillageCode {
                name: "阔克塔斯村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "加依勒玛乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "加依勒玛村委会",
                code: "001",
            },
            VillageCode {
                name: "玛依沙斛村委会",
                code: "002",
            },
            VillageCode {
                name: "阿克托别村委会",
                code: "003",
            },
            VillageCode {
                name: "切格斯加依勒玛村委会",
                code: "004",
            },
            VillageCode {
                name: "克勒迭能村委会",
                code: "005",
            },
            VillageCode {
                name: "沃尔塔喀布尔尕塔勒村委会",
                code: "006",
            },
            VillageCode {
                name: "克尔达拉村委会",
                code: "007",
            },
            VillageCode {
                name: "博旦拜村委会",
                code: "008",
            },
            VillageCode {
                name: "喀拉阿尕什村委会",
                code: "009",
            },
            VillageCode {
                name: "阔克萨孜村委会",
                code: "010",
            },
            VillageCode {
                name: "拜格托别村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "铁热克提乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "铁热克提村委会",
                code: "001",
            },
            VillageCode {
                name: "齐巴尔希力克村委会",
                code: "002",
            },
            VillageCode {
                name: "阿克布拉克村委会",
                code: "003",
            },
            VillageCode {
                name: "白哈巴村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "兵团一八五团",
        code: "008",
        villages: &[
            VillageCode {
                name: "团部社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
        ],
    },
];

static TOWNS_XJ_036: [TownCode; 8] = [
    TownCode {
        name: "青河镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "花海子居委会",
                code: "001",
            },
            VillageCode {
                name: "白桦林居委会",
                code: "002",
            },
            VillageCode {
                name: "山楂园居委会",
                code: "003",
            },
            VillageCode {
                name: "青龙湖居委会",
                code: "004",
            },
            VillageCode {
                name: "青格里居委会",
                code: "005",
            },
            VillageCode {
                name: "阿克朗克村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "塔克什肯镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "友好路居委会",
                code: "001",
            },
            VillageCode {
                name: "迎宾路居委会",
                code: "002",
            },
            VillageCode {
                name: "萨尔布拉克村委会",
                code: "003",
            },
            VillageCode {
                name: "依希根村委会",
                code: "004",
            },
            VillageCode {
                name: "阿克喀仁村委会",
                code: "005",
            },
            VillageCode {
                name: "蒙其克村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "阿热勒托别镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "克孜勒萨依村委会",
                code: "001",
            },
            VillageCode {
                name: "克孜勒希力克村委会",
                code: "002",
            },
            VillageCode {
                name: "煤矿村委会",
                code: "003",
            },
            VillageCode {
                name: "科克托别村委会",
                code: "004",
            },
            VillageCode {
                name: "阿亚克阿克哈仁村委会",
                code: "005",
            },
            VillageCode {
                name: "阔斯阿热勒村委会",
                code: "006",
            },
            VillageCode {
                name: "喀拉尕什村委会",
                code: "007",
            },
            VillageCode {
                name: "科克塔斯村委会",
                code: "008",
            },
            VillageCode {
                name: "巴斯克阿克哈仁村委会",
                code: "009",
            },
            VillageCode {
                name: "乔什喀吐别克村委会",
                code: "010",
            },
            VillageCode {
                name: "喀拉沃楞村委会",
                code: "011",
            },
            VillageCode {
                name: "喀依尔恒村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "阿格达拉镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "和平社区",
                code: "001",
            },
            VillageCode {
                name: "创业社区",
                code: "002",
            },
            VillageCode {
                name: "新牧社区",
                code: "003",
            },
            VillageCode {
                name: "阿格达拉村",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "阿热勒镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "乔夏村委会",
                code: "001",
            },
            VillageCode {
                name: "呼尔森村委会",
                code: "002",
            },
            VillageCode {
                name: "库尔迭宁村委会",
                code: "003",
            },
            VillageCode {
                name: "塔拉特村委会",
                code: "004",
            },
            VillageCode {
                name: "阔布村委会",
                code: "005",
            },
            VillageCode {
                name: "布鲁克村委会",
                code: "006",
            },
            VillageCode {
                name: "拉斯特村委会",
                code: "007",
            },
            VillageCode {
                name: "杜尔根村委会",
                code: "008",
            },
            VillageCode {
                name: "喀让格托海村委会",
                code: "009",
            },
            VillageCode {
                name: "肯莫依纳克村委会",
                code: "010",
            },
            VillageCode {
                name: "冬特村委会",
                code: "011",
            },
            VillageCode {
                name: "达巴特村委会",
                code: "012",
            },
            VillageCode {
                name: "布河坝村委会",
                code: "013",
            },
            VillageCode {
                name: "塔斯托别村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "萨尔托海乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "萨尔托海村委会",
                code: "001",
            },
            VillageCode {
                name: "喀拉乔拉村委会",
                code: "002",
            },
            VillageCode {
                name: "克孜勒玉永克村委会",
                code: "003",
            },
            VillageCode {
                name: "萨尔喀仁村委会",
                code: "004",
            },
            VillageCode {
                name: "玉依塔斯村委会",
                code: "005",
            },
            VillageCode {
                name: "别斯铁热克村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "查干郭勒乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "加勒特尔塔斯村委会",
                code: "001",
            },
            VillageCode {
                name: "科克玉依村委会",
                code: "002",
            },
            VillageCode {
                name: "博塔莫音村委会",
                code: "003",
            },
            VillageCode {
                name: "江布塔斯村委会",
                code: "004",
            },
            VillageCode {
                name: "沙尔布拉克村委会",
                code: "005",
            },
            VillageCode {
                name: "克孜勒萨依村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "阿尕什敖包乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "阿尕什敖包村委会",
                code: "001",
            },
            VillageCode {
                name: "库伦托别村委会",
                code: "002",
            },
            VillageCode {
                name: "加热克努尔村委会",
                code: "003",
            },
            VillageCode {
                name: "唐巴玉孜尔村委会",
                code: "004",
            },
            VillageCode {
                name: "库木喀仁村委会",
                code: "005",
            },
            VillageCode {
                name: "夏尔克塔斯村委会",
                code: "006",
            },
            VillageCode {
                name: "阿克加尔村委会",
                code: "007",
            },
        ],
    },
];

static TOWNS_XJ_037: [TownCode; 8] = [
    TownCode {
        name: "托普铁热克镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "文明路居委会",
                code: "001",
            },
            VillageCode {
                name: "建设路居委会",
                code: "002",
            },
            VillageCode {
                name: "团结路居委会",
                code: "003",
            },
            VillageCode {
                name: "人民路居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "吉木乃镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "别勒阿热克村委会",
                code: "001",
            },
            VillageCode {
                name: "夏尔合特村委会",
                code: "002",
            },
            VillageCode {
                name: "萨尔乌楞村委会",
                code: "003",
            },
            VillageCode {
                name: "托盘村委会",
                code: "004",
            },
            VillageCode {
                name: "克孜勒阿德尔村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "喀尔交镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "喀尔交村委会",
                code: "001",
            },
            VillageCode {
                name: "喀拉吉拉村委会",
                code: "002",
            },
            VillageCode {
                name: "萨尔布拉克村委会",
                code: "003",
            },
            VillageCode {
                name: "阔克阔拉村委会",
                code: "004",
            },
            VillageCode {
                name: "布尔合斯太村委会",
                code: "005",
            },
            VillageCode {
                name: "克孜勒阔拉村委会",
                code: "006",
            },
            VillageCode {
                name: "萨帕克村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "乌拉斯特镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "强德珠尔特村委会",
                code: "001",
            },
            VillageCode {
                name: "萨尔塔木村委会",
                code: "002",
            },
            VillageCode {
                name: "阔克托干木村委会",
                code: "003",
            },
            VillageCode {
                name: "喀拉苏村委会",
                code: "004",
            },
            VillageCode {
                name: "阿尔恰勒村委会",
                code: "005",
            },
            VillageCode {
                name: "播尔克塔勒村委会",
                code: "006",
            },
            VillageCode {
                name: "巴依古西克村委会",
                code: "007",
            },
            VillageCode {
                name: "阔克舍木村委会",
                code: "008",
            },
            VillageCode {
                name: "巴特巴克布拉克村委会",
                code: "009",
            },
            VillageCode {
                name: "阿克加尔村委会",
                code: "010",
            },
            VillageCode {
                name: "克孜勒加尔村委会",
                code: "011",
            },
            VillageCode {
                name: "齐阔尔加村委会",
                code: "012",
            },
            VillageCode {
                name: "吐尕力阿尕什村委会",
                code: "013",
            },
            VillageCode {
                name: "乌拉斯特村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "托斯特乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "章阿托干村委会",
                code: "001",
            },
            VillageCode {
                name: "塔斯特村委会",
                code: "002",
            },
            VillageCode {
                name: "托斯特村委会",
                code: "003",
            },
            VillageCode {
                name: "巴扎尔胡勒村委会",
                code: "004",
            },
            VillageCode {
                name: "阔依塔斯村委会",
                code: "005",
            },
            VillageCode {
                name: "喀拉乔克村委会",
                code: "006",
            },
            VillageCode {
                name: "阿克阔勒吐克村委会",
                code: "007",
            },
            VillageCode {
                name: "森塔斯村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "恰勒什海乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "达冷海齐村委会",
                code: "001",
            },
            VillageCode {
                name: "阿克木尔扎村委会",
                code: "002",
            },
            VillageCode {
                name: "阔尔加村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "别斯铁热克乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "奥夏尔拜村委会",
                code: "001",
            },
            VillageCode {
                name: "库早齐村委会",
                code: "002",
            },
            VillageCode {
                name: "萨尔阿根村委会",
                code: "003",
            },
            VillageCode {
                name: "加勒格孜喀拉尕依村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "兵团一八六团",
        code: "008",
        villages: &[
            VillageCode {
                name: "一八六团团部社区",
                code: "001",
            },
            VillageCode {
                name: "一八六团一连生活区",
                code: "002",
            },
            VillageCode {
                name: "一八六团二连生活区",
                code: "003",
            },
            VillageCode {
                name: "一八六团三连生活区",
                code: "004",
            },
            VillageCode {
                name: "一八六团四连生活区",
                code: "005",
            },
            VillageCode {
                name: "一八六团五连生活区",
                code: "006",
            },
            VillageCode {
                name: "一八六团六连生活区",
                code: "007",
            },
        ],
    },
];

static TOWNS_XJ_038: [TownCode; 8] = [
    TownCode {
        name: "新城街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "七小区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "十四小区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "十六小区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "十七小区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "工二三小区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "九小区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "八小区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "十五小区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "南山社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "向阳街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "一小区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "二十小区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "二十一小区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "二十二小区第一社区居委会",
                code: "004",
            },
            VillageCode {
                name: "二十二小区第二社区居委会",
                code: "005",
            },
            VillageCode {
                name: "二十三小区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "三十一小区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "北龙社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "红山街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "三小区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "二十四小区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "二十五小区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "二十七小区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "三十三小区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "三十四小区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "四十二小区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "火车站小区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "四小区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "二十六小区社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "老街街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "五小区第三社区居委会",
                code: "001",
            },
            VillageCode {
                name: "六小区第一社区居委会",
                code: "002",
            },
            VillageCode {
                name: "六小区第二社区居委会",
                code: "003",
            },
            VillageCode {
                name: "六小区第三社区居委会",
                code: "004",
            },
            VillageCode {
                name: "十一小区第一社区居委会",
                code: "005",
            },
            VillageCode {
                name: "十一小区第二社区居委会",
                code: "006",
            },
            VillageCode {
                name: "十三小区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "十二小区社区第一居委会",
                code: "008",
            },
            VillageCode {
                name: "十二小区社区第二居委会",
                code: "009",
            },
            VillageCode {
                name: "望月坪社区居委会",
                code: "010",
            },
            VillageCode {
                name: "小白杨社区居委会",
                code: "011",
            },
            VillageCode {
                name: "团结路社区居委会",
                code: "012",
            },
            VillageCode {
                name: "五小区社区居委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "东城街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "四十小区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "五十六小区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "六十三小区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "四十八小区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "七十八小区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "明珠社区居委会",
                code: "006",
            },
            VillageCode {
                name: "三十九小区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "四十一小区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "凤凰嘉苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "五十八小区社区居委会",
                code: "010",
            },
            VillageCode {
                name: "三十八小区社区居委会",
                code: "011",
            },
            VillageCode {
                name: "四十三小区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "五十二小区社区居委会",
                code: "013",
            },
            VillageCode {
                name: "北七小区社区居委会",
                code: "014",
            },
            VillageCode {
                name: "大庙社区居委会",
                code: "015",
            },
            VillageCode {
                name: "山丹湖社区居委会",
                code: "016",
            },
            VillageCode {
                name: "马家坪社区居委会",
                code: "017",
            },
            VillageCode {
                name: "河畔社区居委会",
                code: "018",
            },
            VillageCode {
                name: "五十三小区社区居委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "北泉镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "文化宫社区居委会",
                code: "001",
            },
            VillageCode {
                name: "花园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "军垦社区居委会",
                code: "003",
            },
            VillageCode {
                name: "纪念碑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "小林场社区居委会",
                code: "005",
            },
            VillageCode {
                name: "白杨社区居委会",
                code: "006",
            },
            VillageCode {
                name: "阳光社区居委会",
                code: "007",
            },
            VillageCode {
                name: "银泉社区居委会",
                code: "008",
            },
            VillageCode {
                name: "龙福泉社区居委会",
                code: "009",
            },
            VillageCode {
                name: "明珠社区居委会",
                code: "010",
            },
            VillageCode {
                name: "工业新村社区居委会",
                code: "011",
            },
            VillageCode {
                name: "大泉沟村委会",
                code: "012",
            },
            VillageCode {
                name: "清泉集十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "清泉集社区村委会",
                code: "014",
            },
            VillageCode {
                name: "清泉集一连生活区",
                code: "015",
            },
            VillageCode {
                name: "清泉集二连生活区",
                code: "016",
            },
            VillageCode {
                name: "清泉集三连生活区",
                code: "017",
            },
            VillageCode {
                name: "清泉集四连生活区",
                code: "018",
            },
            VillageCode {
                name: "清泉集五连生活区",
                code: "019",
            },
            VillageCode {
                name: "清泉集六连生活区",
                code: "020",
            },
            VillageCode {
                name: "清泉集七连生活区",
                code: "021",
            },
            VillageCode {
                name: "清泉集八连生活区",
                code: "022",
            },
            VillageCode {
                name: "清泉集九连生活区",
                code: "023",
            },
            VillageCode {
                name: "清泉集十连生活区",
                code: "024",
            },
            VillageCode {
                name: "双泉集社区村委会",
                code: "025",
            },
            VillageCode {
                name: "双泉集一连生活区",
                code: "026",
            },
            VillageCode {
                name: "双泉集二连生活区",
                code: "027",
            },
            VillageCode {
                name: "双泉集四连生活区",
                code: "028",
            },
            VillageCode {
                name: "双泉集五连生活区",
                code: "029",
            },
            VillageCode {
                name: "双泉集六连生活区",
                code: "030",
            },
            VillageCode {
                name: "畜牧公司村委会",
                code: "031",
            },
            VillageCode {
                name: "石大实验场村委会",
                code: "032",
            },
            VillageCode {
                name: "石河子总场双泉集七连生活区",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "石河子镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "镇居委会",
                code: "001",
            },
            VillageCode {
                name: "十户窑村委会",
                code: "002",
            },
            VillageCode {
                name: "三宫村委会",
                code: "003",
            },
            VillageCode {
                name: "努尔巴克村委会",
                code: "004",
            },
            VillageCode {
                name: "四宫村委会",
                code: "005",
            },
            VillageCode {
                name: "霍斯阿尔克村委会",
                code: "006",
            },
            VillageCode {
                name: "三十户村委会",
                code: "007",
            },
            VillageCode {
                name: "沙依巴克村委会",
                code: "008",
            },
            VillageCode {
                name: "袁家沟村委会",
                code: "009",
            },
            VillageCode {
                name: "五宫村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "兵团一五二团",
        code: "008",
        villages: &[
            VillageCode {
                name: "城南嘉苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南湾新苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "团部社区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "十连生活区",
                code: "008",
            },
        ],
    },
];

static TOWNS_XJ_039: [TownCode; 20] = [
    TownCode {
        name: "金银川路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "南公园社区",
                code: "001",
            },
            VillageCode {
                name: "绿园社区",
                code: "002",
            },
            VillageCode {
                name: "胡杨社区",
                code: "003",
            },
            VillageCode {
                name: "春晖社区",
                code: "004",
            },
            VillageCode {
                name: "滨水社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "幸福路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "花园社区",
                code: "001",
            },
            VillageCode {
                name: "新苑社区",
                code: "002",
            },
            VillageCode {
                name: "桃园社区",
                code: "003",
            },
            VillageCode {
                name: "学苑社区",
                code: "004",
            },
            VillageCode {
                name: "大学社区",
                code: "005",
            },
            VillageCode {
                name: "北苑社区",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "青松路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "阳光社区",
                code: "001",
            },
            VillageCode {
                name: "塔河社区",
                code: "002",
            },
            VillageCode {
                name: "腾飞社区",
                code: "003",
            },
            VillageCode {
                name: "安居社区",
                code: "004",
            },
            VillageCode {
                name: "迎宾社区",
                code: "005",
            },
            VillageCode {
                name: "建业社区",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "金银川镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "光明路社区",
                code: "001",
            },
            VillageCode {
                name: "胜利路社区",
                code: "002",
            },
            VillageCode {
                name: "希望路社区",
                code: "003",
            },
            VillageCode {
                name: "新皇宫社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "六连生活区",
                code: "010",
            },
            VillageCode {
                name: "七连生活区",
                code: "011",
            },
            VillageCode {
                name: "八连生活区",
                code: "012",
            },
            VillageCode {
                name: "九连生活区",
                code: "013",
            },
            VillageCode {
                name: "十连生活区",
                code: "014",
            },
            VillageCode {
                name: "十二连生活区",
                code: "015",
            },
            VillageCode {
                name: "十三连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十五连生活区",
                code: "018",
            },
            VillageCode {
                name: "十七连生活区",
                code: "019",
            },
            VillageCode {
                name: "十八连生活区",
                code: "020",
            },
            VillageCode {
                name: "十九连生活区",
                code: "021",
            },
            VillageCode {
                name: "二十连生活区",
                code: "022",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "023",
            },
            VillageCode {
                name: "二十四连生活区",
                code: "024",
            },
            VillageCode {
                name: "二十五连生活区",
                code: "025",
            },
            VillageCode {
                name: "二十六连生活区",
                code: "026",
            },
            VillageCode {
                name: "十一连生活区",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "新井子镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "西城社区",
                code: "001",
            },
            VillageCode {
                name: "东城社区",
                code: "002",
            },
            VillageCode {
                name: "沙井子社区",
                code: "003",
            },
            VillageCode {
                name: "一连生活区",
                code: "004",
            },
            VillageCode {
                name: "二连生活区",
                code: "005",
            },
            VillageCode {
                name: "三连生活区",
                code: "006",
            },
            VillageCode {
                name: "四连生活区",
                code: "007",
            },
            VillageCode {
                name: "五连生活区",
                code: "008",
            },
            VillageCode {
                name: "六连生活区",
                code: "009",
            },
            VillageCode {
                name: "七连生活区",
                code: "010",
            },
            VillageCode {
                name: "八连生活区",
                code: "011",
            },
            VillageCode {
                name: "九连生活区",
                code: "012",
            },
            VillageCode {
                name: "十连生活区",
                code: "013",
            },
            VillageCode {
                name: "十一连生活区",
                code: "014",
            },
            VillageCode {
                name: "十二连生活区",
                code: "015",
            },
            VillageCode {
                name: "十三连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十五连生活区",
                code: "018",
            },
            VillageCode {
                name: "十六连生活区",
                code: "019",
            },
            VillageCode {
                name: "十七连生活区",
                code: "020",
            },
            VillageCode {
                name: "十八连生活区",
                code: "021",
            },
            VillageCode {
                name: "十九连生活区",
                code: "022",
            },
            VillageCode {
                name: "二十连生活区",
                code: "023",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "024",
            },
            VillageCode {
                name: "二十二连生活区",
                code: "025",
            },
            VillageCode {
                name: "二十三连生活区",
                code: "026",
            },
            VillageCode {
                name: "二十四连生活区",
                code: "027",
            },
            VillageCode {
                name: "二十五连生活区",
                code: "028",
            },
            VillageCode {
                name: "二十六连生活区",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "甘泉镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "光明路社区",
                code: "001",
            },
            VillageCode {
                name: "文化路社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十二连生活区",
                code: "014",
            },
            VillageCode {
                name: "十三连生活区",
                code: "015",
            },
            VillageCode {
                name: "十四连生活区",
                code: "016",
            },
            VillageCode {
                name: "十五连生活区",
                code: "017",
            },
            VillageCode {
                name: "十六连生活区",
                code: "018",
            },
            VillageCode {
                name: "十七连生活区",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "永宁镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "人民路社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
            VillageCode {
                name: "九连生活区",
                code: "010",
            },
            VillageCode {
                name: "十连生活区",
                code: "011",
            },
            VillageCode {
                name: "十一连生活区",
                code: "012",
            },
            VillageCode {
                name: "十二连生活区",
                code: "013",
            },
            VillageCode {
                name: "一牛场生活区",
                code: "014",
            },
            VillageCode {
                name: "二牛场生活区",
                code: "015",
            },
            VillageCode {
                name: "三牛场生活区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "沙河镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "春风路社区",
                code: "001",
            },
            VillageCode {
                name: "幸福社区",
                code: "002",
            },
            VillageCode {
                name: "朝阳社区",
                code: "003",
            },
            VillageCode {
                name: "滨河社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "六连生活区",
                code: "010",
            },
            VillageCode {
                name: "七连生活区",
                code: "011",
            },
            VillageCode {
                name: "八连生活区",
                code: "012",
            },
            VillageCode {
                name: "九连生活区",
                code: "013",
            },
            VillageCode {
                name: "十连生活区",
                code: "014",
            },
            VillageCode {
                name: "十一连生活区",
                code: "015",
            },
            VillageCode {
                name: "十二连生活区",
                code: "016",
            },
            VillageCode {
                name: "十三连生活区",
                code: "017",
            },
            VillageCode {
                name: "十四连生活区",
                code: "018",
            },
            VillageCode {
                name: "十五连生活区",
                code: "019",
            },
            VillageCode {
                name: "十六连生活区",
                code: "020",
            },
            VillageCode {
                name: "养殖一场生活区",
                code: "021",
            },
            VillageCode {
                name: "养殖二场生活区",
                code: "022",
            },
            VillageCode {
                name: "养殖三场生活区",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "双城镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "幸福路社区",
                code: "001",
            },
            VillageCode {
                name: "迎宾路社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "花桥镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "昆岗社区",
                code: "001",
            },
            VillageCode {
                name: "花桥社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十三连生活区",
                code: "014",
            },
            VillageCode {
                name: "十四连生活区",
                code: "015",
            },
            VillageCode {
                name: "十五连生活区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "幸福镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "幸福路社区",
                code: "001",
            },
            VillageCode {
                name: "红桥社区",
                code: "002",
            },
            VillageCode {
                name: "健康东路社区",
                code: "003",
            },
            VillageCode {
                name: "南一路社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "六连生活区",
                code: "010",
            },
            VillageCode {
                name: "七连生活区",
                code: "011",
            },
            VillageCode {
                name: "八连生活区",
                code: "012",
            },
            VillageCode {
                name: "十连生活区",
                code: "013",
            },
            VillageCode {
                name: "十一连生活区",
                code: "014",
            },
            VillageCode {
                name: "十二连生活区",
                code: "015",
            },
            VillageCode {
                name: "十三连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十五连生活区",
                code: "018",
            },
            VillageCode {
                name: "十六连生活区",
                code: "019",
            },
            VillageCode {
                name: "十七连生活区",
                code: "020",
            },
            VillageCode {
                name: "十八连生活区",
                code: "021",
            },
            VillageCode {
                name: "十九连生活区",
                code: "022",
            },
            VillageCode {
                name: "二十连生活区",
                code: "023",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "024",
            },
            VillageCode {
                name: "二十二连生活区",
                code: "025",
            },
            VillageCode {
                name: "二十三连生活区",
                code: "026",
            },
            VillageCode {
                name: "二十六连生活区",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "金杨镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "阳光社区",
                code: "001",
            },
            VillageCode {
                name: "塞上明珠社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "玛滩镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "迎宾路社区",
                code: "001",
            },
            VillageCode {
                name: "友谊路社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "八连生活区",
                code: "008",
            },
            VillageCode {
                name: "九连生活区",
                code: "009",
            },
            VillageCode {
                name: "十一连生活区",
                code: "010",
            },
            VillageCode {
                name: "十三连生活区",
                code: "011",
            },
            VillageCode {
                name: "十四连生活区",
                code: "012",
            },
            VillageCode {
                name: "十五连生活区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "塔门镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "北京路社区",
                code: "001",
            },
            VillageCode {
                name: "台州路社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "梨花镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "如意社区",
                code: "001",
            },
            VillageCode {
                name: "绿园社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十二连生活区",
                code: "014",
            },
            VillageCode {
                name: "十三连生活区",
                code: "015",
            },
            VillageCode {
                name: "十四连生活区",
                code: "016",
            },
            VillageCode {
                name: "十五连生活区",
                code: "017",
            },
            VillageCode {
                name: "十六连生活区",
                code: "018",
            },
            VillageCode {
                name: "十七连生活区",
                code: "019",
            },
            VillageCode {
                name: "十八连生活区",
                code: "020",
            },
            VillageCode {
                name: "十九连生活区",
                code: "021",
            },
            VillageCode {
                name: "二十连生活区",
                code: "022",
            },
            VillageCode {
                name: "安置区一连生活区",
                code: "023",
            },
            VillageCode {
                name: "安置区二连生活区",
                code: "024",
            },
            VillageCode {
                name: "安置区三连生活区",
                code: "025",
            },
            VillageCode {
                name: "安置区四连生活区",
                code: "026",
            },
            VillageCode {
                name: "安置区五连生活区",
                code: "027",
            },
            VillageCode {
                name: "安置区六连生活区",
                code: "028",
            },
            VillageCode {
                name: "安置区七连生活区",
                code: "029",
            },
            VillageCode {
                name: "安置区八连生活区",
                code: "030",
            },
            VillageCode {
                name: "安置区九连生活区",
                code: "031",
            },
            VillageCode {
                name: "安置区十连生活区",
                code: "032",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "昌安镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "光明路社区",
                code: "001",
            },
            VillageCode {
                name: "迎宾路社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十二连生活区",
                code: "014",
            },
            VillageCode {
                name: "十三连生活区",
                code: "015",
            },
            VillageCode {
                name: "十四连生活区",
                code: "016",
            },
            VillageCode {
                name: "十五连生活区",
                code: "017",
            },
            VillageCode {
                name: "十六连生活区",
                code: "018",
            },
            VillageCode {
                name: "十七连生活区",
                code: "019",
            },
            VillageCode {
                name: "十八连生活区",
                code: "020",
            },
            VillageCode {
                name: "十九连生活区",
                code: "021",
            },
            VillageCode {
                name: "二十连生活区",
                code: "022",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "023",
            },
            VillageCode {
                name: "农二队生活区",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "塔南镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "文明路社区",
                code: "001",
            },
            VillageCode {
                name: "青年路社区",
                code: "002",
            },
            VillageCode {
                name: "台州新村社区",
                code: "003",
            },
            VillageCode {
                name: "南岸社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "六连生活区",
                code: "010",
            },
            VillageCode {
                name: "七连生活区",
                code: "011",
            },
            VillageCode {
                name: "八连生活区",
                code: "012",
            },
            VillageCode {
                name: "九连生活区",
                code: "013",
            },
            VillageCode {
                name: "十连生活区",
                code: "014",
            },
            VillageCode {
                name: "十一连生活区",
                code: "015",
            },
            VillageCode {
                name: "十二连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十五连生活区",
                code: "018",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "019",
            },
            VillageCode {
                name: "二十二连生活区",
                code: "020",
            },
            VillageCode {
                name: "二十三连生活区",
                code: "021",
            },
            VillageCode {
                name: "二十四连生活区",
                code: "022",
            },
            VillageCode {
                name: "二十六连生活区",
                code: "023",
            },
            VillageCode {
                name: "二十八连生活区",
                code: "024",
            },
            VillageCode {
                name: "二十九连生活区",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "新开岭镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "新开岭社区",
                code: "001",
            },
            VillageCode {
                name: "日月星光社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十二连生活区",
                code: "014",
            },
            VillageCode {
                name: "十三连生活区",
                code: "015",
            },
            VillageCode {
                name: "十四连生活区",
                code: "016",
            },
            VillageCode {
                name: "十五连生活区",
                code: "017",
            },
            VillageCode {
                name: "十六连生活区",
                code: "018",
            },
            VillageCode {
                name: "十七连生活区",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "托喀依乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "喀拉墩村委会",
                code: "001",
            },
            VillageCode {
                name: "纳格热哈纳村委会",
                code: "002",
            },
            VillageCode {
                name: "达利亚阿格孜村委会",
                code: "003",
            },
            VillageCode {
                name: "海勒克库都克村委会",
                code: "004",
            },
            VillageCode {
                name: "亚苏克村委会",
                code: "005",
            },
            VillageCode {
                name: "科克库勒村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "西工业园区",
        code: "020",
        villages: &[VillageCode {
            name: "西工业园区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_XJ_040: [TownCode; 17] = [
    TownCode {
        name: "锦绣街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "锦绣社区",
                code: "001",
            },
            VillageCode {
                name: "如意社区",
                code: "002",
            },
            VillageCode {
                name: "吉祥社区",
                code: "003",
            },
            VillageCode {
                name: "和谐社区",
                code: "004",
            },
            VillageCode {
                name: "团结社区",
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
                name: "宏福社区",
                code: "008",
            },
            VillageCode {
                name: "天福社区",
                code: "009",
            },
            VillageCode {
                name: "幸福社区",
                code: "010",
            },
            VillageCode {
                name: "东城社区",
                code: "011",
            },
            VillageCode {
                name: "安康社区",
                code: "012",
            },
            VillageCode {
                name: "玫瑰园社区",
                code: "013",
            },
            VillageCode {
                name: "大学城社区",
                code: "014",
            },
            VillageCode {
                name: "滨河社区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "前海街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "前海社区",
                code: "001",
            },
            VillageCode {
                name: "银花社区",
                code: "002",
            },
            VillageCode {
                name: "西城社区",
                code: "003",
            },
            VillageCode {
                name: "唐城社区",
                code: "004",
            },
            VillageCode {
                name: "祥和社区",
                code: "005",
            },
            VillageCode {
                name: "昆仑社区",
                code: "006",
            },
            VillageCode {
                name: "平安社区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "永安坝街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "永安坝社区",
                code: "001",
            },
            VillageCode {
                name: "达坂山社区",
                code: "002",
            },
            VillageCode {
                name: "建安社区",
                code: "003",
            },
            VillageCode {
                name: "杏花社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "草湖镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "白杨社区",
                code: "001",
            },
            VillageCode {
                name: "西湖社区",
                code: "002",
            },
            VillageCode {
                name: "红柳社区",
                code: "003",
            },
            VillageCode {
                name: "花园社区",
                code: "004",
            },
            VillageCode {
                name: "东湖社区",
                code: "005",
            },
            VillageCode {
                name: "一连生活区",
                code: "006",
            },
            VillageCode {
                name: "二连生活区",
                code: "007",
            },
            VillageCode {
                name: "三连生活区",
                code: "008",
            },
            VillageCode {
                name: "六连生活区",
                code: "009",
            },
            VillageCode {
                name: "七连生活区",
                code: "010",
            },
            VillageCode {
                name: "八连生活区",
                code: "011",
            },
            VillageCode {
                name: "九连生活区",
                code: "012",
            },
            VillageCode {
                name: "良种连生活区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "龙口镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "木华黎社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "前海镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "南湖社区",
                code: "001",
            },
            VillageCode {
                name: "东城社区",
                code: "002",
            },
            VillageCode {
                name: "沁园社区",
                code: "003",
            },
            VillageCode {
                name: "朝阳社区",
                code: "004",
            },
            VillageCode {
                name: "叶河浪花社区",
                code: "005",
            },
            VillageCode {
                name: "一连生活区",
                code: "006",
            },
            VillageCode {
                name: "二连生活区",
                code: "007",
            },
            VillageCode {
                name: "三连生活区",
                code: "008",
            },
            VillageCode {
                name: "四连生活区",
                code: "009",
            },
            VillageCode {
                name: "五连生活区",
                code: "010",
            },
            VillageCode {
                name: "六连生活区",
                code: "011",
            },
            VillageCode {
                name: "八连生活区",
                code: "012",
            },
            VillageCode {
                name: "九连生活区",
                code: "013",
            },
            VillageCode {
                name: "十连生活区",
                code: "014",
            },
            VillageCode {
                name: "十二连生活区",
                code: "015",
            },
            VillageCode {
                name: "十三连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十五连生活区",
                code: "018",
            },
            VillageCode {
                name: "十六连生活区",
                code: "019",
            },
            VillageCode {
                name: "十七连生活区",
                code: "020",
            },
            VillageCode {
                name: "十八连生活区",
                code: "021",
            },
            VillageCode {
                name: "十九连生活区",
                code: "022",
            },
            VillageCode {
                name: "二十连生活区",
                code: "023",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "024",
            },
            VillageCode {
                name: "二十二连生活区",
                code: "025",
            },
            VillageCode {
                name: "二十三连生活区",
                code: "026",
            },
            VillageCode {
                name: "二十四连生活区",
                code: "027",
            },
            VillageCode {
                name: "二十五连生活区",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "永兴镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "幸福社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "四连生活区",
                code: "004",
            },
            VillageCode {
                name: "五连生活区",
                code: "005",
            },
            VillageCode {
                name: "六连生活区",
                code: "006",
            },
            VillageCode {
                name: "七连生活区",
                code: "007",
            },
            VillageCode {
                name: "八连生活区",
                code: "008",
            },
            VillageCode {
                name: "九连生活区",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "兴安镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "兴安社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "嘉和镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "锦绣社区",
                code: "001",
            },
            VillageCode {
                name: "新粤社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十二连生活区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "河东镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "阳光社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "夏河镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "东其社区",
                code: "001",
            },
            VillageCode {
                name: "青湖社区",
                code: "002",
            },
            VillageCode {
                name: "夏河社区",
                code: "003",
            },
            VillageCode {
                name: "一连生活区",
                code: "004",
            },
            VillageCode {
                name: "二连生活区",
                code: "005",
            },
            VillageCode {
                name: "三连生活区",
                code: "006",
            },
            VillageCode {
                name: "四连生活区",
                code: "007",
            },
            VillageCode {
                name: "五连生活区",
                code: "008",
            },
            VillageCode {
                name: "六连生活区",
                code: "009",
            },
            VillageCode {
                name: "七连生活区",
                code: "010",
            },
            VillageCode {
                name: "八连生活区",
                code: "011",
            },
            VillageCode {
                name: "九连生活区",
                code: "012",
            },
            VillageCode {
                name: "十连生活区",
                code: "013",
            },
            VillageCode {
                name: "十一连生活区",
                code: "014",
            },
            VillageCode {
                name: "十二连生活区",
                code: "015",
            },
            VillageCode {
                name: "十三连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十五连生活区",
                code: "018",
            },
            VillageCode {
                name: "十六连生活区",
                code: "019",
            },
            VillageCode {
                name: "十七连生活区",
                code: "020",
            },
            VillageCode {
                name: "十八连生活区",
                code: "021",
            },
            VillageCode {
                name: "十九连生活区",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "永安镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "团部社区",
                code: "001",
            },
            VillageCode {
                name: "金墩社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十二连生活区",
                code: "014",
            },
            VillageCode {
                name: "十三连生活区",
                code: "015",
            },
            VillageCode {
                name: "十四连生活区",
                code: "016",
            },
            VillageCode {
                name: "十五连生活区",
                code: "017",
            },
            VillageCode {
                name: "十六连生活区",
                code: "018",
            },
            VillageCode {
                name: "十七连生活区",
                code: "019",
            },
            VillageCode {
                name: "十八连生活区",
                code: "020",
            },
            VillageCode {
                name: "二十连生活区",
                code: "021",
            },
            VillageCode {
                name: "原种场生活区",
                code: "022",
            },
            VillageCode {
                name: "值班连生活区",
                code: "023",
            },
            VillageCode {
                name: "良种连生活区",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "海安镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "文明社区",
                code: "001",
            },
            VillageCode {
                name: "东莞社区",
                code: "002",
            },
            VillageCode {
                name: "吉祥社区",
                code: "003",
            },
            VillageCode {
                name: "小海子社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "六连生活区",
                code: "009",
            },
            VillageCode {
                name: "七连生活区",
                code: "010",
            },
            VillageCode {
                name: "八连生活区",
                code: "011",
            },
            VillageCode {
                name: "九连生活区",
                code: "012",
            },
            VillageCode {
                name: "十连生活区",
                code: "013",
            },
            VillageCode {
                name: "十一连生活区",
                code: "014",
            },
            VillageCode {
                name: "十二连生活区",
                code: "015",
            },
            VillageCode {
                name: "十三连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十五连生活区",
                code: "018",
            },
            VillageCode {
                name: "十六连生活区",
                code: "019",
            },
            VillageCode {
                name: "十七连生活区",
                code: "020",
            },
            VillageCode {
                name: "十八连生活区",
                code: "021",
            },
            VillageCode {
                name: "十九连生活区",
                code: "022",
            },
            VillageCode {
                name: "二十连生活区",
                code: "023",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "唐驿镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "团结社区",
                code: "001",
            },
            VillageCode {
                name: "和谐社区",
                code: "002",
            },
            VillageCode {
                name: "美丽社区",
                code: "003",
            },
            VillageCode {
                name: "幸福社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "六连生活区",
                code: "010",
            },
            VillageCode {
                name: "八连生活区",
                code: "011",
            },
            VillageCode {
                name: "九连生活区",
                code: "012",
            },
            VillageCode {
                name: "十连生活区",
                code: "013",
            },
            VillageCode {
                name: "十二连生活区",
                code: "014",
            },
            VillageCode {
                name: "十三连生活区",
                code: "015",
            },
            VillageCode {
                name: "十四连生活区",
                code: "016",
            },
            VillageCode {
                name: "十五连生活区",
                code: "017",
            },
            VillageCode {
                name: "十六连生活区",
                code: "018",
            },
            VillageCode {
                name: "十八连生活区",
                code: "019",
            },
            VillageCode {
                name: "十九连生活区",
                code: "020",
            },
            VillageCode {
                name: "二十连生活区",
                code: "021",
            },
            VillageCode {
                name: "二十二连生活区",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "金胡杨镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "希望路社区",
                code: "001",
            },
            VillageCode {
                name: "西二路社区",
                code: "002",
            },
            VillageCode {
                name: "振兴路社区",
                code: "003",
            },
            VillageCode {
                name: "友好北路社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "六连生活区",
                code: "010",
            },
            VillageCode {
                name: "七连生活区",
                code: "011",
            },
            VillageCode {
                name: "八连生活区",
                code: "012",
            },
            VillageCode {
                name: "十七连生活区",
                code: "013",
            },
            VillageCode {
                name: "十九连生活区",
                code: "014",
            },
            VillageCode {
                name: "二十连生活区",
                code: "015",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "016",
            },
            VillageCode {
                name: "二十二连生活区",
                code: "017",
            },
            VillageCode {
                name: "二十三连生活区",
                code: "018",
            },
            VillageCode {
                name: "二十四连生活区",
                code: "019",
            },
            VillageCode {
                name: "良种连生活区",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "东风镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "景新社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "杏花镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "杏花社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "园林连生活区",
                code: "006",
            },
        ],
    },
];

static TOWNS_XJ_041: [TownCode; 6] = [
    TownCode {
        name: "军垦路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "军垦南路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "前进西街社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "振兴街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "向阳路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "天山北路社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "军垦北路社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "西林路社区居民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "青湖路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "天山南路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "猛进社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "龙河湾社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "友谊路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "青湖南路社区居民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "人民路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "东城社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "龙泉社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北海东街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "梧桐东街社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "梧桐镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "团结路社区",
                code: "001",
            },
            VillageCode {
                name: "幸福路社区",
                code: "002",
            },
            VillageCode {
                name: "子弟路社区",
                code: "003",
            },
            VillageCode {
                name: "龙泉路社区",
                code: "004",
            },
            VillageCode {
                name: "晋援路社区",
                code: "005",
            },
            VillageCode {
                name: "凤凰社区",
                code: "006",
            },
            VillageCode {
                name: "一连生活区",
                code: "007",
            },
            VillageCode {
                name: "二连生活区",
                code: "008",
            },
            VillageCode {
                name: "三连生活区",
                code: "009",
            },
            VillageCode {
                name: "四连生活区",
                code: "010",
            },
            VillageCode {
                name: "五连生活区",
                code: "011",
            },
            VillageCode {
                name: "六连生活区",
                code: "012",
            },
            VillageCode {
                name: "七连生活区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "蔡家湖镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "香甜路社区",
                code: "001",
            },
            VillageCode {
                name: "胜利街社区",
                code: "002",
            },
            VillageCode {
                name: "团结街社区",
                code: "003",
            },
            VillageCode {
                name: "一连生活区",
                code: "004",
            },
            VillageCode {
                name: "二连生活区",
                code: "005",
            },
            VillageCode {
                name: "三连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十二连生活区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "青湖镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "南泉社区",
                code: "001",
            },
            VillageCode {
                name: "克拉玛依西街社区",
                code: "002",
            },
            VillageCode {
                name: "重庆路社区",
                code: "003",
            },
            VillageCode {
                name: "五蔡路社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "冯家坝生活区",
                code: "010",
            },
        ],
    },
];

static TOWNS_XJ_042: [TownCode; 6] = [
    TownCode {
        name: "天骄街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "文苑路社区",
                code: "001",
            },
            VillageCode {
                name: "迎宾路社区",
                code: "002",
            },
            VillageCode {
                name: "彩玉路社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "龙疆街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "林茵街社区",
                code: "001",
            },
            VillageCode {
                name: "博望西街社区",
                code: "002",
            },
            VillageCode {
                name: "绿茵路社区",
                code: "003",
            },
            VillageCode {
                name: "彩虹路社区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "军垦街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "文昌路社区",
                code: "001",
            },
            VillageCode {
                name: "秋水路社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "双渠镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "锡伯渡社区",
                code: "001",
            },
            VillageCode {
                name: "四连生活区",
                code: "002",
            },
            VillageCode {
                name: "五连生活区",
                code: "003",
            },
            VillageCode {
                name: "六连生活区",
                code: "004",
            },
            VillageCode {
                name: "七连生活区",
                code: "005",
            },
            VillageCode {
                name: "八连生活区",
                code: "006",
            },
            VillageCode {
                name: "九连生活区",
                code: "007",
            },
            VillageCode {
                name: "十连生活区",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "丰庆镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "丰泽社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "海川镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "碧海社区",
                code: "001",
            },
            VillageCode {
                name: "星海社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
            VillageCode {
                name: "九连生活区",
                code: "010",
            },
            VillageCode {
                name: "十连生活区",
                code: "011",
            },
        ],
    },
];

static TOWNS_XJ_043: [TownCode; 10] = [
    TownCode {
        name: "迎宾街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "河北花苑社区",
                code: "001",
            },
            VillageCode {
                name: "迎宾社区",
                code: "002",
            },
            VillageCode {
                name: "园区管理区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "博古其镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "向阳社区",
                code: "001",
            },
            VillageCode {
                name: "梨华社区",
                code: "002",
            },
            VillageCode {
                name: "梨香社区",
                code: "003",
            },
            VillageCode {
                name: "湖光社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "六连生活区",
                code: "010",
            },
            VillageCode {
                name: "七连生活区",
                code: "011",
            },
            VillageCode {
                name: "八连生活区",
                code: "012",
            },
            VillageCode {
                name: "九连生活区",
                code: "013",
            },
            VillageCode {
                name: "十连生活区",
                code: "014",
            },
            VillageCode {
                name: "十一连生活区",
                code: "015",
            },
            VillageCode {
                name: "十三连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十六连生活区",
                code: "018",
            },
            VillageCode {
                name: "十七连生活区",
                code: "019",
            },
            VillageCode {
                name: "十九连生活区",
                code: "020",
            },
            VillageCode {
                name: "二十连生活区",
                code: "021",
            },
            VillageCode {
                name: "园二连生活区",
                code: "022",
            },
            VillageCode {
                name: "园三连生活区",
                code: "023",
            },
            VillageCode {
                name: "园四连生活区",
                code: "024",
            },
            VillageCode {
                name: "园七连生活区",
                code: "025",
            },
            VillageCode {
                name: "园九连生活区",
                code: "026",
            },
            VillageCode {
                name: "园十连生活区",
                code: "027",
            },
            VillageCode {
                name: "园十一连生活区",
                code: "028",
            },
            VillageCode {
                name: "园十二连生活区",
                code: "029",
            },
            VillageCode {
                name: "园十三连生活区",
                code: "030",
            },
            VillageCode {
                name: "园十四连生活区",
                code: "031",
            },
            VillageCode {
                name: "园一连生活区",
                code: "032",
            },
            VillageCode {
                name: "园五连生活区",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "双丰镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "双丰社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
            VillageCode {
                name: "九连生活区",
                code: "010",
            },
            VillageCode {
                name: "十连生活区",
                code: "011",
            },
            VillageCode {
                name: "园一连生活区",
                code: "012",
            },
            VillageCode {
                name: "园二连生活区",
                code: "013",
            },
            VillageCode {
                name: "园三连生活区",
                code: "014",
            },
            VillageCode {
                name: "园五连生活区",
                code: "015",
            },
            VillageCode {
                name: "园六连生活区",
                code: "016",
            },
            VillageCode {
                name: "畜管站生活区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "河畔镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "通达社区",
                code: "001",
            },
            VillageCode {
                name: "幸福社区",
                code: "002",
            },
            VillageCode {
                name: "亚泰社区",
                code: "003",
            },
            VillageCode {
                name: "一连生活区",
                code: "004",
            },
            VillageCode {
                name: "二连生活区",
                code: "005",
            },
            VillageCode {
                name: "三连生活区",
                code: "006",
            },
            VillageCode {
                name: "四连生活区",
                code: "007",
            },
            VillageCode {
                name: "五连生活区",
                code: "008",
            },
            VillageCode {
                name: "六连生活区",
                code: "009",
            },
            VillageCode {
                name: "七连生活区",
                code: "010",
            },
            VillageCode {
                name: "八连生活区",
                code: "011",
            },
            VillageCode {
                name: "九连生活区",
                code: "012",
            },
            VillageCode {
                name: "十连生活区",
                code: "013",
            },
            VillageCode {
                name: "十一连生活区",
                code: "014",
            },
            VillageCode {
                name: "十二连生活区",
                code: "015",
            },
            VillageCode {
                name: "十三连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十五连生活区",
                code: "018",
            },
            VillageCode {
                name: "十六连生活区",
                code: "019",
            },
            VillageCode {
                name: "十七连生活区",
                code: "020",
            },
            VillageCode {
                name: "绿源糖业生活区",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "高桥镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "顺祥社区",
                code: "001",
            },
            VillageCode {
                name: "振兴社区",
                code: "002",
            },
            VillageCode {
                name: "天格尔社区",
                code: "003",
            },
            VillageCode {
                name: "一连生活区",
                code: "004",
            },
            VillageCode {
                name: "二连生活区",
                code: "005",
            },
            VillageCode {
                name: "三连生活区",
                code: "006",
            },
            VillageCode {
                name: "四连生活区",
                code: "007",
            },
            VillageCode {
                name: "五连生活区",
                code: "008",
            },
            VillageCode {
                name: "六连生活区",
                code: "009",
            },
            VillageCode {
                name: "七连生活区",
                code: "010",
            },
            VillageCode {
                name: "渔场生活区",
                code: "011",
            },
            VillageCode {
                name: "园二连生活区",
                code: "012",
            },
            VillageCode {
                name: "八连生活区",
                code: "013",
            },
            VillageCode {
                name: "九连生活区",
                code: "014",
            },
            VillageCode {
                name: "十连生活区",
                code: "015",
            },
            VillageCode {
                name: "十一连生活区",
                code: "016",
            },
            VillageCode {
                name: "园一连生活区",
                code: "017",
            },
            VillageCode {
                name: "园三连生活区",
                code: "018",
            },
            VillageCode {
                name: "十六连生活区",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "天湖镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "天河社区",
                code: "001",
            },
            VillageCode {
                name: "天湖社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "六连生活区",
                code: "006",
            },
            VillageCode {
                name: "八连生活区",
                code: "007",
            },
            VillageCode {
                name: "九连生活区",
                code: "008",
            },
            VillageCode {
                name: "十连生活区",
                code: "009",
            },
            VillageCode {
                name: "副业队生活区",
                code: "010",
            },
            VillageCode {
                name: "五连生活区",
                code: "011",
            },
            VillageCode {
                name: "农十一队生活区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "开泽镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "民兴社区",
                code: "001",
            },
            VillageCode {
                name: "绿园社区",
                code: "002",
            },
            VillageCode {
                name: "农二连生活区",
                code: "003",
            },
            VillageCode {
                name: "农四连生活区",
                code: "004",
            },
            VillageCode {
                name: "农五连生活区",
                code: "005",
            },
            VillageCode {
                name: "农六连生活区",
                code: "006",
            },
            VillageCode {
                name: "园一连生活区",
                code: "007",
            },
            VillageCode {
                name: "园二连生活区",
                code: "008",
            },
            VillageCode {
                name: "园三连生活区",
                code: "009",
            },
            VillageCode {
                name: "园四连生活区",
                code: "010",
            },
            VillageCode {
                name: "园五连生活区",
                code: "011",
            },
            VillageCode {
                name: "园六连生活区",
                code: "012",
            },
            VillageCode {
                name: "园七连生活区",
                code: "013",
            },
            VillageCode {
                name: "园八连生活区",
                code: "014",
            },
            VillageCode {
                name: "农三连生活区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "米兰镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "米兰社区",
                code: "001",
            },
            VillageCode {
                name: "米兰产业园生活区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "五连生活区",
                code: "005",
            },
            VillageCode {
                name: "三连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "石棉矿生活区",
                code: "008",
            },
            VillageCode {
                name: "洼地钾盐矿生活区",
                code: "009",
            },
            VillageCode {
                name: "四连生活区",
                code: "010",
            },
            VillageCode {
                name: "乌尊硝钾盐矿生活区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "金山镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "明珠社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "四连生活区",
                code: "003",
            },
            VillageCode {
                name: "农二连生活区",
                code: "004",
            },
            VillageCode {
                name: "农三连生活区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "南屯镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "昆仑社区",
                code: "001",
            },
            VillageCode {
                name: "九连生活区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连队（幸福村）",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
        ],
    },
];

static TOWNS_XJ_044: [TownCode; 6] = [
    TownCode {
        name: "明珠街道",
        code: "001",
        villages: &[VillageCode {
            name: "滨河社区居委会",
            code: "001",
        }],
    },
    TownCode {
        name: "双桥镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "昌盛社区居委会",
                code: "001",
            },
            VillageCode {
                name: "和谐社区居委会",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十二连生活区",
                code: "014",
            },
            VillageCode {
                name: "良繁站生活区",
                code: "015",
            },
            VillageCode {
                name: "园艺一连生活区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "石峪镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "托里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "吴都社区居委会",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "博河镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "和兴苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "光明社区居委会",
                code: "002",
            },
            VillageCode {
                name: "幸福社区居委会",
                code: "003",
            },
            VillageCode {
                name: "洪桥社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "七连生活区",
                code: "010",
            },
            VillageCode {
                name: "八连生活区",
                code: "011",
            },
            VillageCode {
                name: "九连生活区",
                code: "012",
            },
            VillageCode {
                name: "十连生活区",
                code: "013",
            },
            VillageCode {
                name: "十一连生活区",
                code: "014",
            },
            VillageCode {
                name: "十二连生活区",
                code: "015",
            },
            VillageCode {
                name: "十三连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十六连生活区",
                code: "018",
            },
            VillageCode {
                name: "十七连生活区",
                code: "019",
            },
            VillageCode {
                name: "十九连生活区",
                code: "020",
            },
            VillageCode {
                name: "二十连生活区",
                code: "021",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "双乐镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "艾比湖社区居委会",
                code: "001",
            },
            VillageCode {
                name: "塔格特社区居委会",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
            VillageCode {
                name: "十一连生活区",
                code: "013",
            },
            VillageCode {
                name: "十二连生活区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "友谊镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "荆楚社区居委会",
                code: "001",
            },
            VillageCode {
                name: "和景社区居委会",
                code: "002",
            },
            VillageCode {
                name: "楚星社区居委会",
                code: "003",
            },
            VillageCode {
                name: "一连生活区",
                code: "004",
            },
            VillageCode {
                name: "二连生活区",
                code: "005",
            },
            VillageCode {
                name: "三连生活区",
                code: "006",
            },
            VillageCode {
                name: "四连生活区",
                code: "007",
            },
            VillageCode {
                name: "五连生活区",
                code: "008",
            },
            VillageCode {
                name: "六连生活区",
                code: "009",
            },
            VillageCode {
                name: "七连生活区",
                code: "010",
            },
            VillageCode {
                name: "八连生活区",
                code: "011",
            },
            VillageCode {
                name: "九连生活区",
                code: "012",
            },
            VillageCode {
                name: "十连生活区",
                code: "013",
            },
            VillageCode {
                name: "十一连生活区",
                code: "014",
            },
            VillageCode {
                name: "十二连生活区",
                code: "015",
            },
            VillageCode {
                name: "十三连生活区",
                code: "016",
            },
            VillageCode {
                name: "十四连生活区",
                code: "017",
            },
            VillageCode {
                name: "十五连生活区",
                code: "018",
            },
            VillageCode {
                name: "十六连生活区",
                code: "019",
            },
        ],
    },
];

static TOWNS_XJ_045: [TownCode; 7] = [
    TownCode {
        name: "金山街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "福安社区居委会",
                code: "001",
            },
            VillageCode {
                name: "可克达拉经济技术开发区社区",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "花城街道",
        code: "002",
        villages: &[VillageCode {
            name: "锦安社区居委会",
            code: "001",
        }],
    },
    TownCode {
        name: "榆树庄镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "幸福社区",
                code: "001",
            },
            VillageCode {
                name: "姊妹湖硅业生活区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "十连生活区",
                code: "008",
            },
            VillageCode {
                name: "十一连生活区",
                code: "009",
            },
            VillageCode {
                name: "十二连生活区",
                code: "010",
            },
            VillageCode {
                name: "十三连生活区",
                code: "011",
            },
            VillageCode {
                name: "十四连生活区",
                code: "012",
            },
            VillageCode {
                name: "十五连生活区",
                code: "013",
            },
            VillageCode {
                name: "十六连生活区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "苇湖镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "永顺社区",
                code: "001",
            },
            VillageCode {
                name: "瑞祥社区",
                code: "002",
            },
            VillageCode {
                name: "文慧园社区",
                code: "003",
            },
            VillageCode {
                name: "阳光社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "六连生活区",
                code: "010",
            },
            VillageCode {
                name: "七连生活区",
                code: "011",
            },
            VillageCode {
                name: "八连生活区",
                code: "012",
            },
            VillageCode {
                name: "九连生活区",
                code: "013",
            },
            VillageCode {
                name: "十连生活区",
                code: "014",
            },
            VillageCode {
                name: "十一连生活区",
                code: "015",
            },
            VillageCode {
                name: "十二连生活区",
                code: "016",
            },
            VillageCode {
                name: "十三连生活区",
                code: "017",
            },
            VillageCode {
                name: "十四连生活区",
                code: "018",
            },
            VillageCode {
                name: "十五连生活区",
                code: "019",
            },
            VillageCode {
                name: "十六连生活区",
                code: "020",
            },
            VillageCode {
                name: "十七连生活区",
                code: "021",
            },
            VillageCode {
                name: "十八连生活区",
                code: "022",
            },
            VillageCode {
                name: "十九连生活区",
                code: "023",
            },
            VillageCode {
                name: "二十连生活区",
                code: "024",
            },
            VillageCode {
                name: "二十一连生活区",
                code: "025",
            },
            VillageCode {
                name: "二十二连生活区",
                code: "026",
            },
            VillageCode {
                name: "绿华糖业生活区",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "长丰镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "伊香社区",
                code: "001",
            },
            VillageCode {
                name: "丽华社区",
                code: "002",
            },
            VillageCode {
                name: "一连生活区",
                code: "003",
            },
            VillageCode {
                name: "二连生活区",
                code: "004",
            },
            VillageCode {
                name: "三连生活区",
                code: "005",
            },
            VillageCode {
                name: "四连生活区",
                code: "006",
            },
            VillageCode {
                name: "五连生活区",
                code: "007",
            },
            VillageCode {
                name: "六连生活区",
                code: "008",
            },
            VillageCode {
                name: "七连生活区",
                code: "009",
            },
            VillageCode {
                name: "八连生活区",
                code: "010",
            },
            VillageCode {
                name: "九连生活区",
                code: "011",
            },
            VillageCode {
                name: "十连生活区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "金梁镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "幸福路社区",
                code: "001",
            },
            VillageCode {
                name: "育才路社区",
                code: "002",
            },
            VillageCode {
                name: "金梁子社区",
                code: "003",
            },
            VillageCode {
                name: "四方糖业生活区",
                code: "004",
            },
            VillageCode {
                name: "九连生活区",
                code: "005",
            },
            VillageCode {
                name: "十连生活区",
                code: "006",
            },
            VillageCode {
                name: "十一连生活区",
                code: "007",
            },
            VillageCode {
                name: "十二连生活区",
                code: "008",
            },
            VillageCode {
                name: "十三连生活区",
                code: "009",
            },
            VillageCode {
                name: "十四连生活区",
                code: "010",
            },
            VillageCode {
                name: "十五连生活区",
                code: "011",
            },
            VillageCode {
                name: "十六连生活区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "金屯镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "团部社区",
                code: "001",
            },
            VillageCode {
                name: "二连生活区",
                code: "002",
            },
            VillageCode {
                name: "三连生活区",
                code: "003",
            },
            VillageCode {
                name: "四连生活区",
                code: "004",
            },
            VillageCode {
                name: "五连生活区",
                code: "005",
            },
            VillageCode {
                name: "六连生活区",
                code: "006",
            },
            VillageCode {
                name: "七连生活区",
                code: "007",
            },
            VillageCode {
                name: "八连生活区",
                code: "008",
            },
            VillageCode {
                name: "十一连生活区",
                code: "009",
            },
            VillageCode {
                name: "十二连生活区",
                code: "010",
            },
            VillageCode {
                name: "十三连生活区",
                code: "011",
            },
            VillageCode {
                name: "十四连生活区",
                code: "012",
            },
        ],
    },
];

static TOWNS_XJ_046: [TownCode; 6] = [
    TownCode {
        name: "玉都街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "康泰社区",
                code: "001",
            },
            VillageCode {
                name: "幸福社区",
                code: "002",
            },
            VillageCode {
                name: "昆玉市经济技术开发区社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "老兵镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "四十七团昆仑社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
            VillageCode {
                name: "九连生活区",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "昆泉镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "科技路社区",
                code: "001",
            },
            VillageCode {
                name: "进步路社区",
                code: "002",
            },
            VillageCode {
                name: "忠义路社区",
                code: "003",
            },
            VillageCode {
                name: "光明社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "五连生活区",
                code: "009",
            },
            VillageCode {
                name: "六连生活区",
                code: "010",
            },
            VillageCode {
                name: "七连生活区",
                code: "011",
            },
            VillageCode {
                name: "八连生活区",
                code: "012",
            },
            VillageCode {
                name: "九连生活区",
                code: "013",
            },
            VillageCode {
                name: "十连生活区",
                code: "014",
            },
            VillageCode {
                name: "十一连生活区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "昆牧镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "福缘社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "玉泉镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "四连生活区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "玉园镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "玉龙社区",
                code: "001",
            },
            VillageCode {
                name: "一连生活区",
                code: "002",
            },
            VillageCode {
                name: "二连生活区",
                code: "003",
            },
            VillageCode {
                name: "三连生活区",
                code: "004",
            },
            VillageCode {
                name: "四连生活区",
                code: "005",
            },
            VillageCode {
                name: "五连生活区",
                code: "006",
            },
            VillageCode {
                name: "六连生活区",
                code: "007",
            },
            VillageCode {
                name: "七连生活区",
                code: "008",
            },
            VillageCode {
                name: "八连生活区",
                code: "009",
            },
            VillageCode {
                name: "九连生活区",
                code: "010",
            },
            VillageCode {
                name: "十连生活区",
                code: "011",
            },
            VillageCode {
                name: "十一连生活区",
                code: "012",
            },
            VillageCode {
                name: "十二连生活区",
                code: "013",
            },
        ],
    },
];

static TOWNS_XJ_047: [TownCode; 5] = [
    TownCode {
        name: "胡杨街道",
        code: "001",
        villages: &[VillageCode {
            name: "虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "共青镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "胡杨苑社区",
                code: "001",
            },
            VillageCode {
                name: "共青路社区",
                code: "002",
            },
            VillageCode {
                name: "光明路社区",
                code: "003",
            },
            VillageCode {
                name: "育才路社区",
                code: "004",
            },
            VillageCode {
                name: "一连生活区",
                code: "005",
            },
            VillageCode {
                name: "二连生活区",
                code: "006",
            },
            VillageCode {
                name: "三连生活区",
                code: "007",
            },
            VillageCode {
                name: "四连生活区",
                code: "008",
            },
            VillageCode {
                name: "六连生活区",
                code: "009",
            },
            VillageCode {
                name: "七连生活区",
                code: "010",
            },
            VillageCode {
                name: "八连生活区",
                code: "011",
            },
            VillageCode {
                name: "九连生活区",
                code: "012",
            },
            VillageCode {
                name: "十连生活区",
                code: "013",
            },
            VillageCode {
                name: "十三连生活区",
                code: "014",
            },
            VillageCode {
                name: "十四连生活区",
                code: "015",
            },
            VillageCode {
                name: "十五连生活区",
                code: "016",
            },
            VillageCode {
                name: "十六连生活区",
                code: "017",
            },
            VillageCode {
                name: "十七连生活区",
                code: "018",
            },
            VillageCode {
                name: "二十连生活区",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "兵团一二五团",
        code: "003",
        villages: &[
            VillageCode {
                name: "十连生活区",
                code: "001",
            },
            VillageCode {
                name: "十三连生活区",
                code: "002",
            },
            VillageCode {
                name: "十八连生活区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "兵团一二八团",
        code: "004",
        villages: &[
            VillageCode {
                name: "团部社区",
                code: "001",
            },
            VillageCode {
                name: "四连生活区",
                code: "002",
            },
            VillageCode {
                name: "六连生活区",
                code: "003",
            },
            VillageCode {
                name: "七连生活区",
                code: "004",
            },
            VillageCode {
                name: "十连生活区",
                code: "005",
            },
            VillageCode {
                name: "十一连生活区",
                code: "006",
            },
            VillageCode {
                name: "十六连生活区",
                code: "007",
            },
            VillageCode {
                name: "十七连生活区",
                code: "008",
            },
            VillageCode {
                name: "十八连生活区",
                code: "009",
            },
            VillageCode {
                name: "十九连生活区",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "兵团一二九团",
        code: "005",
        villages: &[
            VillageCode {
                name: "一连生活区",
                code: "001",
            },
            VillageCode {
                name: "三连生活区",
                code: "002",
            },
            VillageCode {
                name: "四连生活区",
                code: "003",
            },
            VillageCode {
                name: "五连生活区",
                code: "004",
            },
            VillageCode {
                name: "六连生活区",
                code: "005",
            },
            VillageCode {
                name: "七连生活区",
                code: "006",
            },
            VillageCode {
                name: "八连生活区",
                code: "007",
            },
            VillageCode {
                name: "十一连生活区",
                code: "008",
            },
            VillageCode {
                name: "十二连生活区",
                code: "009",
            },
            VillageCode {
                name: "十四连生活区",
                code: "010",
            },
            VillageCode {
                name: "十五连生活区",
                code: "011",
            },
            VillageCode {
                name: "园艺连生活区",
                code: "012",
            },
        ],
    },
];

pub const CITIES_XJ: [CityCode; 48] = [
    CityCode {
        name: "省辖市",
        code: "000",
        towns: &[],
    },
    CityCode {
        name: "乌鲁木齐市",
        code: "001",
        towns: &TOWNS_XJ_001,
    },
    CityCode {
        name: "天山市",
        code: "002",
        towns: &TOWNS_XJ_002,
    },
    CityCode {
        name: "沙依巴克市",
        code: "003",
        towns: &TOWNS_XJ_003,
    },
    CityCode {
        name: "新市市",
        code: "004",
        towns: &TOWNS_XJ_004,
    },
    CityCode {
        name: "水磨沟市",
        code: "005",
        towns: &TOWNS_XJ_005,
    },
    CityCode {
        name: "头屯河市",
        code: "006",
        towns: &TOWNS_XJ_006,
    },
    CityCode {
        name: "达坂城市",
        code: "007",
        towns: &TOWNS_XJ_007,
    },
    CityCode {
        name: "米东市",
        code: "008",
        towns: &TOWNS_XJ_008,
    },
    CityCode {
        name: "高昌市",
        code: "009",
        towns: &TOWNS_XJ_009,
    },
    CityCode {
        name: "鄯善市",
        code: "010",
        towns: &TOWNS_XJ_010,
    },
    CityCode {
        name: "托克逊市",
        code: "011",
        towns: &TOWNS_XJ_011,
    },
    CityCode {
        name: "伊州市",
        code: "012",
        towns: &TOWNS_XJ_012,
    },
    CityCode {
        name: "巴里坤市",
        code: "013",
        towns: &TOWNS_XJ_013,
    },
    CityCode {
        name: "伊吾市",
        code: "014",
        towns: &TOWNS_XJ_014,
    },
    CityCode {
        name: "昌吉市",
        code: "015",
        towns: &TOWNS_XJ_015,
    },
    CityCode {
        name: "阜康市",
        code: "016",
        towns: &TOWNS_XJ_016,
    },
    CityCode {
        name: "呼图壁市",
        code: "017",
        towns: &TOWNS_XJ_017,
    },
    CityCode {
        name: "玛纳斯市",
        code: "018",
        towns: &TOWNS_XJ_018,
    },
    CityCode {
        name: "奇台市",
        code: "019",
        towns: &TOWNS_XJ_019,
    },
    CityCode {
        name: "吉木萨尔市",
        code: "020",
        towns: &TOWNS_XJ_020,
    },
    CityCode {
        name: "木垒市",
        code: "021",
        towns: &TOWNS_XJ_021,
    },
    CityCode {
        name: "库尔勒市",
        code: "022",
        towns: &TOWNS_XJ_022,
    },
    CityCode {
        name: "轮台市",
        code: "023",
        towns: &TOWNS_XJ_023,
    },
    CityCode {
        name: "尉犁市",
        code: "024",
        towns: &TOWNS_XJ_024,
    },
    CityCode {
        name: "若羌市",
        code: "025",
        towns: &TOWNS_XJ_025,
    },
    CityCode {
        name: "且末市",
        code: "026",
        towns: &TOWNS_XJ_026,
    },
    CityCode {
        name: "焉耆市",
        code: "027",
        towns: &TOWNS_XJ_027,
    },
    CityCode {
        name: "和静市",
        code: "028",
        towns: &TOWNS_XJ_028,
    },
    CityCode {
        name: "和硕市",
        code: "029",
        towns: &TOWNS_XJ_029,
    },
    CityCode {
        name: "博湖市",
        code: "030",
        towns: &TOWNS_XJ_030,
    },
    CityCode {
        name: "阿勒泰市",
        code: "031",
        towns: &TOWNS_XJ_031,
    },
    CityCode {
        name: "布尔津市",
        code: "032",
        towns: &TOWNS_XJ_032,
    },
    CityCode {
        name: "富蕴市",
        code: "033",
        towns: &TOWNS_XJ_033,
    },
    CityCode {
        name: "福海市",
        code: "034",
        towns: &TOWNS_XJ_034,
    },
    CityCode {
        name: "哈巴河市",
        code: "035",
        towns: &TOWNS_XJ_035,
    },
    CityCode {
        name: "青河市",
        code: "036",
        towns: &TOWNS_XJ_036,
    },
    CityCode {
        name: "吉木乃市",
        code: "037",
        towns: &TOWNS_XJ_037,
    },
    CityCode {
        name: "石河子市",
        code: "038",
        towns: &TOWNS_XJ_038,
    },
    CityCode {
        name: "阿拉尔市",
        code: "039",
        towns: &TOWNS_XJ_039,
    },
    CityCode {
        name: "图木舒克市",
        code: "040",
        towns: &TOWNS_XJ_040,
    },
    CityCode {
        name: "五家渠市",
        code: "041",
        towns: &TOWNS_XJ_041,
    },
    CityCode {
        name: "北屯市",
        code: "042",
        towns: &TOWNS_XJ_042,
    },
    CityCode {
        name: "铁门关市",
        code: "043",
        towns: &TOWNS_XJ_043,
    },
    CityCode {
        name: "双河市",
        code: "044",
        towns: &TOWNS_XJ_044,
    },
    CityCode {
        name: "可克达拉市",
        code: "045",
        towns: &TOWNS_XJ_045,
    },
    CityCode {
        name: "昆玉市",
        code: "046",
        towns: &TOWNS_XJ_046,
    },
    CityCode {
        name: "胡杨河市",
        code: "047",
        towns: &TOWNS_XJ_047,
    },
];
