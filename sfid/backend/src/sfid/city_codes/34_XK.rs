use super::{CityCode, TownCode, VillageCode};

static TOWNS_XK_001: [TownCode; 12] = [
    TownCode {
        name: "建塘镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "建塘社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "金龙社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北门社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "仓房社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "北郊社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "红坡村民委员会",
                code: "006",
            },
            VillageCode {
                name: "吉迪村民委员会",
                code: "007",
            },
            VillageCode {
                name: "解放村民委员会",
                code: "008",
            },
            VillageCode {
                name: "尼史村民委员会",
                code: "009",
            },
            VillageCode {
                name: "诺西村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "小中甸镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "联合村民委员会",
                code: "001",
            },
            VillageCode {
                name: "和平村民委员会",
                code: "002",
            },
            VillageCode {
                name: "团结村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "虎跳峡镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "红旗村民委员会",
                code: "001",
            },
            VillageCode {
                name: "长胜村民委员会",
                code: "002",
            },
            VillageCode {
                name: "桥头村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东坡村民委员会",
                code: "004",
            },
            VillageCode {
                name: "松鹤村民委员会",
                code: "005",
            },
            VillageCode {
                name: "永胜村民委员会",
                code: "006",
            },
            VillageCode {
                name: "金星村民委员会",
                code: "007",
            },
            VillageCode {
                name: "宝山村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下桥头村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "金江镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "新建村民委员会",
                code: "001",
            },
            VillageCode {
                name: "兴隆村民委员会",
                code: "002",
            },
            VillageCode {
                name: "安乐村民委员会",
                code: "003",
            },
            VillageCode {
                name: "吾竹村民委员会",
                code: "004",
            },
            VillageCode {
                name: "车轴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "仕达村民委员会",
                code: "006",
            },
            VillageCode {
                name: "兴文村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "上江乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "木高村民委员会",
                code: "001",
            },
            VillageCode {
                name: "良美村民委员会",
                code: "002",
            },
            VillageCode {
                name: "福库村民委员会",
                code: "003",
            },
            VillageCode {
                name: "格兰村民委员会",
                code: "004",
            },
            VillageCode {
                name: "士旺村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "三坝纳西族乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "东坝一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "安南村民委员会",
                code: "002",
            },
            VillageCode {
                name: "白地村民委员会",
                code: "003",
            },
            VillageCode {
                name: "瓦刷村民委员会",
                code: "004",
            },
            VillageCode {
                name: "哈巴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "江边村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东坝二村村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "洛吉乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "九龙村民委员会",
                code: "001",
            },
            VillageCode {
                name: "洛吉村民委员会",
                code: "002",
            },
            VillageCode {
                name: "尼汝村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "尼西乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "幸福村民委员会",
                code: "001",
            },
            VillageCode {
                name: "新阳村民委员会",
                code: "002",
            },
            VillageCode {
                name: "汤满村民委员会",
                code: "003",
            },
            VillageCode {
                name: "江东村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "格咱乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "格咱村民委员会",
                code: "001",
            },
            VillageCode {
                name: "翁上村民委员会",
                code: "002",
            },
            VillageCode {
                name: "翁水村民委员会",
                code: "003",
            },
            VillageCode {
                name: "浪都村民委员会",
                code: "004",
            },
            VillageCode {
                name: "纳格拉村民委员会",
                code: "005",
            },
            VillageCode {
                name: "木鲁村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "东旺乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "上游村民委员会",
                code: "001",
            },
            VillageCode {
                name: "跃进村民委员会",
                code: "002",
            },
            VillageCode {
                name: "中心村民委员会",
                code: "003",
            },
            VillageCode {
                name: "新联村民委员会",
                code: "004",
            },
            VillageCode {
                name: "胜利村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "五境乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "霞珠村民委员会",
                code: "001",
            },
            VillageCode {
                name: "仓觉村民委员会",
                code: "002",
            },
            VillageCode {
                name: "泽通村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "迪庆扶贫民族经济开发区",
        code: "012",
        villages: &[
            VillageCode {
                name: "开发区社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新仁村民委员会",
                code: "002",
            },
            VillageCode {
                name: "礼仁村民委员会",
                code: "003",
            },
        ],
    },
];

static TOWNS_XK_002: [TownCode; 8] = [
    TownCode {
        name: "升平镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "阿敦子社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "敦和社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "巨水村民委员会",
                code: "003",
            },
            VillageCode {
                name: "阿东村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "奔子栏镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "奔子栏社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "玉杰村民委员会",
                code: "002",
            },
            VillageCode {
                name: "书松村民委员会",
                code: "003",
            },
            VillageCode {
                name: "叶日村民委员会",
                code: "004",
            },
            VillageCode {
                name: "夺通村民委员会",
                code: "005",
            },
            VillageCode {
                name: "达日村民委员会",
                code: "006",
            },
            VillageCode {
                name: "叶央村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "佛山乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "纳古村民委员会",
                code: "001",
            },
            VillageCode {
                name: "巴美村民委员会",
                code: "002",
            },
            VillageCode {
                name: "江坡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "鲁瓦村民委员会",
                code: "004",
            },
            VillageCode {
                name: "溜洞江村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "云岭乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "果念村民委员会",
                code: "001",
            },
            VillageCode {
                name: "斯农村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西当村民委员会",
                code: "003",
            },
            VillageCode {
                name: "红坡村民委员会",
                code: "004",
            },
            VillageCode {
                name: "查里通村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "燕门乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "拖拉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "巴东村民委员会",
                code: "002",
            },
            VillageCode {
                name: "茨中村民委员会",
                code: "003",
            },
            VillageCode {
                name: "春多乐村民委员会",
                code: "004",
            },
            VillageCode {
                name: "谷扎村民委员会",
                code: "005",
            },
            VillageCode {
                name: "禹功村民委员会",
                code: "006",
            },
            VillageCode {
                name: "石底村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "拖顶傈僳族乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "金珠嘎尺社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "拖顶村民委员会",
                code: "002",
            },
            VillageCode {
                name: "洛沙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "左力村民委员会",
                code: "004",
            },
            VillageCode {
                name: "大村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "洛玉村民委员会",
                code: "006",
            },
            VillageCode {
                name: "普通农村民委员会",
                code: "007",
            },
            VillageCode {
                name: "念萨村民委员会",
                code: "008",
            },
            VillageCode {
                name: "德吉村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "霞若傈僳族乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "得觉屯社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "霞若村民委员会",
                code: "002",
            },
            VillageCode {
                name: "石茸村民委员会",
                code: "003",
            },
            VillageCode {
                name: "夺松村民委员会",
                code: "004",
            },
            VillageCode {
                name: "月仁村民委员会",
                code: "005",
            },
            VillageCode {
                name: "施坝村民委员会",
                code: "006",
            },
            VillageCode {
                name: "各么茸村民委员会",
                code: "007",
            },
            VillageCode {
                name: "粗卡通村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "羊拉乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "雅瑞安和社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "甲功村民委员会",
                code: "002",
            },
            VillageCode {
                name: "羊拉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "茂顶村民委员会",
                code: "004",
            },
        ],
    },
];

static TOWNS_XK_003: [TownCode; 10] = [
    TownCode {
        name: "保和镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "南路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "白鹤山社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "十字街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "永宁社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "保和村民委员会",
                code: "005",
            },
            VillageCode {
                name: "兰永村民委员会",
                code: "006",
            },
            VillageCode {
                name: "永春村民委员会",
                code: "007",
            },
            VillageCode {
                name: "拉河柱村民委员会",
                code: "008",
            },
            VillageCode {
                name: "罗马村民委员会",
                code: "009",
            },
            VillageCode {
                name: "腊八底村民委员会",
                code: "010",
            },
            VillageCode {
                name: "高泉村民委员会",
                code: "011",
            },
            VillageCode {
                name: "拉日村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "叶枝镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "巴丁村民委员会",
                code: "001",
            },
            VillageCode {
                name: "倮那村民委员会",
                code: "002",
            },
            VillageCode {
                name: "梓里村民委员会",
                code: "003",
            },
            VillageCode {
                name: "新洛村民委员会",
                code: "004",
            },
            VillageCode {
                name: "叶枝村民委员会",
                code: "005",
            },
            VillageCode {
                name: "同乐村民委员会",
                code: "006",
            },
            VillageCode {
                name: "松洛村民委员会",
                code: "007",
            },
            VillageCode {
                name: "拉波洛村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "塔城镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "川达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "海尼村民委员会",
                code: "002",
            },
            VillageCode {
                name: "柯那村民委员会",
                code: "003",
            },
            VillageCode {
                name: "塔城村民委员会",
                code: "004",
            },
            VillageCode {
                name: "启别村民委员会",
                code: "005",
            },
            VillageCode {
                name: "巴珠村民委员会",
                code: "006",
            },
            VillageCode {
                name: "其宗村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "永春乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "菊香村民委员会",
                code: "001",
            },
            VillageCode {
                name: "美光村民委员会",
                code: "002",
            },
            VillageCode {
                name: "四保村民委员会",
                code: "003",
            },
            VillageCode {
                name: "庆福村民委员会",
                code: "004",
            },
            VillageCode {
                name: "拖枝村民委员会",
                code: "005",
            },
            VillageCode {
                name: "三家村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "攀天阁乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "皆菊村民委员会",
                code: "001",
            },
            VillageCode {
                name: "美洛村民委员会",
                code: "002",
            },
            VillageCode {
                name: "工农村民委员会",
                code: "003",
            },
            VillageCode {
                name: "安一村民委员会",
                code: "004",
            },
            VillageCode {
                name: "新华村民委员会",
                code: "005",
            },
            VillageCode {
                name: "新乐村民委员会",
                code: "006",
            },
            VillageCode {
                name: "嘎嘎塘村民委员会",
                code: "007",
            },
            VillageCode {
                name: "岔支洛村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "白济汛乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "白济汛村民委员会",
                code: "001",
            },
            VillageCode {
                name: "统维村民委员会",
                code: "002",
            },
            VillageCode {
                name: "永安村民委员会",
                code: "003",
            },
            VillageCode {
                name: "施底村民委员会",
                code: "004",
            },
            VillageCode {
                name: "干坝子村民委员会",
                code: "005",
            },
            VillageCode {
                name: "碧罗村民委员会",
                code: "006",
            },
            VillageCode {
                name: "共厂村民委员会",
                code: "007",
            },
            VillageCode {
                name: "共乐村民委员会",
                code: "008",
            },
            VillageCode {
                name: "共恩村民委员会",
                code: "009",
            },
            VillageCode {
                name: "共园村民委员会",
                code: "010",
            },
            VillageCode {
                name: "共吉村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "康普乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "弄独村民委员会",
                code: "001",
            },
            VillageCode {
                name: "阿倮村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阿尼比村民委员会",
                code: "003",
            },
            VillageCode {
                name: "札子村民委员会",
                code: "004",
            },
            VillageCode {
                name: "念里米村民委员会",
                code: "005",
            },
            VillageCode {
                name: "岔枝村民委员会",
                code: "006",
            },
            VillageCode {
                name: "康普村民委员会",
                code: "007",
            },
            VillageCode {
                name: "普乐村民委员会",
                code: "008",
            },
            VillageCode {
                name: "齐乐村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "巴迪乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "结义村民委员会",
                code: "001",
            },
            VillageCode {
                name: "洛义村民委员会",
                code: "002",
            },
            VillageCode {
                name: "巴迪村民委员会",
                code: "003",
            },
            VillageCode {
                name: "捧八村民委员会",
                code: "004",
            },
            VillageCode {
                name: "真朴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "阿尺打嘎村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "中路乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "佳禾村民委员会",
                code: "001",
            },
            VillageCode {
                name: "新厂村民委员会",
                code: "002",
            },
            VillageCode {
                name: "蕨菜山村民委员会",
                code: "003",
            },
            VillageCode {
                name: "腊八山村民委员会",
                code: "004",
            },
            VillageCode {
                name: "施根登村民委员会",
                code: "005",
            },
            VillageCode {
                name: "咱利村民委员会",
                code: "006",
            },
            VillageCode {
                name: "拉嘎洛村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "维登乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "北甸村民委员会",
                code: "001",
            },
            VillageCode {
                name: "小甸村民委员会",
                code: "002",
            },
            VillageCode {
                name: "山加村民委员会",
                code: "003",
            },
            VillageCode {
                name: "维登村民委员会",
                code: "004",
            },
            VillageCode {
                name: "新农村民委员会",
                code: "005",
            },
            VillageCode {
                name: "富川村民委员会",
                code: "006",
            },
            VillageCode {
                name: "新化村民委员会",
                code: "007",
            },
            VillageCode {
                name: "箐头村民委员会",
                code: "008",
            },
            VillageCode {
                name: "妥洛村民委员会",
                code: "009",
            },
        ],
    },
];

static TOWNS_XK_004: [TownCode; 10] = [
    TownCode {
        name: "大练地街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "新城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "锦秀社区居委会",
                code: "002",
            },
            VillageCode {
                name: "和谐社区居委会",
                code: "003",
            },
            VillageCode {
                name: "赖茂村委会",
                code: "004",
            },
            VillageCode {
                name: "新建村委会",
                code: "005",
            },
            VillageCode {
                name: "大练地村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "六库街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "江西社区居委会",
                code: "001",
            },
            VillageCode {
                name: "重阳社区居委会",
                code: "002",
            },
            VillageCode {
                name: "向阳社区居委会",
                code: "003",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "004",
            },
            VillageCode {
                name: "大龙塘社区居委会",
                code: "005",
            },
            VillageCode {
                name: "排路坝村委会",
                code: "006",
            },
            VillageCode {
                name: "小沙坝村委会",
                code: "007",
            },
            VillageCode {
                name: "新寨村委会",
                code: "008",
            },
            VillageCode {
                name: "新田村委会",
                code: "009",
            },
            VillageCode {
                name: "大密扣村委会",
                code: "010",
            },
            VillageCode {
                name: "段家寨村委会",
                code: "011",
            },
            VillageCode {
                name: "白水河村委会",
                code: "012",
            },
            VillageCode {
                name: "瓦姑村委会",
                code: "013",
            },
            VillageCode {
                name: "六库村委会",
                code: "014",
            },
            VillageCode {
                name: "苗干山村委会",
                code: "015",
            },
            VillageCode {
                name: "双米地村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "鲁掌镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "鲁掌村委会",
                code: "001",
            },
            VillageCode {
                name: "鲁祖村委会",
                code: "002",
            },
            VillageCode {
                name: "洛玛村委会",
                code: "003",
            },
            VillageCode {
                name: "浪坝寨村委会",
                code: "004",
            },
            VillageCode {
                name: "三河村委会",
                code: "005",
            },
            VillageCode {
                name: "登埂村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "片马镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "景朗社区居委会",
                code: "001",
            },
            VillageCode {
                name: "片马村委会",
                code: "002",
            },
            VillageCode {
                name: "片四河村委会",
                code: "003",
            },
            VillageCode {
                name: "古浪村委会",
                code: "004",
            },
            VillageCode {
                name: "岗房村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "上江镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "同心社区居委会",
                code: "001",
            },
            VillageCode {
                name: "叶子花社区居委会",
                code: "002",
            },
            VillageCode {
                name: "丙贡村委会",
                code: "003",
            },
            VillageCode {
                name: "蛮英村委会",
                code: "004",
            },
            VillageCode {
                name: "丙奉村委会",
                code: "005",
            },
            VillageCode {
                name: "付坝村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "老窝镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "老窝村委会",
                code: "001",
            },
            VillageCode {
                name: "荣华村委会",
                code: "002",
            },
            VillageCode {
                name: "中元村委会",
                code: "003",
            },
            VillageCode {
                name: "崇仁村委会",
                code: "004",
            },
            VillageCode {
                name: "银坡村委会",
                code: "005",
            },
            VillageCode {
                name: "云西村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "大兴地镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "维拉坝珠海社区居委会",
                code: "001",
            },
            VillageCode {
                name: "自扁王基村委会",
                code: "002",
            },
            VillageCode {
                name: "木楠村委会",
                code: "003",
            },
            VillageCode {
                name: "团结村委会",
                code: "004",
            },
            VillageCode {
                name: "鲁奎地村委会",
                code: "005",
            },
            VillageCode {
                name: "自基村委会",
                code: "006",
            },
            VillageCode {
                name: "卯照村委会",
                code: "007",
            },
            VillageCode {
                name: "四排拉多村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "称杆乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "恩感思落社区居委会",
                code: "001",
            },
            VillageCode {
                name: "双奎地村委会",
                code: "002",
            },
            VillageCode {
                name: "称杆村委会",
                code: "003",
            },
            VillageCode {
                name: "排把村委会",
                code: "004",
            },
            VillageCode {
                name: "赤耐乃村委会",
                code: "005",
            },
            VillageCode {
                name: "自把村委会",
                code: "006",
            },
            VillageCode {
                name: "堵堵洛村委会",
                code: "007",
            },
            VillageCode {
                name: "玛普拉地村委会",
                code: "008",
            },
            VillageCode {
                name: "勒墨村委会",
                code: "009",
            },
            VillageCode {
                name: "前进村委会",
                code: "010",
            },
            VillageCode {
                name: "王玛基村委会",
                code: "011",
            },
            VillageCode {
                name: "阿赤依堵村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "古登乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "恩河社区居委会",
                code: "001",
            },
            VillageCode {
                name: "季加村委会",
                code: "002",
            },
            VillageCode {
                name: "腊斯底村委会",
                code: "003",
            },
            VillageCode {
                name: "佑雅村委会",
                code: "004",
            },
            VillageCode {
                name: "亚碧罗村委会",
                code: "005",
            },
            VillageCode {
                name: "尼普罗村委会",
                code: "006",
            },
            VillageCode {
                name: "俄夺罗村委会",
                code: "007",
            },
            VillageCode {
                name: "干本村委会",
                code: "008",
            },
            VillageCode {
                name: "马垮底村委会",
                code: "009",
            },
            VillageCode {
                name: "加夺玛村委会",
                code: "010",
            },
            VillageCode {
                name: "色仲村委会",
                code: "011",
            },
            VillageCode {
                name: "念坪村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "洛本卓白族乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "巴尼小镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "托拖村委会",
                code: "002",
            },
            VillageCode {
                name: "保登村委会",
                code: "003",
            },
            VillageCode {
                name: "俄嘎村委会",
                code: "004",
            },
            VillageCode {
                name: "子竹村委会",
                code: "005",
            },
            VillageCode {
                name: "刮然村委会",
                code: "006",
            },
            VillageCode {
                name: "色德村委会",
                code: "007",
            },
            VillageCode {
                name: "金满村委会",
                code: "008",
            },
            VillageCode {
                name: "格甲村委会",
                code: "009",
            },
        ],
    },
];

static TOWNS_XK_005: [TownCode; 7] = [
    TownCode {
        name: "上帕镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "上帕社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "同心社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "福兴社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "碧福社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "润福社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "泽福社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "上帕村委会",
                code: "007",
            },
            VillageCode {
                name: "达友村委会",
                code: "008",
            },
            VillageCode {
                name: "达普洛村委会",
                code: "009",
            },
            VillageCode {
                name: "施底村委会",
                code: "010",
            },
            VillageCode {
                name: "珠明林村委会",
                code: "011",
            },
            VillageCode {
                name: "腊竹底村委会",
                code: "012",
            },
            VillageCode {
                name: "双米底村委会",
                code: "013",
            },
            VillageCode {
                name: "知子洛村委会",
                code: "014",
            },
            VillageCode {
                name: "腊乌村委会",
                code: "015",
            },
            VillageCode {
                name: "古泉村委会",
                code: "016",
            },
            VillageCode {
                name: "木古甲村委会",
                code: "017",
            },
            VillageCode {
                name: "腊吐底村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "匹河怒族乡",
        code: "002",
        villages: &[
            VillageCode {
                name: "怒福社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "普洛村委会",
                code: "002",
            },
            VillageCode {
                name: "瓦娃村委会",
                code: "003",
            },
            VillageCode {
                name: "沙瓦村委会",
                code: "004",
            },
            VillageCode {
                name: "老姆登村委会",
                code: "005",
            },
            VillageCode {
                name: "知子罗村委会",
                code: "006",
            },
            VillageCode {
                name: "棉谷村委会",
                code: "007",
            },
            VillageCode {
                name: "架究村委会",
                code: "008",
            },
            VillageCode {
                name: "托坪村委会",
                code: "009",
            },
            VillageCode {
                name: "果科村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "子里甲乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "子里甲村委会",
                code: "001",
            },
            VillageCode {
                name: "俄科罗村委会",
                code: "002",
            },
            VillageCode {
                name: "腊母甲村委会",
                code: "003",
            },
            VillageCode {
                name: "金秀谷村委会",
                code: "004",
            },
            VillageCode {
                name: "亚谷村委会",
                code: "005",
            },
            VillageCode {
                name: "打吾米村委会",
                code: "006",
            },
            VillageCode {
                name: "双甲村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "架科底乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "架科村委会",
                code: "001",
            },
            VillageCode {
                name: "南安建村委会",
                code: "002",
            },
            VillageCode {
                name: "达大科村委会",
                code: "003",
            },
            VillageCode {
                name: "阿打村委会",
                code: "004",
            },
            VillageCode {
                name: "维独村委会",
                code: "005",
            },
            VillageCode {
                name: "里吾底村委会",
                code: "006",
            },
            VillageCode {
                name: "阿吾摆村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "鹿马登乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "鹿马登村委会",
                code: "001",
            },
            VillageCode {
                name: "亚坪村委会",
                code: "002",
            },
            VillageCode {
                name: "赤洒底村委会",
                code: "003",
            },
            VillageCode {
                name: "娃吐娃村委会",
                code: "004",
            },
            VillageCode {
                name: "麻甲底村委会",
                code: "005",
            },
            VillageCode {
                name: "巴甲朵村委会",
                code: "006",
            },
            VillageCode {
                name: "腊马洛村委会",
                code: "007",
            },
            VillageCode {
                name: "布拉底村委会",
                code: "008",
            },
            VillageCode {
                name: "赤恒底村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "石月亮乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "石月亮社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "利沙底村委会",
                code: "002",
            },
            VillageCode {
                name: "石门登村委会",
                code: "003",
            },
            VillageCode {
                name: "米俄洛村委会",
                code: "004",
            },
            VillageCode {
                name: "知洛村委会",
                code: "005",
            },
            VillageCode {
                name: "咱利村委会",
                code: "006",
            },
            VillageCode {
                name: "资古朵村委会",
                code: "007",
            },
            VillageCode {
                name: "亚朵村委会",
                code: "008",
            },
            VillageCode {
                name: "左洛底村委会",
                code: "009",
            },
            VillageCode {
                name: "拉马底村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "马吉乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "锦福社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "马吉村委会",
                code: "002",
            },
            VillageCode {
                name: "布腊村委会",
                code: "003",
            },
            VillageCode {
                name: "古当村委会",
                code: "004",
            },
            VillageCode {
                name: "木加甲村委会",
                code: "005",
            },
            VillageCode {
                name: "马吉米村委会",
                code: "006",
            },
            VillageCode {
                name: "乔底村委会",
                code: "007",
            },
            VillageCode {
                name: "旺基独村委会",
                code: "008",
            },
        ],
    },
];

static TOWNS_XK_006: [TownCode; 9] = [
    TownCode {
        name: "翠屏街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "永昌社区居委会",
                code: "001",
            },
            VillageCode {
                name: "永安社区居委会",
                code: "002",
            },
            VillageCode {
                name: "永泰社区居委会",
                code: "003",
            },
            VillageCode {
                name: "金龙社区居委会",
                code: "004",
            },
            VillageCode {
                name: "玉泉社区居委会",
                code: "005",
            },
            VillageCode {
                name: "玉屏社区居委会",
                code: "006",
            },
            VillageCode {
                name: "江头河社区居委会",
                code: "007",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "008",
            },
            VillageCode {
                name: "高坪村委会",
                code: "009",
            },
            VillageCode {
                name: "干竹河村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "金顶街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "文兴社区居委会",
                code: "001",
            },
            VillageCode {
                name: "永兴社区居委会",
                code: "002",
            },
            VillageCode {
                name: "永祥社区居委会",
                code: "003",
            },
            VillageCode {
                name: "福坪村委会",
                code: "004",
            },
            VillageCode {
                name: "来龙村委会",
                code: "005",
            },
            VillageCode {
                name: "大龙村委会",
                code: "006",
            },
            VillageCode {
                name: "金凤村委会",
                code: "007",
            },
            VillageCode {
                name: "七联村委会",
                code: "008",
            },
            VillageCode {
                name: "官坪村委会",
                code: "009",
            },
            VillageCode {
                name: "箐门村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "啦井镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "啦井村委会",
                code: "001",
            },
            VillageCode {
                name: "长涧村委会",
                code: "002",
            },
            VillageCode {
                name: "新建村委会",
                code: "003",
            },
            VillageCode {
                name: "布场村委会",
                code: "004",
            },
            VillageCode {
                name: "挂登村委会",
                code: "005",
            },
            VillageCode {
                name: "桃树村委会",
                code: "006",
            },
            VillageCode {
                name: "富和村委会",
                code: "007",
            },
            VillageCode {
                name: "九龙村委会",
                code: "008",
            },
            VillageCode {
                name: "期井村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "营盘镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "梨园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "沧东村委会",
                code: "002",
            },
            VillageCode {
                name: "连城村委会",
                code: "003",
            },
            VillageCode {
                name: "新华村委会",
                code: "004",
            },
            VillageCode {
                name: "鸿尤村委会",
                code: "005",
            },
            VillageCode {
                name: "松柏村委会",
                code: "006",
            },
            VillageCode {
                name: "拉古山村委会",
                code: "007",
            },
            VillageCode {
                name: "拉古村委会",
                code: "008",
            },
            VillageCode {
                name: "凤塔村委会",
                code: "009",
            },
            VillageCode {
                name: "金满村委会",
                code: "010",
            },
            VillageCode {
                name: "恩棋村委会",
                code: "011",
            },
            VillageCode {
                name: "恩罗村委会",
                code: "012",
            },
            VillageCode {
                name: "小桥村委会",
                code: "013",
            },
            VillageCode {
                name: "岩头村委会",
                code: "014",
            },
            VillageCode {
                name: "黄柏村委会",
                code: "015",
            },
            VillageCode {
                name: "和平村委会",
                code: "016",
            },
            VillageCode {
                name: "黄梅村委会",
                code: "017",
            },
            VillageCode {
                name: "白羊村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "通甸镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "八十一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "易门箐社区居委会",
                code: "002",
            },
            VillageCode {
                name: "通甸村委会",
                code: "003",
            },
            VillageCode {
                name: "黄松村委会",
                code: "004",
            },
            VillageCode {
                name: "龙潭村委会",
                code: "005",
            },
            VillageCode {
                name: "东明村委会",
                code: "006",
            },
            VillageCode {
                name: "河边村委会",
                code: "007",
            },
            VillageCode {
                name: "箐头村委会",
                code: "008",
            },
            VillageCode {
                name: "德胜村委会",
                code: "009",
            },
            VillageCode {
                name: "下甸村委会",
                code: "010",
            },
            VillageCode {
                name: "努弓村委会",
                code: "011",
            },
            VillageCode {
                name: "福登村委会",
                code: "012",
            },
            VillageCode {
                name: "丰华村委会",
                code: "013",
            },
            VillageCode {
                name: "水俸村委会",
                code: "014",
            },
            VillageCode {
                name: "金竹村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "河西乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "河西村委会",
                code: "001",
            },
            VillageCode {
                name: "共兴村委会",
                code: "002",
            },
            VillageCode {
                name: "仁兴村委会",
                code: "003",
            },
            VillageCode {
                name: "永兴村委会",
                code: "004",
            },
            VillageCode {
                name: "新发村委会",
                code: "005",
            },
            VillageCode {
                name: "白龙村委会",
                code: "006",
            },
            VillageCode {
                name: "玉狮村委会",
                code: "007",
            },
            VillageCode {
                name: "箐花村委会",
                code: "008",
            },
            VillageCode {
                name: "三界村委会",
                code: "009",
            },
            VillageCode {
                name: "大羊村委会",
                code: "010",
            },
            VillageCode {
                name: "联合村委会",
                code: "011",
            },
            VillageCode {
                name: "胜利村委会",
                code: "012",
            },
            VillageCode {
                name: "胜兴村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "中排乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "中排村委会",
                code: "001",
            },
            VillageCode {
                name: "碧玉河村委会",
                code: "002",
            },
            VillageCode {
                name: "北甸村委会",
                code: "003",
            },
            VillageCode {
                name: "德庆村委会",
                code: "004",
            },
            VillageCode {
                name: "多依村委会",
                code: "005",
            },
            VillageCode {
                name: "怒夺村委会",
                code: "006",
            },
            VillageCode {
                name: "信昌坪村委会",
                code: "007",
            },
            VillageCode {
                name: "大宗村委会",
                code: "008",
            },
            VillageCode {
                name: "小龙村委会",
                code: "009",
            },
            VillageCode {
                name: "烟川村委会",
                code: "010",
            },
            VillageCode {
                name: "大土基村委会",
                code: "011",
            },
            VillageCode {
                name: "克卓村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "石登乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "石登村委会",
                code: "001",
            },
            VillageCode {
                name: "车邑坪村委会",
                code: "002",
            },
            VillageCode {
                name: "三角河村委会",
                code: "003",
            },
            VillageCode {
                name: "拉竹河村委会",
                code: "004",
            },
            VillageCode {
                name: "谷川村委会",
                code: "005",
            },
            VillageCode {
                name: "石中坪村委会",
                code: "006",
            },
            VillageCode {
                name: "小格拉村委会",
                code: "007",
            },
            VillageCode {
                name: "水银厂村委会",
                code: "008",
            },
            VillageCode {
                name: "界坪村委会",
                code: "009",
            },
            VillageCode {
                name: "来登村委会",
                code: "010",
            },
            VillageCode {
                name: "大竹箐村委会",
                code: "011",
            },
            VillageCode {
                name: "回龙村委会",
                code: "012",
            },
            VillageCode {
                name: "庄河村委会",
                code: "013",
            },
            VillageCode {
                name: "仁甸河村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "兔峨乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "永福社区居委会",
                code: "001",
            },
            VillageCode {
                name: "兔峨村委会",
                code: "002",
            },
            VillageCode {
                name: "阿塔登村委会",
                code: "003",
            },
            VillageCode {
                name: "腊马登村委会",
                code: "004",
            },
            VillageCode {
                name: "丰甸村委会",
                code: "005",
            },
            VillageCode {
                name: "江末村委会",
                code: "006",
            },
            VillageCode {
                name: "果力村委会",
                code: "007",
            },
            VillageCode {
                name: "大华村委会",
                code: "008",
            },
            VillageCode {
                name: "扎局村委会",
                code: "009",
            },
            VillageCode {
                name: "吾马普村委会",
                code: "010",
            },
            VillageCode {
                name: "迤场村委会",
                code: "011",
            },
            VillageCode {
                name: "花坪村委会",
                code: "012",
            },
            VillageCode {
                name: "石坪村委会",
                code: "013",
            },
            VillageCode {
                name: "大村头村委会",
                code: "014",
            },
            VillageCode {
                name: "大麦地村委会",
                code: "015",
            },
        ],
    },
];

