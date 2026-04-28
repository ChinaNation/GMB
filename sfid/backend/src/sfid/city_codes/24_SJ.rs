use super::{CityCode, TownCode, VillageCode};

static TOWNS_SJ_001: [TownCode; 1] = [TownCode {
    name: "上海街道",
    code: "001",
    villages: &[
        VillageCode {
            name: "牡丹社区居委会",
            code: "001",
        },
        VillageCode {
            name: "上海新苑社区居委会",
            code: "002",
        },
        VillageCode {
            name: "河上社区居委会",
            code: "003",
        },
        VillageCode {
            name: "凤凰社区居委会",
            code: "004",
        },
        VillageCode {
            name: "西洋社区居委会",
            code: "005",
        },
        VillageCode {
            name: "交通社区居委会",
            code: "006",
        },
        VillageCode {
            name: "万象社区居委会",
            code: "007",
        },
    ],
}];

static TOWNS_SJ_002: [TownCode; 10] = [
    TownCode {
        name: "南京东路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "云南中路居委会",
                code: "001",
            },
            VillageCode {
                name: "龙泉园路居委会",
                code: "002",
            },
            VillageCode {
                name: "贵州路居委会",
                code: "003",
            },
            VillageCode {
                name: "新桥居委会",
                code: "004",
            },
            VillageCode {
                name: "牛庄路居委会",
                code: "005",
            },
            VillageCode {
                name: "厦门路居委会",
                code: "006",
            },
            VillageCode {
                name: "福海居委会",
                code: "007",
            },
            VillageCode {
                name: "承兴居委会",
                code: "008",
            },
            VillageCode {
                name: "三德居委会",
                code: "009",
            },
            VillageCode {
                name: "福瑞居委会",
                code: "010",
            },
            VillageCode {
                name: "平望街居委会",
                code: "011",
            },
            VillageCode {
                name: "小花园居委会",
                code: "012",
            },
            VillageCode {
                name: "长江居委会",
                code: "013",
            },
            VillageCode {
                name: "江阴居委会",
                code: "014",
            },
            VillageCode {
                name: "均乐居委会",
                code: "015",
            },
            VillageCode {
                name: "新昌居委会",
                code: "016",
            },
            VillageCode {
                name: "振兴居委会",
                code: "017",
            },
            VillageCode {
                name: "定兴居委会",
                code: "018",
            },
            VillageCode {
                name: "顺天村居委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "外滩街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "汉口路居委会",
                code: "001",
            },
            VillageCode {
                name: "北京居委会",
                code: "002",
            },
            VillageCode {
                name: "宁波路居委会",
                code: "003",
            },
            VillageCode {
                name: "无锡路居委会",
                code: "004",
            },
            VillageCode {
                name: "东风居委会",
                code: "005",
            },
            VillageCode {
                name: "山东北路居委会",
                code: "006",
            },
            VillageCode {
                name: "虎丘路居委会",
                code: "007",
            },
            VillageCode {
                name: "永安路居委会",
                code: "008",
            },
            VillageCode {
                name: "永胜路居委会",
                code: "009",
            },
            VillageCode {
                name: "中山东一路居委会",
                code: "010",
            },
            VillageCode {
                name: "盛泽居委会",
                code: "011",
            },
            VillageCode {
                name: "宝兴居委会",
                code: "012",
            },
            VillageCode {
                name: "昭通路居委会",
                code: "013",
            },
            VillageCode {
                name: "瑞福居委会",
                code: "014",
            },
            VillageCode {
                name: "云南居委会",
                code: "015",
            },
            VillageCode {
                name: "山西南路居委会",
                code: "016",
            },
            VillageCode {
                name: "新建二村居委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "半淞园路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "民立居委会",
                code: "001",
            },
            VillageCode {
                name: "瞿四居委会",
                code: "002",
            },
            VillageCode {
                name: "西三居委会",
                code: "003",
            },
            VillageCode {
                name: "徽宁居委会",
                code: "004",
            },
            VillageCode {
                name: "高雄居委会",
                code: "005",
            },
            VillageCode {
                name: "瞿二居委会",
                code: "006",
            },
            VillageCode {
                name: "三门峡居委会",
                code: "007",
            },
            VillageCode {
                name: "市民居委会",
                code: "008",
            },
            VillageCode {
                name: "西一居委会",
                code: "009",
            },
            VillageCode {
                name: "制造居委会",
                code: "010",
            },
            VillageCode {
                name: "保屯居委会",
                code: "011",
            },
            VillageCode {
                name: "西二居委会",
                code: "012",
            },
            VillageCode {
                name: "耀江花园居委会",
                code: "013",
            },
            VillageCode {
                name: "黄浦新苑居委会",
                code: "014",
            },
            VillageCode {
                name: "迎勋居委会",
                code: "015",
            },
            VillageCode {
                name: "海西居委会",
                code: "016",
            },
            VillageCode {
                name: "益元居委会",
                code: "017",
            },
            VillageCode {
                name: "中福花苑第一居委会",
                code: "018",
            },
            VillageCode {
                name: "车中居委会",
                code: "019",
            },
            VillageCode {
                name: "普益居委会",
                code: "020",
            },
            VillageCode {
                name: "新村居委会",
                code: "021",
            },
            VillageCode {
                name: "中福花苑第二居委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "小东门街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "新码居委会",
                code: "001",
            },
            VillageCode {
                name: "龙潭居委会",
                code: "002",
            },
            VillageCode {
                name: "西姚居委会",
                code: "003",
            },
            VillageCode {
                name: "中华居委会",
                code: "004",
            },
            VillageCode {
                name: "白渡居委会",
                code: "005",
            },
            VillageCode {
                name: "天灯居委会",
                code: "006",
            },
            VillageCode {
                name: "小石桥居委会",
                code: "007",
            },
            VillageCode {
                name: "乔家居委会",
                code: "008",
            },
            VillageCode {
                name: "金坛居委会",
                code: "009",
            },
            VillageCode {
                name: "赵家宅居委会",
                code: "010",
            },
            VillageCode {
                name: "荷花池居委会",
                code: "011",
            },
            VillageCode {
                name: "阳光居委会",
                code: "012",
            },
            VillageCode {
                name: "多稼居委会",
                code: "013",
            },
            VillageCode {
                name: "万裕居委会",
                code: "014",
            },
            VillageCode {
                name: "府谷居委会",
                code: "015",
            },
            VillageCode {
                name: "桑园居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "豫园街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "四新居委会",
                code: "001",
            },
            VillageCode {
                name: "宝带居委会",
                code: "002",
            },
            VillageCode {
                name: "光启居委会",
                code: "003",
            },
            VillageCode {
                name: "果育居委会",
                code: "004",
            },
            VillageCode {
                name: "太阳都市居委会",
                code: "005",
            },
            VillageCode {
                name: "丹马居委会",
                code: "006",
            },
            VillageCode {
                name: "学院居委会",
                code: "007",
            },
            VillageCode {
                name: "露香居委会",
                code: "008",
            },
            VillageCode {
                name: "广福居委会",
                code: "009",
            },
            VillageCode {
                name: "肇方居委会",
                code: "010",
            },
            VillageCode {
                name: "同庆居委会",
                code: "011",
            },
            VillageCode {
                name: "泰瑞居委会",
                code: "012",
            },
            VillageCode {
                name: "淮海居委会",
                code: "013",
            },
            VillageCode {
                name: "会稽居委会",
                code: "014",
            },
            VillageCode {
                name: "方西居委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "老西门街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "大兴居委会",
                code: "001",
            },
            VillageCode {
                name: "大林居委会",
                code: "002",
            },
            VillageCode {
                name: "文庙居委会",
                code: "003",
            },
            VillageCode {
                name: "学宫街居委会",
                code: "004",
            },
            VillageCode {
                name: "曹家街居委会",
                code: "005",
            },
            VillageCode {
                name: "小桃园居委会",
                code: "006",
            },
            VillageCode {
                name: "方斜居委会",
                code: "007",
            },
            VillageCode {
                name: "牌楼居委会",
                code: "008",
            },
            VillageCode {
                name: "也是园居委会",
                code: "009",
            },
            VillageCode {
                name: "龙门邨居委会",
                code: "010",
            },
            VillageCode {
                name: "小西门居委会",
                code: "011",
            },
            VillageCode {
                name: "唐家湾居委会",
                code: "012",
            },
            VillageCode {
                name: "净土街居委会",
                code: "013",
            },
            VillageCode {
                name: "陆兴居委会",
                code: "014",
            },
            VillageCode {
                name: "明日星城居委会",
                code: "015",
            },
            VillageCode {
                name: "陆迎居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "五里桥街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "桥一居委会",
                code: "001",
            },
            VillageCode {
                name: "桥二居委会",
                code: "002",
            },
            VillageCode {
                name: "中二居委会",
                code: "003",
            },
            VillageCode {
                name: "中一居委会",
                code: "004",
            },
            VillageCode {
                name: "铁一居委会",
                code: "005",
            },
            VillageCode {
                name: "铁二居委会",
                code: "006",
            },
            VillageCode {
                name: "瞿南居委会",
                code: "007",
            },
            VillageCode {
                name: "瞿中居委会",
                code: "008",
            },
            VillageCode {
                name: "瞿东居委会",
                code: "009",
            },
            VillageCode {
                name: "瞿西居委会",
                code: "010",
            },
            VillageCode {
                name: "斜土居委会",
                code: "011",
            },
            VillageCode {
                name: "打浦居委会",
                code: "012",
            },
            VillageCode {
                name: "蒙自居委会",
                code: "013",
            },
            VillageCode {
                name: "龙华居委会",
                code: "014",
            },
            VillageCode {
                name: "丽园居委会",
                code: "015",
            },
            VillageCode {
                name: "桑城居委会",
                code: "016",
            },
            VillageCode {
                name: "紫荆居委会",
                code: "017",
            },
            VillageCode {
                name: "瑞南居委会",
                code: "018",
            },
            VillageCode {
                name: "海悦居委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "打浦桥街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "大同居委会",
                code: "001",
            },
            VillageCode {
                name: "肇东居委会",
                code: "002",
            },
            VillageCode {
                name: "泰康居委会",
                code: "003",
            },
            VillageCode {
                name: "建中居委会",
                code: "004",
            },
            VillageCode {
                name: "丽一居委会",
                code: "005",
            },
            VillageCode {
                name: "蒙西居委会",
                code: "006",
            },
            VillageCode {
                name: "丽二居委会",
                code: "007",
            },
            VillageCode {
                name: "银杏居委会",
                code: "008",
            },
            VillageCode {
                name: "局后居委会",
                code: "009",
            },
            VillageCode {
                name: "建一居委会",
                code: "010",
            },
            VillageCode {
                name: "建三居委会",
                code: "011",
            },
            VillageCode {
                name: "徐二居委会",
                code: "012",
            },
            VillageCode {
                name: "建五居委会",
                code: "013",
            },
            VillageCode {
                name: "锦海居委会",
                code: "014",
            },
            VillageCode {
                name: "南塘居委会",
                code: "015",
            },
            VillageCode {
                name: "汇龙居委会",
                code: "016",
            },
            VillageCode {
                name: "丽三居委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "淮海中路街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "新华居委会",
                code: "001",
            },
            VillageCode {
                name: "复三居委会",
                code: "002",
            },
            VillageCode {
                name: "复四居委会",
                code: "003",
            },
            VillageCode {
                name: "志成居委会",
                code: "004",
            },
            VillageCode {
                name: "大华居委会",
                code: "005",
            },
            VillageCode {
                name: "复兴居委会",
                code: "006",
            },
            VillageCode {
                name: "西成居委会",
                code: "007",
            },
            VillageCode {
                name: "孝和居委会",
                code: "008",
            },
            VillageCode {
                name: "瑞华居委会",
                code: "009",
            },
            VillageCode {
                name: "建六居委会",
                code: "010",
            },
            VillageCode {
                name: "新天地居委会",
                code: "011",
            },
            VillageCode {
                name: "顺昌居委会",
                code: "012",
            },
            VillageCode {
                name: "建东居委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "瑞金二路街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "雁荡居委会",
                code: "001",
            },
            VillageCode {
                name: "延中居委会",
                code: "002",
            },
            VillageCode {
                name: "巨鹿居委会",
                code: "003",
            },
            VillageCode {
                name: "锦江居委会",
                code: "004",
            },
            VillageCode {
                name: "南昌居委会",
                code: "005",
            },
            VillageCode {
                name: "茂名居委会",
                code: "006",
            },
            VillageCode {
                name: "淮中居委会",
                code: "007",
            },
            VillageCode {
                name: "长乐居委会",
                code: "008",
            },
            VillageCode {
                name: "香山居委会",
                code: "009",
            },
            VillageCode {
                name: "永嘉居委会",
                code: "010",
            },
            VillageCode {
                name: "思南居委会",
                code: "011",
            },
            VillageCode {
                name: "陕建居委会",
                code: "012",
            },
            VillageCode {
                name: "瑞成居委会",
                code: "013",
            },
            VillageCode {
                name: "瑞兴居委会",
                code: "014",
            },
            VillageCode {
                name: "建德居委会",
                code: "015",
            },
            VillageCode {
                name: "瑞雪居委会",
                code: "016",
            },
        ],
    },
];

static TOWNS_SJ_003: [TownCode; 14] = [
    TownCode {
        name: "天平路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "上海新村居委会",
                code: "001",
            },
            VillageCode {
                name: "庆余居委会",
                code: "002",
            },
            VillageCode {
                name: "康平居委会",
                code: "003",
            },
            VillageCode {
                name: "广元居委会",
                code: "004",
            },
            VillageCode {
                name: "安亭居委会",
                code: "005",
            },
            VillageCode {
                name: "吴兴居委会",
                code: "006",
            },
            VillageCode {
                name: "天平居委会",
                code: "007",
            },
            VillageCode {
                name: "宛平居委会",
                code: "008",
            },
            VillageCode {
                name: "高安居委会",
                code: "009",
            },
            VillageCode {
                name: "建新居委会",
                code: "010",
            },
            VillageCode {
                name: "永太居委会",
                code: "011",
            },
            VillageCode {
                name: "永嘉新村居委会",
                code: "012",
            },
            VillageCode {
                name: "桃源村居委会",
                code: "013",
            },
            VillageCode {
                name: "息村居委会",
                code: "014",
            },
            VillageCode {
                name: "肇嘉浜居委会",
                code: "015",
            },
            VillageCode {
                name: "慎成居委会",
                code: "016",
            },
            VillageCode {
                name: "嘉善居委会",
                code: "017",
            },
            VillageCode {
                name: "建岳居委会",
                code: "018",
            },
            VillageCode {
                name: "永康居委会",
                code: "019",
            },
            VillageCode {
                name: "太原居委会",
                code: "020",
            },
            VillageCode {
                name: "陕西居委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "湖南路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "淮中居委会",
                code: "001",
            },
            VillageCode {
                name: "安福居委会",
                code: "002",
            },
            VillageCode {
                name: "兴武居委会",
                code: "003",
            },
            VillageCode {
                name: "金波居委会",
                code: "004",
            },
            VillageCode {
                name: "武康居委会",
                code: "005",
            },
            VillageCode {
                name: "春华居委会",
                code: "006",
            },
            VillageCode {
                name: "华康居委会",
                code: "007",
            },
            VillageCode {
                name: "复永居委会",
                code: "008",
            },
            VillageCode {
                name: "新乐居委会",
                code: "009",
            },
            VillageCode {
                name: "张家弄居委会",
                code: "010",
            },
            VillageCode {
                name: "复襄居委会",
                code: "011",
            },
            VillageCode {
                name: "复中居委会",
                code: "012",
            },
            VillageCode {
                name: "东湖居委会",
                code: "013",
            },
            VillageCode {
                name: "陕新居委会",
                code: "014",
            },
            VillageCode {
                name: "淮海居委会",
                code: "015",
            },
            VillageCode {
                name: "延庆居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "斜土路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "日晖新城居委会",
                code: "001",
            },
            VillageCode {
                name: "日晖七村居委会",
                code: "002",
            },
            VillageCode {
                name: "日晖六村第一居委会",
                code: "003",
            },
            VillageCode {
                name: "日晖六村第二居委会",
                code: "004",
            },
            VillageCode {
                name: "康巨居委会",
                code: "005",
            },
            VillageCode {
                name: "江南新村居委会",
                code: "006",
            },
            VillageCode {
                name: "大木桥路第三居委会",
                code: "007",
            },
            VillageCode {
                name: "上影居委会",
                code: "008",
            },
            VillageCode {
                name: "茶陵居委会",
                code: "009",
            },
            VillageCode {
                name: "日晖二村居委会",
                code: "010",
            },
            VillageCode {
                name: "肇嘉浜路第一居委会",
                code: "011",
            },
            VillageCode {
                name: "肇清居委会",
                code: "012",
            },
            VillageCode {
                name: "景泰居委会",
                code: "013",
            },
            VillageCode {
                name: "大木桥路第四居委会",
                code: "014",
            },
            VillageCode {
                name: "大木桥路第五居委会",
                code: "015",
            },
            VillageCode {
                name: "日晖五村居委会",
                code: "016",
            },
            VillageCode {
                name: "恒益居委会",
                code: "017",
            },
            VillageCode {
                name: "尚海湾居委会",
                code: "018",
            },
            VillageCode {
                name: "医学院路居委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "枫林路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "张家浜居委会",
                code: "001",
            },
            VillageCode {
                name: "谨斜居委会",
                code: "002",
            },
            VillageCode {
                name: "东安一村北居委会",
                code: "003",
            },
            VillageCode {
                name: "东安一村南居委会",
                code: "004",
            },
            VillageCode {
                name: "黄家宅居委会",
                code: "005",
            },
            VillageCode {
                name: "西木北居委会",
                code: "006",
            },
            VillageCode {
                name: "西木南居委会",
                code: "007",
            },
            VillageCode {
                name: "沈家里居委会",
                code: "008",
            },
            VillageCode {
                name: "振兴居委会",
                code: "009",
            },
            VillageCode {
                name: "医清居委会",
                code: "010",
            },
            VillageCode {
                name: "东安四村居委会",
                code: "011",
            },
            VillageCode {
                name: "枫林新村居委会",
                code: "012",
            },
            VillageCode {
                name: "平江居委会",
                code: "013",
            },
            VillageCode {
                name: "庄家宅居委会",
                code: "014",
            },
            VillageCode {
                name: "宛南新村第一、二居委会",
                code: "015",
            },
            VillageCode {
                name: "宛南三村居委会",
                code: "016",
            },
            VillageCode {
                name: "宛南四村居委会",
                code: "017",
            },
            VillageCode {
                name: "宛南五村居委会",
                code: "018",
            },
            VillageCode {
                name: "宛南六村居委会",
                code: "019",
            },
            VillageCode {
                name: "天钥新村第一、二居委会",
                code: "020",
            },
            VillageCode {
                name: "天钥新村第三居委会",
                code: "021",
            },
            VillageCode {
                name: "天钥新村第四居委会",
                code: "022",
            },
            VillageCode {
                name: "龙山新村第一居委会",
                code: "023",
            },
            VillageCode {
                name: "龙山新村第二居委会",
                code: "024",
            },
            VillageCode {
                name: "张东居委会",
                code: "025",
            },
            VillageCode {
                name: "安康居委会",
                code: "026",
            },
            VillageCode {
                name: "四季园居委会",
                code: "027",
            },
            VillageCode {
                name: "汇园居委会",
                code: "028",
            },
            VillageCode {
                name: "东安二村居委会",
                code: "029",
            },
            VillageCode {
                name: "徐汇苑居委会",
                code: "030",
            },
            VillageCode {
                name: "东安苑居委会",
                code: "031",
            },
            VillageCode {
                name: "南康居委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "长桥街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "长桥新村第一居委会",
                code: "001",
            },
            VillageCode {
                name: "长桥新二村居委会",
                code: "002",
            },
            VillageCode {
                name: "港口居委会",
                code: "003",
            },
            VillageCode {
                name: "长桥一村居委会",
                code: "004",
            },
            VillageCode {
                name: "长桥三村第一居委会",
                code: "005",
            },
            VillageCode {
                name: "长桥三村第二居委会",
                code: "006",
            },
            VillageCode {
                name: "长桥五村居委会",
                code: "007",
            },
            VillageCode {
                name: "长桥七村居委会",
                code: "008",
            },
            VillageCode {
                name: "汇成一村居委会",
                code: "009",
            },
            VillageCode {
                name: "汇成二村居委会",
                code: "010",
            },
            VillageCode {
                name: "汇成三村居委会",
                code: "011",
            },
            VillageCode {
                name: "汇成四村居委会",
                code: "012",
            },
            VillageCode {
                name: "汇成五村居委会",
                code: "013",
            },
            VillageCode {
                name: "园南二村居委会",
                code: "014",
            },
            VillageCode {
                name: "园南三村居委会",
                code: "015",
            },
            VillageCode {
                name: "光华居委会",
                code: "016",
            },
            VillageCode {
                name: "百龙居委会",
                code: "017",
            },
            VillageCode {
                name: "园南一村居委会",
                code: "018",
            },
            VillageCode {
                name: "罗秀三村居委会",
                code: "019",
            },
            VillageCode {
                name: "楼园居委会",
                code: "020",
            },
            VillageCode {
                name: "长桥八村居委会",
                code: "021",
            },
            VillageCode {
                name: "平福居委会",
                code: "022",
            },
            VillageCode {
                name: "体育花苑居委会",
                code: "023",
            },
            VillageCode {
                name: "华东花苑第一居委会",
                code: "024",
            },
            VillageCode {
                name: "华东花苑第二居委会",
                code: "025",
            },
            VillageCode {
                name: "汇澜园居委会",
                code: "026",
            },
            VillageCode {
                name: "徐汇新城居委会",
                code: "027",
            },
            VillageCode {
                name: "中海瀛台居委会",
                code: "028",
            },
            VillageCode {
                name: "华滨居委会",
                code: "029",
            },
            VillageCode {
                name: "华沁居委会",
                code: "030",
            },
            VillageCode {
                name: "长桥四村居委会",
                code: "031",
            },
            VillageCode {
                name: "罗秀居委会",
                code: "032",
            },
            VillageCode {
                name: "罗秀二村居委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "田林街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "田林一、二村居委会",
                code: "001",
            },
            VillageCode {
                name: "田林三、四村居委会",
                code: "002",
            },
            VillageCode {
                name: "田林十二村居委会",
                code: "003",
            },
            VillageCode {
                name: "锦馨苑居委会",
                code: "004",
            },
            VillageCode {
                name: "新苑第一、二居委会",
                code: "005",
            },
            VillageCode {
                name: "新苑第三、四居委会",
                code: "006",
            },
            VillageCode {
                name: "田林十三村居委会",
                code: "007",
            },
            VillageCode {
                name: "千鹤第三居委会",
                code: "008",
            },
            VillageCode {
                name: "千鹤第五居委会",
                code: "009",
            },
            VillageCode {
                name: "万科华尔兹居委会",
                code: "010",
            },
            VillageCode {
                name: "古宜居委会",
                code: "011",
            },
            VillageCode {
                name: "田林十一村居委会",
                code: "012",
            },
            VillageCode {
                name: "华鼎居委会",
                code: "013",
            },
            VillageCode {
                name: "田林五、六、七村居委会",
                code: "014",
            },
            VillageCode {
                name: "田林八、九、十村居委会",
                code: "015",
            },
            VillageCode {
                name: "新苑第五、六、七居委会",
                code: "016",
            },
            VillageCode {
                name: "长春居委会",
                code: "017",
            },
            VillageCode {
                name: "爱建园居委会",
                code: "018",
            },
            VillageCode {
                name: "小安桥居委会",
                code: "019",
            },
            VillageCode {
                name: "千鹤六居委会",
                code: "020",
            },
            VillageCode {
                name: "尚汇豪庭居委会",
                code: "021",
            },
            VillageCode {
                name: "吴中居委会",
                code: "022",
            },
            VillageCode {
                name: "千鹤第一居委会",
                code: "023",
            },
            VillageCode {
                name: "千鹤第二居委会",
                code: "024",
            },
            VillageCode {
                name: "田林十四村第一居委会",
                code: "025",
            },
            VillageCode {
                name: "田林十四村第二居委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "虹梅路街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "东兰路居委会",
                code: "001",
            },
            VillageCode {
                name: "古美路第一居委会",
                code: "002",
            },
            VillageCode {
                name: "古美路第二居委会",
                code: "003",
            },
            VillageCode {
                name: "钦北居委会",
                code: "004",
            },
            VillageCode {
                name: "古美路第三居委会",
                code: "005",
            },
            VillageCode {
                name: "古美路第四居委会",
                code: "006",
            },
            VillageCode {
                name: "桂林苑居委会",
                code: "007",
            },
            VillageCode {
                name: "虹南居委会",
                code: "008",
            },
            VillageCode {
                name: "虹星居委会",
                code: "009",
            },
            VillageCode {
                name: "航天新苑居委会",
                code: "010",
            },
            VillageCode {
                name: "怡桂苑居委会",
                code: "011",
            },
            VillageCode {
                name: "永兆居委会",
                code: "012",
            },
            VillageCode {
                name: "华悦家园居委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "康健新村街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "寿昌山居委会",
                code: "001",
            },
            VillageCode {
                name: "长虹坊居委会",
                code: "002",
            },
            VillageCode {
                name: "紫鹃园居委会",
                code: "003",
            },
            VillageCode {
                name: "玉兰园居委会",
                code: "004",
            },
            VillageCode {
                name: "康乐小区居委会",
                code: "005",
            },
            VillageCode {
                name: "长青坊居委会",
                code: "006",
            },
            VillageCode {
                name: "樱花园居委会",
                code: "007",
            },
            VillageCode {
                name: "寿祥坊居委会",
                code: "008",
            },
            VillageCode {
                name: "紫薇园居委会",
                code: "009",
            },
            VillageCode {
                name: "茶花桂花居委会",
                code: "010",
            },
            VillageCode {
                name: "长顺海居委会",
                code: "011",
            },
            VillageCode {
                name: "寿益坊居委会",
                code: "012",
            },
            VillageCode {
                name: "月季百藤居委会",
                code: "013",
            },
            VillageCode {
                name: "丁香迎春居委会",
                code: "014",
            },
            VillageCode {
                name: "师大新村居委会",
                code: "015",
            },
            VillageCode {
                name: "桂林路第二居委会",
                code: "016",
            },
            VillageCode {
                name: "桂康居委会",
                code: "017",
            },
            VillageCode {
                name: "欣园居委会",
                code: "018",
            },
            VillageCode {
                name: "联莘居委会",
                code: "019",
            },
            VillageCode {
                name: "金桂苑居委会",
                code: "020",
            },
            VillageCode {
                name: "长丰坊居委会",
                code: "021",
            },
            VillageCode {
                name: "长兴坊居委会",
                code: "022",
            },
            VillageCode {
                name: "紫荆党校居委会",
                code: "023",
            },
            VillageCode {
                name: "康宁馨居委会",
                code: "024",
            },
            VillageCode {
                name: "康强海居委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "徐家汇街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "东塘居委会",
                code: "001",
            },
            VillageCode {
                name: "番禺居委会",
                code: "002",
            },
            VillageCode {
                name: "柿子湾居委会",
                code: "003",
            },
            VillageCode {
                name: "南丹居委会",
                code: "004",
            },
            VillageCode {
                name: "虹交居委会",
                code: "005",
            },
            VillageCode {
                name: "西塘居委会",
                code: "006",
            },
            VillageCode {
                name: "乐山一村居委会",
                code: "007",
            },
            VillageCode {
                name: "乐山二、三村居委会",
                code: "008",
            },
            VillageCode {
                name: "乐山四、五村居委会",
                code: "009",
            },
            VillageCode {
                name: "乐山六、七村居委会",
                code: "010",
            },
            VillageCode {
                name: "乐山八、九村居委会",
                code: "011",
            },
            VillageCode {
                name: "虹二居委会",
                code: "012",
            },
            VillageCode {
                name: "交大新村居委会",
                code: "013",
            },
            VillageCode {
                name: "潘家宅居委会",
                code: "014",
            },
            VillageCode {
                name: "汇站居委会",
                code: "015",
            },
            VillageCode {
                name: "文定居委会",
                code: "016",
            },
            VillageCode {
                name: "陈家宅居委会",
                code: "017",
            },
            VillageCode {
                name: "沈马居委会",
                code: "018",
            },
            VillageCode {
                name: "零陵居委会",
                code: "019",
            },
            VillageCode {
                name: "启明居委会",
                code: "020",
            },
            VillageCode {
                name: "南赵巷居委会",
                code: "021",
            },
            VillageCode {
                name: "王家堂居委会",
                code: "022",
            },
            VillageCode {
                name: "殷家角居委会",
                code: "023",
            },
            VillageCode {
                name: "爱华居委会",
                code: "024",
            },
            VillageCode {
                name: "科汇居委会",
                code: "025",
            },
            VillageCode {
                name: "徐汇新村居委会",
                code: "026",
            },
            VillageCode {
                name: "汇翠居委会",
                code: "027",
            },
            VillageCode {
                name: "名园居委会",
                code: "028",
            },
            VillageCode {
                name: "豪庭居委会",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "凌云路街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "梅陇七村居委会",
                code: "001",
            },
            VillageCode {
                name: "梅陇八村居委会",
                code: "002",
            },
            VillageCode {
                name: "梅陇九村居委会",
                code: "003",
            },
            VillageCode {
                name: "梅陇十村居委会",
                code: "004",
            },
            VillageCode {
                name: "梅陇十一村第一居委会",
                code: "005",
            },
            VillageCode {
                name: "梅陇十一村第二居委会",
                code: "006",
            },
            VillageCode {
                name: "和平居委会",
                code: "007",
            },
            VillageCode {
                name: "陇南居委会",
                code: "008",
            },
            VillageCode {
                name: "梅苑第一居委会",
                code: "009",
            },
            VillageCode {
                name: "梅苑第二居委会",
                code: "010",
            },
            VillageCode {
                name: "龙州居委会",
                code: "011",
            },
            VillageCode {
                name: "闵朱居委会",
                code: "012",
            },
            VillageCode {
                name: "金塘居委会",
                code: "013",
            },
            VillageCode {
                name: "华东理工大学第一居委会",
                code: "014",
            },
            VillageCode {
                name: "华东理工大学第二居委会",
                code: "015",
            },
            VillageCode {
                name: "舒乐居委会",
                code: "016",
            },
            VillageCode {
                name: "家乐苑居委会",
                code: "017",
            },
            VillageCode {
                name: "长陇苑居委会",
                code: "018",
            },
            VillageCode {
                name: "闵秀居委会",
                code: "019",
            },
            VillageCode {
                name: "兴荣苑居委会",
                code: "020",
            },
            VillageCode {
                name: "书香苑居委会",
                code: "021",
            },
            VillageCode {
                name: "华理苑居委会",
                code: "022",
            },
            VillageCode {
                name: "梅陇三村居委会",
                code: "023",
            },
            VillageCode {
                name: "梅陇四村居委会",
                code: "024",
            },
            VillageCode {
                name: "梅陇五村居委会",
                code: "025",
            },
            VillageCode {
                name: "梅陇六村居委会",
                code: "026",
            },
            VillageCode {
                name: "凌云新村居委会",
                code: "027",
            },
            VillageCode {
                name: "阳光居委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "龙华街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "周家湾居委会",
                code: "001",
            },
            VillageCode {
                name: "龙华新村居委会",
                code: "002",
            },
            VillageCode {
                name: "上缝新村居委会",
                code: "003",
            },
            VillageCode {
                name: "俞家湾第二居委会",
                code: "004",
            },
            VillageCode {
                name: "俞家湾第一居委会",
                code: "005",
            },
            VillageCode {
                name: "龙南三、四村居委会",
                code: "006",
            },
            VillageCode {
                name: "龙南七村居委会",
                code: "007",
            },
            VillageCode {
                name: "丰谷路居委会",
                code: "008",
            },
            VillageCode {
                name: "丰谷路第三居委会",
                code: "009",
            },
            VillageCode {
                name: "东蔡居委会",
                code: "010",
            },
            VillageCode {
                name: "龙南第五居委会",
                code: "011",
            },
            VillageCode {
                name: "龙南第六居委会",
                code: "012",
            },
            VillageCode {
                name: "俞家湾第三居委会",
                code: "013",
            },
            VillageCode {
                name: "机场新村居委会",
                code: "014",
            },
            VillageCode {
                name: "狮城花苑居委会",
                code: "015",
            },
            VillageCode {
                name: "云锦路居委会",
                code: "016",
            },
            VillageCode {
                name: "滨江居委会",
                code: "017",
            },
            VillageCode {
                name: "民苑居委会",
                code: "018",
            },
            VillageCode {
                name: "华容路居委会",
                code: "019",
            },
            VillageCode {
                name: "汇龙居委会",
                code: "020",
            },
            VillageCode {
                name: "盛大花苑第一居委会",
                code: "021",
            },
            VillageCode {
                name: "强生花苑居委会",
                code: "022",
            },
            VillageCode {
                name: "樟树苑居委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "漕河泾街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "习勤居委会",
                code: "001",
            },
            VillageCode {
                name: "漕溪四村居委会",
                code: "002",
            },
            VillageCode {
                name: "九弄居委会",
                code: "003",
            },
            VillageCode {
                name: "科苑居委会",
                code: "004",
            },
            VillageCode {
                name: "冠生园居委会",
                code: "005",
            },
            VillageCode {
                name: "凯翔居委会",
                code: "006",
            },
            VillageCode {
                name: "薛家宅居委会",
                code: "007",
            },
            VillageCode {
                name: "康健路居委会",
                code: "008",
            },
            VillageCode {
                name: "金谷园居委会",
                code: "009",
            },
            VillageCode {
                name: "龙漕居委会",
                code: "010",
            },
            VillageCode {
                name: "南一村第一居委会",
                code: "011",
            },
            VillageCode {
                name: "南一村第二居委会",
                code: "012",
            },
            VillageCode {
                name: "中海馨园居委会",
                code: "013",
            },
            VillageCode {
                name: "宏润花园居委会",
                code: "014",
            },
            VillageCode {
                name: "梓树园居委会",
                code: "015",
            },
            VillageCode {
                name: "佳友居委会",
                code: "016",
            },
            VillageCode {
                name: "华富居委会",
                code: "017",
            },
            VillageCode {
                name: "挹翠苑居委会",
                code: "018",
            },
            VillageCode {
                name: "张家园居委会",
                code: "019",
            },
            VillageCode {
                name: "东泉路居委会",
                code: "020",
            },
            VillageCode {
                name: "东荡居委会",
                code: "021",
            },
            VillageCode {
                name: "金牛居委会",
                code: "022",
            },
            VillageCode {
                name: "罗城居委会",
                code: "023",
            },
            VillageCode {
                name: "正南花苑居委会",
                code: "024",
            },
            VillageCode {
                name: "嘉萱苑居委会",
                code: "025",
            },
            VillageCode {
                name: "宾阳路居委会",
                code: "026",
            },
            VillageCode {
                name: "漕东居委会",
                code: "027",
            },
            VillageCode {
                name: "月河居委会",
                code: "028",
            },
            VillageCode {
                name: "馨汇南苑居委会",
                code: "029",
            },
            VillageCode {
                name: "公园道居委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "华泾镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "华建居委会",
                code: "001",
            },
            VillageCode {
                name: "大桥居委会",
                code: "002",
            },
            VillageCode {
                name: "华泾四村居委会",
                code: "003",
            },
            VillageCode {
                name: "华发居委会",
                code: "004",
            },
            VillageCode {
                name: "华阳居委会",
                code: "005",
            },
            VillageCode {
                name: "沙家浜居委会",
                code: "006",
            },
            VillageCode {
                name: "名苑居委会",
                code: "007",
            },
            VillageCode {
                name: "华泾五村居委会",
                code: "008",
            },
            VillageCode {
                name: "光华绿苑居委会",
                code: "009",
            },
            VillageCode {
                name: "华欣家园居委会",
                code: "010",
            },
            VillageCode {
                name: "明丰居委会",
                code: "011",
            },
            VillageCode {
                name: "华臻居委会",
                code: "012",
            },
            VillageCode {
                name: "漓江山水居委会",
                code: "013",
            },
            VillageCode {
                name: "华泾绿苑居委会",
                code: "014",
            },
            VillageCode {
                name: "馨宁居委会",
                code: "015",
            },
            VillageCode {
                name: "印象旭辉居委会",
                code: "016",
            },
            VillageCode {
                name: "盛华景苑居委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "漕河泾新兴技术开发区",
        code: "014",
        villages: &[VillageCode {
            name: "漕河泾新兴技术开发区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_SJ_004: [TownCode; 10] = [
    TownCode {
        name: "华阳路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "华一居委会",
                code: "001",
            },
            VillageCode {
                name: "华二居委会",
                code: "002",
            },
            VillageCode {
                name: "华三居委会",
                code: "003",
            },
            VillageCode {
                name: "华四居委会",
                code: "004",
            },
            VillageCode {
                name: "华五居委会",
                code: "005",
            },
            VillageCode {
                name: "华院居委会",
                code: "006",
            },
            VillageCode {
                name: "秀水居委会",
                code: "007",
            },
            VillageCode {
                name: "长一居委会",
                code: "008",
            },
            VillageCode {
                name: "长支二居委会",
                code: "009",
            },
            VillageCode {
                name: "长二居委会",
                code: "010",
            },
            VillageCode {
                name: "苏一居委会",
                code: "011",
            },
            VillageCode {
                name: "潘东居委会",
                code: "012",
            },
            VillageCode {
                name: "潘西居委会",
                code: "013",
            },
            VillageCode {
                name: "陶家宅居委会",
                code: "014",
            },
            VillageCode {
                name: "徐家宅居委会",
                code: "015",
            },
            VillageCode {
                name: "姚家角居委会",
                code: "016",
            },
            VillageCode {
                name: "建宁居委会",
                code: "017",
            },
            VillageCode {
                name: "潘中居委会",
                code: "018",
            },
            VillageCode {
                name: "天诚居委会",
                code: "019",
            },
            VillageCode {
                name: "飞乐居委会",
                code: "020",
            },
            VillageCode {
                name: "西一居委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "江苏路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "岐山居委会",
                code: "001",
            },
            VillageCode {
                name: "江苏居委会",
                code: "002",
            },
            VillageCode {
                name: "万村居委会",
                code: "003",
            },
            VillageCode {
                name: "南汪居委会",
                code: "004",
            },
            VillageCode {
                name: "东浜居委会",
                code: "005",
            },
            VillageCode {
                name: "愚三居委会",
                code: "006",
            },
            VillageCode {
                name: "曹家堰居委会",
                code: "007",
            },
            VillageCode {
                name: "西浜居委会",
                code: "008",
            },
            VillageCode {
                name: "福世居委会",
                code: "009",
            },
            VillageCode {
                name: "长新居委会",
                code: "010",
            },
            VillageCode {
                name: "华山居委会",
                code: "011",
            },
            VillageCode {
                name: "利西居委会",
                code: "012",
            },
            VillageCode {
                name: "北汪居委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "新华路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "番禺居委会",
                code: "001",
            },
            VillageCode {
                name: "牛桥居委会",
                code: "002",
            },
            VillageCode {
                name: "人民居委会",
                code: "003",
            },
            VillageCode {
                name: "西镇居委会",
                code: "004",
            },
            VillageCode {
                name: "杨宅路居委会",
                code: "005",
            },
            VillageCode {
                name: "香花居委会",
                code: "006",
            },
            VillageCode {
                name: "新华居委会",
                code: "007",
            },
            VillageCode {
                name: "梅安居委会",
                code: "008",
            },
            VillageCode {
                name: "左家宅居委会",
                code: "009",
            },
            VillageCode {
                name: "田渡居委会",
                code: "010",
            },
            VillageCode {
                name: "泰安居委会",
                code: "011",
            },
            VillageCode {
                name: "陈家巷居委会",
                code: "012",
            },
            VillageCode {
                name: "东镇居委会",
                code: "013",
            },
            VillageCode {
                name: "红庄居委会",
                code: "014",
            },
            VillageCode {
                name: "幸福居委会",
                code: "015",
            },
            VillageCode {
                name: "和平居委会",
                code: "016",
            },
            VillageCode {
                name: "张家宅居委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "周家桥街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "范北居委会",
                code: "001",
            },
            VillageCode {
                name: "三泾北宅居委会",
                code: "002",
            },
            VillageCode {
                name: "三泾南宅居委会",
                code: "003",
            },
            VillageCode {
                name: "武夷居委会",
                code: "004",
            },
            VillageCode {
                name: "沈家郎居委会",
                code: "005",
            },
            VillageCode {
                name: "周一居委会",
                code: "006",
            },
            VillageCode {
                name: "周二居委会",
                code: "007",
            },
            VillageCode {
                name: "杨家宅居委会",
                code: "008",
            },
            VillageCode {
                name: "锦屏居委会",
                code: "009",
            },
            VillageCode {
                name: "古南居委会",
                code: "010",
            },
            VillageCode {
                name: "上海花城居委会",
                code: "011",
            },
            VillageCode {
                name: "虹桥新城居委会",
                code: "012",
            },
            VillageCode {
                name: "长宁新城居委会",
                code: "013",
            },
            VillageCode {
                name: "中山公寓居委会",
                code: "014",
            },
            VillageCode {
                name: "大家源居委会",
                code: "015",
            },
            VillageCode {
                name: "虹桥万博花园居委会",
                code: "016",
            },
            VillageCode {
                name: "春天花园居委会",
                code: "017",
            },
            VillageCode {
                name: "天山河畔花园居委会",
                code: "018",
            },
            VillageCode {
                name: "仁恒河滨花园居委会",
                code: "019",
            },
            VillageCode {
                name: "天山华庭居委会",
                code: "020",
            },
            VillageCode {
                name: "新天地河滨花园居委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "天山路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "天山二村居委会",
                code: "001",
            },
            VillageCode {
                name: "天山三村居委会",
                code: "002",
            },
            VillageCode {
                name: "天山四村居委会",
                code: "003",
            },
            VillageCode {
                name: "茅台居委会",
                code: "004",
            },
            VillageCode {
                name: "仙霞居委会",
                code: "005",
            },
            VillageCode {
                name: "天山居委会",
                code: "006",
            },
            VillageCode {
                name: "天义居委会",
                code: "007",
            },
            VillageCode {
                name: "新光居委会",
                code: "008",
            },
            VillageCode {
                name: "新风居委会",
                code: "009",
            },
            VillageCode {
                name: "玉屏居委会",
                code: "010",
            },
            VillageCode {
                name: "遵义居委会",
                code: "011",
            },
            VillageCode {
                name: "友谊居委会",
                code: "012",
            },
            VillageCode {
                name: "紫云居委会",
                code: "013",
            },
            VillageCode {
                name: "联建居委会",
                code: "014",
            },
            VillageCode {
                name: "中紫居委会",
                code: "015",
            },
            VillageCode {
                name: "延西居委会",
                code: "016",
            },
            VillageCode {
                name: "纺大一村居委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "仙霞新村街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "天山五村第一居委会",
                code: "001",
            },
            VillageCode {
                name: "天山五村第三居委会",
                code: "002",
            },
            VillageCode {
                name: "芙蓉江路第一居委会",
                code: "003",
            },
            VillageCode {
                name: "芙蓉江路第二居委会",
                code: "004",
            },
            VillageCode {
                name: "大金更居委会",
                code: "005",
            },
            VillageCode {
                name: "天原二村居委会",
                code: "006",
            },
            VillageCode {
                name: "虹日居委会",
                code: "007",
            },
            VillageCode {
                name: "虹古居委会",
                code: "008",
            },
            VillageCode {
                name: "虹纺居委会",
                code: "009",
            },
            VillageCode {
                name: "虹霞居委会",
                code: "010",
            },
            VillageCode {
                name: "虹旭居委会",
                code: "011",
            },
            VillageCode {
                name: "虹仙居委会",
                code: "012",
            },
            VillageCode {
                name: "锦苑居委会",
                code: "013",
            },
            VillageCode {
                name: "仙逸居委会",
                code: "014",
            },
            VillageCode {
                name: "仙霞路第二居委会",
                code: "015",
            },
            VillageCode {
                name: "杜一居委会",
                code: "016",
            },
            VillageCode {
                name: "水霞居委会",
                code: "017",
            },
            VillageCode {
                name: "古宋居委会",
                code: "018",
            },
            VillageCode {
                name: "茅台新苑居委会",
                code: "019",
            },
            VillageCode {
                name: "威宁小区居委会",
                code: "020",
            },
            VillageCode {
                name: "虹景居委会",
                code: "021",
            },
            VillageCode {
                name: "茅台花苑居委会",
                code: "022",
            },
            VillageCode {
                name: "安龙居委会",
                code: "023",
            },
            VillageCode {
                name: "天山五村第二居委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "虹桥街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "安东居委会",
                code: "001",
            },
            VillageCode {
                name: "何家角居委会",
                code: "002",
            },
            VillageCode {
                name: "长顺居委会",
                code: "003",
            },
            VillageCode {
                name: "新顺居委会",
                code: "004",
            },
            VillageCode {
                name: "长虹居委会",
                code: "005",
            },
            VillageCode {
                name: "虹南居委会",
                code: "006",
            },
            VillageCode {
                name: "虹桥居委会",
                code: "007",
            },
            VillageCode {
                name: "爱建居委会",
                code: "008",
            },
            VillageCode {
                name: "伊犁居委会",
                code: "009",
            },
            VillageCode {
                name: "虹储居委会",
                code: "010",
            },
            VillageCode {
                name: "虹许居委会",
                code: "011",
            },
            VillageCode {
                name: "虹梅居委会",
                code: "012",
            },
            VillageCode {
                name: "虹东居委会",
                code: "013",
            },
            VillageCode {
                name: "中山居委会",
                code: "014",
            },
            VillageCode {
                name: "虹欣居委会",
                code: "015",
            },
            VillageCode {
                name: "古北荣华第一居委会",
                code: "016",
            },
            VillageCode {
                name: "古北荣华第二居委会",
                code: "017",
            },
            VillageCode {
                name: "古北荣华第三居委会",
                code: "018",
            },
            VillageCode {
                name: "古北荣华第四居委会",
                code: "019",
            },
            VillageCode {
                name: "虹桥经济技术开发区居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "程家桥街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "程桥一村居委会",
                code: "001",
            },
            VillageCode {
                name: "程桥二村居委会",
                code: "002",
            },
            VillageCode {
                name: "宝北居委会",
                code: "003",
            },
            VillageCode {
                name: "虹桥机场新村居委会",
                code: "004",
            },
            VillageCode {
                name: "王满泗桥居委会",
                code: "005",
            },
            VillageCode {
                name: "南龚居委会",
                code: "006",
            },
            VillageCode {
                name: "上航新村居委会",
                code: "007",
            },
            VillageCode {
                name: "嘉利豪园居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "北新泾街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "新泾一村第一居委会",
                code: "001",
            },
            VillageCode {
                name: "新泾一村第二居委会",
                code: "002",
            },
            VillageCode {
                name: "新泾二村居委会",
                code: "003",
            },
            VillageCode {
                name: "新泾三村居委会",
                code: "004",
            },
            VillageCode {
                name: "新泾四村居委会",
                code: "005",
            },
            VillageCode {
                name: "新泾五村居委会",
                code: "006",
            },
            VillageCode {
                name: "新泾六村居委会",
                code: "007",
            },
            VillageCode {
                name: "新泾七村居委会",
                code: "008",
            },
            VillageCode {
                name: "新泾八村居委会",
                code: "009",
            },
            VillageCode {
                name: "北翟居委会",
                code: "010",
            },
            VillageCode {
                name: "蒲淞北路居委会",
                code: "011",
            },
            VillageCode {
                name: "哈密新村居委会",
                code: "012",
            },
            VillageCode {
                name: "元丰花园居委会",
                code: "013",
            },
            VillageCode {
                name: "金平居委会",
                code: "014",
            },
            VillageCode {
                name: "剑河家苑居委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "新泾镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "刘家宅小区第一居委会",
                code: "001",
            },
            VillageCode {
                name: "刘家宅小区第二居委会",
                code: "002",
            },
            VillageCode {
                name: "刘家宅小区第三居委会",
                code: "003",
            },
            VillageCode {
                name: "刘家宅小区第四居委会",
                code: "004",
            },
            VillageCode {
                name: "福泉路居委会",
                code: "005",
            },
            VillageCode {
                name: "淮阴路居委会",
                code: "006",
            },
            VillageCode {
                name: "淞虹小区第二居委会",
                code: "007",
            },
            VillageCode {
                name: "淞虹小区第三居委会",
                code: "008",
            },
            VillageCode {
                name: "淞虹小区第四居委会",
                code: "009",
            },
            VillageCode {
                name: "淞虹小区第五居委会",
                code: "010",
            },
            VillageCode {
                name: "华松居委会",
                code: "011",
            },
            VillageCode {
                name: "曙光居委会",
                code: "012",
            },
            VillageCode {
                name: "屈家桥居委会",
                code: "013",
            },
            VillageCode {
                name: "绿园一村居委会",
                code: "014",
            },
            VillageCode {
                name: "绿园新村第五居委会",
                code: "015",
            },
            VillageCode {
                name: "绿园新村第八居委会",
                code: "016",
            },
            VillageCode {
                name: "绿园新村第十一居委会",
                code: "017",
            },
            VillageCode {
                name: "绿园新村第十二居委会",
                code: "018",
            },
            VillageCode {
                name: "北虹居委会",
                code: "019",
            },
            VillageCode {
                name: "程桥居委会",
                code: "020",
            },
            VillageCode {
                name: "中泾居委会",
                code: "021",
            },
            VillageCode {
                name: "林泉路居委会",
                code: "022",
            },
            VillageCode {
                name: "双流路居委会",
                code: "023",
            },
            VillageCode {
                name: "新渔东路居委会",
                code: "024",
            },
            VillageCode {
                name: "定威路居委会",
                code: "025",
            },
            VillageCode {
                name: "虹园新村第九居委会",
                code: "026",
            },
            VillageCode {
                name: "平塘居委会",
                code: "027",
            },
            VillageCode {
                name: "虹康居委会",
                code: "028",
            },
            VillageCode {
                name: "南洋新都居委会",
                code: "029",
            },
            VillageCode {
                name: "新泾居委会",
                code: "030",
            },
            VillageCode {
                name: "怡景苑居委会",
                code: "031",
            },
            VillageCode {
                name: "天山星城居委会",
                code: "032",
            },
            VillageCode {
                name: "新泾北苑居委会",
                code: "033",
            },
        ],
    },
];

static TOWNS_SJ_005: [TownCode; 14] = [
    TownCode {
        name: "江宁路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "新安居委会",
                code: "001",
            },
            VillageCode {
                name: "永乐居委会",
                code: "002",
            },
            VillageCode {
                name: "联宝里居委会",
                code: "003",
            },
            VillageCode {
                name: "恒德里居委会",
                code: "004",
            },
            VillageCode {
                name: "景苑居委会",
                code: "005",
            },
            VillageCode {
                name: "又一村居委会",
                code: "006",
            },
            VillageCode {
                name: "北京居委会",
                code: "007",
            },
            VillageCode {
                name: "众乐里居委会",
                code: "008",
            },
            VillageCode {
                name: "武定坊居委会",
                code: "009",
            },
            VillageCode {
                name: "三星坊居委会",
                code: "010",
            },
            VillageCode {
                name: "蒋家巷居委会",
                code: "011",
            },
            VillageCode {
                name: "海防村居委会",
                code: "012",
            },
            VillageCode {
                name: "通安里居委会",
                code: "013",
            },
            VillageCode {
                name: "三乐里居委会",
                code: "014",
            },
            VillageCode {
                name: "句容里居委会",
                code: "015",
            },
            VillageCode {
                name: "天河居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "石门二路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "奉贤居委会",
                code: "001",
            },
            VillageCode {
                name: "东王居委会",
                code: "002",
            },
            VillageCode {
                name: "新德居委会",
                code: "003",
            },
            VillageCode {
                name: "郑家巷居委会",
                code: "004",
            },
            VillageCode {
                name: "张家宅居委会",
                code: "005",
            },
            VillageCode {
                name: "祥福居委会",
                code: "006",
            },
            VillageCode {
                name: "达安城居委会",
                code: "007",
            },
            VillageCode {
                name: "新福康里居委会",
                code: "008",
            },
            VillageCode {
                name: "恒丰居委会",
                code: "009",
            },
            VillageCode {
                name: "华沁居委会",
                code: "010",
            },
            VillageCode {
                name: "斯文里居委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "南京西路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "延中居委会",
                code: "001",
            },
            VillageCode {
                name: "联华居委会",
                code: "002",
            },
            VillageCode {
                name: "华业居委会",
                code: "003",
            },
            VillageCode {
                name: "陕南居委会",
                code: "004",
            },
            VillageCode {
                name: "古柏居委会",
                code: "005",
            },
            VillageCode {
                name: "陕北居委会",
                code: "006",
            },
            VillageCode {
                name: "新成居委会",
                code: "007",
            },
            VillageCode {
                name: "中凯居委会",
                code: "008",
            },
            VillageCode {
                name: "重华居委会",
                code: "009",
            },
            VillageCode {
                name: "茂北居委会",
                code: "010",
            },
            VillageCode {
                name: "升平居委会",
                code: "011",
            },
            VillageCode {
                name: "威海居委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "静安寺街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "愚谷村居委会",
                code: "001",
            },
            VillageCode {
                name: "四明居委会",
                code: "002",
            },
            VillageCode {
                name: "静安居委会",
                code: "003",
            },
            VillageCode {
                name: "三义坊居委会",
                code: "004",
            },
            VillageCode {
                name: "百乐居委会",
                code: "005",
            },
            VillageCode {
                name: "景华居委会",
                code: "006",
            },
            VillageCode {
                name: "华山居委会",
                code: "007",
            },
            VillageCode {
                name: "海园居委会",
                code: "008",
            },
            VillageCode {
                name: "美丽园居委会",
                code: "009",
            },
            VillageCode {
                name: "裕华居委会",
                code: "010",
            },
            VillageCode {
                name: "嘉园居委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "曹家渡街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "叶庆居委会",
                code: "001",
            },
            VillageCode {
                name: "武南居委会",
                code: "002",
            },
            VillageCode {
                name: "中行别业居委会",
                code: "003",
            },
            VillageCode {
                name: "高荣居委会",
                code: "004",
            },
            VillageCode {
                name: "万航居委会",
                code: "005",
            },
            VillageCode {
                name: "武西居委会",
                code: "006",
            },
            VillageCode {
                name: "长春居委会",
                code: "007",
            },
            VillageCode {
                name: "玉兰村居委会",
                code: "008",
            },
            VillageCode {
                name: "姚西居委会",
                code: "009",
            },
            VillageCode {
                name: "康定居委会",
                code: "010",
            },
            VillageCode {
                name: "四和花园居委会",
                code: "011",
            },
            VillageCode {
                name: "三和花园居委会",
                code: "012",
            },
            VillageCode {
                name: "均泰居委会",
                code: "013",
            },
            VillageCode {
                name: "达安花园居委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "天目西路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "蕃瓜弄居委会",
                code: "001",
            },
            VillageCode {
                name: "卓悦居居委会",
                code: "002",
            },
            VillageCode {
                name: "华康居委会",
                code: "003",
            },
            VillageCode {
                name: "新桥居委会",
                code: "004",
            },
            VillageCode {
                name: "华新居委会",
                code: "005",
            },
            VillageCode {
                name: "地梨港居委会",
                code: "006",
            },
            VillageCode {
                name: "华丰居委会",
                code: "007",
            },
            VillageCode {
                name: "普善居委会",
                code: "008",
            },
            VillageCode {
                name: "铁路新村居委会",
                code: "009",
            },
            VillageCode {
                name: "和泰花苑居委会",
                code: "010",
            },
            VillageCode {
                name: "安源居委会",
                code: "011",
            },
            VillageCode {
                name: "河滨融景居委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "北站街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "新泰居委会",
                code: "001",
            },
            VillageCode {
                name: "顺庆里居委会",
                code: "002",
            },
            VillageCode {
                name: "文安居委会",
                code: "003",
            },
            VillageCode {
                name: "颐福里居委会",
                code: "004",
            },
            VillageCode {
                name: "来安居委会",
                code: "005",
            },
            VillageCode {
                name: "南林居委会",
                code: "006",
            },
            VillageCode {
                name: "三生里居委会",
                code: "007",
            },
            VillageCode {
                name: "永顺居委会",
                code: "008",
            },
            VillageCode {
                name: "南星路居委会",
                code: "009",
            },
            VillageCode {
                name: "海昌居委会",
                code: "010",
            },
            VillageCode {
                name: "高寿里居委会",
                code: "011",
            },
            VillageCode {
                name: "吉庆里居委会",
                code: "012",
            },
            VillageCode {
                name: "晋元居委会",
                code: "013",
            },
            VillageCode {
                name: "华天居委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "宝山路街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "通源居委会",
                code: "001",
            },
            VillageCode {
                name: "宝山路四九九弄居委会",
                code: "002",
            },
            VillageCode {
                name: "三宝居委会",
                code: "003",
            },
            VillageCode {
                name: "儒林里居委会",
                code: "004",
            },
            VillageCode {
                name: "新宝通居委会",
                code: "005",
            },
            VillageCode {
                name: "陆丰居委会",
                code: "006",
            },
            VillageCode {
                name: "象山里居委会",
                code: "007",
            },
            VillageCode {
                name: "会铁居委会",
                code: "008",
            },
            VillageCode {
                name: "王家宅居委会",
                code: "009",
            },
            VillageCode {
                name: "芷江中路二九四弄居委会",
                code: "010",
            },
            VillageCode {
                name: "宝昌路六百弄居委会",
                code: "011",
            },
            VillageCode {
                name: "通阁新村居委会",
                code: "012",
            },
            VillageCode {
                name: "新华德居委会",
                code: "013",
            },
            VillageCode {
                name: "存仁居委会",
                code: "014",
            },
            VillageCode {
                name: "宝华里居委会",
                code: "015",
            },
            VillageCode {
                name: "新汉兴居委会",
                code: "016",
            },
            VillageCode {
                name: "止园新村居委会",
                code: "017",
            },
            VillageCode {
                name: "青云路四三五弄居委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "共和新路街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "柳营新村第一居委会",
                code: "001",
            },
            VillageCode {
                name: "柳营新村第二居委会",
                code: "002",
            },
            VillageCode {
                name: "洛平居委会",
                code: "003",
            },
            VillageCode {
                name: "谈家桥居委会",
                code: "004",
            },
            VillageCode {
                name: "沪北新村居委会",
                code: "005",
            },
            VillageCode {
                name: "谈家宅居委会",
                code: "006",
            },
            VillageCode {
                name: "谈家桥八十弄居委会",
                code: "007",
            },
            VillageCode {
                name: "中山北路八零五弄居委会",
                code: "008",
            },
            VillageCode {
                name: "柳营桥居委会",
                code: "009",
            },
            VillageCode {
                name: "唐家沙居委会",
                code: "010",
            },
            VillageCode {
                name: "洛川居委会",
                code: "011",
            },
            VillageCode {
                name: "延新居委会",
                code: "012",
            },
            VillageCode {
                name: "三阳居委会",
                code: "013",
            },
            VillageCode {
                name: "洛川中路一千一百弄居委会",
                code: "014",
            },
            VillageCode {
                name: "锦佳苑居委会",
                code: "015",
            },
            VillageCode {
                name: "申地居委会",
                code: "016",
            },
            VillageCode {
                name: "永乐居委会",
                code: "017",
            },
            VillageCode {
                name: "和乐居委会",
                code: "018",
            },
            VillageCode {
                name: "锦灏居委会",
                code: "019",
            },
            VillageCode {
                name: "新理想居委会",
                code: "020",
            },
            VillageCode {
                name: "嘉利居委会",
                code: "021",
            },
            VillageCode {
                name: "洛善居委会",
                code: "022",
            },
            VillageCode {
                name: "家豪城居委会",
                code: "023",
            },
            VillageCode {
                name: "中山北路八九九弄居委会",
                code: "024",
            },
            VillageCode {
                name: "延长中路七二七弄居委会",
                code: "025",
            },
            VillageCode {
                name: "黄山锦庭居委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "大宁路街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "延长新村居委会",
                code: "001",
            },
            VillageCode {
                name: "延铁新村居委会",
                code: "002",
            },
            VillageCode {
                name: "大宁一村居委会",
                code: "003",
            },
            VillageCode {
                name: "大宁二村居委会",
                code: "004",
            },
            VillageCode {
                name: "上工新村居委会",
                code: "005",
            },
            VillageCode {
                name: "广延路居委会",
                code: "006",
            },
            VillageCode {
                name: "延长中路四五一弄居委会",
                code: "007",
            },
            VillageCode {
                name: "八方花苑居委会",
                code: "008",
            },
            VillageCode {
                name: "平型关路八零一弄居委会",
                code: "009",
            },
            VillageCode {
                name: "大宁路五四零弄居委会",
                code: "010",
            },
            VillageCode {
                name: "延峰居委会",
                code: "011",
            },
            VillageCode {
                name: "粤秀路居委会",
                code: "012",
            },
            VillageCode {
                name: "大宁路五零五弄居委会",
                code: "013",
            },
            VillageCode {
                name: "新梅共和城居委会",
                code: "014",
            },
            VillageCode {
                name: "大宁路六六七弄居委会",
                code: "015",
            },
            VillageCode {
                name: "慧芝湖花园居委会",
                code: "016",
            },
            VillageCode {
                name: "宝华现代城居委会",
                code: "017",
            },
            VillageCode {
                name: "永和南居委会",
                code: "018",
            },
            VillageCode {
                name: "平型关路二一九九弄居委会",
                code: "019",
            },
            VillageCode {
                name: "永和北居委会",
                code: "020",
            },
            VillageCode {
                name: "虹屿居委会",
                code: "021",
            },
            VillageCode {
                name: "金茂雅苑居委会",
                code: "022",
            },
            VillageCode {
                name: "云平居委会",
                code: "023",
            },
            VillageCode {
                name: "云荣居委会",
                code: "024",
            },
            VillageCode {
                name: "云秀居委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "彭浦新村街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "彭浦新村第一居委会",
                code: "001",
            },
            VillageCode {
                name: "彭浦新村第三居委会",
                code: "002",
            },
            VillageCode {
                name: "彭浦新村第五居委会",
                code: "003",
            },
            VillageCode {
                name: "彭浦新村第七居委会",
                code: "004",
            },
            VillageCode {
                name: "平顺路一八零弄居委会",
                code: "005",
            },
            VillageCode {
                name: "岭南路五三九弄居委会",
                code: "006",
            },
            VillageCode {
                name: "彭新居委会",
                code: "007",
            },
            VillageCode {
                name: "临汾路一二四四弄居委会",
                code: "008",
            },
            VillageCode {
                name: "闻喜路一一一零弄居委会",
                code: "009",
            },
            VillageCode {
                name: "临汾路一五六四弄居委会",
                code: "010",
            },
            VillageCode {
                name: "场中路二四七一弄居委会",
                code: "011",
            },
            VillageCode {
                name: "共和新路四五五五弄居委会",
                code: "012",
            },
            VillageCode {
                name: "平顺路七九零弄居委会",
                code: "013",
            },
            VillageCode {
                name: "临汾路八九四弄居委会",
                code: "014",
            },
            VillageCode {
                name: "场中路二四零一弄居委会",
                code: "015",
            },
            VillageCode {
                name: "闻喜路九三五弄居委会",
                code: "016",
            },
            VillageCode {
                name: "三泉路四二四弄居委会",
                code: "017",
            },
            VillageCode {
                name: "保平居委会",
                code: "018",
            },
            VillageCode {
                name: "三泉路七七零弄居委会",
                code: "019",
            },
            VillageCode {
                name: "三泉路五一七弄居委会",
                code: "020",
            },
            VillageCode {
                name: "三泉路八二一弄居委会",
                code: "021",
            },
            VillageCode {
                name: "三泉路六零一弄居委会",
                code: "022",
            },
            VillageCode {
                name: "三泉路一零一五弄居委会",
                code: "023",
            },
            VillageCode {
                name: "临汾路一五一五弄居委会",
                code: "024",
            },
            VillageCode {
                name: "共康三村居委会",
                code: "025",
            },
            VillageCode {
                name: "共康四村第一居委会",
                code: "026",
            },
            VillageCode {
                name: "共康四村第二居委会",
                code: "027",
            },
            VillageCode {
                name: "保德路一三一六弄居委会",
                code: "028",
            },
            VillageCode {
                name: "曲沃路四三零弄居委会",
                code: "029",
            },
            VillageCode {
                name: "保德路九二一弄居委会",
                code: "030",
            },
            VillageCode {
                name: "场中路二六零一弄居委会",
                code: "031",
            },
            VillageCode {
                name: "艺康苑居委会",
                code: "032",
            },
            VillageCode {
                name: "三泉家园居委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "临汾路街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "岭南路一百弄居委会",
                code: "001",
            },
            VillageCode {
                name: "临汾路二九九弄居委会",
                code: "002",
            },
            VillageCode {
                name: "岭南路二七零弄居委会",
                code: "003",
            },
            VillageCode {
                name: "临汾路三七五弄居委会",
                code: "004",
            },
            VillageCode {
                name: "临汾路三八零弄居委会",
                code: "005",
            },
            VillageCode {
                name: "保德路四二五弄居委会",
                code: "006",
            },
            VillageCode {
                name: "闻喜路二五一弄居委会",
                code: "007",
            },
            VillageCode {
                name: "汾西路二六一弄居委会",
                code: "008",
            },
            VillageCode {
                name: "阳曲路四七零弄居委会",
                code: "009",
            },
            VillageCode {
                name: "阳曲路五七零弄居委会",
                code: "010",
            },
            VillageCode {
                name: "岭南路七百弄居委会",
                code: "011",
            },
            VillageCode {
                name: "汾西路二六零弄居委会",
                code: "012",
            },
            VillageCode {
                name: "阳曲路七六零弄居委会",
                code: "013",
            },
            VillageCode {
                name: "场中路一零一一弄居委会",
                code: "014",
            },
            VillageCode {
                name: "景凤路五二零弄居委会",
                code: "015",
            },
            VillageCode {
                name: "汾西路八十七弄居委会",
                code: "016",
            },
            VillageCode {
                name: "闻喜路五五五弄居委会",
                code: "017",
            },
            VillageCode {
                name: "临汾路九十九弄居委会",
                code: "018",
            },
            VillageCode {
                name: "和源名城第一居委会",
                code: "019",
            },
            VillageCode {
                name: "汾西路八十八弄居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "芷江西路街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "大统居委会",
                code: "001",
            },
            VillageCode {
                name: "共和新路居委会",
                code: "002",
            },
            VillageCode {
                name: "南山居委会",
                code: "003",
            },
            VillageCode {
                name: "洪南山宅居委会",
                code: "004",
            },
            VillageCode {
                name: "新赵家宅居委会",
                code: "005",
            },
            VillageCode {
                name: "芷江西路一二三弄居委会",
                code: "006",
            },
            VillageCode {
                name: "芷江新村居委会",
                code: "007",
            },
            VillageCode {
                name: "永太居委会",
                code: "008",
            },
            VillageCode {
                name: "苏家巷居委会",
                code: "009",
            },
            VillageCode {
                name: "交通公园居委会",
                code: "010",
            },
            VillageCode {
                name: "复元坊居委会",
                code: "011",
            },
            VillageCode {
                name: "中兴路一二三三弄居委会",
                code: "012",
            },
            VillageCode {
                name: "光华坊居委会",
                code: "013",
            },
            VillageCode {
                name: "三兴大楼居委会",
                code: "014",
            },
            VillageCode {
                name: "灵光居委会",
                code: "015",
            },
            VillageCode {
                name: "城上城居委会",
                code: "016",
            },
            VillageCode {
                name: "普善居委会",
                code: "017",
            },
            VillageCode {
                name: "协和居委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "彭浦镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "海鹰居委会",
                code: "001",
            },
            VillageCode {
                name: "翔前居委会",
                code: "002",
            },
            VillageCode {
                name: "场中路八零一弄居委会",
                code: "003",
            },
            VillageCode {
                name: "绿园居委会",
                code: "004",
            },
            VillageCode {
                name: "永和北第一居委会",
                code: "005",
            },
            VillageCode {
                name: "万荣东怡居委会",
                code: "006",
            },
            VillageCode {
                name: "沪太路一零五一弄居委会",
                code: "007",
            },
            VillageCode {
                name: "沪太路一一七零弄居委会",
                code: "008",
            },
            VillageCode {
                name: "永和北第二居委会",
                code: "009",
            },
            VillageCode {
                name: "洪泉居委会",
                code: "010",
            },
            VillageCode {
                name: "万荣新苑居委会",
                code: "011",
            },
            VillageCode {
                name: "共和新路三六五零弄居委会",
                code: "012",
            },
            VillageCode {
                name: "万荣佳苑居委会",
                code: "013",
            },
            VillageCode {
                name: "永和北第三居委会",
                code: "014",
            },
            VillageCode {
                name: "场中路二千八百弄居委会",
                code: "015",
            },
            VillageCode {
                name: "丽园居委会",
                code: "016",
            },
            VillageCode {
                name: "阳城居委会",
                code: "017",
            },
            VillageCode {
                name: "幸福一村居委会",
                code: "018",
            },
            VillageCode {
                name: "幸福二村居委会",
                code: "019",
            },
            VillageCode {
                name: "龙潭居委会",
                code: "020",
            },
            VillageCode {
                name: "运城居委会",
                code: "021",
            },
            VillageCode {
                name: "望景苑居委会",
                code: "022",
            },
            VillageCode {
                name: "江场西路一三六六弄居委会",
                code: "023",
            },
            VillageCode {
                name: "白玉兰馨园居委会",
                code: "024",
            },
            VillageCode {
                name: "龙馨嘉园居委会",
                code: "025",
            },
            VillageCode {
                name: "美景雅苑居委会",
                code: "026",
            },
            VillageCode {
                name: "成亿花园居委会",
                code: "027",
            },
            VillageCode {
                name: "永和家园居委会",
                code: "028",
            },
            VillageCode {
                name: "晋城居委会",
                code: "029",
            },
            VillageCode {
                name: "阳城贵都居委会",
                code: "030",
            },
            VillageCode {
                name: "塘南居委会",
                code: "031",
            },
            VillageCode {
                name: "白遗桥居委会",
                code: "032",
            },
            VillageCode {
                name: "沪太路九三五弄居委会",
                code: "033",
            },
            VillageCode {
                name: "广中西路九九九弄居委会",
                code: "034",
            },
            VillageCode {
                name: "龙盛雅苑居委会",
                code: "035",
            },
            VillageCode {
                name: "灵石路九六三弄居委会",
                code: "036",
            },
            VillageCode {
                name: "彭浦村村委会",
                code: "037",
            },
        ],
    },
];

static TOWNS_SJ_006: [TownCode; 10] = [
    TownCode {
        name: "曹杨新村街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "源园居委会",
                code: "001",
            },
            VillageCode {
                name: "花溪园居委会",
                code: "002",
            },
            VillageCode {
                name: "金岭园居委会",
                code: "003",
            },
            VillageCode {
                name: "杏梅园居委会",
                code: "004",
            },
            VillageCode {
                name: "南岭园居委会",
                code: "005",
            },
            VillageCode {
                name: "南杨园居委会",
                code: "006",
            },
            VillageCode {
                name: "南梅园居委会",
                code: "007",
            },
            VillageCode {
                name: "沙溪园居委会",
                code: "008",
            },
            VillageCode {
                name: "南溪园居委会",
                code: "009",
            },
            VillageCode {
                name: "枫杨园居委会",
                code: "010",
            },
            VillageCode {
                name: "金杨园居委会",
                code: "011",
            },
            VillageCode {
                name: "兰溪园居委会",
                code: "012",
            },
            VillageCode {
                name: "金梅园居委会",
                code: "013",
            },
            VillageCode {
                name: "杏杨园居委会",
                code: "014",
            },
            VillageCode {
                name: "北梅园居委会",
                code: "015",
            },
            VillageCode {
                name: "北岭园居委会",
                code: "016",
            },
            VillageCode {
                name: "兰岭园居委会",
                code: "017",
            },
            VillageCode {
                name: "桂杨园居委会",
                code: "018",
            },
            VillageCode {
                name: "北杨园居委会",
                code: "019",
            },
            VillageCode {
                name: "枫岭园居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "长风新村街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "曹家巷居委会",
                code: "001",
            },
            VillageCode {
                name: "中山桥居委会",
                code: "002",
            },
            VillageCode {
                name: "师大一村居委会",
                code: "003",
            },
            VillageCode {
                name: "新渡口居委会",
                code: "004",
            },
            VillageCode {
                name: "长风一村居委会",
                code: "005",
            },
            VillageCode {
                name: "长风二村第一居委会",
                code: "006",
            },
            VillageCode {
                name: "长风三村居委会",
                code: "007",
            },
            VillageCode {
                name: "长风四村第一居委会",
                code: "008",
            },
            VillageCode {
                name: "大渡河路九十五弄居委会",
                code: "009",
            },
            VillageCode {
                name: "长风四村第二居委会",
                code: "010",
            },
            VillageCode {
                name: "长风二村第二居委会",
                code: "011",
            },
            VillageCode {
                name: "锦绿新城居委会",
                code: "012",
            },
            VillageCode {
                name: "曹家村居委会",
                code: "013",
            },
            VillageCode {
                name: "白玉新村第一居委会",
                code: "014",
            },
            VillageCode {
                name: "白玉新村第二居委会",
                code: "015",
            },
            VillageCode {
                name: "普陀四村第一居委会",
                code: "016",
            },
            VillageCode {
                name: "普陀四村第二居委会",
                code: "017",
            },
            VillageCode {
                name: "普陀二村居委会",
                code: "018",
            },
            VillageCode {
                name: "金沙新村居委会",
                code: "019",
            },
            VillageCode {
                name: "隆德居委会",
                code: "020",
            },
            VillageCode {
                name: "海鑫公寓居委会",
                code: "021",
            },
            VillageCode {
                name: "师大三村居委会",
                code: "022",
            },
            VillageCode {
                name: "中江居委会",
                code: "023",
            },
            VillageCode {
                name: "清水湾居委会",
                code: "024",
            },
            VillageCode {
                name: "世纪同乐居委会",
                code: "025",
            },
            VillageCode {
                name: "紫御豪庭居委会",
                code: "026",
            },
            VillageCode {
                name: "雅仕汇都居委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "长寿路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "绿洲城市花园居委会",
                code: "001",
            },
            VillageCode {
                name: "澳门路居委会",
                code: "002",
            },
            VillageCode {
                name: "陕北居委会",
                code: "003",
            },
            VillageCode {
                name: "正红里居委会",
                code: "004",
            },
            VillageCode {
                name: "梅山苑居委会",
                code: "005",
            },
            VillageCode {
                name: "长鸿居委会",
                code: "006",
            },
            VillageCode {
                name: "长寿新村第三居委会",
                code: "007",
            },
            VillageCode {
                name: "长寿新村第四居委会",
                code: "008",
            },
            VillageCode {
                name: "九茂居委会",
                code: "009",
            },
            VillageCode {
                name: "世纪之门居委会",
                code: "010",
            },
            VillageCode {
                name: "合德里居委会",
                code: "011",
            },
            VillageCode {
                name: "永定新村居委会",
                code: "012",
            },
            VillageCode {
                name: "叶家宅居委会",
                code: "013",
            },
            VillageCode {
                name: "锦绣里居委会",
                code: "014",
            },
            VillageCode {
                name: "武宁新村第一居委会",
                code: "015",
            },
            VillageCode {
                name: "武宁新村第二居委会",
                code: "016",
            },
            VillageCode {
                name: "安全里居委会",
                code: "017",
            },
            VillageCode {
                name: "普雄路居委会",
                code: "018",
            },
            VillageCode {
                name: "谈家渡居委会",
                code: "019",
            },
            VillageCode {
                name: "芙蓉花苑居委会",
                code: "020",
            },
            VillageCode {
                name: "地方天园居委会",
                code: "021",
            },
            VillageCode {
                name: "玉佛城居委会",
                code: "022",
            },
            VillageCode {
                name: "知音苑居委会",
                code: "023",
            },
            VillageCode {
                name: "武宁小城居委会",
                code: "024",
            },
            VillageCode {
                name: "半岛花园居委会",
                code: "025",
            },
            VillageCode {
                name: "秋水云庐居委会",
                code: "026",
            },
            VillageCode {
                name: "河滨围城居委会",
                code: "027",
            },
            VillageCode {
                name: "圣骊澳门苑居委会",
                code: "028",
            },
            VillageCode {
                name: "苏堤春晓居委会",
                code: "029",
            },
            VillageCode {
                name: "新湖明珠城居委会",
                code: "030",
            },
            VillageCode {
                name: "逸流公寓居委会",
                code: "031",
            },
            VillageCode {
                name: "大上海城市花园居委会",
                code: "032",
            },
            VillageCode {
                name: "泰欣嘉园居委会",
                code: "033",
            },
            VillageCode {
                name: "上青佳园居委会",
                code: "034",
            },
            VillageCode {
                name: "梅芳里居委会",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "甘泉路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "延长居委会",
                code: "001",
            },
            VillageCode {
                name: "合阳居委会",
                code: "002",
            },
            VillageCode {
                name: "新宜居委会",
                code: "003",
            },
            VillageCode {
                name: "新沪居委会",
                code: "004",
            },
            VillageCode {
                name: "子长居委会",
                code: "005",
            },
            VillageCode {
                name: "长新居委会",
                code: "006",
            },
            VillageCode {
                name: "周家巷居委会",
                code: "007",
            },
            VillageCode {
                name: "志丹居委会",
                code: "008",
            },
            VillageCode {
                name: "名都花苑居委会",
                code: "009",
            },
            VillageCode {
                name: "黄陵居委会",
                code: "010",
            },
            VillageCode {
                name: "安塞居委会",
                code: "011",
            },
            VillageCode {
                name: "汪家井居委会",
                code: "012",
            },
            VillageCode {
                name: "章家巷居委会",
                code: "013",
            },
            VillageCode {
                name: "新长居委会",
                code: "014",
            },
            VillageCode {
                name: "南泉苑居委会",
                code: "015",
            },
            VillageCode {
                name: "甘泉苑居委会",
                code: "016",
            },
            VillageCode {
                name: "东泉苑居委会",
                code: "017",
            },
            VillageCode {
                name: "悦达第一居委会",
                code: "018",
            },
            VillageCode {
                name: "双山居委会",
                code: "019",
            },
            VillageCode {
                name: "新灵居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "石泉路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "中山新村居委会",
                code: "001",
            },
            VillageCode {
                name: "镇坪路居委会",
                code: "002",
            },
            VillageCode {
                name: "太浜巷居委会",
                code: "003",
            },
            VillageCode {
                name: "薛家厍居委会",
                code: "004",
            },
            VillageCode {
                name: "信仪新村居委会",
                code: "005",
            },
            VillageCode {
                name: "石泉新村第一居委会",
                code: "006",
            },
            VillageCode {
                name: "石泉新村第二居委会",
                code: "007",
            },
            VillageCode {
                name: "石泉六村居委会",
                code: "008",
            },
            VillageCode {
                name: "铁路新村居委会",
                code: "009",
            },
            VillageCode {
                name: "洵阳新村第一居委会",
                code: "010",
            },
            VillageCode {
                name: "洵阳新村第二居委会",
                code: "011",
            },
            VillageCode {
                name: "石岚三村居委会",
                code: "012",
            },
            VillageCode {
                name: "管弄新村第一居委会",
                code: "013",
            },
            VillageCode {
                name: "管弄新村第二居委会",
                code: "014",
            },
            VillageCode {
                name: "管弄新村第三居委会",
                code: "015",
            },
            VillageCode {
                name: "陆家宅第一居委会",
                code: "016",
            },
            VillageCode {
                name: "陆家宅第二居委会",
                code: "017",
            },
            VillageCode {
                name: "联合新村居委会",
                code: "018",
            },
            VillageCode {
                name: "和平新村居委会",
                code: "019",
            },
            VillageCode {
                name: "兰凤新村居委会",
                code: "020",
            },
            VillageCode {
                name: "中宁路居委会",
                code: "021",
            },
            VillageCode {
                name: "秋月枫舍居委会",
                code: "022",
            },
            VillageCode {
                name: "品尊国际居委会",
                code: "023",
            },
            VillageCode {
                name: "申兴华庭居委会",
                code: "024",
            },
            VillageCode {
                name: "城市之星居委会",
                code: "025",
            },
            VillageCode {
                name: "臻如府居委会",
                code: "026",
            },
            VillageCode {
                name: "铜川路第一居委会",
                code: "027",
            },
            VillageCode {
                name: "铜川路第二居委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "宜川路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "宜川四村居委会",
                code: "001",
            },
            VillageCode {
                name: "赵家花园居委会",
                code: "002",
            },
            VillageCode {
                name: "华阴路居委会",
                code: "003",
            },
            VillageCode {
                name: "泰山一村居委会",
                code: "004",
            },
            VillageCode {
                name: "泰山二村居委会",
                code: "005",
            },
            VillageCode {
                name: "泰山宅居委会",
                code: "006",
            },
            VillageCode {
                name: "交通西路居委会",
                code: "007",
            },
            VillageCode {
                name: "平江居委会",
                code: "008",
            },
            VillageCode {
                name: "大洋新村居委会",
                code: "009",
            },
            VillageCode {
                name: "光新村居委会",
                code: "010",
            },
            VillageCode {
                name: "农林村居委会",
                code: "011",
            },
            VillageCode {
                name: "香溢花城社区居委会",
                code: "012",
            },
            VillageCode {
                name: "振华居委会",
                code: "013",
            },
            VillageCode {
                name: "中远两湾城第一居委会",
                code: "014",
            },
            VillageCode {
                name: "中远两湾城第二居委会",
                code: "015",
            },
            VillageCode {
                name: "中远两湾城第三居委会",
                code: "016",
            },
            VillageCode {
                name: "中远两湾城第四居委会",
                code: "017",
            },
            VillageCode {
                name: "宜川六村第一居委会",
                code: "018",
            },
            VillageCode {
                name: "宜川六村第二居委会",
                code: "019",
            },
            VillageCode {
                name: "宜川三村第一居委会",
                code: "020",
            },
            VillageCode {
                name: "宜川三村第二居委会",
                code: "021",
            },
            VillageCode {
                name: "宜川一村第一居委会",
                code: "022",
            },
            VillageCode {
                name: "宜川一村第二居委会",
                code: "023",
            },
            VillageCode {
                name: "宜川二村第一居委会",
                code: "024",
            },
            VillageCode {
                name: "宜川二村第二居委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "万里街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "交暨居委会",
                code: "001",
            },
            VillageCode {
                name: "香泉路居委会",
                code: "002",
            },
            VillageCode {
                name: "万里名轩社区居委会",
                code: "003",
            },
            VillageCode {
                name: "中浩云居委会",
                code: "004",
            },
            VillageCode {
                name: "愉景华庭社区居委会",
                code: "005",
            },
            VillageCode {
                name: "中骏天誉社区居委会",
                code: "006",
            },
            VillageCode {
                name: "颐华第一社区居委会",
                code: "007",
            },
            VillageCode {
                name: "颐华第二社区居委会",
                code: "008",
            },
            VillageCode {
                name: "万里雅筑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "富平社区居委会",
                code: "010",
            },
            VillageCode {
                name: "凯旋华庭社区居委会",
                code: "011",
            },
            VillageCode {
                name: "凯旋公寓社区居委会",
                code: "012",
            },
            VillageCode {
                name: "中环家园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "中环锦园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "中环花苑社区居委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "真如镇街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "南大街居委会",
                code: "001",
            },
            VillageCode {
                name: "北大街居委会",
                code: "002",
            },
            VillageCode {
                name: "水塘街居委会",
                code: "003",
            },
            VillageCode {
                name: "真如西村居委会",
                code: "004",
            },
            VillageCode {
                name: "真北新村第一居委会",
                code: "005",
            },
            VillageCode {
                name: "真北新村第三居委会",
                code: "006",
            },
            VillageCode {
                name: "真西新村第一居委会",
                code: "007",
            },
            VillageCode {
                name: "真西新村第二居委会",
                code: "008",
            },
            VillageCode {
                name: "车站新村居委会",
                code: "009",
            },
            VillageCode {
                name: "真西新村第五居委会",
                code: "010",
            },
            VillageCode {
                name: "海棠苑居委会",
                code: "011",
            },
            VillageCode {
                name: "曹杨八村第一居委会",
                code: "012",
            },
            VillageCode {
                name: "曹杨八村第二居委会",
                code: "013",
            },
            VillageCode {
                name: "真光新村第二居委会",
                code: "014",
            },
            VillageCode {
                name: "真光新村第四居委会",
                code: "015",
            },
            VillageCode {
                name: "真光新村第五居委会",
                code: "016",
            },
            VillageCode {
                name: "真光新村第六居委会",
                code: "017",
            },
            VillageCode {
                name: "真光新村第七居委会",
                code: "018",
            },
            VillageCode {
                name: "真光新村第八居委会",
                code: "019",
            },
            VillageCode {
                name: "真光新村第九居委会",
                code: "020",
            },
            VillageCode {
                name: "真光新村第十居委会",
                code: "021",
            },
            VillageCode {
                name: "清涧新村第二居委会",
                code: "022",
            },
            VillageCode {
                name: "清涧新村第六居委会",
                code: "023",
            },
            VillageCode {
                name: "清涧新村第七居委会",
                code: "024",
            },
            VillageCode {
                name: "清涧新村第八居委会",
                code: "025",
            },
            VillageCode {
                name: "清涧新村第三居委会",
                code: "026",
            },
            VillageCode {
                name: "曹杨花苑居委会",
                code: "027",
            },
            VillageCode {
                name: "曹杨新苑居委会",
                code: "028",
            },
            VillageCode {
                name: "金鼎花苑居委会",
                code: "029",
            },
            VillageCode {
                name: "真光新村第一居委会",
                code: "030",
            },
            VillageCode {
                name: "真北新村第五居委会",
                code: "031",
            },
            VillageCode {
                name: "清涧新村第四居委会",
                code: "032",
            },
            VillageCode {
                name: "真光新村第三居委会",
                code: "033",
            },
            VillageCode {
                name: "中鼎豪园居委会",
                code: "034",
            },
            VillageCode {
                name: "星河世纪城居委会",
                code: "035",
            },
            VillageCode {
                name: "杨家桥第一社区居委会",
                code: "036",
            },
            VillageCode {
                name: "杨家桥第二社区居委会",
                code: "037",
            },
            VillageCode {
                name: "星金恒社区居委会",
                code: "038",
            },
            VillageCode {
                name: "芝川社区居委会",
                code: "039",
            },
        ],
    },
    TownCode {
        name: "长征镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "北巷居委会",
                code: "001",
            },
            VillageCode {
                name: "芝巷居委会",
                code: "002",
            },
            VillageCode {
                name: "新长征花苑第一居委会",
                code: "003",
            },
            VillageCode {
                name: "梅岭北路居委会",
                code: "004",
            },
            VillageCode {
                name: "怒江第二居委会",
                code: "005",
            },
            VillageCode {
                name: "梅川第一居委会",
                code: "006",
            },
            VillageCode {
                name: "梅川第二居委会",
                code: "007",
            },
            VillageCode {
                name: "梅川第四居委会",
                code: "008",
            },
            VillageCode {
                name: "梅川第六居委会",
                code: "009",
            },
            VillageCode {
                name: "怒江第一居委会",
                code: "010",
            },
            VillageCode {
                name: "运旺新村居委会",
                code: "011",
            },
            VillageCode {
                name: "泾泰居委会",
                code: "012",
            },
            VillageCode {
                name: "建德花园居委会",
                code: "013",
            },
            VillageCode {
                name: "真源小区居委会",
                code: "014",
            },
            VillageCode {
                name: "银开居委会",
                code: "015",
            },
            VillageCode {
                name: "东旺居委会",
                code: "016",
            },
            VillageCode {
                name: "祥和居委会",
                code: "017",
            },
            VillageCode {
                name: "新曹杨居委会",
                code: "018",
            },
            VillageCode {
                name: "金沙雅苑居委会",
                code: "019",
            },
            VillageCode {
                name: "万豪居委会",
                code: "020",
            },
            VillageCode {
                name: "建德小区第二居委会",
                code: "021",
            },
            VillageCode {
                name: "未来街区居委会",
                code: "022",
            },
            VillageCode {
                name: "祥和名邸居委会",
                code: "023",
            },
            VillageCode {
                name: "象源丽都居委会",
                code: "024",
            },
            VillageCode {
                name: "嘉广居委会",
                code: "025",
            },
            VillageCode {
                name: "梅岭北路第二居委会",
                code: "026",
            },
            VillageCode {
                name: "建德花园第三居委会",
                code: "027",
            },
            VillageCode {
                name: "丽湖社区居委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "桃浦镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "同济大学沪西校区居委会",
                code: "001",
            },
            VillageCode {
                name: "莲花公寓居委会",
                code: "002",
            },
            VillageCode {
                name: "昆仑花苑居委会",
                code: "003",
            },
            VillageCode {
                name: "李子园六村居委会",
                code: "004",
            },
            VillageCode {
                name: "瑞香苑居委会",
                code: "005",
            },
            VillageCode {
                name: "合欢苑居委会",
                code: "006",
            },
            VillageCode {
                name: "绿春苑居委会",
                code: "007",
            },
            VillageCode {
                name: "雪松苑第一居委会",
                code: "008",
            },
            VillageCode {
                name: "海棠苑居委会",
                code: "009",
            },
            VillageCode {
                name: "和乐苑居委会",
                code: "010",
            },
            VillageCode {
                name: "石榴苑居委会",
                code: "011",
            },
            VillageCode {
                name: "白丽苑居委会",
                code: "012",
            },
            VillageCode {
                name: "紫荆苑居委会",
                code: "013",
            },
            VillageCode {
                name: "杜鹃苑居委会",
                code: "014",
            },
            VillageCode {
                name: "山茶苑居委会",
                code: "015",
            },
            VillageCode {
                name: "迎春苑居委会",
                code: "016",
            },
            VillageCode {
                name: "樱花苑居委会",
                code: "017",
            },
            VillageCode {
                name: "雪松苑第二居委会",
                code: "018",
            },
            VillageCode {
                name: "雪松苑第三居委会",
                code: "019",
            },
            VillageCode {
                name: "紫藤苑居委会",
                code: "020",
            },
            VillageCode {
                name: "圣都汇社区居委会",
                code: "021",
            },
            VillageCode {
                name: "香樟苑居委会",
                code: "022",
            },
            VillageCode {
                name: "南李苑居委会",
                code: "023",
            },
            VillageCode {
                name: "新家园居委会",
                code: "024",
            },
            VillageCode {
                name: "永汇新苑居委会",
                code: "025",
            },
            VillageCode {
                name: "古浪苑居委会",
                code: "026",
            },
            VillageCode {
                name: "阳光建华城第一居委会",
                code: "027",
            },
            VillageCode {
                name: "阳光建华城第二居委会",
                code: "028",
            },
            VillageCode {
                name: "阳光建华城第三居委会",
                code: "029",
            },
            VillageCode {
                name: "阳光建华城第四居委会",
                code: "030",
            },
            VillageCode {
                name: "安祁古浪新苑居委会",
                code: "031",
            },
            VillageCode {
                name: "金祁新城居委会",
                code: "032",
            },
            VillageCode {
                name: "祥和星宇居委会",
                code: "033",
            },
            VillageCode {
                name: "华公馆居委会",
                code: "034",
            },
            VillageCode {
                name: "中环名品居委会",
                code: "035",
            },
            VillageCode {
                name: "锦竹苑社区居委会",
                code: "036",
            },
            VillageCode {
                name: "阳光水岸苑社区居委会",
                code: "037",
            },
            VillageCode {
                name: "荣和怡景园社区居委会",
                code: "038",
            },
            VillageCode {
                name: "金祁新城第二社区居委会",
                code: "039",
            },
            VillageCode {
                name: "阳光建华城第五居委会",
                code: "040",
            },
            VillageCode {
                name: "金光村村委会",
                code: "041",
            },
            VillageCode {
                name: "祁连村村委会",
                code: "042",
            },
            VillageCode {
                name: "春光村村委会",
                code: "043",
            },
            VillageCode {
                name: "李子园村村委会",
                code: "044",
            },
            VillageCode {
                name: "新杨村村委会",
                code: "045",
            },
            VillageCode {
                name: "桃浦村村委会",
                code: "046",
            },
            VillageCode {
                name: "槎浦村村委会",
                code: "047",
            },
            VillageCode {
                name: "真建花苑居委会",
                code: "048",
            },
        ],
    },
];

static TOWNS_SJ_007: [TownCode; 8] = [
    TownCode {
        name: "欧阳路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "欧二居委会",
                code: "001",
            },
            VillageCode {
                name: "欧三居委会",
                code: "002",
            },
            VillageCode {
                name: "欧四居委会",
                code: "003",
            },
            VillageCode {
                name: "欧五居委会",
                code: "004",
            },
            VillageCode {
                name: "建设新村居委会",
                code: "005",
            },
            VillageCode {
                name: "祥德路居委会",
                code: "006",
            },
            VillageCode {
                name: "董家宅居委会",
                code: "007",
            },
            VillageCode {
                name: "幸福村居委会",
                code: "008",
            },
            VillageCode {
                name: "邮电新村居委会",
                code: "009",
            },
            VillageCode {
                name: "大连新村第二居委会",
                code: "010",
            },
            VillageCode {
                name: "大连新村第三居委会",
                code: "011",
            },
            VillageCode {
                name: "大连西居委会",
                code: "012",
            },
            VillageCode {
                name: "祥德东居委会",
                code: "013",
            },
            VillageCode {
                name: "蒋家桥居委会",
                code: "014",
            },
            VillageCode {
                name: "北郊居委会",
                code: "015",
            },
            VillageCode {
                name: "虹仪居委会",
                code: "016",
            },
            VillageCode {
                name: "紫荆居委会",
                code: "017",
            },
            VillageCode {
                name: "模范新村居委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "曲阳路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "玉一居委会",
                code: "001",
            },
            VillageCode {
                name: "玉二居委会",
                code: "002",
            },
            VillageCode {
                name: "玉三居委会",
                code: "003",
            },
            VillageCode {
                name: "玉四居委会",
                code: "004",
            },
            VillageCode {
                name: "曲一居委会",
                code: "005",
            },
            VillageCode {
                name: "曲二居委会",
                code: "006",
            },
            VillageCode {
                name: "东体居委会",
                code: "007",
            },
            VillageCode {
                name: "东四居委会",
                code: "008",
            },
            VillageCode {
                name: "东五居委会",
                code: "009",
            },
            VillageCode {
                name: "密一居委会",
                code: "010",
            },
            VillageCode {
                name: "密二居委会",
                code: "011",
            },
            VillageCode {
                name: "密三居委会",
                code: "012",
            },
            VillageCode {
                name: "赤一居委会",
                code: "013",
            },
            VillageCode {
                name: "赤二居委会",
                code: "014",
            },
            VillageCode {
                name: "赤三居委会",
                code: "015",
            },
            VillageCode {
                name: "运一居委会",
                code: "016",
            },
            VillageCode {
                name: "运二居委会",
                code: "017",
            },
            VillageCode {
                name: "运三居委会",
                code: "018",
            },
            VillageCode {
                name: "林云居委会",
                code: "019",
            },
            VillageCode {
                name: "巴林居委会",
                code: "020",
            },
            VillageCode {
                name: "上农一居委会",
                code: "021",
            },
            VillageCode {
                name: "上农二居委会",
                code: "022",
            },
            VillageCode {
                name: "鸿雁居委会",
                code: "023",
            },
            VillageCode {
                name: "银联居委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "广中路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "西江湾路居委会",
                code: "001",
            },
            VillageCode {
                name: "商业一村居委会",
                code: "002",
            },
            VillageCode {
                name: "商洛小区居委会",
                code: "003",
            },
            VillageCode {
                name: "广中路居委会",
                code: "004",
            },
            VillageCode {
                name: "广中新村第二居委会",
                code: "005",
            },
            VillageCode {
                name: "广中新村第三居委会",
                code: "006",
            },
            VillageCode {
                name: "友谊新村居委会",
                code: "007",
            },
            VillageCode {
                name: "广灵新村居委会",
                code: "008",
            },
            VillageCode {
                name: "新虹居委会",
                code: "009",
            },
            VillageCode {
                name: "灵新居委会",
                code: "010",
            },
            VillageCode {
                name: "金属新村居委会",
                code: "011",
            },
            VillageCode {
                name: "同心路居委会",
                code: "012",
            },
            VillageCode {
                name: "同济居委会",
                code: "013",
            },
            VillageCode {
                name: "何家宅居委会",
                code: "014",
            },
            VillageCode {
                name: "八字桥居委会",
                code: "015",
            },
            VillageCode {
                name: "柳营路居委会",
                code: "016",
            },
            VillageCode {
                name: "黄山路居委会",
                code: "017",
            },
            VillageCode {
                name: "恒业路居委会",
                code: "018",
            },
            VillageCode {
                name: "华昌路居委会",
                code: "019",
            },
            VillageCode {
                name: "横浜路居委会",
                code: "020",
            },
            VillageCode {
                name: "株洲路居委会",
                code: "021",
            },
            VillageCode {
                name: "花园城居委会",
                code: "022",
            },
            VillageCode {
                name: "天通庵路居委会",
                code: "023",
            },
            VillageCode {
                name: "同煌路居委会",
                code: "024",
            },
            VillageCode {
                name: "新同心路居委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "嘉兴路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "垦业居委会",
                code: "001",
            },
            VillageCode {
                name: "海伦居委会",
                code: "002",
            },
            VillageCode {
                name: "庆源居委会",
                code: "003",
            },
            VillageCode {
                name: "瑞康居委会",
                code: "004",
            },
            VillageCode {
                name: "天水居委会",
                code: "005",
            },
            VillageCode {
                name: "庆阳居委会",
                code: "006",
            },
            VillageCode {
                name: "安丘居委会",
                code: "007",
            },
            VillageCode {
                name: "宝元居委会",
                code: "008",
            },
            VillageCode {
                name: "通州居委会",
                code: "009",
            },
            VillageCode {
                name: "临平居委会",
                code: "010",
            },
            VillageCode {
                name: "通海居委会",
                code: "011",
            },
            VillageCode {
                name: "岳州居委会",
                code: "012",
            },
            VillageCode {
                name: "新陆居委会",
                code: "013",
            },
            VillageCode {
                name: "海欧居委会",
                code: "014",
            },
            VillageCode {
                name: "香港丽园居委会",
                code: "015",
            },
            VillageCode {
                name: "瑞虹新城第一居委会",
                code: "016",
            },
            VillageCode {
                name: "飘鹰居委会",
                code: "017",
            },
            VillageCode {
                name: "虹叶居委会",
                code: "018",
            },
            VillageCode {
                name: "自建居委会",
                code: "019",
            },
            VillageCode {
                name: "大连居委会",
                code: "020",
            },
            VillageCode {
                name: "新港居委会",
                code: "021",
            },
            VillageCode {
                name: "和平居委会",
                code: "022",
            },
            VillageCode {
                name: "张桥居委会",
                code: "023",
            },
            VillageCode {
                name: "榆兰居委会",
                code: "024",
            },
            VillageCode {
                name: "天虹居委会",
                code: "025",
            },
            VillageCode {
                name: "瑞鑫居委会",
                code: "026",
            },
            VillageCode {
                name: "梧州居委会",
                code: "027",
            },
            VillageCode {
                name: "璟庭居委会",
                code: "028",
            },
            VillageCode {
                name: "建邦居委会",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "凉城新村街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "广水居委会",
                code: "001",
            },
            VillageCode {
                name: "水电居委会",
                code: "002",
            },
            VillageCode {
                name: "汶一居委会",
                code: "003",
            },
            VillageCode {
                name: "汶二居委会",
                code: "004",
            },
            VillageCode {
                name: "华苑第一居委会",
                code: "005",
            },
            VillageCode {
                name: "华苑第二居委会",
                code: "006",
            },
            VillageCode {
                name: "复旦居委会",
                code: "007",
            },
            VillageCode {
                name: "文苑第一居委会",
                code: "008",
            },
            VillageCode {
                name: "文苑第二居委会",
                code: "009",
            },
            VillageCode {
                name: "文苑第三居委会",
                code: "010",
            },
            VillageCode {
                name: "秀苑居委会",
                code: "011",
            },
            VillageCode {
                name: "凉城新村第一居委会",
                code: "012",
            },
            VillageCode {
                name: "凉城新村第二居委会",
                code: "013",
            },
            VillageCode {
                name: "凉城新村第三居委会",
                code: "014",
            },
            VillageCode {
                name: "凉城新村第四居委会",
                code: "015",
            },
            VillageCode {
                name: "凉城新村第五居委会",
                code: "016",
            },
            VillageCode {
                name: "凉城新村第六居委会",
                code: "017",
            },
            VillageCode {
                name: "广凉居委会",
                code: "018",
            },
            VillageCode {
                name: "广粤居委会",
                code: "019",
            },
            VillageCode {
                name: "梦湖苑居委会",
                code: "020",
            },
            VillageCode {
                name: "馨苑居委会",
                code: "021",
            },
            VillageCode {
                name: "广灵二路居委会",
                code: "022",
            },
            VillageCode {
                name: "科佳居委会",
                code: "023",
            },
            VillageCode {
                name: "中虹居委会",
                code: "024",
            },
            VillageCode {
                name: "锦苑居委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "四川北路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "山一居委会",
                code: "001",
            },
            VillageCode {
                name: "山二居委会",
                code: "002",
            },
            VillageCode {
                name: "山三居委会",
                code: "003",
            },
            VillageCode {
                name: "黄渡路居委会",
                code: "004",
            },
            VillageCode {
                name: "多伦路居委会",
                code: "005",
            },
            VillageCode {
                name: "永美居委会",
                code: "006",
            },
            VillageCode {
                name: "浙兴里居委会",
                code: "007",
            },
            VillageCode {
                name: "余庆坊居委会",
                code: "008",
            },
            VillageCode {
                name: "长春路居委会",
                code: "009",
            },
            VillageCode {
                name: "邢长居委会",
                code: "010",
            },
            VillageCode {
                name: "新乡居委会",
                code: "011",
            },
            VillageCode {
                name: "永德居委会",
                code: "012",
            },
            VillageCode {
                name: "正兴居委会",
                code: "013",
            },
            VillageCode {
                name: "中州路居委会",
                code: "014",
            },
            VillageCode {
                name: "南克俭居委会",
                code: "015",
            },
            VillageCode {
                name: "四川里居委会",
                code: "016",
            },
            VillageCode {
                name: "永丰居委会",
                code: "017",
            },
            VillageCode {
                name: "宝安居委会",
                code: "018",
            },
            VillageCode {
                name: "吉祥居委会",
                code: "019",
            },
            VillageCode {
                name: "衡水居委会",
                code: "020",
            },
            VillageCode {
                name: "虬江居委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "北外滩街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "长治居委会",
                code: "001",
            },
            VillageCode {
                name: "汉阳居委会",
                code: "002",
            },
            VillageCode {
                name: "前进居委会",
                code: "003",
            },
            VillageCode {
                name: "西安居委会",
                code: "004",
            },
            VillageCode {
                name: "惠民居委会",
                code: "005",
            },
            VillageCode {
                name: "春江居委会",
                code: "006",
            },
            VillageCode {
                name: "明华坊居委会",
                code: "007",
            },
            VillageCode {
                name: "提篮桥居委会",
                code: "008",
            },
            VillageCode {
                name: "晋阳居委会",
                code: "009",
            },
            VillageCode {
                name: "东大名居委会",
                code: "010",
            },
            VillageCode {
                name: "北外滩居委会",
                code: "011",
            },
            VillageCode {
                name: "久耕居委会",
                code: "012",
            },
            VillageCode {
                name: "三联居委会",
                code: "013",
            },
            VillageCode {
                name: "蕃兴居委会",
                code: "014",
            },
            VillageCode {
                name: "逢源居委会",
                code: "015",
            },
            VillageCode {
                name: "唐山居委会",
                code: "016",
            },
            VillageCode {
                name: "鸿兴居委会",
                code: "017",
            },
            VillageCode {
                name: "恒泰居委会",
                code: "018",
            },
            VillageCode {
                name: "南天潼居委会",
                code: "019",
            },
            VillageCode {
                name: "昆山居委会",
                code: "020",
            },
            VillageCode {
                name: "海宁居委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "江湾镇街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "奎照路居委会",
                code: "001",
            },
            VillageCode {
                name: "新市南路居委会",
                code: "002",
            },
            VillageCode {
                name: "纪念路居委会",
                code: "003",
            },
            VillageCode {
                name: "文治路居委会",
                code: "004",
            },
            VillageCode {
                name: "车站西路居委会",
                code: "005",
            },
            VillageCode {
                name: "万安路西居委会",
                code: "006",
            },
            VillageCode {
                name: "忠烈居委会",
                code: "007",
            },
            VillageCode {
                name: "丰镇路居委会",
                code: "008",
            },
            VillageCode {
                name: "沽源路居委会",
                code: "009",
            },
            VillageCode {
                name: "新市北路居委会",
                code: "010",
            },
            VillageCode {
                name: "逸仙路居委会",
                code: "011",
            },
            VillageCode {
                name: "仁德路居委会",
                code: "012",
            },
            VillageCode {
                name: "池沟路居委会",
                code: "013",
            },
            VillageCode {
                name: "镇西居委会",
                code: "014",
            },
            VillageCode {
                name: "镇北居委会",
                code: "015",
            },
            VillageCode {
                name: "三门路居委会",
                code: "016",
            },
            VillageCode {
                name: "车南居委会",
                code: "017",
            },
            VillageCode {
                name: "春生居委会",
                code: "018",
            },
            VillageCode {
                name: "丰镇路第二居委会",
                code: "019",
            },
            VillageCode {
                name: "胜利居委会",
                code: "020",
            },
            VillageCode {
                name: "韶嘉居委会",
                code: "021",
            },
            VillageCode {
                name: "学府居委会",
                code: "022",
            },
            VillageCode {
                name: "宝鸿居委会",
                code: "023",
            },
            VillageCode {
                name: "三门路二居委会",
                code: "024",
            },
            VillageCode {
                name: "方浜居委会",
                code: "025",
            },
            VillageCode {
                name: "新南二居委会",
                code: "026",
            },
            VillageCode {
                name: "场中居委会",
                code: "027",
            },
            VillageCode {
                name: "韶嘉第二居委会",
                code: "028",
            },
            VillageCode {
                name: "安汾路居委会",
                code: "029",
            },
            VillageCode {
                name: "欣逸居委会",
                code: "030",
            },
            VillageCode {
                name: "场中二居委会",
                code: "031",
            },
            VillageCode {
                name: "虹湾居委会",
                code: "032",
            },
            VillageCode {
                name: "虹纺居委会",
                code: "033",
            },
            VillageCode {
                name: "虹馥居委会",
                code: "034",
            },
            VillageCode {
                name: "虹彩居委会",
                code: "035",
            },
        ],
    },
];

static TOWNS_SJ_008: [TownCode; 12] = [
    TownCode {
        name: "定海路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "十七棉工房第二居委会",
                code: "001",
            },
            VillageCode {
                name: "十九棉居委会",
                code: "002",
            },
            VillageCode {
                name: "西白林寺第一居委会",
                code: "003",
            },
            VillageCode {
                name: "公助一村居委会",
                code: "004",
            },
            VillageCode {
                name: "东白林寺居委会",
                code: "005",
            },
            VillageCode {
                name: "白洋淀居委会",
                code: "006",
            },
            VillageCode {
                name: "军工路居委会",
                code: "007",
            },
            VillageCode {
                name: "波阳路居委会",
                code: "008",
            },
            VillageCode {
                name: "定海居委会",
                code: "009",
            },
            VillageCode {
                name: "定海港路居委会",
                code: "010",
            },
            VillageCode {
                name: "复兴岛居委会",
                code: "011",
            },
            VillageCode {
                name: "凉州路居委会",
                code: "012",
            },
            VillageCode {
                name: "爱国二村第二居委会",
                code: "013",
            },
            VillageCode {
                name: "隆昌居委会",
                code: "014",
            },
            VillageCode {
                name: "和润苑居委会",
                code: "015",
            },
            VillageCode {
                name: "和乐苑居委会",
                code: "016",
            },
            VillageCode {
                name: "西白林寺第二居委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "平凉路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "许阳居委会",
                code: "001",
            },
            VillageCode {
                name: "惠明居委会",
                code: "002",
            },
            VillageCode {
                name: "纺三居委会",
                code: "003",
            },
            VillageCode {
                name: "明园村居委会",
                code: "004",
            },
            VillageCode {
                name: "怀德居委会",
                code: "005",
            },
            VillageCode {
                name: "上水工房居委会",
                code: "006",
            },
            VillageCode {
                name: "秦家弄居委会",
                code: "007",
            },
            VillageCode {
                name: "兰州居委会",
                code: "008",
            },
            VillageCode {
                name: "福宁居委会",
                code: "009",
            },
            VillageCode {
                name: "霍新居委会",
                code: "010",
            },
            VillageCode {
                name: "万新居委会",
                code: "011",
            },
            VillageCode {
                name: "锦杨苑居委会",
                code: "012",
            },
            VillageCode {
                name: "海杨居委会",
                code: "013",
            },
            VillageCode {
                name: "嘉禄居委会",
                code: "014",
            },
            VillageCode {
                name: "福禄居委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "江浦路街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "吴家浜居委会",
                code: "001",
            },
            VillageCode {
                name: "兰州新村居委会",
                code: "002",
            },
            VillageCode {
                name: "陈家头第一居委会",
                code: "003",
            },
            VillageCode {
                name: "张家浜居委会",
                code: "004",
            },
            VillageCode {
                name: "陈家头第二居委会",
                code: "005",
            },
            VillageCode {
                name: "陈家头第三居委会",
                code: "006",
            },
            VillageCode {
                name: "辽源二村居委会",
                code: "007",
            },
            VillageCode {
                name: "辽源三村居委会",
                code: "008",
            },
            VillageCode {
                name: "姚家桥居委会",
                code: "009",
            },
            VillageCode {
                name: "辽源一村居委会",
                code: "010",
            },
            VillageCode {
                name: "五环居委会",
                code: "011",
            },
            VillageCode {
                name: "辽源四村居委会",
                code: "012",
            },
            VillageCode {
                name: "金鹏居委会",
                code: "013",
            },
            VillageCode {
                name: "又一村居委会",
                code: "014",
            },
            VillageCode {
                name: "大花园居委会",
                code: "015",
            },
            VillageCode {
                name: "星泰居委会",
                code: "016",
            },
            VillageCode {
                name: "阳明居委会",
                code: "017",
            },
            VillageCode {
                name: "金上海居委会",
                code: "018",
            },
            VillageCode {
                name: "辽昆居委会",
                code: "019",
            },
            VillageCode {
                name: "恒阳居委会",
                code: "020",
            },
            VillageCode {
                name: "海上海居委会",
                code: "021",
            },
            VillageCode {
                name: "大连路居委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "四平路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "控江路二零二六弄居委会",
                code: "001",
            },
            VillageCode {
                name: "鞍山三村居委会",
                code: "002",
            },
            VillageCode {
                name: "鞍山四村第一居委会",
                code: "003",
            },
            VillageCode {
                name: "鞍山四村第二居委会",
                code: "004",
            },
            VillageCode {
                name: "鞍山四村第三居委会",
                code: "005",
            },
            VillageCode {
                name: "鞍山五村居委会",
                code: "006",
            },
            VillageCode {
                name: "鞍山六村居委会",
                code: "007",
            },
            VillageCode {
                name: "鞍山七村居委会",
                code: "008",
            },
            VillageCode {
                name: "鞍山八村居委会",
                code: "009",
            },
            VillageCode {
                name: "抚顺路三六三弄居委会",
                code: "010",
            },
            VillageCode {
                name: "大连西路四弄居委会",
                code: "011",
            },
            VillageCode {
                name: "同济新村居委会",
                code: "012",
            },
            VillageCode {
                name: "密云路居委会",
                code: "013",
            },
            VillageCode {
                name: "公交新村居委会",
                code: "014",
            },
            VillageCode {
                name: "铁岭路五十弄居委会",
                code: "015",
            },
            VillageCode {
                name: "铁岭路九十弄居委会",
                code: "016",
            },
            VillageCode {
                name: "鞍山一村第二居委会",
                code: "017",
            },
            VillageCode {
                name: "鞍山一村第三居委会",
                code: "018",
            },
            VillageCode {
                name: "鞍山路三一零弄居委会",
                code: "019",
            },
            VillageCode {
                name: "金安居委会",
                code: "020",
            },
            VillageCode {
                name: "同济绿园居委会",
                code: "021",
            },
            VillageCode {
                name: "鞍山一村第一居委会",
                code: "022",
            },
            VillageCode {
                name: "和平花苑居委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "控江路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "控一居委会",
                code: "001",
            },
            VillageCode {
                name: "欣辉居委会",
                code: "002",
            },
            VillageCode {
                name: "控江四村第一居委会",
                code: "003",
            },
            VillageCode {
                name: "控江四村第二居委会",
                code: "004",
            },
            VillageCode {
                name: "控江二村一零七弄居委会",
                code: "005",
            },
            VillageCode {
                name: "沧州路一八零弄居委会",
                code: "006",
            },
            VillageCode {
                name: "双阳居委会",
                code: "007",
            },
            VillageCode {
                name: "双花居委会",
                code: "008",
            },
            VillageCode {
                name: "控江路一一九七弄居委会",
                code: "009",
            },
            VillageCode {
                name: "新凤城居委会",
                code: "010",
            },
            VillageCode {
                name: "凤城二村第一居委会",
                code: "011",
            },
            VillageCode {
                name: "凤城三村第一居委会",
                code: "012",
            },
            VillageCode {
                name: "凤城三村第二居委会",
                code: "013",
            },
            VillageCode {
                name: "凤城三村第三居委会",
                code: "014",
            },
            VillageCode {
                name: "抚岭居委会",
                code: "015",
            },
            VillageCode {
                name: "黄兴居委会",
                code: "016",
            },
            VillageCode {
                name: "凤南居委会",
                code: "017",
            },
            VillageCode {
                name: "凤新居委会",
                code: "018",
            },
            VillageCode {
                name: "凤城二村第二居委会",
                code: "019",
            },
            VillageCode {
                name: "凤城三村第四居委会",
                code: "020",
            },
            VillageCode {
                name: "控江路一千二百弄居委会",
                code: "021",
            },
            VillageCode {
                name: "望春花居委会",
                code: "022",
            },
            VillageCode {
                name: "凤联居委会",
                code: "023",
            },
            VillageCode {
                name: "大运盛居委会",
                code: "024",
            },
            VillageCode {
                name: "紫城居委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "长白新村街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "上理居委会",
                code: "001",
            },
            VillageCode {
                name: "内江大楼居委会",
                code: "002",
            },
            VillageCode {
                name: "图们路居委会",
                code: "003",
            },
            VillageCode {
                name: "长白路第二居委会",
                code: "004",
            },
            VillageCode {
                name: "长白二村第一居委会",
                code: "005",
            },
            VillageCode {
                name: "安图新村居委会",
                code: "006",
            },
            VillageCode {
                name: "松花新村居委会",
                code: "007",
            },
            VillageCode {
                name: "民治路居委会",
                code: "008",
            },
            VillageCode {
                name: "内江路三八四弄居委会",
                code: "009",
            },
            VillageCode {
                name: "广远新村居委会",
                code: "010",
            },
            VillageCode {
                name: "控江路一二一弄居委会",
                code: "011",
            },
            VillageCode {
                name: "控江路十八弄居委会",
                code: "012",
            },
            VillageCode {
                name: "松延居委会",
                code: "013",
            },
            VillageCode {
                name: "延东居委会",
                code: "014",
            },
            VillageCode {
                name: "松花江路九十五弄居委会",
                code: "015",
            },
            VillageCode {
                name: "长白新城居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "延吉新村街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "敦化路居委会",
                code: "001",
            },
            VillageCode {
                name: "控江路六四五弄居委会",
                code: "002",
            },
            VillageCode {
                name: "控江七村居委会",
                code: "003",
            },
            VillageCode {
                name: "友谊新村居委会",
                code: "004",
            },
            VillageCode {
                name: "松花江路居委会",
                code: "005",
            },
            VillageCode {
                name: "舒兰路居委会",
                code: "006",
            },
            VillageCode {
                name: "内江新村居委会",
                code: "007",
            },
            VillageCode {
                name: "长白三村居委会",
                code: "008",
            },
            VillageCode {
                name: "延吉一村居委会",
                code: "009",
            },
            VillageCode {
                name: "延吉二三村居委会",
                code: "010",
            },
            VillageCode {
                name: "延吉四村居委会",
                code: "011",
            },
            VillageCode {
                name: "延吉五六村居委会",
                code: "012",
            },
            VillageCode {
                name: "延吉七村居委会",
                code: "013",
            },
            VillageCode {
                name: "控江东三村居委会",
                code: "014",
            },
            VillageCode {
                name: "控江西三村居委会",
                code: "015",
            },
            VillageCode {
                name: "控江五村居委会",
                code: "016",
            },
            VillageCode {
                name: "杨家浜居委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "殷行街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "殷行一村居委会",
                code: "001",
            },
            VillageCode {
                name: "殷行二村居委会",
                code: "002",
            },
            VillageCode {
                name: "工农新村居委会",
                code: "003",
            },
            VillageCode {
                name: "工农二村第一居委会",
                code: "004",
            },
            VillageCode {
                name: "工农二村第二居委会",
                code: "005",
            },
            VillageCode {
                name: "工农二村第三居委会",
                code: "006",
            },
            VillageCode {
                name: "工农二村第四居委会",
                code: "007",
            },
            VillageCode {
                name: "工农三村第一居委会",
                code: "008",
            },
            VillageCode {
                name: "闸殷路第一居委会",
                code: "009",
            },
            VillageCode {
                name: "闸殷路第二居委会",
                code: "010",
            },
            VillageCode {
                name: "开鲁一村居委会",
                code: "011",
            },
            VillageCode {
                name: "开鲁二村居委会",
                code: "012",
            },
            VillageCode {
                name: "开鲁三村居委会",
                code: "013",
            },
            VillageCode {
                name: "开鲁四村居委会",
                code: "014",
            },
            VillageCode {
                name: "市光一村第一居委会",
                code: "015",
            },
            VillageCode {
                name: "市光一村第三居委会",
                code: "016",
            },
            VillageCode {
                name: "市光二村第一居委会",
                code: "017",
            },
            VillageCode {
                name: "市光二村第三居委会",
                code: "018",
            },
            VillageCode {
                name: "市光四村第二居委会",
                code: "019",
            },
            VillageCode {
                name: "国和一村第一居委会",
                code: "020",
            },
            VillageCode {
                name: "国和一村第三居委会",
                code: "021",
            },
            VillageCode {
                name: "国和二村第一居委会",
                code: "022",
            },
            VillageCode {
                name: "国和二村第三居委会",
                code: "023",
            },
            VillageCode {
                name: "民星一村第一居委会",
                code: "024",
            },
            VillageCode {
                name: "市光三村第一居委会",
                code: "025",
            },
            VillageCode {
                name: "开鲁五村居委会",
                code: "026",
            },
            VillageCode {
                name: "开鲁六村居委会",
                code: "027",
            },
            VillageCode {
                name: "市光一村第二居委会",
                code: "028",
            },
            VillageCode {
                name: "殷行路四七零弄居委会",
                code: "029",
            },
            VillageCode {
                name: "殷行路三一零弄居委会",
                code: "030",
            },
            VillageCode {
                name: "中原一村居委会",
                code: "031",
            },
            VillageCode {
                name: "闸殷路八十一弄居委会",
                code: "032",
            },
            VillageCode {
                name: "殷行路二百五十弄居委会",
                code: "033",
            },
            VillageCode {
                name: "市光四村第三居委会",
                code: "034",
            },
            VillageCode {
                name: "民星路六百弄居委会",
                code: "035",
            },
            VillageCode {
                name: "工农四村第二居委会",
                code: "036",
            },
            VillageCode {
                name: "工农三村第二居委会",
                code: "037",
            },
            VillageCode {
                name: "工农三村第三居委会",
                code: "038",
            },
            VillageCode {
                name: "工农四村第一居委会",
                code: "039",
            },
            VillageCode {
                name: "中原路九九零弄居委会",
                code: "040",
            },
            VillageCode {
                name: "市光三村第二居委会",
                code: "041",
            },
            VillageCode {
                name: "市光四村第一居委会",
                code: "042",
            },
            VillageCode {
                name: "民星一村第二居委会",
                code: "043",
            },
            VillageCode {
                name: "民星二村居委会",
                code: "044",
            },
            VillageCode {
                name: "国和一村第二居委会",
                code: "045",
            },
            VillageCode {
                name: "国和二村第二居委会",
                code: "046",
            },
            VillageCode {
                name: "城市名园居委会",
                code: "047",
            },
            VillageCode {
                name: "城市庭园居委会",
                code: "048",
            },
            VillageCode {
                name: "城市方园居委会",
                code: "049",
            },
        ],
    },
    TownCode {
        name: "大桥街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "杨家宅居委会",
                code: "001",
            },
            VillageCode {
                name: "永安里居委会",
                code: "002",
            },
            VillageCode {
                name: "周家牌路居委会",
                code: "003",
            },
            VillageCode {
                name: "富禄里居委会",
                code: "004",
            },
            VillageCode {
                name: "长隆居委会",
                code: "005",
            },
            VillageCode {
                name: "锦州湾路居委会",
                code: "006",
            },
            VillageCode {
                name: "临青路居委会",
                code: "007",
            },
            VillageCode {
                name: "双阳路居委会",
                code: "008",
            },
            VillageCode {
                name: "引翔港居委会",
                code: "009",
            },
            VillageCode {
                name: "幸福村居委会",
                code: "010",
            },
            VillageCode {
                name: "长眉居委会",
                code: "011",
            },
            VillageCode {
                name: "宁武路居委会",
                code: "012",
            },
            VillageCode {
                name: "河间路居委会",
                code: "013",
            },
            VillageCode {
                name: "中王家宅居委会",
                code: "014",
            },
            VillageCode {
                name: "平眉居委会",
                code: "015",
            },
            VillageCode {
                name: "新华里居委会",
                code: "016",
            },
            VillageCode {
                name: "紫华居委会",
                code: "017",
            },
            VillageCode {
                name: "渭南路居委会",
                code: "018",
            },
            VillageCode {
                name: "广杭居委会",
                code: "019",
            },
            VillageCode {
                name: "银河苑居委会",
                code: "020",
            },
            VillageCode {
                name: "长阳新苑居委会",
                code: "021",
            },
            VillageCode {
                name: "富阳居委会",
                code: "022",
            },
            VillageCode {
                name: "月坊居委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "五角场街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "武东居委会",
                code: "001",
            },
            VillageCode {
                name: "五一居委会",
                code: "002",
            },
            VillageCode {
                name: "政民居委会",
                code: "003",
            },
            VillageCode {
                name: "建新居委会",
                code: "004",
            },
            VillageCode {
                name: "复旦居委会",
                code: "005",
            },
            VillageCode {
                name: "国定一居委会",
                code: "006",
            },
            VillageCode {
                name: "铁路新村居委会",
                code: "007",
            },
            VillageCode {
                name: "四平居委会",
                code: "008",
            },
            VillageCode {
                name: "四平一居委会",
                code: "009",
            },
            VillageCode {
                name: "国年居委会",
                code: "010",
            },
            VillageCode {
                name: "国顺居委会",
                code: "011",
            },
            VillageCode {
                name: "国权居委会",
                code: "012",
            },
            VillageCode {
                name: "航天居委会",
                code: "013",
            },
            VillageCode {
                name: "政翔殷居委会",
                code: "014",
            },
            VillageCode {
                name: "东郸居委会",
                code: "015",
            },
            VillageCode {
                name: "蓝天居委会",
                code: "016",
            },
            VillageCode {
                name: "三门路居委会",
                code: "017",
            },
            VillageCode {
                name: "吉浦居委会",
                code: "018",
            },
            VillageCode {
                name: "南茶园居委会",
                code: "019",
            },
            VillageCode {
                name: "邯郸路居委会",
                code: "020",
            },
            VillageCode {
                name: "国权北路居委会",
                code: "021",
            },
            VillageCode {
                name: "仁德路居委会",
                code: "022",
            },
            VillageCode {
                name: "北茶园路居委会",
                code: "023",
            },
            VillageCode {
                name: "政通路第一居委会",
                code: "024",
            },
            VillageCode {
                name: "财大居委会",
                code: "025",
            },
            VillageCode {
                name: "国权路第一居委会",
                code: "026",
            },
            VillageCode {
                name: "文化花园居委会",
                code: "027",
            },
            VillageCode {
                name: "三湘居委会",
                code: "028",
            },
            VillageCode {
                name: "正文居委会",
                code: "029",
            },
            VillageCode {
                name: "汇元坊居委会",
                code: "030",
            },
            VillageCode {
                name: "吉浦一居委会",
                code: "031",
            },
            VillageCode {
                name: "创智坊居委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "新江湾城街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "政立路第一居委会",
                code: "001",
            },
            VillageCode {
                name: "时代花园居委会",
                code: "002",
            },
            VillageCode {
                name: "政立路第二居委会",
                code: "003",
            },
            VillageCode {
                name: "建德国际公寓居委会",
                code: "004",
            },
            VillageCode {
                name: "雍景苑居委会",
                code: "005",
            },
            VillageCode {
                name: "东森涵碧居委会",
                code: "006",
            },
            VillageCode {
                name: "江湾国际公寓居委会",
                code: "007",
            },
            VillageCode {
                name: "橡树湾居委会",
                code: "008",
            },
            VillageCode {
                name: "政青路居委会",
                code: "009",
            },
            VillageCode {
                name: "睿达路居委会",
                code: "010",
            },
            VillageCode {
                name: "加州水郡居委会",
                code: "011",
            },
            VillageCode {
                name: "仁恒怡庭居委会",
                code: "012",
            },
            VillageCode {
                name: "尚景园居委会",
                code: "013",
            },
            VillageCode {
                name: "祥生泰宝居委会",
                code: "014",
            },
            VillageCode {
                name: "中建大公馆居委会",
                code: "015",
            },
            VillageCode {
                name: "九龙仓玺园居委会",
                code: "016",
            },
            VillageCode {
                name: "国安路第一居委会",
                code: "017",
            },
            VillageCode {
                name: "华庭香景园居委会",
                code: "018",
            },
            VillageCode {
                name: "尚浦名邸居委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "长海路街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "国和路第一居委会",
                code: "001",
            },
            VillageCode {
                name: "市光居委会",
                code: "002",
            },
            VillageCode {
                name: "市光路第三居委会",
                code: "003",
            },
            VillageCode {
                name: "黑山新村居委会",
                code: "004",
            },
            VillageCode {
                name: "洪东居委会",
                code: "005",
            },
            VillageCode {
                name: "梅林居委会",
                code: "006",
            },
            VillageCode {
                name: "虬江居委会",
                code: "007",
            },
            VillageCode {
                name: "翔殷路四九一弄居委会",
                code: "008",
            },
            VillageCode {
                name: "梅花村居委会",
                code: "009",
            },
            VillageCode {
                name: "长海居委会",
                code: "010",
            },
            VillageCode {
                name: "体院居委会",
                code: "011",
            },
            VillageCode {
                name: "长海一村居委会",
                code: "012",
            },
            VillageCode {
                name: "长海三村居委会",
                code: "013",
            },
            VillageCode {
                name: "浣纱三村居委会",
                code: "014",
            },
            VillageCode {
                name: "浣纱四村居委会",
                code: "015",
            },
            VillageCode {
                name: "浣纱六村居委会",
                code: "016",
            },
            VillageCode {
                name: "市京一村居委会",
                code: "017",
            },
            VillageCode {
                name: "市光路第二居委会",
                code: "018",
            },
            VillageCode {
                name: "民京路第一居委会",
                code: "019",
            },
            VillageCode {
                name: "国顺东路二十六弄居委会",
                code: "020",
            },
            VillageCode {
                name: "中翔居委会",
                code: "021",
            },
            VillageCode {
                name: "兰新居委会",
                code: "022",
            },
            VillageCode {
                name: "中原路九十九弄居委会",
                code: "023",
            },
            VillageCode {
                name: "长海二村居委会",
                code: "024",
            },
            VillageCode {
                name: "佳木斯路居委会",
                code: "025",
            },
            VillageCode {
                name: "佳木斯路三一五弄居委会",
                code: "026",
            },
            VillageCode {
                name: "中农新村居委会",
                code: "027",
            },
            VillageCode {
                name: "翔殷新村居委会",
                code: "028",
            },
            VillageCode {
                name: "教师公寓居委会",
                code: "029",
            },
            VillageCode {
                name: "佳泰居委会",
                code: "030",
            },
            VillageCode {
                name: "黄兴花园居委会",
                code: "031",
            },
            VillageCode {
                name: "世界居委会",
                code: "032",
            },
            VillageCode {
                name: "东方名城居委会",
                code: "033",
            },
            VillageCode {
                name: "佳龙居委会",
                code: "034",
            },
            VillageCode {
                name: "黄兴绿园居委会",
                code: "035",
            },
            VillageCode {
                name: "香阁丽苑居委会",
                code: "036",
            },
            VillageCode {
                name: "文化佳园居委会",
                code: "037",
            },
            VillageCode {
                name: "莱茵居委会",
                code: "038",
            },
            VillageCode {
                name: "民庆家园居委会",
                code: "039",
            },
            VillageCode {
                name: "星云居委会",
                code: "040",
            },
            VillageCode {
                name: "海上硕和城居委会",
                code: "041",
            },
            VillageCode {
                name: "盛世豪园居委会",
                code: "042",
            },
            VillageCode {
                name: "东岸新里居委会",
                code: "043",
            },
        ],
    },
];

static TOWNS_SJ_009: [TownCode; 14] = [
    TownCode {
        name: "江川路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "北街居委会",
                code: "001",
            },
            VillageCode {
                name: "河东居委会",
                code: "002",
            },
            VillageCode {
                name: "华江新村第一居委会",
                code: "003",
            },
            VillageCode {
                name: "华江新村第二居委会",
                code: "004",
            },
            VillageCode {
                name: "高华新村第二居委会",
                code: "005",
            },
            VillageCode {
                name: "高华新村第三居委会",
                code: "006",
            },
            VillageCode {
                name: "东风新村第一居委会",
                code: "007",
            },
            VillageCode {
                name: "东风新村第三居委会",
                code: "008",
            },
            VillageCode {
                name: "新闵新村居委会",
                code: "009",
            },
            VillageCode {
                name: "沧源新村第一居委会",
                code: "010",
            },
            VillageCode {
                name: "沧源新村第二居委会",
                code: "011",
            },
            VillageCode {
                name: "沧源新村第三居委会",
                code: "012",
            },
            VillageCode {
                name: "兰坪新村第一居委会",
                code: "013",
            },
            VillageCode {
                name: "兰坪新村第二居委会",
                code: "014",
            },
            VillageCode {
                name: "汽轮新村第一居委会",
                code: "015",
            },
            VillageCode {
                name: "汽轮新村第三居委会",
                code: "016",
            },
            VillageCode {
                name: "瑞丽新村居委会",
                code: "017",
            },
            VillageCode {
                name: "电机新村第一居委会",
                code: "018",
            },
            VillageCode {
                name: "电机新村第二居委会",
                code: "019",
            },
            VillageCode {
                name: "电机新村第四居委会",
                code: "020",
            },
            VillageCode {
                name: "电机新村第五居委会",
                code: "021",
            },
            VillageCode {
                name: "红旗新村居委会",
                code: "022",
            },
            VillageCode {
                name: "红旗新村第三居委会",
                code: "023",
            },
            VillageCode {
                name: "红旗新村第四居委会",
                code: "024",
            },
            VillageCode {
                name: "红旗新村第五居委会",
                code: "025",
            },
            VillageCode {
                name: "红旗新村第七居委会",
                code: "026",
            },
            VillageCode {
                name: "昆阳新村第一居委会",
                code: "027",
            },
            VillageCode {
                name: "昆阳新村第三居委会",
                code: "028",
            },
            VillageCode {
                name: "鹤北新村第一居委会",
                code: "029",
            },
            VillageCode {
                name: "鹤北新村第三居委会",
                code: "030",
            },
            VillageCode {
                name: "鹤庆新村第一居委会",
                code: "031",
            },
            VillageCode {
                name: "鹤庆新村第二居委会",
                code: "032",
            },
            VillageCode {
                name: "紫藤新园居委会",
                code: "033",
            },
            VillageCode {
                name: "富仕名邸居委会",
                code: "034",
            },
            VillageCode {
                name: "红旗新村第六居委会",
                code: "035",
            },
            VillageCode {
                name: "安宁路第一居委会",
                code: "036",
            },
            VillageCode {
                name: "鹤北新村第四居委会",
                code: "037",
            },
            VillageCode {
                name: "金平新村居委会",
                code: "038",
            },
            VillageCode {
                name: "南洋博仕欣居居委会",
                code: "039",
            },
            VillageCode {
                name: "好第坊文馨苑居委会",
                code: "040",
            },
            VillageCode {
                name: "假日景苑居委会",
                code: "041",
            },
            VillageCode {
                name: "合生城邦城第一居委会",
                code: "042",
            },
            VillageCode {
                name: "合生城邦城第二居委会",
                code: "043",
            },
            VillageCode {
                name: "合生城邦城第三居委会",
                code: "044",
            },
            VillageCode {
                name: "合生城邦城第四居委会",
                code: "045",
            },
            VillageCode {
                name: "永平南路第一居委会",
                code: "046",
            },
            VillageCode {
                name: "凤凰景苑居委会",
                code: "047",
            },
            VillageCode {
                name: "满庭春雅苑居委会",
                code: "048",
            },
            VillageCode {
                name: "瑞丽海赋居委会",
                code: "049",
            },
        ],
    },
    TownCode {
        name: "古美街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "古美一村居委会",
                code: "001",
            },
            VillageCode {
                name: "古美三村居委会",
                code: "002",
            },
            VillageCode {
                name: "古美四村居委会",
                code: "003",
            },
            VillageCode {
                name: "古美七村居委会",
                code: "004",
            },
            VillageCode {
                name: "古美八村居委会",
                code: "005",
            },
            VillageCode {
                name: "古美十村居委会",
                code: "006",
            },
            VillageCode {
                name: "平阳一村居委会",
                code: "007",
            },
            VillageCode {
                name: "平阳三村居委会",
                code: "008",
            },
            VillageCode {
                name: "平阳四村居委会",
                code: "009",
            },
            VillageCode {
                name: "平阳六村居委会",
                code: "010",
            },
            VillageCode {
                name: "平吉一村居委会",
                code: "011",
            },
            VillageCode {
                name: "平吉二村居委会",
                code: "012",
            },
            VillageCode {
                name: "平南新村第一居委会",
                code: "013",
            },
            VillageCode {
                name: "平南新村第二居委会",
                code: "014",
            },
            VillageCode {
                name: "东兰新村第一居委会",
                code: "015",
            },
            VillageCode {
                name: "华梅花苑居委会",
                code: "016",
            },
            VillageCode {
                name: "梅莲苑居委会",
                code: "017",
            },
            VillageCode {
                name: "古龙一村居委会",
                code: "018",
            },
            VillageCode {
                name: "平吉三村居委会",
                code: "019",
            },
            VillageCode {
                name: "平吉四村居委会",
                code: "020",
            },
            VillageCode {
                name: "平吉六村居委会",
                code: "021",
            },
            VillageCode {
                name: "平南新村第三居委会",
                code: "022",
            },
            VillageCode {
                name: "华一新城居委会",
                code: "023",
            },
            VillageCode {
                name: "平阳五村居委会",
                code: "024",
            },
            VillageCode {
                name: "古龙二村居委会",
                code: "025",
            },
            VillageCode {
                name: "东兰新村第二居委会",
                code: "026",
            },
            VillageCode {
                name: "古美九村居委会",
                code: "027",
            },
            VillageCode {
                name: "东兰新村第三居委会",
                code: "028",
            },
            VillageCode {
                name: "古龙三村居委会",
                code: "029",
            },
            VillageCode {
                name: "古龙四村居委会",
                code: "030",
            },
            VillageCode {
                name: "古龙五村居委会",
                code: "031",
            },
            VillageCode {
                name: "古龙六村居委会",
                code: "032",
            },
            VillageCode {
                name: "东兰新村第四居委会",
                code: "033",
            },
            VillageCode {
                name: "平阳二村居委会",
                code: "034",
            },
            VillageCode {
                name: "平吉五村居委会",
                code: "035",
            },
            VillageCode {
                name: "万源城第三居委会",
                code: "036",
            },
            VillageCode {
                name: "万源城第二居委会",
                code: "037",
            },
            VillageCode {
                name: "万源城第四居委会",
                code: "038",
            },
            VillageCode {
                name: "万源城第一居委会",
                code: "039",
            },
            VillageCode {
                name: "同济华城居委会",
                code: "040",
            },
            VillageCode {
                name: "平阳春江居委会",
                code: "041",
            },
            VillageCode {
                name: "平阳东方居委会",
                code: "042",
            },
        ],
    },
    TownCode {
        name: "新虹街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "航华一村第二居委会",
                code: "001",
            },
            VillageCode {
                name: "航华一村第五居委会",
                code: "002",
            },
            VillageCode {
                name: "航华一村第六居委会",
                code: "003",
            },
            VillageCode {
                name: "航华一村第七居委会",
                code: "004",
            },
            VillageCode {
                name: "沙茂居委会",
                code: "005",
            },
            VillageCode {
                name: "华美路居委会",
                code: "006",
            },
            VillageCode {
                name: "华美路第二居委会",
                code: "007",
            },
            VillageCode {
                name: "爱博一村居委会",
                code: "008",
            },
            VillageCode {
                name: "爱博二村居委会",
                code: "009",
            },
            VillageCode {
                name: "爱博三村居委会",
                code: "010",
            },
            VillageCode {
                name: "爱博五村居委会",
                code: "011",
            },
            VillageCode {
                name: "爱博四村居委会",
                code: "012",
            },
            VillageCode {
                name: "万科润园居委会",
                code: "013",
            },
            VillageCode {
                name: "申贵路居委会",
                code: "014",
            },
            VillageCode {
                name: "光华村村委会",
                code: "015",
            },
            VillageCode {
                name: "范巷村村委会",
                code: "016",
            },
            VillageCode {
                name: "红星村村委会",
                code: "017",
            },
            VillageCode {
                name: "新家弄村村委会",
                code: "018",
            },
            VillageCode {
                name: "陶家角村村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "浦锦街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "世博家园第一居委会",
                code: "001",
            },
            VillageCode {
                name: "世博家园第二居委会",
                code: "002",
            },
            VillageCode {
                name: "世博家园第三居委会",
                code: "003",
            },
            VillageCode {
                name: "世博家园第四居委会",
                code: "004",
            },
            VillageCode {
                name: "世博家园第五居委会",
                code: "005",
            },
            VillageCode {
                name: "世博家园第六居委会",
                code: "006",
            },
            VillageCode {
                name: "陈行居委会",
                code: "007",
            },
            VillageCode {
                name: "景江苑居委会",
                code: "008",
            },
            VillageCode {
                name: "景舒苑第一居委会",
                code: "009",
            },
            VillageCode {
                name: "景舒苑第二居委会",
                code: "010",
            },
            VillageCode {
                name: "景舒苑第三居委会",
                code: "011",
            },
            VillageCode {
                name: "滨浦第一居委会",
                code: "012",
            },
            VillageCode {
                name: "滨浦第二居委会",
                code: "013",
            },
            VillageCode {
                name: "新浦江城第一居委会",
                code: "014",
            },
            VillageCode {
                name: "一品漫城第一居委会",
                code: "015",
            },
            VillageCode {
                name: "新浦江城第二居委会",
                code: "016",
            },
            VillageCode {
                name: "一品漫城第二居委会",
                code: "017",
            },
            VillageCode {
                name: "茉莉名邸居委会",
                code: "018",
            },
            VillageCode {
                name: "浦江颐城居委会",
                code: "019",
            },
            VillageCode {
                name: "滨浦新苑第三居委会",
                code: "020",
            },
            VillageCode {
                name: "滨浦新苑第四居委会",
                code: "021",
            },
            VillageCode {
                name: "浦恒馨苑居委会",
                code: "022",
            },
            VillageCode {
                name: "浦江坤庭居委会",
                code: "023",
            },
            VillageCode {
                name: "浦秀苑居委会",
                code: "024",
            },
            VillageCode {
                name: "新浦江城第三居委会",
                code: "025",
            },
            VillageCode {
                name: "丁连村村委会",
                code: "026",
            },
            VillageCode {
                name: "丰收村村委会",
                code: "027",
            },
            VillageCode {
                name: "芦胜村村委会",
                code: "028",
            },
            VillageCode {
                name: "近浦村村委会",
                code: "029",
            },
            VillageCode {
                name: "郁宋村村委会",
                code: "030",
            },
            VillageCode {
                name: "浦江村村委会",
                code: "031",
            },
            VillageCode {
                name: "跃农村村委会",
                code: "032",
            },
            VillageCode {
                name: "勤俭村村委会",
                code: "033",
            },
            VillageCode {
                name: "塘口村村委会",
                code: "034",
            },
            VillageCode {
                name: "为民村村委会",
                code: "035",
            },
            VillageCode {
                name: "亭子村村委会",
                code: "036",
            },
            VillageCode {
                name: "恒星村村委会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "莘庄镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "东街居委会",
                code: "001",
            },
            VillageCode {
                name: "西街居委会",
                code: "002",
            },
            VillageCode {
                name: "莘松一村居委会",
                code: "003",
            },
            VillageCode {
                name: "莘松三村居委会",
                code: "004",
            },
            VillageCode {
                name: "莘松四村居委会",
                code: "005",
            },
            VillageCode {
                name: "莘松五村居委会",
                code: "006",
            },
            VillageCode {
                name: "莘松九村居委会",
                code: "007",
            },
            VillageCode {
                name: "绿梅一村居委会",
                code: "008",
            },
            VillageCode {
                name: "绿梅二村居委会",
                code: "009",
            },
            VillageCode {
                name: "绿梅三村居委会",
                code: "010",
            },
            VillageCode {
                name: "水清一村居委会",
                code: "011",
            },
            VillageCode {
                name: "水清三村居委会",
                code: "012",
            },
            VillageCode {
                name: "报春第一居委会",
                code: "013",
            },
            VillageCode {
                name: "报春第二居委会",
                code: "014",
            },
            VillageCode {
                name: "佳佳花园居委会",
                code: "015",
            },
            VillageCode {
                name: "黎安新村第一居委会",
                code: "016",
            },
            VillageCode {
                name: "黎安新村第二居委会",
                code: "017",
            },
            VillageCode {
                name: "西湖苑居委会",
                code: "018",
            },
            VillageCode {
                name: "沁春园二村居委会",
                code: "019",
            },
            VillageCode {
                name: "新梅花苑居委会",
                code: "020",
            },
            VillageCode {
                name: "绿世界居委会",
                code: "021",
            },
            VillageCode {
                name: "星丰苑居委会",
                code: "022",
            },
            VillageCode {
                name: "莘南花苑居委会",
                code: "023",
            },
            VillageCode {
                name: "莘城公寓居委会",
                code: "024",
            },
            VillageCode {
                name: "团结花苑居委会",
                code: "025",
            },
            VillageCode {
                name: "宝安新苑居委会",
                code: "026",
            },
            VillageCode {
                name: "新申花城居委会",
                code: "027",
            },
            VillageCode {
                name: "众众家园居委会",
                code: "028",
            },
            VillageCode {
                name: "莘城苑居委会",
                code: "029",
            },
            VillageCode {
                name: "丽华公寓居委会",
                code: "030",
            },
            VillageCode {
                name: "圣淘沙花园居委会",
                code: "031",
            },
            VillageCode {
                name: "莘纪苑居委会",
                code: "032",
            },
            VillageCode {
                name: "新梅广场居委会",
                code: "033",
            },
            VillageCode {
                name: "虹莘新村居委会",
                code: "034",
            },
            VillageCode {
                name: "春申新村居委会",
                code: "035",
            },
            VillageCode {
                name: "香树丽舍居委会",
                code: "036",
            },
            VillageCode {
                name: "沁春园一村居委会",
                code: "037",
            },
            VillageCode {
                name: "西环新村居委会",
                code: "038",
            },
            VillageCode {
                name: "世纪名门居委会",
                code: "039",
            },
            VillageCode {
                name: "名都新城居委会",
                code: "040",
            },
            VillageCode {
                name: "阳明花苑居委会",
                code: "041",
            },
            VillageCode {
                name: "康城第一居委会",
                code: "042",
            },
            VillageCode {
                name: "世纪阳光园居委会",
                code: "043",
            },
            VillageCode {
                name: "春申复地城居委会",
                code: "044",
            },
            VillageCode {
                name: "春申万科城居委会",
                code: "045",
            },
            VillageCode {
                name: "东苑新天地居委会",
                code: "046",
            },
            VillageCode {
                name: "东苑利华苑居委会",
                code: "047",
            },
            VillageCode {
                name: "沁春园第三居委会",
                code: "048",
            },
            VillageCode {
                name: "都市星城居委会",
                code: "049",
            },
            VillageCode {
                name: "上海康城第二居委会",
                code: "050",
            },
            VillageCode {
                name: "邻里苑居委会",
                code: "051",
            },
            VillageCode {
                name: "康城第三居委会",
                code: "052",
            },
            VillageCode {
                name: "康城第四居委会",
                code: "053",
            },
            VillageCode {
                name: "莘东两湾苑居委会",
                code: "054",
            },
            VillageCode {
                name: "青春村村委会",
                code: "055",
            },
        ],
    },
    TownCode {
        name: "七宝镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "塘南居委会",
                code: "001",
            },
            VillageCode {
                name: "塘北居委会",
                code: "002",
            },
            VillageCode {
                name: "蒲汇新村居委会",
                code: "003",
            },
            VillageCode {
                name: "佳宝新村第一居委会",
                code: "004",
            },
            VillageCode {
                name: "三佳花苑居委会",
                code: "005",
            },
            VillageCode {
                name: "园艺新村居委会",
                code: "006",
            },
            VillageCode {
                name: "黎明花园居委会",
                code: "007",
            },
            VillageCode {
                name: "宝仪新村居委会",
                code: "008",
            },
            VillageCode {
                name: "茂盛新村居委会",
                code: "009",
            },
            VillageCode {
                name: "万泰花园第一居委会",
                code: "010",
            },
            VillageCode {
                name: "万科城市花园居委会",
                code: "011",
            },
            VillageCode {
                name: "大上海国际花园居委会",
                code: "012",
            },
            VillageCode {
                name: "东方花园居委会",
                code: "013",
            },
            VillageCode {
                name: "富丽公寓居委会",
                code: "014",
            },
            VillageCode {
                name: "红明新村居委会",
                code: "015",
            },
            VillageCode {
                name: "京都苑居委会",
                code: "016",
            },
            VillageCode {
                name: "万兆家园居委会",
                code: "017",
            },
            VillageCode {
                name: "静安新城第一居委会",
                code: "018",
            },
            VillageCode {
                name: "静安新城第二居委会",
                code: "019",
            },
            VillageCode {
                name: "静安新城第三居委会",
                code: "020",
            },
            VillageCode {
                name: "静安新城第四居委会",
                code: "021",
            },
            VillageCode {
                name: "豪世盛地园居委会",
                code: "022",
            },
            VillageCode {
                name: "万科城市花园第二居委会",
                code: "023",
            },
            VillageCode {
                name: "莲浦府邸居委会",
                code: "024",
            },
            VillageCode {
                name: "静安新城第六居委会",
                code: "025",
            },
            VillageCode {
                name: "华林路第一居委会",
                code: "026",
            },
            VillageCode {
                name: "华林路第二居委会",
                code: "027",
            },
            VillageCode {
                name: "牡丹居委会",
                code: "028",
            },
            VillageCode {
                name: "吴宝路第一居委会",
                code: "029",
            },
            VillageCode {
                name: "吴宝路第二居委会",
                code: "030",
            },
            VillageCode {
                name: "皇都花园居委会",
                code: "031",
            },
            VillageCode {
                name: "静安新城第五居委会",
                code: "032",
            },
            VillageCode {
                name: "中春路居委会",
                code: "033",
            },
            VillageCode {
                name: "漕宝路居委会",
                code: "034",
            },
            VillageCode {
                name: "漕宝路第二居委会",
                code: "035",
            },
            VillageCode {
                name: "华林路第四居委会",
                code: "036",
            },
            VillageCode {
                name: "华林路第三居委会",
                code: "037",
            },
            VillageCode {
                name: "万科城市花园第三居委会",
                code: "038",
            },
            VillageCode {
                name: "东方花园第二居委会",
                code: "039",
            },
            VillageCode {
                name: "航华一村第一居委会",
                code: "040",
            },
            VillageCode {
                name: "航华一村第三居委会",
                code: "041",
            },
            VillageCode {
                name: "航华一村第四居委会",
                code: "042",
            },
            VillageCode {
                name: "航华二村第一居委会",
                code: "043",
            },
            VillageCode {
                name: "航华二村第三居委会",
                code: "044",
            },
            VillageCode {
                name: "航华二村第四居委会",
                code: "045",
            },
            VillageCode {
                name: "航华三村第一居委会",
                code: "046",
            },
            VillageCode {
                name: "航华四村第一居委会",
                code: "047",
            },
            VillageCode {
                name: "航华四村第二居委会",
                code: "048",
            },
            VillageCode {
                name: "航华四村第三居委会",
                code: "049",
            },
            VillageCode {
                name: "航华四村第四居委会",
                code: "050",
            },
            VillageCode {
                name: "中春路第三居委会",
                code: "051",
            },
            VillageCode {
                name: "吴宝路第三居委会",
                code: "052",
            },
            VillageCode {
                name: "中春路第二居委会",
                code: "053",
            },
            VillageCode {
                name: "七韵美地苑居委会",
                code: "054",
            },
            VillageCode {
                name: "万科第四居委会",
                code: "055",
            },
            VillageCode {
                name: "漕宝路第三居委会",
                code: "056",
            },
            VillageCode {
                name: "漕宝路第四居委会",
                code: "057",
            },
            VillageCode {
                name: "沪星村村委会",
                code: "058",
            },
            VillageCode {
                name: "七宝村村委会",
                code: "059",
            },
            VillageCode {
                name: "九星村村委会",
                code: "060",
            },
            VillageCode {
                name: "联明村村委会",
                code: "061",
            },
            VillageCode {
                name: "中华村村委会",
                code: "062",
            },
        ],
    },
    TownCode {
        name: "颛桥镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "秀龙居委会",
                code: "001",
            },
            VillageCode {
                name: "众安居委会",
                code: "002",
            },
            VillageCode {
                name: "颛溪新村第五居委会",
                code: "003",
            },
            VillageCode {
                name: "颛溪新村第八居委会",
                code: "004",
            },
            VillageCode {
                name: "银都苑第一居委会",
                code: "005",
            },
            VillageCode {
                name: "银都苑第三居委会",
                code: "006",
            },
            VillageCode {
                name: "金都新村第三居委会",
                code: "007",
            },
            VillageCode {
                name: "兴银花园居委会",
                code: "008",
            },
            VillageCode {
                name: "北桥居委会",
                code: "009",
            },
            VillageCode {
                name: "银桥花园居委会",
                code: "010",
            },
            VillageCode {
                name: "樱缘花园居委会",
                code: "011",
            },
            VillageCode {
                name: "银春花园居委会",
                code: "012",
            },
            VillageCode {
                name: "好世凤凰城居委会",
                code: "013",
            },
            VillageCode {
                name: "莘闵荣顺苑居委会",
                code: "014",
            },
            VillageCode {
                name: "繁盛苑居委会",
                code: "015",
            },
            VillageCode {
                name: "君临花园居委会",
                code: "016",
            },
            VillageCode {
                name: "众众新家园居委会",
                code: "017",
            },
            VillageCode {
                name: "日月华城居委会",
                code: "018",
            },
            VillageCode {
                name: "复地北桥城居委会",
                code: "019",
            },
            VillageCode {
                name: "君莲新城第二居委会",
                code: "020",
            },
            VillageCode {
                name: "君莲新城第一居委会",
                code: "021",
            },
            VillageCode {
                name: "骏苑居委会",
                code: "022",
            },
            VillageCode {
                name: "君莲新城第三居委会",
                code: "023",
            },
            VillageCode {
                name: "翔泰苑居委会",
                code: "024",
            },
            VillageCode {
                name: "金榜新苑居委会",
                code: "025",
            },
            VillageCode {
                name: "都市富苑居委会",
                code: "026",
            },
            VillageCode {
                name: "招商雍华苑居委会",
                code: "027",
            },
            VillageCode {
                name: "君莲新城第四居委会",
                code: "028",
            },
            VillageCode {
                name: "文博水景居委会",
                code: "029",
            },
            VillageCode {
                name: "君莲新城第五居委会",
                code: "030",
            },
            VillageCode {
                name: "星河湾居委会",
                code: "031",
            },
            VillageCode {
                name: "君莲中城苑居委会",
                code: "032",
            },
            VillageCode {
                name: "君莲新城第六居委会",
                code: "033",
            },
            VillageCode {
                name: "向阳村村委会",
                code: "034",
            },
            VillageCode {
                name: "中心村村委会",
                code: "035",
            },
            VillageCode {
                name: "北桥村村委会",
                code: "036",
            },
            VillageCode {
                name: "安乐村村委会",
                code: "037",
            },
            VillageCode {
                name: "灯塔村村委会",
                code: "038",
            },
            VillageCode {
                name: "新闵村村委会",
                code: "039",
            },
            VillageCode {
                name: "黄一村村委会",
                code: "040",
            },
            VillageCode {
                name: "光明村村委会",
                code: "041",
            },
        ],
    },
    TownCode {
        name: "华漕镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "华漕第二居委会",
                code: "001",
            },
            VillageCode {
                name: "紫薇新村居委会",
                code: "002",
            },
            VillageCode {
                name: "诸新路居委会",
                code: "003",
            },
            VillageCode {
                name: "诸翟居委会",
                code: "004",
            },
            VillageCode {
                name: "金丰城第一居委会",
                code: "005",
            },
            VillageCode {
                name: "纪王居委会",
                code: "006",
            },
            VillageCode {
                name: "银杏新村居委会",
                code: "007",
            },
            VillageCode {
                name: "美邻苑居委会",
                code: "008",
            },
            VillageCode {
                name: "南华路居委会",
                code: "009",
            },
            VillageCode {
                name: "九韵城居委会",
                code: "010",
            },
            VillageCode {
                name: "西郊城第一居委会",
                code: "011",
            },
            VillageCode {
                name: "西郊虹韵城居委会",
                code: "012",
            },
            VillageCode {
                name: "爱博六村居委会",
                code: "013",
            },
            VillageCode {
                name: "闵北路居委会",
                code: "014",
            },
            VillageCode {
                name: "爱博七村居委会",
                code: "015",
            },
            VillageCode {
                name: "舒雅苑居委会",
                code: "016",
            },
            VillageCode {
                name: "前湾第一居委会",
                code: "017",
            },
            VillageCode {
                name: "华漕村村委会",
                code: "018",
            },
            VillageCode {
                name: "许浦村村委会",
                code: "019",
            },
            VillageCode {
                name: "王泥浜村村委会",
                code: "020",
            },
            VillageCode {
                name: "卫星村村委会",
                code: "021",
            },
            VillageCode {
                name: "赵家村村委会",
                code: "022",
            },
            VillageCode {
                name: "诸翟村村委会",
                code: "023",
            },
            VillageCode {
                name: "杨家巷村村委会",
                code: "024",
            },
            VillageCode {
                name: "朱家泾村村委会",
                code: "025",
            },
            VillageCode {
                name: "纪王村村委会",
                code: "026",
            },
            VillageCode {
                name: "红卫村村委会",
                code: "027",
            },
            VillageCode {
                name: "鹫山村村委会",
                code: "028",
            },
            VillageCode {
                name: "纪东村村委会",
                code: "029",
            },
            VillageCode {
                name: "纪西村村委会",
                code: "030",
            },
            VillageCode {
                name: "陈家角村村委会",
                code: "031",
            },
            VillageCode {
                name: "石皮弄村村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "虹桥镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "虹桥居委会",
                code: "001",
            },
            VillageCode {
                name: "河南居委会",
                code: "002",
            },
            VillageCode {
                name: "河西居委会",
                code: "003",
            },
            VillageCode {
                name: "虹鹿居委会",
                code: "004",
            },
            VillageCode {
                name: "红春公寓居委会",
                code: "005",
            },
            VillageCode {
                name: "上虹新村居委会",
                code: "006",
            },
            VillageCode {
                name: "华光花园居委会",
                code: "007",
            },
            VillageCode {
                name: "古北新城居委会",
                code: "008",
            },
            VillageCode {
                name: "海申花园居委会",
                code: "009",
            },
            VillageCode {
                name: "振宏公寓居委会",
                code: "010",
            },
            VillageCode {
                name: "锦华公寓居委会",
                code: "011",
            },
            VillageCode {
                name: "金虹大厦居委会",
                code: "012",
            },
            VillageCode {
                name: "虹华苑居委会",
                code: "013",
            },
            VillageCode {
                name: "金斯花园居委会",
                code: "014",
            },
            VillageCode {
                name: "锦绣江南家园居委会",
                code: "015",
            },
            VillageCode {
                name: "虹桥花苑居委会",
                code: "016",
            },
            VillageCode {
                name: "井亭苑居委会",
                code: "017",
            },
            VillageCode {
                name: "古北虹苑居委会",
                code: "018",
            },
            VillageCode {
                name: "名都城居委会",
                code: "019",
            },
            VillageCode {
                name: "万源新城居委会",
                code: "020",
            },
            VillageCode {
                name: "龙柏一村第一居委会",
                code: "021",
            },
            VillageCode {
                name: "龙柏一村第二居委会",
                code: "022",
            },
            VillageCode {
                name: "龙柏二村居委会",
                code: "023",
            },
            VillageCode {
                name: "兰竹居委会",
                code: "024",
            },
            VillageCode {
                name: "龙柏三村居委会",
                code: "025",
            },
            VillageCode {
                name: "先锋公寓居委会",
                code: "026",
            },
            VillageCode {
                name: "龙柏四村居委会",
                code: "027",
            },
            VillageCode {
                name: "龙柏五村居委会",
                code: "028",
            },
            VillageCode {
                name: "龙柏六村居委会",
                code: "029",
            },
            VillageCode {
                name: "龙柏七村居委会",
                code: "030",
            },
            VillageCode {
                name: "金汇华光城居委会",
                code: "031",
            },
            VillageCode {
                name: "金汇花园居委会",
                code: "032",
            },
            VillageCode {
                name: "西郊居委会",
                code: "033",
            },
            VillageCode {
                name: "古北尚郡居委会",
                code: "034",
            },
            VillageCode {
                name: "东苑居委会",
                code: "035",
            },
            VillageCode {
                name: "紫藤居委会",
                code: "036",
            },
            VillageCode {
                name: "名都古北居委会",
                code: "037",
            },
            VillageCode {
                name: "金鹰华庭居委会",
                code: "038",
            },
            VillageCode {
                name: "风度国际居委会",
                code: "039",
            },
        ],
    },
    TownCode {
        name: "梅陇镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "梅陇第一居委会",
                code: "001",
            },
            VillageCode {
                name: "梅陇第二居委会",
                code: "002",
            },
            VillageCode {
                name: "梅陇第三居委会",
                code: "003",
            },
            VillageCode {
                name: "梅陇第四居委会",
                code: "004",
            },
            VillageCode {
                name: "梅陇第五居委会",
                code: "005",
            },
            VillageCode {
                name: "梅陇第六居委会",
                code: "006",
            },
            VillageCode {
                name: "梅陇第七居委会",
                code: "007",
            },
            VillageCode {
                name: "朱行第一居委会",
                code: "008",
            },
            VillageCode {
                name: "朱行第二居委会",
                code: "009",
            },
            VillageCode {
                name: "南方新村第一居委会",
                code: "010",
            },
            VillageCode {
                name: "南方新村第二居委会",
                code: "011",
            },
            VillageCode {
                name: "南方新村第三居委会",
                code: "012",
            },
            VillageCode {
                name: "罗阳新村第一居委会",
                code: "013",
            },
            VillageCode {
                name: "罗阳新村第二居委会",
                code: "014",
            },
            VillageCode {
                name: "罗阳新村第四居委会",
                code: "015",
            },
            VillageCode {
                name: "罗阳新村第五居委会",
                code: "016",
            },
            VillageCode {
                name: "罗阳新村第六居委会",
                code: "017",
            },
            VillageCode {
                name: "罗阳新村第七居委会",
                code: "018",
            },
            VillageCode {
                name: "普乐新村第一居委会",
                code: "019",
            },
            VillageCode {
                name: "普乐新村第二居委会",
                code: "020",
            },
            VillageCode {
                name: "上陇新村居委会",
                code: "021",
            },
            VillageCode {
                name: "鸿福新村居委会",
                code: "022",
            },
            VillageCode {
                name: "莲花新村居委会",
                code: "023",
            },
            VillageCode {
                name: "莲花公寓居委会",
                code: "024",
            },
            VillageCode {
                name: "罗秀苑居委会",
                code: "025",
            },
            VillageCode {
                name: "罗锦苑居委会",
                code: "026",
            },
            VillageCode {
                name: "蔷薇新村第一居委会",
                code: "027",
            },
            VillageCode {
                name: "蔷薇新村第二居委会",
                code: "028",
            },
            VillageCode {
                name: "蔷薇新村第三居委会",
                code: "029",
            },
            VillageCode {
                name: "未名园居委会",
                code: "030",
            },
            VillageCode {
                name: "曹行居委会",
                code: "031",
            },
            VillageCode {
                name: "华唐苑居委会",
                code: "032",
            },
            VillageCode {
                name: "虹景苑居委会",
                code: "033",
            },
            VillageCode {
                name: "源梦苑居委会",
                code: "034",
            },
            VillageCode {
                name: "罗阳新村第三居委会",
                code: "035",
            },
            VillageCode {
                name: "望族苑居委会",
                code: "036",
            },
            VillageCode {
                name: "嘉和花苑居委会",
                code: "037",
            },
            VillageCode {
                name: "高兴花园第一居委会",
                code: "038",
            },
            VillageCode {
                name: "高兴花园第二居委会",
                code: "039",
            },
            VillageCode {
                name: "高兴花园第三居委会",
                code: "040",
            },
            VillageCode {
                name: "世纪苑居委会",
                code: "041",
            },
            VillageCode {
                name: "上海欣苑居委会",
                code: "042",
            },
            VillageCode {
                name: "罗阳新村第八居委会",
                code: "043",
            },
            VillageCode {
                name: "上海春城第一居委会",
                code: "044",
            },
            VillageCode {
                name: "春申花园居委会",
                code: "045",
            },
            VillageCode {
                name: "中梅苑居委会",
                code: "046",
            },
            VillageCode {
                name: "望族新苑居委会",
                code: "047",
            },
            VillageCode {
                name: "罗阳新村第九居委会",
                code: "048",
            },
            VillageCode {
                name: "春申景城居委会",
                code: "049",
            },
            VillageCode {
                name: "上海春城第二居委会",
                code: "050",
            },
            VillageCode {
                name: "都市宜家苑居委会",
                code: "051",
            },
            VillageCode {
                name: "南方城居委会",
                code: "052",
            },
            VillageCode {
                name: "银兆银都居委会",
                code: "053",
            },
            VillageCode {
                name: "燕南麒麟居委会",
                code: "054",
            },
            VillageCode {
                name: "银泰苑居委会",
                code: "055",
            },
            VillageCode {
                name: "银河新都居委会",
                code: "056",
            },
            VillageCode {
                name: "金都路居委会",
                code: "057",
            },
            VillageCode {
                name: "锦梅馨苑居委会",
                code: "058",
            },
            VillageCode {
                name: "上海晶城第一居委会",
                code: "059",
            },
            VillageCode {
                name: "上海晶城第二居委会",
                code: "060",
            },
            VillageCode {
                name: "中海寰宇居委会",
                code: "061",
            },
            VillageCode {
                name: "上海晶城第三居委会",
                code: "062",
            },
            VillageCode {
                name: "梅香苑居委会",
                code: "063",
            },
            VillageCode {
                name: "景华新苑居委会",
                code: "064",
            },
            VillageCode {
                name: "华一村村委会",
                code: "065",
            },
            VillageCode {
                name: "张慕村村委会",
                code: "066",
            },
            VillageCode {
                name: "行西村村委会",
                code: "067",
            },
            VillageCode {
                name: "集心村村委会",
                code: "068",
            },
            VillageCode {
                name: "行南村村委会",
                code: "069",
            },
            VillageCode {
                name: "曙建村村委会",
                code: "070",
            },
            VillageCode {
                name: "五一村村委会",
                code: "071",
            },
            VillageCode {
                name: "永联村村委会",
                code: "072",
            },
            VillageCode {
                name: "民建村村委会",
                code: "073",
            },
            VillageCode {
                name: "车沟村村委会",
                code: "074",
            },
            VillageCode {
                name: "双溪村村委会",
                code: "075",
            },
            VillageCode {
                name: "爱国村村委会",
                code: "076",
            },
            VillageCode {
                name: "许泾村村委会",
                code: "077",
            },
            VillageCode {
                name: "曹行村村委会",
                code: "078",
            },
            VillageCode {
                name: "曹中村村委会",
                code: "079",
            },
        ],
    },
    TownCode {
        name: "吴泾镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "吴泾新村居委会",
                code: "001",
            },
            VillageCode {
                name: "双柏新村居委会",
                code: "002",
            },
            VillageCode {
                name: "氯碱新村居委会",
                code: "003",
            },
            VillageCode {
                name: "通海新村居委会",
                code: "004",
            },
            VillageCode {
                name: "永南新村居委会",
                code: "005",
            },
            VillageCode {
                name: "永北新村居委会",
                code: "006",
            },
            VillageCode {
                name: "新华新村居委会",
                code: "007",
            },
            VillageCode {
                name: "宝秀路居委会",
                code: "008",
            },
            VillageCode {
                name: "紫晶南园居委会",
                code: "009",
            },
            VillageCode {
                name: "枫桦景苑居委会",
                code: "010",
            },
            VillageCode {
                name: "万科阳光苑居委会",
                code: "011",
            },
            VillageCode {
                name: "虹梅新苑第一居委会",
                code: "012",
            },
            VillageCode {
                name: "虹梅新苑第二居委会",
                code: "013",
            },
            VillageCode {
                name: "虹梅景苑第一居委会",
                code: "014",
            },
            VillageCode {
                name: "虹梅景苑第二居委会",
                code: "015",
            },
            VillageCode {
                name: "嘉怡水岸居委会",
                code: "016",
            },
            VillageCode {
                name: "塘泾南苑居委会",
                code: "017",
            },
            VillageCode {
                name: "塘泾北苑居委会",
                code: "018",
            },
            VillageCode {
                name: "紫竹半岛居委会",
                code: "019",
            },
            VillageCode {
                name: "永德宝邸居委会",
                code: "020",
            },
            VillageCode {
                name: "幸福村村委会",
                code: "021",
            },
            VillageCode {
                name: "英武村村委会",
                code: "022",
            },
            VillageCode {
                name: "星火村村委会",
                code: "023",
            },
            VillageCode {
                name: "友爱村村委会",
                code: "024",
            },
            VillageCode {
                name: "共和村村委会",
                code: "025",
            },
            VillageCode {
                name: "和平村村委会",
                code: "026",
            },
            VillageCode {
                name: "新建村村委会",
                code: "027",
            },
            VillageCode {
                name: "塘湾村村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "马桥镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "马桥居委会",
                code: "001",
            },
            VillageCode {
                name: "华银坊居委会",
                code: "002",
            },
            VillageCode {
                name: "飞碟苑居委会",
                code: "003",
            },
            VillageCode {
                name: "元吉新村居委会",
                code: "004",
            },
            VillageCode {
                name: "元祥新村居委会",
                code: "005",
            },
            VillageCode {
                name: "夏朵园居委会",
                code: "006",
            },
            VillageCode {
                name: "敬南路居委会",
                code: "007",
            },
            VillageCode {
                name: "茜昆路居委会",
                code: "008",
            },
            VillageCode {
                name: "景城乐康苑居委会",
                code: "009",
            },
            VillageCode {
                name: "景城银春苑居委会",
                code: "010",
            },
            VillageCode {
                name: "景城保利佳苑居委会",
                code: "011",
            },
            VillageCode {
                name: "景城保利雅苑居委会",
                code: "012",
            },
            VillageCode {
                name: "景城馨苑居委会",
                code: "013",
            },
            VillageCode {
                name: "景城品雅苑居委会",
                code: "014",
            },
            VillageCode {
                name: "景城银康苑居委会",
                code: "015",
            },
            VillageCode {
                name: "景城四季悦园居委会",
                code: "016",
            },
            VillageCode {
                name: "景城和苑居委会",
                code: "017",
            },
            VillageCode {
                name: "银杏里居委会",
                code: "018",
            },
            VillageCode {
                name: "俞塘村村委会",
                code: "019",
            },
            VillageCode {
                name: "工农村村委会",
                code: "020",
            },
            VillageCode {
                name: "旗忠村村委会",
                code: "021",
            },
            VillageCode {
                name: "同心村村委会",
                code: "022",
            },
            VillageCode {
                name: "民主村村委会",
                code: "023",
            },
            VillageCode {
                name: "吴会村村委会",
                code: "024",
            },
            VillageCode {
                name: "金星村村委会",
                code: "025",
            },
            VillageCode {
                name: "彭渡村村委会",
                code: "026",
            },
            VillageCode {
                name: "友好村村委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "浦江镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "杜行居委会",
                code: "001",
            },
            VillageCode {
                name: "鲁汇居委会",
                code: "002",
            },
            VillageCode {
                name: "闵浦第三居委会",
                code: "003",
            },
            VillageCode {
                name: "闵浦新苑第二居委会",
                code: "004",
            },
            VillageCode {
                name: "新汇绿苑居委会",
                code: "005",
            },
            VillageCode {
                name: "宝邸第一居委会",
                code: "006",
            },
            VillageCode {
                name: "浦航新城第二居委会",
                code: "007",
            },
            VillageCode {
                name: "浦航新城第三居委会",
                code: "008",
            },
            VillageCode {
                name: "浦航新城第四居委会",
                code: "009",
            },
            VillageCode {
                name: "浦航新城第五居委会",
                code: "010",
            },
            VillageCode {
                name: "浦航新城第六居委会",
                code: "011",
            },
            VillageCode {
                name: "浦航第七居委会",
                code: "012",
            },
            VillageCode {
                name: "智汇园居委会",
                code: "013",
            },
            VillageCode {
                name: "欣佳宝邸居委会",
                code: "014",
            },
            VillageCode {
                name: "闵浦第一居委会",
                code: "015",
            },
            VillageCode {
                name: "红梅苑居委会",
                code: "016",
            },
            VillageCode {
                name: "浦润苑居委会",
                code: "017",
            },
            VillageCode {
                name: "浦江馨都居委会",
                code: "018",
            },
            VillageCode {
                name: "瑞和华苑居委会",
                code: "019",
            },
            VillageCode {
                name: "汇秀景苑居委会",
                code: "020",
            },
            VillageCode {
                name: "聚缘居委会",
                code: "021",
            },
            VillageCode {
                name: "中虹浦江苑居委会",
                code: "022",
            },
            VillageCode {
                name: "瑞和城第三居委会",
                code: "023",
            },
            VillageCode {
                name: "永康城第一居委会",
                code: "024",
            },
            VillageCode {
                name: "永康城第七居委会",
                code: "025",
            },
            VillageCode {
                name: "永康城第八居委会",
                code: "026",
            },
            VillageCode {
                name: "永康城第九居委会",
                code: "027",
            },
            VillageCode {
                name: "浦江瑞和城第五居委会",
                code: "028",
            },
            VillageCode {
                name: "浦江宝邸第二居委会",
                code: "029",
            },
            VillageCode {
                name: "浦航新城第八居委会",
                code: "030",
            },
            VillageCode {
                name: "永康城第二居委会",
                code: "031",
            },
            VillageCode {
                name: "永康城第三居委会",
                code: "032",
            },
            VillageCode {
                name: "永康城第四居委会",
                code: "033",
            },
            VillageCode {
                name: "永康城第五居委会",
                code: "034",
            },
            VillageCode {
                name: "永康城第六居委会",
                code: "035",
            },
            VillageCode {
                name: "瑞和城第二居委会",
                code: "036",
            },
            VillageCode {
                name: "瑞和城第四居委会",
                code: "037",
            },
            VillageCode {
                name: "瑞和城第六居委会",
                code: "038",
            },
            VillageCode {
                name: "瑞和雅苑第一居委会",
                code: "039",
            },
            VillageCode {
                name: "瑞和城第一居委会",
                code: "040",
            },
            VillageCode {
                name: "瑞和雅苑第二居委会",
                code: "041",
            },
            VillageCode {
                name: "立民村村委会",
                code: "042",
            },
            VillageCode {
                name: "勤劳村村委会",
                code: "043",
            },
            VillageCode {
                name: "知新村村委会",
                code: "044",
            },
            VillageCode {
                name: "东风村村委会",
                code: "045",
            },
            VillageCode {
                name: "建中村村委会",
                code: "046",
            },
            VillageCode {
                name: "友建村村委会",
                code: "047",
            },
            VillageCode {
                name: "苏民村村委会",
                code: "048",
            },
            VillageCode {
                name: "联星村村委会",
                code: "049",
            },
            VillageCode {
                name: "群益村村委会",
                code: "050",
            },
            VillageCode {
                name: "联胜村村委会",
                code: "051",
            },
            VillageCode {
                name: "万里村村委会",
                code: "052",
            },
            VillageCode {
                name: "张行村村委会",
                code: "053",
            },
            VillageCode {
                name: "联合村村委会",
                code: "054",
            },
            VillageCode {
                name: "建东村村委会",
                code: "055",
            },
            VillageCode {
                name: "镇北村村委会",
                code: "056",
            },
            VillageCode {
                name: "胜利村村委会",
                code: "057",
            },
            VillageCode {
                name: "杜行村村委会",
                code: "058",
            },
            VillageCode {
                name: "建新村村委会",
                code: "059",
            },
            VillageCode {
                name: "建岗村村委会",
                code: "060",
            },
            VillageCode {
                name: "联民村村委会",
                code: "061",
            },
            VillageCode {
                name: "跃进村村委会",
                code: "062",
            },
            VillageCode {
                name: "革新村村委会",
                code: "063",
            },
            VillageCode {
                name: "永丰村村委会",
                code: "064",
            },
            VillageCode {
                name: "先进村村委会",
                code: "065",
            },
            VillageCode {
                name: "汇红村村委会",
                code: "066",
            },
            VillageCode {
                name: "新风村村委会",
                code: "067",
            },
            VillageCode {
                name: "光继村村委会",
                code: "068",
            },
            VillageCode {
                name: "汇西村村委会",
                code: "069",
            },
            VillageCode {
                name: "汇东村村委会",
                code: "070",
            },
            VillageCode {
                name: "正义村村委会",
                code: "071",
            },
            VillageCode {
                name: "汇北村村委会",
                code: "072",
            },
            VillageCode {
                name: "北徐村村委会",
                code: "073",
            },
            VillageCode {
                name: "汇南村村委会",
                code: "074",
            },
            VillageCode {
                name: "永新村村委会",
                code: "075",
            },
            VillageCode {
                name: "汇中村村委会",
                code: "076",
            },
        ],
    },
    TownCode {
        name: "莘庄工业区",
        code: "014",
        villages: &[
            VillageCode {
                name: "申莘新村第一居委会",
                code: "001",
            },
            VillageCode {
                name: "春辉新村居委会",
                code: "002",
            },
            VillageCode {
                name: "申莘新村第二居委会",
                code: "003",
            },
            VillageCode {
                name: "申莘新村第三居委会",
                code: "004",
            },
            VillageCode {
                name: "新源路第一居委会",
                code: "005",
            },
            VillageCode {
                name: "南郊别墅居委会",
                code: "006",
            },
            VillageCode {
                name: "鑫峰苑居委会",
                code: "007",
            },
            VillageCode {
                name: "正峰苑居委会",
                code: "008",
            },
            VillageCode {
                name: "天恒名城居委会",
                code: "009",
            },
            VillageCode {
                name: "瓶安路居委会",
                code: "010",
            },
        ],
    },
];

static TOWNS_SJ_010: [TownCode; 9] = [
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

static TOWNS_SJ_011: [TownCode; 12] = [
    TownCode {
        name: "新成路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "迎园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "仓场社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新成社区居委会",
                code: "003",
            },
            VillageCode {
                name: "源珉社区居委会",
                code: "004",
            },
            VillageCode {
                name: "嘉乐社区居委会",
                code: "005",
            },
            VillageCode {
                name: "南陈社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新望社区居委会",
                code: "007",
            },
            VillageCode {
                name: "爱里舍花园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "沧海社区居委会",
                code: "009",
            },
            VillageCode {
                name: "南塘河社区居委会",
                code: "010",
            },
            VillageCode {
                name: "墅沟社区居委会",
                code: "011",
            },
            VillageCode {
                name: "新成村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "真新街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "新丰社区居委会",
                code: "001",
            },
            VillageCode {
                name: "丰一社区居委会",
                code: "002",
            },
            VillageCode {
                name: "丰二社区居委会",
                code: "003",
            },
            VillageCode {
                name: "祁连社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金沙社区居委会",
                code: "005",
            },
            VillageCode {
                name: "新郁社区居委会",
                code: "006",
            },
            VillageCode {
                name: "金汤社区居委会",
                code: "007",
            },
            VillageCode {
                name: "铜川社区居委会",
                code: "008",
            },
            VillageCode {
                name: "双河社区居委会",
                code: "009",
            },
            VillageCode {
                name: "栅桥社区居委会",
                code: "010",
            },
            VillageCode {
                name: "吉镇社区居委会",
                code: "011",
            },
            VillageCode {
                name: "虬江社区居委会",
                code: "012",
            },
            VillageCode {
                name: "清峪社区居委会",
                code: "013",
            },
            VillageCode {
                name: "鼎秀社区居委会",
                code: "014",
            },
            VillageCode {
                name: "梅川社区居委会",
                code: "015",
            },
            VillageCode {
                name: "万镇社区居委会",
                code: "016",
            },
            VillageCode {
                name: "丰西社区居委会",
                code: "017",
            },
            VillageCode {
                name: "金鼎社区居委会",
                code: "018",
            },
            VillageCode {
                name: "丰庄社区居委会",
                code: "019",
            },
            VillageCode {
                name: "金栅桥社区居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "嘉定镇街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "汇龙潭社区居委会",
                code: "001",
            },
            VillageCode {
                name: "州桥社区居委会",
                code: "002",
            },
            VillageCode {
                name: "叶池社区居委会",
                code: "003",
            },
            VillageCode {
                name: "李园一村社区居委会",
                code: "004",
            },
            VillageCode {
                name: "李园二村社区居委会",
                code: "005",
            },
            VillageCode {
                name: "嘉中社区居委会",
                code: "006",
            },
            VillageCode {
                name: "小囡桥社区居委会",
                code: "007",
            },
            VillageCode {
                name: "三皇桥社区居委会",
                code: "008",
            },
            VillageCode {
                name: "花园弄社区居委会",
                code: "009",
            },
            VillageCode {
                name: "西大社区居委会",
                code: "010",
            },
            VillageCode {
                name: "侯黄桥社区居委会",
                code: "011",
            },
            VillageCode {
                name: "梅园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "丽景社区居委会",
                code: "013",
            },
            VillageCode {
                name: "桃园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "秋霞社区居委会",
                code: "015",
            },
            VillageCode {
                name: "塔城路社区居委会",
                code: "016",
            },
            VillageCode {
                name: "银杏社区居委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "南翔镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "古猗园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "德华社区居委会",
                code: "002",
            },
            VillageCode {
                name: "德园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "云翔社区居委会",
                code: "004",
            },
            VillageCode {
                name: "南华社区居委会",
                code: "005",
            },
            VillageCode {
                name: "白鹤社区居委会",
                code: "006",
            },
            VillageCode {
                name: "翔华社区居委会",
                code: "007",
            },
            VillageCode {
                name: "虹翔社区居委会",
                code: "008",
            },
            VillageCode {
                name: "丰翔社区居委会",
                code: "009",
            },
            VillageCode {
                name: "银翔社区居委会",
                code: "010",
            },
            VillageCode {
                name: "永翔社区居委会",
                code: "011",
            },
            VillageCode {
                name: "宝翔社区居委会",
                code: "012",
            },
            VillageCode {
                name: "新翔社区居委会",
                code: "013",
            },
            VillageCode {
                name: "瑞林社区居委会",
                code: "014",
            },
            VillageCode {
                name: "隽翔社区居委会",
                code: "015",
            },
            VillageCode {
                name: "翔北社区居委会",
                code: "016",
            },
            VillageCode {
                name: "芳林社区居委会",
                code: "017",
            },
            VillageCode {
                name: "东园社区居委会",
                code: "018",
            },
            VillageCode {
                name: "天恩社区居委会",
                code: "019",
            },
            VillageCode {
                name: "留云社区居委会",
                code: "020",
            },
            VillageCode {
                name: "清猗社区居委会",
                code: "021",
            },
            VillageCode {
                name: "华丰社区居委会",
                code: "022",
            },
            VillageCode {
                name: "华猗社区居委会",
                code: "023",
            },
            VillageCode {
                name: "金通社区居委会",
                code: "024",
            },
            VillageCode {
                name: "劳动街社区居委会",
                code: "025",
            },
            VillageCode {
                name: "嘉绣社区居委会",
                code: "026",
            },
            VillageCode {
                name: "惠平社区居委会",
                code: "027",
            },
            VillageCode {
                name: "东翔社区居委会",
                code: "028",
            },
            VillageCode {
                name: "翔源社区居委会",
                code: "029",
            },
            VillageCode {
                name: "新裕村村委会",
                code: "030",
            },
            VillageCode {
                name: "永丰村村委会",
                code: "031",
            },
            VillageCode {
                name: "永乐村村委会",
                code: "032",
            },
            VillageCode {
                name: "红翔村村委会",
                code: "033",
            },
            VillageCode {
                name: "曙光村村委会",
                code: "034",
            },
            VillageCode {
                name: "静华村村委会",
                code: "035",
            },
            VillageCode {
                name: "新丰村村委会",
                code: "036",
            },
            VillageCode {
                name: "浏翔村村委会",
                code: "037",
            },
            VillageCode {
                name: "乐惠社区居委会",
                code: "038",
            },
            VillageCode {
                name: "嘉美社区居委会",
                code: "039",
            },
        ],
    },
    TownCode {
        name: "安亭镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "迎春社区居委会",
                code: "001",
            },
            VillageCode {
                name: "红梅社区居委会",
                code: "002",
            },
            VillageCode {
                name: "紫荆社区居委会",
                code: "003",
            },
            VillageCode {
                name: "玉兰第一社区居委会",
                code: "004",
            },
            VillageCode {
                name: "玉兰第二社区居委会",
                code: "005",
            },
            VillageCode {
                name: "玉兰第三社区居委会",
                code: "006",
            },
            VillageCode {
                name: "方泰社区居委会",
                code: "007",
            },
            VillageCode {
                name: "博泰社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新源社区居委会",
                code: "009",
            },
            VillageCode {
                name: "沁富社区居委会",
                code: "010",
            },
            VillageCode {
                name: "黄渡社区居委会",
                code: "011",
            },
            VillageCode {
                name: "绿苑社区居委会",
                code: "012",
            },
            VillageCode {
                name: "春盛苑社区居委会",
                code: "013",
            },
            VillageCode {
                name: "莱英社区居委会",
                code: "014",
            },
            VillageCode {
                name: "新安社区居委会",
                code: "015",
            },
            VillageCode {
                name: "博园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "墨玉社区居委会",
                code: "017",
            },
            VillageCode {
                name: "讴象社区居委会",
                code: "018",
            },
            VillageCode {
                name: "陆巷社区居委会",
                code: "019",
            },
            VillageCode {
                name: "泰顺社区居委会",
                code: "020",
            },
            VillageCode {
                name: "泰东社区居委会",
                code: "021",
            },
            VillageCode {
                name: "金桂社区居委会",
                code: "022",
            },
            VillageCode {
                name: "沁乐社区居委会",
                code: "023",
            },
            VillageCode {
                name: "六泉桥社区居委会",
                code: "024",
            },
            VillageCode {
                name: "安研社区居委会",
                code: "025",
            },
            VillageCode {
                name: "安驰社区居委会",
                code: "026",
            },
            VillageCode {
                name: "昌吉东路社区居委会",
                code: "027",
            },
            VillageCode {
                name: "安城社区居委会",
                code: "028",
            },
            VillageCode {
                name: "安景社区居委会",
                code: "029",
            },
            VillageCode {
                name: "方翔社区居委会",
                code: "030",
            },
            VillageCode {
                name: "塔庙村村委会",
                code: "031",
            },
            VillageCode {
                name: "南安村村委会",
                code: "032",
            },
            VillageCode {
                name: "顾浦村村委会",
                code: "033",
            },
            VillageCode {
                name: "兰塘村村委会",
                code: "034",
            },
            VillageCode {
                name: "向阳村村委会",
                code: "035",
            },
            VillageCode {
                name: "火炬村村委会",
                code: "036",
            },
            VillageCode {
                name: "新泾村村委会",
                code: "037",
            },
            VillageCode {
                name: "前进村村委会",
                code: "038",
            },
            VillageCode {
                name: "塘庄村村委会",
                code: "039",
            },
            VillageCode {
                name: "林家村村委会",
                code: "040",
            },
            VillageCode {
                name: "双浦村村委会",
                code: "041",
            },
            VillageCode {
                name: "吕浦村村委会",
                code: "042",
            },
            VillageCode {
                name: "水产村村委会",
                code: "043",
            },
            VillageCode {
                name: "先锋村村委会",
                code: "044",
            },
            VillageCode {
                name: "赵巷村村委会",
                code: "045",
            },
            VillageCode {
                name: "星光村村委会",
                code: "046",
            },
            VillageCode {
                name: "星明村村委会",
                code: "047",
            },
            VillageCode {
                name: "光明村村委会",
                code: "048",
            },
            VillageCode {
                name: "黄墙村村委会",
                code: "049",
            },
            VillageCode {
                name: "漳浦村村委会",
                code: "050",
            },
            VillageCode {
                name: "顾垒村村委会",
                code: "051",
            },
            VillageCode {
                name: "方泰村村委会",
                code: "052",
            },
            VillageCode {
                name: "陆象村村委会",
                code: "053",
            },
            VillageCode {
                name: "讴思村村委会",
                code: "054",
            },
            VillageCode {
                name: "水产村（方泰）村委会",
                code: "055",
            },
            VillageCode {
                name: "西元村村委会",
                code: "056",
            },
            VillageCode {
                name: "龚闵村村委会",
                code: "057",
            },
            VillageCode {
                name: "泥岗村村委会",
                code: "058",
            },
            VillageCode {
                name: "邓家角村村委会",
                code: "059",
            },
            VillageCode {
                name: "朱家村村委会",
                code: "060",
            },
            VillageCode {
                name: "许家村村委会",
                code: "061",
            },
            VillageCode {
                name: "罗家村村委会",
                code: "062",
            },
            VillageCode {
                name: "钱家村村委会",
                code: "063",
            },
            VillageCode {
                name: "东街村村委会",
                code: "064",
            },
            VillageCode {
                name: "黄沈村村委会",
                code: "065",
            },
            VillageCode {
                name: "老宅村村委会",
                code: "066",
            },
            VillageCode {
                name: "顾家村村委会",
                code: "067",
            },
            VillageCode {
                name: "杨木桥村村委会",
                code: "068",
            },
            VillageCode {
                name: "横河村村委会",
                code: "069",
            },
            VillageCode {
                name: "联西村村委会",
                code: "070",
            },
            VillageCode {
                name: "联群村村委会",
                code: "071",
            },
            VillageCode {
                name: "星塔村村委会",
                code: "072",
            },
            VillageCode {
                name: "安勇社区居委会",
                code: "073",
            },
            VillageCode {
                name: "杭桂社区居委会",
                code: "074",
            },
            VillageCode {
                name: "于塘社区居委会",
                code: "075",
            },
            VillageCode {
                name: "春海社区居委会",
                code: "076",
            },
            VillageCode {
                name: "春雨社区居委会",
                code: "077",
            },
        ],
    },
    TownCode {
        name: "马陆镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "育兰社区居委会",
                code: "001",
            },
            VillageCode {
                name: "天马社区居委会",
                code: "002",
            },
            VillageCode {
                name: "马陆新村居委会",
                code: "003",
            },
            VillageCode {
                name: "戬浜社区居委会",
                code: "004",
            },
            VillageCode {
                name: "育苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "樊家社区居委会",
                code: "006",
            },
            VillageCode {
                name: "包桥社区居委会",
                code: "007",
            },
            VillageCode {
                name: "沥苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "彭赵社区居委会",
                code: "009",
            },
            VillageCode {
                name: "仓新社区居委会",
                code: "010",
            },
            VillageCode {
                name: "嘉新社区居委会",
                code: "011",
            },
            VillageCode {
                name: "洪德一坊社区居委会",
                code: "012",
            },
            VillageCode {
                name: "远香一坊社区居委会",
                code: "013",
            },
            VillageCode {
                name: "德立社区居委会",
                code: "014",
            },
            VillageCode {
                name: "封周一坊社区居委会",
                code: "015",
            },
            VillageCode {
                name: "远香二坊社区居委会",
                code: "016",
            },
            VillageCode {
                name: "白银一坊社区居委会",
                code: "017",
            },
            VillageCode {
                name: "崇信社区居委会",
                code: "018",
            },
            VillageCode {
                name: "双单一坊社区居委会",
                code: "019",
            },
            VillageCode {
                name: "云谷一坊社区居委会",
                code: "020",
            },
            VillageCode {
                name: "康丰社区居委会",
                code: "021",
            },
            VillageCode {
                name: "白银二坊社区居委会",
                code: "022",
            },
            VillageCode {
                name: "双丁一坊社区居委会",
                code: "023",
            },
            VillageCode {
                name: "希望一坊社区居委会",
                code: "024",
            },
            VillageCode {
                name: "白银三坊社区居委会",
                code: "025",
            },
            VillageCode {
                name: "德富一坊社区居委会",
                code: "026",
            },
            VillageCode {
                name: "崇文社区居委会",
                code: "027",
            },
            VillageCode {
                name: "崇福社区居委会",
                code: "028",
            },
            VillageCode {
                name: "德富二坊社区居委会",
                code: "029",
            },
            VillageCode {
                name: "洪德二坊社区居委会",
                code: "030",
            },
            VillageCode {
                name: "枫树林社区居委会",
                code: "031",
            },
            VillageCode {
                name: "洪德三坊社区居委会",
                code: "032",
            },
            VillageCode {
                name: "复华社区居委会",
                code: "033",
            },
            VillageCode {
                name: "金沙湾社区居委会",
                code: "034",
            },
            VillageCode {
                name: "合作一坊社区居委会",
                code: "035",
            },
            VillageCode {
                name: "云屏社区居委会",
                code: "036",
            },
            VillageCode {
                name: "唐家苑社区居委会",
                code: "037",
            },
            VillageCode {
                name: "希望二坊社区居委会",
                code: "038",
            },
            VillageCode {
                name: "洪德四坊社区居委会",
                code: "039",
            },
            VillageCode {
                name: "崇信二坊社区居委会",
                code: "040",
            },
            VillageCode {
                name: "沈徐社区居委会",
                code: "041",
            },
            VillageCode {
                name: "合作二坊社区居委会",
                code: "042",
            },
            VillageCode {
                name: "马陆村村委会",
                code: "043",
            },
            VillageCode {
                name: "樊家村村委会",
                code: "044",
            },
            VillageCode {
                name: "彭赵村村委会",
                code: "045",
            },
            VillageCode {
                name: "包桥村村委会",
                code: "046",
            },
            VillageCode {
                name: "李家村村委会",
                code: "047",
            },
            VillageCode {
                name: "陈村村村委会",
                code: "048",
            },
            VillageCode {
                name: "北管村村委会",
                code: "049",
            },
            VillageCode {
                name: "仓场村村委会",
                code: "050",
            },
            VillageCode {
                name: "立新村村委会",
                code: "051",
            },
            VillageCode {
                name: "大裕村村委会",
                code: "052",
            },
            VillageCode {
                name: "大宏村村委会",
                code: "053",
            },
            VillageCode {
                name: "戬浜村村委会",
                code: "054",
            },
        ],
    },
    TownCode {
        name: "徐行镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "徐行社区居委会",
                code: "001",
            },
            VillageCode {
                name: "曹王社区居委会",
                code: "002",
            },
            VillageCode {
                name: "启悦社区居委会",
                code: "003",
            },
            VillageCode {
                name: "启源社区居委会",
                code: "004",
            },
            VillageCode {
                name: "启宁社区居委会",
                code: "005",
            },
            VillageCode {
                name: "小庙村村委会",
                code: "006",
            },
            VillageCode {
                name: "钱桥村村委会",
                code: "007",
            },
            VillageCode {
                name: "徐行村村委会",
                code: "008",
            },
            VillageCode {
                name: "大石皮村村委会",
                code: "009",
            },
            VillageCode {
                name: "伏虎村村委会",
                code: "010",
            },
            VillageCode {
                name: "红星村村委会",
                code: "011",
            },
            VillageCode {
                name: "和桥村村委会",
                code: "012",
            },
            VillageCode {
                name: "曹王村村委会",
                code: "013",
            },
            VillageCode {
                name: "安新村村委会",
                code: "014",
            },
            VillageCode {
                name: "劳动村村委会",
                code: "015",
            },
            VillageCode {
                name: "启新社区居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "华亭镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "袁家桥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "沁园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "华旺社区居委会",
                code: "003",
            },
            VillageCode {
                name: "联一村村委会",
                code: "004",
            },
            VillageCode {
                name: "联三村村委会",
                code: "005",
            },
            VillageCode {
                name: "北新村村委会",
                code: "006",
            },
            VillageCode {
                name: "华亭村村委会",
                code: "007",
            },
            VillageCode {
                name: "金吕村村委会",
                code: "008",
            },
            VillageCode {
                name: "塔桥村村委会",
                code: "009",
            },
            VillageCode {
                name: "连俊村村委会",
                code: "010",
            },
            VillageCode {
                name: "双塘村村委会",
                code: "011",
            },
            VillageCode {
                name: "毛桥村村委会",
                code: "012",
            },
            VillageCode {
                name: "唐行村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "外冈镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "杏花社区居委会",
                code: "001",
            },
            VillageCode {
                name: "外冈新苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "佳苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "兰郡社区居委会",
                code: "004",
            },
            VillageCode {
                name: "景苑第一社区居委会",
                code: "005",
            },
            VillageCode {
                name: "恒涛社区居委会",
                code: "006",
            },
            VillageCode {
                name: "恒荣社区居委会",
                code: "007",
            },
            VillageCode {
                name: "恒雅社区居委会",
                code: "008",
            },
            VillageCode {
                name: "冈峰村村委会",
                code: "009",
            },
            VillageCode {
                name: "外冈村村委会",
                code: "010",
            },
            VillageCode {
                name: "管家村村委会",
                code: "011",
            },
            VillageCode {
                name: "施晋村村委会",
                code: "012",
            },
            VillageCode {
                name: "陈周村村委会",
                code: "013",
            },
            VillageCode {
                name: "徐秦村村委会",
                code: "014",
            },
            VillageCode {
                name: "大陆村村委会",
                code: "015",
            },
            VillageCode {
                name: "北龚村村委会",
                code: "016",
            },
            VillageCode {
                name: "甘柏村村委会",
                code: "017",
            },
            VillageCode {
                name: "葛隆村村委会",
                code: "018",
            },
            VillageCode {
                name: "古塘村村委会",
                code: "019",
            },
            VillageCode {
                name: "泉泾村村委会",
                code: "020",
            },
            VillageCode {
                name: "中泾村村委会",
                code: "021",
            },
            VillageCode {
                name: "望新村村委会",
                code: "022",
            },
            VillageCode {
                name: "周泾村村委会",
                code: "023",
            },
            VillageCode {
                name: "杨甸村村委会",
                code: "024",
            },
            VillageCode {
                name: "长泾村村委会",
                code: "025",
            },
            VillageCode {
                name: "马门村村委会",
                code: "026",
            },
            VillageCode {
                name: "巨门村村委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "江桥镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "曹安社区居委会",
                code: "001",
            },
            VillageCode {
                name: "江宁社区居委会",
                code: "002",
            },
            VillageCode {
                name: "江安社区居委会",
                code: "003",
            },
            VillageCode {
                name: "杨柳社区居委会",
                code: "004",
            },
            VillageCode {
                name: "江丰社区居委会",
                code: "005",
            },
            VillageCode {
                name: "金华社区居委会",
                code: "006",
            },
            VillageCode {
                name: "恒嘉社区居委会",
                code: "007",
            },
            VillageCode {
                name: "封杨社区居委会",
                code: "008",
            },
            VillageCode {
                name: "富友社区居委会",
                code: "009",
            },
            VillageCode {
                name: "嘉城社区居委会",
                code: "010",
            },
            VillageCode {
                name: "嘉怡社区居委会",
                code: "011",
            },
            VillageCode {
                name: "金岸社区居委会",
                code: "012",
            },
            VillageCode {
                name: "嘉华社区居委会",
                code: "013",
            },
            VillageCode {
                name: "江华社区居委会",
                code: "014",
            },
            VillageCode {
                name: "金桥社区居委会",
                code: "015",
            },
            VillageCode {
                name: "金中社区居委会",
                code: "016",
            },
            VillageCode {
                name: "金莱社区居委会",
                code: "017",
            },
            VillageCode {
                name: "金旺社区居委会",
                code: "018",
            },
            VillageCode {
                name: "金园社区居委会",
                code: "019",
            },
            VillageCode {
                name: "嘉川社区居委会",
                code: "020",
            },
            VillageCode {
                name: "嘉星社区居委会",
                code: "021",
            },
            VillageCode {
                name: "嘉航社区居委会",
                code: "022",
            },
            VillageCode {
                name: "金佳社区居委会",
                code: "023",
            },
            VillageCode {
                name: "金水社区居委会",
                code: "024",
            },
            VillageCode {
                name: "金虹社区居委会",
                code: "025",
            },
            VillageCode {
                name: "金城社区居委会",
                code: "026",
            },
            VillageCode {
                name: "绿一社区居委会",
                code: "027",
            },
            VillageCode {
                name: "嘉禧社区居委会",
                code: "028",
            },
            VillageCode {
                name: "金德社区居委会",
                code: "029",
            },
            VillageCode {
                name: "嘉封社区居委会",
                code: "030",
            },
            VillageCode {
                name: "绿二社区居委会",
                code: "031",
            },
            VillageCode {
                name: "绿三社区居委会",
                code: "032",
            },
            VillageCode {
                name: "金达社区居委会",
                code: "033",
            },
            VillageCode {
                name: "嘉豪社区居委会",
                code: "034",
            },
            VillageCode {
                name: "金耀社区居委会",
                code: "035",
            },
            VillageCode {
                name: "嘉涛社区居委会",
                code: "036",
            },
            VillageCode {
                name: "嘉蓝社区居委会",
                code: "037",
            },
            VillageCode {
                name: "嘉海社区居委会",
                code: "038",
            },
            VillageCode {
                name: "嘉峪社区居委会",
                code: "039",
            },
            VillageCode {
                name: "江佳社区居委会",
                code: "040",
            },
            VillageCode {
                name: "嘉远社区居委会",
                code: "041",
            },
            VillageCode {
                name: "嘉龙社区居委会",
                code: "042",
            },
            VillageCode {
                name: "高潮村村委会",
                code: "043",
            },
            VillageCode {
                name: "幸福村村委会",
                code: "044",
            },
            VillageCode {
                name: "五四村村委会",
                code: "045",
            },
            VillageCode {
                name: "沙河村村委会",
                code: "046",
            },
            VillageCode {
                name: "华庄村村委会",
                code: "047",
            },
            VillageCode {
                name: "建华村村委会",
                code: "048",
            },
            VillageCode {
                name: "火线村村委会",
                code: "049",
            },
            VillageCode {
                name: "太平村村委会",
                code: "050",
            },
            VillageCode {
                name: "红光村村委会",
                code: "051",
            },
            VillageCode {
                name: "年丰村村委会",
                code: "052",
            },
            VillageCode {
                name: "新江村村委会",
                code: "053",
            },
            VillageCode {
                name: "封浜村村委会",
                code: "054",
            },
            VillageCode {
                name: "增建村村委会",
                code: "055",
            },
            VillageCode {
                name: "新华村村委会",
                code: "056",
            },
            VillageCode {
                name: "先农村村委会",
                code: "057",
            },
            VillageCode {
                name: "星火村村委会",
                code: "058",
            },
        ],
    },
    TownCode {
        name: "菊园新区",
        code: "011",
        villages: &[
            VillageCode {
                name: "嘉富社区居委会",
                code: "001",
            },
            VillageCode {
                name: "嘉邦社区居委会",
                code: "002",
            },
            VillageCode {
                name: "嘉宏社区居委会",
                code: "003",
            },
            VillageCode {
                name: "嘉馨社区居委会",
                code: "004",
            },
            VillageCode {
                name: "嘉枫社区居委会",
                code: "005",
            },
            VillageCode {
                name: "泰宸社区居委会",
                code: "006",
            },
            VillageCode {
                name: "宝菊社区居委会",
                code: "007",
            },
            VillageCode {
                name: "嘉保社区居委会",
                code: "008",
            },
            VillageCode {
                name: "嘉悠社区居委会",
                code: "009",
            },
            VillageCode {
                name: "竹筱社区居委会",
                code: "010",
            },
            VillageCode {
                name: "嘉北社区居委会",
                code: "011",
            },
            VillageCode {
                name: "嘉悦社区居委会",
                code: "012",
            },
            VillageCode {
                name: "嘉和社区居委会",
                code: "013",
            },
            VillageCode {
                name: "嘉汇社区居委会",
                code: "014",
            },
            VillageCode {
                name: "嘉盛社区居委会",
                code: "015",
            },
            VillageCode {
                name: "嘉筱社区居委会",
                code: "016",
            },
            VillageCode {
                name: "嘉宜社区居委会",
                code: "017",
            },
            VillageCode {
                name: "嘉莱社区居委会",
                code: "018",
            },
            VillageCode {
                name: "嘉慈社区居委会",
                code: "019",
            },
            VillageCode {
                name: "嘉铭社区居委会",
                code: "020",
            },
            VillageCode {
                name: "嘉呈社区居委会",
                code: "021",
            },
            VillageCode {
                name: "永胜村村委会",
                code: "022",
            },
            VillageCode {
                name: "青冈村村委会",
                code: "023",
            },
            VillageCode {
                name: "六里村村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "嘉定工业区",
        code: "012",
        villages: &[
            VillageCode {
                name: "庆阳社区居委会",
                code: "001",
            },
            VillageCode {
                name: "凤池社区居委会",
                code: "002",
            },
            VillageCode {
                name: "福蕴社区居委会",
                code: "003",
            },
            VillageCode {
                name: "民乐社区居委会",
                code: "004",
            },
            VillageCode {
                name: "横沥社区居委会",
                code: "005",
            },
            VillageCode {
                name: "永盛社区居委会",
                code: "006",
            },
            VillageCode {
                name: "胜辛社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新宝社区居委会",
                code: "008",
            },
            VillageCode {
                name: "越华社区居委会",
                code: "009",
            },
            VillageCode {
                name: "天华社区居委会",
                code: "010",
            },
            VillageCode {
                name: "裕民社区居委会",
                code: "011",
            },
            VillageCode {
                name: "汇旺社区居委会",
                code: "012",
            },
            VillageCode {
                name: "宜景社区居委会",
                code: "013",
            },
            VillageCode {
                name: "丰怡社区居委会",
                code: "014",
            },
            VillageCode {
                name: "娄塘社区居委会",
                code: "015",
            },
            VillageCode {
                name: "汇珠社区居委会",
                code: "016",
            },
            VillageCode {
                name: "梧桐社区居委会",
                code: "017",
            },
            VillageCode {
                name: "蔷薇社区居委会",
                code: "018",
            },
            VillageCode {
                name: "汇源社区居委会",
                code: "019",
            },
            VillageCode {
                name: "慈竹社区居委会",
                code: "020",
            },
            VillageCode {
                name: "辛勤村村委会",
                code: "021",
            },
            VillageCode {
                name: "牌楼村村委会",
                code: "022",
            },
            VillageCode {
                name: "群裕村村委会",
                code: "023",
            },
            VillageCode {
                name: "胜利村村委会",
                code: "024",
            },
            VillageCode {
                name: "虬桥村村委会",
                code: "025",
            },
            VillageCode {
                name: "现龙村村委会",
                code: "026",
            },
            VillageCode {
                name: "娄塘村村委会",
                code: "027",
            },
            VillageCode {
                name: "泾河村村委会",
                code: "028",
            },
            VillageCode {
                name: "赵厅村村委会",
                code: "029",
            },
            VillageCode {
                name: "娄东村村委会",
                code: "030",
            },
            VillageCode {
                name: "草庵村村委会",
                code: "031",
            },
            VillageCode {
                name: "陆渡村村委会",
                code: "032",
            },
            VillageCode {
                name: "灯塔村村委会",
                code: "033",
            },
            VillageCode {
                name: "雨化村村委会",
                code: "034",
            },
            VillageCode {
                name: "旺泾村村委会",
                code: "035",
            },
            VillageCode {
                name: "朱家桥村村委会",
                code: "036",
            },
            VillageCode {
                name: "黎明村村委会",
                code: "037",
            },
            VillageCode {
                name: "白墙村村委会",
                code: "038",
            },
            VillageCode {
                name: "竹桥村村委会",
                code: "039",
            },
            VillageCode {
                name: "三里村村委会",
                code: "040",
            },
            VillageCode {
                name: "嘉盛园社区居委会",
                code: "041",
            },
            VillageCode {
                name: "永泰社区居委会",
                code: "042",
            },
        ],
    },
];

static TOWNS_SJ_012: [TownCode; 42] = [
    TownCode {
        name: "潍坊新村街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "潍坊一村居委会",
                code: "001",
            },
            VillageCode {
                name: "潍坊二村居委会",
                code: "002",
            },
            VillageCode {
                name: "潍坊三村居委会",
                code: "003",
            },
            VillageCode {
                name: "潍坊四村居委会",
                code: "004",
            },
            VillageCode {
                name: "潍坊五村居委会",
                code: "005",
            },
            VillageCode {
                name: "潍坊六七村居委会",
                code: "006",
            },
            VillageCode {
                name: "潍坊八村居委会",
                code: "007",
            },
            VillageCode {
                name: "潍坊九村居委会",
                code: "008",
            },
            VillageCode {
                name: "潍坊十村第一居委会",
                code: "009",
            },
            VillageCode {
                name: "潍坊十村第二居委会",
                code: "010",
            },
            VillageCode {
                name: "竹园居委会",
                code: "011",
            },
            VillageCode {
                name: "源竹居委会",
                code: "012",
            },
            VillageCode {
                name: "竹南居委会",
                code: "013",
            },
            VillageCode {
                name: "福竹居委会",
                code: "014",
            },
            VillageCode {
                name: "张杨居委会",
                code: "015",
            },
            VillageCode {
                name: "朱家滩居委会",
                code: "016",
            },
            VillageCode {
                name: "泉东第一居委会",
                code: "017",
            },
            VillageCode {
                name: "王家宅居委会",
                code: "018",
            },
            VillageCode {
                name: "崂山东路居委会",
                code: "019",
            },
            VillageCode {
                name: "杨家渡居委会",
                code: "020",
            },
            VillageCode {
                name: "张家浜居委会",
                code: "021",
            },
            VillageCode {
                name: "谢家宅居委会",
                code: "022",
            },
            VillageCode {
                name: "陈家宅居委会",
                code: "023",
            },
            VillageCode {
                name: "东南居委会",
                code: "024",
            },
            VillageCode {
                name: "泉东第二居委会",
                code: "025",
            },
            VillageCode {
                name: "香榭丽居委会",
                code: "026",
            },
            VillageCode {
                name: "世茂滨江第一居委会",
                code: "027",
            },
            VillageCode {
                name: "滨江凯旋门居委会",
                code: "028",
            },
            VillageCode {
                name: "东明新村居委会",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "陆家嘴街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "梅园一村居委会",
                code: "001",
            },
            VillageCode {
                name: "汇豪天下居委会",
                code: "002",
            },
            VillageCode {
                name: "梅园三村居委会",
                code: "003",
            },
            VillageCode {
                name: "市新居委会",
                code: "004",
            },
            VillageCode {
                name: "隧成居委会",
                code: "005",
            },
            VillageCode {
                name: "福沈居委会",
                code: "006",
            },
            VillageCode {
                name: "福山居委会",
                code: "007",
            },
            VillageCode {
                name: "光辉居委会",
                code: "008",
            },
            VillageCode {
                name: "松山居委会",
                code: "009",
            },
            VillageCode {
                name: "陈家门居委会",
                code: "010",
            },
            VillageCode {
                name: "林山居委会",
                code: "011",
            },
            VillageCode {
                name: "东昌居委会",
                code: "012",
            },
            VillageCode {
                name: "荣城居委会",
                code: "013",
            },
            VillageCode {
                name: "上港第一居委会",
                code: "014",
            },
            VillageCode {
                name: "招远路居委会",
                code: "015",
            },
            VillageCode {
                name: "三航居委会",
                code: "016",
            },
            VillageCode {
                name: "崂山一村居委会",
                code: "017",
            },
            VillageCode {
                name: "崂山二村居委会",
                code: "018",
            },
            VillageCode {
                name: "崂山三村居委会",
                code: "019",
            },
            VillageCode {
                name: "崂山五村居委会",
                code: "020",
            },
            VillageCode {
                name: "崂山六村居委会",
                code: "021",
            },
            VillageCode {
                name: "乳山二村居委会",
                code: "022",
            },
            VillageCode {
                name: "乳山四村居委会",
                code: "023",
            },
            VillageCode {
                name: "乳山五村居委会",
                code: "024",
            },
            VillageCode {
                name: "仁恒滨江园居委会",
                code: "025",
            },
            VillageCode {
                name: "江临财富居委会",
                code: "026",
            },
            VillageCode {
                name: "菊园居委会",
                code: "027",
            },
            VillageCode {
                name: "浦江茗园居委会",
                code: "028",
            },
            VillageCode {
                name: "东园新村一居委会",
                code: "029",
            },
            VillageCode {
                name: "东园新村二居委会",
                code: "030",
            },
            VillageCode {
                name: "滨江居委会",
                code: "031",
            },
            VillageCode {
                name: "上港第二居委会",
                code: "032",
            },
            VillageCode {
                name: "浦滨居委会",
                code: "033",
            },
            VillageCode {
                name: "崂山四村居委会",
                code: "034",
            },
            VillageCode {
                name: "松林居委会",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "周家渡街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "上南一村居委会",
                code: "001",
            },
            VillageCode {
                name: "上南二村居委会",
                code: "002",
            },
            VillageCode {
                name: "上南三村居委会",
                code: "003",
            },
            VillageCode {
                name: "上南四村居委会",
                code: "004",
            },
            VillageCode {
                name: "上南五村居委会",
                code: "005",
            },
            VillageCode {
                name: "上南六村居委会",
                code: "006",
            },
            VillageCode {
                name: "上南七村居委会",
                code: "007",
            },
            VillageCode {
                name: "上南八村居委会",
                code: "008",
            },
            VillageCode {
                name: "上南九村第一居委会",
                code: "009",
            },
            VillageCode {
                name: "上南十村一居委会",
                code: "010",
            },
            VillageCode {
                name: "上南十村二居委会",
                code: "011",
            },
            VillageCode {
                name: "雪野二村居委会",
                code: "012",
            },
            VillageCode {
                name: "齐河第一居委会",
                code: "013",
            },
            VillageCode {
                name: "齐河第二居委会",
                code: "014",
            },
            VillageCode {
                name: "齐河第三居委会",
                code: "015",
            },
            VillageCode {
                name: "齐河第八居委会",
                code: "016",
            },
            VillageCode {
                name: "云台第一居委会",
                code: "017",
            },
            VillageCode {
                name: "云台第二居委会",
                code: "018",
            },
            VillageCode {
                name: "昌里路第四居委会",
                code: "019",
            },
            VillageCode {
                name: "昌里路第五居委会",
                code: "020",
            },
            VillageCode {
                name: "昌里路第七居委会",
                code: "021",
            },
            VillageCode {
                name: "上南十一村居委会",
                code: "022",
            },
            VillageCode {
                name: "上南十二村居委会",
                code: "023",
            },
            VillageCode {
                name: "云莲第一居委会",
                code: "024",
            },
            VillageCode {
                name: "齐河第七居委会",
                code: "025",
            },
            VillageCode {
                name: "昌里花园居委会",
                code: "026",
            },
            VillageCode {
                name: "齐河第四居委会",
                code: "027",
            },
            VillageCode {
                name: "都市庭院居委会",
                code: "028",
            },
            VillageCode {
                name: "川新居委会",
                code: "029",
            },
            VillageCode {
                name: "上南花苑居委会",
                code: "030",
            },
            VillageCode {
                name: "恒大居委会",
                code: "031",
            },
            VillageCode {
                name: "齐河第五居委会",
                code: "032",
            },
            VillageCode {
                name: "上南九村第二居委会",
                code: "033",
            },
            VillageCode {
                name: "川周居委会",
                code: "034",
            },
            VillageCode {
                name: "云台第三居委会",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "塘桥街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "微山新村居委会",
                code: "001",
            },
            VillageCode {
                name: "微南居委会",
                code: "002",
            },
            VillageCode {
                name: "宁阳路居委会",
                code: "003",
            },
            VillageCode {
                name: "茂兴路居委会",
                code: "004",
            },
            VillageCode {
                name: "香花桥街居委会",
                code: "005",
            },
            VillageCode {
                name: "浦建路居委会",
                code: "006",
            },
            VillageCode {
                name: "塘桥居委会",
                code: "007",
            },
            VillageCode {
                name: "文兰居委会",
                code: "008",
            },
            VillageCode {
                name: "胡家木桥居委会",
                code: "009",
            },
            VillageCode {
                name: "南泉路居委会",
                code: "010",
            },
            VillageCode {
                name: "蓝村居委会",
                code: "011",
            },
            VillageCode {
                name: "蓝高居委会",
                code: "012",
            },
            VillageCode {
                name: "塘东居委会",
                code: "013",
            },
            VillageCode {
                name: "南浦居委会",
                code: "014",
            },
            VillageCode {
                name: "兰东居委会",
                code: "015",
            },
            VillageCode {
                name: "金浦居委会",
                code: "016",
            },
            VillageCode {
                name: "龙阳路居委会",
                code: "017",
            },
            VillageCode {
                name: "南城居委会",
                code: "018",
            },
            VillageCode {
                name: "东方居委会",
                code: "019",
            },
            VillageCode {
                name: "怡东居委会",
                code: "020",
            },
            VillageCode {
                name: "贵龙居委会",
                code: "021",
            },
            VillageCode {
                name: "龙园居委会",
                code: "022",
            },
            VillageCode {
                name: "富都居委会",
                code: "023",
            },
            VillageCode {
                name: "蓝欣居委会",
                code: "024",
            },
            VillageCode {
                name: "浦阳居委会",
                code: "025",
            },
            VillageCode {
                name: "浦明华城居委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "上钢新村街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "上钢一村居委会",
                code: "001",
            },
            VillageCode {
                name: "上钢二村居委会",
                code: "002",
            },
            VillageCode {
                name: "上钢三村居委会",
                code: "003",
            },
            VillageCode {
                name: "上钢四村居委会",
                code: "004",
            },
            VillageCode {
                name: "上钢五村居委会",
                code: "005",
            },
            VillageCode {
                name: "上钢六村居委会",
                code: "006",
            },
            VillageCode {
                name: "上钢七村居委会",
                code: "007",
            },
            VillageCode {
                name: "上钢九村居委会",
                code: "008",
            },
            VillageCode {
                name: "上钢十村居委会",
                code: "009",
            },
            VillageCode {
                name: "德州一村居委会",
                code: "010",
            },
            VillageCode {
                name: "德州二村居委会",
                code: "011",
            },
            VillageCode {
                name: "德州三村居委会",
                code: "012",
            },
            VillageCode {
                name: "德州四村居委会",
                code: "013",
            },
            VillageCode {
                name: "德州五村居委会",
                code: "014",
            },
            VillageCode {
                name: "德州六村居委会",
                code: "015",
            },
            VillageCode {
                name: "德州七村居委会",
                code: "016",
            },
            VillageCode {
                name: "济阳一村居委会",
                code: "017",
            },
            VillageCode {
                name: "济阳二村居委会",
                code: "018",
            },
            VillageCode {
                name: "济阳三村居委会",
                code: "019",
            },
            VillageCode {
                name: "耀华路第一村居委会",
                code: "020",
            },
            VillageCode {
                name: "济中村居委会",
                code: "021",
            },
            VillageCode {
                name: "耀华路第三村居委会",
                code: "022",
            },
            VillageCode {
                name: "上南花城居委会",
                code: "023",
            },
            VillageCode {
                name: "长德居委会",
                code: "024",
            },
            VillageCode {
                name: "耀华路第二村居委会",
                code: "025",
            },
            VillageCode {
                name: "耀华滨江第一居委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "南码头路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "西三里桥居委会",
                code: "001",
            },
            VillageCode {
                name: "塘南居委会",
                code: "002",
            },
            VillageCode {
                name: "银河居委会",
                code: "003",
            },
            VillageCode {
                name: "沂南路居委会",
                code: "004",
            },
            VillageCode {
                name: "东三里桥居委会",
                code: "005",
            },
            VillageCode {
                name: "胶南路居委会",
                code: "006",
            },
            VillageCode {
                name: "临沂一村居委会",
                code: "007",
            },
            VillageCode {
                name: "临沂二村居委会",
                code: "008",
            },
            VillageCode {
                name: "临沂三村居委会",
                code: "009",
            },
            VillageCode {
                name: "临沂五村居委会",
                code: "010",
            },
            VillageCode {
                name: "临沂八村第一居委会",
                code: "011",
            },
            VillageCode {
                name: "临沂八村第二居委会",
                code: "012",
            },
            VillageCode {
                name: "新桥居委会",
                code: "013",
            },
            VillageCode {
                name: "港机新村居委会",
                code: "014",
            },
            VillageCode {
                name: "鹏欣居委会",
                code: "015",
            },
            VillageCode {
                name: "金星居委会",
                code: "016",
            },
            VillageCode {
                name: "建业居委会",
                code: "017",
            },
            VillageCode {
                name: "东盛居委会",
                code: "018",
            },
            VillageCode {
                name: "六里第一居委会",
                code: "019",
            },
            VillageCode {
                name: "六里第二居委会",
                code: "020",
            },
            VillageCode {
                name: "六里第三居委会",
                code: "021",
            },
            VillageCode {
                name: "六里第四居委会",
                code: "022",
            },
            VillageCode {
                name: "龙馥居委会",
                code: "023",
            },
            VillageCode {
                name: "临沂六村居委会",
                code: "024",
            },
            VillageCode {
                name: "临沂七村居委会",
                code: "025",
            },
            VillageCode {
                name: "东方城市花园居委会",
                code: "026",
            },
            VillageCode {
                name: "六里第五居委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "沪东新村街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "沪东新村一居委会",
                code: "001",
            },
            VillageCode {
                name: "沪东新村二居委会",
                code: "002",
            },
            VillageCode {
                name: "沪新居委会",
                code: "003",
            },
            VillageCode {
                name: "沪南居委会",
                code: "004",
            },
            VillageCode {
                name: "船舶新村居委会",
                code: "005",
            },
            VillageCode {
                name: "向东新村居委会",
                code: "006",
            },
            VillageCode {
                name: "北小区居委会",
                code: "007",
            },
            VillageCode {
                name: "朱家门居委会",
                code: "008",
            },
            VillageCode {
                name: "陈家宅居委会",
                code: "009",
            },
            VillageCode {
                name: "寿光路一居委会",
                code: "010",
            },
            VillageCode {
                name: "寿光路二居委会",
                code: "011",
            },
            VillageCode {
                name: "博兴路一居委会",
                code: "012",
            },
            VillageCode {
                name: "博兴路三居委会",
                code: "013",
            },
            VillageCode {
                name: "柳博居委会",
                code: "014",
            },
            VillageCode {
                name: "金桥花苑居委会",
                code: "015",
            },
            VillageCode {
                name: "东波苑第二居委会",
                code: "016",
            },
            VillageCode {
                name: "东波苑第三居委会",
                code: "017",
            },
            VillageCode {
                name: "东波苑第四居委会",
                code: "018",
            },
            VillageCode {
                name: "东波苑第六居委会",
                code: "019",
            },
            VillageCode {
                name: "江南山水居委会",
                code: "020",
            },
            VillageCode {
                name: "莱阳新家园居委会",
                code: "021",
            },
            VillageCode {
                name: "伟莱家园居委会",
                code: "022",
            },
            VillageCode {
                name: "金浦居委会",
                code: "023",
            },
            VillageCode {
                name: "汇佳苑居委会",
                code: "024",
            },
            VillageCode {
                name: "伟业居委会",
                code: "025",
            },
            VillageCode {
                name: "伟锦居委会",
                code: "026",
            },
            VillageCode {
                name: "璞爱居委会",
                code: "027",
            },
            VillageCode {
                name: "同方锦城居委会",
                code: "028",
            },
            VillageCode {
                name: "长岛苑居委会",
                code: "029",
            },
            VillageCode {
                name: "锦河苑居委会",
                code: "030",
            },
            VillageCode {
                name: "东方丽景居委会",
                code: "031",
            },
            VillageCode {
                name: "兰城居委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "金杨新村街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "金杨路二居委会",
                code: "001",
            },
            VillageCode {
                name: "金杨路六居委会",
                code: "002",
            },
            VillageCode {
                name: "金杨路七居委会",
                code: "003",
            },
            VillageCode {
                name: "银山路三居委会",
                code: "004",
            },
            VillageCode {
                name: "金口路一居委会",
                code: "005",
            },
            VillageCode {
                name: "金口路二居委会",
                code: "006",
            },
            VillageCode {
                name: "金台路一居委会",
                code: "007",
            },
            VillageCode {
                name: "枣庄路一居委会",
                code: "008",
            },
            VillageCode {
                name: "灵山路三居委会",
                code: "009",
            },
            VillageCode {
                name: "金杨路一居委会",
                code: "010",
            },
            VillageCode {
                name: "金杨路八居委会",
                code: "011",
            },
            VillageCode {
                name: "银山路四居委会",
                code: "012",
            },
            VillageCode {
                name: "灵山路一居委会",
                code: "013",
            },
            VillageCode {
                name: "罗山一村居委会",
                code: "014",
            },
            VillageCode {
                name: "罗山二村居委会",
                code: "015",
            },
            VillageCode {
                name: "罗山三村一居委会",
                code: "016",
            },
            VillageCode {
                name: "罗山三村二居委会",
                code: "017",
            },
            VillageCode {
                name: "罗山四村居委会",
                code: "018",
            },
            VillageCode {
                name: "罗山五村居委会",
                code: "019",
            },
            VillageCode {
                name: "香山新村一居委会",
                code: "020",
            },
            VillageCode {
                name: "香山新村七居委会",
                code: "021",
            },
            VillageCode {
                name: "庆宁寺居委会",
                code: "022",
            },
            VillageCode {
                name: "金台路第二居委会",
                code: "023",
            },
            VillageCode {
                name: "金台路第三居委会",
                code: "024",
            },
            VillageCode {
                name: "香山新村第三居委会",
                code: "025",
            },
            VillageCode {
                name: "香山新村第四居委会",
                code: "026",
            },
            VillageCode {
                name: "黄山新村二居委会",
                code: "027",
            },
            VillageCode {
                name: "仁和居委会",
                code: "028",
            },
            VillageCode {
                name: "银山路第五居委会",
                code: "029",
            },
            VillageCode {
                name: "始信苑居委会",
                code: "030",
            },
            VillageCode {
                name: "香山五村居委会",
                code: "031",
            },
            VillageCode {
                name: "住友名人花园居委会",
                code: "032",
            },
            VillageCode {
                name: "蝶恋园居委会",
                code: "033",
            },
            VillageCode {
                name: "居家桥居委会",
                code: "034",
            },
            VillageCode {
                name: "金樟居委会",
                code: "035",
            },
            VillageCode {
                name: "罗山六村居委会",
                code: "036",
            },
            VillageCode {
                name: "罗山七村居委会",
                code: "037",
            },
            VillageCode {
                name: "黄山新村一居委会",
                code: "038",
            },
            VillageCode {
                name: "黄山新村三居委会",
                code: "039",
            },
            VillageCode {
                name: "东方之音居委会",
                code: "040",
            },
            VillageCode {
                name: "黄山新城居委会",
                code: "041",
            },
            VillageCode {
                name: "银山路第一居委会",
                code: "042",
            },
            VillageCode {
                name: "罗山八村居委会",
                code: "043",
            },
            VillageCode {
                name: "广洋苑居委会",
                code: "044",
            },
            VillageCode {
                name: "博山东路居委会",
                code: "045",
            },
            VillageCode {
                name: "金杨路四居委会",
                code: "046",
            },
            VillageCode {
                name: "金桥瑞仕花园居委会",
                code: "047",
            },
            VillageCode {
                name: "金口路第三居委会",
                code: "048",
            },
        ],
    },
    TownCode {
        name: "洋泾街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "名门世家居委会",
                code: "001",
            },
            VillageCode {
                name: "西镇居委会",
                code: "002",
            },
            VillageCode {
                name: "阳光二村居委会",
                code: "003",
            },
            VillageCode {
                name: "阳光三村居委会",
                code: "004",
            },
            VillageCode {
                name: "海院新村居委会",
                code: "005",
            },
            VillageCode {
                name: "泾西新村居委会",
                code: "006",
            },
            VillageCode {
                name: "泾东新村居委会",
                code: "007",
            },
            VillageCode {
                name: "巨野路居委会",
                code: "008",
            },
            VillageCode {
                name: "博山路居委会",
                code: "009",
            },
            VillageCode {
                name: "栖山路居委会",
                code: "010",
            },
            VillageCode {
                name: "巨西居委会",
                code: "011",
            },
            VillageCode {
                name: "桃林一居委会",
                code: "012",
            },
            VillageCode {
                name: "巨东居委会",
                code: "013",
            },
            VillageCode {
                name: "羽北居委会",
                code: "014",
            },
            VillageCode {
                name: "凌高居委会",
                code: "015",
            },
            VillageCode {
                name: "凌联一村居委会",
                code: "016",
            },
            VillageCode {
                name: "凌联三村居委会",
                code: "017",
            },
            VillageCode {
                name: "凌联四村居委会",
                code: "018",
            },
            VillageCode {
                name: "桃林二居委会",
                code: "019",
            },
            VillageCode {
                name: "星海居委会",
                code: "020",
            },
            VillageCode {
                name: "崮山第二居委会",
                code: "021",
            },
            VillageCode {
                name: "陆家嘴花园居委会",
                code: "022",
            },
            VillageCode {
                name: "崮山路第一居委会",
                code: "023",
            },
            VillageCode {
                name: "崮洋居委会",
                code: "024",
            },
            VillageCode {
                name: "羽洋居委会",
                code: "025",
            },
            VillageCode {
                name: "森洋居委会",
                code: "026",
            },
            VillageCode {
                name: "民杨居委会",
                code: "027",
            },
            VillageCode {
                name: "巨杨居委会",
                code: "028",
            },
            VillageCode {
                name: "永安居委会",
                code: "029",
            },
            VillageCode {
                name: "海防居委会",
                code: "030",
            },
            VillageCode {
                name: "盛世年华居委会",
                code: "031",
            },
            VillageCode {
                name: "第五大道居委会",
                code: "032",
            },
            VillageCode {
                name: "国际华城居委会",
                code: "033",
            },
            VillageCode {
                name: "维多利居委会",
                code: "034",
            },
            VillageCode {
                name: "玫瑰园居委会",
                code: "035",
            },
            VillageCode {
                name: "海弘居委会",
                code: "036",
            },
            VillageCode {
                name: "山水国际居委会",
                code: "037",
            },
            VillageCode {
                name: "上海滩花园居委会",
                code: "038",
            },
            VillageCode {
                name: "尚海郦景居委会",
                code: "039",
            },
            VillageCode {
                name: "翠璟滨江居委会",
                code: "040",
            },
            VillageCode {
                name: "陆家嘴花园第二居委会",
                code: "041",
            },
        ],
    },
    TownCode {
        name: "浦兴路街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "凌河路二居委会",
                code: "001",
            },
            VillageCode {
                name: "凌河路三居委会",
                code: "002",
            },
            VillageCode {
                name: "凌河路四居委会",
                code: "003",
            },
            VillageCode {
                name: "凌河路五居委会",
                code: "004",
            },
            VillageCode {
                name: "凌河路七居委会",
                code: "005",
            },
            VillageCode {
                name: "凌河路八居委会",
                code: "006",
            },
            VillageCode {
                name: "东陆路一居委会",
                code: "007",
            },
            VillageCode {
                name: "东陆路二居委会",
                code: "008",
            },
            VillageCode {
                name: "东陆路五居委会",
                code: "009",
            },
            VillageCode {
                name: "荷泽路一居委会",
                code: "010",
            },
            VillageCode {
                name: "荷泽路三居委会",
                code: "011",
            },
            VillageCode {
                name: "荷泽路五居委会",
                code: "012",
            },
            VillageCode {
                name: "浦兴路一居委会",
                code: "013",
            },
            VillageCode {
                name: "胶东路一居委会",
                code: "014",
            },
            VillageCode {
                name: "龙臣居委会",
                code: "015",
            },
            VillageCode {
                name: "浦兴路三居委会",
                code: "016",
            },
            VillageCode {
                name: "银桥居委会",
                code: "017",
            },
            VillageCode {
                name: "胶东路第二居委会",
                code: "018",
            },
            VillageCode {
                name: "双桥居委会",
                code: "019",
            },
            VillageCode {
                name: "金鑫居委会",
                code: "020",
            },
            VillageCode {
                name: "金泽苑居委会",
                code: "021",
            },
            VillageCode {
                name: "东荷居委会",
                code: "022",
            },
            VillageCode {
                name: "金鹏居委会",
                code: "023",
            },
            VillageCode {
                name: "中大居委会",
                code: "024",
            },
            VillageCode {
                name: "证大第一居委会",
                code: "025",
            },
            VillageCode {
                name: "长岛路居委会",
                code: "026",
            },
            VillageCode {
                name: "浦兴路第二居委会",
                code: "027",
            },
            VillageCode {
                name: "平度居委会",
                code: "028",
            },
            VillageCode {
                name: "牟平居委会",
                code: "029",
            },
            VillageCode {
                name: "胶东路第三居委会",
                code: "030",
            },
            VillageCode {
                name: "凌河路第六居委会",
                code: "031",
            },
            VillageCode {
                name: "凌河路第一居委会",
                code: "032",
            },
            VillageCode {
                name: "证大第二居委会",
                code: "033",
            },
            VillageCode {
                name: "东陆路三居委会",
                code: "034",
            },
            VillageCode {
                name: "东陆路四居委会",
                code: "035",
            },
            VillageCode {
                name: "金桥湾居委会",
                code: "036",
            },
            VillageCode {
                name: "台儿庄居委会",
                code: "037",
            },
            VillageCode {
                name: "金桥居委会",
                code: "038",
            },
            VillageCode {
                name: "巨峰居委会",
                code: "039",
            },
            VillageCode {
                name: "金东居委会",
                code: "040",
            },
        ],
    },
    TownCode {
        name: "东明路街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "三林苑居委会",
                code: "001",
            },
            VillageCode {
                name: "翠竹苑居委会",
                code: "002",
            },
            VillageCode {
                name: "金光新村一居委会",
                code: "003",
            },
            VillageCode {
                name: "尚桂苑居委会",
                code: "004",
            },
            VillageCode {
                name: "金光居委会",
                code: "005",
            },
            VillageCode {
                name: "凌兆新村第一居委会",
                code: "006",
            },
            VillageCode {
                name: "凌兆新村第二居委会",
                code: "007",
            },
            VillageCode {
                name: "凌兆新村第三居委会",
                code: "008",
            },
            VillageCode {
                name: "凌兆新村第四居委会",
                code: "009",
            },
            VillageCode {
                name: "凌兆新村第五居委会",
                code: "010",
            },
            VillageCode {
                name: "凌兆新村第六居委会",
                code: "011",
            },
            VillageCode {
                name: "凌兆新村第七居委会",
                code: "012",
            },
            VillageCode {
                name: "凌兆新村第八居委会",
                code: "013",
            },
            VillageCode {
                name: "凌兆新村第九居委会",
                code: "014",
            },
            VillageCode {
                name: "凌兆新村第十居委会",
                code: "015",
            },
            VillageCode {
                name: "凌兆新村第十一居委会",
                code: "016",
            },
            VillageCode {
                name: "凌兆新村第十二居委会",
                code: "017",
            },
            VillageCode {
                name: "凌兆新村第十三居委会",
                code: "018",
            },
            VillageCode {
                name: "凌兆新村第十四居委会",
                code: "019",
            },
            VillageCode {
                name: "凌兆新村第十五居委会",
                code: "020",
            },
            VillageCode {
                name: "品华苑居委会",
                code: "021",
            },
            VillageCode {
                name: "红枫苑居委会",
                code: "022",
            },
            VillageCode {
                name: "安居苑居委会",
                code: "023",
            },
            VillageCode {
                name: "品翠苑居委会",
                code: "024",
            },
            VillageCode {
                name: "品新苑居委会",
                code: "025",
            },
            VillageCode {
                name: "棕榈苑居委会",
                code: "026",
            },
            VillageCode {
                name: "金桂苑居委会",
                code: "027",
            },
            VillageCode {
                name: "凌兆新村第十六居委会",
                code: "028",
            },
            VillageCode {
                name: "盛源居委会",
                code: "029",
            },
            VillageCode {
                name: "新月第一居委会",
                code: "030",
            },
            VillageCode {
                name: "新月第二居委会",
                code: "031",
            },
            VillageCode {
                name: "永泰花苑居委会",
                code: "032",
            },
            VillageCode {
                name: "金禾苑居委会",
                code: "033",
            },
            VillageCode {
                name: "金橘苑居委会",
                code: "034",
            },
            VillageCode {
                name: "樱桃苑居委会",
                code: "035",
            },
            VillageCode {
                name: "金谊河畔居委会",
                code: "036",
            },
            VillageCode {
                name: "金色雅筑居委会",
                code: "037",
            },
            VillageCode {
                name: "湾流域居委会",
                code: "038",
            },
            VillageCode {
                name: "新月第三居委会",
                code: "039",
            },
        ],
    },
    TownCode {
        name: "花木街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "牡丹第二居委会",
                code: "001",
            },
            VillageCode {
                name: "牡丹第四居委会",
                code: "002",
            },
            VillageCode {
                name: "牡丹第七居委会",
                code: "003",
            },
            VillageCode {
                name: "培花新村四居委会",
                code: "004",
            },
            VillageCode {
                name: "培花新村六居委会",
                code: "005",
            },
            VillageCode {
                name: "培花新村七居委会",
                code: "006",
            },
            VillageCode {
                name: "由由一居委会",
                code: "007",
            },
            VillageCode {
                name: "由由二居委会",
                code: "008",
            },
            VillageCode {
                name: "由由四居委会",
                code: "009",
            },
            VillageCode {
                name: "由由五居委会",
                code: "010",
            },
            VillageCode {
                name: "由由六居委会",
                code: "011",
            },
            VillageCode {
                name: "由由七居委会",
                code: "012",
            },
            VillageCode {
                name: "牡丹第八居委会",
                code: "013",
            },
            VillageCode {
                name: "东城新村第一居委会",
                code: "014",
            },
            VillageCode {
                name: "锦绣居委会",
                code: "015",
            },
            VillageCode {
                name: "牡丹第一居委会",
                code: "016",
            },
            VillageCode {
                name: "牡丹第六居委会",
                code: "017",
            },
            VillageCode {
                name: "万邦居委会",
                code: "018",
            },
            VillageCode {
                name: "牡丹第五居委会",
                code: "019",
            },
            VillageCode {
                name: "培花新村一居委会",
                code: "020",
            },
            VillageCode {
                name: "培花新村二居委会",
                code: "021",
            },
            VillageCode {
                name: "培花新村三居委会",
                code: "022",
            },
            VillageCode {
                name: "明月居委会",
                code: "023",
            },
            VillageCode {
                name: "东城新村第三居委会",
                code: "024",
            },
            VillageCode {
                name: "东城新村第四居委会",
                code: "025",
            },
            VillageCode {
                name: "培花新村第九居委会",
                code: "026",
            },
            VillageCode {
                name: "牡丹第三居委会",
                code: "027",
            },
            VillageCode {
                name: "东城新村第六居委会",
                code: "028",
            },
            VillageCode {
                name: "蓝天居委会",
                code: "029",
            },
            VillageCode {
                name: "东城新村第二居委会",
                code: "030",
            },
            VillageCode {
                name: "东城新村第五居委会",
                code: "031",
            },
            VillageCode {
                name: "东城第七居委会",
                code: "032",
            },
            VillageCode {
                name: "东城新村第八居委会",
                code: "033",
            },
            VillageCode {
                name: "由由八居委会",
                code: "034",
            },
            VillageCode {
                name: "世茂湖滨花园居委会",
                code: "035",
            },
            VillageCode {
                name: "联洋第一居委会",
                code: "036",
            },
            VillageCode {
                name: "联洋第二居委会",
                code: "037",
            },
            VillageCode {
                name: "联洋第三居委会",
                code: "038",
            },
            VillageCode {
                name: "联洋第四居委会",
                code: "039",
            },
            VillageCode {
                name: "联洋第五居委会",
                code: "040",
            },
            VillageCode {
                name: "联洋第六居委会",
                code: "041",
            },
            VillageCode {
                name: "前程居委会",
                code: "042",
            },
            VillageCode {
                name: "御庭居委会",
                code: "043",
            },
            VillageCode {
                name: "湖庭居委会",
                code: "044",
            },
            VillageCode {
                name: "梧桐居委会",
                code: "045",
            },
            VillageCode {
                name: "东城第九居委会",
                code: "046",
            },
            VillageCode {
                name: "培花新村第八居委会",
                code: "047",
            },
            VillageCode {
                name: "牡丹第九居委会",
                code: "048",
            },
            VillageCode {
                name: "兰庭居委会",
                code: "049",
            },
            VillageCode {
                name: "东城第十居委会",
                code: "050",
            },
            VillageCode {
                name: "东城第十一居委会",
                code: "051",
            },
            VillageCode {
                name: "联洋第七居委会",
                code: "052",
            },
            VillageCode {
                name: "牡丹第十居委会",
                code: "053",
            },
            VillageCode {
                name: "龙沟居委会",
                code: "054",
            },
            VillageCode {
                name: "云山居委会",
                code: "055",
            },
            VillageCode {
                name: "联洋第九居委会",
                code: "056",
            },
            VillageCode {
                name: "联洋第八居委会",
                code: "057",
            },
            VillageCode {
                name: "海桐居委会",
                code: "058",
            },
            VillageCode {
                name: "培花新村第五居委会",
                code: "059",
            },
        ],
    },
    TownCode {
        name: "川沙新镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "城西居委会",
                code: "001",
            },
            VillageCode {
                name: "侨光居委会",
                code: "002",
            },
            VillageCode {
                name: "南市居委会",
                code: "003",
            },
            VillageCode {
                name: "临园居委会",
                code: "004",
            },
            VillageCode {
                name: "园西居委会",
                code: "005",
            },
            VillageCode {
                name: "西市居委会",
                code: "006",
            },
            VillageCode {
                name: "川北居委会",
                code: "007",
            },
            VillageCode {
                name: "新川居委会",
                code: "008",
            },
            VillageCode {
                name: "南桥居委会",
                code: "009",
            },
            VillageCode {
                name: "妙虹居委会",
                code: "010",
            },
            VillageCode {
                name: "东市居委会",
                code: "011",
            },
            VillageCode {
                name: "城南居委会",
                code: "012",
            },
            VillageCode {
                name: "曙光居委会",
                code: "013",
            },
            VillageCode {
                name: "桃园居委会",
                code: "014",
            },
            VillageCode {
                name: "新德居委会",
                code: "015",
            },
            VillageCode {
                name: "普新居委会",
                code: "016",
            },
            VillageCode {
                name: "黄楼居委会",
                code: "017",
            },
            VillageCode {
                name: "妙华居委会",
                code: "018",
            },
            VillageCode {
                name: "妙境居委会",
                code: "019",
            },
            VillageCode {
                name: "天乐居委会",
                code: "020",
            },
            VillageCode {
                name: "明珠居委会",
                code: "021",
            },
            VillageCode {
                name: "新苑居委会",
                code: "022",
            },
            VillageCode {
                name: "华盛居委会",
                code: "023",
            },
            VillageCode {
                name: "妙兰居委会",
                code: "024",
            },
            VillageCode {
                name: "沙田居委会",
                code: "025",
            },
            VillageCode {
                name: "申华居委会",
                code: "026",
            },
            VillageCode {
                name: "玉宇居委会",
                code: "027",
            },
            VillageCode {
                name: "妙龙居委会",
                code: "028",
            },
            VillageCode {
                name: "玉兰居委会",
                code: "029",
            },
            VillageCode {
                name: "妙城居委会",
                code: "030",
            },
            VillageCode {
                name: "妙港居委会",
                code: "031",
            },
            VillageCode {
                name: "万馨居委会",
                code: "032",
            },
            VillageCode {
                name: "翔云居委会",
                code: "033",
            },
            VillageCode {
                name: "舒馨居委会",
                code: "034",
            },
            VillageCode {
                name: "华宇居委会",
                code: "035",
            },
            VillageCode {
                name: "鹿城居委会",
                code: "036",
            },
            VillageCode {
                name: "鹿新居委会",
                code: "037",
            },
            VillageCode {
                name: "川迪第一居委会",
                code: "038",
            },
            VillageCode {
                name: "川迪第二居委会",
                code: "039",
            },
            VillageCode {
                name: "川迪第三居委会",
                code: "040",
            },
            VillageCode {
                name: "馨宇居委会",
                code: "041",
            },
            VillageCode {
                name: "川东居委会",
                code: "042",
            },
            VillageCode {
                name: "德馨居委会",
                code: "043",
            },
            VillageCode {
                name: "佳华居委会",
                code: "044",
            },
            VillageCode {
                name: "新虹居委会",
                code: "045",
            },
            VillageCode {
                name: "虹馨居委会",
                code: "046",
            },
            VillageCode {
                name: "虹宇居委会",
                code: "047",
            },
            VillageCode {
                name: "南佳居委会",
                code: "048",
            },
            VillageCode {
                name: "妙新居委会",
                code: "049",
            },
            VillageCode {
                name: "楼厦居委会",
                code: "050",
            },
            VillageCode {
                name: "川南居委会",
                code: "051",
            },
            VillageCode {
                name: "凌川居委会",
                code: "052",
            },
            VillageCode {
                name: "太平村村委会",
                code: "053",
            },
            VillageCode {
                name: "柴场村村委会",
                code: "054",
            },
            VillageCode {
                name: "陈行村村委会",
                code: "055",
            },
            VillageCode {
                name: "虹桥村村委会",
                code: "056",
            },
            VillageCode {
                name: "南高桥村村委会",
                code: "057",
            },
            VillageCode {
                name: "城南村村委会",
                code: "058",
            },
            VillageCode {
                name: "妙境村村委会",
                code: "059",
            },
            VillageCode {
                name: "长丰村村委会",
                code: "060",
            },
            VillageCode {
                name: "对面村村委会",
                code: "061",
            },
            VillageCode {
                name: "湾镇村村委会",
                code: "062",
            },
            VillageCode {
                name: "杜尹村村委会",
                code: "063",
            },
            VillageCode {
                name: "高桥村村委会",
                code: "064",
            },
            VillageCode {
                name: "长桥村村委会",
                code: "065",
            },
            VillageCode {
                name: "储店村村委会",
                code: "066",
            },
            VillageCode {
                name: "新浜村村委会",
                code: "067",
            },
            VillageCode {
                name: "吴店村村委会",
                code: "068",
            },
            VillageCode {
                name: "八灶村村委会",
                code: "069",
            },
            VillageCode {
                name: "牌楼村村委会",
                code: "070",
            },
            VillageCode {
                name: "七灶村村委会",
                code: "071",
            },
            VillageCode {
                name: "纯新村村委会",
                code: "072",
            },
            VillageCode {
                name: "新春村村委会",
                code: "073",
            },
            VillageCode {
                name: "杜坊村村委会",
                code: "074",
            },
            VillageCode {
                name: "界龙村村委会",
                code: "075",
            },
            VillageCode {
                name: "栏杆村村委会",
                code: "076",
            },
            VillageCode {
                name: "黄楼村村委会",
                code: "077",
            },
            VillageCode {
                name: "金家村村委会",
                code: "078",
            },
            VillageCode {
                name: "棋杆村村委会",
                code: "079",
            },
            VillageCode {
                name: "赵行村村委会",
                code: "080",
            },
            VillageCode {
                name: "和平村村委会",
                code: "081",
            },
            VillageCode {
                name: "华路村村委会",
                code: "082",
            },
            VillageCode {
                name: "大洪村村委会",
                code: "083",
            },
            VillageCode {
                name: "民利村村委会",
                code: "084",
            },
            VillageCode {
                name: "会龙村村委会",
                code: "085",
            },
            VillageCode {
                name: "新吉村村委会",
                code: "086",
            },
            VillageCode {
                name: "其成村村委会",
                code: "087",
            },
            VillageCode {
                name: "七星村村委会",
                code: "088",
            },
            VillageCode {
                name: "民义村村委会",
                code: "089",
            },
            VillageCode {
                name: "汤店村村委会",
                code: "090",
            },
            VillageCode {
                name: "鹿溪村村委会",
                code: "091",
            },
            VillageCode {
                name: "连民村村委会",
                code: "092",
            },
            VillageCode {
                name: "陈桥村村委会",
                code: "093",
            },
            VillageCode {
                name: "果园村村委会",
                code: "094",
            },
        ],
    },
    TownCode {
        name: "高桥镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "上炼新村一居委会",
                code: "001",
            },
            VillageCode {
                name: "上炼新村二居委会",
                code: "002",
            },
            VillageCode {
                name: "高桥新村居委会",
                code: "003",
            },
            VillageCode {
                name: "潼港一村居委会",
                code: "004",
            },
            VillageCode {
                name: "东街居委会",
                code: "005",
            },
            VillageCode {
                name: "南街居委会",
                code: "006",
            },
            VillageCode {
                name: "西街居委会",
                code: "007",
            },
            VillageCode {
                name: "富特三居委会",
                code: "008",
            },
            VillageCode {
                name: "富特四居委会",
                code: "009",
            },
            VillageCode {
                name: "富特五居委会",
                code: "010",
            },
            VillageCode {
                name: "潼港二村居委会",
                code: "011",
            },
            VillageCode {
                name: "潼港三村居委会",
                code: "012",
            },
            VillageCode {
                name: "金高居委会",
                code: "013",
            },
            VillageCode {
                name: "凌桥居委会",
                code: "014",
            },
            VillageCode {
                name: "海高新村居委会",
                code: "015",
            },
            VillageCode {
                name: "潼港五村居委会",
                code: "016",
            },
            VillageCode {
                name: "潼港六村居委会",
                code: "017",
            },
            VillageCode {
                name: "潼港八村居委会",
                code: "018",
            },
            VillageCode {
                name: "凌桥第二居委会",
                code: "019",
            },
            VillageCode {
                name: "潼港四村居委会",
                code: "020",
            },
            VillageCode {
                name: "潼港西八村居委会",
                code: "021",
            },
            VillageCode {
                name: "陆凌居委会",
                code: "022",
            },
            VillageCode {
                name: "永久新村居委会",
                code: "023",
            },
            VillageCode {
                name: "学前街居委会",
                code: "024",
            },
            VillageCode {
                name: "高桥新城第一居委会",
                code: "025",
            },
            VillageCode {
                name: "港城新苑居委会",
                code: "026",
            },
            VillageCode {
                name: "凌桥第三居委会",
                code: "027",
            },
            VillageCode {
                name: "和祥佳园居委会",
                code: "028",
            },
            VillageCode {
                name: "凌桥第四居委会",
                code: "029",
            },
            VillageCode {
                name: "凌桥第六居委会",
                code: "030",
            },
            VillageCode {
                name: "凌桥第五居委会",
                code: "031",
            },
            VillageCode {
                name: "高桥新城第二居委会",
                code: "032",
            },
            VillageCode {
                name: "浦凌佳苑居委会",
                code: "033",
            },
            VillageCode {
                name: "潼港七村居委会",
                code: "034",
            },
            VillageCode {
                name: "凌桥第七居委会",
                code: "035",
            },
            VillageCode {
                name: "南塘村村委会",
                code: "036",
            },
            VillageCode {
                name: "陆凌村村委会",
                code: "037",
            },
            VillageCode {
                name: "屯粮巷村村委会",
                code: "038",
            },
            VillageCode {
                name: "新农村村委会",
                code: "039",
            },
            VillageCode {
                name: "镇北村村委会",
                code: "040",
            },
            VillageCode {
                name: "北新村村委会",
                code: "041",
            },
            VillageCode {
                name: "西新村村委会",
                code: "042",
            },
            VillageCode {
                name: "新益村村委会",
                code: "043",
            },
            VillageCode {
                name: "仓房村村委会",
                code: "044",
            },
            VillageCode {
                name: "三岔港村村委会",
                code: "045",
            },
            VillageCode {
                name: "龙叶村村委会",
                code: "046",
            },
            VillageCode {
                name: "凌桥村村委会",
                code: "047",
            },
            VillageCode {
                name: "顾家圩村村委会",
                code: "048",
            },
        ],
    },
    TownCode {
        name: "北蔡镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "虹南居委会",
                code: "001",
            },
            VillageCode {
                name: "香花居委会",
                code: "002",
            },
            VillageCode {
                name: "莲溪一居委会",
                code: "003",
            },
            VillageCode {
                name: "天池居委会",
                code: "004",
            },
            VillageCode {
                name: "莲溪四居委会",
                code: "005",
            },
            VillageCode {
                name: "莲溪六居委会",
                code: "006",
            },
            VillageCode {
                name: "莲溪八居委会",
                code: "007",
            },
            VillageCode {
                name: "鹏海第一居委会",
                code: "008",
            },
            VillageCode {
                name: "鹏海第二居委会",
                code: "009",
            },
            VillageCode {
                name: "和平居委会",
                code: "010",
            },
            VillageCode {
                name: "海东居委会",
                code: "011",
            },
            VillageCode {
                name: "莲溪九居委会",
                code: "012",
            },
            VillageCode {
                name: "艾东居委会",
                code: "013",
            },
            VillageCode {
                name: "艾南居委会",
                code: "014",
            },
            VillageCode {
                name: "下西浜居委会",
                code: "015",
            },
            VillageCode {
                name: "南新一村居委会",
                code: "016",
            },
            VillageCode {
                name: "南新四村居委会",
                code: "017",
            },
            VillageCode {
                name: "南新六村居委会",
                code: "018",
            },
            VillageCode {
                name: "南新七村居委会",
                code: "019",
            },
            VillageCode {
                name: "南江苑居委会",
                code: "020",
            },
            VillageCode {
                name: "南杨居委会",
                code: "021",
            },
            VillageCode {
                name: "金旋居委会",
                code: "022",
            },
            VillageCode {
                name: "绿川新村一居委会",
                code: "023",
            },
            VillageCode {
                name: "绿川新村二居委会",
                code: "024",
            },
            VillageCode {
                name: "绿川新村三居委会",
                code: "025",
            },
            VillageCode {
                name: "绿川新村四居委会",
                code: "026",
            },
            VillageCode {
                name: "安建居委会",
                code: "027",
            },
            VillageCode {
                name: "龙港居委会",
                code: "028",
            },
            VillageCode {
                name: "民乐苑居委会",
                code: "029",
            },
            VillageCode {
                name: "锦华居委会",
                code: "030",
            },
            VillageCode {
                name: "河东居委会",
                code: "031",
            },
            VillageCode {
                name: "大华第一居委会",
                code: "032",
            },
            VillageCode {
                name: "大华第二居委会",
                code: "033",
            },
            VillageCode {
                name: "鹏海第七居委会",
                code: "034",
            },
            VillageCode {
                name: "御桥第一居委会",
                code: "035",
            },
            VillageCode {
                name: "御桥第二居委会",
                code: "036",
            },
            VillageCode {
                name: "大华三居委会",
                code: "037",
            },
            VillageCode {
                name: "大华六居委会",
                code: "038",
            },
            VillageCode {
                name: "香溢居委会",
                code: "039",
            },
            VillageCode {
                name: "春夏居委会",
                code: "040",
            },
            VillageCode {
                name: "御桥第四居委会",
                code: "041",
            },
            VillageCode {
                name: "艾南花苑居委会",
                code: "042",
            },
            VillageCode {
                name: "御桥第三居委会",
                code: "043",
            },
            VillageCode {
                name: "大华第四居委会",
                code: "044",
            },
            VillageCode {
                name: "振东第一居委会",
                code: "045",
            },
            VillageCode {
                name: "鹏海第八居委会",
                code: "046",
            },
            VillageCode {
                name: "龙博居委会",
                code: "047",
            },
            VillageCode {
                name: "鹏海第三居委会",
                code: "048",
            },
            VillageCode {
                name: "大华第八居委会",
                code: "049",
            },
            VillageCode {
                name: "鹏海第四居委会",
                code: "050",
            },
            VillageCode {
                name: "陈桥居委会",
                code: "051",
            },
            VillageCode {
                name: "莲安居委会",
                code: "052",
            },
            VillageCode {
                name: "下南居委会",
                code: "053",
            },
            VillageCode {
                name: "莲中居委会",
                code: "054",
            },
            VillageCode {
                name: "博华居委会",
                code: "055",
            },
            VillageCode {
                name: "御桥第六居委会",
                code: "056",
            },
            VillageCode {
                name: "大华第七居委会",
                code: "057",
            },
            VillageCode {
                name: "紫叶第一居委会",
                code: "058",
            },
            VillageCode {
                name: "紫叶第二居委会",
                code: "059",
            },
            VillageCode {
                name: "御桥第七居委会",
                code: "060",
            },
            VillageCode {
                name: "鹏海第九居委会",
                code: "061",
            },
            VillageCode {
                name: "华润居委会",
                code: "062",
            },
            VillageCode {
                name: "大华第九居委会",
                code: "063",
            },
            VillageCode {
                name: "同福第一居委会",
                code: "064",
            },
            VillageCode {
                name: "鹏海第六居委会",
                code: "065",
            },
            VillageCode {
                name: "杨桥村村委会",
                code: "066",
            },
            VillageCode {
                name: "中界村村委会",
                code: "067",
            },
            VillageCode {
                name: "联勤村村委会",
                code: "068",
            },
            VillageCode {
                name: "五星村村委会",
                code: "069",
            },
            VillageCode {
                name: "御桥村村委会",
                code: "070",
            },
            VillageCode {
                name: "卫行村村委会",
                code: "071",
            },
            VillageCode {
                name: "一六村村委会",
                code: "072",
            },
            VillageCode {
                name: "南新村村委会",
                code: "073",
            },
            VillageCode {
                name: "同福村村委会",
                code: "074",
            },
        ],
    },
    TownCode {
        name: "合庆镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "合庆镇居委会",
                code: "001",
            },
            VillageCode {
                name: "蔡路镇居委会",
                code: "002",
            },
            VillageCode {
                name: "胜利居委会",
                code: "003",
            },
            VillageCode {
                name: "庆南居委会",
                code: "004",
            },
            VillageCode {
                name: "益华居委会",
                code: "005",
            },
            VillageCode {
                name: "庆东居委会",
                code: "006",
            },
            VillageCode {
                name: "庆华居委会",
                code: "007",
            },
            VillageCode {
                name: "庆利居委会",
                code: "008",
            },
            VillageCode {
                name: "向东村村委会",
                code: "009",
            },
            VillageCode {
                name: "前哨村村委会",
                code: "010",
            },
            VillageCode {
                name: "朝阳村村委会",
                code: "011",
            },
            VillageCode {
                name: "勤奋村村委会",
                code: "012",
            },
            VillageCode {
                name: "向阳村村委会",
                code: "013",
            },
            VillageCode {
                name: "共一村村委会",
                code: "014",
            },
            VillageCode {
                name: "奚家村村委会",
                code: "015",
            },
            VillageCode {
                name: "庆丰村村委会",
                code: "016",
            },
            VillageCode {
                name: "庆星村村委会",
                code: "017",
            },
            VillageCode {
                name: "红星村村委会",
                code: "018",
            },
            VillageCode {
                name: "跃丰村村委会",
                code: "019",
            },
            VillageCode {
                name: "永红村村委会",
                code: "020",
            },
            VillageCode {
                name: "东风村村委会",
                code: "021",
            },
            VillageCode {
                name: "直属村村委会",
                code: "022",
            },
            VillageCode {
                name: "海塘村村委会",
                code: "023",
            },
            VillageCode {
                name: "建光村村委会",
                code: "024",
            },
            VillageCode {
                name: "勤昌村村委会",
                code: "025",
            },
            VillageCode {
                name: "春雷村村委会",
                code: "026",
            },
            VillageCode {
                name: "友谊村村委会",
                code: "027",
            },
            VillageCode {
                name: "青四村村委会",
                code: "028",
            },
            VillageCode {
                name: "青三村村委会",
                code: "029",
            },
            VillageCode {
                name: "大星村村委会",
                code: "030",
            },
            VillageCode {
                name: "蔡路村村委会",
                code: "031",
            },
            VillageCode {
                name: "勤益村村委会",
                code: "032",
            },
            VillageCode {
                name: "益民村村委会",
                code: "033",
            },
            VillageCode {
                name: "跃进村村委会",
                code: "034",
            },
            VillageCode {
                name: "华星村村委会",
                code: "035",
            },
            VillageCode {
                name: "勤俭村村委会",
                code: "036",
            },
            VillageCode {
                name: "营房村村委会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "唐镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "东唐苑居委会",
                code: "001",
            },
            VillageCode {
                name: "王港镇居委会",
                code: "002",
            },
            VillageCode {
                name: "唐人苑居委会",
                code: "003",
            },
            VillageCode {
                name: "金枫居委会",
                code: "004",
            },
            VillageCode {
                name: "暮紫桥居委会",
                code: "005",
            },
            VillageCode {
                name: "绿波苑居委会",
                code: "006",
            },
            VillageCode {
                name: "同馨苑居委会",
                code: "007",
            },
            VillageCode {
                name: "金唐居委会",
                code: "008",
            },
            VillageCode {
                name: "齐爱居委会",
                code: "009",
            },
            VillageCode {
                name: "瀚盛居委会",
                code: "010",
            },
            VillageCode {
                name: "金盛居委会",
                code: "011",
            },
            VillageCode {
                name: "唐丰苑居委会",
                code: "012",
            },
            VillageCode {
                name: "齐友佳苑居委会",
                code: "013",
            },
            VillageCode {
                name: "同福居委会",
                code: "014",
            },
            VillageCode {
                name: "上丰居委会",
                code: "015",
            },
            VillageCode {
                name: "金爵居委会",
                code: "016",
            },
            VillageCode {
                name: "保利居委会",
                code: "017",
            },
            VillageCode {
                name: "天歌居委会",
                code: "018",
            },
            VillageCode {
                name: "金利居委会",
                code: "019",
            },
            VillageCode {
                name: "万嘉居委会",
                code: "020",
            },
            VillageCode {
                name: "澜苑居委会",
                code: "021",
            },
            VillageCode {
                name: "樽苑居委会",
                code: "022",
            },
            VillageCode {
                name: "诚礼居委会",
                code: "023",
            },
            VillageCode {
                name: "悦华居委会",
                code: "024",
            },
            VillageCode {
                name: "浦华居委会",
                code: "025",
            },
            VillageCode {
                name: "绿墅居委会",
                code: "026",
            },
            VillageCode {
                name: "培元居委会",
                code: "027",
            },
            VillageCode {
                name: "新镇村村委会",
                code: "028",
            },
            VillageCode {
                name: "大众村村委会",
                code: "029",
            },
            VillageCode {
                name: "吕三村村委会",
                code: "030",
            },
            VillageCode {
                name: "唐四村村委会",
                code: "031",
            },
            VillageCode {
                name: "民丰村村委会",
                code: "032",
            },
            VillageCode {
                name: "前进村村委会",
                code: "033",
            },
            VillageCode {
                name: "唐镇村村委会",
                code: "034",
            },
            VillageCode {
                name: "机口村村委会",
                code: "035",
            },
            VillageCode {
                name: "一心村村委会",
                code: "036",
            },
            VillageCode {
                name: "虹二村村委会",
                code: "037",
            },
            VillageCode {
                name: "虹三村村委会",
                code: "038",
            },
            VillageCode {
                name: "虹四村村委会",
                code: "039",
            },
            VillageCode {
                name: "大丰村村委会",
                code: "040",
            },
            VillageCode {
                name: "暮二村村委会",
                code: "041",
            },
            VillageCode {
                name: "新虹村村委会",
                code: "042",
            },
            VillageCode {
                name: "小湾村村委会",
                code: "043",
            },
        ],
    },
    TownCode {
        name: "曹路镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "顾路镇居委会",
                code: "001",
            },
            VillageCode {
                name: "龚路镇居委会",
                code: "002",
            },
            VillageCode {
                name: "阳光苑居委会",
                code: "003",
            },
            VillageCode {
                name: "阳光苑第二居委会",
                code: "004",
            },
            VillageCode {
                name: "阳光苑第三居委会",
                code: "005",
            },
            VillageCode {
                name: "曹路居委会",
                code: "006",
            },
            VillageCode {
                name: "龚新居委会",
                code: "007",
            },
            VillageCode {
                name: "丰舍苑居委会",
                code: "008",
            },
            VillageCode {
                name: "丰怡苑居委会",
                code: "009",
            },
            VillageCode {
                name: "银丰苑居委会",
                code: "010",
            },
            VillageCode {
                name: "美地芳邻居委会",
                code: "011",
            },
            VillageCode {
                name: "金钻苑第一居委会",
                code: "012",
            },
            VillageCode {
                name: "舜峰家苑居委会",
                code: "013",
            },
            VillageCode {
                name: "海尚东苑居委会",
                code: "014",
            },
            VillageCode {
                name: "中虹家苑居委会",
                code: "015",
            },
            VillageCode {
                name: "星颂家园居委会",
                code: "016",
            },
            VillageCode {
                name: "佳伟景苑居委会",
                code: "017",
            },
            VillageCode {
                name: "星晓家园居委会",
                code: "018",
            },
            VillageCode {
                name: "星海家园居委会",
                code: "019",
            },
            VillageCode {
                name: "星纳家园居委会",
                code: "020",
            },
            VillageCode {
                name: "星金家园居委会",
                code: "021",
            },
            VillageCode {
                name: "金群苑居委会",
                code: "022",
            },
            VillageCode {
                name: "永华苑居委会",
                code: "023",
            },
            VillageCode {
                name: "万科蓝山居委会",
                code: "024",
            },
            VillageCode {
                name: "华美新苑居委会",
                code: "025",
            },
            VillageCode {
                name: "阳光苑第四居委会",
                code: "026",
            },
            VillageCode {
                name: "康平居委会",
                code: "027",
            },
            VillageCode {
                name: "星丰居委会",
                code: "028",
            },
            VillageCode {
                name: "海鹏居委会",
                code: "029",
            },
            VillageCode {
                name: "悦虹居委会",
                code: "030",
            },
            VillageCode {
                name: "金钻苑第二居委会",
                code: "031",
            },
            VillageCode {
                name: "永乐村村委会",
                code: "032",
            },
            VillageCode {
                name: "民建村村委会",
                code: "033",
            },
            VillageCode {
                name: "光耀村村委会",
                code: "034",
            },
            VillageCode {
                name: "兴东村村委会",
                code: "035",
            },
            VillageCode {
                name: "群乐村村委会",
                code: "036",
            },
            VillageCode {
                name: "众三村村委会",
                code: "037",
            },
            VillageCode {
                name: "建新村村委会",
                code: "038",
            },
            VillageCode {
                name: "顾三村村委会",
                code: "039",
            },
            VillageCode {
                name: "联合村村委会",
                code: "040",
            },
            VillageCode {
                name: "东海村村委会",
                code: "041",
            },
            VillageCode {
                name: "顾东村村委会",
                code: "042",
            },
            VillageCode {
                name: "光明村村委会",
                code: "043",
            },
            VillageCode {
                name: "赵桥村村委会",
                code: "044",
            },
            VillageCode {
                name: "安基村村委会",
                code: "045",
            },
            VillageCode {
                name: "直一村村委会",
                code: "046",
            },
            VillageCode {
                name: "直二村村委会",
                code: "047",
            },
            VillageCode {
                name: "黎明村村委会",
                code: "048",
            },
            VillageCode {
                name: "永和村村委会",
                code: "049",
            },
            VillageCode {
                name: "永利村村委会",
                code: "050",
            },
            VillageCode {
                name: "前锋村村委会",
                code: "051",
            },
            VillageCode {
                name: "启明村村委会",
                code: "052",
            },
            VillageCode {
                name: "迅建村村委会",
                code: "053",
            },
            VillageCode {
                name: "曙光村村委会",
                code: "054",
            },
            VillageCode {
                name: "新华村村委会",
                code: "055",
            },
            VillageCode {
                name: "日新村村委会",
                code: "056",
            },
            VillageCode {
                name: "共新村村委会",
                code: "057",
            },
            VillageCode {
                name: "新光村村委会",
                code: "058",
            },
            VillageCode {
                name: "五四村村委会",
                code: "059",
            },
            VillageCode {
                name: "星火村村委会",
                code: "060",
            },
            VillageCode {
                name: "新星村村委会",
                code: "061",
            },
            VillageCode {
                name: "永丰村村委会",
                code: "062",
            },
        ],
    },
    TownCode {
        name: "金桥镇",
        code: "019",
        villages: &[
            VillageCode {
                name: "张桥居委会",
                code: "001",
            },
            VillageCode {
                name: "佳虹居委会",
                code: "002",
            },
            VillageCode {
                name: "金浦居委会",
                code: "003",
            },
            VillageCode {
                name: "阳光第一居委会",
                code: "004",
            },
            VillageCode {
                name: "金桥新城居委会",
                code: "005",
            },
            VillageCode {
                name: "城市家园居委会",
                code: "006",
            },
            VillageCode {
                name: "阳光第二居委会",
                code: "007",
            },
            VillageCode {
                name: "金葵路第一居委会",
                code: "008",
            },
            VillageCode {
                name: "金葵路第二居委会",
                code: "009",
            },
            VillageCode {
                name: "金葵路第三居委会",
                code: "010",
            },
            VillageCode {
                name: "永业第一居委会",
                code: "011",
            },
            VillageCode {
                name: "永业第二居委会",
                code: "012",
            },
            VillageCode {
                name: "碧云第一居委会",
                code: "013",
            },
            VillageCode {
                name: "碧云第二居委会",
                code: "014",
            },
            VillageCode {
                name: "金石居委会",
                code: "015",
            },
            VillageCode {
                name: "金开居委会",
                code: "016",
            },
            VillageCode {
                name: "碧云社区第三居委会",
                code: "017",
            },
            VillageCode {
                name: "王家桥村村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "高行镇",
        code: "020",
        villages: &[
            VillageCode {
                name: "东沟一居委会",
                code: "001",
            },
            VillageCode {
                name: "东沟二居委会",
                code: "002",
            },
            VillageCode {
                name: "东沟四居委会",
                code: "003",
            },
            VillageCode {
                name: "高行居委会",
                code: "004",
            },
            VillageCode {
                name: "东沟居委会",
                code: "005",
            },
            VillageCode {
                name: "华高居委会",
                code: "006",
            },
            VillageCode {
                name: "华高新村第三居委会",
                code: "007",
            },
            VillageCode {
                name: "华高新村第二居委会",
                code: "008",
            },
            VillageCode {
                name: "高行新村第二居委会",
                code: "009",
            },
            VillageCode {
                name: "东沟新村第三居委会",
                code: "010",
            },
            VillageCode {
                name: "华高新村第四居委会",
                code: "011",
            },
            VillageCode {
                name: "南行居委会",
                code: "012",
            },
            VillageCode {
                name: "东力新村居委会",
                code: "013",
            },
            VillageCode {
                name: "紫翠居委会",
                code: "014",
            },
            VillageCode {
                name: "银杏居委会",
                code: "015",
            },
            VillageCode {
                name: "浦江居委会",
                code: "016",
            },
            VillageCode {
                name: "绿地居委会",
                code: "017",
            },
            VillageCode {
                name: "幸福小镇居委会",
                code: "018",
            },
            VillageCode {
                name: "华庭居委会",
                code: "019",
            },
            VillageCode {
                name: "南新居委会",
                code: "020",
            },
            VillageCode {
                name: "绿洲第一居委会",
                code: "021",
            },
            VillageCode {
                name: "森兰金地居委会",
                code: "022",
            },
            VillageCode {
                name: "绿洲第二居委会",
                code: "023",
            },
            VillageCode {
                name: "绿洲第三居委会",
                code: "024",
            },
            VillageCode {
                name: "绿洲第四居委会",
                code: "025",
            },
            VillageCode {
                name: "东旭居委会",
                code: "026",
            },
            VillageCode {
                name: "森兰海弘居委会",
                code: "027",
            },
            VillageCode {
                name: "森兰尚城居委会",
                code: "028",
            },
            VillageCode {
                name: "海韵居委会",
                code: "029",
            },
            VillageCode {
                name: "森兰绿城居委会",
                code: "030",
            },
            VillageCode {
                name: "汇郡居委会",
                code: "031",
            },
            VillageCode {
                name: "森兰新城居委会",
                code: "032",
            },
            VillageCode {
                name: "森兰仁恒居委会",
                code: "033",
            },
            VillageCode {
                name: "森兰名轩居委会",
                code: "034",
            },
            VillageCode {
                name: "高申居委会",
                code: "035",
            },
            VillageCode {
                name: "南行第二居委会",
                code: "036",
            },
            VillageCode {
                name: "周桥村村委会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "高东镇",
        code: "021",
        villages: &[
            VillageCode {
                name: "杨园新村第一居委会",
                code: "001",
            },
            VillageCode {
                name: "杨园新村第二居委会",
                code: "002",
            },
            VillageCode {
                name: "高东新村居委会",
                code: "003",
            },
            VillageCode {
                name: "千秋嘉苑居委会",
                code: "004",
            },
            VillageCode {
                name: "欣连苑居委会",
                code: "005",
            },
            VillageCode {
                name: "先锋居委会",
                code: "006",
            },
            VillageCode {
                name: "高东新村第二居委会",
                code: "007",
            },
            VillageCode {
                name: "高东新村第三居委会",
                code: "008",
            },
            VillageCode {
                name: "新高苑居委会",
                code: "009",
            },
            VillageCode {
                name: "新高苑第二居委会",
                code: "010",
            },
            VillageCode {
                name: "杨园新村第三居委会",
                code: "011",
            },
            VillageCode {
                name: "杨园新村第四居委会",
                code: "012",
            },
            VillageCode {
                name: "新高苑第三居委会",
                code: "013",
            },
            VillageCode {
                name: "楼夏景园居委会",
                code: "014",
            },
            VillageCode {
                name: "品欣雅苑居委会",
                code: "015",
            },
            VillageCode {
                name: "宝佳苑居委会",
                code: "016",
            },
            VillageCode {
                name: "楼下佳苑居委会",
                code: "017",
            },
            VillageCode {
                name: "珊璜村村委会",
                code: "018",
            },
            VillageCode {
                name: "张家宅村村委会",
                code: "019",
            },
            VillageCode {
                name: "楼下村村委会",
                code: "020",
            },
            VillageCode {
                name: "沙港村村委会",
                code: "021",
            },
            VillageCode {
                name: "金光村村委会",
                code: "022",
            },
            VillageCode {
                name: "竞赛村村委会",
                code: "023",
            },
            VillageCode {
                name: "踊跃村村委会",
                code: "024",
            },
            VillageCode {
                name: "永新村村委会",
                code: "025",
            },
            VillageCode {
                name: "革新村村委会",
                code: "026",
            },
            VillageCode {
                name: "徐路村村委会",
                code: "027",
            },
            VillageCode {
                name: "上游村村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "张江镇",
        code: "022",
        villages: &[
            VillageCode {
                name: "川和路居委会",
                code: "001",
            },
            VillageCode {
                name: "军民路居委会",
                code: "002",
            },
            VillageCode {
                name: "田园居委会",
                code: "003",
            },
            VillageCode {
                name: "杨镇居委会",
                code: "004",
            },
            VillageCode {
                name: "孙桥路居委会",
                code: "005",
            },
            VillageCode {
                name: "棕桐居委会",
                code: "006",
            },
            VillageCode {
                name: "金桐居委会",
                code: "007",
            },
            VillageCode {
                name: "江春居委会",
                code: "008",
            },
            VillageCode {
                name: "丹桂路居委会",
                code: "009",
            },
            VillageCode {
                name: "古桐居委会",
                code: "010",
            },
            VillageCode {
                name: "香楠路居委会",
                code: "011",
            },
            VillageCode {
                name: "孙建路居委会",
                code: "012",
            },
            VillageCode {
                name: "江兰居委会",
                code: "013",
            },
            VillageCode {
                name: "江丰居委会",
                code: "014",
            },
            VillageCode {
                name: "碧波路居委会",
                code: "015",
            },
            VillageCode {
                name: "江夏居委会",
                code: "016",
            },
            VillageCode {
                name: "江衡居委会",
                code: "017",
            },
            VillageCode {
                name: "华晶居委会",
                code: "018",
            },
            VillageCode {
                name: "江苑居委会",
                code: "019",
            },
            VillageCode {
                name: "华顺居委会",
                code: "020",
            },
            VillageCode {
                name: "江益居委会",
                code: "021",
            },
            VillageCode {
                name: "晨晖居委会",
                code: "022",
            },
            VillageCode {
                name: "华科居委会",
                code: "023",
            },
            VillageCode {
                name: "汤臣豪庭居委会",
                code: "024",
            },
            VillageCode {
                name: "城市经典居委会",
                code: "025",
            },
            VillageCode {
                name: "藿香路居委会",
                code: "026",
            },
            VillageCode {
                name: "江薇居委会",
                code: "027",
            },
            VillageCode {
                name: "华杨居委会",
                code: "028",
            },
            VillageCode {
                name: "高木桥路居委会",
                code: "029",
            },
            VillageCode {
                name: "孙环路居委会",
                code: "030",
            },
            VillageCode {
                name: "科苑路居委会",
                code: "031",
            },
            VillageCode {
                name: "亮秀路居委会",
                code: "032",
            },
            VillageCode {
                name: "樟盛苑居委会",
                code: "033",
            },
            VillageCode {
                name: "荣科居委会",
                code: "034",
            },
            VillageCode {
                name: "孙耀路居委会",
                code: "035",
            },
            VillageCode {
                name: "江伟居委会",
                code: "036",
            },
            VillageCode {
                name: "华和居委会",
                code: "037",
            },
            VillageCode {
                name: "华川居委会",
                code: "038",
            },
            VillageCode {
                name: "广兰居委会",
                code: "039",
            },
            VillageCode {
                name: "勤政路居委会",
                code: "040",
            },
            VillageCode {
                name: "汤臣高尔夫居委会",
                code: "041",
            },
            VillageCode {
                name: "中芯花园居委会",
                code: "042",
            },
            VillageCode {
                name: "韩荡村村委会",
                code: "043",
            },
            VillageCode {
                name: "钱堂村村委会",
                code: "044",
            },
            VillageCode {
                name: "劳动村村委会",
                code: "045",
            },
            VillageCode {
                name: "新丰村村委会",
                code: "046",
            },
            VillageCode {
                name: "长元村村委会",
                code: "047",
            },
            VillageCode {
                name: "中心村村委会",
                code: "048",
            },
            VillageCode {
                name: "环东村村委会",
                code: "049",
            },
            VillageCode {
                name: "沔北村村委会",
                code: "050",
            },
        ],
    },
    TownCode {
        name: "三林镇",
        code: "023",
        villages: &[
            VillageCode {
                name: "三林老街居委会",
                code: "001",
            },
            VillageCode {
                name: "南街居委会",
                code: "002",
            },
            VillageCode {
                name: "北街居委会",
                code: "003",
            },
            VillageCode {
                name: "杨新居委会",
                code: "004",
            },
            VillageCode {
                name: "思浦居委会",
                code: "005",
            },
            VillageCode {
                name: "杨东居委会",
                code: "006",
            },
            VillageCode {
                name: "长清路一居委会",
                code: "007",
            },
            VillageCode {
                name: "海阳路居委会",
                code: "008",
            },
            VillageCode {
                name: "香樟园居委会",
                code: "009",
            },
            VillageCode {
                name: "杨南居委会",
                code: "010",
            },
            VillageCode {
                name: "新华居委会",
                code: "011",
            },
            VillageCode {
                name: "三林新村第五居委会",
                code: "012",
            },
            VillageCode {
                name: "仁文居委会",
                code: "013",
            },
            VillageCode {
                name: "华城居委会",
                code: "014",
            },
            VillageCode {
                name: "玲珑苑居委会",
                code: "015",
            },
            VillageCode {
                name: "山水苑居委会",
                code: "016",
            },
            VillageCode {
                name: "申江豪城居委会",
                code: "017",
            },
            VillageCode {
                name: "岭南花苑居委会",
                code: "018",
            },
            VillageCode {
                name: "明丰花苑居委会",
                code: "019",
            },
            VillageCode {
                name: "金苹果花苑居委会",
                code: "020",
            },
            VillageCode {
                name: "永泰路第一居委会",
                code: "021",
            },
            VillageCode {
                name: "永泰路第二居委会",
                code: "022",
            },
            VillageCode {
                name: "翰城居委会",
                code: "023",
            },
            VillageCode {
                name: "永泰路第三居委会",
                code: "024",
            },
            VillageCode {
                name: "三林世博家园第三居委会",
                code: "025",
            },
            VillageCode {
                name: "永泰路第四居委会",
                code: "026",
            },
            VillageCode {
                name: "永泰路第五居委会",
                code: "027",
            },
            VillageCode {
                name: "世博家园第四居委会",
                code: "028",
            },
            VillageCode {
                name: "杨思路第二居委会",
                code: "029",
            },
            VillageCode {
                name: "杨南第二居委会",
                code: "030",
            },
            VillageCode {
                name: "杨思路第三居委会",
                code: "031",
            },
            VillageCode {
                name: "尚东国际居委会",
                code: "032",
            },
            VillageCode {
                name: "长清路第二居委会",
                code: "033",
            },
            VillageCode {
                name: "杨思路第一居委会",
                code: "034",
            },
            VillageCode {
                name: "三林新村第二居委会",
                code: "035",
            },
            VillageCode {
                name: "三林世博家园第五居委会",
                code: "036",
            },
            VillageCode {
                name: "世博家园南二居委会",
                code: "037",
            },
            VillageCode {
                name: "世博家园北二居委会",
                code: "038",
            },
            VillageCode {
                name: "三林博学家园居委会",
                code: "039",
            },
            VillageCode {
                name: "懿德新村第一居委会",
                code: "040",
            },
            VillageCode {
                name: "永泰路第六居委会",
                code: "041",
            },
            VillageCode {
                name: "懿德第二居委会",
                code: "042",
            },
            VillageCode {
                name: "三林新村第三居委会",
                code: "043",
            },
            VillageCode {
                name: "城林美苑居委会",
                code: "044",
            },
            VillageCode {
                name: "绿波家园居委会",
                code: "045",
            },
            VillageCode {
                name: "三杨新村居委会",
                code: "046",
            },
            VillageCode {
                name: "盛世南苑居委会",
                code: "047",
            },
            VillageCode {
                name: "士韵家园居委会",
                code: "048",
            },
            VillageCode {
                name: "同康苑居委会",
                code: "049",
            },
            VillageCode {
                name: "三林新村第四居委会",
                code: "050",
            },
            VillageCode {
                name: "依水园居委会",
                code: "051",
            },
            VillageCode {
                name: "德康苑居委会",
                code: "052",
            },
            VillageCode {
                name: "士韵家园第二居委会",
                code: "053",
            },
            VillageCode {
                name: "依水园第二居委会",
                code: "054",
            },
            VillageCode {
                name: "三林世博家园东一居委会",
                code: "055",
            },
            VillageCode {
                name: "三林世博家园西一居委会",
                code: "056",
            },
            VillageCode {
                name: "城林雅苑居委会",
                code: "057",
            },
            VillageCode {
                name: "前滩紫丁香居委会",
                code: "058",
            },
            VillageCode {
                name: "前滩紫薇居委会",
                code: "059",
            },
            VillageCode {
                name: "前滩紫茉莉居委会",
                code: "060",
            },
            VillageCode {
                name: "前滩紫荆居委会",
                code: "061",
            },
            VillageCode {
                name: "前滩紫藤居委会",
                code: "062",
            },
            VillageCode {
                name: "凌虹居委会",
                code: "063",
            },
            VillageCode {
                name: "东方康安居委会",
                code: "064",
            },
            VillageCode {
                name: "瀚锦苑居委会",
                code: "065",
            },
            VillageCode {
                name: "高青路居委会",
                code: "066",
            },
            VillageCode {
                name: "绿菌苑居委会",
                code: "067",
            },
            VillageCode {
                name: "海阳路第二居委会",
                code: "068",
            },
            VillageCode {
                name: "士韵家园第三居委会",
                code: "069",
            },
            VillageCode {
                name: "东明村村委会",
                code: "070",
            },
            VillageCode {
                name: "三民村村委会",
                code: "071",
            },
            VillageCode {
                name: "联丰村村委会",
                code: "072",
            },
            VillageCode {
                name: "中林村村委会",
                code: "073",
            },
            VillageCode {
                name: "三林村村委会",
                code: "074",
            },
            VillageCode {
                name: "临江村村委会",
                code: "075",
            },
            VillageCode {
                name: "新春村村委会",
                code: "076",
            },
            VillageCode {
                name: "西林村村委会",
                code: "077",
            },
            VillageCode {
                name: "红旗村村委会",
                code: "078",
            },
            VillageCode {
                name: "懿德村村委会",
                code: "079",
            },
            VillageCode {
                name: "天花庵村村委会",
                code: "080",
            },
            VillageCode {
                name: "金光村村委会",
                code: "081",
            },
            VillageCode {
                name: "南阜村村委会",
                code: "082",
            },
            VillageCode {
                name: "胡巷村村委会",
                code: "083",
            },
        ],
    },
    TownCode {
        name: "惠南镇",
        code: "024",
        villages: &[
            VillageCode {
                name: "东门居委会",
                code: "001",
            },
            VillageCode {
                name: "南门居委会",
                code: "002",
            },
            VillageCode {
                name: "北门居委会",
                code: "003",
            },
            VillageCode {
                name: "荡湾居委会",
                code: "004",
            },
            VillageCode {
                name: "西门居委会",
                code: "005",
            },
            VillageCode {
                name: "听北居委会",
                code: "006",
            },
            VillageCode {
                name: "听南居委会",
                code: "007",
            },
            VillageCode {
                name: "梅花居委会",
                code: "008",
            },
            VillageCode {
                name: "东城第一居委会",
                code: "009",
            },
            VillageCode {
                name: "黄路居委会",
                code: "010",
            },
            VillageCode {
                name: "卫星居委会",
                code: "011",
            },
            VillageCode {
                name: "红光居委会",
                code: "012",
            },
            VillageCode {
                name: "靖海居委会",
                code: "013",
            },
            VillageCode {
                name: "海燕居委会",
                code: "014",
            },
            VillageCode {
                name: "泰燕社区居委会",
                code: "015",
            },
            VillageCode {
                name: "迎薰社区居委会",
                code: "016",
            },
            VillageCode {
                name: "西城社区居委会",
                code: "017",
            },
            VillageCode {
                name: "明光居委会",
                code: "018",
            },
            VillageCode {
                name: "观海居委会",
                code: "019",
            },
            VillageCode {
                name: "学海居委会",
                code: "020",
            },
            VillageCode {
                name: "惠城居委会",
                code: "021",
            },
            VillageCode {
                name: "惠园居委会",
                code: "022",
            },
            VillageCode {
                name: "南园居委会",
                code: "023",
            },
            VillageCode {
                name: "绿城居委会",
                code: "024",
            },
            VillageCode {
                name: "南城居委会",
                code: "025",
            },
            VillageCode {
                name: "惠民居委会",
                code: "026",
            },
            VillageCode {
                name: "丰海居委会",
                code: "027",
            },
            VillageCode {
                name: "德盈居委会",
                code: "028",
            },
            VillageCode {
                name: "惠益居委会",
                code: "029",
            },
            VillageCode {
                name: "汇雅苑居委会",
                code: "030",
            },
            VillageCode {
                name: "宝业居委会",
                code: "031",
            },
            VillageCode {
                name: "颐景园居委会",
                code: "032",
            },
            VillageCode {
                name: "海曲雅苑居委会",
                code: "033",
            },
            VillageCode {
                name: "丽水雅苑居委会",
                code: "034",
            },
            VillageCode {
                name: "兰丽苑居委会",
                code: "035",
            },
            VillageCode {
                name: "秀园居委会",
                code: "036",
            },
            VillageCode {
                name: "荣春苑居委会",
                code: "037",
            },
            VillageCode {
                name: "朗馨苑居委会",
                code: "038",
            },
            VillageCode {
                name: "远洋万和居委会",
                code: "039",
            },
            VillageCode {
                name: "康锦苑居委会",
                code: "040",
            },
            VillageCode {
                name: "惠康第一居委会",
                code: "041",
            },
            VillageCode {
                name: "惠康第二居委会",
                code: "042",
            },
            VillageCode {
                name: "泓康居委会",
                code: "043",
            },
            VillageCode {
                name: "建欣苑居委会",
                code: "044",
            },
            VillageCode {
                name: "文竹居委会",
                code: "045",
            },
            VillageCode {
                name: "丽和居委会",
                code: "046",
            },
            VillageCode {
                name: "惠桐居委会",
                code: "047",
            },
            VillageCode {
                name: "东盛居委会",
                code: "048",
            },
            VillageCode {
                name: "听彩居委会",
                code: "049",
            },
            VillageCode {
                name: "文化居委会",
                code: "050",
            },
            VillageCode {
                name: "政海居委会",
                code: "051",
            },
            VillageCode {
                name: "勤奋居委会",
                code: "052",
            },
            VillageCode {
                name: "东城第二居委会",
                code: "053",
            },
            VillageCode {
                name: "鸿飞第一居委会",
                code: "054",
            },
            VillageCode {
                name: "西门村村委会",
                code: "055",
            },
            VillageCode {
                name: "民乐村村委会",
                code: "056",
            },
            VillageCode {
                name: "红光村村委会",
                code: "057",
            },
            VillageCode {
                name: "长江村村委会",
                code: "058",
            },
            VillageCode {
                name: "陆路村村委会",
                code: "059",
            },
            VillageCode {
                name: "勤丰村村委会",
                code: "060",
            },
            VillageCode {
                name: "惠东村村委会",
                code: "061",
            },
            VillageCode {
                name: "塘路村村委会",
                code: "062",
            },
            VillageCode {
                name: "英雄村村委会",
                code: "063",
            },
            VillageCode {
                name: "同治村村委会",
                code: "064",
            },
            VillageCode {
                name: "双店村村委会",
                code: "065",
            },
            VillageCode {
                name: "六灶湾村村委会",
                code: "066",
            },
            VillageCode {
                name: "桥北村村委会",
                code: "067",
            },
            VillageCode {
                name: "四墩村村委会",
                code: "068",
            },
            VillageCode {
                name: "团结村村委会",
                code: "069",
            },
            VillageCode {
                name: "幸福村村委会",
                code: "070",
            },
            VillageCode {
                name: "黄路村村委会",
                code: "071",
            },
            VillageCode {
                name: "远东村村委会",
                code: "072",
            },
            VillageCode {
                name: "海沈村村委会",
                code: "073",
            },
            VillageCode {
                name: "东征村村委会",
                code: "074",
            },
            VillageCode {
                name: "永乐村村委会",
                code: "075",
            },
            VillageCode {
                name: "陆楼村村委会",
                code: "076",
            },
            VillageCode {
                name: "城南村村委会",
                code: "077",
            },
            VillageCode {
                name: "陶桥村村委会",
                code: "078",
            },
            VillageCode {
                name: "徐庙村村委会",
                code: "079",
            },
            VillageCode {
                name: "汇南村村委会",
                code: "080",
            },
        ],
    },
    TownCode {
        name: "周浦镇",
        code: "025",
        villages: &[
            VillageCode {
                name: "向阳居委会",
                code: "001",
            },
            VillageCode {
                name: "东南居委会",
                code: "002",
            },
            VillageCode {
                name: "公元居委会",
                code: "003",
            },
            VillageCode {
                name: "中市居委会",
                code: "004",
            },
            VillageCode {
                name: "周东居委会",
                code: "005",
            },
            VillageCode {
                name: "澧溪居委会",
                code: "006",
            },
            VillageCode {
                name: "瓦屑居委会",
                code: "007",
            },
            VillageCode {
                name: "果园居委会",
                code: "008",
            },
            VillageCode {
                name: "汇丽社区居委会",
                code: "009",
            },
            VillageCode {
                name: "幸福社区居委会",
                code: "010",
            },
            VillageCode {
                name: "安居社区居委会",
                code: "011",
            },
            VillageCode {
                name: "欧风社区居委会",
                code: "012",
            },
            VillageCode {
                name: "汇腾社区居委会",
                code: "013",
            },
            VillageCode {
                name: "海达社区居委会",
                code: "014",
            },
            VillageCode {
                name: "横桥社区居委会",
                code: "015",
            },
            VillageCode {
                name: "丽都苑居委会",
                code: "016",
            },
            VillageCode {
                name: "中虹佳园居委会",
                code: "017",
            },
            VillageCode {
                name: "瑞阳苑居委会",
                code: "018",
            },
            VillageCode {
                name: "中城苑居委会",
                code: "019",
            },
            VillageCode {
                name: "安阁苑居委会",
                code: "020",
            },
            VillageCode {
                name: "小上海新城居委会",
                code: "021",
            },
            VillageCode {
                name: "康泰居委会",
                code: "022",
            },
            VillageCode {
                name: "欣逸居委会",
                code: "023",
            },
            VillageCode {
                name: "印象春城居委会",
                code: "024",
            },
            VillageCode {
                name: "南八灶居委会",
                code: "025",
            },
            VillageCode {
                name: "华庭居委会",
                code: "026",
            },
            VillageCode {
                name: "御沁园居委会",
                code: "027",
            },
            VillageCode {
                name: "华城居委会",
                code: "028",
            },
            VillageCode {
                name: "汇康居委会",
                code: "029",
            },
            VillageCode {
                name: "吉祥里居委会",
                code: "030",
            },
            VillageCode {
                name: "兴盛里居委会",
                code: "031",
            },
            VillageCode {
                name: "颐谷苑居委会",
                code: "032",
            },
            VillageCode {
                name: "昌盛里居委会",
                code: "033",
            },
            VillageCode {
                name: "平安里居委会",
                code: "034",
            },
            VillageCode {
                name: "九龙仓居委会",
                code: "035",
            },
            VillageCode {
                name: "中金海棠湾居委会",
                code: "036",
            },
            VillageCode {
                name: "健康里居委会",
                code: "037",
            },
            VillageCode {
                name: "祥和里居委会",
                code: "038",
            },
            VillageCode {
                name: "安康里居委会",
                code: "039",
            },
            VillageCode {
                name: "保利艾庐居委会",
                code: "040",
            },
            VillageCode {
                name: "繁荣馨苑居委会",
                code: "041",
            },
            VillageCode {
                name: "华盛里居委会",
                code: "042",
            },
            VillageCode {
                name: "三泰里居委会",
                code: "043",
            },
            VillageCode {
                name: "海棠名苑居委会",
                code: "044",
            },
            VillageCode {
                name: "欣周居委会",
                code: "045",
            },
            VillageCode {
                name: "安雅居委会",
                code: "046",
            },
            VillageCode {
                name: "东悦城居委会",
                code: "047",
            },
            VillageCode {
                name: "姚桥村村委会",
                code: "048",
            },
            VillageCode {
                name: "牛桥村村委会",
                code: "049",
            },
            VillageCode {
                name: "周南村村委会",
                code: "050",
            },
            VillageCode {
                name: "沈西村村委会",
                code: "051",
            },
            VillageCode {
                name: "里仁村村委会",
                code: "052",
            },
            VillageCode {
                name: "棋杆村村委会",
                code: "053",
            },
            VillageCode {
                name: "瓦南村村委会",
                code: "054",
            },
            VillageCode {
                name: "红桥村村委会",
                code: "055",
            },
            VillageCode {
                name: "界浜村村委会",
                code: "056",
            },
            VillageCode {
                name: "北庄村村委会",
                code: "057",
            },
        ],
    },
    TownCode {
        name: "新场镇",
        code: "026",
        villages: &[
            VillageCode {
                name: "南大居委会",
                code: "001",
            },
            VillageCode {
                name: "工农居委会",
                code: "002",
            },
            VillageCode {
                name: "北大居委会",
                code: "003",
            },
            VillageCode {
                name: "石笋居委会",
                code: "004",
            },
            VillageCode {
                name: "坦直居委会",
                code: "005",
            },
            VillageCode {
                name: "东城居委会",
                code: "006",
            },
            VillageCode {
                name: "笋南居委会",
                code: "007",
            },
            VillageCode {
                name: "汇锦城居委会",
                code: "008",
            },
            VillageCode {
                name: "东城第二居委会",
                code: "009",
            },
            VillageCode {
                name: "东城第三居委会",
                code: "010",
            },
            VillageCode {
                name: "笋北居委会",
                code: "011",
            },
            VillageCode {
                name: "汇庭居委会",
                code: "012",
            },
            VillageCode {
                name: "王桥村村委会",
                code: "013",
            },
            VillageCode {
                name: "金建村村委会",
                code: "014",
            },
            VillageCode {
                name: "新卫村村委会",
                code: "015",
            },
            VillageCode {
                name: "果园村村委会",
                code: "016",
            },
            VillageCode {
                name: "新南村村委会",
                code: "017",
            },
            VillageCode {
                name: "众安村村委会",
                code: "018",
            },
            VillageCode {
                name: "新场村村委会",
                code: "019",
            },
            VillageCode {
                name: "坦西村村委会",
                code: "020",
            },
            VillageCode {
                name: "仁义村村委会",
                code: "021",
            },
            VillageCode {
                name: "坦东村村委会",
                code: "022",
            },
            VillageCode {
                name: "蒋桥村村委会",
                code: "023",
            },
            VillageCode {
                name: "祝桥村村委会",
                code: "024",
            },
            VillageCode {
                name: "坦南村村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "大团镇",
        code: "027",
        villages: &[
            VillageCode {
                name: "南大居委会",
                code: "001",
            },
            VillageCode {
                name: "北大居委会",
                code: "002",
            },
            VillageCode {
                name: "三墩集镇居委会",
                code: "003",
            },
            VillageCode {
                name: "东大社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东南居委会",
                code: "005",
            },
            VillageCode {
                name: "镇南村村委会",
                code: "006",
            },
            VillageCode {
                name: "车站村村委会",
                code: "007",
            },
            VillageCode {
                name: "果园村村委会",
                code: "008",
            },
            VillageCode {
                name: "金石村村委会",
                code: "009",
            },
            VillageCode {
                name: "赵桥村村委会",
                code: "010",
            },
            VillageCode {
                name: "海潮村村委会",
                code: "011",
            },
            VillageCode {
                name: "邵宅村村委会",
                code: "012",
            },
            VillageCode {
                name: "团西村村委会",
                code: "013",
            },
            VillageCode {
                name: "金园村村委会",
                code: "014",
            },
            VillageCode {
                name: "金桥村村委会",
                code: "015",
            },
            VillageCode {
                name: "园艺村村委会",
                code: "016",
            },
            VillageCode {
                name: "龙树村村委会",
                code: "017",
            },
            VillageCode {
                name: "周埠村村委会",
                code: "018",
            },
            VillageCode {
                name: "扶栏村村委会",
                code: "019",
            },
            VillageCode {
                name: "团新村村委会",
                code: "020",
            },
            VillageCode {
                name: "邵村村村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "康桥镇",
        code: "028",
        villages: &[
            VillageCode {
                name: "周康居委会",
                code: "001",
            },
            VillageCode {
                name: "梓潼居委会",
                code: "002",
            },
            VillageCode {
                name: "百曲居委会",
                code: "003",
            },
            VillageCode {
                name: "营房居委会",
                code: "004",
            },
            VillageCode {
                name: "花墙居委会",
                code: "005",
            },
            VillageCode {
                name: "和合居委会",
                code: "006",
            },
            VillageCode {
                name: "横沔居委会",
                code: "007",
            },
            VillageCode {
                name: "汤巷中心居委会",
                code: "008",
            },
            VillageCode {
                name: "康桥半岛居委会",
                code: "009",
            },
            VillageCode {
                name: "秀龙居委会",
                code: "010",
            },
            VillageCode {
                name: "康桥老街居委会",
                code: "011",
            },
            VillageCode {
                name: "康桥花园居委会",
                code: "012",
            },
            VillageCode {
                name: "双秀家园居委会",
                code: "013",
            },
            VillageCode {
                name: "公元三村居委会",
                code: "014",
            },
            VillageCode {
                name: "南华城居委会",
                code: "015",
            },
            VillageCode {
                name: "昱龙社区居委会",
                code: "016",
            },
            VillageCode {
                name: "美林社区居委会",
                code: "017",
            },
            VillageCode {
                name: "中邦社区居委会",
                code: "018",
            },
            VillageCode {
                name: "海尚康庭居委会",
                code: "019",
            },
            VillageCode {
                name: "康桥宝邸居委会",
                code: "020",
            },
            VillageCode {
                name: "康桥月苑居委会",
                code: "021",
            },
            VillageCode {
                name: "城中花园居委会",
                code: "022",
            },
            VillageCode {
                name: "汤巷馨村居委会",
                code: "023",
            },
            VillageCode {
                name: "文怡苑居委会",
                code: "024",
            },
            VillageCode {
                name: "富康居委会",
                code: "025",
            },
            VillageCode {
                name: "锦绣居委会",
                code: "026",
            },
            VillageCode {
                name: "宁怡居委会",
                code: "027",
            },
            VillageCode {
                name: "天台居委会",
                code: "028",
            },
            VillageCode {
                name: "林语溪居委会",
                code: "029",
            },
            VillageCode {
                name: "香颂居委会",
                code: "030",
            },
            VillageCode {
                name: "御景居委会",
                code: "031",
            },
            VillageCode {
                name: "双秀西园居委会",
                code: "032",
            },
            VillageCode {
                name: "海富居委会",
                code: "033",
            },
            VillageCode {
                name: "康湾苑居委会",
                code: "034",
            },
            VillageCode {
                name: "汤巷雅苑居委会",
                code: "035",
            },
            VillageCode {
                name: "秀怡苑居委会",
                code: "036",
            },
            VillageCode {
                name: "中邦大都会居委会",
                code: "037",
            },
            VillageCode {
                name: "康景居委会",
                code: "038",
            },
            VillageCode {
                name: "绿洲康城第一居委会",
                code: "039",
            },
            VillageCode {
                name: "周康新苑居委会",
                code: "040",
            },
            VillageCode {
                name: "康桥半岛新城居委会",
                code: "041",
            },
            VillageCode {
                name: "康桥花园第二居委会",
                code: "042",
            },
            VillageCode {
                name: "颐盛居委会",
                code: "043",
            },
            VillageCode {
                name: "沔新居委会",
                code: "044",
            },
            VillageCode {
                name: "绿洲康城第二居委会",
                code: "045",
            },
            VillageCode {
                name: "绿洲康城第三居委会",
                code: "046",
            },
            VillageCode {
                name: "都市花园居委会",
                code: "047",
            },
            VillageCode {
                name: "康涵居委会",
                code: "048",
            },
            VillageCode {
                name: "仁怡苑居委会",
                code: "049",
            },
            VillageCode {
                name: "东康名苑居委会",
                code: "050",
            },
            VillageCode {
                name: "百林苑居委会",
                code: "051",
            },
            VillageCode {
                name: "太平村村委会",
                code: "052",
            },
            VillageCode {
                name: "沔青村村委会",
                code: "053",
            },
            VillageCode {
                name: "石门村村委会",
                code: "054",
            },
            VillageCode {
                name: "火箭村村委会",
                code: "055",
            },
            VillageCode {
                name: "怡园村村委会",
                code: "056",
            },
            VillageCode {
                name: "沿南村村委会",
                code: "057",
            },
            VillageCode {
                name: "人南村村委会",
                code: "058",
            },
            VillageCode {
                name: "汤巷村村委会",
                code: "059",
            },
            VillageCode {
                name: "叠桥村村委会",
                code: "060",
            },
            VillageCode {
                name: "沿北村村委会",
                code: "061",
            },
            VillageCode {
                name: "新苗村村委会",
                code: "062",
            },
        ],
    },
    TownCode {
        name: "航头镇",
        code: "029",
        villages: &[
            VillageCode {
                name: "航头居委会",
                code: "001",
            },
            VillageCode {
                name: "下沙居委会",
                code: "002",
            },
            VillageCode {
                name: "瑞和苑居委会",
                code: "003",
            },
            VillageCode {
                name: "鹤鸣居委会",
                code: "004",
            },
            VillageCode {
                name: "长达居委会",
                code: "005",
            },
            VillageCode {
                name: "金色航城居委会",
                code: "006",
            },
            VillageCode {
                name: "东升家园居委会",
                code: "007",
            },
            VillageCode {
                name: "海洲桃花园居委会",
                code: "008",
            },
            VillageCode {
                name: "沉香居委会",
                code: "009",
            },
            VillageCode {
                name: "南馨佳苑居委会",
                code: "010",
            },
            VillageCode {
                name: "恒福家园居委会",
                code: "011",
            },
            VillageCode {
                name: "昱丽家园居委会",
                code: "012",
            },
            VillageCode {
                name: "瑞浦嘉苑居委会",
                code: "013",
            },
            VillageCode {
                name: "航武嘉园居委会",
                code: "014",
            },
            VillageCode {
                name: "聚航苑第一居委会",
                code: "015",
            },
            VillageCode {
                name: "长泰东郊居委会",
                code: "016",
            },
            VillageCode {
                name: "汇康锦苑居委会",
                code: "017",
            },
            VillageCode {
                name: "昱星家园居委会",
                code: "018",
            },
            VillageCode {
                name: "金沁苑第一居委会",
                code: "019",
            },
            VillageCode {
                name: "金沁苑第二居委会",
                code: "020",
            },
            VillageCode {
                name: "东茗苑居委会",
                code: "021",
            },
            VillageCode {
                name: "汇诚佳苑居委会",
                code: "022",
            },
            VillageCode {
                name: "和美苑居委会",
                code: "023",
            },
            VillageCode {
                name: "汇善嘉苑居委会",
                code: "024",
            },
            VillageCode {
                name: "汇仁馨苑居委会",
                code: "025",
            },
            VillageCode {
                name: "瑞馨苑居委会",
                code: "026",
            },
            VillageCode {
                name: "金地艺华年居委会",
                code: "027",
            },
            VillageCode {
                name: "瑞祥苑居委会",
                code: "028",
            },
            VillageCode {
                name: "佳和苑居委会",
                code: "029",
            },
            VillageCode {
                name: "康乐苑居委会",
                code: "030",
            },
            VillageCode {
                name: "聚航苑第二居委会",
                code: "031",
            },
            VillageCode {
                name: "汇贤雅苑居委会",
                code: "032",
            },
            VillageCode {
                name: "云麓里居委会",
                code: "033",
            },
            VillageCode {
                name: "昱东家园居委会",
                code: "034",
            },
            VillageCode {
                name: "海桥村村委会",
                code: "035",
            },
            VillageCode {
                name: "鹤鸣村村委会",
                code: "036",
            },
            VillageCode {
                name: "长达村村委会",
                code: "037",
            },
            VillageCode {
                name: "果园村村委会",
                code: "038",
            },
            VillageCode {
                name: "福善村村委会",
                code: "039",
            },
            VillageCode {
                name: "沉香村村委会",
                code: "040",
            },
            VillageCode {
                name: "鹤东村村委会",
                code: "041",
            },
            VillageCode {
                name: "沈庄村村委会",
                code: "042",
            },
            VillageCode {
                name: "王楼村村委会",
                code: "043",
            },
            VillageCode {
                name: "梅园村村委会",
                code: "044",
            },
            VillageCode {
                name: "牌楼村村委会",
                code: "045",
            },
            VillageCode {
                name: "航东村村委会",
                code: "046",
            },
            VillageCode {
                name: "丰桥村村委会",
                code: "047",
            },
        ],
    },
    TownCode {
        name: "祝桥镇",
        code: "030",
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
                name: "东港花苑居委会",
                code: "003",
            },
            VillageCode {
                name: "盐仓居委会",
                code: "004",
            },
            VillageCode {
                name: "朝阳居委会",
                code: "005",
            },
            VillageCode {
                name: "千汇一村社区居委会",
                code: "006",
            },
            VillageCode {
                name: "千汇二村社区居委会",
                code: "007",
            },
            VillageCode {
                name: "千汇三村社区居委会",
                code: "008",
            },
            VillageCode {
                name: "千汇四村社区居委会",
                code: "009",
            },
            VillageCode {
                name: "祝和北苑居委会",
                code: "010",
            },
            VillageCode {
                name: "江镇居委会",
                code: "011",
            },
            VillageCode {
                name: "思凡一居委会",
                code: "012",
            },
            VillageCode {
                name: "思凡二居委会",
                code: "013",
            },
            VillageCode {
                name: "思凡三居委会",
                code: "014",
            },
            VillageCode {
                name: "施镇居委会",
                code: "015",
            },
            VillageCode {
                name: "晨阳路第一居委会",
                code: "016",
            },
            VillageCode {
                name: "百欣苑居委会",
                code: "017",
            },
            VillageCode {
                name: "天环苑居委会",
                code: "018",
            },
            VillageCode {
                name: "祝安苑居委会",
                code: "019",
            },
            VillageCode {
                name: "东都居委会",
                code: "020",
            },
            VillageCode {
                name: "海霞第一居委会",
                code: "021",
            },
            VillageCode {
                name: "航城第一居委会",
                code: "022",
            },
            VillageCode {
                name: "航城第二居委会",
                code: "023",
            },
            VillageCode {
                name: "东港第一居委会",
                code: "024",
            },
            VillageCode {
                name: "东港第二居委会",
                code: "025",
            },
            VillageCode {
                name: "邓镇第一居委会",
                code: "026",
            },
            VillageCode {
                name: "邓镇第二居委会",
                code: "027",
            },
            VillageCode {
                name: "海霞第二居委会",
                code: "028",
            },
            VillageCode {
                name: "秋亭居委会",
                code: "029",
            },
            VillageCode {
                name: "滨一居委会",
                code: "030",
            },
            VillageCode {
                name: "祝康苑居委会",
                code: "031",
            },
            VillageCode {
                name: "江晖苑居委会",
                code: "032",
            },
            VillageCode {
                name: "同悦居委会",
                code: "033",
            },
            VillageCode {
                name: "卫亭居委会",
                code: "034",
            },
            VillageCode {
                name: "祝祥苑居委会",
                code: "035",
            },
            VillageCode {
                name: "江凌苑居委会",
                code: "036",
            },
            VillageCode {
                name: "东方鸿璟居委会",
                code: "037",
            },
            VillageCode {
                name: "祝和南苑居委会",
                code: "038",
            },
            VillageCode {
                name: "启航居委会",
                code: "039",
            },
            VillageCode {
                name: "凌港城居委会",
                code: "040",
            },
            VillageCode {
                name: "祝城居委会",
                code: "041",
            },
            VillageCode {
                name: "晨阳路第二居委会",
                code: "042",
            },
            VillageCode {
                name: "江通苑居委会",
                code: "043",
            },
            VillageCode {
                name: "祝顺苑居委会",
                code: "044",
            },
            VillageCode {
                name: "亭中村村委会",
                code: "045",
            },
            VillageCode {
                name: "三八村村委会",
                code: "046",
            },
            VillageCode {
                name: "金星村村委会",
                code: "047",
            },
            VillageCode {
                name: "祝东村村委会",
                code: "048",
            },
            VillageCode {
                name: "卫民村村委会",
                code: "049",
            },
            VillageCode {
                name: "星光村村委会",
                code: "050",
            },
            VillageCode {
                name: "明星村村委会",
                code: "051",
            },
            VillageCode {
                name: "祝西村村委会",
                code: "052",
            },
            VillageCode {
                name: "红三村村委会",
                code: "053",
            },
            VillageCode {
                name: "新东村村委会",
                code: "054",
            },
            VillageCode {
                name: "新如村村委会",
                code: "055",
            },
            VillageCode {
                name: "薛洪村村委会",
                code: "056",
            },
            VillageCode {
                name: "义泓村村委会",
                code: "057",
            },
            VillageCode {
                name: "先进村村委会",
                code: "058",
            },
            VillageCode {
                name: "星火村村委会",
                code: "059",
            },
            VillageCode {
                name: "红星村村委会",
                code: "060",
            },
            VillageCode {
                name: "高永村村委会",
                code: "061",
            },
            VillageCode {
                name: "果园村村委会",
                code: "062",
            },
            VillageCode {
                name: "立新村村委会",
                code: "063",
            },
            VillageCode {
                name: "东滨村村委会",
                code: "064",
            },
            VillageCode {
                name: "营前村村委会",
                code: "065",
            },
            VillageCode {
                name: "新营村村委会",
                code: "066",
            },
            VillageCode {
                name: "森林村村委会",
                code: "067",
            },
            VillageCode {
                name: "陈胡村村委会",
                code: "068",
            },
            VillageCode {
                name: "小圩村村委会",
                code: "069",
            },
            VillageCode {
                name: "新和村村委会",
                code: "070",
            },
            VillageCode {
                name: "大沟村村委会",
                code: "071",
            },
            VillageCode {
                name: "卫东村村委会",
                code: "072",
            },
            VillageCode {
                name: "亭东村村委会",
                code: "073",
            },
            VillageCode {
                name: "新生村村委会",
                code: "074",
            },
            VillageCode {
                name: "共和村村委会",
                code: "075",
            },
            VillageCode {
                name: "军民村村委会",
                code: "076",
            },
            VillageCode {
                name: "红旗村村委会",
                code: "077",
            },
            VillageCode {
                name: "东立新村村委会",
                code: "078",
            },
            VillageCode {
                name: "道新村村委会",
                code: "079",
            },
            VillageCode {
                name: "中圩村村委会",
                code: "080",
            },
            VillageCode {
                name: "邓一村村委会",
                code: "081",
            },
            VillageCode {
                name: "邓二村村委会",
                code: "082",
            },
            VillageCode {
                name: "邓三村村委会",
                code: "083",
            },
            VillageCode {
                name: "望三村村委会",
                code: "084",
            },
        ],
    },
    TownCode {
        name: "泥城镇",
        code: "031",
        villages: &[
            VillageCode {
                name: "泥城居委会",
                code: "001",
            },
            VillageCode {
                name: "彭镇居委会",
                code: "002",
            },
            VillageCode {
                name: "云锦苑居委会",
                code: "003",
            },
            VillageCode {
                name: "云帆苑第一居委会",
                code: "004",
            },
            VillageCode {
                name: "云松苑居委会",
                code: "005",
            },
            VillageCode {
                name: "云绣苑第一居委会",
                code: "006",
            },
            VillageCode {
                name: "云翔苑居委会",
                code: "007",
            },
            VillageCode {
                name: "云荷苑居委会",
                code: "008",
            },
            VillageCode {
                name: "云欣苑居委会",
                code: "009",
            },
            VillageCode {
                name: "云霞苑居委会",
                code: "010",
            },
            VillageCode {
                name: "阳光里居委会",
                code: "011",
            },
            VillageCode {
                name: "彭兴苑居委会",
                code: "012",
            },
            VillageCode {
                name: "千祥居委会",
                code: "013",
            },
            VillageCode {
                name: "云帆苑第二居委会",
                code: "014",
            },
            VillageCode {
                name: "云绣苑第二居委会",
                code: "015",
            },
            VillageCode {
                name: "云耀苑居委会",
                code: "016",
            },
            VillageCode {
                name: "云端路第一居委会",
                code: "017",
            },
            VillageCode {
                name: "云端路第二居委会",
                code: "018",
            },
            VillageCode {
                name: "云端路第三居委会",
                code: "019",
            },
            VillageCode {
                name: "人民村村委会",
                code: "020",
            },
            VillageCode {
                name: "横港村村委会",
                code: "021",
            },
            VillageCode {
                name: "公平村村委会",
                code: "022",
            },
            VillageCode {
                name: "龙港村村委会",
                code: "023",
            },
            VillageCode {
                name: "海关村村委会",
                code: "024",
            },
            VillageCode {
                name: "中泐村村委会",
                code: "025",
            },
            VillageCode {
                name: "彭庙村村委会",
                code: "026",
            },
            VillageCode {
                name: "永盛村村委会",
                code: "027",
            },
            VillageCode {
                name: "新泐村村委会",
                code: "028",
            },
            VillageCode {
                name: "马厂村村委会",
                code: "029",
            },
            VillageCode {
                name: "杭园村村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "宣桥镇",
        code: "032",
        villages: &[
            VillageCode {
                name: "南区居委会",
                code: "001",
            },
            VillageCode {
                name: "三灶居委会",
                code: "002",
            },
            VillageCode {
                name: "欣松苑居委会",
                code: "003",
            },
            VillageCode {
                name: "欣兰苑居委会",
                code: "004",
            },
            VillageCode {
                name: "明祥苑居委会",
                code: "005",
            },
            VillageCode {
                name: "枫丹居委会",
                code: "006",
            },
            VillageCode {
                name: "枫庭居委会",
                code: "007",
            },
            VillageCode {
                name: "明和居委会",
                code: "008",
            },
            VillageCode {
                name: "艺泰安邦第一居委会",
                code: "009",
            },
            VillageCode {
                name: "艺泰安邦第二居委会",
                code: "010",
            },
            VillageCode {
                name: "欣秋苑居委会",
                code: "011",
            },
            VillageCode {
                name: "海玥居委会",
                code: "012",
            },
            VillageCode {
                name: "张家桥村村委会",
                code: "013",
            },
            VillageCode {
                name: "新安村村委会",
                code: "014",
            },
            VillageCode {
                name: "中心村村委会",
                code: "015",
            },
            VillageCode {
                name: "陆桥村村委会",
                code: "016",
            },
            VillageCode {
                name: "宣桥村村委会",
                code: "017",
            },
            VillageCode {
                name: "长春村村委会",
                code: "018",
            },
            VillageCode {
                name: "季桥村村委会",
                code: "019",
            },
            VillageCode {
                name: "项埭村村委会",
                code: "020",
            },
            VillageCode {
                name: "光辉村村委会",
                code: "021",
            },
            VillageCode {
                name: "光明村村委会",
                code: "022",
            },
            VillageCode {
                name: "腰路村村委会",
                code: "023",
            },
            VillageCode {
                name: "三灶村村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "书院镇",
        code: "033",
        villages: &[
            VillageCode {
                name: "书院第一居委会",
                code: "001",
            },
            VillageCode {
                name: "新欣居委会",
                code: "002",
            },
            VillageCode {
                name: "东场居委会",
                code: "003",
            },
            VillageCode {
                name: "新舒苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "丽泽社区居委会",
                code: "005",
            },
            VillageCode {
                name: "舒馨居委会",
                code: "006",
            },
            VillageCode {
                name: "东方颐城居委会",
                code: "007",
            },
            VillageCode {
                name: "东场第二居委会",
                code: "008",
            },
            VillageCode {
                name: "新舒苑南苑居委会",
                code: "009",
            },
            VillageCode {
                name: "塘北村村委会",
                code: "010",
            },
            VillageCode {
                name: "洋溢村村委会",
                code: "011",
            },
            VillageCode {
                name: "路南村村委会",
                code: "012",
            },
            VillageCode {
                name: "新北村村委会",
                code: "013",
            },
            VillageCode {
                name: "李雪村村委会",
                code: "014",
            },
            VillageCode {
                name: "中久村村委会",
                code: "015",
            },
            VillageCode {
                name: "外灶村村委会",
                code: "016",
            },
            VillageCode {
                name: "黄华村村委会",
                code: "017",
            },
            VillageCode {
                name: "桃园村村委会",
                code: "018",
            },
            VillageCode {
                name: "余姚村村委会",
                code: "019",
            },
            VillageCode {
                name: "棉场村村委会",
                code: "020",
            },
            VillageCode {
                name: "四灶村村委会",
                code: "021",
            },
            VillageCode {
                name: "洼港村村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "万祥镇",
        code: "034",
        villages: &[
            VillageCode {
                name: "万祥居委会",
                code: "001",
            },
            VillageCode {
                name: "馨苑居委会",
                code: "002",
            },
            VillageCode {
                name: "祥苑居委会",
                code: "003",
            },
            VillageCode {
                name: "兰湾居委会",
                code: "004",
            },
            VillageCode {
                name: "兴隆居委会",
                code: "005",
            },
            VillageCode {
                name: "橘园居委会",
                code: "006",
            },
            VillageCode {
                name: "万兴村村委会",
                code: "007",
            },
            VillageCode {
                name: "万隆村村委会",
                code: "008",
            },
            VillageCode {
                name: "万宏村村委会",
                code: "009",
            },
            VillageCode {
                name: "新振村村委会",
                code: "010",
            },
            VillageCode {
                name: "新建村村委会",
                code: "011",
            },
            VillageCode {
                name: "新路村村委会",
                code: "012",
            },
            VillageCode {
                name: "金路村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "老港镇",
        code: "035",
        villages: &[
            VillageCode {
                name: "老港居委会",
                code: "001",
            },
            VillageCode {
                name: "滨海居委会",
                code: "002",
            },
            VillageCode {
                name: "宏港苑居委会",
                code: "003",
            },
            VillageCode {
                name: "丽港苑居委会",
                code: "004",
            },
            VillageCode {
                name: "牛肚村村委会",
                code: "005",
            },
            VillageCode {
                name: "中港村村委会",
                code: "006",
            },
            VillageCode {
                name: "成日村村委会",
                code: "007",
            },
            VillageCode {
                name: "建港村村委会",
                code: "008",
            },
            VillageCode {
                name: "东河村村委会",
                code: "009",
            },
            VillageCode {
                name: "大河村村委会",
                code: "010",
            },
            VillageCode {
                name: "欣河村村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "南汇新城镇",
        code: "036",
        villages: &[
            VillageCode {
                name: "临港家园居委会",
                code: "001",
            },
            VillageCode {
                name: "宜浩佳园一居委会",
                code: "002",
            },
            VillageCode {
                name: "宜浩佳园二居委会",
                code: "003",
            },
            VillageCode {
                name: "东岸涟城居委会",
                code: "004",
            },
            VillageCode {
                name: "港口居委会",
                code: "005",
            },
            VillageCode {
                name: "果园居委会",
                code: "006",
            },
            VillageCode {
                name: "新芦居委会",
                code: "007",
            },
            VillageCode {
                name: "农场居委会",
                code: "008",
            },
            VillageCode {
                name: "海尚居委会",
                code: "009",
            },
            VillageCode {
                name: "海芦居委会",
                code: "010",
            },
            VillageCode {
                name: "海汇居委会",
                code: "011",
            },
            VillageCode {
                name: "滴水湖馨苑一居委会",
                code: "012",
            },
            VillageCode {
                name: "海滨居委会",
                code: "013",
            },
            VillageCode {
                name: "宜浩欧景一居委会",
                code: "014",
            },
            VillageCode {
                name: "蔚蓝林语居委会",
                code: "015",
            },
            VillageCode {
                name: "金域澜湾居委会",
                code: "016",
            },
            VillageCode {
                name: "滴水湖馨苑二居委会",
                code: "017",
            },
            VillageCode {
                name: "滨河居委会",
                code: "018",
            },
            VillageCode {
                name: "芦茂居委会",
                code: "019",
            },
            VillageCode {
                name: "宜浩欧景二居委会",
                code: "020",
            },
            VillageCode {
                name: "东岸涟城二居委会",
                code: "021",
            },
            VillageCode {
                name: "芦潮居委会",
                code: "022",
            },
            VillageCode {
                name: "芦硕居委会",
                code: "023",
            },
            VillageCode {
                name: "芦云居委会",
                code: "024",
            },
            VillageCode {
                name: "宜浩佳园三居委会",
                code: "025",
            },
            VillageCode {
                name: "宜浩欧景三居委会",
                code: "026",
            },
            VillageCode {
                name: "滴水湖馨苑三居委会",
                code: "027",
            },
            VillageCode {
                name: "方竹馨悦居委会",
                code: "028",
            },
            VillageCode {
                name: "方竹馨雅居委会",
                code: "029",
            },
            VillageCode {
                name: "滴水涟岸居委会",
                code: "030",
            },
            VillageCode {
                name: "海上鹭语居委会",
                code: "031",
            },
            VillageCode {
                name: "云浩东宸居委会",
                code: "032",
            },
            VillageCode {
                name: "汇角村村委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "芦潮港农场",
        code: "037",
        villages: &[VillageCode {
            name: "芦潮港农场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "东海农场",
        code: "038",
        villages: &[VillageCode {
            name: "东海农场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "朝阳农场",
        code: "039",
        villages: &[VillageCode {
            name: "朝阳农场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "中国（上海）自由贸易试验区（保税片区）",
        code: "040",
        villages: &[VillageCode {
            name: "中国（上海）自由贸易试验区（保税片区）虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "金桥经济技术开发区",
        code: "041",
        villages: &[VillageCode {
            name: "金桥经济技术开发区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "张江高科技园区",
        code: "042",
        villages: &[VillageCode {
            name: "张江高科技园区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_SJ_013: [TownCode; 11] = [
    TownCode {
        name: "石化街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "三村居委会",
                code: "001",
            },
            VillageCode {
                name: "四村居委会",
                code: "002",
            },
            VillageCode {
                name: "七村居委会",
                code: "003",
            },
            VillageCode {
                name: "柳城新村居委会",
                code: "004",
            },
            VillageCode {
                name: "九村居委会",
                code: "005",
            },
            VillageCode {
                name: "十村居委会",
                code: "006",
            },
            VillageCode {
                name: "合浦居委会",
                code: "007",
            },
            VillageCode {
                name: "十二村居委会",
                code: "008",
            },
            VillageCode {
                name: "十三村居委会",
                code: "009",
            },
            VillageCode {
                name: "海棠居委会",
                code: "010",
            },
            VillageCode {
                name: "梅州新村居委会",
                code: "011",
            },
            VillageCode {
                name: "临蒙居委会",
                code: "012",
            },
            VillageCode {
                name: "临潮三村居委会",
                code: "013",
            },
            VillageCode {
                name: "滨海一村居委会",
                code: "014",
            },
            VillageCode {
                name: "滨海二村居委会",
                code: "015",
            },
            VillageCode {
                name: "东村居委会",
                code: "016",
            },
            VillageCode {
                name: "东礁新村第一居委会",
                code: "017",
            },
            VillageCode {
                name: "东礁新村第二居委会",
                code: "018",
            },
            VillageCode {
                name: "东泉新村居委会",
                code: "019",
            },
            VillageCode {
                name: "辰凯居委会",
                code: "020",
            },
            VillageCode {
                name: "山鑫阳光城居委会",
                code: "021",
            },
            VillageCode {
                name: "山龙新村居委会",
                code: "022",
            },
            VillageCode {
                name: "桥园新村居委会",
                code: "023",
            },
            VillageCode {
                name: "卫清新村居委会",
                code: "024",
            },
            VillageCode {
                name: "紫卫社区居委会",
                code: "025",
            },
            VillageCode {
                name: "合生居委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "朱泾镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "东林居委会",
                code: "001",
            },
            VillageCode {
                name: "罗星居委会",
                code: "002",
            },
            VillageCode {
                name: "新汇居委会",
                code: "003",
            },
            VillageCode {
                name: "广福居委会",
                code: "004",
            },
            VillageCode {
                name: "西林居委会",
                code: "005",
            },
            VillageCode {
                name: "南圩居委会",
                code: "006",
            },
            VillageCode {
                name: "临源居委会",
                code: "007",
            },
            VillageCode {
                name: "临东居委会",
                code: "008",
            },
            VillageCode {
                name: "北圩居委会",
                code: "009",
            },
            VillageCode {
                name: "钟楼居委会",
                code: "010",
            },
            VillageCode {
                name: "凤翔居委会",
                code: "011",
            },
            VillageCode {
                name: "金龙居委会",
                code: "012",
            },
            VillageCode {
                name: "浦银居委会",
                code: "013",
            },
            VillageCode {
                name: "塘园居委会",
                code: "014",
            },
            VillageCode {
                name: "金汇居委会",
                code: "015",
            },
            VillageCode {
                name: "金来居委会",
                code: "016",
            },
            VillageCode {
                name: "红菱居委会",
                code: "017",
            },
            VillageCode {
                name: "众安居委会",
                code: "018",
            },
            VillageCode {
                name: "新天鸿居委会",
                code: "019",
            },
            VillageCode {
                name: "民主村村委会",
                code: "020",
            },
            VillageCode {
                name: "大茫村村委会",
                code: "021",
            },
            VillageCode {
                name: "待泾村村委会",
                code: "022",
            },
            VillageCode {
                name: "秀州村村委会",
                code: "023",
            },
            VillageCode {
                name: "新泾村村委会",
                code: "024",
            },
            VillageCode {
                name: "万联村村委会",
                code: "025",
            },
            VillageCode {
                name: "长浜村村委会",
                code: "026",
            },
            VillageCode {
                name: "五龙村村委会",
                code: "027",
            },
            VillageCode {
                name: "慧农村村委会",
                code: "028",
            },
            VillageCode {
                name: "牡丹村村委会",
                code: "029",
            },
            VillageCode {
                name: "温河村村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "枫泾镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "和平居委会",
                code: "001",
            },
            VillageCode {
                name: "友好居委会",
                code: "002",
            },
            VillageCode {
                name: "白牛居委会",
                code: "003",
            },
            VillageCode {
                name: "枫阳居委会",
                code: "004",
            },
            VillageCode {
                name: "兴塔居委会",
                code: "005",
            },
            VillageCode {
                name: "枫香居委会",
                code: "006",
            },
            VillageCode {
                name: "新苑居委会",
                code: "007",
            },
            VillageCode {
                name: "枫岸居委会",
                code: "008",
            },
            VillageCode {
                name: "桃源居委会",
                code: "009",
            },
            VillageCode {
                name: "枫兰居委会",
                code: "010",
            },
            VillageCode {
                name: "枫美居委会",
                code: "011",
            },
            VillageCode {
                name: "新元村村委会",
                code: "012",
            },
            VillageCode {
                name: "中洪村村委会",
                code: "013",
            },
            VillageCode {
                name: "俞汇村村委会",
                code: "014",
            },
            VillageCode {
                name: "新春村村委会",
                code: "015",
            },
            VillageCode {
                name: "农兴村村委会",
                code: "016",
            },
            VillageCode {
                name: "钱明村村委会",
                code: "017",
            },
            VillageCode {
                name: "长征村村委会",
                code: "018",
            },
            VillageCode {
                name: "盛新村村委会",
                code: "019",
            },
            VillageCode {
                name: "新华村村委会",
                code: "020",
            },
            VillageCode {
                name: "新义村村委会",
                code: "021",
            },
            VillageCode {
                name: "新新村村委会",
                code: "022",
            },
            VillageCode {
                name: "菖梧村村委会",
                code: "023",
            },
            VillageCode {
                name: "团新村村委会",
                code: "024",
            },
            VillageCode {
                name: "双庙村村委会",
                code: "025",
            },
            VillageCode {
                name: "兴塔村村委会",
                code: "026",
            },
            VillageCode {
                name: "五一村村委会",
                code: "027",
            },
            VillageCode {
                name: "贵泾村村委会",
                code: "028",
            },
            VillageCode {
                name: "新黎村村委会",
                code: "029",
            },
            VillageCode {
                name: "下坊村村委会",
                code: "030",
            },
            VillageCode {
                name: "泖桥村村委会",
                code: "031",
            },
            VillageCode {
                name: "韩坞村村委会",
                code: "032",
            },
            VillageCode {
                name: "卫星村村委会",
                code: "033",
            },
            VillageCode {
                name: "五星村村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "张堰镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "东风居委会",
                code: "001",
            },
            VillageCode {
                name: "解放居委会",
                code: "002",
            },
            VillageCode {
                name: "富民居委会",
                code: "003",
            },
            VillageCode {
                name: "留溪居委会",
                code: "004",
            },
            VillageCode {
                name: "旧港村村委会",
                code: "005",
            },
            VillageCode {
                name: "桑园村村委会",
                code: "006",
            },
            VillageCode {
                name: "甪里村村委会",
                code: "007",
            },
            VillageCode {
                name: "鲁堰村村委会",
                code: "008",
            },
            VillageCode {
                name: "百家村村委会",
                code: "009",
            },
            VillageCode {
                name: "秦望村村委会",
                code: "010",
            },
            VillageCode {
                name: "秦山村村委会",
                code: "011",
            },
            VillageCode {
                name: "秦阳村村委会",
                code: "012",
            },
            VillageCode {
                name: "建农村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "亭林镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "中山居委会",
                code: "001",
            },
            VillageCode {
                name: "新建居委会",
                code: "002",
            },
            VillageCode {
                name: "复兴居委会",
                code: "003",
            },
            VillageCode {
                name: "寺平居委会",
                code: "004",
            },
            VillageCode {
                name: "松隐居委会",
                code: "005",
            },
            VillageCode {
                name: "寺北居委会",
                code: "006",
            },
            VillageCode {
                name: "隆亭居委会",
                code: "007",
            },
            VillageCode {
                name: "华城第一居委会",
                code: "008",
            },
            VillageCode {
                name: "华城第二居委会",
                code: "009",
            },
            VillageCode {
                name: "龙泉村村委会",
                code: "010",
            },
            VillageCode {
                name: "亭东村村委会",
                code: "011",
            },
            VillageCode {
                name: "东新村村委会",
                code: "012",
            },
            VillageCode {
                name: "新巷村村委会",
                code: "013",
            },
            VillageCode {
                name: "油车村村委会",
                code: "014",
            },
            VillageCode {
                name: "亭西村村委会",
                code: "015",
            },
            VillageCode {
                name: "金门村村委会",
                code: "016",
            },
            VillageCode {
                name: "亭北村村委会",
                code: "017",
            },
            VillageCode {
                name: "红阳村村委会",
                code: "018",
            },
            VillageCode {
                name: "浩光村村委会",
                code: "019",
            },
            VillageCode {
                name: "南星村村委会",
                code: "020",
            },
            VillageCode {
                name: "周栅村村委会",
                code: "021",
            },
            VillageCode {
                name: "驳岸村村委会",
                code: "022",
            },
            VillageCode {
                name: "后岗村村委会",
                code: "023",
            },
            VillageCode {
                name: "金明村村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "吕巷镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "吕巷居委会",
                code: "001",
            },
            VillageCode {
                name: "干巷居委会",
                code: "002",
            },
            VillageCode {
                name: "荡田村村委会",
                code: "003",
            },
            VillageCode {
                name: "夹漏村村委会",
                code: "004",
            },
            VillageCode {
                name: "姚家村村委会",
                code: "005",
            },
            VillageCode {
                name: "马新村村委会",
                code: "006",
            },
            VillageCode {
                name: "太平村村委会",
                code: "007",
            },
            VillageCode {
                name: "蔷薇村村委会",
                code: "008",
            },
            VillageCode {
                name: "龙跃村村委会",
                code: "009",
            },
            VillageCode {
                name: "和平村村委会",
                code: "010",
            },
            VillageCode {
                name: "白漾村村委会",
                code: "011",
            },
            VillageCode {
                name: "颜圩村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "廊下镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "廊下居委会",
                code: "001",
            },
            VillageCode {
                name: "景展居委会",
                code: "002",
            },
            VillageCode {
                name: "万春村村委会",
                code: "003",
            },
            VillageCode {
                name: "中联村村委会",
                code: "004",
            },
            VillageCode {
                name: "勇敢村村委会",
                code: "005",
            },
            VillageCode {
                name: "中丰村村委会",
                code: "006",
            },
            VillageCode {
                name: "景阳村村委会",
                code: "007",
            },
            VillageCode {
                name: "中民村村委会",
                code: "008",
            },
            VillageCode {
                name: "山塘村村委会",
                code: "009",
            },
            VillageCode {
                name: "中华村村委会",
                code: "010",
            },
            VillageCode {
                name: "南陆村村委会",
                code: "011",
            },
            VillageCode {
                name: "南塘村村委会",
                code: "012",
            },
            VillageCode {
                name: "光明村村委会",
                code: "013",
            },
            VillageCode {
                name: "友好村村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "金山卫镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "西门居委会",
                code: "001",
            },
            VillageCode {
                name: "钱圩居委会",
                code: "002",
            },
            VillageCode {
                name: "东门居委会",
                code: "003",
            },
            VillageCode {
                name: "南门居委会",
                code: "004",
            },
            VillageCode {
                name: "北门居委会",
                code: "005",
            },
            VillageCode {
                name: "金康居委会",
                code: "006",
            },
            VillageCode {
                name: "海帆居委会",
                code: "007",
            },
            VillageCode {
                name: "东平居委会",
                code: "008",
            },
            VillageCode {
                name: "明卫居委会",
                code: "009",
            },
            VillageCode {
                name: "八一村村委会",
                code: "010",
            },
            VillageCode {
                name: "八二村村委会",
                code: "011",
            },
            VillageCode {
                name: "永久村村委会",
                code: "012",
            },
            VillageCode {
                name: "永联村村委会",
                code: "013",
            },
            VillageCode {
                name: "横浦村村委会",
                code: "014",
            },
            VillageCode {
                name: "卫通村村委会",
                code: "015",
            },
            VillageCode {
                name: "农建村村委会",
                code: "016",
            },
            VillageCode {
                name: "金卫村村委会",
                code: "017",
            },
            VillageCode {
                name: "卫城村村委会",
                code: "018",
            },
            VillageCode {
                name: "塔港村村委会",
                code: "019",
            },
            VillageCode {
                name: "横召村村委会",
                code: "020",
            },
            VillageCode {
                name: "星火村村委会",
                code: "021",
            },
            VillageCode {
                name: "八字村村委会",
                code: "022",
            },
            VillageCode {
                name: "张桥村村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "漕泾镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "花园居委会",
                code: "001",
            },
            VillageCode {
                name: "建龙居委会",
                code: "002",
            },
            VillageCode {
                name: "绿地居委会",
                code: "003",
            },
            VillageCode {
                name: "增丰村村委会",
                code: "004",
            },
            VillageCode {
                name: "东海村村委会",
                code: "005",
            },
            VillageCode {
                name: "营房村村委会",
                code: "006",
            },
            VillageCode {
                name: "沙积村村委会",
                code: "007",
            },
            VillageCode {
                name: "阮巷村村委会",
                code: "008",
            },
            VillageCode {
                name: "蒋庄村村委会",
                code: "009",
            },
            VillageCode {
                name: "护塘村村委会",
                code: "010",
            },
            VillageCode {
                name: "金光村村委会",
                code: "011",
            },
            VillageCode {
                name: "水库村村委会",
                code: "012",
            },
            VillageCode {
                name: "海渔村村委会",
                code: "013",
            },
            VillageCode {
                name: "海涯村村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "山阳镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "山新居委会",
                code: "001",
            },
            VillageCode {
                name: "金海岸居委会",
                code: "002",
            },
            VillageCode {
                name: "戚家墩居委会",
                code: "003",
            },
            VillageCode {
                name: "海趣居委会",
                code: "004",
            },
            VillageCode {
                name: "三岛龙洲居委会",
                code: "005",
            },
            VillageCode {
                name: "联江居委会",
                code: "006",
            },
            VillageCode {
                name: "金世纪居委会",
                code: "007",
            },
            VillageCode {
                name: "金豪居委会",
                code: "008",
            },
            VillageCode {
                name: "海云居委会",
                code: "009",
            },
            VillageCode {
                name: "康建居委会",
                code: "010",
            },
            VillageCode {
                name: "蓝色收获居委会",
                code: "011",
            },
            VillageCode {
                name: "海尚居委会",
                code: "012",
            },
            VillageCode {
                name: "海欣居委会",
                code: "013",
            },
            VillageCode {
                name: "红树林居委会",
                code: "014",
            },
            VillageCode {
                name: "金天地居委会",
                code: "015",
            },
            VillageCode {
                name: "金悦居委会",
                code: "016",
            },
            VillageCode {
                name: "金韵居委会",
                code: "017",
            },
            VillageCode {
                name: "龙泽园居委会",
                code: "018",
            },
            VillageCode {
                name: "金湾居委会",
                code: "019",
            },
            VillageCode {
                name: "万盛居委会",
                code: "020",
            },
            VillageCode {
                name: "旭辉居委会",
                code: "021",
            },
            VillageCode {
                name: "万皓居委会",
                code: "022",
            },
            VillageCode {
                name: "海悦居委会",
                code: "023",
            },
            VillageCode {
                name: "香颂湾居委会",
                code: "024",
            },
            VillageCode {
                name: "海辰居委会",
                code: "025",
            },
            VillageCode {
                name: "同鑫居委会",
                code: "026",
            },
            VillageCode {
                name: "同凯居委会",
                code: "027",
            },
            VillageCode {
                name: "向阳村村委会",
                code: "028",
            },
            VillageCode {
                name: "九龙村村委会",
                code: "029",
            },
            VillageCode {
                name: "杨家村村委会",
                code: "030",
            },
            VillageCode {
                name: "东方村村委会",
                code: "031",
            },
            VillageCode {
                name: "中兴村村委会",
                code: "032",
            },
            VillageCode {
                name: "长兴村村委会",
                code: "033",
            },
            VillageCode {
                name: "华新村村委会",
                code: "034",
            },
            VillageCode {
                name: "渔业村村委会",
                code: "035",
            },
            VillageCode {
                name: "新江村村委会",
                code: "036",
            },
            VillageCode {
                name: "卫东村村委会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "上海湾区高新技术产业开发区",
        code: "011",
        villages: &[
            VillageCode {
                name: "朱行居委会",
                code: "001",
            },
            VillageCode {
                name: "恒信居委会",
                code: "002",
            },
            VillageCode {
                name: "桥湾居委会",
                code: "003",
            },
            VillageCode {
                name: "恒顺居委会",
                code: "004",
            },
            VillageCode {
                name: "红叶居委会",
                code: "005",
            },
            VillageCode {
                name: "恒康居委会",
                code: "006",
            },
            VillageCode {
                name: "金水湖居委会",
                code: "007",
            },
            VillageCode {
                name: "华云居委会",
                code: "008",
            },
            VillageCode {
                name: "欢兴村村委会",
                code: "009",
            },
            VillageCode {
                name: "红光村村委会",
                code: "010",
            },
            VillageCode {
                name: "立新村村委会",
                code: "011",
            },
            VillageCode {
                name: "运河村村委会",
                code: "012",
            },
            VillageCode {
                name: "高楼村村委会",
                code: "013",
            },
            VillageCode {
                name: "胥浦村村委会",
                code: "014",
            },
            VillageCode {
                name: "合兴村村委会",
                code: "015",
            },
            VillageCode {
                name: "保卫村村委会",
                code: "016",
            },
            VillageCode {
                name: "新街村村委会",
                code: "017",
            },
        ],
    },
];

static TOWNS_SJ_014: [TownCode; 20] = [
    TownCode {
        name: "岳阳街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "龙潭社区居委会",
                code: "001",
            },
            VillageCode {
                name: "醉白池社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金沙滩社区居委会",
                code: "003",
            },
            VillageCode {
                name: "太平社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西新桥社区居委会",
                code: "005",
            },
            VillageCode {
                name: "西林塔社区居委会",
                code: "006",
            },
            VillageCode {
                name: "佛字桥社区居委会",
                code: "007",
            },
            VillageCode {
                name: "马路桥社区居委会",
                code: "008",
            },
            VillageCode {
                name: "长桥社区居委会",
                code: "009",
            },
            VillageCode {
                name: "景德路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "通波社区居委会",
                code: "011",
            },
            VillageCode {
                name: "菜花泾社区居委会",
                code: "012",
            },
            VillageCode {
                name: "蒋泾社区居委会",
                code: "013",
            },
            VillageCode {
                name: "荣乐社区居委会",
                code: "014",
            },
            VillageCode {
                name: "人乐社区居委会",
                code: "015",
            },
            VillageCode {
                name: "凤凰新村社区居委会",
                code: "016",
            },
            VillageCode {
                name: "九峰社区居委会",
                code: "017",
            },
            VillageCode {
                name: "黑鱼弄社区居委会",
                code: "018",
            },
            VillageCode {
                name: "高乐社区居委会",
                code: "019",
            },
            VillageCode {
                name: "人民桥社区居委会",
                code: "020",
            },
            VillageCode {
                name: "龙兴社区居委会",
                code: "021",
            },
            VillageCode {
                name: "民乐社区居委会",
                code: "022",
            },
            VillageCode {
                name: "白洋社区居委会",
                code: "023",
            },
            VillageCode {
                name: "方舟园社区居委会",
                code: "024",
            },
            VillageCode {
                name: "松乐苑社区居委会",
                code: "025",
            },
            VillageCode {
                name: "戴家浜社区居委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "永丰街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "秀南社区居委会",
                code: "001",
            },
            VillageCode {
                name: "仓城社区居委会",
                code: "002",
            },
            VillageCode {
                name: "仓桥社区居委会",
                code: "003",
            },
            VillageCode {
                name: "周星社区居委会",
                code: "004",
            },
            VillageCode {
                name: "薛家社区居委会",
                code: "005",
            },
            VillageCode {
                name: "仓吉社区居委会",
                code: "006",
            },
            VillageCode {
                name: "盐仓社区居委会",
                code: "007",
            },
            VillageCode {
                name: "玉乐社区居委会",
                code: "008",
            },
            VillageCode {
                name: "三星苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "花园浜社区居委会",
                code: "010",
            },
            VillageCode {
                name: "玉荣社区居委会",
                code: "011",
            },
            VillageCode {
                name: "银杏苑社区居委会",
                code: "012",
            },
            VillageCode {
                name: "三辰苑社区居委会",
                code: "013",
            },
            VillageCode {
                name: "五丰苑社区居委会",
                code: "014",
            },
            VillageCode {
                name: "兴日家园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "华亭荣园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "百合苑社区居委会",
                code: "017",
            },
            VillageCode {
                name: "新理想社区居委会",
                code: "018",
            },
            VillageCode {
                name: "金色华亭社区居委会",
                code: "019",
            },
            VillageCode {
                name: "金地艺境社区居委会",
                code: "020",
            },
            VillageCode {
                name: "玉树社区居委会",
                code: "021",
            },
            VillageCode {
                name: "海尚名都社区居委会",
                code: "022",
            },
            VillageCode {
                name: "谷水佳苑社区居委会",
                code: "023",
            },
            VillageCode {
                name: "辰丰苑社区居委会",
                code: "024",
            },
            VillageCode {
                name: "玉阳苑社区居委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "方松街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "放生池社区居委会",
                code: "001",
            },
            VillageCode {
                name: "江虹社区居委会",
                code: "002",
            },
            VillageCode {
                name: "天乐社区居委会",
                code: "003",
            },
            VillageCode {
                name: "江中社区居委会",
                code: "004",
            },
            VillageCode {
                name: "绿庭苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "祥和社区居委会",
                code: "006",
            },
            VillageCode {
                name: "兰桥社区居委会",
                code: "007",
            },
            VillageCode {
                name: "鼎信社区居委会",
                code: "008",
            },
            VillageCode {
                name: "开元小区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "世纪小区社区居委会",
                code: "010",
            },
            VillageCode {
                name: "檀香小区社区居委会",
                code: "011",
            },
            VillageCode {
                name: "建设社区居委会",
                code: "012",
            },
            VillageCode {
                name: "湖畔天地社区居委会",
                code: "013",
            },
            VillageCode {
                name: "紫东新苑社区居委会",
                code: "014",
            },
            VillageCode {
                name: "名庭花苑社区居委会",
                code: "015",
            },
            VillageCode {
                name: "海德名园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "久阳文华社区居委会",
                code: "017",
            },
            VillageCode {
                name: "润峰苑社区居委会",
                code: "018",
            },
            VillageCode {
                name: "昌鑫花园社区居委会",
                code: "019",
            },
            VillageCode {
                name: "泰唔士小镇社区居委会",
                code: "020",
            },
            VillageCode {
                name: "原野社区居委会",
                code: "021",
            },
            VillageCode {
                name: "安琪花苑社区居委会",
                code: "022",
            },
            VillageCode {
                name: "华亭社区居委会",
                code: "023",
            },
            VillageCode {
                name: "珠江新城社区居委会",
                code: "024",
            },
            VillageCode {
                name: "月亮河社区居委会",
                code: "025",
            },
            VillageCode {
                name: "上泰绅苑社区居委会",
                code: "026",
            },
            VillageCode {
                name: "锦桂苑社区居委会",
                code: "027",
            },
            VillageCode {
                name: "英郡别苑社区居委会",
                code: "028",
            },
            VillageCode {
                name: "东鼎社区居委会",
                code: "029",
            },
            VillageCode {
                name: "阳光翠庭社区居委会",
                code: "030",
            },
            VillageCode {
                name: "浪琴水岸社区居委会",
                code: "031",
            },
            VillageCode {
                name: "德邑小城社区居委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "中山街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "茸梅社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北门社区居委会",
                code: "002",
            },
            VillageCode {
                name: "白云社区居委会",
                code: "003",
            },
            VillageCode {
                name: "平桥社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东外社区居委会",
                code: "005",
            },
            VillageCode {
                name: "方东社区居委会",
                code: "006",
            },
            VillageCode {
                name: "方西社区居委会",
                code: "007",
            },
            VillageCode {
                name: "南门社区居委会",
                code: "008",
            },
            VillageCode {
                name: "五龙社区居委会",
                code: "009",
            },
            VillageCode {
                name: "花桥社区居委会",
                code: "010",
            },
            VillageCode {
                name: "中山苑社区居委会",
                code: "011",
            },
            VillageCode {
                name: "蓝天一村社区居委会",
                code: "012",
            },
            VillageCode {
                name: "蓝天二村社区居委会",
                code: "013",
            },
            VillageCode {
                name: "蓝天四村社区居委会",
                code: "014",
            },
            VillageCode {
                name: "蓝天五村社区居委会",
                code: "015",
            },
            VillageCode {
                name: "莱顿社区居委会",
                code: "016",
            },
            VillageCode {
                name: "黄渡浜社区居委会",
                code: "017",
            },
            VillageCode {
                name: "同济雅筑社区居委会",
                code: "018",
            },
            VillageCode {
                name: "淡家浜社区居委会",
                code: "019",
            },
            VillageCode {
                name: "茸星社区居委会",
                code: "020",
            },
            VillageCode {
                name: "郭家娄社区居委会",
                code: "021",
            },
            VillageCode {
                name: "茸树社区居委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "广富林街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "松云水苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西子湾社区居委会",
                code: "002",
            },
            VillageCode {
                name: "御上海社区居委会",
                code: "003",
            },
            VillageCode {
                name: "星辰园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "三湘四季社区居委会",
                code: "005",
            },
            VillageCode {
                name: "文翔名苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "蔷薇社区居委会",
                code: "007",
            },
            VillageCode {
                name: "丁香社区居委会",
                code: "008",
            },
            VillageCode {
                name: "谷水湾社区居委会",
                code: "009",
            },
            VillageCode {
                name: "上林社区居委会",
                code: "010",
            },
            VillageCode {
                name: "银源社区居委会",
                code: "011",
            },
            VillageCode {
                name: "辰富社区居委会",
                code: "012",
            },
            VillageCode {
                name: "悦都社区居委会",
                code: "013",
            },
            VillageCode {
                name: "仓兴社区居委会",
                code: "014",
            },
            VillageCode {
                name: "银泽社区居委会",
                code: "015",
            },
            VillageCode {
                name: "银河社区居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "九里亭街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "九里亭社区居委会",
                code: "001",
            },
            VillageCode {
                name: "亭汇社区居委会",
                code: "002",
            },
            VillageCode {
                name: "知雅汇社区居委会",
                code: "003",
            },
            VillageCode {
                name: "中大社区居委会",
                code: "004",
            },
            VillageCode {
                name: "亭北社区居委会",
                code: "005",
            },
            VillageCode {
                name: "朗庭社区居委会",
                code: "006",
            },
            VillageCode {
                name: "涞寅社区居委会",
                code: "007",
            },
            VillageCode {
                name: "亭谊社区居委会",
                code: "008",
            },
            VillageCode {
                name: "绿庭尚城社区居委会",
                code: "009",
            },
            VillageCode {
                name: "九城湖滨社区居委会",
                code: "010",
            },
            VillageCode {
                name: "贝尚湾社区居委会",
                code: "011",
            },
            VillageCode {
                name: "百丽苑社区居委会",
                code: "012",
            },
            VillageCode {
                name: "五洲社区居委会",
                code: "013",
            },
            VillageCode {
                name: "杜巷社区居委会",
                code: "014",
            },
            VillageCode {
                name: "奥园社区居委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "泗泾镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "张泾社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中西社区居委会",
                code: "002",
            },
            VillageCode {
                name: "江川社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新南社区居委会",
                code: "004",
            },
            VillageCode {
                name: "润和苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "润江社区居委会",
                code: "006",
            },
            VillageCode {
                name: "丽水社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新凯一村社区居委会",
                code: "008",
            },
            VillageCode {
                name: "西南社区居委会",
                code: "009",
            },
            VillageCode {
                name: "景港社区居委会",
                code: "010",
            },
            VillageCode {
                name: "景园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "新凯二村社区居委会",
                code: "012",
            },
            VillageCode {
                name: "张施社区居委会",
                code: "013",
            },
            VillageCode {
                name: "叶星社区居委会",
                code: "014",
            },
            VillageCode {
                name: "赵非泾社区居委会",
                code: "015",
            },
            VillageCode {
                name: "打铁桥社区居委会",
                code: "016",
            },
            VillageCode {
                name: "横港社区居委会",
                code: "017",
            },
            VillageCode {
                name: "祥泽社区居委会",
                code: "018",
            },
            VillageCode {
                name: "青松社区居委会",
                code: "019",
            },
            VillageCode {
                name: "古楼社区居委会",
                code: "020",
            },
            VillageCode {
                name: "新凯三村社区居委会",
                code: "021",
            },
            VillageCode {
                name: "新凯四村社区居委会",
                code: "022",
            },
            VillageCode {
                name: "新凯五村社区居委会",
                code: "023",
            },
            VillageCode {
                name: "同润社区居委会",
                code: "024",
            },
            VillageCode {
                name: "新凯六村社区居委会",
                code: "025",
            },
            VillageCode {
                name: "向阳桥社区居委会",
                code: "026",
            },
            VillageCode {
                name: "新凯七村社区居委会",
                code: "027",
            },
            VillageCode {
                name: "新凯八村社区居委会",
                code: "028",
            },
            VillageCode {
                name: "韵意一村社区居委会",
                code: "029",
            },
            VillageCode {
                name: "韵意二村社区居委会",
                code: "030",
            },
            VillageCode {
                name: "韵意三村社区居委会",
                code: "031",
            },
            VillageCode {
                name: "韵意四村社区居委会",
                code: "032",
            },
            VillageCode {
                name: "韵意五村社区居委会",
                code: "033",
            },
            VillageCode {
                name: "韵意六村社区居委会",
                code: "034",
            },
            VillageCode {
                name: "韵意七村社区居委会",
                code: "035",
            },
            VillageCode {
                name: "金地一村社区居委会",
                code: "036",
            },
            VillageCode {
                name: "韵意八村社区居委会",
                code: "037",
            },
            VillageCode {
                name: "金地二村社区居委会",
                code: "038",
            },
            VillageCode {
                name: "金地三村社区居委会",
                code: "039",
            },
            VillageCode {
                name: "晶湖社区居委会",
                code: "040",
            },
            VillageCode {
                name: "七间邨社区居委会",
                code: "041",
            },
            VillageCode {
                name: "恒泽社区居委会",
                code: "042",
            },
        ],
    },
    TownCode {
        name: "佘山镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "陈坊社区居委会",
                code: "001",
            },
            VillageCode {
                name: "天马社区居委会",
                code: "002",
            },
            VillageCode {
                name: "江秋新苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "月湖社区居委会",
                code: "004",
            },
            VillageCode {
                name: "翠鑫苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "佘山家园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "秋潭苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "陈坊新苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "莘泽社区居委会",
                code: "009",
            },
            VillageCode {
                name: "康桂社区居委会",
                code: "010",
            },
            VillageCode {
                name: "茗雅社区居委会",
                code: "011",
            },
            VillageCode {
                name: "结香社区居委会",
                code: "012",
            },
            VillageCode {
                name: "高家社区居委会",
                code: "013",
            },
            VillageCode {
                name: "木槿社区居委会",
                code: "014",
            },
            VillageCode {
                name: "佘山名苑社区居委会",
                code: "015",
            },
            VillageCode {
                name: "云庭社区居委会",
                code: "016",
            },
            VillageCode {
                name: "西霞社区居委会",
                code: "017",
            },
            VillageCode {
                name: "江秋村村委会",
                code: "018",
            },
            VillageCode {
                name: "张朴村村委会",
                code: "019",
            },
            VillageCode {
                name: "北干山村村委会",
                code: "020",
            },
            VillageCode {
                name: "陈坊村村委会",
                code: "021",
            },
            VillageCode {
                name: "陆其浜村村委会",
                code: "022",
            },
            VillageCode {
                name: "卫家埭村村委会",
                code: "023",
            },
            VillageCode {
                name: "新镇村村委会",
                code: "024",
            },
            VillageCode {
                name: "横山村村委会",
                code: "025",
            },
            VillageCode {
                name: "刘家山村村委会",
                code: "026",
            },
            VillageCode {
                name: "新宅村村委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "车墩镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "虬长路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "华阳社区居委会",
                code: "002",
            },
            VillageCode {
                name: "祥东社区居委会",
                code: "003",
            },
            VillageCode {
                name: "影佳社区居委会",
                code: "004",
            },
            VillageCode {
                name: "同乐社区居委会",
                code: "005",
            },
            VillageCode {
                name: "华源社区居委会",
                code: "006",
            },
            VillageCode {
                name: "影振社区居委会",
                code: "007",
            },
            VillageCode {
                name: "华欣社区居委会",
                code: "008",
            },
            VillageCode {
                name: "隆泽社区居委会",
                code: "009",
            },
            VillageCode {
                name: "善德苑社区居委会",
                code: "010",
            },
            VillageCode {
                name: "新晟社区居委会",
                code: "011",
            },
            VillageCode {
                name: "欣哲社区居委会",
                code: "012",
            },
            VillageCode {
                name: "云珠社区居委会",
                code: "013",
            },
            VillageCode {
                name: "高桥村村委会",
                code: "014",
            },
            VillageCode {
                name: "新余村村委会",
                code: "015",
            },
            VillageCode {
                name: "联建村村委会",
                code: "016",
            },
            VillageCode {
                name: "得胜村村委会",
                code: "017",
            },
            VillageCode {
                name: "联庄村村委会",
                code: "018",
            },
            VillageCode {
                name: "汇桥村村委会",
                code: "019",
            },
            VillageCode {
                name: "联民村村委会",
                code: "020",
            },
            VillageCode {
                name: "南门村村委会",
                code: "021",
            },
            VillageCode {
                name: "米市渡村村委会",
                code: "022",
            },
            VillageCode {
                name: "永福村村委会",
                code: "023",
            },
            VillageCode {
                name: "长溇村村委会",
                code: "024",
            },
            VillageCode {
                name: "洋泾村村委会",
                code: "025",
            },
            VillageCode {
                name: "香山村村委会",
                code: "026",
            },
            VillageCode {
                name: "打铁桥村村委会",
                code: "027",
            },
            VillageCode {
                name: "华阳村村委会",
                code: "028",
            },
            VillageCode {
                name: "东门村村委会",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "新桥镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "新乐社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新育社区居委会",
                code: "002",
            },
            VillageCode {
                name: "民益社区居委会",
                code: "003",
            },
            VillageCode {
                name: "场东社区居委会",
                code: "004",
            },
            VillageCode {
                name: "场西社区居委会",
                code: "005",
            },
            VillageCode {
                name: "潘家浜社区居委会",
                code: "006",
            },
            VillageCode {
                name: "场中社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新东苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "明中社区居委会",
                code: "009",
            },
            VillageCode {
                name: "晨星社区居委会",
                code: "010",
            },
            VillageCode {
                name: "春申社区居委会",
                code: "011",
            },
            VillageCode {
                name: "莘松社区居委会",
                code: "012",
            },
            VillageCode {
                name: "春九社区居委会",
                code: "013",
            },
            VillageCode {
                name: "春莘社区居委会",
                code: "014",
            },
            VillageCode {
                name: "明兴社区居委会",
                code: "015",
            },
            VillageCode {
                name: "白马社区居委会",
                code: "016",
            },
            VillageCode {
                name: "明华社区居委会",
                code: "017",
            },
            VillageCode {
                name: "达安社区居委会",
                code: "018",
            },
            VillageCode {
                name: "馨庭社区居委会",
                code: "019",
            },
            VillageCode {
                name: "新弘社区居委会",
                code: "020",
            },
            VillageCode {
                name: "丁浜社区居委会",
                code: "021",
            },
            VillageCode {
                name: "新泾社区居委会",
                code: "022",
            },
            VillageCode {
                name: "庄浜社区居委会",
                code: "023",
            },
            VillageCode {
                name: "华屿社区居委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "洞泾镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "新欣社区居委会",
                code: "001",
            },
            VillageCode {
                name: "长欣社区居委会",
                code: "002",
            },
            VillageCode {
                name: "海欣社区居委会",
                code: "003",
            },
            VillageCode {
                name: "同欣社区居委会",
                code: "004",
            },
            VillageCode {
                name: "砖桥社区居委会",
                code: "005",
            },
            VillageCode {
                name: "渔洋浜社区居委会",
                code: "006",
            },
            VillageCode {
                name: "塘桥社区居委会",
                code: "007",
            },
            VillageCode {
                name: "光星社区居委会",
                code: "008",
            },
            VillageCode {
                name: "荣欣社区居委会",
                code: "009",
            },
            VillageCode {
                name: "平阳社区居委会",
                code: "010",
            },
            VillageCode {
                name: "集贤社区居委会",
                code: "011",
            },
            VillageCode {
                name: "祥欣社区居委会",
                code: "012",
            },
            VillageCode {
                name: "王家厍社区居委会",
                code: "013",
            },
            VillageCode {
                name: "百花社区居委会",
                code: "014",
            },
            VillageCode {
                name: "百鸟社区居委会",
                code: "015",
            },
            VillageCode {
                name: "云庐社区居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "九亭镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "亭中社区居委会",
                code: "001",
            },
            VillageCode {
                name: "亭东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "亭南社区居委会",
                code: "003",
            },
            VillageCode {
                name: "庄家社区居委会",
                code: "004",
            },
            VillageCode {
                name: "兴联社区居委会",
                code: "005",
            },
            VillageCode {
                name: "朱龙社区居委会",
                code: "006",
            },
            VillageCode {
                name: "金吴社区居委会",
                code: "007",
            },
            VillageCode {
                name: "松沪社区居委会",
                code: "008",
            },
            VillageCode {
                name: "小寅社区居委会",
                code: "009",
            },
            VillageCode {
                name: "朱泾浜社区居委会",
                code: "010",
            },
            VillageCode {
                name: "亭源社区居委会",
                code: "011",
            },
            VillageCode {
                name: "北场社区居委会",
                code: "012",
            },
            VillageCode {
                name: "牛车泾社区居委会",
                code: "013",
            },
            VillageCode {
                name: "国亭社区居委会",
                code: "014",
            },
            VillageCode {
                name: "嘉禾社区居委会",
                code: "015",
            },
            VillageCode {
                name: "颐景园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "复地社区居委会",
                code: "017",
            },
            VillageCode {
                name: "天元社区居委会",
                code: "018",
            },
            VillageCode {
                name: "青春社区居委会",
                code: "019",
            },
            VillageCode {
                name: "云润社区居委会",
                code: "020",
            },
            VillageCode {
                name: "涞亭社区居委会",
                code: "021",
            },
            VillageCode {
                name: "象屿都城社区居委会",
                code: "022",
            },
            VillageCode {
                name: "紫金社区居委会",
                code: "023",
            },
            VillageCode {
                name: "九亭家园社区居委会",
                code: "024",
            },
            VillageCode {
                name: "象屿品城社区居委会",
                code: "025",
            },
            VillageCode {
                name: "亭新社区居委会",
                code: "026",
            },
            VillageCode {
                name: "九亭家园第二社区居委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "泖港镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "泖港社区居委会",
                code: "001",
            },
            VillageCode {
                name: "五厍社区居委会",
                code: "002",
            },
            VillageCode {
                name: "港湾社区居委会",
                code: "003",
            },
            VillageCode {
                name: "焦家村村委会",
                code: "004",
            },
            VillageCode {
                name: "新龚村村委会",
                code: "005",
            },
            VillageCode {
                name: "腰泾村村委会",
                code: "006",
            },
            VillageCode {
                name: "胡光村村委会",
                code: "007",
            },
            VillageCode {
                name: "黄桥村村委会",
                code: "008",
            },
            VillageCode {
                name: "范家村村委会",
                code: "009",
            },
            VillageCode {
                name: "泖港村村委会",
                code: "010",
            },
            VillageCode {
                name: "新建村村委会",
                code: "011",
            },
            VillageCode {
                name: "徐厍村村委会",
                code: "012",
            },
            VillageCode {
                name: "南三村村委会",
                code: "013",
            },
            VillageCode {
                name: "兴旺村村委会",
                code: "014",
            },
            VillageCode {
                name: "田黄村村委会",
                code: "015",
            },
            VillageCode {
                name: "曙光村村委会",
                code: "016",
            },
            VillageCode {
                name: "曹家浜村村委会",
                code: "017",
            },
            VillageCode {
                name: "朱定村村委会",
                code: "018",
            },
            VillageCode {
                name: "茹塘村村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "石湖荡镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "古松社区居委会",
                code: "001",
            },
            VillageCode {
                name: "塔汇社区居委会",
                code: "002",
            },
            VillageCode {
                name: "恬润新苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新姚村村委会",
                code: "004",
            },
            VillageCode {
                name: "泖新村村委会",
                code: "005",
            },
            VillageCode {
                name: "新源村村委会",
                code: "006",
            },
            VillageCode {
                name: "洙桥村村委会",
                code: "007",
            },
            VillageCode {
                name: "东夏村村委会",
                code: "008",
            },
            VillageCode {
                name: "东港村村委会",
                code: "009",
            },
            VillageCode {
                name: "金汇村村委会",
                code: "010",
            },
            VillageCode {
                name: "金胜村村委会",
                code: "011",
            },
            VillageCode {
                name: "张庄村村委会",
                code: "012",
            },
            VillageCode {
                name: "新中村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "新浜镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "桃园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "方家哈社区居委会",
                code: "002",
            },
            VillageCode {
                name: "友谊社区居委会",
                code: "003",
            },
            VillageCode {
                name: "鲁星村村委会",
                code: "004",
            },
            VillageCode {
                name: "南杨村村委会",
                code: "005",
            },
            VillageCode {
                name: "陈堵村村委会",
                code: "006",
            },
            VillageCode {
                name: "赵王村村委会",
                code: "007",
            },
            VillageCode {
                name: "林建村村委会",
                code: "008",
            },
            VillageCode {
                name: "许家草村村委会",
                code: "009",
            },
            VillageCode {
                name: "新浜村村委会",
                code: "010",
            },
            VillageCode {
                name: "黄家埭村村委会",
                code: "011",
            },
            VillageCode {
                name: "文华村村委会",
                code: "012",
            },
            VillageCode {
                name: "胡家埭村村委会",
                code: "013",
            },
            VillageCode {
                name: "香塘村村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "叶榭镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "张泽社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中原社区居委会",
                code: "002",
            },
            VillageCode {
                name: "世强社区居委会",
                code: "003",
            },
            VillageCode {
                name: "世源社区居委会",
                code: "004",
            },
            VillageCode {
                name: "同建村村委会",
                code: "005",
            },
            VillageCode {
                name: "金家村村委会",
                code: "006",
            },
            VillageCode {
                name: "东石村村委会",
                code: "007",
            },
            VillageCode {
                name: "兴达村村委会",
                code: "008",
            },
            VillageCode {
                name: "东勤村村委会",
                code: "009",
            },
            VillageCode {
                name: "堰泾村村委会",
                code: "010",
            },
            VillageCode {
                name: "团结村村委会",
                code: "011",
            },
            VillageCode {
                name: "四村村村委会",
                code: "012",
            },
            VillageCode {
                name: "井凌桥村村委会",
                code: "013",
            },
            VillageCode {
                name: "大庙村村委会",
                code: "014",
            },
            VillageCode {
                name: "马桥村村委会",
                code: "015",
            },
            VillageCode {
                name: "八字桥村村委会",
                code: "016",
            },
            VillageCode {
                name: "徐姚村村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "小昆山镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "秦安社区居委会",
                code: "001",
            },
            VillageCode {
                name: "平原社区居委会",
                code: "002",
            },
            VillageCode {
                name: "昆西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "大港社区居委会",
                code: "004",
            },
            VillageCode {
                name: "玉昆一村社区居委会",
                code: "005",
            },
            VillageCode {
                name: "玉昆二村社区居委会",
                code: "006",
            },
            VillageCode {
                name: "文晋苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "翔昆苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "平复苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "山水华庭社区居委会",
                code: "010",
            },
            VillageCode {
                name: "九峯里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "周家浜村村委会",
                code: "012",
            },
            VillageCode {
                name: "汤村村村委会",
                code: "013",
            },
            VillageCode {
                name: "永丰村村委会",
                code: "014",
            },
            VillageCode {
                name: "荡湾村村委会",
                code: "015",
            },
            VillageCode {
                name: "泾德村村委会",
                code: "016",
            },
            VillageCode {
                name: "港丰村村委会",
                code: "017",
            },
            VillageCode {
                name: "大港村村委会",
                code: "018",
            },
            VillageCode {
                name: "陆家埭村村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "松江工业区",
        code: "018",
        villages: &[VillageCode {
            name: "松江工业区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "佘山度假区",
        code: "019",
        villages: &[VillageCode {
            name: "佘山度假区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "上海松江出口加工区",
        code: "020",
        villages: &[VillageCode {
            name: "上海松江出口加工区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_SJ_015: [TownCode; 11] = [
    TownCode {
        name: "夏阳街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "东盛社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东方社区居委会",
                code: "002",
            },
            VillageCode {
                name: "章浜社区居委会",
                code: "003",
            },
            VillageCode {
                name: "青城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "祥龙社区居委会",
                code: "005",
            },
            VillageCode {
                name: "界泾港社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新青浦社区居委会",
                code: "007",
            },
            VillageCode {
                name: "桂花园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "华骥苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "青湖社区居委会",
                code: "010",
            },
            VillageCode {
                name: "夏阳湖社区居委会",
                code: "011",
            },
            VillageCode {
                name: "千步泾社区居委会",
                code: "012",
            },
            VillageCode {
                name: "佳乐苑社区居委会",
                code: "013",
            },
            VillageCode {
                name: "仓桥社区居委会",
                code: "014",
            },
            VillageCode {
                name: "宜达社区居委会",
                code: "015",
            },
            VillageCode {
                name: "青平社区居委会",
                code: "016",
            },
            VillageCode {
                name: "青松社区居委会",
                code: "017",
            },
            VillageCode {
                name: "青华社区居委会",
                code: "018",
            },
            VillageCode {
                name: "青安社区居委会",
                code: "019",
            },
            VillageCode {
                name: "青乐社区居委会",
                code: "020",
            },
            VillageCode {
                name: "青泽社区居委会",
                code: "021",
            },
            VillageCode {
                name: "青园社区居委会",
                code: "022",
            },
            VillageCode {
                name: "南箐园社区居委会",
                code: "023",
            },
            VillageCode {
                name: "青科社区居委会",
                code: "024",
            },
            VillageCode {
                name: "王仙村村委会",
                code: "025",
            },
            VillageCode {
                name: "新阳村村委会",
                code: "026",
            },
            VillageCode {
                name: "塘郁村村委会",
                code: "027",
            },
            VillageCode {
                name: "金家村村委会",
                code: "028",
            },
            VillageCode {
                name: "枫泾村村委会",
                code: "029",
            },
            VillageCode {
                name: "太来村村委会",
                code: "030",
            },
            VillageCode {
                name: "塔湾村村委会",
                code: "031",
            },
            VillageCode {
                name: "城南村村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "盈浦街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "庆华社区居委会",
                code: "001",
            },
            VillageCode {
                name: "庆新社区居委会",
                code: "002",
            },
            VillageCode {
                name: "城北社区居委会",
                code: "003",
            },
            VillageCode {
                name: "龙威社区居委会",
                code: "004",
            },
            VillageCode {
                name: "复兴社区居委会",
                code: "005",
            },
            VillageCode {
                name: "解放社区居委会",
                code: "006",
            },
            VillageCode {
                name: "西部花苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "尚美社区居委会",
                code: "008",
            },
            VillageCode {
                name: "万寿社区居委会",
                code: "009",
            },
            VillageCode {
                name: "盈港社区居委会",
                code: "010",
            },
            VillageCode {
                name: "盈中社区居委会",
                code: "011",
            },
            VillageCode {
                name: "三元河社区居委会",
                code: "012",
            },
            VillageCode {
                name: "盈联社区居委会",
                code: "013",
            },
            VillageCode {
                name: "民乐社区居委会",
                code: "014",
            },
            VillageCode {
                name: "民佳社区居委会",
                code: "015",
            },
            VillageCode {
                name: "绿舟社区居委会",
                code: "016",
            },
            VillageCode {
                name: "上达社区居委会",
                code: "017",
            },
            VillageCode {
                name: "浩泽社区居委会",
                code: "018",
            },
            VillageCode {
                name: "民欣社区居委会",
                code: "019",
            },
            VillageCode {
                name: "华浦社区居委会",
                code: "020",
            },
            VillageCode {
                name: "怡澜社区居委会",
                code: "021",
            },
            VillageCode {
                name: "双桥社区居委会",
                code: "022",
            },
            VillageCode {
                name: "东渡社区居委会",
                code: "023",
            },
            VillageCode {
                name: "赵屯浦社区居委会",
                code: "024",
            },
            VillageCode {
                name: "崧子浦社区居委会",
                code: "025",
            },
            VillageCode {
                name: "贺桥社区居委会",
                code: "026",
            },
            VillageCode {
                name: "淀山浦社区居委会",
                code: "027",
            },
            VillageCode {
                name: "漕盈社区居委会",
                code: "028",
            },
            VillageCode {
                name: "新塘浦社区居委会",
                code: "029",
            },
            VillageCode {
                name: "盘龙浦社区居委会",
                code: "030",
            },
            VillageCode {
                name: "贺桥村村委会",
                code: "031",
            },
            VillageCode {
                name: "南厍村村委会",
                code: "032",
            },
            VillageCode {
                name: "俞家埭村村委会",
                code: "033",
            },
            VillageCode {
                name: "天恩桥村村委会",
                code: "034",
            },
            VillageCode {
                name: "南横村村委会",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "香花桥街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "香花桥居委会",
                code: "001",
            },
            VillageCode {
                name: "大盈居委会",
                code: "002",
            },
            VillageCode {
                name: "青山社区居委会",
                code: "003",
            },
            VillageCode {
                name: "金巷社区居委会",
                code: "004",
            },
            VillageCode {
                name: "民惠社区居委会",
                code: "005",
            },
            VillageCode {
                name: "民惠第二社区居委会",
                code: "006",
            },
            VillageCode {
                name: "都汇华庭社区居委会",
                code: "007",
            },
            VillageCode {
                name: "桃源埔社区居委会",
                code: "008",
            },
            VillageCode {
                name: "清河湾社区居委会",
                code: "009",
            },
            VillageCode {
                name: "友爱社区居委会",
                code: "010",
            },
            VillageCode {
                name: "玉兰花园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "玫瑰湾社区居委会",
                code: "012",
            },
            VillageCode {
                name: "民惠社区第三居委会",
                code: "013",
            },
            VillageCode {
                name: "盈中村村委会",
                code: "014",
            },
            VillageCode {
                name: "胜利村村委会",
                code: "015",
            },
            VillageCode {
                name: "石西村村委会",
                code: "016",
            },
            VillageCode {
                name: "陈桥村村委会",
                code: "017",
            },
            VillageCode {
                name: "杨元村村委会",
                code: "018",
            },
            VillageCode {
                name: "袁家村村委会",
                code: "019",
            },
            VillageCode {
                name: "七汇村村委会",
                code: "020",
            },
            VillageCode {
                name: "郏一村村委会",
                code: "021",
            },
            VillageCode {
                name: "曹泾村村委会",
                code: "022",
            },
            VillageCode {
                name: "金星村村委会",
                code: "023",
            },
            VillageCode {
                name: "朝阳村村委会",
                code: "024",
            },
            VillageCode {
                name: "新姚村村委会",
                code: "025",
            },
            VillageCode {
                name: "天一村村委会",
                code: "026",
            },
            VillageCode {
                name: "向阳村村委会",
                code: "027",
            },
            VillageCode {
                name: "新桥村村委会",
                code: "028",
            },
            VillageCode {
                name: "东方村村委会",
                code: "029",
            },
            VillageCode {
                name: "泾阳村村委会",
                code: "030",
            },
            VillageCode {
                name: "燕南村村委会",
                code: "031",
            },
            VillageCode {
                name: "大联村村委会",
                code: "032",
            },
            VillageCode {
                name: "东斜村村委会",
                code: "033",
            },
            VillageCode {
                name: "金米村村委会",
                code: "034",
            },
            VillageCode {
                name: "爱星村村委会",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "朱家角镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "大新街居委会",
                code: "001",
            },
            VillageCode {
                name: "胜利街居委会",
                code: "002",
            },
            VillageCode {
                name: "东湖街居委会",
                code: "003",
            },
            VillageCode {
                name: "西湖新村居委会",
                code: "004",
            },
            VillageCode {
                name: "北大街居委会",
                code: "005",
            },
            VillageCode {
                name: "东井街居委会",
                code: "006",
            },
            VillageCode {
                name: "大淀湖居委会",
                code: "007",
            },
            VillageCode {
                name: "沈巷居委会",
                code: "008",
            },
            VillageCode {
                name: "东大门社区居委会",
                code: "009",
            },
            VillageCode {
                name: "泰安第一社区居委会",
                code: "010",
            },
            VillageCode {
                name: "泰安第二社区居委会",
                code: "011",
            },
            VillageCode {
                name: "珠溪社区居委会",
                code: "012",
            },
            VillageCode {
                name: "珠湖社区居委会",
                code: "013",
            },
            VillageCode {
                name: "淀湖社区居委会",
                code: "014",
            },
            VillageCode {
                name: "泖阳社区居委会",
                code: "015",
            },
            VillageCode {
                name: "陈家汇社区居委会",
                code: "016",
            },
            VillageCode {
                name: "滨湖社区居委会",
                code: "017",
            },
            VillageCode {
                name: "浦泰社区居委会",
                code: "018",
            },
            VillageCode {
                name: "周荡村村委会",
                code: "019",
            },
            VillageCode {
                name: "横江村村委会",
                code: "020",
            },
            VillageCode {
                name: "盛家埭村村委会",
                code: "021",
            },
            VillageCode {
                name: "张家圩村村委会",
                code: "022",
            },
            VillageCode {
                name: "新旺村村委会",
                code: "023",
            },
            VillageCode {
                name: "新华村村委会",
                code: "024",
            },
            VillageCode {
                name: "小江村村委会",
                code: "025",
            },
            VillageCode {
                name: "周家港村村委会",
                code: "026",
            },
            VillageCode {
                name: "沙家埭村村委会",
                code: "027",
            },
            VillageCode {
                name: "山湾村村委会",
                code: "028",
            },
            VillageCode {
                name: "庆丰村村委会",
                code: "029",
            },
            VillageCode {
                name: "淀峰村村委会",
                code: "030",
            },
            VillageCode {
                name: "山海桥村村委会",
                code: "031",
            },
            VillageCode {
                name: "创建村村委会",
                code: "032",
            },
            VillageCode {
                name: "淀山湖一村村委会",
                code: "033",
            },
            VillageCode {
                name: "水产村村委会",
                code: "034",
            },
            VillageCode {
                name: "安庄村村委会",
                code: "035",
            },
            VillageCode {
                name: "先锋村村委会",
                code: "036",
            },
            VillageCode {
                name: "沈巷村村委会",
                code: "037",
            },
            VillageCode {
                name: "张马村村委会",
                code: "038",
            },
            VillageCode {
                name: "李庄村村委会",
                code: "039",
            },
            VillageCode {
                name: "建新村村委会",
                code: "040",
            },
            VillageCode {
                name: "王金村村委会",
                code: "041",
            },
            VillageCode {
                name: "林家村村委会",
                code: "042",
            },
            VillageCode {
                name: "新胜村村委会",
                code: "043",
            },
            VillageCode {
                name: "张巷村村委会",
                code: "044",
            },
            VillageCode {
                name: "万隆村村委会",
                code: "045",
            },
            VillageCode {
                name: "薛间村村委会",
                code: "046",
            },
        ],
    },
    TownCode {
        name: "练塘镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "下塘居委会",
                code: "001",
            },
            VillageCode {
                name: "湾塘居委会",
                code: "002",
            },
            VillageCode {
                name: "小蒸居委会",
                code: "003",
            },
            VillageCode {
                name: "蒸淀居委会",
                code: "004",
            },
            VillageCode {
                name: "三里塘居委会",
                code: "005",
            },
            VillageCode {
                name: "泖甸村村委会",
                code: "006",
            },
            VillageCode {
                name: "泾花村村委会",
                code: "007",
            },
            VillageCode {
                name: "练东村村委会",
                code: "008",
            },
            VillageCode {
                name: "泾珠村村委会",
                code: "009",
            },
            VillageCode {
                name: "北埭村村委会",
                code: "010",
            },
            VillageCode {
                name: "金前村村委会",
                code: "011",
            },
            VillageCode {
                name: "太北村村委会",
                code: "012",
            },
            VillageCode {
                name: "叶港村村委会",
                code: "013",
            },
            VillageCode {
                name: "朱庄村村委会",
                code: "014",
            },
            VillageCode {
                name: "东泖村村委会",
                code: "015",
            },
            VillageCode {
                name: "东田村村委会",
                code: "016",
            },
            VillageCode {
                name: "联农村村委会",
                code: "017",
            },
            VillageCode {
                name: "双菱村村委会",
                code: "018",
            },
            VillageCode {
                name: "东淇村村委会",
                code: "019",
            },
            VillageCode {
                name: "长河村村委会",
                code: "020",
            },
            VillageCode {
                name: "大新村村委会",
                code: "021",
            },
            VillageCode {
                name: "东厍村村委会",
                code: "022",
            },
            VillageCode {
                name: "张联村村委会",
                code: "023",
            },
            VillageCode {
                name: "徐练村村委会",
                code: "024",
            },
            VillageCode {
                name: "浦南村村委会",
                code: "025",
            },
            VillageCode {
                name: "蒸浦村村委会",
                code: "026",
            },
            VillageCode {
                name: "东庄村村委会",
                code: "027",
            },
            VillageCode {
                name: "蒸夏村村委会",
                code: "028",
            },
            VillageCode {
                name: "芦潼村村委会",
                code: "029",
            },
            VillageCode {
                name: "星浜村村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "金泽镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "金溪居委会",
                code: "001",
            },
            VillageCode {
                name: "金杨居委会",
                code: "002",
            },
            VillageCode {
                name: "西岑居委会",
                code: "003",
            },
            VillageCode {
                name: "莲盛居委会",
                code: "004",
            },
            VillageCode {
                name: "商榻居委会",
                code: "005",
            },
            VillageCode {
                name: "徐李村村委会",
                code: "006",
            },
            VillageCode {
                name: "新池村村委会",
                code: "007",
            },
            VillageCode {
                name: "金泽村村委会",
                code: "008",
            },
            VillageCode {
                name: "杨湾村村委会",
                code: "009",
            },
            VillageCode {
                name: "东西村村委会",
                code: "010",
            },
            VillageCode {
                name: "建国村村委会",
                code: "011",
            },
            VillageCode {
                name: "金姚村村委会",
                code: "012",
            },
            VillageCode {
                name: "新港村村委会",
                code: "013",
            },
            VillageCode {
                name: "岑卜村村委会",
                code: "014",
            },
            VillageCode {
                name: "西岑村村委会",
                code: "015",
            },
            VillageCode {
                name: "三塘村村委会",
                code: "016",
            },
            VillageCode {
                name: "育田村村委会",
                code: "017",
            },
            VillageCode {
                name: "河祝村村委会",
                code: "018",
            },
            VillageCode {
                name: "爱国村村委会",
                code: "019",
            },
            VillageCode {
                name: "东天村村委会",
                code: "020",
            },
            VillageCode {
                name: "任屯村村委会",
                code: "021",
            },
            VillageCode {
                name: "田山庄村村委会",
                code: "022",
            },
            VillageCode {
                name: "龚都村村委会",
                code: "023",
            },
            VillageCode {
                name: "钱盛村村委会",
                code: "024",
            },
            VillageCode {
                name: "莲湖村村委会",
                code: "025",
            },
            VillageCode {
                name: "淀湖村村委会",
                code: "026",
            },
            VillageCode {
                name: "蔡浜村村委会",
                code: "027",
            },
            VillageCode {
                name: "东星村村委会",
                code: "028",
            },
            VillageCode {
                name: "王港村村委会",
                code: "029",
            },
            VillageCode {
                name: "双祥村村委会",
                code: "030",
            },
            VillageCode {
                name: "南新村村委会",
                code: "031",
            },
            VillageCode {
                name: "陈东村村委会",
                code: "032",
            },
            VillageCode {
                name: "雪米村村委会",
                code: "033",
            },
            VillageCode {
                name: "淀西村村委会",
                code: "034",
            },
            VillageCode {
                name: "沙港村村委会",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "赵巷镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "赵巷居委会",
                code: "001",
            },
            VillageCode {
                name: "北崧居委会",
                code: "002",
            },
            VillageCode {
                name: "新镇居委会",
                code: "003",
            },
            VillageCode {
                name: "金葫芦社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金葫芦第二社区居委会",
                code: "005",
            },
            VillageCode {
                name: "崧涵社区居委会",
                code: "006",
            },
            VillageCode {
                name: "崧湖社区居委会",
                code: "007",
            },
            VillageCode {
                name: "佳福东社区居委会",
                code: "008",
            },
            VillageCode {
                name: "崧鑫社区居委会",
                code: "009",
            },
            VillageCode {
                name: "巷佳社区居委会",
                code: "010",
            },
            VillageCode {
                name: "华沁社区居委会",
                code: "011",
            },
            VillageCode {
                name: "华秀社区居委会",
                code: "012",
            },
            VillageCode {
                name: "佳昱社区居委会",
                code: "013",
            },
            VillageCode {
                name: "龙联社区居委会",
                code: "014",
            },
            VillageCode {
                name: "秀景社区居委会",
                code: "015",
            },
            VillageCode {
                name: "佳煌社区居委会",
                code: "016",
            },
            VillageCode {
                name: "佳辉社区居委会",
                code: "017",
            },
            VillageCode {
                name: "逸泰社区居委会",
                code: "018",
            },
            VillageCode {
                name: "逸秀社区居委会",
                code: "019",
            },
            VillageCode {
                name: "龙御社区居委会",
                code: "020",
            },
            VillageCode {
                name: "和瑞社区居委会",
                code: "021",
            },
            VillageCode {
                name: "德康社区居委会",
                code: "022",
            },
            VillageCode {
                name: "南崧村村委会",
                code: "023",
            },
            VillageCode {
                name: "方夏村村委会",
                code: "024",
            },
            VillageCode {
                name: "和睦村村委会",
                code: "025",
            },
            VillageCode {
                name: "垂姚村村委会",
                code: "026",
            },
            VillageCode {
                name: "沈泾塘村村委会",
                code: "027",
            },
            VillageCode {
                name: "崧泽村村委会",
                code: "028",
            },
            VillageCode {
                name: "中步村村委会",
                code: "029",
            },
            VillageCode {
                name: "金汇村村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "徐泾镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "徐泾居委会",
                code: "001",
            },
            VillageCode {
                name: "龙阳居委会",
                code: "002",
            },
            VillageCode {
                name: "宅东居委会",
                code: "003",
            },
            VillageCode {
                name: "京华居委会",
                code: "004",
            },
            VillageCode {
                name: "蟠龙居委会",
                code: "005",
            },
            VillageCode {
                name: "徐安第一社区居委会",
                code: "006",
            },
            VillageCode {
                name: "徐安第二社区居委会",
                code: "007",
            },
            VillageCode {
                name: "高泾社区居委会",
                code: "008",
            },
            VillageCode {
                name: "卫家角第一社区居委会",
                code: "009",
            },
            VillageCode {
                name: "卫家角第二社区居委会",
                code: "010",
            },
            VillageCode {
                name: "徐安第三社区居委会",
                code: "011",
            },
            VillageCode {
                name: "徐安第四社区居委会",
                code: "012",
            },
            VillageCode {
                name: "玉兰清苑社区居委会",
                code: "013",
            },
            VillageCode {
                name: "尚鸿路社区居委会",
                code: "014",
            },
            VillageCode {
                name: "尚泰路社区居委会",
                code: "015",
            },
            VillageCode {
                name: "尚茂路社区居委会",
                code: "016",
            },
            VillageCode {
                name: "仁恒西郊社区居委会",
                code: "017",
            },
            VillageCode {
                name: "卫家角第三社区居委会",
                code: "018",
            },
            VillageCode {
                name: "明珠路社区居委会",
                code: "019",
            },
            VillageCode {
                name: "夏都社区居委会",
                code: "020",
            },
            VillageCode {
                name: "叶联路社区居委会",
                code: "021",
            },
            VillageCode {
                name: "蟠龙馨苑社区居委会",
                code: "022",
            },
            VillageCode {
                name: "蟠文路社区居委会",
                code: "023",
            },
            VillageCode {
                name: "徐盈路社区居委会",
                code: "024",
            },
            VillageCode {
                name: "诸光路社区居委会",
                code: "025",
            },
            VillageCode {
                name: "乐国路社区居委会",
                code: "026",
            },
            VillageCode {
                name: "乐天社区居委会",
                code: "027",
            },
            VillageCode {
                name: "徐乐路社区居委会",
                code: "028",
            },
            VillageCode {
                name: "前明村村委会",
                code: "029",
            },
            VillageCode {
                name: "金云村村委会",
                code: "030",
            },
            VillageCode {
                name: "联民村村委会",
                code: "031",
            },
            VillageCode {
                name: "光联村村委会",
                code: "032",
            },
            VillageCode {
                name: "民主村村委会",
                code: "033",
            },
            VillageCode {
                name: "二联村村委会",
                code: "034",
            },
            VillageCode {
                name: "金联村村委会",
                code: "035",
            },
            VillageCode {
                name: "迮庵村村委会",
                code: "036",
            },
            VillageCode {
                name: "陆家角村村委会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "华新镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "华新居委会",
                code: "001",
            },
            VillageCode {
                name: "凤溪居委会",
                code: "002",
            },
            VillageCode {
                name: "华腾社区居委会",
                code: "003",
            },
            VillageCode {
                name: "西郊半岛社区居委会",
                code: "004",
            },
            VillageCode {
                name: "宝龙社区居委会",
                code: "005",
            },
            VillageCode {
                name: "星尚湾社区居委会",
                code: "006",
            },
            VillageCode {
                name: "华府社区居委会",
                code: "007",
            },
            VillageCode {
                name: "悦欣社区居委会",
                code: "008",
            },
            VillageCode {
                name: "金瑞苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新丰社区居委会",
                code: "010",
            },
            VillageCode {
                name: "春江社区居委会",
                code: "011",
            },
            VillageCode {
                name: "瑞和锦庭社区居委会",
                code: "012",
            },
            VillageCode {
                name: "华悦社区居委会",
                code: "013",
            },
            VillageCode {
                name: "悦澜社区居委会",
                code: "014",
            },
            VillageCode {
                name: "华强社区居委会",
                code: "015",
            },
            VillageCode {
                name: "徐谢村村委会",
                code: "016",
            },
            VillageCode {
                name: "周浜村村委会",
                code: "017",
            },
            VillageCode {
                name: "朱长村村委会",
                code: "018",
            },
            VillageCode {
                name: "华益村村委会",
                code: "019",
            },
            VillageCode {
                name: "陆象村村委会",
                code: "020",
            },
            VillageCode {
                name: "凌家村村委会",
                code: "021",
            },
            VillageCode {
                name: "淮海村村委会",
                code: "022",
            },
            VillageCode {
                name: "马阳村村委会",
                code: "023",
            },
            VillageCode {
                name: "秀龙村村委会",
                code: "024",
            },
            VillageCode {
                name: "新谊村村委会",
                code: "025",
            },
            VillageCode {
                name: "火星村村委会",
                code: "026",
            },
            VillageCode {
                name: "嵩山村村委会",
                code: "027",
            },
            VillageCode {
                name: "北新村村委会",
                code: "028",
            },
            VillageCode {
                name: "新木桥村村委会",
                code: "029",
            },
            VillageCode {
                name: "叙南村村委会",
                code: "030",
            },
            VillageCode {
                name: "叙中村村委会",
                code: "031",
            },
            VillageCode {
                name: "坚强村村委会",
                code: "032",
            },
            VillageCode {
                name: "白马塘村村委会",
                code: "033",
            },
            VillageCode {
                name: "杨家庄村村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "重固镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "福泉社区居委会",
                code: "001",
            },
            VillageCode {
                name: "泉山社区居委会",
                code: "002",
            },
            VillageCode {
                name: "福定社区居委会",
                code: "003",
            },
            VillageCode {
                name: "泉华社区居委会",
                code: "004",
            },
            VillageCode {
                name: "泉祥社区居委会",
                code: "005",
            },
            VillageCode {
                name: "福兆社区居委会",
                code: "006",
            },
            VillageCode {
                name: "回龙村村委会",
                code: "007",
            },
            VillageCode {
                name: "新丰村村委会",
                code: "008",
            },
            VillageCode {
                name: "章堰村村委会",
                code: "009",
            },
            VillageCode {
                name: "徐姚村村委会",
                code: "010",
            },
            VillageCode {
                name: "郏店村村委会",
                code: "011",
            },
            VillageCode {
                name: "毛家角村村委会",
                code: "012",
            },
            VillageCode {
                name: "新联村村委会",
                code: "013",
            },
            VillageCode {
                name: "中新村村委会",
                code: "014",
            },
            VillageCode {
                name: "福泉山村村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "白鹤镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "白鹤一居委会",
                code: "001",
            },
            VillageCode {
                name: "白鹤二居委会",
                code: "002",
            },
            VillageCode {
                name: "赵屯居委会",
                code: "003",
            },
            VillageCode {
                name: "新江居委会",
                code: "004",
            },
            VillageCode {
                name: "白虬江居委会",
                code: "005",
            },
            VillageCode {
                name: "白鹤村村委会",
                code: "006",
            },
            VillageCode {
                name: "沈联村村委会",
                code: "007",
            },
            VillageCode {
                name: "鹤联村村委会",
                code: "008",
            },
            VillageCode {
                name: "青龙村村委会",
                code: "009",
            },
            VillageCode {
                name: "塘湾村村委会",
                code: "010",
            },
            VillageCode {
                name: "胜新村村委会",
                code: "011",
            },
            VillageCode {
                name: "朱浦村村委会",
                code: "012",
            },
            VillageCode {
                name: "金项村村委会",
                code: "013",
            },
            VillageCode {
                name: "新江村村委会",
                code: "014",
            },
            VillageCode {
                name: "王泾村村委会",
                code: "015",
            },
            VillageCode {
                name: "杜村村委会",
                code: "016",
            },
            VillageCode {
                name: "响新村村委会",
                code: "017",
            },
            VillageCode {
                name: "五里村村委会",
                code: "018",
            },
            VillageCode {
                name: "万狮村村委会",
                code: "019",
            },
            VillageCode {
                name: "梅桥村村委会",
                code: "020",
            },
            VillageCode {
                name: "曙光村村委会",
                code: "021",
            },
            VillageCode {
                name: "红旗村村委会",
                code: "022",
            },
            VillageCode {
                name: "南巷村村委会",
                code: "023",
            },
            VillageCode {
                name: "江南村村委会",
                code: "024",
            },
            VillageCode {
                name: "太平村村委会",
                code: "025",
            },
            VillageCode {
                name: "赵屯村村委会",
                code: "026",
            },
        ],
    },
];

static TOWNS_SJ_016: [TownCode; 12] = [
    TownCode {
        name: "西渡街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "鸿宝第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "鸿宝第二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "鸿宝第三社区居委会",
                code: "003",
            },
            VillageCode {
                name: "水闸居委会",
                code: "004",
            },
            VillageCode {
                name: "闸园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "浦江居委会",
                code: "006",
            },
            VillageCode {
                name: "新南社区居委会",
                code: "007",
            },
            VillageCode {
                name: "水岸社区居委会",
                code: "008",
            },
            VillageCode {
                name: "文怡社区居委会",
                code: "009",
            },
            VillageCode {
                name: "金都居委会",
                code: "010",
            },
            VillageCode {
                name: "香榭居委会",
                code: "011",
            },
            VillageCode {
                name: "兰新居委会",
                code: "012",
            },
            VillageCode {
                name: "发展村村委会",
                code: "013",
            },
            VillageCode {
                name: "灯塔村村委会",
                code: "014",
            },
            VillageCode {
                name: "南渡村村委会",
                code: "015",
            },
            VillageCode {
                name: "金港村村委会",
                code: "016",
            },
            VillageCode {
                name: "益民村村委会",
                code: "017",
            },
            VillageCode {
                name: "关港村村委会",
                code: "018",
            },
            VillageCode {
                name: "北新村村委会",
                code: "019",
            },
            VillageCode {
                name: "五宅村村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "奉浦街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "奉浦一居委会",
                code: "001",
            },
            VillageCode {
                name: "奉浦二居委会",
                code: "002",
            },
            VillageCode {
                name: "奉浦三居委会",
                code: "003",
            },
            VillageCode {
                name: "第四社区居委会",
                code: "004",
            },
            VillageCode {
                name: "第五社区居委会",
                code: "005",
            },
            VillageCode {
                name: "第六社区居委会",
                code: "006",
            },
            VillageCode {
                name: "第七社区居委会",
                code: "007",
            },
            VillageCode {
                name: "第九社区居委会",
                code: "008",
            },
            VillageCode {
                name: "肖塘居委会",
                code: "009",
            },
            VillageCode {
                name: "弯弯社区居委会",
                code: "010",
            },
            VillageCode {
                name: "天鹅湾社区居委会",
                code: "011",
            },
            VillageCode {
                name: "秋月社区居委会",
                code: "012",
            },
            VillageCode {
                name: "君望居委会",
                code: "013",
            },
            VillageCode {
                name: "秦塘居委会",
                code: "014",
            },
            VillageCode {
                name: "汇贤居委会",
                code: "015",
            },
            VillageCode {
                name: "高远居委会",
                code: "016",
            },
            VillageCode {
                name: "幸福里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "陈湾村村委会",
                code: "018",
            },
            VillageCode {
                name: "公谊村村委会",
                code: "019",
            },
            VillageCode {
                name: "韩村村委会",
                code: "020",
            },
            VillageCode {
                name: "九华村村委会",
                code: "021",
            },
            VillageCode {
                name: "肖塘村村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "金海街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "金水苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金水新苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "恒盛居委会",
                code: "003",
            },
            VillageCode {
                name: "恒贤居委会",
                code: "004",
            },
            VillageCode {
                name: "金水佳苑居委会",
                code: "005",
            },
            VillageCode {
                name: "金水丽苑居委会",
                code: "006",
            },
            VillageCode {
                name: "少湖社区居委会",
                code: "007",
            },
            VillageCode {
                name: "龙潭社区居委会",
                code: "008",
            },
            VillageCode {
                name: "文杏社区居委会",
                code: "009",
            },
            VillageCode {
                name: "金水和苑居委会",
                code: "010",
            },
            VillageCode {
                name: "金水璟苑居委会",
                code: "011",
            },
            VillageCode {
                name: "正学社区居委会",
                code: "012",
            },
            VillageCode {
                name: "丰乐社区居委会",
                code: "013",
            },
            VillageCode {
                name: "韩树村村委会",
                code: "014",
            },
            VillageCode {
                name: "陈谊村村委会",
                code: "015",
            },
            VillageCode {
                name: "齐贤村村委会",
                code: "016",
            },
            VillageCode {
                name: "龙潭村村委会",
                code: "017",
            },
            VillageCode {
                name: "三长村村委会",
                code: "018",
            },
            VillageCode {
                name: "陈家村村委会",
                code: "019",
            },
            VillageCode {
                name: "屠家村村委会",
                code: "020",
            },
            VillageCode {
                name: "丁家村村委会",
                code: "021",
            },
            VillageCode {
                name: "石家村村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "南桥镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "古华第一居委会",
                code: "001",
            },
            VillageCode {
                name: "古华第二居委会",
                code: "002",
            },
            VillageCode {
                name: "古华第三居委会",
                code: "003",
            },
            VillageCode {
                name: "运河居委会",
                code: "004",
            },
            VillageCode {
                name: "育秀第一居委会",
                code: "005",
            },
            VillageCode {
                name: "育秀第二居委会",
                code: "006",
            },
            VillageCode {
                name: "育秀第三居委会",
                code: "007",
            },
            VillageCode {
                name: "育秀第五居委会",
                code: "008",
            },
            VillageCode {
                name: "育秀第七居委会",
                code: "009",
            },
            VillageCode {
                name: "解放第一居委会",
                code: "010",
            },
            VillageCode {
                name: "解放第二居委会",
                code: "011",
            },
            VillageCode {
                name: "解放第三居委会",
                code: "012",
            },
            VillageCode {
                name: "北街居委会",
                code: "013",
            },
            VillageCode {
                name: "中街居委会",
                code: "014",
            },
            VillageCode {
                name: "南街居委会",
                code: "015",
            },
            VillageCode {
                name: "曙光第一居委会",
                code: "016",
            },
            VillageCode {
                name: "贝港第一居委会",
                code: "017",
            },
            VillageCode {
                name: "贝港第二居委会",
                code: "018",
            },
            VillageCode {
                name: "贝港第三居委会",
                code: "019",
            },
            VillageCode {
                name: "贝港第四居委会",
                code: "020",
            },
            VillageCode {
                name: "贝港第五居委会",
                code: "021",
            },
            VillageCode {
                name: "江海第一居委会",
                code: "022",
            },
            VillageCode {
                name: "江海第二居委会",
                code: "023",
            },
            VillageCode {
                name: "江海第三居委会",
                code: "024",
            },
            VillageCode {
                name: "江海第四居委会",
                code: "025",
            },
            VillageCode {
                name: "江海第五居委会",
                code: "026",
            },
            VillageCode {
                name: "曙光第二社区居委会",
                code: "027",
            },
            VillageCode {
                name: "阳光第一社区居委会",
                code: "028",
            },
            VillageCode {
                name: "正阳第一社区居委会",
                code: "029",
            },
            VillageCode {
                name: "阳光第二社区居委会",
                code: "030",
            },
            VillageCode {
                name: "贝港第六社区居委会",
                code: "031",
            },
            VillageCode {
                name: "正阳第二社区居委会",
                code: "032",
            },
            VillageCode {
                name: "民旺苑第一社区居委会",
                code: "033",
            },
            VillageCode {
                name: "正阳第三社区居委会",
                code: "034",
            },
            VillageCode {
                name: "阳光第三社区居委会",
                code: "035",
            },
            VillageCode {
                name: "朝阳居委会",
                code: "036",
            },
            VillageCode {
                name: "新悦社区居委会",
                code: "037",
            },
            VillageCode {
                name: "银河社区居委会",
                code: "038",
            },
            VillageCode {
                name: "昊海社区居委会",
                code: "039",
            },
            VillageCode {
                name: "富康社区居委会",
                code: "040",
            },
            VillageCode {
                name: "尚贤居委会",
                code: "041",
            },
            VillageCode {
                name: "悦晟居委会",
                code: "042",
            },
            VillageCode {
                name: "名悦居委会",
                code: "043",
            },
            VillageCode {
                name: "沈陆村村委会",
                code: "044",
            },
            VillageCode {
                name: "江海村村委会",
                code: "045",
            },
            VillageCode {
                name: "六墩村村委会",
                code: "046",
            },
            VillageCode {
                name: "曙光村村委会",
                code: "047",
            },
            VillageCode {
                name: "张翁庙村村委会",
                code: "048",
            },
            VillageCode {
                name: "华严村村委会",
                code: "049",
            },
            VillageCode {
                name: "庙泾村村委会",
                code: "050",
            },
            VillageCode {
                name: "光明村村委会",
                code: "051",
            },
            VillageCode {
                name: "杨王村村委会",
                code: "052",
            },
            VillageCode {
                name: "吴塘村村委会",
                code: "053",
            },
            VillageCode {
                name: "灵芝村村委会",
                code: "054",
            },
        ],
    },
    TownCode {
        name: "奉城镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "第二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "第三社区居委会",
                code: "003",
            },
            VillageCode {
                name: "第四社区居委会",
                code: "004",
            },
            VillageCode {
                name: "头桥第一社区居委会",
                code: "005",
            },
            VillageCode {
                name: "头桥第二社区居委会",
                code: "006",
            },
            VillageCode {
                name: "洪庙第一社区居委会",
                code: "007",
            },
            VillageCode {
                name: "洪庙第二社区居委会",
                code: "008",
            },
            VillageCode {
                name: "塘外社区居委会",
                code: "009",
            },
            VillageCode {
                name: "奉城社区第五居委会",
                code: "010",
            },
            VillageCode {
                name: "奉城社区第六居委会",
                code: "011",
            },
            VillageCode {
                name: "奉馨社区居委会",
                code: "012",
            },
            VillageCode {
                name: "浦兰居委会",
                code: "013",
            },
            VillageCode {
                name: "洪韵居委会",
                code: "014",
            },
            VillageCode {
                name: "三角洋居委会",
                code: "015",
            },
            VillageCode {
                name: "灯民村村委会",
                code: "016",
            },
            VillageCode {
                name: "高桥村村委会",
                code: "017",
            },
            VillageCode {
                name: "陈桥村村委会",
                code: "018",
            },
            VillageCode {
                name: "八字村村委会",
                code: "019",
            },
            VillageCode {
                name: "路口村村委会",
                code: "020",
            },
            VillageCode {
                name: "久茂村村委会",
                code: "021",
            },
            VillageCode {
                name: "爱民村村委会",
                code: "022",
            },
            VillageCode {
                name: "永民村村委会",
                code: "023",
            },
            VillageCode {
                name: "联民村村委会",
                code: "024",
            },
            VillageCode {
                name: "新民村村委会",
                code: "025",
            },
            VillageCode {
                name: "东门村村委会",
                code: "026",
            },
            VillageCode {
                name: "城东村村委会",
                code: "027",
            },
            VillageCode {
                name: "南街村村委会",
                code: "028",
            },
            VillageCode {
                name: "奉城村村委会",
                code: "029",
            },
            VillageCode {
                name: "北门村村委会",
                code: "030",
            },
            VillageCode {
                name: "白衣聚村村委会",
                code: "031",
            },
            VillageCode {
                name: "盐行村村委会",
                code: "032",
            },
            VillageCode {
                name: "朱墩村村委会",
                code: "033",
            },
            VillageCode {
                name: "大门村村委会",
                code: "034",
            },
            VillageCode {
                name: "塘外村村委会",
                code: "035",
            },
            VillageCode {
                name: "卫季村村委会",
                code: "036",
            },
            VillageCode {
                name: "护民村村委会",
                code: "037",
            },
            VillageCode {
                name: "洪东村村委会",
                code: "038",
            },
            VillageCode {
                name: "洪北村村委会",
                code: "039",
            },
            VillageCode {
                name: "洪庙村村委会",
                code: "040",
            },
            VillageCode {
                name: "洪西村村委会",
                code: "041",
            },
            VillageCode {
                name: "洪南村村委会",
                code: "042",
            },
            VillageCode {
                name: "协新村村委会",
                code: "043",
            },
            VillageCode {
                name: "朱新村村委会",
                code: "044",
            },
            VillageCode {
                name: "集贤村村委会",
                code: "045",
            },
            VillageCode {
                name: "陆家桥村村委会",
                code: "046",
            },
            VillageCode {
                name: "红旗村村委会",
                code: "047",
            },
            VillageCode {
                name: "冯家村村委会",
                code: "048",
            },
            VillageCode {
                name: "幸福村村委会",
                code: "049",
            },
            VillageCode {
                name: "戴家村村委会",
                code: "050",
            },
            VillageCode {
                name: "分水墩村村委会",
                code: "051",
            },
            VillageCode {
                name: "二桥村村委会",
                code: "052",
            },
            VillageCode {
                name: "蔡家桥村村委会",
                code: "053",
            },
            VillageCode {
                name: "东新市村村委会",
                code: "054",
            },
            VillageCode {
                name: "南宋村村委会",
                code: "055",
            },
            VillageCode {
                name: "北宋村村委会",
                code: "056",
            },
        ],
    },
    TownCode {
        name: "庄行镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "庄行社区居委会",
                code: "001",
            },
            VillageCode {
                name: "邬桥居委会",
                code: "002",
            },
            VillageCode {
                name: "庄行新苑第一居委会",
                code: "003",
            },
            VillageCode {
                name: "丽水湾社区居委会",
                code: "004",
            },
            VillageCode {
                name: "沙港湾社区居委会",
                code: "005",
            },
            VillageCode {
                name: "牡丹里居委会",
                code: "006",
            },
            VillageCode {
                name: "新江南社区居委会",
                code: "007",
            },
            VillageCode {
                name: "吕桥村村委会",
                code: "008",
            },
            VillageCode {
                name: "东风村村委会",
                code: "009",
            },
            VillageCode {
                name: "长浜村村委会",
                code: "010",
            },
            VillageCode {
                name: "长堤村村委会",
                code: "011",
            },
            VillageCode {
                name: "潘垫村村委会",
                code: "012",
            },
            VillageCode {
                name: "新华村村委会",
                code: "013",
            },
            VillageCode {
                name: "杨溇村村委会",
                code: "014",
            },
            VillageCode {
                name: "芦泾村村委会",
                code: "015",
            },
            VillageCode {
                name: "存古村村委会",
                code: "016",
            },
            VillageCode {
                name: "西校村村委会",
                code: "017",
            },
            VillageCode {
                name: "汇安村村委会",
                code: "018",
            },
            VillageCode {
                name: "马路村村委会",
                code: "019",
            },
            VillageCode {
                name: "浦秀村村委会",
                code: "020",
            },
            VillageCode {
                name: "渔沥村村委会",
                code: "021",
            },
            VillageCode {
                name: "新叶村村委会",
                code: "022",
            },
            VillageCode {
                name: "张塘村村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "金汇镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "金汇镇居委会",
                code: "001",
            },
            VillageCode {
                name: "齐贤社区居委会",
                code: "002",
            },
            VillageCode {
                name: "泰日社区居委会",
                code: "003",
            },
            VillageCode {
                name: "泰绿社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金碧社区居委会",
                code: "005",
            },
            VillageCode {
                name: "恒苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "和苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "旺苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "泰顺社区居委会",
                code: "009",
            },
            VillageCode {
                name: "泰和居委会",
                code: "010",
            },
            VillageCode {
                name: "金贤居委会",
                code: "011",
            },
            VillageCode {
                name: "金聚居委会",
                code: "012",
            },
            VillageCode {
                name: "德苑居委会",
                code: "013",
            },
            VillageCode {
                name: "雅苑居委会",
                code: "014",
            },
            VillageCode {
                name: "汇苑居委会",
                code: "015",
            },
            VillageCode {
                name: "秀苑居委会",
                code: "016",
            },
            VillageCode {
                name: "贤苑居委会",
                code: "017",
            },
            VillageCode {
                name: "通苑居委会",
                code: "018",
            },
            VillageCode {
                name: "金中居委会",
                code: "019",
            },
            VillageCode {
                name: "丰苑社区居委会",
                code: "020",
            },
            VillageCode {
                name: "敬苑社区居委会",
                code: "021",
            },
            VillageCode {
                name: "江苑社区居委会",
                code: "022",
            },
            VillageCode {
                name: "金雍社区居委会",
                code: "023",
            },
            VillageCode {
                name: "金汇村村委会",
                code: "024",
            },
            VillageCode {
                name: "金星村村委会",
                code: "025",
            },
            VillageCode {
                name: "东星村村委会",
                code: "026",
            },
            VillageCode {
                name: "新强村村委会",
                code: "027",
            },
            VillageCode {
                name: "白沙村村委会",
                code: "028",
            },
            VillageCode {
                name: "行前村村委会",
                code: "029",
            },
            VillageCode {
                name: "百曲村村委会",
                code: "030",
            },
            VillageCode {
                name: "南行村村委会",
                code: "031",
            },
            VillageCode {
                name: "光辉村村委会",
                code: "032",
            },
            VillageCode {
                name: "明星村村委会",
                code: "033",
            },
            VillageCode {
                name: "北丁村村委会",
                code: "034",
            },
            VillageCode {
                name: "南陈村村委会",
                code: "035",
            },
            VillageCode {
                name: "乐善村村委会",
                code: "036",
            },
            VillageCode {
                name: "梁典村村委会",
                code: "037",
            },
            VillageCode {
                name: "资福村村委会",
                code: "038",
            },
            VillageCode {
                name: "梅园村村委会",
                code: "039",
            },
            VillageCode {
                name: "周家村村委会",
                code: "040",
            },
            VillageCode {
                name: "墩头村村委会",
                code: "041",
            },
        ],
    },
    TownCode {
        name: "四团镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "四团居委会",
                code: "001",
            },
            VillageCode {
                name: "平安居委会",
                code: "002",
            },
            VillageCode {
                name: "邵厂居委会",
                code: "003",
            },
            VillageCode {
                name: "平安第二社区居委会",
                code: "004",
            },
            VillageCode {
                name: "锦港佳苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "海港新苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "平乐居委会",
                code: "007",
            },
            VillageCode {
                name: "天鹏居委会",
                code: "008",
            },
            VillageCode {
                name: "平港居委会",
                code: "009",
            },
            VillageCode {
                name: "欣悦居委会",
                code: "010",
            },
            VillageCode {
                name: "三坎村村委会",
                code: "011",
            },
            VillageCode {
                name: "镇西村村委会",
                code: "012",
            },
            VillageCode {
                name: "大桥村村委会",
                code: "013",
            },
            VillageCode {
                name: "团南村村委会",
                code: "014",
            },
            VillageCode {
                name: "四团村村委会",
                code: "015",
            },
            VillageCode {
                name: "长堰村村委会",
                code: "016",
            },
            VillageCode {
                name: "小荡村村委会",
                code: "017",
            },
            VillageCode {
                name: "新桥村村委会",
                code: "018",
            },
            VillageCode {
                name: "夏家村村委会",
                code: "019",
            },
            VillageCode {
                name: "拾村村村委会",
                code: "020",
            },
            VillageCode {
                name: "渔墩村村委会",
                code: "021",
            },
            VillageCode {
                name: "渔洋村村委会",
                code: "022",
            },
            VillageCode {
                name: "向阳村村委会",
                code: "023",
            },
            VillageCode {
                name: "龙尖村村委会",
                code: "024",
            },
            VillageCode {
                name: "平海村村委会",
                code: "025",
            },
            VillageCode {
                name: "三团港村村委会",
                code: "026",
            },
            VillageCode {
                name: "五四村村委会",
                code: "027",
            },
            VillageCode {
                name: "新建村村委会",
                code: "028",
            },
            VillageCode {
                name: "前哨村村委会",
                code: "029",
            },
            VillageCode {
                name: "横桥村村委会",
                code: "030",
            },
            VillageCode {
                name: "杨家宅村村委会",
                code: "031",
            },
            VillageCode {
                name: "平南村村委会",
                code: "032",
            },
            VillageCode {
                name: "红庄村村委会",
                code: "033",
            },
            VillageCode {
                name: "农展村村委会",
                code: "034",
            },
            VillageCode {
                name: "民福村村委会",
                code: "035",
            },
            VillageCode {
                name: "邵靴村村委会",
                code: "036",
            },
        ],
    },
    TownCode {
        name: "青村镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "青村居委会",
                code: "001",
            },
            VillageCode {
                name: "钱桥居委会",
                code: "002",
            },
            VillageCode {
                name: "北唐社区居委会",
                code: "003",
            },
            VillageCode {
                name: "红星社区居委会",
                code: "004",
            },
            VillageCode {
                name: "青河居委会",
                code: "005",
            },
            VillageCode {
                name: "青韵居委会",
                code: "006",
            },
            VillageCode {
                name: "长丰居委会",
                code: "007",
            },
            VillageCode {
                name: "荣贤居委会",
                code: "008",
            },
            VillageCode {
                name: "青华居委会",
                code: "009",
            },
            VillageCode {
                name: "德泽社区居委会",
                code: "010",
            },
            VillageCode {
                name: "海欣社区居委会",
                code: "011",
            },
            VillageCode {
                name: "清泉社区居委会",
                code: "012",
            },
            VillageCode {
                name: "朱店村村委会",
                code: "013",
            },
            VillageCode {
                name: "钟家村村委会",
                code: "014",
            },
            VillageCode {
                name: "姚家村村委会",
                code: "015",
            },
            VillageCode {
                name: "和中村村委会",
                code: "016",
            },
            VillageCode {
                name: "岳和村村委会",
                code: "017",
            },
            VillageCode {
                name: "陶宅村村委会",
                code: "018",
            },
            VillageCode {
                name: "西吴村村委会",
                code: "019",
            },
            VillageCode {
                name: "北唐村村委会",
                code: "020",
            },
            VillageCode {
                name: "李窑村村委会",
                code: "021",
            },
            VillageCode {
                name: "钱忠村村委会",
                code: "022",
            },
            VillageCode {
                name: "桃园村村委会",
                code: "023",
            },
            VillageCode {
                name: "申隆二村村委会",
                code: "024",
            },
            VillageCode {
                name: "申隆一村村委会",
                code: "025",
            },
            VillageCode {
                name: "元通村村委会",
                code: "026",
            },
            VillageCode {
                name: "金王村村委会",
                code: "027",
            },
            VillageCode {
                name: "吴房村村委会",
                code: "028",
            },
            VillageCode {
                name: "新张村村委会",
                code: "029",
            },
            VillageCode {
                name: "花角村村委会",
                code: "030",
            },
            VillageCode {
                name: "石海村村委会",
                code: "031",
            },
            VillageCode {
                name: "湾张村村委会",
                code: "032",
            },
            VillageCode {
                name: "工农村村委会",
                code: "033",
            },
            VillageCode {
                name: "张弄村村委会",
                code: "034",
            },
            VillageCode {
                name: "解放村村委会",
                code: "035",
            },
            VillageCode {
                name: "南星村村委会",
                code: "036",
            },
        ],
    },
    TownCode {
        name: "柘林镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "柘林镇居委会",
                code: "001",
            },
            VillageCode {
                name: "新寺居委会",
                code: "002",
            },
            VillageCode {
                name: "胡桥居委会",
                code: "003",
            },
            VillageCode {
                name: "目华居委会",
                code: "004",
            },
            VillageCode {
                name: "冯桥居委会",
                code: "005",
            },
            VillageCode {
                name: "海畔新村社区居委会",
                code: "006",
            },
            VillageCode {
                name: "海韵社区居委会",
                code: "007",
            },
            VillageCode {
                name: "佳源社区居委会",
                code: "008",
            },
            VillageCode {
                name: "如意社区居委会",
                code: "009",
            },
            VillageCode {
                name: "如璟社区居委会",
                code: "010",
            },
            VillageCode {
                name: "营房村村委会",
                code: "011",
            },
            VillageCode {
                name: "夹路村村委会",
                code: "012",
            },
            VillageCode {
                name: "柘林村村委会",
                code: "013",
            },
            VillageCode {
                name: "华亭村村委会",
                code: "014",
            },
            VillageCode {
                name: "新塘村村委会",
                code: "015",
            },
            VillageCode {
                name: "新寺村村委会",
                code: "016",
            },
            VillageCode {
                name: "南胜村村委会",
                code: "017",
            },
            VillageCode {
                name: "金海村村委会",
                code: "018",
            },
            VillageCode {
                name: "海湾村村委会",
                code: "019",
            },
            VillageCode {
                name: "胡桥村村委会",
                code: "020",
            },
            VillageCode {
                name: "王家圩村村委会",
                code: "021",
            },
            VillageCode {
                name: "迎龙村村委会",
                code: "022",
            },
            VillageCode {
                name: "法华村村委会",
                code: "023",
            },
            VillageCode {
                name: "三桥村村委会",
                code: "024",
            },
            VillageCode {
                name: "兴园村村委会",
                code: "025",
            },
            VillageCode {
                name: "临海村村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "海湾镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "明城新村社区居委会",
                code: "001",
            },
            VillageCode {
                name: "星火一居委会",
                code: "002",
            },
            VillageCode {
                name: "星火二居委会",
                code: "003",
            },
            VillageCode {
                name: "燎原居委会",
                code: "004",
            },
            VillageCode {
                name: "中港居委会",
                code: "005",
            },
            VillageCode {
                name: "一兴居委会",
                code: "006",
            },
            VillageCode {
                name: "洪卫居委会",
                code: "007",
            },
            VillageCode {
                name: "星火世茂社区居委会",
                code: "008",
            },
            VillageCode {
                name: "云樱社区居委会",
                code: "009",
            },
            VillageCode {
                name: "海兴社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "海湾旅游区",
        code: "012",
        villages: &[
            VillageCode {
                name: "海湾居委会",
                code: "001",
            },
            VillageCode {
                name: "第二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "海棠社区居委会",
                code: "003",
            },
            VillageCode {
                name: "海尚居委会",
                code: "004",
            },
            VillageCode {
                name: "海墅社区居委会",
                code: "005",
            },
            VillageCode {
                name: "新港村村委会",
                code: "006",
            },
        ],
    },
];

static TOWNS_SJ_017: [TownCode; 21] = [
    TownCode {
        name: "城桥镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "南门居委会",
                code: "001",
            },
            VillageCode {
                name: "花园弄居委会",
                code: "002",
            },
            VillageCode {
                name: "新崇居委会",
                code: "003",
            },
            VillageCode {
                name: "城中居委会",
                code: "004",
            },
            VillageCode {
                name: "吴家弄居委会",
                code: "005",
            },
            VillageCode {
                name: "川心街居委会",
                code: "006",
            },
            VillageCode {
                name: "西泯沟居委会",
                code: "007",
            },
            VillageCode {
                name: "东河沿居委会",
                code: "008",
            },
            VillageCode {
                name: "北门社区居委会",
                code: "009",
            },
            VillageCode {
                name: "西门南村居委会",
                code: "010",
            },
            VillageCode {
                name: "西门北村居委会",
                code: "011",
            },
            VillageCode {
                name: "城西居委会",
                code: "012",
            },
            VillageCode {
                name: "东门新村居委会",
                code: "013",
            },
            VillageCode {
                name: "玉环新村居委会",
                code: "014",
            },
            VillageCode {
                name: "江山新村居委会",
                code: "015",
            },
            VillageCode {
                name: "学宫新村居委会",
                code: "016",
            },
            VillageCode {
                name: "湄洲新村居委会",
                code: "017",
            },
            VillageCode {
                name: "观潮新村居委会",
                code: "018",
            },
            VillageCode {
                name: "小港新村居委会",
                code: "019",
            },
            VillageCode {
                name: "永凤居委会",
                code: "020",
            },
            VillageCode {
                name: "怡祥居居委会",
                code: "021",
            },
            VillageCode {
                name: "城东居委会",
                code: "022",
            },
            VillageCode {
                name: "明珠花苑居委会",
                code: "023",
            },
            VillageCode {
                name: "金珠居委会",
                code: "024",
            },
            VillageCode {
                name: "海岛星城居委会",
                code: "025",
            },
            VillageCode {
                name: "金鳌山社区居委会",
                code: "026",
            },
            VillageCode {
                name: "江帆社区居委会",
                code: "027",
            },
            VillageCode {
                name: "金日社区居委会",
                code: "028",
            },
            VillageCode {
                name: "丁家桥社区居委会",
                code: "029",
            },
            VillageCode {
                name: "东江社区居委会",
                code: "030",
            },
            VillageCode {
                name: "海天社区居委会",
                code: "031",
            },
            VillageCode {
                name: "城桥村村委会",
                code: "032",
            },
            VillageCode {
                name: "马桥村村委会",
                code: "033",
            },
            VillageCode {
                name: "运粮村村委会",
                code: "034",
            },
            VillageCode {
                name: "新闸村村委会",
                code: "035",
            },
            VillageCode {
                name: "元六村村委会",
                code: "036",
            },
            VillageCode {
                name: "湾南村村委会",
                code: "037",
            },
            VillageCode {
                name: "利民村村委会",
                code: "038",
            },
            VillageCode {
                name: "老滧港渔村村委会",
                code: "039",
            },
            VillageCode {
                name: "推虾港村村委会",
                code: "040",
            },
            VillageCode {
                name: "鳌山村村委会",
                code: "041",
            },
            VillageCode {
                name: "侯南村村委会",
                code: "042",
            },
            VillageCode {
                name: "聚训村村委会",
                code: "043",
            },
            VillageCode {
                name: "山阳村村委会",
                code: "044",
            },
            VillageCode {
                name: "长兴村村委会",
                code: "045",
            },
        ],
    },
    TownCode {
        name: "堡镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "解放社区居委会",
                code: "001",
            },
            VillageCode {
                name: "电业社区居委会",
                code: "002",
            },
            VillageCode {
                name: "向阳社区居委会",
                code: "003",
            },
            VillageCode {
                name: "正大社区居委会",
                code: "004",
            },
            VillageCode {
                name: "光明社区居委会",
                code: "005",
            },
            VillageCode {
                name: "新港社区居委会",
                code: "006",
            },
            VillageCode {
                name: "玉屏社区居委会",
                code: "007",
            },
            VillageCode {
                name: "交通社区居委会",
                code: "008",
            },
            VillageCode {
                name: "虹宝社区居委会",
                code: "009",
            },
            VillageCode {
                name: "永和村村委会",
                code: "010",
            },
            VillageCode {
                name: "花园村村委会",
                code: "011",
            },
            VillageCode {
                name: "桃源村村委会",
                code: "012",
            },
            VillageCode {
                name: "财贸村村委会",
                code: "013",
            },
            VillageCode {
                name: "堡北村村委会",
                code: "014",
            },
            VillageCode {
                name: "堡港村村委会",
                code: "015",
            },
            VillageCode {
                name: "营房村村委会",
                code: "016",
            },
            VillageCode {
                name: "人民村村委会",
                code: "017",
            },
            VillageCode {
                name: "堡渔村村委会",
                code: "018",
            },
            VillageCode {
                name: "堡兴村村委会",
                code: "019",
            },
            VillageCode {
                name: "菜园村村委会",
                code: "020",
            },
            VillageCode {
                name: "南海村村委会",
                code: "021",
            },
            VillageCode {
                name: "小漾村村委会",
                code: "022",
            },
            VillageCode {
                name: "米行村村委会",
                code: "023",
            },
            VillageCode {
                name: "瀛南村村委会",
                code: "024",
            },
            VillageCode {
                name: "四滧村村委会",
                code: "025",
            },
            VillageCode {
                name: "彷徨村村委会",
                code: "026",
            },
            VillageCode {
                name: "五滧村村委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "新河镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "新南居委会",
                code: "001",
            },
            VillageCode {
                name: "新东居委会",
                code: "002",
            },
            VillageCode {
                name: "新晨居委会",
                code: "003",
            },
            VillageCode {
                name: "新源居委会",
                code: "004",
            },
            VillageCode {
                name: "新景居委会",
                code: "005",
            },
            VillageCode {
                name: "新舟居委会",
                code: "006",
            },
            VillageCode {
                name: "天新村村委会",
                code: "007",
            },
            VillageCode {
                name: "金桥村村委会",
                code: "008",
            },
            VillageCode {
                name: "石路村村委会",
                code: "009",
            },
            VillageCode {
                name: "新梅村村委会",
                code: "010",
            },
            VillageCode {
                name: "三烈村村委会",
                code: "011",
            },
            VillageCode {
                name: "兴教村村委会",
                code: "012",
            },
            VillageCode {
                name: "井亭村村委会",
                code: "013",
            },
            VillageCode {
                name: "新民村村委会",
                code: "014",
            },
            VillageCode {
                name: "永丰村村委会",
                code: "015",
            },
            VillageCode {
                name: "群英村村委会",
                code: "016",
            },
            VillageCode {
                name: "卫东村村委会",
                code: "017",
            },
            VillageCode {
                name: "新建村村委会",
                code: "018",
            },
            VillageCode {
                name: "民生村村委会",
                code: "019",
            },
            VillageCode {
                name: "新隆村村委会",
                code: "020",
            },
            VillageCode {
                name: "进化村村委会",
                code: "021",
            },
            VillageCode {
                name: "强民村村委会",
                code: "022",
            },
            VillageCode {
                name: "新光村村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "庙镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "庙镇居委会",
                code: "001",
            },
            VillageCode {
                name: "猛将庙居委会",
                code: "002",
            },
            VillageCode {
                name: "江口居委会",
                code: "003",
            },
            VillageCode {
                name: "爱民村村委会",
                code: "004",
            },
            VillageCode {
                name: "南星村村委会",
                code: "005",
            },
            VillageCode {
                name: "庙南村村委会",
                code: "006",
            },
            VillageCode {
                name: "庙西村村委会",
                code: "007",
            },
            VillageCode {
                name: "米洪村村委会",
                code: "008",
            },
            VillageCode {
                name: "庙中村村委会",
                code: "009",
            },
            VillageCode {
                name: "庙港村村委会",
                code: "010",
            },
            VillageCode {
                name: "白港村村委会",
                code: "011",
            },
            VillageCode {
                name: "万安村村委会",
                code: "012",
            },
            VillageCode {
                name: "鸽龙村村委会",
                code: "013",
            },
            VillageCode {
                name: "万北村村委会",
                code: "014",
            },
            VillageCode {
                name: "宏达村村委会",
                code: "015",
            },
            VillageCode {
                name: "民华村村委会",
                code: "016",
            },
            VillageCode {
                name: "江镇村村委会",
                code: "017",
            },
            VillageCode {
                name: "启瀛村村委会",
                code: "018",
            },
            VillageCode {
                name: "联益村村委会",
                code: "019",
            },
            VillageCode {
                name: "镇东村村委会",
                code: "020",
            },
            VillageCode {
                name: "通济村村委会",
                code: "021",
            },
            VillageCode {
                name: "和平村村委会",
                code: "022",
            },
            VillageCode {
                name: "合中村村委会",
                code: "023",
            },
            VillageCode {
                name: "小竖村村委会",
                code: "024",
            },
            VillageCode {
                name: "猛东村村委会",
                code: "025",
            },
            VillageCode {
                name: "窑桥村村委会",
                code: "026",
            },
            VillageCode {
                name: "周河村村委会",
                code: "027",
            },
            VillageCode {
                name: "猛西村村委会",
                code: "028",
            },
            VillageCode {
                name: "保东村村委会",
                code: "029",
            },
            VillageCode {
                name: "永乐村村委会",
                code: "030",
            },
            VillageCode {
                name: "保安村村委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "竖新镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "新乐居委会",
                code: "001",
            },
            VillageCode {
                name: "瀛兴居委会",
                code: "002",
            },
            VillageCode {
                name: "堡西村村委会",
                code: "003",
            },
            VillageCode {
                name: "东新村村委会",
                code: "004",
            },
            VillageCode {
                name: "油桥村村委会",
                code: "005",
            },
            VillageCode {
                name: "明强村村委会",
                code: "006",
            },
            VillageCode {
                name: "永兴村村委会",
                code: "007",
            },
            VillageCode {
                name: "竖西村村委会",
                code: "008",
            },
            VillageCode {
                name: "惠民村村委会",
                code: "009",
            },
            VillageCode {
                name: "竖河村村委会",
                code: "010",
            },
            VillageCode {
                name: "竖南村村委会",
                code: "011",
            },
            VillageCode {
                name: "新征村村委会",
                code: "012",
            },
            VillageCode {
                name: "仙桥村村委会",
                code: "013",
            },
            VillageCode {
                name: "大东村村委会",
                code: "014",
            },
            VillageCode {
                name: "响哃村村委会",
                code: "015",
            },
            VillageCode {
                name: "跃进村村委会",
                code: "016",
            },
            VillageCode {
                name: "春风村村委会",
                code: "017",
            },
            VillageCode {
                name: "时桥村村委会",
                code: "018",
            },
            VillageCode {
                name: "椿南村村委会",
                code: "019",
            },
            VillageCode {
                name: "大椿村村委会",
                code: "020",
            },
            VillageCode {
                name: "前哨村村委会",
                code: "021",
            },
            VillageCode {
                name: "育才村村委会",
                code: "022",
            },
            VillageCode {
                name: "前卫村村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "向化镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "向宏居委会",
                code: "001",
            },
            VillageCode {
                name: "六滧村村委会",
                code: "002",
            },
            VillageCode {
                name: "花仓村村委会",
                code: "003",
            },
            VillageCode {
                name: "南江村村委会",
                code: "004",
            },
            VillageCode {
                name: "春光村村委会",
                code: "005",
            },
            VillageCode {
                name: "阜康村村委会",
                code: "006",
            },
            VillageCode {
                name: "向化村村委会",
                code: "007",
            },
            VillageCode {
                name: "齐南村村委会",
                code: "008",
            },
            VillageCode {
                name: "北港村村委会",
                code: "009",
            },
            VillageCode {
                name: "卫星村村委会",
                code: "010",
            },
            VillageCode {
                name: "米新村村委会",
                code: "011",
            },
            VillageCode {
                name: "渔业村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "三星镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "三星居委会",
                code: "001",
            },
            VillageCode {
                name: "沈镇村村委会",
                code: "002",
            },
            VillageCode {
                name: "洪海村村委会",
                code: "003",
            },
            VillageCode {
                name: "东安村村委会",
                code: "004",
            },
            VillageCode {
                name: "平安村村委会",
                code: "005",
            },
            VillageCode {
                name: "协进村村委会",
                code: "006",
            },
            VillageCode {
                name: "海洪港村村委会",
                code: "007",
            },
            VillageCode {
                name: "纯阳村村委会",
                code: "008",
            },
            VillageCode {
                name: "南协村村委会",
                code: "009",
            },
            VillageCode {
                name: "育德村村委会",
                code: "010",
            },
            VillageCode {
                name: "育新村村委会",
                code: "011",
            },
            VillageCode {
                name: "三协村村委会",
                code: "012",
            },
            VillageCode {
                name: "西新村村委会",
                code: "013",
            },
            VillageCode {
                name: "邻江村村委会",
                code: "014",
            },
            VillageCode {
                name: "南桥村村委会",
                code: "015",
            },
            VillageCode {
                name: "新安村村委会",
                code: "016",
            },
            VillageCode {
                name: "永安村村委会",
                code: "017",
            },
            VillageCode {
                name: "大平村村委会",
                code: "018",
            },
            VillageCode {
                name: "海中村村委会",
                code: "019",
            },
            VillageCode {
                name: "北桥村村委会",
                code: "020",
            },
            VillageCode {
                name: "海滨村村委会",
                code: "021",
            },
            VillageCode {
                name: "海安村村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "港沿镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "沿中居委会",
                code: "001",
            },
            VillageCode {
                name: "建中村村委会",
                code: "002",
            },
            VillageCode {
                name: "建华村村委会",
                code: "003",
            },
            VillageCode {
                name: "港沿村村委会",
                code: "004",
            },
            VillageCode {
                name: "齐力村村委会",
                code: "005",
            },
            VillageCode {
                name: "跃马村村委会",
                code: "006",
            },
            VillageCode {
                name: "骏马村村委会",
                code: "007",
            },
            VillageCode {
                name: "富国村村委会",
                code: "008",
            },
            VillageCode {
                name: "富强村村委会",
                code: "009",
            },
            VillageCode {
                name: "惠中村村委会",
                code: "010",
            },
            VillageCode {
                name: "富军村村委会",
                code: "011",
            },
            VillageCode {
                name: "齐成村村委会",
                code: "012",
            },
            VillageCode {
                name: "惠军村村委会",
                code: "013",
            },
            VillageCode {
                name: "梅园村村委会",
                code: "014",
            },
            VillageCode {
                name: "漾滨村村委会",
                code: "015",
            },
            VillageCode {
                name: "合东村村委会",
                code: "016",
            },
            VillageCode {
                name: "合兴村村委会",
                code: "017",
            },
            VillageCode {
                name: "同滧村村委会",
                code: "018",
            },
            VillageCode {
                name: "鲁东村村委会",
                code: "019",
            },
            VillageCode {
                name: "鲁玙村村委会",
                code: "020",
            },
            VillageCode {
                name: "园艺村村委会",
                code: "021",
            },
            VillageCode {
                name: "同心村村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "中兴镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "广福居委会",
                code: "001",
            },
            VillageCode {
                name: "七滧村村委会",
                code: "002",
            },
            VillageCode {
                name: "爱国村村委会",
                code: "003",
            },
            VillageCode {
                name: "红星村村委会",
                code: "004",
            },
            VillageCode {
                name: "滧中村村委会",
                code: "005",
            },
            VillageCode {
                name: "中兴村村委会",
                code: "006",
            },
            VillageCode {
                name: "永南村村委会",
                code: "007",
            },
            VillageCode {
                name: "胜利村村委会",
                code: "008",
            },
            VillageCode {
                name: "北兴村村委会",
                code: "009",
            },
            VillageCode {
                name: "汲浜村村委会",
                code: "010",
            },
            VillageCode {
                name: "永隆村村委会",
                code: "011",
            },
            VillageCode {
                name: "富圩村村委会",
                code: "012",
            },
            VillageCode {
                name: "开港村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "陈家镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "裕弘居委会",
                code: "001",
            },
            VillageCode {
                name: "瀛陈居委会",
                code: "002",
            },
            VillageCode {
                name: "裕鸿佳苑居委会",
                code: "003",
            },
            VillageCode {
                name: "裕鸿佳苑第三社区居委会",
                code: "004",
            },
            VillageCode {
                name: "裕鸿佳苑第二社区居委会",
                code: "005",
            },
            VillageCode {
                name: "裕鸿佳苑第四社区居委会",
                code: "006",
            },
            VillageCode {
                name: "裕鸿佳苑第七社区居委会",
                code: "007",
            },
            VillageCode {
                name: "裕鸿佳苑第八社区居委会",
                code: "008",
            },
            VillageCode {
                name: "裕鸿佳苑第五社区居委会",
                code: "009",
            },
            VillageCode {
                name: "裕鸿佳苑第六社区居委会",
                code: "010",
            },
            VillageCode {
                name: "立新村村委会",
                code: "011",
            },
            VillageCode {
                name: "新桥村村委会",
                code: "012",
            },
            VillageCode {
                name: "铁塔村村委会",
                code: "013",
            },
            VillageCode {
                name: "东海村村委会",
                code: "014",
            },
            VillageCode {
                name: "朝阳村村委会",
                code: "015",
            },
            VillageCode {
                name: "陈南村村委会",
                code: "016",
            },
            VillageCode {
                name: "瀛东村村委会",
                code: "017",
            },
            VillageCode {
                name: "陈西村村委会",
                code: "018",
            },
            VillageCode {
                name: "八滧村村委会",
                code: "019",
            },
            VillageCode {
                name: "协隆村村委会",
                code: "020",
            },
            VillageCode {
                name: "奚家港村村委会",
                code: "021",
            },
            VillageCode {
                name: "花漂村村委会",
                code: "022",
            },
            VillageCode {
                name: "晨光村村委会",
                code: "023",
            },
            VillageCode {
                name: "德云村村委会",
                code: "024",
            },
            VillageCode {
                name: "裕丰村村委会",
                code: "025",
            },
            VillageCode {
                name: "先锋村村委会",
                code: "026",
            },
            VillageCode {
                name: "鸿田村村委会",
                code: "027",
            },
            VillageCode {
                name: "裕北村村委会",
                code: "028",
            },
            VillageCode {
                name: "展宏村村委会",
                code: "029",
            },
            VillageCode {
                name: "裕安村村委会",
                code: "030",
            },
            VillageCode {
                name: "裕西村村委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "绿华镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "新绿居委会",
                code: "001",
            },
            VillageCode {
                name: "绿湖村村委会",
                code: "002",
            },
            VillageCode {
                name: "绿港村村委会",
                code: "003",
            },
            VillageCode {
                name: "华星村村委会",
                code: "004",
            },
            VillageCode {
                name: "华荣村村委会",
                code: "005",
            },
            VillageCode {
                name: "绿园村村委会",
                code: "006",
            },
            VillageCode {
                name: "华西村村委会",
                code: "007",
            },
            VillageCode {
                name: "华渔村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "港西镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "双津村村委会",
                code: "001",
            },
            VillageCode {
                name: "北双村村委会",
                code: "002",
            },
            VillageCode {
                name: "盘西村村委会",
                code: "003",
            },
            VillageCode {
                name: "协西村村委会",
                code: "004",
            },
            VillageCode {
                name: "协北村村委会",
                code: "005",
            },
            VillageCode {
                name: "新港村村委会",
                code: "006",
            },
            VillageCode {
                name: "协兴村村委会",
                code: "007",
            },
            VillageCode {
                name: "团结村村委会",
                code: "008",
            },
            VillageCode {
                name: "富民村村委会",
                code: "009",
            },
            VillageCode {
                name: "排衙村村委会",
                code: "010",
            },
            VillageCode {
                name: "北闸村村委会",
                code: "011",
            },
            VillageCode {
                name: "静南村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "建设镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "富安村村委会",
                code: "001",
            },
            VillageCode {
                name: "虹桥村村委会",
                code: "002",
            },
            VillageCode {
                name: "滧东村村委会",
                code: "003",
            },
            VillageCode {
                name: "建垦村村委会",
                code: "004",
            },
            VillageCode {
                name: "运南村村委会",
                code: "005",
            },
            VillageCode {
                name: "白钥村村委会",
                code: "006",
            },
            VillageCode {
                name: "三星村村委会",
                code: "007",
            },
            VillageCode {
                name: "界东村村委会",
                code: "008",
            },
            VillageCode {
                name: "浜东村村委会",
                code: "009",
            },
            VillageCode {
                name: "建设村村委会",
                code: "010",
            },
            VillageCode {
                name: "蟠南村村委会",
                code: "011",
            },
            VillageCode {
                name: "大同村村委会",
                code: "012",
            },
            VillageCode {
                name: "浜西村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "新海镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "跃进居委会",
                code: "001",
            },
            VillageCode {
                name: "新海居委会",
                code: "002",
            },
            VillageCode {
                name: "新海二村居委会",
                code: "003",
            },
            VillageCode {
                name: "红星居委会",
                code: "004",
            },
            VillageCode {
                name: "长征居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "东平镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "东风新村居委会",
                code: "001",
            },
            VillageCode {
                name: "风伟新村居委会",
                code: "002",
            },
            VillageCode {
                name: "桂林新村居委会",
                code: "003",
            },
            VillageCode {
                name: "长江新村居委会",
                code: "004",
            },
            VillageCode {
                name: "前进新村居委会",
                code: "005",
            },
            VillageCode {
                name: "前哨新村居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "长兴镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "凤凰佳苑居委会",
                code: "001",
            },
            VillageCode {
                name: "清水苑居委会",
                code: "002",
            },
            VillageCode {
                name: "前卫新村居委会",
                code: "003",
            },
            VillageCode {
                name: "滨江苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "凤辰乐苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "鹭岛华庭社区居委会",
                code: "006",
            },
            VillageCode {
                name: "长兴家园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "长兴新苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "临江景苑居委会",
                code: "009",
            },
            VillageCode {
                name: "恒河新苑居委会",
                code: "010",
            },
            VillageCode {
                name: "渔港佳苑居委会",
                code: "011",
            },
            VillageCode {
                name: "长兴璟苑社区居委会",
                code: "012",
            },
            VillageCode {
                name: "新建村村委会",
                code: "013",
            },
            VillageCode {
                name: "合心村村委会",
                code: "014",
            },
            VillageCode {
                name: "鼎丰村村委会",
                code: "015",
            },
            VillageCode {
                name: "大兴村村委会",
                code: "016",
            },
            VillageCode {
                name: "农建村村委会",
                code: "017",
            },
            VillageCode {
                name: "圆东村村委会",
                code: "018",
            },
            VillageCode {
                name: "同心村村委会",
                code: "019",
            },
            VillageCode {
                name: "庆丰村村委会",
                code: "020",
            },
            VillageCode {
                name: "长明村村委会",
                code: "021",
            },
            VillageCode {
                name: "丰产村村委会",
                code: "022",
            },
            VillageCode {
                name: "新港村村委会",
                code: "023",
            },
            VillageCode {
                name: "先进村村委会",
                code: "024",
            },
            VillageCode {
                name: "北兴村村委会",
                code: "025",
            },
            VillageCode {
                name: "红星村村委会",
                code: "026",
            },
            VillageCode {
                name: "团结村村委会",
                code: "027",
            },
            VillageCode {
                name: "建新村村委会",
                code: "028",
            },
            VillageCode {
                name: "石沙村村委会",
                code: "029",
            },
            VillageCode {
                name: "先丰村村委会",
                code: "030",
            },
            VillageCode {
                name: "光荣村村委会",
                code: "031",
            },
            VillageCode {
                name: "长征村村委会",
                code: "032",
            },
            VillageCode {
                name: "潘石村村委会",
                code: "033",
            },
            VillageCode {
                name: "创建村村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "新村乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "新中村村委会",
                code: "001",
            },
            VillageCode {
                name: "新卫村村委会",
                code: "002",
            },
            VillageCode {
                name: "新乐村村委会",
                code: "003",
            },
            VillageCode {
                name: "新浜村村委会",
                code: "004",
            },
            VillageCode {
                name: "新国村村委会",
                code: "005",
            },
            VillageCode {
                name: "新洲村村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "横沙乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "新民居委会",
                code: "001",
            },
            VillageCode {
                name: "兴胜村村委会",
                code: "002",
            },
            VillageCode {
                name: "增产村村委会",
                code: "003",
            },
            VillageCode {
                name: "公平村村委会",
                code: "004",
            },
            VillageCode {
                name: "红旗村村委会",
                code: "005",
            },
            VillageCode {
                name: "东浜村村委会",
                code: "006",
            },
            VillageCode {
                name: "新北村村委会",
                code: "007",
            },
            VillageCode {
                name: "兴隆村村委会",
                code: "008",
            },
            VillageCode {
                name: "丰乐村村委会",
                code: "009",
            },
            VillageCode {
                name: "新春村村委会",
                code: "010",
            },
            VillageCode {
                name: "东兴村村委会",
                code: "011",
            },
            VillageCode {
                name: "新联村村委会",
                code: "012",
            },
            VillageCode {
                name: "惠丰村村委会",
                code: "013",
            },
            VillageCode {
                name: "新永村村委会",
                code: "014",
            },
            VillageCode {
                name: "民建村村委会",
                code: "015",
            },
            VillageCode {
                name: "民生村村委会",
                code: "016",
            },
            VillageCode {
                name: "永发村村委会",
                code: "017",
            },
            VillageCode {
                name: "富民村村委会",
                code: "018",
            },
            VillageCode {
                name: "永胜村村委会",
                code: "019",
            },
            VillageCode {
                name: "东海村村委会",
                code: "020",
            },
            VillageCode {
                name: "民星村村委会",
                code: "021",
            },
            VillageCode {
                name: "江海村村委会",
                code: "022",
            },
            VillageCode {
                name: "民永村村委会",
                code: "023",
            },
            VillageCode {
                name: "民东村村委会",
                code: "024",
            },
            VillageCode {
                name: "海鸿村村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "前卫农场",
        code: "019",
        villages: &[VillageCode {
            name: "前卫农场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "东平林场",
        code: "020",
        villages: &[VillageCode {
            name: "东平林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "上实现代农业园区",
        code: "021",
        villages: &[VillageCode {
            name: "上实现代农业园区虚拟社区",
            code: "001",
        }],
    },
];

pub const CITIES_SJ: [CityCode; 18] = [
    CityCode {
        name: "省辖市",
        code: "000",
        towns: &[],
    },
    CityCode {
        name: "上海市",
        code: "001",
        towns: &TOWNS_SJ_001,
    },
    CityCode {
        name: "黄浦市",
        code: "002",
        towns: &TOWNS_SJ_002,
    },
    CityCode {
        name: "徐汇市",
        code: "003",
        towns: &TOWNS_SJ_003,
    },
    CityCode {
        name: "长宁市",
        code: "004",
        towns: &TOWNS_SJ_004,
    },
    CityCode {
        name: "静安市",
        code: "005",
        towns: &TOWNS_SJ_005,
    },
    CityCode {
        name: "普陀市",
        code: "006",
        towns: &TOWNS_SJ_006,
    },
    CityCode {
        name: "虹口市",
        code: "007",
        towns: &TOWNS_SJ_007,
    },
    CityCode {
        name: "杨浦市",
        code: "008",
        towns: &TOWNS_SJ_008,
    },
    CityCode {
        name: "闵行市",
        code: "009",
        towns: &TOWNS_SJ_009,
    },
    CityCode {
        name: "宝山市",
        code: "010",
        towns: &TOWNS_SJ_010,
    },
    CityCode {
        name: "嘉定市",
        code: "011",
        towns: &TOWNS_SJ_011,
    },
    CityCode {
        name: "浦东新市",
        code: "012",
        towns: &TOWNS_SJ_012,
    },
    CityCode {
        name: "金山市",
        code: "013",
        towns: &TOWNS_SJ_013,
    },
    CityCode {
        name: "松江市",
        code: "014",
        towns: &TOWNS_SJ_014,
    },
    CityCode {
        name: "青浦市",
        code: "015",
        towns: &TOWNS_SJ_015,
    },
    CityCode {
        name: "奉贤市",
        code: "016",
        towns: &TOWNS_SJ_016,
    },
    CityCode {
        name: "崇明市",
        code: "017",
        towns: &TOWNS_SJ_017,
    },
];
