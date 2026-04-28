use super::{CityCode, TownCode, VillageCode};

static TOWNS_BP_001: [TownCode; 17] = [
    TownCode {
        name: "东华门街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "多福巷社区居委会",
                code: "001",
            },
            VillageCode {
                name: "银闸社区居委会",
                code: "002",
            },
            VillageCode {
                name: "东厂社区居委会",
                code: "003",
            },
            VillageCode {
                name: "智德社区居委会",
                code: "004",
            },
            VillageCode {
                name: "南池子社区居委会",
                code: "005",
            },
            VillageCode {
                name: "灯市口社区居委会",
                code: "006",
            },
            VillageCode {
                name: "正义路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "台基厂社区居委会",
                code: "008",
            },
            VillageCode {
                name: "韶九社区居委会",
                code: "009",
            },
            VillageCode {
                name: "王府井社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "景山街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "隆福寺社区居委会",
                code: "001",
            },
            VillageCode {
                name: "吉祥社区居委会",
                code: "002",
            },
            VillageCode {
                name: "黄化门社区居委会",
                code: "003",
            },
            VillageCode {
                name: "钟鼓社区居委会",
                code: "004",
            },
            VillageCode {
                name: "魏家社区居委会",
                code: "005",
            },
            VillageCode {
                name: "汪芝麻社区居委会",
                code: "006",
            },
            VillageCode {
                name: "景山东街社区居委会",
                code: "007",
            },
            VillageCode {
                name: "皇城根北街社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "交道口街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "交东社区居委会",
                code: "001",
            },
            VillageCode {
                name: "福祥社区居委会",
                code: "002",
            },
            VillageCode {
                name: "大兴社区居委会",
                code: "003",
            },
            VillageCode {
                name: "府学社区居委会",
                code: "004",
            },
            VillageCode {
                name: "鼓楼苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "菊儿社区居委会",
                code: "006",
            },
            VillageCode {
                name: "南锣鼓巷社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "安定门街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "交北头条社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北锣鼓巷社区居委会",
                code: "002",
            },
            VillageCode {
                name: "国子监社区居委会",
                code: "003",
            },
            VillageCode {
                name: "钟楼湾社区居委会",
                code: "004",
            },
            VillageCode {
                name: "宝钞南社区居委会",
                code: "005",
            },
            VillageCode {
                name: "五道营社区居委会",
                code: "006",
            },
            VillageCode {
                name: "分司厅社区居委会",
                code: "007",
            },
            VillageCode {
                name: "国旺社区居委会",
                code: "008",
            },
            VillageCode {
                name: "花园社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "北新桥街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "海运仓社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北新仓社区居委会",
                code: "002",
            },
            VillageCode {
                name: "门楼社区居委会",
                code: "003",
            },
            VillageCode {
                name: "民安社区居委会",
                code: "004",
            },
            VillageCode {
                name: "九道湾社区居委会",
                code: "005",
            },
            VillageCode {
                name: "北官厅社区居委会",
                code: "006",
            },
            VillageCode {
                name: "青龙社区居委会",
                code: "007",
            },
            VillageCode {
                name: "小菊社区居委会",
                code: "008",
            },
            VillageCode {
                name: "草园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "前永康社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "东四街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "南门仓社区居委会",
                code: "001",
            },
            VillageCode {
                name: "二条社区居委会",
                code: "002",
            },
            VillageCode {
                name: "六条社区居委会",
                code: "003",
            },
            VillageCode {
                name: "豆瓣社区居委会",
                code: "004",
            },
            VillageCode {
                name: "八条社区居委会",
                code: "005",
            },
            VillageCode {
                name: "总院社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "朝阳门街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "史家社区居委会",
                code: "001",
            },
            VillageCode {
                name: "内务社区居委会",
                code: "002",
            },
            VillageCode {
                name: "演乐社区居委会",
                code: "003",
            },
            VillageCode {
                name: "礼士社区居委会",
                code: "004",
            },
            VillageCode {
                name: "朝内头条社区居委会",
                code: "005",
            },
            VillageCode {
                name: "朝西社区居委会",
                code: "006",
            },
            VillageCode {
                name: "竹杆社区居委会",
                code: "007",
            },
            VillageCode {
                name: "大方家社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新鲜社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "建国门街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "赵家楼社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西总布社区居委会",
                code: "002",
            },
            VillageCode {
                name: "大雅宝社区居委会",
                code: "003",
            },
            VillageCode {
                name: "苏州社区居委会",
                code: "004",
            },
            VillageCode {
                name: "外交部街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "站东社区居委会",
                code: "006",
            },
            VillageCode {
                name: "金宝街北社区居委会",
                code: "007",
            },
            VillageCode {
                name: "东总布社区居委会",
                code: "008",
            },
            VillageCode {
                name: "崇内社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "东直门街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "胡家园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新中街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "清水苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新中西里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "十字坡社区居委会",
                code: "005",
            },
            VillageCode {
                name: "东外大街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "东环社区居委会",
                code: "007",
            },
            VillageCode {
                name: "香河园北里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "工人体育馆社区居委会",
                code: "009",
            },
            VillageCode {
                name: "东外大街北社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "和平里街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "民旺社区居委会",
                code: "001",
            },
            VillageCode {
                name: "安德路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "二区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "七区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "化工社区居委会",
                code: "005",
            },
            VillageCode {
                name: "安德里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "兴化社区居委会",
                code: "007",
            },
            VillageCode {
                name: "人定湖社区居委会",
                code: "008",
            },
            VillageCode {
                name: "小黄庄社区居委会",
                code: "009",
            },
            VillageCode {
                name: "总政社区居委会",
                code: "010",
            },
            VillageCode {
                name: "安贞苑社区居委会",
                code: "011",
            },
            VillageCode {
                name: "地坛社区居委会",
                code: "012",
            },
            VillageCode {
                name: "黄寺社区居委会",
                code: "013",
            },
            VillageCode {
                name: "新建路社区居委会",
                code: "014",
            },
            VillageCode {
                name: "东河沿社区居委会",
                code: "015",
            },
            VillageCode {
                name: "西河沿社区居委会",
                code: "016",
            },
            VillageCode {
                name: "青年湖社区居委会",
                code: "017",
            },
            VillageCode {
                name: "和平里社区居委会",
                code: "018",
            },
            VillageCode {
                name: "交林社区居委会",
                code: "019",
            },
            VillageCode {
                name: "上龙社区居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "前门街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "前门东大街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大江社区居委会",
                code: "002",
            },
            VillageCode {
                name: "草厂社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "崇文门外街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "兴隆都市馨园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新世界家园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "崇文门东大街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "崇文门西大街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西花市南里东区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "西花市南里西区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新怡家园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西花市南里南区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "国瑞城西区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "国瑞城中区社区居委会",
                code: "010",
            },
            VillageCode {
                name: "国瑞城东区社区居委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "东花市街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "东花市北里西区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东花市北里东区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "花市枣苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东花市南里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "广渠门北里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "广渠门外南里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "忠实里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "东花市南里东区社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "龙潭街道",
        code: "014",
        villages: &[
            VillageCode {
                name: "安化楼社区居委会",
                code: "001",
            },
            VillageCode {
                name: "夕照寺社区居委会",
                code: "002",
            },
            VillageCode {
                name: "板厂南里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "华城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "左安漪园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "左安浦园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "幸福社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新家园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "龙潭北里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "光明社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "体育馆路街道",
        code: "015",
        villages: &[
            VillageCode {
                name: "东厅社区居委会",
                code: "001",
            },
            VillageCode {
                name: "葱店社区居委会",
                code: "002",
            },
            VillageCode {
                name: "法华南里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "体育总局社区居委会",
                code: "004",
            },
            VillageCode {
                name: "南岗子社区居委会",
                code: "005",
            },
            VillageCode {
                name: "长青园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "四块玉社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西唐社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "天坛街道",
        code: "016",
        villages: &[
            VillageCode {
                name: "东晓市社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西园子社区居委会",
                code: "002",
            },
            VillageCode {
                name: "永定门内社区居委会",
                code: "003",
            },
            VillageCode {
                name: "泰元社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金鱼池西社区居委会",
                code: "005",
            },
            VillageCode {
                name: "金鱼池社区居委会",
                code: "006",
            },
            VillageCode {
                name: "祈谷社区居委会",
                code: "007",
            },
            VillageCode {
                name: "昭亨社区居委会",
                code: "008",
            },
            VillageCode {
                name: "金台社区居委会",
                code: "009",
            },
            VillageCode {
                name: "广利社区居委会",
                code: "010",
            },
            VillageCode {
                name: "精忠社区居委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "永定门外街道",
        code: "017",
        villages: &[
            VillageCode {
                name: "彭庄社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中海紫御社区居委会",
                code: "002",
            },
            VillageCode {
                name: "百荣嘉园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "管村社区居委会",
                code: "004",
            },
            VillageCode {
                name: "桃园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "民主北街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "琉璃井社区居委会",
                code: "007",
            },
            VillageCode {
                name: "桃杨路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "杨家园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "李村社区居委会",
                code: "010",
            },
            VillageCode {
                name: "宝华里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "定安里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "安乐林社区居委会",
                code: "013",
            },
            VillageCode {
                name: "景泰社区居委会",
                code: "014",
            },
            VillageCode {
                name: "永铁苑社区居委会",
                code: "015",
            },
            VillageCode {
                name: "天天家园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "富莱茵社区居委会",
                code: "017",
            },
            VillageCode {
                name: "革新西里社区居委会",
                code: "018",
            },
            VillageCode {
                name: "革新里社区居委会",
                code: "019",
            },
            VillageCode {
                name: "望坛新苑社区居委会",
                code: "020",
            },
        ],
    },
];

static TOWNS_BP_002: [TownCode; 15] = [
    TownCode {
        name: "西长安街街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "府右街南社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西交民巷社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北新华街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "六部口社区居委会",
                code: "004",
            },
            VillageCode {
                name: "和平门社区居委会",
                code: "005",
            },
            VillageCode {
                name: "钟声社区居委会",
                code: "006",
            },
            VillageCode {
                name: "太仆寺街社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西黄城根南街社区居委会",
                code: "008",
            },
            VillageCode {
                name: "义达里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "西单北社区居委会",
                code: "010",
            },
            VillageCode {
                name: "未英社区居委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "新街口街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "西里二区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西里一区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西里四区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "北草厂社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西里三区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "玉桃园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "西四北头条社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西四北三条社区居委会",
                code: "008",
            },
            VillageCode {
                name: "西四北六条社区居委会",
                code: "009",
            },
            VillageCode {
                name: "育德社区居委会",
                code: "010",
            },
            VillageCode {
                name: "前公用社区居委会",
                code: "011",
            },
            VillageCode {
                name: "半壁街社区居委会",
                code: "012",
            },
            VillageCode {
                name: "南小街社区居委会",
                code: "013",
            },
            VillageCode {
                name: "冠英园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "大觉社区居委会",
                code: "015",
            },
            VillageCode {
                name: "富国里社区居委会",
                code: "016",
            },
            VillageCode {
                name: "安平巷社区居委会",
                code: "017",
            },
            VillageCode {
                name: "官园社区居委会",
                code: "018",
            },
            VillageCode {
                name: "宫门口社区居委会",
                code: "019",
            },
            VillageCode {
                name: "北顺社区居委会",
                code: "020",
            },
            VillageCode {
                name: "中直社区居委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "月坛街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "月坛社区居委会",
                code: "001",
            },
            VillageCode {
                name: "社会路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "铁道部住宅区第三社区居委会",
                code: "003",
            },
            VillageCode {
                name: "三里河一区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "南沙沟社区居委会",
                code: "005",
            },
            VillageCode {
                name: "复兴门北大街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "铁道部住宅区第二、二社区居委会",
                code: "007",
            },
            VillageCode {
                name: "复兴门外大街甲7号院社区居委会",
                code: "008",
            },
            VillageCode {
                name: "三里河二区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "三里河三区第一社区居委会",
                code: "010",
            },
            VillageCode {
                name: "三里河三区第三社区居委会",
                code: "011",
            },
            VillageCode {
                name: "南礼士路社区居委会",
                code: "012",
            },
            VillageCode {
                name: "木樨地社区居委会",
                code: "013",
            },
            VillageCode {
                name: "复兴门外社区居委会",
                code: "014",
            },
            VillageCode {
                name: "全国总工会住宅区社区居委会",
                code: "015",
            },
            VillageCode {
                name: "白云观社区居委会",
                code: "016",
            },
            VillageCode {
                name: "真武庙社区居委会",
                code: "017",
            },
            VillageCode {
                name: "广电总局住宅区第二社区居委会",
                code: "018",
            },
            VillageCode {
                name: "西便门社区居委会",
                code: "019",
            },
            VillageCode {
                name: "铁道部住宅区第二、一社区居委会",
                code: "020",
            },
            VillageCode {
                name: "广电总局住宅区第一社区居委会",
                code: "021",
            },
            VillageCode {
                name: "汽车局河南社区居委会",
                code: "022",
            },
            VillageCode {
                name: "汽车局河北社区居委会",
                code: "023",
            },
            VillageCode {
                name: "公安住宅区社区居委会",
                code: "024",
            },
            VillageCode {
                name: "铁道部住宅区第四社区居委会",
                code: "025",
            },
            VillageCode {
                name: "三里河社区居委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "展览路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "文兴街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "朝阳庵社区居委会",
                code: "002",
            },
            VillageCode {
                name: "三塔社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新华南社区居委会",
                code: "004",
            },
            VillageCode {
                name: "百万庄东社区居委会",
                code: "005",
            },
            VillageCode {
                name: "百万庄西社区居委会",
                code: "006",
            },
            VillageCode {
                name: "车公庄社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新华东社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新华里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "榆树馆社区居委会",
                code: "010",
            },
            VillageCode {
                name: "德宝社区居委会",
                code: "011",
            },
            VillageCode {
                name: "团结社区居委会",
                code: "012",
            },
            VillageCode {
                name: "北营房西里社区居委会",
                code: "013",
            },
            VillageCode {
                name: "北营房东里社区居委会",
                code: "014",
            },
            VillageCode {
                name: "黄瓜园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "露园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "阜外西社区居委会",
                code: "017",
            },
            VillageCode {
                name: "洪茂沟社区居委会",
                code: "018",
            },
            VillageCode {
                name: "阜外东社区居委会",
                code: "019",
            },
            VillageCode {
                name: "南营房社区居委会",
                code: "020",
            },
            VillageCode {
                name: "万明园社区居委会",
                code: "021",
            },
            VillageCode {
                name: "滨河社区居委会",
                code: "022",
            },
            VillageCode {
                name: "扣钟社区居委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "德胜街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "安德路南社区居委会",
                code: "001",
            },
            VillageCode {
                name: "安德路北社区居委会",
                code: "002",
            },
            VillageCode {
                name: "德胜里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "德外大街西社区居委会",
                code: "004",
            },
            VillageCode {
                name: "新明家园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "新风中直社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新外大街北社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新康社区居委会",
                code: "008",
            },
            VillageCode {
                name: "马甸社区居委会",
                code: "009",
            },
            VillageCode {
                name: "人定湖西里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "黄寺大街西社区居委会",
                code: "011",
            },
            VillageCode {
                name: "双旗杆社区居委会",
                code: "012",
            },
            VillageCode {
                name: "北广社区居委会",
                code: "013",
            },
            VillageCode {
                name: "黄寺大街24号社区居委会",
                code: "014",
            },
            VillageCode {
                name: "裕中西里社区居委会",
                code: "015",
            },
            VillageCode {
                name: "裕中东里社区居委会",
                code: "016",
            },
            VillageCode {
                name: "阳光丽景社区居委会",
                code: "017",
            },
            VillageCode {
                name: "德外大街东社区居委会",
                code: "018",
            },
            VillageCode {
                name: "六铺炕北小街社区居委会",
                code: "019",
            },
            VillageCode {
                name: "六铺炕南小街居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "金融街街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "京畿道社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西太平街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "二龙路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东太平街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "温家街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "受水河社区居委会",
                code: "006",
            },
            VillageCode {
                name: "文昌社区居委会",
                code: "007",
            },
            VillageCode {
                name: "手帕社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新文化街社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新华社社区居委会",
                code: "010",
            },
            VillageCode {
                name: "中央音乐学院社区居委会",
                code: "011",
            },
            VillageCode {
                name: "教育部社区居委会",
                code: "012",
            },
            VillageCode {
                name: "民康社区居委会",
                code: "013",
            },
            VillageCode {
                name: "丰汇园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "宏汇园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "丰融园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "丰盛社区居委会",
                code: "017",
            },
            VillageCode {
                name: "大院社区居委会",
                code: "018",
            },
            VillageCode {
                name: "砖塔社区居委会",
                code: "019",
            },
            VillageCode {
                name: "华嘉社区居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "什刹海街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "西四北社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西什库社区居委会",
                code: "002",
            },
            VillageCode {
                name: "爱民街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "大红罗社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西巷社区居委会",
                code: "005",
            },
            VillageCode {
                name: "护国寺社区居委会",
                code: "006",
            },
            VillageCode {
                name: "前铁社区居委会",
                code: "007",
            },
            VillageCode {
                name: "柳荫街社区居委会",
                code: "008",
            },
            VillageCode {
                name: "兴华社区居委会",
                code: "009",
            },
            VillageCode {
                name: "松树街社区居委会",
                code: "010",
            },
            VillageCode {
                name: "白米社区居委会",
                code: "011",
            },
            VillageCode {
                name: "景山社区居委会",
                code: "012",
            },
            VillageCode {
                name: "米粮库社区居委会",
                code: "013",
            },
            VillageCode {
                name: "旧鼓楼社区居委会",
                code: "014",
            },
            VillageCode {
                name: "双寺社区居委会",
                code: "015",
            },
            VillageCode {
                name: "鼓西社区居委会",
                code: "016",
            },
            VillageCode {
                name: "后海社区居委会",
                code: "017",
            },
            VillageCode {
                name: "苇坑社区居委会",
                code: "018",
            },
            VillageCode {
                name: "后海西沿社区居委会",
                code: "019",
            },
            VillageCode {
                name: "西海社区居委会",
                code: "020",
            },
            VillageCode {
                name: "四环社区居委会",
                code: "021",
            },
            VillageCode {
                name: "前海社区居委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "大栅栏街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "前门西河沿社区居委会",
                code: "001",
            },
            VillageCode {
                name: "延寿街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "三井社区居委会",
                code: "003",
            },
            VillageCode {
                name: "大栅栏西街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "石头社区居委会",
                code: "005",
            },
            VillageCode {
                name: "铁树斜街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "百顺社区居委会",
                code: "007",
            },
            VillageCode {
                name: "大安澜营社区居委会",
                code: "008",
            },
            VillageCode {
                name: "煤市街东社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "天桥街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "留学路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "香厂路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "天桥小区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "禄长街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "先农坛社区居委会",
                code: "005",
            },
            VillageCode {
                name: "永安路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "虎坊路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "太平街社区居委会",
                code: "008",
            },
            VillageCode {
                name: "福长街社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "椿树街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "宣武门外东大街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "琉璃厂西街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "椿树园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "四川营社区居委会",
                code: "004",
            },
            VillageCode {
                name: "梁家园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "香炉营社区居委会",
                code: "006",
            },
            VillageCode {
                name: "永光社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "陶然亭街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "米市社区居委会",
                code: "001",
            },
            VillageCode {
                name: "粉房琉璃街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "福州馆社区居委会",
                code: "003",
            },
            VillageCode {
                name: "黑窑厂社区居委会",
                code: "004",
            },
            VillageCode {
                name: "龙泉社区居委会",
                code: "005",
            },
            VillageCode {
                name: "红土店社区居委会",
                code: "006",
            },
            VillageCode {
                name: "南华里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新兴里居委会",
                code: "008",
            },
            VillageCode {
                name: "壹瓶社区居委会",
                code: "009",
            },
            VillageCode {
                name: "大吉巷社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "广安门内街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "西便门西里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西便门东里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "槐柏树街北里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "长椿街西社区居委会",
                code: "004",
            },
            VillageCode {
                name: "槐柏树街南里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "核桃园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "报国寺社区居委会",
                code: "007",
            },
            VillageCode {
                name: "上斜街社区居委会",
                code: "008",
            },
            VillageCode {
                name: "宣武门西大街社区居委会",
                code: "009",
            },
            VillageCode {
                name: "三庙街社区居委会",
                code: "010",
            },
            VillageCode {
                name: "康乐里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "广安东里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "大街东社区居委会",
                code: "013",
            },
            VillageCode {
                name: "老墙根社区居委会",
                code: "014",
            },
            VillageCode {
                name: "长椿街社区居委会",
                code: "015",
            },
            VillageCode {
                name: "西便门内社区居委会",
                code: "016",
            },
            VillageCode {
                name: "长椿里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "校场社区居委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "牛街街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "牛街东里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "春风社区居委会",
                code: "002",
            },
            VillageCode {
                name: "钢院社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南线阁社区居委会",
                code: "004",
            },
            VillageCode {
                name: "菜园北里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "枫桦社区居委会",
                code: "006",
            },
            VillageCode {
                name: "法源寺社区居委会",
                code: "007",
            },
            VillageCode {
                name: "白广路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "牛街西里一区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "牛街西里二区社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "白纸坊街道",
        code: "014",
        villages: &[
            VillageCode {
                name: "双槐里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "右北大街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "樱桃园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "崇效寺社区居委会",
                code: "004",
            },
            VillageCode {
                name: "菜园街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "建功北里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "建功南里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新安中里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新安南里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "右内西街社区居委会",
                code: "010",
            },
            VillageCode {
                name: "右内后身社区居委会",
                code: "011",
            },
            VillageCode {
                name: "光源里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "半步桥社区居委会",
                code: "013",
            },
            VillageCode {
                name: "自新路社区居委会",
                code: "014",
            },
            VillageCode {
                name: "里仁街社区居委会",
                code: "015",
            },
            VillageCode {
                name: "万博苑社区居委会",
                code: "016",
            },
            VillageCode {
                name: "清芷园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "平原里北区社区居委会",
                code: "018",
            },
            VillageCode {
                name: "平原里南区社区居委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "广安门外街道",
        code: "015",
        villages: &[
            VillageCode {
                name: "鸭子桥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "青年湖社区居委会",
                code: "002",
            },
            VillageCode {
                name: "椿树馆社区居委会",
                code: "003",
            },
            VillageCode {
                name: "白菜湾社区居委会",
                code: "004",
            },
            VillageCode {
                name: "车站东街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "车站西街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "车站西街十五号院社区居委会",
                code: "007",
            },
            VillageCode {
                name: "红居南街社区居委会",
                code: "008",
            },
            VillageCode {
                name: "红居街社区居委会",
                code: "009",
            },
            VillageCode {
                name: "手帕口南街社区居委会",
                code: "010",
            },
            VillageCode {
                name: "朗琴园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "红莲南里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "红莲中里社区居委会",
                code: "013",
            },
            VillageCode {
                name: "红莲北里社区居委会",
                code: "014",
            },
            VillageCode {
                name: "湾子街社区居委会",
                code: "015",
            },
            VillageCode {
                name: "马连道社区居委会",
                code: "016",
            },
            VillageCode {
                name: "三义里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "马连道中里社区居委会",
                code: "018",
            },
            VillageCode {
                name: "三义东里社区居委会",
                code: "019",
            },
            VillageCode {
                name: "莲花河社区居委会",
                code: "020",
            },
            VillageCode {
                name: "小马厂西社区居委会",
                code: "021",
            },
            VillageCode {
                name: "天宁寺北里社区居委会",
                code: "022",
            },
            VillageCode {
                name: "天宁寺南里社区居委会",
                code: "023",
            },
            VillageCode {
                name: "手帕口北街社区居委会",
                code: "024",
            },
            VillageCode {
                name: "二热社区居委会",
                code: "025",
            },
            VillageCode {
                name: "乐城社区居委会",
                code: "026",
            },
            VillageCode {
                name: "依莲轩社区居委会",
                code: "027",
            },
            VillageCode {
                name: "蝶翠华庭社区居委会",
                code: "028",
            },
            VillageCode {
                name: "中新佳园社区居委会",
                code: "029",
            },
            VillageCode {
                name: "茶马北街社区居委会",
                code: "030",
            },
            VillageCode {
                name: "茶马南街社区居委会",
                code: "031",
            },
            VillageCode {
                name: "广源社区居委会",
                code: "032",
            },
            VillageCode {
                name: "京铁和园社区居委会",
                code: "033",
            },
            VillageCode {
                name: "小马厂东社区居委会",
                code: "034",
            },
            VillageCode {
                name: "荣丰南社区居委会",
                code: "035",
            },
            VillageCode {
                name: "荣丰北社区居委会",
                code: "036",
            },
            VillageCode {
                name: "茶源社区居委会",
                code: "037",
            },
            VillageCode {
                name: "名苑社区居委会",
                code: "038",
            },
        ],
    },
];

static TOWNS_BP_003: [TownCode; 43] = [
    TownCode {
        name: "建外街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "南郎家园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北郎家园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "永安里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "光华里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "建国里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "秀水社区居委会",
                code: "006",
            },
            VillageCode {
                name: "北郎东社区居委会",
                code: "007",
            },
            VillageCode {
                name: "永安里东社区居委会",
                code: "008",
            },
            VillageCode {
                name: "大北家园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "月河社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "大望家园社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "光华东里社区居民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "朝外街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "体东社区居委会",
                code: "001",
            },
            VillageCode {
                name: "吉庆里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "吉祥里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "三丰里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "雅宝里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "天福园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "芳草地社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "呼家楼街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "金台里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "小庄社区居委会",
                code: "002",
            },
            VillageCode {
                name: "关东店北街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "核桃园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "呼家楼北社区居委会",
                code: "005",
            },
            VillageCode {
                name: "呼家楼南社区居委会",
                code: "006",
            },
            VillageCode {
                name: "金台社区居委会",
                code: "007",
            },
            VillageCode {
                name: "东大桥社区居委会",
                code: "008",
            },
            VillageCode {
                name: "关东店社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新街社区居委会",
                code: "010",
            },
            VillageCode {
                name: "金桐社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "金台路社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "农丰里社区居民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "三里屯街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "幸福一村社区居委会",
                code: "001",
            },
            VillageCode {
                name: "幸福二村社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北三里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东三里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "中三里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "白家庄西里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "中纺里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "南三里社区居民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "左家庄街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "新源里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "三源里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "顺源里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新源西里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "静安里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "左东里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "左南里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "左北里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "曙光里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "曙光凤凰城社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "曙光里西社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "新源南里社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "静安东里社区居民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "香河园街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "西坝河南里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西坝河西里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西坝河中里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "柳芳北里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "柳芳南里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "光熙门北里北社区居委会",
                code: "006",
            },
            VillageCode {
                name: "光熙门北里南社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西坝河东里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "光熙家园社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "和平街街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "胜古庄社区居委会",
                code: "001",
            },
            VillageCode {
                name: "樱花园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "和平东街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "砖角楼社区居委会",
                code: "004",
            },
            VillageCode {
                name: "和平家园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "十四区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "小黄庄社区居委会",
                code: "007",
            },
            VillageCode {
                name: "煤炭科技苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "胜古北社区居委会",
                code: "009",
            },
            VillageCode {
                name: "胜古南社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "和平家园北社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "和平西苑社区居民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "安贞街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "安贞里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "安贞西里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "安华里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "安华西里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "黄寺社区居委会",
                code: "005",
            },
            VillageCode {
                name: "裕民路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "涌溪社区居委会",
                code: "007",
            },
            VillageCode {
                name: "五路居社区居委会",
                code: "008",
            },
            VillageCode {
                name: "安外社区居委会",
                code: "009",
            },
            VillageCode {
                name: "外馆社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "亚运村街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "安慧里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "安慧里南社区居委会",
                code: "002",
            },
            VillageCode {
                name: "华严北里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "华严北里西社区居委会",
                code: "004",
            },
            VillageCode {
                name: "安翔里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "丝竹园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "北辰东路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "冬奥村社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "京民社区居委会",
                code: "009",
            },
            VillageCode {
                name: "祁家豁子社区居委会",
                code: "010",
            },
            VillageCode {
                name: "安慧里北社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "民族园社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "安苑北里社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "华亭社区居民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "小关街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "惠新苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "惠新北里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "高原街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "小关社区居委会",
                code: "004",
            },
            VillageCode {
                name: "惠新里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "小关东街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "惠新东街社区居委会",
                code: "007",
            },
            VillageCode {
                name: "惠新西街社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "惠新南里社区居民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "酒仙桥街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "电子球场社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "红霞路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "中北路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "大山子社区居委会",
                code: "006",
            },
            VillageCode {
                name: "高家园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "怡思苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "驼房营西里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "万红路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "银河湾社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "华信社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "红霞北社区居民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "麦子店街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "枣营南里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "枣营北里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "霞光里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "农展南里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "朝阳公园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "枣营社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "亮马社区居民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "团结湖街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "一二条社区居委会",
                code: "001",
            },
            VillageCode {
                name: "三四条社区居委会",
                code: "002",
            },
            VillageCode {
                name: "中路北社区居委会",
                code: "003",
            },
            VillageCode {
                name: "中路南社区居委会",
                code: "004",
            },
            VillageCode {
                name: "水碓子社区居委会",
                code: "005",
            },
            VillageCode {
                name: "南北里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "白家庄东里社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "三四条北社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "水碓子东路社区居民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "六里屯街道",
        code: "014",
        villages: &[
            VillageCode {
                name: "十里堡北里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "八里庄北里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "八里庄南里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "晨光社区居委会",
                code: "004",
            },
            VillageCode {
                name: "道家园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "碧水园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "甜水园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "秀水园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "六里屯北里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "炫特家园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "甜水西园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "八里庄北里东社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "嘉福园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "延静寺社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "十福社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "晨光东社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "静水园社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "丽景社区居民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "八里庄街道",
        code: "015",
        villages: &[
            VillageCode {
                name: "十里堡社区居委会",
                code: "001",
            },
            VillageCode {
                name: "甘露园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "八里庄东里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "八里庄西里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "红庙社区居委会",
                code: "005",
            },
            VillageCode {
                name: "红庙北里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "延静里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "远洋天地家园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "城市华庭社区居委会",
                code: "009",
            },
            VillageCode {
                name: "朝阳无限社区居委会",
                code: "010",
            },
            VillageCode {
                name: "罗马嘉园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "华贸中心社区居委会",
                code: "012",
            },
            VillageCode {
                name: "十里堡南里社区居委会",
                code: "013",
            },
            VillageCode {
                name: "甘露园中里社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "十里堡东里社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "八里庄东里北社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "延静里东社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "红庙北里第二社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "红庙东社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "慈云寺北里社区居民委员会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "双井街道",
        code: "016",
        villages: &[
            VillageCode {
                name: "垂杨柳东里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "垂杨柳西里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "广和里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "双花园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "广外南社区居委会",
                code: "005",
            },
            VillageCode {
                name: "九龙社区居委会",
                code: "006",
            },
            VillageCode {
                name: "大望社区居委会",
                code: "007",
            },
            VillageCode {
                name: "广泉社区居委会",
                code: "008",
            },
            VillageCode {
                name: "富力社区居委会",
                code: "009",
            },
            VillageCode {
                name: "光环社区居委会",
                code: "010",
            },
            VillageCode {
                name: "九龙南社区居委会",
                code: "011",
            },
            VillageCode {
                name: "百子园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "和平村一社区居委会",
                code: "013",
            },
            VillageCode {
                name: "东柏街社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "九龙山社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "富力西社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "黄木厂社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "茂兴社区居民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "劲松街道",
        code: "017",
        villages: &[
            VillageCode {
                name: "劲松北社区居委会",
                code: "001",
            },
            VillageCode {
                name: "劲松东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "劲松中社区居委会",
                code: "003",
            },
            VillageCode {
                name: "劲松西社区居委会",
                code: "004",
            },
            VillageCode {
                name: "八棵杨社区居委会",
                code: "005",
            },
            VillageCode {
                name: "大郊亭社区居委会",
                code: "006",
            },
            VillageCode {
                name: "农光里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "农光里中社区居委会",
                code: "008",
            },
            VillageCode {
                name: "农光东里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "磨房北里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "百环社区居委会",
                code: "011",
            },
            VillageCode {
                name: "和谐雅园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "西大望路社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "首城社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "西大望路南社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "百环东社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "农光里南社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "劲松一区社区居民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "潘家园街道",
        code: "018",
        villages: &[
            VillageCode {
                name: "磨房南里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "武圣东里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "松榆东里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "松榆里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "松榆西里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "武圣农光社区居委会",
                code: "006",
            },
            VillageCode {
                name: "华威北里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "潘家园东里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "华威西里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "潘家园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "潘家园南里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "华威里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "松榆西里北社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "武圣西里社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "华威北里南社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "潘家园东里西社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "华威西里南社区居民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "垡头街道",
        code: "019",
        villages: &[
            VillageCode {
                name: "垡头一区社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "垡头二区社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "垡头三区社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "垡头东里社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "垡头西里社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "垡头北里社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "翠城馨园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "翠城雅园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "翠城趣园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "翠城盛园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "翠城熙园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "双合家园社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "翠城福园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "祈东家园社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "北焦家园社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "双美家园社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "祈安家园社区居民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "首都机场街道",
        code: "020",
        villages: &[
            VillageCode {
                name: "南路西里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南路东里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西平街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南平里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "机场工作区社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "南磨房地区",
        code: "021",
        villages: &[
            VillageCode {
                name: "东郊社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "百子湾西社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "紫南家园社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "平乐园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "欢乐谷社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "双龙南里社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "南新园社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "百子湾东社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "山水文园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "赛洛城社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "百子湾北社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "广百西路社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "世纪东方城社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "广华新城社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "美景东方社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "石门社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "双龙西社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "华侨城社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "远景社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "广华新城东社区居民委员会",
                code: "020",
            },
            VillageCode {
                name: "南新园西社区居民委员会",
                code: "021",
            },
            VillageCode {
                name: "广泰华亭社区居民委员会",
                code: "022",
            },
            VillageCode {
                name: "广泰华苑社区居民委员会",
                code: "023",
            },
            VillageCode {
                name: "平乐园西社区居民委员会",
                code: "024",
            },
            VillageCode {
                name: "金海社区居民委员会",
                code: "025",
            },
            VillageCode {
                name: "楼梓庄村委会",
                code: "026",
            },
            VillageCode {
                name: "大郊亭村委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "高碑店地区",
        code: "022",
        villages: &[
            VillageCode {
                name: "甘露园南里二区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大黄庄社区居委会",
                code: "002",
            },
            VillageCode {
                name: "通惠家园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "兴隆家园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "丽景馨居社区居委会",
                code: "005",
            },
            VillageCode {
                name: "八里庄社区居委会",
                code: "006",
            },
            VillageCode {
                name: "甘露园南里一区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "康家园西社区居委会",
                code: "008",
            },
            VillageCode {
                name: "康家园东社区居委会",
                code: "009",
            },
            VillageCode {
                name: "高井社区居委会",
                code: "010",
            },
            VillageCode {
                name: "太平庄南社区居委会",
                code: "011",
            },
            VillageCode {
                name: "太平庄北社区居委会",
                code: "012",
            },
            VillageCode {
                name: "花北西社区居委会",
                code: "013",
            },
            VillageCode {
                name: "花北东社区居委会",
                code: "014",
            },
            VillageCode {
                name: "北花园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "高碑店东社区居委会",
                code: "016",
            },
            VillageCode {
                name: "高碑店西社区居委会",
                code: "017",
            },
            VillageCode {
                name: "方家园社区居委会",
                code: "018",
            },
            VillageCode {
                name: "西店社区居委会",
                code: "019",
            },
            VillageCode {
                name: "半壁店社区居委会",
                code: "020",
            },
            VillageCode {
                name: "小郊亭社区居委会",
                code: "021",
            },
            VillageCode {
                name: "高碑店文化园社区居委会",
                code: "022",
            },
            VillageCode {
                name: "花园闸社区居委会",
                code: "023",
            },
            VillageCode {
                name: "力源里社区居委会",
                code: "024",
            },
            VillageCode {
                name: "水南庄社区居委会",
                code: "025",
            },
            VillageCode {
                name: "太平庄社区居委会",
                code: "026",
            },
            VillageCode {
                name: "高碑店古街社区居民委员会",
                code: "027",
            },
            VillageCode {
                name: "汇星苑社区居民委员会",
                code: "028",
            },
            VillageCode {
                name: "通惠家园东社区居民委员会",
                code: "029",
            },
            VillageCode {
                name: "高井村委会",
                code: "030",
            },
            VillageCode {
                name: "北花园村委会",
                code: "031",
            },
            VillageCode {
                name: "高碑店村委会",
                code: "032",
            },
            VillageCode {
                name: "半壁店村委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "将台地区",
        code: "023",
        villages: &[
            VillageCode {
                name: "丽都社区居委会",
                code: "001",
            },
            VillageCode {
                name: "芳园里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "安家楼社区居委会",
                code: "003",
            },
            VillageCode {
                name: "水岸家园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "将府家园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "梵谷水郡社区居委会",
                code: "006",
            },
            VillageCode {
                name: "瞰都嘉园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "驼房营北里社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "阳光上东社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "将府锦苑东社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "将府锦苑西社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "驼房营村委会",
                code: "012",
            },
            VillageCode {
                name: "东八间房村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "太阳宫地区",
        code: "024",
        villages: &[
            VillageCode {
                name: "芍药居一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "芍药居二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "芍药居三社区居委会",
                code: "003",
            },
            VillageCode {
                name: "惠忠庵社区居委会",
                code: "004",
            },
            VillageCode {
                name: "尚家楼社区居委会",
                code: "005",
            },
            VillageCode {
                name: "太阳宫社区居委会",
                code: "006",
            },
            VillageCode {
                name: "十字口社区居委会",
                code: "007",
            },
            VillageCode {
                name: "牛王庙社区居委会",
                code: "008",
            },
            VillageCode {
                name: "芍药居四社区居委会",
                code: "009",
            },
            VillageCode {
                name: "夏家园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "西坝河北里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "芍药居五社区居民居委会",
                code: "012",
            },
            VillageCode {
                name: "太阳宫村委会",
                code: "013",
            },
            VillageCode {
                name: "十字口村委会",
                code: "014",
            },
            VillageCode {
                name: "牛王庙村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "大屯街道",
        code: "025",
        villages: &[
            VillageCode {
                name: "大屯里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "亚运新新家园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "慧忠里第一社区居委会",
                code: "003",
            },
            VillageCode {
                name: "慧忠里第二社区居委会",
                code: "004",
            },
            VillageCode {
                name: "慧忠北里第一社区居委会",
                code: "005",
            },
            VillageCode {
                name: "慧忠北里第二社区居委会",
                code: "006",
            },
            VillageCode {
                name: "安慧北里秀雅社区居委会",
                code: "007",
            },
            VillageCode {
                name: "安慧北里安园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "育慧西里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "育慧里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "世纪村社区居委会",
                code: "011",
            },
            VillageCode {
                name: "安慧东里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "嘉铭园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "欧陆经典社区居委会",
                code: "014",
            },
            VillageCode {
                name: "金泉家园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "安慧东里第二社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "安慧北里逸园社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "中灿家园社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "融华嘉园社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "富成花园社区居民委员会",
                code: "020",
            },
            VillageCode {
                name: "慧忠北里第三社区居民委员会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "望京街道",
        code: "026",
        villages: &[
            VillageCode {
                name: "花家地西里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "方舟苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "大西洋新城社区居委会",
                code: "003",
            },
            VillageCode {
                name: "望京西园四区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "南湖东园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "南湖西里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "花家地社区居委会",
                code: "007",
            },
            VillageCode {
                name: "望花路西里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "望花路东里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "花家地南里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "南湖中园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "花家地西里三区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "花家地北里社区居委会",
                code: "013",
            },
            VillageCode {
                name: "圣星社区居委会",
                code: "014",
            },
            VillageCode {
                name: "爽秋路社区居委会",
                code: "015",
            },
            VillageCode {
                name: "南湖西园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "望京西园三区社区居委会",
                code: "017",
            },
            VillageCode {
                name: "望京园社区居委会",
                code: "018",
            },
            VillageCode {
                name: "南湖西园二区社区居委会",
                code: "019",
            },
            VillageCode {
                name: "望京东园五区社区居委会",
                code: "020",
            },
            VillageCode {
                name: "阜荣街社区居委会",
                code: "021",
            },
            VillageCode {
                name: "望京西路社区居委会",
                code: "022",
            },
            VillageCode {
                name: "夏都雅园社区居民委员会",
                code: "023",
            },
            VillageCode {
                name: "国风社区居民委员会",
                code: "024",
            },
            VillageCode {
                name: "宝星社区居民委员会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "小红门地区",
        code: "027",
        villages: &[
            VillageCode {
                name: "四道口居民委员会",
                code: "001",
            },
            VillageCode {
                name: "三台山居民委员会",
                code: "002",
            },
            VillageCode {
                name: "玉器厂居民委员会",
                code: "003",
            },
            VillageCode {
                name: "恋日绿岛社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "中海城社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "鸿博家园第一社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "鸿博家园第二社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "鸿博家园第三社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "鸿博家园第四社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "鸿博家园第五社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "江南社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "龙爪树社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "小红门村民委员会",
                code: "013",
            },
            VillageCode {
                name: "牌坊村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "龙爪树村民委员会",
                code: "015",
            },
            VillageCode {
                name: "肖村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "十八里店地区",
        code: "028",
        villages: &[
            VillageCode {
                name: "老君堂社区居委会",
                code: "001",
            },
            VillageCode {
                name: "弘善家园第二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "弘善家园第一社区居委会",
                code: "003",
            },
            VillageCode {
                name: "弘善家园第三社区居委会",
                code: "004",
            },
            VillageCode {
                name: "周庄嘉园第一社区居委会",
                code: "005",
            },
            VillageCode {
                name: "十八里店第一社区居委会",
                code: "006",
            },
            VillageCode {
                name: "弘善寺社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "祁庄社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "山水中西园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "白墙子社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "十八里店村委会",
                code: "011",
            },
            VillageCode {
                name: "吕家营村委会",
                code: "012",
            },
            VillageCode {
                name: "十里河村委会",
                code: "013",
            },
            VillageCode {
                name: "周家庄村委会",
                code: "014",
            },
            VillageCode {
                name: "小武基村委会",
                code: "015",
            },
            VillageCode {
                name: "老君堂村委会",
                code: "016",
            },
            VillageCode {
                name: "横街子村委会",
                code: "017",
            },
            VillageCode {
                name: "西直河村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "平房地区",
        code: "029",
        villages: &[
            VillageCode {
                name: "平房社区居委会",
                code: "001",
            },
            VillageCode {
                name: "富华家园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "姚家园西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "星河湾社区居委会",
                code: "004",
            },
            VillageCode {
                name: "华纺易城社区居委会",
                code: "005",
            },
            VillageCode {
                name: "国美家园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "雅成里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "定福家园南社区居委会",
                code: "008",
            },
            VillageCode {
                name: "天鹅湾社区居委会",
                code: "009",
            },
            VillageCode {
                name: "青年路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "姚家园东社区居委会",
                code: "011",
            },
            VillageCode {
                name: "逸翠园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "泓鑫家园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "国美家园第二社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "熙悦尚郡社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "姚家园南社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "平房村委会",
                code: "017",
            },
            VillageCode {
                name: "姚家园村委会",
                code: "018",
            },
            VillageCode {
                name: "黄渠村委会",
                code: "019",
            },
            VillageCode {
                name: "石各庄村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "东风地区",
        code: "030",
        villages: &[
            VillageCode {
                name: "石佛营东里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东润枫景社区居委会",
                code: "002",
            },
            VillageCode {
                name: "石佛营西里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "石佛营南里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "紫萝园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "公园大道社区居委会",
                code: "006",
            },
            VillageCode {
                name: "泛海国际南社区居委会",
                code: "007",
            },
            VillageCode {
                name: "观湖国际社区居委会",
                code: "008",
            },
            VillageCode {
                name: "南十里居社区居委会",
                code: "009",
            },
            VillageCode {
                name: "东风苑社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "东风家园社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "豆各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "将台洼村委会",
                code: "013",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "014",
            },
            VillageCode {
                name: "六里屯村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "奥运村街道",
        code: "031",
        villages: &[
            VillageCode {
                name: "大羊坊社区居委会",
                code: "001",
            },
            VillageCode {
                name: "双泉社区居委会",
                code: "002",
            },
            VillageCode {
                name: "总装社区居委会",
                code: "003",
            },
            VillageCode {
                name: "科学园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "风林绿洲社区居委会",
                code: "005",
            },
            VillageCode {
                name: "北沙滩社区居委会",
                code: "006",
            },
            VillageCode {
                name: "林萃社区居委会",
                code: "007",
            },
            VillageCode {
                name: "绿色家园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "南沙滩社区居委会",
                code: "009",
            },
            VillageCode {
                name: "万科星园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "龙祥社区居委会",
                code: "011",
            },
            VillageCode {
                name: "国奥村社区居委会",
                code: "012",
            },
            VillageCode {
                name: "拂林园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "大羊坊南社区居委会",
                code: "014",
            },
            VillageCode {
                name: "林萃西里社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "北沙滩北社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "林奥嘉园社区居民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "来广营地区",
        code: "032",
        villages: &[
            VillageCode {
                name: "新街坊社区居委会",
                code: "001",
            },
            VillageCode {
                name: "立城苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北苑一号院社区居委会",
                code: "003",
            },
            VillageCode {
                name: "北苑二号院社区居委会",
                code: "004",
            },
            VillageCode {
                name: "北苑三号院社区居委会",
                code: "005",
            },
            VillageCode {
                name: "北苑家园清友园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "北苑家园绣菊园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "北苑家园紫绶园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "朝来绿色家园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "时代庄园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "青年城社区居委会",
                code: "011",
            },
            VillageCode {
                name: "莲葩园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "茉藜园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "黄金苑社区居委会",
                code: "014",
            },
            VillageCode {
                name: "立清路第一社区居委会",
                code: "015",
            },
            VillageCode {
                name: "广达路社区居委会",
                code: "016",
            },
            VillageCode {
                name: "清苑路第一社区居委会",
                code: "017",
            },
            VillageCode {
                name: "红军营社区居委会",
                code: "018",
            },
            VillageCode {
                name: "北卫家园社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "清河营中路社区居民委员会",
                code: "020",
            },
            VillageCode {
                name: "清苑路第二社区居民委员会",
                code: "021",
            },
            VillageCode {
                name: "清苑路第三社区居民委员会",
                code: "022",
            },
            VillageCode {
                name: "北苑社区居民委员会",
                code: "023",
            },
            VillageCode {
                name: "清苑路第四社区居民委员会",
                code: "024",
            },
            VillageCode {
                name: "朝来绿色家园东社区居民委员会",
                code: "025",
            },
            VillageCode {
                name: "筑华年社区居民委员会",
                code: "026",
            },
            VillageCode {
                name: "立清路第二社区居民委员会",
                code: "027",
            },
            VillageCode {
                name: "清苑路第五社区居民委员会",
                code: "028",
            },
            VillageCode {
                name: "广顺社区居民委员会",
                code: "029",
            },
            VillageCode {
                name: "北苑中街社区居民委员会",
                code: "030",
            },
            VillageCode {
                name: "立水桥社区居民委员会",
                code: "031",
            },
            VillageCode {
                name: "新街坊第二社区居民委员会",
                code: "032",
            },
            VillageCode {
                name: "清苑路第六社区居民委员会",
                code: "033",
            },
            VillageCode {
                name: "红军营村委会",
                code: "034",
            },
            VillageCode {
                name: "北湖渠村委会",
                code: "035",
            },
            VillageCode {
                name: "来广营村委会",
                code: "036",
            },
            VillageCode {
                name: "新生村委会",
                code: "037",
            },
            VillageCode {
                name: "清河营村委会",
                code: "038",
            },
        ],
    },
    TownCode {
        name: "常营地区",
        code: "033",
        villages: &[
            VillageCode {
                name: "荟万鸿社区居委会",
                code: "001",
            },
            VillageCode {
                name: "鑫兆佳园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "万象新天社区居委会",
                code: "003",
            },
            VillageCode {
                name: "常营民族家园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "连心园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "苹果派社区居委会",
                code: "006",
            },
            VillageCode {
                name: "常营福第社区居委会",
                code: "007",
            },
            VillageCode {
                name: "畅心园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "常营保利社区居委会",
                code: "009",
            },
            VillageCode {
                name: "丽景园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "住欣家园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "东方华庭社区居委会",
                code: "012",
            },
            VillageCode {
                name: "鑫兆佳园北社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "万象新天北社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "燕保汇鸿家园社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "燕保常营家园社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "和锦园社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "富力阳光美园社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "常营民族家园西社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "北京像素北区治理委员会社区",
                code: "020",
            },
            VillageCode {
                name: "北京像素南区治理委员会社区",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "三间房地区",
        code: "034",
        villages: &[
            VillageCode {
                name: "福怡苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "三间房南里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "定南里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "定北里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "定西北里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "定西南里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "双桥铁路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "双桥路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "双柳社区居委会",
                code: "009",
            },
            VillageCode {
                name: "双惠苑社区居委会",
                code: "010",
            },
            VillageCode {
                name: "绿洲家园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "艺水芳园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "美然动力社区居委会",
                code: "013",
            },
            VillageCode {
                name: "聚福苑社区居委会",
                code: "014",
            },
            VillageCode {
                name: "泰福苑社区居委会",
                code: "015",
            },
            VillageCode {
                name: "华美社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "电建社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "康新社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "三间房东村村委会",
                code: "019",
            },
            VillageCode {
                name: "三间房西村村委会",
                code: "020",
            },
            VillageCode {
                name: "定福庄东村村委会",
                code: "021",
            },
            VillageCode {
                name: "定福庄西村村委会",
                code: "022",
            },
            VillageCode {
                name: "褡裢坡村村委会",
                code: "023",
            },
            VillageCode {
                name: "白家楼村村委会",
                code: "024",
            },
            VillageCode {
                name: "东柳村村委会",
                code: "025",
            },
            VillageCode {
                name: "西柳村村委会",
                code: "026",
            },
            VillageCode {
                name: "新房村村委会",
                code: "027",
            },
            VillageCode {
                name: "北双桥村村委会",
                code: "028",
            },
            VillageCode {
                name: "金家村村委会",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "管庄地区",
        code: "035",
        villages: &[
            VillageCode {
                name: "八里桥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "管庄东里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "管庄西里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "建东苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "京通苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "丽景苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "惠河东里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "瑞祥里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "惠河西里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新天地社区居委会",
                code: "010",
            },
            VillageCode {
                name: "远洋一方嘉园社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "管庄北里社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "远洋一方润园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "新天地一社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "管庄西里一社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "康悦社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "西会村委会",
                code: "017",
            },
            VillageCode {
                name: "东会村委会",
                code: "018",
            },
            VillageCode {
                name: "八里桥村委会",
                code: "019",
            },
            VillageCode {
                name: "果家店村委会",
                code: "020",
            },
            VillageCode {
                name: "塔营村委会",
                code: "021",
            },
            VillageCode {
                name: "小寺村委会",
                code: "022",
            },
            VillageCode {
                name: "重兴寺村委会",
                code: "023",
            },
            VillageCode {
                name: "司辛庄村委会",
                code: "024",
            },
            VillageCode {
                name: "郭家场村委会",
                code: "025",
            },
            VillageCode {
                name: "杨闸村委会",
                code: "026",
            },
            VillageCode {
                name: "管庄村村委会",
                code: "027",
            },
            VillageCode {
                name: "咸宁侯村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "金盏地区",
        code: "036",
        villages: &[
            VillageCode {
                name: "朝阳农场地区居委会",
                code: "001",
            },
            VillageCode {
                name: "楼梓庄居委会",
                code: "002",
            },
            VillageCode {
                name: "金泽家园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "金盏嘉园第一社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金盏嘉园第二社区居委会",
                code: "005",
            },
            VillageCode {
                name: "金泽家园北社区居委会",
                code: "006",
            },
            VillageCode {
                name: "雷庄村村委会",
                code: "007",
            },
            VillageCode {
                name: "东大队村村委会",
                code: "008",
            },
            VillageCode {
                name: "西大队村村委会",
                code: "009",
            },
            VillageCode {
                name: "小店村村委会",
                code: "010",
            },
            VillageCode {
                name: "长店村村委会",
                code: "011",
            },
            VillageCode {
                name: "北马房村村委会",
                code: "012",
            },
            VillageCode {
                name: "楼梓庄村委会",
                code: "013",
            },
            VillageCode {
                name: "沙窝村委会",
                code: "014",
            },
            VillageCode {
                name: "黎各庄村委会",
                code: "015",
            },
            VillageCode {
                name: "马各庄村委会",
                code: "016",
            },
            VillageCode {
                name: "皮村村委会",
                code: "017",
            },
            VillageCode {
                name: "东窑村委会",
                code: "018",
            },
            VillageCode {
                name: "曹各庄村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "孙河地区",
        code: "037",
        villages: &[
            VillageCode {
                name: "康营家园一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "康营家园二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "康营家园三社区居委会",
                code: "003",
            },
            VillageCode {
                name: "康营家园四社区居委会",
                code: "004",
            },
            VillageCode {
                name: "景润苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "清榆园社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "香榆园社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "瑞榆园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "翠榆园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "孙河村委会",
                code: "010",
            },
            VillageCode {
                name: "康营村委会",
                code: "011",
            },
            VillageCode {
                name: "北甸东村委会",
                code: "012",
            },
            VillageCode {
                name: "北甸西村委会",
                code: "013",
            },
            VillageCode {
                name: "西甸村委会",
                code: "014",
            },
            VillageCode {
                name: "下辛堡村委会",
                code: "015",
            },
            VillageCode {
                name: "上辛堡村委会",
                code: "016",
            },
            VillageCode {
                name: "黄港村委会",
                code: "017",
            },
            VillageCode {
                name: "李县坟村委会",
                code: "018",
            },
            VillageCode {
                name: "雷桥村委会",
                code: "019",
            },
            VillageCode {
                name: "沈家坟村委会",
                code: "020",
            },
            VillageCode {
                name: "沙子营村委会",
                code: "021",
            },
            VillageCode {
                name: "苇沟村民委员会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "崔各庄地区",
        code: "038",
        villages: &[
            VillageCode {
                name: "马南里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "京旺家园第一社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "京旺家园第二社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "燕保马泉营家园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "东洲家园社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "广善第一社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新锦社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "京旺家园第三社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "京旺家园第四社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "和平社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "崔各庄村委会",
                code: "011",
            },
            VillageCode {
                name: "善各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "何各庄村委会",
                code: "013",
            },
            VillageCode {
                name: "马泉营村委会",
                code: "014",
            },
            VillageCode {
                name: "奶东村委会",
                code: "015",
            },
            VillageCode {
                name: "奶西村委会",
                code: "016",
            },
            VillageCode {
                name: "索家村委会",
                code: "017",
            },
            VillageCode {
                name: "费家村委会",
                code: "018",
            },
            VillageCode {
                name: "南皋村委会",
                code: "019",
            },
            VillageCode {
                name: "望京村委会",
                code: "020",
            },
            VillageCode {
                name: "草场地村委会",
                code: "021",
            },
            VillageCode {
                name: "东辛店村委会",
                code: "022",
            },
            VillageCode {
                name: "北皋村委会",
                code: "023",
            },
            VillageCode {
                name: "东营村委会",
                code: "024",
            },
            VillageCode {
                name: "黑桥村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "东坝地区",
        code: "039",
        villages: &[
            VillageCode {
                name: "高杨树社区居委会",
                code: "001",
            },
            VillageCode {
                name: "红松园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "红松园北里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "康静里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东坝家园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "奥林匹克花园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "朝阳新城社区居委会",
                code: "007",
            },
            VillageCode {
                name: "丽富嘉园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "常青藤社区居委会",
                code: "009",
            },
            VillageCode {
                name: "景和园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "东泽园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "悦和园社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "金驹家园第一社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "金驹家园第二社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "福润四季社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "坝鑫家园社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "朝阳新城第二社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "汇景苑社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "郑村社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "福园第一社区居民委员会",
                code: "020",
            },
            VillageCode {
                name: "润泽社区居民委员会",
                code: "021",
            },
            VillageCode {
                name: "东湾社区居民委员会",
                code: "022",
            },
            VillageCode {
                name: "福园第二社区居民委员会",
                code: "023",
            },
            VillageCode {
                name: "奥林匹克花园第二社区居民委员会",
                code: "024",
            },
            VillageCode {
                name: "景和园第二社区居民委员会",
                code: "025",
            },
            VillageCode {
                name: "常青藤第二社区居民委员会",
                code: "026",
            },
            VillageCode {
                name: "福园第三社区居民委员会",
                code: "027",
            },
            VillageCode {
                name: "汇景苑第二社区居民委员会",
                code: "028",
            },
            VillageCode {
                name: "七棵树村委会",
                code: "029",
            },
            VillageCode {
                name: "单店村委会",
                code: "030",
            },
            VillageCode {
                name: "西北门村委会",
                code: "031",
            },
            VillageCode {
                name: "后街村委会",
                code: "032",
            },
            VillageCode {
                name: "东风村村委会",
                code: "033",
            },
            VillageCode {
                name: "驹子房村委会",
                code: "034",
            },
            VillageCode {
                name: "三岔河村委会",
                code: "035",
            },
            VillageCode {
                name: "焦庄村村委会",
                code: "036",
            },
            VillageCode {
                name: "东晓景村委会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "黑庄户地区",
        code: "040",
        villages: &[
            VillageCode {
                name: "双桥第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "双桥第二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "康城社区居委会",
                code: "003",
            },
            VillageCode {
                name: "怡景城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "旭园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "东旭社区居委会",
                code: "006",
            },
            VillageCode {
                name: "双桥第三社区居委会",
                code: "007",
            },
            VillageCode {
                name: "大鲁店一村委会",
                code: "008",
            },
            VillageCode {
                name: "大鲁店二村委会",
                code: "009",
            },
            VillageCode {
                name: "大鲁店三村委会",
                code: "010",
            },
            VillageCode {
                name: "小鲁店村委会",
                code: "011",
            },
            VillageCode {
                name: "郎各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "郎辛庄村委会",
                code: "013",
            },
            VillageCode {
                name: "万子营西村委会",
                code: "014",
            },
            VillageCode {
                name: "万子营东村委会",
                code: "015",
            },
            VillageCode {
                name: "黑庄户村委会",
                code: "016",
            },
            VillageCode {
                name: "四合庄村委会",
                code: "017",
            },
            VillageCode {
                name: "定辛庄西村委会",
                code: "018",
            },
            VillageCode {
                name: "定辛庄东村委会",
                code: "019",
            },
            VillageCode {
                name: "双树南村委会",
                code: "020",
            },
            VillageCode {
                name: "双树北村委会",
                code: "021",
            },
            VillageCode {
                name: "苏坟村委会",
                code: "022",
            },
            VillageCode {
                name: "么铺村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "豆各庄地区",
        code: "041",
        villages: &[
            VillageCode {
                name: "青青家园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "绿丰家园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "阳光家园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "京城雅居社区居委会",
                code: "004",
            },
            VillageCode {
                name: "文化传播社区居委会",
                code: "005",
            },
            VillageCode {
                name: "朝丰家园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "富力又一城第一社区居委会",
                code: "007",
            },
            VillageCode {
                name: "富力又一城第二社区居委会",
                code: "008",
            },
            VillageCode {
                name: "明德园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "青荷里社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "御景湾社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "宸欣园社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "富力又一城第三社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "晨风园一社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "晨风园二社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "梧桐湾社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "豆各庄村委会",
                code: "017",
            },
            VillageCode {
                name: "马家湾村委会",
                code: "018",
            },
            VillageCode {
                name: "水牛坊村委会",
                code: "019",
            },
            VillageCode {
                name: "孙家坡村委会",
                code: "020",
            },
            VillageCode {
                name: "孟家屯村委会",
                code: "021",
            },
            VillageCode {
                name: "东马各庄村委会",
                code: "022",
            },
            VillageCode {
                name: "西马各庄村委会",
                code: "023",
            },
            VillageCode {
                name: "于家围南队村委会",
                code: "024",
            },
            VillageCode {
                name: "于家围北队村委会",
                code: "025",
            },
            VillageCode {
                name: "南何家村村委会",
                code: "026",
            },
            VillageCode {
                name: "石槽村村委会",
                code: "027",
            },
            VillageCode {
                name: "黄厂村村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "王四营地区",
        code: "042",
        villages: &[
            VillageCode {
                name: "观音堂社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "白鹿社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "官悦欣园社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "观音堂二社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "海棠社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "孛旺花园社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "百安家园社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "百湾家园一社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "百湾家园二社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "官庄大队村民委员会",
                code: "010",
            },
            VillageCode {
                name: "观音堂大队村民委员会",
                code: "011",
            },
            VillageCode {
                name: "王四营大队村民委员会",
                code: "012",
            },
            VillageCode {
                name: "南花园大队村民委员会",
                code: "013",
            },
            VillageCode {
                name: "道口大队村民委员会",
                code: "014",
            },
            VillageCode {
                name: "孛罗营大队村民委员会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "东湖街道",
        code: "043",
        villages: &[
            VillageCode {
                name: "望京西园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南湖东园北社区居委会",
                code: "002",
            },
            VillageCode {
                name: "南湖中园北社区居委会",
                code: "003",
            },
            VillageCode {
                name: "望京花园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "利泽西园一区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "果岭里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "望湖社区居委会",
                code: "007",
            },
            VillageCode {
                name: "大望京社区居委会",
                code: "008",
            },
            VillageCode {
                name: "东湖湾社区居委会",
                code: "009",
            },
            VillageCode {
                name: "望京西园二区社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "望京花园东社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "康都佳园社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "华彩社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "望馨花园社区居民委员会",
                code: "014",
            },
        ],
    },
];

static TOWNS_BP_004: [TownCode; 26] = [
    TownCode {
        name: "右安门街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "翠林一里社区",
                code: "001",
            },
            VillageCode {
                name: "翠林二里社区",
                code: "002",
            },
            VillageCode {
                name: "翠林三里社区",
                code: "003",
            },
            VillageCode {
                name: "玉林里社区",
                code: "004",
            },
            VillageCode {
                name: "玉林西里社区",
                code: "005",
            },
            VillageCode {
                name: "西铁营社区",
                code: "006",
            },
            VillageCode {
                name: "东滨河路社区",
                code: "007",
            },
            VillageCode {
                name: "东庄社区",
                code: "008",
            },
            VillageCode {
                name: "玉林东里一区社区",
                code: "009",
            },
            VillageCode {
                name: "玉林东里二区社区",
                code: "010",
            },
            VillageCode {
                name: "玉林东里三区社区",
                code: "011",
            },
            VillageCode {
                name: "永乐社区",
                code: "012",
            },
            VillageCode {
                name: "开阳里第一社区",
                code: "013",
            },
            VillageCode {
                name: "开阳里第二社区",
                code: "014",
            },
            VillageCode {
                name: "开阳里第三社区",
                code: "015",
            },
            VillageCode {
                name: "开阳里第四社区",
                code: "016",
            },
            VillageCode {
                name: "亚林苑一社区",
                code: "017",
            },
            VillageCode {
                name: "亚林苑二社区",
                code: "018",
            },
            VillageCode {
                name: "西铁营村",
                code: "019",
            },
            VillageCode {
                name: "右安门村",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "太平桥街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "莲花池社区",
                code: "001",
            },
            VillageCode {
                name: "太平桥西里社区",
                code: "002",
            },
            VillageCode {
                name: "太平桥中里社区",
                code: "003",
            },
            VillageCode {
                name: "太平桥东里社区",
                code: "004",
            },
            VillageCode {
                name: "太平桥南里社区",
                code: "005",
            },
            VillageCode {
                name: "东管头社区",
                code: "006",
            },
            VillageCode {
                name: "三路居社区",
                code: "007",
            },
            VillageCode {
                name: "天伦北里社区",
                code: "008",
            },
            VillageCode {
                name: "菜户营社区",
                code: "009",
            },
            VillageCode {
                name: "万泉寺社区",
                code: "010",
            },
            VillageCode {
                name: "精图社区",
                code: "011",
            },
            VillageCode {
                name: "万润社区",
                code: "012",
            },
            VillageCode {
                name: "首威社区",
                code: "013",
            },
            VillageCode {
                name: "丽湾社区",
                code: "014",
            },
            VillageCode {
                name: "蓝调社区",
                code: "015",
            },
            VillageCode {
                name: "万泉寺东社区",
                code: "016",
            },
            VillageCode {
                name: "金鹏天润社区",
                code: "017",
            },
            VillageCode {
                name: "太平桥村",
                code: "018",
            },
            VillageCode {
                name: "马连道村",
                code: "019",
            },
            VillageCode {
                name: "万泉寺村",
                code: "020",
            },
            VillageCode {
                name: "菜户营村",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "西罗园街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "西罗园第一社区",
                code: "001",
            },
            VillageCode {
                name: "西罗园第二社区",
                code: "002",
            },
            VillageCode {
                name: "西罗园第三社区",
                code: "003",
            },
            VillageCode {
                name: "西罗园第四社区",
                code: "004",
            },
            VillageCode {
                name: "洋桥北里社区",
                code: "005",
            },
            VillageCode {
                name: "洋桥西里社区",
                code: "006",
            },
            VillageCode {
                name: "海户西里北社区",
                code: "007",
            },
            VillageCode {
                name: "角门东里一社区",
                code: "008",
            },
            VillageCode {
                name: "花椒树社区",
                code: "009",
            },
            VillageCode {
                name: "马家堡东里社区",
                code: "010",
            },
            VillageCode {
                name: "鑫福里社区",
                code: "011",
            },
            VillageCode {
                name: "洋桥村社区",
                code: "012",
            },
            VillageCode {
                name: "洋桥东里社区",
                code: "013",
            },
            VillageCode {
                name: "海户西里南社区",
                code: "014",
            },
            VillageCode {
                name: "角门东里二社区",
                code: "015",
            },
            VillageCode {
                name: "四路通社区",
                code: "016",
            },
            VillageCode {
                name: "角门东里三社区",
                code: "017",
            },
            VillageCode {
                name: "西马场北里社区",
                code: "018",
            },
            VillageCode {
                name: "怡然家园社区",
                code: "019",
            },
            VillageCode {
                name: "西马小区社区",
                code: "020",
            },
            VillageCode {
                name: "金润家园一社区",
                code: "021",
            },
            VillageCode {
                name: "福海棠华苑社区",
                code: "022",
            },
            VillageCode {
                name: "金润家园二社区",
                code: "023",
            },
            VillageCode {
                name: "金润家园三社区",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "大红门街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "西罗园南里社区",
                code: "001",
            },
            VillageCode {
                name: "西罗园南里果园社区",
                code: "002",
            },
            VillageCode {
                name: "西罗园南里华远社区",
                code: "003",
            },
            VillageCode {
                name: "海户屯社区",
                code: "004",
            },
            VillageCode {
                name: "木樨园南里社区",
                code: "005",
            },
            VillageCode {
                name: "东罗园社区",
                code: "006",
            },
            VillageCode {
                name: "南顶村社区",
                code: "007",
            },
            VillageCode {
                name: "南顶路社区",
                code: "008",
            },
            VillageCode {
                name: "康泽园社区",
                code: "009",
            },
            VillageCode {
                name: "时村社区",
                code: "010",
            },
            VillageCode {
                name: "大红门东街社区",
                code: "011",
            },
            VillageCode {
                name: "苗圃东里社区",
                code: "012",
            },
            VillageCode {
                name: "苗圃西里社区",
                code: "013",
            },
            VillageCode {
                name: "西马场南里社区",
                code: "014",
            },
            VillageCode {
                name: "建欣苑社区",
                code: "015",
            },
            VillageCode {
                name: "远洋自然社区",
                code: "016",
            },
            VillageCode {
                name: "光彩路社区",
                code: "017",
            },
            VillageCode {
                name: "彩虹城第二社区",
                code: "018",
            },
            VillageCode {
                name: "建欣苑东区社区",
                code: "019",
            },
            VillageCode {
                name: "东罗园村",
                code: "020",
            },
            VillageCode {
                name: "时村",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "南苑街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "东新华社区",
                code: "001",
            },
            VillageCode {
                name: "红房子社区",
                code: "002",
            },
            VillageCode {
                name: "西宏苑社区",
                code: "003",
            },
            VillageCode {
                name: "诚苑社区",
                code: "004",
            },
            VillageCode {
                name: "槐房社区",
                code: "005",
            },
            VillageCode {
                name: "机场社区",
                code: "006",
            },
            VillageCode {
                name: "翠海明苑社区",
                code: "007",
            },
            VillageCode {
                name: "南庭新苑南区社区",
                code: "008",
            },
            VillageCode {
                name: "南庭新苑北区社区",
                code: "009",
            },
            VillageCode {
                name: "阳光星苑社区",
                code: "010",
            },
            VillageCode {
                name: "阳光星苑南区社区",
                code: "011",
            },
            VillageCode {
                name: "合顺家园社区",
                code: "012",
            },
            VillageCode {
                name: "京粮悦谷社区",
                code: "013",
            },
            VillageCode {
                name: "德鑫嘉园社区",
                code: "014",
            },
            VillageCode {
                name: "新宫社区",
                code: "015",
            },
            VillageCode {
                name: "御槐园社区",
                code: "016",
            },
            VillageCode {
                name: "天悦佳苑社区",
                code: "017",
            },
            VillageCode {
                name: "南苑村",
                code: "018",
            },
            VillageCode {
                name: "槐房村",
                code: "019",
            },
            VillageCode {
                name: "新宫村",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "东高地街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "东高地社区",
                code: "001",
            },
            VillageCode {
                name: "三角地第一社区",
                code: "002",
            },
            VillageCode {
                name: "三角地第二社区",
                code: "003",
            },
            VillageCode {
                name: "西洼地社区",
                code: "004",
            },
            VillageCode {
                name: "六营门社区",
                code: "005",
            },
            VillageCode {
                name: "万源东里社区",
                code: "006",
            },
            VillageCode {
                name: "万源西里社区",
                code: "007",
            },
            VillageCode {
                name: "梅源社区",
                code: "008",
            },
            VillageCode {
                name: "东营房社区",
                code: "009",
            },
            VillageCode {
                name: "万源南里社区",
                code: "010",
            },
            VillageCode {
                name: "东高地北社区",
                code: "011",
            },
            VillageCode {
                name: "东高地南社区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "东铁匠营街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "蒲黄榆第一社区",
                code: "001",
            },
            VillageCode {
                name: "蒲黄榆第二社区",
                code: "002",
            },
            VillageCode {
                name: "蒲黄榆第三社区",
                code: "003",
            },
            VillageCode {
                name: "蒲安里第一社区",
                code: "004",
            },
            VillageCode {
                name: "蒲安里第二社区",
                code: "005",
            },
            VillageCode {
                name: "刘家窑第一社区",
                code: "006",
            },
            VillageCode {
                name: "刘家窑第二社区",
                code: "007",
            },
            VillageCode {
                name: "刘家窑第三社区",
                code: "008",
            },
            VillageCode {
                name: "木樨园第一社区",
                code: "009",
            },
            VillageCode {
                name: "木樨园第二社区",
                code: "010",
            },
            VillageCode {
                name: "同仁园社区",
                code: "011",
            },
            VillageCode {
                name: "横七条路第一社区",
                code: "012",
            },
            VillageCode {
                name: "横七条路第二社区",
                code: "013",
            },
            VillageCode {
                name: "横七条路第三社区",
                code: "014",
            },
            VillageCode {
                name: "宋庄路第一社区",
                code: "015",
            },
            VillageCode {
                name: "宋庄路第二社区",
                code: "016",
            },
            VillageCode {
                name: "南方庄社区",
                code: "017",
            },
            VillageCode {
                name: "宋庄路第三社区",
                code: "018",
            },
            VillageCode {
                name: "东木樨园社区",
                code: "019",
            },
            VillageCode {
                name: "定安东里社区",
                code: "020",
            },
            VillageCode {
                name: "贾家花园社区",
                code: "021",
            },
            VillageCode {
                name: "嘉顺园社区",
                code: "022",
            },
            VillageCode {
                name: "东铁匠营村",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "六里桥街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "丰台路口社区",
                code: "001",
            },
            VillageCode {
                name: "望园社区",
                code: "002",
            },
            VillageCode {
                name: "六里桥南里社区",
                code: "003",
            },
            VillageCode {
                name: "六里桥北里社区",
                code: "004",
            },
            VillageCode {
                name: "八一厂社区",
                code: "005",
            },
            VillageCode {
                name: "六里桥社区",
                code: "006",
            },
            VillageCode {
                name: "莲怡园社区",
                code: "007",
            },
            VillageCode {
                name: "莲香园社区",
                code: "008",
            },
            VillageCode {
                name: "金家村第一社区",
                code: "009",
            },
            VillageCode {
                name: "金家村第二社区",
                code: "010",
            },
            VillageCode {
                name: "京铁家园社区",
                code: "011",
            },
            VillageCode {
                name: "保利益丰社区",
                code: "012",
            },
            VillageCode {
                name: "科兴佳园社区",
                code: "013",
            },
            VillageCode {
                name: "西局欣园社区",
                code: "014",
            },
            VillageCode {
                name: "西局玉园社区",
                code: "015",
            },
            VillageCode {
                name: "玉璞家园社区",
                code: "016",
            },
            VillageCode {
                name: "靛厂村",
                code: "017",
            },
            VillageCode {
                name: "小井村",
                code: "018",
            },
            VillageCode {
                name: "六里桥村",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "丰台街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "北大街北里社区",
                code: "001",
            },
            VillageCode {
                name: "东大街西里社区",
                code: "002",
            },
            VillageCode {
                name: "北大街社区",
                code: "003",
            },
            VillageCode {
                name: "东幸福街社区",
                code: "004",
            },
            VillageCode {
                name: "永善社区",
                code: "005",
            },
            VillageCode {
                name: "正阳北里社区",
                code: "006",
            },
            VillageCode {
                name: "东大街东里社区",
                code: "007",
            },
            VillageCode {
                name: "东大街社区",
                code: "008",
            },
            VillageCode {
                name: "前泥洼社区",
                code: "009",
            },
            VillageCode {
                name: "向阳社区",
                code: "010",
            },
            VillageCode {
                name: "建国街社区",
                code: "011",
            },
            VillageCode {
                name: "新华街北社区",
                code: "012",
            },
            VillageCode {
                name: "新华街南社区",
                code: "013",
            },
            VillageCode {
                name: "丰益花园社区",
                code: "014",
            },
            VillageCode {
                name: "丰管路社区",
                code: "015",
            },
            VillageCode {
                name: "游泳场北路社区",
                code: "016",
            },
            VillageCode {
                name: "丽泽景园社区",
                code: "017",
            },
            VillageCode {
                name: "周庄子村",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "新村街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "育芳园社区",
                code: "001",
            },
            VillageCode {
                name: "桥梁厂第一社区",
                code: "002",
            },
            VillageCode {
                name: "桥梁厂第二社区",
                code: "003",
            },
            VillageCode {
                name: "造甲南里社区",
                code: "004",
            },
            VillageCode {
                name: "造甲村社区",
                code: "005",
            },
            VillageCode {
                name: "怡海花园社区",
                code: "006",
            },
            VillageCode {
                name: "三环新城第一社区",
                code: "007",
            },
            VillageCode {
                name: "万年花城第一社区",
                code: "008",
            },
            VillageCode {
                name: "三环新城第二社区",
                code: "009",
            },
            VillageCode {
                name: "三环新城第三社区",
                code: "010",
            },
            VillageCode {
                name: "万年花城第二社区",
                code: "011",
            },
            VillageCode {
                name: "芳菲路社区",
                code: "012",
            },
            VillageCode {
                name: "首经贸中街社区",
                code: "013",
            },
            VillageCode {
                name: "优筑社区",
                code: "014",
            },
            VillageCode {
                name: "鸿业兴园社区",
                code: "015",
            },
            VillageCode {
                name: "鸿业兴园南社区",
                code: "016",
            },
            VillageCode {
                name: "育菲园社区",
                code: "017",
            },
            VillageCode {
                name: "刘孟家园社区",
                code: "018",
            },
            VillageCode {
                name: "樊家村",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "长辛店街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "南墙缝社区",
                code: "001",
            },
            VillageCode {
                name: "合成公社区",
                code: "002",
            },
            VillageCode {
                name: "东山坡社区",
                code: "003",
            },
            VillageCode {
                name: "北关社区",
                code: "004",
            },
            VillageCode {
                name: "西峰寺社区",
                code: "005",
            },
            VillageCode {
                name: "朱家坟南区社区",
                code: "006",
            },
            VillageCode {
                name: "朱家坟北区社区",
                code: "007",
            },
            VillageCode {
                name: "赵辛店社区",
                code: "008",
            },
            VillageCode {
                name: "北岗洼社区",
                code: "009",
            },
            VillageCode {
                name: "崔村二里社区",
                code: "010",
            },
            VillageCode {
                name: "建设里社区",
                code: "011",
            },
            VillageCode {
                name: "东南街社区",
                code: "012",
            },
            VillageCode {
                name: "陈庄社区",
                code: "013",
            },
            VillageCode {
                name: "光明里社区",
                code: "014",
            },
            VillageCode {
                name: "玉皇庄社区",
                code: "015",
            },
            VillageCode {
                name: "长馨园社区",
                code: "016",
            },
            VillageCode {
                name: "中奥嘉园社区",
                code: "017",
            },
            VillageCode {
                name: "中体奥园社区",
                code: "018",
            },
            VillageCode {
                name: "长辛店村",
                code: "019",
            },
            VillageCode {
                name: "赵辛店村",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "云岗街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "南区第一社区",
                code: "001",
            },
            VillageCode {
                name: "南区第二社区",
                code: "002",
            },
            VillageCode {
                name: "云西路社区",
                code: "003",
            },
            VillageCode {
                name: "田城社区",
                code: "004",
            },
            VillageCode {
                name: "北区社区",
                code: "005",
            },
            VillageCode {
                name: "北里社区",
                code: "006",
            },
            VillageCode {
                name: "翠园社区",
                code: "007",
            },
            VillageCode {
                name: "镇岗南里社区",
                code: "008",
            },
            VillageCode {
                name: "张家坟社区",
                code: "009",
            },
            VillageCode {
                name: "朱家坟西山坡社区",
                code: "010",
            },
            VillageCode {
                name: "珠光嘉园社区",
                code: "011",
            },
            VillageCode {
                name: "珠光逸景社区",
                code: "012",
            },
            VillageCode {
                name: "张家坟村",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "方庄街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "芳古园一区第一社区",
                code: "001",
            },
            VillageCode {
                name: "芳古园一区第二社区",
                code: "002",
            },
            VillageCode {
                name: "芳古园二区社区",
                code: "003",
            },
            VillageCode {
                name: "芳城园一区第二社区",
                code: "004",
            },
            VillageCode {
                name: "芳城园二区社区",
                code: "005",
            },
            VillageCode {
                name: "芳城园三区社区",
                code: "006",
            },
            VillageCode {
                name: "芳群园一区社区",
                code: "007",
            },
            VillageCode {
                name: "芳群园二区社区",
                code: "008",
            },
            VillageCode {
                name: "芳群园三区社区",
                code: "009",
            },
            VillageCode {
                name: "芳群园四区社区",
                code: "010",
            },
            VillageCode {
                name: "芳星园一区社区",
                code: "011",
            },
            VillageCode {
                name: "芳星园二区社区",
                code: "012",
            },
            VillageCode {
                name: "芳星园三区社区",
                code: "013",
            },
            VillageCode {
                name: "芳城东里社区",
                code: "014",
            },
            VillageCode {
                name: "紫芳园社区",
                code: "015",
            },
            VillageCode {
                name: "紫芳园南里社区",
                code: "016",
            },
            VillageCode {
                name: "芳城园一区第一社区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "宛平街道",
        code: "014",
        villages: &[
            VillageCode {
                name: "城北社区",
                code: "001",
            },
            VillageCode {
                name: "宛平城社区",
                code: "002",
            },
            VillageCode {
                name: "城南社区",
                code: "003",
            },
            VillageCode {
                name: "晓月苑社区",
                code: "004",
            },
            VillageCode {
                name: "老庄子社区",
                code: "005",
            },
            VillageCode {
                name: "晓月苑第二社区",
                code: "006",
            },
            VillageCode {
                name: "沸城社区",
                code: "007",
            },
            VillageCode {
                name: "城南第二社区",
                code: "008",
            },
            VillageCode {
                name: "景园社区",
                code: "009",
            },
            VillageCode {
                name: "宛平城东关社区",
                code: "010",
            },
            VillageCode {
                name: "新城社区",
                code: "011",
            },
            VillageCode {
                name: "永合庄村",
                code: "012",
            },
            VillageCode {
                name: "北天堂村",
                code: "013",
            },
            VillageCode {
                name: "卢沟桥村",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "马家堡街道",
        code: "015",
        villages: &[
            VillageCode {
                name: "嘉园一里社区",
                code: "001",
            },
            VillageCode {
                name: "嘉园二里社区",
                code: "002",
            },
            VillageCode {
                name: "嘉园三里社区",
                code: "003",
            },
            VillageCode {
                name: "西里第一社区",
                code: "004",
            },
            VillageCode {
                name: "西里第二社区",
                code: "005",
            },
            VillageCode {
                name: "西里第三社区",
                code: "006",
            },
            VillageCode {
                name: "双晨社区",
                code: "007",
            },
            VillageCode {
                name: "角门东里西社区",
                code: "008",
            },
            VillageCode {
                name: "晨宇社区",
                code: "009",
            },
            VillageCode {
                name: "欣汇社区",
                code: "010",
            },
            VillageCode {
                name: "富卓苑社区",
                code: "011",
            },
            VillageCode {
                name: "玉安园社区",
                code: "012",
            },
            VillageCode {
                name: "城南嘉园社区",
                code: "013",
            },
            VillageCode {
                name: "枫竹苑社区",
                code: "014",
            },
            VillageCode {
                name: "星河苑社区",
                code: "015",
            },
            VillageCode {
                name: "镇国寺社区",
                code: "016",
            },
            VillageCode {
                name: "北甲地社区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "和义街道",
        code: "016",
        villages: &[
            VillageCode {
                name: "和义东里第一社区",
                code: "001",
            },
            VillageCode {
                name: "和义东里第二社区",
                code: "002",
            },
            VillageCode {
                name: "和义东里第三社区",
                code: "003",
            },
            VillageCode {
                name: "南苑北里第一社区",
                code: "004",
            },
            VillageCode {
                name: "南苑北里第二社区",
                code: "005",
            },
            VillageCode {
                name: "和义西里第一社区",
                code: "006",
            },
            VillageCode {
                name: "和义西里第二社区",
                code: "007",
            },
            VillageCode {
                name: "和义西里第三社区",
                code: "008",
            },
            VillageCode {
                name: "久敬庄社区",
                code: "009",
            },
            VillageCode {
                name: "大红门锦苑二社区",
                code: "010",
            },
            VillageCode {
                name: "大红门锦苑一社区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "卢沟桥街道",
        code: "017",
        villages: &[
            VillageCode {
                name: "同馨家园社区",
                code: "001",
            },
            VillageCode {
                name: "小瓦窑西里社区",
                code: "002",
            },
            VillageCode {
                name: "小屯社区",
                code: "003",
            },
            VillageCode {
                name: "大瓦窑社区",
                code: "004",
            },
            VillageCode {
                name: "假日万恒社区",
                code: "005",
            },
            VillageCode {
                name: "美域家园社区",
                code: "006",
            },
            VillageCode {
                name: "建邦枫景社区",
                code: "007",
            },
            VillageCode {
                name: "丰泽家园社区",
                code: "008",
            },
            VillageCode {
                name: "假日风景社区",
                code: "009",
            },
            VillageCode {
                name: "康馨家园北区社区",
                code: "010",
            },
            VillageCode {
                name: "康馨家园南区社区",
                code: "011",
            },
            VillageCode {
                name: "莲玉嘉园社区",
                code: "012",
            },
            VillageCode {
                name: "春风雅筑社区",
                code: "013",
            },
            VillageCode {
                name: "兆丰园社区",
                code: "014",
            },
            VillageCode {
                name: "大井村",
                code: "015",
            },
            VillageCode {
                name: "小屯村",
                code: "016",
            },
            VillageCode {
                name: "小瓦窑村",
                code: "017",
            },
            VillageCode {
                name: "张仪村",
                code: "018",
            },
            VillageCode {
                name: "郭庄子村",
                code: "019",
            },
            VillageCode {
                name: "大瓦窑村",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "花乡街道",
        code: "018",
        villages: &[
            VillageCode {
                name: "天伦锦城社区",
                code: "001",
            },
            VillageCode {
                name: "郭公庄幸福家园社区",
                code: "002",
            },
            VillageCode {
                name: "三乐花园社区",
                code: "003",
            },
            VillageCode {
                name: "银地第二社区",
                code: "004",
            },
            VillageCode {
                name: "明春苑社区",
                code: "005",
            },
            VillageCode {
                name: "银地社区",
                code: "006",
            },
            VillageCode {
                name: "康润南里社区",
                code: "007",
            },
            VillageCode {
                name: "康润东里社区",
                code: "008",
            },
            VillageCode {
                name: "康润西里社区",
                code: "009",
            },
            VillageCode {
                name: "郭公庄南街社区",
                code: "010",
            },
            VillageCode {
                name: "郭公庄中街社区",
                code: "011",
            },
            VillageCode {
                name: "郭公庄北街社区",
                code: "012",
            },
            VillageCode {
                name: "新发地村",
                code: "013",
            },
            VillageCode {
                name: "郭公庄村",
                code: "014",
            },
            VillageCode {
                name: "高立庄村",
                code: "015",
            },
            VillageCode {
                name: "羊坊村",
                code: "016",
            },
            VillageCode {
                name: "葆台村",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "成寿寺街道",
        code: "019",
        villages: &[
            VillageCode {
                name: "四方景园社区",
                code: "001",
            },
            VillageCode {
                name: "华苇景苑社区",
                code: "002",
            },
            VillageCode {
                name: "四方景园第二社区",
                code: "003",
            },
            VillageCode {
                name: "成寿寺社区",
                code: "004",
            },
            VillageCode {
                name: "成仪路社区",
                code: "005",
            },
            VillageCode {
                name: "方南家园社区",
                code: "006",
            },
            VillageCode {
                name: "瑞成街社区",
                code: "007",
            },
            VillageCode {
                name: "分中寺村",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "石榴庄街道",
        code: "020",
        villages: &[
            VillageCode {
                name: "石榴园北里第一社区",
                code: "001",
            },
            VillageCode {
                name: "石榴园北里第二社区",
                code: "002",
            },
            VillageCode {
                name: "石榴园南里第一社区",
                code: "003",
            },
            VillageCode {
                name: "石榴园南里第二社区",
                code: "004",
            },
            VillageCode {
                name: "石榴庄东街社区",
                code: "005",
            },
            VillageCode {
                name: "彩虹城社区",
                code: "006",
            },
            VillageCode {
                name: "世华水岸社区",
                code: "007",
            },
            VillageCode {
                name: "石榴庄东街第二社区",
                code: "008",
            },
            VillageCode {
                name: "顶秀欣园社区",
                code: "009",
            },
            VillageCode {
                name: "政馨家园社区",
                code: "010",
            },
            VillageCode {
                name: "红狮家园社区",
                code: "011",
            },
            VillageCode {
                name: "宋家庄社区",
                code: "012",
            },
            VillageCode {
                name: "鑫兆雅园社区",
                code: "013",
            },
            VillageCode {
                name: "双石一社区",
                code: "014",
            },
            VillageCode {
                name: "双石二社区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "玉泉营街道",
        code: "021",
        villages: &[
            VillageCode {
                name: "草桥欣园第一社区",
                code: "001",
            },
            VillageCode {
                name: "草桥欣园第二社区",
                code: "002",
            },
            VillageCode {
                name: "纪家庙社区",
                code: "003",
            },
            VillageCode {
                name: "青秀城社区",
                code: "004",
            },
            VillageCode {
                name: "万柳园社区",
                code: "005",
            },
            VillageCode {
                name: "万柳西园社区",
                code: "006",
            },
            VillageCode {
                name: "风格与林社区",
                code: "007",
            },
            VillageCode {
                name: "草桥社区",
                code: "008",
            },
            VillageCode {
                name: "草桥欣园第三社区",
                code: "009",
            },
            VillageCode {
                name: "黄土岗村",
                code: "010",
            },
            VillageCode {
                name: "纪家庙村",
                code: "011",
            },
            VillageCode {
                name: "草桥村",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "看丹街道",
        code: "022",
        villages: &[
            VillageCode {
                name: "白盆窑天兴家园社区",
                code: "001",
            },
            VillageCode {
                name: "四合欣园社区",
                code: "002",
            },
            VillageCode {
                name: "看丹社区",
                code: "003",
            },
            VillageCode {
                name: "富丰园社区",
                code: "004",
            },
            VillageCode {
                name: "育仁里社区",
                code: "005",
            },
            VillageCode {
                name: "电力机社区",
                code: "006",
            },
            VillageCode {
                name: "丰西社区",
                code: "007",
            },
            VillageCode {
                name: "科学城第一社区",
                code: "008",
            },
            VillageCode {
                name: "科学城第二社区",
                code: "009",
            },
            VillageCode {
                name: "韩庄子第一社区",
                code: "010",
            },
            VillageCode {
                name: "韩庄子第二社区",
                code: "011",
            },
            VillageCode {
                name: "中海九浩苑社区",
                code: "012",
            },
            VillageCode {
                name: "富锦嘉园社区",
                code: "013",
            },
            VillageCode {
                name: "四季家园社区",
                code: "014",
            },
            VillageCode {
                name: "首开华润城社区",
                code: "015",
            },
            VillageCode {
                name: "南开西里社区",
                code: "016",
            },
            VillageCode {
                name: "看丹村",
                code: "017",
            },
            VillageCode {
                name: "榆树庄村",
                code: "018",
            },
            VillageCode {
                name: "六圈村",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "五里店街道",
        code: "023",
        villages: &[
            VillageCode {
                name: "程庄路16号院社区",
                code: "001",
            },
            VillageCode {
                name: "东安街头条十九号院社区",
                code: "002",
            },
            VillageCode {
                name: "六十三号院社区",
                code: "003",
            },
            VillageCode {
                name: "北大地十六号院社区",
                code: "004",
            },
            VillageCode {
                name: "北大地西区社区",
                code: "005",
            },
            VillageCode {
                name: "东安街头条社区",
                code: "006",
            },
            VillageCode {
                name: "东安街社区",
                code: "007",
            },
            VillageCode {
                name: "彩虹南社区",
                code: "008",
            },
            VillageCode {
                name: "彩虹北社区",
                code: "009",
            },
            VillageCode {
                name: "五里店第一社区",
                code: "010",
            },
            VillageCode {
                name: "五里店第二社区",
                code: "011",
            },
            VillageCode {
                name: "丰西路社区",
                code: "012",
            },
            VillageCode {
                name: "油泵厂社区",
                code: "013",
            },
            VillageCode {
                name: "大井社区",
                code: "014",
            },
            VillageCode {
                name: "丰体时代花园社区",
                code: "015",
            },
            VillageCode {
                name: "和风四季社区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "青塔街道",
        code: "024",
        villages: &[
            VillageCode {
                name: "岳各庄社区",
                code: "001",
            },
            VillageCode {
                name: "青塔东里社区",
                code: "002",
            },
            VillageCode {
                name: "青塔西里社区",
                code: "003",
            },
            VillageCode {
                name: "蔚园社区",
                code: "004",
            },
            VillageCode {
                name: "秀园社区",
                code: "005",
            },
            VillageCode {
                name: "芳园社区",
                code: "006",
            },
            VillageCode {
                name: "春园社区",
                code: "007",
            },
            VillageCode {
                name: "长安新城第一社区",
                code: "008",
            },
            VillageCode {
                name: "民岳家园社区",
                code: "009",
            },
            VillageCode {
                name: "长安新城第二社区",
                code: "010",
            },
            VillageCode {
                name: "汇锦苑社区",
                code: "011",
            },
            VillageCode {
                name: "珠江紫台社区",
                code: "012",
            },
            VillageCode {
                name: "阅园社区",
                code: "013",
            },
            VillageCode {
                name: "西府景园社区",
                code: "014",
            },
            VillageCode {
                name: "郑常庄村",
                code: "015",
            },
            VillageCode {
                name: "岳各庄村",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "北宫镇",
        code: "025",
        villages: &[
            VillageCode {
                name: "得秀社区",
                code: "001",
            },
            VillageCode {
                name: "槐树岭社区",
                code: "002",
            },
            VillageCode {
                name: "二七车辆厂社区",
                code: "003",
            },
            VillageCode {
                name: "张郭庄社区",
                code: "004",
            },
            VillageCode {
                name: "芦井社区",
                code: "005",
            },
            VillageCode {
                name: "装甲兵工程学院社区",
                code: "006",
            },
            VillageCode {
                name: "杜家坎社区",
                code: "007",
            },
            VillageCode {
                name: "装技所社区",
                code: "008",
            },
            VillageCode {
                name: "红山郡社区",
                code: "009",
            },
            VillageCode {
                name: "大灰厂社区",
                code: "010",
            },
            VillageCode {
                name: "张郭庄村",
                code: "011",
            },
            VillageCode {
                name: "东河沿村",
                code: "012",
            },
            VillageCode {
                name: "辛庄村",
                code: "013",
            },
            VillageCode {
                name: "大灰厂村",
                code: "014",
            },
            VillageCode {
                name: "李家峪村",
                code: "015",
            },
            VillageCode {
                name: "太子峪村",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "王佐镇",
        code: "026",
        villages: &[
            VillageCode {
                name: "南宫雅苑社区",
                code: "001",
            },
            VillageCode {
                name: "山语城社区",
                code: "002",
            },
            VillageCode {
                name: "翡翠山社区",
                code: "003",
            },
            VillageCode {
                name: "南宫景苑社区",
                code: "004",
            },
            VillageCode {
                name: "西庄店村",
                code: "005",
            },
            VillageCode {
                name: "沙锅村",
                code: "006",
            },
            VillageCode {
                name: "怪村",
                code: "007",
            },
            VillageCode {
                name: "魏各庄村",
                code: "008",
            },
            VillageCode {
                name: "西王佐村",
                code: "009",
            },
            VillageCode {
                name: "南宫村",
                code: "010",
            },
            VillageCode {
                name: "庄户村",
                code: "011",
            },
            VillageCode {
                name: "佃起村",
                code: "012",
            },
        ],
    },
];

static TOWNS_BP_005: [TownCode; 9] = [
    TownCode {
        name: "八宝山街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "玉泉路西社区",
                code: "001",
            },
            VillageCode {
                name: "电子科技情报研究所社区",
                code: "002",
            },
            VillageCode {
                name: "瑞达社区",
                code: "003",
            },
            VillageCode {
                name: "中铁建设有限公司社区",
                code: "004",
            },
            VillageCode {
                name: "鲁谷住宅社区",
                code: "005",
            },
            VillageCode {
                name: "四季园社区",
                code: "006",
            },
            VillageCode {
                name: "永乐东小区北社区",
                code: "007",
            },
            VillageCode {
                name: "永乐东小区南社区",
                code: "008",
            },
            VillageCode {
                name: "三山园社区",
                code: "009",
            },
            VillageCode {
                name: "玉泉西里西社区",
                code: "010",
            },
            VillageCode {
                name: "玉泉西里北社区",
                code: "011",
            },
            VillageCode {
                name: "玉泉西里中社区",
                code: "012",
            },
            VillageCode {
                name: "玉泉西里南社区",
                code: "013",
            },
            VillageCode {
                name: "沁山水北社区",
                code: "014",
            },
            VillageCode {
                name: "沁山水南社区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "老山街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "高能所社区",
                code: "001",
            },
            VillageCode {
                name: "中国科学院大学社区",
                code: "002",
            },
            VillageCode {
                name: "玉泉西路社区",
                code: "003",
            },
            VillageCode {
                name: "何家坟社区",
                code: "004",
            },
            VillageCode {
                name: "老山东里北社区",
                code: "005",
            },
            VillageCode {
                name: "老山东里社区",
                code: "006",
            },
            VillageCode {
                name: "老山东里南社区",
                code: "007",
            },
            VillageCode {
                name: "老山西里社区",
                code: "008",
            },
            VillageCode {
                name: "十一号院社区",
                code: "009",
            },
            VillageCode {
                name: "翠谷玉景苑社区",
                code: "010",
            },
            VillageCode {
                name: "京源路社区",
                code: "011",
            },
            VillageCode {
                name: "玉泉北里二区第一社区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "八角街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "八角北里社区",
                code: "001",
            },
            VillageCode {
                name: "八角中里社区",
                code: "002",
            },
            VillageCode {
                name: "建钢南里社区",
                code: "003",
            },
            VillageCode {
                name: "古城南里社区",
                code: "004",
            },
            VillageCode {
                name: "八角南路社区",
                code: "005",
            },
            VillageCode {
                name: "古城南路社区",
                code: "006",
            },
            VillageCode {
                name: "八角路社区",
                code: "007",
            },
            VillageCode {
                name: "公园北社区",
                code: "008",
            },
            VillageCode {
                name: "八角北路社区",
                code: "009",
            },
            VillageCode {
                name: "八角北路特钢社区",
                code: "010",
            },
            VillageCode {
                name: "杨庄南区社区",
                code: "011",
            },
            VillageCode {
                name: "杨庄中区社区",
                code: "012",
            },
            VillageCode {
                name: "八角南里社区",
                code: "013",
            },
            VillageCode {
                name: "地铁古城家园社区",
                code: "014",
            },
            VillageCode {
                name: "杨庄北区社区",
                code: "015",
            },
            VillageCode {
                name: "黄南苑社区",
                code: "016",
            },
            VillageCode {
                name: "时代花园社区",
                code: "017",
            },
            VillageCode {
                name: "八角景阳东街第一社区",
                code: "018",
            },
            VillageCode {
                name: "八角景阳东街第二社区",
                code: "019",
            },
            VillageCode {
                name: "八角景阳东街第三社区",
                code: "020",
            },
            VillageCode {
                name: "体育场南路社区",
                code: "021",
            },
            VillageCode {
                name: "杨庄北区第二社区",
                code: "022",
            },
            VillageCode {
                name: "体育场西街社区",
                code: "023",
            },
            VillageCode {
                name: "景颂街社区",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "古城街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "北小区社区",
                code: "001",
            },
            VillageCode {
                name: "南路东社区",
                code: "002",
            },
            VillageCode {
                name: "南路西社区",
                code: "003",
            },
            VillageCode {
                name: "环铁社区",
                code: "004",
            },
            VillageCode {
                name: "天翔社区",
                code: "005",
            },
            VillageCode {
                name: "八千平社区",
                code: "006",
            },
            VillageCode {
                name: "古城路社区",
                code: "007",
            },
            VillageCode {
                name: "古城特钢社区",
                code: "008",
            },
            VillageCode {
                name: "十万平社区",
                code: "009",
            },
            VillageCode {
                name: "水泥厂社区",
                code: "010",
            },
            VillageCode {
                name: "老古城东社区",
                code: "011",
            },
            VillageCode {
                name: "老古城西社区",
                code: "012",
            },
            VillageCode {
                name: "滨和园燕堤南路社区",
                code: "013",
            },
            VillageCode {
                name: "滨和园燕堤西街社区",
                code: "014",
            },
            VillageCode {
                name: "滨和园燕堤中街社区",
                code: "015",
            },
            VillageCode {
                name: "老古城南社区",
                code: "016",
            },
            VillageCode {
                name: "天和街社区",
                code: "017",
            },
            VillageCode {
                name: "北辛安第一社区",
                code: "018",
            },
            VillageCode {
                name: "北辛安第二社区",
                code: "019",
            },
            VillageCode {
                name: "古城西路社区",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "苹果园街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "西黄村社区",
                code: "001",
            },
            VillageCode {
                name: "下庄社区",
                code: "002",
            },
            VillageCode {
                name: "八大处社区",
                code: "003",
            },
            VillageCode {
                name: "西井社区",
                code: "004",
            },
            VillageCode {
                name: "海特花园第三社区",
                code: "005",
            },
            VillageCode {
                name: "苹一社区",
                code: "006",
            },
            VillageCode {
                name: "边府社区",
                code: "007",
            },
            VillageCode {
                name: "琅山村社区",
                code: "008",
            },
            VillageCode {
                name: "苹三社区",
                code: "009",
            },
            VillageCode {
                name: "苹二社区",
                code: "010",
            },
            VillageCode {
                name: "苹四社区",
                code: "011",
            },
            VillageCode {
                name: "军区装备部大院社区",
                code: "012",
            },
            VillageCode {
                name: "海特花园第一社区",
                code: "013",
            },
            VillageCode {
                name: "海特花园第二社区",
                code: "014",
            },
            VillageCode {
                name: "西黄新村社区",
                code: "015",
            },
            VillageCode {
                name: "西山枫林第一社区",
                code: "016",
            },
            VillageCode {
                name: "军区大院第一社区",
                code: "017",
            },
            VillageCode {
                name: "西山枫林第二社区",
                code: "018",
            },
            VillageCode {
                name: "西黄新村东里社区",
                code: "019",
            },
            VillageCode {
                name: "西黄新村西里社区",
                code: "020",
            },
            VillageCode {
                name: "东下庄社区",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "金顶街街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "金顶街四区社区",
                code: "001",
            },
            VillageCode {
                name: "金顶街一区社区",
                code: "002",
            },
            VillageCode {
                name: "赵山社区",
                code: "003",
            },
            VillageCode {
                name: "西福村社区",
                code: "004",
            },
            VillageCode {
                name: "铸造村社区",
                code: "005",
            },
            VillageCode {
                name: "模式口东里社区",
                code: "006",
            },
            VillageCode {
                name: "模式口中里社区",
                code: "007",
            },
            VillageCode {
                name: "模式口南里社区",
                code: "008",
            },
            VillageCode {
                name: "模式口北里社区",
                code: "009",
            },
            VillageCode {
                name: "模式口村社区",
                code: "010",
            },
            VillageCode {
                name: "金顶街三区社区",
                code: "011",
            },
            VillageCode {
                name: "金顶街五区社区",
                code: "012",
            },
            VillageCode {
                name: "模式口西里南区社区",
                code: "013",
            },
            VillageCode {
                name: "模式口西里北区社区",
                code: "014",
            },
            VillageCode {
                name: "模式口西里中区社区",
                code: "015",
            },
            VillageCode {
                name: "金顶街二区社区",
                code: "016",
            },
            VillageCode {
                name: "铸造村二区社区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "广宁街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "东山社区",
                code: "001",
            },
            VillageCode {
                name: "新立街社区",
                code: "002",
            },
            VillageCode {
                name: "麻峪社区",
                code: "003",
            },
            VillageCode {
                name: "麻峪北社区",
                code: "004",
            },
            VillageCode {
                name: "高井路社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "五里坨街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "陆军机关军营社区",
                code: "001",
            },
            VillageCode {
                name: "黑石头社区",
                code: "002",
            },
            VillageCode {
                name: "西山机械厂社区",
                code: "003",
            },
            VillageCode {
                name: "高井社区",
                code: "004",
            },
            VillageCode {
                name: "隆恩寺社区",
                code: "005",
            },
            VillageCode {
                name: "东街社区",
                code: "006",
            },
            VillageCode {
                name: "红卫路社区",
                code: "007",
            },
            VillageCode {
                name: "南宫社区",
                code: "008",
            },
            VillageCode {
                name: "隆恩寺新区社区",
                code: "009",
            },
            VillageCode {
                name: "天翠阳光第一社区",
                code: "010",
            },
            VillageCode {
                name: "天翠阳光第二社区",
                code: "011",
            },
            VillageCode {
                name: "天翠阳光第三社区",
                code: "012",
            },
            VillageCode {
                name: "隆恩颐园社区",
                code: "013",
            },
            VillageCode {
                name: "南宫嘉园社区",
                code: "014",
            },
            VillageCode {
                name: "京西景园社区",
                code: "015",
            },
            VillageCode {
                name: "石府路第一社区",
                code: "016",
            },
            VillageCode {
                name: "石府路第二社区",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "鲁谷街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "双锦园社区",
                code: "001",
            },
            VillageCode {
                name: "永乐西北社区",
                code: "002",
            },
            VillageCode {
                name: "永乐西南社区",
                code: "003",
            },
            VillageCode {
                name: "久筑社区",
                code: "004",
            },
            VillageCode {
                name: "五芳园社区",
                code: "005",
            },
            VillageCode {
                name: "重聚园社区",
                code: "006",
            },
            VillageCode {
                name: "依翠园北社区",
                code: "007",
            },
            VillageCode {
                name: "依翠园南社区",
                code: "008",
            },
            VillageCode {
                name: "六合园北社区",
                code: "009",
            },
            VillageCode {
                name: "六合园南社区",
                code: "010",
            },
            VillageCode {
                name: "新华社社区",
                code: "011",
            },
            VillageCode {
                name: "七星园北社区",
                code: "012",
            },
            VillageCode {
                name: "七星园南社区",
                code: "013",
            },
            VillageCode {
                name: "衙门口东社区",
                code: "014",
            },
            VillageCode {
                name: "衙门口西社区",
                code: "015",
            },
            VillageCode {
                name: "衙门口南社区",
                code: "016",
            },
            VillageCode {
                name: "新岚大厦社区",
                code: "017",
            },
            VillageCode {
                name: "重兴园社区",
                code: "018",
            },
            VillageCode {
                name: "聚兴园社区",
                code: "019",
            },
            VillageCode {
                name: "碣石坪社区",
                code: "020",
            },
            VillageCode {
                name: "西厂社区",
                code: "021",
            },
            VillageCode {
                name: "京汉旭城社区",
                code: "022",
            },
        ],
    },
];

static TOWNS_BP_006: [TownCode; 29] = [
    TownCode {
        name: "万寿路街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "翠微路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "翠微路21号社区居委会",
                code: "002",
            },
            VillageCode {
                name: "复兴路20号社区居委会",
                code: "003",
            },
            VillageCode {
                name: "翠微南里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "翠微中里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "翠微北里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "翠微西里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "万寿路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "万寿路28号社区居委会",
                code: "009",
            },
            VillageCode {
                name: "复兴路22号社区居委会",
                code: "010",
            },
            VillageCode {
                name: "复兴路61号社区居委会",
                code: "011",
            },
            VillageCode {
                name: "万寿路8号社区居委会",
                code: "012",
            },
            VillageCode {
                name: "朱各庄10号社区居委会",
                code: "013",
            },
            VillageCode {
                name: "万寿路甲15号社区居委会",
                code: "014",
            },
            VillageCode {
                name: "朱各庄社区居委会",
                code: "015",
            },
            VillageCode {
                name: "万寿路1号社区居委会",
                code: "016",
            },
            VillageCode {
                name: "万寿路西街16号社区居委会",
                code: "017",
            },
            VillageCode {
                name: "复兴路24号社区居委会",
                code: "018",
            },
            VillageCode {
                name: "复兴路26号社区居委会",
                code: "019",
            },
            VillageCode {
                name: "复兴路28号社区居委会",
                code: "020",
            },
            VillageCode {
                name: "今日家园社区居委会",
                code: "021",
            },
            VillageCode {
                name: "太平路24号社区居委会",
                code: "022",
            },
            VillageCode {
                name: "太平路22号社区居委会",
                code: "023",
            },
            VillageCode {
                name: "复兴路32号社区居委会",
                code: "024",
            },
            VillageCode {
                name: "永定路东里社区居委会",
                code: "025",
            },
            VillageCode {
                name: "永定路西里社区居委会",
                code: "026",
            },
            VillageCode {
                name: "五棵松紫金长安社区居委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "永定路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "一街坊社区居委会",
                code: "001",
            },
            VillageCode {
                name: "三街坊东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "三街坊西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "五街坊社区居委会",
                code: "004",
            },
            VillageCode {
                name: "七街坊社区居委会",
                code: "005",
            },
            VillageCode {
                name: "九街坊社区居委会",
                code: "006",
            },
            VillageCode {
                name: "采石路7号社区居委会",
                code: "007",
            },
            VillageCode {
                name: "复兴路40号社区居委会",
                code: "008",
            },
            VillageCode {
                name: "复兴路83号社区居委会",
                code: "009",
            },
            VillageCode {
                name: "太平路27号社区居委会",
                code: "010",
            },
            VillageCode {
                name: "太平路44号社区居委会",
                code: "011",
            },
            VillageCode {
                name: "太平路46号社区居委会",
                code: "012",
            },
            VillageCode {
                name: "铁家坟社区居委会",
                code: "013",
            },
            VillageCode {
                name: "金沟河社区居委会",
                code: "014",
            },
            VillageCode {
                name: "金沟河路1号院社区居委会",
                code: "015",
            },
            VillageCode {
                name: "永金里社区居委会",
                code: "016",
            },
            VillageCode {
                name: "永定路26号院社区居委会",
                code: "017",
            },
            VillageCode {
                name: "阜石路第三社区居委会",
                code: "018",
            },
            VillageCode {
                name: "泽丰苑社区居委会",
                code: "019",
            },
            VillageCode {
                name: "二街坊社区居委会",
                code: "020",
            },
            VillageCode {
                name: "四街坊社区居委会",
                code: "021",
            },
            VillageCode {
                name: "六街坊社区居委会",
                code: "022",
            },
            VillageCode {
                name: "八街坊社区居委会",
                code: "023",
            },
            VillageCode {
                name: "新兴年代社区居委会",
                code: "024",
            },
            VillageCode {
                name: "永定路社区居委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "羊坊店街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "海军机关大院社区居委会",
                code: "001",
            },
            VillageCode {
                name: "会城门社区居委会",
                code: "002",
            },
            VillageCode {
                name: "电信局社区居委会",
                code: "003",
            },
            VillageCode {
                name: "有色设计院社区居委会",
                code: "004",
            },
            VillageCode {
                name: "新华社皇亭子社区居委会",
                code: "005",
            },
            VillageCode {
                name: "铁东社区居委会",
                code: "006",
            },
            VillageCode {
                name: "铁西社区居委会",
                code: "007",
            },
            VillageCode {
                name: "水科院南院社区居委会",
                code: "008",
            },
            VillageCode {
                name: "翠微路第二社区居委会",
                code: "009",
            },
            VillageCode {
                name: "普惠南里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "永红社区居委会",
                code: "011",
            },
            VillageCode {
                name: "普惠寺社区居委会",
                code: "012",
            },
            VillageCode {
                name: "羊坊店社区居委会",
                code: "013",
            },
            VillageCode {
                name: "吴家场铁路5号院社区居委会",
                code: "014",
            },
            VillageCode {
                name: "复兴路乙4号院社区居委会",
                code: "015",
            },
            VillageCode {
                name: "乔建社区居委会",
                code: "016",
            },
            VillageCode {
                name: "颐源居社区居委会",
                code: "017",
            },
            VillageCode {
                name: "科技部社区居委会",
                code: "018",
            },
            VillageCode {
                name: "东风社区居委会",
                code: "019",
            },
            VillageCode {
                name: "小马厂社区居委会",
                code: "020",
            },
            VillageCode {
                name: "西木楼社区居委会",
                code: "021",
            },
            VillageCode {
                name: "茂林居社区居委会",
                code: "022",
            },
            VillageCode {
                name: "三住宅社区居委会",
                code: "023",
            },
            VillageCode {
                name: "什坊院一号院社区居委会",
                code: "024",
            },
            VillageCode {
                name: "军事博物馆社区居委会",
                code: "025",
            },
            VillageCode {
                name: "复兴路23号社区居委会",
                code: "026",
            },
            VillageCode {
                name: "莲花小区社区居委会",
                code: "027",
            },
            VillageCode {
                name: "吴家村路十号院社区居委会",
                code: "028",
            },
            VillageCode {
                name: "玉南路9号社区居委会",
                code: "029",
            },
            VillageCode {
                name: "沄沄国际社区居委会",
                code: "030",
            },
            VillageCode {
                name: "空军机关大院社区居委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "甘家口街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "阜南社区居委会",
                code: "001",
            },
            VillageCode {
                name: "白中社区居委会",
                code: "002",
            },
            VillageCode {
                name: "白堆子社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "花园村社区居委会",
                code: "005",
            },
            VillageCode {
                name: "四道口社区居委会",
                code: "006",
            },
            VillageCode {
                name: "水科院社区居委会",
                code: "007",
            },
            VillageCode {
                name: "机械院社区居委会",
                code: "008",
            },
            VillageCode {
                name: "航天社区居委会",
                code: "009",
            },
            VillageCode {
                name: "建设部社区居委会",
                code: "010",
            },
            VillageCode {
                name: "工商大学社区居委会",
                code: "011",
            },
            VillageCode {
                name: "工运社区居委会",
                code: "012",
            },
            VillageCode {
                name: "空军总医院社区居委会",
                code: "013",
            },
            VillageCode {
                name: "中纺社区居委会",
                code: "014",
            },
            VillageCode {
                name: "西钓社区居委会",
                code: "015",
            },
            VillageCode {
                name: "进口社区居委会",
                code: "016",
            },
            VillageCode {
                name: "潘庄社区居委会",
                code: "017",
            },
            VillageCode {
                name: "甘东社区居委会",
                code: "018",
            },
            VillageCode {
                name: "阜北社区居委会",
                code: "019",
            },
            VillageCode {
                name: "海军总医院社区居委会",
                code: "020",
            },
            VillageCode {
                name: "西三环社区居委会",
                code: "021",
            },
            VillageCode {
                name: "公安部一所社区居委会",
                code: "022",
            },
            VillageCode {
                name: "西钓嘉园社区居委会",
                code: "023",
            },
            VillageCode {
                name: "增光社区居委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "八里庄街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "核二院社区居委会",
                code: "001",
            },
            VillageCode {
                name: "核情报所社区居委会",
                code: "002",
            },
            VillageCode {
                name: "三0四医院社区居委会",
                code: "003",
            },
            VillageCode {
                name: "水文社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东八里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "首师大社区居委会",
                code: "006",
            },
            VillageCode {
                name: "北洼路第三社区居委会",
                code: "007",
            },
            VillageCode {
                name: "首师大北校园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "北玲珑巷社区居委会",
                code: "009",
            },
            VillageCode {
                name: "中海雅园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "鼎力社区居委会",
                code: "011",
            },
            VillageCode {
                name: "双紫园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "八宝庄社区居委会",
                code: "013",
            },
            VillageCode {
                name: "肿瘤医院社区居委会",
                code: "014",
            },
            VillageCode {
                name: "中化社区居委会",
                code: "015",
            },
            VillageCode {
                name: "定慧东里社区居委会",
                code: "016",
            },
            VillageCode {
                name: "西八里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "八里庄北里社区居委会",
                code: "018",
            },
            VillageCode {
                name: "恩济里社区居委会",
                code: "019",
            },
            VillageCode {
                name: "定慧北里第一社区居委会",
                code: "020",
            },
            VillageCode {
                name: "定慧北里第二社区居委会",
                code: "021",
            },
            VillageCode {
                name: "恩济庄社区居委会",
                code: "022",
            },
            VillageCode {
                name: "永安东里社区居委会",
                code: "023",
            },
            VillageCode {
                name: "徐庄社区居委会",
                code: "024",
            },
            VillageCode {
                name: "五路社区居委会",
                code: "025",
            },
            VillageCode {
                name: "美丽园社区居委会",
                code: "026",
            },
            VillageCode {
                name: "北京印象社区居委会",
                code: "027",
            },
            VillageCode {
                name: "世纪新景园社区居委会",
                code: "028",
            },
            VillageCode {
                name: "裕泽园社区居委会",
                code: "029",
            },
            VillageCode {
                name: "颐慧佳园社区居委会",
                code: "030",
            },
            VillageCode {
                name: "定慧西里社区居委会",
                code: "031",
            },
            VillageCode {
                name: "五福玲珑居社区居委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "紫竹院街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "魏公村北区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "魏公村南区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "韦伯豪社区居委会",
                code: "003",
            },
            VillageCode {
                name: "法华寺社区居委会",
                code: "004",
            },
            VillageCode {
                name: "万寿寺社区居委会",
                code: "005",
            },
            VillageCode {
                name: "紫竹社区居委会",
                code: "006",
            },
            VillageCode {
                name: "三虎桥社区居委会",
                code: "007",
            },
            VillageCode {
                name: "北洼路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "厂洼第一社区居委会",
                code: "009",
            },
            VillageCode {
                name: "厂洼第二社区居委会",
                code: "010",
            },
            VillageCode {
                name: "车道沟社区居委会",
                code: "011",
            },
            VillageCode {
                name: "车道沟南里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "北京理工大学社区居委会",
                code: "013",
            },
            VillageCode {
                name: "中央民族大学社区居委会",
                code: "014",
            },
            VillageCode {
                name: "北京外国语大学社区居委会",
                code: "015",
            },
            VillageCode {
                name: "中国青年政治学院社区居委会",
                code: "016",
            },
            VillageCode {
                name: "航天部五院社区居委会",
                code: "017",
            },
            VillageCode {
                name: "万寿山庄社区居委会",
                code: "018",
            },
            VillageCode {
                name: "兵器工业机关社区居委会",
                code: "019",
            },
            VillageCode {
                name: "军乐团社区居委会",
                code: "020",
            },
            VillageCode {
                name: "西苑饭店西区社区居委会",
                code: "021",
            },
            VillageCode {
                name: "厂洼社区居委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "北下关街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "五塔寺社区居委会",
                code: "001",
            },
            VillageCode {
                name: "头堆社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北京动物园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "钢铁研究总院社区居委会",
                code: "004",
            },
            VillageCode {
                name: "上园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "大慧寺社区居委会",
                code: "006",
            },
            VillageCode {
                name: "大柳树社区居委会",
                code: "007",
            },
            VillageCode {
                name: "大柳树北社区居委会",
                code: "008",
            },
            VillageCode {
                name: "中国气象局社区居委会",
                code: "009",
            },
            VillageCode {
                name: "中关村南大街40号社区居委会",
                code: "010",
            },
            VillageCode {
                name: "净土寺社区居委会",
                code: "011",
            },
            VillageCode {
                name: "娘娘庙社区居委会",
                code: "012",
            },
            VillageCode {
                name: "广通苑社区居委会",
                code: "013",
            },
            VillageCode {
                name: "北京交通大学社区居委会",
                code: "014",
            },
            VillageCode {
                name: "中国铁道科学研究院社区居委会",
                code: "015",
            },
            VillageCode {
                name: "南里社区居委会",
                code: "016",
            },
            VillageCode {
                name: "皂君西里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "南里二区社区居委会",
                code: "018",
            },
            VillageCode {
                name: "农影社区居委会",
                code: "019",
            },
            VillageCode {
                name: "中监所社区居委会",
                code: "020",
            },
            VillageCode {
                name: "中国农业科学院社区居委会",
                code: "021",
            },
            VillageCode {
                name: "国防大学军事文化学院社区居委会",
                code: "022",
            },
            VillageCode {
                name: "青云南区社区居委会",
                code: "023",
            },
            VillageCode {
                name: "皂君庙社区居委会",
                code: "024",
            },
            VillageCode {
                name: "皂君东里社区居委会",
                code: "025",
            },
            VillageCode {
                name: "中央财经大学社区居委会",
                code: "026",
            },
            VillageCode {
                name: "大钟寺社区居委会",
                code: "027",
            },
            VillageCode {
                name: "皂君庙南路社区居委会",
                code: "028",
            },
            VillageCode {
                name: "卫生部社区居委会",
                code: "029",
            },
            VillageCode {
                name: "海洋环境预报中心社区居委会",
                code: "030",
            },
            VillageCode {
                name: "交大嘉园社区居委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "北太平庄街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "太平湖社区居委会",
                code: "001",
            },
            VillageCode {
                name: "红联村社区居委会",
                code: "002",
            },
            VillageCode {
                name: "志强南园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新外大街23号院社区居委会",
                code: "004",
            },
            VillageCode {
                name: "红联东村社区居委会",
                code: "005",
            },
            VillageCode {
                name: "文慧园路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "志强北园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "学院南路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "蓟门里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "罗庄社区居委会",
                code: "010",
            },
            VillageCode {
                name: "太月园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "首都体院社区居委会",
                code: "012",
            },
            VillageCode {
                name: "索家坟社区居委会",
                code: "013",
            },
            VillageCode {
                name: "红联北村社区居委会",
                code: "014",
            },
            VillageCode {
                name: "今典花园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "学院南路32号社区居委会",
                code: "016",
            },
            VillageCode {
                name: "邮电大学社区居委会",
                code: "017",
            },
            VillageCode {
                name: "北太平庄社区居委会",
                code: "018",
            },
            VillageCode {
                name: "北三环中路40号社区居委会",
                code: "019",
            },
            VillageCode {
                name: "师范大学社区居委会",
                code: "020",
            },
            VillageCode {
                name: "冶建院社区居委会",
                code: "021",
            },
            VillageCode {
                name: "明光村社区居委会",
                code: "022",
            },
            VillageCode {
                name: "明光北里社区居委会",
                code: "023",
            },
            VillageCode {
                name: "时代之光社区居委会",
                code: "024",
            },
            VillageCode {
                name: "政法大院社区居委会",
                code: "025",
            },
            VillageCode {
                name: "明光村小区社区居委会",
                code: "026",
            },
            VillageCode {
                name: "文慧园社区居委会",
                code: "027",
            },
            VillageCode {
                name: "罗庄东里社区居委会",
                code: "028",
            },
            VillageCode {
                name: "天兆家园社区居委会",
                code: "029",
            },
            VillageCode {
                name: "锦秋知春社区居委会",
                code: "030",
            },
            VillageCode {
                name: "金晖远洋社区居委会",
                code: "031",
            },
            VillageCode {
                name: "师大北路社区居委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "学院路街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "西王庄社区居委会",
                code: "001",
            },
            VillageCode {
                name: "六道口社区居委会",
                code: "002",
            },
            VillageCode {
                name: "志新社区居委会",
                code: "003",
            },
            VillageCode {
                name: "二里庄社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东王庄社区居委会",
                code: "005",
            },
            VillageCode {
                name: "学知园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "地大第二社区居委会",
                code: "007",
            },
            VillageCode {
                name: "健翔园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "建清园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "地大第一社区居委会",
                code: "010",
            },
            VillageCode {
                name: "北科大社区居委会",
                code: "011",
            },
            VillageCode {
                name: "石油大院社区居委会",
                code: "012",
            },
            VillageCode {
                name: "中国农业大学东校区社区居委会",
                code: "013",
            },
            VillageCode {
                name: "语言大学社区居委会",
                code: "014",
            },
            VillageCode {
                name: "石科院社区居委会",
                code: "015",
            },
            VillageCode {
                name: "十五所社区居委会",
                code: "016",
            },
            VillageCode {
                name: "林业大学社区居委会",
                code: "017",
            },
            VillageCode {
                name: "静淑苑社区居委会",
                code: "018",
            },
            VillageCode {
                name: "城建四社区居委会",
                code: "019",
            },
            VillageCode {
                name: "768厂社区居委会",
                code: "020",
            },
            VillageCode {
                name: "中国矿业大学（北京）社区居委会",
                code: "021",
            },
            VillageCode {
                name: "中科院社区居委会",
                code: "022",
            },
            VillageCode {
                name: "二里庄干休所社区居委会",
                code: "023",
            },
            VillageCode {
                name: "富润社区居委会",
                code: "024",
            },
            VillageCode {
                name: "逸成社区居委会",
                code: "025",
            },
            VillageCode {
                name: "展春园社区居委会",
                code: "026",
            },
            VillageCode {
                name: "城华清枫社区居委会",
                code: "027",
            },
            VillageCode {
                name: "学清苑社区居委会",
                code: "028",
            },
            VillageCode {
                name: "五道口嘉园社区居委会",
                code: "029",
            },
            VillageCode {
                name: "月清园社区居委会",
                code: "030",
            },
            VillageCode {
                name: "双泉嘉苑社区居委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "中关村街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "科源社区居委会",
                code: "001",
            },
            VillageCode {
                name: "科春社区居委会",
                code: "002",
            },
            VillageCode {
                name: "黄庄社区居委会",
                code: "003",
            },
            VillageCode {
                name: "科育社区居委会",
                code: "004",
            },
            VillageCode {
                name: "科馨社区居委会",
                code: "005",
            },
            VillageCode {
                name: "科煦社区居委会",
                code: "006",
            },
            VillageCode {
                name: "科汇社区居委会",
                code: "007",
            },
            VillageCode {
                name: "软件社区居委会",
                code: "008",
            },
            VillageCode {
                name: "空间社区居委会",
                code: "009",
            },
            VillageCode {
                name: "航天社区居委会",
                code: "010",
            },
            VillageCode {
                name: "东南社区居委会",
                code: "011",
            },
            VillageCode {
                name: "科星社区居委会",
                code: "012",
            },
            VillageCode {
                name: "新科祥园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "西里社区居委会",
                code: "014",
            },
            VillageCode {
                name: "东里南社区居委会",
                code: "015",
            },
            VillageCode {
                name: "东里北社区居委会",
                code: "016",
            },
            VillageCode {
                name: "北里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "知春里西社区居委会",
                code: "018",
            },
            VillageCode {
                name: "知春里社区居委会",
                code: "019",
            },
            VillageCode {
                name: "知春东里社区居委会",
                code: "020",
            },
            VillageCode {
                name: "白塔庵社区居委会",
                code: "021",
            },
            VillageCode {
                name: "红民村社区居委会",
                code: "022",
            },
            VillageCode {
                name: "青云北社区居委会",
                code: "023",
            },
            VillageCode {
                name: "太阳园社区居委会",
                code: "024",
            },
            VillageCode {
                name: "航勘社区居委会",
                code: "025",
            },
            VillageCode {
                name: "华清园社区居委会",
                code: "026",
            },
            VillageCode {
                name: "豪景佳苑社区居委会",
                code: "027",
            },
            VillageCode {
                name: "希格玛社区居委会",
                code: "028",
            },
            VillageCode {
                name: "航天五院社区居委会",
                code: "029",
            },
            VillageCode {
                name: "熙典华庭社区居委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "海淀街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "海淀路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "芙蓉里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "稻香园北社区居委会",
                code: "003",
            },
            VillageCode {
                name: "稻香园南社区居委会",
                code: "004",
            },
            VillageCode {
                name: "苏州街路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "倒座庙社区居委会",
                code: "006",
            },
            VillageCode {
                name: "合建楼社区居委会",
                code: "007",
            },
            VillageCode {
                name: "海淀南路北社区居委会",
                code: "008",
            },
            VillageCode {
                name: "海淀南路南社区居委会",
                code: "009",
            },
            VillageCode {
                name: "三环社区居委会",
                code: "010",
            },
            VillageCode {
                name: "友谊社区居委会",
                code: "011",
            },
            VillageCode {
                name: "三义庙社区居委会",
                code: "012",
            },
            VillageCode {
                name: "立新社区居委会",
                code: "013",
            },
            VillageCode {
                name: "小南庄社区居委会",
                code: "014",
            },
            VillageCode {
                name: "万泉庄南社区居委会",
                code: "015",
            },
            VillageCode {
                name: "新区社区居委会",
                code: "016",
            },
            VillageCode {
                name: "稻香园西里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "万泉庄北社区居委会",
                code: "018",
            },
            VillageCode {
                name: "紫金庄园社区居委会",
                code: "019",
            },
            VillageCode {
                name: "人大附中社区居委会",
                code: "020",
            },
            VillageCode {
                name: "人民大学社区居委会",
                code: "021",
            },
            VillageCode {
                name: "人大南社区居委会",
                code: "022",
            },
            VillageCode {
                name: "飞达社区居委会",
                code: "023",
            },
            VillageCode {
                name: "苏州桥西社区居委会",
                code: "024",
            },
            VillageCode {
                name: "航空港社区居委会",
                code: "025",
            },
            VillageCode {
                name: "光大锋尚园社区居委会",
                code: "026",
            },
            VillageCode {
                name: "汇新家园社区居委会",
                code: "027",
            },
            VillageCode {
                name: "新起点怡秀园社区居委会",
                code: "028",
            },
            VillageCode {
                name: "阳春新纪元社区居委会",
                code: "029",
            },
            VillageCode {
                name: "万泉新新家园社区居委会",
                code: "030",
            },
            VillageCode {
                name: "碧水云天社区居委会",
                code: "031",
            },
            VillageCode {
                name: "康桥蜂鸟园社区居委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "青龙桥街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "骚子营社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大有庄社区居委会",
                code: "002",
            },
            VillageCode {
                name: "福缘门社区居委会",
                code: "003",
            },
            VillageCode {
                name: "圆明园东里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "颐东苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "遗光寺社区居委会",
                code: "006",
            },
            VillageCode {
                name: "国防大学社区居委会",
                code: "007",
            },
            VillageCode {
                name: "韩家川大院社区居委会",
                code: "008",
            },
            VillageCode {
                name: "国际关系学院社区居委会",
                code: "009",
            },
            VillageCode {
                name: "中央党校社区居委会",
                code: "010",
            },
            VillageCode {
                name: "林业科学研究院社区居委会",
                code: "011",
            },
            VillageCode {
                name: "军事科学院社区居委会",
                code: "012",
            },
            VillageCode {
                name: "309医院社区居委会",
                code: "013",
            },
            VillageCode {
                name: "中国中医科学院西苑医院社区居委会",
                code: "014",
            },
            VillageCode {
                name: "颐和园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "中信所西苑小区社区居委会",
                code: "016",
            },
            VillageCode {
                name: "030院社区居委会",
                code: "017",
            },
            VillageCode {
                name: "厢红旗安河桥社区居委会",
                code: "018",
            },
            VillageCode {
                name: "水磨成府社区居委会",
                code: "019",
            },
            VillageCode {
                name: "西苑挂甲屯社区居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "清华园街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "东楼社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南楼社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西楼社区居委会",
                code: "003",
            },
            VillageCode {
                name: "北区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "中楼社区居委会",
                code: "005",
            },
            VillageCode {
                name: "西南社区居委会",
                code: "006",
            },
            VillageCode {
                name: "西北社区居委会",
                code: "007",
            },
            VillageCode {
                name: "蓝旗营社区居委会",
                code: "008",
            },
            VillageCode {
                name: "荷清苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "双清苑社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "燕园街道",
        code: "014",
        villages: &[
            VillageCode {
                name: "承泽园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "畅春园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "蔚秀园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "校内社区居委会",
                code: "004",
            },
            VillageCode {
                name: "中关园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "燕东园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "燕北园社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "香山街道",
        code: "015",
        villages: &[
            VillageCode {
                name: "香山第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "香山第二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "南植社区居委会",
                code: "003",
            },
            VillageCode {
                name: "红旗村社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西林社区居委会",
                code: "005",
            },
            VillageCode {
                name: "向阳新村社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "清河街道",
        code: "016",
        villages: &[
            VillageCode {
                name: "清河嘉园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "四街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "清河社区居委会",
                code: "003",
            },
            VillageCode {
                name: "安宁里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "火箭军社区居委会",
                code: "005",
            },
            VillageCode {
                name: "长城润滑油社区居委会",
                code: "006",
            },
            VillageCode {
                name: "安宁庄路11号院社区居委会",
                code: "007",
            },
            VillageCode {
                name: "安宁庄东路28号社区居委会",
                code: "008",
            },
            VillageCode {
                name: "美和园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "花园楼社区居委会",
                code: "010",
            },
            VillageCode {
                name: "毛纺南小区社区居委会",
                code: "011",
            },
            VillageCode {
                name: "毛纺北小区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "安宁东路社区居委会",
                code: "013",
            },
            VillageCode {
                name: "安宁北路社区居委会",
                code: "014",
            },
            VillageCode {
                name: "西二旗一里社区居委会",
                code: "015",
            },
            VillageCode {
                name: "安宁庄社区居委会",
                code: "016",
            },
            VillageCode {
                name: "怡美家园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "海清园社区居委会",
                code: "018",
            },
            VillageCode {
                name: "当代城市家园社区居委会",
                code: "019",
            },
            VillageCode {
                name: "清上园社区居委会",
                code: "020",
            },
            VillageCode {
                name: "力度家园社区居委会",
                code: "021",
            },
            VillageCode {
                name: "小营西路32号院社区居委会",
                code: "022",
            },
            VillageCode {
                name: "领秀硅谷社区居委会",
                code: "023",
            },
            VillageCode {
                name: "学府树家园第一社区居委会",
                code: "024",
            },
            VillageCode {
                name: "智学苑社区居委会",
                code: "025",
            },
            VillageCode {
                name: "领秀新硅谷社区居委会",
                code: "026",
            },
            VillageCode {
                name: "学府树家园第二社区居委会",
                code: "027",
            },
            VillageCode {
                name: "科技园社区居委会",
                code: "028",
            },
            VillageCode {
                name: "毛纺东路社区居委会",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "花园路街道",
        code: "017",
        villages: &[
            VillageCode {
                name: "北航社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北医社区居委会",
                code: "002",
            },
            VillageCode {
                name: "塔院干休所社区居委会",
                code: "003",
            },
            VillageCode {
                name: "邮科社区居委会",
                code: "004",
            },
            VillageCode {
                name: "塔院社区居委会",
                code: "005",
            },
            VillageCode {
                name: "志新村二号院社区居委会",
                code: "006",
            },
            VillageCode {
                name: "防化社区居委会",
                code: "007",
            },
            VillageCode {
                name: "北极寺大院社区居委会",
                code: "008",
            },
            VillageCode {
                name: "龙翔路社区居委会",
                code: "009",
            },
            VillageCode {
                name: "花园东路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "花园北路乙28号院社区居委会",
                code: "011",
            },
            VillageCode {
                name: "小关社区居委会",
                code: "012",
            },
            VillageCode {
                name: "北太平庄路社区居委会",
                code: "013",
            },
            VillageCode {
                name: "冠城园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "玉兰园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "月季园第二社区居委会",
                code: "016",
            },
            VillageCode {
                name: "北三环中路43号院社区居委会",
                code: "017",
            },
            VillageCode {
                name: "牤牛桥社区居委会",
                code: "018",
            },
            VillageCode {
                name: "1201社区居委会",
                code: "019",
            },
            VillageCode {
                name: "北三环中路69号院社区居委会",
                code: "020",
            },
            VillageCode {
                name: "北影社区居委会",
                code: "021",
            },
            VillageCode {
                name: "知春路17号院社区居委会",
                code: "022",
            },
            VillageCode {
                name: "西单商场社区居委会",
                code: "023",
            },
            VillageCode {
                name: "金尚嘉园社区居委会",
                code: "024",
            },
            VillageCode {
                name: "塔院四园社区居委会",
                code: "025",
            },
            VillageCode {
                name: "牡丹园社区居委会",
                code: "026",
            },
            VillageCode {
                name: "中央新影社区居委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "西三旗街道",
        code: "018",
        villages: &[
            VillageCode {
                name: "永泰园第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "清缘里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "建材西里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "机械学院联合社区居委会",
                code: "004",
            },
            VillageCode {
                name: "永泰西里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "宝盛里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "建材东里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "清缘东里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "悦秀园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "电科院社区居委会",
                code: "010",
            },
            VillageCode {
                name: "沁春家园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "育新花园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "冶金研究院社区居委会",
                code: "013",
            },
            VillageCode {
                name: "北新集团社区居委会",
                code: "014",
            },
            VillageCode {
                name: "9511工厂联合社区居委会",
                code: "015",
            },
            VillageCode {
                name: "永泰庄社区居委会",
                code: "016",
            },
            VillageCode {
                name: "清润家园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "永泰园第二社区居委会",
                code: "018",
            },
            VillageCode {
                name: "建材城联合社区居委会",
                code: "019",
            },
            VillageCode {
                name: "小营联合社区居委会",
                code: "020",
            },
            VillageCode {
                name: "怡清园社区居委会",
                code: "021",
            },
            VillageCode {
                name: "枫丹丽舍社区居委会",
                code: "022",
            },
            VillageCode {
                name: "知本时代社区居委会",
                code: "023",
            },
            VillageCode {
                name: "清景园社区居委会",
                code: "024",
            },
            VillageCode {
                name: "清缘西里社区居委会",
                code: "025",
            },
            VillageCode {
                name: "富力桃园社区居委会",
                code: "026",
            },
            VillageCode {
                name: "永泰东里社区居委会",
                code: "027",
            },
            VillageCode {
                name: "梧桐苑社区居委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "马连洼街道",
        code: "019",
        villages: &[
            VillageCode {
                name: "梅园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "菊园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "竹园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "天秀花园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "兰园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "水利总队社区居委会",
                code: "006",
            },
            VillageCode {
                name: "百草园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "农业大学社区居委会",
                code: "008",
            },
            VillageCode {
                name: "农科社区居委会",
                code: "009",
            },
            VillageCode {
                name: "63919部队社区居委会",
                code: "010",
            },
            VillageCode {
                name: "肖家河社区居委会",
                code: "011",
            },
            VillageCode {
                name: "百旺家苑社区居委会",
                code: "012",
            },
            VillageCode {
                name: "天秀古月园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "农大南路社区居委会",
                code: "014",
            },
            VillageCode {
                name: "西北旺社区居委会",
                code: "015",
            },
            VillageCode {
                name: "百旺茉莉园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "倚山庭苑社区居委会",
                code: "017",
            },
            VillageCode {
                name: "如缘居社区居委会",
                code: "018",
            },
            VillageCode {
                name: "芳怡园社区居委会",
                code: "019",
            },
            VillageCode {
                name: "正黄旗北大社区居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "田村路街道",
        code: "020",
        villages: &[
            VillageCode {
                name: "西木社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东营房社区居委会",
                code: "002",
            },
            VillageCode {
                name: "半壁店第一社区居委会",
                code: "003",
            },
            VillageCode {
                name: "半壁店第二社区居委会",
                code: "004",
            },
            VillageCode {
                name: "永达社区居委会",
                code: "005",
            },
            VillageCode {
                name: "阜石路第一社区居委会",
                code: "006",
            },
            VillageCode {
                name: "阜石路第四社区居委会",
                code: "007",
            },
            VillageCode {
                name: "永景园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "玉海园一里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "玉海园二里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "玉海园三里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "玉海园五里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "王致和社区居委会",
                code: "013",
            },
            VillageCode {
                name: "田村社区居委会",
                code: "014",
            },
            VillageCode {
                name: "山南社区居委会",
                code: "015",
            },
            VillageCode {
                name: "建西苑社区居委会",
                code: "016",
            },
            VillageCode {
                name: "玉阜嘉园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "乐府家园社区居委会",
                code: "018",
            },
            VillageCode {
                name: "幸福社区居委会",
                code: "019",
            },
            VillageCode {
                name: "兰德华庭社区居委会",
                code: "020",
            },
            VillageCode {
                name: "景宜里社区居委会",
                code: "021",
            },
            VillageCode {
                name: "瑞和园社区居委会",
                code: "022",
            },
            VillageCode {
                name: "武颐嘉园社区居委会",
                code: "023",
            },
            VillageCode {
                name: "玉泉北里社区居委会",
                code: "024",
            },
            VillageCode {
                name: "金玉府北里社区居委会",
                code: "025",
            },
            VillageCode {
                name: "金玉府南里社区居委会",
                code: "026",
            },
            VillageCode {
                name: "玉泉嘉园社区居委会",
                code: "027",
            },
            VillageCode {
                name: "天合家园社区居委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "上地街道",
        code: "021",
        villages: &[
            VillageCode {
                name: "上地东里第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "上地东里第二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "上地西里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东馨园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "马连洼北路1号院社区居委会",
                code: "005",
            },
            VillageCode {
                name: "体大颐清园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "树村社区居委会",
                code: "007",
            },
            VillageCode {
                name: "紫成嘉园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "万树园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "上地科技园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "上地南路社区居委会",
                code: "011",
            },
            VillageCode {
                name: "上地八一社区居委会",
                code: "012",
            },
            VillageCode {
                name: "博雅西园社区居委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "万柳地区",
        code: "022",
        villages: &[
            VillageCode {
                name: "六郎庄社区居委会",
                code: "001",
            },
            VillageCode {
                name: "功德寺社区居委会",
                code: "002",
            },
            VillageCode {
                name: "柳浪裕和社区居委会",
                code: "003",
            },
            VillageCode {
                name: "青龙桥村委会",
                code: "004",
            },
            VillageCode {
                name: "树村村委会",
                code: "005",
            },
            VillageCode {
                name: "万柳（集团）股份经济合作社六郎庄生活区",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "东升地区",
        code: "023",
        villages: &[
            VillageCode {
                name: "八家社区居委会",
                code: "001",
            },
            VillageCode {
                name: "前屯社区居委会",
                code: "002",
            },
            VillageCode {
                name: "马坊社区居委会",
                code: "003",
            },
            VillageCode {
                name: "奥北社区居委会",
                code: "004",
            },
            VillageCode {
                name: "文龙社区居委会",
                code: "005",
            },
            VillageCode {
                name: "龙岗社区居委会",
                code: "006",
            },
            VillageCode {
                name: "观林园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "龙樾社区居委会",
                code: "008",
            },
            VillageCode {
                name: "文晟社区居委会",
                code: "009",
            },
            VillageCode {
                name: "马坊村委会",
                code: "010",
            },
            VillageCode {
                name: "清河村委会",
                code: "011",
            },
            VillageCode {
                name: "小营股份经济合作社生活区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "曙光街道",
        code: "024",
        villages: &[
            VillageCode {
                name: "曙光花园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "世纪城东区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "世纪城西区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "农科院社区居委会",
                code: "004",
            },
            VillageCode {
                name: "武警总部蓝靛厂小区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "火器营第三社区居委会",
                code: "006",
            },
            VillageCode {
                name: "上河村社区居委会",
                code: "007",
            },
            VillageCode {
                name: "空军指挥学院社区居委会",
                code: "008",
            },
            VillageCode {
                name: "怡丽北园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "诚品建筑社区居委会",
                code: "010",
            },
            VillageCode {
                name: "世纪城晴雪园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "世纪城时雨园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "烟树园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "望塔园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "远大园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "晨月园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "金雅园社区居委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "温泉地区",
        code: "025",
        villages: &[
            VillageCode {
                name: "温泉社区居委会",
                code: "001",
            },
            VillageCode {
                name: "白家疃社区居委会",
                code: "002",
            },
            VillageCode {
                name: "航材院社区居委会",
                code: "003",
            },
            VillageCode {
                name: "三0四所社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西颐社区居委会",
                code: "005",
            },
            VillageCode {
                name: "颐阳一区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "颐阳二区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "杨庄社区居委会",
                code: "008",
            },
            VillageCode {
                name: "辰尚社区居委会",
                code: "009",
            },
            VillageCode {
                name: "凯盛家园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "温泉水岸家园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "环保园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "创客社区居委会",
                code: "013",
            },
            VillageCode {
                name: "东太社区居委会",
                code: "014",
            },
            VillageCode {
                name: "画眉山社区居委会",
                code: "015",
            },
            VillageCode {
                name: "温泉村委会",
                code: "016",
            },
            VillageCode {
                name: "白家疃村委会",
                code: "017",
            },
            VillageCode {
                name: "高里掌村股份经济合作社生活区",
                code: "018",
            },
            VillageCode {
                name: "辛庄村股份经济合作社生活区",
                code: "019",
            },
            VillageCode {
                name: "杨家庄村股份经济合作社生活区",
                code: "020",
            },
            VillageCode {
                name: "太舟坞村股份经济合作社生活区",
                code: "021",
            },
            VillageCode {
                name: "东埠头村股份经济合作社生活区",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "四季青地区",
        code: "026",
        villages: &[
            VillageCode {
                name: "门头村社区居委会",
                code: "001",
            },
            VillageCode {
                name: "巨山家园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "和泓四季社区居委会",
                code: "003",
            },
            VillageCode {
                name: "天香颐北里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西郊机场社区居委会",
                code: "005",
            },
            VillageCode {
                name: "郦城社区居委会",
                code: "006",
            },
            VillageCode {
                name: "闵航南里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "宝山社区居委会",
                code: "008",
            },
            VillageCode {
                name: "玉泉社区居委会",
                code: "009",
            },
            VillageCode {
                name: "西山社区居委会",
                code: "010",
            },
            VillageCode {
                name: "西冉社区居委会",
                code: "011",
            },
            VillageCode {
                name: "振兴社区居委会",
                code: "012",
            },
            VillageCode {
                name: "常青社区居委会",
                code: "013",
            },
            VillageCode {
                name: "高庄社区居委会",
                code: "014",
            },
            VillageCode {
                name: "田村村委会",
                code: "015",
            },
            VillageCode {
                name: "宝山村委会",
                code: "016",
            },
            VillageCode {
                name: "振兴村委会",
                code: "017",
            },
            VillageCode {
                name: "西冉村委会",
                code: "018",
            },
            VillageCode {
                name: "西山村委会",
                code: "019",
            },
            VillageCode {
                name: "双新村委会",
                code: "020",
            },
            VillageCode {
                name: "玉泉村委会",
                code: "021",
            },
            VillageCode {
                name: "门头村村委会",
                code: "022",
            },
            VillageCode {
                name: "巨山村委会",
                code: "023",
            },
            VillageCode {
                name: "香山村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "西北旺地区",
        code: "027",
        villages: &[
            VillageCode {
                name: "六里屯社区居委会",
                code: "001",
            },
            VillageCode {
                name: "亮甲店社区居委会",
                code: "002",
            },
            VillageCode {
                name: "屯佃社区居委会",
                code: "003",
            },
            VillageCode {
                name: "大牛坊社区居委会",
                code: "004",
            },
            VillageCode {
                name: "小辛店社区居委会",
                code: "005",
            },
            VillageCode {
                name: "友谊嘉园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "西六里屯社区居委会",
                code: "007",
            },
            VillageCode {
                name: "航天城社区居委会",
                code: "008",
            },
            VillageCode {
                name: "西山林语社区居委会",
                code: "009",
            },
            VillageCode {
                name: "冷泉社区居委会",
                code: "010",
            },
            VillageCode {
                name: "韩家川社区居委会",
                code: "011",
            },
            VillageCode {
                name: "唐家岭社区居委会",
                code: "012",
            },
            VillageCode {
                name: "土井社区居委会",
                code: "013",
            },
            VillageCode {
                name: "航天城五院社区居委会",
                code: "014",
            },
            VillageCode {
                name: "燕保辛店家园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "青棠湾社区居委会",
                code: "016",
            },
            VillageCode {
                name: "天阅西山社区居委会",
                code: "017",
            },
            VillageCode {
                name: "永靓家园社区居委会",
                code: "018",
            },
            VillageCode {
                name: "宫悦园社区居委会",
                code: "019",
            },
            VillageCode {
                name: "西玉河社区居委会",
                code: "020",
            },
            VillageCode {
                name: "西北旺村委会",
                code: "021",
            },
            VillageCode {
                name: "韩家川村委会",
                code: "022",
            },
            VillageCode {
                name: "冷泉村委会",
                code: "023",
            },
            VillageCode {
                name: "屯佃村委会",
                code: "024",
            },
            VillageCode {
                name: "永丰屯村委会",
                code: "025",
            },
            VillageCode {
                name: "唐家岭村股份经济合作社生活区",
                code: "026",
            },
            VillageCode {
                name: "土井村股份经济合作社生活区",
                code: "027",
            },
            VillageCode {
                name: "东北旺村股份经济合作社生活区",
                code: "028",
            },
            VillageCode {
                name: "六里屯村股份经济合作社生活区",
                code: "029",
            },
            VillageCode {
                name: "大牛坊村股份经济合作社生活区",
                code: "030",
            },
            VillageCode {
                name: "东玉河村股份经济合作社生活区",
                code: "031",
            },
            VillageCode {
                name: "小牛坊村股份经济合作社生活区",
                code: "032",
            },
            VillageCode {
                name: "亮甲店村股份经济合作社生活区",
                code: "033",
            },
            VillageCode {
                name: "皇后店村股份经济合作社生活区",
                code: "034",
            },
            VillageCode {
                name: "西玉河村股份经济合作社生活区",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "苏家坨地区",
        code: "028",
        villages: &[
            VillageCode {
                name: "北分瑞利社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北安河社区居委会",
                code: "002",
            },
            VillageCode {
                name: "聂各庄社区居委会",
                code: "003",
            },
            VillageCode {
                name: "稻香湖社区居委会",
                code: "004",
            },
            VillageCode {
                name: "前沙涧社区居委会",
                code: "005",
            },
            VillageCode {
                name: "同泽园东里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "同泽园西里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "安河家园东区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "安河家园西区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "安岚佳苑社区居委会",
                code: "010",
            },
            VillageCode {
                name: "凤锦家园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "苏三四村委会",
                code: "012",
            },
            VillageCode {
                name: "西小营村委会",
                code: "013",
            },
            VillageCode {
                name: "柳林村委会",
                code: "014",
            },
            VillageCode {
                name: "后沙涧村委会",
                code: "015",
            },
            VillageCode {
                name: "草厂村委会",
                code: "016",
            },
            VillageCode {
                name: "西埠头村委会",
                code: "017",
            },
            VillageCode {
                name: "七王坟村委会",
                code: "018",
            },
            VillageCode {
                name: "梁家园村委会",
                code: "019",
            },
            VillageCode {
                name: "台头村委会",
                code: "020",
            },
            VillageCode {
                name: "聂各庄村委会",
                code: "021",
            },
            VillageCode {
                name: "三星庄村股份经济合作社生活区",
                code: "022",
            },
            VillageCode {
                name: "苏一二村股份经济合作社生活区",
                code: "023",
            },
            VillageCode {
                name: "北庄子村股份经济合作社生活区",
                code: "024",
            },
            VillageCode {
                name: "前沙涧村股份经济合作社生活区",
                code: "025",
            },
            VillageCode {
                name: "南安河村股份经济合作社生活区",
                code: "026",
            },
            VillageCode {
                name: "徐各庄村股份经济合作社生活区",
                code: "027",
            },
            VillageCode {
                name: "周家巷村股份经济合作社生活区",
                code: "028",
            },
            VillageCode {
                name: "北安河村股份经济合作社生活区",
                code: "029",
            },
            VillageCode {
                name: "车耳营村股份经济合作社生活区",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "上庄地区",
        code: "029",
        villages: &[
            VillageCode {
                name: "上庄家园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "馨瑞嘉园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "三嘉信苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "翠北嘉园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "馨悦家园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "馨怡嘉园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "泽信社区居委会",
                code: "007",
            },
            VillageCode {
                name: "东马坊村委会",
                code: "008",
            },
            VillageCode {
                name: "上庄村委会",
                code: "009",
            },
            VillageCode {
                name: "前章村村委会",
                code: "010",
            },
            VillageCode {
                name: "白水洼村委会",
                code: "011",
            },
            VillageCode {
                name: "西马坊村委会",
                code: "012",
            },
            VillageCode {
                name: "常乐村委会",
                code: "013",
            },
            VillageCode {
                name: "东小营村委会",
                code: "014",
            },
            VillageCode {
                name: "罗家坟村委会",
                code: "015",
            },
            VillageCode {
                name: "皂甲屯村委会",
                code: "016",
            },
            VillageCode {
                name: "李家坟村委会",
                code: "017",
            },
            VillageCode {
                name: "南玉河村委会",
                code: "018",
            },
            VillageCode {
                name: "北玉河村委会",
                code: "019",
            },
            VillageCode {
                name: "永泰庄村委会",
                code: "020",
            },
            VillageCode {
                name: "梅所屯村委会",
                code: "021",
            },
            VillageCode {
                name: "双塔村委会",
                code: "022",
            },
            VillageCode {
                name: "西闸村委会",
                code: "023",
            },
            VillageCode {
                name: "河北村村委会",
                code: "024",
            },
            VillageCode {
                name: "八家村委会",
                code: "025",
            },
            VillageCode {
                name: "后章村村委会",
                code: "026",
            },
            VillageCode {
                name: "西辛力屯村委会",
                code: "027",
            },
        ],
    },
];

static TOWNS_BP_007: [TownCode; 13] = [
    TownCode {
        name: "大峪街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "德露苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "月季园东里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "月季园一区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "月季园二区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "新桥南大街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "双峪社区居委会",
                code: "006",
            },
            VillageCode {
                name: "向阳社区居委会",
                code: "007",
            },
            VillageCode {
                name: "向阳东里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "龙泉花园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "剧场东街社区居委会",
                code: "010",
            },
            VillageCode {
                name: "增产路社区居委会",
                code: "011",
            },
            VillageCode {
                name: "新自建社区居委会",
                code: "012",
            },
            VillageCode {
                name: "南路一社区居委会",
                code: "013",
            },
            VillageCode {
                name: "南路二社区居委会",
                code: "014",
            },
            VillageCode {
                name: "峪园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "永新社区居委会",
                code: "016",
            },
            VillageCode {
                name: "桃园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "增产路东区社区居委会",
                code: "018",
            },
            VillageCode {
                name: "新桥社区居委会",
                code: "019",
            },
            VillageCode {
                name: "新桥西区社区居委会",
                code: "020",
            },
            VillageCode {
                name: "承泽苑社区居委会",
                code: "021",
            },
            VillageCode {
                name: "中门花园社区居委会",
                code: "022",
            },
            VillageCode {
                name: "绿岛家园社区居委会",
                code: "023",
            },
            VillageCode {
                name: "绮霞苑社区居委会",
                code: "024",
            },
            VillageCode {
                name: "滨河西区社区居委会",
                code: "025",
            },
            VillageCode {
                name: "葡东社区居委会",
                code: "026",
            },
            VillageCode {
                name: "临镜苑社区居委会",
                code: "027",
            },
            VillageCode {
                name: "惠民家园社区居委会",
                code: "028",
            },
            VillageCode {
                name: "龙山三区社区居委会",
                code: "029",
            },
            VillageCode {
                name: "龙山一区社区居委会",
                code: "030",
            },
            VillageCode {
                name: "龙山二区社区居委会",
                code: "031",
            },
            VillageCode {
                name: "丽湾西园社区居委会",
                code: "032",
            },
            VillageCode {
                name: "龙坡社区居委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "城子街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "市场街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "桥东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "城子大街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "城子西街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "七棵树东街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "广场社区居委会",
                code: "006",
            },
            VillageCode {
                name: "向阳社区居委会",
                code: "007",
            },
            VillageCode {
                name: "七棵树西街社区居委会",
                code: "008",
            },
            VillageCode {
                name: "矿桥东街社区居委会",
                code: "009",
            },
            VillageCode {
                name: "西宁路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "华新建社区居委会",
                code: "011",
            },
            VillageCode {
                name: "新老宿舍社区居委会",
                code: "012",
            },
            VillageCode {
                name: "蓝龙家园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "龙门新区一区社区居委会",
                code: "014",
            },
            VillageCode {
                name: "龙门新区三区社区居委会",
                code: "015",
            },
            VillageCode {
                name: "龙门新区四区社区居委会",
                code: "016",
            },
            VillageCode {
                name: "龙门新区五区社区居委会",
                code: "017",
            },
            VillageCode {
                name: "龙门新区六区社区居委会",
                code: "018",
            },
            VillageCode {
                name: "燕保家园社区居委会",
                code: "019",
            },
            VillageCode {
                name: "城子东街社区居委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "东辛房街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "北涧沟社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西山社区居委会",
                code: "002",
            },
            VillageCode {
                name: "圈门社区居委会",
                code: "003",
            },
            VillageCode {
                name: "石门营新区一区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "石门营新区五区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "石门营新区六区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "石门营新区七区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "石门营新区四区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "石门营新区二区社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "大台街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "落坡岭社区居委会",
                code: "001",
            },
            VillageCode {
                name: "桃园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "双红社区居委会",
                code: "003",
            },
            VillageCode {
                name: "大台社区居委会",
                code: "004",
            },
            VillageCode {
                name: "黄土台社区居委会",
                code: "005",
            },
            VillageCode {
                name: "灰地社区居委会",
                code: "006",
            },
            VillageCode {
                name: "玉皇庙社区居委会",
                code: "007",
            },
            VillageCode {
                name: "木城涧社区居委会",
                code: "008",
            },
            VillageCode {
                name: "千军台社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "王平地区",
        code: "005",
        villages: &[
            VillageCode {
                name: "色树坟社区居委会",
                code: "001",
            },
            VillageCode {
                name: "河北社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "惠和新苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "安家庄村村委会",
                code: "005",
            },
            VillageCode {
                name: "吕家坡村村委会",
                code: "006",
            },
            VillageCode {
                name: "西王平村村委会",
                code: "007",
            },
            VillageCode {
                name: "东王平村村委会",
                code: "008",
            },
            VillageCode {
                name: "南涧村村委会",
                code: "009",
            },
            VillageCode {
                name: "河北村村委会",
                code: "010",
            },
            VillageCode {
                name: "色树坟村村委会",
                code: "011",
            },
            VillageCode {
                name: "西石古岩村村委会",
                code: "012",
            },
            VillageCode {
                name: "东石古岩村村委会",
                code: "013",
            },
            VillageCode {
                name: "西马各庄村村委会",
                code: "014",
            },
            VillageCode {
                name: "东马各庄村村委会",
                code: "015",
            },
            VillageCode {
                name: "南港村村委会",
                code: "016",
            },
            VillageCode {
                name: "韭园村村委会",
                code: "017",
            },
            VillageCode {
                name: "桥耳涧村村委会",
                code: "018",
            },
            VillageCode {
                name: "西落坡村村委会",
                code: "019",
            },
            VillageCode {
                name: "东落坡村村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "永定地区",
        code: "006",
        villages: &[
            VillageCode {
                name: "南区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "永兴社区居委会",
                code: "003",
            },
            VillageCode {
                name: "永安社区居委会",
                code: "004",
            },
            VillageCode {
                name: "信园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "嘉园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "永兴嘉园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "小园一区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "小园二区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "小园三区社区居委会",
                code: "010",
            },
            VillageCode {
                name: "曹各庄一区社区居委会",
                code: "011",
            },
            VillageCode {
                name: "曹各庄二区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "曹各庄三区社区居委会",
                code: "013",
            },
            VillageCode {
                name: "润西山社区居委会",
                code: "014",
            },
            VillageCode {
                name: "梧桐苑社区居委会",
                code: "015",
            },
            VillageCode {
                name: "丽景长安社区居委会",
                code: "016",
            },
            VillageCode {
                name: "西悦嘉园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "京西嘉苑社区居委会",
                code: "018",
            },
            VillageCode {
                name: "四季怡园社区居委会",
                code: "019",
            },
            VillageCode {
                name: "上悦嘉园社区居委会",
                code: "020",
            },
            VillageCode {
                name: "翡翠家园社区居委会",
                code: "021",
            },
            VillageCode {
                name: "西山燕庐家园社区居委会",
                code: "022",
            },
            VillageCode {
                name: "迎晖北苑社区居委会",
                code: "023",
            },
            VillageCode {
                name: "迎晖南苑社区居委会",
                code: "024",
            },
            VillageCode {
                name: "云翔嘉苑社区居委会",
                code: "025",
            },
            VillageCode {
                name: "云泽嘉苑社区居委会",
                code: "026",
            },
            VillageCode {
                name: "云梦嘉苑社区居委会",
                code: "027",
            },
            VillageCode {
                name: "上岸村委会",
                code: "028",
            },
            VillageCode {
                name: "桥户营村委会",
                code: "029",
            },
            VillageCode {
                name: "曹各庄村委会",
                code: "030",
            },
            VillageCode {
                name: "冯村村委会",
                code: "031",
            },
            VillageCode {
                name: "艾洼村委会",
                code: "032",
            },
            VillageCode {
                name: "万佛堂村委会",
                code: "033",
            },
            VillageCode {
                name: "何各庄村委会",
                code: "034",
            },
            VillageCode {
                name: "石厂村委会",
                code: "035",
            },
            VillageCode {
                name: "岢罗坨村委会",
                code: "036",
            },
            VillageCode {
                name: "王村村委会",
                code: "037",
            },
            VillageCode {
                name: "石门营村委会",
                code: "038",
            },
            VillageCode {
                name: "小园村委会",
                code: "039",
            },
            VillageCode {
                name: "栗元庄村委会",
                code: "040",
            },
            VillageCode {
                name: "卧龙岗村委会",
                code: "041",
            },
            VillageCode {
                name: "西辛称村委会",
                code: "042",
            },
            VillageCode {
                name: "东辛称村委会",
                code: "043",
            },
            VillageCode {
                name: "白庄子村委会",
                code: "044",
            },
            VillageCode {
                name: "四道桥村委会",
                code: "045",
            },
            VillageCode {
                name: "坝房子村委会",
                code: "046",
            },
            VillageCode {
                name: "侯庄子村委会",
                code: "047",
            },
            VillageCode {
                name: "贵石村委会",
                code: "048",
            },
            VillageCode {
                name: "卫星队村委会",
                code: "049",
            },
            VillageCode {
                name: "秋坡村委会",
                code: "050",
            },
            VillageCode {
                name: "石佛村委会",
                code: "051",
            },
        ],
    },
    TownCode {
        name: "龙泉地区",
        code: "007",
        villages: &[
            VillageCode {
                name: "东南街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中北街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西前街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "水闸西路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "琉璃渠社区居委会",
                code: "005",
            },
            VillageCode {
                name: "龙泉务社区居委会",
                code: "006",
            },
            VillageCode {
                name: "梨园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "峪新社区居委会",
                code: "008",
            },
            VillageCode {
                name: "倚山嘉园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "中门家园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "龙门新区二区社区居委会",
                code: "011",
            },
            VillageCode {
                name: "中门寺南坡一区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "中门寺南坡二区社区居委会",
                code: "013",
            },
            VillageCode {
                name: "高家园新区社区居委会",
                code: "014",
            },
            VillageCode {
                name: "西山艺境社区居委会",
                code: "015",
            },
            VillageCode {
                name: "大峪花园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "大峪村委会",
                code: "017",
            },
            VillageCode {
                name: "城子村委会",
                code: "018",
            },
            VillageCode {
                name: "龙泉雾村委会",
                code: "019",
            },
            VillageCode {
                name: "琉璃渠村委会",
                code: "020",
            },
            VillageCode {
                name: "三家店村委会",
                code: "021",
            },
            VillageCode {
                name: "中门寺村委会",
                code: "022",
            },
            VillageCode {
                name: "门头口村委会",
                code: "023",
            },
            VillageCode {
                name: "天桥浮村委会",
                code: "024",
            },
            VillageCode {
                name: "三店村委会",
                code: "025",
            },
            VillageCode {
                name: "西龙门村委会",
                code: "026",
            },
            VillageCode {
                name: "东龙门村委会",
                code: "027",
            },
            VillageCode {
                name: "西辛房村委会",
                code: "028",
            },
            VillageCode {
                name: "东辛房村委会",
                code: "029",
            },
            VillageCode {
                name: "石石巷村委会",
                code: "030",
            },
            VillageCode {
                name: "滑石道村委会",
                code: "031",
            },
            VillageCode {
                name: "岳家坡村委会",
                code: "032",
            },
            VillageCode {
                name: "赵家洼村委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "潭柘寺镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "潭柘新二区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "潭柘新一区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "檀香嘉园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "北村村委会",
                code: "004",
            },
            VillageCode {
                name: "东村村委会",
                code: "005",
            },
            VillageCode {
                name: "南村村委会",
                code: "006",
            },
            VillageCode {
                name: "鲁家滩村委会",
                code: "007",
            },
            VillageCode {
                name: "南辛房村委会",
                code: "008",
            },
            VillageCode {
                name: "桑峪村委会",
                code: "009",
            },
            VillageCode {
                name: "平原村委会",
                code: "010",
            },
            VillageCode {
                name: "王坡村委会",
                code: "011",
            },
            VillageCode {
                name: "贾沟村委会",
                code: "012",
            },
            VillageCode {
                name: "草甸水村委会",
                code: "013",
            },
            VillageCode {
                name: "赵家台村委会",
                code: "014",
            },
            VillageCode {
                name: "阳坡元村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "军庄镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "杨坨社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北四社区居委会",
                code: "002",
            },
            VillageCode {
                name: "惠通新苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "军庄村委会",
                code: "004",
            },
            VillageCode {
                name: "灰峪村委会",
                code: "005",
            },
            VillageCode {
                name: "西杨坨村委会",
                code: "006",
            },
            VillageCode {
                name: "东杨坨村委会",
                code: "007",
            },
            VillageCode {
                name: "孟悟村委会",
                code: "008",
            },
            VillageCode {
                name: "新村村委会",
                code: "009",
            },
            VillageCode {
                name: "东山村委会",
                code: "010",
            },
            VillageCode {
                name: "香峪村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "雁翅镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "雁翅社区居委会",
                code: "001",
            },
            VillageCode {
                name: "河南台村委会",
                code: "002",
            },
            VillageCode {
                name: "雁翅村委会",
                code: "003",
            },
            VillageCode {
                name: "芹峪村委会",
                code: "004",
            },
            VillageCode {
                name: "下马岭村委会",
                code: "005",
            },
            VillageCode {
                name: "饮马鞍村委会",
                code: "006",
            },
            VillageCode {
                name: "太子墓村委会",
                code: "007",
            },
            VillageCode {
                name: "付家台村委会",
                code: "008",
            },
            VillageCode {
                name: "青白口村委会",
                code: "009",
            },
            VillageCode {
                name: "珠窝村委会",
                code: "010",
            },
            VillageCode {
                name: "碣石村委会",
                code: "011",
            },
            VillageCode {
                name: "黄土贵村委会",
                code: "012",
            },
            VillageCode {
                name: "泗家水村委会",
                code: "013",
            },
            VillageCode {
                name: "淤白村委会",
                code: "014",
            },
            VillageCode {
                name: "高台村委会",
                code: "015",
            },
            VillageCode {
                name: "松树村委会",
                code: "016",
            },
            VillageCode {
                name: "田庄村委会",
                code: "017",
            },
            VillageCode {
                name: "苇子水村委会",
                code: "018",
            },
            VillageCode {
                name: "大村村委会",
                code: "019",
            },
            VillageCode {
                name: "房良村委会",
                code: "020",
            },
            VillageCode {
                name: "杨村村委会",
                code: "021",
            },
            VillageCode {
                name: "马套村委会",
                code: "022",
            },
            VillageCode {
                name: "山神庙村委会",
                code: "023",
            },
            VillageCode {
                name: "跃进村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "斋堂镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "斋堂小城镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西斋堂村委会",
                code: "002",
            },
            VillageCode {
                name: "东斋堂村委会",
                code: "003",
            },
            VillageCode {
                name: "马栏村委会",
                code: "004",
            },
            VillageCode {
                name: "火村村委会",
                code: "005",
            },
            VillageCode {
                name: "高铺村委会",
                code: "006",
            },
            VillageCode {
                name: "青龙涧村委会",
                code: "007",
            },
            VillageCode {
                name: "黄岭西村委会",
                code: "008",
            },
            VillageCode {
                name: "双石头村委会",
                code: "009",
            },
            VillageCode {
                name: "川底下村委会",
                code: "010",
            },
            VillageCode {
                name: "柏峪村委会",
                code: "011",
            },
            VillageCode {
                name: "牛战村委会",
                code: "012",
            },
            VillageCode {
                name: "白虎头村委会",
                code: "013",
            },
            VillageCode {
                name: "新兴村村委会",
                code: "014",
            },
            VillageCode {
                name: "向阳口村委会",
                code: "015",
            },
            VillageCode {
                name: "沿河城村委会",
                code: "016",
            },
            VillageCode {
                name: "王龙口村委会",
                code: "017",
            },
            VillageCode {
                name: "沿河口村委会",
                code: "018",
            },
            VillageCode {
                name: "龙门口村委会",
                code: "019",
            },
            VillageCode {
                name: "林子台村委会",
                code: "020",
            },
            VillageCode {
                name: "西胡林村委会",
                code: "021",
            },
            VillageCode {
                name: "东胡林村委会",
                code: "022",
            },
            VillageCode {
                name: "军响村委会",
                code: "023",
            },
            VillageCode {
                name: "桑峪村委会",
                code: "024",
            },
            VillageCode {
                name: "灵水村委会",
                code: "025",
            },
            VillageCode {
                name: "法城村委会",
                code: "026",
            },
            VillageCode {
                name: "杨家村村委会",
                code: "027",
            },
            VillageCode {
                name: "张家村村委会",
                code: "028",
            },
            VillageCode {
                name: "吕家村村委会",
                code: "029",
            },
            VillageCode {
                name: "杨家峪村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "清水镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "燕家台村委会",
                code: "001",
            },
            VillageCode {
                name: "李家庄村委会",
                code: "002",
            },
            VillageCode {
                name: "梁家庄村委会",
                code: "003",
            },
            VillageCode {
                name: "台上村委会",
                code: "004",
            },
            VillageCode {
                name: "上清水村委会",
                code: "005",
            },
            VillageCode {
                name: "下清水村委会",
                code: "006",
            },
            VillageCode {
                name: "田寺村委会",
                code: "007",
            },
            VillageCode {
                name: "西达么村委会",
                code: "008",
            },
            VillageCode {
                name: "洪水峪村委会",
                code: "009",
            },
            VillageCode {
                name: "上达么村委会",
                code: "010",
            },
            VillageCode {
                name: "达么庄村委会",
                code: "011",
            },
            VillageCode {
                name: "椴木沟村委会",
                code: "012",
            },
            VillageCode {
                name: "梁家铺村委会",
                code: "013",
            },
            VillageCode {
                name: "塔河村委会",
                code: "014",
            },
            VillageCode {
                name: "黄安村委会",
                code: "015",
            },
            VillageCode {
                name: "龙王村委会",
                code: "016",
            },
            VillageCode {
                name: "黄安坨村委会",
                code: "017",
            },
            VillageCode {
                name: "黄塔村委会",
                code: "018",
            },
            VillageCode {
                name: "八亩堰村委会",
                code: "019",
            },
            VillageCode {
                name: "简昌村委会",
                code: "020",
            },
            VillageCode {
                name: "艾峪村委会",
                code: "021",
            },
            VillageCode {
                name: "双涧子村委会",
                code: "022",
            },
            VillageCode {
                name: "张家铺村委会",
                code: "023",
            },
            VillageCode {
                name: "杜家庄村委会",
                code: "024",
            },
            VillageCode {
                name: "张家庄村委会",
                code: "025",
            },
            VillageCode {
                name: "齐家庄村委会",
                code: "026",
            },
            VillageCode {
                name: "双塘涧村委会",
                code: "027",
            },
            VillageCode {
                name: "天河水村委会",
                code: "028",
            },
            VillageCode {
                name: "胜利村委会",
                code: "029",
            },
            VillageCode {
                name: "小龙门村委会",
                code: "030",
            },
            VillageCode {
                name: "洪水口村委会",
                code: "031",
            },
            VillageCode {
                name: "江水河村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "妙峰山镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "陇驾庄村委会",
                code: "001",
            },
            VillageCode {
                name: "丁家滩村委会",
                code: "002",
            },
            VillageCode {
                name: "水峪嘴村委会",
                code: "003",
            },
            VillageCode {
                name: "斜河涧村委会",
                code: "004",
            },
            VillageCode {
                name: "陈家庄村委会",
                code: "005",
            },
            VillageCode {
                name: "担礼村委会",
                code: "006",
            },
            VillageCode {
                name: "下苇甸村委会",
                code: "007",
            },
            VillageCode {
                name: "桃园村委会",
                code: "008",
            },
            VillageCode {
                name: "南庄村委会",
                code: "009",
            },
            VillageCode {
                name: "樱桃沟村委会",
                code: "010",
            },
            VillageCode {
                name: "涧沟村委会",
                code: "011",
            },
            VillageCode {
                name: "上苇甸村委会",
                code: "012",
            },
            VillageCode {
                name: "炭厂村委会",
                code: "013",
            },
            VillageCode {
                name: "大沟村委会",
                code: "014",
            },
            VillageCode {
                name: "禅房村委会",
                code: "015",
            },
            VillageCode {
                name: "黄台村委会",
                code: "016",
            },
            VillageCode {
                name: "岭角村委会",
                code: "017",
            },
        ],
    },
];

static TOWNS_BP_008: [TownCode; 28] = [
    TownCode {
        name: "城关街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "万宁桥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "城北社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "北街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "永安西里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "南里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "南城社区居委会",
                code: "007",
            },
            VillageCode {
                name: "农林路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "南沿里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新东关社区居委会",
                code: "010",
            },
            VillageCode {
                name: "大石河社区居委会",
                code: "011",
            },
            VillageCode {
                name: "矿机社区居委会",
                code: "012",
            },
            VillageCode {
                name: "管道局社区居委会",
                code: "013",
            },
            VillageCode {
                name: "化工四厂社区居委会",
                code: "014",
            },
            VillageCode {
                name: "城东社区居委会",
                code: "015",
            },
            VillageCode {
                name: "永乐园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "永兴达社区居委会",
                code: "017",
            },
            VillageCode {
                name: "兴房东里社区居委会",
                code: "018",
            },
            VillageCode {
                name: "福星家园社区居委会",
                code: "019",
            },
            VillageCode {
                name: "府东里社区居委会",
                code: "020",
            },
            VillageCode {
                name: "永安家园社区居委会",
                code: "021",
            },
            VillageCode {
                name: "蓝城家园社区居委会",
                code: "022",
            },
            VillageCode {
                name: "原香嘉苑社区居民委员会",
                code: "023",
            },
            VillageCode {
                name: "原香漫谷社区居民委员会",
                code: "024",
            },
            VillageCode {
                name: "顾册村委会",
                code: "025",
            },
            VillageCode {
                name: "北市村委会",
                code: "026",
            },
            VillageCode {
                name: "东坟村委会",
                code: "027",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "028",
            },
            VillageCode {
                name: "东瓜地村委会",
                code: "029",
            },
            VillageCode {
                name: "田各庄村委会",
                code: "030",
            },
            VillageCode {
                name: "瓜市村委会",
                code: "031",
            },
            VillageCode {
                name: "马各庄村委会",
                code: "032",
            },
            VillageCode {
                name: "饶乐府村委会",
                code: "033",
            },
            VillageCode {
                name: "丁家洼村委会",
                code: "034",
            },
            VillageCode {
                name: "羊头岗村委会",
                code: "035",
            },
            VillageCode {
                name: "八十亩地村委会",
                code: "036",
            },
            VillageCode {
                name: "前朱各庄村委会",
                code: "037",
            },
            VillageCode {
                name: "后朱各庄村委会",
                code: "038",
            },
            VillageCode {
                name: "洪寺村委会",
                code: "039",
            },
            VillageCode {
                name: "塔湾村委会",
                code: "040",
            },
            VillageCode {
                name: "迎风坡村委会",
                code: "041",
            },
            VillageCode {
                name: "东街村委会",
                code: "042",
            },
            VillageCode {
                name: "南街村委会",
                code: "043",
            },
            VillageCode {
                name: "南关村委会",
                code: "044",
            },
            VillageCode {
                name: "西街村委会",
                code: "045",
            },
            VillageCode {
                name: "北关村委会",
                code: "046",
            },
        ],
    },
    TownCode {
        name: "新镇街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "东平街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "原新街社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "向阳街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "向阳里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "宏塔社区居委会",
                code: "002",
            },
            VillageCode {
                name: "燕东路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "富燕新村第一社区居委会",
                code: "004",
            },
            VillageCode {
                name: "富燕新村第二社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "东风街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "南里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "羊耳峪北里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "羊耳峪里第一社区居委会",
                code: "005",
            },
            VillageCode {
                name: "羊耳峪里第二社区居委会",
                code: "006",
            },
            VillageCode {
                name: "羊耳峪西区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "燕和园社区居民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "迎风街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "高家坡社区居委会",
                code: "001",
            },
            VillageCode {
                name: "迎风四里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "迎风五里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "杏花西里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "杏花东里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "迎风六里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "迎风西里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "杰辉苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "迎风一里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "凤凰亭社区居委会",
                code: "010",
            },
            VillageCode {
                name: "幸福家园社区居民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "星城街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "星城第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "星城第二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "星城第三社区居委会",
                code: "003",
            },
            VillageCode {
                name: "星城第四社区居委会",
                code: "004",
            },
            VillageCode {
                name: "星城第五社区居委会",
                code: "005",
            },
            VillageCode {
                name: "星城第六社区居委会",
                code: "006",
            },
            VillageCode {
                name: "星城第七社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "良乡地区",
        code: "007",
        villages: &[
            VillageCode {
                name: "尚锦佳苑社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "舒朗苑社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "南刘庄村委会",
                code: "003",
            },
            VillageCode {
                name: "西石羊村委会",
                code: "004",
            },
            VillageCode {
                name: "后石羊村委会",
                code: "005",
            },
            VillageCode {
                name: "东石羊村委会",
                code: "006",
            },
            VillageCode {
                name: "张谢村委会",
                code: "007",
            },
            VillageCode {
                name: "江村村委会",
                code: "008",
            },
            VillageCode {
                name: "侯庄村委会",
                code: "009",
            },
            VillageCode {
                name: "下禅坊村委会",
                code: "010",
            },
            VillageCode {
                name: "刘丈村委会",
                code: "011",
            },
            VillageCode {
                name: "南庄子村委会",
                code: "012",
            },
            VillageCode {
                name: "邢家坞村委会",
                code: "013",
            },
            VillageCode {
                name: "官道村委会",
                code: "014",
            },
            VillageCode {
                name: "小营村委会",
                code: "015",
            },
            VillageCode {
                name: "鲁村村委会",
                code: "016",
            },
            VillageCode {
                name: "黑古台村委会",
                code: "017",
            },
            VillageCode {
                name: "富庄村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "周口店地区",
        code: "008",
        villages: &[
            VillageCode {
                name: "周口店社区居委会",
                code: "001",
            },
            VillageCode {
                name: "长沟峪社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金巢社区居委会",
                code: "003",
            },
            VillageCode {
                name: "红光社区居委会",
                code: "004",
            },
            VillageCode {
                name: "鑫山矿社区居委会",
                code: "005",
            },
            VillageCode {
                name: "南韩继村委会",
                code: "006",
            },
            VillageCode {
                name: "瓦井村委会",
                code: "007",
            },
            VillageCode {
                name: "新街村委会",
                code: "008",
            },
            VillageCode {
                name: "大韩继村委会",
                code: "009",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "010",
            },
            VillageCode {
                name: "周口村村委会",
                code: "011",
            },
            VillageCode {
                name: "云峰寺村委会",
                code: "012",
            },
            VillageCode {
                name: "周口店村委会",
                code: "013",
            },
            VillageCode {
                name: "娄子水村委会",
                code: "014",
            },
            VillageCode {
                name: "拴马庄村委会",
                code: "015",
            },
            VillageCode {
                name: "黄院村委会",
                code: "016",
            },
            VillageCode {
                name: "龙宝峪村委会",
                code: "017",
            },
            VillageCode {
                name: "黄山店村委会",
                code: "018",
            },
            VillageCode {
                name: "黄元寺村委会",
                code: "019",
            },
            VillageCode {
                name: "良各庄村委会",
                code: "020",
            },
            VillageCode {
                name: "西庄村委会",
                code: "021",
            },
            VillageCode {
                name: "车厂村委会",
                code: "022",
            },
            VillageCode {
                name: "涞沥水村委会",
                code: "023",
            },
            VillageCode {
                name: "泗马沟村委会",
                code: "024",
            },
            VillageCode {
                name: "北下寺村委会",
                code: "025",
            },
            VillageCode {
                name: "葫芦棚村委会",
                code: "026",
            },
            VillageCode {
                name: "长流水村委会",
                code: "027",
            },
            VillageCode {
                name: "山口村委会",
                code: "028",
            },
            VillageCode {
                name: "官地村委会",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "琉璃河地区",
        code: "009",
        villages: &[
            VillageCode {
                name: "二街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "窗纱厂社区居委会",
                code: "002",
            },
            VillageCode {
                name: "琉璃河水泥厂社区居委会",
                code: "003",
            },
            VillageCode {
                name: "建材工业学校社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金果林社区居委会",
                code: "005",
            },
            VillageCode {
                name: "二街村委会",
                code: "006",
            },
            VillageCode {
                name: "三街村委会",
                code: "007",
            },
            VillageCode {
                name: "李庄村委会",
                code: "008",
            },
            VillageCode {
                name: "白庄村委会",
                code: "009",
            },
            VillageCode {
                name: "杨户屯村委会",
                code: "010",
            },
            VillageCode {
                name: "周庄村委会",
                code: "011",
            },
            VillageCode {
                name: "福兴村委会",
                code: "012",
            },
            VillageCode {
                name: "平各庄村委会",
                code: "013",
            },
            VillageCode {
                name: "北洛村委会",
                code: "014",
            },
            VillageCode {
                name: "南洛村委会",
                code: "015",
            },
            VillageCode {
                name: "古庄村委会",
                code: "016",
            },
            VillageCode {
                name: "祖村村委会",
                code: "017",
            },
            VillageCode {
                name: "北章村委会",
                code: "018",
            },
            VillageCode {
                name: "兴礼村委会",
                code: "019",
            },
            VillageCode {
                name: "庄头村委会",
                code: "020",
            },
            VillageCode {
                name: "立教村委会",
                code: "021",
            },
            VillageCode {
                name: "董家林村委会",
                code: "022",
            },
            VillageCode {
                name: "刘李店村委会",
                code: "023",
            },
            VillageCode {
                name: "洄城村委会",
                code: "024",
            },
            VillageCode {
                name: "黄土坡村委会",
                code: "025",
            },
            VillageCode {
                name: "东南召村委会",
                code: "026",
            },
            VillageCode {
                name: "西南召村委会",
                code: "027",
            },
            VillageCode {
                name: "东南吕村委会",
                code: "028",
            },
            VillageCode {
                name: "西南吕村委会",
                code: "029",
            },
            VillageCode {
                name: "保兴庄村委会",
                code: "030",
            },
            VillageCode {
                name: "路村村委会",
                code: "031",
            },
            VillageCode {
                name: "南白村委会",
                code: "032",
            },
            VillageCode {
                name: "北白村委会",
                code: "033",
            },
            VillageCode {
                name: "八间房村委会",
                code: "034",
            },
            VillageCode {
                name: "薛庄村委会",
                code: "035",
            },
            VillageCode {
                name: "石村村委会",
                code: "036",
            },
            VillageCode {
                name: "常舍村委会",
                code: "037",
            },
            VillageCode {
                name: "西地村委会",
                code: "038",
            },
            VillageCode {
                name: "务滋村委会",
                code: "039",
            },
            VillageCode {
                name: "赵营村委会",
                code: "040",
            },
            VillageCode {
                name: "任营村委会",
                code: "041",
            },
            VillageCode {
                name: "万里村委会",
                code: "042",
            },
            VillageCode {
                name: "肖场村委会",
                code: "043",
            },
            VillageCode {
                name: "窑上村委会",
                code: "044",
            },
            VillageCode {
                name: "大陶村委会",
                code: "045",
            },
            VillageCode {
                name: "小陶村委会",
                code: "046",
            },
            VillageCode {
                name: "官庄村委会",
                code: "047",
            },
            VillageCode {
                name: "贾河村委会",
                code: "048",
            },
            VillageCode {
                name: "鲍庄村委会",
                code: "049",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "050",
            },
            VillageCode {
                name: "五间房村委会",
                code: "051",
            },
            VillageCode {
                name: "韩营村委会",
                code: "052",
            },
        ],
    },
    TownCode {
        name: "拱辰街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "一街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "三街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "拱辰大街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "宜春里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "梅花庄社区居委会",
                code: "005",
            },
            VillageCode {
                name: "北关东路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "行宫园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "长虹北里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "昊天小区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "北京电力设备总厂社区居委会",
                code: "010",
            },
            VillageCode {
                name: "北京送变电公司社区居委会",
                code: "011",
            },
            VillageCode {
                name: "北京电力建设公司社区居委会",
                code: "012",
            },
            VillageCode {
                name: "飞机场社区居委会",
                code: "013",
            },
            VillageCode {
                name: "一街第二社区居委会",
                code: "014",
            },
            VillageCode {
                name: "文化路社区居委会",
                code: "015",
            },
            VillageCode {
                name: "罗府街社区居委会",
                code: "016",
            },
            VillageCode {
                name: "三街第二社区居委会",
                code: "017",
            },
            VillageCode {
                name: "拱辰北大街社区居委会",
                code: "018",
            },
            VillageCode {
                name: "西北关社区居委会",
                code: "019",
            },
            VillageCode {
                name: "鸿顺园社区居委会",
                code: "020",
            },
            VillageCode {
                name: "玉竹园社区居委会",
                code: "021",
            },
            VillageCode {
                name: "伟业嘉园社区居委会",
                code: "022",
            },
            VillageCode {
                name: "瑞雪春堂社区居委会",
                code: "023",
            },
            VillageCode {
                name: "绿地花都苑社区居委会",
                code: "024",
            },
            VillageCode {
                name: "翠林湾嘉园社区居委会",
                code: "025",
            },
            VillageCode {
                name: "邑尚佳苑社区居委会",
                code: "026",
            },
            VillageCode {
                name: "紫汇家园社区居委会",
                code: "027",
            },
            VillageCode {
                name: "伟创嘉园社区居委会",
                code: "028",
            },
            VillageCode {
                name: "胜茂嘉苑社区居委会",
                code: "029",
            },
            VillageCode {
                name: "翠堤清苑社区居民委员会",
                code: "030",
            },
            VillageCode {
                name: "绿湾星苑社区居民委员会",
                code: "031",
            },
            VillageCode {
                name: "伊林郡社区居民委员会",
                code: "032",
            },
            VillageCode {
                name: "意墅园社区居民委员会",
                code: "033",
            },
            VillageCode {
                name: "智汇雅苑社区居民委员会",
                code: "034",
            },
            VillageCode {
                name: "东羊庄社区居民委员会",
                code: "035",
            },
            VillageCode {
                name: "二街社区居民委员会",
                code: "036",
            },
            VillageCode {
                name: "睿府嘉园社区居民委员会",
                code: "037",
            },
            VillageCode {
                name: "二街村委会",
                code: "038",
            },
            VillageCode {
                name: "四街村委会",
                code: "039",
            },
            VillageCode {
                name: "五街村委会",
                code: "040",
            },
            VillageCode {
                name: "南关村委会",
                code: "041",
            },
            VillageCode {
                name: "东关村委会",
                code: "042",
            },
            VillageCode {
                name: "后店村委会",
                code: "043",
            },
            VillageCode {
                name: "吴店村委会",
                code: "044",
            },
            VillageCode {
                name: "黄辛庄村委会",
                code: "045",
            },
            VillageCode {
                name: "渔儿沟村委会",
                code: "046",
            },
            VillageCode {
                name: "大南关村委会",
                code: "047",
            },
            VillageCode {
                name: "纸房村委会",
                code: "048",
            },
            VillageCode {
                name: "常庄村委会",
                code: "049",
            },
            VillageCode {
                name: "梨村村委会",
                code: "050",
            },
            VillageCode {
                name: "东羊庄村委会",
                code: "051",
            },
            VillageCode {
                name: "梅花庄村委会",
                code: "052",
            },
            VillageCode {
                name: "小西庄村委会",
                code: "053",
            },
            VillageCode {
                name: "辛瓜地村委会",
                code: "054",
            },
            VillageCode {
                name: "南广阳城村委会",
                code: "055",
            },
        ],
    },
    TownCode {
        name: "西潞街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "夏庄社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西潞东里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "月华东里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "北潞园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "苏庄一里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "西潞园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "苏庄二里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西路大街社区居委会",
                code: "008",
            },
            VillageCode {
                name: "苏庄三里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "海逸半岛社区居委会",
                code: "010",
            },
            VillageCode {
                name: "太平庄西里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "北潞春社区居委会",
                code: "012",
            },
            VillageCode {
                name: "金鸽园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "太平庄东里社区居委会",
                code: "014",
            },
            VillageCode {
                name: "海悦嘉园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "北潞馨社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "西潞园南里社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "詹庄村委会",
                code: "018",
            },
            VillageCode {
                name: "安庄村委会",
                code: "019",
            },
            VillageCode {
                name: "固村村委会",
                code: "020",
            },
            VillageCode {
                name: "太平庄村委会",
                code: "021",
            },
            VillageCode {
                name: "南上岗村委会",
                code: "022",
            },
            VillageCode {
                name: "东沿村村委会",
                code: "023",
            },
            VillageCode {
                name: "苏庄村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "阎村镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "梨园东里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "消防器材厂社区居委会",
                code: "002",
            },
            VillageCode {
                name: "桥梁厂社区居委会",
                code: "003",
            },
            VillageCode {
                name: "绿城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "万紫嘉园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "乐活家园社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "紫园社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "云瑞嘉园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "大紫草坞村委会",
                code: "009",
            },
            VillageCode {
                name: "小紫草坞村委会",
                code: "010",
            },
            VillageCode {
                name: "前沿村委会",
                code: "011",
            },
            VillageCode {
                name: "后沿村委会",
                code: "012",
            },
            VillageCode {
                name: "张庄村委会",
                code: "013",
            },
            VillageCode {
                name: "公主坟村委会",
                code: "014",
            },
            VillageCode {
                name: "北坊村委会",
                code: "015",
            },
            VillageCode {
                name: "南坊村委会",
                code: "016",
            },
            VillageCode {
                name: "吴庄村委会",
                code: "017",
            },
            VillageCode {
                name: "焦庄村委会",
                code: "018",
            },
            VillageCode {
                name: "大董村委会",
                code: "019",
            },
            VillageCode {
                name: "小董村委会",
                code: "020",
            },
            VillageCode {
                name: "西坟村委会",
                code: "021",
            },
            VillageCode {
                name: "开古庄村委会",
                code: "022",
            },
            VillageCode {
                name: "南梨园村委会",
                code: "023",
            },
            VillageCode {
                name: "二合庄村委会",
                code: "024",
            },
            VillageCode {
                name: "大十三里村委会",
                code: "025",
            },
            VillageCode {
                name: "小十三里村委会",
                code: "026",
            },
            VillageCode {
                name: "后十三里村委会",
                code: "027",
            },
            VillageCode {
                name: "肖庄村委会",
                code: "028",
            },
            VillageCode {
                name: "元武屯村委会",
                code: "029",
            },
            VillageCode {
                name: "炒米店村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "窦店镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "亚新特种建材公司社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金鑫苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "沁园春景社区居委会",
                code: "003",
            },
            VillageCode {
                name: "田家园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "窦店社区居委会",
                code: "005",
            },
            VillageCode {
                name: "京南嘉园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "山水汇豪苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "于庄社区居委会",
                code: "008",
            },
            VillageCode {
                name: "乐汇家园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "汇景嘉园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "腾龙家园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "华城家园社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "燕都世界名园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "提香草堂社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "窦店村委会",
                code: "015",
            },
            VillageCode {
                name: "白草洼村委会",
                code: "016",
            },
            VillageCode {
                name: "芦村村委会",
                code: "017",
            },
            VillageCode {
                name: "板桥村委会",
                code: "018",
            },
            VillageCode {
                name: "西安庄村委会",
                code: "019",
            },
            VillageCode {
                name: "田家园村委会",
                code: "020",
            },
            VillageCode {
                name: "瓦窑头村委会",
                code: "021",
            },
            VillageCode {
                name: "苏村村委会",
                code: "022",
            },
            VillageCode {
                name: "于庄村委会",
                code: "023",
            },
            VillageCode {
                name: "下坡店村委会",
                code: "024",
            },
            VillageCode {
                name: "七里店村委会",
                code: "025",
            },
            VillageCode {
                name: "望楚村委会",
                code: "026",
            },
            VillageCode {
                name: "一街村委会",
                code: "027",
            },
            VillageCode {
                name: "二街村委会",
                code: "028",
            },
            VillageCode {
                name: "三街村委会",
                code: "029",
            },
            VillageCode {
                name: "后街村委会",
                code: "030",
            },
            VillageCode {
                name: "小高舍村委会",
                code: "031",
            },
            VillageCode {
                name: "大高舍村委会",
                code: "032",
            },
            VillageCode {
                name: "丁各庄村委会",
                code: "033",
            },
            VillageCode {
                name: "刘平庄村委会",
                code: "034",
            },
            VillageCode {
                name: "袁庄村委会",
                code: "035",
            },
            VillageCode {
                name: "六股道村委会",
                code: "036",
            },
            VillageCode {
                name: "普安屯村委会",
                code: "037",
            },
            VillageCode {
                name: "兴隆庄村委会",
                code: "038",
            },
            VillageCode {
                name: "辛庄户村委会",
                code: "039",
            },
            VillageCode {
                name: "两间房村委会",
                code: "040",
            },
            VillageCode {
                name: "前柳村委会",
                code: "041",
            },
            VillageCode {
                name: "陈家房村委会",
                code: "042",
            },
            VillageCode {
                name: "北柳村委会",
                code: "043",
            },
            VillageCode {
                name: "河口村委会",
                code: "044",
            },
        ],
    },
    TownCode {
        name: "石楼镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "铁路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "吉羊村委会",
                code: "002",
            },
            VillageCode {
                name: "二站村委会",
                code: "003",
            },
            VillageCode {
                name: "石楼村委会",
                code: "004",
            },
            VillageCode {
                name: "双孝村委会",
                code: "005",
            },
            VillageCode {
                name: "支楼村委会",
                code: "006",
            },
            VillageCode {
                name: "杨驸马庄村委会",
                code: "007",
            },
            VillageCode {
                name: "襄驸马庄村委会",
                code: "008",
            },
            VillageCode {
                name: "大次洛村委会",
                code: "009",
            },
            VillageCode {
                name: "坨头村委会",
                code: "010",
            },
            VillageCode {
                name: "双柳树村委会",
                code: "011",
            },
            VillageCode {
                name: "梨园店村委会",
                code: "012",
            },
            VillageCode {
                name: "夏村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "长阳镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "长阳社区居委会",
                code: "001",
            },
            VillageCode {
                name: "长龙苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "碧桂园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "碧波园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "加州水郡东区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "大宁山庄社区居委会",
                code: "006",
            },
            VillageCode {
                name: "徜徉集社区居委会",
                code: "007",
            },
            VillageCode {
                name: "嘉州水郡北区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "嘉州水郡南区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "天泰新景社区居委会",
                code: "010",
            },
            VillageCode {
                name: "馨然嘉园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "建邦嘉园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "悦都苑社区居委会",
                code: "013",
            },
            VillageCode {
                name: "半岛家园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "熙景嘉园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "朗悦嘉园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "云湾家园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "熙兆嘉园社区居委会",
                code: "018",
            },
            VillageCode {
                name: "悦都新苑社区居委会",
                code: "019",
            },
            VillageCode {
                name: "悦然馨苑社区居委会",
                code: "020",
            },
            VillageCode {
                name: "领峰四季园社区居委会",
                code: "021",
            },
            VillageCode {
                name: "溪雅苑社区居委会",
                code: "022",
            },
            VillageCode {
                name: "广悦居社区居委会",
                code: "023",
            },
            VillageCode {
                name: "禾香雅园社区居委会",
                code: "024",
            },
            VillageCode {
                name: "紫云家园社区居民委员会",
                code: "025",
            },
            VillageCode {
                name: "天瑞嘉园社区居民委员会",
                code: "026",
            },
            VillageCode {
                name: "碧桂园北区社区居民委员会",
                code: "027",
            },
            VillageCode {
                name: "碧桂园南区社区居民委员会",
                code: "028",
            },
            VillageCode {
                name: "畅和园社区居民委员会",
                code: "029",
            },
            VillageCode {
                name: "稻香悦家园社区居民委员会",
                code: "030",
            },
            VillageCode {
                name: "金域缇香家园社区居民委员会",
                code: "031",
            },
            VillageCode {
                name: "康泽佳苑北区社区居民委员会",
                code: "032",
            },
            VillageCode {
                name: "康泽佳苑南区社区居民委员会",
                code: "033",
            },
            VillageCode {
                name: "铭品嘉苑社区居民委员会",
                code: "034",
            },
            VillageCode {
                name: "清苑嘉园社区居民委员会",
                code: "035",
            },
            VillageCode {
                name: "上林佳苑联合社区居民委员会",
                code: "036",
            },
            VillageCode {
                name: "天丰苑社区居民委员会",
                code: "037",
            },
            VillageCode {
                name: "万和家园社区居民委员会",
                code: "038",
            },
            VillageCode {
                name: "新里程家园社区居民委员会",
                code: "039",
            },
            VillageCode {
                name: "燕保阜盛家园社区居民委员会",
                code: "040",
            },
            VillageCode {
                name: "宜居园社区居民委员会",
                code: "041",
            },
            VillageCode {
                name: "原香小镇社区居民委员会",
                code: "042",
            },
            VillageCode {
                name: "长景新园社区居民委员会",
                code: "043",
            },
            VillageCode {
                name: "长龙家园社区居民委员会",
                code: "044",
            },
            VillageCode {
                name: "半岛家园南区社区居民委员会",
                code: "045",
            },
            VillageCode {
                name: "碧岸澜庭社区居民委员会",
                code: "046",
            },
            VillageCode {
                name: "康泽佳苑南区二里社区居民委员会",
                code: "047",
            },
            VillageCode {
                name: "康泽佳苑南区一里社区居民委员会",
                code: "048",
            },
            VillageCode {
                name: "西悦居社区居民委员会",
                code: "049",
            },
            VillageCode {
                name: "馨然嘉园北区社区居民委员会",
                code: "050",
            },
            VillageCode {
                name: "杨庄子社区居民委员会",
                code: "051",
            },
            VillageCode {
                name: "张家场社区居民委员会",
                code: "052",
            },
            VillageCode {
                name: "长景新园北区社区居民委员会",
                code: "053",
            },
            VillageCode {
                name: "紫云家园东区社区居民委员会",
                code: "054",
            },
            VillageCode {
                name: "长阳一村村委会",
                code: "055",
            },
            VillageCode {
                name: "长阳二村村委会",
                code: "056",
            },
            VillageCode {
                name: "黄管屯村委会",
                code: "057",
            },
            VillageCode {
                name: "哑叭河村委会",
                code: "058",
            },
            VillageCode {
                name: "北广阳城村委会",
                code: "059",
            },
            VillageCode {
                name: "水碾屯一村村委会",
                code: "060",
            },
            VillageCode {
                name: "水碾屯二村村委会",
                code: "061",
            },
            VillageCode {
                name: "军留庄村委会",
                code: "062",
            },
            VillageCode {
                name: "张家场村委会",
                code: "063",
            },
            VillageCode {
                name: "牛家场村委会",
                code: "064",
            },
            VillageCode {
                name: "保合庄村委会",
                code: "065",
            },
            VillageCode {
                name: "杨庄子村委会",
                code: "066",
            },
            VillageCode {
                name: "长营村委会",
                code: "067",
            },
            VillageCode {
                name: "马厂村委会",
                code: "068",
            },
            VillageCode {
                name: "高岭村委会",
                code: "069",
            },
            VillageCode {
                name: "稻田一村村委会",
                code: "070",
            },
            VillageCode {
                name: "稻田二村村委会",
                code: "071",
            },
            VillageCode {
                name: "稻田三村村委会",
                code: "072",
            },
            VillageCode {
                name: "稻田四村村委会",
                code: "073",
            },
            VillageCode {
                name: "稻田五村村委会",
                code: "074",
            },
            VillageCode {
                name: "高佃一村村委会",
                code: "075",
            },
            VillageCode {
                name: "高佃二村村委会",
                code: "076",
            },
            VillageCode {
                name: "高佃三村村委会",
                code: "077",
            },
            VillageCode {
                name: "高佃四村村委会",
                code: "078",
            },
            VillageCode {
                name: "大宁村委会",
                code: "079",
            },
            VillageCode {
                name: "温庄子村委会",
                code: "080",
            },
            VillageCode {
                name: "独义村委会",
                code: "081",
            },
            VillageCode {
                name: "朱岗子村委会",
                code: "082",
            },
            VillageCode {
                name: "阎仙垡村委会",
                code: "083",
            },
            VillageCode {
                name: "葫芦垡村委会",
                code: "084",
            },
            VillageCode {
                name: "夏场村委会",
                code: "085",
            },
            VillageCode {
                name: "佛满村委会",
                code: "086",
            },
            VillageCode {
                name: "赵庄村委会",
                code: "087",
            },
            VillageCode {
                name: "公议庄村委会",
                code: "088",
            },
            VillageCode {
                name: "西场村委会",
                code: "089",
            },
            VillageCode {
                name: "篱笆房村村委会",
                code: "090",
            },
        ],
    },
    TownCode {
        name: "河北镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "房山矿社区居委会",
                code: "001",
            },
            VillageCode {
                name: "黄土坡军工路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "惠景新苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "磁家务村委会",
                code: "004",
            },
            VillageCode {
                name: "万佛堂村委会",
                code: "005",
            },
            VillageCode {
                name: "半壁店村委会",
                code: "006",
            },
            VillageCode {
                name: "黄土坡村委会",
                code: "007",
            },
            VillageCode {
                name: "三福村委会",
                code: "008",
            },
            VillageCode {
                name: "河东村委会",
                code: "009",
            },
            VillageCode {
                name: "东庄子村委会",
                code: "010",
            },
            VillageCode {
                name: "檀木港村委会",
                code: "011",
            },
            VillageCode {
                name: "三十亩地村委会",
                code: "012",
            },
            VillageCode {
                name: "东港村委会",
                code: "013",
            },
            VillageCode {
                name: "李各庄村委会",
                code: "014",
            },
            VillageCode {
                name: "河北村委会",
                code: "015",
            },
            VillageCode {
                name: "河南村委会",
                code: "016",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "017",
            },
            VillageCode {
                name: "南道村委会",
                code: "018",
            },
            VillageCode {
                name: "杏元村委会",
                code: "019",
            },
            VillageCode {
                name: "口儿村委会",
                code: "020",
            },
            VillageCode {
                name: "他窖村委会",
                code: "021",
            },
            VillageCode {
                name: "南车营村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "长沟镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "西厢苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "长荷苑社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "南正村委会",
                code: "003",
            },
            VillageCode {
                name: "北正村委会",
                code: "004",
            },
            VillageCode {
                name: "双磨村委会",
                code: "005",
            },
            VillageCode {
                name: "南良各庄村委会",
                code: "006",
            },
            VillageCode {
                name: "北良各庄村委会",
                code: "007",
            },
            VillageCode {
                name: "东良各庄村委会",
                code: "008",
            },
            VillageCode {
                name: "东长沟村委会",
                code: "009",
            },
            VillageCode {
                name: "西长沟村委会",
                code: "010",
            },
            VillageCode {
                name: "太和庄村委会",
                code: "011",
            },
            VillageCode {
                name: "沿村村委会",
                code: "012",
            },
            VillageCode {
                name: "坟庄村委会",
                code: "013",
            },
            VillageCode {
                name: "东甘池村委会",
                code: "014",
            },
            VillageCode {
                name: "南甘池村委会",
                code: "015",
            },
            VillageCode {
                name: "北甘池村委会",
                code: "016",
            },
            VillageCode {
                name: "西甘池村委会",
                code: "017",
            },
            VillageCode {
                name: "六甲房村委会",
                code: "018",
            },
            VillageCode {
                name: "三座庵村委会",
                code: "019",
            },
            VillageCode {
                name: "黄元井村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "大石窝镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "王家磨村委会",
                code: "001",
            },
            VillageCode {
                name: "蔡庄村委会",
                code: "002",
            },
            VillageCode {
                name: "下滩村委会",
                code: "003",
            },
            VillageCode {
                name: "郑家磨村委会",
                code: "004",
            },
            VillageCode {
                name: "土堤村委会",
                code: "005",
            },
            VillageCode {
                name: "镇江营村委会",
                code: "006",
            },
            VillageCode {
                name: "塔照村委会",
                code: "007",
            },
            VillageCode {
                name: "南尚乐村委会",
                code: "008",
            },
            VillageCode {
                name: "北尚乐村委会",
                code: "009",
            },
            VillageCode {
                name: "南河村委会",
                code: "010",
            },
            VillageCode {
                name: "惠南庄村委会",
                code: "011",
            },
            VillageCode {
                name: "广润庄村委会",
                code: "012",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "013",
            },
            VillageCode {
                name: "石窝村委会",
                code: "014",
            },
            VillageCode {
                name: "半壁店村委会",
                code: "015",
            },
            VillageCode {
                name: "独树村委会",
                code: "016",
            },
            VillageCode {
                name: "岩上村委会",
                code: "017",
            },
            VillageCode {
                name: "下营村委会",
                code: "018",
            },
            VillageCode {
                name: "高庄村委会",
                code: "019",
            },
            VillageCode {
                name: "前石门村委会",
                code: "020",
            },
            VillageCode {
                name: "后石门村委会",
                code: "021",
            },
            VillageCode {
                name: "下庄村委会",
                code: "022",
            },
            VillageCode {
                name: "三岔村委会",
                code: "023",
            },
            VillageCode {
                name: "水头村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "张坊镇",
        code: "019",
        villages: &[
            VillageCode {
                name: "大峪沟村委会",
                code: "001",
            },
            VillageCode {
                name: "北白岱村委会",
                code: "002",
            },
            VillageCode {
                name: "蔡家口村委会",
                code: "003",
            },
            VillageCode {
                name: "东关上村委会",
                code: "004",
            },
            VillageCode {
                name: "三合庄村委会",
                code: "005",
            },
            VillageCode {
                name: "瓦沟村委会",
                code: "006",
            },
            VillageCode {
                name: "千河口村委会",
                code: "007",
            },
            VillageCode {
                name: "穆家口村委会",
                code: "008",
            },
            VillageCode {
                name: "广录庄村委会",
                code: "009",
            },
            VillageCode {
                name: "南白岱村委会",
                code: "010",
            },
            VillageCode {
                name: "西白岱村委会",
                code: "011",
            },
            VillageCode {
                name: "史各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "张坊村委会",
                code: "013",
            },
            VillageCode {
                name: "片上村委会",
                code: "014",
            },
            VillageCode {
                name: "下寺村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "十渡镇",
        code: "020",
        villages: &[
            VillageCode {
                name: "平峪村委会",
                code: "001",
            },
            VillageCode {
                name: "北石门村委会",
                code: "002",
            },
            VillageCode {
                name: "西石门村委会",
                code: "003",
            },
            VillageCode {
                name: "前头港村委会",
                code: "004",
            },
            VillageCode {
                name: "西河村委会",
                code: "005",
            },
            VillageCode {
                name: "西庄村委会",
                code: "006",
            },
            VillageCode {
                name: "九渡村委会",
                code: "007",
            },
            VillageCode {
                name: "八渡村委会",
                code: "008",
            },
            VillageCode {
                name: "十渡村委会",
                code: "009",
            },
            VillageCode {
                name: "马安村委会",
                code: "010",
            },
            VillageCode {
                name: "卧龙村委会",
                code: "011",
            },
            VillageCode {
                name: "六合村委会",
                code: "012",
            },
            VillageCode {
                name: "东太平村委会",
                code: "013",
            },
            VillageCode {
                name: "西太平村委会",
                code: "014",
            },
            VillageCode {
                name: "新村村委会",
                code: "015",
            },
            VillageCode {
                name: "西关上村委会",
                code: "016",
            },
            VillageCode {
                name: "六渡村委会",
                code: "017",
            },
            VillageCode {
                name: "七渡村委会",
                code: "018",
            },
            VillageCode {
                name: "五合村委会",
                code: "019",
            },
            VillageCode {
                name: "栗元厂村委会",
                code: "020",
            },
            VillageCode {
                name: "王老铺村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "青龙湖镇",
        code: "021",
        villages: &[
            VillageCode {
                name: "京煤集团化工厂社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北京昊煜京强水泥厂社区居委会",
                code: "002",
            },
            VillageCode {
                name: "宜青街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "晓幼营村委会",
                code: "004",
            },
            VillageCode {
                name: "西石府村委会",
                code: "005",
            },
            VillageCode {
                name: "常乐寺村委会",
                code: "006",
            },
            VillageCode {
                name: "北四位村委会",
                code: "007",
            },
            VillageCode {
                name: "南四位村委会",
                code: "008",
            },
            VillageCode {
                name: "焦各庄村委会",
                code: "009",
            },
            VillageCode {
                name: "小苑上村委会",
                code: "010",
            },
            VillageCode {
                name: "青龙头村委会",
                code: "011",
            },
            VillageCode {
                name: "崇各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "豆各庄村委会",
                code: "013",
            },
            VillageCode {
                name: "庙耳岗村委会",
                code: "014",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "015",
            },
            VillageCode {
                name: "芦上坟村委会",
                code: "016",
            },
            VillageCode {
                name: "大苑村委会",
                code: "017",
            },
            VillageCode {
                name: "北刘庄村委会",
                code: "018",
            },
            VillageCode {
                name: "大马村委会",
                code: "019",
            },
            VillageCode {
                name: "小马村委会",
                code: "020",
            },
            VillageCode {
                name: "果各庄村委会",
                code: "021",
            },
            VillageCode {
                name: "西庄户村委会",
                code: "022",
            },
            VillageCode {
                name: "岗上村委会",
                code: "023",
            },
            VillageCode {
                name: "坨里村委会",
                code: "024",
            },
            VillageCode {
                name: "上万村委会",
                code: "025",
            },
            VillageCode {
                name: "北车营村委会",
                code: "026",
            },
            VillageCode {
                name: "辛开口村委会",
                code: "027",
            },
            VillageCode {
                name: "漫水河村委会",
                code: "028",
            },
            VillageCode {
                name: "南观村委会",
                code: "029",
            },
            VillageCode {
                name: "口头村委会",
                code: "030",
            },
            VillageCode {
                name: "沙窝村委会",
                code: "031",
            },
            VillageCode {
                name: "大苑上村委会",
                code: "032",
            },
            VillageCode {
                name: "马家沟村委会",
                code: "033",
            },
            VillageCode {
                name: "水峪村委会",
                code: "034",
            },
            VillageCode {
                name: "石梯村委会",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "韩村河镇",
        code: "022",
        villages: &[
            VillageCode {
                name: "大自然新城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东营村委会",
                code: "002",
            },
            VillageCode {
                name: "赵各庄村委会",
                code: "003",
            },
            VillageCode {
                name: "西营村委会",
                code: "004",
            },
            VillageCode {
                name: "小次洛村委会",
                code: "005",
            },
            VillageCode {
                name: "韩村河村委会",
                code: "006",
            },
            VillageCode {
                name: "西东村委会",
                code: "007",
            },
            VillageCode {
                name: "曹章村委会",
                code: "008",
            },
            VillageCode {
                name: "七贤村委会",
                code: "009",
            },
            VillageCode {
                name: "潘家庄村委会",
                code: "010",
            },
            VillageCode {
                name: "郑庄村委会",
                code: "011",
            },
            VillageCode {
                name: "崇义村委会",
                code: "012",
            },
            VillageCode {
                name: "五侯村委会",
                code: "013",
            },
            VillageCode {
                name: "岳各庄村委会",
                code: "014",
            },
            VillageCode {
                name: "尤家坟村委会",
                code: "015",
            },
            VillageCode {
                name: "东南章村委会",
                code: "016",
            },
            VillageCode {
                name: "西南章村委会",
                code: "017",
            },
            VillageCode {
                name: "龙门口村委会",
                code: "018",
            },
            VillageCode {
                name: "二龙岗村委会",
                code: "019",
            },
            VillageCode {
                name: "皇后台村委会",
                code: "020",
            },
            VillageCode {
                name: "天开村委会",
                code: "021",
            },
            VillageCode {
                name: "东周各庄村委会",
                code: "022",
            },
            VillageCode {
                name: "西周各庄村委会",
                code: "023",
            },
            VillageCode {
                name: "上中院村委会",
                code: "024",
            },
            VillageCode {
                name: "下中院村委会",
                code: "025",
            },
            VillageCode {
                name: "孤山口村委会",
                code: "026",
            },
            VillageCode {
                name: "圣水峪村委会",
                code: "027",
            },
            VillageCode {
                name: "罗家峪村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "霞云岭乡",
        code: "023",
        villages: &[
            VillageCode {
                name: "堂上村委会",
                code: "001",
            },
            VillageCode {
                name: "大地港村委会",
                code: "002",
            },
            VillageCode {
                name: "四马台村委会",
                code: "003",
            },
            VillageCode {
                name: "龙门台村委会",
                code: "004",
            },
            VillageCode {
                name: "庄户台村委会",
                code: "005",
            },
            VillageCode {
                name: "王家台村委会",
                code: "006",
            },
            VillageCode {
                name: "石板台村委会",
                code: "007",
            },
            VillageCode {
                name: "四合村委会",
                code: "008",
            },
            VillageCode {
                name: "霞云岭村委会",
                code: "009",
            },
            VillageCode {
                name: "三流水村委会",
                code: "010",
            },
            VillageCode {
                name: "大草岭村委会",
                code: "011",
            },
            VillageCode {
                name: "上石堡村委会",
                code: "012",
            },
            VillageCode {
                name: "北直河村委会",
                code: "013",
            },
            VillageCode {
                name: "下石堡村委会",
                code: "014",
            },
            VillageCode {
                name: "银水村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "南窖乡",
        code: "024",
        villages: &[
            VillageCode {
                name: "花港村委会",
                code: "001",
            },
            VillageCode {
                name: "中窖村委会",
                code: "002",
            },
            VillageCode {
                name: "大西沟村委会",
                code: "003",
            },
            VillageCode {
                name: "水峪村委会",
                code: "004",
            },
            VillageCode {
                name: "南窖村委会",
                code: "005",
            },
            VillageCode {
                name: "北安村委会",
                code: "006",
            },
            VillageCode {
                name: "南安村委会",
                code: "007",
            },
            VillageCode {
                name: "三合村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "佛子庄乡",
        code: "025",
        villages: &[
            VillageCode {
                name: "陈家台村委会",
                code: "001",
            },
            VillageCode {
                name: "东班各庄村委会",
                code: "002",
            },
            VillageCode {
                name: "西班各庄村委会",
                code: "003",
            },
            VillageCode {
                name: "陈家坟村委会",
                code: "004",
            },
            VillageCode {
                name: "北峪村委会",
                code: "005",
            },
            VillageCode {
                name: "黑龙关村委会",
                code: "006",
            },
            VillageCode {
                name: "佛子庄村委会",
                code: "007",
            },
            VillageCode {
                name: "红煤厂村委会",
                code: "008",
            },
            VillageCode {
                name: "北窖村委会",
                code: "009",
            },
            VillageCode {
                name: "下英水村委会",
                code: "010",
            },
            VillageCode {
                name: "中英水村委会",
                code: "011",
            },
            VillageCode {
                name: "上英水村委会",
                code: "012",
            },
            VillageCode {
                name: "西安村委会",
                code: "013",
            },
            VillageCode {
                name: "查儿村委会",
                code: "014",
            },
            VillageCode {
                name: "长操村委会",
                code: "015",
            },
            VillageCode {
                name: "山川村委会",
                code: "016",
            },
            VillageCode {
                name: "贾峪口村委会",
                code: "017",
            },
            VillageCode {
                name: "石板房村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "大安山乡",
        code: "026",
        villages: &[
            VillageCode {
                name: "大安山矿社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大安山村委会",
                code: "002",
            },
            VillageCode {
                name: "西苑村委会",
                code: "003",
            },
            VillageCode {
                name: "寺尚村委会",
                code: "004",
            },
            VillageCode {
                name: "赵亩地村委会",
                code: "005",
            },
            VillageCode {
                name: "宝地洼村委会",
                code: "006",
            },
            VillageCode {
                name: "瞧煤涧村委会",
                code: "007",
            },
            VillageCode {
                name: "中山村委会",
                code: "008",
            },
            VillageCode {
                name: "水峪村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "史家营乡",
        code: "027",
        villages: &[
            VillageCode {
                name: "元阳水村委会",
                code: "001",
            },
            VillageCode {
                name: "柳林水村委会",
                code: "002",
            },
            VillageCode {
                name: "杨林水村委会",
                code: "003",
            },
            VillageCode {
                name: "青林台村委会",
                code: "004",
            },
            VillageCode {
                name: "秋林铺村委会",
                code: "005",
            },
            VillageCode {
                name: "莲花庵村委会",
                code: "006",
            },
            VillageCode {
                name: "曹家房村委会",
                code: "007",
            },
            VillageCode {
                name: "史家营村委会",
                code: "008",
            },
            VillageCode {
                name: "大村涧村委会",
                code: "009",
            },
            VillageCode {
                name: "西岳台村委会",
                code: "010",
            },
            VillageCode {
                name: "青土涧村委会",
                code: "011",
            },
            VillageCode {
                name: "金鸡台村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "蒲洼乡",
        code: "028",
        villages: &[
            VillageCode {
                name: "鱼斗泉村委会",
                code: "001",
            },
            VillageCode {
                name: "芦子水村委会",
                code: "002",
            },
            VillageCode {
                name: "东村村委会",
                code: "003",
            },
            VillageCode {
                name: "宝水村委会",
                code: "004",
            },
            VillageCode {
                name: "蒲洼村委会",
                code: "005",
            },
            VillageCode {
                name: "富合村委会",
                code: "006",
            },
            VillageCode {
                name: "森水村委会",
                code: "007",
            },
            VillageCode {
                name: "议合村委会",
                code: "008",
            },
        ],
    },
];

static TOWNS_BP_009: [TownCode; 22] = [
    TownCode {
        name: "中仓街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "悟仙观社区居委会",
                code: "001",
            },
            VillageCode {
                name: "白将军社区居委会",
                code: "002",
            },
            VillageCode {
                name: "东里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "西营社区居委会",
                code: "004",
            },
            VillageCode {
                name: "中仓社区居委会",
                code: "005",
            },
            VillageCode {
                name: "小园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "四员厅社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西上园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新华园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "莲花寺社区居委会",
                code: "010",
            },
            VillageCode {
                name: "中上园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "滨河社区居委会",
                code: "012",
            },
            VillageCode {
                name: "佟麟阁社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "商务公寓工作站社区",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "新华街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "天桥湾社区居委会",
                code: "001",
            },
            VillageCode {
                name: "如意社区居委会",
                code: "002",
            },
            VillageCode {
                name: "盛业家园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "京贸国际城社区居委会",
                code: "004",
            },
            VillageCode {
                name: "京贸北区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "河畔雅园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "保利绿地商务公寓工作站社区",
                code: "007",
            },
            VillageCode {
                name: "新光大中心商务公寓工作站社区",
                code: "008",
            },
            VillageCode {
                name: "侨商总部基地商务公寓工作站社区",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "北苑街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "新华西街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中山街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "复兴南里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "北苑桥社区居委会",
                code: "004",
            },
            VillageCode {
                name: "后南仓社区居委会",
                code: "005",
            },
            VillageCode {
                name: "新城南街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "帅府社区居委会",
                code: "007",
            },
            VillageCode {
                name: "玉带路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "西关社区居委会",
                code: "009",
            },
            VillageCode {
                name: "长桥园社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "果园西社区居委会",
                code: "011",
            },
            VillageCode {
                name: "滨惠南三街社区居委会",
                code: "012",
            },
            VillageCode {
                name: "官园社区居委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "玉桥街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "葛布店北里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "葛布店南里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "玉桥北里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "玉桥南里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "运河大街社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "乔庄北街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "梨花园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "艺苑西里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "玉桥东里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "柳岸方园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "柳馨园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "玉桥南里南社区居委会",
                code: "012",
            },
            VillageCode {
                name: "新通国际社区居委会",
                code: "013",
            },
            VillageCode {
                name: "玉桥东里南社区居委会",
                code: "014",
            },
            VillageCode {
                name: "运乔嘉园社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "艺苑东里社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "葛布店东里社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "梨园东里社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "玉桥西里社区居民委员会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "潞源街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "含英园东社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "含英园西社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "朗清园北社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "朗清园南社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "朗清园东社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "潞源社区",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "通运街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "星河社区居委会",
                code: "001",
            },
            VillageCode {
                name: "运河园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "运河湾社区居委会",
                code: "003",
            },
            VillageCode {
                name: "京贸家园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "紫荆雅园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "水仙园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "荔景园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "芙蓉社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "牡丹园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "融御社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "文景街道",
        code: "007",
        villages: &[VillageCode {
            name: "文景街道虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "九棵树街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "苏荷雅居社区居委会",
                code: "001",
            },
            VillageCode {
                name: "格瑞雅居社区居委会",
                code: "002",
            },
            VillageCode {
                name: "怡乐中街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "龙鼎园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金侨时代家园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "新城嘉园社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "翠景北里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "园景西区社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "云景里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "云景西里社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "翠屏北里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "玉兰湾社区居民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "临河里街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "土桥社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "潞阳桥社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "净水园社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "花石苑社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "运河滨江社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "铭悦园社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "砖厂南里社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "华悦园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "玫瑰园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "裕馨社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "华锦园社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "华星园社区居民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "杨庄街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "京贸国际社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新华联家园北区社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "五里店社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "锦园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "广通社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "科印社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "天时名苑社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "杨庄南里南区社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "杨庄南里西区社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "杨庄通广嘉园社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "世纪星城兴业园社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "世纪星城西社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "靓景明居社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "新华联家园南区社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "京贸南区社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "怡乐园社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "京铁社区居民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "潞邑街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "潞邑社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "潞苑南里社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "龙旺庄社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "东潞苑西区社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "运通园社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "潞苑嘉园社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "通瑞嘉苑社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "潞苑北里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "潞苑南里西社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "清水湾社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "静水园东社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "通瑞嘉苑西社区居民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "宋庄镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "宋庄村委会",
                code: "001",
            },
            VillageCode {
                name: "高各庄村委会",
                code: "002",
            },
            VillageCode {
                name: "翟里村委会",
                code: "003",
            },
            VillageCode {
                name: "北寺庄村委会",
                code: "004",
            },
            VillageCode {
                name: "小杨各庄村委会",
                code: "005",
            },
            VillageCode {
                name: "白庙村委会",
                code: "006",
            },
            VillageCode {
                name: "任庄村委会",
                code: "007",
            },
            VillageCode {
                name: "辛店村委会",
                code: "008",
            },
            VillageCode {
                name: "喇嘛庄村委会",
                code: "009",
            },
            VillageCode {
                name: "大兴庄村委会",
                code: "010",
            },
            VillageCode {
                name: "小堡村委会",
                code: "011",
            },
            VillageCode {
                name: "疃里村委会",
                code: "012",
            },
            VillageCode {
                name: "六合村委会",
                code: "013",
            },
            VillageCode {
                name: "后夏公庄村委会",
                code: "014",
            },
            VillageCode {
                name: "前夏公庄村委会",
                code: "015",
            },
            VillageCode {
                name: "邢各庄村委会",
                code: "016",
            },
            VillageCode {
                name: "丁各庄村委会",
                code: "017",
            },
            VillageCode {
                name: "高辛庄村委会",
                code: "018",
            },
            VillageCode {
                name: "菜园村委会",
                code: "019",
            },
            VillageCode {
                name: "小邓各庄村委会",
                code: "020",
            },
            VillageCode {
                name: "大邓各庄村委会",
                code: "021",
            },
            VillageCode {
                name: "师姑庄村委会",
                code: "022",
            },
            VillageCode {
                name: "北刘各庄村委会",
                code: "023",
            },
            VillageCode {
                name: "摇不动村委会",
                code: "024",
            },
            VillageCode {
                name: "关辛庄村委会",
                code: "025",
            },
            VillageCode {
                name: "西赵村委会",
                code: "026",
            },
            VillageCode {
                name: "港北村委会",
                code: "027",
            },
            VillageCode {
                name: "南马庄村委会",
                code: "028",
            },
            VillageCode {
                name: "郝各庄村委会",
                code: "029",
            },
            VillageCode {
                name: "徐辛庄村委会",
                code: "030",
            },
            VillageCode {
                name: "管头村委会",
                code: "031",
            },
            VillageCode {
                name: "吴各庄村委会",
                code: "032",
            },
            VillageCode {
                name: "葛渠村委会",
                code: "033",
            },
            VillageCode {
                name: "寨辛庄村委会",
                code: "034",
            },
            VillageCode {
                name: "寨里村委会",
                code: "035",
            },
            VillageCode {
                name: "北窑上村委会",
                code: "036",
            },
            VillageCode {
                name: "王辛庄村委会",
                code: "037",
            },
            VillageCode {
                name: "岗子村委会",
                code: "038",
            },
            VillageCode {
                name: "内军庄村委会",
                code: "039",
            },
            VillageCode {
                name: "平家疃村委会",
                code: "040",
            },
            VillageCode {
                name: "小营村委会",
                code: "041",
            },
            VillageCode {
                name: "草寺村委会",
                code: "042",
            },
            VillageCode {
                name: "尹各庄村委会",
                code: "043",
            },
            VillageCode {
                name: "富豪村委会",
                code: "044",
            },
            VillageCode {
                name: "大庞村村委会",
                code: "045",
            },
            VillageCode {
                name: "双埠头村委会",
                code: "046",
            },
            VillageCode {
                name: "沟渠庄村委会",
                code: "047",
            },
        ],
    },
    TownCode {
        name: "张家湾镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "北许场村委会",
                code: "001",
            },
            VillageCode {
                name: "张辛庄村委会",
                code: "002",
            },
            VillageCode {
                name: "上马头村委会",
                code: "003",
            },
            VillageCode {
                name: "梁各庄村委会",
                code: "004",
            },
            VillageCode {
                name: "土桥村委会",
                code: "005",
            },
            VillageCode {
                name: "皇木厂村委会",
                code: "006",
            },
            VillageCode {
                name: "南许场村委会",
                code: "007",
            },
            VillageCode {
                name: "张湾镇村委会",
                code: "008",
            },
            VillageCode {
                name: "张湾村委会",
                code: "009",
            },
            VillageCode {
                name: "大高力庄村委会",
                code: "010",
            },
            VillageCode {
                name: "上店村委会",
                code: "011",
            },
            VillageCode {
                name: "贾各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "东定福庄村委会",
                code: "013",
            },
            VillageCode {
                name: "西定福庄村委会",
                code: "014",
            },
            VillageCode {
                name: "立禅庵村委会",
                code: "015",
            },
            VillageCode {
                name: "宽街村委会",
                code: "016",
            },
            VillageCode {
                name: "唐小庄村委会",
                code: "017",
            },
            VillageCode {
                name: "施园村委会",
                code: "018",
            },
            VillageCode {
                name: "里二泗村委会",
                code: "019",
            },
            VillageCode {
                name: "烧酒巷村委会",
                code: "020",
            },
            VillageCode {
                name: "瓜厂村委会",
                code: "021",
            },
            VillageCode {
                name: "马营村委会",
                code: "022",
            },
            VillageCode {
                name: "何各庄村委会",
                code: "023",
            },
            VillageCode {
                name: "牌楼营村委会",
                code: "024",
            },
            VillageCode {
                name: "齐善庄村委会",
                code: "025",
            },
            VillageCode {
                name: "南姚园村委会",
                code: "026",
            },
            VillageCode {
                name: "大辛庄村委会",
                code: "027",
            },
            VillageCode {
                name: "枣林庄村委会",
                code: "028",
            },
            VillageCode {
                name: "姚辛庄村委会",
                code: "029",
            },
            VillageCode {
                name: "中街村委会",
                code: "030",
            },
            VillageCode {
                name: "前街村委会",
                code: "031",
            },
            VillageCode {
                name: "后街村委会",
                code: "032",
            },
            VillageCode {
                name: "苍头村委会",
                code: "033",
            },
            VillageCode {
                name: "十里庄村委会",
                code: "034",
            },
            VillageCode {
                name: "南火垡村委会",
                code: "035",
            },
            VillageCode {
                name: "三间房村委会",
                code: "036",
            },
            VillageCode {
                name: "样田村委会",
                code: "037",
            },
            VillageCode {
                name: "垡头村委会",
                code: "038",
            },
            VillageCode {
                name: "陆辛庄村委会",
                code: "039",
            },
            VillageCode {
                name: "北大化村委会",
                code: "040",
            },
            VillageCode {
                name: "大北关村委会",
                code: "041",
            },
            VillageCode {
                name: "小北关村委会",
                code: "042",
            },
            VillageCode {
                name: "南大化村委会",
                code: "043",
            },
            VillageCode {
                name: "柳营村委会",
                code: "044",
            },
            VillageCode {
                name: "高营村委会",
                code: "045",
            },
            VillageCode {
                name: "坨堤村委会",
                code: "046",
            },
            VillageCode {
                name: "西永和屯村委会",
                code: "047",
            },
            VillageCode {
                name: "东永和屯村委会",
                code: "048",
            },
            VillageCode {
                name: "王各庄村委会",
                code: "049",
            },
            VillageCode {
                name: "苍上村委会",
                code: "050",
            },
            VillageCode {
                name: "后坨村委会",
                code: "051",
            },
            VillageCode {
                name: "后青山村委会",
                code: "052",
            },
            VillageCode {
                name: "前青山村委会",
                code: "053",
            },
            VillageCode {
                name: "后南关村委会",
                code: "054",
            },
            VillageCode {
                name: "前南关村委会",
                code: "055",
            },
            VillageCode {
                name: "北仪阁村委会",
                code: "056",
            },
            VillageCode {
                name: "小耕垡村委会",
                code: "057",
            },
        ],
    },
    TownCode {
        name: "漷县镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "绿茵小区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "绿茵西区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金三角社区居委会",
                code: "003",
            },
            VillageCode {
                name: "漷县村村委会",
                code: "004",
            },
            VillageCode {
                name: "中辛庄村委会",
                code: "005",
            },
            VillageCode {
                name: "郭庄村村委会",
                code: "006",
            },
            VillageCode {
                name: "王楼村委会",
                code: "007",
            },
            VillageCode {
                name: "吴营村村委会",
                code: "008",
            },
            VillageCode {
                name: "靛庄村村委会",
                code: "009",
            },
            VillageCode {
                name: "许各庄村委会",
                code: "010",
            },
            VillageCode {
                name: "南阳村村委会",
                code: "011",
            },
            VillageCode {
                name: "翟各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "马务村村委会",
                code: "013",
            },
            VillageCode {
                name: "苏庄村村委会",
                code: "014",
            },
            VillageCode {
                name: "榆林庄村委会",
                code: "015",
            },
            VillageCode {
                name: "长凌营村村委会",
                code: "016",
            },
            VillageCode {
                name: "杨堤村村委会",
                code: "017",
            },
            VillageCode {
                name: "三黄庄村村委会",
                code: "018",
            },
            VillageCode {
                name: "后地村村委会",
                code: "019",
            },
            VillageCode {
                name: "沈庄村村委会",
                code: "020",
            },
            VillageCode {
                name: "小香仪村村委会",
                code: "021",
            },
            VillageCode {
                name: "大香仪村委会",
                code: "022",
            },
            VillageCode {
                name: "高庄村村委会",
                code: "023",
            },
            VillageCode {
                name: "东黄垡村村委会",
                code: "024",
            },
            VillageCode {
                name: "西黄垡村村委会",
                code: "025",
            },
            VillageCode {
                name: "马堤村村委会",
                code: "026",
            },
            VillageCode {
                name: "马头村村委会",
                code: "027",
            },
            VillageCode {
                name: "石槽村村委会",
                code: "028",
            },
            VillageCode {
                name: "毛庄村村委会",
                code: "029",
            },
            VillageCode {
                name: "草厂村委会",
                code: "030",
            },
            VillageCode {
                name: "南丁庄村委会",
                code: "031",
            },
            VillageCode {
                name: "东鲁村村委会",
                code: "032",
            },
            VillageCode {
                name: "西鲁村村委会",
                code: "033",
            },
            VillageCode {
                name: "周起营村委会",
                code: "034",
            },
            VillageCode {
                name: "黄厂铺村委会",
                code: "035",
            },
            VillageCode {
                name: "北堤寺村委会",
                code: "036",
            },
            VillageCode {
                name: "觅子店村委会",
                code: "037",
            },
            VillageCode {
                name: "凌庄村委会",
                code: "038",
            },
            VillageCode {
                name: "马庄村委会",
                code: "039",
            },
            VillageCode {
                name: "曹庄村委会",
                code: "040",
            },
            VillageCode {
                name: "侯黄庄村村民委员会",
                code: "041",
            },
            VillageCode {
                name: "张庄村委会",
                code: "042",
            },
            VillageCode {
                name: "东寺庄村委会",
                code: "043",
            },
            VillageCode {
                name: "小屯村委会",
                code: "044",
            },
            VillageCode {
                name: "纪各庄村委会",
                code: "045",
            },
            VillageCode {
                name: "大柳树村委会",
                code: "046",
            },
            VillageCode {
                name: "军屯村委会",
                code: "047",
            },
            VillageCode {
                name: "后尖平村委会",
                code: "048",
            },
            VillageCode {
                name: "徐官屯村委会",
                code: "049",
            },
            VillageCode {
                name: "东定安村委会",
                code: "050",
            },
            VillageCode {
                name: "西定安村委会",
                code: "051",
            },
            VillageCode {
                name: "柏庄村委会",
                code: "052",
            },
            VillageCode {
                name: "前尖平村委会",
                code: "053",
            },
            VillageCode {
                name: "李辛庄村委会",
                code: "054",
            },
            VillageCode {
                name: "尚武集村委会",
                code: "055",
            },
            VillageCode {
                name: "龙庄村委会",
                code: "056",
            },
            VillageCode {
                name: "南屯村委会",
                code: "057",
            },
            VillageCode {
                name: "穆家坟村委会",
                code: "058",
            },
            VillageCode {
                name: "军庄村委会",
                code: "059",
            },
            VillageCode {
                name: "边槐庄村委会",
                code: "060",
            },
            VillageCode {
                name: "梁家务村委会",
                code: "061",
            },
            VillageCode {
                name: "罗庄村委会",
                code: "062",
            },
            VillageCode {
                name: "后元化村委会",
                code: "063",
            },
            VillageCode {
                name: "前元化村委会",
                code: "064",
            },
        ],
    },
    TownCode {
        name: "马驹桥镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "新海南里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新海北里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新海祥和社区居委会",
                code: "003",
            },
            VillageCode {
                name: "瑞晶苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "宏仁社区居委会",
                code: "005",
            },
            VillageCode {
                name: "国风美仑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "香雪兰溪社区居委会",
                code: "007",
            },
            VillageCode {
                name: "富力尚悦居社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "逸景家园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "兴贸北街社区居委会",
                code: "010",
            },
            VillageCode {
                name: "景盛社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "一街村委会",
                code: "012",
            },
            VillageCode {
                name: "二街村委会",
                code: "013",
            },
            VillageCode {
                name: "三街村委会",
                code: "014",
            },
            VillageCode {
                name: "北门口村委会",
                code: "015",
            },
            VillageCode {
                name: "大葛庄村委会",
                code: "016",
            },
            VillageCode {
                name: "东店村委会",
                code: "017",
            },
            VillageCode {
                name: "西店村委会",
                code: "018",
            },
            VillageCode {
                name: "西后街村委会",
                code: "019",
            },
            VillageCode {
                name: "辛屯村委会",
                code: "020",
            },
            VillageCode {
                name: "大白村村委会",
                code: "021",
            },
            VillageCode {
                name: "马村村委会",
                code: "022",
            },
            VillageCode {
                name: "小白村村委会",
                code: "023",
            },
            VillageCode {
                name: "姚村村委会",
                code: "024",
            },
            VillageCode {
                name: "张各庄村委会",
                code: "025",
            },
            VillageCode {
                name: "古庄村委会",
                code: "026",
            },
            VillageCode {
                name: "杨秀店村委会",
                code: "027",
            },
            VillageCode {
                name: "周营村委会",
                code: "028",
            },
            VillageCode {
                name: "小张湾村委会",
                code: "029",
            },
            VillageCode {
                name: "房辛店村委会",
                code: "030",
            },
            VillageCode {
                name: "张村村委会",
                code: "031",
            },
            VillageCode {
                name: "郭村村委会",
                code: "032",
            },
            VillageCode {
                name: "柴务村委会",
                code: "033",
            },
            VillageCode {
                name: "大周易村委会",
                code: "034",
            },
            VillageCode {
                name: "小周易村委会",
                code: "035",
            },
            VillageCode {
                name: "史村村委会",
                code: "036",
            },
            VillageCode {
                name: "前银子村委会",
                code: "037",
            },
            VillageCode {
                name: "后银子村委会",
                code: "038",
            },
            VillageCode {
                name: "驸马庄村委会",
                code: "039",
            },
            VillageCode {
                name: "南堤村委会",
                code: "040",
            },
            VillageCode {
                name: "大杜社村委会",
                code: "041",
            },
            VillageCode {
                name: "团瓢庄村委会",
                code: "042",
            },
            VillageCode {
                name: "后堰上村委会",
                code: "043",
            },
            VillageCode {
                name: "前堰上村委会",
                code: "044",
            },
            VillageCode {
                name: "姚辛庄村委会",
                code: "045",
            },
            VillageCode {
                name: "陈各庄村委会",
                code: "046",
            },
            VillageCode {
                name: "南小营村委会",
                code: "047",
            },
            VillageCode {
                name: "西田阳村委会",
                code: "048",
            },
            VillageCode {
                name: "东田阳村委会",
                code: "049",
            },
            VillageCode {
                name: "小杜社村委会",
                code: "050",
            },
            VillageCode {
                name: "六郎庄村委会",
                code: "051",
            },
            VillageCode {
                name: "西马各庄村委会",
                code: "052",
            },
            VillageCode {
                name: "小松垡村委会",
                code: "053",
            },
            VillageCode {
                name: "大松垡村委会",
                code: "054",
            },
            VillageCode {
                name: "神驹村委会",
                code: "055",
            },
            VillageCode {
                name: "柏福村委会",
                code: "056",
            },
        ],
    },
    TownCode {
        name: "西集镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "运潮馨苑社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "西仪佳园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "西集村委会",
                code: "003",
            },
            VillageCode {
                name: "于辛庄村委会",
                code: "004",
            },
            VillageCode {
                name: "协各庄村委会",
                code: "005",
            },
            VillageCode {
                name: "侯各庄村委会",
                code: "006",
            },
            VillageCode {
                name: "胡庄村委会",
                code: "007",
            },
            VillageCode {
                name: "赵庄村委会",
                code: "008",
            },
            VillageCode {
                name: "武辛庄村委会",
                code: "009",
            },
            VillageCode {
                name: "车屯村委会",
                code: "010",
            },
            VillageCode {
                name: "前东仪村委会",
                code: "011",
            },
            VillageCode {
                name: "史东仪村委会",
                code: "012",
            },
            VillageCode {
                name: "侯东仪村村委会",
                code: "013",
            },
            VillageCode {
                name: "黄东仪村村委会",
                code: "014",
            },
            VillageCode {
                name: "尹家河村村委会",
                code: "015",
            },
            VillageCode {
                name: "林屯村村委会",
                code: "016",
            },
            VillageCode {
                name: "王上村村委会",
                code: "017",
            },
            VillageCode {
                name: "岳上村村委会",
                code: "018",
            },
            VillageCode {
                name: "石上村村委会",
                code: "019",
            },
            VillageCode {
                name: "曹刘各庄村委会",
                code: "020",
            },
            VillageCode {
                name: "东辛庄村村委会",
                code: "021",
            },
            VillageCode {
                name: "后寨府村村委会",
                code: "022",
            },
            VillageCode {
                name: "小灰店村村委会",
                code: "023",
            },
            VillageCode {
                name: "大灰店村村委会",
                code: "024",
            },
            VillageCode {
                name: "大沙务村村委会",
                code: "025",
            },
            VillageCode {
                name: "小沙务村村委会",
                code: "026",
            },
            VillageCode {
                name: "南小庄村村委会",
                code: "027",
            },
            VillageCode {
                name: "安辛庄村村委会",
                code: "028",
            },
            VillageCode {
                name: "肖家林村村委会",
                code: "029",
            },
            VillageCode {
                name: "前寨府村村委会",
                code: "030",
            },
            VillageCode {
                name: "桥上村村委会",
                code: "031",
            },
            VillageCode {
                name: "杜店村村委会",
                code: "032",
            },
            VillageCode {
                name: "牛牧屯村村委会",
                code: "033",
            },
            VillageCode {
                name: "上坡村村委会",
                code: "034",
            },
            VillageCode {
                name: "和合站村村委会",
                code: "035",
            },
            VillageCode {
                name: "吕家湾村村委会",
                code: "036",
            },
            VillageCode {
                name: "杨家洼村委会",
                code: "037",
            },
            VillageCode {
                name: "辛集村村委会",
                code: "038",
            },
            VillageCode {
                name: "郎东村委会",
                code: "039",
            },
            VillageCode {
                name: "郎西村委会",
                code: "040",
            },
            VillageCode {
                name: "马坊村委会",
                code: "041",
            },
            VillageCode {
                name: "小辛庄村委会",
                code: "042",
            },
            VillageCode {
                name: "任辛庄村村委会",
                code: "043",
            },
            VillageCode {
                name: "沙古堆村委会",
                code: "044",
            },
            VillageCode {
                name: "望君疃村委会",
                code: "045",
            },
            VillageCode {
                name: "杜柳棵村委会",
                code: "046",
            },
            VillageCode {
                name: "太平庄村委会",
                code: "047",
            },
            VillageCode {
                name: "供给店村委会",
                code: "048",
            },
            VillageCode {
                name: "儒林村委会",
                code: "049",
            },
            VillageCode {
                name: "小屯村委会",
                code: "050",
            },
            VillageCode {
                name: "张各庄村委会",
                code: "051",
            },
            VillageCode {
                name: "金各庄村委会",
                code: "052",
            },
            VillageCode {
                name: "老庄户村委会",
                code: "053",
            },
            VillageCode {
                name: "何各庄村村委会",
                code: "054",
            },
            VillageCode {
                name: "冯各庄村村委会",
                code: "055",
            },
            VillageCode {
                name: "金坨村委会",
                code: "056",
            },
            VillageCode {
                name: "王庄村村委会",
                code: "057",
            },
            VillageCode {
                name: "耿楼村村委会",
                code: "058",
            },
            VillageCode {
                name: "陈桁村委会",
                code: "059",
            },
        ],
    },
    TownCode {
        name: "台湖镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "定海园一里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "定海园二里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "润枫领尚社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "拾景园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "印象北里社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "印象南里社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "银河湾社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "璟秀欣苑社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "盛达嘉园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "东惠南里社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "东惠北里社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "通和社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "阳光花庭社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "台湖村委会",
                code: "014",
            },
            VillageCode {
                name: "铺头村委会",
                code: "015",
            },
            VillageCode {
                name: "朱家垡村委会",
                code: "016",
            },
            VillageCode {
                name: "田家府村委会",
                code: "017",
            },
            VillageCode {
                name: "前营村委会",
                code: "018",
            },
            VillageCode {
                name: "口子村委会",
                code: "019",
            },
            VillageCode {
                name: "江场村委会",
                code: "020",
            },
            VillageCode {
                name: "胡家垡村委会",
                code: "021",
            },
            VillageCode {
                name: "周坡庄村委会",
                code: "022",
            },
            VillageCode {
                name: "外郎营村委会",
                code: "023",
            },
            VillageCode {
                name: "玉甫上营村委会",
                code: "024",
            },
            VillageCode {
                name: "蒋辛庄村委会",
                code: "025",
            },
            VillageCode {
                name: "东下营村委会",
                code: "026",
            },
            VillageCode {
                name: "西下营村委会",
                code: "027",
            },
            VillageCode {
                name: "北火垡村委会",
                code: "028",
            },
            VillageCode {
                name: "唐大庄村委会",
                code: "029",
            },
            VillageCode {
                name: "北姚园村委会",
                code: "030",
            },
            VillageCode {
                name: "碱厂村委会",
                code: "031",
            },
            VillageCode {
                name: "尖垡村委会",
                code: "032",
            },
            VillageCode {
                name: "兴武林村委会",
                code: "033",
            },
            VillageCode {
                name: "窑上村委会",
                code: "034",
            },
            VillageCode {
                name: "次一村委会",
                code: "035",
            },
            VillageCode {
                name: "次二村委会",
                code: "036",
            },
            VillageCode {
                name: "垛子村委会",
                code: "037",
            },
            VillageCode {
                name: "永隆屯村委会",
                code: "038",
            },
            VillageCode {
                name: "桂家坟村委会",
                code: "039",
            },
            VillageCode {
                name: "大地村委会",
                code: "040",
            },
            VillageCode {
                name: "徐庄村委会",
                code: "041",
            },
            VillageCode {
                name: "新河村委会",
                code: "042",
            },
            VillageCode {
                name: "高古庄村委会",
                code: "043",
            },
            VillageCode {
                name: "桑元村委会",
                code: "044",
            },
            VillageCode {
                name: "水南村委会",
                code: "045",
            },
            VillageCode {
                name: "董村村委会",
                code: "046",
            },
            VillageCode {
                name: "北神树村委会",
                code: "047",
            },
            VillageCode {
                name: "丁庄村委会",
                code: "048",
            },
            VillageCode {
                name: "白庄村委会",
                code: "049",
            },
            VillageCode {
                name: "马庄村委会",
                code: "050",
            },
            VillageCode {
                name: "东石村委会",
                code: "051",
            },
            VillageCode {
                name: "麦庄村委会",
                code: "052",
            },
            VillageCode {
                name: "西太平庄村委会",
                code: "053",
            },
            VillageCode {
                name: "北小营村委会",
                code: "054",
            },
        ],
    },
    TownCode {
        name: "永乐店镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "永乐店一村村委会",
                code: "001",
            },
            VillageCode {
                name: "永乐店二村村委会",
                code: "002",
            },
            VillageCode {
                name: "永乐店三村村委会",
                code: "003",
            },
            VillageCode {
                name: "新西庄村委会",
                code: "004",
            },
            VillageCode {
                name: "陈辛庄村村委会",
                code: "005",
            },
            VillageCode {
                name: "邓庄村委会",
                code: "006",
            },
            VillageCode {
                name: "后甫村委会",
                code: "007",
            },
            VillageCode {
                name: "东张各庄村委会",
                code: "008",
            },
            VillageCode {
                name: "老槐庄村委会",
                code: "009",
            },
            VillageCode {
                name: "孔庄村委会",
                code: "010",
            },
            VillageCode {
                name: "大羊村委会",
                code: "011",
            },
            VillageCode {
                name: "小南地村委会",
                code: "012",
            },
            VillageCode {
                name: "南堤寺东村村委会",
                code: "013",
            },
            VillageCode {
                name: "南堤寺西村村委会",
                code: "014",
            },
            VillageCode {
                name: "小务村村委会",
                code: "015",
            },
            VillageCode {
                name: "西槐庄村委会",
                code: "016",
            },
            VillageCode {
                name: "坚村村委会",
                code: "017",
            },
            VillageCode {
                name: "小安村村委会",
                code: "018",
            },
            VillageCode {
                name: "后营村委会",
                code: "019",
            },
            VillageCode {
                name: "马合店村委会",
                code: "020",
            },
            VillageCode {
                name: "鲁城村委会",
                code: "021",
            },
            VillageCode {
                name: "大务村委会",
                code: "022",
            },
            VillageCode {
                name: "东河村村委会",
                code: "023",
            },
            VillageCode {
                name: "西河村村委会",
                code: "024",
            },
            VillageCode {
                name: "德仁务前街村委会",
                code: "025",
            },
            VillageCode {
                name: "德仁务中街村委会",
                code: "026",
            },
            VillageCode {
                name: "德仁务后街村委会",
                code: "027",
            },
            VillageCode {
                name: "柴厂屯村委会",
                code: "028",
            },
            VillageCode {
                name: "后马坊村委会",
                code: "029",
            },
            VillageCode {
                name: "前马坊村委会",
                code: "030",
            },
            VillageCode {
                name: "半截河村委会",
                code: "031",
            },
            VillageCode {
                name: "兴隆庄村委会",
                code: "032",
            },
            VillageCode {
                name: "三垡村委会",
                code: "033",
            },
            VillageCode {
                name: "小甸屯村委会",
                code: "034",
            },
            VillageCode {
                name: "胡家村村委会",
                code: "035",
            },
            VillageCode {
                name: "应寺村委会",
                code: "036",
            },
            VillageCode {
                name: "熬硝营村委会",
                code: "037",
            },
            VillageCode {
                name: "临沟屯村委会",
                code: "038",
            },
        ],
    },
    TownCode {
        name: "潞城镇",
        code: "019",
        villages: &[
            VillageCode {
                name: "胡各庄村委会",
                code: "001",
            },
            VillageCode {
                name: "魏庄村委会",
                code: "002",
            },
            VillageCode {
                name: "东杨庄村委会",
                code: "003",
            },
            VillageCode {
                name: "霍屯村委会",
                code: "004",
            },
            VillageCode {
                name: "古城村委会",
                code: "005",
            },
            VillageCode {
                name: "杨坨村委会",
                code: "006",
            },
            VillageCode {
                name: "郝家府村委会",
                code: "007",
            },
            VillageCode {
                name: "辛安屯村委会",
                code: "008",
            },
            VillageCode {
                name: "孙各庄村委会",
                code: "009",
            },
            VillageCode {
                name: "后屯村委会",
                code: "010",
            },
            VillageCode {
                name: "常屯村委会",
                code: "011",
            },
            VillageCode {
                name: "召里村委会",
                code: "012",
            },
            VillageCode {
                name: "堡辛村委会",
                code: "013",
            },
            VillageCode {
                name: "大台村委会",
                code: "014",
            },
            VillageCode {
                name: "前北营村委会",
                code: "015",
            },
            VillageCode {
                name: "后北营村委会",
                code: "016",
            },
            VillageCode {
                name: "大营村委会",
                code: "017",
            },
            VillageCode {
                name: "留庄村委会",
                code: "018",
            },
            VillageCode {
                name: "东夏园村委会",
                code: "019",
            },
            VillageCode {
                name: "庙上村委会",
                code: "020",
            },
            VillageCode {
                name: "东小营村委会",
                code: "021",
            },
            VillageCode {
                name: "西堡村委会",
                code: "022",
            },
            VillageCode {
                name: "东堡村委会",
                code: "023",
            },
            VillageCode {
                name: "七级村委会",
                code: "024",
            },
            VillageCode {
                name: "黎辛庄村委会",
                code: "025",
            },
            VillageCode {
                name: "南刘各庄村委会",
                code: "026",
            },
            VillageCode {
                name: "八各庄村委会",
                code: "027",
            },
            VillageCode {
                name: "侉店村委会",
                code: "028",
            },
            VillageCode {
                name: "后榆村委会",
                code: "029",
            },
            VillageCode {
                name: "前榆林庄村村委会",
                code: "030",
            },
            VillageCode {
                name: "贾后疃村委会",
                code: "031",
            },
            VillageCode {
                name: "东前营村委会",
                code: "032",
            },
            VillageCode {
                name: "前疃村委会",
                code: "033",
            },
            VillageCode {
                name: "卜落垡村委会",
                code: "034",
            },
            VillageCode {
                name: "东刘庄村委会",
                code: "035",
            },
            VillageCode {
                name: "大甘棠村委会",
                code: "036",
            },
            VillageCode {
                name: "小甘棠村委会",
                code: "037",
            },
            VillageCode {
                name: "岔道村委会",
                code: "038",
            },
            VillageCode {
                name: "凌家庙村村委会",
                code: "039",
            },
            VillageCode {
                name: "武疃村委会",
                code: "040",
            },
            VillageCode {
                name: "李疃村委会",
                code: "041",
            },
            VillageCode {
                name: "燕山营村委会",
                code: "042",
            },
            VillageCode {
                name: "兴各庄村委会",
                code: "043",
            },
            VillageCode {
                name: "肖庄村委会",
                code: "044",
            },
            VillageCode {
                name: "大豆各庄村委会",
                code: "045",
            },
            VillageCode {
                name: "小豆各庄村委会",
                code: "046",
            },
            VillageCode {
                name: "武窑村委会",
                code: "047",
            },
            VillageCode {
                name: "夏店村委会",
                code: "048",
            },
            VillageCode {
                name: "崔家楼村村委会",
                code: "049",
            },
            VillageCode {
                name: "大东各庄村委会",
                code: "050",
            },
            VillageCode {
                name: "小东各庄村委会",
                code: "051",
            },
            VillageCode {
                name: "谢楼村委会",
                code: "052",
            },
            VillageCode {
                name: "康各庄村委会",
                code: "053",
            },
            VillageCode {
                name: "太子府村委会",
                code: "054",
            },
        ],
    },
    TownCode {
        name: "永顺镇",
        code: "020",
        villages: &[
            VillageCode {
                name: "天赐良园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "富河园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西马庄社区居委会",
                code: "003",
            },
            VillageCode {
                name: "永顺南里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "永顺西里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "竹木场社区居委会",
                code: "006",
            },
            VillageCode {
                name: "杨富店社区居委会",
                code: "007",
            },
            VillageCode {
                name: "岳庄社区居委会",
                code: "008",
            },
            VillageCode {
                name: "西潞苑北区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "悦澜家园社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "惠兰美居社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "榆东一街社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "永顺东里社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "永顺村村委会",
                code: "014",
            },
            VillageCode {
                name: "北马庄村委会",
                code: "015",
            },
            VillageCode {
                name: "范庄村委会",
                code: "016",
            },
            VillageCode {
                name: "刘庄村委会",
                code: "017",
            },
            VillageCode {
                name: "李庄村村委会",
                code: "018",
            },
            VillageCode {
                name: "焦王庄村委会",
                code: "019",
            },
            VillageCode {
                name: "苏坨村委会",
                code: "020",
            },
            VillageCode {
                name: "小潞邑村委会",
                code: "021",
            },
            VillageCode {
                name: "龙旺庄村委会",
                code: "022",
            },
            VillageCode {
                name: "耿庄村村委会",
                code: "023",
            },
            VillageCode {
                name: "王家场村委会",
                code: "024",
            },
            VillageCode {
                name: "邓家窑村委会",
                code: "025",
            },
            VillageCode {
                name: "西马庄村委会",
                code: "026",
            },
            VillageCode {
                name: "新建村村委会",
                code: "027",
            },
            VillageCode {
                name: "杨庄村村委会",
                code: "028",
            },
            VillageCode {
                name: "果元村村委会",
                code: "029",
            },
            VillageCode {
                name: "南关村委会",
                code: "030",
            },
            VillageCode {
                name: "上营村村委会",
                code: "031",
            },
            VillageCode {
                name: "乔庄村村委会",
                code: "032",
            },
            VillageCode {
                name: "小圣庙村委会",
                code: "033",
            },
            VillageCode {
                name: "前上坡村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "梨园镇",
        code: "021",
        villages: &[
            VillageCode {
                name: "万盛北里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "京洲园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "群芳园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "颐瑞西里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "颐瑞东里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "欣达园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "曼城家园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "大方居社区居委会",
                code: "008",
            },
            VillageCode {
                name: "通景园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "新城乐居社区居委会",
                code: "010",
            },
            VillageCode {
                name: "怡然社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "群芳一园社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "通大家园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "大方居北区社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "晟世嘉园社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "颐瑞北区社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "群芳雅园社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "车里坟村委会",
                code: "018",
            },
            VillageCode {
                name: "三间房村委会",
                code: "019",
            },
            VillageCode {
                name: "北杨洼村委会",
                code: "020",
            },
            VillageCode {
                name: "九棵树村委会",
                code: "021",
            },
            VillageCode {
                name: "东总屯村委会",
                code: "022",
            },
            VillageCode {
                name: "西总屯村委会",
                code: "023",
            },
            VillageCode {
                name: "李老公庄村委会",
                code: "024",
            },
            VillageCode {
                name: "梨园村委会",
                code: "025",
            },
            VillageCode {
                name: "刘老公庄村委会",
                code: "026",
            },
            VillageCode {
                name: "小街一队村委会",
                code: "027",
            },
            VillageCode {
                name: "小街二队村委会",
                code: "028",
            },
            VillageCode {
                name: "小街三队村委会",
                code: "029",
            },
            VillageCode {
                name: "西小马庄村委会",
                code: "030",
            },
            VillageCode {
                name: "半壁店村委会",
                code: "031",
            },
            VillageCode {
                name: "孙王场村委会",
                code: "032",
            },
            VillageCode {
                name: "孙庄村委会",
                code: "033",
            },
            VillageCode {
                name: "砖厂村委会",
                code: "034",
            },
            VillageCode {
                name: "公庄村委会",
                code: "035",
            },
            VillageCode {
                name: "大稿村委会",
                code: "036",
            },
            VillageCode {
                name: "小稿村委会",
                code: "037",
            },
            VillageCode {
                name: "魏家坟村委会",
                code: "038",
            },
            VillageCode {
                name: "东小马庄村委会",
                code: "039",
            },
            VillageCode {
                name: "大马庄村委会",
                code: "040",
            },
            VillageCode {
                name: "高楼金村委会",
                code: "041",
            },
            VillageCode {
                name: "曹园村委会",
                code: "042",
            },
            VillageCode {
                name: "将军坟村委会",
                code: "043",
            },
        ],
    },
    TownCode {
        name: "于家务回族乡",
        code: "022",
        villages: &[
            VillageCode {
                name: "于家务西里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "永济社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "于家务村委会",
                code: "003",
            },
            VillageCode {
                name: "南仪阁村委会",
                code: "004",
            },
            VillageCode {
                name: "北辛店村委会",
                code: "005",
            },
            VillageCode {
                name: "大耕垡村委会",
                code: "006",
            },
            VillageCode {
                name: "东马各庄村委会",
                code: "007",
            },
            VillageCode {
                name: "西马坊村委会",
                code: "008",
            },
            VillageCode {
                name: "神仙村委会",
                code: "009",
            },
            VillageCode {
                name: "果村村委会",
                code: "010",
            },
            VillageCode {
                name: "渠头村村委会",
                code: "011",
            },
            VillageCode {
                name: "富各庄村村委会",
                code: "012",
            },
            VillageCode {
                name: "满庄村村委会",
                code: "013",
            },
            VillageCode {
                name: "王各庄村委会",
                code: "014",
            },
            VillageCode {
                name: "崔各庄村委会",
                code: "015",
            },
            VillageCode {
                name: "南三间房村村委会",
                code: "016",
            },
            VillageCode {
                name: "小海字村村委会",
                code: "017",
            },
            VillageCode {
                name: "枣林村村委会",
                code: "018",
            },
            VillageCode {
                name: "吴寺村村委会",
                code: "019",
            },
            VillageCode {
                name: "仇庄村村委会",
                code: "020",
            },
            VillageCode {
                name: "南刘庄村委会",
                code: "021",
            },
            VillageCode {
                name: "东垡村村委会",
                code: "022",
            },
            VillageCode {
                name: "西垡村村委会",
                code: "023",
            },
            VillageCode {
                name: "后伏村委会",
                code: "024",
            },
            VillageCode {
                name: "前伏村委会",
                code: "025",
            },
        ],
    },
];

static TOWNS_BP_010: [TownCode; 25] = [
    TownCode {
        name: "胜利街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "幸福西街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "义宾街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "义宾北区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "义宾南区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "前进社区居委会",
                code: "005",
            },
            VillageCode {
                name: "太平社区居委会",
                code: "006",
            },
            VillageCode {
                name: "胜利小区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "建新北区第一社区居委会",
                code: "008",
            },
            VillageCode {
                name: "建新北区第二社区居委会",
                code: "009",
            },
            VillageCode {
                name: "建新北区第三社区居委会",
                code: "010",
            },
            VillageCode {
                name: "建新南区第一社区居委会",
                code: "011",
            },
            VillageCode {
                name: "建新南区第二社区居委会",
                code: "012",
            },
            VillageCode {
                name: "怡馨家园第一社区居委会",
                code: "013",
            },
            VillageCode {
                name: "怡馨家园第二社区居委会",
                code: "014",
            },
            VillageCode {
                name: "龙府花园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "双兴南区第一社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "红杉一品社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "永欣嘉园社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "华玺瀚楟社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "站前北街社区居民委员会",
                code: "020",
            },
            VillageCode {
                name: "双兴南区第二社区居民委员会",
                code: "021",
            },
            VillageCode {
                name: "双兴北区第一社区居民委员会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "光明街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "滨河小区第一社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "裕龙花园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "双拥社区居委会",
                code: "003",
            },
            VillageCode {
                name: "裕龙六区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "裕龙五区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "裕龙四区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "东兴第一社区居委会",
                code: "007",
            },
            VillageCode {
                name: "东兴第二社区居委会",
                code: "008",
            },
            VillageCode {
                name: "东兴第三社区居委会",
                code: "009",
            },
            VillageCode {
                name: "双兴东区社区居委会",
                code: "010",
            },
            VillageCode {
                name: "幸福东区社区居委会",
                code: "011",
            },
            VillageCode {
                name: "裕龙三区社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "滨河小区第二社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "金汉绿港社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "绿港家园社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "裕龙北区社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "东兴第四社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "金港家园社区居民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "仁和地区",
        code: "003",
        villages: &[
            VillageCode {
                name: "太阳城第一社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "鼎顺嘉园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "港馨家园东区社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "太阳城第二社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "太阳城第三社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "石各庄村委会",
                code: "006",
            },
            VillageCode {
                name: "前进村委会",
                code: "007",
            },
            VillageCode {
                name: "复兴村委会",
                code: "008",
            },
            VillageCode {
                name: "太平村委会",
                code: "009",
            },
            VillageCode {
                name: "沙坨村委会",
                code: "010",
            },
            VillageCode {
                name: "河南村村委会",
                code: "011",
            },
            VillageCode {
                name: "胡各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "塔河村委会",
                code: "013",
            },
            VillageCode {
                name: "米各庄村委会",
                code: "014",
            },
            VillageCode {
                name: "窑坡村委会",
                code: "015",
            },
            VillageCode {
                name: "陶家坟村委会",
                code: "016",
            },
            VillageCode {
                name: "平各庄村委会",
                code: "017",
            },
            VillageCode {
                name: "北兴村委会",
                code: "018",
            },
            VillageCode {
                name: "临河村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "后沙峪地区",
        code: "004",
        villages: &[
            VillageCode {
                name: "双裕西区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "香花畦社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "江山赋社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "双裕东区社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "蓝尚家园社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "金成裕雅苑社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "博裕雅苑社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "云赋家园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "顺颐名苑社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "庆峪嘉园社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "蓝境佳园社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "西泗上村委会",
                code: "012",
            },
            VillageCode {
                name: "古城村委会",
                code: "013",
            },
            VillageCode {
                name: "罗各庄村委会",
                code: "014",
            },
            VillageCode {
                name: "马头庄村委会",
                code: "015",
            },
            VillageCode {
                name: "后沙峪村委会",
                code: "016",
            },
            VillageCode {
                name: "火神营村委会",
                code: "017",
            },
            VillageCode {
                name: "铁匠营村委会",
                code: "018",
            },
            VillageCode {
                name: "枯柳树村委会",
                code: "019",
            },
            VillageCode {
                name: "回民营村委会",
                code: "020",
            },
            VillageCode {
                name: "董各庄村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "天竺地区",
        code: "005",
        villages: &[
            VillageCode {
                name: "希望家园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "南竺园一区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "蓝天社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "天竺村委会",
                code: "004",
            },
            VillageCode {
                name: "楼台村委会",
                code: "005",
            },
            VillageCode {
                name: "岗山村委会",
                code: "006",
            },
            VillageCode {
                name: "龙山村委会",
                code: "007",
            },
            VillageCode {
                name: "桃山村委会",
                code: "008",
            },
            VillageCode {
                name: "杨二营村委会",
                code: "009",
            },
            VillageCode {
                name: "二十里堡村委会",
                code: "010",
            },
            VillageCode {
                name: "小王辛庄村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "杨镇地区",
        code: "006",
        villages: &[
            VillageCode {
                name: "杨镇双阳社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "仙泽园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "杨镇鑫园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "澜庭社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "一街村委会",
                code: "005",
            },
            VillageCode {
                name: "二街村委会",
                code: "006",
            },
            VillageCode {
                name: "三街村委会",
                code: "007",
            },
            VillageCode {
                name: "张家务村委会",
                code: "008",
            },
            VillageCode {
                name: "齐家务村委会",
                code: "009",
            },
            VillageCode {
                name: "杜庄村委会",
                code: "010",
            },
            VillageCode {
                name: "东庄户村委会",
                code: "011",
            },
            VillageCode {
                name: "老庄户村委会",
                code: "012",
            },
            VillageCode {
                name: "二郎庙村委会",
                code: "013",
            },
            VillageCode {
                name: "沟东村委会",
                code: "014",
            },
            VillageCode {
                name: "东疃村委会",
                code: "015",
            },
            VillageCode {
                name: "红寺村委会",
                code: "016",
            },
            VillageCode {
                name: "下坡村委会",
                code: "017",
            },
            VillageCode {
                name: "下营村委会",
                code: "018",
            },
            VillageCode {
                name: "安乐庄村委会",
                code: "019",
            },
            VillageCode {
                name: "汉石桥村委会",
                code: "020",
            },
            VillageCode {
                name: "沙子营村委会",
                code: "021",
            },
            VillageCode {
                name: "小店村委会",
                code: "022",
            },
            VillageCode {
                name: "辛庄子村委会",
                code: "023",
            },
            VillageCode {
                name: "田家营村委会",
                code: "024",
            },
            VillageCode {
                name: "高各庄村委会",
                code: "025",
            },
            VillageCode {
                name: "王辛庄村委会",
                code: "026",
            },
            VillageCode {
                name: "井上村委会",
                code: "027",
            },
            VillageCode {
                name: "荆坨村委会",
                code: "028",
            },
            VillageCode {
                name: "侉子营村委会",
                code: "029",
            },
            VillageCode {
                name: "李辛庄村委会",
                code: "030",
            },
            VillageCode {
                name: "松各庄村委会",
                code: "031",
            },
            VillageCode {
                name: "破罗口村委会",
                code: "032",
            },
            VillageCode {
                name: "别庄村委会",
                code: "033",
            },
            VillageCode {
                name: "徐庄村委会",
                code: "034",
            },
            VillageCode {
                name: "良庄村委会",
                code: "035",
            },
            VillageCode {
                name: "大曹庄村委会",
                code: "036",
            },
            VillageCode {
                name: "周庄村委会",
                code: "037",
            },
            VillageCode {
                name: "沙岭村委会",
                code: "038",
            },
            VillageCode {
                name: "曾庄村委会",
                code: "039",
            },
            VillageCode {
                name: "白塔村委会",
                code: "040",
            },
            VillageCode {
                name: "东焦各庄村村民委员会",
                code: "041",
            },
            VillageCode {
                name: "于庄村委会",
                code: "042",
            },
            VillageCode {
                name: "东庞村村民委员会",
                code: "043",
            },
            VillageCode {
                name: "西庞村村民委员会",
                code: "044",
            },
            VillageCode {
                name: "辛庄户村委会",
                code: "045",
            },
            VillageCode {
                name: "大三渠村委会",
                code: "046",
            },
        ],
    },
    TownCode {
        name: "牛栏山地区",
        code: "007",
        villages: &[
            VillageCode {
                name: "牛栏山第一社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "香醍漫步社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "香醍溪岸社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "好望山社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "安纳湖社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "赢麓家园社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "北孙各庄村委会",
                code: "007",
            },
            VillageCode {
                name: "龙王头村委会",
                code: "008",
            },
            VillageCode {
                name: "富各庄村委会",
                code: "009",
            },
            VillageCode {
                name: "北军营村委会",
                code: "010",
            },
            VillageCode {
                name: "芦正卷村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "相各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "官志卷村委会",
                code: "013",
            },
            VillageCode {
                name: "东范各庄村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "后晏子村委会",
                code: "015",
            },
            VillageCode {
                name: "前晏子村委会",
                code: "016",
            },
            VillageCode {
                name: "兰家营村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "姚各庄村委会",
                code: "018",
            },
            VillageCode {
                name: "半壁店村委会",
                code: "019",
            },
            VillageCode {
                name: "张家庄村委会",
                code: "020",
            },
            VillageCode {
                name: "下坡屯村委会",
                code: "021",
            },
            VillageCode {
                name: "史家口村委会",
                code: "022",
            },
            VillageCode {
                name: "金牛村委会",
                code: "023",
            },
            VillageCode {
                name: "先进村委会",
                code: "024",
            },
            VillageCode {
                name: "禾丰村委会",
                code: "025",
            },
            VillageCode {
                name: "安乐村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "南法信地区",
        code: "008",
        villages: &[
            VillageCode {
                name: "华英园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东海洪村委会",
                code: "002",
            },
            VillageCode {
                name: "西海洪村委会",
                code: "003",
            },
            VillageCode {
                name: "南卷村委会",
                code: "004",
            },
            VillageCode {
                name: "三家店村委会",
                code: "005",
            },
            VillageCode {
                name: "东杜兰村委会",
                code: "006",
            },
            VillageCode {
                name: "西杜兰村委会",
                code: "007",
            },
            VillageCode {
                name: "北法信村委会",
                code: "008",
            },
            VillageCode {
                name: "焦各庄村委会",
                code: "009",
            },
            VillageCode {
                name: "大江洼村委会",
                code: "010",
            },
            VillageCode {
                name: "刘家河村委会",
                code: "011",
            },
            VillageCode {
                name: "南法信村委会",
                code: "012",
            },
            VillageCode {
                name: "十里堡村委会",
                code: "013",
            },
            VillageCode {
                name: "马家营村委会",
                code: "014",
            },
            VillageCode {
                name: "哨马营村委会",
                code: "015",
            },
            VillageCode {
                name: "冯家营村委会",
                code: "016",
            },
            VillageCode {
                name: "卸甲营村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "马坡地区",
        code: "009",
        villages: &[
            VillageCode {
                name: "佳和宜园第一社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "佳和宜园第二社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "中晟馨苑北区社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "泰和宜园西区社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "良正卷村委会",
                code: "005",
            },
            VillageCode {
                name: "庙卷村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "衙门村委会",
                code: "007",
            },
            VillageCode {
                name: "石家营村委会",
                code: "008",
            },
            VillageCode {
                name: "毛家营村委会",
                code: "009",
            },
            VillageCode {
                name: "姚店村委会",
                code: "010",
            },
            VillageCode {
                name: "马卷村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "石园街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "五里仓第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "五里仓第二社区居委会",
                code: "002",
            },
            VillageCode {
                name: "石园西区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "石园东区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "石园北区第一社区居委会",
                code: "005",
            },
            VillageCode {
                name: "石园北区第二社区居委会",
                code: "006",
            },
            VillageCode {
                name: "石园南区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "燕京社区居委会",
                code: "008",
            },
            VillageCode {
                name: "石园东苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "港馨家园第一社区居委会",
                code: "010",
            },
            VillageCode {
                name: "港馨家园第二社区居委会",
                code: "011",
            },
            VillageCode {
                name: "石园北区第三社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "仁和花园第一社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "仁和花园第二社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "合院第一社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "港馨家园第三社区居委会",
                code: "016",
            },
            VillageCode {
                name: "合院第二社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "仁和花园第三社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "晟品景园社区居民委员会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "空港街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "蓝星花园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "万科城市花园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "裕祥花园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "三山新新家园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "天一社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "新国展国际社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "莲竹花园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "莫奈花园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "吉祥花园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "翠竹新村第一社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "翠竹新村第二社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "双龙社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "天竺新新家园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "中粮祥云社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "香蜜湾社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "誉天下社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "天竺花园社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "满庭芳社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "嘉园社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "首航社区居民委员会",
                code: "020",
            },
            VillageCode {
                name: "优山美地社区居民委员会",
                code: "021",
            },
            VillageCode {
                name: "枫泉花园社区居民委员会",
                code: "022",
            },
            VillageCode {
                name: "金港嘉园社区居民委员会",
                code: "023",
            },
            VillageCode {
                name: "金地社区居民委员会",
                code: "024",
            },
            VillageCode {
                name: "诺德阅墅社区居民委员会",
                code: "025",
            },
            VillageCode {
                name: "西白辛庄村委会",
                code: "026",
            },
            VillageCode {
                name: "燕王庄村委会",
                code: "027",
            },
            VillageCode {
                name: "西田各庄村委会",
                code: "028",
            },
            VillageCode {
                name: "东庄村委会",
                code: "029",
            },
            VillageCode {
                name: "花梨坎村委会",
                code: "030",
            },
            VillageCode {
                name: "薛大人庄村委会",
                code: "031",
            },
            VillageCode {
                name: "前沙峪村委会",
                code: "032",
            },
            VillageCode {
                name: "吉祥庄村委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "双丰街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "马坡花园第一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "马坡花园第二社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "富力湾社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "泰和宜园第一社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "新马家园社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "顺悦家园第一社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "金宝花园社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "香悦第一社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "北辰花园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "中晟馨苑社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "香悦第二社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "顺兴社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "鲁能润园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "花溪渡社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "鲁能溪园社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "顺悦家园第二社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "兰庭珑湾社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "国誉府社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "香悦第三社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "向阳村委会",
                code: "020",
            },
            VillageCode {
                name: "西丰乐村委会",
                code: "021",
            },
            VillageCode {
                name: "北上坡村委会",
                code: "022",
            },
            VillageCode {
                name: "大营村委会",
                code: "023",
            },
            VillageCode {
                name: "东马坡村委会",
                code: "024",
            },
            VillageCode {
                name: "肖家坡村委会",
                code: "025",
            },
            VillageCode {
                name: "东丰乐村委会",
                code: "026",
            },
            VillageCode {
                name: "小孙各庄村委会",
                code: "027",
            },
            VillageCode {
                name: "西马坡村委会",
                code: "028",
            },
            VillageCode {
                name: "白各庄村委会",
                code: "029",
            },
            VillageCode {
                name: "秦武姚村委会",
                code: "030",
            },
            VillageCode {
                name: "荆卷村委会",
                code: "031",
            },
            VillageCode {
                name: "向前村委会",
                code: "032",
            },
            VillageCode {
                name: "庄头村委会",
                code: "033",
            },
            VillageCode {
                name: "泥河村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "旺泉街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "铁十六局社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西辛第一社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西辛社区居委会",
                code: "003",
            },
            VillageCode {
                name: "西辛北区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "宏城花园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "前进花园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "牡丹苑社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "望泉家园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "梅兰家园社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "澜西园二区社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "澜西园三区社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "澜西园四区社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "悦君家园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "玉兰苑社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "梅香社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "望泉西里北社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "望泉西里南一社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "望泉西里南二社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "望泉西里南三社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "望泉西里南四社区居民委员会",
                code: "020",
            },
            VillageCode {
                name: "龙泉苑社区居民委员会",
                code: "021",
            },
            VillageCode {
                name: "石景苑社区居民委员会",
                code: "022",
            },
            VillageCode {
                name: "石门苑第一社区居民委员会",
                code: "023",
            },
            VillageCode {
                name: "石门苑第二社区居民委员会",
                code: "024",
            },
            VillageCode {
                name: "石门村委会",
                code: "025",
            },
            VillageCode {
                name: "沙井村委会",
                code: "026",
            },
            VillageCode {
                name: "望泉寺村委会",
                code: "027",
            },
            VillageCode {
                name: "梅沟营村委会",
                code: "028",
            },
            VillageCode {
                name: "军营村委会",
                code: "029",
            },
            VillageCode {
                name: "吴家营村委会",
                code: "030",
            },
            VillageCode {
                name: "杨家营村委会",
                code: "031",
            },
            VillageCode {
                name: "杜各庄村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "高丽营镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "丽喜花园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "观唐社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "一村村委会",
                code: "003",
            },
            VillageCode {
                name: "二村村委会",
                code: "004",
            },
            VillageCode {
                name: "三村村委会",
                code: "005",
            },
            VillageCode {
                name: "四村村委会",
                code: "006",
            },
            VillageCode {
                name: "五村村委会",
                code: "007",
            },
            VillageCode {
                name: "六村村委会",
                code: "008",
            },
            VillageCode {
                name: "七村村委会",
                code: "009",
            },
            VillageCode {
                name: "八村村委会",
                code: "010",
            },
            VillageCode {
                name: "南王路村委会",
                code: "011",
            },
            VillageCode {
                name: "北王路村委会",
                code: "012",
            },
            VillageCode {
                name: "西王路村委会",
                code: "013",
            },
            VillageCode {
                name: "唐自头村委会",
                code: "014",
            },
            VillageCode {
                name: "于庄村委会",
                code: "015",
            },
            VillageCode {
                name: "张喜庄村委会",
                code: "016",
            },
            VillageCode {
                name: "东马各庄村委会",
                code: "017",
            },
            VillageCode {
                name: "西马各庄村委会",
                code: "018",
            },
            VillageCode {
                name: "夏县营村委会",
                code: "019",
            },
            VillageCode {
                name: "河津营村委会",
                code: "020",
            },
            VillageCode {
                name: "南郎中村委会",
                code: "021",
            },
            VillageCode {
                name: "后渠河村委会",
                code: "022",
            },
            VillageCode {
                name: "前渠河村委会",
                code: "023",
            },
            VillageCode {
                name: "闫家营村委会",
                code: "024",
            },
            VillageCode {
                name: "水坡村委会",
                code: "025",
            },
            VillageCode {
                name: "羊房村委会",
                code: "026",
            },
            VillageCode {
                name: "文化营村委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "李桥镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "樱花园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "馨港庄园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "畅顺园社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "苏活社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "李家桥村委会",
                code: "005",
            },
            VillageCode {
                name: "英各庄村委会",
                code: "006",
            },
            VillageCode {
                name: "张辛村委会",
                code: "007",
            },
            VillageCode {
                name: "临清村委会",
                code: "008",
            },
            VillageCode {
                name: "南半壁店村委会",
                code: "009",
            },
            VillageCode {
                name: "后桥村委会",
                code: "010",
            },
            VillageCode {
                name: "庄子营村委会",
                code: "011",
            },
            VillageCode {
                name: "洼子村委会",
                code: "012",
            },
            VillageCode {
                name: "头二营村委会",
                code: "013",
            },
            VillageCode {
                name: "三四营村委会",
                code: "014",
            },
            VillageCode {
                name: "西树行村委会",
                code: "015",
            },
            VillageCode {
                name: "西大坨村委会",
                code: "016",
            },
            VillageCode {
                name: "北河村委会",
                code: "017",
            },
            VillageCode {
                name: "沙浮村委会",
                code: "018",
            },
            VillageCode {
                name: "王家场村委会",
                code: "019",
            },
            VillageCode {
                name: "沿河村委会",
                code: "020",
            },
            VillageCode {
                name: "芦各庄村委会",
                code: "021",
            },
            VillageCode {
                name: "史庄村委会",
                code: "022",
            },
            VillageCode {
                name: "吴庄村委会",
                code: "023",
            },
            VillageCode {
                name: "永青村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "郭庄村委会",
                code: "025",
            },
            VillageCode {
                name: "南河村委会",
                code: "026",
            },
            VillageCode {
                name: "北桃园村委会",
                code: "027",
            },
            VillageCode {
                name: "南桃园村委会",
                code: "028",
            },
            VillageCode {
                name: "安里村委会",
                code: "029",
            },
            VillageCode {
                name: "苏庄村委会",
                code: "030",
            },
            VillageCode {
                name: "官庄村委会",
                code: "031",
            },
            VillageCode {
                name: "堡子村委会",
                code: "032",
            },
            VillageCode {
                name: "沮沟村委会",
                code: "033",
            },
            VillageCode {
                name: "北庄头村委会",
                code: "034",
            },
            VillageCode {
                name: "南庄头村委会",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "李遂镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "宣庄户村委会",
                code: "001",
            },
            VillageCode {
                name: "魏辛庄村委会",
                code: "002",
            },
            VillageCode {
                name: "后营村委会",
                code: "003",
            },
            VillageCode {
                name: "前营村委会",
                code: "004",
            },
            VillageCode {
                name: "葛代子村委会",
                code: "005",
            },
            VillageCode {
                name: "沟北村委会",
                code: "006",
            },
            VillageCode {
                name: "柳各庄村委会",
                code: "007",
            },
            VillageCode {
                name: "李遂村委会",
                code: "008",
            },
            VillageCode {
                name: "西营村委会",
                code: "009",
            },
            VillageCode {
                name: "东营村委会",
                code: "010",
            },
            VillageCode {
                name: "李庄村委会",
                code: "011",
            },
            VillageCode {
                name: "崇国庄村委会",
                code: "012",
            },
            VillageCode {
                name: "陈庄村委会",
                code: "013",
            },
            VillageCode {
                name: "赵庄村委会",
                code: "014",
            },
            VillageCode {
                name: "太平辛庄村委会",
                code: "015",
            },
            VillageCode {
                name: "牌楼村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "南彩镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "彩丰社区居委会",
                code: "001",
            },
            VillageCode {
                name: "月亮湾绿色家园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "前薛各庄村委会",
                code: "003",
            },
            VillageCode {
                name: "后薛各庄村委会",
                code: "004",
            },
            VillageCode {
                name: "南彩村委会",
                code: "005",
            },
            VillageCode {
                name: "坞里村委会",
                code: "006",
            },
            VillageCode {
                name: "双营村委会",
                code: "007",
            },
            VillageCode {
                name: "南小营村委会",
                code: "008",
            },
            VillageCode {
                name: "洼里村委会",
                code: "009",
            },
            VillageCode {
                name: "望渠村委会",
                code: "010",
            },
            VillageCode {
                name: "道仙庄村委会",
                code: "011",
            },
            VillageCode {
                name: "东江头村委会",
                code: "012",
            },
            VillageCode {
                name: "西江头村委会",
                code: "013",
            },
            VillageCode {
                name: "于辛庄村委会",
                code: "014",
            },
            VillageCode {
                name: "大兴庄村委会",
                code: "015",
            },
            VillageCode {
                name: "太平庄村委会",
                code: "016",
            },
            VillageCode {
                name: "水屯村委会",
                code: "017",
            },
            VillageCode {
                name: "九王庄村委会",
                code: "018",
            },
            VillageCode {
                name: "前俸伯村委会",
                code: "019",
            },
            VillageCode {
                name: "后俸伯村委会",
                code: "020",
            },
            VillageCode {
                name: "河北村村委会",
                code: "021",
            },
            VillageCode {
                name: "杜刘庄村委会",
                code: "022",
            },
            VillageCode {
                name: "北彩村委会",
                code: "023",
            },
            VillageCode {
                name: "柳行村委会",
                code: "024",
            },
            VillageCode {
                name: "黄家场村委会",
                code: "025",
            },
            VillageCode {
                name: "桥头村委会",
                code: "026",
            },
            VillageCode {
                name: "前郝家疃村委会",
                code: "027",
            },
            VillageCode {
                name: "后郝家疃村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "北务镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "北务村委会",
                code: "001",
            },
            VillageCode {
                name: "郭家务村委会",
                code: "002",
            },
            VillageCode {
                name: "陈辛庄村委会",
                code: "003",
            },
            VillageCode {
                name: "林上村委会",
                code: "004",
            },
            VillageCode {
                name: "仓上村委会",
                code: "005",
            },
            VillageCode {
                name: "道口村委会",
                code: "006",
            },
            VillageCode {
                name: "王各庄村委会",
                code: "007",
            },
            VillageCode {
                name: "闫家渠村委会",
                code: "008",
            },
            VillageCode {
                name: "南辛庄户村委会",
                code: "009",
            },
            VillageCode {
                name: "于地村委会",
                code: "010",
            },
            VillageCode {
                name: "庄子村委会",
                code: "011",
            },
            VillageCode {
                name: "小珠宝村委会",
                code: "012",
            },
            VillageCode {
                name: "东地村委会",
                code: "013",
            },
            VillageCode {
                name: "珠宝屯村委会",
                code: "014",
            },
            VillageCode {
                name: "马庄村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "大孙各庄镇",
        code: "019",
        villages: &[
            VillageCode {
                name: "大孙各庄村委会",
                code: "001",
            },
            VillageCode {
                name: "客家庄村委会",
                code: "002",
            },
            VillageCode {
                name: "西辛庄村委会",
                code: "003",
            },
            VillageCode {
                name: "户耳山村委会",
                code: "004",
            },
            VillageCode {
                name: "宗家店村委会",
                code: "005",
            },
            VillageCode {
                name: "柴家林村委会",
                code: "006",
            },
            VillageCode {
                name: "顾家庄村委会",
                code: "007",
            },
            VillageCode {
                name: "小故现村委会",
                code: "008",
            },
            VillageCode {
                name: "田各庄村委会",
                code: "009",
            },
            VillageCode {
                name: "吴雄寺村委会",
                code: "010",
            },
            VillageCode {
                name: "小宋各庄村委会",
                code: "011",
            },
            VillageCode {
                name: "小塘村委会",
                code: "012",
            },
            VillageCode {
                name: "南聂庄村委会",
                code: "013",
            },
            VillageCode {
                name: "王户庄村委会",
                code: "014",
            },
            VillageCode {
                name: "龙庭侯村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "老公庄村委会",
                code: "016",
            },
            VillageCode {
                name: "大坝洼庄村委会",
                code: "017",
            },
            VillageCode {
                name: "小坝洼庄村委会",
                code: "018",
            },
            VillageCode {
                name: "大塘村委会",
                code: "019",
            },
            VillageCode {
                name: "佟辛庄村委会",
                code: "020",
            },
            VillageCode {
                name: "薛庄村委会",
                code: "021",
            },
            VillageCode {
                name: "前岭上村委会",
                code: "022",
            },
            VillageCode {
                name: "后岭上村委会",
                code: "023",
            },
            VillageCode {
                name: "东华山村委会",
                code: "024",
            },
            VillageCode {
                name: "西华山村委会",
                code: "025",
            },
            VillageCode {
                name: "大段村村民委员会",
                code: "026",
            },
            VillageCode {
                name: "小段村村民委员会",
                code: "027",
            },
            VillageCode {
                name: "谢辛庄村委会",
                code: "028",
            },
            VillageCode {
                name: "赵家峪村委会",
                code: "029",
            },
            VillageCode {
                name: "湘王庄村委会",
                code: "030",
            },
            VillageCode {
                name: "四福庄村委会",
                code: "031",
            },
            VillageCode {
                name: "后陆马村村民委员会",
                code: "032",
            },
            VillageCode {
                name: "前陆马村村民委员会",
                code: "033",
            },
            VillageCode {
                name: "西尹家府村委会",
                code: "034",
            },
            VillageCode {
                name: "东尹家府村委会",
                code: "035",
            },
            VillageCode {
                name: "大崔各庄村委会",
                code: "036",
            },
            VillageCode {
                name: "大石各庄村委会",
                code: "037",
            },
            VillageCode {
                name: "大田庄村委会",
                code: "038",
            },
            VillageCode {
                name: "大洛泡村委会",
                code: "039",
            },
        ],
    },
    TownCode {
        name: "张镇",
        code: "020",
        villages: &[
            VillageCode {
                name: "永强家园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "浅山香邑社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "良山村委会",
                code: "003",
            },
            VillageCode {
                name: "小三渠村委会",
                code: "004",
            },
            VillageCode {
                name: "麻林山村委会",
                code: "005",
            },
            VillageCode {
                name: "贾家洼子村委会",
                code: "006",
            },
            VillageCode {
                name: "李家洼子村委会",
                code: "007",
            },
            VillageCode {
                name: "吕布屯村委会",
                code: "008",
            },
            VillageCode {
                name: "雁户庄村委会",
                code: "009",
            },
            VillageCode {
                name: "港西村委会",
                code: "010",
            },
            VillageCode {
                name: "大故现村委会",
                code: "011",
            },
            VillageCode {
                name: "刘辛庄村委会",
                code: "012",
            },
            VillageCode {
                name: "张各庄村委会",
                code: "013",
            },
            VillageCode {
                name: "厂门口村委会",
                code: "014",
            },
            VillageCode {
                name: "虫王庙村委会",
                code: "015",
            },
            VillageCode {
                name: "北营村委会",
                code: "016",
            },
            VillageCode {
                name: "西营村委会",
                code: "017",
            },
            VillageCode {
                name: "小曹庄村委会",
                code: "018",
            },
            VillageCode {
                name: "驻马庄村委会",
                code: "019",
            },
            VillageCode {
                name: "柏树庄村委会",
                code: "020",
            },
            VillageCode {
                name: "白辛庄村委会",
                code: "021",
            },
            VillageCode {
                name: "赵各庄村委会",
                code: "022",
            },
            VillageCode {
                name: "后王会村委会",
                code: "023",
            },
            VillageCode {
                name: "前王会村委会",
                code: "024",
            },
            VillageCode {
                name: "后苏桥村委会",
                code: "025",
            },
            VillageCode {
                name: "前苏桥村委会",
                code: "026",
            },
            VillageCode {
                name: "王庄村委会",
                code: "027",
            },
            VillageCode {
                name: "聂庄村委会",
                code: "028",
            },
            VillageCode {
                name: "朱庄村委会",
                code: "029",
            },
            VillageCode {
                name: "侯庄村委会",
                code: "030",
            },
            VillageCode {
                name: "行宫村委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "龙湾屯镇",
        code: "021",
        villages: &[
            VillageCode {
                name: "山里辛庄村委会",
                code: "001",
            },
            VillageCode {
                name: "七连庄村委会",
                code: "002",
            },
            VillageCode {
                name: "柳庄户村委会",
                code: "003",
            },
            VillageCode {
                name: "南坞村委会",
                code: "004",
            },
            VillageCode {
                name: "树行村委会",
                code: "005",
            },
            VillageCode {
                name: "张中坞村委会",
                code: "006",
            },
            VillageCode {
                name: "史中坞村委会",
                code: "007",
            },
            VillageCode {
                name: "丁甲庄村委会",
                code: "008",
            },
            VillageCode {
                name: "小北坞村委会",
                code: "009",
            },
            VillageCode {
                name: "大北坞村委会",
                code: "010",
            },
            VillageCode {
                name: "焦庄户村委会",
                code: "011",
            },
            VillageCode {
                name: "唐洞村委会",
                code: "012",
            },
            VillageCode {
                name: "龙湾屯村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "木林镇",
        code: "022",
        villages: &[
            VillageCode {
                name: "木林村委会",
                code: "001",
            },
            VillageCode {
                name: "陈各庄村委会",
                code: "002",
            },
            VillageCode {
                name: "蒋各庄村委会",
                code: "003",
            },
            VillageCode {
                name: "魏家店村委会",
                code: "004",
            },
            VillageCode {
                name: "东沿头村委会",
                code: "005",
            },
            VillageCode {
                name: "西沿头村委会",
                code: "006",
            },
            VillageCode {
                name: "长林庄村委会",
                code: "007",
            },
            VillageCode {
                name: "孝德村委会",
                code: "008",
            },
            VillageCode {
                name: "唐指山村委会",
                code: "009",
            },
            VillageCode {
                name: "贾山村委会",
                code: "010",
            },
            VillageCode {
                name: "茶棚村委会",
                code: "011",
            },
            VillageCode {
                name: "安辛庄村委会",
                code: "012",
            },
            VillageCode {
                name: "王泮庄村委会",
                code: "013",
            },
            VillageCode {
                name: "大韩庄村委会",
                code: "014",
            },
            VillageCode {
                name: "小韩庄村委会",
                code: "015",
            },
            VillageCode {
                name: "马坊村委会",
                code: "016",
            },
            VillageCode {
                name: "上园子村委会",
                code: "017",
            },
            VillageCode {
                name: "大林村委会",
                code: "018",
            },
            VillageCode {
                name: "陈家坨村委会",
                code: "019",
            },
            VillageCode {
                name: "李各庄村委会",
                code: "020",
            },
            VillageCode {
                name: "业兴庄村委会",
                code: "021",
            },
            VillageCode {
                name: "陀头庙村委会",
                code: "022",
            },
            VillageCode {
                name: "荣各庄村委会",
                code: "023",
            },
            VillageCode {
                name: "前王各庄村委会",
                code: "024",
            },
            VillageCode {
                name: "后王各庄村委会",
                code: "025",
            },
            VillageCode {
                name: "潘家坟村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "北小营镇",
        code: "023",
        villages: &[
            VillageCode {
                name: "永利社区居委会",
                code: "001",
            },
            VillageCode {
                name: "水色时光花园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北小营村委会",
                code: "003",
            },
            VillageCode {
                name: "上辇村委会",
                code: "004",
            },
            VillageCode {
                name: "北府村委会",
                code: "005",
            },
            VillageCode {
                name: "东乌鸡村委会",
                code: "006",
            },
            VillageCode {
                name: "西乌鸡村委会",
                code: "007",
            },
            VillageCode {
                name: "榆林村委会",
                code: "008",
            },
            VillageCode {
                name: "后礼务村委会",
                code: "009",
            },
            VillageCode {
                name: "前礼务村委会",
                code: "010",
            },
            VillageCode {
                name: "马辛庄村委会",
                code: "011",
            },
            VillageCode {
                name: "前鲁各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "后鲁各庄村委会",
                code: "013",
            },
            VillageCode {
                name: "仇家店村委会",
                code: "014",
            },
            VillageCode {
                name: "西府村委会",
                code: "015",
            },
            VillageCode {
                name: "东府村委会",
                code: "016",
            },
            VillageCode {
                name: "小胡营村委会",
                code: "017",
            },
            VillageCode {
                name: "大胡营村委会",
                code: "018",
            },
            VillageCode {
                name: "牛富屯村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "北石槽镇",
        code: "024",
        villages: &[
            VillageCode {
                name: "西赵各庄村委会",
                code: "001",
            },
            VillageCode {
                name: "下西市村委会",
                code: "002",
            },
            VillageCode {
                name: "良善庄村委会",
                code: "003",
            },
            VillageCode {
                name: "西范各庄村委会",
                code: "004",
            },
            VillageCode {
                name: "南石槽村委会",
                code: "005",
            },
            VillageCode {
                name: "北石槽村委会",
                code: "006",
            },
            VillageCode {
                name: "东石槽村委会",
                code: "007",
            },
            VillageCode {
                name: "东辛庄村委会",
                code: "008",
            },
            VillageCode {
                name: "寺上村委会",
                code: "009",
            },
            VillageCode {
                name: "营尔村委会",
                code: "010",
            },
            VillageCode {
                name: "武各庄村委会",
                code: "011",
            },
            VillageCode {
                name: "刘各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "中滩营村委会",
                code: "013",
            },
            VillageCode {
                name: "二张营村委会",
                code: "014",
            },
            VillageCode {
                name: "大柳树营村委会",
                code: "015",
            },
            VillageCode {
                name: "李家史山村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "赵全营镇",
        code: "025",
        villages: &[
            VillageCode {
                name: "板桥新苑社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "禧悦家园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "西小营村委会",
                code: "003",
            },
            VillageCode {
                name: "北郎中村委会",
                code: "004",
            },
            VillageCode {
                name: "前桑园村委会",
                code: "005",
            },
            VillageCode {
                name: "后桑园村委会",
                code: "006",
            },
            VillageCode {
                name: "白庙村委会",
                code: "007",
            },
            VillageCode {
                name: "马家堡村委会",
                code: "008",
            },
            VillageCode {
                name: "大官庄村委会",
                code: "009",
            },
            VillageCode {
                name: "小官庄村委会",
                code: "010",
            },
            VillageCode {
                name: "西陈各庄村委会",
                code: "011",
            },
            VillageCode {
                name: "赵全营村委会",
                code: "012",
            },
            VillageCode {
                name: "小高丽村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "去碑营村委会",
                code: "014",
            },
            VillageCode {
                name: "豹房村委会",
                code: "015",
            },
            VillageCode {
                name: "忻州营村委会",
                code: "016",
            },
            VillageCode {
                name: "红铜营村委会",
                code: "017",
            },
            VillageCode {
                name: "板桥村委会",
                code: "018",
            },
            VillageCode {
                name: "西绛州营村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "东绛洲营村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "稷山营村委会",
                code: "021",
            },
            VillageCode {
                name: "东水泉村委会",
                code: "022",
            },
            VillageCode {
                name: "西水泉村委会",
                code: "023",
            },
            VillageCode {
                name: "联庄村委会",
                code: "024",
            },
            VillageCode {
                name: "河庄村委会",
                code: "025",
            },
            VillageCode {
                name: "解放村委会",
                code: "026",
            },
            VillageCode {
                name: "燕华营村委会",
                code: "027",
            },
        ],
    },
];

static TOWNS_BP_011: [TownCode; 22] = [
    TownCode {
        name: "城北街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "一街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "二街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "三街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "五街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "六街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "八街社区居委会",
                code: "006",
            },
            VillageCode {
                name: "西关社区居委会",
                code: "007",
            },
            VillageCode {
                name: "永安社区居委会",
                code: "008",
            },
            VillageCode {
                name: "清秀园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "松园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "朝凤社区居委会",
                code: "011",
            },
            VillageCode {
                name: "政法社区居委会",
                code: "012",
            },
            VillageCode {
                name: "水关社区居委会",
                code: "013",
            },
            VillageCode {
                name: "南环里社区居委会",
                code: "014",
            },
            VillageCode {
                name: "燕平路社区居委会",
                code: "015",
            },
            VillageCode {
                name: "亢山社区居委会",
                code: "016",
            },
            VillageCode {
                name: "东关北里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "安福苑社区居委会",
                code: "018",
            },
            VillageCode {
                name: "北城根社区居委会",
                code: "019",
            },
            VillageCode {
                name: "建明里社区居委会",
                code: "020",
            },
            VillageCode {
                name: "玉虚观社区居委会",
                code: "021",
            },
            VillageCode {
                name: "史家坑社区居委会",
                code: "022",
            },
            VillageCode {
                name: "西环里社区居委会",
                code: "023",
            },
            VillageCode {
                name: "京科苑社区居委会",
                code: "024",
            },
            VillageCode {
                name: "东关南里社区居委会",
                code: "025",
            },
            VillageCode {
                name: "城角路社区居委会",
                code: "026",
            },
            VillageCode {
                name: "宁馨苑社区居委会",
                code: "027",
            },
            VillageCode {
                name: "国通家园社区居委会",
                code: "028",
            },
            VillageCode {
                name: "五城社区居委会",
                code: "029",
            },
            VillageCode {
                name: "灰厂路社区居委会",
                code: "030",
            },
            VillageCode {
                name: "富松社区居委会",
                code: "031",
            },
            VillageCode {
                name: "南关社区居委会",
                code: "032",
            },
            VillageCode {
                name: "创新园社区居委会",
                code: "033",
            },
            VillageCode {
                name: "昌园社区居委会",
                code: "034",
            },
            VillageCode {
                name: "亢山前路社区居委会",
                code: "035",
            },
            VillageCode {
                name: "北环社区居委会",
                code: "036",
            },
            VillageCode {
                name: "宅新苑社区居委会",
                code: "037",
            },
            VillageCode {
                name: "嘉和园社区居委会",
                code: "038",
            },
            VillageCode {
                name: "裕祥社区居委会",
                code: "039",
            },
            VillageCode {
                name: "新悦家园社区居委会",
                code: "040",
            },
            VillageCode {
                name: "京科苑东区社区居委会",
                code: "041",
            },
            VillageCode {
                name: "观山悦社区居委会",
                code: "042",
            },
            VillageCode {
                name: "创新园南区社区居委会",
                code: "043",
            },
            VillageCode {
                name: "怡园社区居委会",
                code: "044",
            },
            VillageCode {
                name: "宽街社区居委会",
                code: "045",
            },
            VillageCode {
                name: "创新园东区社区居委会",
                code: "046",
            },
            VillageCode {
                name: "二街村委会",
                code: "047",
            },
            VillageCode {
                name: "三街村委会",
                code: "048",
            },
            VillageCode {
                name: "六街村委会",
                code: "049",
            },
            VillageCode {
                name: "西关村委会",
                code: "050",
            },
            VillageCode {
                name: "朝凤村委会",
                code: "051",
            },
        ],
    },
    TownCode {
        name: "南口地区",
        code: "002",
        villages: &[
            VillageCode {
                name: "十一条社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金隅旺和园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "兴隆街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "隆盛街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "水厂路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "南口村社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新兴路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "玻璃公司社区居委会",
                code: "008",
            },
            VillageCode {
                name: "南厂东社区居委会",
                code: "009",
            },
            VillageCode {
                name: "南厂西社区居委会",
                code: "010",
            },
            VillageCode {
                name: "保温瓶公司社区居委会",
                code: "011",
            },
            VillageCode {
                name: "南农社区居委会",
                code: "012",
            },
            VillageCode {
                name: "太平庄村委会",
                code: "013",
            },
            VillageCode {
                name: "虎峪村委会",
                code: "014",
            },
            VillageCode {
                name: "陈庄村委会",
                code: "015",
            },
            VillageCode {
                name: "红泥沟村委会",
                code: "016",
            },
            VillageCode {
                name: "雪山村委会",
                code: "017",
            },
            VillageCode {
                name: "龙虎台村委会",
                code: "018",
            },
            VillageCode {
                name: "燕磨峪村委会",
                code: "019",
            },
            VillageCode {
                name: "七间房村委会",
                code: "020",
            },
            VillageCode {
                name: "辛力庄村委会",
                code: "021",
            },
            VillageCode {
                name: "南口村委会",
                code: "022",
            },
            VillageCode {
                name: "龙潭村委会",
                code: "023",
            },
            VillageCode {
                name: "居庸关村委会",
                code: "024",
            },
            VillageCode {
                name: "羊台子村委会",
                code: "025",
            },
            VillageCode {
                name: "马庄村委会",
                code: "026",
            },
            VillageCode {
                name: "南口镇村委会",
                code: "027",
            },
            VillageCode {
                name: "马坊村委会",
                code: "028",
            },
            VillageCode {
                name: "后桃洼村委会",
                code: "029",
            },
            VillageCode {
                name: "前桃洼村委会",
                code: "030",
            },
            VillageCode {
                name: "长水峪村委会",
                code: "031",
            },
            VillageCode {
                name: "檀峪村委会",
                code: "032",
            },
            VillageCode {
                name: "王庄村委会",
                code: "033",
            },
            VillageCode {
                name: "曹庄村委会",
                code: "034",
            },
            VillageCode {
                name: "花塔村委会",
                code: "035",
            },
            VillageCode {
                name: "兴隆口村委会",
                code: "036",
            },
            VillageCode {
                name: "新元村委会",
                code: "037",
            },
            VillageCode {
                name: "东李庄村委会",
                code: "038",
            },
            VillageCode {
                name: "西李庄村委会",
                code: "039",
            },
            VillageCode {
                name: "响潭村委会",
                code: "040",
            },
        ],
    },
    TownCode {
        name: "马池口地区",
        code: "003",
        villages: &[
            VillageCode {
                name: "念头社区居委会",
                code: "001",
            },
            VillageCode {
                name: "马池口村委会",
                code: "002",
            },
            VillageCode {
                name: "东坨村委会",
                code: "003",
            },
            VillageCode {
                name: "西坨村委会",
                code: "004",
            },
            VillageCode {
                name: "东闸村委会",
                code: "005",
            },
            VillageCode {
                name: "北庄户村委会",
                code: "006",
            },
            VillageCode {
                name: "楼自庄村委会",
                code: "007",
            },
            VillageCode {
                name: "土城村委会",
                code: "008",
            },
            VillageCode {
                name: "横桥村委会",
                code: "009",
            },
            VillageCode {
                name: "白浮村委会",
                code: "010",
            },
            VillageCode {
                name: "下念头村委会",
                code: "011",
            },
            VillageCode {
                name: "宏道村委会",
                code: "012",
            },
            VillageCode {
                name: "上念头村委会",
                code: "013",
            },
            VillageCode {
                name: "百泉庄村委会",
                code: "014",
            },
            VillageCode {
                name: "奤夿屯村委会",
                code: "015",
            },
            VillageCode {
                name: "亭自庄村委会",
                code: "016",
            },
            VillageCode {
                name: "北小营村委会",
                code: "017",
            },
            VillageCode {
                name: "乃干屯村委会",
                code: "018",
            },
            VillageCode {
                name: "丈头村委会",
                code: "019",
            },
            VillageCode {
                name: "辛店村委会",
                code: "020",
            },
            VillageCode {
                name: "土楼村委会",
                code: "021",
            },
            VillageCode {
                name: "葛村村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "沙河地区",
        code: "004",
        villages: &[
            VillageCode {
                name: "南一社区居委会",
                code: "001",
            },
            VillageCode {
                name: "路松东街社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "东一社区居委会",
                code: "003",
            },
            VillageCode {
                name: "西二社区居委会",
                code: "004",
            },
            VillageCode {
                name: "北二社区居委会",
                code: "005",
            },
            VillageCode {
                name: "站前路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "沙阳路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "保利罗兰香谷社区居委会",
                code: "008",
            },
            VillageCode {
                name: "兆丰家园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "北街家园第一社区居委会",
                code: "010",
            },
            VillageCode {
                name: "北街家园第二社区居委会",
                code: "011",
            },
            VillageCode {
                name: "北街家园第三社区居委会",
                code: "012",
            },
            VillageCode {
                name: "碧水庄园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "于善街南社区居委会",
                code: "014",
            },
            VillageCode {
                name: "冠芳园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "五福家园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "巩华新村社区居委会",
                code: "017",
            },
            VillageCode {
                name: "滟澜新宸社区居委会",
                code: "018",
            },
            VillageCode {
                name: "恒大幸福家园第一社区居委会",
                code: "019",
            },
            VillageCode {
                name: "恒大幸福家园第二社区居委会",
                code: "020",
            },
            VillageCode {
                name: "路松街社区居委会",
                code: "021",
            },
            VillageCode {
                name: "紫荆香谷社区居委会",
                code: "022",
            },
            VillageCode {
                name: "祥业家园社区居委会",
                code: "023",
            },
            VillageCode {
                name: "北街家园第四社区居委会",
                code: "024",
            },
            VillageCode {
                name: "北街家园第五社区居委会",
                code: "025",
            },
            VillageCode {
                name: "丽春湖社区居民委员会",
                code: "026",
            },
            VillageCode {
                name: "西沙屯村委会",
                code: "027",
            },
            VillageCode {
                name: "老牛湾村委会",
                code: "028",
            },
            VillageCode {
                name: "南一村委会",
                code: "029",
            },
            VillageCode {
                name: "东一村委会",
                code: "030",
            },
            VillageCode {
                name: "西二村委会",
                code: "031",
            },
            VillageCode {
                name: "北二村委会",
                code: "032",
            },
            VillageCode {
                name: "辛力屯村委会",
                code: "033",
            },
            VillageCode {
                name: "路庄村委会",
                code: "034",
            },
            VillageCode {
                name: "踩河村委会",
                code: "035",
            },
            VillageCode {
                name: "于辛庄村委会",
                code: "036",
            },
            VillageCode {
                name: "满井东队村委会",
                code: "037",
            },
            VillageCode {
                name: "满井西队村委会",
                code: "038",
            },
            VillageCode {
                name: "松兰堡村委会",
                code: "039",
            },
            VillageCode {
                name: "王庄村委会",
                code: "040",
            },
            VillageCode {
                name: "小寨村委会",
                code: "041",
            },
            VillageCode {
                name: "大洼村委会",
                code: "042",
            },
            VillageCode {
                name: "七里渠南村委会",
                code: "043",
            },
            VillageCode {
                name: "七里渠北村委会",
                code: "044",
            },
            VillageCode {
                name: "白各庄村委会",
                code: "045",
            },
            VillageCode {
                name: "豆各庄村委会",
                code: "046",
            },
            VillageCode {
                name: "小沙河村委会",
                code: "047",
            },
        ],
    },
    TownCode {
        name: "城南街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "龙凤山砂石厂社区居委会",
                code: "001",
            },
            VillageCode {
                name: "凉水河社区居委会",
                code: "002",
            },
            VillageCode {
                name: "化庄社区居委会",
                code: "003",
            },
            VillageCode {
                name: "山峡社区居委会",
                code: "004",
            },
            VillageCode {
                name: "昌盛园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "郝庄家园北区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "拓然家苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "水屯家园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "秋实家园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新汇园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "畅春阁社区居委会",
                code: "011",
            },
            VillageCode {
                name: "郝庄家园西区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "世涛天朗社区居委会",
                code: "013",
            },
            VillageCode {
                name: "富泉花园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "介山社区居委会",
                code: "015",
            },
            VillageCode {
                name: "龙山锦园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "绿海家园社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "水屯村委会",
                code: "018",
            },
            VillageCode {
                name: "南郝庄村委会",
                code: "019",
            },
            VillageCode {
                name: "旧县村委会",
                code: "020",
            },
            VillageCode {
                name: "北郝庄村委会",
                code: "021",
            },
            VillageCode {
                name: "邓庄村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "东小口地区",
        code: "006",
        villages: &[
            VillageCode {
                name: "九台社区居委会",
                code: "001",
            },
            VillageCode {
                name: "都市芳园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "森林大第家园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "森林大第家园南区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "悦府家园东区社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "悦府家园西区社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "东小口村委会",
                code: "007",
            },
            VillageCode {
                name: "中滩村委会",
                code: "008",
            },
            VillageCode {
                name: "芦家村村委会",
                code: "009",
            },
            VillageCode {
                name: "单家村村委会",
                code: "010",
            },
            VillageCode {
                name: "店上村委会",
                code: "011",
            },
            VillageCode {
                name: "兰各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "半截塔村委会",
                code: "013",
            },
            VillageCode {
                name: "魏窑村委会",
                code: "014",
            },
            VillageCode {
                name: "小辛庄村委会",
                code: "015",
            },
            VillageCode {
                name: "马连店村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "天通苑北街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "天通东苑第三北社区居委会",
                code: "001",
            },
            VillageCode {
                name: "天通西苑第二西社区居委会",
                code: "002",
            },
            VillageCode {
                name: "天通西苑第三社区居委会",
                code: "003",
            },
            VillageCode {
                name: "天通西苑第四社区居委会",
                code: "004",
            },
            VillageCode {
                name: "天通中苑第一社区居委会",
                code: "005",
            },
            VillageCode {
                name: "太平家园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "天通中苑第二社区居委会",
                code: "007",
            },
            VillageCode {
                name: "天通中苑第三社区居委会",
                code: "008",
            },
            VillageCode {
                name: "天通北苑第一东社区居委会",
                code: "009",
            },
            VillageCode {
                name: "天通北苑第一西社区居委会",
                code: "010",
            },
            VillageCode {
                name: "天通北苑第二东社区居委会",
                code: "011",
            },
            VillageCode {
                name: "天通北苑第二西社区居委会",
                code: "012",
            },
            VillageCode {
                name: "天通北苑第三东社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "天通北苑第三西社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "天通西苑第二东社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "天通东苑第三南社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "太平庄村委会",
                code: "017",
            },
            VillageCode {
                name: "白坊村委会",
                code: "018",
            },
            VillageCode {
                name: "狮子营村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "天通苑南街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "东辰社区居委会",
                code: "001",
            },
            VillageCode {
                name: "佳运园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "天通苑第二社区居委会",
                code: "003",
            },
            VillageCode {
                name: "天通西苑第一社区居委会",
                code: "004",
            },
            VillageCode {
                name: "天通东苑第一社区居委会",
                code: "005",
            },
            VillageCode {
                name: "天通东苑第二社区居委会",
                code: "006",
            },
            VillageCode {
                name: "嘉诚花园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "清水园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "北方明珠社区居委会",
                code: "009",
            },
            VillageCode {
                name: "天通东苑第四社区居委会",
                code: "010",
            },
            VillageCode {
                name: "顶秀青溪社区居委会",
                code: "011",
            },
            VillageCode {
                name: "奥北中心社区居委会",
                code: "012",
            },
            VillageCode {
                name: "溪城珑原社区居委会",
                code: "013",
            },
            VillageCode {
                name: "正辰中心社区居委会",
                code: "014",
            },
            VillageCode {
                name: "天通苑第六社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "天通东苑第二北社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "溪城家园社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "天通东苑第一东社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "天通苑第一社区居委会",
                code: "019",
            },
            VillageCode {
                name: "陈营村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "霍营街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "华龙苑南里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "华龙苑北里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "蓝天园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "天鑫家园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "霍营小区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "上坡佳园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "华龙苑中里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "流星花园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "龙回苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "和谐家园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "田园风光雅苑社区居委会",
                code: "011",
            },
            VillageCode {
                name: "龙锦苑一区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "龙锦苑东一区社区居委会",
                code: "013",
            },
            VillageCode {
                name: "龙锦苑东二区社区居委会",
                code: "014",
            },
            VillageCode {
                name: "龙锦苑东五区社区居委会",
                code: "015",
            },
            VillageCode {
                name: "龙锦苑东三区社区居委会",
                code: "016",
            },
            VillageCode {
                name: "龙锦苑东四区社区居委会",
                code: "017",
            },
            VillageCode {
                name: "紫金新干线社区居委会",
                code: "018",
            },
            VillageCode {
                name: "霍家营社区居委会",
                code: "019",
            },
            VillageCode {
                name: "流星花园南区社区居民委员会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "回龙观街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "龙博苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "万润家园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "龙城社区居委会",
                code: "003",
            },
            VillageCode {
                name: "万龙社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东村家园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "龙乡社区居委会",
                code: "006",
            },
            VillageCode {
                name: "吉晟别墅社区居委会",
                code: "007",
            },
            VillageCode {
                name: "南店社区居委会",
                code: "008",
            },
            VillageCode {
                name: "金榜园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新康园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "龙兴园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "龙兴园北区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "瑞旗家园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "二拨子社区居委会",
                code: "014",
            },
            VillageCode {
                name: "蓝天嘉园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "回龙观新村社区居委会",
                code: "016",
            },
            VillageCode {
                name: "融泽家园一区社区居委会",
                code: "017",
            },
            VillageCode {
                name: "融泽家园二区社区居委会",
                code: "018",
            },
            VillageCode {
                name: "金域华府社区居委会",
                code: "019",
            },
            VillageCode {
                name: "金域国际社区居委会",
                code: "020",
            },
            VillageCode {
                name: "融泽家园三区社区居委会",
                code: "021",
            },
            VillageCode {
                name: "新龙城西区社区居委会",
                code: "022",
            },
            VillageCode {
                name: "新龙城东区社区居委会",
                code: "023",
            },
            VillageCode {
                name: "瑞旗家园二区社区居民委员会",
                code: "024",
            },
            VillageCode {
                name: "北京人家社区居民委员会",
                code: "025",
            },
            VillageCode {
                name: "上奥世纪社区居民委员会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "龙泽园街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "佰嘉城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "龙华园二区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "龙华园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "慧华苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "通达园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "龙泽苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "龙泽苑东区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "龙腾苑二区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "龙腾苑三区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "龙腾苑四区社区居委会",
                code: "010",
            },
            VillageCode {
                name: "龙腾苑五区社区居委会",
                code: "011",
            },
            VillageCode {
                name: "龙腾苑六区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "风雅园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "龙禧苑社区居委会",
                code: "014",
            },
            VillageCode {
                name: "龙禧苑二区社区居委会",
                code: "015",
            },
            VillageCode {
                name: "龙锦苑二区社区居委会",
                code: "016",
            },
            VillageCode {
                name: "龙锦苑四区社区居委会",
                code: "017",
            },
            VillageCode {
                name: "龙锦苑五区社区居委会",
                code: "018",
            },
            VillageCode {
                name: "龙锦苑六区社区居委会",
                code: "019",
            },
            VillageCode {
                name: "龙跃苑一区社区居委会",
                code: "020",
            },
            VillageCode {
                name: "龙跃苑二区社区居委会",
                code: "021",
            },
            VillageCode {
                name: "龙跃苑三区社区居委会",
                code: "022",
            },
            VillageCode {
                name: "龙跃苑四区社区居委会",
                code: "023",
            },
            VillageCode {
                name: "龙跃苑东二区社区居委会",
                code: "024",
            },
            VillageCode {
                name: "龙跃苑东四五社区居委会",
                code: "025",
            },
            VillageCode {
                name: "北郊农场社区居委会",
                code: "026",
            },
            VillageCode {
                name: "天龙苑社区居委会",
                code: "027",
            },
            VillageCode {
                name: "北店嘉园社区居委会",
                code: "028",
            },
            VillageCode {
                name: "良庄家园社区居委会",
                code: "029",
            },
            VillageCode {
                name: "智慧社社区居委会",
                code: "030",
            },
            VillageCode {
                name: "国仕汇社区居委会",
                code: "031",
            },
            VillageCode {
                name: "国风美唐社区居委会",
                code: "032",
            },
            VillageCode {
                name: "天露园社区居委会",
                code: "033",
            },
            VillageCode {
                name: "云趣园南里社区居委会",
                code: "034",
            },
            VillageCode {
                name: "云趣园北里社区居委会",
                code: "035",
            },
            VillageCode {
                name: "三合庄村委会",
                code: "036",
            },
        ],
    },
    TownCode {
        name: "史各庄街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "农学院社区居委会",
                code: "001",
            },
            VillageCode {
                name: "昌艺园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "领秀慧谷社区居委会",
                code: "003",
            },
            VillageCode {
                name: "领秀慧谷北区居委会",
                code: "004",
            },
            VillageCode {
                name: "朱辛庄村委会",
                code: "005",
            },
            VillageCode {
                name: "东半壁店村委会",
                code: "006",
            },
            VillageCode {
                name: "西半壁店村委会",
                code: "007",
            },
            VillageCode {
                name: "定福皇庄村委会",
                code: "008",
            },
            VillageCode {
                name: "史各庄村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "阳坊镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "阳坊社区居委会",
                code: "001",
            },
            VillageCode {
                name: "阳坊村委会",
                code: "002",
            },
            VillageCode {
                name: "前白虎涧村委会",
                code: "003",
            },
            VillageCode {
                name: "后白虎涧村委会",
                code: "004",
            },
            VillageCode {
                name: "东贯市村委会",
                code: "005",
            },
            VillageCode {
                name: "西贯市村委会",
                code: "006",
            },
            VillageCode {
                name: "八口村委会",
                code: "007",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "008",
            },
            VillageCode {
                name: "史家桥村委会",
                code: "009",
            },
            VillageCode {
                name: "西马坊村委会",
                code: "010",
            },
            VillageCode {
                name: "四家庄村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "小汤山镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "市场街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大东流社区居委会",
                code: "002",
            },
            VillageCode {
                name: "太阳城社区居委会",
                code: "003",
            },
            VillageCode {
                name: "汤南社区居委会",
                code: "004",
            },
            VillageCode {
                name: "龙脉社区居委会",
                code: "005",
            },
            VillageCode {
                name: "金汤社区居委会",
                code: "006",
            },
            VillageCode {
                name: "小汤山村委会",
                code: "007",
            },
            VillageCode {
                name: "尚信村委会",
                code: "008",
            },
            VillageCode {
                name: "讲礼村委会",
                code: "009",
            },
            VillageCode {
                name: "马坊村委会",
                code: "010",
            },
            VillageCode {
                name: "官牛坊村委会",
                code: "011",
            },
            VillageCode {
                name: "阿苏卫村委会",
                code: "012",
            },
            VillageCode {
                name: "葫芦河村委会",
                code: "013",
            },
            VillageCode {
                name: "大柳树村委会",
                code: "014",
            },
            VillageCode {
                name: "大汤山村委会",
                code: "015",
            },
            VillageCode {
                name: "后牛坊村委会",
                code: "016",
            },
            VillageCode {
                name: "大东流村委会",
                code: "017",
            },
            VillageCode {
                name: "土沟村委会",
                code: "018",
            },
            VillageCode {
                name: "酸枣岭村委会",
                code: "019",
            },
            VillageCode {
                name: "前蔺沟村委会",
                code: "020",
            },
            VillageCode {
                name: "后蔺沟村委会",
                code: "021",
            },
            VillageCode {
                name: "小东流村委会",
                code: "022",
            },
            VillageCode {
                name: "常兴庄村委会",
                code: "023",
            },
            VillageCode {
                name: "大赴任庄村委会",
                code: "024",
            },
            VillageCode {
                name: "小赴任庄村委会",
                code: "025",
            },
            VillageCode {
                name: "赴任辛庄村委会",
                code: "026",
            },
            VillageCode {
                name: "南官庄村委会",
                code: "027",
            },
            VillageCode {
                name: "赖马庄村委会",
                code: "028",
            },
            VillageCode {
                name: "西官庄村委会",
                code: "029",
            },
            VillageCode {
                name: "东官庄村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "南邵镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "北郡嘉源社区居委会",
                code: "001",
            },
            VillageCode {
                name: "长滩庭苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "廊桥水岸社区居委会",
                code: "003",
            },
            VillageCode {
                name: "国惠村社区居委会",
                code: "004",
            },
            VillageCode {
                name: "路劲家园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "风景丽苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "麓鸣花园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "青秀尚城一区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "桥东社区居委会",
                code: "009",
            },
            VillageCode {
                name: "青秀尚城二区社区居委会",
                code: "010",
            },
            VillageCode {
                name: "南邵村委会",
                code: "011",
            },
            VillageCode {
                name: "姜屯村委会",
                code: "012",
            },
            VillageCode {
                name: "张各庄村委会",
                code: "013",
            },
            VillageCode {
                name: "景文屯村委会",
                code: "014",
            },
            VillageCode {
                name: "纪窑村委会",
                code: "015",
            },
            VillageCode {
                name: "金家坟村委会",
                code: "016",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "017",
            },
            VillageCode {
                name: "四合庄村委会",
                code: "018",
            },
            VillageCode {
                name: "东营村委会",
                code: "019",
            },
            VillageCode {
                name: "张营村委会",
                code: "020",
            },
            VillageCode {
                name: "何营村委会",
                code: "021",
            },
            VillageCode {
                name: "小北哨村委会",
                code: "022",
            },
            VillageCode {
                name: "北邵洼村委会",
                code: "023",
            },
            VillageCode {
                name: "官高村委会",
                code: "024",
            },
            VillageCode {
                name: "三合庄村委会",
                code: "025",
            },
            VillageCode {
                name: "营坊村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "崔村镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "西崔村委会",
                code: "001",
            },
            VillageCode {
                name: "西辛峰村委会",
                code: "002",
            },
            VillageCode {
                name: "大辛峰村委会",
                code: "003",
            },
            VillageCode {
                name: "棉山村委会",
                code: "004",
            },
            VillageCode {
                name: "南庄营村委会",
                code: "005",
            },
            VillageCode {
                name: "南庄村委会",
                code: "006",
            },
            VillageCode {
                name: "东崔村委会",
                code: "007",
            },
            VillageCode {
                name: "真顺村委会",
                code: "008",
            },
            VillageCode {
                name: "麻峪村委会",
                code: "009",
            },
            VillageCode {
                name: "香堂村委会",
                code: "010",
            },
            VillageCode {
                name: "西峪村委会",
                code: "011",
            },
            VillageCode {
                name: "八家村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "百善镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "善缘家园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "林溪园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "百善村委会",
                code: "003",
            },
            VillageCode {
                name: "吕各庄村委会",
                code: "004",
            },
            VillageCode {
                name: "半壁街村委会",
                code: "005",
            },
            VillageCode {
                name: "下东廓村委会",
                code: "006",
            },
            VillageCode {
                name: "上东廓村委会",
                code: "007",
            },
            VillageCode {
                name: "牛房圈村委会",
                code: "008",
            },
            VillageCode {
                name: "二德庄村委会",
                code: "009",
            },
            VillageCode {
                name: "东沙屯村委会",
                code: "010",
            },
            VillageCode {
                name: "孟祖村委会",
                code: "011",
            },
            VillageCode {
                name: "良各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "狮子营村委会",
                code: "013",
            },
            VillageCode {
                name: "泥洼村委会",
                code: "014",
            },
            VillageCode {
                name: "钟家营村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "北七家镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "燕城苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "王府公寓社区居委会",
                code: "002",
            },
            VillageCode {
                name: "望都家园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "宏福苑西区社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "冠华苑西区社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "温泉花园A区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "温泉花园B区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西湖新村社区居委会",
                code: "008",
            },
            VillageCode {
                name: "名佳花园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "名流花园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "蓬莱公寓社区居委会",
                code: "011",
            },
            VillageCode {
                name: "北亚花园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "王府花园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "八仙别墅社区居委会",
                code: "014",
            },
            VillageCode {
                name: "桃园公寓社区居委会",
                code: "015",
            },
            VillageCode {
                name: "冠雅苑社区居委会",
                code: "016",
            },
            VillageCode {
                name: "宏福苑社区居委会",
                code: "017",
            },
            VillageCode {
                name: "美树假日嘉园社区居委会",
                code: "018",
            },
            VillageCode {
                name: "望都新地社区居委会",
                code: "019",
            },
            VillageCode {
                name: "宏福苑东区社区居委会",
                code: "020",
            },
            VillageCode {
                name: "金色漫香苑社区居委会",
                code: "021",
            },
            VillageCode {
                name: "枫树家园社区居委会",
                code: "022",
            },
            VillageCode {
                name: "冠华苑社区居委会",
                code: "023",
            },
            VillageCode {
                name: "世纪星城社区居委会",
                code: "024",
            },
            VillageCode {
                name: "名佳花园三区社区居委会",
                code: "025",
            },
            VillageCode {
                name: "沟自头村委会",
                code: "026",
            },
            VillageCode {
                name: "北七家村委会",
                code: "027",
            },
            VillageCode {
                name: "岭上村委会",
                code: "028",
            },
            VillageCode {
                name: "鲁疃村委会",
                code: "029",
            },
            VillageCode {
                name: "东二旗村委会",
                code: "030",
            },
            VillageCode {
                name: "羊各庄村委会",
                code: "031",
            },
            VillageCode {
                name: "八仙庄村委会",
                code: "032",
            },
            VillageCode {
                name: "曹碾村委会",
                code: "033",
            },
            VillageCode {
                name: "白庙村委会",
                code: "034",
            },
            VillageCode {
                name: "东三旗村委会",
                code: "035",
            },
            VillageCode {
                name: "平西府村委会",
                code: "036",
            },
            VillageCode {
                name: "平坊村委会",
                code: "037",
            },
            VillageCode {
                name: "西沙各庄村委会",
                code: "038",
            },
            VillageCode {
                name: "郑各庄村委会",
                code: "039",
            },
            VillageCode {
                name: "东沙各庄村委会",
                code: "040",
            },
            VillageCode {
                name: "燕丹村委会",
                code: "041",
            },
            VillageCode {
                name: "歇甲庄村委会",
                code: "042",
            },
            VillageCode {
                name: "南七家庄村委会",
                code: "043",
            },
            VillageCode {
                name: "海鶄落村委会",
                code: "044",
            },
        ],
    },
    TownCode {
        name: "兴寿镇",
        code: "019",
        villages: &[
            VillageCode {
                name: "兴寿村委会",
                code: "001",
            },
            VillageCode {
                name: "肖村村委会",
                code: "002",
            },
            VillageCode {
                name: "香屯村委会",
                code: "003",
            },
            VillageCode {
                name: "沙坨村委会",
                code: "004",
            },
            VillageCode {
                name: "东庄村委会",
                code: "005",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "006",
            },
            VillageCode {
                name: "桃林村委会",
                code: "007",
            },
            VillageCode {
                name: "秦城村委会",
                code: "008",
            },
            VillageCode {
                name: "象房村委会",
                code: "009",
            },
            VillageCode {
                name: "东新城村委会",
                code: "010",
            },
            VillageCode {
                name: "东营村委会",
                code: "011",
            },
            VillageCode {
                name: "西营村委会",
                code: "012",
            },
            VillageCode {
                name: "麦庄村委会",
                code: "013",
            },
            VillageCode {
                name: "西新城村委会",
                code: "014",
            },
            VillageCode {
                name: "下苑村委会",
                code: "015",
            },
            VillageCode {
                name: "秦家屯村委会",
                code: "016",
            },
            VillageCode {
                name: "上苑村委会",
                code: "017",
            },
            VillageCode {
                name: "桃峪口村委会",
                code: "018",
            },
            VillageCode {
                name: "暴峪泉村委会",
                code: "019",
            },
            VillageCode {
                name: "半壁店村委会",
                code: "020",
            },
            VillageCode {
                name: "上西市村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "流村镇",
        code: "020",
        villages: &[
            VillageCode {
                name: "北庄村村委会",
                code: "001",
            },
            VillageCode {
                name: "下店村委会",
                code: "002",
            },
            VillageCode {
                name: "上店村委会",
                code: "003",
            },
            VillageCode {
                name: "南流村委会",
                code: "004",
            },
            VillageCode {
                name: "西峰山村委会",
                code: "005",
            },
            VillageCode {
                name: "北流村委会",
                code: "006",
            },
            VillageCode {
                name: "新建村委会",
                code: "007",
            },
            VillageCode {
                name: "白羊城村委会",
                code: "008",
            },
            VillageCode {
                name: "古将村委会",
                code: "009",
            },
            VillageCode {
                name: "黑寨村委会",
                code: "010",
            },
            VillageCode {
                name: "王家园村委会",
                code: "011",
            },
            VillageCode {
                name: "高崖口村委会",
                code: "012",
            },
            VillageCode {
                name: "韩台村委会",
                code: "013",
            },
            VillageCode {
                name: "北照台村委会",
                code: "014",
            },
            VillageCode {
                name: "狼儿峪村委会",
                code: "015",
            },
            VillageCode {
                name: "菩萨鹿村委会",
                code: "016",
            },
            VillageCode {
                name: "发电站村委会",
                code: "017",
            },
            VillageCode {
                name: "漆园村委会",
                code: "018",
            },
            VillageCode {
                name: "瓦窑村委会",
                code: "019",
            },
            VillageCode {
                name: "新开村委会",
                code: "020",
            },
            VillageCode {
                name: "溜石港村委会",
                code: "021",
            },
            VillageCode {
                name: "小水峪村委会",
                code: "022",
            },
            VillageCode {
                name: "王峪村委会",
                code: "023",
            },
            VillageCode {
                name: "老峪沟村委会",
                code: "024",
            },
            VillageCode {
                name: "马刨泉村委会",
                code: "025",
            },
            VillageCode {
                name: "黄土洼村委会",
                code: "026",
            },
            VillageCode {
                name: "禾子涧村委会",
                code: "027",
            },
            VillageCode {
                name: "长峪城村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "十三陵镇",
        code: "021",
        villages: &[
            VillageCode {
                name: "北新村社区居委会",
                code: "001",
            },
            VillageCode {
                name: "十三陵胡庄社区居委会",
                code: "002",
            },
            VillageCode {
                name: "胡庄村委会",
                code: "003",
            },
            VillageCode {
                name: "石牌坊村委会",
                code: "004",
            },
            VillageCode {
                name: "涧头村委会",
                code: "005",
            },
            VillageCode {
                name: "大宫门村委会",
                code: "006",
            },
            VillageCode {
                name: "仙人洞村委会",
                code: "007",
            },
            VillageCode {
                name: "南新村村委会",
                code: "008",
            },
            VillageCode {
                name: "西山口村委会",
                code: "009",
            },
            VillageCode {
                name: "长陵园村委会",
                code: "010",
            },
            VillageCode {
                name: "康陵园村委会",
                code: "011",
            },
            VillageCode {
                name: "小宫门村委会",
                code: "012",
            },
            VillageCode {
                name: "王庄村委会",
                code: "013",
            },
            VillageCode {
                name: "泰陵园村委会",
                code: "014",
            },
            VillageCode {
                name: "悼陵监村委会",
                code: "015",
            },
            VillageCode {
                name: "万娘坟村委会",
                code: "016",
            },
            VillageCode {
                name: "德胜口村委会",
                code: "017",
            },
            VillageCode {
                name: "果庄村委会",
                code: "018",
            },
            VillageCode {
                name: "景陵村委会",
                code: "019",
            },
            VillageCode {
                name: "德陵村委会",
                code: "020",
            },
            VillageCode {
                name: "永陵村委会",
                code: "021",
            },
            VillageCode {
                name: "昭陵村委会",
                code: "022",
            },
            VillageCode {
                name: "献陵村委会",
                code: "023",
            },
            VillageCode {
                name: "长陵村委会",
                code: "024",
            },
            VillageCode {
                name: "东水峪村委会",
                code: "025",
            },
            VillageCode {
                name: "庆陵村委会",
                code: "026",
            },
            VillageCode {
                name: "裕陵村委会",
                code: "027",
            },
            VillageCode {
                name: "老君堂村委会",
                code: "028",
            },
            VillageCode {
                name: "黄泉寺村委会",
                code: "029",
            },
            VillageCode {
                name: "茂陵村委会",
                code: "030",
            },
            VillageCode {
                name: "燕子口村委会",
                code: "031",
            },
            VillageCode {
                name: "康陵村委会",
                code: "032",
            },
            VillageCode {
                name: "泰陵村委会",
                code: "033",
            },
            VillageCode {
                name: "石头园村委会",
                code: "034",
            },
            VillageCode {
                name: "锥石口村委会",
                code: "035",
            },
            VillageCode {
                name: "麻峪房村委会",
                code: "036",
            },
            VillageCode {
                name: "下口村委会",
                code: "037",
            },
            VillageCode {
                name: "上口村委会",
                code: "038",
            },
            VillageCode {
                name: "碓臼峪村委会",
                code: "039",
            },
            VillageCode {
                name: "大岭沟村委会",
                code: "040",
            },
        ],
    },
    TownCode {
        name: "延寿镇",
        code: "022",
        villages: &[
            VillageCode {
                name: "黑山寨村委会",
                code: "001",
            },
            VillageCode {
                name: "沙岭村委会",
                code: "002",
            },
            VillageCode {
                name: "望宝川村委会",
                code: "003",
            },
            VillageCode {
                name: "慈悲峪村委会",
                code: "004",
            },
            VillageCode {
                name: "南庄村委会",
                code: "005",
            },
            VillageCode {
                name: "分水岭村委会",
                code: "006",
            },
            VillageCode {
                name: "辛庄村委会",
                code: "007",
            },
            VillageCode {
                name: "北庄村委会",
                code: "008",
            },
            VillageCode {
                name: "下庄村委会",
                code: "009",
            },
            VillageCode {
                name: "上庄村委会",
                code: "010",
            },
            VillageCode {
                name: "海字村委会",
                code: "011",
            },
            VillageCode {
                name: "西湖村委会",
                code: "012",
            },
            VillageCode {
                name: "湖门村委会",
                code: "013",
            },
            VillageCode {
                name: "连山石村委会",
                code: "014",
            },
            VillageCode {
                name: "花果山村委会",
                code: "015",
            },
            VillageCode {
                name: "木厂村委会",
                code: "016",
            },
            VillageCode {
                name: "百合村委会",
                code: "017",
            },
        ],
    },
];

static TOWNS_BP_012: [TownCode; 26] = [
    TownCode {
        name: "兴丰街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "富强西里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "富强东里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "黄村西里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "黄村中里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "兴华中里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "兴华东里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "富强南里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "康居社区居委会",
                code: "008",
            },
            VillageCode {
                name: "三合南里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "瑞康家园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "黄村东里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "清城北区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "清城南区社区居委会",
                code: "013",
            },
            VillageCode {
                name: "三合北里社区居委会",
                code: "014",
            },
            VillageCode {
                name: "三合中里社区居委会",
                code: "015",
            },
            VillageCode {
                name: "佟馨家园南里社区居委会",
                code: "016",
            },
            VillageCode {
                name: "佟馨家园北里社区居委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "林校路街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "车站南里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "车站中里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "义和庄东里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "义和庄南里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "饮马井社区居委会",
                code: "005",
            },
            VillageCode {
                name: "铁路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "兴水社区居委会",
                code: "007",
            },
            VillageCode {
                name: "永华南里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "兴政西里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "兴政东里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "林校北里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "车站北里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "建兴社区居委会",
                code: "013",
            },
            VillageCode {
                name: "永华北里社区居委会",
                code: "014",
            },
            VillageCode {
                name: "兴华南里社区居委会",
                code: "015",
            },
            VillageCode {
                name: "火神庙社区居委会",
                code: "016",
            },
            VillageCode {
                name: "兴政中里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "罗奇营一区社区居委会",
                code: "018",
            },
            VillageCode {
                name: "罗奇营二区社区居委会",
                code: "019",
            },
            VillageCode {
                name: "义和庄北里社区居委会",
                code: "020",
            },
            VillageCode {
                name: "新源西里社区居委会",
                code: "021",
            },
            VillageCode {
                name: "新源大街26号院社区",
                code: "022",
            },
            VillageCode {
                name: "新源大街27号院社区",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "清源街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "滨河西里南区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "滨河西里北区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "清源西里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "丽园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "兴华园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "枣园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "枣园东里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "康顺园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "滨河东里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "滨河北里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "枣园东里北区社区居委会",
                code: "011",
            },
            VillageCode {
                name: "丽园南区社区居委会",
                code: "012",
            },
            VillageCode {
                name: "学院社区居委会",
                code: "013",
            },
            VillageCode {
                name: "彩虹新城社区居委会",
                code: "014",
            },
            VillageCode {
                name: "兴康家园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "康秀园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "康馨园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "枣园北里社区居委会",
                code: "018",
            },
            VillageCode {
                name: "枣园尚城社区居委会",
                code: "019",
            },
            VillageCode {
                name: "国际港社区居委会",
                code: "020",
            },
            VillageCode {
                name: "康庄路五十号院社区居委会",
                code: "021",
            },
            VillageCode {
                name: "兴盛街187号院社区居委会",
                code: "022",
            },
            VillageCode {
                name: "兴盛街189号院社区居委会",
                code: "023",
            },
            VillageCode {
                name: "康宜园社区居委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "亦庄地区",
        code: "004",
        villages: &[
            VillageCode {
                name: "贵园北里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "贵园东里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "贵园南里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "富源里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "泰河园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "晓康社区居委会",
                code: "006",
            },
            VillageCode {
                name: "广德苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "星岛社区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "三羊里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "泰河园一里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "泰河园七里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "开泰东里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "三羊东里社区居委会",
                code: "013",
            },
            VillageCode {
                name: "鹿华苑二里社区居委会",
                code: "014",
            },
            VillageCode {
                name: "鹿华苑一里社区居委会",
                code: "015",
            },
            VillageCode {
                name: "贵园西里社区居委会",
                code: "016",
            },
            VillageCode {
                name: "鹿海园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "鹿华苑三里社区居委会",
                code: "018",
            },
            VillageCode {
                name: "鹿华苑四里社区居委会",
                code: "019",
            },
            VillageCode {
                name: "亦庄镇东工业区社区",
                code: "020",
            },
            VillageCode {
                name: "亦庄镇南部多功能配套区社区",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "黄村地区",
        code: "005",
        villages: &[
            VillageCode {
                name: "长丰园一区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "明春西园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新凤社区居委会",
                code: "003",
            },
            VillageCode {
                name: "长丰园三区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金色漫香郡社区居委会",
                code: "005",
            },
            VillageCode {
                name: "新兴家园一里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "浣溪谷社区居委会",
                code: "007",
            },
            VillageCode {
                name: "格林雅苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "华远和煦里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "悦景苑社区居委会",
                code: "010",
            },
            VillageCode {
                name: "长丰园二区社区居委会",
                code: "011",
            },
            VillageCode {
                name: "新兴家园二里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "前高米店村村委会",
                code: "013",
            },
            VillageCode {
                name: "西黄村村委会",
                code: "014",
            },
            VillageCode {
                name: "海子角村村委会",
                code: "015",
            },
            VillageCode {
                name: "大庄村村委会",
                code: "016",
            },
            VillageCode {
                name: "前大营村村委会",
                code: "017",
            },
            VillageCode {
                name: "狼各庄西村村委会",
                code: "018",
            },
            VillageCode {
                name: "狼各庄东村村委会",
                code: "019",
            },
            VillageCode {
                name: "西庄村村委会",
                code: "020",
            },
            VillageCode {
                name: "高家铺村村委会",
                code: "021",
            },
            VillageCode {
                name: "狼垡一村村委会",
                code: "022",
            },
            VillageCode {
                name: "狼垡二村村委会",
                code: "023",
            },
            VillageCode {
                name: "狼垡三村村委会",
                code: "024",
            },
            VillageCode {
                name: "狼垡四村村委会",
                code: "025",
            },
            VillageCode {
                name: "立垡村村委会",
                code: "026",
            },
            VillageCode {
                name: "西芦城村村委会",
                code: "027",
            },
            VillageCode {
                name: "东芦城村村委会",
                code: "028",
            },
            VillageCode {
                name: "鹅房村村委会",
                code: "029",
            },
            VillageCode {
                name: "宋庄村村委会",
                code: "030",
            },
            VillageCode {
                name: "后辛庄村村委会",
                code: "031",
            },
            VillageCode {
                name: "前辛庄村村委会",
                code: "032",
            },
            VillageCode {
                name: "太福庄村村委会",
                code: "033",
            },
            VillageCode {
                name: "周村村委会",
                code: "034",
            },
            VillageCode {
                name: "刘一村村委会",
                code: "035",
            },
            VillageCode {
                name: "刘二村村委会",
                code: "036",
            },
            VillageCode {
                name: "三间房村村委会",
                code: "037",
            },
            VillageCode {
                name: "辛店村村委会",
                code: "038",
            },
            VillageCode {
                name: "霍村村委会",
                code: "039",
            },
            VillageCode {
                name: "邢各庄村村委会",
                code: "040",
            },
            VillageCode {
                name: "王立庄村村委会",
                code: "041",
            },
            VillageCode {
                name: "桂村村委会",
                code: "042",
            },
            VillageCode {
                name: "李村村委会",
                code: "043",
            },
            VillageCode {
                name: "孙村村委会",
                code: "044",
            },
            VillageCode {
                name: "郭上坡村村委会",
                code: "045",
            },
            VillageCode {
                name: "孙村工业区社区",
                code: "046",
            },
            VillageCode {
                name: "芦城工业区社区",
                code: "047",
            },
        ],
    },
    TownCode {
        name: "旧宫地区",
        code: "006",
        villages: &[
            VillageCode {
                name: "清逸园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "清欣园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "清和园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "清乐园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "红星楼社区居委会",
                code: "005",
            },
            VillageCode {
                name: "五福堂社区居委会",
                code: "006",
            },
            VillageCode {
                name: "红星北里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "宣颐家园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "上林苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "德茂楼社区居委会",
                code: "010",
            },
            VillageCode {
                name: "绿洲家园社区居委会",
                code: "011",
            },
            VillageCode {
                name: "美然社区居委会",
                code: "012",
            },
            VillageCode {
                name: "灵秀山庄社区居委会",
                code: "013",
            },
            VillageCode {
                name: "清逸西园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "佳和园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "育龙家园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "幻星家园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "德林园社区居委会",
                code: "018",
            },
            VillageCode {
                name: "德茂佳苑社区居委会",
                code: "019",
            },
            VillageCode {
                name: "润星家园社区居委会",
                code: "020",
            },
            VillageCode {
                name: "云龙家园社区居委会",
                code: "021",
            },
            VillageCode {
                name: "成和园社区居委会",
                code: "022",
            },
            VillageCode {
                name: "美丽新世界社区居委会",
                code: "023",
            },
            VillageCode {
                name: "上筑家园社区居委会",
                code: "024",
            },
            VillageCode {
                name: "润枫锦尚社区居委会",
                code: "025",
            },
            VillageCode {
                name: "盛悦居社区居委会",
                code: "026",
            },
            VillageCode {
                name: "紫郡府社区居委会",
                code: "027",
            },
            VillageCode {
                name: "国韵村社区居委会",
                code: "028",
            },
            VillageCode {
                name: "文锦苑社区居委会",
                code: "029",
            },
            VillageCode {
                name: "有德家苑社区居委会",
                code: "030",
            },
            VillageCode {
                name: "旧宫新苑南区社区居委会",
                code: "031",
            },
            VillageCode {
                name: "旧宫新苑北区社区居委会",
                code: "032",
            },
            VillageCode {
                name: "庑殿家苑南区社区居委会",
                code: "033",
            },
            VillageCode {
                name: "庑殿家苑北区社区居委会",
                code: "034",
            },
            VillageCode {
                name: "旧宫工业园社区",
                code: "035",
            },
            VillageCode {
                name: "南郊旧宫场区社区",
                code: "036",
            },
        ],
    },
    TownCode {
        name: "西红门地区",
        code: "007",
        villages: &[
            VillageCode {
                name: "星光社区居委会",
                code: "001",
            },
            VillageCode {
                name: "九龙社区居委会",
                code: "002",
            },
            VillageCode {
                name: "宏福社区居委会",
                code: "003",
            },
            VillageCode {
                name: "福星花园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "绿林苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "金华园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "瑞海北区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "瑞海南区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "宏大园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "星苑社区居委会",
                code: "010",
            },
            VillageCode {
                name: "兴日苑社区居委会",
                code: "011",
            },
            VillageCode {
                name: "博苑社区居委会",
                code: "012",
            },
            VillageCode {
                name: "礼域北区社区居委会",
                code: "013",
            },
            VillageCode {
                name: "礼域南区社区居委会",
                code: "014",
            },
            VillageCode {
                name: "都市公寓社区居委会",
                code: "015",
            },
            VillageCode {
                name: "欣荣社区居委会",
                code: "016",
            },
            VillageCode {
                name: "欣清社区居委会",
                code: "017",
            },
            VillageCode {
                name: "宏业东区社区居委会",
                code: "018",
            },
            VillageCode {
                name: "宏业西区社区居委会",
                code: "019",
            },
            VillageCode {
                name: "同兴社区居委会",
                code: "020",
            },
            VillageCode {
                name: "金荣园社区居委会",
                code: "021",
            },
            VillageCode {
                name: "瑞海家园社区居委会",
                code: "022",
            },
            VillageCode {
                name: "曦月社区居委会",
                code: "023",
            },
            VillageCode {
                name: "星语社区居委会",
                code: "024",
            },
            VillageCode {
                name: "月苑社区居委会",
                code: "025",
            },
            VillageCode {
                name: "宏盛社区居委会",
                code: "026",
            },
            VillageCode {
                name: "兴都社区居委会",
                code: "027",
            },
            VillageCode {
                name: "同庆社区居委会",
                code: "028",
            },
            VillageCode {
                name: "西红门一村村委会",
                code: "029",
            },
            VillageCode {
                name: "西红门二村村委会",
                code: "030",
            },
            VillageCode {
                name: "西红门三村村委会",
                code: "031",
            },
            VillageCode {
                name: "西红门四村村委会",
                code: "032",
            },
            VillageCode {
                name: "西红门六村村委会",
                code: "033",
            },
            VillageCode {
                name: "西红门七村村委会",
                code: "034",
            },
            VillageCode {
                name: "西红门八村村委会",
                code: "035",
            },
            VillageCode {
                name: "西红门十村村委会",
                code: "036",
            },
            VillageCode {
                name: "西红门十一村村委会",
                code: "037",
            },
            VillageCode {
                name: "西红门十二村村委会",
                code: "038",
            },
            VillageCode {
                name: "新三余庄村村委会",
                code: "039",
            },
            VillageCode {
                name: "老三余庄村村委会",
                code: "040",
            },
            VillageCode {
                name: "寿保庄村村委会",
                code: "041",
            },
            VillageCode {
                name: "大白楼村村委会",
                code: "042",
            },
            VillageCode {
                name: "大生庄村村委会",
                code: "043",
            },
            VillageCode {
                name: "金星庄村村委会",
                code: "044",
            },
            VillageCode {
                name: "志远庄村村委会",
                code: "045",
            },
            VillageCode {
                name: "建新庄村村委会",
                code: "046",
            },
            VillageCode {
                name: "团河北村村委会",
                code: "047",
            },
            VillageCode {
                name: "团河南村村委会",
                code: "048",
            },
            VillageCode {
                name: "振亚庄村村委会",
                code: "049",
            },
            VillageCode {
                name: "小白楼村村委会",
                code: "050",
            },
            VillageCode {
                name: "西红门镇新建工业区社区",
                code: "051",
            },
        ],
    },
    TownCode {
        name: "瀛海地区",
        code: "008",
        villages: &[
            VillageCode {
                name: "兴海园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南海家园一里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "南海家园二里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南海家园三里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "南海家园四里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "南海家园五里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "南海家园六里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "南海家园七里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "鹿海园五里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "瀛海家园一里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "瀛海家园二里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "金茂悦社区居委会",
                code: "012",
            },
            VillageCode {
                name: "三槐家园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "永旭嘉园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "海梓嘉园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "金域东郡社区居委会",
                code: "016",
            },
            VillageCode {
                name: "兴悦家园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "瀛海朗苑社区居委会",
                code: "018",
            },
            VillageCode {
                name: "金茂嘉园社区居委会",
                code: "019",
            },
            VillageCode {
                name: "兴瀛嘉苑社区居委会",
                code: "020",
            },
            VillageCode {
                name: "和悦华锦社区居委会",
                code: "021",
            },
            VillageCode {
                name: "亦城亦景社区居委会",
                code: "022",
            },
            VillageCode {
                name: "金禧嘉园社区居委会",
                code: "023",
            },
            VillageCode {
                name: "瀛海镇工业区社区",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "观音寺街道",
        code: "009",
        villages: &[
            VillageCode {
                name: "双河北里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新安里东区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "团河社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南湖园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "观音寺南里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "观音寺社区居委会",
                code: "006",
            },
            VillageCode {
                name: "双河南里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "新居里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "盛春坊社区居委会",
                code: "009",
            },
            VillageCode {
                name: "金华里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "福苑社区居委会",
                code: "011",
            },
            VillageCode {
                name: "泰中花园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "观音寺北里社区居委会",
                code: "013",
            },
            VillageCode {
                name: "首座御园一里社区居委会",
                code: "014",
            },
            VillageCode {
                name: "盛嘉华苑社区居委会",
                code: "015",
            },
            VillageCode {
                name: "首座御园二里社区居委会",
                code: "016",
            },
            VillageCode {
                name: "首座御园三里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "首座御园四里社区居委会",
                code: "018",
            },
            VillageCode {
                name: "双河北里尚城社区居委会",
                code: "019",
            },
            VillageCode {
                name: "美澜湾西区社区居委会",
                code: "020",
            },
            VillageCode {
                name: "美澜湾东区社区居委会",
                code: "021",
            },
            VillageCode {
                name: "观燕社区居委会",
                code: "022",
            },
            VillageCode {
                name: "新安里西区社区居委会",
                code: "023",
            },
            VillageCode {
                name: "福海佳园社区居委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "天宫院街道",
        code: "010",
        villages: &[
            VillageCode {
                name: "海子角社区居委会",
                code: "001",
            },
            VillageCode {
                name: "天堂河社区居委会",
                code: "002",
            },
            VillageCode {
                name: "海子角东里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "海子角南里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "海子角西里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "海子角北里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "矿林庄社区居委会",
                code: "007",
            },
            VillageCode {
                name: "天宫院社区居委会",
                code: "008",
            },
            VillageCode {
                name: "兴宇社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "融汇社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "新源时代社区居委会",
                code: "011",
            },
            VillageCode {
                name: "天宫院中里社区居委会",
                code: "012",
            },
            VillageCode {
                name: "天宫院西里社区居委会",
                code: "013",
            },
            VillageCode {
                name: "新源时代西里社区居委会",
                code: "014",
            },
            VillageCode {
                name: "新源时代中里社区居委会",
                code: "015",
            },
            VillageCode {
                name: "天宫院南里社区居委会",
                code: "016",
            },
            VillageCode {
                name: "天宫院北里社区居委会",
                code: "017",
            },
            VillageCode {
                name: "兴宇西里社区居委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "高米店街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "康盛园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "康和园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "康隆园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "兴涛社区居委会",
                code: "004",
            },
            VillageCode {
                name: "兴盛园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "香海园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "香留园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "金惠园二区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "金惠园三区社区居委会",
                code: "009",
            },
            VillageCode {
                name: "郁花园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "郁花园二里社区居委会",
                code: "011",
            },
            VillageCode {
                name: "茉莉社区居委会",
                code: "012",
            },
            VillageCode {
                name: "绿地社区居委会",
                code: "013",
            },
            VillageCode {
                name: "香旺园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "康邑园社区居委会",
                code: "015",
            },
            VillageCode {
                name: "康泰园社区居委会",
                code: "016",
            },
            VillageCode {
                name: "双高花园社区居委会",
                code: "017",
            },
            VillageCode {
                name: "郁花园三里社区居委会",
                code: "018",
            },
            VillageCode {
                name: "香乐园社区居委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "荣华街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "天华园一里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "天华园二里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "天华园三里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "天宝园金地格林小镇社区居委会",
                code: "004",
            },
            VillageCode {
                name: "天宝园卡尔百丽社区居委会",
                code: "005",
            },
            VillageCode {
                name: "天宝园上海沙龙社区居委会",
                code: "006",
            },
            VillageCode {
                name: "天宝园大雄郁金香舍社区居委会",
                code: "007",
            },
            VillageCode {
                name: "林肯公园社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "博兴街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "中芯花园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "亦城茗苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "博客雅苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "赢海庄园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "观海苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "通泰文园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "科创家园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "亦城景园社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "青云店镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "青云里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "老观里村村委会",
                code: "002",
            },
            VillageCode {
                name: "顾庄村村委会",
                code: "003",
            },
            VillageCode {
                name: "东辛屯村村委会",
                code: "004",
            },
            VillageCode {
                name: "大迴城村村委会",
                code: "005",
            },
            VillageCode {
                name: "东迴城村村委会",
                code: "006",
            },
            VillageCode {
                name: "石洲营村村委会",
                code: "007",
            },
            VillageCode {
                name: "孝义营村村委会",
                code: "008",
            },
            VillageCode {
                name: "沙堆营村村委会",
                code: "009",
            },
            VillageCode {
                name: "霍洲营村村委会",
                code: "010",
            },
            VillageCode {
                name: "垡上营村村委会",
                code: "011",
            },
            VillageCode {
                name: "解州营村村委会",
                code: "012",
            },
            VillageCode {
                name: "尚庄村村委会",
                code: "013",
            },
            VillageCode {
                name: "沙子营村村委会",
                code: "014",
            },
            VillageCode {
                name: "青云店一村村委会",
                code: "015",
            },
            VillageCode {
                name: "青云店二村一村村委会",
                code: "016",
            },
            VillageCode {
                name: "青云店二村三村村委会",
                code: "017",
            },
            VillageCode {
                name: "青云店三村一村村委会",
                code: "018",
            },
            VillageCode {
                name: "青云店三村二村村委会",
                code: "019",
            },
            VillageCode {
                name: "青云店三村三村村委会",
                code: "020",
            },
            VillageCode {
                name: "青云店四村村委会",
                code: "021",
            },
            VillageCode {
                name: "青云店五村村委会",
                code: "022",
            },
            VillageCode {
                name: "青云店六村村委会",
                code: "023",
            },
            VillageCode {
                name: "东店村村委会",
                code: "024",
            },
            VillageCode {
                name: "西杭子村村委会",
                code: "025",
            },
            VillageCode {
                name: "小谷店村村委会",
                code: "026",
            },
            VillageCode {
                name: "东孙村村委会",
                code: "027",
            },
            VillageCode {
                name: "太平庄村村委会",
                code: "028",
            },
            VillageCode {
                name: "大谷店村村委会",
                code: "029",
            },
            VillageCode {
                name: "西鲍辛庄村村委会",
                code: "030",
            },
            VillageCode {
                name: "东鲍辛庄村村委会",
                code: "031",
            },
            VillageCode {
                name: "马凤岗村村委会",
                code: "032",
            },
            VillageCode {
                name: "东趙村村委会",
                code: "033",
            },
            VillageCode {
                name: "北野场村村委会",
                code: "034",
            },
            VillageCode {
                name: "南大红门村村委会",
                code: "035",
            },
            VillageCode {
                name: "北辛屯村村委会",
                code: "036",
            },
            VillageCode {
                name: "北店村村委会",
                code: "037",
            },
            VillageCode {
                name: "小铺头村村委会",
                code: "038",
            },
            VillageCode {
                name: "曹村村委会",
                code: "039",
            },
            VillageCode {
                name: "寺上村村委会",
                code: "040",
            },
            VillageCode {
                name: "枣林村村委会",
                code: "041",
            },
            VillageCode {
                name: "大张本庄村村委会",
                code: "042",
            },
            VillageCode {
                name: "小张本庄村村委会",
                code: "043",
            },
            VillageCode {
                name: "泥营村村委会",
                code: "044",
            },
            VillageCode {
                name: "垡上村村委会",
                code: "045",
            },
            VillageCode {
                name: "西大屯村村委会",
                code: "046",
            },
            VillageCode {
                name: "中大屯村村委会",
                code: "047",
            },
            VillageCode {
                name: "东大屯村村委会",
                code: "048",
            },
            VillageCode {
                name: "杨各庄村村委会",
                code: "049",
            },
            VillageCode {
                name: "高庄村村委会",
                code: "050",
            },
            VillageCode {
                name: "青云店镇工业开发区社区",
                code: "051",
            },
        ],
    },
    TownCode {
        name: "采育镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "育星苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "恒盛社区居委会",
                code: "002",
            },
            VillageCode {
                name: "满庭春社区居委会",
                code: "003",
            },
            VillageCode {
                name: "荣墅社区居委会",
                code: "004",
            },
            VillageCode {
                name: "如遇苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "宽育合院社区居委会",
                code: "006",
            },
            VillageCode {
                name: "育新中里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "育新西里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "育新南里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "育新北里社区居委会",
                code: "010",
            },
            VillageCode {
                name: "大黑垡村村委会",
                code: "011",
            },
            VillageCode {
                name: "宁家湾村村委会",
                code: "012",
            },
            VillageCode {
                name: "北辛店村村委会",
                code: "013",
            },
            VillageCode {
                name: "南辛店一村村委会",
                code: "014",
            },
            VillageCode {
                name: "南辛店二村村委会",
                code: "015",
            },
            VillageCode {
                name: "北山东村村委会",
                code: "016",
            },
            VillageCode {
                name: "北营村村委会",
                code: "017",
            },
            VillageCode {
                name: "西营一村村委会",
                code: "018",
            },
            VillageCode {
                name: "西营二村村委会",
                code: "019",
            },
            VillageCode {
                name: "西营三村村委会",
                code: "020",
            },
            VillageCode {
                name: "西营四村村委会",
                code: "021",
            },
            VillageCode {
                name: "东营一村村委会",
                code: "022",
            },
            VillageCode {
                name: "东营二村村委会",
                code: "023",
            },
            VillageCode {
                name: "南营二村村委会",
                code: "024",
            },
            VillageCode {
                name: "南三一村村委会",
                code: "025",
            },
            VillageCode {
                name: "南三二村村委会",
                code: "026",
            },
            VillageCode {
                name: "南三三村村委会",
                code: "027",
            },
            VillageCode {
                name: "南山东营一村村委会",
                code: "028",
            },
            VillageCode {
                name: "南山东营二村村委会",
                code: "029",
            },
            VillageCode {
                name: "施家务村村委会",
                code: "030",
            },
            VillageCode {
                name: "西辛庄村村委会",
                code: "031",
            },
            VillageCode {
                name: "东庄村村委会",
                code: "032",
            },
            VillageCode {
                name: "后甫村村委会",
                code: "033",
            },
            VillageCode {
                name: "前甫村村委会",
                code: "034",
            },
            VillageCode {
                name: "屯留营村村委会",
                code: "035",
            },
            VillageCode {
                name: "岳街村村委会",
                code: "036",
            },
            VillageCode {
                name: "邵各庄村村委会",
                code: "037",
            },
            VillageCode {
                name: "下黎城村村委会",
                code: "038",
            },
            VillageCode {
                name: "沙窝营村村委会",
                code: "039",
            },
            VillageCode {
                name: "潘铁营村村委会",
                code: "040",
            },
            VillageCode {
                name: "辛庄营村村委会",
                code: "041",
            },
            VillageCode {
                name: "韩营村村委会",
                code: "042",
            },
            VillageCode {
                name: "铜佛寺村村委会",
                code: "043",
            },
            VillageCode {
                name: "广佛寺村村委会",
                code: "044",
            },
            VillageCode {
                name: "包头营村村委会",
                code: "045",
            },
            VillageCode {
                name: "大皮营一村村委会",
                code: "046",
            },
            VillageCode {
                name: "大皮营二村村委会",
                code: "047",
            },
            VillageCode {
                name: "大皮营三村村委会",
                code: "048",
            },
            VillageCode {
                name: "小皮营村村委会",
                code: "049",
            },
            VillageCode {
                name: "杨堤村村委会",
                code: "050",
            },
            VillageCode {
                name: "利市营村村委会",
                code: "051",
            },
            VillageCode {
                name: "东潞州村村委会",
                code: "052",
            },
            VillageCode {
                name: "大同营村村委会",
                code: "053",
            },
            VillageCode {
                name: "山西营村村委会",
                code: "054",
            },
            VillageCode {
                name: "大里庄村村委会",
                code: "055",
            },
            VillageCode {
                name: "东半壁店村村委会",
                code: "056",
            },
            VillageCode {
                name: "倪家村村委会",
                code: "057",
            },
            VillageCode {
                name: "龙门庄村村委会",
                code: "058",
            },
            VillageCode {
                name: "张各庄村村委会",
                code: "059",
            },
            VillageCode {
                name: "哱啰庄村村委会",
                code: "060",
            },
            VillageCode {
                name: "康营村村委会",
                code: "061",
            },
            VillageCode {
                name: "延寿营村村委会",
                code: "062",
            },
            VillageCode {
                name: "庙洼营村村委会",
                code: "063",
            },
            VillageCode {
                name: "凤河营村村委会",
                code: "064",
            },
            VillageCode {
                name: "沙窝店村村委会",
                code: "065",
            },
            VillageCode {
                name: "北京采育经济开发区社区",
                code: "066",
            },
        ],
    },
    TownCode {
        name: "安定镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "堡林庄村村委会",
                code: "001",
            },
            VillageCode {
                name: "后安定村村委会",
                code: "002",
            },
            VillageCode {
                name: "前安定村村委会",
                code: "003",
            },
            VillageCode {
                name: "沙河村村委会",
                code: "004",
            },
            VillageCode {
                name: "站上村村委会",
                code: "005",
            },
            VillageCode {
                name: "高店村村委会",
                code: "006",
            },
            VillageCode {
                name: "后野厂村村委会",
                code: "007",
            },
            VillageCode {
                name: "前野厂村村委会",
                code: "008",
            },
            VillageCode {
                name: "杜庄屯村村委会",
                code: "009",
            },
            VillageCode {
                name: "洪士庄村村委会",
                code: "010",
            },
            VillageCode {
                name: "潘家马房村村委会",
                code: "011",
            },
            VillageCode {
                name: "郑福庄村村委会",
                code: "012",
            },
            VillageCode {
                name: "驴房村村委会",
                code: "013",
            },
            VillageCode {
                name: "兴安营村村委会",
                code: "014",
            },
            VillageCode {
                name: "善台子村村委会",
                code: "015",
            },
            VillageCode {
                name: "西芦各庄村村委会",
                code: "016",
            },
            VillageCode {
                name: "东芦各庄村村委会",
                code: "017",
            },
            VillageCode {
                name: "车站村村委会",
                code: "018",
            },
            VillageCode {
                name: "汤营村村委会",
                code: "019",
            },
            VillageCode {
                name: "伙达营村村委会",
                code: "020",
            },
            VillageCode {
                name: "通洲马坊村村委会",
                code: "021",
            },
            VillageCode {
                name: "于家务村村委会",
                code: "022",
            },
            VillageCode {
                name: "后辛房村村委会",
                code: "023",
            },
            VillageCode {
                name: "前辛房村村委会",
                code: "024",
            },
            VillageCode {
                name: "西白塔村村委会",
                code: "025",
            },
            VillageCode {
                name: "东白塔村村委会",
                code: "026",
            },
            VillageCode {
                name: "周园子村村委会",
                code: "027",
            },
            VillageCode {
                name: "徐柏村村委会",
                code: "028",
            },
            VillageCode {
                name: "皋营村村委会",
                code: "029",
            },
            VillageCode {
                name: "马各庄村村委会",
                code: "030",
            },
            VillageCode {
                name: "佟家务村村委会",
                code: "031",
            },
            VillageCode {
                name: "大渠村村委会",
                code: "032",
            },
            VillageCode {
                name: "佟营村村委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "礼贤镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "庆贤南里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "庆贤北里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "敬贤家园北里一区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "敬贤家园北里二区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "敬贤家园中里一区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "敬贤家园中里二区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "敬贤家园南里一区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "敬贤家园南里二区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "龙头村村委会",
                code: "009",
            },
            VillageCode {
                name: "王庄村村委会",
                code: "010",
            },
            VillageCode {
                name: "西段家务村村委会",
                code: "011",
            },
            VillageCode {
                name: "东段家务村村委会",
                code: "012",
            },
            VillageCode {
                name: "平地村村委会",
                code: "013",
            },
            VillageCode {
                name: "河北头村村委会",
                code: "014",
            },
            VillageCode {
                name: "小刘各庄村村委会",
                code: "015",
            },
            VillageCode {
                name: "伍各庄村村委会",
                code: "016",
            },
            VillageCode {
                name: "内官庄村村委会",
                code: "017",
            },
            VillageCode {
                name: "佃子村村委会",
                code: "018",
            },
            VillageCode {
                name: "孙家营村村委会",
                code: "019",
            },
            VillageCode {
                name: "赵家园村村委会",
                code: "020",
            },
            VillageCode {
                name: "紫各庄村村委会",
                code: "021",
            },
            VillageCode {
                name: "小马坊村村委会",
                code: "022",
            },
            VillageCode {
                name: "礼贤一村村委会",
                code: "023",
            },
            VillageCode {
                name: "礼贤二村村委会",
                code: "024",
            },
            VillageCode {
                name: "礼贤三村村委会",
                code: "025",
            },
            VillageCode {
                name: "田家营村村委会",
                code: "026",
            },
            VillageCode {
                name: "西里河村村委会",
                code: "027",
            },
            VillageCode {
                name: "祁各庄村村委会",
                code: "028",
            },
            VillageCode {
                name: "李各庄村村委会",
                code: "029",
            },
            VillageCode {
                name: "荆家务村村委会",
                code: "030",
            },
            VillageCode {
                name: "柏树庄村村委会",
                code: "031",
            },
            VillageCode {
                name: "王化庄村村委会",
                code: "032",
            },
            VillageCode {
                name: "东黄垡村村委会",
                code: "033",
            },
            VillageCode {
                name: "贺北村村委会",
                code: "034",
            },
            VillageCode {
                name: "西白疃村村委会",
                code: "035",
            },
            VillageCode {
                name: "东白疃村村委会",
                code: "036",
            },
            VillageCode {
                name: "苑南村村委会",
                code: "037",
            },
            VillageCode {
                name: "后杨各庄村村委会",
                code: "038",
            },
            VillageCode {
                name: "前杨各庄村村委会",
                code: "039",
            },
            VillageCode {
                name: "黎明村村委会",
                code: "040",
            },
            VillageCode {
                name: "宏升村村委会",
                code: "041",
            },
            VillageCode {
                name: "中心村村委会",
                code: "042",
            },
            VillageCode {
                name: "昕升村村委会",
                code: "043",
            },
            VillageCode {
                name: "东安村村委会",
                code: "044",
            },
            VillageCode {
                name: "西郏河村村委会",
                code: "045",
            },
            VillageCode {
                name: "东郏河村村委会",
                code: "046",
            },
            VillageCode {
                name: "石柱子村村委会",
                code: "047",
            },
            VillageCode {
                name: "董各庄村村委会",
                code: "048",
            },
            VillageCode {
                name: "西梁各庄村村委会",
                code: "049",
            },
            VillageCode {
                name: "东梁各庄村村委会",
                code: "050",
            },
        ],
    },
    TownCode {
        name: "榆垡镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "榆垡新城嘉园北里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "榆垡新城嘉园南里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "榆垡椿荷墅社区居委会",
                code: "003",
            },
            VillageCode {
                name: "榆垡空港新苑一里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "榆垡空港新苑二里社区居委会",
                code: "005",
            },
            VillageCode {
                name: "榆垡空港新苑三里社区居委会",
                code: "006",
            },
            VillageCode {
                name: "榆垡空港新苑四里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "榆垡空港新苑五里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "榆垡空港新苑六里社区居委会",
                code: "009",
            },
            VillageCode {
                name: "空港富苑社区居委会",
                code: "010",
            },
            VillageCode {
                name: "空港贵苑社区居委会",
                code: "011",
            },
            VillageCode {
                name: "空港荣苑社区居委会",
                code: "012",
            },
            VillageCode {
                name: "空港华苑社区居委会",
                code: "013",
            },
            VillageCode {
                name: "空港茗苑社区居委会",
                code: "014",
            },
            VillageCode {
                name: "空港雅苑社区居委会",
                code: "015",
            },
            VillageCode {
                name: "空港合苑社区居委会",
                code: "016",
            },
            VillageCode {
                name: "石垡村村委会",
                code: "017",
            },
            VillageCode {
                name: "西黄垡村村委会",
                code: "018",
            },
            VillageCode {
                name: "留士庄村村委会",
                code: "019",
            },
            VillageCode {
                name: "小黄垡村村委会",
                code: "020",
            },
            VillageCode {
                name: "履磕村村委会",
                code: "021",
            },
            VillageCode {
                name: "刘家铺村村委会",
                code: "022",
            },
            VillageCode {
                name: "闫家场村村委会",
                code: "023",
            },
            VillageCode {
                name: "西麻各庄村村委会",
                code: "024",
            },
            VillageCode {
                name: "东麻各庄村村委会",
                code: "025",
            },
            VillageCode {
                name: "邓家屯村村委会",
                code: "026",
            },
            VillageCode {
                name: "魏各庄村村委会",
                code: "027",
            },
            VillageCode {
                name: "西瓮各庄村村委会",
                code: "028",
            },
            VillageCode {
                name: "东瓮各庄村村委会",
                code: "029",
            },
            VillageCode {
                name: "新桥村村委会",
                code: "030",
            },
            VillageCode {
                name: "孙各庄村村委会",
                code: "031",
            },
            VillageCode {
                name: "景家场村村委会",
                code: "032",
            },
            VillageCode {
                name: "辛庄村村委会",
                code: "033",
            },
            VillageCode {
                name: "大练庄村村委会",
                code: "034",
            },
            VillageCode {
                name: "黄各庄村村委会",
                code: "035",
            },
            VillageCode {
                name: "榆垡村村委会",
                code: "036",
            },
            VillageCode {
                name: "求贤村村委会",
                code: "037",
            },
            VillageCode {
                name: "太子务村村委会",
                code: "038",
            },
            VillageCode {
                name: "东庄营村村委会",
                code: "039",
            },
            VillageCode {
                name: "訚家铺村村委会",
                code: "040",
            },
            VillageCode {
                name: "西胡林村村委会",
                code: "041",
            },
            VillageCode {
                name: "东胡林村村委会",
                code: "042",
            },
            VillageCode {
                name: "张家务村村委会",
                code: "043",
            },
            VillageCode {
                name: "十里铺村村委会",
                code: "044",
            },
            VillageCode {
                name: "西张华村村委会",
                code: "045",
            },
            VillageCode {
                name: "东张华村村委会",
                code: "046",
            },
            VillageCode {
                name: "南张华村村委会",
                code: "047",
            },
            VillageCode {
                name: "康张华村村委会",
                code: "048",
            },
            VillageCode {
                name: "小店村村委会",
                code: "049",
            },
            VillageCode {
                name: "刘各庄村村委会",
                code: "050",
            },
            VillageCode {
                name: "小押堤村村委会",
                code: "051",
            },
            VillageCode {
                name: "辛安庄村村委会",
                code: "052",
            },
            VillageCode {
                name: "王家屯村村委会",
                code: "053",
            },
            VillageCode {
                name: "曹辛庄村村委会",
                code: "054",
            },
            VillageCode {
                name: "香营村村委会",
                code: "055",
            },
            VillageCode {
                name: "曹各庄村村委会",
                code: "056",
            },
            VillageCode {
                name: "辛村村委会",
                code: "057",
            },
            VillageCode {
                name: "马家屯村村委会",
                code: "058",
            },
            VillageCode {
                name: "西押堤村村委会",
                code: "059",
            },
            VillageCode {
                name: "东押堤村村委会",
                code: "060",
            },
            VillageCode {
                name: "石佛寺村村委会",
                code: "061",
            },
            VillageCode {
                name: "贾屯村村委会",
                code: "062",
            },
            VillageCode {
                name: "崔指挥营村村委会",
                code: "063",
            },
            VillageCode {
                name: "榆垡镇工业区社区",
                code: "064",
            },
        ],
    },
    TownCode {
        name: "庞各庄镇",
        code: "019",
        villages: &[
            VillageCode {
                name: "御佳园南里社区居委会",
                code: "001",
            },
            VillageCode {
                name: "御佳园北里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "富力丹麦小镇社区居委会",
                code: "003",
            },
            VillageCode {
                name: "富力华庭苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "龙景湾社区居委会",
                code: "005",
            },
            VillageCode {
                name: "丽水佳园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "隆兴园东里社区居委会",
                code: "007",
            },
            VillageCode {
                name: "隆兴园西里社区居委会",
                code: "008",
            },
            VillageCode {
                name: "云锦佳园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "隆盛园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "李窑村村委会",
                code: "011",
            },
            VillageCode {
                name: "西中堡村村委会",
                code: "012",
            },
            VillageCode {
                name: "东中堡村村委会",
                code: "013",
            },
            VillageCode {
                name: "四各庄村村委会",
                code: "014",
            },
            VillageCode {
                name: "小庄子村村委会",
                code: "015",
            },
            VillageCode {
                name: "幸福村村委会",
                code: "016",
            },
            VillageCode {
                name: "团结村村委会",
                code: "017",
            },
            VillageCode {
                name: "繁荣村村委会",
                code: "018",
            },
            VillageCode {
                name: "民生村村委会",
                code: "019",
            },
            VillageCode {
                name: "北李渠村村委会",
                code: "020",
            },
            VillageCode {
                name: "河南村村委会",
                code: "021",
            },
            VillageCode {
                name: "宋各庄村村委会",
                code: "022",
            },
            VillageCode {
                name: "南李渠村村委会",
                code: "023",
            },
            VillageCode {
                name: "孙场村村委会",
                code: "024",
            },
            VillageCode {
                name: "薛营村村委会",
                code: "025",
            },
            VillageCode {
                name: "西义堂村村委会",
                code: "026",
            },
            VillageCode {
                name: "东义堂村村委会",
                code: "027",
            },
            VillageCode {
                name: "南义堂村村委会",
                code: "028",
            },
            VillageCode {
                name: "南园子村村委会",
                code: "029",
            },
            VillageCode {
                name: "北顿垡村村委会",
                code: "030",
            },
            VillageCode {
                name: "张新庄村村委会",
                code: "031",
            },
            VillageCode {
                name: "南小营村村委会",
                code: "032",
            },
            VillageCode {
                name: "西梨园村村委会",
                code: "033",
            },
            VillageCode {
                name: "东梨园村村委会",
                code: "034",
            },
            VillageCode {
                name: "李家巷村村委会",
                code: "035",
            },
            VillageCode {
                name: "王场村村委会",
                code: "036",
            },
            VillageCode {
                name: "加录垡村村委会",
                code: "037",
            },
            VillageCode {
                name: "南顿垡村村委会",
                code: "038",
            },
            VillageCode {
                name: "钥匙头村村委会",
                code: "039",
            },
            VillageCode {
                name: "鲍家铺村村委会",
                code: "040",
            },
            VillageCode {
                name: "北章客村村委会",
                code: "041",
            },
            VillageCode {
                name: "留民庄村村委会",
                code: "042",
            },
            VillageCode {
                name: "西高各庄村村委会",
                code: "043",
            },
            VillageCode {
                name: "东高各庄村村委会",
                code: "044",
            },
            VillageCode {
                name: "保安庄村村委会",
                code: "045",
            },
            VillageCode {
                name: "梁家务村村委会",
                code: "046",
            },
            VillageCode {
                name: "田家窑村村委会",
                code: "047",
            },
            VillageCode {
                name: "丁村村委会",
                code: "048",
            },
            VillageCode {
                name: "定福庄村村委会",
                code: "049",
            },
            VillageCode {
                name: "西南次村村委会",
                code: "050",
            },
            VillageCode {
                name: "东南次村村委会",
                code: "051",
            },
            VillageCode {
                name: "张公垡村村委会",
                code: "052",
            },
            VillageCode {
                name: "赵村村委会",
                code: "053",
            },
            VillageCode {
                name: "梨花村村委会",
                code: "054",
            },
            VillageCode {
                name: "福上村村委会",
                code: "055",
            },
            VillageCode {
                name: "东黑垡村村委会",
                code: "056",
            },
            VillageCode {
                name: "西黑垡村村委会",
                code: "057",
            },
            VillageCode {
                name: "常各庄村村委会",
                code: "058",
            },
            VillageCode {
                name: "北曹各庄村村委会",
                code: "059",
            },
            VillageCode {
                name: "前曹各庄村村委会",
                code: "060",
            },
            VillageCode {
                name: "韩家铺村村委会",
                code: "061",
            },
            VillageCode {
                name: "南章客村村委会",
                code: "062",
            },
            VillageCode {
                name: "南地村村委会",
                code: "063",
            },
            VillageCode {
                name: "庞各庄镇开发区社区",
                code: "064",
            },
        ],
    },
    TownCode {
        name: "北臧村镇",
        code: "020",
        villages: &[
            VillageCode {
                name: "六合庄村村委会",
                code: "001",
            },
            VillageCode {
                name: "马村村委会",
                code: "002",
            },
            VillageCode {
                name: "新立村村委会",
                code: "003",
            },
            VillageCode {
                name: "桑马房村村委会",
                code: "004",
            },
            VillageCode {
                name: "八家村村委会",
                code: "005",
            },
            VillageCode {
                name: "西大营村村委会",
                code: "006",
            },
            VillageCode {
                name: "大臧村村委会",
                code: "007",
            },
            VillageCode {
                name: "赵家场村村委会",
                code: "008",
            },
            VillageCode {
                name: "巴园子村村委会",
                code: "009",
            },
            VillageCode {
                name: "诸葛营村村委会",
                code: "010",
            },
            VillageCode {
                name: "西王庄村村委会",
                code: "011",
            },
            VillageCode {
                name: "皮各庄一村村委会",
                code: "012",
            },
            VillageCode {
                name: "皮各庄二村村委会",
                code: "013",
            },
            VillageCode {
                name: "皮各庄三村村委会",
                code: "014",
            },
            VillageCode {
                name: "梨园村村委会",
                code: "015",
            },
            VillageCode {
                name: "前管营村村委会",
                code: "016",
            },
            VillageCode {
                name: "北高各庄村村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "魏善庄镇",
        code: "021",
        villages: &[
            VillageCode {
                name: "善康社区居委会",
                code: "001",
            },
            VillageCode {
                name: "善海东里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "善海西里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "善海北里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "魏新社区居委会",
                code: "005",
            },
            VillageCode {
                name: "后大营村村委会",
                code: "006",
            },
            VillageCode {
                name: "吴庄村村委会",
                code: "007",
            },
            VillageCode {
                name: "西芦垡村村委会",
                code: "008",
            },
            VillageCode {
                name: "东芦垡村村委会",
                code: "009",
            },
            VillageCode {
                name: "韩村村委会",
                code: "010",
            },
            VillageCode {
                name: "羊坊村村委会",
                code: "011",
            },
            VillageCode {
                name: "查家马房村村委会",
                code: "012",
            },
            VillageCode {
                name: "伊庄村村委会",
                code: "013",
            },
            VillageCode {
                name: "兴隆庄村村委会",
                code: "014",
            },
            VillageCode {
                name: "北研垡村村委会",
                code: "015",
            },
            VillageCode {
                name: "车站村村委会",
                code: "016",
            },
            VillageCode {
                name: "魏善庄村村委会",
                code: "017",
            },
            VillageCode {
                name: "王各庄村村委会",
                code: "018",
            },
            VillageCode {
                name: "穆园子村村委会",
                code: "019",
            },
            VillageCode {
                name: "赵庄子村村委会",
                code: "020",
            },
            VillageCode {
                name: "崔家庄一村村委会",
                code: "021",
            },
            VillageCode {
                name: "崔家庄二村村委会",
                code: "022",
            },
            VillageCode {
                name: "河北辛庄村村委会",
                code: "023",
            },
            VillageCode {
                name: "河南辛庄村村委会",
                code: "024",
            },
            VillageCode {
                name: "大刘各庄村村委会",
                code: "025",
            },
            VillageCode {
                name: "东枣林庄村村委会",
                code: "026",
            },
            VillageCode {
                name: "西枣林庄村村委会",
                code: "027",
            },
            VillageCode {
                name: "三顺庄村村委会",
                code: "028",
            },
            VillageCode {
                name: "陈各庄村村委会",
                code: "029",
            },
            VillageCode {
                name: "北田各庄村村委会",
                code: "030",
            },
            VillageCode {
                name: "南田各庄村村委会",
                code: "031",
            },
            VillageCode {
                name: "后苑上村村委会",
                code: "032",
            },
            VillageCode {
                name: "前苑上村村委会",
                code: "033",
            },
            VillageCode {
                name: "岳家务村村委会",
                code: "034",
            },
            VillageCode {
                name: "魏庄村村委会",
                code: "035",
            },
            VillageCode {
                name: "半壁店村村委会",
                code: "036",
            },
            VillageCode {
                name: "西南研垡村村委会",
                code: "037",
            },
            VillageCode {
                name: "东南研垡村村委会",
                code: "038",
            },
            VillageCode {
                name: "大狼垡村村委会",
                code: "039",
            },
            VillageCode {
                name: "西沙窝村村委会",
                code: "040",
            },
            VillageCode {
                name: "东沙窝村村委会",
                code: "041",
            },
            VillageCode {
                name: "李家场村村委会",
                code: "042",
            },
            VillageCode {
                name: "刘家场村村委会",
                code: "043",
            },
            VillageCode {
                name: "张家场村村委会",
                code: "044",
            },
        ],
    },
    TownCode {
        name: "长子营镇",
        code: "022",
        villages: &[
            VillageCode {
                name: "牛坊村村委会",
                code: "001",
            },
            VillageCode {
                name: "朱脑村村委会",
                code: "002",
            },
            VillageCode {
                name: "李家务村村委会",
                code: "003",
            },
            VillageCode {
                name: "北辛庄村村委会",
                code: "004",
            },
            VillageCode {
                name: "留民营村村委会",
                code: "005",
            },
            VillageCode {
                name: "赵县营村村委会",
                code: "006",
            },
            VillageCode {
                name: "窦营村村委会",
                code: "007",
            },
            VillageCode {
                name: "靳七营村村委会",
                code: "008",
            },
            VillageCode {
                name: "北泗上村村委会",
                code: "009",
            },
            VillageCode {
                name: "郑二营村村委会",
                code: "010",
            },
            VillageCode {
                name: "沁水营村村委会",
                code: "011",
            },
            VillageCode {
                name: "上长子营村村委会",
                code: "012",
            },
            VillageCode {
                name: "下长子营村村委会",
                code: "013",
            },
            VillageCode {
                name: "河津营村村委会",
                code: "014",
            },
            VillageCode {
                name: "安场村村委会",
                code: "015",
            },
            VillageCode {
                name: "小黑垡村村委会",
                code: "016",
            },
            VillageCode {
                name: "白庙村村委会",
                code: "017",
            },
            VillageCode {
                name: "上黎城村村委会",
                code: "018",
            },
            VillageCode {
                name: "孙庄村村委会",
                code: "019",
            },
            VillageCode {
                name: "北蒲洲营村村委会",
                code: "020",
            },
            VillageCode {
                name: "潞城营一村村委会",
                code: "021",
            },
            VillageCode {
                name: "潞城营二村村委会",
                code: "022",
            },
            VillageCode {
                name: "潞城营三村村委会",
                code: "023",
            },
            VillageCode {
                name: "潞城营四村村委会",
                code: "024",
            },
            VillageCode {
                name: "佟庄村村委会",
                code: "025",
            },
            VillageCode {
                name: "永和庄村村委会",
                code: "026",
            },
            VillageCode {
                name: "南蒲洲营村村委会",
                code: "027",
            },
            VillageCode {
                name: "车固营一村村委会",
                code: "028",
            },
            VillageCode {
                name: "车固营二村村委会",
                code: "029",
            },
            VillageCode {
                name: "周营村村委会",
                code: "030",
            },
            VillageCode {
                name: "公和庄村村委会",
                code: "031",
            },
            VillageCode {
                name: "罗庄一村村委会",
                code: "032",
            },
            VillageCode {
                name: "罗庄二村村委会",
                code: "033",
            },
            VillageCode {
                name: "罗庄三村村委会",
                code: "034",
            },
            VillageCode {
                name: "朱庄村村委会",
                code: "035",
            },
            VillageCode {
                name: "和顺场村村委会",
                code: "036",
            },
            VillageCode {
                name: "西北台村村委会",
                code: "037",
            },
            VillageCode {
                name: "东北台村村委会",
                code: "038",
            },
            VillageCode {
                name: "再城营一村村委会",
                code: "039",
            },
            VillageCode {
                name: "再城营二村村委会",
                code: "040",
            },
            VillageCode {
                name: "赤鲁村村委会",
                code: "041",
            },
            VillageCode {
                name: "李堡村村委会",
                code: "042",
            },
            VillageCode {
                name: "军民结合产业园社区",
                code: "043",
            },
        ],
    },
    TownCode {
        name: "北京经济技术开发区",
        code: "023",
        villages: &[VillageCode {
            name: "北京经济技术开发区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "中关村国家自主创新示范区大兴生物医药产业基地",
        code: "024",
        villages: &[VillageCode {
            name: "中关村国家自主创新示范区生物医药产业基地虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "大兴经济开发区",
        code: "025",
        villages: &[
            VillageCode {
                name: "大兴经济开发区一区社区",
                code: "001",
            },
            VillageCode {
                name: "大兴经济开发区二区社区",
                code: "002",
            },
            VillageCode {
                name: "大兴经济开发区三区社区",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "大兴国际机场",
        code: "026",
        villages: &[VillageCode {
            name: "大兴机场工作区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_BP_013: [TownCode; 17] = [
    TownCode {
        name: "泉河街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "富乐社区居委会",
                code: "001",
            },
            VillageCode {
                name: "富乐北里社区居委会",
                code: "002",
            },
            VillageCode {
                name: "滨湖社区居委会",
                code: "003",
            },
            VillageCode {
                name: "湖光社区居委会",
                code: "004",
            },
            VillageCode {
                name: "北园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "杨家园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "金台园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "航天工程大学社区居委会",
                code: "008",
            },
            VillageCode {
                name: "馥郁苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "于家园二区社区居委会",
                code: "010",
            },
            VillageCode {
                name: "开放路社区居委会",
                code: "011",
            },
            VillageCode {
                name: "新贤家园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "南大街村委会",
                code: "013",
            },
            VillageCode {
                name: "新贤街村委会",
                code: "014",
            },
            VillageCode {
                name: "后城街村委会",
                code: "015",
            },
            VillageCode {
                name: "钓鱼台村委会",
                code: "016",
            },
            VillageCode {
                name: "潘家园村委会",
                code: "017",
            },
            VillageCode {
                name: "杨家园村委会",
                code: "018",
            },
            VillageCode {
                name: "于家园村委会",
                code: "019",
            },
            VillageCode {
                name: "小中富乐村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "龙山街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "商业街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南城社区居委会",
                code: "002",
            },
            VillageCode {
                name: "车站路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "龙湖新村社区居委会",
                code: "004",
            },
            VillageCode {
                name: "南华园一区社区居委会",
                code: "005",
            },
            VillageCode {
                name: "丽湖社区居委会",
                code: "006",
            },
            VillageCode {
                name: "南华园四区社区居委会",
                code: "007",
            },
            VillageCode {
                name: "望怀社区居委会",
                code: "008",
            },
            VillageCode {
                name: "西园社区居委会",
                code: "009",
            },
            VillageCode {
                name: "迎宾路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "南华园三区社区居委会",
                code: "011",
            },
            VillageCode {
                name: "龙祥社区居委会",
                code: "012",
            },
            VillageCode {
                name: "东关村委会",
                code: "013",
            },
            VillageCode {
                name: "南关村委会",
                code: "014",
            },
            VillageCode {
                name: "东大街村委会",
                code: "015",
            },
            VillageCode {
                name: "下元村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "怀柔地区",
        code: "003",
        villages: &[
            VillageCode {
                name: "红螺家园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "滨河馨居社区居委会",
                code: "002",
            },
            VillageCode {
                name: "石厂村委会",
                code: "003",
            },
            VillageCode {
                name: "葛各庄村委会",
                code: "004",
            },
            VillageCode {
                name: "唐自口村委会",
                code: "005",
            },
            VillageCode {
                name: "张各长村委会",
                code: "006",
            },
            VillageCode {
                name: "王化村委会",
                code: "007",
            },
            VillageCode {
                name: "大屯村委会",
                code: "008",
            },
            VillageCode {
                name: "大中富乐村委会",
                code: "009",
            },
            VillageCode {
                name: "刘各长村委会",
                code: "010",
            },
            VillageCode {
                name: "东四村村委会",
                code: "011",
            },
            VillageCode {
                name: "芦庄村委会",
                code: "012",
            },
            VillageCode {
                name: "红螺镇村委会",
                code: "013",
            },
            VillageCode {
                name: "西三村村委会",
                code: "014",
            },
            VillageCode {
                name: "甘涧峪村委会",
                code: "015",
            },
            VillageCode {
                name: "郭家坞村委会",
                code: "016",
            },
            VillageCode {
                name: "红军庄村委会",
                code: "017",
            },
            VillageCode {
                name: "孟庄村委会",
                code: "018",
            },
            VillageCode {
                name: "兴隆庄村委会",
                code: "019",
            },
            VillageCode {
                name: "卧龙岗村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "雁栖地区",
        code: "004",
        villages: &[
            VillageCode {
                name: "新村社区居委会",
                code: "001",
            },
            VillageCode {
                name: "柏泉社区居委会",
                code: "002",
            },
            VillageCode {
                name: "乐园庄村委会",
                code: "003",
            },
            VillageCode {
                name: "陈各庄村委会",
                code: "004",
            },
            VillageCode {
                name: "下庄村委会",
                code: "005",
            },
            VillageCode {
                name: "范各庄村委会",
                code: "006",
            },
            VillageCode {
                name: "永乐庄村委会",
                code: "007",
            },
            VillageCode {
                name: "北台下村委会",
                code: "008",
            },
            VillageCode {
                name: "北台上村委会",
                code: "009",
            },
            VillageCode {
                name: "下辛庄村委会",
                code: "010",
            },
            VillageCode {
                name: "泉水头村委会",
                code: "011",
            },
            VillageCode {
                name: "柏崖厂村委会",
                code: "012",
            },
            VillageCode {
                name: "长元村委会",
                code: "013",
            },
            VillageCode {
                name: "莲花池村委会",
                code: "014",
            },
            VillageCode {
                name: "神堂峪村委会",
                code: "015",
            },
            VillageCode {
                name: "官地村委会",
                code: "016",
            },
            VillageCode {
                name: "石片村委会",
                code: "017",
            },
            VillageCode {
                name: "北湾村委会",
                code: "018",
            },
            VillageCode {
                name: "大地村委会",
                code: "019",
            },
            VillageCode {
                name: "头道梁村委会",
                code: "020",
            },
            VillageCode {
                name: "西栅子村委会",
                code: "021",
            },
            VillageCode {
                name: "八道河村委会",
                code: "022",
            },
            VillageCode {
                name: "交界河村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "庙城地区",
        code: "005",
        villages: &[
            VillageCode {
                name: "庙城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金山社区居委会",
                code: "002",
            },
            VillageCode {
                name: "高两河村委会",
                code: "003",
            },
            VillageCode {
                name: "李两河村委会",
                code: "004",
            },
            VillageCode {
                name: "小杜两河村委会",
                code: "005",
            },
            VillageCode {
                name: "刘两河村委会",
                code: "006",
            },
            VillageCode {
                name: "大杜两河村委会",
                code: "007",
            },
            VillageCode {
                name: "肖两河村委会",
                code: "008",
            },
            VillageCode {
                name: "赵各庄村委会",
                code: "009",
            },
            VillageCode {
                name: "霍各庄村委会",
                code: "010",
            },
            VillageCode {
                name: "焦村村委会",
                code: "011",
            },
            VillageCode {
                name: "彩各庄村委会",
                code: "012",
            },
            VillageCode {
                name: "庙城村委会",
                code: "013",
            },
            VillageCode {
                name: "桃山村委会",
                code: "014",
            },
            VillageCode {
                name: "王史山村委会",
                code: "015",
            },
            VillageCode {
                name: "孙史山村委会",
                code: "016",
            },
            VillageCode {
                name: "高各庄村委会",
                code: "017",
            },
            VillageCode {
                name: "郑重庄村委会",
                code: "018",
            },
            VillageCode {
                name: "西台上村委会",
                code: "019",
            },
            VillageCode {
                name: "西台下村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "北房镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "幸福东园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "裕华园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "宰相庄村委会",
                code: "003",
            },
            VillageCode {
                name: "安各庄村委会",
                code: "004",
            },
            VillageCode {
                name: "北房村委会",
                code: "005",
            },
            VillageCode {
                name: "南房村委会",
                code: "006",
            },
            VillageCode {
                name: "黄吉营村委会",
                code: "007",
            },
            VillageCode {
                name: "驸马庄村委会",
                code: "008",
            },
            VillageCode {
                name: "梨园庄村委会",
                code: "009",
            },
            VillageCode {
                name: "郑家庄村委会",
                code: "010",
            },
            VillageCode {
                name: "韦里村委会",
                code: "011",
            },
            VillageCode {
                name: "小罗山村委会",
                code: "012",
            },
            VillageCode {
                name: "大罗山村委会",
                code: "013",
            },
            VillageCode {
                name: "小辛庄村委会",
                code: "014",
            },
            VillageCode {
                name: "大周各庄村委会",
                code: "015",
            },
            VillageCode {
                name: "小周各庄村委会",
                code: "016",
            },
            VillageCode {
                name: "新房子村委会",
                code: "017",
            },
            VillageCode {
                name: "胜利村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "杨宋镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "凤翔社区居委会",
                code: "001",
            },
            VillageCode {
                name: "杨宋庄村委会",
                code: "002",
            },
            VillageCode {
                name: "仙台村委会",
                code: "003",
            },
            VillageCode {
                name: "西树行村委会",
                code: "004",
            },
            VillageCode {
                name: "北年丰村委会",
                code: "005",
            },
            VillageCode {
                name: "南年丰村委会",
                code: "006",
            },
            VillageCode {
                name: "四季屯村委会",
                code: "007",
            },
            VillageCode {
                name: "解村村委会",
                code: "008",
            },
            VillageCode {
                name: "耿辛庄村委会",
                code: "009",
            },
            VillageCode {
                name: "张各庄满族村委会",
                code: "010",
            },
            VillageCode {
                name: "花园村委会",
                code: "011",
            },
            VillageCode {
                name: "郭庄村委会",
                code: "012",
            },
            VillageCode {
                name: "安乐庄村委会",
                code: "013",
            },
            VillageCode {
                name: "张自口村委会",
                code: "014",
            },
            VillageCode {
                name: "太平庄满族村委会",
                code: "015",
            },
            VillageCode {
                name: "梭草村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "桥梓镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "茶坞铁路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "前桥梓村委会",
                code: "002",
            },
            VillageCode {
                name: "后桥梓村委会",
                code: "003",
            },
            VillageCode {
                name: "山立庄村委会",
                code: "004",
            },
            VillageCode {
                name: "东茶坞村委会",
                code: "005",
            },
            VillageCode {
                name: "西茶坞村委会",
                code: "006",
            },
            VillageCode {
                name: "前茶坞村委会",
                code: "007",
            },
            VillageCode {
                name: "平义分村委会",
                code: "008",
            },
            VillageCode {
                name: "沙峪口村委会",
                code: "009",
            },
            VillageCode {
                name: "新王峪村委会",
                code: "010",
            },
            VillageCode {
                name: "上王峪村委会",
                code: "011",
            },
            VillageCode {
                name: "苏峪口村委会",
                code: "012",
            },
            VillageCode {
                name: "岐庄村委会",
                code: "013",
            },
            VillageCode {
                name: "东凤山村委会",
                code: "014",
            },
            VillageCode {
                name: "红林村委会",
                code: "015",
            },
            VillageCode {
                name: "口头村委会",
                code: "016",
            },
            VillageCode {
                name: "凯甲庄村委会",
                code: "017",
            },
            VillageCode {
                name: "北宅村委会",
                code: "018",
            },
            VillageCode {
                name: "峪口村委会",
                code: "019",
            },
            VillageCode {
                name: "峪沟村委会",
                code: "020",
            },
            VillageCode {
                name: "一渡河村委会",
                code: "021",
            },
            VillageCode {
                name: "后辛庄村委会",
                code: "022",
            },
            VillageCode {
                name: "前辛庄村委会",
                code: "023",
            },
            VillageCode {
                name: "秦家东庄村委会",
                code: "024",
            },
            VillageCode {
                name: "杨家东庄村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "怀北镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "怀北铁路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西庄村委会",
                code: "002",
            },
            VillageCode {
                name: "东庄村委会",
                code: "003",
            },
            VillageCode {
                name: "怀北庄村委会",
                code: "004",
            },
            VillageCode {
                name: "龙各庄村委会",
                code: "005",
            },
            VillageCode {
                name: "神山村委会",
                code: "006",
            },
            VillageCode {
                name: "邓各庄村委会",
                code: "007",
            },
            VillageCode {
                name: "大水峪村委会",
                code: "008",
            },
            VillageCode {
                name: "河防口村委会",
                code: "009",
            },
            VillageCode {
                name: "椴树岭村委会",
                code: "010",
            },
            VillageCode {
                name: "新峰村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "汤河口镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "汤河口社区居委会",
                code: "001",
            },
            VillageCode {
                name: "小梁前村委会",
                code: "002",
            },
            VillageCode {
                name: "二号沟门村委会",
                code: "003",
            },
            VillageCode {
                name: "黄花甸子村委会",
                code: "004",
            },
            VillageCode {
                name: "许营村委会",
                code: "005",
            },
            VillageCode {
                name: "银河沟村委会",
                code: "006",
            },
            VillageCode {
                name: "大栅子村委会",
                code: "007",
            },
            VillageCode {
                name: "庄户沟门村委会",
                code: "008",
            },
            VillageCode {
                name: "东帽湾村委会",
                code: "009",
            },
            VillageCode {
                name: "西帽湾村委会",
                code: "010",
            },
            VillageCode {
                name: "大榆树村委会",
                code: "011",
            },
            VillageCode {
                name: "新地村委会",
                code: "012",
            },
            VillageCode {
                name: "汤河口村委会",
                code: "013",
            },
            VillageCode {
                name: "河东村委会",
                code: "014",
            },
            VillageCode {
                name: "大蒲池沟村委会",
                code: "015",
            },
            VillageCode {
                name: "连石沟村委会",
                code: "016",
            },
            VillageCode {
                name: "古石沟门村委会",
                code: "017",
            },
            VillageCode {
                name: "东黄梁村委会",
                code: "018",
            },
            VillageCode {
                name: "卜营村委会",
                code: "019",
            },
            VillageCode {
                name: "大黄塘村委会",
                code: "020",
            },
            VillageCode {
                name: "小黄塘村委会",
                code: "021",
            },
            VillageCode {
                name: "后安岭村委会",
                code: "022",
            },
            VillageCode {
                name: "东湾子村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "渤海镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "渤海所村委会",
                code: "001",
            },
            VillageCode {
                name: "景峪村委会",
                code: "002",
            },
            VillageCode {
                name: "龙泉庄村委会",
                code: "003",
            },
            VillageCode {
                name: "白木村委会",
                code: "004",
            },
            VillageCode {
                name: "沙峪村委会",
                code: "005",
            },
            VillageCode {
                name: "南冶村委会",
                code: "006",
            },
            VillageCode {
                name: "洞台村委会",
                code: "007",
            },
            VillageCode {
                name: "铁矿峪村委会",
                code: "008",
            },
            VillageCode {
                name: "大榛峪村委会",
                code: "009",
            },
            VillageCode {
                name: "庄户村委会",
                code: "010",
            },
            VillageCode {
                name: "三岔村委会",
                code: "011",
            },
            VillageCode {
                name: "兴隆城村委会",
                code: "012",
            },
            VillageCode {
                name: "六渡河村委会",
                code: "013",
            },
            VillageCode {
                name: "四渡河村委会",
                code: "014",
            },
            VillageCode {
                name: "三渡河村委会",
                code: "015",
            },
            VillageCode {
                name: "马道峪村委会",
                code: "016",
            },
            VillageCode {
                name: "苇店村委会",
                code: "017",
            },
            VillageCode {
                name: "辛营村委会",
                code: "018",
            },
            VillageCode {
                name: "北沟村委会",
                code: "019",
            },
            VillageCode {
                name: "田仙峪村委会",
                code: "020",
            },
            VillageCode {
                name: "慕田峪村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "九渡河镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "四渡河村委会",
                code: "001",
            },
            VillageCode {
                name: "黄坎村委会",
                code: "002",
            },
            VillageCode {
                name: "吉寺村委会",
                code: "003",
            },
            VillageCode {
                name: "团泉村委会",
                code: "004",
            },
            VillageCode {
                name: "局里村委会",
                code: "005",
            },
            VillageCode {
                name: "花木村委会",
                code: "006",
            },
            VillageCode {
                name: "九渡河村委会",
                code: "007",
            },
            VillageCode {
                name: "黄花镇村委会",
                code: "008",
            },
            VillageCode {
                name: "东宫村委会",
                code: "009",
            },
            VillageCode {
                name: "西台村委会",
                code: "010",
            },
            VillageCode {
                name: "黄花城村委会",
                code: "011",
            },
            VillageCode {
                name: "撞道口村委会",
                code: "012",
            },
            VillageCode {
                name: "石湖峪村委会",
                code: "013",
            },
            VillageCode {
                name: "西水峪村委会",
                code: "014",
            },
            VillageCode {
                name: "二道关村委会",
                code: "015",
            },
            VillageCode {
                name: "杏树台村委会",
                code: "016",
            },
            VillageCode {
                name: "庙上村委会",
                code: "017",
            },
            VillageCode {
                name: "红庙村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "琉璃庙镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "后山铺村委会",
                code: "001",
            },
            VillageCode {
                name: "东峪村委会",
                code: "002",
            },
            VillageCode {
                name: "龙泉峪村委会",
                code: "003",
            },
            VillageCode {
                name: "柏查子村委会",
                code: "004",
            },
            VillageCode {
                name: "琉璃庙村委会",
                code: "005",
            },
            VillageCode {
                name: "得田沟村委会",
                code: "006",
            },
            VillageCode {
                name: "碾子湾村委会",
                code: "007",
            },
            VillageCode {
                name: "老公营村委会",
                code: "008",
            },
            VillageCode {
                name: "安洲坝村委会",
                code: "009",
            },
            VillageCode {
                name: "西湾子村委会",
                code: "010",
            },
            VillageCode {
                name: "前安岭村委会",
                code: "011",
            },
            VillageCode {
                name: "双文铺村委会",
                code: "012",
            },
            VillageCode {
                name: "青石岭村委会",
                code: "013",
            },
            VillageCode {
                name: "白河北村委会",
                code: "014",
            },
            VillageCode {
                name: "狼虎哨村委会",
                code: "015",
            },
            VillageCode {
                name: "西台子村委会",
                code: "016",
            },
            VillageCode {
                name: "崎峰茶村委会",
                code: "017",
            },
            VillageCode {
                name: "孙胡沟村委会",
                code: "018",
            },
            VillageCode {
                name: "长岭沟门村委会",
                code: "019",
            },
            VillageCode {
                name: "鱼水洞村委会",
                code: "020",
            },
            VillageCode {
                name: "河北村委会",
                code: "021",
            },
            VillageCode {
                name: "八亩地村委会",
                code: "022",
            },
            VillageCode {
                name: "二台子村委会",
                code: "023",
            },
            VillageCode {
                name: "杨树下村委会",
                code: "024",
            },
            VillageCode {
                name: "梁根村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "宝山镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "宝山寺村委会",
                code: "001",
            },
            VillageCode {
                name: "养渔池村委会",
                code: "002",
            },
            VillageCode {
                name: "超梁子村委会",
                code: "003",
            },
            VillageCode {
                name: "对石村委会",
                code: "004",
            },
            VillageCode {
                name: "西黄梁村委会",
                code: "005",
            },
            VillageCode {
                name: "盘道沟村委会",
                code: "006",
            },
            VillageCode {
                name: "西帽山村委会",
                code: "007",
            },
            VillageCode {
                name: "牛圈子村委会",
                code: "008",
            },
            VillageCode {
                name: "大黄木厂村委会",
                code: "009",
            },
            VillageCode {
                name: "小黄木厂村委会",
                code: "010",
            },
            VillageCode {
                name: "下坊村委会",
                code: "011",
            },
            VillageCode {
                name: "转年村委会",
                code: "012",
            },
            VillageCode {
                name: "杨树下村委会",
                code: "013",
            },
            VillageCode {
                name: "郑栅子村委会",
                code: "014",
            },
            VillageCode {
                name: "温栅子村委会",
                code: "015",
            },
            VillageCode {
                name: "下栅子村委会",
                code: "016",
            },
            VillageCode {
                name: "四道河村委会",
                code: "017",
            },
            VillageCode {
                name: "道德坑村委会",
                code: "018",
            },
            VillageCode {
                name: "阳坡村委会",
                code: "019",
            },
            VillageCode {
                name: "松树台村委会",
                code: "020",
            },
            VillageCode {
                name: "四道窝铺村委会",
                code: "021",
            },
            VillageCode {
                name: "碾子村委会",
                code: "022",
            },
            VillageCode {
                name: "菜树甸村委会",
                code: "023",
            },
            VillageCode {
                name: "三块石村委会",
                code: "024",
            },
            VillageCode {
                name: "江村村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "长哨营满族乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "东南沟村委会",
                code: "001",
            },
            VillageCode {
                name: "老西沟村委会",
                code: "002",
            },
            VillageCode {
                name: "长哨营村委会",
                code: "003",
            },
            VillageCode {
                name: "遥岭村委会",
                code: "004",
            },
            VillageCode {
                name: "杨树湾村委会",
                code: "005",
            },
            VillageCode {
                name: "二道河村委会",
                code: "006",
            },
            VillageCode {
                name: "三岔口村委会",
                code: "007",
            },
            VillageCode {
                name: "大地村委会",
                code: "008",
            },
            VillageCode {
                name: "榆树湾村委会",
                code: "009",
            },
            VillageCode {
                name: "古洞沟村委会",
                code: "010",
            },
            VillageCode {
                name: "七道梁村委会",
                code: "011",
            },
            VillageCode {
                name: "北湾村委会",
                code: "012",
            },
            VillageCode {
                name: "东辛店村委会",
                code: "013",
            },
            VillageCode {
                name: "北干沟村委会",
                code: "014",
            },
            VillageCode {
                name: "四道河村委会",
                code: "015",
            },
            VillageCode {
                name: "大沟村委会",
                code: "016",
            },
            VillageCode {
                name: "项栅子村委会",
                code: "017",
            },
            VillageCode {
                name: "七道河村委会",
                code: "018",
            },
            VillageCode {
                name: "八道河村委会",
                code: "019",
            },
            VillageCode {
                name: "西沟村委会",
                code: "020",
            },
            VillageCode {
                name: "后沟村委会",
                code: "021",
            },
            VillageCode {
                name: "上孟营村委会",
                code: "022",
            },
            VillageCode {
                name: "老沟门村委会",
                code: "023",
            },
            VillageCode {
                name: "三道窝铺村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "喇叭沟门满族乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "帽山村委会",
                code: "001",
            },
            VillageCode {
                name: "胡营村委会",
                code: "002",
            },
            VillageCode {
                name: "四道穴村委会",
                code: "003",
            },
            VillageCode {
                name: "西府营村委会",
                code: "004",
            },
            VillageCode {
                name: "中榆树店村委会",
                code: "005",
            },
            VillageCode {
                name: "下河北村委会",
                code: "006",
            },
            VillageCode {
                name: "孙栅子村委会",
                code: "007",
            },
            VillageCode {
                name: "北辛店村委会",
                code: "008",
            },
            VillageCode {
                name: "苗营村委会",
                code: "009",
            },
            VillageCode {
                name: "官帽山村委会",
                code: "010",
            },
            VillageCode {
                name: "喇叭沟门村委会",
                code: "011",
            },
            VillageCode {
                name: "大甸子村委会",
                code: "012",
            },
            VillageCode {
                name: "东岔村委会",
                code: "013",
            },
            VillageCode {
                name: "对角沟门村委会",
                code: "014",
            },
            VillageCode {
                name: "上台子村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "北京雁栖经济开发区",
        code: "017",
        villages: &[VillageCode {
            name: "北京雁栖经济开发区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_BP_014: [TownCode; 18] = [
    TownCode {
        name: "滨河街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "金谷东园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "平粮社区居委会",
                code: "002",
            },
            VillageCode {
                name: "向阳社区居委会",
                code: "003",
            },
            VillageCode {
                name: "金谷园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "承平园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "北小区社区居委会",
                code: "006",
            },
            VillageCode {
                name: "建西社区居委会",
                code: "007",
            },
            VillageCode {
                name: "南小区社区居委会",
                code: "008",
            },
            VillageCode {
                name: "滨河社区居委会",
                code: "009",
            },
            VillageCode {
                name: "金海社区居委会",
                code: "010",
            },
            VillageCode {
                name: "建南社区居委会",
                code: "011",
            },
            VillageCode {
                name: "府前社区居委会",
                code: "012",
            },
            VillageCode {
                name: "林荫家园社区居委会",
                code: "013",
            },
            VillageCode {
                name: "绿谷新苑社区居委会",
                code: "014",
            },
            VillageCode {
                name: "泃河湾社区居委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "兴谷街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "乐园东社区居委会",
                code: "001",
            },
            VillageCode {
                name: "兴谷园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "光明社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新星社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金乡东社区居委会",
                code: "005",
            },
            VillageCode {
                name: "金乡西社区居委会",
                code: "006",
            },
            VillageCode {
                name: "园丁社区居委会",
                code: "007",
            },
            VillageCode {
                name: "乐园西社区居委会",
                code: "008",
            },
            VillageCode {
                name: "阳光社区居委会",
                code: "009",
            },
            VillageCode {
                name: "兴谷家园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "邑上原著社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "腾龙社区居委会",
                code: "012",
            },
            VillageCode {
                name: "观岭家园社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "杜辛庄村委会",
                code: "014",
            },
            VillageCode {
                name: "中罗庄村委会",
                code: "015",
            },
            VillageCode {
                name: "上纸寨村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "渔阳地区",
        code: "003",
        villages: &[
            VillageCode {
                name: "海关西园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "胜利社区居委会",
                code: "002",
            },
            VillageCode {
                name: "太和园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "建兰居委会",
                code: "004",
            },
            VillageCode {
                name: "迎宾花园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "岳泰嘉园社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "仁和社区居委会",
                code: "007",
            },
            VillageCode {
                name: "洳河社区居委会",
                code: "008",
            },
            VillageCode {
                name: "西寺渠村委会",
                code: "009",
            },
            VillageCode {
                name: "东寺渠村委会",
                code: "010",
            },
            VillageCode {
                name: "园田队村委会",
                code: "011",
            },
            VillageCode {
                name: "胜利街村委会",
                code: "012",
            },
            VillageCode {
                name: "平安街村委会",
                code: "013",
            },
            VillageCode {
                name: "和平街村委会",
                code: "014",
            },
            VillageCode {
                name: "太平街村委会",
                code: "015",
            },
            VillageCode {
                name: "岳各庄村委会",
                code: "016",
            },
            VillageCode {
                name: "赵各庄村委会",
                code: "017",
            },
            VillageCode {
                name: "北台头村委会",
                code: "018",
            },
            VillageCode {
                name: "西鹿角村委会",
                code: "019",
            },
            VillageCode {
                name: "下纸寨村委会",
                code: "020",
            },
            VillageCode {
                name: "东鹿角村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "峪口地区",
        code: "004",
        villages: &[
            VillageCode {
                name: "峪口社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西营村委会",
                code: "002",
            },
            VillageCode {
                name: "东凡各庄村委会",
                code: "003",
            },
            VillageCode {
                name: "西凡各庄村委会",
                code: "004",
            },
            VillageCode {
                name: "三白山村委会",
                code: "005",
            },
            VillageCode {
                name: "胡家营村委会",
                code: "006",
            },
            VillageCode {
                name: "兴隆庄村委会",
                code: "007",
            },
            VillageCode {
                name: "中桥村委会",
                code: "008",
            },
            VillageCode {
                name: "蔡坨村委会",
                code: "009",
            },
            VillageCode {
                name: "南营村委会",
                code: "010",
            },
            VillageCode {
                name: "坨头寺村委会",
                code: "011",
            },
            VillageCode {
                name: "胡辛庄村委会",
                code: "012",
            },
            VillageCode {
                name: "梨各庄村委会",
                code: "013",
            },
            VillageCode {
                name: "北杨家桥村委会",
                code: "014",
            },
            VillageCode {
                name: "南杨家桥村委会",
                code: "015",
            },
            VillageCode {
                name: "桥头村委会",
                code: "016",
            },
            VillageCode {
                name: "厂门口村委会",
                code: "017",
            },
            VillageCode {
                name: "云峰寺村委会",
                code: "018",
            },
            VillageCode {
                name: "大官庄村委会",
                code: "019",
            },
            VillageCode {
                name: "小官庄村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "马坊地区",
        code: "005",
        villages: &[
            VillageCode {
                name: "汇景社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "慧谷社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "新农社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "腾飞社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "东店村委会",
                code: "005",
            },
            VillageCode {
                name: "三条街村委会",
                code: "006",
            },
            VillageCode {
                name: "二条街村委会",
                code: "007",
            },
            VillageCode {
                name: "西大街村委会",
                code: "008",
            },
            VillageCode {
                name: "蒋里庄村委会",
                code: "009",
            },
            VillageCode {
                name: "塔寺村委会",
                code: "010",
            },
            VillageCode {
                name: "石佛寺村委会",
                code: "011",
            },
            VillageCode {
                name: "李蔡街村委会",
                code: "012",
            },
            VillageCode {
                name: "早立庄村委会",
                code: "013",
            },
            VillageCode {
                name: "河北村村委会",
                code: "014",
            },
            VillageCode {
                name: "小屯村委会",
                code: "015",
            },
            VillageCode {
                name: "英城村委会",
                code: "016",
            },
            VillageCode {
                name: "果各庄村委会",
                code: "017",
            },
            VillageCode {
                name: "洼里村委会",
                code: "018",
            },
            VillageCode {
                name: "梨羊村委会",
                code: "019",
            },
            VillageCode {
                name: "打铁庄村委会",
                code: "020",
            },
            VillageCode {
                name: "西太平庄村委会",
                code: "021",
            },
            VillageCode {
                name: "新建队村委会",
                code: "022",
            },
            VillageCode {
                name: "东撞村委会",
                code: "023",
            },
            VillageCode {
                name: "杈子庄村委会",
                code: "024",
            },
            VillageCode {
                name: "北石渠村委会",
                code: "025",
            },
            VillageCode {
                name: "河奎村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "金海湖地区",
        code: "006",
        villages: &[
            VillageCode {
                name: "罗汉石居委会",
                code: "001",
            },
            VillageCode {
                name: "东马各庄居委会",
                code: "002",
            },
            VillageCode {
                name: "韩庄村委会",
                code: "003",
            },
            VillageCode {
                name: "胡庄村委会",
                code: "004",
            },
            VillageCode {
                name: "东土门村委会",
                code: "005",
            },
            VillageCode {
                name: "马屯村委会",
                code: "006",
            },
            VillageCode {
                name: "祖务村委会",
                code: "007",
            },
            VillageCode {
                name: "耿井村委会",
                code: "008",
            },
            VillageCode {
                name: "晏庄村委会",
                code: "009",
            },
            VillageCode {
                name: "上宅村委会",
                code: "010",
            },
            VillageCode {
                name: "滑子村委会",
                code: "011",
            },
            VillageCode {
                name: "洙水村委会",
                code: "012",
            },
            VillageCode {
                name: "水峪村委会",
                code: "013",
            },
            VillageCode {
                name: "向阳村村委会",
                code: "014",
            },
            VillageCode {
                name: "海子村委会",
                code: "015",
            },
            VillageCode {
                name: "靠山集村委会",
                code: "016",
            },
            VillageCode {
                name: "郭家屯村委会",
                code: "017",
            },
            VillageCode {
                name: "东上营村委会",
                code: "018",
            },
            VillageCode {
                name: "茅山后村委会",
                code: "019",
            },
            VillageCode {
                name: "小东沟村委会",
                code: "020",
            },
            VillageCode {
                name: "彰作村委会",
                code: "021",
            },
            VillageCode {
                name: "红石门村委会",
                code: "022",
            },
            VillageCode {
                name: "中心村村委会",
                code: "023",
            },
            VillageCode {
                name: "将军关村委会",
                code: "024",
            },
            VillageCode {
                name: "黑水湾村委会",
                code: "025",
            },
            VillageCode {
                name: "黄草洼村委会",
                code: "026",
            },
            VillageCode {
                name: "红石坎村委会",
                code: "027",
            },
            VillageCode {
                name: "上堡子村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "东高村镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "东高村村委会",
                code: "001",
            },
            VillageCode {
                name: "西高村村委会",
                code: "002",
            },
            VillageCode {
                name: "南埝头村委会",
                code: "003",
            },
            VillageCode {
                name: "大旺务村委会",
                code: "004",
            },
            VillageCode {
                name: "大庄户村委会",
                code: "005",
            },
            VillageCode {
                name: "赵家务村委会",
                code: "006",
            },
            VillageCode {
                name: "赵庄户村委会",
                code: "007",
            },
            VillageCode {
                name: "克头村委会",
                code: "008",
            },
            VillageCode {
                name: "前台头村委会",
                code: "009",
            },
            VillageCode {
                name: "南张岱村委会",
                code: "010",
            },
            VillageCode {
                name: "北张岱村委会",
                code: "011",
            },
            VillageCode {
                name: "张岱辛撞村委会",
                code: "012",
            },
            VillageCode {
                name: "青杨屯村委会",
                code: "013",
            },
            VillageCode {
                name: "崔家庄村委会",
                code: "014",
            },
            VillageCode {
                name: "侯家庄村委会",
                code: "015",
            },
            VillageCode {
                name: "门楼庄村委会",
                code: "016",
            },
            VillageCode {
                name: "鲍家庄村委会",
                code: "017",
            },
            VillageCode {
                name: "高家庄村委会",
                code: "018",
            },
            VillageCode {
                name: "曹家庄村委会",
                code: "019",
            },
            VillageCode {
                name: "普贤屯村委会",
                code: "020",
            },
            VillageCode {
                name: "南宅村委会",
                code: "021",
            },
            VillageCode {
                name: "南宅庄户村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "山东庄镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "棠韵家园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "桥头营村委会",
                code: "002",
            },
            VillageCode {
                name: "西沥津村委会",
                code: "003",
            },
            VillageCode {
                name: "大坎村委会",
                code: "004",
            },
            VillageCode {
                name: "东洼村委会",
                code: "005",
            },
            VillageCode {
                name: "北寺村委会",
                code: "006",
            },
            VillageCode {
                name: "李辛庄村委会",
                code: "007",
            },
            VillageCode {
                name: "北屯村委会",
                code: "008",
            },
            VillageCode {
                name: "大北关村委会",
                code: "009",
            },
            VillageCode {
                name: "小北关村委会",
                code: "010",
            },
            VillageCode {
                name: "山东庄村委会",
                code: "011",
            },
            VillageCode {
                name: "鱼子山村委会",
                code: "012",
            },
            VillageCode {
                name: "桃棚村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "南独乐河镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "南独乐河村委会",
                code: "001",
            },
            VillageCode {
                name: "北独乐河村委会",
                code: "002",
            },
            VillageCode {
                name: "刘家河村委会",
                code: "003",
            },
            VillageCode {
                name: "峨嵋山村委会",
                code: "004",
            },
            VillageCode {
                name: "北寨村委会",
                code: "005",
            },
            VillageCode {
                name: "公爷坟村委会",
                code: "006",
            },
            VillageCode {
                name: "峰台村委会",
                code: "007",
            },
            VillageCode {
                name: "张辛庄村委会",
                code: "008",
            },
            VillageCode {
                name: "望马台村委会",
                code: "009",
            },
            VillageCode {
                name: "甘营村委会",
                code: "010",
            },
            VillageCode {
                name: "南山村村委会",
                code: "011",
            },
            VillageCode {
                name: "新农村村委会",
                code: "012",
            },
            VillageCode {
                name: "新立村村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "大华山镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "前北宫村委会",
                code: "001",
            },
            VillageCode {
                name: "后北宫村委会",
                code: "002",
            },
            VillageCode {
                name: "胜利村村委会",
                code: "003",
            },
            VillageCode {
                name: "陈庄子村委会",
                code: "004",
            },
            VillageCode {
                name: "苏子峪村委会",
                code: "005",
            },
            VillageCode {
                name: "山门沟村委会",
                code: "006",
            },
            VillageCode {
                name: "麻子峪村委会",
                code: "007",
            },
            VillageCode {
                name: "挂甲峪村委会",
                code: "008",
            },
            VillageCode {
                name: "大华山村委会",
                code: "009",
            },
            VillageCode {
                name: "砖瓦窑村委会",
                code: "010",
            },
            VillageCode {
                name: "泉水峪村委会",
                code: "011",
            },
            VillageCode {
                name: "西峪村委会",
                code: "012",
            },
            VillageCode {
                name: "西长峪村委会",
                code: "013",
            },
            VillageCode {
                name: "西牛峪村委会",
                code: "014",
            },
            VillageCode {
                name: "瓦官头村委会",
                code: "015",
            },
            VillageCode {
                name: "梯子峪村委会",
                code: "016",
            },
            VillageCode {
                name: "李家峪村委会",
                code: "017",
            },
            VillageCode {
                name: "东辛撞村委会",
                code: "018",
            },
            VillageCode {
                name: "大峪子村委会",
                code: "019",
            },
            VillageCode {
                name: "小峪子村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "夏各庄镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "礼义园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "知义园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "仁义园社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "信义园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "张各庄村委会",
                code: "005",
            },
            VillageCode {
                name: "杨各庄村委会",
                code: "006",
            },
            VillageCode {
                name: "马各庄村委会",
                code: "007",
            },
            VillageCode {
                name: "龙家务村委会",
                code: "008",
            },
            VillageCode {
                name: "贤王庄村委会",
                code: "009",
            },
            VillageCode {
                name: "王都庄村委会",
                code: "010",
            },
            VillageCode {
                name: "陈太务村委会",
                code: "011",
            },
            VillageCode {
                name: "纪太务村委会",
                code: "012",
            },
            VillageCode {
                name: "魏太务村委会",
                code: "013",
            },
            VillageCode {
                name: "南太务村委会",
                code: "014",
            },
            VillageCode {
                name: "夏各庄村委会",
                code: "015",
            },
            VillageCode {
                name: "安固村委会",
                code: "016",
            },
            VillageCode {
                name: "稻地村委会",
                code: "017",
            },
            VillageCode {
                name: "杨庄户村委会",
                code: "018",
            },
            VillageCode {
                name: "大岭后村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "马昌营镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "紫贵佳苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "圪塔头村委会",
                code: "002",
            },
            VillageCode {
                name: "王官屯村委会",
                code: "003",
            },
            VillageCode {
                name: "毛官营村委会",
                code: "004",
            },
            VillageCode {
                name: "王各庄村委会",
                code: "005",
            },
            VillageCode {
                name: "马昌营村委会",
                code: "006",
            },
            VillageCode {
                name: "魏辛庄村委会",
                code: "007",
            },
            VillageCode {
                name: "东陈各庄村委会",
                code: "008",
            },
            VillageCode {
                name: "西陈各庄村委会",
                code: "009",
            },
            VillageCode {
                name: "东双营村委会",
                code: "010",
            },
            VillageCode {
                name: "西双营村委会",
                code: "011",
            },
            VillageCode {
                name: "南定福庄村委会",
                code: "012",
            },
            VillageCode {
                name: "北定福庄村委会",
                code: "013",
            },
            VillageCode {
                name: "薄各庄村委会",
                code: "014",
            },
            VillageCode {
                name: "天井村委会",
                code: "015",
            },
            VillageCode {
                name: "前芮营村委会",
                code: "016",
            },
            VillageCode {
                name: "后芮营村委会",
                code: "017",
            },
            VillageCode {
                name: "西海子村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "王辛庄镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "太后村委会",
                code: "001",
            },
            VillageCode {
                name: "北上营村委会",
                code: "002",
            },
            VillageCode {
                name: "中胡家务村委会",
                code: "003",
            },
            VillageCode {
                name: "熊耳营村委会",
                code: "004",
            },
            VillageCode {
                name: "东古村村委会",
                code: "005",
            },
            VillageCode {
                name: "西古村村委会",
                code: "006",
            },
            VillageCode {
                name: "太平庄村委会",
                code: "007",
            },
            VillageCode {
                name: "大辛寨村委会",
                code: "008",
            },
            VillageCode {
                name: "小辛寨村委会",
                code: "009",
            },
            VillageCode {
                name: "贾各庄村委会",
                code: "010",
            },
            VillageCode {
                name: "齐各庄村委会",
                code: "011",
            },
            VillageCode {
                name: "王辛庄村委会",
                code: "012",
            },
            VillageCode {
                name: "后罗庄村委会",
                code: "013",
            },
            VillageCode {
                name: "许家务村委会",
                code: "014",
            },
            VillageCode {
                name: "莲花潭村委会",
                code: "015",
            },
            VillageCode {
                name: "放光村委会",
                code: "016",
            },
            VillageCode {
                name: "杨家会村委会",
                code: "017",
            },
            VillageCode {
                name: "井峪村委会",
                code: "018",
            },
            VillageCode {
                name: "北辛庄村委会",
                code: "019",
            },
            VillageCode {
                name: "翟各庄村委会",
                code: "020",
            },
            VillageCode {
                name: "西杏园村委会",
                code: "021",
            },
            VillageCode {
                name: "东杏园村委会",
                code: "022",
            },
            VillageCode {
                name: "乐政务村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "大兴庄镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "如歌家园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "洳苑嘉园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "大兴庄村委会",
                code: "003",
            },
            VillageCode {
                name: "鲁各庄村委会",
                code: "004",
            },
            VillageCode {
                name: "白各庄村委会",
                code: "005",
            },
            VillageCode {
                name: "北城子村委会",
                code: "006",
            },
            VillageCode {
                name: "东柏店村委会",
                code: "007",
            },
            VillageCode {
                name: "北埝头村委会",
                code: "008",
            },
            VillageCode {
                name: "唐庄子村委会",
                code: "009",
            },
            VillageCode {
                name: "西柏店村委会",
                code: "010",
            },
            VillageCode {
                name: "周庄子村委会",
                code: "011",
            },
            VillageCode {
                name: "韩屯村委会",
                code: "012",
            },
            VillageCode {
                name: "吉卧村委会",
                code: "013",
            },
            VillageCode {
                name: "良庄子村委会",
                code: "014",
            },
            VillageCode {
                name: "三府庄村委会",
                code: "015",
            },
            VillageCode {
                name: "陈良屯村委会",
                code: "016",
            },
            VillageCode {
                name: "西石桥村委会",
                code: "017",
            },
            VillageCode {
                name: "东石桥村委会",
                code: "018",
            },
            VillageCode {
                name: "管家庄村委会",
                code: "019",
            },
            VillageCode {
                name: "周村村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "刘家店镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "凤落滩村委会",
                code: "001",
            },
            VillageCode {
                name: "北店村委会",
                code: "002",
            },
            VillageCode {
                name: "北吉山村委会",
                code: "003",
            },
            VillageCode {
                name: "前吉山村委会",
                code: "004",
            },
            VillageCode {
                name: "松棚村委会",
                code: "005",
            },
            VillageCode {
                name: "刘家店村委会",
                code: "006",
            },
            VillageCode {
                name: "孔城峪村委会",
                code: "007",
            },
            VillageCode {
                name: "万庄子村委会",
                code: "008",
            },
            VillageCode {
                name: "胡家店村委会",
                code: "009",
            },
            VillageCode {
                name: "寅洞村委会",
                code: "010",
            },
            VillageCode {
                name: "辛庄子村委会",
                code: "011",
            },
            VillageCode {
                name: "江米洞村委会",
                code: "012",
            },
            VillageCode {
                name: "行宫村委会",
                code: "013",
            },
            VillageCode {
                name: "东山下村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "镇罗营镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "上镇村委会",
                code: "001",
            },
            VillageCode {
                name: "大庙峪村委会",
                code: "002",
            },
            VillageCode {
                name: "季家沟村委会",
                code: "003",
            },
            VillageCode {
                name: "北四道岭村委会",
                code: "004",
            },
            VillageCode {
                name: "东四道岭村委会",
                code: "005",
            },
            VillageCode {
                name: "下营村委会",
                code: "006",
            },
            VillageCode {
                name: "上营村委会",
                code: "007",
            },
            VillageCode {
                name: "桃园村委会",
                code: "008",
            },
            VillageCode {
                name: "见子庄村委会",
                code: "009",
            },
            VillageCode {
                name: "东牛角峪村委会",
                code: "010",
            },
            VillageCode {
                name: "五里庙村委会",
                code: "011",
            },
            VillageCode {
                name: "西寺峪村委会",
                code: "012",
            },
            VillageCode {
                name: "东寺峪村委会",
                code: "013",
            },
            VillageCode {
                name: "核桃洼村委会",
                code: "014",
            },
            VillageCode {
                name: "关上村委会",
                code: "015",
            },
            VillageCode {
                name: "北水峪村委会",
                code: "016",
            },
            VillageCode {
                name: "清水湖村委会",
                code: "017",
            },
            VillageCode {
                name: "杨家台村委会",
                code: "018",
            },
            VillageCode {
                name: "张家台村委会",
                code: "019",
            },
            VillageCode {
                name: "玻璃台村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "黄松峪乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "黄松峪村委会",
                code: "001",
            },
            VillageCode {
                name: "黑豆峪村委会",
                code: "002",
            },
            VillageCode {
                name: "白云寺村委会",
                code: "003",
            },
            VillageCode {
                name: "大东沟村委会",
                code: "004",
            },
            VillageCode {
                name: "梨树沟村委会",
                code: "005",
            },
            VillageCode {
                name: "塔洼村委会",
                code: "006",
            },
            VillageCode {
                name: "刁窝村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "熊儿寨乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "熊儿寨村委会",
                code: "001",
            },
            VillageCode {
                name: "北土门村委会",
                code: "002",
            },
            VillageCode {
                name: "南岔村委会",
                code: "003",
            },
            VillageCode {
                name: "魏家湾村委会",
                code: "004",
            },
            VillageCode {
                name: "东沟村委会",
                code: "005",
            },
            VillageCode {
                name: "东长峪村委会",
                code: "006",
            },
            VillageCode {
                name: "花峪村委会",
                code: "007",
            },
            VillageCode {
                name: "老泉口村委会",
                code: "008",
            },
        ],
    },
];

static TOWNS_BP_015: [TownCode; 21] = [
    TownCode {
        name: "鼓楼街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "白檀社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "鼓楼社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "鼓楼南区社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "宾阳里社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "宾阳北里社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "宾阳西里社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "北源里社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "东菜园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "行宫社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "石桥社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "沿湖社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "车站路社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "车站路南区社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "檀州家园社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "云秀花园社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "宾阳社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "太扬家园社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "行宫南区社区居民委员会",
                code: "018",
            },
            VillageCode {
                name: "亚澜湾社区居民委员会",
                code: "019",
            },
            VillageCode {
                name: "长安东区社区居民委员会",
                code: "020",
            },
            VillageCode {
                name: "长安西区社区居民委员会",
                code: "021",
            },
            VillageCode {
                name: "檀城东区社区居民委员会",
                code: "022",
            },
            VillageCode {
                name: "檀城西区社区居民委员会",
                code: "023",
            },
            VillageCode {
                name: "花园东区社区居民委员会",
                code: "024",
            },
            VillageCode {
                name: "花园西区社区居民委员会",
                code: "025",
            },
            VillageCode {
                name: "向阳西社区居民委员会",
                code: "026",
            },
            VillageCode {
                name: "阳光社区居民委员会",
                code: "027",
            },
            VillageCode {
                name: "御东园社区居民委员会",
                code: "028",
            },
            VillageCode {
                name: "云北社区居民委员会",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "果园街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "康居社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新北路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "兴云社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "果园西里社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "果园新里社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "果园新里北区社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "密西花园社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "季庄社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "康馨雅苑社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "瑞和园社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "学府花园社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "绿地社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "福荣社区居民委员会",
                code: "013",
            },
            VillageCode {
                name: "嘉益社区居民委员会",
                code: "014",
            },
            VillageCode {
                name: "上河湾社区居民委员会",
                code: "015",
            },
            VillageCode {
                name: "澜悦社区居民委员会",
                code: "016",
            },
            VillageCode {
                name: "清水湾社区居民委员会",
                code: "017",
            },
            VillageCode {
                name: "润博园社区居民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "檀营地区",
        code: "003",
        villages: &[
            VillageCode {
                name: "檀营社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "第一社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "第二社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "密云镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "季庄村民委员会",
                code: "001",
            },
            VillageCode {
                name: "大唐庄村民委员会",
                code: "002",
            },
            VillageCode {
                name: "小唐庄村民委员会",
                code: "003",
            },
            VillageCode {
                name: "王家楼村民委员会",
                code: "004",
            },
            VillageCode {
                name: "西户部庄村民委员会",
                code: "005",
            },
            VillageCode {
                name: "李各庄村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "溪翁庄镇",
        code: "005",
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
                name: "第四社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "第五社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "润溪社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "云溪社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "澜茵山社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "北白岩村民委员会",
                code: "009",
            },
            VillageCode {
                name: "溪翁庄村民委员会",
                code: "010",
            },
            VillageCode {
                name: "金叵罗村民委员会",
                code: "011",
            },
            VillageCode {
                name: "石马峪村民委员会",
                code: "012",
            },
            VillageCode {
                name: "走马庄村民委员会",
                code: "013",
            },
            VillageCode {
                name: "尖岩村民委员会",
                code: "014",
            },
            VillageCode {
                name: "东智东村民委员会",
                code: "015",
            },
            VillageCode {
                name: "东智西村民委员会",
                code: "016",
            },
            VillageCode {
                name: "东智北村民委员会",
                code: "017",
            },
            VillageCode {
                name: "石墙沟村民委员会",
                code: "018",
            },
            VillageCode {
                name: "黑山寺村民委员会",
                code: "019",
            },
            VillageCode {
                name: "立新庄村民委员会",
                code: "020",
            },
            VillageCode {
                name: "白草洼村民委员会",
                code: "021",
            },
            VillageCode {
                name: "东营子村民委员会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "西田各庄镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "西田各庄社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "大辛庄居民委员会",
                code: "002",
            },
            VillageCode {
                name: "沿村居民委员会",
                code: "003",
            },
            VillageCode {
                name: "西田各庄村民委员会",
                code: "004",
            },
            VillageCode {
                name: "董各庄村民委员会",
                code: "005",
            },
            VillageCode {
                name: "仓头村民委员会",
                code: "006",
            },
            VillageCode {
                name: "渤海寨村民委员会",
                code: "007",
            },
            VillageCode {
                name: "水洼屯村民委员会",
                code: "008",
            },
            VillageCode {
                name: "西恒河村民委员会",
                code: "009",
            },
            VillageCode {
                name: "疃里村民委员会",
                code: "010",
            },
            VillageCode {
                name: "沿村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "大辛庄村民委员会",
                code: "012",
            },
            VillageCode {
                name: "西智村民委员会",
                code: "013",
            },
            VillageCode {
                name: "太子务村民委员会",
                code: "014",
            },
            VillageCode {
                name: "东户部庄村民委员会",
                code: "015",
            },
            VillageCode {
                name: "韩各庄村民委员会",
                code: "016",
            },
            VillageCode {
                name: "于家台村民委员会",
                code: "017",
            },
            VillageCode {
                name: "西山村民委员会",
                code: "018",
            },
            VillageCode {
                name: "建新村民委员会",
                code: "019",
            },
            VillageCode {
                name: "朝阳村民委员会",
                code: "020",
            },
            VillageCode {
                name: "卸甲山村民委员会",
                code: "021",
            },
            VillageCode {
                name: "马营村民委员会",
                code: "022",
            },
            VillageCode {
                name: "西康各庄村民委员会",
                code: "023",
            },
            VillageCode {
                name: "西庄户村民委员会",
                code: "024",
            },
            VillageCode {
                name: "西沙地村民委员会",
                code: "025",
            },
            VillageCode {
                name: "小水峪村民委员会",
                code: "026",
            },
            VillageCode {
                name: "兴盛村民委员会",
                code: "027",
            },
            VillageCode {
                name: "牛盆峪村民委员会",
                code: "028",
            },
            VillageCode {
                name: "白道峪村民委员会",
                code: "029",
            },
            VillageCode {
                name: "小石尖村民委员会",
                code: "030",
            },
            VillageCode {
                name: "署地村民委员会",
                code: "031",
            },
            VillageCode {
                name: "新王庄村民委员会",
                code: "032",
            },
            VillageCode {
                name: "青甸村民委员会",
                code: "033",
            },
            VillageCode {
                name: "黄坨子村民委员会",
                code: "034",
            },
            VillageCode {
                name: "坟庄村民委员会",
                code: "035",
            },
            VillageCode {
                name: "龚庄子村民委员会",
                code: "036",
            },
            VillageCode {
                name: "河北庄村民委员会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "十里堡镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "燕落寨社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "明珠社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "王各庄社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "博世庄园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "海怡庄园社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "清水潭村民委员会",
                code: "006",
            },
            VillageCode {
                name: "统军庄村民委员会",
                code: "007",
            },
            VillageCode {
                name: "程家庄村民委员会",
                code: "008",
            },
            VillageCode {
                name: "庄禾屯村民委员会",
                code: "009",
            },
            VillageCode {
                name: "河漕村民委员会",
                code: "010",
            },
            VillageCode {
                name: "十里堡村民委员会",
                code: "011",
            },
            VillageCode {
                name: "靳各寨村民委员会",
                code: "012",
            },
            VillageCode {
                name: "岭东村民委员会",
                code: "013",
            },
            VillageCode {
                name: "双井村民委员会",
                code: "014",
            },
            VillageCode {
                name: "水泉村民委员会",
                code: "015",
            },
            VillageCode {
                name: "杨新庄村民委员会",
                code: "016",
            },
            VillageCode {
                name: "红光村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "河南寨镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "新北区社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新中区社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "新西区社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "平头村民委员会",
                code: "004",
            },
            VillageCode {
                name: "前金沟村民委员会",
                code: "005",
            },
            VillageCode {
                name: "金沟村民委员会",
                code: "006",
            },
            VillageCode {
                name: "两河村民委员会",
                code: "007",
            },
            VillageCode {
                name: "沙坞村民委员会",
                code: "008",
            },
            VillageCode {
                name: "赶河厂村民委员会",
                code: "009",
            },
            VillageCode {
                name: "新兴村民委员会",
                code: "010",
            },
            VillageCode {
                name: "莲花瓣村民委员会",
                code: "011",
            },
            VillageCode {
                name: "钓鱼台村民委员会",
                code: "012",
            },
            VillageCode {
                name: "南单家庄村民委员会",
                code: "013",
            },
            VillageCode {
                name: "台上村民委员会",
                code: "014",
            },
            VillageCode {
                name: "下屯村民委员会",
                code: "015",
            },
            VillageCode {
                name: "南金沟屯村民委员会",
                code: "016",
            },
            VillageCode {
                name: "荆栗园村民委员会",
                code: "017",
            },
            VillageCode {
                name: "团结村民委员会",
                code: "018",
            },
            VillageCode {
                name: "中庄村民委员会",
                code: "019",
            },
            VillageCode {
                name: "套里村民委员会",
                code: "020",
            },
            VillageCode {
                name: "芦古庄村民委员会",
                code: "021",
            },
            VillageCode {
                name: "北金沟屯村民委员会",
                code: "022",
            },
            VillageCode {
                name: "河南寨村民委员会",
                code: "023",
            },
            VillageCode {
                name: "北单家庄村民委员会",
                code: "024",
            },
            VillageCode {
                name: "宁村村民委员会",
                code: "025",
            },
            VillageCode {
                name: "圣水头村民委员会",
                code: "026",
            },
            VillageCode {
                name: "东套里村民委员会",
                code: "027",
            },
            VillageCode {
                name: "东鱼家台村民委员会",
                code: "028",
            },
            VillageCode {
                name: "陈各庄村民委员会",
                code: "029",
            },
            VillageCode {
                name: "提辖庄村民委员会",
                code: "030",
            },
            VillageCode {
                name: "山口庄村民委员会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "巨各庄镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "新生社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "铁矿社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "沙厂社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "豆各庄社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "查子沟村民委员会",
                code: "005",
            },
            VillageCode {
                name: "达峪村民委员会",
                code: "006",
            },
            VillageCode {
                name: "楼峪村民委员会",
                code: "007",
            },
            VillageCode {
                name: "水树峪村民委员会",
                code: "008",
            },
            VillageCode {
                name: "沙厂村民委员会",
                code: "009",
            },
            VillageCode {
                name: "前厂村民委员会",
                code: "010",
            },
            VillageCode {
                name: "牛角峪村民委员会",
                code: "011",
            },
            VillageCode {
                name: "康各庄村民委员会",
                code: "012",
            },
            VillageCode {
                name: "海子村民委员会",
                code: "013",
            },
            VillageCode {
                name: "久远庄村民委员会",
                code: "014",
            },
            VillageCode {
                name: "塘子村民委员会",
                code: "015",
            },
            VillageCode {
                name: "赵家庄村民委员会",
                code: "016",
            },
            VillageCode {
                name: "豆各庄村民委员会",
                code: "017",
            },
            VillageCode {
                name: "巨各庄村民委员会",
                code: "018",
            },
            VillageCode {
                name: "八家庄村民委员会",
                code: "019",
            },
            VillageCode {
                name: "金山子村民委员会",
                code: "020",
            },
            VillageCode {
                name: "张家庄村民委员会",
                code: "021",
            },
            VillageCode {
                name: "霍各庄村民委员会",
                code: "022",
            },
            VillageCode {
                name: "水峪村民委员会",
                code: "023",
            },
            VillageCode {
                name: "黄各庄村民委员会",
                code: "024",
            },
            VillageCode {
                name: "前焦家坞村民委员会",
                code: "025",
            },
            VillageCode {
                name: "塘峪村民委员会",
                code: "026",
            },
            VillageCode {
                name: "后焦家坞村民委员会",
                code: "027",
            },
            VillageCode {
                name: "丰各庄村民委员会",
                code: "028",
            },
            VillageCode {
                name: "东白岩村民委员会",
                code: "029",
            },
            VillageCode {
                name: "蔡家洼村民委员会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "穆家峪镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "穆家峪社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新农村社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "前栗园社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "刘林池村民委员会",
                code: "004",
            },
            VillageCode {
                name: "新农村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "前栗园村民委员会",
                code: "006",
            },
            VillageCode {
                name: "后栗园村民委员会",
                code: "007",
            },
            VillageCode {
                name: "沙峪沟村民委员会",
                code: "008",
            },
            VillageCode {
                name: "大石岭村民委员会",
                code: "009",
            },
            VillageCode {
                name: "水漳村民委员会",
                code: "010",
            },
            VillageCode {
                name: "达峪沟村民委员会",
                code: "011",
            },
            VillageCode {
                name: "荆稍坟村民委员会",
                code: "012",
            },
            VillageCode {
                name: "羊山村民委员会",
                code: "013",
            },
            VillageCode {
                name: "南穆家峪村民委员会",
                code: "014",
            },
            VillageCode {
                name: "北穆家峪回族村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "九松山村民委员会",
                code: "016",
            },
            VillageCode {
                name: "西穆家峪村民委员会",
                code: "017",
            },
            VillageCode {
                name: "阁老峪村民委员会",
                code: "018",
            },
            VillageCode {
                name: "娄子峪村民委员会",
                code: "019",
            },
            VillageCode {
                name: "达岩村民委员会",
                code: "020",
            },
            VillageCode {
                name: "辛安庄村民委员会",
                code: "021",
            },
            VillageCode {
                name: "荆子峪村民委员会",
                code: "022",
            },
            VillageCode {
                name: "上峪村民委员会",
                code: "023",
            },
            VillageCode {
                name: "庄头峪村民委员会",
                code: "024",
            },
            VillageCode {
                name: "碱厂村民委员会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "太师屯镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "太师屯镇居民委员会",
                code: "001",
            },
            VillageCode {
                name: "光明居民委员会",
                code: "002",
            },
            VillageCode {
                name: "正阳居民委员会",
                code: "003",
            },
            VillageCode {
                name: "永安居民委员会",
                code: "004",
            },
            VillageCode {
                name: "北山社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "黄各庄村民委员会",
                code: "006",
            },
            VillageCode {
                name: "许庄子村民委员会",
                code: "007",
            },
            VillageCode {
                name: "流河峪村民委员会",
                code: "008",
            },
            VillageCode {
                name: "前八家庄村民委员会",
                code: "009",
            },
            VillageCode {
                name: "后八家庄村民委员会",
                code: "010",
            },
            VillageCode {
                name: "龙潭沟村民委员会",
                code: "011",
            },
            VillageCode {
                name: "上庄子村民委员会",
                code: "012",
            },
            VillageCode {
                name: "东田各庄村民委员会",
                code: "013",
            },
            VillageCode {
                name: "流河沟村民委员会",
                code: "014",
            },
            VillageCode {
                name: "葡萄园村民委员会",
                code: "015",
            },
            VillageCode {
                name: "太师屯村民委员会",
                code: "016",
            },
            VillageCode {
                name: "太师庄村民委员会",
                code: "017",
            },
            VillageCode {
                name: "上金山村民委员会",
                code: "018",
            },
            VillageCode {
                name: "大漕村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "小漕村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "城子村民委员会",
                code: "021",
            },
            VillageCode {
                name: "松树峪村民委员会",
                code: "022",
            },
            VillageCode {
                name: "东学各庄村民委员会",
                code: "023",
            },
            VillageCode {
                name: "松树掌村民委员会",
                code: "024",
            },
            VillageCode {
                name: "桑园村民委员会",
                code: "025",
            },
            VillageCode {
                name: "黑古沿村民委员会",
                code: "026",
            },
            VillageCode {
                name: "前南台村民委员会",
                code: "027",
            },
            VillageCode {
                name: "后南台村民委员会",
                code: "028",
            },
            VillageCode {
                name: "头道岭村民委员会",
                code: "029",
            },
            VillageCode {
                name: "光明队村民委员会",
                code: "030",
            },
            VillageCode {
                name: "二道河村民委员会",
                code: "031",
            },
            VillageCode {
                name: "车道峪村民委员会",
                code: "032",
            },
            VillageCode {
                name: "沙峪村民委员会",
                code: "033",
            },
            VillageCode {
                name: "令公村民委员会",
                code: "034",
            },
            VillageCode {
                name: "南沟村民委员会",
                code: "035",
            },
            VillageCode {
                name: "石岩井村民委员会",
                code: "036",
            },
            VillageCode {
                name: "东庄禾村民委员会",
                code: "037",
            },
            VillageCode {
                name: "马场村民委员会",
                code: "038",
            },
            VillageCode {
                name: "落洼村民委员会",
                code: "039",
            },
        ],
    },
    TownCode {
        name: "高岭镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "高岭社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "上甸子社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "下会村民委员会",
                code: "003",
            },
            VillageCode {
                name: "辛庄村民委员会",
                code: "004",
            },
            VillageCode {
                name: "放马峪村民委员会",
                code: "005",
            },
            VillageCode {
                name: "高岭村民委员会",
                code: "006",
            },
            VillageCode {
                name: "高岭屯村民委员会",
                code: "007",
            },
            VillageCode {
                name: "白河涧村民委员会",
                code: "008",
            },
            VillageCode {
                name: "瑶亭村民委员会",
                code: "009",
            },
            VillageCode {
                name: "芹菜岭村民委员会",
                code: "010",
            },
            VillageCode {
                name: "东关村民委员会",
                code: "011",
            },
            VillageCode {
                name: "石匣村民委员会",
                code: "012",
            },
            VillageCode {
                name: "大屯村民委员会",
                code: "013",
            },
            VillageCode {
                name: "栗榛寨村民委员会",
                code: "014",
            },
            VillageCode {
                name: "四合村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "小开岭村民委员会",
                code: "016",
            },
            VillageCode {
                name: "大开岭村民委员会",
                code: "017",
            },
            VillageCode {
                name: "上甸子村民委员会",
                code: "018",
            },
            VillageCode {
                name: "下甸子村民委员会",
                code: "019",
            },
            VillageCode {
                name: "下河村民委员会",
                code: "020",
            },
            VillageCode {
                name: "郝家台村民委员会",
                code: "021",
            },
            VillageCode {
                name: "界牌峪村民委员会",
                code: "022",
            },
            VillageCode {
                name: "田庄村民委员会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "不老屯镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "不老屯居民委员会",
                code: "001",
            },
            VillageCode {
                name: "燕落居民委员会",
                code: "002",
            },
            VillageCode {
                name: "杨各庄村民委员会",
                code: "003",
            },
            VillageCode {
                name: "董各庄村民委员会",
                code: "004",
            },
            VillageCode {
                name: "沙峪里村民委员会",
                code: "005",
            },
            VillageCode {
                name: "学艺厂村民委员会",
                code: "006",
            },
            VillageCode {
                name: "转山子村民委员会",
                code: "007",
            },
            VillageCode {
                name: "黄土坎村民委员会",
                code: "008",
            },
            VillageCode {
                name: "燕落村民委员会",
                code: "009",
            },
            VillageCode {
                name: "不老屯村民委员会",
                code: "010",
            },
            VillageCode {
                name: "白土沟村民委员会",
                code: "011",
            },
            VillageCode {
                name: "丑山子村民委员会",
                code: "012",
            },
            VillageCode {
                name: "边庄子村民委员会",
                code: "013",
            },
            VillageCode {
                name: "车道岭村民委员会",
                code: "014",
            },
            VillageCode {
                name: "兵马营村民委员会",
                code: "015",
            },
            VillageCode {
                name: "柳树沟村民委员会",
                code: "016",
            },
            VillageCode {
                name: "大窝铺村民委员会",
                code: "017",
            },
            VillageCode {
                name: "永乐村民委员会",
                code: "018",
            },
            VillageCode {
                name: "西学各庄村民委员会",
                code: "019",
            },
            VillageCode {
                name: "香水峪村民委员会",
                code: "020",
            },
            VillageCode {
                name: "北香峪村民委员会",
                code: "021",
            },
            VillageCode {
                name: "南香峪村民委员会",
                code: "022",
            },
            VillageCode {
                name: "半城子村民委员会",
                code: "023",
            },
            VillageCode {
                name: "史庄子村民委员会",
                code: "024",
            },
            VillageCode {
                name: "古石峪村民委员会",
                code: "025",
            },
            VillageCode {
                name: "陈家峪村民委员会",
                code: "026",
            },
            VillageCode {
                name: "阳坡地村民委员会",
                code: "027",
            },
            VillageCode {
                name: "西坨古村民委员会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "冯家峪镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "冯家峪镇社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "保峪岭村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西庄子村民委员会",
                code: "003",
            },
            VillageCode {
                name: "石洞子村民委员会",
                code: "004",
            },
            VillageCode {
                name: "冯家峪村民委员会",
                code: "005",
            },
            VillageCode {
                name: "西口外村民委员会",
                code: "006",
            },
            VillageCode {
                name: "西白莲峪村民委员会",
                code: "007",
            },
            VillageCode {
                name: "三岔口村民委员会",
                code: "008",
            },
            VillageCode {
                name: "朱家峪村民委员会",
                code: "009",
            },
            VillageCode {
                name: "下营村民委员会",
                code: "010",
            },
            VillageCode {
                name: "白马关村民委员会",
                code: "011",
            },
            VillageCode {
                name: "番字牌村民委员会",
                code: "012",
            },
            VillageCode {
                name: "黄梁根村民委员会",
                code: "013",
            },
            VillageCode {
                name: "西苍峪村民委员会",
                code: "014",
            },
            VillageCode {
                name: "司营子村民委员会",
                code: "015",
            },
            VillageCode {
                name: "前火岭村民委员会",
                code: "016",
            },
            VillageCode {
                name: "石湖根村民委员会",
                code: "017",
            },
            VillageCode {
                name: "北栅子村民委员会",
                code: "018",
            },
            VillageCode {
                name: "南台子村民委员会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "古北口镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "南菜园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "古北口社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北头社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "东山社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "古北口村民委员会",
                code: "005",
            },
            VillageCode {
                name: "河西村民委员会",
                code: "006",
            },
            VillageCode {
                name: "潮关村民委员会",
                code: "007",
            },
            VillageCode {
                name: "杨庄子村民委员会",
                code: "008",
            },
            VillageCode {
                name: "龙洋村民委员会",
                code: "009",
            },
            VillageCode {
                name: "北甸子村民委员会",
                code: "010",
            },
            VillageCode {
                name: "北台村民委员会",
                code: "011",
            },
            VillageCode {
                name: "汤河村民委员会",
                code: "012",
            },
            VillageCode {
                name: "司马台村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "大城子镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "宏城社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "北沟村民委员会",
                code: "002",
            },
            VillageCode {
                name: "梯子峪村民委员会",
                code: "003",
            },
            VillageCode {
                name: "墙子路村民委员会",
                code: "004",
            },
            VillageCode {
                name: "南沟村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下栅子村民委员会",
                code: "006",
            },
            VillageCode {
                name: "苍术会村民委员会",
                code: "007",
            },
            VillageCode {
                name: "柏崖村民委员会",
                code: "008",
            },
            VillageCode {
                name: "程各庄村民委员会",
                code: "009",
            },
            VillageCode {
                name: "庄户峪村民委员会",
                code: "010",
            },
            VillageCode {
                name: "张庄子村民委员会",
                code: "011",
            },
            VillageCode {
                name: "杨各庄村民委员会",
                code: "012",
            },
            VillageCode {
                name: "高庄子村民委员会",
                code: "013",
            },
            VillageCode {
                name: "大城子村民委员会",
                code: "014",
            },
            VillageCode {
                name: "聂家峪村民委员会",
                code: "015",
            },
            VillageCode {
                name: "方耳峪村民委员会",
                code: "016",
            },
            VillageCode {
                name: "后甸村民委员会",
                code: "017",
            },
            VillageCode {
                name: "王各庄村民委员会",
                code: "018",
            },
            VillageCode {
                name: "河下村民委员会",
                code: "019",
            },
            VillageCode {
                name: "庄头村民委员会",
                code: "020",
            },
            VillageCode {
                name: "大龙门村民委员会",
                code: "021",
            },
            VillageCode {
                name: "碰河寺村民委员会",
                code: "022",
            },
            VillageCode {
                name: "张泉村民委员会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "东邵渠镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "东升居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东进居民委员会",
                code: "002",
            },
            VillageCode {
                name: "太保庄村民委员会",
                code: "003",
            },
            VillageCode {
                name: "高各庄村民委员会",
                code: "004",
            },
            VillageCode {
                name: "东邵渠村民委员会",
                code: "005",
            },
            VillageCode {
                name: "石峨村民委员会",
                code: "006",
            },
            VillageCode {
                name: "界牌村民委员会",
                code: "007",
            },
            VillageCode {
                name: "史长峪村民委员会",
                code: "008",
            },
            VillageCode {
                name: "大石门村民委员会",
                code: "009",
            },
            VillageCode {
                name: "西邵渠村民委员会",
                code: "010",
            },
            VillageCode {
                name: "东葫芦峪村民委员会",
                code: "011",
            },
            VillageCode {
                name: "西葫芦峪村民委员会",
                code: "012",
            },
            VillageCode {
                name: "大岭村民委员会",
                code: "013",
            },
            VillageCode {
                name: "小岭村民委员会",
                code: "014",
            },
            VillageCode {
                name: "银冶岭村民委员会",
                code: "015",
            },
            VillageCode {
                name: "南达峪村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "北庄镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "北庄社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "暖泉会村民委员会",
                code: "002",
            },
            VillageCode {
                name: "朱家湾村民委员会",
                code: "003",
            },
            VillageCode {
                name: "抗峪村民委员会",
                code: "004",
            },
            VillageCode {
                name: "大岭村民委员会",
                code: "005",
            },
            VillageCode {
                name: "土门村民委员会",
                code: "006",
            },
            VillageCode {
                name: "苇子峪村民委员会",
                code: "007",
            },
            VillageCode {
                name: "东庄村民委员会",
                code: "008",
            },
            VillageCode {
                name: "干峪沟村民委员会",
                code: "009",
            },
            VillageCode {
                name: "北庄村民委员会",
                code: "010",
            },
            VillageCode {
                name: "营房村民委员会",
                code: "011",
            },
            VillageCode {
                name: "杨家堡村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "新城子镇",
        code: "019",
        villages: &[
            VillageCode {
                name: "新城子镇社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "花园村民委员会",
                code: "002",
            },
            VillageCode {
                name: "大角峪村民委员会",
                code: "003",
            },
            VillageCode {
                name: "曹家路村民委员会",
                code: "004",
            },
            VillageCode {
                name: "蔡家甸村民委员会",
                code: "005",
            },
            VillageCode {
                name: "东沟村民委员会",
                code: "006",
            },
            VillageCode {
                name: "崔家峪村民委员会",
                code: "007",
            },
            VillageCode {
                name: "二道沟村民委员会",
                code: "008",
            },
            VillageCode {
                name: "头道沟村民委员会",
                code: "009",
            },
            VillageCode {
                name: "小口村民委员会",
                code: "010",
            },
            VillageCode {
                name: "遥桥峪村民委员会",
                code: "011",
            },
            VillageCode {
                name: "新城子村民委员会",
                code: "012",
            },
            VillageCode {
                name: "巴各庄村民委员会",
                code: "013",
            },
            VillageCode {
                name: "太古石村民委员会",
                code: "014",
            },
            VillageCode {
                name: "吉家营村民委员会",
                code: "015",
            },
            VillageCode {
                name: "苏家峪村民委员会",
                code: "016",
            },
            VillageCode {
                name: "塔沟村民委员会",
                code: "017",
            },
            VillageCode {
                name: "大树洼村民委员会",
                code: "018",
            },
            VillageCode {
                name: "坡头村民委员会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "石城镇",
        code: "020",
        villages: &[
            VillageCode {
                name: "石城镇社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "梨树沟村民委员会",
                code: "002",
            },
            VillageCode {
                name: "水堡子村民委员会",
                code: "003",
            },
            VillageCode {
                name: "王庄村民委员会",
                code: "004",
            },
            VillageCode {
                name: "石城村民委员会",
                code: "005",
            },
            VillageCode {
                name: "石塘路村民委员会",
                code: "006",
            },
            VillageCode {
                name: "河北村民委员会",
                code: "007",
            },
            VillageCode {
                name: "西湾子村民委员会",
                code: "008",
            },
            VillageCode {
                name: "黄峪口村民委员会",
                code: "009",
            },
            VillageCode {
                name: "捧河岩村民委员会",
                code: "010",
            },
            VillageCode {
                name: "张家坟村民委员会",
                code: "011",
            },
            VillageCode {
                name: "二平台村民委员会",
                code: "012",
            },
            VillageCode {
                name: "贾峪村民委员会",
                code: "013",
            },
            VillageCode {
                name: "四合堂村民委员会",
                code: "014",
            },
            VillageCode {
                name: "红星村民委员会",
                code: "015",
            },
            VillageCode {
                name: "黄土梁村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "中关村科技园区密云园",
        code: "021",
        villages: &[
            VillageCode {
                name: "中心区社区",
                code: "001",
            },
            VillageCode {
                name: "云西区社区",
                code: "002",
            },
            VillageCode {
                name: "商务区社区",
                code: "003",
            },
        ],
    },
];

static TOWNS_BP_016: [TownCode; 18] = [
    TownCode {
        name: "百泉街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "湖南社区居委会",
                code: "001",
            },
            VillageCode {
                name: "燕水佳园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "颖泽洲社区居委会",
                code: "003",
            },
            VillageCode {
                name: "振兴北社区居委会",
                code: "004",
            },
            VillageCode {
                name: "振兴南社区居委会",
                code: "005",
            },
            VillageCode {
                name: "莲花苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "国润家园社区居委会",
                code: "007",
            },
            VillageCode {
                name: "舜泽园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "上都首府家园社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "香水园街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "东外社区居委会",
                code: "001",
            },
            VillageCode {
                name: "川北东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "川北西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "高塔社区居委会",
                code: "004",
            },
            VillageCode {
                name: "恒安社区居委会",
                code: "005",
            },
            VillageCode {
                name: "石河营东社区居委会",
                code: "006",
            },
            VillageCode {
                name: "石河营西社区居委会",
                code: "007",
            },
            VillageCode {
                name: "双路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "泰安社区居委会",
                code: "009",
            },
            VillageCode {
                name: "新兴东社区居委会",
                code: "010",
            },
            VillageCode {
                name: "新兴西社区居委会",
                code: "011",
            },
            VillageCode {
                name: "兴运嘉园社区居委会",
                code: "012",
            },
            VillageCode {
                name: "和润社区居委会",
                code: "013",
            },
            VillageCode {
                name: "庆园社区居委会",
                code: "014",
            },
            VillageCode {
                name: "集贤社区居委会",
                code: "015",
            },
            VillageCode {
                name: "庆隆社区居委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "儒林街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "康安社区居委会",
                code: "001",
            },
            VillageCode {
                name: "永安社区居委会",
                code: "002",
            },
            VillageCode {
                name: "温泉南区东里社区居委会",
                code: "003",
            },
            VillageCode {
                name: "温泉南区西里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "胜芳园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "儒林苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "温泉馨苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "悦安居社区居委会",
                code: "008",
            },
            VillageCode {
                name: "格兰山水二期社区居委会",
                code: "009",
            },
            VillageCode {
                name: "格兰山水二期南区社区居委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "延庆镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "延庆镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "博园雅居东区社区居委会",
                code: "002",
            },
            VillageCode {
                name: "博园雅居西区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "解放街村委会",
                code: "004",
            },
            VillageCode {
                name: "自由街村委会",
                code: "005",
            },
            VillageCode {
                name: "民主街村委会",
                code: "006",
            },
            VillageCode {
                name: "胜利街村委会",
                code: "007",
            },
            VillageCode {
                name: "东关村委会",
                code: "008",
            },
            VillageCode {
                name: "西关村委会",
                code: "009",
            },
            VillageCode {
                name: "北关村委会",
                code: "010",
            },
            VillageCode {
                name: "蒋家堡村委会",
                code: "011",
            },
            VillageCode {
                name: "双营村委会",
                code: "012",
            },
            VillageCode {
                name: "广积屯村委会",
                code: "013",
            },
            VillageCode {
                name: "王泉营村委会",
                code: "014",
            },
            VillageCode {
                name: "司家营村委会",
                code: "015",
            },
            VillageCode {
                name: "百眼泉村委会",
                code: "016",
            },
            VillageCode {
                name: "民主村村委会",
                code: "017",
            },
            VillageCode {
                name: "南辛堡村委会",
                code: "018",
            },
            VillageCode {
                name: "小营村委会",
                code: "019",
            },
            VillageCode {
                name: "石河营村委会",
                code: "020",
            },
            VillageCode {
                name: "莲花池村委会",
                code: "021",
            },
            VillageCode {
                name: "上水磨村委会",
                code: "022",
            },
            VillageCode {
                name: "下水磨村委会",
                code: "023",
            },
            VillageCode {
                name: "王庄村委会",
                code: "024",
            },
            VillageCode {
                name: "三里河村委会",
                code: "025",
            },
            VillageCode {
                name: "赵庄村委会",
                code: "026",
            },
            VillageCode {
                name: "八里庄村委会",
                code: "027",
            },
            VillageCode {
                name: "孟庄村委会",
                code: "028",
            },
            VillageCode {
                name: "老仁庄村委会",
                code: "029",
            },
            VillageCode {
                name: "祁家堡村委会",
                code: "030",
            },
            VillageCode {
                name: "米家堡村委会",
                code: "031",
            },
            VillageCode {
                name: "唐家堡村委会",
                code: "032",
            },
            VillageCode {
                name: "卓家营村委会",
                code: "033",
            },
            VillageCode {
                name: "陶庄村委会",
                code: "034",
            },
            VillageCode {
                name: "鲁庄村委会",
                code: "035",
            },
            VillageCode {
                name: "郎庄村委会",
                code: "036",
            },
            VillageCode {
                name: "张庄村委会",
                code: "037",
            },
            VillageCode {
                name: "西辛庄村委会",
                code: "038",
            },
            VillageCode {
                name: "小河屯村委会",
                code: "039",
            },
            VillageCode {
                name: "付于屯村委会",
                code: "040",
            },
            VillageCode {
                name: "东五里营村委会",
                code: "041",
            },
            VillageCode {
                name: "新白庙村委会",
                code: "042",
            },
            VillageCode {
                name: "东屯村委会",
                code: "043",
            },
            VillageCode {
                name: "中屯村委会",
                code: "044",
            },
            VillageCode {
                name: "西屯村委会",
                code: "045",
            },
            VillageCode {
                name: "西白庙村委会",
                code: "046",
            },
        ],
    },
    TownCode {
        name: "康庄镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "康庄镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "望都佳园社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北玻嘉园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "榆林堡村委会",
                code: "004",
            },
            VillageCode {
                name: "一街村委会",
                code: "005",
            },
            VillageCode {
                name: "二街村委会",
                code: "006",
            },
            VillageCode {
                name: "三街村委会",
                code: "007",
            },
            VillageCode {
                name: "四街村委会",
                code: "008",
            },
            VillageCode {
                name: "刁千营村委会",
                code: "009",
            },
            VillageCode {
                name: "马坊村委会",
                code: "010",
            },
            VillageCode {
                name: "西桑园村委会",
                code: "011",
            },
            VillageCode {
                name: "西红寺村委会",
                code: "012",
            },
            VillageCode {
                name: "郭家堡村委会",
                code: "013",
            },
            VillageCode {
                name: "小北堡村委会",
                code: "014",
            },
            VillageCode {
                name: "大丰营村委会",
                code: "015",
            },
            VillageCode {
                name: "大营村委会",
                code: "016",
            },
            VillageCode {
                name: "火烧营村委会",
                code: "017",
            },
            VillageCode {
                name: "太平庄村委会",
                code: "018",
            },
            VillageCode {
                name: "张老营村委会",
                code: "019",
            },
            VillageCode {
                name: "许家营村委会",
                code: "020",
            },
            VillageCode {
                name: "马营村委会",
                code: "021",
            },
            VillageCode {
                name: "苗家堡村委会",
                code: "022",
            },
            VillageCode {
                name: "刘浩营村委会",
                code: "023",
            },
            VillageCode {
                name: "屯军营村委会",
                code: "024",
            },
            VillageCode {
                name: "小曹营村委会",
                code: "025",
            },
            VillageCode {
                name: "大王庄村委会",
                code: "026",
            },
            VillageCode {
                name: "北曹营村委会",
                code: "027",
            },
            VillageCode {
                name: "南曹营村委会",
                code: "028",
            },
            VillageCode {
                name: "小王庄村委会",
                code: "029",
            },
            VillageCode {
                name: "小丰营村委会",
                code: "030",
            },
            VillageCode {
                name: "东红寺村委会",
                code: "031",
            },
            VillageCode {
                name: "王家堡村委会",
                code: "032",
            },
            VillageCode {
                name: "东官坊村委会",
                code: "033",
            },
            VillageCode {
                name: "大路村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "八达岭镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "八达岭镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "石峡村委会",
                code: "002",
            },
            VillageCode {
                name: "帮水峪村委会",
                code: "003",
            },
            VillageCode {
                name: "里炮村委会",
                code: "004",
            },
            VillageCode {
                name: "外炮村委会",
                code: "005",
            },
            VillageCode {
                name: "营城子村委会",
                code: "006",
            },
            VillageCode {
                name: "东曹营村委会",
                code: "007",
            },
            VillageCode {
                name: "大浮坨村委会",
                code: "008",
            },
            VillageCode {
                name: "小浮坨村委会",
                code: "009",
            },
            VillageCode {
                name: "程家窑村委会",
                code: "010",
            },
            VillageCode {
                name: "岔道村委会",
                code: "011",
            },
            VillageCode {
                name: "西拨子村委会",
                code: "012",
            },
            VillageCode {
                name: "南园村委会",
                code: "013",
            },
            VillageCode {
                name: "东沟村委会",
                code: "014",
            },
            VillageCode {
                name: "石佛寺村委会",
                code: "015",
            },
            VillageCode {
                name: "三堡村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "永宁镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "永宁镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "河湾村委会",
                code: "002",
            },
            VillageCode {
                name: "北沟村委会",
                code: "003",
            },
            VillageCode {
                name: "清泉铺村委会",
                code: "004",
            },
            VillageCode {
                name: "罗家台村委会",
                code: "005",
            },
            VillageCode {
                name: "王家堡村委会",
                code: "006",
            },
            VillageCode {
                name: "水口子村委会",
                code: "007",
            },
            VillageCode {
                name: "偏坡峪村委会",
                code: "008",
            },
            VillageCode {
                name: "二铺村委会",
                code: "009",
            },
            VillageCode {
                name: "营城村委会",
                code: "010",
            },
            VillageCode {
                name: "马蹄湾村委会",
                code: "011",
            },
            VillageCode {
                name: "西山沟村委会",
                code: "012",
            },
            VillageCode {
                name: "永新堡村委会",
                code: "013",
            },
            VillageCode {
                name: "狮子营村委会",
                code: "014",
            },
            VillageCode {
                name: "上磨村委会",
                code: "015",
            },
            VillageCode {
                name: "吴坊营村委会",
                code: "016",
            },
            VillageCode {
                name: "小庄科村委会",
                code: "017",
            },
            VillageCode {
                name: "前平坊村委会",
                code: "018",
            },
            VillageCode {
                name: "孔化营村委会",
                code: "019",
            },
            VillageCode {
                name: "新华营村委会",
                code: "020",
            },
            VillageCode {
                name: "左所屯村委会",
                code: "021",
            },
            VillageCode {
                name: "北关村委会",
                code: "022",
            },
            VillageCode {
                name: "西关村委会",
                code: "023",
            },
            VillageCode {
                name: "小南园村委会",
                code: "024",
            },
            VillageCode {
                name: "盛世营村委会",
                code: "025",
            },
            VillageCode {
                name: "南关村委会",
                code: "026",
            },
            VillageCode {
                name: "太平街村委会",
                code: "027",
            },
            VillageCode {
                name: "利民街村委会",
                code: "028",
            },
            VillageCode {
                name: "和平街村委会",
                code: "029",
            },
            VillageCode {
                name: "阜民街村委会",
                code: "030",
            },
            VillageCode {
                name: "王家山村委会",
                code: "031",
            },
            VillageCode {
                name: "南张庄村委会",
                code: "032",
            },
            VillageCode {
                name: "东灰岭村委会",
                code: "033",
            },
            VillageCode {
                name: "彭家窑村委会",
                code: "034",
            },
            VillageCode {
                name: "西灰岭村委会",
                code: "035",
            },
            VillageCode {
                name: "头司村委会",
                code: "036",
            },
            VillageCode {
                name: "四司村委会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "旧县镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "旧县镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "白草洼村委会",
                code: "002",
            },
            VillageCode {
                name: "三里庄村委会",
                code: "003",
            },
            VillageCode {
                name: "烧窑峪村委会",
                code: "004",
            },
            VillageCode {
                name: "北张庄村委会",
                code: "005",
            },
            VillageCode {
                name: "白羊峪村委会",
                code: "006",
            },
            VillageCode {
                name: "黄峪口村委会",
                code: "007",
            },
            VillageCode {
                name: "白河堡村委会",
                code: "008",
            },
            VillageCode {
                name: "闫家庄村委会",
                code: "009",
            },
            VillageCode {
                name: "耿家营村委会",
                code: "010",
            },
            VillageCode {
                name: "车坊村委会",
                code: "011",
            },
            VillageCode {
                name: "旧县村委会",
                code: "012",
            },
            VillageCode {
                name: "东羊坊村委会",
                code: "013",
            },
            VillageCode {
                name: "米粮屯村委会",
                code: "014",
            },
            VillageCode {
                name: "古城村委会",
                code: "015",
            },
            VillageCode {
                name: "常家营村委会",
                code: "016",
            },
            VillageCode {
                name: "常里营村委会",
                code: "017",
            },
            VillageCode {
                name: "盆窑村委会",
                code: "018",
            },
            VillageCode {
                name: "团山村委会",
                code: "019",
            },
            VillageCode {
                name: "大柏老村委会",
                code: "020",
            },
            VillageCode {
                name: "小柏老村委会",
                code: "021",
            },
            VillageCode {
                name: "西龙湾村委会",
                code: "022",
            },
            VillageCode {
                name: "东龙湾村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "张山营镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "张山营镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "龙聚山庄社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西大庄科村委会",
                code: "003",
            },
            VillageCode {
                name: "佛峪口村委会",
                code: "004",
            },
            VillageCode {
                name: "水峪村委会",
                code: "005",
            },
            VillageCode {
                name: "胡家营村委会",
                code: "006",
            },
            VillageCode {
                name: "姚家营村委会",
                code: "007",
            },
            VillageCode {
                name: "东门营村委会",
                code: "008",
            },
            VillageCode {
                name: "下营村委会",
                code: "009",
            },
            VillageCode {
                name: "西五里营村委会",
                code: "010",
            },
            VillageCode {
                name: "前黑龙庙村委会",
                code: "011",
            },
            VillageCode {
                name: "后黑龙庙村委会",
                code: "012",
            },
            VillageCode {
                name: "西卓家营村委会",
                code: "013",
            },
            VillageCode {
                name: "下芦凤营村委会",
                code: "014",
            },
            VillageCode {
                name: "上芦凤营村委会",
                code: "015",
            },
            VillageCode {
                name: "张山营村委会",
                code: "016",
            },
            VillageCode {
                name: "马庄村委会",
                code: "017",
            },
            VillageCode {
                name: "小河屯村委会",
                code: "018",
            },
            VillageCode {
                name: "上板泉村委会",
                code: "019",
            },
            VillageCode {
                name: "下板泉村委会",
                code: "020",
            },
            VillageCode {
                name: "玉皇庙村委会",
                code: "021",
            },
            VillageCode {
                name: "西羊坊村委会",
                code: "022",
            },
            VillageCode {
                name: "辛家堡村委会",
                code: "023",
            },
            VillageCode {
                name: "丁家堡村委会",
                code: "024",
            },
            VillageCode {
                name: "靳家堡村委会",
                code: "025",
            },
            VillageCode {
                name: "田宋营村委会",
                code: "026",
            },
            VillageCode {
                name: "吴庄村委会",
                code: "027",
            },
            VillageCode {
                name: "龙聚山庄村委会",
                code: "028",
            },
            VillageCode {
                name: "晏家堡村委会",
                code: "029",
            },
            VillageCode {
                name: "中羊坊村委会",
                code: "030",
            },
            VillageCode {
                name: "黄柏寺村委会",
                code: "031",
            },
            VillageCode {
                name: "上郝庄村委会",
                code: "032",
            },
            VillageCode {
                name: "韩郝庄村委会",
                code: "033",
            },
            VillageCode {
                name: "苏庄村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "四海镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "四海镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西沟里村委会",
                code: "002",
            },
            VillageCode {
                name: "西沟外村委会",
                code: "003",
            },
            VillageCode {
                name: "四海村委会",
                code: "004",
            },
            VillageCode {
                name: "椴木沟村委会",
                code: "005",
            },
            VillageCode {
                name: "菜食河村委会",
                code: "006",
            },
            VillageCode {
                name: "海字口村委会",
                code: "007",
            },
            VillageCode {
                name: "岔石口村委会",
                code: "008",
            },
            VillageCode {
                name: "永安堡村委会",
                code: "009",
            },
            VillageCode {
                name: "郭家湾村委会",
                code: "010",
            },
            VillageCode {
                name: "石窑村委会",
                code: "011",
            },
            VillageCode {
                name: "大胜岭村委会",
                code: "012",
            },
            VillageCode {
                name: "南湾村委会",
                code: "013",
            },
            VillageCode {
                name: "黑汉岭村委会",
                code: "014",
            },
            VillageCode {
                name: "大吉祥村委会",
                code: "015",
            },
            VillageCode {
                name: "上花楼村委会",
                code: "016",
            },
            VillageCode {
                name: "王顺沟村委会",
                code: "017",
            },
            VillageCode {
                name: "前山村委会",
                code: "018",
            },
            VillageCode {
                name: "楼梁村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "千家店镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "千家店镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "河口村委会",
                code: "002",
            },
            VillageCode {
                name: "石槽村委会",
                code: "003",
            },
            VillageCode {
                name: "红石湾村委会",
                code: "004",
            },
            VillageCode {
                name: "千家店村委会",
                code: "005",
            },
            VillageCode {
                name: "河南村委会",
                code: "006",
            },
            VillageCode {
                name: "下德龙湾村委会",
                code: "007",
            },
            VillageCode {
                name: "水头村委会",
                code: "008",
            },
            VillageCode {
                name: "大石窑村委会",
                code: "009",
            },
            VillageCode {
                name: "红旗甸村委会",
                code: "010",
            },
            VillageCode {
                name: "六道河村委会",
                code: "011",
            },
            VillageCode {
                name: "大栜树村委会",
                code: "012",
            },
            VillageCode {
                name: "沙梁子村委会",
                code: "013",
            },
            VillageCode {
                name: "四潭沟村委会",
                code: "014",
            },
            VillageCode {
                name: "下湾村委会",
                code: "015",
            },
            VillageCode {
                name: "菜木沟村委会",
                code: "016",
            },
            VillageCode {
                name: "牤牛沟村委会",
                code: "017",
            },
            VillageCode {
                name: "水泉沟村委会",
                code: "018",
            },
            VillageCode {
                name: "花盆村委会",
                code: "019",
            },
            VillageCode {
                name: "平台子村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "沈家营镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "沈家营镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "天成家园北社区居委会",
                code: "002",
            },
            VillageCode {
                name: "天成家园南社区居委会",
                code: "003",
            },
            VillageCode {
                name: "沈家营村委会",
                code: "004",
            },
            VillageCode {
                name: "东王化营村委会",
                code: "005",
            },
            VillageCode {
                name: "冯庄村委会",
                code: "006",
            },
            VillageCode {
                name: "新合营村委会",
                code: "007",
            },
            VillageCode {
                name: "曹官营村委会",
                code: "008",
            },
            VillageCode {
                name: "临河村委会",
                code: "009",
            },
            VillageCode {
                name: "香村营村委会",
                code: "010",
            },
            VillageCode {
                name: "北老君堂村委会",
                code: "011",
            },
            VillageCode {
                name: "兴安堡村委会",
                code: "012",
            },
            VillageCode {
                name: "魏家营村委会",
                code: "013",
            },
            VillageCode {
                name: "连家营村委会",
                code: "014",
            },
            VillageCode {
                name: "前吕庄村委会",
                code: "015",
            },
            VillageCode {
                name: "后吕庄村委会",
                code: "016",
            },
            VillageCode {
                name: "马匹营村委会",
                code: "017",
            },
            VillageCode {
                name: "孙庄村委会",
                code: "018",
            },
            VillageCode {
                name: "下郝庄村委会",
                code: "019",
            },
            VillageCode {
                name: "北梁村委会",
                code: "020",
            },
            VillageCode {
                name: "八里店村委会",
                code: "021",
            },
            VillageCode {
                name: "西王化营村委会",
                code: "022",
            },
            VillageCode {
                name: "河东村委会",
                code: "023",
            },
            VillageCode {
                name: "下花园村委会",
                code: "024",
            },
            VillageCode {
                name: "上花园村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "大榆树镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "大榆树镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "姜家台村委会",
                code: "002",
            },
            VillageCode {
                name: "陈家营村委会",
                code: "003",
            },
            VillageCode {
                name: "杨户庄村委会",
                code: "004",
            },
            VillageCode {
                name: "阜高营村委会",
                code: "005",
            },
            VillageCode {
                name: "奚官营村委会",
                code: "006",
            },
            VillageCode {
                name: "下辛庄村委会",
                code: "007",
            },
            VillageCode {
                name: "上辛庄村委会",
                code: "008",
            },
            VillageCode {
                name: "宗家营村委会",
                code: "009",
            },
            VillageCode {
                name: "大榆树村委会",
                code: "010",
            },
            VillageCode {
                name: "高庙屯村委会",
                code: "011",
            },
            VillageCode {
                name: "刘家堡村委会",
                code: "012",
            },
            VillageCode {
                name: "北红门村委会",
                code: "013",
            },
            VillageCode {
                name: "南红门村委会",
                code: "014",
            },
            VillageCode {
                name: "东桑园村委会",
                code: "015",
            },
            VillageCode {
                name: "大泥河村委会",
                code: "016",
            },
            VillageCode {
                name: "小泥河村委会",
                code: "017",
            },
            VillageCode {
                name: "小张家口村委会",
                code: "018",
            },
            VillageCode {
                name: "下屯村委会",
                code: "019",
            },
            VillageCode {
                name: "东杏园村委会",
                code: "020",
            },
            VillageCode {
                name: "西杏园村委会",
                code: "021",
            },
            VillageCode {
                name: "岳家营村委会",
                code: "022",
            },
            VillageCode {
                name: "簸箕营村委会",
                code: "023",
            },
            VillageCode {
                name: "新宝庄村委会",
                code: "024",
            },
            VillageCode {
                name: "程家营村委会",
                code: "025",
            },
            VillageCode {
                name: "军营村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "井庄镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "井庄镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南老君堂村委会",
                code: "002",
            },
            VillageCode {
                name: "艾官营村委会",
                code: "003",
            },
            VillageCode {
                name: "宝林寺村委会",
                code: "004",
            },
            VillageCode {
                name: "东小营村委会",
                code: "005",
            },
            VillageCode {
                name: "王木营村委会",
                code: "006",
            },
            VillageCode {
                name: "房老营村委会",
                code: "007",
            },
            VillageCode {
                name: "井家庄村委会",
                code: "008",
            },
            VillageCode {
                name: "小胡家营村委会",
                code: "009",
            },
            VillageCode {
                name: "东石河村委会",
                code: "010",
            },
            VillageCode {
                name: "三司村委会",
                code: "011",
            },
            VillageCode {
                name: "二司村委会",
                code: "012",
            },
            VillageCode {
                name: "柳沟村委会",
                code: "013",
            },
            VillageCode {
                name: "果树园村委会",
                code: "014",
            },
            VillageCode {
                name: "王仲营村委会",
                code: "015",
            },
            VillageCode {
                name: "东红山村委会",
                code: "016",
            },
            VillageCode {
                name: "张伍堡村委会",
                code: "017",
            },
            VillageCode {
                name: "西红山村委会",
                code: "018",
            },
            VillageCode {
                name: "八家村委会",
                code: "019",
            },
            VillageCode {
                name: "东沟村委会",
                code: "020",
            },
            VillageCode {
                name: "西二道河村委会",
                code: "021",
            },
            VillageCode {
                name: "窑湾村委会",
                code: "022",
            },
            VillageCode {
                name: "老银庄村委会",
                code: "023",
            },
            VillageCode {
                name: "冯家庙村委会",
                code: "024",
            },
            VillageCode {
                name: "孟家窑村委会",
                code: "025",
            },
            VillageCode {
                name: "曹碾村委会",
                code: "026",
            },
            VillageCode {
                name: "箭杆岭村委会",
                code: "027",
            },
            VillageCode {
                name: "莲花滩村委会",
                code: "028",
            },
            VillageCode {
                name: "门泉石村委会",
                code: "029",
            },
            VillageCode {
                name: "碓臼石村委会",
                code: "030",
            },
            VillageCode {
                name: "北地村委会",
                code: "031",
            },
            VillageCode {
                name: "西三岔村委会",
                code: "032",
            },
        ],
    },
    TownCode {
        name: "大庄科乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "大庄科乡社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东二道河村委会",
                code: "002",
            },
            VillageCode {
                name: "台自沟村委会",
                code: "003",
            },
            VillageCode {
                name: "榆木沟村委会",
                code: "004",
            },
            VillageCode {
                name: "东太平庄村委会",
                code: "005",
            },
            VillageCode {
                name: "黄土梁村委会",
                code: "006",
            },
            VillageCode {
                name: "小庄科村委会",
                code: "007",
            },
            VillageCode {
                name: "里长沟村委会",
                code: "008",
            },
            VillageCode {
                name: "大庄科村委会",
                code: "009",
            },
            VillageCode {
                name: "汉家川河北村委会",
                code: "010",
            },
            VillageCode {
                name: "汉家川河南村委会",
                code: "011",
            },
            VillageCode {
                name: "董家沟村委会",
                code: "012",
            },
            VillageCode {
                name: "慈母川村委会",
                code: "013",
            },
            VillageCode {
                name: "沙门村委会",
                code: "014",
            },
            VillageCode {
                name: "景而沟村委会",
                code: "015",
            },
            VillageCode {
                name: "沙塘沟村委会",
                code: "016",
            },
            VillageCode {
                name: "霹破石村委会",
                code: "017",
            },
            VillageCode {
                name: "铁炉村委会",
                code: "018",
            },
            VillageCode {
                name: "西沙梁村委会",
                code: "019",
            },
            VillageCode {
                name: "瓦庙村委会",
                code: "020",
            },
            VillageCode {
                name: "车岭村委会",
                code: "021",
            },
            VillageCode {
                name: "松树沟村委会",
                code: "022",
            },
            VillageCode {
                name: "暖水面村委会",
                code: "023",
            },
            VillageCode {
                name: "水泉沟村委会",
                code: "024",
            },
            VillageCode {
                name: "旺泉沟村委会",
                code: "025",
            },
            VillageCode {
                name: "龙泉峪村委会",
                code: "026",
            },
            VillageCode {
                name: "香屯村委会",
                code: "027",
            },
            VillageCode {
                name: "东三岔村委会",
                code: "028",
            },
            VillageCode {
                name: "解字石村委会",
                code: "029",
            },
            VillageCode {
                name: "东王庄村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "刘斌堡乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "刘斌堡乡社区居委会",
                code: "001",
            },
            VillageCode {
                name: "刘斌堡村委会",
                code: "002",
            },
            VillageCode {
                name: "大观头村委会",
                code: "003",
            },
            VillageCode {
                name: "周四沟村委会",
                code: "004",
            },
            VillageCode {
                name: "红果寺村委会",
                code: "005",
            },
            VillageCode {
                name: "上虎叫村委会",
                code: "006",
            },
            VillageCode {
                name: "下虎叫村委会",
                code: "007",
            },
            VillageCode {
                name: "营盘村委会",
                code: "008",
            },
            VillageCode {
                name: "营东沟村委会",
                code: "009",
            },
            VillageCode {
                name: "马道梁村委会",
                code: "010",
            },
            VillageCode {
                name: "山西沟村委会",
                code: "011",
            },
            VillageCode {
                name: "山东沟村委会",
                code: "012",
            },
            VillageCode {
                name: "山南沟村委会",
                code: "013",
            },
            VillageCode {
                name: "小观头村委会",
                code: "014",
            },
            VillageCode {
                name: "观西沟村委会",
                code: "015",
            },
            VillageCode {
                name: "姚官岭村委会",
                code: "016",
            },
            VillageCode {
                name: "小吉祥村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "香营乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "香营乡社区居委会",
                code: "001",
            },
            VillageCode {
                name: "屈家窑村委会",
                code: "002",
            },
            VillageCode {
                name: "黑峪口村委会",
                code: "003",
            },
            VillageCode {
                name: "上垙村委会",
                code: "004",
            },
            VillageCode {
                name: "下垙村委会",
                code: "005",
            },
            VillageCode {
                name: "山底下村委会",
                code: "006",
            },
            VillageCode {
                name: "东白庙村委会",
                code: "007",
            },
            VillageCode {
                name: "孟官屯村委会",
                code: "008",
            },
            VillageCode {
                name: "小堡村委会",
                code: "009",
            },
            VillageCode {
                name: "香营村委会",
                code: "010",
            },
            VillageCode {
                name: "新庄堡村委会",
                code: "011",
            },
            VillageCode {
                name: "后所屯村委会",
                code: "012",
            },
            VillageCode {
                name: "里仁堡村委会",
                code: "013",
            },
            VillageCode {
                name: "聂庄村委会",
                code: "014",
            },
            VillageCode {
                name: "庄科村委会",
                code: "015",
            },
            VillageCode {
                name: "高家窑村委会",
                code: "016",
            },
            VillageCode {
                name: "小川村委会",
                code: "017",
            },
            VillageCode {
                name: "八道河村委会",
                code: "018",
            },
            VillageCode {
                name: "南窑村委会",
                code: "019",
            },
            VillageCode {
                name: "三道沟村委会",
                code: "020",
            },
            VillageCode {
                name: "东边村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "珍珠泉乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "珍珠泉乡社区居委会",
                code: "001",
            },
            VillageCode {
                name: "珍珠泉村委会",
                code: "002",
            },
            VillageCode {
                name: "称沟湾村委会",
                code: "003",
            },
            VillageCode {
                name: "庙梁村委会",
                code: "004",
            },
            VillageCode {
                name: "下水沟村委会",
                code: "005",
            },
            VillageCode {
                name: "上水沟村委会",
                code: "006",
            },
            VillageCode {
                name: "下花楼村委会",
                code: "007",
            },
            VillageCode {
                name: "八亩地村委会",
                code: "008",
            },
            VillageCode {
                name: "转山子村委会",
                code: "009",
            },
            VillageCode {
                name: "水泉子村委会",
                code: "010",
            },
            VillageCode {
                name: "双金草村委会",
                code: "011",
            },
            VillageCode {
                name: "小川村委会",
                code: "012",
            },
            VillageCode {
                name: "小铺村委会",
                code: "013",
            },
            VillageCode {
                name: "仓米道村委会",
                code: "014",
            },
            VillageCode {
                name: "南天门村委会",
                code: "015",
            },
            VillageCode {
                name: "桃条沟村委会",
                code: "016",
            },
        ],
    },
];

pub const CITIES_BP: [CityCode; 17] = [
    CityCode {
        name: "省辖市",
        code: "000",
        towns: &[],
    },
    CityCode {
        name: "东城市",
        code: "001",
        towns: &TOWNS_BP_001,
    },
    CityCode {
        name: "西城市",
        code: "002",
        towns: &TOWNS_BP_002,
    },
    CityCode {
        name: "朝阳市",
        code: "003",
        towns: &TOWNS_BP_003,
    },
    CityCode {
        name: "丰台市",
        code: "004",
        towns: &TOWNS_BP_004,
    },
    CityCode {
        name: "石景山市",
        code: "005",
        towns: &TOWNS_BP_005,
    },
    CityCode {
        name: "海淀市",
        code: "006",
        towns: &TOWNS_BP_006,
    },
    CityCode {
        name: "门头沟市",
        code: "007",
        towns: &TOWNS_BP_007,
    },
    CityCode {
        name: "房山市",
        code: "008",
        towns: &TOWNS_BP_008,
    },
    CityCode {
        name: "通州市",
        code: "009",
        towns: &TOWNS_BP_009,
    },
    CityCode {
        name: "顺义市",
        code: "010",
        towns: &TOWNS_BP_010,
    },
    CityCode {
        name: "昌平市",
        code: "011",
        towns: &TOWNS_BP_011,
    },
    CityCode {
        name: "大兴市",
        code: "012",
        towns: &TOWNS_BP_012,
    },
    CityCode {
        name: "怀柔市",
        code: "013",
        towns: &TOWNS_BP_013,
    },
    CityCode {
        name: "平谷市",
        code: "014",
        towns: &TOWNS_BP_014,
    },
    CityCode {
        name: "密云市",
        code: "015",
        towns: &TOWNS_BP_015,
    },
    CityCode {
        name: "延庆市",
        code: "016",
        towns: &TOWNS_BP_016,
    },
];