static TOWNS_XK_007: [TownCode; 15] = [
    TownCode {
        name: "城关镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "嘎东街居委会",
                code: "001",
            },
            VillageCode {
                name: "帮达街居委会",
                code: "002",
            },
            VillageCode {
                name: "四川桥居委会",
                code: "003",
            },
            VillageCode {
                name: "启赤街社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "达吉街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "马草坝居委会",
                code: "006",
            },
            VillageCode {
                name: "夏通街居委会",
                code: "007",
            },
            VillageCode {
                name: "昌庆街居委会",
                code: "008",
            },
            VillageCode {
                name: "卧龙街居委会",
                code: "009",
            },
            VillageCode {
                name: "德吉社区居委会",
                code: "010",
            },
            VillageCode {
                name: "利民社区居委会",
                code: "011",
            },
            VillageCode {
                name: "小恩达村村委会",
                code: "012",
            },
            VillageCode {
                name: "白格村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "生格村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "生达村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "通夏村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "野堆村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "达普村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "格地村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "达瓦村村民委员会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "俄洛镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "俄洛村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "珠古村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "郎达村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "郭穷村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "约达村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "孔玛村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "格巴村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "沙通村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "仁达村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "加林村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "曲尼村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "雄达村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "卡若镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "休索村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "加卡村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "瓦约村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "波妥村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "卡若村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "左巴村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "达修村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "多然村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "达布村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "波乃村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "芒达乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "西强村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "达德村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "莫堆村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "白措村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "莫美村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "扎玛村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "芒达村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "委日村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "索土村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "左巴村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "瓦洛村村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "约巴乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "玉来村村委会",
                code: "001",
            },
            VillageCode {
                name: "玛日村村委会",
                code: "002",
            },
            VillageCode {
                name: "嘎然村村委会",
                code: "003",
            },
            VillageCode {
                name: "乃通村村委会",
                code: "004",
            },
            VillageCode {
                name: "唐卡村村委会",
                code: "005",
            },
            VillageCode {
                name: "拉美村村委会",
                code: "006",
            },
            VillageCode {
                name: "达村村委会",
                code: "007",
            },
            VillageCode {
                name: "巴洛村村委会",
                code: "008",
            },
            VillageCode {
                name: "拉日村村委会",
                code: "009",
            },
            VillageCode {
                name: "约俄村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "妥坝乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "珠古村村委会",
                code: "001",
            },
            VillageCode {
                name: "妥坝村村委会",
                code: "002",
            },
            VillageCode {
                name: "夏雅村村委会",
                code: "003",
            },
            VillageCode {
                name: "乐瓦村村委会",
                code: "004",
            },
            VillageCode {
                name: "热霍村村委会",
                code: "005",
            },
            VillageCode {
                name: "康巴村村委会",
                code: "006",
            },
            VillageCode {
                name: "诺玛村村委会",
                code: "007",
            },
            VillageCode {
                name: "珍嘎村村委会",
                code: "008",
            },
            VillageCode {
                name: "龙亚村村委会",
                code: "009",
            },
            VillageCode {
                name: "杂庆村村委会",
                code: "010",
            },
            VillageCode {
                name: "然索村村委会",
                code: "011",
            },
            VillageCode {
                name: "然达村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "拉多乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "恰龙村村委会",
                code: "001",
            },
            VillageCode {
                name: "瓦措村村委会",
                code: "002",
            },
            VillageCode {
                name: "贡西村村委会",
                code: "003",
            },
            VillageCode {
                name: "曲色村村委会",
                code: "004",
            },
            VillageCode {
                name: "塔玛村村委会",
                code: "005",
            },
            VillageCode {
                name: "康多村村委会",
                code: "006",
            },
            VillageCode {
                name: "达日村村委会",
                code: "007",
            },
            VillageCode {
                name: "夏日村村委会",
                code: "008",
            },
            VillageCode {
                name: "巴郭村村委会",
                code: "009",
            },
            VillageCode {
                name: "嘎来村村委会",
                code: "010",
            },
            VillageCode {
                name: "西那村村委会",
                code: "011",
            },
            VillageCode {
                name: "达多村村委会",
                code: "012",
            },
            VillageCode {
                name: "嘎扣村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "面达乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "玛左村村委会",
                code: "001",
            },
            VillageCode {
                name: "冷达村村委会",
                code: "002",
            },
            VillageCode {
                name: "崩热村村委会",
                code: "003",
            },
            VillageCode {
                name: "措荣村村委会",
                code: "004",
            },
            VillageCode {
                name: "诺通村村委会",
                code: "005",
            },
            VillageCode {
                name: "热索村村委会",
                code: "006",
            },
            VillageCode {
                name: "果帕村村委会",
                code: "007",
            },
            VillageCode {
                name: "格杂村村委会",
                code: "008",
            },
            VillageCode {
                name: "巴通村村委会",
                code: "009",
            },
            VillageCode {
                name: "字多村村委会",
                code: "010",
            },
            VillageCode {
                name: "莫巴村村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "嘎玛乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "瓦孜村村委会",
                code: "001",
            },
            VillageCode {
                name: "查拉村村委会",
                code: "002",
            },
            VillageCode {
                name: "当土村村委会",
                code: "003",
            },
            VillageCode {
                name: "鸟东村村委会",
                code: "004",
            },
            VillageCode {
                name: "江委村村委会",
                code: "005",
            },
            VillageCode {
                name: "也多村村委会",
                code: "006",
            },
            VillageCode {
                name: "香达村村委会",
                code: "007",
            },
            VillageCode {
                name: "达那村村委会",
                code: "008",
            },
            VillageCode {
                name: "嘎玛村村委会",
                code: "009",
            },
            VillageCode {
                name: "里土村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "柴维乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "金通村村委会",
                code: "001",
            },
            VillageCode {
                name: "加荣村村委会",
                code: "002",
            },
            VillageCode {
                name: "翁达岗村村委会",
                code: "003",
            },
            VillageCode {
                name: "多雄村村委会",
                code: "004",
            },
            VillageCode {
                name: "古强村村委会",
                code: "005",
            },
            VillageCode {
                name: "差达村村委会",
                code: "006",
            },
            VillageCode {
                name: "格瓦村村委会",
                code: "007",
            },
            VillageCode {
                name: "柴维村村委会",
                code: "008",
            },
            VillageCode {
                name: "多拉多村村委会",
                code: "009",
            },
            VillageCode {
                name: "嘎日村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "日通乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "果吉村村委会",
                code: "001",
            },
            VillageCode {
                name: "温达村村委会",
                code: "002",
            },
            VillageCode {
                name: "布妥村村委会",
                code: "003",
            },
            VillageCode {
                name: "达东村村委会",
                code: "004",
            },
            VillageCode {
                name: "尼追村村委会",
                code: "005",
            },
            VillageCode {
                name: "香达村村委会",
                code: "006",
            },
            VillageCode {
                name: "冻多村村委会",
                code: "007",
            },
            VillageCode {
                name: "肖堆村村委会",
                code: "008",
            },
            VillageCode {
                name: "瓦列村村委会",
                code: "009",
            },
            VillageCode {
                name: "列沙村村委会",
                code: "010",
            },
            VillageCode {
                name: "日通村村委会",
                code: "011",
            },
            VillageCode {
                name: "雄达村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "如意乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "杜嘎村村委会",
                code: "001",
            },
            VillageCode {
                name: "桑多村村委会",
                code: "002",
            },
            VillageCode {
                name: "桑那村村委会",
                code: "003",
            },
            VillageCode {
                name: "如意村村委会",
                code: "004",
            },
            VillageCode {
                name: "永嘎村村委会",
                code: "005",
            },
            VillageCode {
                name: "约日村村委会",
                code: "006",
            },
            VillageCode {
                name: "达若村村委会",
                code: "007",
            },
            VillageCode {
                name: "普然村村委会",
                code: "008",
            },
            VillageCode {
                name: "桑恩村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "埃西乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "漠巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "向尼村村委会",
                code: "002",
            },
            VillageCode {
                name: "达青村村委会",
                code: "003",
            },
            VillageCode {
                name: "岗村村委会",
                code: "004",
            },
            VillageCode {
                name: "邦迪村村委会",
                code: "005",
            },
            VillageCode {
                name: "亚玛村村委会",
                code: "006",
            },
            VillageCode {
                name: "热亚村村委会",
                code: "007",
            },
            VillageCode {
                name: "娘达村村委会",
                code: "008",
            },
            VillageCode {
                name: "哈拉村村委会",
                code: "009",
            },
            VillageCode {
                name: "蒙普村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "若巴乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "格岗村村委会",
                code: "001",
            },
            VillageCode {
                name: "若尼村村委会",
                code: "002",
            },
            VillageCode {
                name: "郭那村村委会",
                code: "003",
            },
            VillageCode {
                name: "嘎达村村委会",
                code: "004",
            },
            VillageCode {
                name: "瓦扎村村委会",
                code: "005",
            },
            VillageCode {
                name: "博巴村村委会",
                code: "006",
            },
            VillageCode {
                name: "扎格村村委会",
                code: "007",
            },
            VillageCode {
                name: "卡堆村村委会",
                code: "008",
            },
            VillageCode {
                name: "香宗村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "沙贡乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "小土村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "达东村村委会",
                code: "002",
            },
            VillageCode {
                name: "莫仲村村委会",
                code: "003",
            },
            VillageCode {
                name: "穷卡村村委会",
                code: "004",
            },
            VillageCode {
                name: "温达村村委会",
                code: "005",
            },
            VillageCode {
                name: "格秀村村委会",
                code: "006",
            },
            VillageCode {
                name: "卡洛村村委会",
                code: "007",
            },
            VillageCode {
                name: "约宗村村委会",
                code: "008",
            },
            VillageCode {
                name: "多普村村委会",
                code: "009",
            },
            VillageCode {
                name: "果洛村村委会",
                code: "010",
            },
        ],
    },
];

static TOWNS_XK_008: [TownCode; 13] = [
    TownCode {
        name: "江达镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "新林社区居委会",
                code: "001",
            },
            VillageCode {
                name: "聚康社区居委会",
                code: "002",
            },
            VillageCode {
                name: "瓦许村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "嘎四村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "敏达村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "麦冬村村委会",
                code: "006",
            },
            VillageCode {
                name: "江达村村委会",
                code: "007",
            },
            VillageCode {
                name: "嘎通村村委会",
                code: "008",
            },
            VillageCode {
                name: "岗达村村委会",
                code: "009",
            },
            VillageCode {
                name: "汪巴村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "岗托镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "矮拉村村委会",
                code: "001",
            },
            VillageCode {
                name: "岗托村村委会",
                code: "002",
            },
            VillageCode {
                name: "作如村村委会",
                code: "003",
            },
            VillageCode {
                name: "岩巴村村委会",
                code: "004",
            },
            VillageCode {
                name: "航格村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "卡贡乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "色沙村村委会",
                code: "001",
            },
            VillageCode {
                name: "车所村村委会",
                code: "002",
            },
            VillageCode {
                name: "达色村村委会",
                code: "003",
            },
            VillageCode {
                name: "卡贡村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "岩比乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "德巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "岩比村村委会",
                code: "002",
            },
            VillageCode {
                name: "华荣村村委会",
                code: "003",
            },
            VillageCode {
                name: "东扎村村委会",
                code: "004",
            },
            VillageCode {
                name: "杂拥村村委会",
                code: "005",
            },
            VillageCode {
                name: "格达村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "邓柯乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "青稞村村委会",
                code: "001",
            },
            VillageCode {
                name: "沙嘎村村委会",
                code: "002",
            },
            VillageCode {
                name: "直巴村村委会",
                code: "003",
            },
            VillageCode {
                name: "色日村村委会",
                code: "004",
            },
            VillageCode {
                name: "巴龙村村委会",
                code: "005",
            },
            VillageCode {
                name: "郎吉村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "生达乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "格宗村村委会",
                code: "001",
            },
            VillageCode {
                name: "宁邦村村委会",
                code: "002",
            },
            VillageCode {
                name: "布特村村委会",
                code: "003",
            },
            VillageCode {
                name: "洛玛村村委会",
                code: "004",
            },
            VillageCode {
                name: "色巴村村委会",
                code: "005",
            },
            VillageCode {
                name: "鲁色村村委会",
                code: "006",
            },
            VillageCode {
                name: "拉池村村委会",
                code: "007",
            },
            VillageCode {
                name: "仁达村村委会",
                code: "008",
            },
            VillageCode {
                name: "日崩村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "娘西乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "古巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "加桑村村委会",
                code: "002",
            },
            VillageCode {
                name: "嘎玖村村委会",
                code: "003",
            },
            VillageCode {
                name: "瓦根村村委会",
                code: "004",
            },
            VillageCode {
                name: "强白村村委会",
                code: "005",
            },
            VillageCode {
                name: "帮达村村委会",
                code: "006",
            },
            VillageCode {
                name: "山岩村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "字呷乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "日木村村委会",
                code: "001",
            },
            VillageCode {
                name: "上格色村村委会",
                code: "002",
            },
            VillageCode {
                name: "下格色村村委会",
                code: "003",
            },
            VillageCode {
                name: "上白玛村村委会",
                code: "004",
            },
            VillageCode {
                name: "下白玛村村委会",
                code: "005",
            },
            VillageCode {
                name: "上格日贡村村委会",
                code: "006",
            },
            VillageCode {
                name: "下格日贡村村委会",
                code: "007",
            },
            VillageCode {
                name: "塔字村村委会",
                code: "008",
            },
            VillageCode {
                name: "支巴村村委会",
                code: "009",
            },
            VillageCode {
                name: "字呷村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "青泥洞乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "索日村村委会",
                code: "001",
            },
            VillageCode {
                name: "巴纳村村委会",
                code: "002",
            },
            VillageCode {
                name: "觉拥村村委会",
                code: "003",
            },
            VillageCode {
                name: "热拥村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "汪布顶乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "查格村村委会",
                code: "001",
            },
            VillageCode {
                name: "卡松多村村委会",
                code: "002",
            },
            VillageCode {
                name: "汪布顶村村委会",
                code: "003",
            },
            VillageCode {
                name: "然灯村村委会",
                code: "004",
            },
            VillageCode {
                name: "来玛村村委会",
                code: "005",
            },
            VillageCode {
                name: "卓格村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "德登乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "仁真都村村委会",
                code: "001",
            },
            VillageCode {
                name: "吉然村村委会",
                code: "002",
            },
            VillageCode {
                name: "地普村村委会",
                code: "003",
            },
            VillageCode {
                name: "门莫村村委会",
                code: "004",
            },
            VillageCode {
                name: "夏吉村村委会",
                code: "005",
            },
            VillageCode {
                name: "鲁格村村委会",
                code: "006",
            },
            VillageCode {
                name: "措日村村委会",
                code: "007",
            },
            VillageCode {
                name: "梦青村村委会",
                code: "008",
            },
            VillageCode {
                name: "神青龙村村委会",
                code: "009",
            },
            VillageCode {
                name: "嘎戎村村委会",
                code: "010",
            },
            VillageCode {
                name: "多堆村村委会",
                code: "011",
            },
            VillageCode {
                name: "外青村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "同普乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "格巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "夏荣村村委会",
                code: "002",
            },
            VillageCode {
                name: "木巴村村委会",
                code: "003",
            },
            VillageCode {
                name: "吉巴村村委会",
                code: "004",
            },
            VillageCode {
                name: "瓦足村村委会",
                code: "005",
            },
            VillageCode {
                name: "娘麦村村委会",
                code: "006",
            },
            VillageCode {
                name: "东斗村村委会",
                code: "007",
            },
            VillageCode {
                name: "江巴村村委会",
                code: "008",
            },
            VillageCode {
                name: "格亚村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "波罗乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "古色村村委会",
                code: "001",
            },
            VillageCode {
                name: "冲桑村村委会",
                code: "002",
            },
            VillageCode {
                name: "外冲村村委会",
                code: "003",
            },
            VillageCode {
                name: "阿当村村委会",
                code: "004",
            },
            VillageCode {
                name: "热多村村委会",
                code: "005",
            },
            VillageCode {
                name: "俄彭村村委会",
                code: "006",
            },
            VillageCode {
                name: "宁巴村村委会",
                code: "007",
            },
            VillageCode {
                name: "波公村村委会",
                code: "008",
            },
        ],
    },
];

static TOWNS_XK_009: [TownCode; 12] = [
    TownCode {
        name: "莫洛镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "登卡社区居委会",
                code: "001",
            },
            VillageCode {
                name: "扎西社区居委会",
                code: "002",
            },
            VillageCode {
                name: "贡桑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "林通村村委会",
                code: "004",
            },
            VillageCode {
                name: "莫洛村村委会",
                code: "005",
            },
            VillageCode {
                name: "果普村村委会",
                code: "006",
            },
            VillageCode {
                name: "查雄普村村委会",
                code: "007",
            },
            VillageCode {
                name: "若果村村委会",
                code: "008",
            },
            VillageCode {
                name: "爱玉村村委会",
                code: "009",
            },
            VillageCode {
                name: "洞托村村委会",
                code: "010",
            },
            VillageCode {
                name: "贡中村村委会",
                code: "011",
            },
            VillageCode {
                name: "夏日村村委会",
                code: "012",
            },
            VillageCode {
                name: "米来村村委会",
                code: "013",
            },
            VillageCode {
                name: "达龙村村委会",
                code: "014",
            },
            VillageCode {
                name: "丈中村村委会",
                code: "015",
            },
            VillageCode {
                name: "苦达村村委会",
                code: "016",
            },
            VillageCode {
                name: "泽仁本村村委会",
                code: "017",
            },
            VillageCode {
                name: "卡托村村委会",
                code: "018",
            },
            VillageCode {
                name: "色然村村委会",
                code: "019",
            },
            VillageCode {
                name: "多吉村村委会",
                code: "020",
            },
            VillageCode {
                name: "根当村村委会",
                code: "021",
            },
            VillageCode {
                name: "来日玛村村委会",
                code: "022",
            },
            VillageCode {
                name: "阿卡村村委会",
                code: "023",
            },
            VillageCode {
                name: "拉玛村村委会",
                code: "024",
            },
            VillageCode {
                name: "觉龙村村委会",
                code: "025",
            },
            VillageCode {
                name: "曲松村村委会",
                code: "026",
            },
            VillageCode {
                name: "帮措村村委会",
                code: "027",
            },
            VillageCode {
                name: "插托村村委会",
                code: "028",
            },
            VillageCode {
                name: "俄底村村委会",
                code: "029",
            },
            VillageCode {
                name: "德麦村村委会",
                code: "030",
            },
            VillageCode {
                name: "阿果村村委会",
                code: "031",
            },
            VillageCode {
                name: "拉荣村村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "相皮乡",
        code: "002",
        villages: &[
            VillageCode {
                name: "查然村村委会",
                code: "001",
            },
            VillageCode {
                name: "左玉村村委会",
                code: "002",
            },
            VillageCode {
                name: "左堆村村委会",
                code: "003",
            },
            VillageCode {
                name: "曲麦村村委会",
                code: "004",
            },
            VillageCode {
                name: "普雄村村委会",
                code: "005",
            },
            VillageCode {
                name: "宋西村村委会",
                code: "006",
            },
            VillageCode {
                name: "洛巴村村委会",
                code: "007",
            },
            VillageCode {
                name: "相皮村村委会",
                code: "008",
            },
            VillageCode {
                name: "解放村村委会",
                code: "009",
            },
            VillageCode {
                name: "夏如村村委会",
                code: "010",
            },
            VillageCode {
                name: "曲贡村村委会",
                code: "011",
            },
            VillageCode {
                name: "巴普村村委会",
                code: "012",
            },
            VillageCode {
                name: "桑珠荣亚中村村委会",
                code: "013",
            },
            VillageCode {
                name: "桑珠荣玛中村村委会",
                code: "014",
            },
            VillageCode {
                name: "嘎托村村委会",
                code: "015",
            },
            VillageCode {
                name: "麦东村村委会",
                code: "016",
            },
            VillageCode {
                name: "扎特村村委会",
                code: "017",
            },
            VillageCode {
                name: "然龙村村委会",
                code: "018",
            },
            VillageCode {
                name: "孜荣村村委会",
                code: "019",
            },
            VillageCode {
                name: "杂空顶村村委会",
                code: "020",
            },
            VillageCode {
                name: "然玛多村村委会",
                code: "021",
            },
            VillageCode {
                name: "郎果村村委会",
                code: "022",
            },
            VillageCode {
                name: "色嘎村村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "哈加乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "永果村村委会",
                code: "001",
            },
            VillageCode {
                name: "油扎牧场村委会",
                code: "002",
            },
            VillageCode {
                name: "娘列村村委会",
                code: "003",
            },
            VillageCode {
                name: "亚玛中村村委会",
                code: "004",
            },
            VillageCode {
                name: "普孜村村委会",
                code: "005",
            },
            VillageCode {
                name: "边巴村村委会",
                code: "006",
            },
            VillageCode {
                name: "果布村村委会",
                code: "007",
            },
            VillageCode {
                name: "宗布村村委会",
                code: "008",
            },
            VillageCode {
                name: "巴拉牧场村委会",
                code: "009",
            },
            VillageCode {
                name: "哈加村村委会",
                code: "010",
            },
            VillageCode {
                name: "曲卡村村委会",
                code: "011",
            },
            VillageCode {
                name: "马荣村村委会",
                code: "012",
            },
            VillageCode {
                name: "嘎空村村委会",
                code: "013",
            },
            VillageCode {
                name: "果托村村委会",
                code: "014",
            },
            VillageCode {
                name: "孟达村村委会",
                code: "015",
            },
            VillageCode {
                name: "曲登村村委会",
                code: "016",
            },
            VillageCode {
                name: "多坝村村委会",
                code: "017",
            },
            VillageCode {
                name: "相崩村村委会",
                code: "018",
            },
            VillageCode {
                name: "加热村村委会",
                code: "019",
            },
            VillageCode {
                name: "迥然村村委会",
                code: "020",
            },
            VillageCode {
                name: "阿洛村村委会",
                code: "021",
            },
            VillageCode {
                name: "连达村村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "雄松乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "加卡村村委会",
                code: "001",
            },
            VillageCode {
                name: "巴洛村村委会",
                code: "002",
            },
            VillageCode {
                name: "夏亚村村委会",
                code: "003",
            },
            VillageCode {
                name: "德村村委会",
                code: "004",
            },
            VillageCode {
                name: "上缺所村村委会",
                code: "005",
            },
            VillageCode {
                name: "下缺所村村委会",
                code: "006",
            },
            VillageCode {
                name: "岗托村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "拉妥乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "达松村村委会",
                code: "001",
            },
            VillageCode {
                name: "巴林村村委会",
                code: "002",
            },
            VillageCode {
                name: "措西村村委会",
                code: "003",
            },
            VillageCode {
                name: "塔林村村委会",
                code: "004",
            },
            VillageCode {
                name: "拉玛村村委会",
                code: "005",
            },
            VillageCode {
                name: "鲁杰村村委会",
                code: "006",
            },
            VillageCode {
                name: "拉德村村委会",
                code: "007",
            },
            VillageCode {
                name: "罗玛村村委会",
                code: "008",
            },
            VillageCode {
                name: "宗巴村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "阿旺乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "金珠村村委会",
                code: "001",
            },
            VillageCode {
                name: "多热村村委会",
                code: "002",
            },
            VillageCode {
                name: "通奎村村委会",
                code: "003",
            },
            VillageCode {
                name: "扎龙村村委会",
                code: "004",
            },
            VillageCode {
                name: "颂庆村村委会",
                code: "005",
            },
            VillageCode {
                name: "拉果村村委会",
                code: "006",
            },
            VillageCode {
                name: "冬青村村委会",
                code: "007",
            },
            VillageCode {
                name: "那玉村村委会",
                code: "008",
            },
            VillageCode {
                name: "莫农村村委会",
                code: "009",
            },
            VillageCode {
                name: "多扎村村委会",
                code: "010",
            },
            VillageCode {
                name: "维多村村委会",
                code: "011",
            },
            VillageCode {
                name: "东如村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "木协乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "木协村村委会",
                code: "001",
            },
            VillageCode {
                name: "则达村村委会",
                code: "002",
            },
            VillageCode {
                name: "拉巴村村委会",
                code: "003",
            },
            VillageCode {
                name: "也古村村委会",
                code: "004",
            },
            VillageCode {
                name: "上罗娘村村委会",
                code: "005",
            },
            VillageCode {
                name: "下罗娘村村委会",
                code: "006",
            },
            VillageCode {
                name: "类乌西村村委会",
                code: "007",
            },
            VillageCode {
                name: "党学村村委会",
                code: "008",
            },
            VillageCode {
                name: "康布村村委会",
                code: "009",
            },
            VillageCode {
                name: "果木村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "罗麦乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "罗麦村村委会",
                code: "001",
            },
            VillageCode {
                name: "龙旺村村委会",
                code: "002",
            },
            VillageCode {
                name: "列特村村委会",
                code: "003",
            },
            VillageCode {
                name: "古巴村村委会",
                code: "004",
            },
            VillageCode {
                name: "色扎村村委会",
                code: "005",
            },
            VillageCode {
                name: "从昌村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "沙东乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "兰因村村委会",
                code: "001",
            },
            VillageCode {
                name: "果麦村村委会",
                code: "002",
            },
            VillageCode {
                name: "布堆村村委会",
                code: "003",
            },
            VillageCode {
                name: "格果村村委会",
                code: "004",
            },
            VillageCode {
                name: "雄巴村村委会",
                code: "005",
            },
            VillageCode {
                name: "阿香村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "克日乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "莫扎村村委会",
                code: "001",
            },
            VillageCode {
                name: "冲录村村委会",
                code: "002",
            },
            VillageCode {
                name: "西西村村委会",
                code: "003",
            },
            VillageCode {
                name: "克日村村委会",
                code: "004",
            },
            VillageCode {
                name: "登巴村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "则巴乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "则普村村委会",
                code: "001",
            },
            VillageCode {
                name: "则麦村村委会",
                code: "002",
            },
            VillageCode {
                name: "瓦堆村村委会",
                code: "003",
            },
            VillageCode {
                name: "嘎色村村委会",
                code: "004",
            },
            VillageCode {
                name: "哈池村村委会",
                code: "005",
            },
            VillageCode {
                name: "朗日村村委会",
                code: "006",
            },
            VillageCode {
                name: "果龙村村委会",
                code: "007",
            },
            VillageCode {
                name: "卫通村村委会",
                code: "008",
            },
            VillageCode {
                name: "达曲村村委会",
                code: "009",
            },
            VillageCode {
                name: "夏日村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "敏都乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "敏都村村委会",
                code: "001",
            },
            VillageCode {
                name: "卡巴村村委会",
                code: "002",
            },
            VillageCode {
                name: "雄果村村委会",
                code: "003",
            },
            VillageCode {
                name: "麦巴村村委会",
                code: "004",
            },
            VillageCode {
                name: "果巴村村委会",
                code: "005",
            },
            VillageCode {
                name: "贡巴村村委会",
                code: "006",
            },
            VillageCode {
                name: "瓦堆村村委会",
                code: "007",
            },
            VillageCode {
                name: "马觉村村委会",
                code: "008",
            },
        ],
    },
];

static TOWNS_XK_010: [TownCode; 10] = [
    TownCode {
        name: "类乌齐镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "觉恩村村委会",
                code: "001",
            },
            VillageCode {
                name: "尼扎村村委会",
                code: "002",
            },
            VillageCode {
                name: "金达卡村村委会",
                code: "003",
            },
            VillageCode {
                name: "达郭村村委会",
                code: "004",
            },
            VillageCode {
                name: "宗龙村村委会",
                code: "005",
            },
            VillageCode {
                name: "君达村村委会",
                code: "006",
            },
            VillageCode {
                name: "扎日村村委会",
                code: "007",
            },
            VillageCode {
                name: "孟达村村委会",
                code: "008",
            },
            VillageCode {
                name: "香迁村村委会",
                code: "009",
            },
            VillageCode {
                name: "扎沙村村委会",
                code: "010",
            },
            VillageCode {
                name: "新热村村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "桑多镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "巴仁巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "冬孜巷社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "扎西居委会",
                code: "003",
            },
            VillageCode {
                name: "德勒居委会",
                code: "004",
            },
            VillageCode {
                name: "恩达村村委会",
                code: "005",
            },
            VillageCode {
                name: "扎西贡村村委会",
                code: "006",
            },
            VillageCode {
                name: "桑多村村委会",
                code: "007",
            },
            VillageCode {
                name: "达日通村村委会",
                code: "008",
            },
            VillageCode {
                name: "生格贡村村委会",
                code: "009",
            },
            VillageCode {
                name: "贺日村村委会",
                code: "010",
            },
            VillageCode {
                name: "热扎卡村村委会",
                code: "011",
            },
            VillageCode {
                name: "扎通卡村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "加桑卡乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "桑卡村村委会",
                code: "001",
            },
            VillageCode {
                name: "堆瓦村村委会",
                code: "002",
            },
            VillageCode {
                name: "瓦日村村委会",
                code: "003",
            },
            VillageCode {
                name: "边普村村委会",
                code: "004",
            },
            VillageCode {
                name: "国瓦村村委会",
                code: "005",
            },
            VillageCode {
                name: "乌然村村委会",
                code: "006",
            },
            VillageCode {
                name: "东登卡村村委会",
                code: "007",
            },
            VillageCode {
                name: "康沙村村委会",
                code: "008",
            },
            VillageCode {
                name: "孟卡村村委会",
                code: "009",
            },
            VillageCode {
                name: "美才村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "长毛岭乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "木尺村村委会",
                code: "001",
            },
            VillageCode {
                name: "珠江村村委会",
                code: "002",
            },
            VillageCode {
                name: "学塔村村委会",
                code: "003",
            },
            VillageCode {
                name: "岗塔村村委会",
                code: "004",
            },
            VillageCode {
                name: "普穷村村委会",
                code: "005",
            },
            VillageCode {
                name: "德塔村村委会",
                code: "006",
            },
            VillageCode {
                name: "岗雄村村委会",
                code: "007",
            },
            VillageCode {
                name: "沙尼村村委会",
                code: "008",
            },
            VillageCode {
                name: "达雄村村委会",
                code: "009",
            },
            VillageCode {
                name: "龙桑村村委会",
                code: "010",
            },
            VillageCode {
                name: "协塘村村委会",
                code: "011",
            },
            VillageCode {
                name: "岗格村村委会",
                code: "012",
            },
            VillageCode {
                name: "贡达村村委会",
                code: "013",
            },
            VillageCode {
                name: "曲格村村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "岗色乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "比苍村村委会",
                code: "001",
            },
            VillageCode {
                name: "居美村村委会",
                code: "002",
            },
            VillageCode {
                name: "彭雪村村委会",
                code: "003",
            },
            VillageCode {
                name: "岗达村村委会",
                code: "004",
            },
            VillageCode {
                name: "拉恩村村委会",
                code: "005",
            },
            VillageCode {
                name: "马曲村村委会",
                code: "006",
            },
            VillageCode {
                name: "岗穷村村委会",
                code: "007",
            },
            VillageCode {
                name: "多苏村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "吉多乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "达如村村委会",
                code: "001",
            },
            VillageCode {
                name: "格然多村村委会",
                code: "002",
            },
            VillageCode {
                name: "阿珠村村委会",
                code: "003",
            },
            VillageCode {
                name: "达孜村村委会",
                code: "004",
            },
            VillageCode {
                name: "香巴村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "滨达乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "滨达村村委会",
                code: "001",
            },
            VillageCode {
                name: "热西村村委会",
                code: "002",
            },
            VillageCode {
                name: "央宗村村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "卡玛多乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "吉青村村委会",
                code: "001",
            },
            VillageCode {
                name: "井林村村委会",
                code: "002",
            },
            VillageCode {
                name: "郭龙村村委会",
                code: "003",
            },
            VillageCode {
                name: "嘎吉村村委会",
                code: "004",
            },
            VillageCode {
                name: "协玛村村委会",
                code: "005",
            },
            VillageCode {
                name: "拉龙村村委会",
                code: "006",
            },
            VillageCode {
                name: "卡玛多村村委会",
                code: "007",
            },
            VillageCode {
                name: "帮嘎村村委会",
                code: "008",
            },
            VillageCode {
                name: "哲龙村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "尚卡乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "然爱村村委会",
                code: "001",
            },
            VillageCode {
                name: "珠多村村委会",
                code: "002",
            },
            VillageCode {
                name: "尚卡村村委会",
                code: "003",
            },
            VillageCode {
                name: "尚日村村委会",
                code: "004",
            },
            VillageCode {
                name: "吉多村村委会",
                code: "005",
            },
            VillageCode {
                name: "索村村委会",
                code: "006",
            },
            VillageCode {
                name: "达拉村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "伊日乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "伊多村村委会",
                code: "001",
            },
            VillageCode {
                name: "帮日村村委会",
                code: "002",
            },
            VillageCode {
                name: "珠达村村委会",
                code: "003",
            },
            VillageCode {
                name: "亚中村村委会",
                code: "004",
            },
            VillageCode {
                name: "崩日村村委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_XK_011: [TownCode; 13] = [
    TownCode {
        name: "丁青镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "丁青社区居委会",
                code: "001",
            },
            VillageCode {
                name: "杂旭居委会",
                code: "002",
            },
            VillageCode {
                name: "丁青村村委会",
                code: "003",
            },
            VillageCode {
                name: "茶龙村村委会",
                code: "004",
            },
            VillageCode {
                name: "色康村村委会",
                code: "005",
            },
            VillageCode {
                name: "热昌村村委会",
                code: "006",
            },
            VillageCode {
                name: "仲白村村委会",
                code: "007",
            },
            VillageCode {
                name: "布托村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "尺犊镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "瓦河村村委会",
                code: "001",
            },
            VillageCode {
                name: "上依村村委会",
                code: "002",
            },
            VillageCode {
                name: "俄列村村委会",
                code: "003",
            },
            VillageCode {
                name: "玛色村村委会",
                code: "004",
            },
            VillageCode {
                name: "索果村村委会",
                code: "005",
            },
            VillageCode {
                name: "乌巴村村委会",
                code: "006",
            },
            VillageCode {
                name: "巴格村村委会",
                code: "007",
            },
            VillageCode {
                name: "巴登村村委会",
                code: "008",
            },
            VillageCode {
                name: "汝桑村村委会",
                code: "009",
            },
            VillageCode {
                name: "瓦巴村村委会",
                code: "010",
            },
            VillageCode {
                name: "迪巴村村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "觉恩乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "达旭村村委会",
                code: "001",
            },
            VillageCode {
                name: "觉恩村村委会",
                code: "002",
            },
            VillageCode {
                name: "巴河村村委会",
                code: "003",
            },
            VillageCode {
                name: "卡龙村村委会",
                code: "004",
            },
            VillageCode {
                name: "金卡村村委会",
                code: "005",
            },
            VillageCode {
                name: "绒通村村委会",
                code: "006",
            },
            VillageCode {
                name: "麦日村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "沙贡乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "然强村村委会",
                code: "001",
            },
            VillageCode {
                name: "沙贡村村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "当堆乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "依塔西村村委会",
                code: "001",
            },
            VillageCode {
                name: "当堆村村委会",
                code: "002",
            },
            VillageCode {
                name: "斯熔村村委会",
                code: "003",
            },
            VillageCode {
                name: "洛霍村村委会",
                code: "004",
            },
            VillageCode {
                name: "白日村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "桑多乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "桑多村村委会",
                code: "001",
            },
            VillageCode {
                name: "郡休村村委会",
                code: "002",
            },
            VillageCode {
                name: "安拉村村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "木塔乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "木塔村村委会",
                code: "001",
            },
            VillageCode {
                name: "羊塔村村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "布塔乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "布塔村村委会",
                code: "001",
            },
            VillageCode {
                name: "日塔村村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "巴达乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "达麦村村委会",
                code: "001",
            },
            VillageCode {
                name: "格巴村村委会",
                code: "002",
            },
            VillageCode {
                name: "达堆村村委会",
                code: "003",
            },
            VillageCode {
                name: "巴巴村村委会",
                code: "004",
            },
            VillageCode {
                name: "波巴村村委会",
                code: "005",
            },
            VillageCode {
                name: "邮巴村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "甘岩乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "岩堆村村委会",
                code: "001",
            },
            VillageCode {
                name: "色达村村委会",
                code: "002",
            },
            VillageCode {
                name: "甘岩村村委会",
                code: "003",
            },
            VillageCode {
                name: "卡崩村村委会",
                code: "004",
            },
            VillageCode {
                name: "布堆村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "嘎塔乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "相扎村村委会",
                code: "001",
            },
            VillageCode {
                name: "贡日村村委会",
                code: "002",
            },
            VillageCode {
                name: "江塔村村委会",
                code: "003",
            },
            VillageCode {
                name: "嘎塔村村委会",
                code: "004",
            },
            VillageCode {
                name: "果东村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "色扎乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "色扎村村委会",
                code: "001",
            },
            VillageCode {
                name: "汝化村村委会",
                code: "002",
            },
            VillageCode {
                name: "贡桑村村委会",
                code: "003",
            },
            VillageCode {
                name: "卡通村村委会",
                code: "004",
            },
            VillageCode {
                name: "索巴村村委会",
                code: "005",
            },
            VillageCode {
                name: "木查村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "协雄乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "穹娜村村委会",
                code: "001",
            },
            VillageCode {
                name: "协堆村村委会",
                code: "002",
            },
            VillageCode {
                name: "协雄村村委会",
                code: "003",
            },
            VillageCode {
                name: "协麦村村委会",
                code: "004",
            },
            VillageCode {
                name: "朗通村村委会",
                code: "005",
            },
            VillageCode {
                name: "夏拉村村委会",
                code: "006",
            },
        ],
    },
];

