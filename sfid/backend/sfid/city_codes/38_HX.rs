use super::{CityCode, TownCode, VillageCode};

static TOWNS_HX_001: [TownCode; 78] = [
    TownCode {
        name: "东街街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "交通巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "甘泉社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "长沙门社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "金安苑社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "饮马桥社区居民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "南街街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "南关社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "西来寺社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "佛城社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "泰安社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "丹霞社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "祁连社区居民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "西街街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "小寺庙社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "西站社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北环路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "新乐社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "北关社区居民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "北街街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "税亭社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东湖社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "王母宫社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "流泉社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "火车站街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "康乐社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "张火路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "下安社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "下安村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "东泉村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "街道直辖村民小组",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "梁家墩镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "梁家墩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "迎恩村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "五号村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "清凉寺村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "四闸村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "三工村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "刘家沟村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "六闸村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "太和村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "六号村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "上秦镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "李家湾村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "付家寨村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "王家墩村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "安里闸村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "八里堡村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "庙儿闸村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上秦村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "徐赵寨村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "高升庵村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "下秦村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "金家湾村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "哈寨子村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "东王堡村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "安家庄村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "缪家堡村村民委员会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "大满镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "柏家沟村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "新华村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新新村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "城西闸村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "石子坝村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "平顺村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "什信村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "马均村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "东闸村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "西闸村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "兰家寨村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "朝元村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "四号村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "紫家寨村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "小堡子村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "朱家庄村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "李家墩村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "汤家什村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "黑城子村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "大沟村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "新庙村村民委员会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "沙井镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "五个墩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上游村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "九闸村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "寺儿沟村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "水磨湾村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下利沟村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东四村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "南湾村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "沙井村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "南沟村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "先锋村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "新民村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "古城村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "小闸村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "三号村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "瞭马墩村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "民兴村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "柳树寨村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "西六村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "小河村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "西二村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "梁家堡村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "坝庙村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "东沟村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "东三村村民委员会",
                code: "025",
            },
            VillageCode {
                name: "东五村村民委员会",
                code: "026",
            },
            VillageCode {
                name: "兴隆村村民委员会",
                code: "027",
            },
            VillageCode {
                name: "双墩子村村民委员会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "乌江镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "平原堡社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "谢家湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "元丰村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "贾家寨村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "敬依村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "乌江村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "管寨村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "东湖村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "平原村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "安镇村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "天乐村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "大湾村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "小湾村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "永丰村村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "甘浚镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "祁连村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "小泉村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "甘浚村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "星光村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "三关村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "头号村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "光明村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "速展村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "工联村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "巴吉村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "晨光村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "谈家洼村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "西洞村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "东寺村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "高家庄村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "中沟村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "毛家湾村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "新墩镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "滨河新区白塔社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "滨河新区滨河社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "滨河新区青松社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "滨河新区五松园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "滨河新区崇文社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "滨河新区南华社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "流泉村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "北关村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "白塔村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "西关村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "青松村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "南华村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "新墩村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "双塔村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "园艺村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "双堡村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "柏闸村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "隋家寺村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "城儿闸村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "花儿村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "南闸村村民委员会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "党寨镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "上寨村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "下寨村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "马站村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "党寨村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "杨家墩村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "花家洼村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "烟墩村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "田家闸村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "中卫村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "雷寨村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "三十里店村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "陈家墩村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "汪家堡村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "廿里堡村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "陈寨村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "宋王寨村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "小寨村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "七号村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "沿沟村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "十号村村民委员会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "碱滩镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "老寺庙社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "普家庄村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "永星村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "古城村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "甲子墩村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "刘家庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "碱滩村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "幸福村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "草湖村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "二坝村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "三坝村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "杨家庄村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "太平村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "野水地村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "老仁坝村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "永定村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "三闸镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "庚名村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "三闸村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "二闸村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "瓦窑村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "高寨村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "天桥村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "韩家墩村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "符家堡村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "杨家寨村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "草原村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "新建村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "红沙窝村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "小满镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "九园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "五星村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "满家庙村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "店子闸村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "王其闸村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "古浪村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "康宁村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "金城村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "石桥村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "中华村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "张家寨村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "甘城村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "黎明村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "杨家闸村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "大柏村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "河南闸村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "小满村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "明永镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "沿河村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "武家闸村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "孙家闸村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "沤波村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "中南村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "明永村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "永和村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "上崖村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下崖村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "夹河村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "燎烟村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "永济村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "长安镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "万家墩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上头闸村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "庄墩村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "上四闸村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "五座桥村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "郭家堡村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "洪信村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "头号村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下二闸村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "前进村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "河满村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "八一村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "南关村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "龙渠乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "三清湾村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "木笼坝村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "龙首村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下堡村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "头闸村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "水源村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "墩源村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "保安村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "新胜村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "什八名村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "白城村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "高庙村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "安阳乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "苗家堡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "明家城村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "毛家寺村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "帖家城村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "郎家城村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "贺家城村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "王阜庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "五一村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "高寺儿村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "金王庄村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "花寨乡",
        code: "021",
        villages: &[
            VillageCode {
                name: "花寨村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "余家城村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "滚家庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "新城村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "滚家城村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "柏杨树村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "西阳村村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "靖安乡",
        code: "022",
        villages: &[
            VillageCode {
                name: "上堡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "新沟村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "靖平村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "靖安村村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "平山湖蒙古族乡",
        code: "023",
        villages: &[
            VillageCode {
                name: "平山湖村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "紫泥泉村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "红泉村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "张掖经济技术开发区",
        code: "024",
        villages: &[VillageCode {
            name: "经济开发区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "红湾寺镇",
        code: "025",
        villages: &[
            VillageCode {
                name: "隆畅社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "红湾社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "裕兴社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "皇城镇",
        code: "026",
        villages: &[
            VillageCode {
                name: "北极村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "北峰村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "北湾村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "红旗村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "向阳村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "营盘村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "大湖滩村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "皇城村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "水关村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "宁昌村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "长方村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "西水滩村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "金子滩村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "西城村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "河东村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "河西村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "东顶村村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "康乐镇",
        code: "027",
        villages: &[
            VillageCode {
                name: "杨哥村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "德合隆村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "康丰村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "赛鼎村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "巴音村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "大草滩村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "红石窝村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "上游村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "隆丰村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "墩台子村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "桦树湾村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "青台子村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "榆木庄村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "马蹄藏族乡",
        code: "028",
        villages: &[
            VillageCode {
                name: "大泉村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东城子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新升村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "南城子村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "石峰村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "圈坡村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "徐家湾村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "荷草村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "二道沟村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "楼庄子村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "正南沟村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "嘉卜斯村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "八一村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "芭蕉湾村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "黄草沟村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "肖家湾村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "横路沟村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "大都麻村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "马蹄村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "药草村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "长岭村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "小寺村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "大坡头村村民委员会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "白银蒙古族乡",
        code: "029",
        villages: &[
            VillageCode {
                name: "白银村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东牛毛村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西牛毛村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "大河乡",
        code: "030",
        villages: &[
            VillageCode {
                name: "光华村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "大滩村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "红湾村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东岭村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "西岭村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "西岔河村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "西河村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "营盘村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "天桥湾村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "松木滩村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "老虎沟村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "大岔村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "白庄子村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "喇嘛湾村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "西柳沟村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "旧寺湾村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "红边子村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "金畅河村村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "明花乡",
        code: "031",
        villages: &[
            VillageCode {
                name: "前滩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "灰泉子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "刺窝泉村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "深井子村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "湖边子村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "贺家墩村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "黄土坡村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "南沟村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "中沙井村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "小海子村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "上井村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "双海子村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "许三湾村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "黄河湾村村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "祁丰蔵族乡",
        code: "032",
        villages: &[
            VillageCode {
                name: "黄草坝村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "甘坝口村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "祁林村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "红山村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "青稞地村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "瓷窑口村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "观山村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "文殊村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "堡子滩村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "祁文村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "腰泉村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "珠龙关村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "陶丰村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "甘肃省绵羊育种场",
        code: "033",
        villages: &[VillageCode {
            name: "绵羊育种场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "张掖宝瓶河牧场",
        code: "034",
        villages: &[VillageCode {
            name: "宝瓶河牧场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "洪水镇",
        code: "035",
        villages: &[
            VillageCode {
                name: "县府街居民委员会",
                code: "001",
            },
            VillageCode {
                name: "团结巷居民委员会",
                code: "002",
            },
            VillageCode {
                name: "东街居民委员会",
                code: "003",
            },
            VillageCode {
                name: "南街居民委员会",
                code: "004",
            },
            VillageCode {
                name: "西街居民委员会",
                code: "005",
            },
            VillageCode {
                name: "北街居民委员会",
                code: "006",
            },
            VillageCode {
                name: "嘉园社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "东圃社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "文昌社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "金山社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "城关村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "八一村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "乐民村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "益民村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "新丰村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "黄青村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "新墩村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "费寨村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "汤庄村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "吴庄村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "刘山村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "戎庄村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "烧房村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "叶官村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "里仁村村民委员会",
                code: "025",
            },
            VillageCode {
                name: "苏庄村村民委员会",
                code: "026",
            },
            VillageCode {
                name: "马庄村村民委员会",
                code: "027",
            },
            VillageCode {
                name: "于庄村村民委员会",
                code: "028",
            },
            VillageCode {
                name: "单庄村村民委员会",
                code: "029",
            },
            VillageCode {
                name: "李尤村村民委员会",
                code: "030",
            },
            VillageCode {
                name: "刘总旗村村民委员会",
                code: "031",
            },
            VillageCode {
                name: "上柴村村民委员会",
                code: "032",
            },
            VillageCode {
                name: "下柴村村民委员会",
                code: "033",
            },
            VillageCode {
                name: "友爱村村民委员会",
                code: "034",
            },
            VillageCode {
                name: "红石湾村村民委员会",
                code: "035",
            },
            VillageCode {
                name: "老号村村民委员会",
                code: "036",
            },
            VillageCode {
                name: "山城村村民委员会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "六坝镇",
        code: "036",
        villages: &[
            VillageCode {
                name: "圆梦苑社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "复兴苑社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "富强苑社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "幸福苑社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "六坝村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "西上坝村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东上坝村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "四堡村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "四坝村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "铨将村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "韩武村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "海潮坝村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "王官村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "五坝村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "五庄村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "赵岗村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "柴庄村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "金山村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "北滩村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "新民村村民委员会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "新天镇",
        code: "037",
        villages: &[
            VillageCode {
                name: "山寨村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "太平村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "马均村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "吴油村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "王什村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "王庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "韩营村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "周陆村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "上姚村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "下姚村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "新天堡村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "许沙村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "吕庄村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "杏园村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "马庄村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "钱寨村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "李寨村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "闫户村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "三寨村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "二寨村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "林山村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "薛寨村村民委员会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "南古镇",
        code: "038",
        villages: &[
            VillageCode {
                name: "城东村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "城南村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "岔家堡村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "左卫村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "马蹄村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "彭刘村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "闫城村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "甘店村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "周庄村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "景会村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "高郝村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "柳谷村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "东朱村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "西朱村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "田庄村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "左卫营村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "王庄村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "杨坊村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "何庄村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "上花园村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "下花园村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "毛城村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "克寨村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "创业村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "黑崖头村村民委员会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "永固镇",
        code: "039",
        villages: &[
            VillageCode {
                name: "八卦村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东街村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "南关村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "姚寨村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "西村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "滕庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "总寨村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "牛顺村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "邓庄村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "杨家树庄村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "三堡镇",
        code: "040",
        villages: &[
            VillageCode {
                name: "库陀村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "团结村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "下二坝村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "展庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "全营村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "陈庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下吾旗村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "徐家寨村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "三堡村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "韩庄村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "易家新庄村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "任官村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "宏寺村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "何家沟村村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "南丰镇",
        code: "041",
        villages: &[
            VillageCode {
                name: "炒面庄村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "秦庄村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "张连庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "胡庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "马营墩村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "边庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "双庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "渠湾村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "永丰村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "杨家圈村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "何庄村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "张家沟湾村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "玉带村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "铁城村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "黑山村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "冰沟村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "民联镇",
        code: "042",
        villages: &[
            VillageCode {
                name: "龙山村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "郭家湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "杨庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "黄庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "高寨村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "河湾村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "屯粮村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "东升村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "复兴村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "下翟寨村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "上翟寨村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "雷台村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "贾西村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "刘信村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "顾寨村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "张明村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "东寨村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "西寨村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "太和村村民委员会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "顺化镇",
        code: "043",
        villages: &[
            VillageCode {
                name: "顺化村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "青松村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "土城村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "油房村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "列四坝村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "旧堡村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "曹营村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "张宋村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "宗家寨村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "上天乐村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "新天乐村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "下天乐村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "松树村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "丰乐镇",
        code: "044",
        villages: &[
            VillageCode {
                name: "武城村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "易家湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "卧马山村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "白庙村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "张满村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "新庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "刘庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "涌泉村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "双营村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "何庄村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "民乐生态工业园区",
        code: "045",
        villages: &[
            VillageCode {
                name: "圆梦苑社区居民委员会生活区",
                code: "001",
            },
            VillageCode {
                name: "复兴苑社区居民委员会生活区",
                code: "002",
            },
            VillageCode {
                name: "富强苑社区居民委员会生活区",
                code: "003",
            },
            VillageCode {
                name: "幸福苑社区居民委员会生活区",
                code: "004",
            },
            VillageCode {
                name: "开发区社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "沙河镇",
        code: "046",
        villages: &[
            VillageCode {
                name: "乐民社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东关街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "沙河街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "颐和社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "惠民社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "东寨村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "西寨村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "化音村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "兰家堡村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "沙河村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "西关村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "五三村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "何强村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "花园村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "闸湾村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "新丰村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "西头号村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "新民村村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "新华镇",
        code: "047",
        villages: &[
            VillageCode {
                name: "大寨村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "长庄村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "富强村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "胜利村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "宣威村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "西街村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "向前村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "新柳村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "亢寨村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "新华村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "明泉村村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "蓼泉镇",
        code: "048",
        villages: &[
            VillageCode {
                name: "唐湾村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "墩子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "湾子村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "蓼泉村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "寨子村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "新添村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "双泉村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下庄村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "平川镇",
        code: "049",
        villages: &[
            VillageCode {
                name: "黄家堡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "五里墩村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "一工程村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "平川村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "三一村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "三二村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "三三村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "芦湾村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "四坝村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "贾家墩村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "板桥镇",
        code: "050",
        villages: &[
            VillageCode {
                name: "土桥村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "红沟村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "友好村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "古城村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "板桥村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "西湾村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东柳村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "西柳村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "壕洼村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "鸭暖镇",
        code: "051",
        villages: &[
            VillageCode {
                name: "小鸭村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "张湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "昭武村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "大鸭村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "暖泉村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "五泉村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "华强村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "白寨村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "曹庄村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "小屯村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "古寨村村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "倪家营镇",
        code: "052",
        villages: &[
            VillageCode {
                name: "梨园村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "南台村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "高庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "马郡村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "汪家墩村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "倪家营村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下营村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "黄家湾村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "国营临泽农场",
        code: "053",
        villages: &[VillageCode {
            name: "农村虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "五泉林场",
        code: "054",
        villages: &[VillageCode {
            name: "五泉林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "沙河林场",
        code: "055",
        villages: &[VillageCode {
            name: "沙河林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "小泉子治沙站",
        code: "056",
        villages: &[VillageCode {
            name: "小泉子治沙站虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "园艺场",
        code: "057",
        villages: &[VillageCode {
            name: "园艺场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "良种繁殖场",
        code: "058",
        villages: &[VillageCode {
            name: "良种场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "城关镇",
        code: "059",
        villages: &[
            VillageCode {
                name: "新建东村居民委员会",
                code: "001",
            },
            VillageCode {
                name: "人民东路居民委员会",
                code: "002",
            },
            VillageCode {
                name: "医院西路居民委员会",
                code: "003",
            },
            VillageCode {
                name: "人民西路居民委员会",
                code: "004",
            },
            VillageCode {
                name: "长征路居民委员会",
                code: "005",
            },
            VillageCode {
                name: "新建南村居民委员会",
                code: "006",
            },
            VillageCode {
                name: "滨河居民委员会",
                code: "007",
            },
            VillageCode {
                name: "东苑居民委员会",
                code: "008",
            },
            VillageCode {
                name: "国庆村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "宣化镇",
        code: "060",
        villages: &[
            VillageCode {
                name: "利丰村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "利号村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "贞号村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "高桥村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "寨子村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "站南村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "站北村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "蒋家庄村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "宣化村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "台子寺村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "王马湾村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "乐一村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "乐二村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "乐三村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "朱家堡村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "上庄村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "南华镇",
        code: "061",
        villages: &[
            VillageCode {
                name: "南苑社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "小海子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "墩仁村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "大庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "南寨子村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "义和村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "先锋村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "智号村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "礼号村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "胜利村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "信号村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "南岔村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "成号村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "明水村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "明永村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "永进村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "巷道镇",
        code: "062",
        villages: &[
            VillageCode {
                name: "渠口村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "果园村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "八一村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "三桥村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "巷道村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "东湾村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "南湾村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "槐树村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "沙坡村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "高地村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "小寺村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "五里墩村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "西八里村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "王家村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "东联村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "红联村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "太安村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "殷家庄村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "元号村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "元兴村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "元丰村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "正远村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "殷家桥村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "亨号村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "利沟村村民委员会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "合黎镇",
        code: "063",
        villages: &[
            VillageCode {
                name: "五一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "五二村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "五三村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "五四村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "六一村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "六二村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "六三村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "六四村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "七坝村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "八坝村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "骆驼城镇",
        code: "064",
        villages: &[
            VillageCode {
                name: "碱泉子村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "梧桐村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新联村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "红新村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "团结村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "建康村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "永胜村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "前进村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "果树村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "新民村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "新建村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "骆驼城村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "西滩村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "新坝镇",
        code: "065",
        villages: &[
            VillageCode {
                name: "暖泉村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "新沟村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "顺德村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下坝村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "照中村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "照二村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "照一村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "元山子村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "小坝村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "上坝村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "楼庄村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "西庄子村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "新生村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "官沟村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "曙光村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "许三湾村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "红沙河村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "光明村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "小泉村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "边沟村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "红崖子村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "霞光村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "西上村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "东上村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "和平村村民委员会",
                code: "025",
            },
            VillageCode {
                name: "西大村村民委员会",
                code: "026",
            },
            VillageCode {
                name: "东大村村民委员会",
                code: "027",
            },
            VillageCode {
                name: "古城村村民委员会",
                code: "028",
            },
            VillageCode {
                name: "六洋村村民委员会",
                code: "029",
            },
            VillageCode {
                name: "黄蒿村村民委员会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "黑泉镇",
        code: "066",
        villages: &[
            VillageCode {
                name: "定安村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "定平村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "向阳村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "永丰村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "新开村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "黑泉村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "小坝村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "沙沟村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "镇江村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "九坝村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "十坝村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "胭脂堡村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "罗城镇",
        code: "067",
        villages: &[
            VillageCode {
                name: "张墩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "花墙子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "河西村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "常丰村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "天城村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "侯庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "万丰村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "罗城村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "红山村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "桥儿湾村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "盐池村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "双丰村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "甘肃高台工业园区",
        code: "068",
        villages: &[VillageCode {
            name: "高台工业园区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "清泉镇",
        code: "069",
        villages: &[
            VillageCode {
                name: "长城社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "北街社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "南街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "文化街社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "县府街社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "新城社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "世博社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "永宁社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "和盛社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "滨河社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "博兴社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "焉支社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "北滩村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "东街村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "西街村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "南关村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "南湖村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "南湾村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "双桥村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "清泉村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "祁店村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "拾号村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "北湾村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "郑庄村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "郇庄村村民委员会",
                code: "025",
            },
            VillageCode {
                name: "城北村村民委员会",
                code: "026",
            },
            VillageCode {
                name: "红寺湖村村民委员会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "位奇镇",
        code: "070",
        villages: &[
            VillageCode {
                name: "位奇村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "十里堡村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "二十里堡村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "高寨村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "四坝村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "永兴村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "马寨村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "张湾村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "孙家营村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "暖泉村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "朱湾村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "柳荫村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "芦堡村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "汪庄村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "新开村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "侯山村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "霍城镇",
        code: "071",
        villages: &[
            VillageCode {
                name: "下西山村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上西山村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "双湖村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "沙沟村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "东关村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "西关村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "周庄村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "王庄村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "西坡村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "下河西村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "上河西村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "刘庄村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "杜庄村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "泉头村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "东山村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "陈户镇",
        code: "072",
        villages: &[
            VillageCode {
                name: "三十里堡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "刘伏村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西门村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东门村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "沙河湾村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "王城村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "张庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "岸头村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "陈户村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "周坑村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "盘山村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "寺沟村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "范营村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "孙营村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "山湾村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "焉支村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "大马营镇",
        code: "073",
        villages: &[
            VillageCode {
                name: "马营村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "磨湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新墩村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "夹河村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "圈沟村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "窑坡村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "双泉村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "前山村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "新泉村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "楼庄村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "城南村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "花寨村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "高湖村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "上山湾村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "上河村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "中河村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "下河村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "东乐镇",
        code: "074",
        villages: &[
            VillageCode {
                name: "山羊堡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "西屯村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "城西村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "城东村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "大寨村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "小寨村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "五墩村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "十里堡村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "大桥村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "静安村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "老军乡",
        code: "075",
        villages: &[
            VillageCode {
                name: "焦湾村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "老军村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "祝庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "孙庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "李泉村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "潘庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "郭泉村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "羊虎沟村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "硖口村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "丰城村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "李桥乡",
        code: "076",
        villages: &[
            VillageCode {
                name: "杨坝村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "高庙村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "吴宁村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "巴寨村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "河湾村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下寨村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上寨村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "周庄村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "东沟村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "西沟村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "国营山丹农场",
        code: "077",
        villages: &[VillageCode {
            name: "山丹农场虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "中牧公司山丹马场",
        code: "078",
        villages: &[
            VillageCode {
                name: "马场一场居委会",
                code: "001",
            },
            VillageCode {
                name: "马场二场居委会",
                code: "002",
            },
            VillageCode {
                name: "马场三场居委会",
                code: "003",
            },
            VillageCode {
                name: "马场四场居委会",
                code: "004",
            },
            VillageCode {
                name: "总场居委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_HX_002: [TownCode; 19] = [
    TownCode {
        name: "观音桥镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "观音村民委员会",
                code: "001",
            },
            VillageCode {
                name: "石旁村民委员会",
                code: "002",
            },
            VillageCode {
                name: "斯滔村民委员会",
                code: "003",
            },
            VillageCode {
                name: "麦斯卡村民委员会",
                code: "004",
            },
            VillageCode {
                name: "松都村民委员会",
                code: "005",
            },
            VillageCode {
                name: "麦地沟村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "安宁镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "安宁村民委员会",
                code: "001",
            },
            VillageCode {
                name: "莫莫扎村民委员会",
                code: "002",
            },
            VillageCode {
                name: "八角碉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "炭厂沟村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "勒乌镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "城北社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "城南社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "沐林社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "龙河村民委员会",
                code: "004",
            },
            VillageCode {
                name: "安顺村民委员会",
                code: "005",
            },
            VillageCode {
                name: "八步里村民委员会",
                code: "006",
            },
            VillageCode {
                name: "角木牛村民委员会",
                code: "007",
            },
            VillageCode {
                name: "前锋村民委员会",
                code: "008",
            },
            VillageCode {
                name: "二甲村民委员会",
                code: "009",
            },
            VillageCode {
                name: "新开宗村民委员会",
                code: "010",
            },
            VillageCode {
                name: "勒乌村民委员会",
                code: "011",
            },
            VillageCode {
                name: "金马坪村民委员会",
                code: "012",
            },
            VillageCode {
                name: "牧场村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "马奈镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "八角塘村民委员会",
                code: "001",
            },
            VillageCode {
                name: "独足沟村民委员会",
                code: "002",
            },
            VillageCode {
                name: "白纳溪村民委员会",
                code: "003",
            },
            VillageCode {
                name: "马奈村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "沙耳乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "丹扎木村民委员会",
                code: "001",
            },
            VillageCode {
                name: "山埂子村民委员会",
                code: "002",
            },
            VillageCode {
                name: "克尔玛村民委员会",
                code: "003",
            },
            VillageCode {
                name: "沙耳尼村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "庆宁乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "团结村民委员会",
                code: "001",
            },
            VillageCode {
                name: "庆宁村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新沙村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "咯尔乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "金江村民委员会",
                code: "001",
            },
            VillageCode {
                name: "复兴村民委员会",
                code: "002",
            },
            VillageCode {
                name: "德胜村民委员会",
                code: "003",
            },
            VillageCode {
                name: "五甲村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "河东乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "八字口村民委员会",
                code: "001",
            },
            VillageCode {
                name: "结木村民委员会",
                code: "002",
            },
            VillageCode {
                name: "旦甲木足村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "河西乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "甲咱村民委员会",
                code: "001",
            },
            VillageCode {
                name: "马道村民委员会",
                code: "002",
            },
            VillageCode {
                name: "乃当村民委员会",
                code: "003",
            },
            VillageCode {
                name: "杨家湾村民委员会",
                code: "004",
            },
            VillageCode {
                name: "木居里村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "集沐乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "周山村民委员会",
                code: "001",
            },
            VillageCode {
                name: "雅京村民委员会",
                code: "002",
            },
            VillageCode {
                name: "根扎村民委员会",
                code: "003",
            },
            VillageCode {
                name: "业隆村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "撒瓦脚乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "甲布脚村民委员会",
                code: "001",
            },
            VillageCode {
                name: "木赤村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "卡拉脚乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "布基村民委员会",
                code: "001",
            },
            VillageCode {
                name: "玛目都村民委员会",
                code: "002",
            },
            VillageCode {
                name: "二普鲁村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "俄热乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "科山村民委员会",
                code: "001",
            },
            VillageCode {
                name: "依斗村民委员会",
                code: "002",
            },
            VillageCode {
                name: "二楷村民委员会",
                code: "003",
            },
            VillageCode {
                name: "嘎斯都村民委员会",
                code: "004",
            },
            VillageCode {
                name: "马尼柯村民委员会",
                code: "005",
            },
            VillageCode {
                name: "英布汝村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "二嘎里乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "二嘎里村民委员会",
                code: "001",
            },
            VillageCode {
                name: "查拉沟村民委员会",
                code: "002",
            },
            VillageCode {
                name: "四甲壁村民委员会",
                code: "003",
            },
            VillageCode {
                name: "雅夏村民委员会",
                code: "004",
            },
            VillageCode {
                name: "白塔村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "阿科里乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "铁基斯果尔村民委员会",
                code: "001",
            },
            VillageCode {
                name: "阿科里村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "卡撒乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "脚姆塘村民委员会",
                code: "001",
            },
            VillageCode {
                name: "色尔岭村民委员会",
                code: "002",
            },
            VillageCode {
                name: "巴拉塘村民委员会",
                code: "003",
            },
            VillageCode {
                name: "马厂村民委员会",
                code: "004",
            },
            VillageCode {
                name: "猫碉村民委员会",
                code: "005",
            },
            VillageCode {
                name: "三埂子村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "曾达乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "曾达村民委员会",
                code: "001",
            },
            VillageCode {
                name: "木尔都村民委员会",
                code: "002",
            },
            VillageCode {
                name: "坛罐窑村民委员会",
                code: "003",
            },
            VillageCode {
                name: "倪家坪村民委员会",
                code: "004",
            },
            VillageCode {
                name: "大沟村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "独松乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "卡拉塘村民委员会",
                code: "001",
            },
            VillageCode {
                name: "正里塘村民委员会",
                code: "002",
            },
            VillageCode {
                name: "嘎伍岭村民委员会",
                code: "003",
            },
            VillageCode {
                name: "卡苏村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "毛日乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "热它村民委员会",
                code: "001",
            },
            VillageCode {
                name: "壳它村民委员会",
                code: "002",
            },
            VillageCode {
                name: "毛日村民委员会",
                code: "003",
            },
            VillageCode {
                name: "甲克村民委员会",
                code: "004",
            },
            VillageCode {
                name: "七一村民委员会",
                code: "005",
            },
        ],
    },
];

static TOWNS_HX_003: [TownCode; 10] = [
    TownCode {
        name: "城关镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "永福苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "昌康苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "天锦苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "宝河苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "昌宁苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "北景苑社区居委会",
                code: "006",
            },
            VillageCode {
                name: "天绣苑社区居委会",
                code: "007",
            },
            VillageCode {
                name: "西岚苑社区居委会",
                code: "008",
            },
            VillageCode {
                name: "南龙苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "宝山苑社区居委会",
                code: "010",
            },
            VillageCode {
                name: "赵家庄村委会",
                code: "011",
            },
            VillageCode {
                name: "黄家学村委会",
                code: "012",
            },
            VillageCode {
                name: "沙沟岔村委会",
                code: "013",
            },
            VillageCode {
                name: "中庄子村委会",
                code: "014",
            },
            VillageCode {
                name: "大坝村村委会",
                code: "015",
            },
            VillageCode {
                name: "小坝村村委会",
                code: "016",
            },
            VillageCode {
                name: "北海子村委会",
                code: "017",
            },
            VillageCode {
                name: "金川东村委会",
                code: "018",
            },
            VillageCode {
                name: "金川西村委会",
                code: "019",
            },
            VillageCode {
                name: "直峡山村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "河西堡镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "车站路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "玉河路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "永河路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "昌河路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "银河路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "金河路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "金昌路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "河西堡村委会",
                code: "008",
            },
            VillageCode {
                name: "沙窝村委会",
                code: "009",
            },
            VillageCode {
                name: "鸳鸯池村委会",
                code: "010",
            },
            VillageCode {
                name: "河东堡村委会",
                code: "011",
            },
            VillageCode {
                name: "西庄子村委会",
                code: "012",
            },
            VillageCode {
                name: "下洼子村委会",
                code: "013",
            },
            VillageCode {
                name: "宗家庄村委会",
                code: "014",
            },
            VillageCode {
                name: "黄家泉村委会",
                code: "015",
            },
            VillageCode {
                name: "上三庄村委会",
                code: "016",
            },
            VillageCode {
                name: "青山堡村委会",
                code: "017",
            },
            VillageCode {
                name: "大寨子村委会",
                code: "018",
            },
            VillageCode {
                name: "寺门村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "新城子镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "赵定庄村委会",
                code: "001",
            },
            VillageCode {
                name: "刘克庄村委会",
                code: "002",
            },
            VillageCode {
                name: "塔儿湾村委会",
                code: "003",
            },
            VillageCode {
                name: "农林场村委会",
                code: "004",
            },
            VillageCode {
                name: "毛家庄村委会",
                code: "005",
            },
            VillageCode {
                name: "兆田村委会",
                code: "006",
            },
            VillageCode {
                name: "西湾村委会",
                code: "007",
            },
            VillageCode {
                name: "马营沟村委会",
                code: "008",
            },
            VillageCode {
                name: "唐家坡村委会",
                code: "009",
            },
            VillageCode {
                name: "新城子村委会",
                code: "010",
            },
            VillageCode {
                name: "邵家庄村委会",
                code: "011",
            },
            VillageCode {
                name: "通信堡村委会",
                code: "012",
            },
            VillageCode {
                name: "南湾村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "朱王堡镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "头沟村委会",
                code: "001",
            },
            VillageCode {
                name: "三沟村委会",
                code: "002",
            },
            VillageCode {
                name: "梅南村委会",
                code: "003",
            },
            VillageCode {
                name: "新堡子村委会",
                code: "004",
            },
            VillageCode {
                name: "朱王堡村委会",
                code: "005",
            },
            VillageCode {
                name: "汤宁村委会",
                code: "006",
            },
            VillageCode {
                name: "郑家堡村委会",
                code: "007",
            },
            VillageCode {
                name: "董家堡村委会",
                code: "008",
            },
            VillageCode {
                name: "下汤村委会",
                code: "009",
            },
            VillageCode {
                name: "流泉村委会",
                code: "010",
            },
            VillageCode {
                name: "刘正村委会",
                code: "011",
            },
            VillageCode {
                name: "陈仓村委会",
                code: "012",
            },
            VillageCode {
                name: "梅北村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "东寨镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "头坝村委会",
                code: "001",
            },
            VillageCode {
                name: "永丰村委会",
                code: "002",
            },
            VillageCode {
                name: "上三坝村委会",
                code: "003",
            },
            VillageCode {
                name: "二坝村委会",
                code: "004",
            },
            VillageCode {
                name: "下二坝村委会",
                code: "005",
            },
            VillageCode {
                name: "新二坝村委会",
                code: "006",
            },
            VillageCode {
                name: "上四坝村委会",
                code: "007",
            },
            VillageCode {
                name: "双桥村委会",
                code: "008",
            },
            VillageCode {
                name: "下三坝村委会",
                code: "009",
            },
            VillageCode {
                name: "龙口村委会",
                code: "010",
            },
            VillageCode {
                name: "下四坝村委会",
                code: "011",
            },
            VillageCode {
                name: "红光新村村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "水源镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "杜家寨村委会",
                code: "001",
            },
            VillageCode {
                name: "胜利村委会",
                code: "002",
            },
            VillageCode {
                name: "宋家沟村委会",
                code: "003",
            },
            VillageCode {
                name: "华家沟村委会",
                code: "004",
            },
            VillageCode {
                name: "方沟村委会",
                code: "005",
            },
            VillageCode {
                name: "永宁村委会",
                code: "006",
            },
            VillageCode {
                name: "北地村委会",
                code: "007",
            },
            VillageCode {
                name: "东沟村委会",
                code: "008",
            },
            VillageCode {
                name: "赵沟村委会",
                code: "009",
            },
            VillageCode {
                name: "新沟村委会",
                code: "010",
            },
            VillageCode {
                name: "西沟村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "红山窑镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "土沟村委会",
                code: "001",
            },
            VillageCode {
                name: "高古城村委会",
                code: "002",
            },
            VillageCode {
                name: "河沿子村委会",
                code: "003",
            },
            VillageCode {
                name: "姚家寨村委会",
                code: "004",
            },
            VillageCode {
                name: "夹河村委会",
                code: "005",
            },
            VillageCode {
                name: "红山窑村委会",
                code: "006",
            },
            VillageCode {
                name: "山头村委会",
                code: "007",
            },
            VillageCode {
                name: "水泉子村委会",
                code: "008",
            },
            VillageCode {
                name: "永胜村委会",
                code: "009",
            },
            VillageCode {
                name: "王信堡村委会",
                code: "010",
            },
            VillageCode {
                name: "马家坪村委会",
                code: "011",
            },
            VillageCode {
                name: "毛卜喇村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "焦家庄镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "红庙墩村委会",
                code: "001",
            },
            VillageCode {
                name: "北泉村委会",
                code: "002",
            },
            VillageCode {
                name: "双磨街村委会",
                code: "003",
            },
            VillageCode {
                name: "河滩村委会",
                code: "004",
            },
            VillageCode {
                name: "杏树庄村委会",
                code: "005",
            },
            VillageCode {
                name: "梅家寺村委会",
                code: "006",
            },
            VillageCode {
                name: "南沿沟村委会",
                code: "007",
            },
            VillageCode {
                name: "水磨关村委会",
                code: "008",
            },
            VillageCode {
                name: "焦家庄村委会",
                code: "009",
            },
            VillageCode {
                name: "楼庄子村委会",
                code: "010",
            },
            VillageCode {
                name: "骊靬村委会",
                code: "011",
            },
            VillageCode {
                name: "陈家寨村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "六坝镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "玉宝村委会",
                code: "001",
            },
            VillageCode {
                name: "五坝村委会",
                code: "002",
            },
            VillageCode {
                name: "上六坝村委会",
                code: "003",
            },
            VillageCode {
                name: "七坝村委会",
                code: "004",
            },
            VillageCode {
                name: "六坝村委会",
                code: "005",
            },
            VillageCode {
                name: "星海村委会",
                code: "006",
            },
            VillageCode {
                name: "下七坝村委会",
                code: "007",
            },
            VillageCode {
                name: "南庄村委会",
                code: "008",
            },
            VillageCode {
                name: "八坝村委会",
                code: "009",
            },
            VillageCode {
                name: "下排村委会",
                code: "010",
            },
            VillageCode {
                name: "九坝村委会",
                code: "011",
            },
            VillageCode {
                name: "团庄村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "南坝乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "祁庄村委会",
                code: "001",
            },
            VillageCode {
                name: "永安村委会",
                code: "002",
            },
            VillageCode {
                name: "永丰村委会",
                code: "003",
            },
            VillageCode {
                name: "西校村委会",
                code: "004",
            },
            VillageCode {
                name: "何家湾村委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_HX_004: [TownCode; 50] = [
    TownCode {
        name: "东大街街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "南苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大众社区居委会",
                code: "002",
            },
            VillageCode {
                name: "文庙路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "会馆巷社区居委会",
                code: "004",
            },
            VillageCode {
                name: "杨府巷社区居委会",
                code: "005",
            },
            VillageCode {
                name: "古钟楼社区居委会",
                code: "006",
            },
            VillageCode {
                name: "雷台社区居委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "西大街街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "达府社区居委会",
                code: "001",
            },
            VillageCode {
                name: "雨亭巷社区居委会",
                code: "002",
            },
            VillageCode {
                name: "靶场社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东小井社区居委会",
                code: "004",
            },
            VillageCode {
                name: "仓巷社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "东关街街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "福利路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "富民社区居委会",
                code: "002",
            },
            VillageCode {
                name: "寺巷子社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东关花园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "丹阳社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "西关街街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "科技巷社区居委会",
                code: "001",
            },
            VillageCode {
                name: "体育路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "皇台社区居委会",
                code: "004",
            },
            VillageCode {
                name: "九条岭矿区社区居委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "火车站街街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "火车站社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大新社区居委会",
                code: "002",
            },
            VillageCode {
                name: "惠民社区居委会",
                code: "003",
            },
            VillageCode {
                name: "朝阳社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "地质新村街街道",
        code: "006",
        villages: &[VillageCode {
            name: "地质新村虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "荣华街街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "荣华社区居委会",
                code: "001",
            },
            VillageCode {
                name: "惠泽社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新关村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "宣武街街道",
        code: "008",
        villages: &[
            VillageCode {
                name: "惠政社区居委会",
                code: "001",
            },
            VillageCode {
                name: "姑臧社区居委会",
                code: "002",
            },
            VillageCode {
                name: "银武社区居委会",
                code: "003",
            },
            VillageCode {
                name: "宋家园社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "黄羊河街道",
        code: "009",
        villages: &[VillageCode {
            name: "黄羊河新华虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "黄羊镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "岔路口社区居委会",
                code: "001",
            },
            VillageCode {
                name: "新街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "科教园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新店社区居委会",
                code: "004",
            },
            VillageCode {
                name: "广场社区居委会",
                code: "005",
            },
            VillageCode {
                name: "黄羊村委会",
                code: "006",
            },
            VillageCode {
                name: "广场村委会",
                code: "007",
            },
            VillageCode {
                name: "横沟村委会",
                code: "008",
            },
            VillageCode {
                name: "土塔村委会",
                code: "009",
            },
            VillageCode {
                name: "长丰村委会",
                code: "010",
            },
            VillageCode {
                name: "三河村委会",
                code: "011",
            },
            VillageCode {
                name: "渠中村委会",
                code: "012",
            },
            VillageCode {
                name: "七里村委会",
                code: "013",
            },
            VillageCode {
                name: "西河村委会",
                code: "014",
            },
            VillageCode {
                name: "中腰村委会",
                code: "015",
            },
            VillageCode {
                name: "李宽寨村委会",
                code: "016",
            },
            VillageCode {
                name: "唐沟村委会",
                code: "017",
            },
            VillageCode {
                name: "大墩村委会",
                code: "018",
            },
            VillageCode {
                name: "新店村委会",
                code: "019",
            },
            VillageCode {
                name: "峡沟村委会",
                code: "020",
            },
            VillageCode {
                name: "平沟村委会",
                code: "021",
            },
            VillageCode {
                name: "天桥村委会",
                code: "022",
            },
            VillageCode {
                name: "严庄村委会",
                code: "023",
            },
            VillageCode {
                name: "二坝村委会",
                code: "024",
            },
            VillageCode {
                name: "上庄村委会",
                code: "025",
            },
            VillageCode {
                name: "新中村委会",
                code: "026",
            },
            VillageCode {
                name: "杨房村委会",
                code: "027",
            },
            VillageCode {
                name: "山泉村委会",
                code: "028",
            },
            VillageCode {
                name: "荣昌村民委员会",
                code: "029",
            },
            VillageCode {
                name: "武威黄羊工业园区社区",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "武南镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "阳光社区居委会",
                code: "001",
            },
            VillageCode {
                name: "育才社区居委会",
                code: "002",
            },
            VillageCode {
                name: "百花园社区居委会",
                code: "003",
            },
            VillageCode {
                name: "迎宾社区居委会",
                code: "004",
            },
            VillageCode {
                name: "花明村社区居委会",
                code: "005",
            },
            VillageCode {
                name: "武南社区居委会",
                code: "006",
            },
            VillageCode {
                name: "范家寨村委会",
                code: "007",
            },
            VillageCode {
                name: "花盛村委会",
                code: "008",
            },
            VillageCode {
                name: "青石村村委会",
                code: "009",
            },
            VillageCode {
                name: "元湖村委会",
                code: "010",
            },
            VillageCode {
                name: "马行河村委会",
                code: "011",
            },
            VillageCode {
                name: "宋府沟村委会",
                code: "012",
            },
            VillageCode {
                name: "西寨村委会",
                code: "013",
            },
            VillageCode {
                name: "武南村村委会",
                code: "014",
            },
            VillageCode {
                name: "大河村委会",
                code: "015",
            },
            VillageCode {
                name: "上中畦村委会",
                code: "016",
            },
            VillageCode {
                name: "下中畦村委会",
                code: "017",
            },
            VillageCode {
                name: "鲁子沟村委会",
                code: "018",
            },
            VillageCode {
                name: "小东河村委会",
                code: "019",
            },
            VillageCode {
                name: "张林村委会",
                code: "020",
            },
            VillageCode {
                name: "百塔村委会",
                code: "021",
            },
            VillageCode {
                name: "唐新庄村委会",
                code: "022",
            },
            VillageCode {
                name: "柏树庄村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "清源镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "幸福家园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "王庄社区居委会",
                code: "002",
            },
            VillageCode {
                name: "清颐佳苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "阳光佳苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "新地村委会",
                code: "005",
            },
            VillageCode {
                name: "新东村委会",
                code: "006",
            },
            VillageCode {
                name: "清泉村委会",
                code: "007",
            },
            VillageCode {
                name: "新西村委会",
                code: "008",
            },
            VillageCode {
                name: "发展村委会",
                code: "009",
            },
            VillageCode {
                name: "王家新庄村委会",
                code: "010",
            },
            VillageCode {
                name: "羊庄村委会",
                code: "011",
            },
            VillageCode {
                name: "中沙村委会",
                code: "012",
            },
            VillageCode {
                name: "东槽村委会",
                code: "013",
            },
            VillageCode {
                name: "宣家庄村委会",
                code: "014",
            },
            VillageCode {
                name: "清源村村委会",
                code: "015",
            },
            VillageCode {
                name: "曾家堡村委会",
                code: "016",
            },
            VillageCode {
                name: "刘广村委会",
                code: "017",
            },
            VillageCode {
                name: "周府庄村委会",
                code: "018",
            },
            VillageCode {
                name: "蔡家寨村委会",
                code: "019",
            },
            VillageCode {
                name: "东城嘉园村（居）委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "永昌镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "白洪社区居委会",
                code: "001",
            },
            VillageCode {
                name: "昌宁苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "永和嘉苑社区居委会",
                code: "003",
            },
            VillageCode {
                name: "校西新聚苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "白洪村委会",
                code: "005",
            },
            VillageCode {
                name: "张英村委会",
                code: "006",
            },
            VillageCode {
                name: "校西村委会",
                code: "007",
            },
            VillageCode {
                name: "东坡村委会",
                code: "008",
            },
            VillageCode {
                name: "烟下村委会",
                code: "009",
            },
            VillageCode {
                name: "和丰村委会",
                code: "010",
            },
            VillageCode {
                name: "和寨村委会",
                code: "011",
            },
            VillageCode {
                name: "山高村委会",
                code: "012",
            },
            VillageCode {
                name: "中沟村委会",
                code: "013",
            },
            VillageCode {
                name: "南沟村委会",
                code: "014",
            },
            VillageCode {
                name: "刘沛村委会",
                code: "015",
            },
            VillageCode {
                name: "石碑村委会",
                code: "016",
            },
            VillageCode {
                name: "梧桐村委会",
                code: "017",
            },
            VillageCode {
                name: "马珣村委会",
                code: "018",
            },
            VillageCode {
                name: "水磨村委会",
                code: "019",
            },
            VillageCode {
                name: "张兴村委会",
                code: "020",
            },
            VillageCode {
                name: "下源村委会",
                code: "021",
            },
            VillageCode {
                name: "上源村委会",
                code: "022",
            },
            VillageCode {
                name: "白云村委会",
                code: "023",
            },
            VillageCode {
                name: "张义村委会",
                code: "024",
            },
            VillageCode {
                name: "羊桐村委会",
                code: "025",
            },
            VillageCode {
                name: "校东村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "双城镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "南安社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "幸福社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "幸福村委会",
                code: "003",
            },
            VillageCode {
                name: "南安村委会",
                code: "004",
            },
            VillageCode {
                name: "前进村委会",
                code: "005",
            },
            VillageCode {
                name: "齐家湖村委会",
                code: "006",
            },
            VillageCode {
                name: "高头沟村委会",
                code: "007",
            },
            VillageCode {
                name: "达桐村委会",
                code: "008",
            },
            VillageCode {
                name: "中山村委会",
                code: "009",
            },
            VillageCode {
                name: "双城村委会",
                code: "010",
            },
            VillageCode {
                name: "徐信村委会",
                code: "011",
            },
            VillageCode {
                name: "宏济村委会",
                code: "012",
            },
            VillageCode {
                name: "宏庄村委会",
                code: "013",
            },
            VillageCode {
                name: "安全村委会",
                code: "014",
            },
            VillageCode {
                name: "河西沟村委会",
                code: "015",
            },
            VillageCode {
                name: "小果园村委会",
                code: "016",
            },
            VillageCode {
                name: "北安村委会",
                code: "017",
            },
            VillageCode {
                name: "羊儿村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "丰乐镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "丰乐社区居委会",
                code: "001",
            },
            VillageCode {
                name: "龙口村委会",
                code: "002",
            },
            VillageCode {
                name: "丰乐村委会",
                code: "003",
            },
            VillageCode {
                name: "昌隆村委会",
                code: "004",
            },
            VillageCode {
                name: "怀西村委会",
                code: "005",
            },
            VillageCode {
                name: "沙滩村委会",
                code: "006",
            },
            VillageCode {
                name: "截河村委会",
                code: "007",
            },
            VillageCode {
                name: "新桥村委会",
                code: "008",
            },
            VillageCode {
                name: "泉沟村委会",
                code: "009",
            },
            VillageCode {
                name: "寨子村委会",
                code: "010",
            },
            VillageCode {
                name: "头牌村委会",
                code: "011",
            },
            VillageCode {
                name: "沙城村委会",
                code: "012",
            },
            VillageCode {
                name: "民生村委会",
                code: "013",
            },
            VillageCode {
                name: "青林村委会",
                code: "014",
            },
            VillageCode {
                name: "红林村委会",
                code: "015",
            },
            VillageCode {
                name: "乔家寺村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "高坝镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "台庄社区居委会",
                code: "001",
            },
            VillageCode {
                name: "蜻蜓社区居委会",
                code: "002",
            },
            VillageCode {
                name: "碌碡民盛家苑居民委员会",
                code: "003",
            },
            VillageCode {
                name: "五里社区居委会",
                code: "004",
            },
            VillageCode {
                name: "十三里民乐苑社区居委会",
                code: "005",
            },
            VillageCode {
                name: "天马社区居委会",
                code: "006",
            },
            VillageCode {
                name: "桃园新村社区居委会",
                code: "007",
            },
            VillageCode {
                name: "鼎宁家园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "同心苑社区居委会",
                code: "009",
            },
            VillageCode {
                name: "镜堂花园社区居委会",
                code: "010",
            },
            VillageCode {
                name: "石岭村委会",
                code: "011",
            },
            VillageCode {
                name: "左二坝村委会",
                code: "012",
            },
            VillageCode {
                name: "建设村委会",
                code: "013",
            },
            VillageCode {
                name: "新民村委会",
                code: "014",
            },
            VillageCode {
                name: "红中村委会",
                code: "015",
            },
            VillageCode {
                name: "红沿村委会",
                code: "016",
            },
            VillageCode {
                name: "红崖村委会",
                code: "017",
            },
            VillageCode {
                name: "路南村委会",
                code: "018",
            },
            VillageCode {
                name: "同益村委会",
                code: "019",
            },
            VillageCode {
                name: "同心村委会",
                code: "020",
            },
            VillageCode {
                name: "丁家园村委会",
                code: "021",
            },
            VillageCode {
                name: "高坝村委会",
                code: "022",
            },
            VillageCode {
                name: "碌碡村委会",
                code: "023",
            },
            VillageCode {
                name: "台庄村委会",
                code: "024",
            },
            VillageCode {
                name: "上马儿村委会",
                code: "025",
            },
            VillageCode {
                name: "新庙村委会",
                code: "026",
            },
            VillageCode {
                name: "五里村委会",
                code: "027",
            },
            VillageCode {
                name: "阳春村委会",
                code: "028",
            },
            VillageCode {
                name: "小七坝村委会",
                code: "029",
            },
            VillageCode {
                name: "楼庄村委会",
                code: "030",
            },
            VillageCode {
                name: "六坝村委会",
                code: "031",
            },
            VillageCode {
                name: "严家村委会",
                code: "032",
            },
            VillageCode {
                name: "十三里村委会",
                code: "033",
            },
            VillageCode {
                name: "刘畦村委会",
                code: "034",
            },
            VillageCode {
                name: "蜻蜓村委会",
                code: "035",
            },
            VillageCode {
                name: "蔡家村委会",
                code: "036",
            },
        ],
    },
    TownCode {
        name: "金羊镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "新鲜小区社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金海嘉苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "松涛社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东沟社区居委会",
                code: "004",
            },
            VillageCode {
                name: "窑沟社区居委会",
                code: "005",
            },
            VillageCode {
                name: "东沟村委会",
                code: "006",
            },
            VillageCode {
                name: "窑沟村委会",
                code: "007",
            },
            VillageCode {
                name: "新城村委会",
                code: "008",
            },
            VillageCode {
                name: "杏园村委会",
                code: "009",
            },
            VillageCode {
                name: "新鲜村委会",
                code: "010",
            },
            VillageCode {
                name: "蔡家庄村委会",
                code: "011",
            },
            VillageCode {
                name: "宋家园村委会",
                code: "012",
            },
            VillageCode {
                name: "皇娘娘台村委会",
                code: "013",
            },
            VillageCode {
                name: "平苑村委会",
                code: "014",
            },
            VillageCode {
                name: "五一村委会",
                code: "015",
            },
            VillageCode {
                name: "三盘磨村委会",
                code: "016",
            },
            VillageCode {
                name: "海藏村委会",
                code: "017",
            },
            VillageCode {
                name: "海上村委会",
                code: "018",
            },
            VillageCode {
                name: "松涛村委会",
                code: "019",
            },
            VillageCode {
                name: "郭家寨村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "和平镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "大众嘉苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "和平社区居委会",
                code: "002",
            },
            VillageCode {
                name: "韩寨村委会",
                code: "003",
            },
            VillageCode {
                name: "和平村委会",
                code: "004",
            },
            VillageCode {
                name: "新胜村委会",
                code: "005",
            },
            VillageCode {
                name: "大众村委会",
                code: "006",
            },
            VillageCode {
                name: "南园村委会",
                code: "007",
            },
            VillageCode {
                name: "枣园村委会",
                code: "008",
            },
            VillageCode {
                name: "牌楼村委会",
                code: "009",
            },
            VillageCode {
                name: "臧家庄村委会",
                code: "010",
            },
            VillageCode {
                name: "新五村委会",
                code: "011",
            },
            VillageCode {
                name: "中庄村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "羊下坝镇",
        code: "019",
        villages: &[
            VillageCode {
                name: "羊下坝社区居委会",
                code: "001",
            },
            VillageCode {
                name: "上二沟村委会",
                code: "002",
            },
            VillageCode {
                name: "下二沟村委会",
                code: "003",
            },
            VillageCode {
                name: "三沟村委会",
                code: "004",
            },
            VillageCode {
                name: "四沟村委会",
                code: "005",
            },
            VillageCode {
                name: "上双村委会",
                code: "006",
            },
            VillageCode {
                name: "五沟村委会",
                code: "007",
            },
            VillageCode {
                name: "六沟村委会",
                code: "008",
            },
            VillageCode {
                name: "七沟村委会",
                code: "009",
            },
            VillageCode {
                name: "丁家湾村委会",
                code: "010",
            },
            VillageCode {
                name: "地湾村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "中坝镇",
        code: "020",
        villages: &[
            VillageCode {
                name: "上坝社区居委会",
                code: "001",
            },
            VillageCode {
                name: "裕和苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "头沟社区居委会",
                code: "003",
            },
            VillageCode {
                name: "下畦居民委员会",
                code: "004",
            },
            VillageCode {
                name: "上坝村委会",
                code: "005",
            },
            VillageCode {
                name: "花寨村委会",
                code: "006",
            },
            VillageCode {
                name: "汪泉村委会",
                code: "007",
            },
            VillageCode {
                name: "下畦村委会",
                code: "008",
            },
            VillageCode {
                name: "中坝村委会",
                code: "009",
            },
            VillageCode {
                name: "头沟村委会",
                code: "010",
            },
            VillageCode {
                name: "高楼村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "永丰镇",
        code: "021",
        villages: &[
            VillageCode {
                name: "毛沟村委会",
                code: "001",
            },
            VillageCode {
                name: "大路村委会",
                code: "002",
            },
            VillageCode {
                name: "朵浪村委会",
                code: "003",
            },
            VillageCode {
                name: "朵云村委会",
                code: "004",
            },
            VillageCode {
                name: "永丰村村委会",
                code: "005",
            },
            VillageCode {
                name: "四十里村委会",
                code: "006",
            },
            VillageCode {
                name: "四坝桥村委会",
                code: "007",
            },
            VillageCode {
                name: "沿沟村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "古城镇",
        code: "022",
        villages: &[
            VillageCode {
                name: "怡和祥社区居委会",
                code: "001",
            },
            VillageCode {
                name: "古城村委会",
                code: "002",
            },
            VillageCode {
                name: "下古村委会",
                code: "003",
            },
            VillageCode {
                name: "六林村委会",
                code: "004",
            },
            VillageCode {
                name: "祁山村委会",
                code: "005",
            },
            VillageCode {
                name: "上古村委会",
                code: "006",
            },
            VillageCode {
                name: "八五村委会",
                code: "007",
            },
            VillageCode {
                name: "光明路村委会",
                code: "008",
            },
            VillageCode {
                name: "三畦村委会",
                code: "009",
            },
            VillageCode {
                name: "三坝村委会",
                code: "010",
            },
            VillageCode {
                name: "小河村委会",
                code: "011",
            },
            VillageCode {
                name: "中河村委会",
                code: "012",
            },
            VillageCode {
                name: "上河村委会",
                code: "013",
            },
            VillageCode {
                name: "九五村委会",
                code: "014",
            },
            VillageCode {
                name: "校尉村委会",
                code: "015",
            },
            VillageCode {
                name: "陈庄村委会",
                code: "016",
            },
            VillageCode {
                name: "长流村委会",
                code: "017",
            },
            VillageCode {
                name: "头坝村委会",
                code: "018",
            },
            VillageCode {
                name: "东河村委会",
                code: "019",
            },
            VillageCode {
                name: "河北村委会",
                code: "020",
            },
            VillageCode {
                name: "五畦村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "张义镇",
        code: "023",
        villages: &[
            VillageCode {
                name: "堡子村委会",
                code: "001",
            },
            VillageCode {
                name: "张庄村委会",
                code: "002",
            },
            VillageCode {
                name: "石头坝村委会",
                code: "003",
            },
            VillageCode {
                name: "康庄村委会",
                code: "004",
            },
            VillageCode {
                name: "大庄村委会",
                code: "005",
            },
            VillageCode {
                name: "河湾村委会",
                code: "006",
            },
            VillageCode {
                name: "沙金台村委会",
                code: "007",
            },
            VillageCode {
                name: "石咀村委会",
                code: "008",
            },
            VillageCode {
                name: "中路村委会",
                code: "009",
            },
            VillageCode {
                name: "常水村委会",
                code: "010",
            },
            VillageCode {
                name: "澄新村委会",
                code: "011",
            },
            VillageCode {
                name: "灯山村委会",
                code: "012",
            },
            VillageCode {
                name: "刘庄村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "发放镇",
        code: "024",
        villages: &[
            VillageCode {
                name: "发放社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "贾家墩社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "双桥社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "小路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "双河村委会",
                code: "005",
            },
            VillageCode {
                name: "新兴村委会",
                code: "006",
            },
            VillageCode {
                name: "安置村委会",
                code: "007",
            },
            VillageCode {
                name: "马儿村委会",
                code: "008",
            },
            VillageCode {
                name: "发放村委会",
                code: "009",
            },
            VillageCode {
                name: "贾家墩村委会",
                code: "010",
            },
            VillageCode {
                name: "双桥村委会",
                code: "011",
            },
            VillageCode {
                name: "小路村委会",
                code: "012",
            },
            VillageCode {
                name: "马莲村委会",
                code: "013",
            },
            VillageCode {
                name: "屯庄村委会",
                code: "014",
            },
            VillageCode {
                name: "西沟村委会",
                code: "015",
            },
            VillageCode {
                name: "六畦村委会",
                code: "016",
            },
            VillageCode {
                name: "双树村委会",
                code: "017",
            },
            VillageCode {
                name: "东沟庙村委会",
                code: "018",
            },
            VillageCode {
                name: "朱家庄村委会",
                code: "019",
            },
            VillageCode {
                name: "沙子沟村委会",
                code: "020",
            },
            VillageCode {
                name: "王家墩村委会",
                code: "021",
            },
            VillageCode {
                name: "下沙子村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "西营镇",
        code: "025",
        villages: &[
            VillageCode {
                name: "接垴社区居委会",
                code: "001",
            },
            VillageCode {
                name: "上六村委会",
                code: "002",
            },
            VillageCode {
                name: "碑岭村委会",
                code: "003",
            },
            VillageCode {
                name: "红星村委会",
                code: "004",
            },
            VillageCode {
                name: "接垴畦村委会",
                code: "005",
            },
            VillageCode {
                name: "双庄村委会",
                code: "006",
            },
            VillageCode {
                name: "宏寺村委会",
                code: "007",
            },
            VillageCode {
                name: "花亭村委会",
                code: "008",
            },
            VillageCode {
                name: "五沟湾村委会",
                code: "009",
            },
            VillageCode {
                name: "三沟湾村委会",
                code: "010",
            },
            VillageCode {
                name: "二沟村委会",
                code: "011",
            },
            VillageCode {
                name: "营儿村委会",
                code: "012",
            },
            VillageCode {
                name: "陈鲁村委会",
                code: "013",
            },
            VillageCode {
                name: "后兴村委会",
                code: "014",
            },
            VillageCode {
                name: "前兴村委会",
                code: "015",
            },
            VillageCode {
                name: "永丰堡村委会",
                code: "016",
            },
            VillageCode {
                name: "杂沟村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "四坝镇",
        code: "026",
        villages: &[
            VillageCode {
                name: "寨子社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北仓村委会",
                code: "002",
            },
            VillageCode {
                name: "杨家寨子村委会",
                code: "003",
            },
            VillageCode {
                name: "四坝村委会",
                code: "004",
            },
            VillageCode {
                name: "前庄村委会",
                code: "005",
            },
            VillageCode {
                name: "海湾村委会",
                code: "006",
            },
            VillageCode {
                name: "三岔村委会",
                code: "007",
            },
            VillageCode {
                name: "南仓村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "洪祥镇",
        code: "027",
        villages: &[
            VillageCode {
                name: "祥瑞苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "刘家沟村委会",
                code: "002",
            },
            VillageCode {
                name: "洪祥村委会",
                code: "003",
            },
            VillageCode {
                name: "天泉村委会",
                code: "004",
            },
            VillageCode {
                name: "新泉村委会",
                code: "005",
            },
            VillageCode {
                name: "陈儿村委会",
                code: "006",
            },
            VillageCode {
                name: "果园村委会",
                code: "007",
            },
            VillageCode {
                name: "陈家沟村委会",
                code: "008",
            },
            VillageCode {
                name: "陈春村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "谢河镇",
        code: "028",
        villages: &[
            VillageCode {
                name: "五中社区居委会",
                code: "001",
            },
            VillageCode {
                name: "四上村委会",
                code: "002",
            },
            VillageCode {
                name: "五坝村委会",
                code: "003",
            },
            VillageCode {
                name: "新路村委会",
                code: "004",
            },
            VillageCode {
                name: "四中村委会",
                code: "005",
            },
            VillageCode {
                name: "五中村委会",
                code: "006",
            },
            VillageCode {
                name: "武家寨村委会",
                code: "007",
            },
            VillageCode {
                name: "谢河村委会",
                code: "008",
            },
            VillageCode {
                name: "石岗村委会",
                code: "009",
            },
            VillageCode {
                name: "李府寨村委会",
                code: "010",
            },
            VillageCode {
                name: "叶家村委会",
                code: "011",
            },
            VillageCode {
                name: "庙山村委会",
                code: "012",
            },
            VillageCode {
                name: "付相庄村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "金沙镇",
        code: "029",
        villages: &[
            VillageCode {
                name: "金厦社区居委会",
                code: "001",
            },
            VillageCode {
                name: "于郭庄社区居委会",
                code: "002",
            },
            VillageCode {
                name: "朱家庄社区居委会",
                code: "003",
            },
            VillageCode {
                name: "赵家磨社区居委会",
                code: "004",
            },
            VillageCode {
                name: "于家庄村委会",
                code: "005",
            },
            VillageCode {
                name: "郭家庄村委会",
                code: "006",
            },
            VillageCode {
                name: "金沙村委会",
                code: "007",
            },
            VillageCode {
                name: "水坑村委会",
                code: "008",
            },
            VillageCode {
                name: "中沟村委会",
                code: "009",
            },
            VillageCode {
                name: "赵磨村委会",
                code: "010",
            },
            VillageCode {
                name: "朱庄村委会",
                code: "011",
            },
            VillageCode {
                name: "李磨村委会",
                code: "012",
            },
            VillageCode {
                name: "吴府村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "松树镇",
        code: "030",
        villages: &[
            VillageCode {
                name: "上三畦村委会",
                code: "001",
            },
            VillageCode {
                name: "莲花山村委会",
                code: "002",
            },
            VillageCode {
                name: "科畦村委会",
                code: "003",
            },
            VillageCode {
                name: "上二畦村委会",
                code: "004",
            },
            VillageCode {
                name: "南河村委会",
                code: "005",
            },
            VillageCode {
                name: "中堡村委会",
                code: "006",
            },
            VillageCode {
                name: "团庄村委会",
                code: "007",
            },
            VillageCode {
                name: "冯良村委会",
                code: "008",
            },
            VillageCode {
                name: "松树村委会",
                code: "009",
            },
            VillageCode {
                name: "槐树村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "怀安镇",
        code: "031",
        villages: &[
            VillageCode {
                name: "北河中心社区居委会",
                code: "001",
            },
            VillageCode {
                name: "怀安村委会",
                code: "002",
            },
            VillageCode {
                name: "高寺村委会",
                code: "003",
            },
            VillageCode {
                name: "芦家沟村委会",
                code: "004",
            },
            VillageCode {
                name: "三中村委会",
                code: "005",
            },
            VillageCode {
                name: "北河村委会",
                code: "006",
            },
            VillageCode {
                name: "二十里村委会",
                code: "007",
            },
            VillageCode {
                name: "驿城村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "下双镇",
        code: "032",
        villages: &[
            VillageCode {
                name: "蓄水村委会",
                code: "001",
            },
            VillageCode {
                name: "沙河村委会",
                code: "002",
            },
            VillageCode {
                name: "涨泗村委会",
                code: "003",
            },
            VillageCode {
                name: "南水村委会",
                code: "004",
            },
            VillageCode {
                name: "下双村委会",
                code: "005",
            },
            VillageCode {
                name: "河水村委会",
                code: "006",
            },
            VillageCode {
                name: "于家湾村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "清水镇",
        code: "033",
        villages: &[
            VillageCode {
                name: "菖盛佳苑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "杨台社区居委会",
                code: "002",
            },
            VillageCode {
                name: "王锐社区居委会",
                code: "003",
            },
            VillageCode {
                name: "上四沟村委会",
                code: "004",
            },
            VillageCode {
                name: "王锐沟村委会",
                code: "005",
            },
            VillageCode {
                name: "苏邓沟村委会",
                code: "006",
            },
            VillageCode {
                name: "张清堡村委会",
                code: "007",
            },
            VillageCode {
                name: "清溪村委会",
                code: "008",
            },
            VillageCode {
                name: "王盛寨村委会",
                code: "009",
            },
            VillageCode {
                name: "菖蒲沟村委会",
                code: "010",
            },
            VillageCode {
                name: "杨台村委会",
                code: "011",
            },
            VillageCode {
                name: "白塔沟村委会",
                code: "012",
            },
            VillageCode {
                name: "河西村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "河东镇",
        code: "034",
        villages: &[
            VillageCode {
                name: "中心社区居委会",
                code: "001",
            },
            VillageCode {
                name: "上腰墩村委会",
                code: "002",
            },
            VillageCode {
                name: "下腰墩村委会",
                code: "003",
            },
            VillageCode {
                name: "王家庄村委会",
                code: "004",
            },
            VillageCode {
                name: "河东村委会",
                code: "005",
            },
            VillageCode {
                name: "头坝河村委会",
                code: "006",
            },
            VillageCode {
                name: "乐安村委会",
                code: "007",
            },
            VillageCode {
                name: "汪家寨村委会",
                code: "008",
            },
            VillageCode {
                name: "五桥村委会",
                code: "009",
            },
            VillageCode {
                name: "钦赐地村委会",
                code: "010",
            },
            VillageCode {
                name: "达家寨村委会",
                code: "011",
            },
            VillageCode {
                name: "新腰墩村委会",
                code: "012",
            },
            VillageCode {
                name: "荣兴村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "五和镇",
        code: "035",
        villages: &[
            VillageCode {
                name: "五和社区居委会",
                code: "001",
            },
            VillageCode {
                name: "沙金村委会",
                code: "002",
            },
            VillageCode {
                name: "五和村委会",
                code: "003",
            },
            VillageCode {
                name: "五爱村委会",
                code: "004",
            },
            VillageCode {
                name: "下寨村委会",
                code: "005",
            },
            VillageCode {
                name: "侯吉村委会",
                code: "006",
            },
            VillageCode {
                name: "胜利村委会",
                code: "007",
            },
            VillageCode {
                name: "支寨村委会",
                code: "008",
            },
            VillageCode {
                name: "新沟村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "长城镇",
        code: "036",
        villages: &[
            VillageCode {
                name: "汉明新城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "长富社区居委会",
                code: "002",
            },
            VillageCode {
                name: "红水村委会",
                code: "003",
            },
            VillageCode {
                name: "西湖村委会",
                code: "004",
            },
            VillageCode {
                name: "前营村委会",
                code: "005",
            },
            VillageCode {
                name: "大湾村委会",
                code: "006",
            },
            VillageCode {
                name: "岸门村委会",
                code: "007",
            },
            VillageCode {
                name: "新庄村委会",
                code: "008",
            },
            VillageCode {
                name: "上营村委会",
                code: "009",
            },
            VillageCode {
                name: "高沟村委会",
                code: "010",
            },
            VillageCode {
                name: "十二墩村委会",
                code: "011",
            },
            VillageCode {
                name: "长城村委会",
                code: "012",
            },
            VillageCode {
                name: "五墩村委会",
                code: "013",
            },
            VillageCode {
                name: "长富村民委员会",
                code: "014",
            },
            VillageCode {
                name: "长瑞村民委员会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "吴家井镇",
        code: "037",
        villages: &[
            VillageCode {
                name: "四方墩村委会",
                code: "001",
            },
            VillageCode {
                name: "七星村委会",
                code: "002",
            },
            VillageCode {
                name: "新建村委会",
                code: "003",
            },
            VillageCode {
                name: "吴家井村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "金河镇",
        code: "038",
        villages: &[
            VillageCode {
                name: "富泉社区居委会",
                code: "001",
            },
            VillageCode {
                name: "王景寨村委会",
                code: "002",
            },
            VillageCode {
                name: "陈家寨村委会",
                code: "003",
            },
            VillageCode {
                name: "郑家庄村委会",
                code: "004",
            },
            VillageCode {
                name: "李家寨村委会",
                code: "005",
            },
            VillageCode {
                name: "旧庄村委会",
                code: "006",
            },
            VillageCode {
                name: "大庄村委会",
                code: "007",
            },
            VillageCode {
                name: "沟口村委会",
                code: "008",
            },
            VillageCode {
                name: "老庄村委会",
                code: "009",
            },
            VillageCode {
                name: "蔡家滩村委会",
                code: "010",
            },
            VillageCode {
                name: "富泉村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "韩佐镇",
        code: "039",
        villages: &[
            VillageCode {
                name: "高坝沟村委会",
                code: "001",
            },
            VillageCode {
                name: "阳畦村委会",
                code: "002",
            },
            VillageCode {
                name: "宏化村委会",
                code: "003",
            },
            VillageCode {
                name: "韩佐村委会",
                code: "004",
            },
            VillageCode {
                name: "头畦村委会",
                code: "005",
            },
            VillageCode {
                name: "二畦村委会",
                code: "006",
            },
            VillageCode {
                name: "禅树村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "大柳镇",
        code: "040",
        villages: &[
            VillageCode {
                name: "柳苑新村社区居委会",
                code: "001",
            },
            VillageCode {
                name: "桥坡村委会",
                code: "002",
            },
            VillageCode {
                name: "大柳村委会",
                code: "003",
            },
            VillageCode {
                name: "烟房村委会",
                code: "004",
            },
            VillageCode {
                name: "东社村委会",
                code: "005",
            },
            VillageCode {
                name: "西社村委会",
                code: "006",
            },
            VillageCode {
                name: "湖沿村委会",
                code: "007",
            },
            VillageCode {
                name: "王城村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "柏树镇",
        code: "041",
        villages: &[
            VillageCode {
                name: "清水中畦社区居委会",
                code: "001",
            },
            VillageCode {
                name: "清水村委会",
                code: "002",
            },
            VillageCode {
                name: "中畦村委会",
                code: "003",
            },
            VillageCode {
                name: "接引村委会",
                code: "004",
            },
            VillageCode {
                name: "右三坝村委会",
                code: "005",
            },
            VillageCode {
                name: "小寨村委会",
                code: "006",
            },
            VillageCode {
                name: "下五畦村委会",
                code: "007",
            },
            VillageCode {
                name: "桥儿村委会",
                code: "008",
            },
            VillageCode {
                name: "张家寨村委会",
                code: "009",
            },
            VillageCode {
                name: "杨寨村委会",
                code: "010",
            },
            VillageCode {
                name: "柏树村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "金塔镇",
        code: "042",
        villages: &[
            VillageCode {
                name: "何家湾村委会",
                code: "001",
            },
            VillageCode {
                name: "黄康寨村委会",
                code: "002",
            },
            VillageCode {
                name: "金塔村委会",
                code: "003",
            },
            VillageCode {
                name: "青铜村委会",
                code: "004",
            },
            VillageCode {
                name: "湾子村委会",
                code: "005",
            },
            VillageCode {
                name: "中心村委会",
                code: "006",
            },
            VillageCode {
                name: "右二坝村委会",
                code: "007",
            },
            VillageCode {
                name: "右五坝村委会",
                code: "008",
            },
            VillageCode {
                name: "华畦村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "九墩镇",
        code: "043",
        villages: &[
            VillageCode {
                name: "九墩社区居委会",
                code: "001",
            },
            VillageCode {
                name: "小泉村委会",
                code: "002",
            },
            VillageCode {
                name: "史家湖村委会",
                code: "003",
            },
            VillageCode {
                name: "九墩村委会",
                code: "004",
            },
            VillageCode {
                name: "光明村委会",
                code: "005",
            },
            VillageCode {
                name: "下窝村委会",
                code: "006",
            },
            VillageCode {
                name: "平乐村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "金山镇",
        code: "044",
        villages: &[
            VillageCode {
                name: "金山社区居委会",
                code: "001",
            },
            VillageCode {
                name: "小口子村委会",
                code: "002",
            },
            VillageCode {
                name: "大口子村委会",
                code: "003",
            },
            VillageCode {
                name: "炭山村委会",
                code: "004",
            },
            VillageCode {
                name: "营盘村委会",
                code: "005",
            },
            VillageCode {
                name: "崖湾村委会",
                code: "006",
            },
            VillageCode {
                name: "金山村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "新华镇",
        code: "045",
        villages: &[
            VillageCode {
                name: "头坝寨村委会",
                code: "001",
            },
            VillageCode {
                name: "李府村委会",
                code: "002",
            },
            VillageCode {
                name: "徐庄村委会",
                code: "003",
            },
            VillageCode {
                name: "新华村委会",
                code: "004",
            },
            VillageCode {
                name: "深沟村委会",
                code: "005",
            },
            VillageCode {
                name: "马蹄村委会",
                code: "006",
            },
            VillageCode {
                name: "马莲滩村委会",
                code: "007",
            },
            VillageCode {
                name: "缠山村委会",
                code: "008",
            },
            VillageCode {
                name: "夹河村委会",
                code: "009",
            },
            VillageCode {
                name: "南营村委会",
                code: "010",
            },
            VillageCode {
                name: "穿城村委会",
                code: "011",
            },
            VillageCode {
                name: "石关村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "康宁镇",
        code: "046",
        villages: &[
            VillageCode {
                name: "山湾村委会",
                code: "001",
            },
            VillageCode {
                name: "康宁村委会",
                code: "002",
            },
            VillageCode {
                name: "西湾村委会",
                code: "003",
            },
            VillageCode {
                name: "新寨村委会",
                code: "004",
            },
            VillageCode {
                name: "东湖村委会",
                code: "005",
            },
            VillageCode {
                name: "龙泉村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "九墩滩生态建设指挥部",
        code: "047",
        villages: &[
            VillageCode {
                name: "富康村委会",
                code: "001",
            },
            VillageCode {
                name: "沿河村委会",
                code: "002",
            },
            VillageCode {
                name: "十墩村委会",
                code: "003",
            },
            VillageCode {
                name: "黄花村委会",
                code: "004",
            },
            VillageCode {
                name: "红水河村委会",
                code: "005",
            },
            VillageCode {
                name: "富民村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "邓马营湖生态建设指挥部",
        code: "048",
        villages: &[
            VillageCode {
                name: "荣华新村社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "荣华新村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "富强新村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "凉州工业园区",
        code: "049",
        villages: &[VillageCode {
            name: "凉州工业园区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "武威工业园区",
        code: "050",
        villages: &[VillageCode {
            name: "武威工业园区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_HX_005: [TownCode; 18] = [
    TownCode {
        name: "三雷镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "东街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "北街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东关社区居委会",
                code: "005",
            },
            VillageCode {
                name: "新关社区居委会",
                code: "006",
            },
            VillageCode {
                name: "新民社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "勤锋社区居委会",
                code: "008",
            },
            VillageCode {
                name: "新民村委会",
                code: "009",
            },
            VillageCode {
                name: "上陶村委会",
                code: "010",
            },
            VillageCode {
                name: "中陶村委会",
                code: "011",
            },
            VillageCode {
                name: "三陶村委会",
                code: "012",
            },
            VillageCode {
                name: "新陶村委会",
                code: "013",
            },
            VillageCode {
                name: "三新村委会",
                code: "014",
            },
            VillageCode {
                name: "建新村委会",
                code: "015",
            },
            VillageCode {
                name: "上管村委会",
                code: "016",
            },
            VillageCode {
                name: "中管村委会",
                code: "017",
            },
            VillageCode {
                name: "下管村委会",
                code: "018",
            },
            VillageCode {
                name: "赵湖村委会",
                code: "019",
            },
            VillageCode {
                name: "上雷村委会",
                code: "020",
            },
            VillageCode {
                name: "中雷村委会",
                code: "021",
            },
            VillageCode {
                name: "下雷村委会",
                code: "022",
            },
            VillageCode {
                name: "渠尾村委会",
                code: "023",
            },
            VillageCode {
                name: "武威市石羊河林业总场生活区",
                code: "024",
            },
            VillageCode {
                name: "石羊河林业总场大滩分场生活区",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "东坝镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "东坝镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "拐湾社区居委会",
                code: "002",
            },
            VillageCode {
                name: "中岔村委会",
                code: "003",
            },
            VillageCode {
                name: "蒿子湖村委会",
                code: "004",
            },
            VillageCode {
                name: "六坝村委会",
                code: "005",
            },
            VillageCode {
                name: "上截村委会",
                code: "006",
            },
            VillageCode {
                name: "西沟村委会",
                code: "007",
            },
            VillageCode {
                name: "左新村委会",
                code: "008",
            },
            VillageCode {
                name: "拐湾村委会",
                code: "009",
            },
            VillageCode {
                name: "裕民村委会",
                code: "010",
            },
            VillageCode {
                name: "白古村委会",
                code: "011",
            },
            VillageCode {
                name: "新华村委会",
                code: "012",
            },
            VillageCode {
                name: "连丰村委会",
                code: "013",
            },
            VillageCode {
                name: "东一村委会",
                code: "014",
            },
            VillageCode {
                name: "东二村委会",
                code: "015",
            },
            VillageCode {
                name: "石羊河林业总场义粮滩林场生活区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "泉山镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "泉山镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "团结社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "复明村委会",
                code: "003",
            },
            VillageCode {
                name: "小西村委会",
                code: "004",
            },
            VillageCode {
                name: "团结村委会",
                code: "005",
            },
            VillageCode {
                name: "福元村委会",
                code: "006",
            },
            VillageCode {
                name: "新西村委会",
                code: "007",
            },
            VillageCode {
                name: "永宁村委会",
                code: "008",
            },
            VillageCode {
                name: "西六村委会",
                code: "009",
            },
            VillageCode {
                name: "中营村委会",
                code: "010",
            },
            VillageCode {
                name: "和平村委会",
                code: "011",
            },
            VillageCode {
                name: "合盛村委会",
                code: "012",
            },
            VillageCode {
                name: "西营村委会",
                code: "013",
            },
            VillageCode {
                name: "复成村委会",
                code: "014",
            },
            VillageCode {
                name: "石羊河林业总场泉山分场生活区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "西渠镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "西渠镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "号顺社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "食珍村委会",
                code: "003",
            },
            VillageCode {
                name: "始成村委会",
                code: "004",
            },
            VillageCode {
                name: "民政村委会",
                code: "005",
            },
            VillageCode {
                name: "幸福村委会",
                code: "006",
            },
            VillageCode {
                name: "民旗村委会",
                code: "007",
            },
            VillageCode {
                name: "丰政村委会",
                code: "008",
            },
            VillageCode {
                name: "建立村委会",
                code: "009",
            },
            VillageCode {
                name: "爱恒村委会",
                code: "010",
            },
            VillageCode {
                name: "尚坐村委会",
                code: "011",
            },
            VillageCode {
                name: "首好村委会",
                code: "012",
            },
            VillageCode {
                name: "北沟村委会",
                code: "013",
            },
            VillageCode {
                name: "制产村委会",
                code: "014",
            },
            VillageCode {
                name: "扶拱村委会",
                code: "015",
            },
            VillageCode {
                name: "致祥村委会",
                code: "016",
            },
            VillageCode {
                name: "大坝村委会",
                code: "017",
            },
            VillageCode {
                name: "板湖村委会",
                code: "018",
            },
            VillageCode {
                name: "三元村委会",
                code: "019",
            },
            VillageCode {
                name: "三附村委会",
                code: "020",
            },
            VillageCode {
                name: "万顺村委会",
                code: "021",
            },
            VillageCode {
                name: "东胜村委会",
                code: "022",
            },
            VillageCode {
                name: "水盛村委会",
                code: "023",
            },
            VillageCode {
                name: "东容村委会",
                code: "024",
            },
            VillageCode {
                name: "外西村委会",
                code: "025",
            },
            VillageCode {
                name: "巨元村委会",
                code: "026",
            },
            VillageCode {
                name: "火坎村委会",
                code: "027",
            },
            VillageCode {
                name: "西金村委会",
                code: "028",
            },
            VillageCode {
                name: "玉成村委会",
                code: "029",
            },
            VillageCode {
                name: "珠明村委会",
                code: "030",
            },
            VillageCode {
                name: "字云村委会",
                code: "031",
            },
            VillageCode {
                name: "姜桂村委会",
                code: "032",
            },
            VillageCode {
                name: "芥玉村委会",
                code: "033",
            },
            VillageCode {
                name: "出鲜村委会",
                code: "034",
            },
            VillageCode {
                name: "号顺村委会",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "东湖镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "东湖镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西岁村委会",
                code: "002",
            },
            VillageCode {
                name: "大号村委会",
                code: "003",
            },
            VillageCode {
                name: "永庆村委会",
                code: "004",
            },
            VillageCode {
                name: "冬固村委会",
                code: "005",
            },
            VillageCode {
                name: "秋成村委会",
                code: "006",
            },
            VillageCode {
                name: "致力村委会",
                code: "007",
            },
            VillageCode {
                name: "宿积村委会",
                code: "008",
            },
            VillageCode {
                name: "附余村委会",
                code: "009",
            },
            VillageCode {
                name: "往致村委会",
                code: "010",
            },
            VillageCode {
                name: "调元村委会",
                code: "011",
            },
            VillageCode {
                name: "阳和村委会",
                code: "012",
            },
            VillageCode {
                name: "署适村委会",
                code: "013",
            },
            VillageCode {
                name: "苍厚村委会",
                code: "014",
            },
            VillageCode {
                name: "雨顺村委会",
                code: "015",
            },
            VillageCode {
                name: "维结村委会",
                code: "016",
            },
            VillageCode {
                name: "正新村委会",
                code: "017",
            },
            VillageCode {
                name: "雨圣村委会",
                code: "018",
            },
            VillageCode {
                name: "下月村委会",
                code: "019",
            },
            VillageCode {
                name: "洪圣村委会",
                code: "020",
            },
            VillageCode {
                name: "西辰村委会",
                code: "021",
            },
            VillageCode {
                name: "红英村委会",
                code: "022",
            },
            VillageCode {
                name: "上润村委会",
                code: "023",
            },
            VillageCode {
                name: "下润村委会",
                code: "024",
            },
            VillageCode {
                name: "东润村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "红砂岗镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "花儿园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "花儿园村委会",
                code: "002",
            },
            VillageCode {
                name: "周家井村委会",
                code: "003",
            },
            VillageCode {
                name: "红砂岗村委会",
                code: "004",
            },
            VillageCode {
                name: "红砂岗镇管委会管理区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "昌宁镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "昌宁镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "安宁社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北井村委会",
                code: "003",
            },
            VillageCode {
                name: "大海子村委会",
                code: "004",
            },
            VillageCode {
                name: "梧桐墩村委会",
                code: "005",
            },
            VillageCode {
                name: "头井子村委会",
                code: "006",
            },
            VillageCode {
                name: "昌盛村委会",
                code: "007",
            },
            VillageCode {
                name: "永安村委会",
                code: "008",
            },
            VillageCode {
                name: "中胜村委会",
                code: "009",
            },
            VillageCode {
                name: "安宁村委会",
                code: "010",
            },
            VillageCode {
                name: "昌宁村委会",
                code: "011",
            },
            VillageCode {
                name: "华建村委会",
                code: "012",
            },
            VillageCode {
                name: "阜康村委会",
                code: "013",
            },
            VillageCode {
                name: "兴安村委会",
                code: "014",
            },
            VillageCode {
                name: "窑街煤电集团民勤县瑞霖生态农林有限责任公司生活区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "重兴镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "扎子沟村委会",
                code: "001",
            },
            VillageCode {
                name: "野马泉村委会",
                code: "002",
            },
            VillageCode {
                name: "新地村委会",
                code: "003",
            },
            VillageCode {
                name: "上案村委会",
                code: "004",
            },
            VillageCode {
                name: "下案村委会",
                code: "005",
            },
            VillageCode {
                name: "红旗村委会",
                code: "006",
            },
            VillageCode {
                name: "双桥村委会",
                code: "007",
            },
            VillageCode {
                name: "东风村委会",
                code: "008",
            },
            VillageCode {
                name: "黑山村委会",
                code: "009",
            },
            VillageCode {
                name: "石羊河林业总场红崖山分场生活区",
                code: "010",
            },
            VillageCode {
                name: "石羊河林业总场扎子沟分场生活区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "薛百镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "五星社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "河东村委会",
                code: "002",
            },
            VillageCode {
                name: "上新村委会",
                code: "003",
            },
            VillageCode {
                name: "宋和村委会",
                code: "004",
            },
            VillageCode {
                name: "更名村委会",
                code: "005",
            },
            VillageCode {
                name: "五星村委会",
                code: "006",
            },
            VillageCode {
                name: "薛百村委会",
                code: "007",
            },
            VillageCode {
                name: "长城村委会",
                code: "008",
            },
            VillageCode {
                name: "双楼村委会",
                code: "009",
            },
            VillageCode {
                name: "张八村委会",
                code: "010",
            },
            VillageCode {
                name: "茂林村委会",
                code: "011",
            },
            VillageCode {
                name: "张麻村委会",
                code: "012",
            },
            VillageCode {
                name: "何大村委会",
                code: "013",
            },
            VillageCode {
                name: "石羊河林业总场防风林实验站生活区",
                code: "014",
            },
            VillageCode {
                name: "石羊河林业总场小坝口分场生活区",
                code: "015",
            },
            VillageCode {
                name: "治沙试验站生活区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "大坝镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "文化社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "祁润村委会",
                code: "002",
            },
            VillageCode {
                name: "张五村委会",
                code: "003",
            },
            VillageCode {
                name: "曹城村委会",
                code: "004",
            },
            VillageCode {
                name: "城近村委会",
                code: "005",
            },
            VillageCode {
                name: "六沟村委会",
                code: "006",
            },
            VillageCode {
                name: "城西村委会",
                code: "007",
            },
            VillageCode {
                name: "田斌村委会",
                code: "008",
            },
            VillageCode {
                name: "张茂村委会",
                code: "009",
            },
            VillageCode {
                name: "文一村委会",
                code: "010",
            },
            VillageCode {
                name: "文二村委会",
                code: "011",
            },
            VillageCode {
                name: "八一村委会",
                code: "012",
            },
            VillageCode {
                name: "王谋村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "苏武镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "苏武社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "川心村委会",
                code: "002",
            },
            VillageCode {
                name: "新润村委会",
                code: "003",
            },
            VillageCode {
                name: "王和村委会",
                code: "004",
            },
            VillageCode {
                name: "兴墩村委会",
                code: "005",
            },
            VillageCode {
                name: "上浪村委会",
                code: "006",
            },
            VillageCode {
                name: "东湖村委会",
                code: "007",
            },
            VillageCode {
                name: "西茨村委会",
                code: "008",
            },
            VillageCode {
                name: "西湖村委会",
                code: "009",
            },
            VillageCode {
                name: "三合村委会",
                code: "010",
            },
            VillageCode {
                name: "上东村委会",
                code: "011",
            },
            VillageCode {
                name: "下东村委会",
                code: "012",
            },
            VillageCode {
                name: "泉水村委会",
                code: "013",
            },
            VillageCode {
                name: "橙槽村委会",
                code: "014",
            },
            VillageCode {
                name: "三坝村委会",
                code: "015",
            },
            VillageCode {
                name: "蒲秧村委会",
                code: "016",
            },
            VillageCode {
                name: "苏山村委会",
                code: "017",
            },
            VillageCode {
                name: "元泰村委会",
                code: "018",
            },
            VillageCode {
                name: "学粮村委会",
                code: "019",
            },
            VillageCode {
                name: "五坝村委会",
                code: "020",
            },
            VillageCode {
                name: "许岔村委会",
                code: "021",
            },
            VillageCode {
                name: "中沟村委会",
                code: "022",
            },
            VillageCode {
                name: "邓岔村委会",
                code: "023",
            },
            VillageCode {
                name: "千户村委会",
                code: "024",
            },
            VillageCode {
                name: "羊路村委会",
                code: "025",
            },
            VillageCode {
                name: "龙一村委会",
                code: "026",
            },
            VillageCode {
                name: "龙二村委会",
                code: "027",
            },
            VillageCode {
                name: "石羊河林业总厂苏武山林场生活区",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "大滩镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "大滩社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "上泉村委会",
                code: "002",
            },
            VillageCode {
                name: "下泉村委会",
                code: "003",
            },
            VillageCode {
                name: "东大村委会",
                code: "004",
            },
            VillageCode {
                name: "三坪村委会",
                code: "005",
            },
            VillageCode {
                name: "红墙村委会",
                code: "006",
            },
            VillageCode {
                name: "大西村委会",
                code: "007",
            },
            VillageCode {
                name: "北东村委会",
                code: "008",
            },
            VillageCode {
                name: "北新村委会",
                code: "009",
            },
            VillageCode {
                name: "北西村委会",
                code: "010",
            },
            VillageCode {
                name: "北中村委会",
                code: "011",
            },
            VillageCode {
                name: "石羊河林业总场大滩园林场生活区",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "双茨科镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "二分园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中路村委会",
                code: "002",
            },
            VillageCode {
                name: "小新村委会",
                code: "003",
            },
            VillageCode {
                name: "头坝村委会",
                code: "004",
            },
            VillageCode {
                name: "关路村委会",
                code: "005",
            },
            VillageCode {
                name: "三杰东村委会",
                code: "006",
            },
            VillageCode {
                name: "二分村委会",
                code: "007",
            },
            VillageCode {
                name: "红东村委会",
                code: "008",
            },
            VillageCode {
                name: "上东村委会",
                code: "009",
            },
            VillageCode {
                name: "红中村委会",
                code: "010",
            },
            VillageCode {
                name: "红光村委会",
                code: "011",
            },
            VillageCode {
                name: "红政村委会",
                code: "012",
            },
            VillageCode {
                name: "红星村委会",
                code: "013",
            },
            VillageCode {
                name: "三杰西村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "红沙梁镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "建设社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "刘家地村委会",
                code: "002",
            },
            VillageCode {
                name: "高来旺村委会",
                code: "003",
            },
            VillageCode {
                name: "上王化村委会",
                code: "004",
            },
            VillageCode {
                name: "孙指挥村委会",
                code: "005",
            },
            VillageCode {
                name: "化音村委会",
                code: "006",
            },
            VillageCode {
                name: "建设村委会",
                code: "007",
            },
            VillageCode {
                name: "复指挥村委会",
                code: "008",
            },
            VillageCode {
                name: "义地村委会",
                code: "009",
            },
            VillageCode {
                name: "花寨村委会",
                code: "010",
            },
            VillageCode {
                name: "小东村委会",
                code: "011",
            },
            VillageCode {
                name: "新沟村委会",
                code: "012",
            },
            VillageCode {
                name: "上沟村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "蔡旗镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "沙滩社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "蔡旗社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "高庙村委会",
                code: "003",
            },
            VillageCode {
                name: "月牙村委会",
                code: "004",
            },
            VillageCode {
                name: "官沟村委会",
                code: "005",
            },
            VillageCode {
                name: "蔡旗村委会",
                code: "006",
            },
            VillageCode {
                name: "金家庄村委会",
                code: "007",
            },
            VillageCode {
                name: "麻家湾村委会",
                code: "008",
            },
            VillageCode {
                name: "野潴湾村委会",
                code: "009",
            },
            VillageCode {
                name: "小西沟村委会",
                code: "010",
            },
            VillageCode {
                name: "沙滩村委会",
                code: "011",
            },
            VillageCode {
                name: "煌辉新村村委会",
                code: "012",
            },
            VillageCode {
                name: "石羊河林业总场小西沟分场生活区",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "夹河镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "新粮地社区居委会",
                code: "001",
            },
            VillageCode {
                name: "肖案村委会",
                code: "002",
            },
            VillageCode {
                name: "国栋村委会",
                code: "003",
            },
            VillageCode {
                name: "中坪村委会",
                code: "004",
            },
            VillageCode {
                name: "星火村委会",
                code: "005",
            },
            VillageCode {
                name: "大坑沿村委会",
                code: "006",
            },
            VillageCode {
                name: "新粮地村委会",
                code: "007",
            },
            VillageCode {
                name: "曙光村委会",
                code: "008",
            },
            VillageCode {
                name: "南坪村委会",
                code: "009",
            },
            VillageCode {
                name: "刘案村委会",
                code: "010",
            },
            VillageCode {
                name: "朱案村委会",
                code: "011",
            },
            VillageCode {
                name: "七案村委会",
                code: "012",
            },
            VillageCode {
                name: "黄案村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "收成镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "天成社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "002",
            },
            VillageCode {
                name: "丰庆村委会",
                code: "003",
            },
            VillageCode {
                name: "珍宝村委会",
                code: "004",
            },
            VillageCode {
                name: "流裕村委会",
                code: "005",
            },
            VillageCode {
                name: "永丰村委会",
                code: "006",
            },
            VillageCode {
                name: "泗湖村委会",
                code: "007",
            },
            VillageCode {
                name: "中和村委会",
                code: "008",
            },
            VillageCode {
                name: "礼智村委会",
                code: "009",
            },
            VillageCode {
                name: "附智村委会",
                code: "010",
            },
            VillageCode {
                name: "兴盛村委会",
                code: "011",
            },
            VillageCode {
                name: "天成村委会",
                code: "012",
            },
            VillageCode {
                name: "宙和村委会",
                code: "013",
            },
            VillageCode {
                name: "盈科村委会",
                code: "014",
            },
            VillageCode {
                name: "中兴村委会",
                code: "015",
            },
            VillageCode {
                name: "黄岭村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "南湖镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "南湖社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "夹岗井村委会",
                code: "002",
            },
            VillageCode {
                name: "麻莲井村委会",
                code: "003",
            },
            VillageCode {
                name: "甘草井村委会",
                code: "004",
            },
            VillageCode {
                name: "西井村委会",
                code: "005",
            },
            VillageCode {
                name: "南井村委会",
                code: "006",
            },
        ],
    },
];

static TOWNS_HX_006: [TownCode; 19] = [
    TownCode {
        name: "古浪镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "街西社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "昌松社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "街东社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "新苑社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "昌灵路社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "峡峰村民委员会",
                code: "006",
            },
            VillageCode {
                name: "丰泉村民委员会",
                code: "007",
            },
            VillageCode {
                name: "暖泉村民委员会",
                code: "008",
            },
            VillageCode {
                name: "小桥村民委员会",
                code: "009",
            },
            VillageCode {
                name: "联泉村民委员会",
                code: "010",
            },
            VillageCode {
                name: "胡家湾村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "泗水镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "泗水村民委员会",
                code: "001",
            },
            VillageCode {
                name: "周庄村民委员会",
                code: "002",
            },
            VillageCode {
                name: "光丰村民委员会",
                code: "003",
            },
            VillageCode {
                name: "光辉村民委员会",
                code: "004",
            },
            VillageCode {
                name: "下四坝村民委员会",
                code: "005",
            },
            VillageCode {
                name: "上四坝村民委员会",
                code: "006",
            },
            VillageCode {
                name: "双塔村民委员会",
                code: "007",
            },
            VillageCode {
                name: "铁门村民委员会",
                code: "008",
            },
            VillageCode {
                name: "三坝村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "土门镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "众兴社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "土门村民委员会",
                code: "002",
            },
            VillageCode {
                name: "漪泉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "台子村民委员会",
                code: "004",
            },
            VillageCode {
                name: "教场村民委员会",
                code: "005",
            },
            VillageCode {
                name: "青萍村民委员会",
                code: "006",
            },
            VillageCode {
                name: "大湾村民委员会",
                code: "007",
            },
            VillageCode {
                name: "三关村民委员会",
                code: "008",
            },
            VillageCode {
                name: "新胜村民委员会",
                code: "009",
            },
            VillageCode {
                name: "和乐村民委员会",
                code: "010",
            },
            VillageCode {
                name: "胡家边村民委员会",
                code: "011",
            },
            VillageCode {
                name: "王府村民委员会",
                code: "012",
            },
            VillageCode {
                name: "宝塔寺村民委员会",
                code: "013",
            },
            VillageCode {
                name: "永东村民委员会",
                code: "014",
            },
            VillageCode {
                name: "永西村民委员会",
                code: "015",
            },
            VillageCode {
                name: "新丰村民委员会",
                code: "016",
            },
            VillageCode {
                name: "西滩村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "大靖镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "明珠社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东关村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西关村民委员会",
                code: "003",
            },
            VillageCode {
                name: "北关村民委员会",
                code: "004",
            },
            VillageCode {
                name: "三台村民委员会",
                code: "005",
            },
            VillageCode {
                name: "园艺村民委员会",
                code: "006",
            },
            VillageCode {
                name: "双城村民委员会",
                code: "007",
            },
            VillageCode {
                name: "大庄村民委员会",
                code: "008",
            },
            VillageCode {
                name: "龙岗村民委员会",
                code: "009",
            },
            VillageCode {
                name: "长城村民委员会",
                code: "010",
            },
            VillageCode {
                name: "砂河塘村民委员会",
                code: "011",
            },
            VillageCode {
                name: "樊家滩村民委员会",
                code: "012",
            },
            VillageCode {
                name: "下庄村民委员会",
                code: "013",
            },
            VillageCode {
                name: "圈城村民委员会",
                code: "014",
            },
            VillageCode {
                name: "上庄村民委员会",
                code: "015",
            },
            VillageCode {
                name: "峡口村民委员会",
                code: "016",
            },
            VillageCode {
                name: "洋胡塘村民委员会",
                code: "017",
            },
            VillageCode {
                name: "刘家滩村民委员会",
                code: "018",
            },
            VillageCode {
                name: "褚家窝铺村民委员会",
                code: "019",
            },
            VillageCode {
                name: "新华村民委员会",
                code: "020",
            },
            VillageCode {
                name: "大墩村民委员会",
                code: "021",
            },
            VillageCode {
                name: "白家窝铺村民委员会",
                code: "022",
            },
            VillageCode {
                name: "干新村民委员会",
                code: "023",
            },
            VillageCode {
                name: "上湾村民委员会",
                code: "024",
            },
            VillageCode {
                name: "红柳湾村民委员会",
                code: "025",
            },
            VillageCode {
                name: "民丰村民委员会",
                code: "026",
            },
            VillageCode {
                name: "代家窝铺村民委员会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "裴家营镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "裴家营村民委员会",
                code: "001",
            },
            VillageCode {
                name: "哈家台村民委员会",
                code: "002",
            },
            VillageCode {
                name: "塘坊村民委员会",
                code: "003",
            },
            VillageCode {
                name: "王家庄村民委员会",
                code: "004",
            },
            VillageCode {
                name: "岳家滩村民委员会",
                code: "005",
            },
            VillageCode {
                name: "孟家庄村民委员会",
                code: "006",
            },
            VillageCode {
                name: "高岭村民委员会",
                code: "007",
            },
            VillageCode {
                name: "北滩村民委员会",
                code: "008",
            },
            VillageCode {
                name: "华新村民委员会",
                code: "009",
            },
            VillageCode {
                name: "槐湾村民委员会",
                code: "010",
            },
            VillageCode {
                name: "小岭滩村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "海子滩镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "谭家井村民委员会",
                code: "001",
            },
            VillageCode {
                name: "谭新村民委员会",
                code: "002",
            },
            VillageCode {
                name: "民权村民委员会",
                code: "003",
            },
            VillageCode {
                name: "马场滩村民委员会",
                code: "004",
            },
            VillageCode {
                name: "土沟井村民委员会",
                code: "005",
            },
            VillageCode {
                name: "民新村民委员会",
                code: "006",
            },
            VillageCode {
                name: "元庄村民委员会",
                code: "007",
            },
            VillageCode {
                name: "元新村民委员会",
                code: "008",
            },
            VillageCode {
                name: "海子村民委员会",
                code: "009",
            },
            VillageCode {
                name: "景新村民委员会",
                code: "010",
            },
            VillageCode {
                name: "高家窝铺村民委员会",
                code: "011",
            },
            VillageCode {
                name: "岘子村民委员会",
                code: "012",
            },
            VillageCode {
                name: "李家窝铺村民委员会",
                code: "013",
            },
            VillageCode {
                name: "上冰村民委员会",
                code: "014",
            },
            VillageCode {
                name: "下冰村民委员会",
                code: "015",
            },
            VillageCode {
                name: "红沙滩村民委员会",
                code: "016",
            },
            VillageCode {
                name: "二咀子村民委员会",
                code: "017",
            },
            VillageCode {
                name: "东新村民委员会",
                code: "018",
            },
            VillageCode {
                name: "新民村民委员会",
                code: "019",
            },
            VillageCode {
                name: "草原井村民委员会",
                code: "020",
            },
            VillageCode {
                name: "和谐村民委员会",
                code: "021",
            },
            VillageCode {
                name: "张家沙河村民委员会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "定宁镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "定宁村民委员会",
                code: "001",
            },
            VillageCode {
                name: "晨光村民委员会",
                code: "002",
            },
            VillageCode {
                name: "双庙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "肖营村民委员会",
                code: "004",
            },
            VillageCode {
                name: "曙光村民委员会",
                code: "005",
            },
            VillageCode {
                name: "高家湾村民委员会",
                code: "006",
            },
            VillageCode {
                name: "长流村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "黄羊川镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "张家墩村民委员会",
                code: "001",
            },
            VillageCode {
                name: "周家庄村民委员会",
                code: "002",
            },
            VillageCode {
                name: "菜子口村民委员会",
                code: "003",
            },
            VillageCode {
                name: "一棵树村民委员会",
                code: "004",
            },
            VillageCode {
                name: "大南冲村民委员会",
                code: "005",
            },
            VillageCode {
                name: "马圈滩村民委员会",
                code: "006",
            },
            VillageCode {
                name: "石门山村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "黑松驿镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "黑松驿村民委员会",
                code: "001",
            },
            VillageCode {
                name: "小坡村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "永丰滩镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "新河村民委员会",
                code: "001",
            },
            VillageCode {
                name: "庵门村民委员会",
                code: "002",
            },
            VillageCode {
                name: "六墩村民委员会",
                code: "003",
            },
            VillageCode {
                name: "建设村民委员会",
                code: "004",
            },
            VillageCode {
                name: "新建村民委员会",
                code: "005",
            },
            VillageCode {
                name: "三墩村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东台村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "黄花滩镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "旱石河台村民委员会",
                code: "001",
            },
            VillageCode {
                name: "马路滩村民委员会",
                code: "002",
            },
            VillageCode {
                name: "白板滩村民委员会",
                code: "003",
            },
            VillageCode {
                name: "麻黄台村民委员会",
                code: "004",
            },
            VillageCode {
                name: "四墩村民委员会",
                code: "005",
            },
            VillageCode {
                name: "新西村民委员会",
                code: "006",
            },
            VillageCode {
                name: "二墩村民委员会",
                code: "007",
            },
            VillageCode {
                name: "黄花滩村民委员会",
                code: "008",
            },
            VillageCode {
                name: "金滩村民委员会",
                code: "009",
            },
            VillageCode {
                name: "富康新村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "西靖镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "西靖村民委员会",
                code: "001",
            },
            VillageCode {
                name: "古山村民委员会",
                code: "002",
            },
            VillageCode {
                name: "平源村民委员会",
                code: "003",
            },
            VillageCode {
                name: "高峰村民委员会",
                code: "004",
            },
            VillageCode {
                name: "七墩台村民委员会",
                code: "005",
            },
            VillageCode {
                name: "感恩新村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "阳光新村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "圆梦新村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "惠民新村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "康乐新村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "民权镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "金星村民委员会",
                code: "001",
            },
            VillageCode {
                name: "西川村民委员会",
                code: "002",
            },
            VillageCode {
                name: "杜庄村民委员会",
                code: "003",
            },
            VillageCode {
                name: "团庄村民委员会",
                code: "004",
            },
            VillageCode {
                name: "台子村民委员会",
                code: "005",
            },
            VillageCode {
                name: "红旗村民委员会",
                code: "006",
            },
            VillageCode {
                name: "峡口村民委员会",
                code: "007",
            },
            VillageCode {
                name: "山湾村民委员会",
                code: "008",
            },
            VillageCode {
                name: "沙河沿村民委员会",
                code: "009",
            },
            VillageCode {
                name: "民权村民委员会",
                code: "010",
            },
            VillageCode {
                name: "长岭村民委员会",
                code: "011",
            },
            VillageCode {
                name: "土沟村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "直滩镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "直滩村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东中滩村民委员会",
                code: "002",
            },
            VillageCode {
                name: "龙泉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "大岭村民委员会",
                code: "004",
            },
            VillageCode {
                name: "兴岭村民委员会",
                code: "005",
            },
            VillageCode {
                name: "十支村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东分支村民委员会",
                code: "007",
            },
            VillageCode {
                name: "西分支村民委员会",
                code: "008",
            },
            VillageCode {
                name: "新井村民委员会",
                code: "009",
            },
            VillageCode {
                name: "石坡村民委员会",
                code: "010",
            },
            VillageCode {
                name: "新城村民委员会",
                code: "011",
            },
            VillageCode {
                name: "老城村民委员会",
                code: "012",
            },
            VillageCode {
                name: "大沙沟村民委员会",
                code: "013",
            },
            VillageCode {
                name: "龙湾村民委员会",
                code: "014",
            },
            VillageCode {
                name: "中滩村民委员会",
                code: "015",
            },
            VillageCode {
                name: "上滩村民委员会",
                code: "016",
            },
            VillageCode {
                name: "建丰村民委员会",
                code: "017",
            },
            VillageCode {
                name: "新川村民委员会",
                code: "018",
            },
            VillageCode {
                name: "井新村民委员会",
                code: "019",
            },
            VillageCode {
                name: "发展村民委员会",
                code: "020",
            },
            VillageCode {
                name: "团结村民委员会",
                code: "021",
            },
            VillageCode {
                name: "富兴村民委员会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "古丰镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "古丰村民委员会",
                code: "001",
            },
            VillageCode {
                name: "西山堡村民委员会",
                code: "002",
            },
            VillageCode {
                name: "冰沟墩村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "新堡乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "兴民新村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "富源新村村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "干城乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "富民新村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "爱民新村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "立民新村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "为民新村村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "横梁乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "新安新村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "幸福新村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "康宁新村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "春晖新村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "长兴新村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "朝阳新村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "十八里堡乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "十八里堡村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东庙儿沟村民委员会",
                code: "002",
            },
            VillageCode {
                name: "赵家庄村民委员会",
                code: "003",
            },
            VillageCode {
                name: "曹家台村民委员会",
                code: "004",
            },
        ],
    },
];

static TOWNS_HX_007: [TownCode; 21] = [
    TownCode {
        name: "华藏寺镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "兴盛社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "学勤社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "团结社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "和谐社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "华秀社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "秀龙社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "文润社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "祥瑞社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "延禧社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "永兴新村社区居委会",
                code: "010",
            },
            VillageCode {
                name: "黄草川新村社区居委会",
                code: "011",
            },
            VillageCode {
                name: "红大新村社区居委会",
                code: "012",
            },
            VillageCode {
                name: "红大村委会",
                code: "013",
            },
            VillageCode {
                name: "红明村委会",
                code: "014",
            },
            VillageCode {
                name: "阳山村委会",
                code: "015",
            },
            VillageCode {
                name: "中庄村委会",
                code: "016",
            },
            VillageCode {
                name: "南山村委会",
                code: "017",
            },
            VillageCode {
                name: "华藏寺村委会",
                code: "018",
            },
            VillageCode {
                name: "岔口驿村委会",
                code: "019",
            },
            VillageCode {
                name: "栗家庄村委会",
                code: "020",
            },
            VillageCode {
                name: "周家窑村委会",
                code: "021",
            },
            VillageCode {
                name: "黄草川村民委员会",
                code: "022",
            },
            VillageCode {
                name: "野雉沟村委会",
                code: "023",
            },
            VillageCode {
                name: "韭菜沟村委会",
                code: "024",
            },
            VillageCode {
                name: "阳洼台村委会",
                code: "025",
            },
            VillageCode {
                name: "柏林村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "打柴沟镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "打柴沟社区居委会",
                code: "001",
            },
            VillageCode {
                name: "火石沟新村社区居委会",
                code: "002",
            },
            VillageCode {
                name: "安门村委会",
                code: "003",
            },
            VillageCode {
                name: "上河东村委会",
                code: "004",
            },
            VillageCode {
                name: "下河东村委会",
                code: "005",
            },
            VillageCode {
                name: "庙儿沟村委会",
                code: "006",
            },
            VillageCode {
                name: "石板沟村委会",
                code: "007",
            },
            VillageCode {
                name: "安家河村委会",
                code: "008",
            },
            VillageCode {
                name: "铁腰村委会",
                code: "009",
            },
            VillageCode {
                name: "打柴沟村委会",
                code: "010",
            },
            VillageCode {
                name: "大庄村委会",
                code: "011",
            },
            VillageCode {
                name: "深沟村委会",
                code: "012",
            },
            VillageCode {
                name: "金强驿村委会",
                code: "013",
            },
            VillageCode {
                name: "石灰沟村委会",
                code: "014",
            },
            VillageCode {
                name: "下十八村委会",
                code: "015",
            },
            VillageCode {
                name: "火石沟村委会",
                code: "016",
            },
            VillageCode {
                name: "友谊村委会",
                code: "017",
            },
            VillageCode {
                name: "大湾村委会",
                code: "018",
            },
            VillageCode {
                name: "多隆村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "安远镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "安远居委会",
                code: "001",
            },
            VillageCode {
                name: "大泉头村委会",
                code: "002",
            },
            VillageCode {
                name: "安远村委会",
                code: "003",
            },
            VillageCode {
                name: "直沟村委会",
                code: "004",
            },
            VillageCode {
                name: "柳树沟村委会",
                code: "005",
            },
            VillageCode {
                name: "南泥湾村委会",
                code: "006",
            },
            VillageCode {
                name: "兰泉村委会",
                code: "007",
            },
            VillageCode {
                name: "乌鞘岭村委会",
                code: "008",
            },
            VillageCode {
                name: "野狐湾村委会",
                code: "009",
            },
            VillageCode {
                name: "极乐村委会",
                code: "010",
            },
            VillageCode {
                name: "马家台村委会",
                code: "011",
            },
            VillageCode {
                name: "黑河滩村委会",
                code: "012",
            },
            VillageCode {
                name: "白塔村委会",
                code: "013",
            },
            VillageCode {
                name: "河底村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "炭山岭镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "炭山镇中河社区居委会",
                code: "001",
            },
            VillageCode {
                name: "炭山岭二台社区居委会",
                code: "002",
            },
            VillageCode {
                name: "炭山岭镇石界子社区居委会",
                code: "003",
            },
            VillageCode {
                name: "炭山岭新村社区居委会",
                code: "004",
            },
            VillageCode {
                name: "天安新村社区居委会",
                code: "005",
            },
            VillageCode {
                name: "塔窝村委会",
                code: "006",
            },
            VillageCode {
                name: "拉卜子村委会",
                code: "007",
            },
            VillageCode {
                name: "菜籽湾村委会",
                code: "008",
            },
            VillageCode {
                name: "金沙村委会",
                code: "009",
            },
            VillageCode {
                name: "上岗岭村委会",
                code: "010",
            },
            VillageCode {
                name: "关朵村委会",
                code: "011",
            },
            VillageCode {
                name: "阿沿沟村委会",
                code: "012",
            },
            VillageCode {
                name: "四台沟村委会",
                code: "013",
            },
            VillageCode {
                name: "炭山岭村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "哈溪镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "哈溪镇居委会",
                code: "001",
            },
            VillageCode {
                name: "友爱村委会",
                code: "002",
            },
            VillageCode {
                name: "团结村委会",
                code: "003",
            },
            VillageCode {
                name: "西滩村委会",
                code: "004",
            },
            VillageCode {
                name: "古城村委会",
                code: "005",
            },
            VillageCode {
                name: "水泉村委会",
                code: "006",
            },
            VillageCode {
                name: "双龙村委会",
                code: "007",
            },
            VillageCode {
                name: "东滩村委会",
                code: "008",
            },
            VillageCode {
                name: "河沿村委会",
                code: "009",
            },
            VillageCode {
                name: "尖山村委会",
                code: "010",
            },
            VillageCode {
                name: "前进村委会",
                code: "011",
            },
            VillageCode {
                name: "长岭村委会",
                code: "012",
            },
            VillageCode {
                name: "茶岗村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "赛什斯镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "古城居委会",
                code: "001",
            },
            VillageCode {
                name: "先明峡村委会",
                code: "002",
            },
            VillageCode {
                name: "拉干村委会",
                code: "003",
            },
            VillageCode {
                name: "土城村委会",
                code: "004",
            },
            VillageCode {
                name: "野狐川村委会",
                code: "005",
            },
            VillageCode {
                name: "下古城村委会",
                code: "006",
            },
            VillageCode {
                name: "麻渣塘村委会",
                code: "007",
            },
            VillageCode {
                name: "阳洼村委会",
                code: "008",
            },
            VillageCode {
                name: "克岔村委会",
                code: "009",
            },
            VillageCode {
                name: "大滩村委会",
                code: "010",
            },
            VillageCode {
                name: "上古城村委会",
                code: "011",
            },
            VillageCode {
                name: "东大寺村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "石门镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "社区居委会",
                code: "001",
            },
            VillageCode {
                name: "维芨滩村委会",
                code: "002",
            },
            VillageCode {
                name: "石门村委会",
                code: "003",
            },
            VillageCode {
                name: "大塘村委会",
                code: "004",
            },
            VillageCode {
                name: "马营坡村委会",
                code: "005",
            },
            VillageCode {
                name: "岔岔洼村委会",
                code: "006",
            },
            VillageCode {
                name: "石板湾村委会",
                code: "007",
            },
            VillageCode {
                name: "宽沟村委会",
                code: "008",
            },
            VillageCode {
                name: "火烧城村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "松山镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "松山镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "德吉新村社区居委会",
                code: "002",
            },
            VillageCode {
                name: "祥瑞新村社区居委会",
                code: "003",
            },
            VillageCode {
                name: "阿岗湾村委会",
                code: "004",
            },
            VillageCode {
                name: "中大沟村委会",
                code: "005",
            },
            VillageCode {
                name: "芨芨滩村委会",
                code: "006",
            },
            VillageCode {
                name: "石塘村委会",
                code: "007",
            },
            VillageCode {
                name: "红石村委会",
                code: "008",
            },
            VillageCode {
                name: "滩口村委会",
                code: "009",
            },
            VillageCode {
                name: "松山村委会",
                code: "010",
            },
            VillageCode {
                name: "鞍子山村委会",
                code: "011",
            },
            VillageCode {
                name: "藏民村委会",
                code: "012",
            },
            VillageCode {
                name: "达隆村委会",
                code: "013",
            },
            VillageCode {
                name: "蕨麻村委会",
                code: "014",
            },
            VillageCode {
                name: "黑马圈河村委会",
                code: "015",
            },
            VillageCode {
                name: "华吉塘村委会",
                code: "016",
            },
            VillageCode {
                name: "达秀村委会",
                code: "017",
            },
            VillageCode {
                name: "秀杰村委会",
                code: "018",
            },
            VillageCode {
                name: "藜香村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "天堂镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "天堂镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "天堂村委会",
                code: "002",
            },
            VillageCode {
                name: "那威村委会",
                code: "003",
            },
            VillageCode {
                name: "本康村委会",
                code: "004",
            },
            VillageCode {
                name: "科拉村委会",
                code: "005",
            },
            VillageCode {
                name: "菊花村委会",
                code: "006",
            },
            VillageCode {
                name: "雪龙村委会",
                code: "007",
            },
            VillageCode {
                name: "查干村委会",
                code: "008",
            },
            VillageCode {
                name: "业土村委会",
                code: "009",
            },
            VillageCode {
                name: "麻科村委会",
                code: "010",
            },
            VillageCode {
                name: "朱岔村委会",
                code: "011",
            },
            VillageCode {
                name: "保干村委会",
                code: "012",
            },
            VillageCode {
                name: "大湾村委会",
                code: "013",
            },
            VillageCode {
                name: "小科什旦村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "朵什镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "松林社区居委会",
                code: "001",
            },
            VillageCode {
                name: "直岔村委会",
                code: "002",
            },
            VillageCode {
                name: "南冲村委会",
                code: "003",
            },
            VillageCode {
                name: "黑沟村委会",
                code: "004",
            },
            VillageCode {
                name: "石沟村委会",
                code: "005",
            },
            VillageCode {
                name: "旱泉沟村委会",
                code: "006",
            },
            VillageCode {
                name: "窑洞湾村委会",
                code: "007",
            },
            VillageCode {
                name: "煤场村委会",
                code: "008",
            },
            VillageCode {
                name: "寺掌村委会",
                code: "009",
            },
            VillageCode {
                name: "龙沟村委会",
                code: "010",
            },
            VillageCode {
                name: "茶树沟村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "西大滩镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "马莲沟村委会",
                code: "001",
            },
            VillageCode {
                name: "西沟村委会",
                code: "002",
            },
            VillageCode {
                name: "上泉村委会",
                code: "003",
            },
            VillageCode {
                name: "西大滩村委会",
                code: "004",
            },
            VillageCode {
                name: "白土台村委会",
                code: "005",
            },
            VillageCode {
                name: "马场村委会",
                code: "006",
            },
            VillageCode {
                name: "东泉村委会",
                code: "007",
            },
            VillageCode {
                name: "土星村委会",
                code: "008",
            },
            VillageCode {
                name: "坝堵村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "抓喜秀龙镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "永丰村委会",
                code: "001",
            },
            VillageCode {
                name: "炭窑沟村委会",
                code: "002",
            },
            VillageCode {
                name: "红疙瘩村委会",
                code: "003",
            },
            VillageCode {
                name: "代乾村委会",
                code: "004",
            },
            VillageCode {
                name: "南泥沟村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "大红沟镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "红沟寺村委会",
                code: "001",
            },
            VillageCode {
                name: "马路村委会",
                code: "002",
            },
            VillageCode {
                name: "大红沟村委会",
                code: "003",
            },
            VillageCode {
                name: "东圈湾村委会",
                code: "004",
            },
            VillageCode {
                name: "灰条沟村委会",
                code: "005",
            },
            VillageCode {
                name: "东怀村委会",
                code: "006",
            },
            VillageCode {
                name: "西顶村委会",
                code: "007",
            },
            VillageCode {
                name: "下西顶村委会",
                code: "008",
            },
            VillageCode {
                name: "大沟村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "祁连镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "天山村委会",
                code: "001",
            },
            VillageCode {
                name: "马场滩村委会",
                code: "002",
            },
            VillageCode {
                name: "祁连村委会",
                code: "003",
            },
            VillageCode {
                name: "岔山村委会",
                code: "004",
            },
            VillageCode {
                name: "石大板村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "东坪乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "扎帐村委会",
                code: "001",
            },
            VillageCode {
                name: "大麦花村委会",
                code: "002",
            },
            VillageCode {
                name: "先锋村委会",
                code: "003",
            },
            VillageCode {
                name: "坪山村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "赛拉隆乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "皮袋湾村委会",
                code: "001",
            },
            VillageCode {
                name: "吐鲁村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "东大滩乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "酸茨沟村委会",
                code: "001",
            },
            VillageCode {
                name: "圈湾村委会",
                code: "002",
            },
            VillageCode {
                name: "边坡沟村委会",
                code: "003",
            },
            VillageCode {
                name: "东大滩村委会",
                code: "004",
            },
            VillageCode {
                name: "华郎村委会",
                code: "005",
            },
            VillageCode {
                name: "上圈湾村委会",
                code: "006",
            },
            VillageCode {
                name: "下四村委会",
                code: "007",
            },
            VillageCode {
                name: "水泉沟村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "毛藏乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "毛藏村委会",
                code: "001",
            },
            VillageCode {
                name: "华山村委会",
                code: "002",
            },
            VillageCode {
                name: "泉台村委会",
                code: "003",
            },
            VillageCode {
                name: "大小台村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "旦马乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "细水河村委会",
                code: "001",
            },
            VillageCode {
                name: "横路村委会",
                code: "002",
            },
            VillageCode {
                name: "细水村委会",
                code: "003",
            },
            VillageCode {
                name: "大水村委会",
                code: "004",
            },
            VillageCode {
                name: "康路村委会",
                code: "005",
            },
            VillageCode {
                name: "白羊圈村委会",
                code: "006",
            },
            VillageCode {
                name: "土塔村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "天祝建材厂",
        code: "020",
        villages: &[VillageCode {
            name: "天祝建材厂虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "天祝煤电公司",
        code: "021",
        villages: &[VillageCode {
            name: "天祝煤电公司虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_HX_008: [TownCode; 24] = [
    TownCode {
        name: "东街街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "交通巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "甘泉社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "长沙门社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "金安苑社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "饮马桥社区居民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "南街街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "南关社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "西来寺社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "佛城社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "泰安社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "丹霞社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "祁连社区居民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "西街街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "小寺庙社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "西站社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北环路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "新乐社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "北关社区居民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "北街街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "税亭社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东湖社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "王母宫社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "流泉社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "火车站街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "康乐社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "张火路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "下安社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "下安村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "东泉村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "街道直辖村民小组",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "梁家墩镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "梁家墩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "迎恩村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "五号村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "清凉寺村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "四闸村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "三工村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "刘家沟村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "六闸村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "太和村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "六号村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "上秦镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "李家湾村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "付家寨村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "王家墩村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "安里闸村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "八里堡村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "庙儿闸村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上秦村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "徐赵寨村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "高升庵村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "下秦村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "金家湾村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "哈寨子村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "东王堡村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "安家庄村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "缪家堡村村民委员会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "大满镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "柏家沟村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "新华村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新新村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "城西闸村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "石子坝村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "平顺村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "什信村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "马均村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "东闸村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "西闸村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "兰家寨村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "朝元村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "四号村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "紫家寨村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "小堡子村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "朱家庄村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "李家墩村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "汤家什村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "黑城子村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "大沟村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "新庙村村民委员会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "沙井镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "五个墩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上游村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "九闸村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "寺儿沟村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "水磨湾村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下利沟村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东四村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "南湾村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "沙井村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "南沟村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "先锋村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "新民村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "古城村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "小闸村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "三号村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "瞭马墩村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "民兴村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "柳树寨村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "西六村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "小河村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "西二村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "梁家堡村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "坝庙村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "东沟村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "东三村村民委员会",
                code: "025",
            },
            VillageCode {
                name: "东五村村民委员会",
                code: "026",
            },
            VillageCode {
                name: "兴隆村村民委员会",
                code: "027",
            },
            VillageCode {
                name: "双墩子村村民委员会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "乌江镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "平原堡社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "谢家湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "元丰村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "贾家寨村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "敬依村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "乌江村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "管寨村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "东湖村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "平原村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "安镇村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "天乐村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "大湾村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "小湾村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "永丰村村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "甘浚镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "祁连村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "小泉村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "甘浚村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "星光村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "三关村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "头号村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "光明村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "速展村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "工联村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "巴吉村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "晨光村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "谈家洼村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "西洞村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "东寺村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "高家庄村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "中沟村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "毛家湾村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "新墩镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "滨河新区白塔社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "滨河新区滨河社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "滨河新区青松社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "滨河新区五松园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "滨河新区崇文社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "滨河新区南华社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "流泉村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "北关村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "白塔村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "西关村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "青松村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "南华村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "新墩村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "双塔村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "园艺村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "双堡村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "柏闸村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "隋家寺村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "城儿闸村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "花儿村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "南闸村村民委员会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "党寨镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "上寨村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "下寨村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "马站村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "党寨村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "杨家墩村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "花家洼村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "烟墩村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "田家闸村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "中卫村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "雷寨村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "三十里店村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "陈家墩村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "汪家堡村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "廿里堡村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "陈寨村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "宋王寨村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "小寨村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "七号村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "沿沟村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "十号村村民委员会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "碱滩镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "老寺庙社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "普家庄村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "永星村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "古城村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "甲子墩村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "刘家庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "碱滩村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "幸福村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "草湖村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "二坝村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "三坝村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "杨家庄村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "太平村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "野水地村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "老仁坝村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "永定村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "三闸镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "庚名村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "三闸村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "二闸村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "瓦窑村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "高寨村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "天桥村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "韩家墩村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "符家堡村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "杨家寨村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "草原村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "新建村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "红沙窝村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "小满镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "九园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "五星村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "满家庙村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "店子闸村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "王其闸村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "古浪村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "康宁村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "金城村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "石桥村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "中华村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "张家寨村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "甘城村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "黎明村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "杨家闸村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "大柏村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "河南闸村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "小满村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "明永镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "沿河村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "武家闸村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "孙家闸村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "沤波村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "中南村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "明永村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "永和村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "上崖村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下崖村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "夹河村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "燎烟村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "永济村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "长安镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "万家墩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上头闸村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "庄墩村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "上四闸村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "五座桥村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "郭家堡村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "洪信村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "头号村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下二闸村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "前进村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "河满村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "八一村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "南关村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "龙渠乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "三清湾村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "木笼坝村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "龙首村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下堡村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "头闸村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "水源村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "墩源村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "保安村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "新胜村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "什八名村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "白城村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "高庙村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "安阳乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "苗家堡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "明家城村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "毛家寺村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "帖家城村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "郎家城村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "贺家城村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "王阜庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "五一村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "高寺儿村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "金王庄村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "花寨乡",
        code: "021",
        villages: &[
            VillageCode {
                name: "花寨村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "余家城村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "滚家庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "新城村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "滚家城村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "柏杨树村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "西阳村村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "靖安乡",
        code: "022",
        villages: &[
            VillageCode {
                name: "上堡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "新沟村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "靖平村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "靖安村村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "平山湖蒙古族乡",
        code: "023",
        villages: &[
            VillageCode {
                name: "平山湖村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "紫泥泉村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "红泉村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "张掖经济技术开发区",
        code: "024",
        villages: &[VillageCode {
            name: "经济开发区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_HX_009: [TownCode; 10] = [
    TownCode {
        name: "红湾寺镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "隆畅社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "红湾社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "裕兴社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "皇城镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "北极村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "北峰村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "北湾村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "红旗村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "向阳村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "营盘村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "大湖滩村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "皇城村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "水关村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "宁昌村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "长方村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "西水滩村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "金子滩村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "西城村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "河东村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "河西村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "东顶村村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "康乐镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "杨哥村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "德合隆村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "康丰村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "赛鼎村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "巴音村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "大草滩村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "红石窝村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "上游村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "隆丰村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "墩台子村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "桦树湾村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "青台子村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "榆木庄村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "马蹄藏族乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "大泉村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东城子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新升村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "南城子村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "石峰村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "圈坡村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "徐家湾村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "荷草村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "二道沟村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "楼庄子村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "正南沟村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "嘉卜斯村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "八一村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "芭蕉湾村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "黄草沟村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "肖家湾村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "横路沟村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "大都麻村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "马蹄村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "药草村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "长岭村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "小寺村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "大坡头村村民委员会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "白银蒙古族乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "白银村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东牛毛村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西牛毛村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "大河乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "光华村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "大滩村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "红湾村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东岭村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "西岭村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "西岔河村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "西河村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "营盘村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "天桥湾村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "松木滩村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "老虎沟村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "大岔村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "白庄子村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "喇嘛湾村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "西柳沟村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "旧寺湾村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "红边子村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "金畅河村村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "明花乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "前滩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "灰泉子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "刺窝泉村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "深井子村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "湖边子村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "贺家墩村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "黄土坡村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "南沟村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "中沙井村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "小海子村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "上井村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "双海子村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "许三湾村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "黄河湾村村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "祁丰蔵族乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "黄草坝村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "甘坝口村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "祁林村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "红山村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "青稞地村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "瓷窑口村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "观山村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "文殊村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "堡子滩村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "祁文村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "腰泉村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "珠龙关村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "陶丰村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "甘肃省绵羊育种场",
        code: "009",
        villages: &[VillageCode {
            name: "绵羊育种场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "张掖宝瓶河牧场",
        code: "010",
        villages: &[VillageCode {
            name: "宝瓶河牧场虚拟生活区",
            code: "001",
        }],
    },
];

static TOWNS_HX_010: [TownCode; 11] = [
    TownCode {
        name: "洪水镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "县府街居民委员会",
                code: "001",
            },
            VillageCode {
                name: "团结巷居民委员会",
                code: "002",
            },
            VillageCode {
                name: "东街居民委员会",
                code: "003",
            },
            VillageCode {
                name: "南街居民委员会",
                code: "004",
            },
            VillageCode {
                name: "西街居民委员会",
                code: "005",
            },
            VillageCode {
                name: "北街居民委员会",
                code: "006",
            },
            VillageCode {
                name: "嘉园社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "东圃社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "文昌社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "金山社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "城关村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "八一村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "乐民村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "益民村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "新丰村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "黄青村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "新墩村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "费寨村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "汤庄村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "吴庄村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "刘山村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "戎庄村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "烧房村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "叶官村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "里仁村村民委员会",
                code: "025",
            },
            VillageCode {
                name: "苏庄村村民委员会",
                code: "026",
            },
            VillageCode {
                name: "马庄村村民委员会",
                code: "027",
            },
            VillageCode {
                name: "于庄村村民委员会",
                code: "028",
            },
            VillageCode {
                name: "单庄村村民委员会",
                code: "029",
            },
            VillageCode {
                name: "李尤村村民委员会",
                code: "030",
            },
            VillageCode {
                name: "刘总旗村村民委员会",
                code: "031",
            },
            VillageCode {
                name: "上柴村村民委员会",
                code: "032",
            },
            VillageCode {
                name: "下柴村村民委员会",
                code: "033",
            },
            VillageCode {
                name: "友爱村村民委员会",
                code: "034",
            },
            VillageCode {
                name: "红石湾村村民委员会",
                code: "035",
            },
            VillageCode {
                name: "老号村村民委员会",
                code: "036",
            },
            VillageCode {
                name: "山城村村民委员会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "六坝镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "圆梦苑社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "复兴苑社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "富强苑社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "幸福苑社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "六坝村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "西上坝村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东上坝村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "四堡村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "四坝村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "铨将村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "韩武村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "海潮坝村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "王官村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "五坝村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "五庄村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "赵岗村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "柴庄村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "金山村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "北滩村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "新民村村民委员会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "新天镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "山寨村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "太平村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "马均村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "吴油村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "王什村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "王庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "韩营村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "周陆村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "上姚村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "下姚村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "新天堡村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "许沙村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "吕庄村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "杏园村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "马庄村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "钱寨村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "李寨村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "闫户村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "三寨村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "二寨村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "林山村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "薛寨村村民委员会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "南古镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "城东村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "城南村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "岔家堡村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "左卫村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "马蹄村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "彭刘村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "闫城村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "甘店村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "周庄村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "景会村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "高郝村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "柳谷村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "东朱村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "西朱村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "田庄村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "左卫营村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "王庄村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "杨坊村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "何庄村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "上花园村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "下花园村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "毛城村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "克寨村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "创业村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "黑崖头村村民委员会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "永固镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "八卦村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东街村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "南关村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "姚寨村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "西村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "滕庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "总寨村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "牛顺村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "邓庄村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "杨家树庄村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "三堡镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "库陀村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "团结村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "下二坝村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "展庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "全营村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "陈庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下吾旗村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "徐家寨村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "三堡村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "韩庄村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "易家新庄村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "任官村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "宏寺村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "何家沟村村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "南丰镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "炒面庄村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "秦庄村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "张连庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "胡庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "马营墩村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "边庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "双庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "渠湾村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "永丰村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "杨家圈村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "何庄村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "张家沟湾村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "玉带村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "铁城村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "黑山村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "冰沟村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "民联镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "龙山村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "郭家湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "杨庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "黄庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "高寨村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "河湾村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "屯粮村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "东升村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "复兴村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "下翟寨村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "上翟寨村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "雷台村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "贾西村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "刘信村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "顾寨村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "张明村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "东寨村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "西寨村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "太和村村民委员会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "顺化镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "顺化村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "青松村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "土城村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "油房村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "列四坝村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "旧堡村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "曹营村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "张宋村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "宗家寨村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "上天乐村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "新天乐村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "下天乐村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "松树村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "丰乐镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "武城村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "易家湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "卧马山村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "白庙村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "张满村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "新庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "刘庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "涌泉村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "双营村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "何庄村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "民乐生态工业园区",
        code: "011",
        villages: &[
            VillageCode {
                name: "圆梦苑社区居民委员会生活区",
                code: "001",
            },
            VillageCode {
                name: "复兴苑社区居民委员会生活区",
                code: "002",
            },
            VillageCode {
                name: "富强苑社区居民委员会生活区",
                code: "003",
            },
            VillageCode {
                name: "幸福苑社区居民委员会生活区",
                code: "004",
            },
            VillageCode {
                name: "开发区社区",
                code: "005",
            },
        ],
    },
];

static TOWNS_HX_011: [TownCode; 13] = [
    TownCode {
        name: "沙河镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "乐民社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东关街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "沙河街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "颐和社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "惠民社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "东寨村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "西寨村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "化音村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "兰家堡村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "沙河村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "西关村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "五三村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "何强村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "花园村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "闸湾村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "新丰村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "西头号村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "新民村村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "新华镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "大寨村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "长庄村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "富强村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "胜利村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "宣威村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "西街村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "向前村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "新柳村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "亢寨村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "新华村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "明泉村村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "蓼泉镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "唐湾村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "墩子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "湾子村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "蓼泉村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "寨子村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "新添村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "双泉村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下庄村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "平川镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "黄家堡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "五里墩村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "一工程村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "平川村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "三一村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "三二村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "三三村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "芦湾村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "四坝村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "贾家墩村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "板桥镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "土桥村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "红沟村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "友好村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "古城村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "板桥村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "西湾村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东柳村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "西柳村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "壕洼村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "鸭暖镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "小鸭村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "张湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "昭武村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "大鸭村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "暖泉村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "五泉村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "华强村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "白寨村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "曹庄村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "小屯村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "古寨村村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "倪家营镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "梨园村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "南台村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "高庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "马郡村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "汪家墩村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "倪家营村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下营村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "黄家湾村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "国营临泽农场",
        code: "008",
        villages: &[VillageCode {
            name: "农村虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "五泉林场",
        code: "009",
        villages: &[VillageCode {
            name: "五泉林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "沙河林场",
        code: "010",
        villages: &[VillageCode {
            name: "沙河林场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "小泉子治沙站",
        code: "011",
        villages: &[VillageCode {
            name: "小泉子治沙站虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "园艺场",
        code: "012",
        villages: &[VillageCode {
            name: "园艺场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "良种繁殖场",
        code: "013",
        villages: &[VillageCode {
            name: "良种场虚拟生活区",
            code: "001",
        }],
    },
];

static TOWNS_HX_012: [TownCode; 10] = [
    TownCode {
        name: "城关镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "新建东村居民委员会",
                code: "001",
            },
            VillageCode {
                name: "人民东路居民委员会",
                code: "002",
            },
            VillageCode {
                name: "医院西路居民委员会",
                code: "003",
            },
            VillageCode {
                name: "人民西路居民委员会",
                code: "004",
            },
            VillageCode {
                name: "长征路居民委员会",
                code: "005",
            },
            VillageCode {
                name: "新建南村居民委员会",
                code: "006",
            },
            VillageCode {
                name: "滨河居民委员会",
                code: "007",
            },
            VillageCode {
                name: "东苑居民委员会",
                code: "008",
            },
            VillageCode {
                name: "国庆村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "宣化镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "利丰村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "利号村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "贞号村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "高桥村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "寨子村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "站南村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "站北村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "蒋家庄村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "宣化村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "台子寺村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "王马湾村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "乐一村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "乐二村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "乐三村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "朱家堡村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "上庄村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "南华镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "南苑社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "小海子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "墩仁村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "大庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "南寨子村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "义和村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "先锋村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "智号村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "礼号村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "胜利村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "信号村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "南岔村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "成号村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "明水村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "明永村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "永进村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "巷道镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "渠口村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "果园村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "八一村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "三桥村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "巷道村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "东湾村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "南湾村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "槐树村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "沙坡村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "高地村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "小寺村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "五里墩村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "西八里村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "王家村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "东联村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "红联村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "太安村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "殷家庄村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "元号村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "元兴村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "元丰村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "正远村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "殷家桥村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "亨号村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "利沟村村民委员会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "合黎镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "五一村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "五二村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "五三村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "五四村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "六一村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "六二村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "六三村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "六四村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "七坝村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "八坝村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "骆驼城镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "碱泉子村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "梧桐村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新联村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "红新村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "团结村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "建康村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "永胜村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "前进村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "果树村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "新民村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "新建村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "骆驼城村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "西滩村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "新坝镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "暖泉村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "新沟村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "顺德村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下坝村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "照中村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "照二村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "照一村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "元山子村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "小坝村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "上坝村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "楼庄村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "西庄子村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "新生村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "官沟村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "曙光村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "许三湾村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "红沙河村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "光明村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "小泉村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "边沟村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "红崖子村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "霞光村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "西上村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "东上村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "和平村村民委员会",
                code: "025",
            },
            VillageCode {
                name: "西大村村民委员会",
                code: "026",
            },
            VillageCode {
                name: "东大村村民委员会",
                code: "027",
            },
            VillageCode {
                name: "古城村村民委员会",
                code: "028",
            },
            VillageCode {
                name: "六洋村村民委员会",
                code: "029",
            },
            VillageCode {
                name: "黄蒿村村民委员会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "黑泉镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "定安村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "定平村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "向阳村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "永丰村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "新开村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "黑泉村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "小坝村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "沙沟村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "镇江村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "九坝村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "十坝村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "胭脂堡村村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "罗城镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "张墩村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "花墙子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "河西村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "常丰村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "天城村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "侯庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "万丰村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "罗城村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "红山村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "桥儿湾村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "盐池村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "双丰村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "甘肃高台工业园区",
        code: "010",
        villages: &[VillageCode {
            name: "高台工业园区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_HX_013: [TownCode; 10] = [
    TownCode {
        name: "清泉镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "长城社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "北街社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "南街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "文化街社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "县府街社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "新城社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "世博社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "永宁社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "和盛社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "滨河社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "博兴社区居民委员会",
                code: "011",
            },
            VillageCode {
                name: "焉支社区居民委员会",
                code: "012",
            },
            VillageCode {
                name: "北滩村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "东街村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "西街村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "南关村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "南湖村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "南湾村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "双桥村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "清泉村村民委员会",
                code: "020",
            },
            VillageCode {
                name: "祁店村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "拾号村村民委员会",
                code: "022",
            },
            VillageCode {
                name: "北湾村村民委员会",
                code: "023",
            },
            VillageCode {
                name: "郑庄村村民委员会",
                code: "024",
            },
            VillageCode {
                name: "郇庄村村民委员会",
                code: "025",
            },
            VillageCode {
                name: "城北村村民委员会",
                code: "026",
            },
            VillageCode {
                name: "红寺湖村村民委员会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "位奇镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "位奇村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "东湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "十里堡村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "二十里堡村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "高寨村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "四坝村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "永兴村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "马寨村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "张湾村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "孙家营村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "暖泉村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "朱湾村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "柳荫村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "芦堡村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "汪庄村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "新开村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "侯山村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "霍城镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "下西山村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上西山村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "双湖村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "沙沟村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "东关村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "西关村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "周庄村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "王庄村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "西坡村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "下河西村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "上河西村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "刘庄村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "杜庄村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "泉头村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "东山村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "陈户镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "三十里堡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "刘伏村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "西门村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东门村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "沙河湾村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "王城村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "张庄村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "岸头村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "陈户村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "周坑村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "盘山村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "寺沟村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "范营村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "孙营村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "山湾村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "焉支村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "大马营镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "马营村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "磨湾村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "新墩村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "夹河村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "圈沟村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "窑坡村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "双泉村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "前山村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "新泉村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "楼庄村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "城南村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "花寨村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "高湖村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "上山湾村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "上河村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "中河村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "下河村村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "东乐镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "山羊堡村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "西屯村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "城西村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "城东村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "大寨村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "小寨村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "五墩村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "十里堡村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "大桥村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "静安村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "老军乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "焦湾村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "老军村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "祝庄村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "孙庄村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "李泉村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "潘庄村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "郭泉村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "羊虎沟村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "硖口村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "丰城村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "李桥乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "杨坝村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "高庙村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "吴宁村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "巴寨村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "河湾村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下寨村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上寨村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "周庄村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "东沟村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "西沟村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "国营山丹农场",
        code: "009",
        villages: &[VillageCode {
            name: "山丹农场虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "中牧公司山丹马场",
        code: "010",
        villages: &[
            VillageCode {
                name: "马场一场居委会",
                code: "001",
            },
            VillageCode {
                name: "马场二场居委会",
                code: "002",
            },
            VillageCode {
                name: "马场三场居委会",
                code: "003",
            },
            VillageCode {
                name: "马场四场居委会",
                code: "004",
            },
            VillageCode {
                name: "总场居委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_HX_014: [TownCode; 24] = [
    TownCode {
        name: "东北街街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "民主街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北新街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "汉唐街北社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "东南街街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "东文化街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东关苑社区居委会",
                code: "002",
            },
            VillageCode {
                name: "汉唐街南社区居委会",
                code: "003",
            },
            VillageCode {
                name: "金泉路社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "工业园街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "解放路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "祁连路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "南苑社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "新城街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "广厦社区居委会",
                code: "001",
            },
            VillageCode {
                name: "中深沟社区居委会",
                code: "002",
            },
            VillageCode {
                name: "阳关路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "官北沟社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "西北街街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "仓后街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "同德巷社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金河社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "西南街街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "富康路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西文化街社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "玉门油田生活基地街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "飞天路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "肃州路社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "西洞镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "新东村委会",
                code: "001",
            },
            VillageCode {
                name: "新西村委会",
                code: "002",
            },
            VillageCode {
                name: "滚坝村委会",
                code: "003",
            },
            VillageCode {
                name: "西洞村委会",
                code: "004",
            },
            VillageCode {
                name: "罗马村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "清水镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "盐池村委会",
                code: "001",
            },
            VillageCode {
                name: "中寨村委会",
                code: "002",
            },
            VillageCode {
                name: "清水村委会",
                code: "003",
            },
            VillageCode {
                name: "半坡村委会",
                code: "004",
            },
            VillageCode {
                name: "榆林坝村委会",
                code: "005",
            },
            VillageCode {
                name: "沙山村委会",
                code: "006",
            },
            VillageCode {
                name: "马营村委会",
                code: "007",
            },
            VillageCode {
                name: "屯升村委会",
                code: "008",
            },
            VillageCode {
                name: "西一村委会",
                code: "009",
            },
            VillageCode {
                name: "西二村委会",
                code: "010",
            },
            VillageCode {
                name: "西三村委会",
                code: "011",
            },
            VillageCode {
                name: "上寨村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "总寨镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "三奇堡村委会",
                code: "001",
            },
            VillageCode {
                name: "西店村委会",
                code: "002",
            },
            VillageCode {
                name: "沙河村委会",
                code: "003",
            },
            VillageCode {
                name: "沙格楞村委会",
                code: "004",
            },
            VillageCode {
                name: "单闸村委会",
                code: "005",
            },
            VillageCode {
                name: "总寨村委会",
                code: "006",
            },
            VillageCode {
                name: "双闸村委会",
                code: "007",
            },
            VillageCode {
                name: "清泉村委会",
                code: "008",
            },
            VillageCode {
                name: "店闸村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "金佛寺镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "上河清村委会",
                code: "001",
            },
            VillageCode {
                name: "西寨村委会",
                code: "002",
            },
            VillageCode {
                name: "二坝村委会",
                code: "003",
            },
            VillageCode {
                name: "红寺堡村委会",
                code: "004",
            },
            VillageCode {
                name: "丰乐口村委会",
                code: "005",
            },
            VillageCode {
                name: "红山堡村委会",
                code: "006",
            },
            VillageCode {
                name: "观山村委会",
                code: "007",
            },
            VillageCode {
                name: "上三截村委会",
                code: "008",
            },
            VillageCode {
                name: "金佛寺村委会",
                code: "009",
            },
            VillageCode {
                name: "下四截村委会",
                code: "010",
            },
            VillageCode {
                name: "观山口村委会",
                code: "011",
            },
            VillageCode {
                name: "小庄村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "上坝镇",
        code: "012",
        villages: &[
            VillageCode {
                name: "新上村委会",
                code: "001",
            },
            VillageCode {
                name: "光辉村委会",
                code: "002",
            },
            VillageCode {
                name: "下坝村委会",
                code: "003",
            },
            VillageCode {
                name: "上坝村委会",
                code: "004",
            },
            VillageCode {
                name: "小沟村委会",
                code: "005",
            },
            VillageCode {
                name: "旧墩村委会",
                code: "006",
            },
            VillageCode {
                name: "营尔村委会",
                code: "007",
            },
            VillageCode {
                name: "东沟村委会",
                code: "008",
            },
            VillageCode {
                name: "上红村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "三墩镇",
        code: "013",
        villages: &[
            VillageCode {
                name: "三墩村委会",
                code: "001",
            },
            VillageCode {
                name: "夹边沟村委会",
                code: "002",
            },
            VillageCode {
                name: "长城村委会",
                code: "003",
            },
            VillageCode {
                name: "二墩堡村委会",
                code: "004",
            },
            VillageCode {
                name: "二墩村委会",
                code: "005",
            },
            VillageCode {
                name: "双桥村委会",
                code: "006",
            },
            VillageCode {
                name: "中渠村委会",
                code: "007",
            },
            VillageCode {
                name: "双塔村委会",
                code: "008",
            },
            VillageCode {
                name: "下坝村委会",
                code: "009",
            },
            VillageCode {
                name: "仰沟村委会",
                code: "010",
            },
            VillageCode {
                name: "临水村委会",
                code: "011",
            },
            VillageCode {
                name: "闇门村委会",
                code: "012",
            },
            VillageCode {
                name: "鸳鸯村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "银达镇",
        code: "014",
        villages: &[
            VillageCode {
                name: "拐坝桥村委会",
                code: "001",
            },
            VillageCode {
                name: "银达村委会",
                code: "002",
            },
            VillageCode {
                name: "蒲上沟村委会",
                code: "003",
            },
            VillageCode {
                name: "佘新村委会",
                code: "004",
            },
            VillageCode {
                name: "谭家堡村委会",
                code: "005",
            },
            VillageCode {
                name: "杨洪村委会",
                code: "006",
            },
            VillageCode {
                name: "明沙窝村委会",
                code: "007",
            },
            VillageCode {
                name: "黑水沟村委会",
                code: "008",
            },
            VillageCode {
                name: "怀茂村委会",
                code: "009",
            },
            VillageCode {
                name: "六分村委会",
                code: "010",
            },
            VillageCode {
                name: "怀中村委会",
                code: "011",
            },
            VillageCode {
                name: "关明村委会",
                code: "012",
            },
            VillageCode {
                name: "西坝村委会",
                code: "013",
            },
            VillageCode {
                name: "南坝村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "西峰镇",
        code: "015",
        villages: &[
            VillageCode {
                name: "塔尔寺村委会",
                code: "001",
            },
            VillageCode {
                name: "西峰寺村委会",
                code: "002",
            },
            VillageCode {
                name: "中深沟村委会",
                code: "003",
            },
            VillageCode {
                name: "官北沟村委会",
                code: "004",
            },
            VillageCode {
                name: "苜场沟村委会",
                code: "005",
            },
            VillageCode {
                name: "张良沟村委会",
                code: "006",
            },
            VillageCode {
                name: "蒲莱村委会",
                code: "007",
            },
            VillageCode {
                name: "沙子坝村委会",
                code: "008",
            },
            VillageCode {
                name: "侯家沟村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "泉湖镇",
        code: "016",
        villages: &[
            VillageCode {
                name: "四坝村委会",
                code: "001",
            },
            VillageCode {
                name: "沙滩村委会",
                code: "002",
            },
            VillageCode {
                name: "花寨村委会",
                code: "003",
            },
            VillageCode {
                name: "头墩村委会",
                code: "004",
            },
            VillageCode {
                name: "永久村委会",
                code: "005",
            },
            VillageCode {
                name: "营门村委会",
                code: "006",
            },
            VillageCode {
                name: "春光村委会",
                code: "007",
            },
            VillageCode {
                name: "泉湖村委会",
                code: "008",
            },
            VillageCode {
                name: "水磨沟村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "果园镇",
        code: "017",
        villages: &[
            VillageCode {
                name: "边湾农场社区居委会",
                code: "001",
            },
            VillageCode {
                name: "果园沟村委会",
                code: "002",
            },
            VillageCode {
                name: "高闸沟村委会",
                code: "003",
            },
            VillageCode {
                name: "中所沟村委会",
                code: "004",
            },
            VillageCode {
                name: "西沟村委会",
                code: "005",
            },
            VillageCode {
                name: "丁家闸村委会",
                code: "006",
            },
            VillageCode {
                name: "小坝沟村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "下河清镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "下河清农场社区居委会",
                code: "001",
            },
            VillageCode {
                name: "楼庄村委会",
                code: "002",
            },
            VillageCode {
                name: "皇城村委会",
                code: "003",
            },
            VillageCode {
                name: "紫金村委会",
                code: "004",
            },
            VillageCode {
                name: "五坝村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "铧尖镇",
        code: "019",
        villages: &[
            VillageCode {
                name: "铧尖村委会",
                code: "001",
            },
            VillageCode {
                name: "小沙渠村委会",
                code: "002",
            },
            VillageCode {
                name: "上三沟村委会",
                code: "003",
            },
            VillageCode {
                name: "漫水滩村委会",
                code: "004",
            },
            VillageCode {
                name: "集泉村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "东洞镇",
        code: "020",
        villages: &[
            VillageCode {
                name: "东洞村委会",
                code: "001",
            },
            VillageCode {
                name: "四号村委会",
                code: "002",
            },
            VillageCode {
                name: "旧沟村委会",
                code: "003",
            },
            VillageCode {
                name: "小庙村委会",
                code: "004",
            },
            VillageCode {
                name: "新沟村委会",
                code: "005",
            },
            VillageCode {
                name: "棉花滩村委会",
                code: "006",
            },
            VillageCode {
                name: "石灰窑村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "丰乐镇",
        code: "021",
        villages: &[
            VillageCode {
                name: "大庄村委会",
                code: "001",
            },
            VillageCode {
                name: "前所村委会",
                code: "002",
            },
            VillageCode {
                name: "涌泉村委会",
                code: "003",
            },
            VillageCode {
                name: "中截村委会",
                code: "004",
            },
            VillageCode {
                name: "二坝村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "黄泥堡乡",
        code: "022",
        villages: &[
            VillageCode {
                name: "沙枣园子村委会",
                code: "001",
            },
            VillageCode {
                name: "新湖村委会",
                code: "002",
            },
            VillageCode {
                name: "黄泥堡村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "酒泉经济技术开发区",
        code: "023",
        villages: &[VillageCode {
            name: "酒泉经济技术开发区虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "十号基地",
        code: "024",
        villages: &[VillageCode {
            name: "十号基地虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_HX_015: [TownCode; 10] = [
    TownCode {
        name: "中东镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "上三分村委会",
                code: "001",
            },
            VillageCode {
                name: "官营沟村委会",
                code: "002",
            },
            VillageCode {
                name: "上四分村委会",
                code: "003",
            },
            VillageCode {
                name: "三湾沟村委会",
                code: "004",
            },
            VillageCode {
                name: "营田地村委会",
                code: "005",
            },
            VillageCode {
                name: "团结村委会",
                code: "006",
            },
            VillageCode {
                name: "中五村委会",
                code: "007",
            },
            VillageCode {
                name: "下四分村委会",
                code: "008",
            },
            VillageCode {
                name: "梧桐坝村委会",
                code: "009",
            },
            VillageCode {
                name: "王子庄村委会",
                code: "010",
            },
            VillageCode {
                name: "中东镇潮湖生活区",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "鼎新镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "上元村委会",
                code: "001",
            },
            VillageCode {
                name: "洪号村委会",
                code: "002",
            },
            VillageCode {
                name: "新西村委会",
                code: "003",
            },
            VillageCode {
                name: "进化村委会",
                code: "004",
            },
            VillageCode {
                name: "头分村委会",
                code: "005",
            },
            VillageCode {
                name: "东明村委会",
                code: "006",
            },
            VillageCode {
                name: "新民村委会",
                code: "007",
            },
            VillageCode {
                name: "友好村委会",
                code: "008",
            },
            VillageCode {
                name: "夹墩湾村委会",
                code: "009",
            },
            VillageCode {
                name: "芨芨村委会",
                code: "010",
            },
            VillageCode {
                name: "双树村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "金塔镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "东南街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东北街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西南街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "西北街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金大村委会",
                code: "005",
            },
            VillageCode {
                name: "塔院村委会",
                code: "006",
            },
            VillageCode {
                name: "边沟村委会",
                code: "007",
            },
            VillageCode {
                name: "西沟村委会",
                code: "008",
            },
            VillageCode {
                name: "东星村委会",
                code: "009",
            },
            VillageCode {
                name: "红光村委会",
                code: "010",
            },
            VillageCode {
                name: "中杰村委会",
                code: "011",
            },
            VillageCode {
                name: "营泉村委会",
                code: "012",
            },
            VillageCode {
                name: "胜利村委会",
                code: "013",
            },
            VillageCode {
                name: "五星村委会",
                code: "014",
            },
            VillageCode {
                name: "上杰村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "东坝镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "梧盛村委会",
                code: "001",
            },
            VillageCode {
                name: "渠东村委会",
                code: "002",
            },
            VillageCode {
                name: "三上村委会",
                code: "003",
            },
            VillageCode {
                name: "大坝村委会",
                code: "004",
            },
            VillageCode {
                name: "下黑树窝村委会",
                code: "005",
            },
            VillageCode {
                name: "三下村委会",
                code: "006",
            },
            VillageCode {
                name: "烽火坪村委会",
                code: "007",
            },
            VillageCode {
                name: "小河口村委会",
                code: "008",
            },
            VillageCode {
                name: "永光村委会",
                code: "009",
            },
            VillageCode {
                name: "榆树沟村委会",
                code: "010",
            },
            VillageCode {
                name: "古墩子村委会",
                code: "011",
            },
            VillageCode {
                name: "下新坝村委会",
                code: "012",
            },
            VillageCode {
                name: "大柳林村委会",
                code: "013",
            },
            VillageCode {
                name: "红星村委会",
                code: "014",
            },
            VillageCode {
                name: "天潭村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "航天镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "航天村委会",
                code: "001",
            },
            VillageCode {
                name: "西和村委会",
                code: "002",
            },
            VillageCode {
                name: "双城村委会",
                code: "003",
            },
            VillageCode {
                name: "三星村委会",
                code: "004",
            },
            VillageCode {
                name: "天仓村委会",
                code: "005",
            },
            VillageCode {
                name: "大湾村委会",
                code: "006",
            },
            VillageCode {
                name: "永胜村委会",
                code: "007",
            },
            VillageCode {
                name: "东岔村委会",
                code: "008",
            },
            VillageCode {
                name: "永联村委会",
                code: "009",
            },
            VillageCode {
                name: "营盘村委会",
                code: "010",
            },
            VillageCode {
                name: "中丰村委会",
                code: "011",
            },
            VillageCode {
                name: "双湾村委会",
                code: "012",
            },
            VillageCode {
                name: "金关村委会",
                code: "013",
            },
            VillageCode {
                name: "沙红山村委会",
                code: "014",
            },
            VillageCode {
                name: "十四号基地管理区",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "大庄子镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "大庄子村委会",
                code: "001",
            },
            VillageCode {
                name: "牛头湾村委会",
                code: "002",
            },
            VillageCode {
                name: "三墩村委会",
                code: "003",
            },
            VillageCode {
                name: "永丰村委会",
                code: "004",
            },
            VillageCode {
                name: "头墩村委会",
                code: "005",
            },
            VillageCode {
                name: "双新村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "西坝镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "生地湾社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西移村委会",
                code: "002",
            },
            VillageCode {
                name: "西红村委会",
                code: "003",
            },
            VillageCode {
                name: "晨光村委会",
                code: "004",
            },
            VillageCode {
                name: "金马村委会",
                code: "005",
            },
            VillageCode {
                name: "张家墩村委会",
                code: "006",
            },
            VillageCode {
                name: "高桥村委会",
                code: "007",
            },
            VillageCode {
                name: "共和村委会",
                code: "008",
            },
            VillageCode {
                name: "双雁村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "古城乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "头号村委会",
                code: "001",
            },
            VillageCode {
                name: "移庆村委会",
                code: "002",
            },
            VillageCode {
                name: "四分村委会",
                code: "003",
            },
            VillageCode {
                name: "上东沟村委会",
                code: "004",
            },
            VillageCode {
                name: "旧寺墩村委会",
                code: "005",
            },
            VillageCode {
                name: "古城村委会",
                code: "006",
            },
            VillageCode {
                name: "新光村委会",
                code: "007",
            },
            VillageCode {
                name: "新丰村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "羊井子湾乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "金新村委会",
                code: "001",
            },
            VillageCode {
                name: "羊井子湾村委会",
                code: "002",
            },
            VillageCode {
                name: "大泉湾村委会",
                code: "003",
            },
            VillageCode {
                name: "榆树井村委会",
                code: "004",
            },
            VillageCode {
                name: "黄茨梁村委会",
                code: "005",
            },
            VillageCode {
                name: "双古城村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "工业集中区",
        code: "010",
        villages: &[
            VillageCode {
                name: "金鑫工业园社区",
                code: "001",
            },
            VillageCode {
                name: "北河湾矿化工业园社区",
                code: "002",
            },
            VillageCode {
                name: "穿山驯矿化工业园社区",
                code: "003",
            },
        ],
    },
];

static TOWNS_HX_016: [TownCode; 15] = [
    TownCode {
        name: "渊泉镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "疏勒社区居委会",
                code: "001",
            },
            VillageCode {
                name: "榆林社区居委会",
                code: "002",
            },
            VillageCode {
                name: "渊泉社区居委会",
                code: "003",
            },
            VillageCode {
                name: "祁连社区居委会",
                code: "004",
            },
            VillageCode {
                name: "常乐社区居委会",
                code: "005",
            },
            VillageCode {
                name: "张芝社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "柳园镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "公园路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "团结巷社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "三道沟镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "三道沟社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东湖村委会",
                code: "002",
            },
            VillageCode {
                name: "四道沟村委会",
                code: "003",
            },
            VillageCode {
                name: "三道沟村委会",
                code: "004",
            },
            VillageCode {
                name: "山水梁村委会",
                code: "005",
            },
            VillageCode {
                name: "北滩村委会",
                code: "006",
            },
            VillageCode {
                name: "五四村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "南岔镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "农垦社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "十工村委会",
                code: "002",
            },
            VillageCode {
                name: "九南村委会",
                code: "003",
            },
            VillageCode {
                name: "九北村委会",
                code: "004",
            },
            VillageCode {
                name: "八工村委会",
                code: "005",
            },
            VillageCode {
                name: "开工村委会",
                code: "006",
            },
            VillageCode {
                name: "六工村委会",
                code: "007",
            },
            VillageCode {
                name: "七工村委会",
                code: "008",
            },
            VillageCode {
                name: "南岔村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "锁阳城镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "农丰村委会",
                code: "001",
            },
            VillageCode {
                name: "中渠村委会",
                code: "002",
            },
            VillageCode {
                name: "新沟村委会",
                code: "003",
            },
            VillageCode {
                name: "常乐村委会",
                code: "004",
            },
            VillageCode {
                name: "南坝村委会",
                code: "005",
            },
            VillageCode {
                name: "堡子村委会",
                code: "006",
            },
            VillageCode {
                name: "北桥子村委会",
                code: "007",
            },
            VillageCode {
                name: "东巴兔村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "瓜州镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "农垦企业社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "三工村委会",
                code: "002",
            },
            VillageCode {
                name: "瓜州村委会",
                code: "003",
            },
            VillageCode {
                name: "头工村委会",
                code: "004",
            },
            VillageCode {
                name: "南苑村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "西湖镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "农垦四工农场社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "城北村委会",
                code: "002",
            },
            VillageCode {
                name: "北沟村委会",
                code: "003",
            },
            VillageCode {
                name: "中沟村委会",
                code: "004",
            },
            VillageCode {
                name: "四工村委会",
                code: "005",
            },
            VillageCode {
                name: "向阳村委会",
                code: "006",
            },
            VillageCode {
                name: "西湖村委会",
                code: "007",
            },
            VillageCode {
                name: "安康村委会",
                code: "008",
            },
            VillageCode {
                name: "农垦西湖农场生活区",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "河东镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "桥湾社区居委会",
                code: "001",
            },
            VillageCode {
                name: "双泉村委会",
                code: "002",
            },
            VillageCode {
                name: "五道沟村委会",
                code: "003",
            },
            VillageCode {
                name: "六道沟村委会",
                code: "004",
            },
            VillageCode {
                name: "七道沟村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "双塔镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "金河村委会",
                code: "001",
            },
            VillageCode {
                name: "新华村委会",
                code: "002",
            },
            VillageCode {
                name: "古城村委会",
                code: "003",
            },
            VillageCode {
                name: "福泉村委会",
                code: "004",
            },
            VillageCode {
                name: "月牙墩村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "腰站子东乡族镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "辉铜村委会",
                code: "001",
            },
            VillageCode {
                name: "草湖沟村委会",
                code: "002",
            },
            VillageCode {
                name: "唐墩村委会",
                code: "003",
            },
            VillageCode {
                name: "腰站子村委会",
                code: "004",
            },
            VillageCode {
                name: "扎花营村委会",
                code: "005",
            },
            VillageCode {
                name: "马家泉村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "布隆吉乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "九上村委会",
                code: "001",
            },
            VillageCode {
                name: "九下村委会",
                code: "002",
            },
            VillageCode {
                name: "布隆吉村委会",
                code: "003",
            },
            VillageCode {
                name: "潘家庄村委会",
                code: "004",
            },
            VillageCode {
                name: "双塔村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "七墩回族东乡族乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "三墩村委会",
                code: "001",
            },
            VillageCode {
                name: "锦华村委会",
                code: "002",
            },
            VillageCode {
                name: "汇源村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "广至藏族乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "卓园村委会",
                code: "001",
            },
            VillageCode {
                name: "卓尼村委会",
                code: "002",
            },
            VillageCode {
                name: "洮砚村委会",
                code: "003",
            },
            VillageCode {
                name: "岷县村委会",
                code: "004",
            },
            VillageCode {
                name: "新堡村委会",
                code: "005",
            },
            VillageCode {
                name: "临潭村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "沙河回族乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "常顺村委会",
                code: "001",
            },
            VillageCode {
                name: "沙河村委会",
                code: "002",
            },
            VillageCode {
                name: "临河村委会",
                code: "003",
            },
            VillageCode {
                name: "民和村委会",
                code: "004",
            },
            VillageCode {
                name: "河洲村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "梁湖乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "小宛农场社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "双州村委会",
                code: "002",
            },
            VillageCode {
                name: "雁湖村委会",
                code: "003",
            },
            VillageCode {
                name: "陈家庄村委会",
                code: "004",
            },
            VillageCode {
                name: "银河村委会",
                code: "005",
            },
            VillageCode {
                name: "岷州村委会",
                code: "006",
            },
            VillageCode {
                name: "金梧村委会",
                code: "007",
            },
            VillageCode {
                name: "青山村委会",
                code: "008",
            },
            VillageCode {
                name: "小宛村委会",
                code: "009",
            },
        ],
    },
];

static TOWNS_HX_017: [TownCode; 4] = [
    TownCode {
        name: "党城湾镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "紫亭社区居委会",
                code: "001",
            },
            VillageCode {
                name: "巴音社区居委会",
                code: "002",
            },
            VillageCode {
                name: "城关村委会",
                code: "003",
            },
            VillageCode {
                name: "东山村委会",
                code: "004",
            },
            VillageCode {
                name: "党城村委会",
                code: "005",
            },
            VillageCode {
                name: "城北村委会",
                code: "006",
            },
            VillageCode {
                name: "青山道村委会",
                code: "007",
            },
            VillageCode {
                name: "马场村委会",
                code: "008",
            },
            VillageCode {
                name: "红柳峡村委会",
                code: "009",
            },
            VillageCode {
                name: "浩布勒格村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "马鬃山镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "巴音布勒格村",
                code: "001",
            },
            VillageCode {
                name: "明水村委会",
                code: "002",
            },
            VillageCode {
                name: "云母头村委会",
                code: "003",
            },
            VillageCode {
                name: "马鬃山村委会",
                code: "004",
            },
            VillageCode {
                name: "饮马峡村委会",
                code: "005",
            },
            VillageCode {
                name: "金庙沟村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "盐池湾乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "纳仁郭勒村委会",
                code: "001",
            },
            VillageCode {
                name: "雕尔力吉村委会",
                code: "002",
            },
            VillageCode {
                name: "乌兰布拉格村委会",
                code: "003",
            },
            VillageCode {
                name: "奎腾郭勒村委会",
                code: "004",
            },
            VillageCode {
                name: "阿尔格勒泰村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "石包城乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "鹰嘴山村委会",
                code: "001",
            },
            VillageCode {
                name: "石包城村委会",
                code: "002",
            },
            VillageCode {
                name: "石坂墩村委会",
                code: "003",
            },
            VillageCode {
                name: "哈什哈尔村委会",
                code: "004",
            },
            VillageCode {
                name: "公岔村委会",
                code: "005",
            },
            VillageCode {
                name: "鱼儿红村委会",
                code: "006",
            },
            VillageCode {
                name: "金沟村委会",
                code: "007",
            },
        ],
    },
];

static TOWNS_HX_018: [TownCode; 5] = [
    TownCode {
        name: "红柳湾镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "团结社区居委会",
                code: "001",
            },
            VillageCode {
                name: "民主社区居委会",
                code: "002",
            },
            VillageCode {
                name: "金山社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新村社区居委会",
                code: "004",
            },
            VillageCode {
                name: "加尔乌宗村民委员会",
                code: "005",
            },
            VillageCode {
                name: "大坝图村民委员会",
                code: "006",
            },
            VillageCode {
                name: "红柳湾村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "阿克旗乡",
        code: "002",
        villages: &[
            VillageCode {
                name: "东格列克村民委员会",
                code: "001",
            },
            VillageCode {
                name: "安南坝村民委员会",
                code: "002",
            },
            VillageCode {
                name: "多坝沟村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "阿勒腾乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "哈尔腾村民委员会",
                code: "001",
            },
            VillageCode {
                name: "乌呼图村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "阿伊纳乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "苏干湖村民委员会",
                code: "001",
            },
            VillageCode {
                name: "阿克塔木村民委员会",
                code: "002",
            },
            VillageCode {
                name: "塞什腾村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "阿克塞县工业园区",
        code: "005",
        villages: &[VillageCode {
            name: "阿克塞县工业园区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_HX_019: [TownCode; 16] = [
    TownCode {
        name: "新市区街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "南街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "玉关路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "兰新社区居委会",
                code: "004",
            },
            VillageCode {
                name: "铁人路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "迎宾路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "农垦建司社区居委会",
                code: "007",
            },
            VillageCode {
                name: "官庄社区居委会",
                code: "008",
            },
            VillageCode {
                name: "人民路社区居委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "玉门镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "沙梁子社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南门村委会",
                code: "002",
            },
            VillageCode {
                name: "东渠村委会",
                code: "003",
            },
            VillageCode {
                name: "代家滩村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "赤金镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "铁人路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "铁人村委会",
                code: "002",
            },
            VillageCode {
                name: "营田村委会",
                code: "003",
            },
            VillageCode {
                name: "东湖村委会",
                code: "004",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "005",
            },
            VillageCode {
                name: "金峡村委会",
                code: "006",
            },
            VillageCode {
                name: "新风村委会",
                code: "007",
            },
            VillageCode {
                name: "光明村委会",
                code: "008",
            },
            VillageCode {
                name: "西湖村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "花海镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "和政社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南渠村委会",
                code: "002",
            },
            VillageCode {
                name: "黄水桥村委会",
                code: "003",
            },
            VillageCode {
                name: "中渠村委会",
                code: "004",
            },
            VillageCode {
                name: "西泉村委会",
                code: "005",
            },
            VillageCode {
                name: "金湾村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "老君庙镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "钻井路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "安康路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "友谊路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "自由路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "民族路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "和平路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "建设路社区居委会",
                code: "007",
            },
            VillageCode {
                name: "青年路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "幸福路社区居委会",
                code: "009",
            },
            VillageCode {
                name: "广场路社区居委会",
                code: "010",
            },
            VillageCode {
                name: "白杨河村委会",
                code: "011",
            },
            VillageCode {
                name: "跃进村委会",
                code: "012",
            },
            VillageCode {
                name: "清泉村委会",
                code: "013",
            },
            VillageCode {
                name: "新民堡村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "黄闸湾镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "梁子沟村委会",
                code: "001",
            },
            VillageCode {
                name: "泽湖村委会",
                code: "002",
            },
            VillageCode {
                name: "黄闸湾村委会",
                code: "003",
            },
            VillageCode {
                name: "黄花营村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "下西号镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "河东村委会",
                code: "001",
            },
            VillageCode {
                name: "下东号村委会",
                code: "002",
            },
            VillageCode {
                name: "塔尔湾村委会",
                code: "003",
            },
            VillageCode {
                name: "石河子村委会",
                code: "004",
            },
            VillageCode {
                name: "西红号村委会",
                code: "005",
            },
            VillageCode {
                name: "川北镇村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "柳河镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "官庄子村委会",
                code: "001",
            },
            VillageCode {
                name: "二道沟村委会",
                code: "002",
            },
            VillageCode {
                name: "东风村委会",
                code: "003",
            },
            VillageCode {
                name: "红旗村委会",
                code: "004",
            },
            VillageCode {
                name: "蘑菇滩村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "昌马镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "水峡村委会",
                code: "001",
            },
            VillageCode {
                name: "南湖村委会",
                code: "002",
            },
            VillageCode {
                name: "上游村委会",
                code: "003",
            },
            VillageCode {
                name: "昌马村委会",
                code: "004",
            },
            VillageCode {
                name: "东湾村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "柳湖镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "华西村委会",
                code: "001",
            },
            VillageCode {
                name: "富民村委会",
                code: "002",
            },
            VillageCode {
                name: "岷州村委会",
                code: "003",
            },
            VillageCode {
                name: "小康村委会",
                code: "004",
            },
            VillageCode {
                name: "兴旺村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "六墩镇",
        code: "011",
        villages: &[
            VillageCode {
                name: "柳北村委会",
                code: "001",
            },
            VillageCode {
                name: "昌和村委会",
                code: "002",
            },
            VillageCode {
                name: "安康村委会",
                code: "003",
            },
            VillageCode {
                name: "昌盛村委会",
                code: "004",
            },
            VillageCode {
                name: "安和村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "小金湾东乡族乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "龙泉村委会",
                code: "001",
            },
            VillageCode {
                name: "东兴村委会",
                code: "002",
            },
            VillageCode {
                name: "富源村委会",
                code: "003",
            },
            VillageCode {
                name: "金柳村委会",
                code: "004",
            },
            VillageCode {
                name: "马家峪村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "独山子东乡族乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "源泉村委会",
                code: "001",
            },
            VillageCode {
                name: "春柳村委会",
                code: "002",
            },
            VillageCode {
                name: "金泉村委会",
                code: "003",
            },
            VillageCode {
                name: "金旺村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "国营饮马农场",
        code: "014",
        villages: &[VillageCode {
            name: "国营饮马农场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "国营黄花农场",
        code: "015",
        villages: &[VillageCode {
            name: "国营黄花农场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "甘肃矿区",
        code: "016",
        villages: &[VillageCode {
            name: "甘肃矿区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_HX_020: [TownCode; 10] = [
    TownCode {
        name: "七里镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "祁连社区居委会",
                code: "001",
            },
            VillageCode {
                name: "白马塔村委会",
                code: "002",
            },
            VillageCode {
                name: "杜家墩村委会",
                code: "003",
            },
            VillageCode {
                name: "三号桥村委会",
                code: "004",
            },
            VillageCode {
                name: "铁家堡村委会",
                code: "005",
            },
            VillageCode {
                name: "南台堡村委会",
                code: "006",
            },
            VillageCode {
                name: "大庙村委会",
                code: "007",
            },
            VillageCode {
                name: "秦家湾村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "沙州镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "梨园社区居委会",
                code: "001",
            },
            VillageCode {
                name: "文庙社区居委会",
                code: "002",
            },
            VillageCode {
                name: "北台社区居委会",
                code: "003",
            },
            VillageCode {
                name: "红当社区居委会",
                code: "004",
            },
            VillageCode {
                name: "古城社区居委会",
                code: "005",
            },
            VillageCode {
                name: "桥北社区居委会",
                code: "006",
            },
            VillageCode {
                name: "北街社区居委会",
                code: "007",
            },
            VillageCode {
                name: "南街社区居委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "肃州镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "祁家桥村委会",
                code: "001",
            },
            VillageCode {
                name: "高台堡村委会",
                code: "002",
            },
            VillageCode {
                name: "魏家桥村委会",
                code: "003",
            },
            VillageCode {
                name: "肃州庙村委会",
                code: "004",
            },
            VillageCode {
                name: "板桥村委会",
                code: "005",
            },
            VillageCode {
                name: "武威庙村委会",
                code: "006",
            },
            VillageCode {
                name: "河州堡村委会",
                code: "007",
            },
            VillageCode {
                name: "孟家桥村委会",
                code: "008",
            },
            VillageCode {
                name: "杨家堡村委会",
                code: "009",
            },
            VillageCode {
                name: "姚家沟村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "莫高镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "新店台村委会",
                code: "001",
            },
            VillageCode {
                name: "五墩村委会",
                code: "002",
            },
            VillageCode {
                name: "新墩村委会",
                code: "003",
            },
            VillageCode {
                name: "苏家堡村委会",
                code: "004",
            },
            VillageCode {
                name: "窦家墩村委会",
                code: "005",
            },
            VillageCode {
                name: "三危村委会",
                code: "006",
            },
            VillageCode {
                name: "甘家堡村委会",
                code: "007",
            },
            VillageCode {
                name: "泾桥村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "转渠口镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "秦安村委会",
                code: "001",
            },
            VillageCode {
                name: "五圣宫村委会",
                code: "002",
            },
            VillageCode {
                name: "东沙门村委会",
                code: "003",
            },
            VillageCode {
                name: "阶州村委会",
                code: "004",
            },
            VillageCode {
                name: "定西村委会",
                code: "005",
            },
            VillageCode {
                name: "漳县村委会",
                code: "006",
            },
            VillageCode {
                name: "雷家墩村委会",
                code: "007",
            },
            VillageCode {
                name: "盐茶村委会",
                code: "008",
            },
            VillageCode {
                name: "吕家庄村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "阳关镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "营盘村委会",
                code: "001",
            },
            VillageCode {
                name: "阳关村委会",
                code: "002",
            },
            VillageCode {
                name: "寿昌村委会",
                code: "003",
            },
            VillageCode {
                name: "龙勒村委会",
                code: "004",
            },
            VillageCode {
                name: "二墩村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "月牙泉镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "杨家桥村委会",
                code: "001",
            },
            VillageCode {
                name: "合水村委会",
                code: "002",
            },
            VillageCode {
                name: "月牙泉村委会",
                code: "003",
            },
            VillageCode {
                name: "鸣山村委会",
                code: "004",
            },
            VillageCode {
                name: "中渠村委会",
                code: "005",
            },
            VillageCode {
                name: "兰州村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "郭家堡镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "六号桥村委会",
                code: "001",
            },
            VillageCode {
                name: "前进村委会",
                code: "002",
            },
            VillageCode {
                name: "大泉村委会",
                code: "003",
            },
            VillageCode {
                name: "梁家堡村委会",
                code: "004",
            },
            VillageCode {
                name: "土塔村委会",
                code: "005",
            },
            VillageCode {
                name: "七号桥村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "黄渠镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "黄墩子社区居委会",
                code: "001",
            },
            VillageCode {
                name: "清水村委会",
                code: "002",
            },
            VillageCode {
                name: "闸坝梁村委会",
                code: "003",
            },
            VillageCode {
                name: "代家墩村委会",
                code: "004",
            },
            VillageCode {
                name: "常丰村委会",
                code: "005",
            },
            VillageCode {
                name: "芭子场村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "青海石油管理局生活基地",
        code: "010",
        villages: &[VillageCode {
            name: "青海石油生活基地虚拟社区",
            code: "001",
        }],
    },
];

pub const CITIES_HX: [CityCode; 21] = [
    CityCode {
        name: "省辖市",
        code: "000",
        towns: &[],
    },
    CityCode {
        name: "张掖市",
        code: "001",
        towns: &TOWNS_HX_001,
    },
    CityCode {
        name: "金川市",
        code: "002",
        towns: &TOWNS_HX_002,
    },
    CityCode {
        name: "永昌市",
        code: "003",
        towns: &TOWNS_HX_003,
    },
    CityCode {
        name: "凉州市",
        code: "004",
        towns: &TOWNS_HX_004,
    },
    CityCode {
        name: "民勤市",
        code: "005",
        towns: &TOWNS_HX_005,
    },
    CityCode {
        name: "古浪市",
        code: "006",
        towns: &TOWNS_HX_006,
    },
    CityCode {
        name: "天祝市",
        code: "007",
        towns: &TOWNS_HX_007,
    },
    CityCode {
        name: "甘州市",
        code: "008",
        towns: &TOWNS_HX_008,
    },
    CityCode {
        name: "肃南市",
        code: "009",
        towns: &TOWNS_HX_009,
    },
    CityCode {
        name: "民乐市",
        code: "010",
        towns: &TOWNS_HX_010,
    },
    CityCode {
        name: "临泽市",
        code: "011",
        towns: &TOWNS_HX_011,
    },
    CityCode {
        name: "高台市",
        code: "012",
        towns: &TOWNS_HX_012,
    },
    CityCode {
        name: "山丹市",
        code: "013",
        towns: &TOWNS_HX_013,
    },
    CityCode {
        name: "肃州市",
        code: "014",
        towns: &TOWNS_HX_014,
    },
    CityCode {
        name: "金塔市",
        code: "015",
        towns: &TOWNS_HX_015,
    },
    CityCode {
        name: "瓜州市",
        code: "016",
        towns: &TOWNS_HX_016,
    },
    CityCode {
        name: "肃北市",
        code: "017",
        towns: &TOWNS_HX_017,
    },
    CityCode {
        name: "阿克塞市",
        code: "018",
        towns: &TOWNS_HX_018,
    },
    CityCode {
        name: "玉门市",
        code: "019",
        towns: &TOWNS_HX_019,
    },
    CityCode {
        name: "敦煌市",
        code: "020",
        towns: &TOWNS_HX_020,
    },
];