static TOWNS_XK_012: [TownCode; 13] = [
    TownCode {
        name: "烟多镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "烟多居委会",
                code: "001",
            },
            VillageCode {
                name: "幸福重庆新村村委会",
                code: "002",
            },
            VillageCode {
                name: "中铝新村村委会",
                code: "003",
            },
            VillageCode {
                name: "色嘎村村委会",
                code: "004",
            },
            VillageCode {
                name: "帮嘎村村委会",
                code: "005",
            },
            VillageCode {
                name: "帮隆村村委会",
                code: "006",
            },
            VillageCode {
                name: "达巴村村委会",
                code: "007",
            },
            VillageCode {
                name: "奶奎村村委会",
                code: "008",
            },
            VillageCode {
                name: "巴西村村委会",
                code: "009",
            },
            VillageCode {
                name: "夺赤村村委会",
                code: "010",
            },
            VillageCode {
                name: "居雪村村委会",
                code: "011",
            },
            VillageCode {
                name: "聂沃村村委会",
                code: "012",
            },
            VillageCode {
                name: "梅巴村村委会",
                code: "013",
            },
            VillageCode {
                name: "雪东村村委会",
                code: "014",
            },
            VillageCode {
                name: "瓦巴村村委会",
                code: "015",
            },
            VillageCode {
                name: "达浪村村委会",
                code: "016",
            },
            VillageCode {
                name: "如给村村委会",
                code: "017",
            },
            VillageCode {
                name: "察俄村村委会",
                code: "018",
            },
            VillageCode {
                name: "给如村村委会",
                code: "019",
            },
            VillageCode {
                name: "列康村村委会",
                code: "020",
            },
            VillageCode {
                name: "白久村村委会",
                code: "021",
            },
            VillageCode {
                name: "索贡村村委会",
                code: "022",
            },
            VillageCode {
                name: "亚莫村村委会",
                code: "023",
            },
            VillageCode {
                name: "聂欧村村委会",
                code: "024",
            },
            VillageCode {
                name: "拉叶村村委会",
                code: "025",
            },
            VillageCode {
                name: "卡松村村委会",
                code: "026",
            },
            VillageCode {
                name: "结强村村委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "香堆镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "香堆居委会",
                code: "001",
            },
            VillageCode {
                name: "筑梦新村村委会",
                code: "002",
            },
            VillageCode {
                name: "拉西村村委会",
                code: "003",
            },
            VillageCode {
                name: "果日村村委会",
                code: "004",
            },
            VillageCode {
                name: "当佐村村委会",
                code: "005",
            },
            VillageCode {
                name: "达巴村村委会",
                code: "006",
            },
            VillageCode {
                name: "嘎查村",
                code: "007",
            },
            VillageCode {
                name: "热孜村村委会",
                code: "008",
            },
            VillageCode {
                name: "坤达村村委会",
                code: "009",
            },
            VillageCode {
                name: "学龙村村委会",
                code: "010",
            },
            VillageCode {
                name: "旺布村村委会",
                code: "011",
            },
            VillageCode {
                name: "仁达村村委会",
                code: "012",
            },
            VillageCode {
                name: "仁江村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "吉塘镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "吉塘居委会",
                code: "001",
            },
            VillageCode {
                name: "吉祥广东新村村委会",
                code: "002",
            },
            VillageCode {
                name: "色热西村村委会",
                code: "003",
            },
            VillageCode {
                name: "亚许村村委会",
                code: "004",
            },
            VillageCode {
                name: "雪谢村村委会",
                code: "005",
            },
            VillageCode {
                name: "酉西村村委会",
                code: "006",
            },
            VillageCode {
                name: "达布村村委会",
                code: "007",
            },
            VillageCode {
                name: "卡仁村村委会",
                code: "008",
            },
            VillageCode {
                name: "莫东村村委会",
                code: "009",
            },
            VillageCode {
                name: "雪通村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "宗沙乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "拉松村村委会",
                code: "001",
            },
            VillageCode {
                name: "察姆村村委会",
                code: "002",
            },
            VillageCode {
                name: "宗沙村村委会",
                code: "003",
            },
            VillageCode {
                name: "热觉村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "卡贡乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "金多村村委会",
                code: "001",
            },
            VillageCode {
                name: "宾果村村委会",
                code: "002",
            },
            VillageCode {
                name: "邓学村村委会",
                code: "003",
            },
            VillageCode {
                name: "莫日村村委会",
                code: "004",
            },
            VillageCode {
                name: "依然村村委会",
                code: "005",
            },
            VillageCode {
                name: "卡贡村村委会",
                code: "006",
            },
            VillageCode {
                name: "村帮村村委会",
                code: "007",
            },
            VillageCode {
                name: "索赤村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "荣周乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "青山中铝新村村委会",
                code: "001",
            },
            VillageCode {
                name: "麦堆村村委会",
                code: "002",
            },
            VillageCode {
                name: "佐通村村委会",
                code: "003",
            },
            VillageCode {
                name: "荣周村村委会",
                code: "004",
            },
            VillageCode {
                name: "姆巴村村委会",
                code: "005",
            },
            VillageCode {
                name: "栋扎村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "巴日乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "白西村村委会",
                code: "001",
            },
            VillageCode {
                name: "罗松村村委会",
                code: "002",
            },
            VillageCode {
                name: "拉堆村村委会",
                code: "003",
            },
            VillageCode {
                name: "帕拉村村委会",
                code: "004",
            },
            VillageCode {
                name: "拉麦村村委会",
                code: "005",
            },
            VillageCode {
                name: "拉冲村村委会",
                code: "006",
            },
            VillageCode {
                name: "尼珠村村委会",
                code: "007",
            },
            VillageCode {
                name: "吉列村村委会",
                code: "008",
            },
            VillageCode {
                name: "俄宗村村委会",
                code: "009",
            },
            VillageCode {
                name: "温雅村村委会",
                code: "010",
            },
            VillageCode {
                name: "仁堆村村委会",
                code: "011",
            },
            VillageCode {
                name: "白娘村村委会",
                code: "012",
            },
            VillageCode {
                name: "吉嘎村村委会",
                code: "013",
            },
            VillageCode {
                name: "德娘村村委会",
                code: "014",
            },
            VillageCode {
                name: "雄热村村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "阿孜乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "阿琼村村委会",
                code: "001",
            },
            VillageCode {
                name: "阿都村村委会",
                code: "002",
            },
            VillageCode {
                name: "觉萨村村委会",
                code: "003",
            },
            VillageCode {
                name: "珠扎村村委会",
                code: "004",
            },
            VillageCode {
                name: "孜久村村委会",
                code: "005",
            },
            VillageCode {
                name: "阿贡村村委会",
                code: "006",
            },
            VillageCode {
                name: "邓普村村委会",
                code: "007",
            },
            VillageCode {
                name: "江嘎村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "王卡乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "绿水新村村委会",
                code: "001",
            },
            VillageCode {
                name: "则努村村委会",
                code: "002",
            },
            VillageCode {
                name: "帕罗村村委会",
                code: "003",
            },
            VillageCode {
                name: "帕贡村村委会",
                code: "004",
            },
            VillageCode {
                name: "益热村村委会",
                code: "005",
            },
            VillageCode {
                name: "则曲村村委会",
                code: "006",
            },
            VillageCode {
                name: "协地村村委会",
                code: "007",
            },
            VillageCode {
                name: "夺巴村村委会",
                code: "008",
            },
            VillageCode {
                name: "王吉村村委会",
                code: "009",
            },
            VillageCode {
                name: "娘曲村村委会",
                code: "010",
            },
            VillageCode {
                name: "玛恩村村委会",
                code: "011",
            },
            VillageCode {
                name: "波热村村委会",
                code: "012",
            },
            VillageCode {
                name: "恩达村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "新卡乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "乃帕村村委会",
                code: "001",
            },
            VillageCode {
                name: "达也村村委会",
                code: "002",
            },
            VillageCode {
                name: "瓦江村村委会",
                code: "003",
            },
            VillageCode {
                name: "新卡村村委会",
                code: "004",
            },
            VillageCode {
                name: "克琼村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "肯通乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "堆热村村委会",
                code: "001",
            },
            VillageCode {
                name: "达如村村委会",
                code: "002",
            },
            VillageCode {
                name: "爱如村村委会",
                code: "003",
            },
            VillageCode {
                name: "多雄村村委会",
                code: "004",
            },
            VillageCode {
                name: "吉孜村村委会",
                code: "005",
            },
            VillageCode {
                name: "协堆村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "扩达乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "旺达村村委会",
                code: "001",
            },
            VillageCode {
                name: "巴曲村村委会",
                code: "002",
            },
            VillageCode {
                name: "达加苦村村委会",
                code: "003",
            },
            VillageCode {
                name: "嘎益村村委会",
                code: "004",
            },
            VillageCode {
                name: "知大达村村委会",
                code: "005",
            },
            VillageCode {
                name: "面穷苦村村委会",
                code: "006",
            },
            VillageCode {
                name: "玛多苦村村委会",
                code: "007",
            },
            VillageCode {
                name: "嘎莫村村委会",
                code: "008",
            },
            VillageCode {
                name: "瓦贡村村委会",
                code: "009",
            },
            VillageCode {
                name: "玛佐村村委会",
                code: "010",
            },
            VillageCode {
                name: "那普村村委会",
                code: "011",
            },
            VillageCode {
                name: "多庆村村委会",
                code: "012",
            },
            VillageCode {
                name: "邦热村村委会",
                code: "013",
            },
            VillageCode {
                name: "格日玛村村委会",
                code: "014",
            },
            VillageCode {
                name: "孔曼多村村委会",
                code: "015",
            },
            VillageCode {
                name: "多桑村村委会",
                code: "016",
            },
            VillageCode {
                name: "都达村村委会",
                code: "017",
            },
            VillageCode {
                name: "果巴村村委会",
                code: "018",
            },
            VillageCode {
                name: "宗多村村委会",
                code: "019",
            },
            VillageCode {
                name: "岗卡村村委会",
                code: "020",
            },
            VillageCode {
                name: "岗泽村村委会",
                code: "021",
            },
            VillageCode {
                name: "列尼村村委会",
                code: "022",
            },
            VillageCode {
                name: "伍巴村村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "察拉乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "察拉村村委会",
                code: "001",
            },
            VillageCode {
                name: "夏达村村委会",
                code: "002",
            },
            VillageCode {
                name: "学达村村委会",
                code: "003",
            },
            VillageCode {
                name: "卡达村村委会",
                code: "004",
            },
            VillageCode {
                name: "金巴村村委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_XK_013: [TownCode; 14] = [
    TownCode {
        name: "白玛镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "白玛社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "约巴村村委会",
                code: "002",
            },
            VillageCode {
                name: "珠巴村村委会",
                code: "003",
            },
            VillageCode {
                name: "乃然村村委会",
                code: "004",
            },
            VillageCode {
                name: "日吉村村委会",
                code: "005",
            },
            VillageCode {
                name: "旺比村村委会",
                code: "006",
            },
            VillageCode {
                name: "沙木村村委会",
                code: "007",
            },
            VillageCode {
                name: "丁卡村村委会",
                code: "008",
            },
            VillageCode {
                name: "西巴村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "帮达镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "邦达村村委会",
                code: "001",
            },
            VillageCode {
                name: "索直村村委会",
                code: "002",
            },
            VillageCode {
                name: "克色村村委会",
                code: "003",
            },
            VillageCode {
                name: "同尼村村委会",
                code: "004",
            },
            VillageCode {
                name: "查龙村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "然乌镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "瓦巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "然那村村委会",
                code: "002",
            },
            VillageCode {
                name: "宗巴村村委会",
                code: "003",
            },
            VillageCode {
                name: "来古村村委会",
                code: "004",
            },
            VillageCode {
                name: "卡堆村村委会",
                code: "005",
            },
            VillageCode {
                name: "然乌村村委会",
                code: "006",
            },
            VillageCode {
                name: "阿日村村委会",
                code: "007",
            },
            VillageCode {
                name: "雅则村村委会",
                code: "008",
            },
            VillageCode {
                name: "达巴村村委会",
                code: "009",
            },
            VillageCode {
                name: "康沙村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "同卡镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "波查村村委会",
                code: "001",
            },
            VillageCode {
                name: "觉龙村村委会",
                code: "002",
            },
            VillageCode {
                name: "然多村村委会",
                code: "003",
            },
            VillageCode {
                name: "俄觉村村委会",
                code: "004",
            },
            VillageCode {
                name: "卡顶村村委会",
                code: "005",
            },
            VillageCode {
                name: "郎巴村村委会",
                code: "006",
            },
            VillageCode {
                name: "帕西村村委会",
                code: "007",
            },
            VillageCode {
                name: "吉巴村村委会",
                code: "008",
            },
            VillageCode {
                name: "亚同村村委会",
                code: "009",
            },
            VillageCode {
                name: "沙热村村委会",
                code: "010",
            },
            VillageCode {
                name: "古日村村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "郭庆乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "岗塔村村委会",
                code: "001",
            },
            VillageCode {
                name: "觉约村村委会",
                code: "002",
            },
            VillageCode {
                name: "多色村村委会",
                code: "003",
            },
            VillageCode {
                name: "觉美村村委会",
                code: "004",
            },
            VillageCode {
                name: "拉龙村村委会",
                code: "005",
            },
            VillageCode {
                name: "拉交村村委会",
                code: "006",
            },
            VillageCode {
                name: "那塔村村委会",
                code: "007",
            },
            VillageCode {
                name: "日楚村村委会",
                code: "008",
            },
            VillageCode {
                name: "觉尼村村委会",
                code: "009",
            },
            VillageCode {
                name: "拥然村村委会",
                code: "010",
            },
            VillageCode {
                name: "尼恰村村委会",
                code: "011",
            },
            VillageCode {
                name: "觉村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "拉根乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "尼巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "拉根村村委会",
                code: "002",
            },
            VillageCode {
                name: "绕巴村村委会",
                code: "003",
            },
            VillageCode {
                name: "瓦来村村委会",
                code: "004",
            },
            VillageCode {
                name: "瓦达村村委会",
                code: "005",
            },
            VillageCode {
                name: "列日村村委会",
                code: "006",
            },
            VillageCode {
                name: "冷贡村村委会",
                code: "007",
            },
            VillageCode {
                name: "多日多龙村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "益庆乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "曲扎村村委会",
                code: "001",
            },
            VillageCode {
                name: "尼琼村村委会",
                code: "002",
            },
            VillageCode {
                name: "索那村村委会",
                code: "003",
            },
            VillageCode {
                name: "羊达村村委会",
                code: "004",
            },
            VillageCode {
                name: "崩庆村村委会",
                code: "005",
            },
            VillageCode {
                name: "多庆村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "吉中乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "卡穷村村委会",
                code: "001",
            },
            VillageCode {
                name: "吉龙村村委会",
                code: "002",
            },
            VillageCode {
                name: "新贡村村委会",
                code: "003",
            },
            VillageCode {
                name: "毕青村村委会",
                code: "004",
            },
            VillageCode {
                name: "毕琼村村委会",
                code: "005",
            },
            VillageCode {
                name: "集中村村委会",
                code: "006",
            },
            VillageCode {
                name: "那德村村委会",
                code: "007",
            },
            VillageCode {
                name: "木觉村村委会",
                code: "008",
            },
            VillageCode {
                name: "洛龙村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "卡瓦白庆乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "拉巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "吉卡村村委会",
                code: "002",
            },
            VillageCode {
                name: "卡瓦村村委会",
                code: "003",
            },
            VillageCode {
                name: "卡堆村村委会",
                code: "004",
            },
            VillageCode {
                name: "扎巴村村委会",
                code: "005",
            },
            VillageCode {
                name: "卡色村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "吉达乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "果拉村村委会",
                code: "001",
            },
            VillageCode {
                name: "仲沙村村委会",
                code: "002",
            },
            VillageCode {
                name: "拉然村村委会",
                code: "003",
            },
            VillageCode {
                name: "江查村村委会",
                code: "004",
            },
            VillageCode {
                name: "郎宗村村委会",
                code: "005",
            },
            VillageCode {
                name: "吉达村村委会",
                code: "006",
            },
            VillageCode {
                name: "同空村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "夏里乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "吉热村村委会",
                code: "001",
            },
            VillageCode {
                name: "泽金村村委会",
                code: "002",
            },
            VillageCode {
                name: "崩夏村村委会",
                code: "003",
            },
            VillageCode {
                name: "外巴村村委会",
                code: "004",
            },
            VillageCode {
                name: "北查村村委会",
                code: "005",
            },
            VillageCode {
                name: "左西村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "拥巴乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "拥巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "果沙村村委会",
                code: "002",
            },
            VillageCode {
                name: "然那村村委会",
                code: "003",
            },
            VillageCode {
                name: "娜帕村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "瓦乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "夏巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "雪科村村委会",
                code: "002",
            },
            VillageCode {
                name: "茹帕村村委会",
                code: "003",
            },
            VillageCode {
                name: "瓦巴村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "林卡乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "多恩村村委会",
                code: "001",
            },
            VillageCode {
                name: "果巴村村委会",
                code: "002",
            },
            VillageCode {
                name: "卡龙村村委会",
                code: "003",
            },
            VillageCode {
                name: "色巴村村委会",
                code: "004",
            },
            VillageCode {
                name: "查卡村村委会",
                code: "005",
            },
            VillageCode {
                name: "普龙村村委会",
                code: "006",
            },
            VillageCode {
                name: "尼巴村村委会",
                code: "007",
            },
            VillageCode {
                name: "略觉村村委会",
                code: "008",
            },
            VillageCode {
                name: "布则村村委会",
                code: "009",
            },
            VillageCode {
                name: "叶巴村委会村",
                code: "010",
            },
            VillageCode {
                name: "旺珠村村委会",
                code: "011",
            },
            VillageCode {
                name: "孜嘎村村委会",
                code: "012",
            },
            VillageCode {
                name: "冷宜村村委会",
                code: "013",
            },
        ],
    },
];

static TOWNS_XK_014: [TownCode; 10] = [
    TownCode {
        name: "旺达镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "旺达社区居委会",
                code: "001",
            },
            VillageCode {
                name: "四方祥和新村村委会",
                code: "002",
            },
            VillageCode {
                name: "列达村委会",
                code: "003",
            },
            VillageCode {
                name: "兵达村委会",
                code: "004",
            },
            VillageCode {
                name: "波科村委会",
                code: "005",
            },
            VillageCode {
                name: "俄比村委会",
                code: "006",
            },
            VillageCode {
                name: "麻科村委会",
                code: "007",
            },
            VillageCode {
                name: "东达村委会",
                code: "008",
            },
            VillageCode {
                name: "乌雅村委会",
                code: "009",
            },
            VillageCode {
                name: "左巴村委会",
                code: "010",
            },
            VillageCode {
                name: "则巴村委会",
                code: "011",
            },
            VillageCode {
                name: "冷加村委会",
                code: "012",
            },
            VillageCode {
                name: "夯达村委会",
                code: "013",
            },
            VillageCode {
                name: "普绒村委会",
                code: "014",
            },
            VillageCode {
                name: "木龙村委会",
                code: "015",
            },
            VillageCode {
                name: "拉达村委会",
                code: "016",
            },
            VillageCode {
                name: "孟青村委会",
                code: "017",
            },
            VillageCode {
                name: "马普村委会",
                code: "018",
            },
            VillageCode {
                name: "孟琼村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "田妥镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "雅卓幸福新村村委会",
                code: "001",
            },
            VillageCode {
                name: "格如村委会",
                code: "002",
            },
            VillageCode {
                name: "然尼村委会",
                code: "003",
            },
            VillageCode {
                name: "江达村委会",
                code: "004",
            },
            VillageCode {
                name: "亚中村委会",
                code: "005",
            },
            VillageCode {
                name: "果热村委会",
                code: "006",
            },
            VillageCode {
                name: "塔鲁村委会",
                code: "007",
            },
            VillageCode {
                name: "田妥村委会",
                code: "008",
            },
            VillageCode {
                name: "德列比村委会",
                code: "009",
            },
            VillageCode {
                name: "金达村委会",
                code: "010",
            },
            VillageCode {
                name: "德达村委会",
                code: "011",
            },
            VillageCode {
                name: "色贡村委会",
                code: "012",
            },
            VillageCode {
                name: "沙益村委会",
                code: "013",
            },
            VillageCode {
                name: "米扎村委会",
                code: "014",
            },
            VillageCode {
                name: "帮达村委会",
                code: "015",
            },
            VillageCode {
                name: "嘎益村委会",
                code: "016",
            },
            VillageCode {
                name: "夺巴村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "扎玉镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "德吉新村村委会",
                code: "001",
            },
            VillageCode {
                name: "瓦巴村委会",
                code: "002",
            },
            VillageCode {
                name: "雪巴村委会",
                code: "003",
            },
            VillageCode {
                name: "达巴村委会",
                code: "004",
            },
            VillageCode {
                name: "扎西村委会",
                code: "005",
            },
            VillageCode {
                name: "夏库村委会",
                code: "006",
            },
            VillageCode {
                name: "吾同村委会",
                code: "007",
            },
            VillageCode {
                name: "德贡村委会",
                code: "008",
            },
            VillageCode {
                name: "查库村委会",
                code: "009",
            },
            VillageCode {
                name: "玉贡村委会",
                code: "010",
            },
            VillageCode {
                name: "吾沙村委会",
                code: "011",
            },
            VillageCode {
                name: "成德村委会",
                code: "012",
            },
            VillageCode {
                name: "瓦巴通村委会",
                code: "013",
            },
            VillageCode {
                name: "德巴村委会",
                code: "014",
            },
            VillageCode {
                name: "宗巴村委会",
                code: "015",
            },
            VillageCode {
                name: "米巴村委会",
                code: "016",
            },
            VillageCode {
                name: "巴瓦村委会",
                code: "017",
            },
            VillageCode {
                name: "卡尼村委会",
                code: "018",
            },
            VillageCode {
                name: "生普村委会",
                code: "019",
            },
            VillageCode {
                name: "碧西村委会",
                code: "020",
            },
            VillageCode {
                name: "然米村委会",
                code: "021",
            },
            VillageCode {
                name: "吉邓村委会",
                code: "022",
            },
            VillageCode {
                name: "碧巴村委会",
                code: "023",
            },
            VillageCode {
                name: "巴藏村委会",
                code: "024",
            },
            VillageCode {
                name: "巴玉村委会",
                code: "025",
            },
            VillageCode {
                name: "巴给村委会",
                code: "026",
            },
            VillageCode {
                name: "吉普村委会",
                code: "027",
            },
            VillageCode {
                name: "中邓村委会",
                code: "028",
            },
            VillageCode {
                name: "然根村委会",
                code: "029",
            },
            VillageCode {
                name: "理巴村委会",
                code: "030",
            },
            VillageCode {
                name: "然巴村委会",
                code: "031",
            },
            VillageCode {
                name: "地巴村委会",
                code: "032",
            },
            VillageCode {
                name: "地库村委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "东坝乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "军拥村委会",
                code: "001",
            },
            VillageCode {
                name: "普卡村委会",
                code: "002",
            },
            VillageCode {
                name: "坝雪村委会",
                code: "003",
            },
            VillageCode {
                name: "加坝村委会",
                code: "004",
            },
            VillageCode {
                name: "格瓦村委会",
                code: "005",
            },
            VillageCode {
                name: "埃西村委会",
                code: "006",
            },
            VillageCode {
                name: "沙益村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "仁果乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "若巴村委会",
                code: "001",
            },
            VillageCode {
                name: "沙龙村委会",
                code: "002",
            },
            VillageCode {
                name: "益西村委会",
                code: "003",
            },
            VillageCode {
                name: "仁果村委会",
                code: "004",
            },
            VillageCode {
                name: "新德村委会",
                code: "005",
            },
            VillageCode {
                name: "左科村委会",
                code: "006",
            },
            VillageCode {
                name: "东坝村委会",
                code: "007",
            },
            VillageCode {
                name: "坝巴村委会",
                code: "008",
            },
            VillageCode {
                name: "加卡村委会",
                code: "009",
            },
            VillageCode {
                name: "吞拥村委会",
                code: "010",
            },
            VillageCode {
                name: "兰果村委会",
                code: "011",
            },
            VillageCode {
                name: "青果村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "绕金乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "左巴村委会",
                code: "001",
            },
            VillageCode {
                name: "绕金村委会",
                code: "002",
            },
            VillageCode {
                name: "贡日村委会",
                code: "003",
            },
            VillageCode {
                name: "日巴村委会",
                code: "004",
            },
            VillageCode {
                name: "巴坝村委会",
                code: "005",
            },
            VillageCode {
                name: "普拉村委会",
                code: "006",
            },
            VillageCode {
                name: "绕丝村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "碧土乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "布然村委会",
                code: "001",
            },
            VillageCode {
                name: "碧土村委会",
                code: "002",
            },
            VillageCode {
                name: "地巴村委会",
                code: "003",
            },
            VillageCode {
                name: "扎郎村委会",
                code: "004",
            },
            VillageCode {
                name: "沙多村委会",
                code: "005",
            },
            VillageCode {
                name: "龙西村委会",
                code: "006",
            },
            VillageCode {
                name: "甲郎村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "美玉乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "美玉村委会",
                code: "001",
            },
            VillageCode {
                name: "卡扎村委会",
                code: "002",
            },
            VillageCode {
                name: "日雪村委会",
                code: "003",
            },
            VillageCode {
                name: "俄龙村委会",
                code: "004",
            },
            VillageCode {
                name: "斜库村委会",
                code: "005",
            },
            VillageCode {
                name: "边玉村委会",
                code: "006",
            },
            VillageCode {
                name: "乌碧村委会",
                code: "007",
            },
            VillageCode {
                name: "然仲村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "中林卡乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "瓦堆村委会",
                code: "001",
            },
            VillageCode {
                name: "瓦美村委会",
                code: "002",
            },
            VillageCode {
                name: "洛巴村委会",
                code: "003",
            },
            VillageCode {
                name: "俄巴村委会",
                code: "004",
            },
            VillageCode {
                name: "普拉村委会",
                code: "005",
            },
            VillageCode {
                name: "左西村委会",
                code: "006",
            },
            VillageCode {
                name: "嘎宗村委会",
                code: "007",
            },
            VillageCode {
                name: "十字卡村委会",
                code: "008",
            },
            VillageCode {
                name: "若巴村委会",
                code: "009",
            },
            VillageCode {
                name: "种青村委会",
                code: "010",
            },
            VillageCode {
                name: "琼卡村委会",
                code: "011",
            },
            VillageCode {
                name: "拉巴村委会",
                code: "012",
            },
            VillageCode {
                name: "拉琼村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "下林卡乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "古巴村委会",
                code: "001",
            },
            VillageCode {
                name: "旧巴村委会",
                code: "002",
            },
            VillageCode {
                name: "果热村委会",
                code: "003",
            },
            VillageCode {
                name: "旭日村委会",
                code: "004",
            },
            VillageCode {
                name: "西巴村委会",
                code: "005",
            },
            VillageCode {
                name: "达巴村委会",
                code: "006",
            },
            VillageCode {
                name: "甲巴村委会",
                code: "007",
            },
            VillageCode {
                name: "友巴村委会",
                code: "008",
            },
        ],
    },
];

static TOWNS_XK_015: [TownCode; 16] = [
    TownCode {
        name: "嘎托镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "嘎托居委会",
                code: "001",
            },
            VillageCode {
                name: "达吉村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "火拉村村委会",
                code: "003",
            },
            VillageCode {
                name: "嘎托村村委会",
                code: "004",
            },
            VillageCode {
                name: "巴拉村村委会",
                code: "005",
            },
            VillageCode {
                name: "加它村村委会",
                code: "006",
            },
            VillageCode {
                name: "普拉村村委会",
                code: "007",
            },
            VillageCode {
                name: "达空村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "吉修村村委会",
                code: "009",
            },
            VillageCode {
                name: "平德村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "雅卓村村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "如美镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "拉乌村村委会",
                code: "001",
            },
            VillageCode {
                name: "如美村村委会",
                code: "002",
            },
            VillageCode {
                name: "达日村村委会",
                code: "003",
            },
            VillageCode {
                name: "卡均村村委会",
                code: "004",
            },
            VillageCode {
                name: "竹卡村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "索多西乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "安麦西村村委会",
                code: "001",
            },
            VillageCode {
                name: "角比西村村委会",
                code: "002",
            },
            VillageCode {
                name: "格朗西村村委会",
                code: "003",
            },
            VillageCode {
                name: "达海龙村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "莽岭乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "上莽岭村村委会",
                code: "001",
            },
            VillageCode {
                name: "下莽岭村村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "宗西乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "宗西村村委会",
                code: "001",
            },
            VillageCode {
                name: "宗荣村村委会",
                code: "002",
            },
            VillageCode {
                name: "达拉村村委会",
                code: "003",
            },
            VillageCode {
                name: "通古村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "昂多乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "吉措村村委会",
                code: "001",
            },
            VillageCode {
                name: "曲塔村村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "措瓦乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "措瓦村村委会",
                code: "001",
            },
            VillageCode {
                name: "日许村村委会",
                code: "002",
            },
            VillageCode {
                name: "通沙村村委会",
                code: "003",
            },
            VillageCode {
                name: "塔亚村村委会",
                code: "004",
            },
            VillageCode {
                name: "仲日村村委会",
                code: "005",
            },
            VillageCode {
                name: "库孜村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "洛尼乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "洛尼村村委会",
                code: "001",
            },
            VillageCode {
                name: "当佐村村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "戈波乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "戈波村村委会",
                code: "001",
            },
            VillageCode {
                name: "支巴村村委会",
                code: "002",
            },
            VillageCode {
                name: "南格村村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "帮达乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "毛尼村村委会",
                code: "001",
            },
            VillageCode {
                name: "帮达村村委会",
                code: "002",
            },
            VillageCode {
                name: "加嘎村村委会",
                code: "003",
            },
            VillageCode {
                name: "金珠村村委会",
                code: "004",
            },
            VillageCode {
                name: "然堆村村委会",
                code: "005",
            },
            VillageCode {
                name: "加尼顶村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "徐中乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "徐中村村委会",
                code: "001",
            },
            VillageCode {
                name: "哈扎村村委会",
                code: "002",
            },
            VillageCode {
                name: "卡布村村委会",
                code: "003",
            },
            VillageCode {
                name: "门巴村村委会",
                code: "004",
            },
            VillageCode {
                name: "尼玛莎村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "曲登乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "邓巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "曲登村村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "木许乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "木许村村委会",
                code: "001",
            },
            VillageCode {
                name: "阿东村村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "纳西民族乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "盐井新村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "加达村村委会",
                code: "002",
            },
            VillageCode {
                name: "上盐井村村委会",
                code: "003",
            },
            VillageCode {
                name: "觉龙村村委会",
                code: "004",
            },
            VillageCode {
                name: "纳西村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "朱巴龙乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "达嘎顶村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "草地贡村村委会",
                code: "002",
            },
            VillageCode {
                name: "朱巴龙村村委会",
                code: "003",
            },
            VillageCode {
                name: "松瓦村村委会",
                code: "004",
            },
            VillageCode {
                name: "西松贡村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "曲孜卡乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "圣雅新村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "小昌都村村委会",
                code: "002",
            },
            VillageCode {
                name: "拉久西村村委会",
                code: "003",
            },
            VillageCode {
                name: "达许村村委会",
                code: "004",
            },
        ],
    },
];

static TOWNS_XK_016: [TownCode; 11] = [
    TownCode {
        name: "孜托镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "孜托居委会",
                code: "001",
            },
            VillageCode {
                name: "德康社区居委会",
                code: "002",
            },
            VillageCode {
                name: "尼亚村村委会",
                code: "003",
            },
            VillageCode {
                name: "夏果村村委会",
                code: "004",
            },
            VillageCode {
                name: "格亚村村委会",
                code: "005",
            },
            VillageCode {
                name: "加日扎村村委会",
                code: "006",
            },
            VillageCode {
                name: "然昌村村委会",
                code: "007",
            },
            VillageCode {
                name: "德通村村委会",
                code: "008",
            },
            VillageCode {
                name: "古曲村村委会",
                code: "009",
            },
            VillageCode {
                name: "达贡村村委会",
                code: "010",
            },
            VillageCode {
                name: "中松村村委会",
                code: "011",
            },
            VillageCode {
                name: "朗错村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "硕督镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "硕督村村委会",
                code: "001",
            },
            VillageCode {
                name: "久嘎村村委会",
                code: "002",
            },
            VillageCode {
                name: "达翁村村委会",
                code: "003",
            },
            VillageCode {
                name: "拉依村村委会",
                code: "004",
            },
            VillageCode {
                name: "荣雄村村委会",
                code: "005",
            },
            VillageCode {
                name: "日许村村委会",
                code: "006",
            },
            VillageCode {
                name: "孜普卡村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "康沙镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "康沙村村委会",
                code: "001",
            },
            VillageCode {
                name: "牛格村村委会",
                code: "002",
            },
            VillageCode {
                name: "也堆村村委会",
                code: "003",
            },
            VillageCode {
                name: "纳龙村村委会",
                code: "004",
            },
            VillageCode {
                name: "查然村村委会",
                code: "005",
            },
            VillageCode {
                name: "德嘎村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "马利镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "马利村村委会",
                code: "001",
            },
            VillageCode {
                name: "久修村村委会",
                code: "002",
            },
            VillageCode {
                name: "瓦河村村委会",
                code: "003",
            },
            VillageCode {
                name: "布许村村委会",
                code: "004",
            },
            VillageCode {
                name: "夏玉村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "达龙乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "达龙村村委会",
                code: "001",
            },
            VillageCode {
                name: "布达村村委会",
                code: "002",
            },
            VillageCode {
                name: "色底村村委会",
                code: "003",
            },
            VillageCode {
                name: "荣折村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "新荣乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "通那村村委会",
                code: "001",
            },
            VillageCode {
                name: "拉加村村委会",
                code: "002",
            },
            VillageCode {
                name: "克多村村委会",
                code: "003",
            },
            VillageCode {
                name: "白托村村委会",
                code: "004",
            },
            VillageCode {
                name: "板凳村村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "白达乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "白托村村委会",
                code: "001",
            },
            VillageCode {
                name: "通尼村村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "玉西乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "拉绕村村委会",
                code: "001",
            },
            VillageCode {
                name: "巴村村委会",
                code: "002",
            },
            VillageCode {
                name: "日许村村委会",
                code: "003",
            },
            VillageCode {
                name: "色然村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "腊久乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "西通坝村村委会",
                code: "001",
            },
            VillageCode {
                name: "母许村村委会",
                code: "002",
            },
            VillageCode {
                name: "查瓦村村委会",
                code: "003",
            },
            VillageCode {
                name: "江云村村委会",
                code: "004",
            },
            VillageCode {
                name: "八美村村委会",
                code: "005",
            },
            VillageCode {
                name: "巴堆村村委会",
                code: "006",
            },
            VillageCode {
                name: "多尼村村委会",
                code: "007",
            },
            VillageCode {
                name: "堆村村委会",
                code: "008",
            },
            VillageCode {
                name: "萨玛村村委会",
                code: "009",
            },
            VillageCode {
                name: "张贡村村委会",
                code: "010",
            },
            VillageCode {
                name: "中瓦村村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "俄西乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "雪瓦通村村委会",
                code: "001",
            },
            VillageCode {
                name: "贡中村村委会",
                code: "002",
            },
            VillageCode {
                name: "次琼村村委会",
                code: "003",
            },
            VillageCode {
                name: "扎嘎村村委会",
                code: "004",
            },
            VillageCode {
                name: "西湖村村委会",
                code: "005",
            },
            VillageCode {
                name: "娘娘村村委会",
                code: "006",
            },
            VillageCode {
                name: "伟村村委会",
                code: "007",
            },
            VillageCode {
                name: "达邓村村委会",
                code: "008",
            },
            VillageCode {
                name: "也依村村委会",
                code: "009",
            },
            VillageCode {
                name: "甲瓦村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "中亦乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "中亦村村委会",
                code: "001",
            },
            VillageCode {
                name: "亚许村村委会",
                code: "002",
            },
            VillageCode {
                name: "嘴村村委会",
                code: "003",
            },
            VillageCode {
                name: "加果村村委会",
                code: "004",
            },
        ],
    },
];

static TOWNS_XK_017: [TownCode; 11] = [
    TownCode {
        name: "边坝镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "显俄村村委会",
                code: "001",
            },
            VillageCode {
                name: "普玉一村村委会",
                code: "002",
            },
            VillageCode {
                name: "普玉二村村委会",
                code: "003",
            },
            VillageCode {
                name: "洛亚玛村村委会",
                code: "004",
            },
            VillageCode {
                name: "夏林村村委会",
                code: "005",
            },
            VillageCode {
                name: "登卡村村委会",
                code: "006",
            },
            VillageCode {
                name: "布扎村村委会",
                code: "007",
            },
            VillageCode {
                name: "多许村村委会",
                code: "008",
            },
            VillageCode {
                name: "热塔村村委会",
                code: "009",
            },
            VillageCode {
                name: "宗古村村委会",
                code: "010",
            },
            VillageCode {
                name: "拥村村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "草卡镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "东托社区居委会",
                code: "001",
            },
            VillageCode {
                name: "民族路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "桑卡社区居委会",
                code: "003",
            },
            VillageCode {
                name: "格吉村村委会",
                code: "004",
            },
            VillageCode {
                name: "旺卡村村委会",
                code: "005",
            },
            VillageCode {
                name: "索村村委会",
                code: "006",
            },
            VillageCode {
                name: "来义村村委会",
                code: "007",
            },
            VillageCode {
                name: "拉托村村委会",
                code: "008",
            },
            VillageCode {
                name: "藏巴村村委会",
                code: "009",
            },
            VillageCode {
                name: "卓归村村委会",
                code: "010",
            },
            VillageCode {
                name: "昌沙村村委会",
                code: "011",
            },
            VillageCode {
                name: "麦加村村委会",
                code: "012",
            },
            VillageCode {
                name: "达根村村委会",
                code: "013",
            },
            VillageCode {
                name: "拉贡村村委会",
                code: "014",
            },
            VillageCode {
                name: "丹达村村委会",
                code: "015",
            },
            VillageCode {
                name: "苏东村村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "沙丁乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "通内村村委会",
                code: "001",
            },
            VillageCode {
                name: "沙丁村村委会",
                code: "002",
            },
            VillageCode {
                name: "东地村村委会",
                code: "003",
            },
            VillageCode {
                name: "日普村村委会",
                code: "004",
            },
            VillageCode {
                name: "格尼村村委会",
                code: "005",
            },
            VillageCode {
                name: "松许村村委会",
                code: "006",
            },
            VillageCode {
                name: "知内村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "金岭乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "卡许村村委会",
                code: "001",
            },
            VillageCode {
                name: "玉贡股村村委会",
                code: "002",
            },
            VillageCode {
                name: "卓格村村委会",
                code: "003",
            },
            VillageCode {
                name: "郎杰贡村村委会",
                code: "004",
            },
            VillageCode {
                name: "玉坝村村委会",
                code: "005",
            },
            VillageCode {
                name: "结玉村村委会",
                code: "006",
            },
            VillageCode {
                name: "通东村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "加贡乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "加贡村村委会",
                code: "001",
            },
            VillageCode {
                name: "加布村村委会",
                code: "002",
            },
            VillageCode {
                name: "国庆村村委会",
                code: "003",
            },
            VillageCode {
                name: "益布村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "马武乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "粗卡娘村村委会",
                code: "001",
            },
            VillageCode {
                name: "贡龙村村委会",
                code: "002",
            },
            VillageCode {
                name: "达如村村委会",
                code: "003",
            },
            VillageCode {
                name: "马武村村委会",
                code: "004",
            },
            VillageCode {
                name: "查日村村委会",
                code: "005",
            },
            VillageCode {
                name: "拉加村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "热玉乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "热玉村村委会",
                code: "001",
            },
            VillageCode {
                name: "机贡村村委会",
                code: "002",
            },
            VillageCode {
                name: "东美村村委会",
                code: "003",
            },
            VillageCode {
                name: "嘎贡村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "尼木乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "许巴村村委会",
                code: "001",
            },
            VillageCode {
                name: "江果堆村村委会",
                code: "002",
            },
            VillageCode {
                name: "叶嘎村村委会",
                code: "003",
            },
            VillageCode {
                name: "尼木村村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "马秀乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "布谷村村委会",
                code: "001",
            },
            VillageCode {
                name: "马秀村村委会",
                code: "002",
            },
            VillageCode {
                name: "曲桑村村委会",
                code: "003",
            },
            VillageCode {
                name: "许巴村村委会",
                code: "004",
            },
            VillageCode {
                name: "推村村委会",
                code: "005",
            },
            VillageCode {
                name: "果玉村村委会",
                code: "006",
            },
            VillageCode {
                name: "宗琼村村委会",
                code: "007",
            },
            VillageCode {
                name: "玉湖村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "拉孜乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "觉瓦村村委会",
                code: "001",
            },
            VillageCode {
                name: "拉孜村村委会",
                code: "002",
            },
            VillageCode {
                name: "绕村村委会",
                code: "003",
            },
            VillageCode {
                name: "雄日村村委会",
                code: "004",
            },
            VillageCode {
                name: "珠村村委会",
                code: "005",
            },
            VillageCode {
                name: "如村村委会",
                code: "006",
            },
            VillageCode {
                name: "达孜村村委会",
                code: "007",
            },
            VillageCode {
                name: "岗水村村委会",
                code: "008",
            },
            VillageCode {
                name: "门贡村村委会",
                code: "009",
            },
            VillageCode {
                name: "根巴村村委会",
                code: "010",
            },
            VillageCode {
                name: "批果村村委会",
                code: "011",
            },
            VillageCode {
                name: "过查村村委会",
                code: "012",
            },
            VillageCode {
                name: "森卡村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "都瓦乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "卡达村村委会",
                code: "001",
            },
            VillageCode {
                name: "达多村村委会",
                code: "002",
            },
            VillageCode {
                name: "加荣村村委会",
                code: "003",
            },
            VillageCode {
                name: "扎根村村委会",
                code: "004",
            },
            VillageCode {
                name: "郭龙村村委会",
                code: "005",
            },
            VillageCode {
                name: "瓦地村村委会",
                code: "006",
            },
        ],
    },
];

static TOWNS_XK_018: [TownCode; 17] = [
    TownCode {
        name: "炉城街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "向阳社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "水井子社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "子耳社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "光明社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "清泉一村民委员会",
                code: "005",
            },
            VillageCode {
                name: "清泉二村民委员会",
                code: "006",
            },
            VillageCode {
                name: "升航村民委员会",
                code: "007",
            },
            VillageCode {
                name: "大风湾村民委员会",
                code: "008",
            },
            VillageCode {
                name: "白土村民委员会",
                code: "009",
            },
            VillageCode {
                name: "大河沟村民委员会",
                code: "010",
            },
            VillageCode {
                name: "子耳村民委员会",
                code: "011",
            },
            VillageCode {
                name: "柳杨村民委员会",
                code: "012",
            },
            VillageCode {
                name: "菜园子村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "榆林街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "公主桥社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "驷马桥社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "木雅社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "情歌社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "老榆林村民委员会",
                code: "005",
            },
            VillageCode {
                name: "折多塘村民委员会",
                code: "006",
            },
            VillageCode {
                name: "两岔路村民委员会",
                code: "007",
            },
            VillageCode {
                name: "新榆林村民委员会",
                code: "008",
            },
            VillageCode {
                name: "南无村民委员会",
                code: "009",
            },
            VillageCode {
                name: "跑马山村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "姑咱镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "第一居民委员会",
                code: "001",
            },
            VillageCode {
                name: "第二居民委员会",
                code: "002",
            },
            VillageCode {
                name: "第三居民委员会",
                code: "003",
            },
            VillageCode {
                name: "上瓦斯村民委员会",
                code: "004",
            },
            VillageCode {
                name: "下瓦斯村民委员会",
                code: "005",
            },
            VillageCode {
                name: "羊厂村民委员会",
                code: "006",
            },
            VillageCode {
                name: "章古村民委员会",
                code: "007",
            },
            VillageCode {
                name: "浸水村民委员会",
                code: "008",
            },
            VillageCode {
                name: "达杠村民委员会",
                code: "009",
            },
            VillageCode {
                name: "日地村民委员会",
                code: "010",
            },
            VillageCode {
                name: "时济村民委员会",
                code: "011",
            },
            VillageCode {
                name: "大坝村民委员会",
                code: "012",
            },
            VillageCode {
                name: "叫吉村民委员会",
                code: "013",
            },
            VillageCode {
                name: "日角村民委员会",
                code: "014",
            },
            VillageCode {
                name: "若吉村民委员会",
                code: "015",
            },
            VillageCode {
                name: "庄上村民委员会",
                code: "016",
            },
            VillageCode {
                name: "抗州村民委员会",
                code: "017",
            },
            VillageCode {
                name: "郎鼓村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "新都桥镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "新都桥镇居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新一村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新二村民委员会",
                code: "003",
            },
            VillageCode {
                name: "上柏桑一村民委员会",
                code: "004",
            },
            VillageCode {
                name: "上柏桑二村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下柏桑一村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下柏桑二村民委员会",
                code: "007",
            },
            VillageCode {
                name: "下柏桑三村民委员会",
                code: "008",
            },
            VillageCode {
                name: "拔桑一村民委员会",
                code: "009",
            },
            VillageCode {
                name: "拔桑二村民委员会",
                code: "010",
            },
            VillageCode {
                name: "东俄洛一村民委员会",
                code: "011",
            },
            VillageCode {
                name: "东俄洛二村民委员会",
                code: "012",
            },
            VillageCode {
                name: "东俄洛三村民委员会",
                code: "013",
            },
            VillageCode {
                name: "居里村民委员会",
                code: "014",
            },
            VillageCode {
                name: "营官村民委员会",
                code: "015",
            },
            VillageCode {
                name: "瓦泽村民委员会",
                code: "016",
            },
            VillageCode {
                name: "麦巴村民委员会",
                code: "017",
            },
            VillageCode {
                name: "安良村民委员会",
                code: "018",
            },
            VillageCode {
                name: "水桥村民委员会",
                code: "019",
            },
            VillageCode {
                name: "鱼子西二村民委员会",
                code: "020",
            },
            VillageCode {
                name: "鱼子西三村民委员会",
                code: "021",
            },
            VillageCode {
                name: "瓦板村民委员会",
                code: "022",
            },
            VillageCode {
                name: "贡巴村民委员会",
                code: "023",
            },
            VillageCode {
                name: "俄依村民委员会",
                code: "024",
            },
            VillageCode {
                name: "白玉村民委员会",
                code: "025",
            },
            VillageCode {
                name: "卡吾村民委员会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "塔公镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "夏马龙村民委员会",
                code: "001",
            },
            VillageCode {
                name: "色其卡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "江巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "各日马村民委员会",
                code: "004",
            },
            VillageCode {
                name: "龙古一村民委员会",
                code: "005",
            },
            VillageCode {
                name: "龙古二村民委员会",
                code: "006",
            },
            VillageCode {
                name: "八郎村民委员会",
                code: "007",
            },
            VillageCode {
                name: "古弄村民委员会",
                code: "008",
            },
            VillageCode {
                name: "日沙一村民委员会",
                code: "009",
            },
            VillageCode {
                name: "日沙二村民委员会",
                code: "010",
            },
            VillageCode {
                name: "日沙三村民委员会",
                code: "011",
            },
            VillageCode {
                name: "塔公村民委员会",
                code: "012",
            },
            VillageCode {
                name: "然罗村民委员会",
                code: "013",
            },
            VillageCode {
                name: "巴日村民委员会",
                code: "014",
            },
            VillageCode {
                name: "多拉村民委员会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "沙德镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "沙德村民委员会",
                code: "001",
            },
            VillageCode {
                name: "生古村民委员会",
                code: "002",
            },
            VillageCode {
                name: "瓦约村民委员会",
                code: "003",
            },
            VillageCode {
                name: "拉哈村民委员会",
                code: "004",
            },
            VillageCode {
                name: "上赤吉西村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下赤吉西村民委员会",
                code: "006",
            },
            VillageCode {
                name: "俄巴绒村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "金汤镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "汤坝村民委员会",
                code: "001",
            },
            VillageCode {
                name: "寇家河坝村民委员会",
                code: "002",
            },
            VillageCode {
                name: "青杠一村民委员会",
                code: "003",
            },
            VillageCode {
                name: "青杠二村民委员会",
                code: "004",
            },
            VillageCode {
                name: "青杠三村民委员会",
                code: "005",
            },
            VillageCode {
                name: "青杠四村民委员会",
                code: "006",
            },
            VillageCode {
                name: "新联上村民委员会",
                code: "007",
            },
            VillageCode {
                name: "新联下村民委员会",
                code: "008",
            },
            VillageCode {
                name: "新房子村民委员会",
                code: "009",
            },
            VillageCode {
                name: "高碉村民委员会",
                code: "010",
            },
            VillageCode {
                name: "陇须村民委员会",
                code: "011",
            },
            VillageCode {
                name: "先锋一村民委员会",
                code: "012",
            },
            VillageCode {
                name: "先锋二村民委员会",
                code: "013",
            },
            VillageCode {
                name: "先锋三村民委员会",
                code: "014",
            },
            VillageCode {
                name: "边坝村民委员会",
                code: "015",
            },
            VillageCode {
                name: "大火地村民委员会",
                code: "016",
            },
            VillageCode {
                name: "老五大寺村民委员会",
                code: "017",
            },
            VillageCode {
                name: "新五大寺村民委员会",
                code: "018",
            },
            VillageCode {
                name: "江坝村民委员会",
                code: "019",
            },
            VillageCode {
                name: "二郎村民委员会",
                code: "020",
            },
            VillageCode {
                name: "河坝村民委员会",
                code: "021",
            },
            VillageCode {
                name: "昌须村民委员会",
                code: "022",
            },
            VillageCode {
                name: "赤绒村民委员会",
                code: "023",
            },
            VillageCode {
                name: "庄子村民委员会",
                code: "024",
            },
            VillageCode {
                name: "昌坝村民委员会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "甲根坝镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "木雅村民委员会",
                code: "001",
            },
            VillageCode {
                name: "昌木村民委员会",
                code: "002",
            },
            VillageCode {
                name: "日泽村民委员会",
                code: "003",
            },
            VillageCode {
                name: "亚弄村民委员会",
                code: "004",
            },
            VillageCode {
                name: "立泽村民委员会",
                code: "005",
            },
            VillageCode {
                name: "日欧村民委员会",
                code: "006",
            },
            VillageCode {
                name: "提吾村民委员会",
                code: "007",
            },
            VillageCode {
                name: "阿加上村民委员会",
                code: "008",
            },
            VillageCode {
                name: "扎日村民委员会",
                code: "009",
            },
            VillageCode {
                name: "木枯村民委员会",
                code: "010",
            },
            VillageCode {
                name: "日头村民委员会",
                code: "011",
            },
            VillageCode {
                name: "木都村民委员会",
                code: "012",
            },
            VillageCode {
                name: "马达村民委员会",
                code: "013",
            },
            VillageCode {
                name: "夺让村民委员会",
                code: "014",
            },
            VillageCode {
                name: "日吾村民委员会",
                code: "015",
            },
            VillageCode {
                name: "江德村民委员会",
                code: "016",
            },
            VillageCode {
                name: "纳梯村民委员会",
                code: "017",
            },
            VillageCode {
                name: "提弄村民委员会",
                code: "018",
            },
            VillageCode {
                name: "马色村民委员会",
                code: "019",
            },
            VillageCode {
                name: "朋布西村民委员会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "贡嘎山镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "六巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "色乌绒一村民委员会",
                code: "002",
            },
            VillageCode {
                name: "色乌绒二村民委员会",
                code: "003",
            },
            VillageCode {
                name: "上木居村民委员会",
                code: "004",
            },
            VillageCode {
                name: "下木居村民委员会",
                code: "005",
            },
            VillageCode {
                name: "玉龙西村民委员会",
                code: "006",
            },
            VillageCode {
                name: "贡嘎山村民委员会",
                code: "007",
            },
            VillageCode {
                name: "程子村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "鱼通镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "干沟村民委员会",
                code: "001",
            },
            VillageCode {
                name: "野坝村民委员会",
                code: "002",
            },
            VillageCode {
                name: "牛棚子村民委员会",
                code: "003",
            },
            VillageCode {
                name: "舍联村民委员会",
                code: "004",
            },
            VillageCode {
                name: "勒树村民委员会",
                code: "005",
            },
            VillageCode {
                name: "龙安村民委员会",
                code: "006",
            },
            VillageCode {
                name: "赶羊村民委员会",
                code: "007",
            },
            VillageCode {
                name: "俄包村民委员会",
                code: "008",
            },
            VillageCode {
                name: "前溪村民委员会",
                code: "009",
            },
            VillageCode {
                name: "初咱村民委员会",
                code: "010",
            },
            VillageCode {
                name: "雄楼村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "雅拉乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "头道桥村民委员会",
                code: "001",
            },
            VillageCode {
                name: "二道桥村民委员会",
                code: "002",
            },
            VillageCode {
                name: "三道桥村民委员会",
                code: "003",
            },
            VillageCode {
                name: "曲公村民委员会",
                code: "004",
            },
            VillageCode {
                name: "蒙庆村民委员会",
                code: "005",
            },
            VillageCode {
                name: "新兴村民委员会",
                code: "006",
            },
            VillageCode {
                name: "王母村民委员会",
                code: "007",
            },
            VillageCode {
                name: "中谷村民委员会",
                code: "008",
            },
            VillageCode {
                name: "鱼斯村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "麦崩乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "为舍村民委员会",
                code: "001",
            },
            VillageCode {
                name: "日央村民委员会",
                code: "002",
            },
            VillageCode {
                name: "敏迁村民委员会",
                code: "003",
            },
            VillageCode {
                name: "磨子沟村民委员会",
                code: "004",
            },
            VillageCode {
                name: "昌昌村民委员会",
                code: "005",
            },
            VillageCode {
                name: "瓜达村民委员会",
                code: "006",
            },
            VillageCode {
                name: "厂马村民委员会",
                code: "007",
            },
            VillageCode {
                name: "含泥村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下火地村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "捧塔乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "新兴上村民委员会",
                code: "001",
            },
            VillageCode {
                name: "新兴下村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阳林村民委员会",
                code: "003",
            },
            VillageCode {
                name: "三家寨村民委员会",
                code: "004",
            },
            VillageCode {
                name: "捧塔村民委员会",
                code: "005",
            },
            VillageCode {
                name: "和平村民委员会",
                code: "006",
            },
            VillageCode {
                name: "团结村民委员会",
                code: "007",
            },
            VillageCode {
                name: "解放一村民委员会",
                code: "008",
            },
            VillageCode {
                name: "解放二村民委员会",
                code: "009",
            },
            VillageCode {
                name: "两河口村民委员会",
                code: "010",
            },
            VillageCode {
                name: "桐林村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "普沙绒乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "宜代村民委员会",
                code: "001",
            },
            VillageCode {
                name: "冰古村民委员会",
                code: "002",
            },
            VillageCode {
                name: "长草坪村民委员会",
                code: "003",
            },
            VillageCode {
                name: "莲花湖村民委员会",
                code: "004",
            },
            VillageCode {
                name: "火山村民委员会",
                code: "005",
            },
            VillageCode {
                name: "普沙绒村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "吉居乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "吉居村民委员会",
                code: "001",
            },
            VillageCode {
                name: "马蹄村民委员会",
                code: "002",
            },
            VillageCode {
                name: "各坝村民委员会",
                code: "003",
            },
            VillageCode {
                name: "宋玉村民委员会",
                code: "004",
            },
            VillageCode {
                name: "菜玉村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "呷巴乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "立启村民委员会",
                code: "001",
            },
            VillageCode {
                name: "呷巴上村民委员会",
                code: "002",
            },
            VillageCode {
                name: "呷巴下村民委员会",
                code: "003",
            },
            VillageCode {
                name: "自弄村民委员会",
                code: "004",
            },
            VillageCode {
                name: "司泽村民委员会",
                code: "005",
            },
            VillageCode {
                name: "塔拉上村民委员会",
                code: "006",
            },
            VillageCode {
                name: "塔拉下村民委员会",
                code: "007",
            },
            VillageCode {
                name: "铁索村民委员会",
                code: "008",
            },
            VillageCode {
                name: "具弄村民委员会",
                code: "009",
            },
            VillageCode {
                name: "木弄村民委员会",
                code: "010",
            },
            VillageCode {
                name: "俄达门巴村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "孔玉乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "阿斗沟村民委员会",
                code: "001",
            },
            VillageCode {
                name: "寸达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "挖郎村民委员会",
                code: "003",
            },
            VillageCode {
                name: "折骆村民委员会",
                code: "004",
            },
            VillageCode {
                name: "崩沙村民委员会",
                code: "005",
            },
            VillageCode {
                name: "巴郎村民委员会",
                code: "006",
            },
            VillageCode {
                name: "四家寨村民委员会",
                code: "007",
            },
            VillageCode {
                name: "莫玉村民委员会",
                code: "008",
            },
            VillageCode {
                name: "色龙村民委员会",
                code: "009",
            },
            VillageCode {
                name: "角坝村民委员会",
                code: "010",
            },
            VillageCode {
                name: "河坝村民委员会",
                code: "011",
            },
            VillageCode {
                name: "门坝村民委员会",
                code: "012",
            },
        ],
    },
];

static TOWNS_XK_019: [TownCode; 9] = [
    TownCode {
        name: "泸桥镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "南段社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "北段社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "沙坝社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "新桥村民委员会",
                code: "004",
            },
            VillageCode {
                name: "团结村民委员会",
                code: "005",
            },
            VillageCode {
                name: "咱里村民委员会",
                code: "006",
            },
            VillageCode {
                name: "泸桥村民委员会",
                code: "007",
            },
            VillageCode {
                name: "沙坝村民委员会",
                code: "008",
            },
            VillageCode {
                name: "安乐坝村民委员会",
                code: "009",
            },
            VillageCode {
                name: "大坝村民委员会",
                code: "010",
            },
            VillageCode {
                name: "押卓庄子村民委员会",
                code: "011",
            },
            VillageCode {
                name: "瓦窑岗村民委员会",
                code: "012",
            },
            VillageCode {
                name: "海子环环村民委员会",
                code: "013",
            },
            VillageCode {
                name: "三岔村民委员会",
                code: "014",
            },
            VillageCode {
                name: "田坝村民委员会",
                code: "015",
            },
            VillageCode {
                name: "下田村民委员会",
                code: "016",
            },
            VillageCode {
                name: "紫河村民委员会",
                code: "017",
            },
            VillageCode {
                name: "木杉村民委员会",
                code: "018",
            },
            VillageCode {
                name: "磨河村民委员会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "冷碛镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "老街道居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新街道居民委员会",
                code: "002",
            },
            VillageCode {
                name: "扒湾村民委员会",
                code: "003",
            },
            VillageCode {
                name: "团结村民委员会",
                code: "004",
            },
            VillageCode {
                name: "木瓜沟村民委员会",
                code: "005",
            },
            VillageCode {
                name: "桐子林村民委员会",
                code: "006",
            },
            VillageCode {
                name: "潘沟村民委员会",
                code: "007",
            },
            VillageCode {
                name: "黑沟村民委员会",
                code: "008",
            },
            VillageCode {
                name: "尖茶坪村民委员会",
                code: "009",
            },
            VillageCode {
                name: "甘露寺村民委员会",
                code: "010",
            },
            VillageCode {
                name: "松林村民委员会",
                code: "011",
            },
            VillageCode {
                name: "杵坭村民委员会",
                code: "012",
            },
            VillageCode {
                name: "邓油房村民委员会",
                code: "013",
            },
            VillageCode {
                name: "瓦斯营盘村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "兴隆镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "街道居民委员会",
                code: "001",
            },
            VillageCode {
                name: "兴隆村民委员会",
                code: "002",
            },
            VillageCode {
                name: "沈村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "堡子村民委员会",
                code: "004",
            },
            VillageCode {
                name: "乌支索村民委员会",
                code: "005",
            },
            VillageCode {
                name: "瓦斯村民委员会",
                code: "006",
            },
            VillageCode {
                name: "银厂村民委员会",
                code: "007",
            },
            VillageCode {
                name: "阳山村民委员会",
                code: "008",
            },
            VillageCode {
                name: "毛家寨村民委员会",
                code: "009",
            },
            VillageCode {
                name: "和平村民委员会",
                code: "010",
            },
            VillageCode {
                name: "牛背山村民委员会",
                code: "011",
            },
            VillageCode {
                name: "化林村民委员会",
                code: "012",
            },
            VillageCode {
                name: "盐水溪村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "磨西镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "老街社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "咱地村民委员会",
                code: "002",
            },
            VillageCode {
                name: "柏秧坪村民委员会",
                code: "003",
            },
            VillageCode {
                name: "大杉树村民委员会",
                code: "004",
            },
            VillageCode {
                name: "磨岗岭村民委员会",
                code: "005",
            },
            VillageCode {
                name: "青冈坪村民委员会",
                code: "006",
            },
            VillageCode {
                name: "共和村民委员会",
                code: "007",
            },
            VillageCode {
                name: "磨子沟村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "燕子沟镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "新龙门子村民委员会",
                code: "001",
            },
            VillageCode {
                name: "跃进坪村民委员会",
                code: "002",
            },
            VillageCode {
                name: "南门关村民委员会",
                code: "003",
            },
            VillageCode {
                name: "喇嘛沟村民委员会",
                code: "004",
            },
            VillageCode {
                name: "大坪村民委员会",
                code: "005",
            },
            VillageCode {
                name: "燕子沟村民委员会",
                code: "006",
            },
            VillageCode {
                name: "新兴村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "得妥镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "南头村民委员会",
                code: "001",
            },
            VillageCode {
                name: "发旺村民委员会",
                code: "002",
            },
            VillageCode {
                name: "北头村民委员会",
                code: "003",
            },
            VillageCode {
                name: "金光村民委员会",
                code: "004",
            },
            VillageCode {
                name: "天池山村民委员会",
                code: "005",
            },
            VillageCode {
                name: "椒子坪村民委员会",
                code: "006",
            },
            VillageCode {
                name: "马列村民委员会",
                code: "007",
            },
            VillageCode {
                name: "联合村民委员会",
                code: "008",
            },
            VillageCode {
                name: "紫雅场村民委员会",
                code: "009",
            },
            VillageCode {
                name: "湾东村民委员会",
                code: "010",
            },
            VillageCode {
                name: "友谊村民委员会",
                code: "011",
            },
            VillageCode {
                name: "幸福村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "烹坝镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "烹坝村民委员会",
                code: "001",
            },
            VillageCode {
                name: "沙湾村民委员会",
                code: "002",
            },
            VillageCode {
                name: "马厂村民委员会",
                code: "003",
            },
            VillageCode {
                name: "黄草坪村民委员会",
                code: "004",
            },
            VillageCode {
                name: "喇嘛寺村民委员会",
                code: "005",
            },
            VillageCode {
                name: "固包村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "德威镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "咱威村民委员会",
                code: "001",
            },
            VillageCode {
                name: "寨子村民委员会",
                code: "002",
            },
            VillageCode {
                name: "堡子村民委员会",
                code: "003",
            },
            VillageCode {
                name: "河坝村民委员会",
                code: "004",
            },
            VillageCode {
                name: "磨子村民委员会",
                code: "005",
            },
            VillageCode {
                name: "沙坝村民委员会",
                code: "006",
            },
            VillageCode {
                name: "奎武村民委员会",
                code: "007",
            },
            VillageCode {
                name: "刘河坝村民委员会",
                code: "008",
            },
            VillageCode {
                name: "安家湾村民委员会",
                code: "009",
            },
            VillageCode {
                name: "长沙坝村民委员会",
                code: "010",
            },
            VillageCode {
                name: "加郡村民委员会",
                code: "011",
            },
            VillageCode {
                name: "海子村民委员会",
                code: "012",
            },
            VillageCode {
                name: "金洞子村民委员会",
                code: "013",
            },
            VillageCode {
                name: "庄子村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "岚安乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "昂州村民委员会",
                code: "001",
            },
            VillageCode {
                name: "昂乌村民委员会",
                code: "002",
            },
            VillageCode {
                name: "脚乌村民委员会",
                code: "003",
            },
            VillageCode {
                name: "乌坭岗村民委员会",
                code: "004",
            },
        ],
    },
];

static TOWNS_XK_020: [TownCode; 12] = [
    TownCode {
        name: "章谷镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "三岔河居民委员会",
                code: "001",
            },
            VillageCode {
                name: "步行街居民委员会",
                code: "002",
            },
            VillageCode {
                name: "西河桥居民委员会",
                code: "003",
            },
            VillageCode {
                name: "五里牌居民委员会",
                code: "004",
            },
            VillageCode {
                name: "白呷依村民委员会",
                code: "005",
            },
            VillageCode {
                name: "城关村民委员会",
                code: "006",
            },
            VillageCode {
                name: "水子一村民委员会",
                code: "007",
            },
            VillageCode {
                name: "水子二村民委员会",
                code: "008",
            },
            VillageCode {
                name: "大马村民委员会",
                code: "009",
            },
            VillageCode {
                name: "长纳村民委员会",
                code: "010",
            },
            VillageCode {
                name: "各宗村民委员会",
                code: "011",
            },
            VillageCode {
                name: "边古村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "巴底镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "木尔洛村民委员会",
                code: "001",
            },
            VillageCode {
                name: "色足村民委员会",
                code: "002",
            },
            VillageCode {
                name: "沈洛村民委员会",
                code: "003",
            },
            VillageCode {
                name: "木尔约村民委员会",
                code: "004",
            },
            VillageCode {
                name: "木纳山村民委员会",
                code: "005",
            },
            VillageCode {
                name: "崃依村民委员会",
                code: "006",
            },
            VillageCode {
                name: "齐鲁村民委员会",
                code: "007",
            },
            VillageCode {
                name: "培尔村民委员会",
                code: "008",
            },
            VillageCode {
                name: "阿拉伯村民委员会",
                code: "009",
            },
            VillageCode {
                name: "柏松塘村民委员会",
                code: "010",
            },
            VillageCode {
                name: "木兰村民委员会",
                code: "011",
            },
            VillageCode {
                name: "沈足一村民委员会",
                code: "012",
            },
            VillageCode {
                name: "沈足二村民委员会",
                code: "013",
            },
            VillageCode {
                name: "牧业村民委员会",
                code: "014",
            },
            VillageCode {
                name: "二基坪村民委员会",
                code: "015",
            },
            VillageCode {
                name: "邛山村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "革什扎镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "布科村民委员会",
                code: "001",
            },
            VillageCode {
                name: "柯尔金村民委员会",
                code: "002",
            },
            VillageCode {
                name: "卓斯尼村民委员会",
                code: "003",
            },
            VillageCode {
                name: "吉牛村民委员会",
                code: "004",
            },
            VillageCode {
                name: "洛尔村民委员会",
                code: "005",
            },
            VillageCode {
                name: "俄洛村民委员会",
                code: "006",
            },
            VillageCode {
                name: "安古村民委员会",
                code: "007",
            },
            VillageCode {
                name: "累累村民委员会",
                code: "008",
            },
            VillageCode {
                name: "瓦坝村民委员会",
                code: "009",
            },
            VillageCode {
                name: "吉汝村民委员会",
                code: "010",
            },
            VillageCode {
                name: "大桑村民委员会",
                code: "011",
            },
            VillageCode {
                name: "瓦足村民委员会",
                code: "012",
            },
            VillageCode {
                name: "燕窝沟村民委员会",
                code: "013",
            },
            VillageCode {
                name: "三道桥村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "东谷镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "井备村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东谷村民委员会",
                code: "002",
            },
            VillageCode {
                name: "东马村民委员会",
                code: "003",
            },
            VillageCode {
                name: "牦牛村民委员会",
                code: "004",
            },
            VillageCode {
                name: "永西村民委员会",
                code: "005",
            },
            VillageCode {
                name: "拔冲村民委员会",
                code: "006",
            },
            VillageCode {
                name: "阴山村民委员会",
                code: "007",
            },
            VillageCode {
                name: "祚雅村民委员会",
                code: "008",
            },
            VillageCode {
                name: "科里村民委员会",
                code: "009",
            },
            VillageCode {
                name: "纳交村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "墨尔多山镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "克格依村民委员会",
                code: "001",
            },
            VillageCode {
                name: "基卡依村民委员会",
                code: "002",
            },
            VillageCode {
                name: "呷仁依村民委员会",
                code: "003",
            },
            VillageCode {
                name: "波色龙村民委员会",
                code: "004",
            },
            VillageCode {
                name: "罕额依村民委员会",
                code: "005",
            },
            VillageCode {
                name: "岭垄村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上纳顶村民委员会",
                code: "007",
            },
            VillageCode {
                name: "中纳顶村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下纳顶村民委员会",
                code: "009",
            },
            VillageCode {
                name: "斯交村民委员会",
                code: "010",
            },
            VillageCode {
                name: "岳扎街村民委员会",
                code: "011",
            },
            VillageCode {
                name: "岳扎坝村民委员会",
                code: "012",
            },
            VillageCode {
                name: "卡桠桥村民委员会",
                code: "013",
            },
            VillageCode {
                name: "红五月村民委员会",
                code: "014",
            },
            VillageCode {
                name: "前进村民委员会",
                code: "015",
            },
            VillageCode {
                name: "八科村民委员会",
                code: "016",
            },
            VillageCode {
                name: "科尔金村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "甲居镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "甲居一村民委员会",
                code: "001",
            },
            VillageCode {
                name: "甲居二村民委员会",
                code: "002",
            },
            VillageCode {
                name: "甲居三村民委员会",
                code: "003",
            },
            VillageCode {
                name: "聂呷村民委员会",
                code: "004",
            },
            VillageCode {
                name: "拖瓦村民委员会",
                code: "005",
            },
            VillageCode {
                name: "喀咔村民委员会",
                code: "006",
            },
            VillageCode {
                name: "幺姑村民委员会",
                code: "007",
            },
            VillageCode {
                name: "高顶村民委员会",
                code: "008",
            },
            VillageCode {
                name: "聂拉村民委员会",
                code: "009",
            },
            VillageCode {
                name: "小巴旺村民委员会",
                code: "010",
            },
            VillageCode {
                name: "小聂呷村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "格宗镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "格宗村民委员会",
                code: "001",
            },
            VillageCode {
                name: "江达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "朱家山村民委员会",
                code: "003",
            },
            VillageCode {
                name: "竹子沟村民委员会",
                code: "004",
            },
            VillageCode {
                name: "俄呷村民委员会",
                code: "005",
            },
            VillageCode {
                name: "开绕村民委员会",
                code: "006",
            },
            VillageCode {
                name: "江口新村民委员会",
                code: "007",
            },
            VillageCode {
                name: "羊马村民委员会",
                code: "008",
            },
            VillageCode {
                name: "龙坝村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "半扇门镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "阿娘沟一村民委员会",
                code: "001",
            },
            VillageCode {
                name: "阿娘沟二村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阿娘沟四村民委员会",
                code: "003",
            },
            VillageCode {
                name: "阿娘寨村民委员会",
                code: "004",
            },
            VillageCode {
                name: "腊月山一村民委员会",
                code: "005",
            },
            VillageCode {
                name: "腊月山三村民委员会",
                code: "006",
            },
            VillageCode {
                name: "火龙沟一村民委员会",
                code: "007",
            },
            VillageCode {
                name: "火龙沟二村民委员会",
                code: "008",
            },
            VillageCode {
                name: "半扇门村民委员会",
                code: "009",
            },
            VillageCode {
                name: "碉坪村民委员会",
                code: "010",
            },
            VillageCode {
                name: "核桃坪村民委员会",
                code: "011",
            },
            VillageCode {
                name: "麦龙村民委员会",
                code: "012",
            },
            VillageCode {
                name: "大邑村民委员会",
                code: "013",
            },
            VillageCode {
                name: "团结村民委员会",
                code: "014",
            },
            VillageCode {
                name: "关州村民委员会",
                code: "015",
            },
            VillageCode {
                name: "喇嘛寺村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "丹东镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "边耳村民委员会",
                code: "001",
            },
            VillageCode {
                name: "二马村民委员会",
                code: "002",
            },
            VillageCode {
                name: "二瓦槽村民委员会",
                code: "003",
            },
            VillageCode {
                name: "党岭村民委员会",
                code: "004",
            },
            VillageCode {
                name: "牙科村民委员会",
                code: "005",
            },
            VillageCode {
                name: "各尔沟村民委员会",
                code: "006",
            },
            VillageCode {
                name: "磨子沟村民委员会",
                code: "007",
            },
            VillageCode {
                name: "丹东村民委员会",
                code: "008",
            },
            VillageCode {
                name: "二道桥村民委员会",
                code: "009",
            },
            VillageCode {
                name: "莫斯卡村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "巴旺乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "卡卡村民委员会",
                code: "001",
            },
            VillageCode {
                name: "水卡子村民委员会",
                code: "002",
            },
            VillageCode {
                name: "齐支村民委员会",
                code: "003",
            },
            VillageCode {
                name: "光都村民委员会",
                code: "004",
            },
            VillageCode {
                name: "燕尔岩村民委员会",
                code: "005",
            },
            VillageCode {
                name: "德洛村民委员会",
                code: "006",
            },
            VillageCode {
                name: "扎科村民委员会",
                code: "007",
            },
            VillageCode {
                name: "格呷村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "梭坡乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "纳依村民委员会",
                code: "001",
            },
            VillageCode {
                name: "左比村民委员会",
                code: "002",
            },
            VillageCode {
                name: "莫洛村民委员会",
                code: "003",
            },
            VillageCode {
                name: "共布村民委员会",
                code: "004",
            },
            VillageCode {
                name: "弄中村民委员会",
                code: "005",
            },
            VillageCode {
                name: "泽周村民委员会",
                code: "006",
            },
            VillageCode {
                name: "呷拉村民委员会",
                code: "007",
            },
            VillageCode {
                name: "泽公村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "太平桥乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "上宅龙村民委员会",
                code: "001",
            },
            VillageCode {
                name: "太平桥村民委员会",
                code: "002",
            },
            VillageCode {
                name: "下宅龙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "黑风顶村民委员会",
                code: "004",
            },
            VillageCode {
                name: "丹扎村民委员会",
                code: "005",
            },
            VillageCode {
                name: "纳粘村民委员会",
                code: "006",
            },
            VillageCode {
                name: "三木扎村民委员会",
                code: "007",
            },
            VillageCode {
                name: "各洛寨村民委员会",
                code: "008",
            },
            VillageCode {
                name: "长胜店村民委员会",
                code: "009",
            },
        ],
    },
];

static TOWNS_XK_021: [TownCode; 16] = [
    TownCode {
        name: "呷尔镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "民族广场社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "文化路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "狮子山社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "呷尔村民委员会",
                code: "004",
            },
            VillageCode {
                name: "华丘村民委员会",
                code: "005",
            },
            VillageCode {
                name: "察尔村民委员会",
                code: "006",
            },
            VillageCode {
                name: "扎日村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "烟袋镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "祥和社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "桤木林村民委员会",
                code: "002",
            },
            VillageCode {
                name: "白岩子村民委员会",
                code: "003",
            },
            VillageCode {
                name: "烟袋村民委员会",
                code: "004",
            },
            VillageCode {
                name: "毛菇厂村民委员会",
                code: "005",
            },
            VillageCode {
                name: "烂铺子村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "三垭镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "龙塘子村民委员会",
                code: "001",
            },
            VillageCode {
                name: "老鸹村民委员会",
                code: "002",
            },
            VillageCode {
                name: "郎呷村民委员会",
                code: "003",
            },
            VillageCode {
                name: "马颈子村民委员会",
                code: "004",
            },
            VillageCode {
                name: "俄尔村民委员会",
                code: "005",
            },
            VillageCode {
                name: "大铺子村民委员会",
                code: "006",
            },
            VillageCode {
                name: "洼铺子村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "雪洼龙镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "耳朵村民委员会",
                code: "001",
            },
            VillageCode {
                name: "甲铺子村民委员会",
                code: "002",
            },
            VillageCode {
                name: "花椒坪村民委员会",
                code: "003",
            },
            VillageCode {
                name: "河口村民委员会",
                code: "004",
            },
            VillageCode {
                name: "洛让村民委员会",
                code: "005",
            },
            VillageCode {
                name: "雪洼村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "湾坝镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "高碉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "挖金村民委员会",
                code: "002",
            },
            VillageCode {
                name: "草坪子村民委员会",
                code: "003",
            },
            VillageCode {
                name: "湾子村民委员会",
                code: "004",
            },
            VillageCode {
                name: "小伙房村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "汤古镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "汤古村民委员会",
                code: "001",
            },
            VillageCode {
                name: "崩崩冲村民委员会",
                code: "002",
            },
            VillageCode {
                name: "伍须村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "乌拉溪镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "河坝村民委员会",
                code: "001",
            },
            VillageCode {
                name: "坡上村民委员会",
                code: "002",
            },
            VillageCode {
                name: "石头沟村民委员会",
                code: "003",
            },
            VillageCode {
                name: "偏桥村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "魁多镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "魁多村民委员会",
                code: "001",
            },
            VillageCode {
                name: "里伍村民委员会",
                code: "002",
            },
            VillageCode {
                name: "甲坝村民委员会",
                code: "003",
            },
            VillageCode {
                name: "海底村民委员会",
                code: "004",
            },
            VillageCode {
                name: "江郎村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "乃渠镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "七日村民委员会",
                code: "001",
            },
            VillageCode {
                name: "水打坝村民委员会",
                code: "002",
            },
            VillageCode {
                name: "烂碉村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "三岩龙乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "柏林村民委员会",
                code: "001",
            },
            VillageCode {
                name: "田根村民委员会",
                code: "002",
            },
            VillageCode {
                name: "白杨坪村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "上团乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "放马坪村民委员会",
                code: "001",
            },
            VillageCode {
                name: "运脚村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "八窝龙乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "下铺子村民委员会",
                code: "001",
            },
            VillageCode {
                name: "烂尼巴村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "子耳彝族乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "庙子坪村民委员会",
                code: "001",
            },
            VillageCode {
                name: "杜公村民委员会",
                code: "002",
            },
            VillageCode {
                name: "万年村民委员会",
                code: "003",
            },
            VillageCode {
                name: "麻窝村民委员会",
                code: "004",
            },
            VillageCode {
                name: "银厂弯村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "小金彝族乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "小金村民委员会",
                code: "001",
            },
            VillageCode {
                name: "碉房村民委员会",
                code: "002",
            },
            VillageCode {
                name: "洋桥村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "朵洛彝族乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "曲窝村民委员会",
                code: "001",
            },
            VillageCode {
                name: "船板沟村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "洪坝乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "中心村民委员会",
                code: "001",
            },
            VillageCode {
                name: "羊圈门村民委员会",
                code: "002",
            },
        ],
    },
];

static TOWNS_XK_022: [TownCode; 16] = [
    TownCode {
        name: "河口镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "河口镇居民委员会",
                code: "001",
            },
            VillageCode {
                name: "渡口社区委员会",
                code: "002",
            },
            VillageCode {
                name: "城厢村民委员会",
                code: "003",
            },
            VillageCode {
                name: "本达宗村民委员会",
                code: "004",
            },
            VillageCode {
                name: "三道桥村民委员会",
                code: "005",
            },
            VillageCode {
                name: "山背后村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下渡村民委员会",
                code: "007",
            },
            VillageCode {
                name: "麻子石村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "呷拉镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "呷拉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "西地村民委员会",
                code: "002",
            },
            VillageCode {
                name: "脚泥堡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "白孜村民委员会",
                code: "004",
            },
            VillageCode {
                name: "苦乐村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "西俄洛镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "康巴汉子村民委员会",
                code: "001",
            },
            VillageCode {
                name: "俄洛堆村民委员会",
                code: "002",
            },
            VillageCode {
                name: "汪堆村民委员会",
                code: "003",
            },
            VillageCode {
                name: "苦则村民委员会",
                code: "004",
            },
            VillageCode {
                name: "牛角洞村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "红龙镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "措柯一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "措柯二村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "东来一村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东来二村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "马它马村村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "麻郎措镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "麻郎措村民委员会",
                code: "001",
            },
            VillageCode {
                name: "唐俄村民委员会",
                code: "002",
            },
            VillageCode {
                name: "巴德村民委员会",
                code: "003",
            },
            VillageCode {
                name: "唐足村民委员会",
                code: "004",
            },
            VillageCode {
                name: "牙巴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "唐岗村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "波斯河镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "孜河村民委员会",
                code: "001",
            },
            VillageCode {
                name: "南根村民委员会",
                code: "002",
            },
            VillageCode {
                name: "邓科村民委员会",
                code: "003",
            },
            VillageCode {
                name: "雨日村民委员会",
                code: "004",
            },
            VillageCode {
                name: "下日村民委员会",
                code: "005",
            },
            VillageCode {
                name: "日衣村民委员会",
                code: "006",
            },
            VillageCode {
                name: "俄古村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "八角楼乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "八角楼村民委员会",
                code: "001",
            },
            VillageCode {
                name: "松茸村民委员会",
                code: "002",
            },
            VillageCode {
                name: "木泽西村民委员会",
                code: "003",
            },
            VillageCode {
                name: "扎日村民委员会",
                code: "004",
            },
            VillageCode {
                name: "同达村民委员会",
                code: "005",
            },
            VillageCode {
                name: "维锡村民委员会",
                code: "006",
            },
            VillageCode {
                name: "王呷村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "普巴绒乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "甲德村民委员会",
                code: "001",
            },
            VillageCode {
                name: "亚中村民委员会",
                code: "002",
            },
            VillageCode {
                name: "普古村民委员会",
                code: "003",
            },
            VillageCode {
                name: "日孜村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "祝桑乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "奔达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "夺雅宗村民委员会",
                code: "002",
            },
            VillageCode {
                name: "真达村民委员会",
                code: "003",
            },
            VillageCode {
                name: "尼玛宗村民委员会",
                code: "004",
            },
            VillageCode {
                name: "本孜村民委员会",
                code: "005",
            },
            VillageCode {
                name: "德察村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "米龙乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "米龙村民委员会",
                code: "001",
            },
            VillageCode {
                name: "程章村民委员会",
                code: "002",
            },
            VillageCode {
                name: "然公村民委员会",
                code: "003",
            },
            VillageCode {
                name: "陇冬村民委员会",
                code: "004",
            },
            VillageCode {
                name: "本孜绒村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "八衣绒乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "格日村民委员会",
                code: "001",
            },
            VillageCode {
                name: "木灰村民委员会",
                code: "002",
            },
            VillageCode {
                name: "茨马绒村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "牙衣河乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "牙衣河村民委员会",
                code: "001",
            },
            VillageCode {
                name: "木恩村民委员会",
                code: "002",
            },
            VillageCode {
                name: "江中堂村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "德差乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "下德差村民委员会",
                code: "001",
            },
            VillageCode {
                name: "中德差村民委员会",
                code: "002",
            },
            VillageCode {
                name: "上德差村民委员会",
                code: "003",
            },
            VillageCode {
                name: "吕村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "布孜村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "柯拉乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "解放村民委员会",
                code: "001",
            },
            VillageCode {
                name: "益因一村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "伙拉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "益因村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "瓦多乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "交吾村民委员会",
                code: "001",
            },
            VillageCode {
                name: "杜米村民委员会",
                code: "002",
            },
            VillageCode {
                name: "白龙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "中古村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "木绒乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "安桂村民委员会",
                code: "001",
            },
            VillageCode {
                name: "木绒村民委员会",
                code: "002",
            },
            VillageCode {
                name: "沙学村民委员会",
                code: "003",
            },
            VillageCode {
                name: "新卫村民委员会",
                code: "004",
            },
        ],
    },
];

static TOWNS_XK_023: [TownCode; 19] = [
    TownCode {
        name: "鲜水镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "鲜水居民委员会",
                code: "001",
            },
            VillageCode {
                name: "易水居民委员会",
                code: "002",
            },
            VillageCode {
                name: "孜龙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "道孚沟村民委员会",
                code: "004",
            },
            VillageCode {
                name: "团结村民委员会",
                code: "005",
            },
            VillageCode {
                name: "东门村民委员会",
                code: "006",
            },
            VillageCode {
                name: "胜利村民委员会",
                code: "007",
            },
            VillageCode {
                name: "前进村民委员会",
                code: "008",
            },
            VillageCode {
                name: "足湾村民委员会",
                code: "009",
            },
            VillageCode {
                name: "勒斯加村民委员会",
                code: "010",
            },
            VillageCode {
                name: "易日村民委员会",
                code: "011",
            },
            VillageCode {
                name: "亚洛加村民委员会",
                code: "012",
            },
            VillageCode {
                name: "朱倭村民委员会",
                code: "013",
            },
            VillageCode {
                name: "新江沟村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "八美镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "八美镇居民委员会",
                code: "001",
            },
            VillageCode {
                name: "曲尔村民委员会",
                code: "002",
            },
            VillageCode {
                name: "莎江村民委员会",
                code: "003",
            },
            VillageCode {
                name: "中谷村民委员会",
                code: "004",
            },
            VillageCode {
                name: "少乌村民委员会",
                code: "005",
            },
            VillageCode {
                name: "卡马村民委员会",
                code: "006",
            },
            VillageCode {
                name: "河垭村民委员会",
                code: "007",
            },
            VillageCode {
                name: "志麦通村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下瓦西村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "亚卓镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "乌拉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "容须卡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "亚玛子村民委员会",
                code: "003",
            },
            VillageCode {
                name: "盘龙村民委员会",
                code: "004",
            },
            VillageCode {
                name: "红顶村民委员会",
                code: "005",
            },
            VillageCode {
                name: "地入村民委员会",
                code: "006",
            },
            VillageCode {
                name: "呷拉坎村民委员会",
                code: "007",
            },
            VillageCode {
                name: "扎西岭村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "玉科镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "银克村民委员会",
                code: "001",
            },
            VillageCode {
                name: "兴岛科村民委员会",
                code: "002",
            },
            VillageCode {
                name: "维柯村民委员会",
                code: "003",
            },
            VillageCode {
                name: "呷科村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "仲尼镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "亚中村民委员会",
                code: "001",
            },
            VillageCode {
                name: "麻中村民委员会",
                code: "002",
            },
            VillageCode {
                name: "向秋村民委员会",
                code: "003",
            },
            VillageCode {
                name: "俄估村民委员会",
                code: "004",
            },
            VillageCode {
                name: "扎然村民委员会",
                code: "005",
            },
            VillageCode {
                name: "牧玖村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "泰宁镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "街村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "先锋村民委员会",
                code: "002",
            },
            VillageCode {
                name: "下一村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下二村民委员会",
                code: "004",
            },
            VillageCode {
                name: "上农村民委员会",
                code: "005",
            },
            VillageCode {
                name: "上牧村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "瓦日镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "尧日村民委员会",
                code: "001",
            },
            VillageCode {
                name: "卓卡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "热瓦村民委员会",
                code: "003",
            },
            VillageCode {
                name: "布日窝村民委员会",
                code: "004",
            },
            VillageCode {
                name: "孟拖村民委员会",
                code: "005",
            },
            VillageCode {
                name: "扎嘎村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "麻孜乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "菜孜坡村民委员会",
                code: "001",
            },
            VillageCode {
                name: "沟普村民委员会",
                code: "002",
            },
            VillageCode {
                name: "功龙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "德尔瓦村民委员会",
                code: "004",
            },
            VillageCode {
                name: "洛尔瓦村民委员会",
                code: "005",
            },
            VillageCode {
                name: "居日村民委员会",
                code: "006",
            },
            VillageCode {
                name: "特尔瓦村民委员会",
                code: "007",
            },
            VillageCode {
                name: "崩龙村民委员会",
                code: "008",
            },
            VillageCode {
                name: "油龙村民委员会",
                code: "009",
            },
            VillageCode {
                name: "小各卡村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "孔色乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "金卡村民委员会",
                code: "001",
            },
            VillageCode {
                name: "克郎村民委员会",
                code: "002",
            },
            VillageCode {
                name: "麻湾村民委员会",
                code: "003",
            },
            VillageCode {
                name: "约威村民委员会",
                code: "004",
            },
            VillageCode {
                name: "瓦依村民委员会",
                code: "005",
            },
            VillageCode {
                name: "亚拖村民委员会",
                code: "006",
            },
            VillageCode {
                name: "昌孜村民委员会",
                code: "007",
            },
            VillageCode {
                name: "格勒村民委员会",
                code: "008",
            },
            VillageCode {
                name: "若斯拉村民委员会",
                code: "009",
            },
            VillageCode {
                name: "呷拉村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "葛卡乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "甲拨村民委员会",
                code: "001",
            },
            VillageCode {
                name: "觉洛寺村民委员会",
                code: "002",
            },
            VillageCode {
                name: "各卡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "加拉宗村民委员会",
                code: "004",
            },
            VillageCode {
                name: "冻坡呷村民委员会",
                code: "005",
            },
            VillageCode {
                name: "沙湾村民委员会",
                code: "006",
            },
            VillageCode {
                name: "农甫村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "扎拖乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "波洛塘村民委员会",
                code: "001",
            },
            VillageCode {
                name: "一地瓦孜村民委员会",
                code: "002",
            },
            VillageCode {
                name: "扎贡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "扎拖村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "下拖乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "下瓦然村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上瓦然村民委员会",
                code: "002",
            },
            VillageCode {
                name: "麦里村民委员会",
                code: "003",
            },
            VillageCode {
                name: "杰荣村民委员会",
                code: "004",
            },
            VillageCode {
                name: "德吉村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "木茹乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "瓦达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "克尔鲁村民委员会",
                code: "002",
            },
            VillageCode {
                name: "牧业村民委员会",
                code: "003",
            },
            VillageCode {
                name: "古碉村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "甲斯孔乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "卡美村民委员会",
                code: "001",
            },
            VillageCode {
                name: "故拥村民委员会",
                code: "002",
            },
            VillageCode {
                name: "热鲁村民委员会",
                code: "003",
            },
            VillageCode {
                name: "洛须卡村民委员会",
                code: "004",
            },
            VillageCode {
                name: "上甲村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下甲村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "七美乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "曲龙科村民委员会",
                code: "001",
            },
            VillageCode {
                name: "五重科村民委员会",
                code: "002",
            },
            VillageCode {
                name: "白日村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "银恩乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "沙玛尔科村民委员会",
                code: "001",
            },
            VillageCode {
                name: "却哇鲁科村民委员会",
                code: "002",
            },
            VillageCode {
                name: "脚窝村民委员会",
                code: "003",
            },
            VillageCode {
                name: "呷郎村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "龙灯乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "拉日村民委员会",
                code: "001",
            },
            VillageCode {
                name: "燃姑村民委员会",
                code: "002",
            },
            VillageCode {
                name: "柯尔卡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "挪吾托村民委员会",
                code: "004",
            },
            VillageCode {
                name: "夏普隆村民委员会",
                code: "005",
            },
            VillageCode {
                name: "集寺中村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "色卡乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "茶垭村民委员会",
                code: "001",
            },
            VillageCode {
                name: "龙布村民委员会",
                code: "002",
            },
            VillageCode {
                name: "亚日村民委员会",
                code: "003",
            },
            VillageCode {
                name: "扎日村民委员会",
                code: "004",
            },
            VillageCode {
                name: "建巴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "格西村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "沙冲乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "比里村民委员会",
                code: "001",
            },
            VillageCode {
                name: "吉亚村民委员会",
                code: "002",
            },
            VillageCode {
                name: "白马村民委员会",
                code: "003",
            },
        ],
    },
];

static TOWNS_XK_024: [TownCode; 15] = [
    TownCode {
        name: "新都镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "望果社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "章谷社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "霍尔社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "幸福社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "益娘村民委员会",
                code: "005",
            },
            VillageCode {
                name: "上街村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下街村民委员会",
                code: "007",
            },
            VillageCode {
                name: "七湾村民委员会",
                code: "008",
            },
            VillageCode {
                name: "查尔瓦村民委员会",
                code: "009",
            },
            VillageCode {
                name: "昌龙村民委员会",
                code: "010",
            },
            VillageCode {
                name: "格色村民委员会",
                code: "011",
            },
            VillageCode {
                name: "新都一村民委员会",
                code: "012",
            },
            VillageCode {
                name: "新都二村民委员会",
                code: "013",
            },
            VillageCode {
                name: "新都三村民委员会",
                code: "014",
            },
            VillageCode {
                name: "朱德村民委员会",
                code: "015",
            },
            VillageCode {
                name: "色德村民委员会",
                code: "016",
            },
            VillageCode {
                name: "秋日村民委员会",
                code: "017",
            },
            VillageCode {
                name: "俄日村民委员会",
                code: "018",
            },
            VillageCode {
                name: "德拉龙村民委员会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "朱倭镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "更达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "杜柏村民委员会",
                code: "002",
            },
            VillageCode {
                name: "朱倭村民委员会",
                code: "003",
            },
            VillageCode {
                name: "虾扎村民委员会",
                code: "004",
            },
            VillageCode {
                name: "颠古村民委员会",
                code: "005",
            },
            VillageCode {
                name: "卡烈村民委员会",
                code: "006",
            },
            VillageCode {
                name: "克羊壁村民委员会",
                code: "007",
            },
            VillageCode {
                name: "日郎达村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "虾拉沱镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "虾拉沱村民委员会",
                code: "001",
            },
            VillageCode {
                name: "独马村民委员会",
                code: "002",
            },
            VillageCode {
                name: "戈巴龙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "热固村民委员会",
                code: "004",
            },
            VillageCode {
                name: "斯中村民委员会",
                code: "005",
            },
            VillageCode {
                name: "绒巴龙村民委员会",
                code: "006",
            },
            VillageCode {
                name: "阿拉沟充古村民委员会",
                code: "007",
            },
            VillageCode {
                name: "邓达村民委员会",
                code: "008",
            },
            VillageCode {
                name: "章达村民委员会",
                code: "009",
            },
            VillageCode {
                name: "忠仁达村民委员会",
                code: "010",
            },
            VillageCode {
                name: "吉绒村民委员会",
                code: "011",
            },
            VillageCode {
                name: "克木村民委员会",
                code: "012",
            },
            VillageCode {
                name: "扎交村民委员会",
                code: "013",
            },
            VillageCode {
                name: "若海村民委员会",
                code: "014",
            },
            VillageCode {
                name: "阿初村民委员会",
                code: "015",
            },
            VillageCode {
                name: "色色村民委员会",
                code: "016",
            },
            VillageCode {
                name: "尤斯村民委员会",
                code: "017",
            },
            VillageCode {
                name: "瓦达村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "上罗柯马镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "二村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "三村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "四村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "德庆村民委员会",
                code: "005",
            },
            VillageCode {
                name: "八村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "扎特尔村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "泥巴乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "次郎村民委员会",
                code: "001",
            },
            VillageCode {
                name: "棒达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "朱巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "易绕村民委员会",
                code: "004",
            },
            VillageCode {
                name: "古西村民委员会",
                code: "005",
            },
            VillageCode {
                name: "呷巴村民委员会",
                code: "006",
            },
            VillageCode {
                name: "四季村民委员会",
                code: "007",
            },
            VillageCode {
                name: "旺达村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "雅德乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "邓达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "昌达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "然柳村民委员会",
                code: "003",
            },
            VillageCode {
                name: "格鲁村民委员会",
                code: "004",
            },
            VillageCode {
                name: "瓦角村民委员会",
                code: "005",
            },
            VillageCode {
                name: "降达村民委员会",
                code: "006",
            },
            VillageCode {
                name: "小安批村民委员会",
                code: "007",
            },
            VillageCode {
                name: "康古村民委员会",
                code: "008",
            },
            VillageCode {
                name: "须须村民委员会",
                code: "009",
            },
            VillageCode {
                name: "交纳村民委员会",
                code: "010",
            },
            VillageCode {
                name: "固理村民委员会",
                code: "011",
            },
            VillageCode {
                name: "布麦贡村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "洛秋乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "穷各村民委员会",
                code: "001",
            },
            VillageCode {
                name: "瓦贡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "洛尔巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "然玛贡村民委员会",
                code: "004",
            },
            VillageCode {
                name: "洛尔巴新村民委员会",
                code: "005",
            },
            VillageCode {
                name: "易日村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "仁达乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "仁达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "格色村民委员会",
                code: "002",
            },
            VillageCode {
                name: "呷拉宗村民委员会",
                code: "003",
            },
            VillageCode {
                name: "勒格村民委员会",
                code: "004",
            },
            VillageCode {
                name: "玉麦比村民委员会",
                code: "005",
            },
            VillageCode {
                name: "易日村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "旦都乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "秋所村民委员会",
                code: "001",
            },
            VillageCode {
                name: "加斗村民委员会",
                code: "002",
            },
            VillageCode {
                name: "沙湾村民委员会",
                code: "003",
            },
            VillageCode {
                name: "更达村民委员会",
                code: "004",
            },
            VillageCode {
                name: "马居村民委员会",
                code: "005",
            },
            VillageCode {
                name: "加郎村民委员会",
                code: "006",
            },
            VillageCode {
                name: "蚌达村民委员会",
                code: "007",
            },
            VillageCode {
                name: "克里村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "充古乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "充古村民委员会",
                code: "001",
            },
            VillageCode {
                name: "卡莎村民委员会",
                code: "002",
            },
            VillageCode {
                name: "德依村民委员会",
                code: "003",
            },
            VillageCode {
                name: "阿都村民委员会",
                code: "004",
            },
            VillageCode {
                name: "进达村民委员会",
                code: "005",
            },
            VillageCode {
                name: "马交村民委员会",
                code: "006",
            },
            VillageCode {
                name: "卓若村民委员会",
                code: "007",
            },
            VillageCode {
                name: "青卡村民委员会",
                code: "008",
            },
            VillageCode {
                name: "各汝村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "更知乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "知日玛二村民委员会",
                code: "001",
            },
            VillageCode {
                name: "更达二村民委员会",
                code: "002",
            },
            VillageCode {
                name: "措口村民委员会",
                code: "003",
            },
            VillageCode {
                name: "瓦亚村民委员会",
                code: "004",
            },
            VillageCode {
                name: "知加村民委员会",
                code: "005",
            },
            VillageCode {
                name: "德雅村民委员会",
                code: "006",
            },
            VillageCode {
                name: "修贡村民委员会",
                code: "007",
            },
            VillageCode {
                name: "八一村民委员会",
                code: "008",
            },
            VillageCode {
                name: "扎龙村民委员会",
                code: "009",
            },
            VillageCode {
                name: "提贡村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "卡娘乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "吉扎村民委员会",
                code: "001",
            },
            VillageCode {
                name: "卡娘村民委员会",
                code: "002",
            },
            VillageCode {
                name: "杜瓦村民委员会",
                code: "003",
            },
            VillageCode {
                name: "知底村民委员会",
                code: "004",
            },
            VillageCode {
                name: "觉底村民委员会",
                code: "005",
            },
            VillageCode {
                name: "知日村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东谷村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "宗塔乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "色柯马村民委员会",
                code: "001",
            },
            VillageCode {
                name: "塔瓦村民委员会",
                code: "002",
            },
            VillageCode {
                name: "岗柯村民委员会",
                code: "003",
            },
            VillageCode {
                name: "拉恰玛村民委员会",
                code: "004",
            },
            VillageCode {
                name: "吉柯村民委员会",
                code: "005",
            },
            VillageCode {
                name: "角龙村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "宗麦乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "本学村民委员会",
                code: "001",
            },
            VillageCode {
                name: "绒沙马村民委员会",
                code: "002",
            },
            VillageCode {
                name: "三果村民委员会",
                code: "003",
            },
            VillageCode {
                name: "阿吾村民委员会",
                code: "004",
            },
            VillageCode {
                name: "呷麦村民委员会",
                code: "005",
            },
            VillageCode {
                name: "然育村民委员会",
                code: "006",
            },
            VillageCode {
                name: "双马村民委员会",
                code: "007",
            },
            VillageCode {
                name: "阿拉村民委员会",
                code: "008",
            },
            VillageCode {
                name: "交西村民委员会",
                code: "009",
            },
            VillageCode {
                name: "托塔村民委员会",
                code: "010",
            },
            VillageCode {
                name: "仁真村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "下罗柯马乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "马庆瓦村民委员会",
                code: "001",
            },
            VillageCode {
                name: "日阿塔马村民委员会",
                code: "002",
            },
            VillageCode {
                name: "加它马村民委员会",
                code: "003",
            },
            VillageCode {
                name: "达色村民委员会",
                code: "004",
            },
            VillageCode {
                name: "阿色玛村民委员会",
                code: "005",
            },
            VillageCode {
                name: "甲色村民委员会",
                code: "006",
            },
            VillageCode {
                name: "玖玛村民委员会",
                code: "007",
            },
            VillageCode {
                name: "阿拉村民委员会",
                code: "008",
            },
        ],
    },
];

static TOWNS_XK_025: [TownCode; 21] = [
    TownCode {
        name: "甘孜镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "清河街社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "旭日林社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "解放街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "新区社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "德巴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "瓦巴村民委员会",
                code: "006",
            },
            VillageCode {
                name: "根布夏村民委员会",
                code: "007",
            },
            VillageCode {
                name: "麻达卡村民委员会",
                code: "008",
            },
            VillageCode {
                name: "错地卡村民委员会",
                code: "009",
            },
            VillageCode {
                name: "甲布卡村民委员会",
                code: "010",
            },
            VillageCode {
                name: "西戈甲村民委员会",
                code: "011",
            },
            VillageCode {
                name: "青柯普底村民委员会",
                code: "012",
            },
            VillageCode {
                name: "仁色底村民委员会",
                code: "013",
            },
            VillageCode {
                name: "卡加卡村民委员会",
                code: "014",
            },
            VillageCode {
                name: "甲卡村民委员会",
                code: "015",
            },
            VillageCode {
                name: "普苏村民委员会",
                code: "016",
            },
            VillageCode {
                name: "九日村民委员会",
                code: "017",
            },
            VillageCode {
                name: "门达村民委员会",
                code: "018",
            },
            VillageCode {
                name: "河坝村民委员会",
                code: "019",
            },
            VillageCode {
                name: "打金滩村民委员会",
                code: "020",
            },
            VillageCode {
                name: "新市区村民委员会",
                code: "021",
            },
            VillageCode {
                name: "雅桥村民委员会",
                code: "022",
            },
            VillageCode {
                name: "曲勒朗果村民委员会",
                code: "023",
            },
            VillageCode {
                name: "然巴然格村民委员会",
                code: "024",
            },
            VillageCode {
                name: "斯俄村民委员会",
                code: "025",
            },
            VillageCode {
                name: "贡曲村民委员会",
                code: "026",
            },
            VillageCode {
                name: "日安村民委员会",
                code: "027",
            },
            VillageCode {
                name: "霍古都村民委员会",
                code: "028",
            },
            VillageCode {
                name: "也哈村民委员会",
                code: "029",
            },
            VillageCode {
                name: "也伦达村民委员会",
                code: "030",
            },
            VillageCode {
                name: "吉绒达村民委员会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "查龙镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "吉且一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "吉且二村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "查龙一村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "查龙二村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "纳卡村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "来马镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "迷鲁村民委员会",
                code: "001",
            },
            VillageCode {
                name: "鲁须村民委员会",
                code: "002",
            },
            VillageCode {
                name: "叶早村民委员会",
                code: "003",
            },
            VillageCode {
                name: "纳洼村民委员会",
                code: "004",
            },
            VillageCode {
                name: "里拉村民委员会",
                code: "005",
            },
            VillageCode {
                name: "觉日村民委员会",
                code: "006",
            },
            VillageCode {
                name: "夺日村民委员会",
                code: "007",
            },
            VillageCode {
                name: "来马村民委员会",
                code: "008",
            },
            VillageCode {
                name: "雅子村民委员会",
                code: "009",
            },
            VillageCode {
                name: "马达村民委员会",
                code: "010",
            },
            VillageCode {
                name: "地格村民委员会",
                code: "011",
            },
            VillageCode {
                name: "冷达村民委员会",
                code: "012",
            },
            VillageCode {
                name: "康朱村民委员会",
                code: "013",
            },
            VillageCode {
                name: "格通村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "呷拉乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "笨得古村民委员会",
                code: "001",
            },
            VillageCode {
                name: "亚龙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "柯多村民委员会",
                code: "003",
            },
            VillageCode {
                name: "夺拖村民委员会",
                code: "004",
            },
            VillageCode {
                name: "自公底村民委员会",
                code: "005",
            },
            VillageCode {
                name: "呷拉村民委员会",
                code: "006",
            },
            VillageCode {
                name: "阿日然村民委员会",
                code: "007",
            },
            VillageCode {
                name: "仲卡村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "色西底乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "木西村民委员会",
                code: "001",
            },
            VillageCode {
                name: "西龙卡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "亚卡巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "德然亚书村民委员会",
                code: "004",
            },
            VillageCode {
                name: "德然麻书村民委员会",
                code: "005",
            },
            VillageCode {
                name: "珠巴村民委员会",
                code: "006",
            },
            VillageCode {
                name: "尼隆村民委员会",
                code: "007",
            },
            VillageCode {
                name: "则打村民委员会",
                code: "008",
            },
            VillageCode {
                name: "曲卡龙村民委员会",
                code: "009",
            },
            VillageCode {
                name: "甲衣村民委员会",
                code: "010",
            },
            VillageCode {
                name: "色西村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "南多乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "南多村民委员会",
                code: "001",
            },
            VillageCode {
                name: "曲卡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "然呷村民委员会",
                code: "003",
            },
            VillageCode {
                name: "俄绒村民委员会",
                code: "004",
            },
            VillageCode {
                name: "则曲村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "生康乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "然达底村民委员会",
                code: "001",
            },
            VillageCode {
                name: "学巴底村民委员会",
                code: "002",
            },
            VillageCode {
                name: "巴学底村民委员会",
                code: "003",
            },
            VillageCode {
                name: "白日村民委员会",
                code: "004",
            },
            VillageCode {
                name: "仲若村民委员会",
                code: "005",
            },
            VillageCode {
                name: "德西顶村民委员会",
                code: "006",
            },
            VillageCode {
                name: "达瓦贡村民委员会",
                code: "007",
            },
            VillageCode {
                name: "门洛村民委员会",
                code: "008",
            },
            VillageCode {
                name: "丹果村民委员会",
                code: "009",
            },
            VillageCode {
                name: "巴瓦村民委员会",
                code: "010",
            },
            VillageCode {
                name: "仲柯村民委员会",
                code: "011",
            },
            VillageCode {
                name: "莫穷村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "贡隆乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "堆温果村民委员会",
                code: "001",
            },
            VillageCode {
                name: "莫绒隆村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西启卡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "达合村民委员会",
                code: "004",
            },
            VillageCode {
                name: "学仁多村民委员会",
                code: "005",
            },
            VillageCode {
                name: "夏拉卡村民委员会",
                code: "006",
            },
            VillageCode {
                name: "麦卡村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "扎科乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "青尼村民委员会",
                code: "001",
            },
            VillageCode {
                name: "生达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "海拉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "银达村民委员会",
                code: "004",
            },
            VillageCode {
                name: "协巴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "查衣村民委员会",
                code: "006",
            },
            VillageCode {
                name: "查多村民委员会",
                code: "007",
            },
            VillageCode {
                name: "地龙村民委员会",
                code: "008",
            },
            VillageCode {
                name: "仲麦村民委员会",
                code: "009",
            },
            VillageCode {
                name: "安达村民委员会",
                code: "010",
            },
            VillageCode {
                name: "大巴卡村民委员会",
                code: "011",
            },
            VillageCode {
                name: "麦玉隆村民委员会",
                code: "012",
            },
            VillageCode {
                name: "协旦达村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "昔色乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "阿然隆村民委员会",
                code: "001",
            },
            VillageCode {
                name: "西松隆村民委员会",
                code: "002",
            },
            VillageCode {
                name: "洛夏村民委员会",
                code: "003",
            },
            VillageCode {
                name: "仁达村民委员会",
                code: "004",
            },
            VillageCode {
                name: "格夏村民委员会",
                code: "005",
            },
            VillageCode {
                name: "青沙村民委员会",
                code: "006",
            },
            VillageCode {
                name: "亚龙村民委员会",
                code: "007",
            },
            VillageCode {
                name: "色别村民委员会",
                code: "008",
            },
            VillageCode {
                name: "哈旦达村民委员会",
                code: "009",
            },
            VillageCode {
                name: "下村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "上村村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "卡攻乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "仲穷村民委员会",
                code: "001",
            },
            VillageCode {
                name: "浪子村民委员会",
                code: "002",
            },
            VillageCode {
                name: "安西村民委员会",
                code: "003",
            },
            VillageCode {
                name: "庄果村民委员会",
                code: "004",
            },
            VillageCode {
                name: "亚书村民委员会",
                code: "005",
            },
            VillageCode {
                name: "麻书村民委员会",
                code: "006",
            },
            VillageCode {
                name: "岔拉村民委员会",
                code: "007",
            },
            VillageCode {
                name: "莫衣村民委员会",
                code: "008",
            },
            VillageCode {
                name: "格沙村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "仁果乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "俄多村民委员会",
                code: "001",
            },
            VillageCode {
                name: "吾中村民委员会",
                code: "002",
            },
            VillageCode {
                name: "拉尼村民委员会",
                code: "003",
            },
            VillageCode {
                name: "洛拉村民委员会",
                code: "004",
            },
            VillageCode {
                name: "桑都村民委员会",
                code: "005",
            },
            VillageCode {
                name: "仁果上村民委员会",
                code: "006",
            },
            VillageCode {
                name: "仁果下村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "拖坝乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "拖坝村民委员会",
                code: "001",
            },
            VillageCode {
                name: "楚洛村民委员会",
                code: "002",
            },
            VillageCode {
                name: "卡呷村民委员会",
                code: "003",
            },
            VillageCode {
                name: "扎来村民委员会",
                code: "004",
            },
            VillageCode {
                name: "普衣隆村民委员会",
                code: "005",
            },
            VillageCode {
                name: "移民扶贫新村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "竹溪村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "庭卡乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "庭卡村民委员会",
                code: "001",
            },
            VillageCode {
                name: "苦绒村民委员会",
                code: "002",
            },
            VillageCode {
                name: "洛戈村民委员会",
                code: "003",
            },
            VillageCode {
                name: "拉西村民委员会",
                code: "004",
            },
            VillageCode {
                name: "斯兰达村民委员会",
                code: "005",
            },
            VillageCode {
                name: "巴俄底村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "下雄乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "德且一队村民委员会",
                code: "001",
            },
            VillageCode {
                name: "德且二队村民委员会",
                code: "002",
            },
            VillageCode {
                name: "德西村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下雄村民委员会",
                code: "004",
            },
            VillageCode {
                name: "则色村民委员会",
                code: "005",
            },
            VillageCode {
                name: "打本一队村民委员会",
                code: "006",
            },
            VillageCode {
                name: "打本二队村民委员会",
                code: "007",
            },
            VillageCode {
                name: "阿德村民委员会",
                code: "008",
            },
            VillageCode {
                name: "然洛村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "四通达乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "四通达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "尼赤村民委员会",
                code: "002",
            },
            VillageCode {
                name: "支拉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "日都村民委员会",
                code: "004",
            },
            VillageCode {
                name: "卡苏村民委员会",
                code: "005",
            },
            VillageCode {
                name: "棒多村民委员会",
                code: "006",
            },
            VillageCode {
                name: "瓦拉达村民委员会",
                code: "007",
            },
            VillageCode {
                name: "则衣村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "夺多乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "夺多村民委员会",
                code: "001",
            },
            VillageCode {
                name: "汪达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "果木村民委员会",
                code: "003",
            },
            VillageCode {
                name: "牛日村民委员会",
                code: "004",
            },
            VillageCode {
                name: "拉多村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "泥柯乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "仁达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "彭达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "布日邓措村民委员会",
                code: "003",
            },
            VillageCode {
                name: "夺衣村民委员会",
                code: "004",
            },
            VillageCode {
                name: "昌谷村民委员会",
                code: "005",
            },
            VillageCode {
                name: "和谐村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "茶扎乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "木通一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "木通二村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "戈柯村民委员会",
                code: "003",
            },
            VillageCode {
                name: "银多村民委员会",
                code: "004",
            },
            VillageCode {
                name: "色须塘村民委员会",
                code: "005",
            },
            VillageCode {
                name: "夺呷村民委员会",
                code: "006",
            },
            VillageCode {
                name: "雅绒村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "大德乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "阿加一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "阿加二村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "曲隆村民委员会",
                code: "003",
            },
            VillageCode {
                name: "共玛村民委员会",
                code: "004",
            },
            VillageCode {
                name: "章隆村民委员会",
                code: "005",
            },
            VillageCode {
                name: "土霍村民委员会",
                code: "006",
            },
            VillageCode {
                name: "甲绒村民委员会",
                code: "007",
            },
            VillageCode {
                name: "拉绒村民委员会",
                code: "008",
            },
            VillageCode {
                name: "打隆村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "卡龙乡",
        code: "021",
        villages: &[
            VillageCode {
                name: "阿沙一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "阿沙二村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "夺普村民委员会",
                code: "003",
            },
            VillageCode {
                name: "卡龙村民委员会",
                code: "004",
            },
            VillageCode {
                name: "夺绒塘一村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "夺绒塘二村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "哈西一村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "哈西二村村民委员会",
                code: "008",
            },
        ],
    },
];

static TOWNS_XK_026: [TownCode; 16] = [
    TownCode {
        name: "如龙镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "城区村民委员会",
                code: "002",
            },
            VillageCode {
                name: "东格村民委员会",
                code: "003",
            },
            VillageCode {
                name: "吴地村民委员会",
                code: "004",
            },
            VillageCode {
                name: "高山村民委员会",
                code: "005",
            },
            VillageCode {
                name: "俄日村民委员会",
                code: "006",
            },
            VillageCode {
                name: "卡鲁村民委员会",
                code: "007",
            },
            VillageCode {
                name: "益西村民委员会",
                code: "008",
            },
            VillageCode {
                name: "故西村民委员会",
                code: "009",
            },
            VillageCode {
                name: "达日村民委员会",
                code: "010",
            },
            VillageCode {
                name: "土古村民委员会",
                code: "011",
            },
            VillageCode {
                name: "仁古村民委员会",
                code: "012",
            },
            VillageCode {
                name: "曲格村民委员会",
                code: "013",
            },
            VillageCode {
                name: "阿呷村民委员会",
                code: "014",
            },
            VillageCode {
                name: "银龙村民委员会",
                code: "015",
            },
            VillageCode {
                name: "依鲁村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "拉日马镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "扎宗村民委员会",
                code: "001",
            },
            VillageCode {
                name: "色戈村民委员会",
                code: "002",
            },
            VillageCode {
                name: "泽龙多村民委员会",
                code: "003",
            },
            VillageCode {
                name: "松多顶村民委员会",
                code: "004",
            },
            VillageCode {
                name: "拉麦村民委员会",
                code: "005",
            },
            VillageCode {
                name: "康多村民委员会",
                code: "006",
            },
            VillageCode {
                name: "更卡村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "大盖镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "大盖村民委员会",
                code: "001",
            },
            VillageCode {
                name: "赤措村民委员会",
                code: "002",
            },
            VillageCode {
                name: "木鲁村民委员会",
                code: "003",
            },
            VillageCode {
                name: "麦柯村民委员会",
                code: "004",
            },
            VillageCode {
                name: "汤科村民委员会",
                code: "005",
            },
            VillageCode {
                name: "阿吉村民委员会",
                code: "006",
            },
            VillageCode {
                name: "竹青村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "通宵镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "察亚所村民委员会",
                code: "001",
            },
            VillageCode {
                name: "察麻所村民委员会",
                code: "002",
            },
            VillageCode {
                name: "洛鲁村民委员会",
                code: "003",
            },
            VillageCode {
                name: "足然村民委员会",
                code: "004",
            },
            VillageCode {
                name: "呷德村民委员会",
                code: "005",
            },
            VillageCode {
                name: "塔布村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "色威镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "色威村民委员会",
                code: "001",
            },
            VillageCode {
                name: "寺庙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "俄色村民委员会",
                code: "003",
            },
            VillageCode {
                name: "泽西村民委员会",
                code: "004",
            },
            VillageCode {
                name: "益麦村民委员会",
                code: "005",
            },
            VillageCode {
                name: "克日多村民委员会",
                code: "006",
            },
            VillageCode {
                name: "谷日村民委员会",
                code: "007",
            },
            VillageCode {
                name: "尼托村民委员会",
                code: "008",
            },
            VillageCode {
                name: "桑郎村民委员会",
                code: "009",
            },
            VillageCode {
                name: "切依村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "尤拉西镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "尤拉西村民委员会",
                code: "001",
            },
            VillageCode {
                name: "洛足村民委员会",
                code: "002",
            },
            VillageCode {
                name: "觉然村民委员会",
                code: "003",
            },
            VillageCode {
                name: "忙布村民委员会",
                code: "004",
            },
            VillageCode {
                name: "多则村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "沙堆乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "然真村民委员会",
                code: "001",
            },
            VillageCode {
                name: "各中村民委员会",
                code: "002",
            },
            VillageCode {
                name: "科查村民委员会",
                code: "003",
            },
            VillageCode {
                name: "女汝村民委员会",
                code: "004",
            },
            VillageCode {
                name: "觉里村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "绕鲁乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "壮巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "茶下村民委员会",
                code: "002",
            },
            VillageCode {
                name: "基洛村民委员会",
                code: "003",
            },
            VillageCode {
                name: "绕鲁村民委员会",
                code: "004",
            },
            VillageCode {
                name: "相堆村民委员会",
                code: "005",
            },
            VillageCode {
                name: "学麦村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "博美乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "拉巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "德麦巴村民委员会",
                code: "002",
            },
            VillageCode {
                name: "博美村民委员会",
                code: "003",
            },
            VillageCode {
                name: "仁乃村民委员会",
                code: "004",
            },
            VillageCode {
                name: "波洛村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "子拖西乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "呷戈村民委员会",
                code: "001",
            },
            VillageCode {
                name: "所差村民委员会",
                code: "002",
            },
            VillageCode {
                name: "当巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "郎村村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "和平乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "竹青村民委员会",
                code: "001",
            },
            VillageCode {
                name: "麻西村民委员会",
                code: "002",
            },
            VillageCode {
                name: "甲西村民委员会",
                code: "003",
            },
            VillageCode {
                name: "日巴村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "洛古乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "东风村民委员会",
                code: "001",
            },
            VillageCode {
                name: "亚所村民委员会",
                code: "002",
            },
            VillageCode {
                name: "日古村民委员会",
                code: "003",
            },
            VillageCode {
                name: "泽科村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "雄龙西乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "提巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "古鲁村民委员会",
                code: "002",
            },
            VillageCode {
                name: "哈米村民委员会",
                code: "003",
            },
            VillageCode {
                name: "腰古村民委员会",
                code: "004",
            },
            VillageCode {
                name: "卡鲁村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "麻日乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "麦坝村民委员会",
                code: "001",
            },
            VillageCode {
                name: "德坝村民委员会",
                code: "002",
            },
            VillageCode {
                name: "南多村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "友谊乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "古鲁村民委员会",
                code: "001",
            },
            VillageCode {
                name: "皮察村民委员会",
                code: "002",
            },
            VillageCode {
                name: "措日村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "银多乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "阿色三村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "阿色一村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阿色二村村民委员会",
                code: "003",
            },
        ],
    },
];

static TOWNS_XK_027: [TownCode; 23] = [
    TownCode {
        name: "更庆镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "一居民委员会",
                code: "001",
            },
            VillageCode {
                name: "二居民委员会",
                code: "002",
            },
            VillageCode {
                name: "茶马街居民委员会",
                code: "003",
            },
            VillageCode {
                name: "西布村民委员会",
                code: "004",
            },
            VillageCode {
                name: "八美村民委员会",
                code: "005",
            },
            VillageCode {
                name: "尼木村民委员会",
                code: "006",
            },
            VillageCode {
                name: "五一桥村民委员会",
                code: "007",
            },
            VillageCode {
                name: "郎达村民委员会",
                code: "008",
            },
            VillageCode {
                name: "莫学村民委员会",
                code: "009",
            },
            VillageCode {
                name: "戈姑村民委员会",
                code: "010",
            },
            VillageCode {
                name: "欧普村民委员会",
                code: "011",
            },
            VillageCode {
                name: "班达村民委员会",
                code: "012",
            },
            VillageCode {
                name: "下压坝村民委员会",
                code: "013",
            },
            VillageCode {
                name: "压巴村民委员会",
                code: "014",
            },
            VillageCode {
                name: "杨西村民委员会",
                code: "015",
            },
            VillageCode {
                name: "呷中村民委员会",
                code: "016",
            },
            VillageCode {
                name: "普热村民委员会",
                code: "017",
            },
            VillageCode {
                name: "拉普村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "马尼干戈镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "马尼村民委员会",
                code: "001",
            },
            VillageCode {
                name: "措巴村民委员会",
                code: "002",
            },
            VillageCode {
                name: "洞真村民委员会",
                code: "003",
            },
            VillageCode {
                name: "雪科村民委员会",
                code: "004",
            },
            VillageCode {
                name: "拉绒村民委员会",
                code: "005",
            },
            VillageCode {
                name: "曲西村民委员会",
                code: "006",
            },
            VillageCode {
                name: "格公村民委员会",
                code: "007",
            },
            VillageCode {
                name: "达戈村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "竹庆镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "竹庆村民委员会",
                code: "001",
            },
            VillageCode {
                name: "扎东村民委员会",
                code: "002",
            },
            VillageCode {
                name: "更达村民委员会",
                code: "003",
            },
            VillageCode {
                name: "拉加村民委员会",
                code: "004",
            },
            VillageCode {
                name: "八色村民委员会",
                code: "005",
            },
            VillageCode {
                name: "档木村民委员会",
                code: "006",
            },
            VillageCode {
                name: "协庆村民委员会",
                code: "007",
            },
            VillageCode {
                name: "拥青村民委员会",
                code: "008",
            },
            VillageCode {
                name: "更龙村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "阿须镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "磨勒村民委员会",
                code: "001",
            },
            VillageCode {
                name: "浪隆村民委员会",
                code: "002",
            },
            VillageCode {
                name: "麦青村民委员会",
                code: "003",
            },
            VillageCode {
                name: "龙真村民委员会",
                code: "004",
            },
            VillageCode {
                name: "真隆村民委员会",
                code: "005",
            },
            VillageCode {
                name: "让尼村民委员会",
                code: "006",
            },
            VillageCode {
                name: "模通村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "错阿镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "错通村民委员会",
                code: "001",
            },
            VillageCode {
                name: "绒岔村民委员会",
                code: "002",
            },
            VillageCode {
                name: "马达村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "麦宿镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "绒麦隆村民委员会",
                code: "001",
            },
            VillageCode {
                name: "荒达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "木岳村民委员会",
                code: "003",
            },
            VillageCode {
                name: "美丽村民委员会",
                code: "004",
            },
            VillageCode {
                name: "日卡村民委员会",
                code: "005",
            },
            VillageCode {
                name: "贡空村民委员会",
                code: "006",
            },
            VillageCode {
                name: "绒达村民委员会",
                code: "007",
            },
            VillageCode {
                name: "新地村民委员会",
                code: "008",
            },
            VillageCode {
                name: "卡沙村民委员会",
                code: "009",
            },
            VillageCode {
                name: "马东村民委员会",
                code: "010",
            },
            VillageCode {
                name: "真通村民委员会",
                code: "011",
            },
            VillageCode {
                name: "基足村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "打滚镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "尼穷村民委员会",
                code: "001",
            },
            VillageCode {
                name: "然尼村民委员会",
                code: "002",
            },
            VillageCode {
                name: "康秋村民委员会",
                code: "003",
            },
            VillageCode {
                name: "呷拖村民委员会",
                code: "004",
            },
            VillageCode {
                name: "芒布村民委员会",
                code: "005",
            },
            VillageCode {
                name: "俄柯村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "龚垭镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "龚垭村民委员会",
                code: "001",
            },
            VillageCode {
                name: "普西村民委员会",
                code: "002",
            },
            VillageCode {
                name: "更达村民委员会",
                code: "003",
            },
            VillageCode {
                name: "格苏村民委员会",
                code: "004",
            },
            VillageCode {
                name: "康公村民委员会",
                code: "005",
            },
            VillageCode {
                name: "雨托村民委员会",
                code: "006",
            },
            VillageCode {
                name: "血呷村民委员会",
                code: "007",
            },
            VillageCode {
                name: "折学村民委员会",
                code: "008",
            },
            VillageCode {
                name: "洞庄村民委员会",
                code: "009",
            },
            VillageCode {
                name: "秧达村民委员会",
                code: "010",
            },
            VillageCode {
                name: "来格村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "温拖镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "康郎村民委员会",
                code: "001",
            },
            VillageCode {
                name: "地茶村民委员会",
                code: "002",
            },
            VillageCode {
                name: "仁青里村民委员会",
                code: "003",
            },
            VillageCode {
                name: "温拖村民委员会",
                code: "004",
            },
            VillageCode {
                name: "满金村民委员会",
                code: "005",
            },
            VillageCode {
                name: "党批村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "中扎科镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "曲公村民委员会",
                code: "001",
            },
            VillageCode {
                name: "窝坝村民委员会",
                code: "002",
            },
            VillageCode {
                name: "扎多村民委员会",
                code: "003",
            },
            VillageCode {
                name: "多达村民委员会",
                code: "004",
            },
            VillageCode {
                name: "同鸠村民委员会",
                code: "005",
            },
            VillageCode {
                name: "月拉村民委员会",
                code: "006",
            },
            VillageCode {
                name: "瓦通村民委员会",
                code: "007",
            },
            VillageCode {
                name: "呷依村民委员会",
                code: "008",
            },
            VillageCode {
                name: "雄拖村民委员会",
                code: "009",
            },
            VillageCode {
                name: "上卡村民委员会",
                code: "010",
            },
            VillageCode {
                name: "地龙村民委员会",
                code: "011",
            },
            VillageCode {
                name: "村科村民委员会",
                code: "012",
            },
            VillageCode {
                name: "芒科村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "岳巴乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "阿木拉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "日炯村民委员会",
                code: "002",
            },
            VillageCode {
                name: "理公村民委员会",
                code: "003",
            },
            VillageCode {
                name: "岳巴寨村民委员会",
                code: "004",
            },
            VillageCode {
                name: "底绒村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "八帮乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "曲池西村民委员会",
                code: "001",
            },
            VillageCode {
                name: "然青龙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "梅林村民委员会",
                code: "003",
            },
            VillageCode {
                name: "上八坞村民委员会",
                code: "004",
            },
            VillageCode {
                name: "泽池村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下八坞村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上白卡村民委员会",
                code: "007",
            },
            VillageCode {
                name: "下白卡村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "白垭乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "冷茶村民委员会",
                code: "001",
            },
            VillageCode {
                name: "林学村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阿池村民委员会",
                code: "003",
            },
            VillageCode {
                name: "尼朱村民委员会",
                code: "004",
            },
            VillageCode {
                name: "日火村民委员会",
                code: "005",
            },
            VillageCode {
                name: "窝色村民委员会",
                code: "006",
            },
            VillageCode {
                name: "茶安村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "汪布顶乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "西巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "亚且村民委员会",
                code: "002",
            },
            VillageCode {
                name: "龚加村民委员会",
                code: "003",
            },
            VillageCode {
                name: "汪布顶村民委员会",
                code: "004",
            },
            VillageCode {
                name: "扎西旦村民委员会",
                code: "005",
            },
            VillageCode {
                name: "各麦村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "柯洛洞乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "色巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "独木岭村民委员会",
                code: "002",
            },
            VillageCode {
                name: "夺色达村民委员会",
                code: "003",
            },
            VillageCode {
                name: "郎达村民委员会",
                code: "004",
            },
            VillageCode {
                name: "措普村民委员会",
                code: "005",
            },
            VillageCode {
                name: "牛麦村民委员会",
                code: "006",
            },
            VillageCode {
                name: "燃卡村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "卡松渡乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "银多村民委员会",
                code: "001",
            },
            VillageCode {
                name: "然卡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "扎拉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "卡莫村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "俄南乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "俄南村民委员会",
                code: "001",
            },
            VillageCode {
                name: "真达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "绒加村民委员会",
                code: "003",
            },
            VillageCode {
                name: "马龙村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "俄支乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "洞中达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "绒娘村民委员会",
                code: "002",
            },
            VillageCode {
                name: "俄支村民委员会",
                code: "003",
            },
            VillageCode {
                name: "烟达村民委员会",
                code: "004",
            },
            VillageCode {
                name: "热水塘村民委员会",
                code: "005",
            },
            VillageCode {
                name: "安戈玛村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "玉隆乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "白日一村民委员会",
                code: "001",
            },
            VillageCode {
                name: "白日二村民委员会",
                code: "002",
            },
            VillageCode {
                name: "火然村民委员会",
                code: "003",
            },
            VillageCode {
                name: "绒青村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "上燃姑乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "夺巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "麻邛村民委员会",
                code: "002",
            },
            VillageCode {
                name: "亚西村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "年古乡",
        code: "021",
        villages: &[
            VillageCode {
                name: "娃巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "门达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "年古村民委员会",
                code: "003",
            },
            VillageCode {
                name: "同古村民委员会",
                code: "004",
            },
            VillageCode {
                name: "若达村民委员会",
                code: "005",
            },
            VillageCode {
                name: "真达村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "浪多乡",
        code: "022",
        villages: &[
            VillageCode {
                name: "生巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "能麦村民委员会",
                code: "002",
            },
            VillageCode {
                name: "志巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "洞达村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "亚丁乡",
        code: "023",
        villages: &[
            VillageCode {
                name: "吉绒村民委员会",
                code: "001",
            },
            VillageCode {
                name: "根达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "吉黑村民委员会",
                code: "003",
            },
            VillageCode {
                name: "岸日村民委员会",
                code: "004",
            },
            VillageCode {
                name: "扎青村民委员会",
                code: "005",
            },
            VillageCode {
                name: "呷曲通村民委员会",
                code: "006",
            },
            VillageCode {
                name: "扎多村民委员会",
                code: "007",
            },
            VillageCode {
                name: "吉龙村民委员会",
                code: "008",
            },
        ],
    },
];

static TOWNS_XK_028: [TownCode; 16] = [
    TownCode {
        name: "建设镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "河东社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "河西社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "亚通村民委员会",
                code: "003",
            },
            VillageCode {
                name: "日西村民委员会",
                code: "004",
            },
            VillageCode {
                name: "扎盘村民委员会",
                code: "005",
            },
            VillageCode {
                name: "播麦村民委员会",
                code: "006",
            },
            VillageCode {
                name: "达科村民委员会",
                code: "007",
            },
            VillageCode {
                name: "呷巴村民委员会",
                code: "008",
            },
            VillageCode {
                name: "麻通村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "阿察镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "阿察村民委员会",
                code: "001",
            },
            VillageCode {
                name: "查科村民委员会",
                code: "002",
            },
            VillageCode {
                name: "昌拖村民委员会",
                code: "003",
            },
            VillageCode {
                name: "班充村民委员会",
                code: "004",
            },
            VillageCode {
                name: "亚青寺管理局社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "河坡镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "根呷村民委员会",
                code: "001",
            },
            VillageCode {
                name: "先锋村民委员会",
                code: "002",
            },
            VillageCode {
                name: "麦学村民委员会",
                code: "003",
            },
            VillageCode {
                name: "德来村民委员会",
                code: "004",
            },
            VillageCode {
                name: "格学村民委员会",
                code: "005",
            },
            VillageCode {
                name: "埃西村民委员会",
                code: "006",
            },
            VillageCode {
                name: "普马村民委员会",
                code: "007",
            },
            VillageCode {
                name: "小吾村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下达村民委员会",
                code: "009",
            },
            VillageCode {
                name: "定欧村民委员会",
                code: "010",
            },
            VillageCode {
                name: "则吾村民委员会",
                code: "011",
            },
            VillageCode {
                name: "仁白村民委员会",
                code: "012",
            },
            VillageCode {
                name: "麦达村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "盖玉镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "德来村民委员会",
                code: "001",
            },
            VillageCode {
                name: "洛格村民委员会",
                code: "002",
            },
            VillageCode {
                name: "沙拖村民委员会",
                code: "003",
            },
            VillageCode {
                name: "打乙西村民委员会",
                code: "004",
            },
            VillageCode {
                name: "亚达村民委员会",
                code: "005",
            },
            VillageCode {
                name: "洞中村民委员会",
                code: "006",
            },
            VillageCode {
                name: "德沙孔村民委员会",
                code: "007",
            },
            VillageCode {
                name: "协巴村民委员会",
                code: "008",
            },
            VillageCode {
                name: "郎帮村民委员会",
                code: "009",
            },
            VillageCode {
                name: "火龙村民委员会",
                code: "010",
            },
            VillageCode {
                name: "苏日村民委员会",
                code: "011",
            },
            VillageCode {
                name: "山岩村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "金沙乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "沙丁村民委员会",
                code: "001",
            },
            VillageCode {
                name: "伍仲村民委员会",
                code: "002",
            },
            VillageCode {
                name: "多来村民委员会",
                code: "003",
            },
            VillageCode {
                name: "哈巴村民委员会",
                code: "004",
            },
            VillageCode {
                name: "作英村民委员会",
                code: "005",
            },
            VillageCode {
                name: "播欧村民委员会",
                code: "006",
            },
            VillageCode {
                name: "亚力西村民委员会",
                code: "007",
            },
            VillageCode {
                name: "仁宗村民委员会",
                code: "008",
            },
            VillageCode {
                name: "阿作村民委员会",
                code: "009",
            },
            VillageCode {
                name: "吉松村民委员会",
                code: "010",
            },
            VillageCode {
                name: "喀耶村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "绒盖乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "绒盖村民委员会",
                code: "001",
            },
            VillageCode {
                name: "协塘村民委员会",
                code: "002",
            },
            VillageCode {
                name: "优巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "俄巴村民委员会",
                code: "004",
            },
            VillageCode {
                name: "协达村民委员会",
                code: "005",
            },
            VillageCode {
                name: "沟沟村民委员会",
                code: "006",
            },
            VillageCode {
                name: "俄它村民委员会",
                code: "007",
            },
            VillageCode {
                name: "仲边村民委员会",
                code: "008",
            },
            VillageCode {
                name: "则生村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "章都乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "马拉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "金龙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "玉桑村民委员会",
                code: "003",
            },
            VillageCode {
                name: "戈德村民委员会",
                code: "004",
            },
            VillageCode {
                name: "查卡村民委员会",
                code: "005",
            },
            VillageCode {
                name: "东拖村民委员会",
                code: "006",
            },
            VillageCode {
                name: "阿色村民委员会",
                code: "007",
            },
            VillageCode {
                name: "边坝村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "麻绒乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "麻绒村民委员会",
                code: "001",
            },
            VillageCode {
                name: "德来村民委员会",
                code: "002",
            },
            VillageCode {
                name: "如当村民委员会",
                code: "003",
            },
            VillageCode {
                name: "格塔村民委员会",
                code: "004",
            },
            VillageCode {
                name: "协加村民委员会",
                code: "005",
            },
            VillageCode {
                name: "然本村民委员会",
                code: "006",
            },
            VillageCode {
                name: "马门村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "热加乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "藏东村民委员会",
                code: "001",
            },
            VillageCode {
                name: "帕美村民委员会",
                code: "002",
            },
            VillageCode {
                name: "孜巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "拉龙村民委员会",
                code: "004",
            },
            VillageCode {
                name: "当巴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "盖公村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上牛沙村民委员会",
                code: "007",
            },
            VillageCode {
                name: "下牛沙村民委员会",
                code: "008",
            },
            VillageCode {
                name: "卡龙村民委员会",
                code: "009",
            },
            VillageCode {
                name: "然章村民委员会",
                code: "010",
            },
            VillageCode {
                name: "也康村民委员会",
                code: "011",
            },
            VillageCode {
                name: "也良村民委员会",
                code: "012",
            },
            VillageCode {
                name: "学多村民委员会",
                code: "013",
            },
            VillageCode {
                name: "拉巴村民委员会",
                code: "014",
            },
            VillageCode {
                name: "沙坝村民委员会",
                code: "015",
            },
            VillageCode {
                name: "勒沙村民委员会",
                code: "016",
            },
            VillageCode {
                name: "勒吉村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "登龙乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "伍沙村民委员会",
                code: "001",
            },
            VillageCode {
                name: "洞托村民委员会",
                code: "002",
            },
            VillageCode {
                name: "定戈村民委员会",
                code: "003",
            },
            VillageCode {
                name: "康通村民委员会",
                code: "004",
            },
            VillageCode {
                name: "龚巴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "诺宗村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "赠科乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "扎马村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上巴卡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "格沙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "安卡村民委员会",
                code: "004",
            },
            VillageCode {
                name: "则达村民委员会",
                code: "005",
            },
            VillageCode {
                name: "上比沙村民委员会",
                code: "006",
            },
            VillageCode {
                name: "八垭村民委员会",
                code: "007",
            },
            VillageCode {
                name: "下比沙村民委员会",
                code: "008",
            },
            VillageCode {
                name: "热卡村民委员会",
                code: "009",
            },
            VillageCode {
                name: "尼龙村民委员会",
                code: "010",
            },
            VillageCode {
                name: "岳达村民委员会",
                code: "011",
            },
            VillageCode {
                name: "洛巴村民委员会",
                code: "012",
            },
            VillageCode {
                name: "依里村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "麻邛乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "麻邛村民委员会",
                code: "001",
            },
            VillageCode {
                name: "甲旭村民委员会",
                code: "002",
            },
            VillageCode {
                name: "安章村民委员会",
                code: "003",
            },
            VillageCode {
                name: "根日村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "辽西乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "辽西村民委员会",
                code: "001",
            },
            VillageCode {
                name: "昌麦村民委员会",
                code: "002",
            },
            VillageCode {
                name: "达科村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "纳塔乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "纳塔村民委员会",
                code: "001",
            },
            VillageCode {
                name: "卡塔村民委员会",
                code: "002",
            },
            VillageCode {
                name: "纳邛村民委员会",
                code: "003",
            },
            VillageCode {
                name: "金都村民委员会",
                code: "004",
            },
            VillageCode {
                name: "昌根村民委员会",
                code: "005",
            },
            VillageCode {
                name: "措拉村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "安孜乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "门马二村民委员会",
                code: "001",
            },
            VillageCode {
                name: "如须村民委员会",
                code: "002",
            },
            VillageCode {
                name: "门马一村民委员会",
                code: "003",
            },
            VillageCode {
                name: "麻绒村民委员会",
                code: "004",
            },
            VillageCode {
                name: "昌戈村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "沙马乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "德托村民委员会",
                code: "001",
            },
            VillageCode {
                name: "布格村民委员会",
                code: "002",
            },
            VillageCode {
                name: "门呷村民委员会",
                code: "003",
            },
            VillageCode {
                name: "瓦岗村民委员会",
                code: "004",
            },
            VillageCode {
                name: "德西村民委员会",
                code: "005",
            },
        ],
    },
];

static TOWNS_XK_029: [TownCode; 21] = [
    TownCode {
        name: "尼呷镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "一社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "二社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "沙马村民委员会",
                code: "003",
            },
            VillageCode {
                name: "阿弟村民委员会",
                code: "004",
            },
            VillageCode {
                name: "低龙村民委员会",
                code: "005",
            },
            VillageCode {
                name: "古恩村民委员会",
                code: "006",
            },
            VillageCode {
                name: "沙岔村民委员会",
                code: "007",
            },
            VillageCode {
                name: "菊母村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "洛须镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "洛须镇社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "温拖村民委员会",
                code: "002",
            },
            VillageCode {
                name: "拉空龙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "洛须龙村民委员会",
                code: "004",
            },
            VillageCode {
                name: "更沙村民委员会",
                code: "005",
            },
            VillageCode {
                name: "纳扎村民委员会",
                code: "006",
            },
            VillageCode {
                name: "俄巴纳村民委员会",
                code: "007",
            },
            VillageCode {
                name: "龙溪卡村民委员会",
                code: "008",
            },
            VillageCode {
                name: "丹达村民委员会",
                code: "009",
            },
            VillageCode {
                name: "格巴村民委员会",
                code: "010",
            },
            VillageCode {
                name: "仲沙村民委员会",
                code: "011",
            },
            VillageCode {
                name: "卡洛村民委员会",
                code: "012",
            },
            VillageCode {
                name: "上岭村民委员会",
                code: "013",
            },
            VillageCode {
                name: "呷坡村民委员会",
                code: "014",
            },
            VillageCode {
                name: "上普马村民委员会",
                code: "015",
            },
            VillageCode {
                name: "昌拖村民委员会",
                code: "016",
            },
            VillageCode {
                name: "麻呷村民委员会",
                code: "017",
            },
            VillageCode {
                name: "然日村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "色须镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "瓦土村民委员会",
                code: "002",
            },
            VillageCode {
                name: "赤贡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "张君村民委员会",
                code: "004",
            },
            VillageCode {
                name: "赤哇村民委员会",
                code: "005",
            },
            VillageCode {
                name: "打龙村民委员会",
                code: "006",
            },
            VillageCode {
                name: "色洞村民委员会",
                code: "007",
            },
            VillageCode {
                name: "修波村民委员会",
                code: "008",
            },
            VillageCode {
                name: "拉龙村民委员会",
                code: "009",
            },
            VillageCode {
                name: "仁绒村民委员会",
                code: "010",
            },
            VillageCode {
                name: "日扎村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "虾扎镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "冻日村民委员会",
                code: "001",
            },
            VillageCode {
                name: "查地村民委员会",
                code: "002",
            },
            VillageCode {
                name: "纽戈村民委员会",
                code: "003",
            },
            VillageCode {
                name: "它须村民委员会",
                code: "004",
            },
            VillageCode {
                name: "阿色村民委员会",
                code: "005",
            },
            VillageCode {
                name: "马东村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "温波镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "错卡村民委员会",
                code: "001",
            },
            VillageCode {
                name: "阿加村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阿沙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "日影村民委员会",
                code: "004",
            },
            VillageCode {
                name: "曲绒村民委员会",
                code: "005",
            },
            VillageCode {
                name: "你虾村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "蒙宜镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "蒙宜村民委员会",
                code: "001",
            },
            VillageCode {
                name: "各龙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "俄马龙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "呷加村民委员会",
                code: "004",
            },
            VillageCode {
                name: "扎虾村民委员会",
                code: "005",
            },
            VillageCode {
                name: "蒙格村民委员会",
                code: "006",
            },
            VillageCode {
                name: "俄蒙村民委员会",
                code: "007",
            },
            VillageCode {
                name: "科龙村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "阿日扎镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "错布村民委员会",
                code: "001",
            },
            VillageCode {
                name: "扎起村民委员会",
                code: "002",
            },
            VillageCode {
                name: "翁布村民委员会",
                code: "003",
            },
            VillageCode {
                name: "各普村民委员会",
                code: "004",
            },
            VillageCode {
                name: "恰冲村民委员会",
                code: "005",
            },
            VillageCode {
                name: "各冲村民委员会",
                code: "006",
            },
            VillageCode {
                name: "马崩村民委员会",
                code: "007",
            },
            VillageCode {
                name: "邦充村民委员会",
                code: "008",
            },
            VillageCode {
                name: "麻曲村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "真达乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "志巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "细绒刀马村民委员会",
                code: "002",
            },
            VillageCode {
                name: "麻达村民委员会",
                code: "003",
            },
            VillageCode {
                name: "紫夫村民委员会",
                code: "004",
            },
            VillageCode {
                name: "真达村民委员会",
                code: "005",
            },
            VillageCode {
                name: "当巴村民委员会",
                code: "006",
            },
            VillageCode {
                name: "甲日村民委员会",
                code: "007",
            },
            VillageCode {
                name: "更思村民委员会",
                code: "008",
            },
            VillageCode {
                name: "洞古村民委员会",
                code: "009",
            },
            VillageCode {
                name: "傲一村民委员会",
                code: "010",
            },
            VillageCode {
                name: "普马村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "奔达乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "昌戈村民委员会",
                code: "001",
            },
            VillageCode {
                name: "满真村民委员会",
                code: "002",
            },
            VillageCode {
                name: "呷巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "奔达村民委员会",
                code: "004",
            },
            VillageCode {
                name: "阴巴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "格绒村民委员会",
                code: "006",
            },
            VillageCode {
                name: "然子村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "正科乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "下拉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "正通村民委员会",
                code: "002",
            },
            VillageCode {
                name: "普巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "甲松村民委员会",
                code: "004",
            },
            VillageCode {
                name: "更生村民委员会",
                code: "005",
            },
            VillageCode {
                name: "娘巴村民委员会",
                code: "006",
            },
            VillageCode {
                name: "生巴村民委员会",
                code: "007",
            },
            VillageCode {
                name: "然足村民委员会",
                code: "008",
            },
            VillageCode {
                name: "曲德村民委员会",
                code: "009",
            },
            VillageCode {
                name: "许巴村民委员会",
                code: "010",
            },
            VillageCode {
                name: "更萨村民委员会",
                code: "011",
            },
            VillageCode {
                name: "正科村民委员会",
                code: "012",
            },
            VillageCode {
                name: "钟青村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "德荣马乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "谷恩村民委员会",
                code: "001",
            },
            VillageCode {
                name: "龙仁村民委员会",
                code: "002",
            },
            VillageCode {
                name: "扎马村民委员会",
                code: "003",
            },
            VillageCode {
                name: "错茶村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "长沙贡马乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "查拉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "查堆村民委员会",
                code: "002",
            },
            VillageCode {
                name: "约瓦村民委员会",
                code: "003",
            },
            VillageCode {
                name: "口司村民委员会",
                code: "004",
            },
            VillageCode {
                name: "色更村民委员会",
                code: "005",
            },
            VillageCode {
                name: "查格村民委员会",
                code: "006",
            },
            VillageCode {
                name: "查麦村民委员会",
                code: "007",
            },
            VillageCode {
                name: "岗青村民委员会",
                code: "008",
            },
            VillageCode {
                name: "绕瓦村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "呷衣乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "呷依村民委员会",
                code: "001",
            },
            VillageCode {
                name: "俄布村民委员会",
                code: "002",
            },
            VillageCode {
                name: "尼达村民委员会",
                code: "003",
            },
            VillageCode {
                name: "八若村民委员会",
                code: "004",
            },
            VillageCode {
                name: "扎绒村民委员会",
                code: "005",
            },
            VillageCode {
                name: "当拉村民委员会",
                code: "006",
            },
            VillageCode {
                name: "俄青村民委员会",
                code: "007",
            },
            VillageCode {
                name: "觉吉村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "格孟乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "格孟村民委员会",
                code: "001",
            },
            VillageCode {
                name: "扎母村民委员会",
                code: "002",
            },
            VillageCode {
                name: "港穷村民委员会",
                code: "003",
            },
            VillageCode {
                name: "格贡村民委员会",
                code: "004",
            },
            VillageCode {
                name: "呷日村民委员会",
                code: "005",
            },
            VillageCode {
                name: "格嘎村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "新荣乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "翁穷村民委员会",
                code: "001",
            },
            VillageCode {
                name: "贡考村民委员会",
                code: "002",
            },
            VillageCode {
                name: "果然村民委员会",
                code: "003",
            },
            VillageCode {
                name: "夺呷村民委员会",
                code: "004",
            },
            VillageCode {
                name: "虾雄村民委员会",
                code: "005",
            },
            VillageCode {
                name: "火然村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "宜牛乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "宜牛村民委员会",
                code: "001",
            },
            VillageCode {
                name: "杜龙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "本日村民委员会",
                code: "003",
            },
            VillageCode {
                name: "捉日村民委员会",
                code: "004",
            },
            VillageCode {
                name: "扎龙村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "起坞乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "格拖村民委员会",
                code: "001",
            },
            VillageCode {
                name: "扎干村民委员会",
                code: "002",
            },
            VillageCode {
                name: "起坞村民委员会",
                code: "003",
            },
            VillageCode {
                name: "甲冲村民委员会",
                code: "004",
            },
            VillageCode {
                name: "觉悟村民委员会",
                code: "005",
            },
            VillageCode {
                name: "重地松村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "长须贡马乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "托思村民委员会",
                code: "001",
            },
            VillageCode {
                name: "河差村民委员会",
                code: "002",
            },
            VillageCode {
                name: "江马村民委员会",
                code: "003",
            },
            VillageCode {
                name: "哈伟村民委员会",
                code: "004",
            },
            VillageCode {
                name: "日美村民委员会",
                code: "005",
            },
            VillageCode {
                name: "尔马底村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "长沙干马乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "曲麦村民委员会",
                code: "001",
            },
            VillageCode {
                name: "伍拉村民委员会",
                code: "002",
            },
            VillageCode {
                name: "生保村民委员会",
                code: "003",
            },
            VillageCode {
                name: "虎热村民委员会",
                code: "004",
            },
            VillageCode {
                name: "约达村民委员会",
                code: "005",
            },
            VillageCode {
                name: "俄加村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "长须干马乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "同呷村民委员会",
                code: "001",
            },
            VillageCode {
                name: "莫日村民委员会",
                code: "002",
            },
            VillageCode {
                name: "领多村民委员会",
                code: "003",
            },
            VillageCode {
                name: "阿都村民委员会",
                code: "004",
            },
            VillageCode {
                name: "热乌村民委员会",
                code: "005",
            },
            VillageCode {
                name: "地香村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "瓦须乡",
        code: "021",
        villages: &[
            VillageCode {
                name: "哈达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "香钦村民委员会",
                code: "002",
            },
            VillageCode {
                name: "干补村民委员会",
                code: "003",
            },
            VillageCode {
                name: "热亚村民委员会",
                code: "004",
            },
            VillageCode {
                name: "舍龙村民委员会",
                code: "005",
            },
            VillageCode {
                name: "夺树村民委员会",
                code: "006",
            },
        ],
    },
];

static TOWNS_XK_030: [TownCode; 16] = [
    TownCode {
        name: "色柯镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "吉祥社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "团结社区居民委员",
                code: "002",
            },
            VillageCode {
                name: "光明社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "安康社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "约若一村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "约若二村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "解放一村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "解放二村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "幸福一村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "幸福二村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "姑咱一村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "姑咱二村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "翁达镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "吉日村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "吉沟村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "翁达村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "更达村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "明达村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "旭尔沟村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "洛若镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "下洛若村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上洛若村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "瓦各村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "曲西村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "日冉村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "扎玛村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "知青村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "沙玛村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "甲西村（左翼）村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "甲修村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "五明佛学院社区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "泥朵镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "格则村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "棒须村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阿吾拉加村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "方仓村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "恰仓村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "查哈尔干玛村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上若撒村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "中若撒村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下若撒村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "东然一村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "东然二村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "克果一村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "克果二村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "甲学镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "切柯村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "草坡村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "雅洛村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "热吾底村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "甲学村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "阿拉甲学村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "二加其村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "容柯村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "克果乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "贡却一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "贡却二村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "贡却三村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "泽西一村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "泽西二村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "泽西三村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "吉日过娃一村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "吉日过娃二村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "然充乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "拉加村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "桑桑村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "呷吉村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "达玛村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "呷加村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "德玛村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "查启村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "佐村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "康勒乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "汪扎一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "汪扎二村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "汪扎三村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "打西日它玛一村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "打西日它玛二村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "阿交撤萨一村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "阿交撤萨二村村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "大章乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "豆加村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "嘎志玛一村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "嘎志玛二村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "加西（右翼）村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "确仓村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "打西贡玛村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下多村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "下门村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "大则乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "厚门村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "约更玛村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "扎门村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "卓更塘村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "那宗塘村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "三郎多村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "玛格村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "泽吾村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "亚龙乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "扎穷村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "下邱果一村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "下邱果二村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下邱果三村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "上邱果一村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "上邱果二村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "色多玛一村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "色多玛二村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "塔子乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "俄儿村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "洞拉村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "洞青村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "吉泽村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "蚌珠村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "雅尔布村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "泸角村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "降央村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "拉隆村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "年龙乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "下修它村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "其哈玛村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "日撒玛村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "俄日村村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "霍西乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "扎卡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "瓦尔村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "玛岗村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "念柯村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "勒柯村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "德汾村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "拉让村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "拉当村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "甲日马村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "瓦热柯村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "甲柯村村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "旭日乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "旭日村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "仁达村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "江达村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "龚古村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "泽登达村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "修灯龙村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "杨各乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "觉洛村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上甲斗村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "下甲斗村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "加更达村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "支巴村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "麦旭村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "亚旭村村民委员会",
                code: "007",
            },
        ],
    },
];

static TOWNS_XK_031: [TownCode; 22] = [
    TownCode {
        name: "高城镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "高城社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "无量河社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "白塔社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "格聂社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "康巴社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "更登亚批社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "哈戈村民委员会",
                code: "007",
            },
            VillageCode {
                name: "替然色巴一村民委员会",
                code: "008",
            },
            VillageCode {
                name: "替然色巴二村民委员会",
                code: "009",
            },
            VillageCode {
                name: "泽曲村民委员会",
                code: "010",
            },
            VillageCode {
                name: "替然尼巴村民委员会",
                code: "011",
            },
            VillageCode {
                name: "德西一村民委员会",
                code: "012",
            },
            VillageCode {
                name: "德西二村民委员会",
                code: "013",
            },
            VillageCode {
                name: "德西三村民委员会",
                code: "014",
            },
            VillageCode {
                name: "车马村民委员会",
                code: "015",
            },
            VillageCode {
                name: "珍呷村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "甲洼镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "下甲洼村民委员会",
                code: "001",
            },
            VillageCode {
                name: "江达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "卡娘村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东珠村民委员会",
                code: "004",
            },
            VillageCode {
                name: "俄丁村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "格聂镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "门子村民委员会",
                code: "001",
            },
            VillageCode {
                name: "格青村民委员会",
                code: "002",
            },
            VillageCode {
                name: "俄多村民委员会",
                code: "003",
            },
            VillageCode {
                name: "日戈村民委员会",
                code: "004",
            },
            VillageCode {
                name: "依拉克村民委员会",
                code: "005",
            },
            VillageCode {
                name: "然日卡村民委员会",
                code: "006",
            },
            VillageCode {
                name: "喇嘛垭村民委员会",
                code: "007",
            },
            VillageCode {
                name: "章纳村民委员会",
                code: "008",
            },
            VillageCode {
                name: "扎拉村民委员会",
                code: "009",
            },
            VillageCode {
                name: "告巫村民委员会",
                code: "010",
            },
            VillageCode {
                name: "乃干多村民委员会",
                code: "011",
            },
            VillageCode {
                name: "则巴村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "木拉镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "黄烟村民委员会",
                code: "001",
            },
            VillageCode {
                name: "哈拉村民委员会",
                code: "002",
            },
            VillageCode {
                name: "格西村民委员会",
                code: "003",
            },
            VillageCode {
                name: "马岩村民委员会",
                code: "004",
            },
            VillageCode {
                name: "则工村民委员会",
                code: "005",
            },
            VillageCode {
                name: "细忠村民委员会",
                code: "006",
            },
            VillageCode {
                name: "作尼村民委员会",
                code: "007",
            },
            VillageCode {
                name: "热拉村民委员会",
                code: "008",
            },
            VillageCode {
                name: "卡下村民委员会",
                code: "009",
            },
            VillageCode {
                name: "措绒村民委员会",
                code: "010",
            },
            VillageCode {
                name: "哈依村民委员会",
                code: "011",
            },
            VillageCode {
                name: "月依村民委员会",
                code: "012",
            },
            VillageCode {
                name: "拿中村民委员会",
                code: "013",
            },
            VillageCode {
                name: "乃沙村民委员会",
                code: "014",
            },
            VillageCode {
                name: "曲呷村民委员会",
                code: "015",
            },
            VillageCode {
                name: "麻依村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "君坝镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "卡共村民委员会",
                code: "001",
            },
            VillageCode {
                name: "合中村民委员会",
                code: "002",
            },
            VillageCode {
                name: "伦多村民委员会",
                code: "003",
            },
            VillageCode {
                name: "火古龙村民委员会",
                code: "004",
            },
            VillageCode {
                name: "若西村民委员会",
                code: "005",
            },
            VillageCode {
                name: "格拉村民委员会",
                code: "006",
            },
            VillageCode {
                name: "协合村民委员会",
                code: "007",
            },
            VillageCode {
                name: "仁坝村民委员会",
                code: "008",
            },
            VillageCode {
                name: "俄何村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "拉波镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "然东村民委员会",
                code: "001",
            },
            VillageCode {
                name: "中扎村民委员会",
                code: "002",
            },
            VillageCode {
                name: "扎扎村民委员会",
                code: "003",
            },
            VillageCode {
                name: "容古村民委员会",
                code: "004",
            },
            VillageCode {
                name: "拉美村民委员会",
                code: "005",
            },
            VillageCode {
                name: "正呷村民委员会",
                code: "006",
            },
            VillageCode {
                name: "协宗村民委员会",
                code: "007",
            },
            VillageCode {
                name: "唐多村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "觉吾镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "作吾村民委员会",
                code: "001",
            },
            VillageCode {
                name: "卡达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "四合村民委员会",
                code: "003",
            },
            VillageCode {
                name: "觉吾村民委员会",
                code: "004",
            },
            VillageCode {
                name: "马里村民委员会",
                code: "005",
            },
            VillageCode {
                name: "业姆村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "哈依乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "哈依村民委员会",
                code: "001",
            },
            VillageCode {
                name: "濯绒村民委员会",
                code: "002",
            },
            VillageCode {
                name: "呷依布里村民委员会",
                code: "003",
            },
            VillageCode {
                name: "桑多村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "莫坝乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "下莫坝村民委员会",
                code: "001",
            },
            VillageCode {
                name: "理坝村民委员会",
                code: "002",
            },
            VillageCode {
                name: "中莫坝村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "亚火乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "下坝村民委员会",
                code: "001",
            },
            VillageCode {
                name: "亚火村民委员会",
                code: "002",
            },
            VillageCode {
                name: "麻火村民委员会",
                code: "003",
            },
            VillageCode {
                name: "五花村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "绒坝乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "增达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "仁达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "额达村民委员会",
                code: "003",
            },
            VillageCode {
                name: "阿宗村民委员会",
                code: "004",
            },
            VillageCode {
                name: "扎西村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "呷洼乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "尼依村民委员会",
                code: "001",
            },
            VillageCode {
                name: "布日村民委员会",
                code: "002",
            },
            VillageCode {
                name: "门斗格村民委员会",
                code: "003",
            },
            VillageCode {
                name: "所底村民委员会",
                code: "004",
            },
            VillageCode {
                name: "香波村民委员会",
                code: "005",
            },
            VillageCode {
                name: "日斗村民委员会",
                code: "006",
            },
            VillageCode {
                name: "日西新村民委员会",
                code: "007",
            },
            VillageCode {
                name: "日里新村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "奔戈乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "托仁村民委员会",
                code: "001",
            },
            VillageCode {
                name: "扎吉贡巴村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阿超村民委员会",
                code: "003",
            },
            VillageCode {
                name: "格扎村民委员会",
                code: "004",
            },
            VillageCode {
                name: "卡灰村民委员会",
                code: "005",
            },
            VillageCode {
                name: "扎呷村民委员会",
                code: "006",
            },
            VillageCode {
                name: "萨戈村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "村戈乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "芒康村民委员会",
                code: "001",
            },
            VillageCode {
                name: "托仁村民委员会",
                code: "002",
            },
            VillageCode {
                name: "雄拉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "村戈村民委员会",
                code: "004",
            },
            VillageCode {
                name: "觉塔村民委员会",
                code: "005",
            },
            VillageCode {
                name: "查卡村民委员会",
                code: "006",
            },
            VillageCode {
                name: "牧民新村村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "禾尼乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "禾然色巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "安久村民委员会",
                code: "002",
            },
            VillageCode {
                name: "克日泽洼村民委员会",
                code: "003",
            },
            VillageCode {
                name: "岭戈村民委员会",
                code: "004",
            },
            VillageCode {
                name: "禾然尼巴村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "曲登乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "额合村民委员会",
                code: "001",
            },
            VillageCode {
                name: "混查一村民委员会",
                code: "002",
            },
            VillageCode {
                name: "浑查村民委员会",
                code: "003",
            },
            VillageCode {
                name: "普查村民委员会",
                code: "004",
            },
            VillageCode {
                name: "哥合村民委员会",
                code: "005",
            },
            VillageCode {
                name: "混查二村民委员会",
                code: "006",
            },
            VillageCode {
                name: "泽洛村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "上木拉乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "乌沙村民委员会",
                code: "001",
            },
            VillageCode {
                name: "奔戈村民委员会",
                code: "002",
            },
            VillageCode {
                name: "亚公村民委员会",
                code: "003",
            },
            VillageCode {
                name: "增德村民委员会",
                code: "004",
            },
            VillageCode {
                name: "格中村民委员会",
                code: "005",
            },
            VillageCode {
                name: "红龙村民委员会",
                code: "006",
            },
            VillageCode {
                name: "旺达村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "濯桑乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "康呷村民委员会",
                code: "001",
            },
            VillageCode {
                name: "汝村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "汉戈村民委员会",
                code: "003",
            },
            VillageCode {
                name: "业务村民委员会",
                code: "004",
            },
            VillageCode {
                name: "易久村民委员会",
                code: "005",
            },
            VillageCode {
                name: "若拉村民委员会",
                code: "006",
            },
            VillageCode {
                name: "古君村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "藏坝乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "安多村民委员会",
                code: "001",
            },
            VillageCode {
                name: "扎西村民委员会",
                code: "002",
            },
            VillageCode {
                name: "亚中村民委员会",
                code: "003",
            },
            VillageCode {
                name: "固中村民委员会",
                code: "004",
            },
            VillageCode {
                name: "大信乃村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "格木乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "加细村民委员会",
                code: "001",
            },
            VillageCode {
                name: "察卡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "学说村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "麦洼乡",
        code: "021",
        villages: &[
            VillageCode {
                name: "大中村民委员会",
                code: "001",
            },
            VillageCode {
                name: "措洼村民委员会",
                code: "002",
            },
            VillageCode {
                name: "热鲁村民委员会",
                code: "003",
            },
            VillageCode {
                name: "埃地村民委员会",
                code: "004",
            },
            VillageCode {
                name: "卡龚村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "德巫乡",
        code: "022",
        villages: &[
            VillageCode {
                name: "当巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "白中村民委员会",
                code: "002",
            },
            VillageCode {
                name: "日瓦桥村民委员会",
                code: "003",
            },
            VillageCode {
                name: "措洼村民委员会",
                code: "004",
            },
            VillageCode {
                name: "日乃村民委员会",
                code: "005",
            },
            VillageCode {
                name: "协巫村民委员会",
                code: "006",
            },
        ],
    },
];

static TOWNS_XK_032: [TownCode; 17] = [
    TownCode {
        name: "夏邛镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "夏邛镇社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "四里龙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "巴邛西村民委员会",
                code: "003",
            },
            VillageCode {
                name: "泽曲伙村民委员会",
                code: "004",
            },
            VillageCode {
                name: "拉宗伙村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下桑卡村民委员会",
                code: "006",
            },
            VillageCode {
                name: "架炮顶村民委员会",
                code: "007",
            },
            VillageCode {
                name: "茶雪村民委员会",
                code: "008",
            },
            VillageCode {
                name: "生奔扎村民委员会",
                code: "009",
            },
            VillageCode {
                name: "孔打伙二村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "洛布通顶村民委员会",
                code: "011",
            },
            VillageCode {
                name: "独角龙村民委员会",
                code: "012",
            },
            VillageCode {
                name: "江巴顶村民委员会",
                code: "013",
            },
            VillageCode {
                name: "坝伙村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "中咱镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "中咱村民委员会",
                code: "001",
            },
            VillageCode {
                name: "仁德村民委员会",
                code: "002",
            },
            VillageCode {
                name: "雪波村民委员会",
                code: "003",
            },
            VillageCode {
                name: "洛玉贡村民委员会",
                code: "004",
            },
            VillageCode {
                name: "下中咱村民委员会",
                code: "005",
            },
            VillageCode {
                name: "里甫村民委员会",
                code: "006",
            },
            VillageCode {
                name: "波浪村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "措拉镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "麻通村民委员会",
                code: "001",
            },
            VillageCode {
                name: "俊工村民委员会",
                code: "002",
            },
            VillageCode {
                name: "玉绒村民委员会",
                code: "003",
            },
            VillageCode {
                name: "德西村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "甲英镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "党巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "英戈贡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "鱼卡通村民委员会",
                code: "003",
            },
            VillageCode {
                name: "老雅哇村民委员会",
                code: "004",
            },
            VillageCode {
                name: "冲茶村民委员会",
                code: "005",
            },
            VillageCode {
                name: "波戈西村民委员会",
                code: "006",
            },
            VillageCode {
                name: "普达村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "地巫镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "尼木龙村民委员会",
                code: "001",
            },
            VillageCode {
                name: "贡伙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "安里顶村民委员会",
                code: "003",
            },
            VillageCode {
                name: "仁娘村民委员会",
                code: "004",
            },
            VillageCode {
                name: "坝伙村民委员会",
                code: "005",
            },
            VillageCode {
                name: "甲雪村民委员会",
                code: "006",
            },
            VillageCode {
                name: "热思村民委员会",
                code: "007",
            },
            VillageCode {
                name: "四点坝村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "拉哇乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "拉哇村民委员会",
                code: "001",
            },
            VillageCode {
                name: "洛毕村民委员会",
                code: "002",
            },
            VillageCode {
                name: "毕英村民委员会",
                code: "003",
            },
            VillageCode {
                name: "索英村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "竹巴龙乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "自林贡村民委员会",
                code: "001",
            },
            VillageCode {
                name: "三各贡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "基里村民委员会",
                code: "003",
            },
            VillageCode {
                name: "水磨沟村民委员会",
                code: "004",
            },
            VillageCode {
                name: "拉扎西村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "苏哇龙乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "苏哇龙村民委员会",
                code: "001",
            },
            VillageCode {
                name: "王大龙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "南戈村民委员会",
                code: "003",
            },
            VillageCode {
                name: "归哇村民委员会",
                code: "004",
            },
            VillageCode {
                name: "呷顶村民委员会",
                code: "005",
            },
            VillageCode {
                name: "贡巴村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "昌波乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "戈郎村民委员会",
                code: "001",
            },
            VillageCode {
                name: "鱼底村民委员会",
                code: "002",
            },
            VillageCode {
                name: "锐哇村民委员会",
                code: "003",
            },
            VillageCode {
                name: "得木村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "亚日贡乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "刀许村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东南多村民委员会",
                code: "002",
            },
            VillageCode {
                name: "亚日贡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "白日贡村民委员会",
                code: "004",
            },
            VillageCode {
                name: "红日贡村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "波密乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "波堆村民委员会",
                code: "001",
            },
            VillageCode {
                name: "波免村民委员会",
                code: "002",
            },
            VillageCode {
                name: "格木村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "莫多乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "桑龙西村民委员会",
                code: "001",
            },
            VillageCode {
                name: "莫多村民委员会",
                code: "002",
            },
            VillageCode {
                name: "郎翁村民委员会",
                code: "003",
            },
            VillageCode {
                name: "岗佐村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "松多乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "松多村民委员会",
                code: "001",
            },
            VillageCode {
                name: "郎多一村民委员会",
                code: "002",
            },
            VillageCode {
                name: "郎多二村民委员会",
                code: "003",
            },
            VillageCode {
                name: "龙巴村民委员会",
                code: "004",
            },
            VillageCode {
                name: "吉恩龙村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下莫西村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "波戈溪乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "益金村民委员会",
                code: "001",
            },
            VillageCode {
                name: "波戈溪村民委员会",
                code: "002",
            },
            VillageCode {
                name: "夺格村民委员会",
                code: "003",
            },
            VillageCode {
                name: "那多村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "茶洛乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "茶洛村民委员会",
                code: "001",
            },
            VillageCode {
                name: "达塔村民委员会",
                code: "002",
            },
            VillageCode {
                name: "尼戈村民委员会",
                code: "003",
            },
            VillageCode {
                name: "西德村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "列衣乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "仲堆村民委员会",
                code: "001",
            },
            VillageCode {
                name: "自热村民委员会",
                code: "002",
            },
            VillageCode {
                name: "热乃村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "德达乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "上德达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "中德达村民委员会",
                code: "002",
            },
            VillageCode {
                name: "曲呷村民委员会",
                code: "003",
            },
            VillageCode {
                name: "牧业村村民委员会",
                code: "004",
            },
        ],
    },
];

static TOWNS_XK_033: [TownCode; 10] = [
    TownCode {
        name: "香巴拉镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "幸福社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "香巴拉社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "巴姆山社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "渔洼仲村民委员会",
                code: "004",
            },
            VillageCode {
                name: "色尔宫村民委员会",
                code: "005",
            },
            VillageCode {
                name: "热郎宫村民委员会",
                code: "006",
            },
            VillageCode {
                name: "冷龙村民委员会",
                code: "007",
            },
            VillageCode {
                name: "东宫村民委员会",
                code: "008",
            },
            VillageCode {
                name: "登仲村民委员会",
                code: "009",
            },
            VillageCode {
                name: "阿央仲村民委员会",
                code: "010",
            },
            VillageCode {
                name: "马色村民委员会",
                code: "011",
            },
            VillageCode {
                name: "沙孜村民委员会",
                code: "012",
            },
            VillageCode {
                name: "则鲁村民委员会",
                code: "013",
            },
            VillageCode {
                name: "岗色村民委员会",
                code: "014",
            },
            VillageCode {
                name: "边边哨村民委员会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "青德镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "热宫村民委员会",
                code: "001",
            },
            VillageCode {
                name: "布机村民委员会",
                code: "002",
            },
            VillageCode {
                name: "仲德村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下坝村民委员会",
                code: "004",
            },
            VillageCode {
                name: "呷乃卡村民委员会",
                code: "005",
            },
            VillageCode {
                name: "仁堆村民委员会",
                code: "006",
            },
            VillageCode {
                name: "木差村民委员会",
                code: "007",
            },
            VillageCode {
                name: "青麦村民委员会",
                code: "008",
            },
            VillageCode {
                name: "巴吾村民委员会",
                code: "009",
            },
            VillageCode {
                name: "黑达村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "热打镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "热打村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东均村民委员会",
                code: "002",
            },
            VillageCode {
                name: "下洼村民委员会",
                code: "003",
            },
            VillageCode {
                name: "木鱼村民委员会",
                code: "004",
            },
            VillageCode {
                name: "阿都村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "沙贡乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "章吉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "同颠村民委员会",
                code: "002",
            },
            VillageCode {
                name: "达根村民委员会",
                code: "003",
            },
            VillageCode {
                name: "色坝村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "水洼乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "白格村民委员会",
                code: "001",
            },
            VillageCode {
                name: "俄扎村民委员会",
                code: "002",
            },
            VillageCode {
                name: "水洼村民委员会",
                code: "003",
            },
            VillageCode {
                name: "那拉岗村民委员会",
                code: "004",
            },
            VillageCode {
                name: "雨洼村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "然乌乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "东尔村民委员会",
                code: "001",
            },
            VillageCode {
                name: "克麦村民委员会",
                code: "002",
            },
            VillageCode {
                name: "纳木村民委员会",
                code: "003",
            },
            VillageCode {
                name: "热麦村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "洞松乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "固松村民委员会",
                code: "001",
            },
            VillageCode {
                name: "热斗村民委员会",
                code: "002",
            },
            VillageCode {
                name: "克斗村民委员会",
                code: "003",
            },
            VillageCode {
                name: "卡心村民委员会",
                code: "004",
            },
            VillageCode {
                name: "洞松村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "定波乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "绒公村民委员会",
                code: "001",
            },
            VillageCode {
                name: "白兰斗村民委员会",
                code: "002",
            },
            VillageCode {
                name: "万绒村民委员会",
                code: "003",
            },
            VillageCode {
                name: "普通村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "正斗乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "白坝村民委员会",
                code: "001",
            },
            VillageCode {
                name: "正斗村民委员会",
                code: "002",
            },
            VillageCode {
                name: "仁额村民委员会",
                code: "003",
            },
            VillageCode {
                name: "永德村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "白依乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "青打村民委员会",
                code: "001",
            },
            VillageCode {
                name: "布吉村民委员会",
                code: "002",
            },
            VillageCode {
                name: "跃卡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "纳力村民委员会",
                code: "004",
            },
        ],
    },
];

static TOWNS_XK_034: [TownCode; 13] = [
    TownCode {
        name: "金珠镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "德西社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "贡巴社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "上茹布村民委员会",
                code: "003",
            },
            VillageCode {
                name: "所瓦村民委员会",
                code: "004",
            },
            VillageCode {
                name: "扎冲村民委员会",
                code: "005",
            },
            VillageCode {
                name: "子洛村民委员会",
                code: "006",
            },
            VillageCode {
                name: "平洛七家村民委员会",
                code: "007",
            },
            VillageCode {
                name: "中茹布洗澡塘村民委员会",
                code: "008",
            },
            VillageCode {
                name: "仲麦村民委员会",
                code: "009",
            },
            VillageCode {
                name: "培光村民委员会",
                code: "010",
            },
            VillageCode {
                name: "额依扒戈村民委员会",
                code: "011",
            },
            VillageCode {
                name: "上下龙古村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "香格里拉镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "呷拥社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "俄初村民委员会",
                code: "002",
            },
            VillageCode {
                name: "亚丁村民委员会",
                code: "003",
            },
            VillageCode {
                name: "康古村民委员会",
                code: "004",
            },
            VillageCode {
                name: "热光村民委员会",
                code: "005",
            },
            VillageCode {
                name: "日美村民委员会",
                code: "006",
            },
            VillageCode {
                name: "瓦龙村民委员会",
                code: "007",
            },
            VillageCode {
                name: "阿称贡村民委员会",
                code: "008",
            },
            VillageCode {
                name: "拉木格村民委员会",
                code: "009",
            },
            VillageCode {
                name: "郎日村民委员会",
                code: "010",
            },
            VillageCode {
                name: "叶尔红村民委员会",
                code: "011",
            },
            VillageCode {
                name: "仁村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "桑堆镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "桑堆村民委员会",
                code: "001",
            },
            VillageCode {
                name: "吉乙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "所冲村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "吉呷镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "呷拉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "尼公村民委员会",
                code: "002",
            },
            VillageCode {
                name: "巨水村民委员会",
                code: "003",
            },
            VillageCode {
                name: "吉尔同村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "噶通镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "永当村民委员会",
                code: "001",
            },
            VillageCode {
                name: "仲堆仲麦村民委员会",
                code: "002",
            },
            VillageCode {
                name: "寒桑村民委员会",
                code: "003",
            },
            VillageCode {
                name: "仲久村民委员会",
                code: "004",
            },
            VillageCode {
                name: "普牙村民委员会",
                code: "005",
            },
            VillageCode {
                name: "自俄村民委员会",
                code: "006",
            },
            VillageCode {
                name: "白中村民委员会",
                code: "007",
            },
            VillageCode {
                name: "龙伍村民委员会",
                code: "008",
            },
            VillageCode {
                name: "寒洪村民委员会",
                code: "009",
            },
            VillageCode {
                name: "普尼村民委员会",
                code: "010",
            },
            VillageCode {
                name: "八美村民委员会",
                code: "011",
            },
            VillageCode {
                name: "色拉村民委员会",
                code: "012",
            },
            VillageCode {
                name: "口提村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "省母乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "省母村民委员会",
                code: "001",
            },
            VillageCode {
                name: "冉子村民委员会",
                code: "002",
            },
            VillageCode {
                name: "查花村民委员会",
                code: "003",
            },
            VillageCode {
                name: "各瓦村民委员会",
                code: "004",
            },
            VillageCode {
                name: "日火村民委员会",
                code: "005",
            },
            VillageCode {
                name: "协坡村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "巨龙乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "同古村民委员会",
                code: "001",
            },
            VillageCode {
                name: "西沙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "卫仲村民委员会",
                code: "003",
            },
            VillageCode {
                name: "瓦龙村民委员会",
                code: "004",
            },
            VillageCode {
                name: "然央村民委员会",
                code: "005",
            },
            VillageCode {
                name: "布郎村民委员会",
                code: "006",
            },
            VillageCode {
                name: "举冲村民委员会",
                code: "007",
            },
            VillageCode {
                name: "各瓦村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "邓坡乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "上邓坡村民委员会",
                code: "001",
            },
            VillageCode {
                name: "下邓坡村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "木拉乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "培考村民委员会",
                code: "001",
            },
            VillageCode {
                name: "半岛村民委员会",
                code: "002",
            },
            VillageCode {
                name: "马武村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "赤土乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "阿西村民委员会",
                code: "001",
            },
            VillageCode {
                name: "孔仁村民委员会",
                code: "002",
            },
            VillageCode {
                name: "贡沙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "建丁村民委员会",
                code: "004",
            },
            VillageCode {
                name: "沙堆村民委员会",
                code: "005",
            },
            VillageCode {
                name: "克瓦村民委员会",
                code: "006",
            },
            VillageCode {
                name: "德甲村民委员会",
                code: "007",
            },
            VillageCode {
                name: "子定村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "蒙自乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "瓦格村民委员会",
                code: "001",
            },
            VillageCode {
                name: "俄伦村民委员会",
                code: "002",
            },
            VillageCode {
                name: "自龙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "黑龙村民委员会",
                code: "004",
            },
            VillageCode {
                name: "亚中村民委员会",
                code: "005",
            },
            VillageCode {
                name: "桑达村民委员会",
                code: "006",
            },
            VillageCode {
                name: "格沙村民委员会",
                code: "007",
            },
            VillageCode {
                name: "米绒村民委员会",
                code: "008",
            },
            VillageCode {
                name: "贡关村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "各卡乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "卡斯村民委员会",
                code: "001",
            },
            VillageCode {
                name: "泥龙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "白合村民委员会",
                code: "003",
            },
            VillageCode {
                name: "思子村民委员会",
                code: "004",
            },
            VillageCode {
                name: "各瓦村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "俄牙同乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "俄眉村民委员会",
                code: "001",
            },
            VillageCode {
                name: "同顶村民委员会",
                code: "002",
            },
            VillageCode {
                name: "下同村民委员会",
                code: "003",
            },
            VillageCode {
                name: "向英村民委员会",
                code: "004",
            },
            VillageCode {
                name: "翁孜村民委员会",
                code: "005",
            },
            VillageCode {
                name: "同乡村民委员会",
                code: "006",
            },
            VillageCode {
                name: "牙垭村民委员会",
                code: "007",
            },
        ],
    },
];

static TOWNS_XK_035: [TownCode; 10] = [
    TownCode {
        name: "瓦卡镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "瓦卡居民委员会",
                code: "001",
            },
            VillageCode {
                name: "瓦卡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阿洛贡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "曲岗顶村民委员会",
                code: "004",
            },
            VillageCode {
                name: "阿称村民委员会",
                code: "005",
            },
            VillageCode {
                name: "子实村民委员会",
                code: "006",
            },
            VillageCode {
                name: "子庚村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "白松镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "白松村民委员会",
                code: "001",
            },
            VillageCode {
                name: "同归村民委员会",
                code: "002",
            },
            VillageCode {
                name: "日麦村民委员会",
                code: "003",
            },
            VillageCode {
                name: "依打村民委员会",
                code: "004",
            },
            VillageCode {
                name: "门扎村民委员会",
                code: "005",
            },
            VillageCode {
                name: "列依村民委员会",
                code: "006",
            },
            VillageCode {
                name: "亭子村民委员会",
                code: "007",
            },
            VillageCode {
                name: "夺松贡村民委员会",
                code: "008",
            },
            VillageCode {
                name: "日惹村民委员会",
                code: "009",
            },
            VillageCode {
                name: "点仲村民委员会",
                code: "010",
            },
            VillageCode {
                name: "夺松村民委员会",
                code: "011",
            },
            VillageCode {
                name: "日当村民委员会",
                code: "012",
            },
            VillageCode {
                name: "日堆村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "日雨镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "日堆村民委员会",
                code: "001",
            },
            VillageCode {
                name: "如贡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "因都二村民委员会",
                code: "003",
            },
            VillageCode {
                name: "日麦村民委员会",
                code: "004",
            },
            VillageCode {
                name: "因都三村民委员会",
                code: "005",
            },
            VillageCode {
                name: "拥真村民委员会",
                code: "006",
            },
            VillageCode {
                name: "得木同村民委员会",
                code: "007",
            },
            VillageCode {
                name: "龚古村民委员会",
                code: "008",
            },
            VillageCode {
                name: "绒丁村民委员会",
                code: "009",
            },
            VillageCode {
                name: "扎叶村民委员会",
                code: "010",
            },
            VillageCode {
                name: "绒学村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "太阳谷镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "河西居民委员会",
                code: "001",
            },
            VillageCode {
                name: "河东居民委员会",
                code: "002",
            },
            VillageCode {
                name: "松麦村民委员会",
                code: "003",
            },
            VillageCode {
                name: "扎格村民委员会",
                code: "004",
            },
            VillageCode {
                name: "扎丁村民委员会",
                code: "005",
            },
            VillageCode {
                name: "格绒村民委员会",
                code: "006",
            },
            VillageCode {
                name: "鱼根村民委员会",
                code: "007",
            },
            VillageCode {
                name: "因达村民委员会",
                code: "008",
            },
            VillageCode {
                name: "松堆村民委员会",
                code: "009",
            },
            VillageCode {
                name: "约日村民委员会",
                code: "010",
            },
            VillageCode {
                name: "冉绒村民委员会",
                code: "011",
            },
            VillageCode {
                name: "冻谷村民委员会",
                code: "012",
            },
            VillageCode {
                name: "尼日村民委员会",
                code: "013",
            },
            VillageCode {
                name: "卡龚村民委员会",
                code: "014",
            },
            VillageCode {
                name: "下绒村民委员会",
                code: "015",
            },
            VillageCode {
                name: "浪中村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "徐龙乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "张仁村民委员会",
                code: "001",
            },
            VillageCode {
                name: "鱼波村民委员会",
                code: "002",
            },
            VillageCode {
                name: "嘎校村民委员会",
                code: "003",
            },
            VillageCode {
                name: "布中村民委员会",
                code: "004",
            },
            VillageCode {
                name: "徐麦村民委员会",
                code: "005",
            },
            VillageCode {
                name: "尼中村民委员会",
                code: "006",
            },
            VillageCode {
                name: "宗绒村民委员会",
                code: "007",
            },
            VillageCode {
                name: "莫丁村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "奔都乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "奔都村民委员会",
                code: "001",
            },
            VillageCode {
                name: "沙中村民委员会",
                code: "002",
            },
            VillageCode {
                name: "莫木上村民委员会",
                code: "003",
            },
            VillageCode {
                name: "莫木中村民委员会",
                code: "004",
            },
            VillageCode {
                name: "莫木下村民委员会",
                code: "005",
            },
            VillageCode {
                name: "俄木学村民委员会",
                code: "006",
            },
            VillageCode {
                name: "甲音村民委员会",
                code: "007",
            },
            VillageCode {
                name: "纳莫村民委员会",
                code: "008",
            },
            VillageCode {
                name: "嘎色村民委员会",
                code: "009",
            },
            VillageCode {
                name: "奔色村民委员会",
                code: "010",
            },
            VillageCode {
                name: "木格村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "八日乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "学巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "共巴村民委员会",
                code: "002",
            },
            VillageCode {
                name: "白顶村民委员会",
                code: "003",
            },
            VillageCode {
                name: "通都村民委员会",
                code: "004",
            },
            VillageCode {
                name: "子哇村民委员会",
                code: "005",
            },
            VillageCode {
                name: "通古村民委员会",
                code: "006",
            },
            VillageCode {
                name: "黑里村民委员会",
                code: "007",
            },
            VillageCode {
                name: "亚岗村民委员会",
                code: "008",
            },
            VillageCode {
                name: "嘎拥村民委员会",
                code: "009",
            },
            VillageCode {
                name: "让古村民委员会",
                code: "010",
            },
            VillageCode {
                name: "日西顶村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "古学乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "古学村民委员会",
                code: "001",
            },
            VillageCode {
                name: "德则村民委员会",
                code: "002",
            },
            VillageCode {
                name: "比拥村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下拥村民委员会",
                code: "004",
            },
            VillageCode {
                name: "卡日贡村民委员会",
                code: "005",
            },
            VillageCode {
                name: "日丁村民委员会",
                code: "006",
            },
            VillageCode {
                name: "日中村民委员会",
                code: "007",
            },
            VillageCode {
                name: "毛屋村民委员会",
                code: "008",
            },
            VillageCode {
                name: "左贡村民委员会",
                code: "009",
            },
            VillageCode {
                name: "塔恩同村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "贡波乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "联谊村民委员会",
                code: "001",
            },
            VillageCode {
                name: "中木村民委员会",
                code: "002",
            },
            VillageCode {
                name: "普雅村民委员会",
                code: "003",
            },
            VillageCode {
                name: "贡堆村民委员会",
                code: "004",
            },
            VillageCode {
                name: "日归村民委员会",
                code: "005",
            },
            VillageCode {
                name: "木拥村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "茨巫乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "卡工村民委员会",
                code: "001",
            },
            VillageCode {
                name: "杠然村民委员会",
                code: "002",
            },
            VillageCode {
                name: "支然村民委员会",
                code: "003",
            },
            VillageCode {
                name: "日拥村民委员会",
                code: "004",
            },
            VillageCode {
                name: "吉冲村民委员会",
                code: "005",
            },
            VillageCode {
                name: "亚郎村民委员会",
                code: "006",
            },
            VillageCode {
                name: "绒各村民委员会",
                code: "007",
            },
            VillageCode {
                name: "曲岗村民委员会",
                code: "008",
            },
            VillageCode {
                name: "郎达村民委员会",
                code: "009",
            },
            VillageCode {
                name: "绒都村民委员会",
                code: "010",
            },
        ],
    },
];

pub const CITIES_XK: [CityCode; 36] = [
    CityCode {
        name: "省辖市",
        code: "000",
        towns: &[],
    },
    CityCode {
        name: "香格里拉市",
        code: "001",
        towns: &TOWNS_XK_001,
    },
    CityCode {
        name: "德钦市",
        code: "002",
        towns: &TOWNS_XK_002,
    },
    CityCode {
        name: "维西市",
        code: "003",
        towns: &TOWNS_XK_003,
    },
    CityCode {
        name: "泸水市",
        code: "004",
        towns: &TOWNS_XK_004,
    },
    CityCode {
        name: "福贡市",
        code: "005",
        towns: &TOWNS_XK_005,
    },
    CityCode {
        name: "兰坪市",
        code: "006",
        towns: &TOWNS_XK_006,
    },
    CityCode {
        name: "卡若市",
        code: "007",
        towns: &TOWNS_XK_007,
    },
    CityCode {
        name: "江达市",
        code: "008",
        towns: &TOWNS_XK_008,
    },
    CityCode {
        name: "贡觉市",
        code: "009",
        towns: &TOWNS_XK_009,
    },
    CityCode {
        name: "类乌齐市",
        code: "010",
        towns: &TOWNS_XK_010,
    },
    CityCode {
        name: "丁青市",
        code: "011",
        towns: &TOWNS_XK_011,
    },
    CityCode {
        name: "察雅市",
        code: "012",
        towns: &TOWNS_XK_012,
    },
    CityCode {
        name: "八宿市",
        code: "013",
        towns: &TOWNS_XK_013,
    },
    CityCode {
        name: "左贡市",
        code: "014",
        towns: &TOWNS_XK_014,
    },
    CityCode {
        name: "芒康市",
        code: "015",
        towns: &TOWNS_XK_015,
    },
    CityCode {
        name: "洛隆市",
        code: "016",
        towns: &TOWNS_XK_016,
    },
    CityCode {
        name: "边坝市",
        code: "017",
        towns: &TOWNS_XK_017,
    },
    CityCode {
        name: "康定市",
        code: "018",
        towns: &TOWNS_XK_018,
    },
    CityCode {
        name: "泸定市",
        code: "019",
        towns: &TOWNS_XK_019,
    },
    CityCode {
        name: "丹巴市",
        code: "020",
        towns: &TOWNS_XK_020,
    },
    CityCode {
        name: "九龙市",
        code: "021",
        towns: &TOWNS_XK_021,
    },
    CityCode {
        name: "雅江市",
        code: "022",
        towns: &TOWNS_XK_022,
    },
    CityCode {
        name: "道孚市",
        code: "023",
        towns: &TOWNS_XK_023,
    },
    CityCode {
        name: "炉霍市",
        code: "024",
        towns: &TOWNS_XK_024,
    },
    CityCode {
        name: "甘孜市",
        code: "025",
        towns: &TOWNS_XK_025,
    },
    CityCode {
        name: "新龙市",
        code: "026",
        towns: &TOWNS_XK_026,
    },
    CityCode {
        name: "德格市",
        code: "027",
        towns: &TOWNS_XK_027,
    },
    CityCode {
        name: "白玉市",
        code: "028",
        towns: &TOWNS_XK_028,
    },
    CityCode {
        name: "石渠市",
        code: "029",
        towns: &TOWNS_XK_029,
    },
    CityCode {
        name: "色达市",
        code: "030",
        towns: &TOWNS_XK_030,
    },
    CityCode {
        name: "理塘市",
        code: "031",
        towns: &TOWNS_XK_031,
    },
    CityCode {
        name: "巴塘市",
        code: "032",
        towns: &TOWNS_XK_032,
    },
    CityCode {
        name: "乡城市",
        code: "033",
        towns: &TOWNS_XK_033,
    },
    CityCode {
        name: "稻城市",
        code: "034",
        towns: &TOWNS_XK_034,
    },
    CityCode {
        name: "得荣市",
        code: "035",
        towns: &TOWNS_XK_035,
    },
];
