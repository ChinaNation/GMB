use super::{CityCode, TownCode, VillageCode};

static TOWNS_QH_001: [TownCode; 80] = [
    TownCode {
        name: "东关大街街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "北关社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "慈幼社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "五一社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "清真巷街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "团结社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "夏都花园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "南小街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "凤凰园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "磨尔园社区居民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "大众街街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "凯旋社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "园山社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "树林巷社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "共和路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "德令哈社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "安泰社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "康西社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "梨园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "友谊村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "先进村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "周家泉街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "杨家巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "建国路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "为民巷社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "白家河湾社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "联合村村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "火车站街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "车站社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "中庄社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "幸福社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "富民路社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "八一路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "学院社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "康东社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "康南社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "康宁社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "博雅路南社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "团结村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "林家崖街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "蓝天社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "站西社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "林家崖村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "路家庄村村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "乐家湾镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "乐家湾社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "金桥路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "明杏社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "泉景社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "上十里铺村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "塔尔山村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下十里铺村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "乐家湾村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "杨沟湾村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "韵家口镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "东兴社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东盛社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "纺织社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "育才路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "中庄村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "褚家营村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "小寨村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "韵家口村民委员会",
                code: "008",
            },
            VillageCode {
                name: "泮子山村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "付家寨村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "朱家庄村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "王家庄村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "曹家寨村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "东川工业园",
        code: "010",
        villages: &[VillageCode {
            name: "东川工业园虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "人民街街道",
        code: "011",
        villages: &[
            VillageCode {
                name: "南关街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "水井巷社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "南滩街道",
        code: "012",
        villages: &[
            VillageCode {
                name: "农建社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南山东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "南山西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "南山社区居委会",
                code: "004",
            },
            VillageCode {
                name: "新青社区居委会",
                code: "005",
            },
            VillageCode {
                name: "建新社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "仓门街街道",
        code: "013",
        villages: &[
            VillageCode {
                name: "石坡街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "前营街社区居委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "礼让街街道",
        code: "014",
        villages: &[
            VillageCode {
                name: "长江路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "七一路西社区居委会",
                code: "002",
            },
            VillageCode {
                name: "解放路社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "饮马街街道",
        code: "015",
        villages: &[
            VillageCode {
                name: "东大街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "上滨河路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "文化街社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "南川东路街道",
        code: "016",
        villages: &[
            VillageCode {
                name: "龙泰社区居委会",
                code: "001",
            },
            VillageCode {
                name: "二机社区居委会",
                code: "002",
            },
            VillageCode {
                name: "兴旺社区居委会",
                code: "003",
            },
            VillageCode {
                name: "瑞驰社区居委会",
                code: "004",
            },
            VillageCode {
                name: "水磨村村委会",
                code: "005",
            },
            VillageCode {
                name: "红光村村委会",
                code: "006",
            },
            VillageCode {
                name: "南酉山村村委会",
                code: "007",
            },
            VillageCode {
                name: "南园村村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "南川西路街道",
        code: "017",
        villages: &[
            VillageCode {
                name: "福禄巷北社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "西台社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "东台社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "福禄巷南社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "香格里拉社区居委会",
                code: "005",
            },
            VillageCode {
                name: "安宁路居委会",
                code: "006",
            },
            VillageCode {
                name: "沈家寨村委会",
                code: "007",
            },
            VillageCode {
                name: "红星村委会",
                code: "008",
            },
            VillageCode {
                name: "新青村委会",
                code: "009",
            },
            VillageCode {
                name: "园树村委会",
                code: "010",
            },
            VillageCode {
                name: "贾小村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "总寨镇",
        code: "018",
        villages: &[
            VillageCode {
                name: "清华路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "金十字社区居委会",
                code: "002",
            },
            VillageCode {
                name: "新城社区居委会",
                code: "003",
            },
            VillageCode {
                name: "王斌堡村委会",
                code: "004",
            },
            VillageCode {
                name: "张家庄村委会",
                code: "005",
            },
            VillageCode {
                name: "清河村委会",
                code: "006",
            },
            VillageCode {
                name: "清水河村委会",
                code: "007",
            },
            VillageCode {
                name: "总南村委会",
                code: "008",
            },
            VillageCode {
                name: "总北村委会",
                code: "009",
            },
            VillageCode {
                name: "谢家寨村委会",
                code: "010",
            },
            VillageCode {
                name: "塘马坊村委会",
                code: "011",
            },
            VillageCode {
                name: "杜家庄村委会",
                code: "012",
            },
            VillageCode {
                name: "泉尔湾村委会",
                code: "013",
            },
            VillageCode {
                name: "新庄村委会",
                code: "014",
            },
            VillageCode {
                name: "逯家寨村委会",
                code: "015",
            },
            VillageCode {
                name: "王家山村委会",
                code: "016",
            },
            VillageCode {
                name: "元堡子村委会",
                code: "017",
            },
            VillageCode {
                name: "星家村委会",
                code: "018",
            },
            VillageCode {
                name: "新安村委会",
                code: "019",
            },
            VillageCode {
                name: "享堂村委会",
                code: "020",
            },
            VillageCode {
                name: "陈家窑村委会",
                code: "021",
            },
            VillageCode {
                name: "莫家沟村委会",
                code: "022",
            },
            VillageCode {
                name: "下细沟村委会",
                code: "023",
            },
            VillageCode {
                name: "上细沟村委会",
                code: "024",
            },
            VillageCode {
                name: "下野牛沟村委会",
                code: "025",
            },
            VillageCode {
                name: "上野牛沟村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "南川工业园",
        code: "019",
        villages: &[VillageCode {
            name: "南川工业园虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "西关大街街道",
        code: "020",
        villages: &[
            VillageCode {
                name: "南气象巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "贾小社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北气象巷社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "古城台街道",
        code: "021",
        villages: &[
            VillageCode {
                name: "学院巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "青年巷社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "昆仑路东居委会",
                code: "003",
            },
            VillageCode {
                name: "昆仑路西居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "虎台街道",
        code: "022",
        villages: &[
            VillageCode {
                name: "海晏路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "医财巷东居委会",
                code: "002",
            },
            VillageCode {
                name: "医财巷西居委会",
                code: "003",
            },
            VillageCode {
                name: "冷湖路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "新西社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "殷家庄社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "虎台社区居委会",
                code: "007",
            },
            VillageCode {
                name: "杨家寨村委会",
                code: "008",
            },
            VillageCode {
                name: "苏家河湾村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "胜利路街道",
        code: "023",
        villages: &[
            VillageCode {
                name: "西交通巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "公园巷社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "东交通巷社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "北商业巷社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "兴海路街道",
        code: "024",
        villages: &[
            VillageCode {
                name: "兴胜巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "中华巷社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "尕寺巷社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "文汇路街道",
        code: "025",
        villages: &[
            VillageCode {
                name: "文亭巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "文博路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "海湖广场社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "科普路社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "通海路街道",
        code: "026",
        villages: &[
            VillageCode {
                name: "文成路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "桃李路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "光华路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "文景街西社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "彭家寨镇",
        code: "027",
        villages: &[
            VillageCode {
                name: "西川南路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "富兴路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "彭家寨村委会",
                code: "003",
            },
            VillageCode {
                name: "西北园村委会",
                code: "004",
            },
            VillageCode {
                name: "刘家寨村委会",
                code: "005",
            },
            VillageCode {
                name: "汉庄村委会",
                code: "006",
            },
            VillageCode {
                name: "张家湾村委会",
                code: "007",
            },
            VillageCode {
                name: "杨家湾村委会",
                code: "008",
            },
            VillageCode {
                name: "阴山堂村委会",
                code: "009",
            },
            VillageCode {
                name: "火西村委会",
                code: "010",
            },
            VillageCode {
                name: "火东村委会",
                code: "011",
            },
            VillageCode {
                name: "晨光村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "朝阳街道",
        code: "028",
        villages: &[
            VillageCode {
                name: "朝阳社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "山川社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北川河东路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "祁连路西社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "北山社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "朝阳西路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "北园村委会",
                code: "007",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "008",
            },
            VillageCode {
                name: "祁家城村委会",
                code: "009",
            },
            VillageCode {
                name: "寺台子村委会",
                code: "010",
            },
            VillageCode {
                name: "新民村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "小桥大街街道",
        code: "029",
        villages: &[
            VillageCode {
                name: "建设巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新海桥社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "毛胜寺社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "小桥社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "新世纪社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "西海路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "毛胜寺村委会",
                code: "007",
            },
            VillageCode {
                name: "北杏园村委会",
                code: "008",
            },
            VillageCode {
                name: "陶家寨村委会",
                code: "009",
            },
            VillageCode {
                name: "陶新村委会",
                code: "010",
            },
            VillageCode {
                name: "小桥村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "马坊街道",
        code: "030",
        villages: &[
            VillageCode {
                name: "西杏园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "马坊东社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "青工社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "汽运社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "光明社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "新村社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "欣乐社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "幸福社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "海湖桥西社区居委会",
                code: "009",
            },
            VillageCode {
                name: "西杏园村委会",
                code: "010",
            },
            VillageCode {
                name: "马坊村委会",
                code: "011",
            },
            VillageCode {
                name: "盐庄村委会",
                code: "012",
            },
            VillageCode {
                name: "三其村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "火车西站街道",
        code: "031",
        villages: &[
            VillageCode {
                name: "萨尔斯堡社区",
                code: "001",
            },
            VillageCode {
                name: "美丽水街社区",
                code: "002",
            },
            VillageCode {
                name: "湟水河畔社区",
                code: "003",
            },
            VillageCode {
                name: "火车西站社区",
                code: "004",
            },
            VillageCode {
                name: "盐庄社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "大堡子镇",
        code: "032",
        villages: &[
            VillageCode {
                name: "一机床社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "工具厂社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "大堡子村委会",
                code: "003",
            },
            VillageCode {
                name: "严小村委会",
                code: "004",
            },
            VillageCode {
                name: "宋家寨村委会",
                code: "005",
            },
            VillageCode {
                name: "晋家湾村委会",
                code: "006",
            },
            VillageCode {
                name: "鲍家寨村委会",
                code: "007",
            },
            VillageCode {
                name: "朱南村委会",
                code: "008",
            },
            VillageCode {
                name: "朱北村委会",
                code: "009",
            },
            VillageCode {
                name: "吧浪村委会",
                code: "010",
            },
            VillageCode {
                name: "吴仲村委会",
                code: "011",
            },
            VillageCode {
                name: "汪家寨村委会",
                code: "012",
            },
            VillageCode {
                name: "乙其寨村委会",
                code: "013",
            },
            VillageCode {
                name: "陶南村委会",
                code: "014",
            },
            VillageCode {
                name: "陶北村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "廿里铺镇",
        code: "033",
        villages: &[
            VillageCode {
                name: "生物园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "泉湾社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "高教路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "廿里铺村委会",
                code: "004",
            },
            VillageCode {
                name: "花园台村委会",
                code: "005",
            },
            VillageCode {
                name: "孙家寨村委会",
                code: "006",
            },
            VillageCode {
                name: "小寨村委会",
                code: "007",
            },
            VillageCode {
                name: "莫家庄村委会",
                code: "008",
            },
            VillageCode {
                name: "新村委会",
                code: "009",
            },
            VillageCode {
                name: "石头磊村委会",
                code: "010",
            },
            VillageCode {
                name: "魏家庄村委会",
                code: "011",
            },
            VillageCode {
                name: "九家湾村委会",
                code: "012",
            },
            VillageCode {
                name: "郭家塔村委会",
                code: "013",
            },
            VillageCode {
                name: "双苏堡村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "生物科技产业园",
        code: "034",
        villages: &[VillageCode {
            name: "生物科技产业园虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "田家寨镇",
        code: "035",
        villages: &[
            VillageCode {
                name: "田家寨社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "泗洱河村委会",
                code: "002",
            },
            VillageCode {
                name: "谢家台村委会",
                code: "003",
            },
            VillageCode {
                name: "毛家台村委会",
                code: "004",
            },
            VillageCode {
                name: "田家寨村委会",
                code: "005",
            },
            VillageCode {
                name: "毛一村委会",
                code: "006",
            },
            VillageCode {
                name: "毛二村委会",
                code: "007",
            },
            VillageCode {
                name: "河湾村委会",
                code: "008",
            },
            VillageCode {
                name: "新村村委会",
                code: "009",
            },
            VillageCode {
                name: "李家台村委会",
                code: "010",
            },
            VillageCode {
                name: "梁家村委会",
                code: "011",
            },
            VillageCode {
                name: "石沟村委会",
                code: "012",
            },
            VillageCode {
                name: "谢家村委会",
                code: "013",
            },
            VillageCode {
                name: "大卡阳村委会",
                code: "014",
            },
            VillageCode {
                name: "小卡阳村委会",
                code: "015",
            },
            VillageCode {
                name: "甘家村委会",
                code: "016",
            },
            VillageCode {
                name: "公牙村委会",
                code: "017",
            },
            VillageCode {
                name: "喇家村委会",
                code: "018",
            },
            VillageCode {
                name: "窑洞村委会",
                code: "019",
            },
            VillageCode {
                name: "下营一村委会",
                code: "020",
            },
            VillageCode {
                name: "上营一村委会",
                code: "021",
            },
            VillageCode {
                name: "拉尕村委会",
                code: "022",
            },
            VillageCode {
                name: "黄蒿台村委会",
                code: "023",
            },
            VillageCode {
                name: "流水沟村委会",
                code: "024",
            },
            VillageCode {
                name: "群塔村委会",
                code: "025",
            },
            VillageCode {
                name: "阳坡一村委会",
                code: "026",
            },
            VillageCode {
                name: "鸽堂村委会",
                code: "027",
            },
            VillageCode {
                name: "丹麻村委会",
                code: "028",
            },
            VillageCode {
                name: "坪台村委会",
                code: "029",
            },
            VillageCode {
                name: "永丰村委会",
                code: "030",
            },
            VillageCode {
                name: "安宁村委会",
                code: "031",
            },
            VillageCode {
                name: "李家庄村委会",
                code: "032",
            },
            VillageCode {
                name: "阳坡二村委会",
                code: "033",
            },
            VillageCode {
                name: "阴坡村委会",
                code: "034",
            },
            VillageCode {
                name: "尕院村委会",
                code: "035",
            },
            VillageCode {
                name: "上营二村委会",
                code: "036",
            },
            VillageCode {
                name: "下营二村委会",
                code: "037",
            },
            VillageCode {
                name: "卜家台村委会",
                code: "038",
            },
            VillageCode {
                name: "台口子村委会",
                code: "039",
            },
            VillageCode {
                name: "沙尔湾村委会",
                code: "040",
            },
            VillageCode {
                name: "上洛麻村委会",
                code: "041",
            },
            VillageCode {
                name: "下洛麻村委会",
                code: "042",
            },
            VillageCode {
                name: "鲍家村委会",
                code: "043",
            },
            VillageCode {
                name: "马昌沟村委会",
                code: "044",
            },
        ],
    },
    TownCode {
        name: "上新庄镇",
        code: "036",
        villages: &[
            VillageCode {
                name: "上新庄社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "刘小庄村委会",
                code: "002",
            },
            VillageCode {
                name: "班麻坡村委会",
                code: "003",
            },
            VillageCode {
                name: "东台村委会",
                code: "004",
            },
            VillageCode {
                name: "西庄村委会",
                code: "005",
            },
            VillageCode {
                name: "东沟滩村委会",
                code: "006",
            },
            VillageCode {
                name: "马家滩村委会",
                code: "007",
            },
            VillageCode {
                name: "红牙合村委会",
                code: "008",
            },
            VillageCode {
                name: "尧滩村委会",
                code: "009",
            },
            VillageCode {
                name: "尧湾村委会",
                code: "010",
            },
            VillageCode {
                name: "下峡门村委会",
                code: "011",
            },
            VillageCode {
                name: "上峡门村委会",
                code: "012",
            },
            VillageCode {
                name: "申北村委会",
                code: "013",
            },
            VillageCode {
                name: "申南村委会",
                code: "014",
            },
            VillageCode {
                name: "水草沟村委会",
                code: "015",
            },
            VillageCode {
                name: "河滩村委会",
                code: "016",
            },
            VillageCode {
                name: "黑城村委会",
                code: "017",
            },
            VillageCode {
                name: "上新庄村委会",
                code: "018",
            },
            VillageCode {
                name: "阳坡台村委会",
                code: "019",
            },
            VillageCode {
                name: "地广村委会",
                code: "020",
            },
            VillageCode {
                name: "华山村委会",
                code: "021",
            },
            VillageCode {
                name: "骟马台村委会",
                code: "022",
            },
            VillageCode {
                name: "加牙村委会",
                code: "023",
            },
            VillageCode {
                name: "新城村委会",
                code: "024",
            },
            VillageCode {
                name: "周德村委会",
                code: "025",
            },
            VillageCode {
                name: "班隆村委会",
                code: "026",
            },
            VillageCode {
                name: "马场村委会",
                code: "027",
            },
            VillageCode {
                name: "七家庄村委会",
                code: "028",
            },
            VillageCode {
                name: "海马沟村委会",
                code: "029",
            },
            VillageCode {
                name: "下台村委会",
                code: "030",
            },
            VillageCode {
                name: "上台村委会",
                code: "031",
            },
            VillageCode {
                name: "白路尔村委会",
                code: "032",
            },
            VillageCode {
                name: "白石头村委会",
                code: "033",
            },
            VillageCode {
                name: "静房村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "鲁沙尔镇",
        code: "037",
        villages: &[
            VillageCode {
                name: "金塔社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "团结社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "和平社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "莲湖社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "水滩村委会",
                code: "005",
            },
            VillageCode {
                name: "孔家村委会",
                code: "006",
            },
            VillageCode {
                name: "赵家庄村委会",
                code: "007",
            },
            VillageCode {
                name: "昂藏村委会",
                code: "008",
            },
            VillageCode {
                name: "和平村委会",
                code: "009",
            },
            VillageCode {
                name: "河滩村委会",
                code: "010",
            },
            VillageCode {
                name: "团结村委会",
                code: "011",
            },
            VillageCode {
                name: "东山村委会",
                code: "012",
            },
            VillageCode {
                name: "西山村委会",
                code: "013",
            },
            VillageCode {
                name: "塔尔湾村委会",
                code: "014",
            },
            VillageCode {
                name: "青一村委会",
                code: "015",
            },
            VillageCode {
                name: "青二村委会",
                code: "016",
            },
            VillageCode {
                name: "南门村委会",
                code: "017",
            },
            VillageCode {
                name: "海马泉村委会",
                code: "018",
            },
            VillageCode {
                name: "新村村委会",
                code: "019",
            },
            VillageCode {
                name: "红崖沟村委会",
                code: "020",
            },
            VillageCode {
                name: "陈家滩村委会",
                code: "021",
            },
            VillageCode {
                name: "西村村委会",
                code: "022",
            },
            VillageCode {
                name: "东村村委会",
                code: "023",
            },
            VillageCode {
                name: "徐家寨村委会",
                code: "024",
            },
            VillageCode {
                name: "石咀一村委会",
                code: "025",
            },
            VillageCode {
                name: "下重台村委会",
                code: "026",
            },
            VillageCode {
                name: "白土庄村委会",
                code: "027",
            },
            VillageCode {
                name: "地窑村委会",
                code: "028",
            },
            VillageCode {
                name: "阴坡村委会",
                code: "029",
            },
            VillageCode {
                name: "阳坡村委会",
                code: "030",
            },
            VillageCode {
                name: "石咀二村委会",
                code: "031",
            },
            VillageCode {
                name: "吊庄村委会",
                code: "032",
            },
            VillageCode {
                name: "甘河沿村委会",
                code: "033",
            },
            VillageCode {
                name: "阿家庄村委会",
                code: "034",
            },
            VillageCode {
                name: "朱家庄村委会",
                code: "035",
            },
            VillageCode {
                name: "青石坡村委会",
                code: "036",
            },
            VillageCode {
                name: "上重台村委会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "甘河滩镇",
        code: "038",
        villages: &[
            VillageCode {
                name: "甘河滩社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "甘河村委会",
                code: "002",
            },
            VillageCode {
                name: "页沟村委会",
                code: "003",
            },
            VillageCode {
                name: "坡东村委会",
                code: "004",
            },
            VillageCode {
                name: "坡西村委会",
                code: "005",
            },
            VillageCode {
                name: "隆寺干村委会",
                code: "006",
            },
            VillageCode {
                name: "下中沟村委会",
                code: "007",
            },
            VillageCode {
                name: "上中沟村委会",
                code: "008",
            },
            VillageCode {
                name: "元山尔村委会",
                code: "009",
            },
            VillageCode {
                name: "卡跃村委会",
                code: "010",
            },
            VillageCode {
                name: "上营村委会",
                code: "011",
            },
            VillageCode {
                name: "下营村委会",
                code: "012",
            },
            VillageCode {
                name: "上河湾村委会",
                code: "013",
            },
            VillageCode {
                name: "下河湾村委会",
                code: "014",
            },
            VillageCode {
                name: "李九村委会",
                code: "015",
            },
            VillageCode {
                name: "前跃村委会",
                code: "016",
            },
            VillageCode {
                name: "东湾村委会",
                code: "017",
            },
            VillageCode {
                name: "黄一村委会",
                code: "018",
            },
            VillageCode {
                name: "黄二村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "共和镇",
        code: "039",
        villages: &[
            VillageCode {
                name: "共和社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "北村村委会",
                code: "002",
            },
            VillageCode {
                name: "南村村委会",
                code: "003",
            },
            VillageCode {
                name: "山甲村委会",
                code: "004",
            },
            VillageCode {
                name: "石城村委会",
                code: "005",
            },
            VillageCode {
                name: "后营村委会",
                code: "006",
            },
            VillageCode {
                name: "前营村委会",
                code: "007",
            },
            VillageCode {
                name: "木场村委会",
                code: "008",
            },
            VillageCode {
                name: "上直沟村委会",
                code: "009",
            },
            VillageCode {
                name: "花勒城村委会",
                code: "010",
            },
            VillageCode {
                name: "王家山村委会",
                code: "011",
            },
            VillageCode {
                name: "苏尔吉村委会",
                code: "012",
            },
            VillageCode {
                name: "转嘴村委会",
                code: "013",
            },
            VillageCode {
                name: "东岔村委会",
                code: "014",
            },
            VillageCode {
                name: "西岔村委会",
                code: "015",
            },
            VillageCode {
                name: "盘道村委会",
                code: "016",
            },
            VillageCode {
                name: "西台村委会",
                code: "017",
            },
            VillageCode {
                name: "东台村委会",
                code: "018",
            },
            VillageCode {
                name: "新湾村委会",
                code: "019",
            },
            VillageCode {
                name: "葱湾村委会",
                code: "020",
            },
            VillageCode {
                name: "达草沟村委会",
                code: "021",
            },
            VillageCode {
                name: "下马申村委会",
                code: "022",
            },
            VillageCode {
                name: "上马申村委会",
                code: "023",
            },
            VillageCode {
                name: "河湾村委会",
                code: "024",
            },
            VillageCode {
                name: "后街村委会",
                code: "025",
            },
            VillageCode {
                name: "新庄村委会",
                code: "026",
            },
            VillageCode {
                name: "尕庄村委会",
                code: "027",
            },
            VillageCode {
                name: "庄科脑村委会",
                code: "028",
            },
            VillageCode {
                name: "尖达村委会",
                code: "029",
            },
            VillageCode {
                name: "萱麻湾村委会",
                code: "030",
            },
            VillageCode {
                name: "押必村委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "多巴镇",
        code: "040",
        villages: &[
            VillageCode {
                name: "多巴金城社区",
                code: "001",
            },
            VillageCode {
                name: "多巴通海社区",
                code: "002",
            },
            VillageCode {
                name: "小寨村委会",
                code: "003",
            },
            VillageCode {
                name: "双寨村委会",
                code: "004",
            },
            VillageCode {
                name: "大崖沟村委会",
                code: "005",
            },
            VillageCode {
                name: "韦家庄村委会",
                code: "006",
            },
            VillageCode {
                name: "甘河门村委会",
                code: "007",
            },
            VillageCode {
                name: "新墩村委会",
                code: "008",
            },
            VillageCode {
                name: "康城村委会",
                code: "009",
            },
            VillageCode {
                name: "城东村委会",
                code: "010",
            },
            VillageCode {
                name: "城西村委会",
                code: "011",
            },
            VillageCode {
                name: "王家庄村委会",
                code: "012",
            },
            VillageCode {
                name: "城中村委会",
                code: "013",
            },
            VillageCode {
                name: "银格达村委会",
                code: "014",
            },
            VillageCode {
                name: "丰胜村委会",
                code: "015",
            },
            VillageCode {
                name: "国寺营村委会",
                code: "016",
            },
            VillageCode {
                name: "石板沟村委会",
                code: "017",
            },
            VillageCode {
                name: "扎麻隆村委会",
                code: "018",
            },
            VillageCode {
                name: "马申茂村委会",
                code: "019",
            },
            VillageCode {
                name: "加拉山村委会",
                code: "020",
            },
            VillageCode {
                name: "尚什家村委会",
                code: "021",
            },
            VillageCode {
                name: "羊圈村委会",
                code: "022",
            },
            VillageCode {
                name: "多巴四村委会",
                code: "023",
            },
            VillageCode {
                name: "指挥庄村委会",
                code: "024",
            },
            VillageCode {
                name: "多巴二村委会",
                code: "025",
            },
            VillageCode {
                name: "燕尔沟村委会",
                code: "026",
            },
            VillageCode {
                name: "大掌村委会",
                code: "027",
            },
            VillageCode {
                name: "多巴三村委会",
                code: "028",
            },
            VillageCode {
                name: "多巴一村委会",
                code: "029",
            },
            VillageCode {
                name: "黑嘴村委会",
                code: "030",
            },
            VillageCode {
                name: "沙窝尔村委会",
                code: "031",
            },
            VillageCode {
                name: "幸福村委会",
                code: "032",
            },
            VillageCode {
                name: "初哇村委会",
                code: "033",
            },
            VillageCode {
                name: "玉拉村委会",
                code: "034",
            },
            VillageCode {
                name: "合尔营村委会",
                code: "035",
            },
            VillageCode {
                name: "丹麻寺村委会",
                code: "036",
            },
            VillageCode {
                name: "奔巴口村委会",
                code: "037",
            },
            VillageCode {
                name: "油房台村委会",
                code: "038",
            },
            VillageCode {
                name: "年家庄村委会",
                code: "039",
            },
            VillageCode {
                name: "杨家台村委会",
                code: "040",
            },
            VillageCode {
                name: "北沟村委会",
                code: "041",
            },
            VillageCode {
                name: "目尔加村委会",
                code: "042",
            },
            VillageCode {
                name: "拉卡山村委会",
                code: "043",
            },
            VillageCode {
                name: "尕尔加村委会",
                code: "044",
            },
            VillageCode {
                name: "中村村委会",
                code: "045",
            },
            VillageCode {
                name: "洛尔洞村委会",
                code: "046",
            },
        ],
    },
    TownCode {
        name: "拦隆口镇",
        code: "041",
        villages: &[
            VillageCode {
                name: "拦隆口社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新村村委会",
                code: "002",
            },
            VillageCode {
                name: "扎什营村委会",
                code: "003",
            },
            VillageCode {
                name: "巴达村委会",
                code: "004",
            },
            VillageCode {
                name: "班仲营村委会",
                code: "005",
            },
            VillageCode {
                name: "端巴营村委会",
                code: "006",
            },
            VillageCode {
                name: "西岔村委会",
                code: "007",
            },
            VillageCode {
                name: "下鲁尔加村委会",
                code: "008",
            },
            VillageCode {
                name: "上鲁尔加村委会",
                code: "009",
            },
            VillageCode {
                name: "拦隆口村委会",
                code: "010",
            },
            VillageCode {
                name: "白杨口村委会",
                code: "011",
            },
            VillageCode {
                name: "东拉科村委会",
                code: "012",
            },
            VillageCode {
                name: "南门一村委会",
                code: "013",
            },
            VillageCode {
                name: "前庄村委会",
                code: "014",
            },
            VillageCode {
                name: "中庄村委会",
                code: "015",
            },
            VillageCode {
                name: "上庄村委会",
                code: "016",
            },
            VillageCode {
                name: "卡阳村委会",
                code: "017",
            },
            VillageCode {
                name: "白崖一村委会",
                code: "018",
            },
            VillageCode {
                name: "泥隆台村委会",
                code: "019",
            },
            VillageCode {
                name: "泥隆口村委会",
                code: "020",
            },
            VillageCode {
                name: "桥西村委会",
                code: "021",
            },
            VillageCode {
                name: "千西村委会",
                code: "022",
            },
            VillageCode {
                name: "千东村委会",
                code: "023",
            },
            VillageCode {
                name: "铁家营村委会",
                code: "024",
            },
            VillageCode {
                name: "上营村委会",
                code: "025",
            },
            VillageCode {
                name: "上寺村委会",
                code: "026",
            },
            VillageCode {
                name: "拦隆一村委会",
                code: "027",
            },
            VillageCode {
                name: "拦隆二村委会",
                code: "028",
            },
            VillageCode {
                name: "白崖二村委会",
                code: "029",
            },
            VillageCode {
                name: "合尔营村委会",
                code: "030",
            },
            VillageCode {
                name: "麻子营村委会",
                code: "031",
            },
            VillageCode {
                name: "后河尔村委会",
                code: "032",
            },
            VillageCode {
                name: "佰什营村委会",
                code: "033",
            },
            VillageCode {
                name: "图巴营村委会",
                code: "034",
            },
            VillageCode {
                name: "尼麻隆村委会",
                code: "035",
            },
            VillageCode {
                name: "上红土沟村委会",
                code: "036",
            },
            VillageCode {
                name: "下红土沟村委会",
                code: "037",
            },
            VillageCode {
                name: "红林村委会",
                code: "038",
            },
            VillageCode {
                name: "民族村委会",
                code: "039",
            },
            VillageCode {
                name: "邦隆村委会",
                code: "040",
            },
            VillageCode {
                name: "民联村委会",
                code: "041",
            },
            VillageCode {
                name: "峡口村委会",
                code: "042",
            },
            VillageCode {
                name: "南门二村委会",
                code: "043",
            },
        ],
    },
    TownCode {
        name: "上五庄镇",
        code: "042",
        villages: &[
            VillageCode {
                name: "上五庄社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "合尔盖村委会",
                code: "002",
            },
            VillageCode {
                name: "北纳村委会",
                code: "003",
            },
            VillageCode {
                name: "马场村委会",
                code: "004",
            },
            VillageCode {
                name: "友爱村委会",
                code: "005",
            },
            VillageCode {
                name: "邦吧村委会",
                code: "006",
            },
            VillageCode {
                name: "华科村委会",
                code: "007",
            },
            VillageCode {
                name: "纳卜藏村委会",
                code: "008",
            },
            VillageCode {
                name: "包勒村委会",
                code: "009",
            },
            VillageCode {
                name: "拉斯目村委会",
                code: "010",
            },
            VillageCode {
                name: "北庄村委会",
                code: "011",
            },
            VillageCode {
                name: "峡口村委会",
                code: "012",
            },
            VillageCode {
                name: "甫崖村委会",
                code: "013",
            },
            VillageCode {
                name: "拉尔宁一村委会",
                code: "014",
            },
            VillageCode {
                name: "拉尔宁二村委会",
                code: "015",
            },
            VillageCode {
                name: "拉尔宁三村委会",
                code: "016",
            },
            VillageCode {
                name: "黄草沟村委会",
                code: "017",
            },
            VillageCode {
                name: "大寺沟一村委会",
                code: "018",
            },
            VillageCode {
                name: "大寺沟二村委会",
                code: "019",
            },
            VillageCode {
                name: "业宏村委会",
                code: "020",
            },
            VillageCode {
                name: "拉目台村委会",
                code: "021",
            },
            VillageCode {
                name: "小寺沟村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "李家山镇",
        code: "043",
        villages: &[
            VillageCode {
                name: "李家山社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "崖头村委会",
                code: "002",
            },
            VillageCode {
                name: "董家湾村委会",
                code: "003",
            },
            VillageCode {
                name: "柳树庄村委会",
                code: "004",
            },
            VillageCode {
                name: "王家堡村委会",
                code: "005",
            },
            VillageCode {
                name: "卡约村委会",
                code: "006",
            },
            VillageCode {
                name: "下坪村委会",
                code: "007",
            },
            VillageCode {
                name: "上坪村委会",
                code: "008",
            },
            VillageCode {
                name: "包家庄村委会",
                code: "009",
            },
            VillageCode {
                name: "陈家庄村委会",
                code: "010",
            },
            VillageCode {
                name: "汉水沟村委会",
                code: "011",
            },
            VillageCode {
                name: "毛尔茨沟村委会",
                code: "012",
            },
            VillageCode {
                name: "马营村委会",
                code: "013",
            },
            VillageCode {
                name: "下油房村委会",
                code: "014",
            },
            VillageCode {
                name: "纳家村委会",
                code: "015",
            },
            VillageCode {
                name: "岗岔村委会",
                code: "016",
            },
            VillageCode {
                name: "上西河村委会",
                code: "017",
            },
            VillageCode {
                name: "下西河村委会",
                code: "018",
            },
            VillageCode {
                name: "新添堡村委会",
                code: "019",
            },
            VillageCode {
                name: "河湾村委会",
                code: "020",
            },
            VillageCode {
                name: "吉家村委会",
                code: "021",
            },
            VillageCode {
                name: "甘家村委会",
                code: "022",
            },
            VillageCode {
                name: "新庄村委会",
                code: "023",
            },
            VillageCode {
                name: "勺麻营村委会",
                code: "024",
            },
            VillageCode {
                name: "大路村委会",
                code: "025",
            },
            VillageCode {
                name: "李家山村委会",
                code: "026",
            },
            VillageCode {
                name: "马圈沟村委会",
                code: "027",
            },
            VillageCode {
                name: "金跃村委会",
                code: "028",
            },
            VillageCode {
                name: "峡口村委会",
                code: "029",
            },
            VillageCode {
                name: "阳坡村委会",
                code: "030",
            },
            VillageCode {
                name: "阴坡村委会",
                code: "031",
            },
            VillageCode {
                name: "恰罗村委会",
                code: "032",
            },
            VillageCode {
                name: "塔尔沟村委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "西堡镇",
        code: "044",
        villages: &[
            VillageCode {
                name: "西堡社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "西堡村委会",
                code: "002",
            },
            VillageCode {
                name: "佐署村委会",
                code: "003",
            },
            VillageCode {
                name: "堡子村委会",
                code: "004",
            },
            VillageCode {
                name: "东花园村委会",
                code: "005",
            },
            VillageCode {
                name: "西花园村委会",
                code: "006",
            },
            VillageCode {
                name: "羊圈村委会",
                code: "007",
            },
            VillageCode {
                name: "寺尔寨村委会",
                code: "008",
            },
            VillageCode {
                name: "新平村委会",
                code: "009",
            },
            VillageCode {
                name: "东堡村委会",
                code: "010",
            },
            VillageCode {
                name: "西两旗村委会",
                code: "011",
            },
            VillageCode {
                name: "东两旗村委会",
                code: "012",
            },
            VillageCode {
                name: "葛家寨一村委会",
                code: "013",
            },
            VillageCode {
                name: "葛家寨二村委会",
                code: "014",
            },
            VillageCode {
                name: "条子沟村委会",
                code: "015",
            },
            VillageCode {
                name: "丰台沟村委会",
                code: "016",
            },
            VillageCode {
                name: "羊圈沟村委会",
                code: "017",
            },
            VillageCode {
                name: "青山村委会",
                code: "018",
            },
            VillageCode {
                name: "张李窑村委会",
                code: "019",
            },
            VillageCode {
                name: "鲍家沟村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "群加藏族乡",
        code: "045",
        villages: &[
            VillageCode {
                name: "唐阳村委会",
                code: "001",
            },
            VillageCode {
                name: "上圈村委会",
                code: "002",
            },
            VillageCode {
                name: "下圈村委会",
                code: "003",
            },
            VillageCode {
                name: "土康村委会",
                code: "004",
            },
            VillageCode {
                name: "来路村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "土门关乡",
        code: "046",
        villages: &[
            VillageCode {
                name: "土门关村委会",
                code: "001",
            },
            VillageCode {
                name: "坝沟村委会",
                code: "002",
            },
            VillageCode {
                name: "坝沟门村委会",
                code: "003",
            },
            VillageCode {
                name: "年坝村委会",
                code: "004",
            },
            VillageCode {
                name: "林马村委会",
                code: "005",
            },
            VillageCode {
                name: "后沟村委会",
                code: "006",
            },
            VillageCode {
                name: "加汝尔村委会",
                code: "007",
            },
            VillageCode {
                name: "下山庄村委会",
                code: "008",
            },
            VillageCode {
                name: "上山庄村委会",
                code: "009",
            },
            VillageCode {
                name: "红岭村委会",
                code: "010",
            },
            VillageCode {
                name: "王沟尔村委会",
                code: "011",
            },
            VillageCode {
                name: "贾尔藏村委会",
                code: "012",
            },
            VillageCode {
                name: "关跃村委会",
                code: "013",
            },
            VillageCode {
                name: "青峰村委会",
                code: "014",
            },
            VillageCode {
                name: "业隆村委会",
                code: "015",
            },
            VillageCode {
                name: "牙加村委会",
                code: "016",
            },
            VillageCode {
                name: "秋子沟村委会",
                code: "017",
            },
            VillageCode {
                name: "上阿卡村委会",
                code: "018",
            },
            VillageCode {
                name: "下阿卡村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "汉东回族乡",
        code: "047",
        villages: &[
            VillageCode {
                name: "后尧村委会",
                code: "001",
            },
            VillageCode {
                name: "冰沟村委会",
                code: "002",
            },
            VillageCode {
                name: "下麻尔村委会",
                code: "003",
            },
            VillageCode {
                name: "前窑村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "大才回族乡",
        code: "048",
        villages: &[
            VillageCode {
                name: "曲渠村委会",
                code: "001",
            },
            VillageCode {
                name: "上错隆村委会",
                code: "002",
            },
            VillageCode {
                name: "前沟村委会",
                code: "003",
            },
            VillageCode {
                name: "中沟村委会",
                code: "004",
            },
            VillageCode {
                name: "下后沟村委会",
                code: "005",
            },
            VillageCode {
                name: "下白崖村委会",
                code: "006",
            },
            VillageCode {
                name: "马场村委会",
                code: "007",
            },
            VillageCode {
                name: "上白崖村委会",
                code: "008",
            },
            VillageCode {
                name: "小沟尔村委会",
                code: "009",
            },
            VillageCode {
                name: "立欠村委会",
                code: "010",
            },
            VillageCode {
                name: "占林村委会",
                code: "011",
            },
            VillageCode {
                name: "小磨石沟村委会",
                code: "012",
            },
            VillageCode {
                name: "大磨石沟村委会",
                code: "013",
            },
            VillageCode {
                name: "上后沟村委会",
                code: "014",
            },
            VillageCode {
                name: "扎子村委会",
                code: "015",
            },
            VillageCode {
                name: "大才村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "海子沟乡",
        code: "049",
        villages: &[
            VillageCode {
                name: "中庄村委会",
                code: "001",
            },
            VillageCode {
                name: "阿滩村委会",
                code: "002",
            },
            VillageCode {
                name: "普通村委会",
                code: "003",
            },
            VillageCode {
                name: "陶家村委会",
                code: "004",
            },
            VillageCode {
                name: "甘沟村委会",
                code: "005",
            },
            VillageCode {
                name: "东沟村委会",
                code: "006",
            },
            VillageCode {
                name: "总堡村委会",
                code: "007",
            },
            VillageCode {
                name: "景家庄村委会",
                code: "008",
            },
            VillageCode {
                name: "万家坪村委会",
                code: "009",
            },
            VillageCode {
                name: "水滩村委会",
                code: "010",
            },
            VillageCode {
                name: "杨库托村委会",
                code: "011",
            },
            VillageCode {
                name: "王家庄村委会",
                code: "012",
            },
            VillageCode {
                name: "沟脑村委会",
                code: "013",
            },
            VillageCode {
                name: "海南庄村委会",
                code: "014",
            },
            VillageCode {
                name: "古城沟村委会",
                code: "015",
            },
            VillageCode {
                name: "东沟脑村委会",
                code: "016",
            },
            VillageCode {
                name: "黑沟村委会",
                code: "017",
            },
            VillageCode {
                name: "松家沟村委会",
                code: "018",
            },
            VillageCode {
                name: "顾家岭村委会",
                code: "019",
            },
            VillageCode {
                name: "薛姓庄村委会",
                code: "020",
            },
            VillageCode {
                name: "大有山村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "甘河工业园",
        code: "050",
        villages: &[VillageCode {
            name: "甘河工业园虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "康川街道",
        code: "051",
        villages: &[
            VillageCode {
                name: "锦绣苑居委会",
                code: "001",
            },
            VillageCode {
                name: "伊信苑居委会",
                code: "002",
            },
            VillageCode {
                name: "海欣苑居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "桥头镇",
        code: "052",
        villages: &[
            VillageCode {
                name: "人民路南居委会",
                code: "001",
            },
            VillageCode {
                name: "人民路北居委会",
                code: "002",
            },
            VillageCode {
                name: "园林路北居委会",
                code: "003",
            },
            VillageCode {
                name: "园林路南居委会",
                code: "004",
            },
            VillageCode {
                name: "解放路北居委会",
                code: "005",
            },
            VillageCode {
                name: "解放路南居委会",
                code: "006",
            },
            VillageCode {
                name: "元朔居委会",
                code: "007",
            },
            VillageCode {
                name: "铝电居委会",
                code: "008",
            },
            VillageCode {
                name: "小石山居委会",
                code: "009",
            },
            VillageCode {
                name: "矿山居委会",
                code: "010",
            },
            VillageCode {
                name: "八一居委会",
                code: "011",
            },
            VillageCode {
                name: "福园居委会",
                code: "012",
            },
            VillageCode {
                name: "牦牛山居委会",
                code: "013",
            },
            VillageCode {
                name: "大煤洞村委会",
                code: "014",
            },
            VillageCode {
                name: "元树尔村委会",
                code: "015",
            },
            VillageCode {
                name: "闇门滩村委会",
                code: "016",
            },
            VillageCode {
                name: "胡基沟村委会",
                code: "017",
            },
            VillageCode {
                name: "小煤洞村委会",
                code: "018",
            },
            VillageCode {
                name: "向阳堡村委会",
                code: "019",
            },
            VillageCode {
                name: "红河限村委会",
                code: "020",
            },
            VillageCode {
                name: "毛家寨村委会",
                code: "021",
            },
            VillageCode {
                name: "老营庄村委会",
                code: "022",
            },
            VillageCode {
                name: "大湾村委会",
                code: "023",
            },
            VillageCode {
                name: "水泉湾村委会",
                code: "024",
            },
            VillageCode {
                name: "毛家沟村委会",
                code: "025",
            },
            VillageCode {
                name: "贺家寨村委会",
                code: "026",
            },
            VillageCode {
                name: "古城村委会",
                code: "027",
            },
            VillageCode {
                name: "后庄村委会",
                code: "028",
            },
            VillageCode {
                name: "后子沟村委会",
                code: "029",
            },
            VillageCode {
                name: "南关村委会",
                code: "030",
            },
            VillageCode {
                name: "上关村委会",
                code: "031",
            },
            VillageCode {
                name: "上庙沟村委会",
                code: "032",
            },
            VillageCode {
                name: "下庙沟村委会",
                code: "033",
            },
            VillageCode {
                name: "新城村委会",
                code: "034",
            },
            VillageCode {
                name: "窑庄村委会",
                code: "035",
            },
        ],
    },
    TownCode {
        name: "城关镇",
        code: "053",
        villages: &[
            VillageCode {
                name: "佰胜居委会",
                code: "001",
            },
            VillageCode {
                name: "西关村委会",
                code: "002",
            },
            VillageCode {
                name: "上毛佰胜村委会",
                code: "003",
            },
            VillageCode {
                name: "下毛佰胜村委会",
                code: "004",
            },
            VillageCode {
                name: "西门村委会",
                code: "005",
            },
            VillageCode {
                name: "城关村委会",
                code: "006",
            },
            VillageCode {
                name: "东门村委会",
                code: "007",
            },
            VillageCode {
                name: "好来村委会",
                code: "008",
            },
            VillageCode {
                name: "龙曲村委会",
                code: "009",
            },
            VillageCode {
                name: "塔哇村委会",
                code: "010",
            },
            VillageCode {
                name: "贝寺村委会",
                code: "011",
            },
            VillageCode {
                name: "张家庄村委会",
                code: "012",
            },
            VillageCode {
                name: "大庄村委会",
                code: "013",
            },
            VillageCode {
                name: "阳坡庄村委会",
                code: "014",
            },
            VillageCode {
                name: "柳树庄村委会",
                code: "015",
            },
            VillageCode {
                name: "李家磨村委会",
                code: "016",
            },
            VillageCode {
                name: "下寺咀村委会",
                code: "017",
            },
            VillageCode {
                name: "上寺咀村委会",
                code: "018",
            },
            VillageCode {
                name: "沙巴图村委会",
                code: "019",
            },
            VillageCode {
                name: "林家台村委会",
                code: "020",
            },
            VillageCode {
                name: "铁家庄村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "塔尔镇",
        code: "054",
        villages: &[
            VillageCode {
                name: "塔尔湾居委会",
                code: "001",
            },
            VillageCode {
                name: "塔尔湾村委会",
                code: "002",
            },
            VillageCode {
                name: "贲坑滩村委会",
                code: "003",
            },
            VillageCode {
                name: "上旧庄村委会",
                code: "004",
            },
            VillageCode {
                name: "石家庄村委会",
                code: "005",
            },
            VillageCode {
                name: "下旧庄村委会",
                code: "006",
            },
            VillageCode {
                name: "中庄村委会",
                code: "007",
            },
            VillageCode {
                name: "泉沟村委会",
                code: "008",
            },
            VillageCode {
                name: "凉州庄村委会",
                code: "009",
            },
            VillageCode {
                name: "河州庄村委会",
                code: "010",
            },
            VillageCode {
                name: "韭菜沟村委会",
                code: "011",
            },
            VillageCode {
                name: "半沟村委会",
                code: "012",
            },
            VillageCode {
                name: "药草滩东庄村委会",
                code: "013",
            },
            VillageCode {
                name: "格达庄村委会",
                code: "014",
            },
            VillageCode {
                name: "塔尔沟村委会",
                code: "015",
            },
            VillageCode {
                name: "王庄村委会",
                code: "016",
            },
            VillageCode {
                name: "药草滩西庄村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "东峡镇",
        code: "055",
        villages: &[
            VillageCode {
                name: "衙门庄居委会",
                code: "001",
            },
            VillageCode {
                name: "衙门庄村委会",
                code: "002",
            },
            VillageCode {
                name: "南滩村委会",
                code: "003",
            },
            VillageCode {
                name: "麻其村委会",
                code: "004",
            },
            VillageCode {
                name: "克麻尔村委会",
                code: "005",
            },
            VillageCode {
                name: "杏花庄村委会",
                code: "006",
            },
            VillageCode {
                name: "仙米村委会",
                code: "007",
            },
            VillageCode {
                name: "田家沟村委会",
                code: "008",
            },
            VillageCode {
                name: "尔麻村委会",
                code: "009",
            },
            VillageCode {
                name: "康乐村委会",
                code: "010",
            },
            VillageCode {
                name: "老虎沟村委会",
                code: "011",
            },
            VillageCode {
                name: "刘家庄村委会",
                code: "012",
            },
            VillageCode {
                name: "多隆村委会",
                code: "013",
            },
            VillageCode {
                name: "元墩子村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "黄家寨镇",
        code: "056",
        villages: &[
            VillageCode {
                name: "黄东居委会",
                code: "001",
            },
            VillageCode {
                name: "青铝居委会",
                code: "002",
            },
            VillageCode {
                name: "东柳村委会",
                code: "003",
            },
            VillageCode {
                name: "黄东村委会",
                code: "004",
            },
            VillageCode {
                name: "黄西村委会",
                code: "005",
            },
            VillageCode {
                name: "上陶家寨村委会",
                code: "006",
            },
            VillageCode {
                name: "下陶家寨村委会",
                code: "007",
            },
            VillageCode {
                name: "上赵家磨村委会",
                code: "008",
            },
            VillageCode {
                name: "下赵家磨村委会",
                code: "009",
            },
            VillageCode {
                name: "许家寨村委会",
                code: "010",
            },
            VillageCode {
                name: "杨家寨村委会",
                code: "011",
            },
            VillageCode {
                name: "阿家村委会",
                code: "012",
            },
            VillageCode {
                name: "陈家村委会",
                code: "013",
            },
            VillageCode {
                name: "大哈门村委会",
                code: "014",
            },
            VillageCode {
                name: "平地庄村委会",
                code: "015",
            },
            VillageCode {
                name: "平乐村委会",
                code: "016",
            },
            VillageCode {
                name: "上柴堡村委会",
                code: "017",
            },
            VillageCode {
                name: "下柴堡村委会",
                code: "018",
            },
            VillageCode {
                name: "索家村委会",
                code: "019",
            },
            VillageCode {
                name: "寺尔庄村委会",
                code: "020",
            },
            VillageCode {
                name: "台台村委会",
                code: "021",
            },
            VillageCode {
                name: "兴太村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "长宁镇",
        code: "057",
        villages: &[
            VillageCode {
                name: "长宁居委会",
                code: "001",
            },
            VillageCode {
                name: "长宁村委会",
                code: "002",
            },
            VillageCode {
                name: "甘沟门村委会",
                code: "003",
            },
            VillageCode {
                name: "韩家山村委会",
                code: "004",
            },
            VillageCode {
                name: "宋家庄村委会",
                code: "005",
            },
            VillageCode {
                name: "王家庄村委会",
                code: "006",
            },
            VillageCode {
                name: "下严家庄村委会",
                code: "007",
            },
            VillageCode {
                name: "新寨村委会",
                code: "008",
            },
            VillageCode {
                name: "新添堡村委会",
                code: "009",
            },
            VillageCode {
                name: "上鲍堡村委会",
                code: "010",
            },
            VillageCode {
                name: "鲍家寨西村委会",
                code: "011",
            },
            VillageCode {
                name: "鲍家寨东村委会",
                code: "012",
            },
            VillageCode {
                name: "陈家庄村委会",
                code: "013",
            },
            VillageCode {
                name: "戴家庄村委会",
                code: "014",
            },
            VillageCode {
                name: "后子河东村委会",
                code: "015",
            },
            VillageCode {
                name: "河滩村委会",
                code: "016",
            },
            VillageCode {
                name: "红崖村委会",
                code: "017",
            },
            VillageCode {
                name: "康家村委会",
                code: "018",
            },
            VillageCode {
                name: "上孙家寨村委会",
                code: "019",
            },
            VillageCode {
                name: "双庙村委会",
                code: "020",
            },
            VillageCode {
                name: "田家村委会",
                code: "021",
            },
            VillageCode {
                name: "汪家村委会",
                code: "022",
            },
            VillageCode {
                name: "殷家村委会",
                code: "023",
            },
            VillageCode {
                name: "后子河西村委会",
                code: "024",
            },
            VillageCode {
                name: "中咀山村委会",
                code: "025",
            },
            VillageCode {
                name: "后子河尕庄村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "景阳镇",
        code: "058",
        villages: &[
            VillageCode {
                name: "景阳居委会",
                code: "001",
            },
            VillageCode {
                name: "大寺村委会",
                code: "002",
            },
            VillageCode {
                name: "小寺村委会",
                code: "003",
            },
            VillageCode {
                name: "甘树湾村委会",
                code: "004",
            },
            VillageCode {
                name: "后山村委会",
                code: "005",
            },
            VillageCode {
                name: "山城村委会",
                code: "006",
            },
            VillageCode {
                name: "什家村委会",
                code: "007",
            },
            VillageCode {
                name: "寺沟村委会",
                code: "008",
            },
            VillageCode {
                name: "苏家堡村委会",
                code: "009",
            },
            VillageCode {
                name: "土关村委会",
                code: "010",
            },
            VillageCode {
                name: "大寨村委会",
                code: "011",
            },
            VillageCode {
                name: "小寨村委会",
                code: "012",
            },
            VillageCode {
                name: "哈门村委会",
                code: "013",
            },
            VillageCode {
                name: "金冲村委会",
                code: "014",
            },
            VillageCode {
                name: "兰冲村委会",
                code: "015",
            },
            VillageCode {
                name: "龙泉村委会",
                code: "016",
            },
            VillageCode {
                name: "泉头村委会",
                code: "017",
            },
            VillageCode {
                name: "上岗冲村委会",
                code: "018",
            },
            VillageCode {
                name: "下岗冲村委会",
                code: "019",
            },
            VillageCode {
                name: "中岭村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "多林镇",
        code: "059",
        villages: &[
            VillageCode {
                name: "哈州居委会",
                code: "001",
            },
            VillageCode {
                name: "藏龙庄村委会",
                code: "002",
            },
            VillageCode {
                name: "哈州村委会",
                code: "003",
            },
            VillageCode {
                name: "口子庄村委会",
                code: "004",
            },
            VillageCode {
                name: "马场庄村委会",
                code: "005",
            },
            VillageCode {
                name: "上宽村委会",
                code: "006",
            },
            VillageCode {
                name: "上浪加村委会",
                code: "007",
            },
            VillageCode {
                name: "石头滩村委会",
                code: "008",
            },
            VillageCode {
                name: "吴什庄村委会",
                code: "009",
            },
            VillageCode {
                name: "下宽村委会",
                code: "010",
            },
            VillageCode {
                name: "下浪加村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "新庄镇",
        code: "060",
        villages: &[
            VillageCode {
                name: "新庄居委会",
                code: "001",
            },
            VillageCode {
                name: "新庄村委会",
                code: "002",
            },
            VillageCode {
                name: "申哇村委会",
                code: "003",
            },
            VillageCode {
                name: "硖门村委会",
                code: "004",
            },
            VillageCode {
                name: "台其庄村委会",
                code: "005",
            },
            VillageCode {
                name: "尕庄村委会",
                code: "006",
            },
            VillageCode {
                name: "中滩村委会",
                code: "007",
            },
            VillageCode {
                name: "吉仓村委会",
                code: "008",
            },
            VillageCode {
                name: "兰隆村委会",
                code: "009",
            },
            VillageCode {
                name: "红石崖村委会",
                code: "010",
            },
            VillageCode {
                name: "下山村委会",
                code: "011",
            },
            VillageCode {
                name: "上山村委会",
                code: "012",
            },
            VillageCode {
                name: "李家山村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "青林乡",
        code: "061",
        villages: &[
            VillageCode {
                name: "上阳山村委会",
                code: "001",
            },
            VillageCode {
                name: "下阳山村委会",
                code: "002",
            },
            VillageCode {
                name: "生地村委会",
                code: "003",
            },
            VillageCode {
                name: "毛合湾村委会",
                code: "004",
            },
            VillageCode {
                name: "卧马村委会",
                code: "005",
            },
            VillageCode {
                name: "雪里河村委会",
                code: "006",
            },
            VillageCode {
                name: "中庄沟村委会",
                code: "007",
            },
            VillageCode {
                name: "麻哈村委会",
                code: "008",
            },
            VillageCode {
                name: "棉格勒村委会",
                code: "009",
            },
            VillageCode {
                name: "白土垭豁村委会",
                code: "010",
            },
            VillageCode {
                name: "泉家湾村委会",
                code: "011",
            },
            VillageCode {
                name: "柳林滩村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "青山乡",
        code: "062",
        villages: &[
            VillageCode {
                name: "贺家庄村委会",
                code: "001",
            },
            VillageCode {
                name: "佐士图村委会",
                code: "002",
            },
            VillageCode {
                name: "龙卧村委会",
                code: "003",
            },
            VillageCode {
                name: "聂家沟村委会",
                code: "004",
            },
            VillageCode {
                name: "利顺村委会",
                code: "005",
            },
            VillageCode {
                name: "青山村委会",
                code: "006",
            },
            VillageCode {
                name: "沙岱村委会",
                code: "007",
            },
            VillageCode {
                name: "红泉村委会",
                code: "008",
            },
            VillageCode {
                name: "沙尕图村委会",
                code: "009",
            },
            VillageCode {
                name: "东山村委会",
                code: "010",
            },
            VillageCode {
                name: "官巴村委会",
                code: "011",
            },
            VillageCode {
                name: "多兰村委会",
                code: "012",
            },
            VillageCode {
                name: "西山村委会",
                code: "013",
            },
            VillageCode {
                name: "古娄马场村委会",
                code: "014",
            },
            VillageCode {
                name: "西北岔村委会",
                code: "015",
            },
            VillageCode {
                name: "生地乙卡村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "逊让乡",
        code: "063",
        villages: &[
            VillageCode {
                name: "逊布村委会",
                code: "001",
            },
            VillageCode {
                name: "塘坊村委会",
                code: "002",
            },
            VillageCode {
                name: "尕漏村委会",
                code: "003",
            },
            VillageCode {
                name: "上后拉村委会",
                code: "004",
            },
            VillageCode {
                name: "武胜沟村委会",
                code: "005",
            },
            VillageCode {
                name: "后拉村委会",
                code: "006",
            },
            VillageCode {
                name: "兰家村委会",
                code: "007",
            },
            VillageCode {
                name: "安宁滩村委会",
                code: "008",
            },
            VillageCode {
                name: "逊布沟村委会",
                code: "009",
            },
            VillageCode {
                name: "庄头村委会",
                code: "010",
            },
            VillageCode {
                name: "八里庄村委会",
                code: "011",
            },
            VillageCode {
                name: "古谷家村委会",
                code: "012",
            },
            VillageCode {
                name: "麻什藏村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "极乐乡",
        code: "064",
        villages: &[
            VillageCode {
                name: "极拉上庄村委会",
                code: "001",
            },
            VillageCode {
                name: "极拉下庄村委会",
                code: "002",
            },
            VillageCode {
                name: "极拉口村委会",
                code: "003",
            },
            VillageCode {
                name: "下和衷村委会",
                code: "004",
            },
            VillageCode {
                name: "上和衷村委会",
                code: "005",
            },
            VillageCode {
                name: "阳坡村委会",
                code: "006",
            },
            VillageCode {
                name: "宗阳沟村委会",
                code: "007",
            },
            VillageCode {
                name: "深沟村委会",
                code: "008",
            },
            VillageCode {
                name: "极拉后庄村委会",
                code: "009",
            },
            VillageCode {
                name: "崖湾村委会",
                code: "010",
            },
            VillageCode {
                name: "岔水坝村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "石山乡",
        code: "065",
        villages: &[
            VillageCode {
                name: "铧尖村委会",
                code: "001",
            },
            VillageCode {
                name: "冰沟村委会",
                code: "002",
            },
            VillageCode {
                name: "红垭豁村委会",
                code: "003",
            },
            VillageCode {
                name: "上丰积村委会",
                code: "004",
            },
            VillageCode {
                name: "下丰积村委会",
                code: "005",
            },
            VillageCode {
                name: "石板滩村委会",
                code: "006",
            },
            VillageCode {
                name: "西坡村委会",
                code: "007",
            },
            VillageCode {
                name: "小沟村委会",
                code: "008",
            },
            VillageCode {
                name: "杂户村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "宝库乡",
        code: "066",
        villages: &[
            VillageCode {
                name: "牛场居委会",
                code: "001",
            },
            VillageCode {
                name: "油房卡村委会",
                code: "002",
            },
            VillageCode {
                name: "巴音牧委会",
                code: "003",
            },
            VillageCode {
                name: "俄博图村委会",
                code: "004",
            },
            VillageCode {
                name: "寺塘村委会",
                code: "005",
            },
            VillageCode {
                name: "孔家梁村委会",
                code: "006",
            },
            VillageCode {
                name: "五间房村委会",
                code: "007",
            },
            VillageCode {
                name: "张家滩村委会",
                code: "008",
            },
            VillageCode {
                name: "纳塄沟村委会",
                code: "009",
            },
            VillageCode {
                name: "哈家咀村委会",
                code: "010",
            },
            VillageCode {
                name: "水草滩村委会",
                code: "011",
            },
            VillageCode {
                name: "祁汉沟村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "斜沟乡",
        code: "067",
        villages: &[
            VillageCode {
                name: "河滩庄村委会",
                code: "001",
            },
            VillageCode {
                name: "大业坝村委会",
                code: "002",
            },
            VillageCode {
                name: "小业坝村委会",
                code: "003",
            },
            VillageCode {
                name: "斜沟村委会",
                code: "004",
            },
            VillageCode {
                name: "上窑洞庄村委会",
                code: "005",
            },
            VillageCode {
                name: "下窑洞庄村委会",
                code: "006",
            },
            VillageCode {
                name: "业坝台村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "良教乡",
        code: "068",
        villages: &[
            VillageCode {
                name: "下治泉村委会",
                code: "001",
            },
            VillageCode {
                name: "上治泉村委会",
                code: "002",
            },
            VillageCode {
                name: "桥尔沟村委会",
                code: "003",
            },
            VillageCode {
                name: "前跃村委会",
                code: "004",
            },
            VillageCode {
                name: "良教沟村委会",
                code: "005",
            },
            VillageCode {
                name: "白崖村委会",
                code: "006",
            },
            VillageCode {
                name: "下甘沟村委会",
                code: "007",
            },
            VillageCode {
                name: "上甘沟村委会",
                code: "008",
            },
            VillageCode {
                name: "松林村委会",
                code: "009",
            },
            VillageCode {
                name: "沙布村委会",
                code: "010",
            },
            VillageCode {
                name: "石庄村委会",
                code: "011",
            },
            VillageCode {
                name: "煤洞沟村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "向化藏族乡",
        code: "069",
        villages: &[
            VillageCode {
                name: "流水口村委会",
                code: "001",
            },
            VillageCode {
                name: "达隆村委会",
                code: "002",
            },
            VillageCode {
                name: "立树尔村委会",
                code: "003",
            },
            VillageCode {
                name: "将军沟村委会",
                code: "004",
            },
            VillageCode {
                name: "麻庄村委会",
                code: "005",
            },
            VillageCode {
                name: "三角城村委会",
                code: "006",
            },
            VillageCode {
                name: "上滩村委会",
                code: "007",
            },
            VillageCode {
                name: "下滩村委会",
                code: "008",
            },
            VillageCode {
                name: "驿卡村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "桦林乡",
        code: "070",
        villages: &[
            VillageCode {
                name: "胜利村委会",
                code: "001",
            },
            VillageCode {
                name: "阿家沟村委会",
                code: "002",
            },
            VillageCode {
                name: "瓜拉大庄村委会",
                code: "003",
            },
            VillageCode {
                name: "东庄村委会",
                code: "004",
            },
            VillageCode {
                name: "鄂博沟村委会",
                code: "005",
            },
            VillageCode {
                name: "关巴村委会",
                code: "006",
            },
            VillageCode {
                name: "吕顺村委会",
                code: "007",
            },
            VillageCode {
                name: "七棵树村委会",
                code: "008",
            },
            VillageCode {
                name: "台子村委会",
                code: "009",
            },
            VillageCode {
                name: "西沟村委会",
                code: "010",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "011",
            },
            VillageCode {
                name: "桦林庄村委会",
                code: "012",
            },
            VillageCode {
                name: "贲哇沟村委会",
                code: "013",
            },
            VillageCode {
                name: "峡口村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "朔北藏族乡",
        code: "071",
        villages: &[
            VillageCode {
                name: "县东居委会",
                code: "001",
            },
            VillageCode {
                name: "阿家堡村委会",
                code: "002",
            },
            VillageCode {
                name: "菜子口村委会",
                code: "003",
            },
            VillageCode {
                name: "代同庄村委会",
                code: "004",
            },
            VillageCode {
                name: "李家堡村委会",
                code: "005",
            },
            VillageCode {
                name: "马场村委会",
                code: "006",
            },
            VillageCode {
                name: "下吉哇村委会",
                code: "007",
            },
            VillageCode {
                name: "小龙院村委会",
                code: "008",
            },
            VillageCode {
                name: "药匠台村委会",
                code: "009",
            },
            VillageCode {
                name: "永丰村委会",
                code: "010",
            },
            VillageCode {
                name: "八寺崖村委会",
                code: "011",
            },
            VillageCode {
                name: "白崖沟村委会",
                code: "012",
            },
            VillageCode {
                name: "边麻沟村委会",
                code: "013",
            },
            VillageCode {
                name: "花科庄村委会",
                code: "014",
            },
            VillageCode {
                name: "东至沟村委会",
                code: "015",
            },
            VillageCode {
                name: "拉浪台村委会",
                code: "016",
            },
            VillageCode {
                name: "旧拉浪村委会",
                code: "017",
            },
            VillageCode {
                name: "麻家庄村委会",
                code: "018",
            },
            VillageCode {
                name: "郑家沟村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "城关镇",
        code: "072",
        villages: &[
            VillageCode {
                name: "城台社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "人民街社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "南小路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "万安街社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "万丰社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "西大街社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "西关街社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "城郊社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "涌兴村委会",
                code: "009",
            },
            VillageCode {
                name: "万丰村委会",
                code: "010",
            },
            VillageCode {
                name: "纳隆口村委会",
                code: "011",
            },
            VillageCode {
                name: "河拉台村委会",
                code: "012",
            },
            VillageCode {
                name: "国光村委会",
                code: "013",
            },
            VillageCode {
                name: "光华村委会",
                code: "014",
            },
            VillageCode {
                name: "尕庄村委会",
                code: "015",
            },
            VillageCode {
                name: "董家庄村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "大华镇",
        code: "073",
        villages: &[
            VillageCode {
                name: "拉拉口村委会",
                code: "001",
            },
            VillageCode {
                name: "何家庄村委会",
                code: "002",
            },
            VillageCode {
                name: "大华村委会",
                code: "003",
            },
            VillageCode {
                name: "池汉村委会",
                code: "004",
            },
            VillageCode {
                name: "新胜村委会",
                code: "005",
            },
            VillageCode {
                name: "石崖庄村委会",
                code: "006",
            },
            VillageCode {
                name: "三条沟村委会",
                code: "007",
            },
            VillageCode {
                name: "莫布拉脑村委会",
                code: "008",
            },
            VillageCode {
                name: "莫布拉村委会",
                code: "009",
            },
            VillageCode {
                name: "拉卓奈村委会",
                code: "010",
            },
            VillageCode {
                name: "窑洞村委会",
                code: "011",
            },
            VillageCode {
                name: "黄茂村委会",
                code: "012",
            },
            VillageCode {
                name: "巴汉村委会",
                code: "013",
            },
            VillageCode {
                name: "塔湾村委会",
                code: "014",
            },
            VillageCode {
                name: "崖根村委会",
                code: "015",
            },
            VillageCode {
                name: "红土湾村委会",
                code: "016",
            },
            VillageCode {
                name: "河南村委会",
                code: "017",
            },
            VillageCode {
                name: "后庄村委会",
                code: "018",
            },
            VillageCode {
                name: "石嘴村委会",
                code: "019",
            },
            VillageCode {
                name: "巴燕吉盖村委会",
                code: "020",
            },
            VillageCode {
                name: "晒尔村委会",
                code: "021",
            },
            VillageCode {
                name: "托思胡村委会",
                code: "022",
            },
            VillageCode {
                name: "牙麻岔村委会",
                code: "023",
            },
            VillageCode {
                name: "阿家图村委会",
                code: "024",
            },
            VillageCode {
                name: "纳隆沟村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "东峡乡",
        code: "074",
        villages: &[
            VillageCode {
                name: "石崖庄村委会",
                code: "001",
            },
            VillageCode {
                name: "新民村委会",
                code: "002",
            },
            VillageCode {
                name: "响河村委会",
                code: "003",
            },
            VillageCode {
                name: "下脖项村委会",
                code: "004",
            },
            VillageCode {
                name: "峨头山村委会",
                code: "005",
            },
            VillageCode {
                name: "兰占巴村委会",
                code: "006",
            },
            VillageCode {
                name: "灰条沟村委会",
                code: "007",
            },
            VillageCode {
                name: "灰条口村委会",
                code: "008",
            },
            VillageCode {
                name: "拉尔贯村委会",
                code: "009",
            },
            VillageCode {
                name: "北山村委会",
                code: "010",
            },
            VillageCode {
                name: "柏树堂村委会",
                code: "011",
            },
            VillageCode {
                name: "山岔村委会",
                code: "012",
            },
            VillageCode {
                name: "炭窑村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "日月藏族乡",
        code: "075",
        villages: &[
            VillageCode {
                name: "兔尔干村委会",
                code: "001",
            },
            VillageCode {
                name: "药水村委会",
                code: "002",
            },
            VillageCode {
                name: "雪隆村委会",
                code: "003",
            },
            VillageCode {
                name: "小茶石浪村委会",
                code: "004",
            },
            VillageCode {
                name: "大茶石浪村委会",
                code: "005",
            },
            VillageCode {
                name: "下若药村委会",
                code: "006",
            },
            VillageCode {
                name: "兔尔台村委会",
                code: "007",
            },
            VillageCode {
                name: "乙细村委会",
                code: "008",
            },
            VillageCode {
                name: "寺滩村委会",
                code: "009",
            },
            VillageCode {
                name: "上若药村委会",
                code: "010",
            },
            VillageCode {
                name: "山根村委会",
                code: "011",
            },
            VillageCode {
                name: "若药堂村委会",
                code: "012",
            },
            VillageCode {
                name: "日月山村委会",
                code: "013",
            },
            VillageCode {
                name: "前滩村委会",
                code: "014",
            },
            VillageCode {
                name: "莫多吉村委会",
                code: "015",
            },
            VillageCode {
                name: "克素尔村委会",
                code: "016",
            },
            VillageCode {
                name: "哈城村委会",
                code: "017",
            },
            VillageCode {
                name: "尕庄村委会",
                code: "018",
            },
            VillageCode {
                name: "尕恰莫多村委会",
                code: "019",
            },
            VillageCode {
                name: "大石头村委会",
                code: "020",
            },
            VillageCode {
                name: "池汉素村委会",
                code: "021",
            },
            VillageCode {
                name: "本炕村委会",
                code: "022",
            },
            VillageCode {
                name: "牧场村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "和平乡",
        code: "076",
        villages: &[
            VillageCode {
                name: "茶汉素村委会",
                code: "001",
            },
            VillageCode {
                name: "马场湾村委会",
                code: "002",
            },
            VillageCode {
                name: "董家脑村委会",
                code: "003",
            },
            VillageCode {
                name: "小高陵村委会",
                code: "004",
            },
            VillageCode {
                name: "大高陵村委会",
                code: "005",
            },
            VillageCode {
                name: "上拉雾台村委会",
                code: "006",
            },
            VillageCode {
                name: "下拉雾台村委会",
                code: "007",
            },
            VillageCode {
                name: "曲布滩村委会",
                code: "008",
            },
            VillageCode {
                name: "马家湾村委会",
                code: "009",
            },
            VillageCode {
                name: "马场台村委会",
                code: "010",
            },
            VillageCode {
                name: "隆和村委会",
                code: "011",
            },
            VillageCode {
                name: "加牙麻村委会",
                code: "012",
            },
            VillageCode {
                name: "和平村委会",
                code: "013",
            },
            VillageCode {
                name: "尕庄村委会",
                code: "014",
            },
            VillageCode {
                name: "茶曲村委会",
                code: "015",
            },
            VillageCode {
                name: "蒙古道村委会",
                code: "016",
            },
            VillageCode {
                name: "草沟村委会",
                code: "017",
            },
            VillageCode {
                name: "白水村委会",
                code: "018",
            },
            VillageCode {
                name: "泉尔湾村委会",
                code: "019",
            },
            VillageCode {
                name: "刘家台村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "波航乡",
        code: "077",
        villages: &[
            VillageCode {
                name: "波航村委会",
                code: "001",
            },
            VillageCode {
                name: "纳隆村委会",
                code: "002",
            },
            VillageCode {
                name: "西岔村委会",
                code: "003",
            },
            VillageCode {
                name: "南岔村委会",
                code: "004",
            },
            VillageCode {
                name: "石崖湾村委会",
                code: "005",
            },
            VillageCode {
                name: "上台村委会",
                code: "006",
            },
            VillageCode {
                name: "下台村委会",
                code: "007",
            },
            VillageCode {
                name: "上泉尔村委会",
                code: "008",
            },
            VillageCode {
                name: "泉尔湾村委会",
                code: "009",
            },
            VillageCode {
                name: "麻尼台村委会",
                code: "010",
            },
            VillageCode {
                name: "浪湾村委会",
                code: "011",
            },
            VillageCode {
                name: "胡思洞村委会",
                code: "012",
            },
            VillageCode {
                name: "甘沟村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "申中乡",
        code: "078",
        villages: &[
            VillageCode {
                name: "卡路村委会",
                code: "001",
            },
            VillageCode {
                name: "申中村委会",
                code: "002",
            },
            VillageCode {
                name: "前沟村委会",
                code: "003",
            },
            VillageCode {
                name: "莫布拉村委会",
                code: "004",
            },
            VillageCode {
                name: "庙沟脑村委会",
                code: "005",
            },
            VillageCode {
                name: "立达村委会",
                code: "006",
            },
            VillageCode {
                name: "口子村委会",
                code: "007",
            },
            VillageCode {
                name: "星泉村委会",
                code: "008",
            },
            VillageCode {
                name: "俊家庄村委会",
                code: "009",
            },
            VillageCode {
                name: "韭菜沟村委会",
                code: "010",
            },
            VillageCode {
                name: "窑庄村委会",
                code: "011",
            },
            VillageCode {
                name: "后沟村委会",
                code: "012",
            },
            VillageCode {
                name: "河拉村委会",
                code: "013",
            },
            VillageCode {
                name: "庙沟村委会",
                code: "014",
            },
            VillageCode {
                name: "大山根村委会",
                code: "015",
            },
            VillageCode {
                name: "大路村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "巴燕乡",
        code: "079",
        villages: &[
            VillageCode {
                name: "巴燕村委会",
                code: "001",
            },
            VillageCode {
                name: "元山村委会",
                code: "002",
            },
            VillageCode {
                name: "新寺村委会",
                code: "003",
            },
            VillageCode {
                name: "下寺村委会",
                code: "004",
            },
            VillageCode {
                name: "下浪湾村委会",
                code: "005",
            },
            VillageCode {
                name: "下胡丹村委会",
                code: "006",
            },
            VillageCode {
                name: "西岭台村委会",
                code: "007",
            },
            VillageCode {
                name: "石门尔村委会",
                code: "008",
            },
            VillageCode {
                name: "上浪湾村委会",
                code: "009",
            },
            VillageCode {
                name: "上胡旦村委会",
                code: "010",
            },
            VillageCode {
                name: "莫合尔村委会",
                code: "011",
            },
            VillageCode {
                name: "居士浪村委会",
                code: "012",
            },
            VillageCode {
                name: "福海村委会",
                code: "013",
            },
            VillageCode {
                name: "巴燕峡村委会",
                code: "014",
            },
            VillageCode {
                name: "扎汉村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "寺寨乡",
        code: "080",
        villages: &[
            VillageCode {
                name: "小寺尔村委会",
                code: "001",
            },
            VillageCode {
                name: "铧尖村委会",
                code: "002",
            },
            VillageCode {
                name: "下寨村委会",
                code: "003",
            },
            VillageCode {
                name: "西扎湾村委会",
                code: "004",
            },
            VillageCode {
                name: "草原村委会",
                code: "005",
            },
            VillageCode {
                name: "长岭村委会",
                code: "006",
            },
            VillageCode {
                name: "上寨村委会",
                code: "007",
            },
            VillageCode {
                name: "簸箕湾村委会",
                code: "008",
            },
            VillageCode {
                name: "乌图村委会",
                code: "009",
            },
            VillageCode {
                name: "烽火村委会",
                code: "010",
            },
            VillageCode {
                name: "马脊岭村委会",
                code: "011",
            },
            VillageCode {
                name: "阳坡湾村委会",
                code: "012",
            },
            VillageCode {
                name: "五岭村委会",
                code: "013",
            },
        ],
    },
];

static TOWNS_QH_002: [TownCode; 10] = [
    TownCode {
        name: "东关大街街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "北关社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "慈幼社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "五一社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "清真巷街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "团结社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "夏都花园社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "南小街社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "凤凰园社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "磨尔园社区居民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "大众街街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "凯旋社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "园山社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "树林巷社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "共和路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "德令哈社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "安泰社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "康西社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "梨园社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "友谊村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "先进村村民委员会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "周家泉街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "杨家巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "建国路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "为民巷社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "白家河湾社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "联合村村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "火车站街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "车站社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "中庄社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "幸福社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "富民路社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "八一路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "学院社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "康东社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "康南社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "康宁社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "博雅路南社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "团结村村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "林家崖街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "蓝天社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "站西社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "林家崖村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "路家庄村村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "乐家湾镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "乐家湾社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "金桥路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "明杏社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "泉景社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "上十里铺村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "塔尔山村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "下十里铺村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "乐家湾村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "杨沟湾村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "韵家口镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "东兴社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "东盛社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "纺织社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "育才路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "中庄村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "褚家营村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "小寨村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "韵家口村民委员会",
                code: "008",
            },
            VillageCode {
                name: "泮子山村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "付家寨村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "朱家庄村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "王家庄村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "曹家寨村村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "东川工业园",
        code: "010",
        villages: &[VillageCode {
            name: "东川工业园虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_QH_003: [TownCode; 7] = [
    TownCode {
        name: "城中街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "龙城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "五星社区居委会",
                code: "002",
            },
            VillageCode {
                name: "东门社区居委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "公园街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "弯塘社区居委会",
                code: "001",
            },
            VillageCode {
                name: "柳侯社区居委会",
                code: "002",
            },
            VillageCode {
                name: "罗池社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东台社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "中南街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "福柳新都社区居委会",
                code: "001",
            },
            VillageCode {
                name: "西门社区居委会",
                code: "002",
            },
            VillageCode {
                name: "映山社区居委会",
                code: "003",
            },
            VillageCode {
                name: "青云社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "沿江街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "桂中社区居委会",
                code: "001",
            },
            VillageCode {
                name: "潭东社区居委会",
                code: "002",
            },
            VillageCode {
                name: "王座社区居委会",
                code: "003",
            },
            VillageCode {
                name: "金泰苑社区居委会",
                code: "004",
            },
            VillageCode {
                name: "鹿山社区居委会",
                code: "005",
            },
            VillageCode {
                name: "文源社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "潭中街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "广西科技大学社区居委会",
                code: "001",
            },
            VillageCode {
                name: "文昌社区居委会",
                code: "002",
            },
            VillageCode {
                name: "西雅社区居委会",
                code: "003",
            },
            VillageCode {
                name: "阳光100社区居委会",
                code: "004",
            },
            VillageCode {
                name: "康顺社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "晨华社区居委会",
                code: "006",
            },
            VillageCode {
                name: "窑埠村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "河东街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "山水社区居委会",
                code: "001",
            },
            VillageCode {
                name: "清华坊社区居委会",
                code: "002",
            },
            VillageCode {
                name: "文博社区居委会",
                code: "003",
            },
            VillageCode {
                name: "鹿园社区居委会",
                code: "004",
            },
            VillageCode {
                name: "前茅社区居委会",
                code: "005",
            },
            VillageCode {
                name: "河东村委会",
                code: "006",
            },
            VillageCode {
                name: "牛车坪村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "静兰街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "三门江社区居委会",
                code: "001",
            },
            VillageCode {
                name: "华展社区居委会",
                code: "002",
            },
            VillageCode {
                name: "春风社区居委会",
                code: "003",
            },
            VillageCode {
                name: "静兰村委会",
                code: "004",
            },
            VillageCode {
                name: "柳东村委会",
                code: "005",
            },
            VillageCode {
                name: "环江村委会",
                code: "006",
            },
        ],
    },
];

static TOWNS_QH_004: [TownCode; 8] = [
    TownCode {
        name: "西关大街街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "南气象巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "贾小社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北气象巷社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "古城台街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "学院巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "青年巷社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "昆仑路东居委会",
                code: "003",
            },
            VillageCode {
                name: "昆仑路西居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "虎台街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "海晏路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "医财巷东居委会",
                code: "002",
            },
            VillageCode {
                name: "医财巷西居委会",
                code: "003",
            },
            VillageCode {
                name: "冷湖路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "新西社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "殷家庄社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "虎台社区居委会",
                code: "007",
            },
            VillageCode {
                name: "杨家寨村委会",
                code: "008",
            },
            VillageCode {
                name: "苏家河湾村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "胜利路街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "西交通巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "公园巷社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "东交通巷社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "北商业巷社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "兴海路街道",
        code: "005",
        villages: &[
            VillageCode {
                name: "兴胜巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "中华巷社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "尕寺巷社区居民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "文汇路街道",
        code: "006",
        villages: &[
            VillageCode {
                name: "文亭巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "文博路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "海湖广场社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "科普路社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "通海路街道",
        code: "007",
        villages: &[
            VillageCode {
                name: "文成路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "桃李路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "光华路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "文景街西社区居委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "彭家寨镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "西川南路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "富兴路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "彭家寨村委会",
                code: "003",
            },
            VillageCode {
                name: "西北园村委会",
                code: "004",
            },
            VillageCode {
                name: "刘家寨村委会",
                code: "005",
            },
            VillageCode {
                name: "汉庄村委会",
                code: "006",
            },
            VillageCode {
                name: "张家湾村委会",
                code: "007",
            },
            VillageCode {
                name: "杨家湾村委会",
                code: "008",
            },
            VillageCode {
                name: "阴山堂村委会",
                code: "009",
            },
            VillageCode {
                name: "火西村委会",
                code: "010",
            },
            VillageCode {
                name: "火东村委会",
                code: "011",
            },
            VillageCode {
                name: "晨光村委会",
                code: "012",
            },
        ],
    },
];

static TOWNS_QH_005: [TownCode; 7] = [
    TownCode {
        name: "朝阳街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "朝阳社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "山川社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "北川河东路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "祁连路西社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "北山社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "朝阳西路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "北园村委会",
                code: "007",
            },
            VillageCode {
                name: "朝阳村委会",
                code: "008",
            },
            VillageCode {
                name: "祁家城村委会",
                code: "009",
            },
            VillageCode {
                name: "寺台子村委会",
                code: "010",
            },
            VillageCode {
                name: "新民村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "小桥大街街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "建设巷社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新海桥社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "毛胜寺社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "小桥社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "新世纪社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "西海路社区居委会",
                code: "006",
            },
            VillageCode {
                name: "毛胜寺村委会",
                code: "007",
            },
            VillageCode {
                name: "北杏园村委会",
                code: "008",
            },
            VillageCode {
                name: "陶家寨村委会",
                code: "009",
            },
            VillageCode {
                name: "陶新村委会",
                code: "010",
            },
            VillageCode {
                name: "小桥村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "马坊街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "西杏园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "马坊东社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "青工社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "汽运社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "光明社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "新村社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "欣乐社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "幸福社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "海湖桥西社区居委会",
                code: "009",
            },
            VillageCode {
                name: "西杏园村委会",
                code: "010",
            },
            VillageCode {
                name: "马坊村委会",
                code: "011",
            },
            VillageCode {
                name: "盐庄村委会",
                code: "012",
            },
            VillageCode {
                name: "三其村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "火车西站街道",
        code: "004",
        villages: &[
            VillageCode {
                name: "萨尔斯堡社区",
                code: "001",
            },
            VillageCode {
                name: "美丽水街社区",
                code: "002",
            },
            VillageCode {
                name: "湟水河畔社区",
                code: "003",
            },
            VillageCode {
                name: "火车西站社区",
                code: "004",
            },
            VillageCode {
                name: "盐庄社区",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "大堡子镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "一机床社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "工具厂社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "大堡子村委会",
                code: "003",
            },
            VillageCode {
                name: "严小村委会",
                code: "004",
            },
            VillageCode {
                name: "宋家寨村委会",
                code: "005",
            },
            VillageCode {
                name: "晋家湾村委会",
                code: "006",
            },
            VillageCode {
                name: "鲍家寨村委会",
                code: "007",
            },
            VillageCode {
                name: "朱南村委会",
                code: "008",
            },
            VillageCode {
                name: "朱北村委会",
                code: "009",
            },
            VillageCode {
                name: "吧浪村委会",
                code: "010",
            },
            VillageCode {
                name: "吴仲村委会",
                code: "011",
            },
            VillageCode {
                name: "汪家寨村委会",
                code: "012",
            },
            VillageCode {
                name: "乙其寨村委会",
                code: "013",
            },
            VillageCode {
                name: "陶南村委会",
                code: "014",
            },
            VillageCode {
                name: "陶北村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "廿里铺镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "生物园社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "泉湾社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "高教路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "廿里铺村委会",
                code: "004",
            },
            VillageCode {
                name: "花园台村委会",
                code: "005",
            },
            VillageCode {
                name: "孙家寨村委会",
                code: "006",
            },
            VillageCode {
                name: "小寨村委会",
                code: "007",
            },
            VillageCode {
                name: "莫家庄村委会",
                code: "008",
            },
            VillageCode {
                name: "新村委会",
                code: "009",
            },
            VillageCode {
                name: "石头磊村委会",
                code: "010",
            },
            VillageCode {
                name: "魏家庄村委会",
                code: "011",
            },
            VillageCode {
                name: "九家湾村委会",
                code: "012",
            },
            VillageCode {
                name: "郭家塔村委会",
                code: "013",
            },
            VillageCode {
                name: "双苏堡村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "生物科技产业园",
        code: "007",
        villages: &[VillageCode {
            name: "生物科技产业园虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_QH_006: [TownCode; 17] = [
    TownCode {
        name: "田家寨镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "田家寨社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "泗洱河村委会",
                code: "002",
            },
            VillageCode {
                name: "谢家台村委会",
                code: "003",
            },
            VillageCode {
                name: "毛家台村委会",
                code: "004",
            },
            VillageCode {
                name: "田家寨村委会",
                code: "005",
            },
            VillageCode {
                name: "毛一村委会",
                code: "006",
            },
            VillageCode {
                name: "毛二村委会",
                code: "007",
            },
            VillageCode {
                name: "河湾村委会",
                code: "008",
            },
            VillageCode {
                name: "新村村委会",
                code: "009",
            },
            VillageCode {
                name: "李家台村委会",
                code: "010",
            },
            VillageCode {
                name: "梁家村委会",
                code: "011",
            },
            VillageCode {
                name: "石沟村委会",
                code: "012",
            },
            VillageCode {
                name: "谢家村委会",
                code: "013",
            },
            VillageCode {
                name: "大卡阳村委会",
                code: "014",
            },
            VillageCode {
                name: "小卡阳村委会",
                code: "015",
            },
            VillageCode {
                name: "甘家村委会",
                code: "016",
            },
            VillageCode {
                name: "公牙村委会",
                code: "017",
            },
            VillageCode {
                name: "喇家村委会",
                code: "018",
            },
            VillageCode {
                name: "窑洞村委会",
                code: "019",
            },
            VillageCode {
                name: "下营一村委会",
                code: "020",
            },
            VillageCode {
                name: "上营一村委会",
                code: "021",
            },
            VillageCode {
                name: "拉尕村委会",
                code: "022",
            },
            VillageCode {
                name: "黄蒿台村委会",
                code: "023",
            },
            VillageCode {
                name: "流水沟村委会",
                code: "024",
            },
            VillageCode {
                name: "群塔村委会",
                code: "025",
            },
            VillageCode {
                name: "阳坡一村委会",
                code: "026",
            },
            VillageCode {
                name: "鸽堂村委会",
                code: "027",
            },
            VillageCode {
                name: "丹麻村委会",
                code: "028",
            },
            VillageCode {
                name: "坪台村委会",
                code: "029",
            },
            VillageCode {
                name: "永丰村委会",
                code: "030",
            },
            VillageCode {
                name: "安宁村委会",
                code: "031",
            },
            VillageCode {
                name: "李家庄村委会",
                code: "032",
            },
            VillageCode {
                name: "阳坡二村委会",
                code: "033",
            },
            VillageCode {
                name: "阴坡村委会",
                code: "034",
            },
            VillageCode {
                name: "尕院村委会",
                code: "035",
            },
            VillageCode {
                name: "上营二村委会",
                code: "036",
            },
            VillageCode {
                name: "下营二村委会",
                code: "037",
            },
            VillageCode {
                name: "卜家台村委会",
                code: "038",
            },
            VillageCode {
                name: "台口子村委会",
                code: "039",
            },
            VillageCode {
                name: "沙尔湾村委会",
                code: "040",
            },
            VillageCode {
                name: "上洛麻村委会",
                code: "041",
            },
            VillageCode {
                name: "下洛麻村委会",
                code: "042",
            },
            VillageCode {
                name: "鲍家村委会",
                code: "043",
            },
            VillageCode {
                name: "马昌沟村委会",
                code: "044",
            },
        ],
    },
    TownCode {
        name: "上新庄镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "上新庄社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "刘小庄村委会",
                code: "002",
            },
            VillageCode {
                name: "班麻坡村委会",
                code: "003",
            },
            VillageCode {
                name: "东台村委会",
                code: "004",
            },
            VillageCode {
                name: "西庄村委会",
                code: "005",
            },
            VillageCode {
                name: "东沟滩村委会",
                code: "006",
            },
            VillageCode {
                name: "马家滩村委会",
                code: "007",
            },
            VillageCode {
                name: "红牙合村委会",
                code: "008",
            },
            VillageCode {
                name: "尧滩村委会",
                code: "009",
            },
            VillageCode {
                name: "尧湾村委会",
                code: "010",
            },
            VillageCode {
                name: "下峡门村委会",
                code: "011",
            },
            VillageCode {
                name: "上峡门村委会",
                code: "012",
            },
            VillageCode {
                name: "申北村委会",
                code: "013",
            },
            VillageCode {
                name: "申南村委会",
                code: "014",
            },
            VillageCode {
                name: "水草沟村委会",
                code: "015",
            },
            VillageCode {
                name: "河滩村委会",
                code: "016",
            },
            VillageCode {
                name: "黑城村委会",
                code: "017",
            },
            VillageCode {
                name: "上新庄村委会",
                code: "018",
            },
            VillageCode {
                name: "阳坡台村委会",
                code: "019",
            },
            VillageCode {
                name: "地广村委会",
                code: "020",
            },
            VillageCode {
                name: "华山村委会",
                code: "021",
            },
            VillageCode {
                name: "骟马台村委会",
                code: "022",
            },
            VillageCode {
                name: "加牙村委会",
                code: "023",
            },
            VillageCode {
                name: "新城村委会",
                code: "024",
            },
            VillageCode {
                name: "周德村委会",
                code: "025",
            },
            VillageCode {
                name: "班隆村委会",
                code: "026",
            },
            VillageCode {
                name: "马场村委会",
                code: "027",
            },
            VillageCode {
                name: "七家庄村委会",
                code: "028",
            },
            VillageCode {
                name: "海马沟村委会",
                code: "029",
            },
            VillageCode {
                name: "下台村委会",
                code: "030",
            },
            VillageCode {
                name: "上台村委会",
                code: "031",
            },
            VillageCode {
                name: "白路尔村委会",
                code: "032",
            },
            VillageCode {
                name: "白石头村委会",
                code: "033",
            },
            VillageCode {
                name: "静房村委会",
                code: "034",
            },
        ],
    },
    TownCode {
        name: "鲁沙尔镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "金塔社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "团结社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "和平社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "莲湖社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "水滩村委会",
                code: "005",
            },
            VillageCode {
                name: "孔家村委会",
                code: "006",
            },
            VillageCode {
                name: "赵家庄村委会",
                code: "007",
            },
            VillageCode {
                name: "昂藏村委会",
                code: "008",
            },
            VillageCode {
                name: "和平村委会",
                code: "009",
            },
            VillageCode {
                name: "河滩村委会",
                code: "010",
            },
            VillageCode {
                name: "团结村委会",
                code: "011",
            },
            VillageCode {
                name: "东山村委会",
                code: "012",
            },
            VillageCode {
                name: "西山村委会",
                code: "013",
            },
            VillageCode {
                name: "塔尔湾村委会",
                code: "014",
            },
            VillageCode {
                name: "青一村委会",
                code: "015",
            },
            VillageCode {
                name: "青二村委会",
                code: "016",
            },
            VillageCode {
                name: "南门村委会",
                code: "017",
            },
            VillageCode {
                name: "海马泉村委会",
                code: "018",
            },
            VillageCode {
                name: "新村村委会",
                code: "019",
            },
            VillageCode {
                name: "红崖沟村委会",
                code: "020",
            },
            VillageCode {
                name: "陈家滩村委会",
                code: "021",
            },
            VillageCode {
                name: "西村村委会",
                code: "022",
            },
            VillageCode {
                name: "东村村委会",
                code: "023",
            },
            VillageCode {
                name: "徐家寨村委会",
                code: "024",
            },
            VillageCode {
                name: "石咀一村委会",
                code: "025",
            },
            VillageCode {
                name: "下重台村委会",
                code: "026",
            },
            VillageCode {
                name: "白土庄村委会",
                code: "027",
            },
            VillageCode {
                name: "地窑村委会",
                code: "028",
            },
            VillageCode {
                name: "阴坡村委会",
                code: "029",
            },
            VillageCode {
                name: "阳坡村委会",
                code: "030",
            },
            VillageCode {
                name: "石咀二村委会",
                code: "031",
            },
            VillageCode {
                name: "吊庄村委会",
                code: "032",
            },
            VillageCode {
                name: "甘河沿村委会",
                code: "033",
            },
            VillageCode {
                name: "阿家庄村委会",
                code: "034",
            },
            VillageCode {
                name: "朱家庄村委会",
                code: "035",
            },
            VillageCode {
                name: "青石坡村委会",
                code: "036",
            },
            VillageCode {
                name: "上重台村委会",
                code: "037",
            },
        ],
    },
    TownCode {
        name: "甘河滩镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "甘河滩社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "甘河村委会",
                code: "002",
            },
            VillageCode {
                name: "页沟村委会",
                code: "003",
            },
            VillageCode {
                name: "坡东村委会",
                code: "004",
            },
            VillageCode {
                name: "坡西村委会",
                code: "005",
            },
            VillageCode {
                name: "隆寺干村委会",
                code: "006",
            },
            VillageCode {
                name: "下中沟村委会",
                code: "007",
            },
            VillageCode {
                name: "上中沟村委会",
                code: "008",
            },
            VillageCode {
                name: "元山尔村委会",
                code: "009",
            },
            VillageCode {
                name: "卡跃村委会",
                code: "010",
            },
            VillageCode {
                name: "上营村委会",
                code: "011",
            },
            VillageCode {
                name: "下营村委会",
                code: "012",
            },
            VillageCode {
                name: "上河湾村委会",
                code: "013",
            },
            VillageCode {
                name: "下河湾村委会",
                code: "014",
            },
            VillageCode {
                name: "李九村委会",
                code: "015",
            },
            VillageCode {
                name: "前跃村委会",
                code: "016",
            },
            VillageCode {
                name: "东湾村委会",
                code: "017",
            },
            VillageCode {
                name: "黄一村委会",
                code: "018",
            },
            VillageCode {
                name: "黄二村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "共和镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "共和社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "北村村委会",
                code: "002",
            },
            VillageCode {
                name: "南村村委会",
                code: "003",
            },
            VillageCode {
                name: "山甲村委会",
                code: "004",
            },
            VillageCode {
                name: "石城村委会",
                code: "005",
            },
            VillageCode {
                name: "后营村委会",
                code: "006",
            },
            VillageCode {
                name: "前营村委会",
                code: "007",
            },
            VillageCode {
                name: "木场村委会",
                code: "008",
            },
            VillageCode {
                name: "上直沟村委会",
                code: "009",
            },
            VillageCode {
                name: "花勒城村委会",
                code: "010",
            },
            VillageCode {
                name: "王家山村委会",
                code: "011",
            },
            VillageCode {
                name: "苏尔吉村委会",
                code: "012",
            },
            VillageCode {
                name: "转嘴村委会",
                code: "013",
            },
            VillageCode {
                name: "东岔村委会",
                code: "014",
            },
            VillageCode {
                name: "西岔村委会",
                code: "015",
            },
            VillageCode {
                name: "盘道村委会",
                code: "016",
            },
            VillageCode {
                name: "西台村委会",
                code: "017",
            },
            VillageCode {
                name: "东台村委会",
                code: "018",
            },
            VillageCode {
                name: "新湾村委会",
                code: "019",
            },
            VillageCode {
                name: "葱湾村委会",
                code: "020",
            },
            VillageCode {
                name: "达草沟村委会",
                code: "021",
            },
            VillageCode {
                name: "下马申村委会",
                code: "022",
            },
            VillageCode {
                name: "上马申村委会",
                code: "023",
            },
            VillageCode {
                name: "河湾村委会",
                code: "024",
            },
            VillageCode {
                name: "后街村委会",
                code: "025",
            },
            VillageCode {
                name: "新庄村委会",
                code: "026",
            },
            VillageCode {
                name: "尕庄村委会",
                code: "027",
            },
            VillageCode {
                name: "庄科脑村委会",
                code: "028",
            },
            VillageCode {
                name: "尖达村委会",
                code: "029",
            },
            VillageCode {
                name: "萱麻湾村委会",
                code: "030",
            },
            VillageCode {
                name: "押必村委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "多巴镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "多巴金城社区",
                code: "001",
            },
            VillageCode {
                name: "多巴通海社区",
                code: "002",
            },
            VillageCode {
                name: "小寨村委会",
                code: "003",
            },
            VillageCode {
                name: "双寨村委会",
                code: "004",
            },
            VillageCode {
                name: "大崖沟村委会",
                code: "005",
            },
            VillageCode {
                name: "韦家庄村委会",
                code: "006",
            },
            VillageCode {
                name: "甘河门村委会",
                code: "007",
            },
            VillageCode {
                name: "新墩村委会",
                code: "008",
            },
            VillageCode {
                name: "康城村委会",
                code: "009",
            },
            VillageCode {
                name: "城东村委会",
                code: "010",
            },
            VillageCode {
                name: "城西村委会",
                code: "011",
            },
            VillageCode {
                name: "王家庄村委会",
                code: "012",
            },
            VillageCode {
                name: "城中村委会",
                code: "013",
            },
            VillageCode {
                name: "银格达村委会",
                code: "014",
            },
            VillageCode {
                name: "丰胜村委会",
                code: "015",
            },
            VillageCode {
                name: "国寺营村委会",
                code: "016",
            },
            VillageCode {
                name: "石板沟村委会",
                code: "017",
            },
            VillageCode {
                name: "扎麻隆村委会",
                code: "018",
            },
            VillageCode {
                name: "马申茂村委会",
                code: "019",
            },
            VillageCode {
                name: "加拉山村委会",
                code: "020",
            },
            VillageCode {
                name: "尚什家村委会",
                code: "021",
            },
            VillageCode {
                name: "羊圈村委会",
                code: "022",
            },
            VillageCode {
                name: "多巴四村委会",
                code: "023",
            },
            VillageCode {
                name: "指挥庄村委会",
                code: "024",
            },
            VillageCode {
                name: "多巴二村委会",
                code: "025",
            },
            VillageCode {
                name: "燕尔沟村委会",
                code: "026",
            },
            VillageCode {
                name: "大掌村委会",
                code: "027",
            },
            VillageCode {
                name: "多巴三村委会",
                code: "028",
            },
            VillageCode {
                name: "多巴一村委会",
                code: "029",
            },
            VillageCode {
                name: "黑嘴村委会",
                code: "030",
            },
            VillageCode {
                name: "沙窝尔村委会",
                code: "031",
            },
            VillageCode {
                name: "幸福村委会",
                code: "032",
            },
            VillageCode {
                name: "初哇村委会",
                code: "033",
            },
            VillageCode {
                name: "玉拉村委会",
                code: "034",
            },
            VillageCode {
                name: "合尔营村委会",
                code: "035",
            },
            VillageCode {
                name: "丹麻寺村委会",
                code: "036",
            },
            VillageCode {
                name: "奔巴口村委会",
                code: "037",
            },
            VillageCode {
                name: "油房台村委会",
                code: "038",
            },
            VillageCode {
                name: "年家庄村委会",
                code: "039",
            },
            VillageCode {
                name: "杨家台村委会",
                code: "040",
            },
            VillageCode {
                name: "北沟村委会",
                code: "041",
            },
            VillageCode {
                name: "目尔加村委会",
                code: "042",
            },
            VillageCode {
                name: "拉卡山村委会",
                code: "043",
            },
            VillageCode {
                name: "尕尔加村委会",
                code: "044",
            },
            VillageCode {
                name: "中村村委会",
                code: "045",
            },
            VillageCode {
                name: "洛尔洞村委会",
                code: "046",
            },
        ],
    },
    TownCode {
        name: "拦隆口镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "拦隆口社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "新村村委会",
                code: "002",
            },
            VillageCode {
                name: "扎什营村委会",
                code: "003",
            },
            VillageCode {
                name: "巴达村委会",
                code: "004",
            },
            VillageCode {
                name: "班仲营村委会",
                code: "005",
            },
            VillageCode {
                name: "端巴营村委会",
                code: "006",
            },
            VillageCode {
                name: "西岔村委会",
                code: "007",
            },
            VillageCode {
                name: "下鲁尔加村委会",
                code: "008",
            },
            VillageCode {
                name: "上鲁尔加村委会",
                code: "009",
            },
            VillageCode {
                name: "拦隆口村委会",
                code: "010",
            },
            VillageCode {
                name: "白杨口村委会",
                code: "011",
            },
            VillageCode {
                name: "东拉科村委会",
                code: "012",
            },
            VillageCode {
                name: "南门一村委会",
                code: "013",
            },
            VillageCode {
                name: "前庄村委会",
                code: "014",
            },
            VillageCode {
                name: "中庄村委会",
                code: "015",
            },
            VillageCode {
                name: "上庄村委会",
                code: "016",
            },
            VillageCode {
                name: "卡阳村委会",
                code: "017",
            },
            VillageCode {
                name: "白崖一村委会",
                code: "018",
            },
            VillageCode {
                name: "泥隆台村委会",
                code: "019",
            },
            VillageCode {
                name: "泥隆口村委会",
                code: "020",
            },
            VillageCode {
                name: "桥西村委会",
                code: "021",
            },
            VillageCode {
                name: "千西村委会",
                code: "022",
            },
            VillageCode {
                name: "千东村委会",
                code: "023",
            },
            VillageCode {
                name: "铁家营村委会",
                code: "024",
            },
            VillageCode {
                name: "上营村委会",
                code: "025",
            },
            VillageCode {
                name: "上寺村委会",
                code: "026",
            },
            VillageCode {
                name: "拦隆一村委会",
                code: "027",
            },
            VillageCode {
                name: "拦隆二村委会",
                code: "028",
            },
            VillageCode {
                name: "白崖二村委会",
                code: "029",
            },
            VillageCode {
                name: "合尔营村委会",
                code: "030",
            },
            VillageCode {
                name: "麻子营村委会",
                code: "031",
            },
            VillageCode {
                name: "后河尔村委会",
                code: "032",
            },
            VillageCode {
                name: "佰什营村委会",
                code: "033",
            },
            VillageCode {
                name: "图巴营村委会",
                code: "034",
            },
            VillageCode {
                name: "尼麻隆村委会",
                code: "035",
            },
            VillageCode {
                name: "上红土沟村委会",
                code: "036",
            },
            VillageCode {
                name: "下红土沟村委会",
                code: "037",
            },
            VillageCode {
                name: "红林村委会",
                code: "038",
            },
            VillageCode {
                name: "民族村委会",
                code: "039",
            },
            VillageCode {
                name: "邦隆村委会",
                code: "040",
            },
            VillageCode {
                name: "民联村委会",
                code: "041",
            },
            VillageCode {
                name: "峡口村委会",
                code: "042",
            },
            VillageCode {
                name: "南门二村委会",
                code: "043",
            },
        ],
    },
    TownCode {
        name: "上五庄镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "上五庄社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "合尔盖村委会",
                code: "002",
            },
            VillageCode {
                name: "北纳村委会",
                code: "003",
            },
            VillageCode {
                name: "马场村委会",
                code: "004",
            },
            VillageCode {
                name: "友爱村委会",
                code: "005",
            },
            VillageCode {
                name: "邦吧村委会",
                code: "006",
            },
            VillageCode {
                name: "华科村委会",
                code: "007",
            },
            VillageCode {
                name: "纳卜藏村委会",
                code: "008",
            },
            VillageCode {
                name: "包勒村委会",
                code: "009",
            },
            VillageCode {
                name: "拉斯目村委会",
                code: "010",
            },
            VillageCode {
                name: "北庄村委会",
                code: "011",
            },
            VillageCode {
                name: "峡口村委会",
                code: "012",
            },
            VillageCode {
                name: "甫崖村委会",
                code: "013",
            },
            VillageCode {
                name: "拉尔宁一村委会",
                code: "014",
            },
            VillageCode {
                name: "拉尔宁二村委会",
                code: "015",
            },
            VillageCode {
                name: "拉尔宁三村委会",
                code: "016",
            },
            VillageCode {
                name: "黄草沟村委会",
                code: "017",
            },
            VillageCode {
                name: "大寺沟一村委会",
                code: "018",
            },
            VillageCode {
                name: "大寺沟二村委会",
                code: "019",
            },
            VillageCode {
                name: "业宏村委会",
                code: "020",
            },
            VillageCode {
                name: "拉目台村委会",
                code: "021",
            },
            VillageCode {
                name: "小寺沟村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "李家山镇",
        code: "009",
        villages: &[
            VillageCode {
                name: "李家山社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "崖头村委会",
                code: "002",
            },
            VillageCode {
                name: "董家湾村委会",
                code: "003",
            },
            VillageCode {
                name: "柳树庄村委会",
                code: "004",
            },
            VillageCode {
                name: "王家堡村委会",
                code: "005",
            },
            VillageCode {
                name: "卡约村委会",
                code: "006",
            },
            VillageCode {
                name: "下坪村委会",
                code: "007",
            },
            VillageCode {
                name: "上坪村委会",
                code: "008",
            },
            VillageCode {
                name: "包家庄村委会",
                code: "009",
            },
            VillageCode {
                name: "陈家庄村委会",
                code: "010",
            },
            VillageCode {
                name: "汉水沟村委会",
                code: "011",
            },
            VillageCode {
                name: "毛尔茨沟村委会",
                code: "012",
            },
            VillageCode {
                name: "马营村委会",
                code: "013",
            },
            VillageCode {
                name: "下油房村委会",
                code: "014",
            },
            VillageCode {
                name: "纳家村委会",
                code: "015",
            },
            VillageCode {
                name: "岗岔村委会",
                code: "016",
            },
            VillageCode {
                name: "上西河村委会",
                code: "017",
            },
            VillageCode {
                name: "下西河村委会",
                code: "018",
            },
            VillageCode {
                name: "新添堡村委会",
                code: "019",
            },
            VillageCode {
                name: "河湾村委会",
                code: "020",
            },
            VillageCode {
                name: "吉家村委会",
                code: "021",
            },
            VillageCode {
                name: "甘家村委会",
                code: "022",
            },
            VillageCode {
                name: "新庄村委会",
                code: "023",
            },
            VillageCode {
                name: "勺麻营村委会",
                code: "024",
            },
            VillageCode {
                name: "大路村委会",
                code: "025",
            },
            VillageCode {
                name: "李家山村委会",
                code: "026",
            },
            VillageCode {
                name: "马圈沟村委会",
                code: "027",
            },
            VillageCode {
                name: "金跃村委会",
                code: "028",
            },
            VillageCode {
                name: "峡口村委会",
                code: "029",
            },
            VillageCode {
                name: "阳坡村委会",
                code: "030",
            },
            VillageCode {
                name: "阴坡村委会",
                code: "031",
            },
            VillageCode {
                name: "恰罗村委会",
                code: "032",
            },
            VillageCode {
                name: "塔尔沟村委会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "西堡镇",
        code: "010",
        villages: &[
            VillageCode {
                name: "西堡社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "西堡村委会",
                code: "002",
            },
            VillageCode {
                name: "佐署村委会",
                code: "003",
            },
            VillageCode {
                name: "堡子村委会",
                code: "004",
            },
            VillageCode {
                name: "东花园村委会",
                code: "005",
            },
            VillageCode {
                name: "西花园村委会",
                code: "006",
            },
            VillageCode {
                name: "羊圈村委会",
                code: "007",
            },
            VillageCode {
                name: "寺尔寨村委会",
                code: "008",
            },
            VillageCode {
                name: "新平村委会",
                code: "009",
            },
            VillageCode {
                name: "东堡村委会",
                code: "010",
            },
            VillageCode {
                name: "西两旗村委会",
                code: "011",
            },
            VillageCode {
                name: "东两旗村委会",
                code: "012",
            },
            VillageCode {
                name: "葛家寨一村委会",
                code: "013",
            },
            VillageCode {
                name: "葛家寨二村委会",
                code: "014",
            },
            VillageCode {
                name: "条子沟村委会",
                code: "015",
            },
            VillageCode {
                name: "丰台沟村委会",
                code: "016",
            },
            VillageCode {
                name: "羊圈沟村委会",
                code: "017",
            },
            VillageCode {
                name: "青山村委会",
                code: "018",
            },
            VillageCode {
                name: "张李窑村委会",
                code: "019",
            },
            VillageCode {
                name: "鲍家沟村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "群加藏族乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "唐阳村委会",
                code: "001",
            },
            VillageCode {
                name: "上圈村委会",
                code: "002",
            },
            VillageCode {
                name: "下圈村委会",
                code: "003",
            },
            VillageCode {
                name: "土康村委会",
                code: "004",
            },
            VillageCode {
                name: "来路村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "土门关乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "土门关村委会",
                code: "001",
            },
            VillageCode {
                name: "坝沟村委会",
                code: "002",
            },
            VillageCode {
                name: "坝沟门村委会",
                code: "003",
            },
            VillageCode {
                name: "年坝村委会",
                code: "004",
            },
            VillageCode {
                name: "林马村委会",
                code: "005",
            },
            VillageCode {
                name: "后沟村委会",
                code: "006",
            },
            VillageCode {
                name: "加汝尔村委会",
                code: "007",
            },
            VillageCode {
                name: "下山庄村委会",
                code: "008",
            },
            VillageCode {
                name: "上山庄村委会",
                code: "009",
            },
            VillageCode {
                name: "红岭村委会",
                code: "010",
            },
            VillageCode {
                name: "王沟尔村委会",
                code: "011",
            },
            VillageCode {
                name: "贾尔藏村委会",
                code: "012",
            },
            VillageCode {
                name: "关跃村委会",
                code: "013",
            },
            VillageCode {
                name: "青峰村委会",
                code: "014",
            },
            VillageCode {
                name: "业隆村委会",
                code: "015",
            },
            VillageCode {
                name: "牙加村委会",
                code: "016",
            },
            VillageCode {
                name: "秋子沟村委会",
                code: "017",
            },
            VillageCode {
                name: "上阿卡村委会",
                code: "018",
            },
            VillageCode {
                name: "下阿卡村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "汉东回族乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "后尧村委会",
                code: "001",
            },
            VillageCode {
                name: "冰沟村委会",
                code: "002",
            },
            VillageCode {
                name: "下麻尔村委会",
                code: "003",
            },
            VillageCode {
                name: "前窑村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "大才回族乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "曲渠村委会",
                code: "001",
            },
            VillageCode {
                name: "上错隆村委会",
                code: "002",
            },
            VillageCode {
                name: "前沟村委会",
                code: "003",
            },
            VillageCode {
                name: "中沟村委会",
                code: "004",
            },
            VillageCode {
                name: "下后沟村委会",
                code: "005",
            },
            VillageCode {
                name: "下白崖村委会",
                code: "006",
            },
            VillageCode {
                name: "马场村委会",
                code: "007",
            },
            VillageCode {
                name: "上白崖村委会",
                code: "008",
            },
            VillageCode {
                name: "小沟尔村委会",
                code: "009",
            },
            VillageCode {
                name: "立欠村委会",
                code: "010",
            },
            VillageCode {
                name: "占林村委会",
                code: "011",
            },
            VillageCode {
                name: "小磨石沟村委会",
                code: "012",
            },
            VillageCode {
                name: "大磨石沟村委会",
                code: "013",
            },
            VillageCode {
                name: "上后沟村委会",
                code: "014",
            },
            VillageCode {
                name: "扎子村委会",
                code: "015",
            },
            VillageCode {
                name: "大才村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "海子沟乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "中庄村委会",
                code: "001",
            },
            VillageCode {
                name: "阿滩村委会",
                code: "002",
            },
            VillageCode {
                name: "普通村委会",
                code: "003",
            },
            VillageCode {
                name: "陶家村委会",
                code: "004",
            },
            VillageCode {
                name: "甘沟村委会",
                code: "005",
            },
            VillageCode {
                name: "东沟村委会",
                code: "006",
            },
            VillageCode {
                name: "总堡村委会",
                code: "007",
            },
            VillageCode {
                name: "景家庄村委会",
                code: "008",
            },
            VillageCode {
                name: "万家坪村委会",
                code: "009",
            },
            VillageCode {
                name: "水滩村委会",
                code: "010",
            },
            VillageCode {
                name: "杨库托村委会",
                code: "011",
            },
            VillageCode {
                name: "王家庄村委会",
                code: "012",
            },
            VillageCode {
                name: "沟脑村委会",
                code: "013",
            },
            VillageCode {
                name: "海南庄村委会",
                code: "014",
            },
            VillageCode {
                name: "古城沟村委会",
                code: "015",
            },
            VillageCode {
                name: "东沟脑村委会",
                code: "016",
            },
            VillageCode {
                name: "黑沟村委会",
                code: "017",
            },
            VillageCode {
                name: "松家沟村委会",
                code: "018",
            },
            VillageCode {
                name: "顾家岭村委会",
                code: "019",
            },
            VillageCode {
                name: "薛姓庄村委会",
                code: "020",
            },
            VillageCode {
                name: "大有山村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "甘河工业园",
        code: "016",
        villages: &[VillageCode {
            name: "甘河工业园虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "康川街道",
        code: "017",
        villages: &[
            VillageCode {
                name: "锦绣苑居委会",
                code: "001",
            },
            VillageCode {
                name: "伊信苑居委会",
                code: "002",
            },
            VillageCode {
                name: "海欣苑居委会",
                code: "003",
            },
        ],
    },
];

static TOWNS_QH_007: [TownCode; 6] = [
    TownCode {
        name: "大通街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "远望社区居委会",
                code: "001",
            },
            VillageCode {
                name: "站后社区居委会",
                code: "002",
            },
            VillageCode {
                name: "居北社区居委会",
                code: "003",
            },
            VillageCode {
                name: "居南社区居委会",
                code: "004",
            },
            VillageCode {
                name: "矿南社区居委会",
                code: "005",
            },
            VillageCode {
                name: "瀚城社区居委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "上窑镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "上窑社区居委会",
                code: "001",
            },
            VillageCode {
                name: "张郢村委会",
                code: "002",
            },
            VillageCode {
                name: "方楼村委会",
                code: "003",
            },
            VillageCode {
                name: "外窑村委会",
                code: "004",
            },
            VillageCode {
                name: "云南岗村委会",
                code: "005",
            },
            VillageCode {
                name: "红光村委会",
                code: "006",
            },
            VillageCode {
                name: "马庙村村委会",
                code: "007",
            },
            VillageCode {
                name: "窑河村委会",
                code: "008",
            },
            VillageCode {
                name: "余巷村委会",
                code: "009",
            },
            VillageCode {
                name: "马岗村委会",
                code: "010",
            },
            VillageCode {
                name: "上窑村委会",
                code: "011",
            },
            VillageCode {
                name: "泉源村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "洛河镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "洛河社区居委会",
                code: "001",
            },
            VillageCode {
                name: "洛西社区居委会",
                code: "002",
            },
            VillageCode {
                name: "洛电社区居委会",
                code: "003",
            },
            VillageCode {
                name: "农场社区居委会",
                code: "004",
            },
            VillageCode {
                name: "金庄社区居委会",
                code: "005",
            },
            VillageCode {
                name: "田东社区居委会",
                code: "006",
            },
            VillageCode {
                name: "朝阳社区居委会",
                code: "007",
            },
            VillageCode {
                name: "胡圩社区居委会",
                code: "008",
            },
            VillageCode {
                name: "淮建村村委会",
                code: "009",
            },
            VillageCode {
                name: "洛河村委会",
                code: "010",
            },
            VillageCode {
                name: "刘郑村村委会",
                code: "011",
            },
            VillageCode {
                name: "陈郢村村委会",
                code: "012",
            },
            VillageCode {
                name: "刘郢村村委会",
                code: "013",
            },
            VillageCode {
                name: "陈庄村委会",
                code: "014",
            },
            VillageCode {
                name: "王庄村委会",
                code: "015",
            },
            VillageCode {
                name: "林巷村村委会",
                code: "016",
            },
            VillageCode {
                name: "西湖村委会",
                code: "017",
            },
            VillageCode {
                name: "宫集村村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "九龙岗镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "红旗社区居委会",
                code: "001",
            },
            VillageCode {
                name: "淮舜社区居委会",
                code: "002",
            },
            VillageCode {
                name: "重华社区居委会",
                code: "003",
            },
            VillageCode {
                name: "新建社区居委会",
                code: "004",
            },
            VillageCode {
                name: "泽润园社区居委会",
                code: "005",
            },
            VillageCode {
                name: "夏农村委会",
                code: "006",
            },
            VillageCode {
                name: "夏菜村委会",
                code: "007",
            },
            VillageCode {
                name: "曹店村委会",
                code: "008",
            },
            VillageCode {
                name: "魏嘴村委会",
                code: "009",
            },
            VillageCode {
                name: "王楼村委会",
                code: "010",
            },
            VillageCode {
                name: "方岗村委会",
                code: "011",
            },
            VillageCode {
                name: "九龙岗村委会",
                code: "012",
            },
            VillageCode {
                name: "陈巷村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "孔店乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "新街村村委会",
                code: "001",
            },
            VillageCode {
                name: "王祠村村委会",
                code: "002",
            },
            VillageCode {
                name: "东河村村委会",
                code: "003",
            },
            VillageCode {
                name: "古堆村委会",
                code: "004",
            },
            VillageCode {
                name: "大郢村村委会",
                code: "005",
            },
            VillageCode {
                name: "洪圩村村委会",
                code: "006",
            },
            VillageCode {
                name: "柿元村村委会",
                code: "007",
            },
            VillageCode {
                name: "河沿村村委会",
                code: "008",
            },
            VillageCode {
                name: "欢灯村村委会",
                code: "009",
            },
            VillageCode {
                name: "松林村委会",
                code: "010",
            },
            VillageCode {
                name: "孔店村村委会",
                code: "011",
            },
            VillageCode {
                name: "胡拐村村委会",
                code: "012",
            },
            VillageCode {
                name: "刘庄村村委会",
                code: "013",
            },
            VillageCode {
                name: "舜南村村委会",
                code: "014",
            },
            VillageCode {
                name: "安塘村村委会",
                code: "015",
            },
            VillageCode {
                name: "毛郢村村委会",
                code: "016",
            },
            VillageCode {
                name: "黄山村委会",
                code: "017",
            },
            VillageCode {
                name: "费郢村委会",
                code: "018",
            },
            VillageCode {
                name: "吴大郢村村委会",
                code: "019",
            },
            VillageCode {
                name: "马厂村委会",
                code: "020",
            },
            VillageCode {
                name: "沈大郢村村委会",
                code: "021",
            },
            VillageCode {
                name: "新华村村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "淮南经济开发区",
        code: "006",
        villages: &[VillageCode {
            name: "龙池社区居委会",
            code: "001",
        }],
    },
];

static TOWNS_QH_008: [TownCode; 9] = [
    TownCode {
        name: "城关镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城台社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "人民街社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "南小路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "万安街社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "万丰社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "西大街社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "西关街社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "城郊社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "涌兴村委会",
                code: "009",
            },
            VillageCode {
                name: "万丰村委会",
                code: "010",
            },
            VillageCode {
                name: "纳隆口村委会",
                code: "011",
            },
            VillageCode {
                name: "河拉台村委会",
                code: "012",
            },
            VillageCode {
                name: "国光村委会",
                code: "013",
            },
            VillageCode {
                name: "光华村委会",
                code: "014",
            },
            VillageCode {
                name: "尕庄村委会",
                code: "015",
            },
            VillageCode {
                name: "董家庄村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "大华镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "拉拉口村委会",
                code: "001",
            },
            VillageCode {
                name: "何家庄村委会",
                code: "002",
            },
            VillageCode {
                name: "大华村委会",
                code: "003",
            },
            VillageCode {
                name: "池汉村委会",
                code: "004",
            },
            VillageCode {
                name: "新胜村委会",
                code: "005",
            },
            VillageCode {
                name: "石崖庄村委会",
                code: "006",
            },
            VillageCode {
                name: "三条沟村委会",
                code: "007",
            },
            VillageCode {
                name: "莫布拉脑村委会",
                code: "008",
            },
            VillageCode {
                name: "莫布拉村委会",
                code: "009",
            },
            VillageCode {
                name: "拉卓奈村委会",
                code: "010",
            },
            VillageCode {
                name: "窑洞村委会",
                code: "011",
            },
            VillageCode {
                name: "黄茂村委会",
                code: "012",
            },
            VillageCode {
                name: "巴汉村委会",
                code: "013",
            },
            VillageCode {
                name: "塔湾村委会",
                code: "014",
            },
            VillageCode {
                name: "崖根村委会",
                code: "015",
            },
            VillageCode {
                name: "红土湾村委会",
                code: "016",
            },
            VillageCode {
                name: "河南村委会",
                code: "017",
            },
            VillageCode {
                name: "后庄村委会",
                code: "018",
            },
            VillageCode {
                name: "石嘴村委会",
                code: "019",
            },
            VillageCode {
                name: "巴燕吉盖村委会",
                code: "020",
            },
            VillageCode {
                name: "晒尔村委会",
                code: "021",
            },
            VillageCode {
                name: "托思胡村委会",
                code: "022",
            },
            VillageCode {
                name: "牙麻岔村委会",
                code: "023",
            },
            VillageCode {
                name: "阿家图村委会",
                code: "024",
            },
            VillageCode {
                name: "纳隆沟村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "东峡乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "石崖庄村委会",
                code: "001",
            },
            VillageCode {
                name: "新民村委会",
                code: "002",
            },
            VillageCode {
                name: "响河村委会",
                code: "003",
            },
            VillageCode {
                name: "下脖项村委会",
                code: "004",
            },
            VillageCode {
                name: "峨头山村委会",
                code: "005",
            },
            VillageCode {
                name: "兰占巴村委会",
                code: "006",
            },
            VillageCode {
                name: "灰条沟村委会",
                code: "007",
            },
            VillageCode {
                name: "灰条口村委会",
                code: "008",
            },
            VillageCode {
                name: "拉尔贯村委会",
                code: "009",
            },
            VillageCode {
                name: "北山村委会",
                code: "010",
            },
            VillageCode {
                name: "柏树堂村委会",
                code: "011",
            },
            VillageCode {
                name: "山岔村委会",
                code: "012",
            },
            VillageCode {
                name: "炭窑村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "日月藏族乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "兔尔干村委会",
                code: "001",
            },
            VillageCode {
                name: "药水村委会",
                code: "002",
            },
            VillageCode {
                name: "雪隆村委会",
                code: "003",
            },
            VillageCode {
                name: "小茶石浪村委会",
                code: "004",
            },
            VillageCode {
                name: "大茶石浪村委会",
                code: "005",
            },
            VillageCode {
                name: "下若药村委会",
                code: "006",
            },
            VillageCode {
                name: "兔尔台村委会",
                code: "007",
            },
            VillageCode {
                name: "乙细村委会",
                code: "008",
            },
            VillageCode {
                name: "寺滩村委会",
                code: "009",
            },
            VillageCode {
                name: "上若药村委会",
                code: "010",
            },
            VillageCode {
                name: "山根村委会",
                code: "011",
            },
            VillageCode {
                name: "若药堂村委会",
                code: "012",
            },
            VillageCode {
                name: "日月山村委会",
                code: "013",
            },
            VillageCode {
                name: "前滩村委会",
                code: "014",
            },
            VillageCode {
                name: "莫多吉村委会",
                code: "015",
            },
            VillageCode {
                name: "克素尔村委会",
                code: "016",
            },
            VillageCode {
                name: "哈城村委会",
                code: "017",
            },
            VillageCode {
                name: "尕庄村委会",
                code: "018",
            },
            VillageCode {
                name: "尕恰莫多村委会",
                code: "019",
            },
            VillageCode {
                name: "大石头村委会",
                code: "020",
            },
            VillageCode {
                name: "池汉素村委会",
                code: "021",
            },
            VillageCode {
                name: "本炕村委会",
                code: "022",
            },
            VillageCode {
                name: "牧场村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "和平乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "茶汉素村委会",
                code: "001",
            },
            VillageCode {
                name: "马场湾村委会",
                code: "002",
            },
            VillageCode {
                name: "董家脑村委会",
                code: "003",
            },
            VillageCode {
                name: "小高陵村委会",
                code: "004",
            },
            VillageCode {
                name: "大高陵村委会",
                code: "005",
            },
            VillageCode {
                name: "上拉雾台村委会",
                code: "006",
            },
            VillageCode {
                name: "下拉雾台村委会",
                code: "007",
            },
            VillageCode {
                name: "曲布滩村委会",
                code: "008",
            },
            VillageCode {
                name: "马家湾村委会",
                code: "009",
            },
            VillageCode {
                name: "马场台村委会",
                code: "010",
            },
            VillageCode {
                name: "隆和村委会",
                code: "011",
            },
            VillageCode {
                name: "加牙麻村委会",
                code: "012",
            },
            VillageCode {
                name: "和平村委会",
                code: "013",
            },
            VillageCode {
                name: "尕庄村委会",
                code: "014",
            },
            VillageCode {
                name: "茶曲村委会",
                code: "015",
            },
            VillageCode {
                name: "蒙古道村委会",
                code: "016",
            },
            VillageCode {
                name: "草沟村委会",
                code: "017",
            },
            VillageCode {
                name: "白水村委会",
                code: "018",
            },
            VillageCode {
                name: "泉尔湾村委会",
                code: "019",
            },
            VillageCode {
                name: "刘家台村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "波航乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "波航村委会",
                code: "001",
            },
            VillageCode {
                name: "纳隆村委会",
                code: "002",
            },
            VillageCode {
                name: "西岔村委会",
                code: "003",
            },
            VillageCode {
                name: "南岔村委会",
                code: "004",
            },
            VillageCode {
                name: "石崖湾村委会",
                code: "005",
            },
            VillageCode {
                name: "上台村委会",
                code: "006",
            },
            VillageCode {
                name: "下台村委会",
                code: "007",
            },
            VillageCode {
                name: "上泉尔村委会",
                code: "008",
            },
            VillageCode {
                name: "泉尔湾村委会",
                code: "009",
            },
            VillageCode {
                name: "麻尼台村委会",
                code: "010",
            },
            VillageCode {
                name: "浪湾村委会",
                code: "011",
            },
            VillageCode {
                name: "胡思洞村委会",
                code: "012",
            },
            VillageCode {
                name: "甘沟村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "申中乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "卡路村委会",
                code: "001",
            },
            VillageCode {
                name: "申中村委会",
                code: "002",
            },
            VillageCode {
                name: "前沟村委会",
                code: "003",
            },
            VillageCode {
                name: "莫布拉村委会",
                code: "004",
            },
            VillageCode {
                name: "庙沟脑村委会",
                code: "005",
            },
            VillageCode {
                name: "立达村委会",
                code: "006",
            },
            VillageCode {
                name: "口子村委会",
                code: "007",
            },
            VillageCode {
                name: "星泉村委会",
                code: "008",
            },
            VillageCode {
                name: "俊家庄村委会",
                code: "009",
            },
            VillageCode {
                name: "韭菜沟村委会",
                code: "010",
            },
            VillageCode {
                name: "窑庄村委会",
                code: "011",
            },
            VillageCode {
                name: "后沟村委会",
                code: "012",
            },
            VillageCode {
                name: "河拉村委会",
                code: "013",
            },
            VillageCode {
                name: "庙沟村委会",
                code: "014",
            },
            VillageCode {
                name: "大山根村委会",
                code: "015",
            },
            VillageCode {
                name: "大路村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "巴燕乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "巴燕村委会",
                code: "001",
            },
            VillageCode {
                name: "元山村委会",
                code: "002",
            },
            VillageCode {
                name: "新寺村委会",
                code: "003",
            },
            VillageCode {
                name: "下寺村委会",
                code: "004",
            },
            VillageCode {
                name: "下浪湾村委会",
                code: "005",
            },
            VillageCode {
                name: "下胡丹村委会",
                code: "006",
            },
            VillageCode {
                name: "西岭台村委会",
                code: "007",
            },
            VillageCode {
                name: "石门尔村委会",
                code: "008",
            },
            VillageCode {
                name: "上浪湾村委会",
                code: "009",
            },
            VillageCode {
                name: "上胡旦村委会",
                code: "010",
            },
            VillageCode {
                name: "莫合尔村委会",
                code: "011",
            },
            VillageCode {
                name: "居士浪村委会",
                code: "012",
            },
            VillageCode {
                name: "福海村委会",
                code: "013",
            },
            VillageCode {
                name: "巴燕峡村委会",
                code: "014",
            },
            VillageCode {
                name: "扎汉村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "寺寨乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "小寺尔村委会",
                code: "001",
            },
            VillageCode {
                name: "铧尖村委会",
                code: "002",
            },
            VillageCode {
                name: "下寨村委会",
                code: "003",
            },
            VillageCode {
                name: "西扎湾村委会",
                code: "004",
            },
            VillageCode {
                name: "草原村委会",
                code: "005",
            },
            VillageCode {
                name: "长岭村委会",
                code: "006",
            },
            VillageCode {
                name: "上寨村委会",
                code: "007",
            },
            VillageCode {
                name: "簸箕湾村委会",
                code: "008",
            },
            VillageCode {
                name: "乌图村委会",
                code: "009",
            },
            VillageCode {
                name: "烽火村委会",
                code: "010",
            },
            VillageCode {
                name: "马脊岭村委会",
                code: "011",
            },
            VillageCode {
                name: "阳坡湾村委会",
                code: "012",
            },
            VillageCode {
                name: "五岭村委会",
                code: "013",
            },
        ],
    },
];

static TOWNS_QH_009: [TownCode; 21] = [
    TownCode {
        name: "碾伯街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "朝阳山社区居委会",
                code: "001",
            },
            VillageCode {
                name: "古城东大街居委会",
                code: "002",
            },
            VillageCode {
                name: "古城西大街居委会",
                code: "003",
            },
            VillageCode {
                name: "新乐东大街居委会",
                code: "004",
            },
            VillageCode {
                name: "新乐西大街居委会",
                code: "005",
            },
            VillageCode {
                name: "怡春居委会",
                code: "006",
            },
            VillageCode {
                name: "水磨营居委会",
                code: "007",
            },
            VillageCode {
                name: "东门巷村委会",
                code: "008",
            },
            VillageCode {
                name: "东关村委会",
                code: "009",
            },
            VillageCode {
                name: "北一村委会",
                code: "010",
            },
            VillageCode {
                name: "北二村委会",
                code: "011",
            },
            VillageCode {
                name: "城中村委会",
                code: "012",
            },
            VillageCode {
                name: "河门村委会",
                code: "013",
            },
            VillageCode {
                name: "西门村委会",
                code: "014",
            },
            VillageCode {
                name: "下寨村委会",
                code: "015",
            },
            VillageCode {
                name: "上寨村委会",
                code: "016",
            },
            VillageCode {
                name: "东庄村委会",
                code: "017",
            },
            VillageCode {
                name: "后营村委会",
                code: "018",
            },
            VillageCode {
                name: "黄家村委会",
                code: "019",
            },
            VillageCode {
                name: "徐家沙沟村委会",
                code: "020",
            },
            VillageCode {
                name: "八家村委会",
                code: "021",
            },
            VillageCode {
                name: "王家村委会",
                code: "022",
            },
            VillageCode {
                name: "苏家村委会",
                code: "023",
            },
            VillageCode {
                name: "八里桥村委会",
                code: "024",
            },
            VillageCode {
                name: "沙坝村委会",
                code: "025",
            },
            VillageCode {
                name: "下李家村委会",
                code: "026",
            },
            VillageCode {
                name: "河湾村委会",
                code: "027",
            },
            VillageCode {
                name: "前庄村委会",
                code: "028",
            },
            VillageCode {
                name: "后庄村委会",
                code: "029",
            },
            VillageCode {
                name: "邓家庄村委会",
                code: "030",
            },
            VillageCode {
                name: "杨家门村委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "岗沟街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "七里店社区居委会",
                code: "001",
            },
            VillageCode {
                name: "文化街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "滨河路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "七里店村委会",
                code: "004",
            },
            VillageCode {
                name: "七里店东村村委会",
                code: "005",
            },
            VillageCode {
                name: "李家村委会",
                code: "006",
            },
            VillageCode {
                name: "熊沈家村委会",
                code: "007",
            },
            VillageCode {
                name: "上教场村委会",
                code: "008",
            },
            VillageCode {
                name: "下教场村委会",
                code: "009",
            },
            VillageCode {
                name: "晁家村委会",
                code: "010",
            },
            VillageCode {
                name: "崖湾村委会",
                code: "011",
            },
            VillageCode {
                name: "赵家村委会",
                code: "012",
            },
            VillageCode {
                name: "高家村委会",
                code: "013",
            },
            VillageCode {
                name: "西岗村委会",
                code: "014",
            },
            VillageCode {
                name: "九哈家村委会",
                code: "015",
            },
            VillageCode {
                name: "东岗村委会",
                code: "016",
            },
            VillageCode {
                name: "贾湾村委会",
                code: "017",
            },
            VillageCode {
                name: "土桥村委会",
                code: "018",
            },
            VillageCode {
                name: "汤官营村委会",
                code: "019",
            },
            VillageCode {
                name: "陶马家村委会",
                code: "020",
            },
            VillageCode {
                name: "水磨湾村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "雨润镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "雨润居委会",
                code: "001",
            },
            VillageCode {
                name: "汉庄村委会",
                code: "002",
            },
            VillageCode {
                name: "上杏园村委会",
                code: "003",
            },
            VillageCode {
                name: "下杏园村委会",
                code: "004",
            },
            VillageCode {
                name: "羊圈村委会",
                code: "005",
            },
            VillageCode {
                name: "大地湾村委会",
                code: "006",
            },
            VillageCode {
                name: "迭尔沟村委会",
                code: "007",
            },
            VillageCode {
                name: "红坡村委会",
                code: "008",
            },
            VillageCode {
                name: "刘家村委会",
                code: "009",
            },
            VillageCode {
                name: "深沟村委会",
                code: "010",
            },
            VillageCode {
                name: "荒滩村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "寿乐镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "寿乐居委会",
                code: "001",
            },
            VillageCode {
                name: "土官口村委会",
                code: "002",
            },
            VillageCode {
                name: "马家湾村委会",
                code: "003",
            },
            VillageCode {
                name: "王佛寺村委会",
                code: "004",
            },
            VillageCode {
                name: "陈家堡村委会",
                code: "005",
            },
            VillageCode {
                name: "仓岭沟村委会",
                code: "006",
            },
            VillageCode {
                name: "土官沟村委会",
                code: "007",
            },
            VillageCode {
                name: "牧场村委会",
                code: "008",
            },
            VillageCode {
                name: "仓岭顶村委会",
                code: "009",
            },
            VillageCode {
                name: "昂么村委会",
                code: "010",
            },
            VillageCode {
                name: "尕扎村委会",
                code: "011",
            },
            VillageCode {
                name: "联合村委会",
                code: "012",
            },
            VillageCode {
                name: "阳关沟村委会",
                code: "013",
            },
            VillageCode {
                name: "新堡子村委会",
                code: "014",
            },
            VillageCode {
                name: "熊家湾村委会",
                code: "015",
            },
            VillageCode {
                name: "王家庄村委会",
                code: "016",
            },
            VillageCode {
                name: "杨家山村委会",
                code: "017",
            },
            VillageCode {
                name: "杨家岗村委会",
                code: "018",
            },
            VillageCode {
                name: "上李家村委会",
                code: "019",
            },
            VillageCode {
                name: "窑庄村委会",
                code: "020",
            },
            VillageCode {
                name: "薛家庄村委会",
                code: "021",
            },
            VillageCode {
                name: "薛青村委会",
                code: "022",
            },
            VillageCode {
                name: "龙沟门村委会",
                code: "023",
            },
            VillageCode {
                name: "龙沟寺村委会",
                code: "024",
            },
            VillageCode {
                name: "祁家山村委会",
                code: "025",
            },
            VillageCode {
                name: "对巴子村委会",
                code: "026",
            },
            VillageCode {
                name: "李家台村委会",
                code: "027",
            },
            VillageCode {
                name: "赵家寺村委会",
                code: "028",
            },
            VillageCode {
                name: "赵家湾村委会",
                code: "029",
            },
            VillageCode {
                name: "上衙门村委会",
                code: "030",
            },
            VillageCode {
                name: "仓家峡村委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "高庙镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "高庙镇社区",
                code: "001",
            },
            VillageCode {
                name: "东村村委会",
                code: "002",
            },
            VillageCode {
                name: "西村村委会",
                code: "003",
            },
            VillageCode {
                name: "保家村委会",
                code: "004",
            },
            VillageCode {
                name: "李家村委会",
                code: "005",
            },
            VillageCode {
                name: "大路村委会",
                code: "006",
            },
            VillageCode {
                name: "新盛村委会",
                code: "007",
            },
            VillageCode {
                name: "段堡村委会",
                code: "008",
            },
            VillageCode {
                name: "新庄村委会",
                code: "009",
            },
            VillageCode {
                name: "扎门村委会",
                code: "010",
            },
            VillageCode {
                name: "老庄村委会",
                code: "011",
            },
            VillageCode {
                name: "柳湾村委会",
                code: "012",
            },
            VillageCode {
                name: "长里村委会",
                code: "013",
            },
            VillageCode {
                name: "下沟村委会",
                code: "014",
            },
            VillageCode {
                name: "旱地湾村委会",
                code: "015",
            },
            VillageCode {
                name: "田蒲家村委会",
                code: "016",
            },
            VillageCode {
                name: "寺磨庄村委会",
                code: "017",
            },
            VillageCode {
                name: "蒲家墩村委会",
                code: "018",
            },
            VillageCode {
                name: "白崖子村委会",
                code: "019",
            },
            VillageCode {
                name: "晁马家村委会",
                code: "020",
            },
            VillageCode {
                name: "老鸦村委会",
                code: "021",
            },
            VillageCode {
                name: "郎家村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "洪水镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "洪水镇社区",
                code: "001",
            },
            VillageCode {
                name: "店子村委会",
                code: "002",
            },
            VillageCode {
                name: "马趟村委会",
                code: "003",
            },
            VillageCode {
                name: "阿东村委会",
                code: "004",
            },
            VillageCode {
                name: "阿西村委会",
                code: "005",
            },
            VillageCode {
                name: "上窑洞村委会",
                code: "006",
            },
            VillageCode {
                name: "河西村委会",
                code: "007",
            },
            VillageCode {
                name: "下街村委会",
                code: "008",
            },
            VillageCode {
                name: "高家湾村委会",
                code: "009",
            },
            VillageCode {
                name: "双一村委会",
                code: "010",
            },
            VillageCode {
                name: "双二村委会",
                code: "011",
            },
            VillageCode {
                name: "姜湾村委会",
                code: "012",
            },
            VillageCode {
                name: "下沈家村委会",
                code: "013",
            },
            VillageCode {
                name: "马家营村委会",
                code: "014",
            },
            VillageCode {
                name: "李家壕村委会",
                code: "015",
            },
            VillageCode {
                name: "下王家村委会",
                code: "016",
            },
            VillageCode {
                name: "上王家村委会",
                code: "017",
            },
            VillageCode {
                name: "吴家庄村委会",
                code: "018",
            },
            VillageCode {
                name: "大寨子村委会",
                code: "019",
            },
            VillageCode {
                name: "袁家庄村委会",
                code: "020",
            },
            VillageCode {
                name: "洪水坪村委会",
                code: "021",
            },
            VillageCode {
                name: "石岭村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "高店镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "高店镇社区",
                code: "001",
            },
            VillageCode {
                name: "西门村委会",
                code: "002",
            },
            VillageCode {
                name: "河滩村委会",
                code: "003",
            },
            VillageCode {
                name: "东门村委会",
                code: "004",
            },
            VillageCode {
                name: "大峡村委会",
                code: "005",
            },
            VillageCode {
                name: "峡口村委会",
                code: "006",
            },
            VillageCode {
                name: "柳树湾村委会",
                code: "007",
            },
            VillageCode {
                name: "河滩寨村委会",
                code: "008",
            },
            VillageCode {
                name: "上杨家村委会",
                code: "009",
            },
            VillageCode {
                name: "下杨家村委会",
                code: "010",
            },
            VillageCode {
                name: "红庄村委会",
                code: "011",
            },
            VillageCode {
                name: "湾子村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "瞿昙镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "瞿昙镇社区",
                code: "001",
            },
            VillageCode {
                name: "新联村委会",
                code: "002",
            },
            VillageCode {
                name: "河西村委会",
                code: "003",
            },
            VillageCode {
                name: "徐家台村委会",
                code: "004",
            },
            VillageCode {
                name: "磨台村委会",
                code: "005",
            },
            VillageCode {
                name: "斜沟门村委会",
                code: "006",
            },
            VillageCode {
                name: "斜下村委会",
                code: "007",
            },
            VillageCode {
                name: "斜中村委会",
                code: "008",
            },
            VillageCode {
                name: "斜上村委会",
                code: "009",
            },
            VillageCode {
                name: "朵巴营村委会",
                code: "010",
            },
            VillageCode {
                name: "龙占村委会",
                code: "011",
            },
            VillageCode {
                name: "官隆湾村委会",
                code: "012",
            },
            VillageCode {
                name: "角营村委会",
                code: "013",
            },
            VillageCode {
                name: "浪上村委会",
                code: "014",
            },
            VillageCode {
                name: "浪下村委会",
                code: "015",
            },
            VillageCode {
                name: "石坡村委会",
                code: "016",
            },
            VillageCode {
                name: "台沿村委会",
                code: "017",
            },
            VillageCode {
                name: "中心村委会",
                code: "018",
            },
            VillageCode {
                name: "车路村委会",
                code: "019",
            },
            VillageCode {
                name: "阴坡村委会",
                code: "020",
            },
            VillageCode {
                name: "阳坡村委会",
                code: "021",
            },
            VillageCode {
                name: "中庄村委会",
                code: "022",
            },
            VillageCode {
                name: "隆国村委会",
                code: "023",
            },
            VillageCode {
                name: "韩家村委会",
                code: "024",
            },
            VillageCode {
                name: "祁家村委会",
                code: "025",
            },
            VillageCode {
                name: "口子村委会",
                code: "026",
            },
            VillageCode {
                name: "脑庄村委会",
                code: "027",
            },
            VillageCode {
                name: "大树村委会",
                code: "028",
            },
            VillageCode {
                name: "红庄村委会",
                code: "029",
            },
            VillageCode {
                name: "盛家村委会",
                code: "030",
            },
            VillageCode {
                name: "魏家村委会",
                code: "031",
            },
            VillageCode {
                name: "杨家村委会",
                code: "032",
            },
            VillageCode {
                name: "段家村委会",
                code: "033",
            },
            VillageCode {
                name: "周家村委会",
                code: "034",
            },
            VillageCode {
                name: "窑庄村委会",
                code: "035",
            },
            VillageCode {
                name: "晁家村委会",
                code: "036",
            },
        ],
    },
    TownCode {
        name: "共和乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "联星村委会",
                code: "001",
            },
            VillageCode {
                name: "许家寨村委会",
                code: "002",
            },
            VillageCode {
                name: "高营村委会",
                code: "003",
            },
            VillageCode {
                name: "书卜村委会",
                code: "004",
            },
            VillageCode {
                name: "大庄村委会",
                code: "005",
            },
            VillageCode {
                name: "拉科村委会",
                code: "006",
            },
            VillageCode {
                name: "拉日村委会",
                code: "007",
            },
            VillageCode {
                name: "磨石沟村委会",
                code: "008",
            },
            VillageCode {
                name: "虎林村委会",
                code: "009",
            },
            VillageCode {
                name: "桦林村委会",
                code: "010",
            },
            VillageCode {
                name: "祁家堡村委会",
                code: "011",
            },
            VillageCode {
                name: "洒龙村委会",
                code: "012",
            },
            VillageCode {
                name: "克什加村委会",
                code: "013",
            },
            VillageCode {
                name: "马厂村委会",
                code: "014",
            },
            VillageCode {
                name: "民族村委会",
                code: "015",
            },
            VillageCode {
                name: "童家村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "中岭乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "业善洼村委会",
                code: "001",
            },
            VillageCode {
                name: "马家洼村委会",
                code: "002",
            },
            VillageCode {
                name: "泉沟村委会",
                code: "003",
            },
            VillageCode {
                name: "上岭村委会",
                code: "004",
            },
            VillageCode {
                name: "平顶村委会",
                code: "005",
            },
            VillageCode {
                name: "梅家洼村委会",
                code: "006",
            },
            VillageCode {
                name: "中岭村委会",
                code: "007",
            },
            VillageCode {
                name: "甘沟脑村委会",
                code: "008",
            },
            VillageCode {
                name: "草场村委会",
                code: "009",
            },
            VillageCode {
                name: "吴家洼村委会",
                code: "010",
            },
            VillageCode {
                name: "平坦村委会",
                code: "011",
            },
            VillageCode {
                name: "铲铲洼村委会",
                code: "012",
            },
            VillageCode {
                name: "大水泉村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "李家乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "烂泥沟村委会",
                code: "001",
            },
            VillageCode {
                name: "甘沟岭村委会",
                code: "002",
            },
            VillageCode {
                name: "大洼村委会",
                code: "003",
            },
            VillageCode {
                name: "马圈村委会",
                code: "004",
            },
            VillageCode {
                name: "民族村委会",
                code: "005",
            },
            VillageCode {
                name: "合尔红村委会",
                code: "006",
            },
            VillageCode {
                name: "公擦沟村委会",
                code: "007",
            },
            VillageCode {
                name: "阿塔岭村委会",
                code: "008",
            },
            VillageCode {
                name: "西马营村委会",
                code: "009",
            },
            VillageCode {
                name: "东马营村委会",
                code: "010",
            },
            VillageCode {
                name: "丹科尔村委会",
                code: "011",
            },
            VillageCode {
                name: "陈家磨村委会",
                code: "012",
            },
            VillageCode {
                name: "尕泉湾村委会",
                code: "013",
            },
            VillageCode {
                name: "和尔茨村委会",
                code: "014",
            },
            VillageCode {
                name: "交界湾村委会",
                code: "015",
            },
            VillageCode {
                name: "山庄村委会",
                code: "016",
            },
            VillageCode {
                name: "双坪村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "下营乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "下营村委会",
                code: "001",
            },
            VillageCode {
                name: "白土庄村委会",
                code: "002",
            },
            VillageCode {
                name: "坑坑村委会",
                code: "003",
            },
            VillageCode {
                name: "上营村委会",
                code: "004",
            },
            VillageCode {
                name: "塔春村委会",
                code: "005",
            },
            VillageCode {
                name: "茶龙村委会",
                code: "006",
            },
            VillageCode {
                name: "祝家村委会",
                code: "007",
            },
            VillageCode {
                name: "堡子村委会",
                code: "008",
            },
            VillageCode {
                name: "卡金门村委会",
                code: "009",
            },
            VillageCode {
                name: "大庄村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "芦花乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "寺院村委会",
                code: "001",
            },
            VillageCode {
                name: "西坡村委会",
                code: "002",
            },
            VillageCode {
                name: "丰洼村委会",
                code: "003",
            },
            VillageCode {
                name: "查干村委会",
                code: "004",
            },
            VillageCode {
                name: "三条沟村委会",
                code: "005",
            },
            VillageCode {
                name: "营盘湾村委会",
                code: "006",
            },
            VillageCode {
                name: "朵家湾村委会",
                code: "007",
            },
            VillageCode {
                name: "东岭村委会",
                code: "008",
            },
            VillageCode {
                name: "十字村委会",
                code: "009",
            },
            VillageCode {
                name: "牙合村委会",
                code: "010",
            },
            VillageCode {
                name: "九架山村委会",
                code: "011",
            },
            VillageCode {
                name: "转花湾村委会",
                code: "012",
            },
            VillageCode {
                name: "城背后村委会",
                code: "013",
            },
            VillageCode {
                name: "王家湾村委会",
                code: "014",
            },
            VillageCode {
                name: "本康岭村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "马营乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "马莲沟村委会",
                code: "001",
            },
            VillageCode {
                name: "湾塘村委会",
                code: "002",
            },
            VillageCode {
                name: "龙王岗村委会",
                code: "003",
            },
            VillageCode {
                name: "昆仑村委会",
                code: "004",
            },
            VillageCode {
                name: "上浪卡村委会",
                code: "005",
            },
            VillageCode {
                name: "姜洞村委会",
                code: "006",
            },
            VillageCode {
                name: "龙床村委会",
                code: "007",
            },
            VillageCode {
                name: "牙合村委会",
                code: "008",
            },
            VillageCode {
                name: "连丰村委会",
                code: "009",
            },
            VillageCode {
                name: "卡拉村委会",
                code: "010",
            },
            VillageCode {
                name: "白崖坪村委会",
                code: "011",
            },
            VillageCode {
                name: "墩湾村委会",
                code: "012",
            },
            VillageCode {
                name: "胜利村委会",
                code: "013",
            },
            VillageCode {
                name: "脑庄村委会",
                code: "014",
            },
            VillageCode {
                name: "八架山村委会",
                code: "015",
            },
            VillageCode {
                name: "康巴村委会",
                code: "016",
            },
            VillageCode {
                name: "古城村委会",
                code: "017",
            },
            VillageCode {
                name: "北坪村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "马厂乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "马厂村村委会",
                code: "001",
            },
            VillageCode {
                name: "那家庄村委会",
                code: "002",
            },
            VillageCode {
                name: "孟家湾村委会",
                code: "003",
            },
            VillageCode {
                name: "白石头村委会",
                code: "004",
            },
            VillageCode {
                name: "保家湾村委会",
                code: "005",
            },
            VillageCode {
                name: "小岭子村委会",
                code: "006",
            },
            VillageCode {
                name: "八旦村委会",
                code: "007",
            },
            VillageCode {
                name: "泉儿湾村委会",
                code: "008",
            },
            VillageCode {
                name: "岔沟村委会",
                code: "009",
            },
            VillageCode {
                name: "甘沟滩村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "蒲台乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "千户台村委会",
                code: "001",
            },
            VillageCode {
                name: "赵家庄村委会",
                code: "002",
            },
            VillageCode {
                name: "辛家庄村委会",
                code: "003",
            },
            VillageCode {
                name: "李家台村委会",
                code: "004",
            },
            VillageCode {
                name: "范家坪村委会",
                code: "005",
            },
            VillageCode {
                name: "地洼村委会",
                code: "006",
            },
            VillageCode {
                name: "候白家村委会",
                code: "007",
            },
            VillageCode {
                name: "雷盛家村委会",
                code: "008",
            },
            VillageCode {
                name: "圈窝村委会",
                code: "009",
            },
            VillageCode {
                name: "羊起台村委会",
                code: "010",
            },
            VillageCode {
                name: "赵宝湾村委会",
                code: "011",
            },
            VillageCode {
                name: "寺沟脑村委会",
                code: "012",
            },
            VillageCode {
                name: "小甘沟村委会",
                code: "013",
            },
            VillageCode {
                name: "赵家坪村委会",
                code: "014",
            },
            VillageCode {
                name: "东台村委会",
                code: "015",
            },
            VillageCode {
                name: "头庄村委会",
                code: "016",
            },
            VillageCode {
                name: "化庄村委会",
                code: "017",
            },
            VillageCode {
                name: "山桃村委会",
                code: "018",
            },
            VillageCode {
                name: "黑窑洞村委会",
                code: "019",
            },
            VillageCode {
                name: "尹家村委会",
                code: "020",
            },
            VillageCode {
                name: "严家山村委会",
                code: "021",
            },
            VillageCode {
                name: "西沟村委会",
                code: "022",
            },
            VillageCode {
                name: "郭家村委会",
                code: "023",
            },
            VillageCode {
                name: "下半沟村委会",
                code: "024",
            },
            VillageCode {
                name: "新庄湾村委会",
                code: "025",
            },
            VillageCode {
                name: "大麦沟村委会",
                code: "026",
            },
            VillageCode {
                name: "中岭村委会",
                code: "027",
            },
            VillageCode {
                name: "上岭村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "中坝乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "麻尼台村委会",
                code: "001",
            },
            VillageCode {
                name: "牙昂村委会",
                code: "002",
            },
            VillageCode {
                name: "中坝庄村委会",
                code: "003",
            },
            VillageCode {
                name: "红庄沟村委会",
                code: "004",
            },
            VillageCode {
                name: "交头村委会",
                code: "005",
            },
            VillageCode {
                name: "洒口村委会",
                code: "006",
            },
            VillageCode {
                name: "大湾村委会",
                code: "007",
            },
            VillageCode {
                name: "泉脑村委会",
                code: "008",
            },
            VillageCode {
                name: "山丹坡村委会",
                code: "009",
            },
            VillageCode {
                name: "四庄村委会",
                code: "010",
            },
            VillageCode {
                name: "洪三村委会",
                code: "011",
            },
            VillageCode {
                name: "柏杨沟村委会",
                code: "012",
            },
            VillageCode {
                name: "何家山村委会",
                code: "013",
            },
            VillageCode {
                name: "确石湾村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "峰堆乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "营盘村委会",
                code: "001",
            },
            VillageCode {
                name: "上一村委会",
                code: "002",
            },
            VillageCode {
                name: "上二村委会",
                code: "003",
            },
            VillageCode {
                name: "刘家寺村委会",
                code: "004",
            },
            VillageCode {
                name: "下帐房村委会",
                code: "005",
            },
            VillageCode {
                name: "联村村委会",
                code: "006",
            },
            VillageCode {
                name: "李庄村委会",
                code: "007",
            },
            VillageCode {
                name: "红沟门村委会",
                code: "008",
            },
            VillageCode {
                name: "熊家村委会",
                code: "009",
            },
            VillageCode {
                name: "上阳洼村委会",
                code: "010",
            },
            VillageCode {
                name: "下阳洼村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "城台乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "拉甘驿村委会",
                code: "001",
            },
            VillageCode {
                name: "台子村委会",
                code: "002",
            },
            VillageCode {
                name: "城子村委会",
                code: "003",
            },
            VillageCode {
                name: "山城村委会",
                code: "004",
            },
            VillageCode {
                name: "坝口村委会",
                code: "005",
            },
            VillageCode {
                name: "河东村委会",
                code: "006",
            },
            VillageCode {
                name: "小沟村委会",
                code: "007",
            },
            VillageCode {
                name: "新庄村委会",
                code: "008",
            },
            VillageCode {
                name: "衙门庄村委会",
                code: "009",
            },
            VillageCode {
                name: "泉湾村委会",
                code: "010",
            },
            VillageCode {
                name: "下台村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "达拉乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "袁家台村委会",
                code: "001",
            },
            VillageCode {
                name: "宁过村委会",
                code: "002",
            },
            VillageCode {
                name: "马趟村委会",
                code: "003",
            },
            VillageCode {
                name: "马圈沟村委会",
                code: "004",
            },
            VillageCode {
                name: "白草台村委会",
                code: "005",
            },
            VillageCode {
                name: "泉洼村委会",
                code: "006",
            },
            VillageCode {
                name: "甘沟山村委会",
                code: "007",
            },
            VillageCode {
                name: "春洒村委会",
                code: "008",
            },
            VillageCode {
                name: "烂泥滩村委会",
                code: "009",
            },
            VillageCode {
                name: "王家滩村委会",
                code: "010",
            },
            VillageCode {
                name: "李家昂村委会",
                code: "011",
            },
            VillageCode {
                name: "达拉滩村委会",
                code: "012",
            },
            VillageCode {
                name: "麻洞村委会",
                code: "013",
            },
            VillageCode {
                name: "杜家洼村委会",
                code: "014",
            },
            VillageCode {
                name: "红沟村委会",
                code: "015",
            },
            VillageCode {
                name: "拉卡村委会",
                code: "016",
            },
            VillageCode {
                name: "长沟村委会",
                code: "017",
            },
            VillageCode {
                name: "扎什加村委会",
                code: "018",
            },
            VillageCode {
                name: "大庄村委会",
                code: "019",
            },
            VillageCode {
                name: "白崖子村委会",
                code: "020",
            },
            VillageCode {
                name: "前半沟村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "海东工业园区乐都工业园",
        code: "021",
        villages: &[
            VillageCode {
                name: "类似居委会西区",
                code: "001",
            },
            VillageCode {
                name: "类似居委会北区",
                code: "002",
            },
        ],
    },
];

static TOWNS_QH_010: [TownCode; 9] = [
    TownCode {
        name: "平安街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "乐都路居委会",
                code: "001",
            },
            VillageCode {
                name: "平安路居委会",
                code: "002",
            },
            VillageCode {
                name: "湟中路居委会",
                code: "003",
            },
            VillageCode {
                name: "化隆路居委会",
                code: "004",
            },
            VillageCode {
                name: "棉纺厂居委会",
                code: "005",
            },
            VillageCode {
                name: "东庄村委会",
                code: "006",
            },
            VillageCode {
                name: "上庄村委会",
                code: "007",
            },
            VillageCode {
                name: "大路村委会",
                code: "008",
            },
            VillageCode {
                name: "东村村委会",
                code: "009",
            },
            VillageCode {
                name: "上滩村委会",
                code: "010",
            },
            VillageCode {
                name: "张家寨村委会",
                code: "011",
            },
            VillageCode {
                name: "中村村委会",
                code: "012",
            },
            VillageCode {
                name: "南村村委会",
                code: "013",
            },
            VillageCode {
                name: "西村村委会",
                code: "014",
            },
            VillageCode {
                name: "西营村委会",
                code: "015",
            },
            VillageCode {
                name: "杨家村委会",
                code: "016",
            },
            VillageCode {
                name: "窑房村委会",
                code: "017",
            },
            VillageCode {
                name: "红岭村委会",
                code: "018",
            },
            VillageCode {
                name: "沈家村委会",
                code: "019",
            },
            VillageCode {
                name: "白家村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "小峡街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "小峡镇居委会",
                code: "001",
            },
            VillageCode {
                name: "下店村委会",
                code: "002",
            },
            VillageCode {
                name: "西上庄村委会",
                code: "003",
            },
            VillageCode {
                name: "红土庄村委会",
                code: "004",
            },
            VillageCode {
                name: "王家庄村委会",
                code: "005",
            },
            VillageCode {
                name: "卅里铺村委会",
                code: "006",
            },
            VillageCode {
                name: "上店村委会",
                code: "007",
            },
            VillageCode {
                name: "百草湾村委会",
                code: "008",
            },
            VillageCode {
                name: "上红庄村委会",
                code: "009",
            },
            VillageCode {
                name: "下红庄村委会",
                code: "010",
            },
            VillageCode {
                name: "柳湾村委会",
                code: "011",
            },
            VillageCode {
                name: "石家营村委会",
                code: "012",
            },
            VillageCode {
                name: "古城崖村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "三合镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "三合镇居委会",
                code: "001",
            },
            VillageCode {
                name: "三合村委会",
                code: "002",
            },
            VillageCode {
                name: "骆驼堡村委会",
                code: "003",
            },
            VillageCode {
                name: "东崖头村委会",
                code: "004",
            },
            VillageCode {
                name: "西崖头村委会",
                code: "005",
            },
            VillageCode {
                name: "冰岭山村委会",
                code: "006",
            },
            VillageCode {
                name: "祁新庄村委会",
                code: "007",
            },
            VillageCode {
                name: "张其寨村委会",
                code: "008",
            },
            VillageCode {
                name: "条岭村委会",
                code: "009",
            },
            VillageCode {
                name: "索尔干村委会",
                code: "010",
            },
            VillageCode {
                name: "寺台村委会",
                code: "011",
            },
            VillageCode {
                name: "新安村委会",
                code: "012",
            },
            VillageCode {
                name: "湾子村委会",
                code: "013",
            },
            VillageCode {
                name: "仲家村委会",
                code: "014",
            },
            VillageCode {
                name: "翻身村委会",
                code: "015",
            },
            VillageCode {
                name: "庄廓村委会",
                code: "016",
            },
            VillageCode {
                name: "窑洞村委会",
                code: "017",
            },
            VillageCode {
                name: "邦业隆村委会",
                code: "018",
            },
            VillageCode {
                name: "瓦窑台村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "洪水泉乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "井尔沟村委会",
                code: "001",
            },
            VillageCode {
                name: "槽子村委会",
                code: "002",
            },
            VillageCode {
                name: "马圈村委会",
                code: "003",
            },
            VillageCode {
                name: "糜子湾村委会",
                code: "004",
            },
            VillageCode {
                name: "黄鼠湾村委会",
                code: "005",
            },
            VillageCode {
                name: "永安村委会",
                code: "006",
            },
            VillageCode {
                name: "洪水泉村委会",
                code: "007",
            },
            VillageCode {
                name: "沟滩村委会",
                code: "008",
            },
            VillageCode {
                name: "硝水泉村委会",
                code: "009",
            },
            VillageCode {
                name: "北岭村委会",
                code: "010",
            },
            VillageCode {
                name: "韭菜沟村委会",
                code: "011",
            },
            VillageCode {
                name: "阿吉营村委会",
                code: "012",
            },
            VillageCode {
                name: "沙义岭村委会",
                code: "013",
            },
            VillageCode {
                name: "拉树岭村委会",
                code: "014",
            },
            VillageCode {
                name: "永固村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "石灰窑乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "石灰窑村委会",
                code: "001",
            },
            VillageCode {
                name: "宜麻村委会",
                code: "002",
            },
            VillageCode {
                name: "业隆村委会",
                code: "003",
            },
            VillageCode {
                name: "黎明村委会",
                code: "004",
            },
            VillageCode {
                name: "阳坡山村委会",
                code: "005",
            },
            VillageCode {
                name: "窑庄村委会",
                code: "006",
            },
            VillageCode {
                name: "处处沟村委会",
                code: "007",
            },
            VillageCode {
                name: "红崖村委会",
                code: "008",
            },
            VillageCode {
                name: "下河滩村委会",
                code: "009",
            },
            VillageCode {
                name: "唐隆台村委会",
                code: "010",
            },
            VillageCode {
                name: "上唐隆村委会",
                code: "011",
            },
            VillageCode {
                name: "上法台村委会",
                code: "012",
            },
            VillageCode {
                name: "下法台村委会",
                code: "013",
            },
            VillageCode {
                name: "石卦寺村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "古城乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "古城村委会",
                code: "001",
            },
            VillageCode {
                name: "总门村委会",
                code: "002",
            },
            VillageCode {
                name: "北村村委会",
                code: "003",
            },
            VillageCode {
                name: "沙卡村委会",
                code: "004",
            },
            VillageCode {
                name: "木场村委会",
                code: "005",
            },
            VillageCode {
                name: "且尔甫村委会",
                code: "006",
            },
            VillageCode {
                name: "山城村委会",
                code: "007",
            },
            VillageCode {
                name: "牌楼沟村委会",
                code: "008",
            },
            VillageCode {
                name: "新庄尔村委会",
                code: "009",
            },
            VillageCode {
                name: "角加村委会",
                code: "010",
            },
            VillageCode {
                name: "扎门村委会",
                code: "011",
            },
            VillageCode {
                name: "石壁村委会",
                code: "012",
            },
            VillageCode {
                name: "六台村委会",
                code: "013",
            },
            VillageCode {
                name: "黑林滩村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "沙沟乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "沙沟村委会",
                code: "001",
            },
            VillageCode {
                name: "大寨子村委会",
                code: "002",
            },
            VillageCode {
                name: "树尔湾村委会",
                code: "003",
            },
            VillageCode {
                name: "石沟沿村委会",
                code: "004",
            },
            VillageCode {
                name: "芦草沟村委会",
                code: "005",
            },
            VillageCode {
                name: "侯家庄村委会",
                code: "006",
            },
            VillageCode {
                name: "中庄村委会",
                code: "007",
            },
            VillageCode {
                name: "桑昂村委会",
                code: "008",
            },
            VillageCode {
                name: "牙扎村委会",
                code: "009",
            },
            VillageCode {
                name: "四方顶村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "巴藏沟乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "索家村委会",
                code: "001",
            },
            VillageCode {
                name: "上星家村委会",
                code: "002",
            },
            VillageCode {
                name: "巴家村委会",
                code: "003",
            },
            VillageCode {
                name: "下星家村委会",
                code: "004",
            },
            VillageCode {
                name: "下马家村委会",
                code: "005",
            },
            VillageCode {
                name: "尔官村委会",
                code: "006",
            },
            VillageCode {
                name: "李家村委会",
                code: "007",
            },
            VillageCode {
                name: "河东村委会",
                code: "008",
            },
            VillageCode {
                name: "清泉村委会",
                code: "009",
            },
            VillageCode {
                name: "上马家村委会",
                code: "010",
            },
            VillageCode {
                name: "下郭尔村委会",
                code: "011",
            },
            VillageCode {
                name: "堂寺尔村委会",
                code: "012",
            },
            VillageCode {
                name: "上郭尔村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "曹家堡临空综合经济园平安园区",
        code: "009",
        villages: &[VillageCode {
            name: "曹家堡临空综合经济园平安园区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_QH_011: [TownCode; 22] = [
    TownCode {
        name: "川口镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "西大街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南大街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "史纳社区居委会",
                code: "003",
            },
            VillageCode {
                name: "北大街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "东大街社区居委会",
                code: "005",
            },
            VillageCode {
                name: "民镁社区居委会",
                code: "006",
            },
            VillageCode {
                name: "川垣社区居委会",
                code: "007",
            },
            VillageCode {
                name: "海鸿社区居民委员会",
                code: "008",
            },
            VillageCode {
                name: "金塔社区居民委员会",
                code: "009",
            },
            VillageCode {
                name: "馨怡社区居民委员会",
                code: "010",
            },
            VillageCode {
                name: "红卫村委会",
                code: "011",
            },
            VillageCode {
                name: "享堂村委会",
                code: "012",
            },
            VillageCode {
                name: "史纳村委会",
                code: "013",
            },
            VillageCode {
                name: "米拉湾村委会",
                code: "014",
            },
            VillageCode {
                name: "山城村委会",
                code: "015",
            },
            VillageCode {
                name: "果园村委会",
                code: "016",
            },
            VillageCode {
                name: "吉家堡村委会",
                code: "017",
            },
            VillageCode {
                name: "南庄子村委会",
                code: "018",
            },
            VillageCode {
                name: "川口村委会",
                code: "019",
            },
            VillageCode {
                name: "南山村委会",
                code: "020",
            },
            VillageCode {
                name: "东垣村委会",
                code: "021",
            },
            VillageCode {
                name: "驮岭村委会",
                code: "022",
            },
            VillageCode {
                name: "边墙村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "古鄯镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "古鄯镇社区",
                code: "001",
            },
            VillageCode {
                name: "古鄯村委会",
                code: "002",
            },
            VillageCode {
                name: "三姓庄村委会",
                code: "003",
            },
            VillageCode {
                name: "范家河村委会",
                code: "004",
            },
            VillageCode {
                name: "桦林滩村委会",
                code: "005",
            },
            VillageCode {
                name: "郭家山村委会",
                code: "006",
            },
            VillageCode {
                name: "山庄村委会",
                code: "007",
            },
            VillageCode {
                name: "尖岭村委会",
                code: "008",
            },
            VillageCode {
                name: "三岔村委会",
                code: "009",
            },
            VillageCode {
                name: "联合村委会",
                code: "010",
            },
            VillageCode {
                name: "菜子湾村委会",
                code: "011",
            },
            VillageCode {
                name: "夏家河村委会",
                code: "012",
            },
            VillageCode {
                name: "小岭村委会",
                code: "013",
            },
            VillageCode {
                name: "七里村委会",
                code: "014",
            },
            VillageCode {
                name: "徐家庄村委会",
                code: "015",
            },
            VillageCode {
                name: "岘子村委会",
                code: "016",
            },
            VillageCode {
                name: "马营庄村委会",
                code: "017",
            },
            VillageCode {
                name: "牙合村委会",
                code: "018",
            },
            VillageCode {
                name: "刘家湾村委会",
                code: "019",
            },
            VillageCode {
                name: "来家山村委会",
                code: "020",
            },
            VillageCode {
                name: "邓家山村委会",
                code: "021",
            },
            VillageCode {
                name: "桦林嘴村委会",
                code: "022",
            },
            VillageCode {
                name: "后山村委会",
                code: "023",
            },
            VillageCode {
                name: "李家山村委会",
                code: "024",
            },
            VillageCode {
                name: "柴沟村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "马营镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "马营镇社区",
                code: "001",
            },
            VillageCode {
                name: "马营村委会",
                code: "002",
            },
            VillageCode {
                name: "马家村委会",
                code: "003",
            },
            VillageCode {
                name: "鲍家山村委会",
                code: "004",
            },
            VillageCode {
                name: "沙塄沟村委会",
                code: "005",
            },
            VillageCode {
                name: "三联村委会",
                code: "006",
            },
            VillageCode {
                name: "安家村委会",
                code: "007",
            },
            VillageCode {
                name: "菜园村委会",
                code: "008",
            },
            VillageCode {
                name: "罗家沟村委会",
                code: "009",
            },
            VillageCode {
                name: "双泉堡村委会",
                code: "010",
            },
            VillageCode {
                name: "朱家山村委会",
                code: "011",
            },
            VillageCode {
                name: "阳山村委会",
                code: "012",
            },
            VillageCode {
                name: "王家村委会",
                code: "013",
            },
            VillageCode {
                name: "洒大庄村委会",
                code: "014",
            },
            VillageCode {
                name: "大滩村委会",
                code: "015",
            },
            VillageCode {
                name: "和平村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "官亭镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "官厅镇社区",
                code: "001",
            },
            VillageCode {
                name: "官中村委会",
                code: "002",
            },
            VillageCode {
                name: "光辉村委会",
                code: "003",
            },
            VillageCode {
                name: "梧石村委会",
                code: "004",
            },
            VillageCode {
                name: "先锋村委会",
                code: "005",
            },
            VillageCode {
                name: "前进村委会",
                code: "006",
            },
            VillageCode {
                name: "河沿村委会",
                code: "007",
            },
            VillageCode {
                name: "赵木川村委会",
                code: "008",
            },
            VillageCode {
                name: "寨子村委会",
                code: "009",
            },
            VillageCode {
                name: "鲍家村委会",
                code: "010",
            },
            VillageCode {
                name: "喇家村委会",
                code: "011",
            },
            VillageCode {
                name: "官西村委会",
                code: "012",
            },
            VillageCode {
                name: "官东村委会",
                code: "013",
            },
            VillageCode {
                name: "别落村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "巴州镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "巴州镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "巴州一村委会",
                code: "002",
            },
            VillageCode {
                name: "洒力池村委会",
                code: "003",
            },
            VillageCode {
                name: "下马家村委会",
                code: "004",
            },
            VillageCode {
                name: "上马家村委会",
                code: "005",
            },
            VillageCode {
                name: "祁家村委会",
                code: "006",
            },
            VillageCode {
                name: "万泉堡村委会",
                code: "007",
            },
            VillageCode {
                name: "巴州二村委会",
                code: "008",
            },
            VillageCode {
                name: "羊羔滩村委会",
                code: "009",
            },
            VillageCode {
                name: "巴州垣村委会",
                code: "010",
            },
            VillageCode {
                name: "凉尔湾村委会",
                code: "011",
            },
            VillageCode {
                name: "麻家湾村委会",
                code: "012",
            },
            VillageCode {
                name: "老官坪村委会",
                code: "013",
            },
            VillageCode {
                name: "黄池村委会",
                code: "014",
            },
            VillageCode {
                name: "胡家村委会",
                code: "015",
            },
            VillageCode {
                name: "大焦土村委会",
                code: "016",
            },
            VillageCode {
                name: "杨家湾村委会",
                code: "017",
            },
            VillageCode {
                name: "下宣村委会",
                code: "018",
            },
            VillageCode {
                name: "阳山村委会",
                code: "019",
            },
            VillageCode {
                name: "上宣村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "满坪镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "满坪镇社区",
                code: "001",
            },
            VillageCode {
                name: "满坪村委会",
                code: "002",
            },
            VillageCode {
                name: "集场村委会",
                code: "003",
            },
            VillageCode {
                name: "清泉村委会",
                code: "004",
            },
            VillageCode {
                name: "大庄村委会",
                code: "005",
            },
            VillageCode {
                name: "山庄村委会",
                code: "006",
            },
            VillageCode {
                name: "河口村委会",
                code: "007",
            },
            VillageCode {
                name: "陈家村委会",
                code: "008",
            },
            VillageCode {
                name: "朵儿卜村委会",
                code: "009",
            },
            VillageCode {
                name: "阳龙坪村委会",
                code: "010",
            },
            VillageCode {
                name: "沙拉坡村委会",
                code: "011",
            },
            VillageCode {
                name: "东湾村委会",
                code: "012",
            },
            VillageCode {
                name: "大滩村委会",
                code: "013",
            },
            VillageCode {
                name: "浪塘村委会",
                code: "014",
            },
            VillageCode {
                name: "新建村委会",
                code: "015",
            },
            VillageCode {
                name: "傲沟村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "李二堡镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "李二堡镇社区",
                code: "001",
            },
            VillageCode {
                name: "李家村委会",
                code: "002",
            },
            VillageCode {
                name: "焦土村委会",
                code: "003",
            },
            VillageCode {
                name: "前庞村委会",
                code: "004",
            },
            VillageCode {
                name: "祁家村委会",
                code: "005",
            },
            VillageCode {
                name: "康各岱村委会",
                code: "006",
            },
            VillageCode {
                name: "范家村委会",
                code: "007",
            },
            VillageCode {
                name: "乐园村委会",
                code: "008",
            },
            VillageCode {
                name: "马莲滩村委会",
                code: "009",
            },
            VillageCode {
                name: "牙尔山村委会",
                code: "010",
            },
            VillageCode {
                name: "张家湾村委会",
                code: "011",
            },
            VillageCode {
                name: "塘尔垣村委会",
                code: "012",
            },
            VillageCode {
                name: "寺尔庄村委会",
                code: "013",
            },
            VillageCode {
                name: "河西沟村委会",
                code: "014",
            },
            VillageCode {
                name: "上藏村委会",
                code: "015",
            },
            VillageCode {
                name: "下藏村委会",
                code: "016",
            },
            VillageCode {
                name: "窑洞村委会",
                code: "017",
            },
            VillageCode {
                name: "松山村委会",
                code: "018",
            },
            VillageCode {
                name: "邦岭村委会",
                code: "019",
            },
            VillageCode {
                name: "开阳村委会",
                code: "020",
            },
            VillageCode {
                name: "上岭村委会",
                code: "021",
            },
            VillageCode {
                name: "公家庄村委会",
                code: "022",
            },
            VillageCode {
                name: "石庄村委会",
                code: "023",
            },
            VillageCode {
                name: "山庄村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "峡门镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "硖门镇社区",
                code: "001",
            },
            VillageCode {
                name: "孙家庄村委会",
                code: "002",
            },
            VillageCode {
                name: "直沟村委会",
                code: "003",
            },
            VillageCode {
                name: "深巴村委会",
                code: "004",
            },
            VillageCode {
                name: "甘池村委会",
                code: "005",
            },
            VillageCode {
                name: "硖门村委会",
                code: "006",
            },
            VillageCode {
                name: "阳坡村委会",
                code: "007",
            },
            VillageCode {
                name: "腰路村委会",
                code: "008",
            },
            VillageCode {
                name: "康阳村委会",
                code: "009",
            },
            VillageCode {
                name: "巴子沟村委会",
                code: "010",
            },
            VillageCode {
                name: "李家庄村委会",
                code: "011",
            },
            VillageCode {
                name: "石家庄村委会",
                code: "012",
            },
            VillageCode {
                name: "甲子山村委会",
                code: "013",
            },
            VillageCode {
                name: "赵家山村委会",
                code: "014",
            },
            VillageCode {
                name: "抓咱村委会",
                code: "015",
            },
            VillageCode {
                name: "若木池村委会",
                code: "016",
            },
            VillageCode {
                name: "铁家庄村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "马场垣乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "香水村委会",
                code: "001",
            },
            VillageCode {
                name: "团结村委会",
                code: "002",
            },
            VillageCode {
                name: "金星村委会",
                code: "003",
            },
            VillageCode {
                name: "翠泉村委会",
                code: "004",
            },
            VillageCode {
                name: "下川口村委会",
                code: "005",
            },
            VillageCode {
                name: "马聚垣村委会",
                code: "006",
            },
            VillageCode {
                name: "磨湾子村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "北山乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "先锋村委会",
                code: "001",
            },
            VillageCode {
                name: "庄子沟村委会",
                code: "002",
            },
            VillageCode {
                name: "罗家湾村委会",
                code: "003",
            },
            VillageCode {
                name: "德兴村委会",
                code: "004",
            },
            VillageCode {
                name: "牙合村委会",
                code: "005",
            },
            VillageCode {
                name: "永进村委会",
                code: "006",
            },
            VillageCode {
                name: "宽都兰村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "松树乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "崖湾村委会",
                code: "001",
            },
            VillageCode {
                name: "路家堡村委会",
                code: "002",
            },
            VillageCode {
                name: "加仁村委会",
                code: "003",
            },
            VillageCode {
                name: "百姓村委会",
                code: "004",
            },
            VillageCode {
                name: "松树村委会",
                code: "005",
            },
            VillageCode {
                name: "店子村委会",
                code: "006",
            },
            VillageCode {
                name: "湖拉海村委会",
                code: "007",
            },
            VillageCode {
                name: "牙合村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "西沟乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "官地村委会",
                code: "001",
            },
            VillageCode {
                name: "南垣村委会",
                code: "002",
            },
            VillageCode {
                name: "马家河村委会",
                code: "003",
            },
            VillageCode {
                name: "三方村委会",
                code: "004",
            },
            VillageCode {
                name: "要先村委会",
                code: "005",
            },
            VillageCode {
                name: "凉坪村委会",
                code: "006",
            },
            VillageCode {
                name: "大滩村委会",
                code: "007",
            },
            VillageCode {
                name: "西巷村委会",
                code: "008",
            },
            VillageCode {
                name: "樊家滩村委会",
                code: "009",
            },
            VillageCode {
                name: "复兴村委会",
                code: "010",
            },
            VillageCode {
                name: "才丰沟村委会",
                code: "011",
            },
            VillageCode {
                name: "瓦窑村委会",
                code: "012",
            },
            VillageCode {
                name: "塔先村委会",
                code: "013",
            },
            VillageCode {
                name: "山庄村委会",
                code: "014",
            },
            VillageCode {
                name: "红崖子村委会",
                code: "015",
            },
            VillageCode {
                name: "南方庄村委会",
                code: "016",
            },
            VillageCode {
                name: "麻地沟村委会",
                code: "017",
            },
            VillageCode {
                name: "张家庄村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "总堡乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "三垣村委会",
                code: "001",
            },
            VillageCode {
                name: "三家村委会",
                code: "002",
            },
            VillageCode {
                name: "垣坡村委会",
                code: "003",
            },
            VillageCode {
                name: "高家村委会",
                code: "004",
            },
            VillageCode {
                name: "中垣村委会",
                code: "005",
            },
            VillageCode {
                name: "总垣村委会",
                code: "006",
            },
            VillageCode {
                name: "深巴村委会",
                code: "007",
            },
            VillageCode {
                name: "总堡村委会",
                code: "008",
            },
            VillageCode {
                name: "台尔哇村委会",
                code: "009",
            },
            VillageCode {
                name: "占沟村委会",
                code: "010",
            },
            VillageCode {
                name: "哈家村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "隆治乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "桥头村委会",
                code: "001",
            },
            VillageCode {
                name: "顶顶山村委会",
                code: "002",
            },
            VillageCode {
                name: "永平村委会",
                code: "003",
            },
            VillageCode {
                name: "秦家岭村委会",
                code: "004",
            },
            VillageCode {
                name: "张家村委会",
                code: "005",
            },
            VillageCode {
                name: "铁家村委会",
                code: "006",
            },
            VillageCode {
                name: "白武家村委会",
                code: "007",
            },
            VillageCode {
                name: "前山村委会",
                code: "008",
            },
            VillageCode {
                name: "李家村委会",
                code: "009",
            },
            VillageCode {
                name: "后山村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "大庄乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "大庄村委会",
                code: "001",
            },
            VillageCode {
                name: "孟家村委会",
                code: "002",
            },
            VillageCode {
                name: "委家台村委会",
                code: "003",
            },
            VillageCode {
                name: "哈家圈村委会",
                code: "004",
            },
            VillageCode {
                name: "高兴湾村委会",
                code: "005",
            },
            VillageCode {
                name: "崔家湾村委会",
                code: "006",
            },
            VillageCode {
                name: "塔卧村委会",
                code: "007",
            },
            VillageCode {
                name: "台集村委会",
                code: "008",
            },
            VillageCode {
                name: "三家湾村委会",
                code: "009",
            },
            VillageCode {
                name: "马家川村委会",
                code: "010",
            },
            VillageCode {
                name: "韩家岭村委会",
                code: "011",
            },
            VillageCode {
                name: "东山村委会",
                code: "012",
            },
            VillageCode {
                name: "塘卡村委会",
                code: "013",
            },
            VillageCode {
                name: "李家岭村委会",
                code: "014",
            },
            VillageCode {
                name: "丁家嘴村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "转导乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "酒坊村委会",
                code: "001",
            },
            VillageCode {
                name: "转导村委会",
                code: "002",
            },
            VillageCode {
                name: "忠孝村委会",
                code: "003",
            },
            VillageCode {
                name: "前坪村委会",
                code: "004",
            },
            VillageCode {
                name: "落龙沟村委会",
                code: "005",
            },
            VillageCode {
                name: "河纳沟村委会",
                code: "006",
            },
            VillageCode {
                name: "后沟村委会",
                code: "007",
            },
            VillageCode {
                name: "红花村委会",
                code: "008",
            },
            VillageCode {
                name: "王家庄村委会",
                code: "009",
            },
            VillageCode {
                name: "后坪村委会",
                code: "010",
            },
            VillageCode {
                name: "接官岭村委会",
                code: "011",
            },
            VillageCode {
                name: "苏家湾村委会",
                code: "012",
            },
            VillageCode {
                name: "王家山村委会",
                code: "013",
            },
            VillageCode {
                name: "大湾村委会",
                code: "014",
            },
            VillageCode {
                name: "中湾村委会",
                code: "015",
            },
            VillageCode {
                name: "三湾村委会",
                code: "016",
            },
            VillageCode {
                name: "红合岘村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "前河乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "台其村委会",
                code: "001",
            },
            VillageCode {
                name: "张家寺村委会",
                code: "002",
            },
            VillageCode {
                name: "前河村委会",
                code: "003",
            },
            VillageCode {
                name: "田家村委会",
                code: "004",
            },
            VillageCode {
                name: "芒拉村委会",
                code: "005",
            },
            VillageCode {
                name: "甘家川村委会",
                code: "006",
            },
            VillageCode {
                name: "丰一村委会",
                code: "007",
            },
            VillageCode {
                name: "丰二村委会",
                code: "008",
            },
            VillageCode {
                name: "木家寺村委会",
                code: "009",
            },
            VillageCode {
                name: "卧田村委会",
                code: "010",
            },
            VillageCode {
                name: "上湾村委会",
                code: "011",
            },
            VillageCode {
                name: "下湾村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "甘沟乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "李家村委会",
                code: "001",
            },
            VillageCode {
                name: "东山村委会",
                code: "002",
            },
            VillageCode {
                name: "静宁村委会",
                code: "003",
            },
            VillageCode {
                name: "前进村委会",
                code: "004",
            },
            VillageCode {
                name: "民族村委会",
                code: "005",
            },
            VillageCode {
                name: "盖子滩村委会",
                code: "006",
            },
            VillageCode {
                name: "团结村委会",
                code: "007",
            },
            VillageCode {
                name: "咱干村委会",
                code: "008",
            },
            VillageCode {
                name: "解放村委会",
                code: "009",
            },
            VillageCode {
                name: "互助村委会",
                code: "010",
            },
            VillageCode {
                name: "峡门村委会",
                code: "011",
            },
            VillageCode {
                name: "韩家嘴村委会",
                code: "012",
            },
            VillageCode {
                name: "光明村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "中川乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "清一村委会",
                code: "001",
            },
            VillageCode {
                name: "虎狼城村委会",
                code: "002",
            },
            VillageCode {
                name: "河东村委会",
                code: "003",
            },
            VillageCode {
                name: "河西村委会",
                code: "004",
            },
            VillageCode {
                name: "红崖村委会",
                code: "005",
            },
            VillageCode {
                name: "向阳村委会",
                code: "006",
            },
            VillageCode {
                name: "农场村委会",
                code: "007",
            },
            VillageCode {
                name: "光明村委会",
                code: "008",
            },
            VillageCode {
                name: "前进村委会",
                code: "009",
            },
            VillageCode {
                name: "清二村委会",
                code: "010",
            },
            VillageCode {
                name: "朱家岭村委会",
                code: "011",
            },
            VillageCode {
                name: "草滩村委会",
                code: "012",
            },
            VillageCode {
                name: "金田村委会",
                code: "013",
            },
            VillageCode {
                name: "美一村委会",
                code: "014",
            },
            VillageCode {
                name: "美二村委会",
                code: "015",
            },
            VillageCode {
                name: "峡口村委会",
                code: "016",
            },
            VillageCode {
                name: "团结村委会",
                code: "017",
            },
            VillageCode {
                name: "民主村委会",
                code: "018",
            },
            VillageCode {
                name: "八大山村委会",
                code: "019",
            },
            VillageCode {
                name: "盘格村委会",
                code: "020",
            },
            VillageCode {
                name: "魏家山村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "杏儿乡",
        code: "020",
        villages: &[
            VillageCode {
                name: "哦哇村委会",
                code: "001",
            },
            VillageCode {
                name: "日扎村委会",
                code: "002",
            },
            VillageCode {
                name: "协拉村委会",
                code: "003",
            },
            VillageCode {
                name: "卡洒哇村委会",
                code: "004",
            },
            VillageCode {
                name: "大庄村委会",
                code: "005",
            },
            VillageCode {
                name: "胜利村委会",
                code: "006",
            },
            VillageCode {
                name: "乱石头村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "核桃庄乡",
        code: "021",
        villages: &[
            VillageCode {
                name: "核桃庄村委会",
                code: "001",
            },
            VillageCode {
                name: "陶家村委会",
                code: "002",
            },
            VillageCode {
                name: "钟家村委会",
                code: "003",
            },
            VillageCode {
                name: "里长村委会",
                code: "004",
            },
            VillageCode {
                name: "高崖村委会",
                code: "005",
            },
            VillageCode {
                name: "排子山村委会",
                code: "006",
            },
            VillageCode {
                name: "安家村委会",
                code: "007",
            },
            VillageCode {
                name: "堡子村委会",
                code: "008",
            },
            VillageCode {
                name: "五方村委会",
                code: "009",
            },
            VillageCode {
                name: "大庄村委会",
                code: "010",
            },
            VillageCode {
                name: "牙合村委会",
                code: "011",
            },
            VillageCode {
                name: "大库土村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "新民乡",
        code: "022",
        villages: &[
            VillageCode {
                name: "千户湾村委会",
                code: "001",
            },
            VillageCode {
                name: "下川村委会",
                code: "002",
            },
            VillageCode {
                name: "候家山村委会",
                code: "003",
            },
            VillageCode {
                name: "下山村委会",
                code: "004",
            },
            VillageCode {
                name: "毛拉山村委会",
                code: "005",
            },
            VillageCode {
                name: "公巴台村委会",
                code: "006",
            },
            VillageCode {
                name: "若多村委会",
                code: "007",
            },
            VillageCode {
                name: "三岔村委会",
                code: "008",
            },
            VillageCode {
                name: "龙卧村委会",
                code: "009",
            },
            VillageCode {
                name: "雪沟村委会",
                code: "010",
            },
            VillageCode {
                name: "古岱村委会",
                code: "011",
            },
            VillageCode {
                name: "东湾村委会",
                code: "012",
            },
            VillageCode {
                name: "苏家庄村委会",
                code: "013",
            },
            VillageCode {
                name: "松树沟村委会",
                code: "014",
            },
            VillageCode {
                name: "马场村委会",
                code: "015",
            },
            VillageCode {
                name: "斗斗坡村委会",
                code: "016",
            },
        ],
    },
];

static TOWNS_QH_012: [TownCode; 20] = [
    TownCode {
        name: "高寨街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "高寨镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "曹家堡村委会",
                code: "002",
            },
            VillageCode {
                name: "东村村委会",
                code: "003",
            },
            VillageCode {
                name: "东庄村委会",
                code: "004",
            },
            VillageCode {
                name: "北庄村委会",
                code: "005",
            },
            VillageCode {
                name: "西湾村委会",
                code: "006",
            },
            VillageCode {
                name: "西庄村委会",
                code: "007",
            },
            VillageCode {
                name: "中村村委会",
                code: "008",
            },
            VillageCode {
                name: "西村村委会",
                code: "009",
            },
            VillageCode {
                name: "上寨村委会",
                code: "010",
            },
            VillageCode {
                name: "下寨村委会",
                code: "011",
            },
            VillageCode {
                name: "小寨村委会",
                code: "012",
            },
            VillageCode {
                name: "星家村委会",
                code: "013",
            },
            VillageCode {
                name: "站家村委会",
                code: "014",
            },
            VillageCode {
                name: "白马村委会",
                code: "015",
            },
            VillageCode {
                name: "小红沟村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "威远镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "北街社区居委会",
                code: "001",
            },
            VillageCode {
                name: "南街社区居委会",
                code: "002",
            },
            VillageCode {
                name: "东街社区居委会",
                code: "003",
            },
            VillageCode {
                name: "西街社区居委会",
                code: "004",
            },
            VillageCode {
                name: "西林泰社区居委会",
                code: "005",
            },
            VillageCode {
                name: "明珠社区居委会",
                code: "006",
            },
            VillageCode {
                name: "阳光社区居委会",
                code: "007",
            },
            VillageCode {
                name: "鼓楼花园社区居委会",
                code: "008",
            },
            VillageCode {
                name: "安定村委会",
                code: "009",
            },
            VillageCode {
                name: "白崖村委会",
                code: "010",
            },
            VillageCode {
                name: "班家湾村委会",
                code: "011",
            },
            VillageCode {
                name: "大寺村村委会",
                code: "012",
            },
            VillageCode {
                name: "大寺路村村委会",
                code: "013",
            },
            VillageCode {
                name: "古城村委会",
                code: "014",
            },
            VillageCode {
                name: "红咀尔村委会",
                code: "015",
            },
            VillageCode {
                name: "红崖村委会",
                code: "016",
            },
            VillageCode {
                name: "兰家村委会",
                code: "017",
            },
            VillageCode {
                name: "凉州营村委会",
                code: "018",
            },
            VillageCode {
                name: "纳家村委会",
                code: "019",
            },
            VillageCode {
                name: "前跃村委会",
                code: "020",
            },
            VillageCode {
                name: "深沟村委会",
                code: "021",
            },
            VillageCode {
                name: "寺壕子村委会",
                code: "022",
            },
            VillageCode {
                name: "西坡村委会",
                code: "023",
            },
            VillageCode {
                name: "西上街村委会",
                code: "024",
            },
            VillageCode {
                name: "西下街村委会",
                code: "025",
            },
            VillageCode {
                name: "小寺村委会",
                code: "026",
            },
            VillageCode {
                name: "崖头村委会",
                code: "027",
            },
            VillageCode {
                name: "余家村委会",
                code: "028",
            },
            VillageCode {
                name: "卓扎沟村委会",
                code: "029",
            },
            VillageCode {
                name: "卓扎滩村委会",
                code: "030",
            },
            VillageCode {
                name: "小庄村委会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "丹麻镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "丹麻镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "补家村委会",
                code: "002",
            },
            VillageCode {
                name: "岔尔沟门村委会",
                code: "003",
            },
            VillageCode {
                name: "东家村委会",
                code: "004",
            },
            VillageCode {
                name: "锦州村委会",
                code: "005",
            },
            VillageCode {
                name: "拉庄村委会",
                code: "006",
            },
            VillageCode {
                name: "山城村委会",
                code: "007",
            },
            VillageCode {
                name: "松德村委会",
                code: "008",
            },
            VillageCode {
                name: "索卜沟村委会",
                code: "009",
            },
            VillageCode {
                name: "索卜滩村委会",
                code: "010",
            },
            VillageCode {
                name: "哇麻村委会",
                code: "011",
            },
            VillageCode {
                name: "汪家村委会",
                code: "012",
            },
            VillageCode {
                name: "温家村委会",
                code: "013",
            },
            VillageCode {
                name: "西丹麻村委会",
                code: "014",
            },
            VillageCode {
                name: "新添堡村委会",
                code: "015",
            },
            VillageCode {
                name: "泽林村委会",
                code: "016",
            },
            VillageCode {
                name: "桦林村委会",
                code: "017",
            },
            VillageCode {
                name: "东丹麻村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "南门峡镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "南门峡镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "北沟脑村委会",
                code: "002",
            },
            VillageCode {
                name: "东沟村委会",
                code: "003",
            },
            VillageCode {
                name: "古边村委会",
                code: "004",
            },
            VillageCode {
                name: "卷槽村委会",
                code: "005",
            },
            VillageCode {
                name: "老虎沟村委会",
                code: "006",
            },
            VillageCode {
                name: "麻其村委会",
                code: "007",
            },
            VillageCode {
                name: "磨尔沟村委会",
                code: "008",
            },
            VillageCode {
                name: "七塔尔村委会",
                code: "009",
            },
            VillageCode {
                name: "祁家庄村委会",
                code: "010",
            },
            VillageCode {
                name: "西坡村委会",
                code: "011",
            },
            VillageCode {
                name: "西山根村委会",
                code: "012",
            },
            VillageCode {
                name: "峡口村委会",
                code: "013",
            },
            VillageCode {
                name: "尕寺加村委会",
                code: "014",
            },
            VillageCode {
                name: "却藏寺村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "加定镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "加定镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "桥头村委会",
                code: "002",
            },
            VillageCode {
                name: "加塘村委会",
                code: "003",
            },
            VillageCode {
                name: "浪士当村委会",
                code: "004",
            },
            VillageCode {
                name: "下河村委会",
                code: "005",
            },
            VillageCode {
                name: "扎隆沟村委会",
                code: "006",
            },
            VillageCode {
                name: "扎隆口村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "塘川镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "塘川镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "雷家堡村委会",
                code: "002",
            },
            VillageCode {
                name: "甘一村委会",
                code: "003",
            },
            VillageCode {
                name: "甘二村委会",
                code: "004",
            },
            VillageCode {
                name: "后山村委会",
                code: "005",
            },
            VillageCode {
                name: "刘家村委会",
                code: "006",
            },
            VillageCode {
                name: "三其村委会",
                code: "007",
            },
            VillageCode {
                name: "上山城村委会",
                code: "008",
            },
            VillageCode {
                name: "水湾村委会",
                code: "009",
            },
            VillageCode {
                name: "陶家寨村委会",
                code: "010",
            },
            VillageCode {
                name: "汪家村委会",
                code: "011",
            },
            VillageCode {
                name: "五上村委会",
                code: "012",
            },
            VillageCode {
                name: "五下村委会",
                code: "013",
            },
            VillageCode {
                name: "下山城村委会",
                code: "014",
            },
            VillageCode {
                name: "总寨村委会",
                code: "015",
            },
            VillageCode {
                name: "双树村委会",
                code: "016",
            },
            VillageCode {
                name: "大沟村委会",
                code: "017",
            },
            VillageCode {
                name: "大通苑村委会",
                code: "018",
            },
            VillageCode {
                name: "大庄村委会",
                code: "019",
            },
            VillageCode {
                name: "董家村委会",
                code: "020",
            },
            VillageCode {
                name: "高羌村委会",
                code: "021",
            },
            VillageCode {
                name: "黄家湾村委会",
                code: "022",
            },
            VillageCode {
                name: "吉家沟村委会",
                code: "023",
            },
            VillageCode {
                name: "坪地村委会",
                code: "024",
            },
            VillageCode {
                name: "什字村委会",
                code: "025",
            },
            VillageCode {
                name: "包家口村委会",
                code: "026",
            },
            VillageCode {
                name: "新元村委会",
                code: "027",
            },
            VillageCode {
                name: "周家村委会",
                code: "028",
            },
            VillageCode {
                name: "朱家口村委会",
                code: "029",
            },
        ],
    },
    TownCode {
        name: "五十镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "五十镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "桑士哥村委会",
                code: "002",
            },
            VillageCode {
                name: "班彦村委会",
                code: "003",
            },
            VillageCode {
                name: "保家村委会",
                code: "004",
            },
            VillageCode {
                name: "北庄村委会",
                code: "005",
            },
            VillageCode {
                name: "荷包村委会",
                code: "006",
            },
            VillageCode {
                name: "奎浪村委会",
                code: "007",
            },
            VillageCode {
                name: "拉洞村委会",
                code: "008",
            },
            VillageCode {
                name: "拉日村委会",
                code: "009",
            },
            VillageCode {
                name: "柳家村委会",
                code: "010",
            },
            VillageCode {
                name: "三庄村委会",
                code: "011",
            },
            VillageCode {
                name: "上滩村委会",
                code: "012",
            },
            VillageCode {
                name: "下滩村委会",
                code: "013",
            },
            VillageCode {
                name: "寺滩村委会",
                code: "014",
            },
            VillageCode {
                name: "土观村委会",
                code: "015",
            },
            VillageCode {
                name: "五十村委会",
                code: "016",
            },
            VillageCode {
                name: "卓科村委会",
                code: "017",
            },
            VillageCode {
                name: "桦林村委会",
                code: "018",
            },
            VillageCode {
                name: "巴洪村委会",
                code: "019",
            },
            VillageCode {
                name: "扎巴村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "五峰镇",
        code: "008",
        villages: &[
            VillageCode {
                name: "五峰镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "上庄村委会",
                code: "002",
            },
            VillageCode {
                name: "北沟村委会",
                code: "003",
            },
            VillageCode {
                name: "苍家村委会",
                code: "004",
            },
            VillageCode {
                name: "海子村委会",
                code: "005",
            },
            VillageCode {
                name: "后头沟村委会",
                code: "006",
            },
            VillageCode {
                name: "纳家村委会",
                code: "007",
            },
            VillageCode {
                name: "平峰村委会",
                code: "008",
            },
            VillageCode {
                name: "七塔儿村委会",
                code: "009",
            },
            VillageCode {
                name: "上马村委会",
                code: "010",
            },
            VillageCode {
                name: "下马一村委会",
                code: "011",
            },
            VillageCode {
                name: "下马二村委会",
                code: "012",
            },
            VillageCode {
                name: "石湾村委会",
                code: "013",
            },
            VillageCode {
                name: "新庄村委会",
                code: "014",
            },
            VillageCode {
                name: "兴隆村委会",
                code: "015",
            },
            VillageCode {
                name: "支高村委会",
                code: "016",
            },
            VillageCode {
                name: "转咀村委会",
                code: "017",
            },
            VillageCode {
                name: "白多峨村委会",
                code: "018",
            },
            VillageCode {
                name: "陈家台村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "台子乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "上台村委会",
                code: "001",
            },
            VillageCode {
                name: "菜滩村委会",
                code: "002",
            },
            VillageCode {
                name: "菜子沟村委会",
                code: "003",
            },
            VillageCode {
                name: "长寿村委会",
                code: "004",
            },
            VillageCode {
                name: "出路沟村委会",
                code: "005",
            },
            VillageCode {
                name: "多士代村委会",
                code: "006",
            },
            VillageCode {
                name: "格隆村委会",
                code: "007",
            },
            VillageCode {
                name: "河东村委会",
                code: "008",
            },
            VillageCode {
                name: "楼子滩村委会",
                code: "009",
            },
            VillageCode {
                name: "恰卡村委会",
                code: "010",
            },
            VillageCode {
                name: "塘巴村委会",
                code: "011",
            },
            VillageCode {
                name: "哇麻村委会",
                code: "012",
            },
            VillageCode {
                name: "峡门村委会",
                code: "013",
            },
            VillageCode {
                name: "阿士记村委会",
                code: "014",
            },
            VillageCode {
                name: "下台一村委会",
                code: "015",
            },
            VillageCode {
                name: "下台二村委会",
                code: "016",
            },
            VillageCode {
                name: "新城村委会",
                code: "017",
            },
            VillageCode {
                name: "新合村委会",
                code: "018",
            },
            VillageCode {
                name: "直沟村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "西山乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "和平村委会",
                code: "001",
            },
            VillageCode {
                name: "郭家沟村委会",
                code: "002",
            },
            VillageCode {
                name: "刘家沟村委会",
                code: "003",
            },
            VillageCode {
                name: "麻莲滩村委会",
                code: "004",
            },
            VillageCode {
                name: "邵代村委会",
                code: "005",
            },
            VillageCode {
                name: "铁家村委会",
                code: "006",
            },
            VillageCode {
                name: "湾地村委会",
                code: "007",
            },
            VillageCode {
                name: "王家沟村委会",
                code: "008",
            },
            VillageCode {
                name: "王家山村委会",
                code: "009",
            },
            VillageCode {
                name: "王家庄村委会",
                code: "010",
            },
            VillageCode {
                name: "西沟底村委会",
                code: "011",
            },
            VillageCode {
                name: "西沟坪村委会",
                code: "012",
            },
            VillageCode {
                name: "牙合村委会",
                code: "013",
            },
            VillageCode {
                name: "杨徐村委会",
                code: "014",
            },
            VillageCode {
                name: "张家沟村委会",
                code: "015",
            },
            VillageCode {
                name: "郑家山村委会",
                code: "016",
            },
            VillageCode {
                name: "东山村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "红崖子沟乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "蔡家村委会",
                code: "001",
            },
            VillageCode {
                name: "大庄廓村委会",
                code: "002",
            },
            VillageCode {
                name: "担水路村委会",
                code: "003",
            },
            VillageCode {
                name: "加克村委会",
                code: "004",
            },
            VillageCode {
                name: "老幼村委会",
                code: "005",
            },
            VillageCode {
                name: "流水沟村委会",
                code: "006",
            },
            VillageCode {
                name: "芦草沟村委会",
                code: "007",
            },
            VillageCode {
                name: "马圈村委会",
                code: "008",
            },
            VillageCode {
                name: "西山村委会",
                code: "009",
            },
            VillageCode {
                name: "张家村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "巴扎藏族乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "抓什究村委会",
                code: "001",
            },
            VillageCode {
                name: "财隆村委会",
                code: "002",
            },
            VillageCode {
                name: "甘冲沟村委会",
                code: "003",
            },
            VillageCode {
                name: "甘冲口村委会",
                code: "004",
            },
            VillageCode {
                name: "学科村委会",
                code: "005",
            },
            VillageCode {
                name: "元圃村委会",
                code: "006",
            },
            VillageCode {
                name: "柏木峡村委会",
                code: "007",
            },
            VillageCode {
                name: "峡塘村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "哈拉直沟乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "尚家村委会",
                code: "001",
            },
            VillageCode {
                name: "蔡家村委会",
                code: "002",
            },
            VillageCode {
                name: "费家村委会",
                code: "003",
            },
            VillageCode {
                name: "蒋家村委会",
                code: "004",
            },
            VillageCode {
                name: "里外台村委会",
                code: "005",
            },
            VillageCode {
                name: "毛荷堡村委会",
                code: "006",
            },
            VillageCode {
                name: "师家村委会",
                code: "007",
            },
            VillageCode {
                name: "孙家村委会",
                code: "008",
            },
            VillageCode {
                name: "魏家堡村委会",
                code: "009",
            },
            VillageCode {
                name: "新庄村委会",
                code: "010",
            },
            VillageCode {
                name: "杏园村委会",
                code: "011",
            },
            VillageCode {
                name: "盐昌村委会",
                code: "012",
            },
            VillageCode {
                name: "白崖村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "松多藏族乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "什八洞沟村委会",
                code: "001",
            },
            VillageCode {
                name: "哈什村委会",
                code: "002",
            },
            VillageCode {
                name: "花园村委会",
                code: "003",
            },
            VillageCode {
                name: "麻洞村委会",
                code: "004",
            },
            VillageCode {
                name: "马营村委会",
                code: "005",
            },
            VillageCode {
                name: "前隆村委会",
                code: "006",
            },
            VillageCode {
                name: "松多村委会",
                code: "007",
            },
            VillageCode {
                name: "本康沟村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "东山乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "大庄村委会",
                code: "001",
            },
            VillageCode {
                name: "岔尔沟村委会",
                code: "002",
            },
            VillageCode {
                name: "大泉村委会",
                code: "003",
            },
            VillageCode {
                name: "东山村委会",
                code: "004",
            },
            VillageCode {
                name: "贺尔村委会",
                code: "005",
            },
            VillageCode {
                name: "吉家岭村委会",
                code: "006",
            },
            VillageCode {
                name: "联大村委会",
                code: "007",
            },
            VillageCode {
                name: "上元保村委会",
                code: "008",
            },
            VillageCode {
                name: "寺尔村委会",
                code: "009",
            },
            VillageCode {
                name: "下李村委会",
                code: "010",
            },
            VillageCode {
                name: "下元保村委会",
                code: "011",
            },
            VillageCode {
                name: "白牙合村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "东和乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "宋家庄村委会",
                code: "001",
            },
            VillageCode {
                name: "大桦林村委会",
                code: "002",
            },
            VillageCode {
                name: "小桦林村委会",
                code: "003",
            },
            VillageCode {
                name: "黑庄村委会",
                code: "004",
            },
            VillageCode {
                name: "克麻村委会",
                code: "005",
            },
            VillageCode {
                name: "李家庄村委会",
                code: "006",
            },
            VillageCode {
                name: "柳树沟村委会",
                code: "007",
            },
            VillageCode {
                name: "麻吉村委会",
                code: "008",
            },
            VillageCode {
                name: "山城村委会",
                code: "009",
            },
            VillageCode {
                name: "魏家滩村委会",
                code: "010",
            },
            VillageCode {
                name: "新庄村委会",
                code: "011",
            },
            VillageCode {
                name: "姚家沟村委会",
                code: "012",
            },
            VillageCode {
                name: "元山村委会",
                code: "013",
            },
            VillageCode {
                name: "袁家庄村委会",
                code: "014",
            },
            VillageCode {
                name: "朱家台村委会",
                code: "015",
            },
            VillageCode {
                name: "尕寺加村委会",
                code: "016",
            },
            VillageCode {
                name: "大庄村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "东沟乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "塘拉村委会",
                code: "001",
            },
            VillageCode {
                name: "大庄村委会",
                code: "002",
            },
            VillageCode {
                name: "尔开村委会",
                code: "003",
            },
            VillageCode {
                name: "沟脑村委会",
                code: "004",
            },
            VillageCode {
                name: "花园村委会",
                code: "005",
            },
            VillageCode {
                name: "卡子村委会",
                code: "006",
            },
            VillageCode {
                name: "口子村委会",
                code: "007",
            },
            VillageCode {
                name: "龙一村委会",
                code: "008",
            },
            VillageCode {
                name: "龙二村委会",
                code: "009",
            },
            VillageCode {
                name: "洛少村委会",
                code: "010",
            },
            VillageCode {
                name: "纳卡村委会",
                code: "011",
            },
            VillageCode {
                name: "年先村委会",
                code: "012",
            },
            VillageCode {
                name: "石窝村委会",
                code: "013",
            },
            VillageCode {
                name: "姚马村委会",
                code: "014",
            },
            VillageCode {
                name: "昝扎村委会",
                code: "015",
            },
            VillageCode {
                name: "曹家村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "林川乡",
        code: "018",
        villages: &[
            VillageCode {
                name: "贺尔村委会",
                code: "001",
            },
            VillageCode {
                name: "大河欠村委会",
                code: "002",
            },
            VillageCode {
                name: "小河欠村委会",
                code: "003",
            },
            VillageCode {
                name: "河欠口村委会",
                code: "004",
            },
            VillageCode {
                name: "韭菜沟村委会",
                code: "005",
            },
            VillageCode {
                name: "马场村委会",
                code: "006",
            },
            VillageCode {
                name: "唐日台村委会",
                code: "007",
            },
            VillageCode {
                name: "许家村委会",
                code: "008",
            },
            VillageCode {
                name: "窑庄村委会",
                code: "009",
            },
            VillageCode {
                name: "作干村委会",
                code: "010",
            },
            VillageCode {
                name: "包马村委会",
                code: "011",
            },
            VillageCode {
                name: "保家村委会",
                code: "012",
            },
            VillageCode {
                name: "苍家村委会",
                code: "013",
            },
            VillageCode {
                name: "马家村委会",
                code: "014",
            },
            VillageCode {
                name: "泥麻村委会",
                code: "015",
            },
            VillageCode {
                name: "水洞村委会",
                code: "016",
            },
            VillageCode {
                name: "峡门村委会",
                code: "017",
            },
            VillageCode {
                name: "新庄村委会",
                code: "018",
            },
            VillageCode {
                name: "尕寺加村委会",
                code: "019",
            },
            VillageCode {
                name: "昝扎村委会",
                code: "020",
            },
            VillageCode {
                name: "巴扎村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "蔡家堡乡",
        code: "019",
        villages: &[
            VillageCode {
                name: "岩崖村委会",
                code: "001",
            },
            VillageCode {
                name: "大庄一村委会",
                code: "002",
            },
            VillageCode {
                name: "东家沟村委会",
                code: "003",
            },
            VillageCode {
                name: "关家山村委会",
                code: "004",
            },
            VillageCode {
                name: "后湾村委会",
                code: "005",
            },
            VillageCode {
                name: "刘李山村委会",
                code: "006",
            },
            VillageCode {
                name: "马莲滩村委会",
                code: "007",
            },
            VillageCode {
                name: "泉湾村委会",
                code: "008",
            },
            VillageCode {
                name: "上刘家村委会",
                code: "009",
            },
            VillageCode {
                name: "孙家湾村委会",
                code: "010",
            },
            VillageCode {
                name: "杨家湾村委会",
                code: "011",
            },
            VillageCode {
                name: "包刘村委会",
                code: "012",
            },
            VillageCode {
                name: "大庄二村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "曹家堡临空综合经济园互助园区",
        code: "020",
        villages: &[VillageCode {
            name: "曹家堡临空综合经济园互助园区虚拟社区",
            code: "001",
        }],
    },
];

static TOWNS_QH_013: [TownCode; 17] = [
    TownCode {
        name: "巴燕镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城东居民委员会",
                code: "001",
            },
            VillageCode {
                name: "城西居民委员会",
                code: "002",
            },
            VillageCode {
                name: "城南居民委员会",
                code: "003",
            },
            VillageCode {
                name: "下加合村民委员会",
                code: "004",
            },
            VillageCode {
                name: "金家庄村民委员会",
                code: "005",
            },
            VillageCode {
                name: "什杰列村民委员会",
                code: "006",
            },
            VillageCode {
                name: "上圈村民委员会",
                code: "007",
            },
            VillageCode {
                name: "尕西沟村民委员会",
                code: "008",
            },
            VillageCode {
                name: "西上村民委员会",
                code: "009",
            },
            VillageCode {
                name: "西下村民委员会",
                code: "010",
            },
            VillageCode {
                name: "南街村民委员会",
                code: "011",
            },
            VillageCode {
                name: "北街村民委员会",
                code: "012",
            },
            VillageCode {
                name: "东上村民委员会",
                code: "013",
            },
            VillageCode {
                name: "儒家沟村民委员会",
                code: "014",
            },
            VillageCode {
                name: "东下村民委员会",
                code: "015",
            },
            VillageCode {
                name: "上加合村民委员会",
                code: "016",
            },
            VillageCode {
                name: "下地滩村民委员会",
                code: "017",
            },
            VillageCode {
                name: "上地滩村民委员会",
                code: "018",
            },
            VillageCode {
                name: "下卧力尕村民委员会",
                code: "019",
            },
            VillageCode {
                name: "上卧力尕村民委员会",
                code: "020",
            },
            VillageCode {
                name: "哈洞村民委员会",
                code: "021",
            },
            VillageCode {
                name: "绽麻村民委员会",
                code: "022",
            },
            VillageCode {
                name: "中拉干村民委员会",
                code: "023",
            },
            VillageCode {
                name: "上拉干村民委员会",
                code: "024",
            },
            VillageCode {
                name: "下胡拉村民委员会",
                code: "025",
            },
            VillageCode {
                name: "下山根村民委员会",
                code: "026",
            },
            VillageCode {
                name: "下沟村民委员会",
                code: "027",
            },
            VillageCode {
                name: "卜隆村民委员会",
                code: "028",
            },
            VillageCode {
                name: "上吾具村民委员会",
                code: "029",
            },
            VillageCode {
                name: "下吾具村民委员会",
                code: "030",
            },
            VillageCode {
                name: "克麻村民委员会",
                code: "031",
            },
            VillageCode {
                name: "后沟村民委员会",
                code: "032",
            },
            VillageCode {
                name: "藏滩村民委员会",
                code: "033",
            },
            VillageCode {
                name: "辛家窑村民委员会",
                code: "034",
            },
            VillageCode {
                name: "瑶湾村民委员会",
                code: "035",
            },
            VillageCode {
                name: "李家庄村民委员会",
                code: "036",
            },
            VillageCode {
                name: "阿扎卜扎村民委员会",
                code: "037",
            },
            VillageCode {
                name: "水乃海村民委员会",
                code: "038",
            },
            VillageCode {
                name: "马场村民委员会",
                code: "039",
            },
            VillageCode {
                name: "庙尔沟村民委员会",
                code: "040",
            },
            VillageCode {
                name: "寺尔沟村民委员会",
                code: "041",
            },
        ],
    },
    TownCode {
        name: "群科镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "群科居委会",
                code: "001",
            },
            VillageCode {
                name: "群科新区城中居民委员会",
                code: "002",
            },
            VillageCode {
                name: "群科新区城北居民委员会",
                code: "003",
            },
            VillageCode {
                name: "群科新区城西居民委员会",
                code: "004",
            },
            VillageCode {
                name: "文卜具村民委员会",
                code: "005",
            },
            VillageCode {
                name: "雪什藏村民委员会",
                code: "006",
            },
            VillageCode {
                name: "群科村民委员会",
                code: "007",
            },
            VillageCode {
                name: "若加村民委员会",
                code: "008",
            },
            VillageCode {
                name: "舍仁村民委员会",
                code: "009",
            },
            VillageCode {
                name: "木哈村民委员会",
                code: "010",
            },
            VillageCode {
                name: "先口一村民委员会",
                code: "011",
            },
            VillageCode {
                name: "先口二村民委员会",
                code: "012",
            },
            VillageCode {
                name: "乙沙一村民委员会",
                code: "013",
            },
            VillageCode {
                name: "乙沙二村民委员会",
                code: "014",
            },
            VillageCode {
                name: "科木其村民委员会",
                code: "015",
            },
            VillageCode {
                name: "格尔麻村民委员会",
                code: "016",
            },
            VillageCode {
                name: "滩南村民委员会",
                code: "017",
            },
            VillageCode {
                name: "滩心村民委员会",
                code: "018",
            },
            VillageCode {
                name: "滩北村民委员会",
                code: "019",
            },
            VillageCode {
                name: "向东村民委员会",
                code: "020",
            },
            VillageCode {
                name: "东风村民委员会",
                code: "021",
            },
            VillageCode {
                name: "工农兵村民委员会",
                code: "022",
            },
            VillageCode {
                name: "新村一村民委员会",
                code: "023",
            },
            VillageCode {
                name: "新村二村民委员会",
                code: "024",
            },
            VillageCode {
                name: "邮电村民委员会",
                code: "025",
            },
            VillageCode {
                name: "日兰村民委员会",
                code: "026",
            },
            VillageCode {
                name: "团结一村民委员会",
                code: "027",
            },
            VillageCode {
                name: "团结二村民委员会",
                code: "028",
            },
            VillageCode {
                name: "则塘村民委员会",
                code: "029",
            },
            VillageCode {
                name: "公义村民委员会",
                code: "030",
            },
            VillageCode {
                name: "加洛乎村民委员会",
                code: "031",
            },
            VillageCode {
                name: "水库滩村民委员会",
                code: "032",
            },
            VillageCode {
                name: "安达其哈村民委员会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "牙什尕镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "牙什尕镇居委会",
                code: "001",
            },
            VillageCode {
                name: "李家峡居民委员会",
                code: "002",
            },
            VillageCode {
                name: "下多巴二村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下多巴一村民委员会",
                code: "004",
            },
            VillageCode {
                name: "参果滩一村民委员会",
                code: "005",
            },
            VillageCode {
                name: "参果滩二村民委员会",
                code: "006",
            },
            VillageCode {
                name: "参果滩三村民委员会",
                code: "007",
            },
            VillageCode {
                name: "唐沙一村民委员会",
                code: "008",
            },
            VillageCode {
                name: "唐沙二村民委员会",
                code: "009",
            },
            VillageCode {
                name: "唐沙三村民委员会",
                code: "010",
            },
            VillageCode {
                name: "完干滩村民委员会",
                code: "011",
            },
            VillageCode {
                name: "拉公麻村民委员会",
                code: "012",
            },
            VillageCode {
                name: "城车村民委员会",
                code: "013",
            },
            VillageCode {
                name: "上滩村民委员会",
                code: "014",
            },
            VillageCode {
                name: "宗尕堂村民委员会",
                code: "015",
            },
            VillageCode {
                name: "沙拉沟村民委员会",
                code: "016",
            },
            VillageCode {
                name: "盘龙曲麻村民委员会",
                code: "017",
            },
            VillageCode {
                name: "上多巴村民委员会",
                code: "018",
            },
            VillageCode {
                name: "牙什尕村民委员会",
                code: "019",
            },
            VillageCode {
                name: "哇尔江村民委员会",
                code: "020",
            },
            VillageCode {
                name: "园艺场村民委员会",
                code: "021",
            },
            VillageCode {
                name: "曲日麻卡村民委员会",
                code: "022",
            },
            VillageCode {
                name: "黄河沿村民委员会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "甘都镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "甘都居委会",
                code: "001",
            },
            VillageCode {
                name: "公伯峡居委会",
                code: "002",
            },
            VillageCode {
                name: "牙路乎村民委员会",
                code: "003",
            },
            VillageCode {
                name: "苏合加村民委员会",
                code: "004",
            },
            VillageCode {
                name: "关巴村民委员会",
                code: "005",
            },
            VillageCode {
                name: "阿河滩村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东滩一村民委员会",
                code: "007",
            },
            VillageCode {
                name: "东滩二村民委员会",
                code: "008",
            },
            VillageCode {
                name: "东滩三村民委员会",
                code: "009",
            },
            VillageCode {
                name: "东滩四村民委员会",
                code: "010",
            },
            VillageCode {
                name: "东滩五村民委员会",
                code: "011",
            },
            VillageCode {
                name: "东滩六村民委员会",
                code: "012",
            },
            VillageCode {
                name: "东滩七村民委员会",
                code: "013",
            },
            VillageCode {
                name: "东风村民委员会",
                code: "014",
            },
            VillageCode {
                name: "桥头村民委员会",
                code: "015",
            },
            VillageCode {
                name: "下四合生村民委员会",
                code: "016",
            },
            VillageCode {
                name: "上四合生村民委员会",
                code: "017",
            },
            VillageCode {
                name: "工什加村民委员会",
                code: "018",
            },
            VillageCode {
                name: "朱乎隆村民委员会",
                code: "019",
            },
            VillageCode {
                name: "拉木村民委员会",
                code: "020",
            },
            VillageCode {
                name: "列卜加村民委员会",
                code: "021",
            },
            VillageCode {
                name: "甘都街村民委员会",
                code: "022",
            },
            VillageCode {
                name: "水车村民委员会",
                code: "023",
            },
            VillageCode {
                name: "西滩村民委员会",
                code: "024",
            },
            VillageCode {
                name: "阿化村民委员会",
                code: "025",
            },
            VillageCode {
                name: "唐寺岗村民委员会",
                code: "026",
            },
            VillageCode {
                name: "牙目村民委员会",
                code: "027",
            },
            VillageCode {
                name: "隆康一村民委员会",
                code: "028",
            },
            VillageCode {
                name: "隆康二村民委员会",
                code: "029",
            },
            VillageCode {
                name: "隆康三村民委员会",
                code: "030",
            },
            VillageCode {
                name: "幸福村民委员会",
                code: "031",
            },
        ],
    },
    TownCode {
        name: "扎巴镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "扎巴居委会",
                code: "001",
            },
            VillageCode {
                name: "阿岱村民委员会",
                code: "002",
            },
            VillageCode {
                name: "下扎巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "窑洞村民委员会",
                code: "004",
            },
            VillageCode {
                name: "拉让滩村民委员会",
                code: "005",
            },
            VillageCode {
                name: "拉曲滩一村民委员会",
                code: "006",
            },
            VillageCode {
                name: "拉曲滩二村民委员会",
                code: "007",
            },
            VillageCode {
                name: "扎巴一村民委员会",
                code: "008",
            },
            VillageCode {
                name: "扎巴二村民委员会",
                code: "009",
            },
            VillageCode {
                name: "扎巴三村民委员会",
                code: "010",
            },
            VillageCode {
                name: "扎巴四村民委员会",
                code: "011",
            },
            VillageCode {
                name: "科台村民委员会",
                code: "012",
            },
            VillageCode {
                name: "阴坡村民委员会",
                code: "013",
            },
            VillageCode {
                name: "阳坡村民委员会",
                code: "014",
            },
            VillageCode {
                name: "浪隆村民委员会",
                code: "015",
            },
            VillageCode {
                name: "曲先昂村民委员会",
                code: "016",
            },
            VillageCode {
                name: "结拉村民委员会",
                code: "017",
            },
            VillageCode {
                name: "冷扎村民委员会",
                code: "018",
            },
            VillageCode {
                name: "知亥买村民委员会",
                code: "019",
            },
            VillageCode {
                name: "黄麻村民委员会",
                code: "020",
            },
            VillageCode {
                name: "下洛乎藏村民委员会",
                code: "021",
            },
            VillageCode {
                name: "滩滩村民委员会",
                code: "022",
            },
            VillageCode {
                name: "上脑村民委员会",
                code: "023",
            },
            VillageCode {
                name: "吉康村民委员会",
                code: "024",
            },
            VillageCode {
                name: "香乙麻村民委员会",
                code: "025",
            },
            VillageCode {
                name: "洛福村民委员会",
                code: "026",
            },
            VillageCode {
                name: "挖隆沟村民委员会",
                code: "027",
            },
            VillageCode {
                name: "扎让村民委员会",
                code: "028",
            },
            VillageCode {
                name: "大拉曲村民委员会",
                code: "029",
            },
            VillageCode {
                name: "关沙村民委员会",
                code: "030",
            },
            VillageCode {
                name: "四哈宁村民委员会",
                code: "031",
            },
            VillageCode {
                name: "乙沙尔村民委员会",
                code: "032",
            },
            VillageCode {
                name: "阿卡拉村民委员会",
                code: "033",
            },
            VillageCode {
                name: "西滩村民委员会",
                code: "034",
            },
            VillageCode {
                name: "双格达村民委员会",
                code: "035",
            },
            VillageCode {
                name: "南滩村民委员会",
                code: "036",
            },
            VillageCode {
                name: "全藏村民委员会",
                code: "037",
            },
            VillageCode {
                name: "本康沟村民委员会",
                code: "038",
            },
            VillageCode {
                name: "扎拉毛村民委员会",
                code: "039",
            },
        ],
    },
    TownCode {
        name: "昂思多镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "昂思多居委会",
                code: "001",
            },
            VillageCode {
                name: "沙吾昂村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阳坡村民委员会",
                code: "003",
            },
            VillageCode {
                name: "阴坡村民委员会",
                code: "004",
            },
            VillageCode {
                name: "红卡哇一村民委员会",
                code: "005",
            },
            VillageCode {
                name: "红卡哇二村民委员会",
                code: "006",
            },
            VillageCode {
                name: "公拜岭村民委员会",
                code: "007",
            },
            VillageCode {
                name: "关相口村民委员会",
                code: "008",
            },
            VillageCode {
                name: "尖巴昂村民委员会",
                code: "009",
            },
            VillageCode {
                name: "尕吾塘村民委员会",
                code: "010",
            },
            VillageCode {
                name: "拉昂村民委员会",
                code: "011",
            },
            VillageCode {
                name: "吾则塘村民委员会",
                code: "012",
            },
            VillageCode {
                name: "梅加村民委员会",
                code: "013",
            },
            VillageCode {
                name: "玉麦街村民委员会",
                code: "014",
            },
            VillageCode {
                name: "雄村民委员会",
                code: "015",
            },
            VillageCode {
                name: "若麻岭村民委员会",
                code: "016",
            },
            VillageCode {
                name: "香加岭村民委员会",
                code: "017",
            },
            VillageCode {
                name: "先群村民委员会",
                code: "018",
            },
            VillageCode {
                name: "关沙村民委员会",
                code: "019",
            },
            VillageCode {
                name: "具乎扎村民委员会",
                code: "020",
            },
            VillageCode {
                name: "公布昂村民委员会",
                code: "021",
            },
            VillageCode {
                name: "尕麻甫村民委员会",
                code: "022",
            },
            VillageCode {
                name: "河滩庄村民委员会",
                code: "023",
            },
            VillageCode {
                name: "尔尕昂村民委员会",
                code: "024",
            },
            VillageCode {
                name: "尕什加村民委员会",
                code: "025",
            },
            VillageCode {
                name: "山卡拉村民委员会",
                code: "026",
            },
            VillageCode {
                name: "德加村民委员会",
                code: "027",
            },
            VillageCode {
                name: "扎浪滩村民委员会",
                code: "028",
            },
            VillageCode {
                name: "洛忙村民委员会",
                code: "029",
            },
            VillageCode {
                name: "牙什扎村民委员会",
                code: "030",
            },
            VillageCode {
                name: "五道岭村民委员会",
                code: "031",
            },
            VillageCode {
                name: "寺台村民委员会",
                code: "032",
            },
            VillageCode {
                name: "白土庄村民委员会",
                code: "033",
            },
        ],
    },
    TownCode {
        name: "雄先藏族乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "雄先村民委员会",
                code: "001",
            },
            VillageCode {
                name: "麻加村民委员会",
                code: "002",
            },
            VillageCode {
                name: "其大吉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "沙索麻村民委员会",
                code: "004",
            },
            VillageCode {
                name: "其先村民委员会",
                code: "005",
            },
            VillageCode {
                name: "角加村民委员会",
                code: "006",
            },
            VillageCode {
                name: "完加村民委员会",
                code: "007",
            },
            VillageCode {
                name: "电岗村民委员会",
                code: "008",
            },
            VillageCode {
                name: "东由村民委员会",
                code: "009",
            },
            VillageCode {
                name: "唐春村民委员会",
                code: "010",
            },
            VillageCode {
                name: "巴麻塘村民委员会",
                code: "011",
            },
            VillageCode {
                name: "花科村民委员会",
                code: "012",
            },
            VillageCode {
                name: "洛麻村民委员会",
                code: "013",
            },
            VillageCode {
                name: "乙么昂村民委员会",
                code: "014",
            },
            VillageCode {
                name: "下米乃亥村民委员会",
                code: "015",
            },
            VillageCode {
                name: "上米乃亥村民委员会",
                code: "016",
            },
            VillageCode {
                name: "正尕村民委员会",
                code: "017",
            },
            VillageCode {
                name: "拉格堂村民委员会",
                code: "018",
            },
            VillageCode {
                name: "江扎村民委员会",
                code: "019",
            },
            VillageCode {
                name: "街道村民委员会",
                code: "020",
            },
            VillageCode {
                name: "卡阳村民委员会",
                code: "021",
            },
            VillageCode {
                name: "东棚村民委员会",
                code: "022",
            },
            VillageCode {
                name: "主洞村民委员会",
                code: "023",
            },
            VillageCode {
                name: "年办村民委员会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "初麻乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "扎西庄村民委员会",
                code: "001",
            },
            VillageCode {
                name: "安关村民委员会",
                code: "002",
            },
            VillageCode {
                name: "公保吾具村民委员会",
                code: "003",
            },
            VillageCode {
                name: "安具乎村民委员会",
                code: "004",
            },
            VillageCode {
                name: "主庄村民委员会",
                code: "005",
            },
            VillageCode {
                name: "初麻一村民委员会",
                code: "006",
            },
            VillageCode {
                name: "初麻二村民委员会",
                code: "007",
            },
            VillageCode {
                name: "滩果台村民委员会",
                code: "008",
            },
            VillageCode {
                name: "滩果村民委员会",
                code: "009",
            },
            VillageCode {
                name: "拉尕鲁村民委员会",
                code: "010",
            },
            VillageCode {
                name: "沙尔洞村民委员会",
                code: "011",
            },
            VillageCode {
                name: "上恰藏村民委员会",
                code: "012",
            },
            VillageCode {
                name: "下恰藏村民委员会",
                code: "013",
            },
            VillageCode {
                name: "拉许村民委员会",
                code: "014",
            },
            VillageCode {
                name: "沙让村民委员会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "查甫藏族乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "查甫一村民委员会",
                code: "001",
            },
            VillageCode {
                name: "查甫二村民委员会",
                code: "002",
            },
            VillageCode {
                name: "药水泉村民委员会",
                code: "003",
            },
            VillageCode {
                name: "索拉村民委员会",
                code: "004",
            },
            VillageCode {
                name: "东台村民委员会",
                code: "005",
            },
            VillageCode {
                name: "上曲加村民委员会",
                code: "006",
            },
            VillageCode {
                name: "中曲加村民委员会",
                code: "007",
            },
            VillageCode {
                name: "来洞村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下曲加村民委员会",
                code: "009",
            },
            VillageCode {
                name: "查让村民委员会",
                code: "010",
            },
            VillageCode {
                name: "加斜村民委员会",
                code: "011",
            },
            VillageCode {
                name: "跃洞村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "塔加藏族乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "白家集村民委员会",
                code: "001",
            },
            VillageCode {
                name: "塔加二村民委员会",
                code: "002",
            },
            VillageCode {
                name: "塔加一村民委员会",
                code: "003",
            },
            VillageCode {
                name: "尕洞村民委员会",
                code: "004",
            },
            VillageCode {
                name: "德扎村民委员会",
                code: "005",
            },
            VillageCode {
                name: "白家拉卡村民委员会",
                code: "006",
            },
            VillageCode {
                name: "牙什扎村民委员会",
                code: "007",
            },
            VillageCode {
                name: "贡什加村民委员会",
                code: "008",
            },
            VillageCode {
                name: "曹旦麻村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "金源藏族乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "雄哇村民委员会",
                code: "001",
            },
            VillageCode {
                name: "上科巴村民委员会",
                code: "002",
            },
            VillageCode {
                name: "下科巴村民委员会",
                code: "003",
            },
            VillageCode {
                name: "恰加村民委员会",
                code: "004",
            },
            VillageCode {
                name: "旦麻村民委员会",
                code: "005",
            },
            VillageCode {
                name: "土哇仓村民委员会",
                code: "006",
            },
            VillageCode {
                name: "支哈加村民委员会",
                code: "007",
            },
            VillageCode {
                name: "安关雄哇村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下什堂村民委员会",
                code: "009",
            },
            VillageCode {
                name: "多西村民委员会",
                code: "010",
            },
            VillageCode {
                name: "日古村民委员会",
                code: "011",
            },
            VillageCode {
                name: "阿吾卜具村民委员会",
                code: "012",
            },
            VillageCode {
                name: "尖科村民委员会",
                code: "013",
            },
            VillageCode {
                name: "桑加吾具村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "二塘乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "二塘村民委员会",
                code: "001",
            },
            VillageCode {
                name: "工哇滩一村民委员会",
                code: "002",
            },
            VillageCode {
                name: "工哇滩二村民委员会",
                code: "003",
            },
            VillageCode {
                name: "尼昂村民委员会",
                code: "004",
            },
            VillageCode {
                name: "香里胡拉村民委员会",
                code: "005",
            },
            VillageCode {
                name: "上滩村民委员会",
                code: "006",
            },
            VillageCode {
                name: "格许村民委员会",
                code: "007",
            },
            VillageCode {
                name: "角扎村民委员会",
                code: "008",
            },
            VillageCode {
                name: "庄子湾村民委员会",
                code: "009",
            },
            VillageCode {
                name: "红牙合村民委员会",
                code: "010",
            },
            VillageCode {
                name: "隆欠村民委员会",
                code: "011",
            },
            VillageCode {
                name: "尕什加村民委员会",
                code: "012",
            },
            VillageCode {
                name: "二塘沟村民委员会",
                code: "013",
            },
            VillageCode {
                name: "尕吾山村民委员会",
                code: "014",
            },
            VillageCode {
                name: "科却村民委员会",
                code: "015",
            },
            VillageCode {
                name: "大塘村民委员会",
                code: "016",
            },
            VillageCode {
                name: "三塘村民委员会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "谢家滩乡",
        code: "013",
        villages: &[
            VillageCode {
                name: "谢家滩村民委员会",
                code: "001",
            },
            VillageCode {
                name: "工扎村民委员会",
                code: "002",
            },
            VillageCode {
                name: "拉扎村民委员会",
                code: "003",
            },
            VillageCode {
                name: "窑隆村民委员会",
                code: "004",
            },
            VillageCode {
                name: "卡昂村民委员会",
                code: "005",
            },
            VillageCode {
                name: "朱家湾村民委员会",
                code: "006",
            },
            VillageCode {
                name: "九道湾村民委员会",
                code: "007",
            },
            VillageCode {
                name: "吊沟村民委员会",
                code: "008",
            },
            VillageCode {
                name: "丁家湾村民委员会",
                code: "009",
            },
            VillageCode {
                name: "尔多其那村民委员会",
                code: "010",
            },
            VillageCode {
                name: "韩家窑村民委员会",
                code: "011",
            },
            VillageCode {
                name: "牙合村民委员会",
                code: "012",
            },
            VillageCode {
                name: "马塘村民委员会",
                code: "013",
            },
            VillageCode {
                name: "合群村民委员会",
                code: "014",
            },
            VillageCode {
                name: "下河滩村民委员会",
                code: "015",
            },
            VillageCode {
                name: "阴坡村民委员会",
                code: "016",
            },
            VillageCode {
                name: "西门泉村民委员会",
                code: "017",
            },
            VillageCode {
                name: "卷坑村民委员会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "德恒隆乡",
        code: "014",
        villages: &[
            VillageCode {
                name: "德恒隆一村民委员会",
                code: "001",
            },
            VillageCode {
                name: "德恒隆二村民委员会",
                code: "002",
            },
            VillageCode {
                name: "纳加村民委员会",
                code: "003",
            },
            VillageCode {
                name: "支乎具村民委员会",
                code: "004",
            },
            VillageCode {
                name: "措扎村民委员会",
                code: "005",
            },
            VillageCode {
                name: "黄吾具村民委员会",
                code: "006",
            },
            VillageCode {
                name: "拉村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "若索村民委员会",
                code: "008",
            },
            VillageCode {
                name: "哇西村民委员会",
                code: "009",
            },
            VillageCode {
                name: "东加村民委员会",
                code: "010",
            },
            VillageCode {
                name: "列村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "安措村民委员会",
                code: "012",
            },
            VillageCode {
                name: "卡什代村民委员会",
                code: "013",
            },
            VillageCode {
                name: "牙曲村民委员会",
                code: "014",
            },
            VillageCode {
                name: "石乃海村民委员会",
                code: "015",
            },
            VillageCode {
                name: "西后加吾具村民委员会",
                code: "016",
            },
            VillageCode {
                name: "哇加村民委员会",
                code: "017",
            },
            VillageCode {
                name: "甲加村民委员会",
                code: "018",
            },
            VillageCode {
                name: "哇加滩村民委员会",
                code: "019",
            },
            VillageCode {
                name: "牙曲滩村民委员会",
                code: "020",
            },
            VillageCode {
                name: "团结村民委员会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "沙连堡乡",
        code: "015",
        villages: &[
            VillageCode {
                name: "关巴湾村民委员会",
                code: "001",
            },
            VillageCode {
                name: "加仓村民委员会",
                code: "002",
            },
            VillageCode {
                name: "尔洞门村民委员会",
                code: "003",
            },
            VillageCode {
                name: "其后昂村民委员会",
                code: "004",
            },
            VillageCode {
                name: "沙连堡一村民委员会",
                code: "005",
            },
            VillageCode {
                name: "沙连堡二村民委员会",
                code: "006",
            },
            VillageCode {
                name: "科才昂村民委员会",
                code: "007",
            },
            VillageCode {
                name: "上塔加村民委员会",
                code: "008",
            },
            VillageCode {
                name: "下塔加村民委员会",
                code: "009",
            },
            VillageCode {
                name: "乙什春一村民委员会",
                code: "010",
            },
            VillageCode {
                name: "乙什春二村民委员会",
                code: "011",
            },
            VillageCode {
                name: "古浪村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "阿什努乡",
        code: "016",
        villages: &[
            VillageCode {
                name: "阿什努二村民委员会",
                code: "001",
            },
            VillageCode {
                name: "阿什努一村民委员会",
                code: "002",
            },
            VillageCode {
                name: "日芒村民委员会",
                code: "003",
            },
            VillageCode {
                name: "羊隆村民委员会",
                code: "004",
            },
            VillageCode {
                name: "若兰村民委员会",
                code: "005",
            },
            VillageCode {
                name: "阿藏吾具村民委员会",
                code: "006",
            },
            VillageCode {
                name: "松赛村民委员会",
                code: "007",
            },
            VillageCode {
                name: "列仁村民委员会",
                code: "008",
            },
            VillageCode {
                name: "纳哈龙村民委员会",
                code: "009",
            },
            VillageCode {
                name: "尕加村民委员会",
                code: "010",
            },
            VillageCode {
                name: "全吉村民委员会",
                code: "011",
            },
            VillageCode {
                name: "俄加村民委员会",
                code: "012",
            },
            VillageCode {
                name: "白加村民委员会",
                code: "013",
            },
            VillageCode {
                name: "列什洞村民委员会",
                code: "014",
            },
            VillageCode {
                name: "赛什库村民委员会",
                code: "015",
            },
            VillageCode {
                name: "多杰拉卡村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "石大仓乡",
        code: "017",
        villages: &[
            VillageCode {
                name: "石大仓村民委员会",
                code: "001",
            },
            VillageCode {
                name: "台力盖一村民委员会",
                code: "002",
            },
            VillageCode {
                name: "台力盖二村民委员会",
                code: "003",
            },
            VillageCode {
                name: "支哈堂村民委员会",
                code: "004",
            },
            VillageCode {
                name: "项加吾具村民委员会",
                code: "005",
            },
            VillageCode {
                name: "关藏村民委员会",
                code: "006",
            },
            VillageCode {
                name: "大岭村民委员会",
                code: "007",
            },
            VillageCode {
                name: "小金源村民委员会",
                code: "008",
            },
            VillageCode {
                name: "斯吉海村民委员会",
                code: "009",
            },
            VillageCode {
                name: "沙让村民委员会",
                code: "010",
            },
            VillageCode {
                name: "大加沿村民委员会",
                code: "011",
            },
            VillageCode {
                name: "文加山村民委员会",
                code: "012",
            },
            VillageCode {
                name: "旦庄村民委员会",
                code: "013",
            },
            VillageCode {
                name: "吉加村民委员会",
                code: "014",
            },
            VillageCode {
                name: "香塔村民委员会",
                code: "015",
            },
            VillageCode {
                name: "高跃村民委员会",
                code: "016",
            },
            VillageCode {
                name: "拉卡村民委员会",
                code: "017",
            },
        ],
    },
];

static TOWNS_QH_014: [TownCode; 9] = [
    TownCode {
        name: "积石镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "积石镇城东社区",
                code: "001",
            },
            VillageCode {
                name: "积石镇城西社区",
                code: "002",
            },
            VillageCode {
                name: "积石镇城中社区",
                code: "003",
            },
            VillageCode {
                name: "积石镇城北社区",
                code: "004",
            },
            VillageCode {
                name: "下草滩坝村委会",
                code: "005",
            },
            VillageCode {
                name: "西街村委会",
                code: "006",
            },
            VillageCode {
                name: "上草滩坝村委会",
                code: "007",
            },
            VillageCode {
                name: "东街村委会",
                code: "008",
            },
            VillageCode {
                name: "加入村委会",
                code: "009",
            },
            VillageCode {
                name: "瓦匠庄村委会",
                code: "010",
            },
            VillageCode {
                name: "托坝村委会",
                code: "011",
            },
            VillageCode {
                name: "线尕拉村委会",
                code: "012",
            },
            VillageCode {
                name: "沙坝塘村委会",
                code: "013",
            },
            VillageCode {
                name: "石头坡村委会",
                code: "014",
            },
            VillageCode {
                name: "西沟村委会",
                code: "015",
            },
            VillageCode {
                name: "丁江村委会",
                code: "016",
            },
            VillageCode {
                name: "大别列村委会",
                code: "017",
            },
            VillageCode {
                name: "尕别列村委会",
                code: "018",
            },
            VillageCode {
                name: "伊麻目村委会",
                code: "019",
            },
            VillageCode {
                name: "河北村委会",
                code: "020",
            },
            VillageCode {
                name: "新建村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "白庄镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "白庄镇社区",
                code: "001",
            },
            VillageCode {
                name: "下白庄村委会",
                code: "002",
            },
            VillageCode {
                name: "上白庄村委会",
                code: "003",
            },
            VillageCode {
                name: "来塘村委会",
                code: "004",
            },
            VillageCode {
                name: "塘洛尕村委会",
                code: "005",
            },
            VillageCode {
                name: "白庄村委会",
                code: "006",
            },
            VillageCode {
                name: "上张尕村委会",
                code: "007",
            },
            VillageCode {
                name: "下张尕村委会",
                code: "008",
            },
            VillageCode {
                name: "立庄村委会",
                code: "009",
            },
            VillageCode {
                name: "上拉边村委会",
                code: "010",
            },
            VillageCode {
                name: "下拉边村委会",
                code: "011",
            },
            VillageCode {
                name: "山根村委会",
                code: "012",
            },
            VillageCode {
                name: "扎木村委会",
                code: "013",
            },
            VillageCode {
                name: "昌克村委会",
                code: "014",
            },
            VillageCode {
                name: "团结村委会",
                code: "015",
            },
            VillageCode {
                name: "乙日亥村委会",
                code: "016",
            },
            VillageCode {
                name: "米牙亥村委会",
                code: "017",
            },
            VillageCode {
                name: "朱格村委会",
                code: "018",
            },
            VillageCode {
                name: "江布日村委会",
                code: "019",
            },
            VillageCode {
                name: "条井村委会",
                code: "020",
            },
            VillageCode {
                name: "上科哇村委会",
                code: "021",
            },
            VillageCode {
                name: "下科哇村委会",
                code: "022",
            },
            VillageCode {
                name: "苏乎撒村委会",
                code: "023",
            },
            VillageCode {
                name: "麻日村委会",
                code: "024",
            },
            VillageCode {
                name: "牙日村委会",
                code: "025",
            },
            VillageCode {
                name: "格达村委会",
                code: "026",
            },
            VillageCode {
                name: "吾科村委会",
                code: "027",
            },
            VillageCode {
                name: "强宁村委会",
                code: "028",
            },
        ],
    },
    TownCode {
        name: "街子镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "街子镇社区",
                code: "001",
            },
            VillageCode {
                name: "团结村委会",
                code: "002",
            },
            VillageCode {
                name: "托龙都村委会",
                code: "003",
            },
            VillageCode {
                name: "牙门曲乎村委会",
                code: "004",
            },
            VillageCode {
                name: "三兰巴海村委会",
                code: "005",
            },
            VillageCode {
                name: "沈家村委会",
                code: "006",
            },
            VillageCode {
                name: "上坊村委会",
                code: "007",
            },
            VillageCode {
                name: "马家村委会",
                code: "008",
            },
            VillageCode {
                name: "三立方村委会",
                code: "009",
            },
            VillageCode {
                name: "苏哇什村委会",
                code: "010",
            },
            VillageCode {
                name: "洋巴扎村委会",
                code: "011",
            },
            VillageCode {
                name: "波拉亥村委会",
                code: "012",
            },
            VillageCode {
                name: "吾土贝那亥村委会",
                code: "013",
            },
            VillageCode {
                name: "洋苦浪村委会",
                code: "014",
            },
            VillageCode {
                name: "波立吉村委会",
                code: "015",
            },
            VillageCode {
                name: "古吉来村委会",
                code: "016",
            },
            VillageCode {
                name: "塘坊村委会",
                code: "017",
            },
            VillageCode {
                name: "果河拉村委会",
                code: "018",
            },
            VillageCode {
                name: "果什滩村委会",
                code: "019",
            },
            VillageCode {
                name: "孟达山村委会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "道帏藏族乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "古雷村委会",
                code: "001",
            },
            VillageCode {
                name: "起台堡村委会",
                code: "002",
            },
            VillageCode {
                name: "贺龙堡村委会",
                code: "003",
            },
            VillageCode {
                name: "比隆村委会",
                code: "004",
            },
            VillageCode {
                name: "贺塘村委会",
                code: "005",
            },
            VillageCode {
                name: "贺庄村委会",
                code: "006",
            },
            VillageCode {
                name: "宁巴村委会",
                code: "007",
            },
            VillageCode {
                name: "张沙村委会",
                code: "008",
            },
            VillageCode {
                name: "多哇村委会",
                code: "009",
            },
            VillageCode {
                name: "多什则村委会",
                code: "010",
            },
            VillageCode {
                name: "俄家村委会",
                code: "011",
            },
            VillageCode {
                name: "吾曼道村委会",
                code: "012",
            },
            VillageCode {
                name: "拉科村委会",
                code: "013",
            },
            VillageCode {
                name: "拉木龙哇村委会",
                code: "014",
            },
            VillageCode {
                name: "德曼村委会",
                code: "015",
            },
            VillageCode {
                name: "循哇村委会",
                code: "016",
            },
            VillageCode {
                name: "立伦村委会",
                code: "017",
            },
            VillageCode {
                name: "王家村委会",
                code: "018",
            },
            VillageCode {
                name: "三木仓村委会",
                code: "019",
            },
            VillageCode {
                name: "旦麻村委会",
                code: "020",
            },
            VillageCode {
                name: "加仓村委会",
                code: "021",
            },
            VillageCode {
                name: "木洪村委会",
                code: "022",
            },
            VillageCode {
                name: "牙木村委会",
                code: "023",
            },
            VillageCode {
                name: "铁尕楞村委会",
                code: "024",
            },
            VillageCode {
                name: "吾曼村委会",
                code: "025",
            },
            VillageCode {
                name: "克麻村委会",
                code: "026",
            },
            VillageCode {
                name: "夕冲村委会",
                code: "027",
            },
        ],
    },
    TownCode {
        name: "清水乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "石巷村委会",
                code: "001",
            },
            VillageCode {
                name: "下滩村委会",
                code: "002",
            },
            VillageCode {
                name: "田盖村委会",
                code: "003",
            },
            VillageCode {
                name: "阿什匠村委会",
                code: "004",
            },
            VillageCode {
                name: "乙麻亥村委会",
                code: "005",
            },
            VillageCode {
                name: "上庄村委会",
                code: "006",
            },
            VillageCode {
                name: "下庄村委会",
                code: "007",
            },
            VillageCode {
                name: "阿么叉村委会",
                code: "008",
            },
            VillageCode {
                name: "红庄村委会",
                code: "009",
            },
            VillageCode {
                name: "大寺古村委会",
                code: "010",
            },
            VillageCode {
                name: "瓦匠庄村委会",
                code: "011",
            },
            VillageCode {
                name: "唐才村委会",
                code: "012",
            },
            VillageCode {
                name: "大庄村委会",
                code: "013",
            },
            VillageCode {
                name: "专堂村委会",
                code: "014",
            },
            VillageCode {
                name: "塔沙坡村委会",
                code: "015",
            },
            VillageCode {
                name: "木厂村委会",
                code: "016",
            },
            VillageCode {
                name: "索同村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "岗察藏族乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "岗察村委会",
                code: "001",
            },
            VillageCode {
                name: "卡索村委会",
                code: "002",
            },
            VillageCode {
                name: "苏化村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "查汗都斯乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "下庄村委会",
                code: "001",
            },
            VillageCode {
                name: "牙藏村委会",
                code: "002",
            },
            VillageCode {
                name: "哈大亥村委会",
                code: "003",
            },
            VillageCode {
                name: "苏只村委会",
                code: "004",
            },
            VillageCode {
                name: "乙麻亥村委会",
                code: "005",
            },
            VillageCode {
                name: "新村村委会",
                code: "006",
            },
            VillageCode {
                name: "团结村村委会",
                code: "007",
            },
            VillageCode {
                name: "白庄村委会",
                code: "008",
            },
            VillageCode {
                name: "阿河滩村委会",
                code: "009",
            },
            VillageCode {
                name: "中庄村委会",
                code: "010",
            },
            VillageCode {
                name: "大庄村委会",
                code: "011",
            },
            VillageCode {
                name: "新建村委会",
                code: "012",
            },
            VillageCode {
                name: "红光上村村委会",
                code: "013",
            },
            VillageCode {
                name: "红光下村村委会",
                code: "014",
            },
            VillageCode {
                name: "赞卜乎村村委会",
                code: "015",
            },
            VillageCode {
                name: "古什群村委会",
                code: "016",
            },
            VillageCode {
                name: "繁殖场村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "文都藏族乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "拉兄村委会",
                code: "001",
            },
            VillageCode {
                name: "相玉村委会",
                code: "002",
            },
            VillageCode {
                name: "白草毛村委会",
                code: "003",
            },
            VillageCode {
                name: "拉代村委会",
                code: "004",
            },
            VillageCode {
                name: "旦麻村委会",
                code: "005",
            },
            VillageCode {
                name: "毛玉村委会",
                code: "006",
            },
            VillageCode {
                name: "合哇村委会",
                code: "007",
            },
            VillageCode {
                name: "王仓麻村委会",
                code: "008",
            },
            VillageCode {
                name: "抽子村委会",
                code: "009",
            },
            VillageCode {
                name: "拉龙哇村委会",
                code: "010",
            },
            VillageCode {
                name: "修藏村委会",
                code: "011",
            },
            VillageCode {
                name: "公麻村委会",
                code: "012",
            },
            VillageCode {
                name: "牙训村委会",
                code: "013",
            },
            VillageCode {
                name: "江甲村委会",
                code: "014",
            },
            VillageCode {
                name: "日茫村委会",
                code: "015",
            },
            VillageCode {
                name: "哇库村委会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "尕楞藏族乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "牙尕村委会",
                code: "001",
            },
            VillageCode {
                name: "麻尕村委会",
                code: "002",
            },
            VillageCode {
                name: "香沙村委会",
                code: "003",
            },
            VillageCode {
                name: "洛哇村委会",
                code: "004",
            },
            VillageCode {
                name: "哇龙村委会",
                code: "005",
            },
            VillageCode {
                name: "比塘村委会",
                code: "006",
            },
            VillageCode {
                name: "建设堂村委会",
                code: "007",
            },
            VillageCode {
                name: "曲卜藏村委会",
                code: "008",
            },
            VillageCode {
                name: "秀日村委会",
                code: "009",
            },
            VillageCode {
                name: "仁务村委会",
                code: "010",
            },
            VillageCode {
                name: "宗占村委会",
                code: "011",
            },
        ],
    },
];

static TOWNS_QH_015: [TownCode; 14] = [
    TownCode {
        name: "浩门镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "康乐路居委会",
                code: "001",
            },
            VillageCode {
                name: "花园路居委会",
                code: "002",
            },
            VillageCode {
                name: "向阳路居委会",
                code: "003",
            },
            VillageCode {
                name: "气象路居委会",
                code: "004",
            },
            VillageCode {
                name: "北关村委会",
                code: "005",
            },
            VillageCode {
                name: "南关村委会",
                code: "006",
            },
            VillageCode {
                name: "团结村委会",
                code: "007",
            },
            VillageCode {
                name: "西关村委会",
                code: "008",
            },
            VillageCode {
                name: "二道崖湾村委会",
                code: "009",
            },
            VillageCode {
                name: "头塘村委会",
                code: "010",
            },
            VillageCode {
                name: "疙瘩村委会",
                code: "011",
            },
            VillageCode {
                name: "煤窑沟村委会",
                code: "012",
            },
            VillageCode {
                name: "小沙沟村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "青石咀镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "吊沟路居委会",
                code: "001",
            },
            VillageCode {
                name: "宁张路居委会",
                code: "002",
            },
            VillageCode {
                name: "马场社区居委会",
                code: "003",
            },
            VillageCode {
                name: "青石咀村委会",
                code: "004",
            },
            VillageCode {
                name: "石头沟村委会",
                code: "005",
            },
            VillageCode {
                name: "黑石头村委会",
                code: "006",
            },
            VillageCode {
                name: "红山咀村委会",
                code: "007",
            },
            VillageCode {
                name: "红牙河村委会",
                code: "008",
            },
            VillageCode {
                name: "德庆营村委会",
                code: "009",
            },
            VillageCode {
                name: "下吊沟村委会",
                code: "010",
            },
            VillageCode {
                name: "上吊沟村委会",
                code: "011",
            },
            VillageCode {
                name: "上铁迈村委会",
                code: "012",
            },
            VillageCode {
                name: "尕大滩村委会",
                code: "013",
            },
            VillageCode {
                name: "白土沟村委会",
                code: "014",
            },
            VillageCode {
                name: "下大滩村委会",
                code: "015",
            },
            VillageCode {
                name: "大滩村委会",
                code: "016",
            },
            VillageCode {
                name: "东铁迈村委会",
                code: "017",
            },
            VillageCode {
                name: "西铁迈村委会",
                code: "018",
            },
            VillageCode {
                name: "红沟村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "泉口镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "旱台村委会",
                code: "001",
            },
            VillageCode {
                name: "东沙河村委会",
                code: "002",
            },
            VillageCode {
                name: "大庄村委会",
                code: "003",
            },
            VillageCode {
                name: "花崖村委会",
                code: "004",
            },
            VillageCode {
                name: "黄田村委会",
                code: "005",
            },
            VillageCode {
                name: "西沙河村委会",
                code: "006",
            },
            VillageCode {
                name: "牙合村委会",
                code: "007",
            },
            VillageCode {
                name: "腰巴村委会",
                code: "008",
            },
            VillageCode {
                name: "大湾村委会",
                code: "009",
            },
            VillageCode {
                name: "多麻滩村委会",
                code: "010",
            },
            VillageCode {
                name: "俄堡沟村委会",
                code: "011",
            },
            VillageCode {
                name: "后沟村委会",
                code: "012",
            },
            VillageCode {
                name: "黄树湾村委会",
                code: "013",
            },
            VillageCode {
                name: "泉沟台村委会",
                code: "014",
            },
            VillageCode {
                name: "沈家湾村委会",
                code: "015",
            },
            VillageCode {
                name: "西河坝村委会",
                code: "016",
            },
            VillageCode {
                name: "窑洞庄村委会",
                code: "017",
            },
            VillageCode {
                name: "中滩村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "东川镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "孔家庄社区居委会",
                code: "001",
            },
            VillageCode {
                name: "孔家庄村委会",
                code: "002",
            },
            VillageCode {
                name: "碱沟村委会",
                code: "003",
            },
            VillageCode {
                name: "塔龙滩村委会",
                code: "004",
            },
            VillageCode {
                name: "尕牧龙上村村委会",
                code: "005",
            },
            VillageCode {
                name: "尕牧龙中村村委会",
                code: "006",
            },
            VillageCode {
                name: "尕牧龙下村村委会",
                code: "007",
            },
            VillageCode {
                name: "巴哈村委会",
                code: "008",
            },
            VillageCode {
                name: "甘沟村委会",
                code: "009",
            },
            VillageCode {
                name: "麻当村委会",
                code: "010",
            },
            VillageCode {
                name: "却藏村委会",
                code: "011",
            },
            VillageCode {
                name: "寺尔沟村委会",
                code: "012",
            },
            VillageCode {
                name: "香卡村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "北山乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "北山根村委会",
                code: "001",
            },
            VillageCode {
                name: "大泉村委会",
                code: "002",
            },
            VillageCode {
                name: "沙沟梁村委会",
                code: "003",
            },
            VillageCode {
                name: "沙沟脑村委会",
                code: "004",
            },
            VillageCode {
                name: "上金巴台村委会",
                code: "005",
            },
            VillageCode {
                name: "下金巴台村委会",
                code: "006",
            },
            VillageCode {
                name: "东滩村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "麻莲乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "中麻莲村委会",
                code: "001",
            },
            VillageCode {
                name: "包哈图村委会",
                code: "002",
            },
            VillageCode {
                name: "葱花滩村委会",
                code: "003",
            },
            VillageCode {
                name: "瓜拉村委会",
                code: "004",
            },
            VillageCode {
                name: "下麻莲村委会",
                code: "005",
            },
            VillageCode {
                name: "白崖沟村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "西滩乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "东马场村委会",
                code: "001",
            },
            VillageCode {
                name: "边麻掌村委会",
                code: "002",
            },
            VillageCode {
                name: "簸箕湾村委会",
                code: "003",
            },
            VillageCode {
                name: "东山村委会",
                code: "004",
            },
            VillageCode {
                name: "老龙湾村委会",
                code: "005",
            },
            VillageCode {
                name: "纳隆村委会",
                code: "006",
            },
            VillageCode {
                name: "上西滩村委会",
                code: "007",
            },
            VillageCode {
                name: "下西滩村委会",
                code: "008",
            },
            VillageCode {
                name: "西马场村委会",
                code: "009",
            },
            VillageCode {
                name: "崖头村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "阴田乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "上阴田村委会",
                code: "001",
            },
            VillageCode {
                name: "大沟口村委会",
                code: "002",
            },
            VillageCode {
                name: "大沟脑村委会",
                code: "003",
            },
            VillageCode {
                name: "卡子沟村委会",
                code: "004",
            },
            VillageCode {
                name: "米麻隆村委会",
                code: "005",
            },
            VillageCode {
                name: "措隆滩村委会",
                code: "006",
            },
            VillageCode {
                name: "下阴田村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "仙米乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "大庄村委会",
                code: "001",
            },
            VillageCode {
                name: "达隆村委会",
                code: "002",
            },
            VillageCode {
                name: "德欠村委会",
                code: "003",
            },
            VillageCode {
                name: "龙浪村委会",
                code: "004",
            },
            VillageCode {
                name: "梅花村委会",
                code: "005",
            },
            VillageCode {
                name: "桥滩村委会",
                code: "006",
            },
            VillageCode {
                name: "塔里华村委会",
                code: "007",
            },
            VillageCode {
                name: "讨拉村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "珠固乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "玉龙滩村委会",
                code: "001",
            },
            VillageCode {
                name: "东旭村委会",
                code: "002",
            },
            VillageCode {
                name: "雪龙滩村委会",
                code: "003",
            },
            VillageCode {
                name: "初麻院村委会",
                code: "004",
            },
            VillageCode {
                name: "珠固寺村委会",
                code: "005",
            },
            VillageCode {
                name: "元树村委会",
                code: "006",
            },
            VillageCode {
                name: "德宗村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "苏吉滩乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "察汉达吾村委会",
                code: "001",
            },
            VillageCode {
                name: "苏吉湾村委会",
                code: "002",
            },
            VillageCode {
                name: "燕麦图呼村委会",
                code: "003",
            },
            VillageCode {
                name: "药草梁村委会",
                code: "004",
            },
            VillageCode {
                name: "扎麻图村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "皇城蒙古族乡",
        code: "012",
        villages: &[
            VillageCode {
                name: "马营村委会",
                code: "001",
            },
            VillageCode {
                name: "东滩村委会",
                code: "002",
            },
            VillageCode {
                name: "北山村委会",
                code: "003",
            },
            VillageCode {
                name: "西滩村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "门源监狱",
        code: "013",
        villages: &[
            VillageCode {
                name: "场部社区",
                code: "001",
            },
            VillageCode {
                name: "一队",
                code: "002",
            },
            VillageCode {
                name: "三队",
                code: "003",
            },
            VillageCode {
                name: "六队",
                code: "004",
            },
            VillageCode {
                name: "七队",
                code: "005",
            },
            VillageCode {
                name: "八队",
                code: "006",
            },
            VillageCode {
                name: "九队",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "门源种马场",
        code: "014",
        villages: &[
            VillageCode {
                name: "场部生活区",
                code: "001",
            },
            VillageCode {
                name: "乌兰生活区",
                code: "002",
            },
            VillageCode {
                name: "永安城生活区",
                code: "003",
            },
            VillageCode {
                name: "四牙合生活区",
                code: "004",
            },
            VillageCode {
                name: "药草梁生活区",
                code: "005",
            },
        ],
    },
];

static TOWNS_QH_016: [TownCode; 7] = [
    TownCode {
        name: "八宝镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城东社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "城西社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "新城社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "拉洞新型社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "西村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "东措台村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "东村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "白杨沟村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "宝瓶河村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "冰沟村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "高楞村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "黄藏寺村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "卡力岗村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "拉洞村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "拉洞台村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "麻拉河村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "白土垭豁村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "下庄村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "营盘台村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "夹木村村民委员会",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "峨堡镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "峨堡村牧民委员会",
                code: "001",
            },
            VillageCode {
                name: "白石崖村牧民委员会",
                code: "002",
            },
            VillageCode {
                name: "黄草沟村牧民委员会",
                code: "003",
            },
            VillageCode {
                name: "芒扎村牧民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "默勒镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "老日根村牧民委员会",
                code: "001",
            },
            VillageCode {
                name: "才什土村牧民委员会",
                code: "002",
            },
            VillageCode {
                name: "瓦日尕村牧民委员会",
                code: "003",
            },
            VillageCode {
                name: "多隆村牧民委员会",
                code: "004",
            },
            VillageCode {
                name: "海浪村牧民委员会",
                code: "005",
            },
            VillageCode {
                name: "扎沙村牧民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "扎麻什乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "鸽子洞村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "地盘子村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "郭米村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "河北村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "河东村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "河西村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "棉沙湾村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "夏塘村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "阿柔乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "草大坂村牧民委员会",
                code: "001",
            },
            VillageCode {
                name: "青阳沟村牧民委员会",
                code: "002",
            },
            VillageCode {
                name: "日旭村牧民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "野牛沟乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "边麻村牧民委员会",
                code: "001",
            },
            VillageCode {
                name: "大泉村牧民委员会",
                code: "002",
            },
            VillageCode {
                name: "大浪村牧民委员会",
                code: "003",
            },
            VillageCode {
                name: "达玉村牧民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "央隆乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "央隆社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "托勒村牧民委员会",
                code: "002",
            },
            VillageCode {
                name: "夏尔格村牧民委员会",
                code: "003",
            },
            VillageCode {
                name: "曲库村牧民委员会",
                code: "004",
            },
            VillageCode {
                name: "阿尔格村牧民委员会",
                code: "005",
            },
        ],
    },
];

static TOWNS_QH_017: [TownCode; 6] = [
    TownCode {
        name: "三角城镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "三角城社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "和平社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "海湖社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "海峰村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "黄草掌村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "三角城村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "西岔村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "三联村村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "西海镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "城南社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "城北社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "城东社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "城西社区居民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "金滩乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "岳峰村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "金滩村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "东达村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "海东村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "姜柳盛村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "新泉村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "光明村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "道阳村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "仓开村村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "哈勒景蒙古族乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "哈勒景村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "永丰村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "乌兰哈达村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "青海湖乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "达玉五谷村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "同宝村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "达玉日秀村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "达玉德吉村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "塔列村村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "甘子河乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "达玉村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "尕海村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "俄日村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "热水村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "托华村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "德州村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "温都村村民委员会",
                code: "007",
            },
        ],
    },
];

static TOWNS_QH_018: [TownCode; 5] = [
    TownCode {
        name: "沙柳河镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城东社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "城南社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "城西社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "城北社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "三角城种羊场社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "红山村委会",
                code: "006",
            },
            VillageCode {
                name: "尕曲村委会",
                code: "007",
            },
            VillageCode {
                name: "潘保村委会",
                code: "008",
            },
            VillageCode {
                name: "恩乃村委会",
                code: "009",
            },
            VillageCode {
                name: "新海村委会",
                code: "010",
            },
            VillageCode {
                name: "果洛藏贡麻村委会",
                code: "011",
            },
            VillageCode {
                name: "河东村委会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "哈尔盖镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "哈尔盖镇社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "公贡麻村委会",
                code: "002",
            },
            VillageCode {
                name: "果洛藏秀麻村委会",
                code: "003",
            },
            VillageCode {
                name: "察拉村委会",
                code: "004",
            },
            VillageCode {
                name: "切察村委会",
                code: "005",
            },
            VillageCode {
                name: "环仓秀麻村委会",
                code: "006",
            },
            VillageCode {
                name: "亚秀麻村委会",
                code: "007",
            },
            VillageCode {
                name: "塘渠村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "伊克乌兰乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "青海湖农场社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "刚察贡麻牧委会",
                code: "002",
            },
            VillageCode {
                name: "角什科贡麻牧委会",
                code: "003",
            },
            VillageCode {
                name: "亚秀牧委会",
                code: "004",
            },
            VillageCode {
                name: "尚木多牧委会",
                code: "005",
            },
            VillageCode {
                name: "亚贡麻牧委会",
                code: "006",
            },
            VillageCode {
                name: "角什科秀麻牧委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "泉吉乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "泉吉乡社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "黄玉农场社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "新泉村委会",
                code: "003",
            },
            VillageCode {
                name: "扎苏合村委会",
                code: "004",
            },
            VillageCode {
                name: "宁夏村委会",
                code: "005",
            },
            VillageCode {
                name: "冶合茂村委会",
                code: "006",
            },
            VillageCode {
                name: "切吉村委会",
                code: "007",
            },
            VillageCode {
                name: "年乃索麻村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "吉尔孟乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "环仓贡麻村委会",
                code: "001",
            },
            VillageCode {
                name: "日芒村委会",
                code: "002",
            },
            VillageCode {
                name: "秀脑休麻村委会",
                code: "003",
            },
            VillageCode {
                name: "秀脑贡麻村委会",
                code: "004",
            },
            VillageCode {
                name: "向阳村委会",
                code: "005",
            },
        ],
    },
];

static TOWNS_QH_019: [TownCode; 11] = [
    TownCode {
        name: "隆务镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "隆务街社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "热贡路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "四合吉社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "河东路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "青年路社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "城南社区",
                code: "006",
            },
            VillageCode {
                name: "霍尔加社区",
                code: "007",
            },
            VillageCode {
                name: "隆务村委会",
                code: "008",
            },
            VillageCode {
                name: "吾屯下庄村委会",
                code: "009",
            },
            VillageCode {
                name: "吾屯上庄村委会",
                code: "010",
            },
            VillageCode {
                name: "加查么村委会",
                code: "011",
            },
            VillageCode {
                name: "加毛村委会",
                code: "012",
            },
            VillageCode {
                name: "措玉村委会",
                code: "013",
            },
            VillageCode {
                name: "向朝阳村委会",
                code: "014",
            },
            VillageCode {
                name: "牙浪村委会",
                code: "015",
            },
            VillageCode {
                name: "依里村委会",
                code: "016",
            },
            VillageCode {
                name: "阿宁村委会",
                code: "017",
            },
            VillageCode {
                name: "娘洛村委会",
                code: "018",
            },
        ],
    },
    TownCode {
        name: "保安镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "保安社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "城外村委会",
                code: "002",
            },
            VillageCode {
                name: "城内村委会",
                code: "003",
            },
            VillageCode {
                name: "全都村委会",
                code: "004",
            },
            VillageCode {
                name: "双处村委会",
                code: "005",
            },
            VillageCode {
                name: "下庄村委会",
                code: "006",
            },
            VillageCode {
                name: "尕队村委会",
                code: "007",
            },
            VillageCode {
                name: "新城村委会",
                code: "008",
            },
            VillageCode {
                name: "石哈龙村委会",
                code: "009",
            },
            VillageCode {
                name: "群吾村委会",
                code: "010",
            },
            VillageCode {
                name: "浪加村委会",
                code: "011",
            },
            VillageCode {
                name: "东干木村委会",
                code: "012",
            },
            VillageCode {
                name: "银扎木村委会",
                code: "013",
            },
            VillageCode {
                name: "赛加村委会",
                code: "014",
            },
            VillageCode {
                name: "卡加村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "多哇镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "南塔哇社区",
                code: "001",
            },
            VillageCode {
                name: "北塔哇社区",
                code: "002",
            },
            VillageCode {
                name: "直跃村委会",
                code: "003",
            },
            VillageCode {
                name: "尖德村委会",
                code: "004",
            },
            VillageCode {
                name: "交隆务村委会",
                code: "005",
            },
            VillageCode {
                name: "卡什加村委会",
                code: "006",
            },
            VillageCode {
                name: "曲日那村委会",
                code: "007",
            },
            VillageCode {
                name: "东维村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "兰采乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "兰采村委会",
                code: "001",
            },
            VillageCode {
                name: "土房村委会",
                code: "002",
            },
            VillageCode {
                name: "环却乎村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "双朋西乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "双朋西村委会",
                code: "001",
            },
            VillageCode {
                name: "环主村委会",
                code: "002",
            },
            VillageCode {
                name: "协智村委会",
                code: "003",
            },
            VillageCode {
                name: "宁他村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "扎毛乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "扎毛村委会",
                code: "001",
            },
            VillageCode {
                name: "卡苏乎牧委会",
                code: "002",
            },
            VillageCode {
                name: "和日村委会",
                code: "003",
            },
            VillageCode {
                name: "立仓村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "黄乃亥乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "日秀玛村委会",
                code: "001",
            },
            VillageCode {
                name: "群吾村委会",
                code: "002",
            },
            VillageCode {
                name: "阿吾乎村委会",
                code: "003",
            },
            VillageCode {
                name: "羊直沟村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "曲库乎乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "多哇村委会",
                code: "001",
            },
            VillageCode {
                name: "古德村委会",
                code: "002",
            },
            VillageCode {
                name: "瓜什则村委会",
                code: "003",
            },
            VillageCode {
                name: "江龙农业村委会",
                code: "004",
            },
            VillageCode {
                name: "江龙牧业村牧委会",
                code: "005",
            },
            VillageCode {
                name: "索乃亥村委会",
                code: "006",
            },
            VillageCode {
                name: "江什加村委会",
                code: "007",
            },
            VillageCode {
                name: "木合沙村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "年都乎乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "年都乎村委会",
                code: "001",
            },
            VillageCode {
                name: "录合相村委会",
                code: "002",
            },
            VillageCode {
                name: "郭么日村委会",
                code: "003",
            },
            VillageCode {
                name: "曲么村委会",
                code: "004",
            },
            VillageCode {
                name: "夏卜浪村委会",
                code: "005",
            },
            VillageCode {
                name: "尕沙日村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "瓜什则乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "阿旦村委会",
                code: "001",
            },
            VillageCode {
                name: "郭进村委会",
                code: "002",
            },
            VillageCode {
                name: "尕什加村委会",
                code: "003",
            },
            VillageCode {
                name: "西合来村委会",
                code: "004",
            },
            VillageCode {
                name: "力吉村委会",
                code: "005",
            },
            VillageCode {
                name: "赛庆村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "加吾乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "协治村委会",
                code: "001",
            },
            VillageCode {
                name: "俄毛村委会",
                code: "002",
            },
            VillageCode {
                name: "吉仓村委会",
                code: "003",
            },
            VillageCode {
                name: "加吾岗村委会",
                code: "004",
            },
            VillageCode {
                name: "江日村委会",
                code: "005",
            },
            VillageCode {
                name: "东维村委会",
                code: "006",
            },
        ],
    },
];

static TOWNS_QH_020: [TownCode; 9] = [
    TownCode {
        name: "马克堂镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "黄河路居委会",
                code: "001",
            },
            VillageCode {
                name: "申宝路居委会",
                code: "002",
            },
            VillageCode {
                name: "噶丹林社区居委会",
                code: "003",
            },
            VillageCode {
                name: "尖扎滩移民社区居委会",
                code: "004",
            },
            VillageCode {
                name: "智合滩移民社区居委会",
                code: "005",
            },
            VillageCode {
                name: "滨河社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "马克唐村委会",
                code: "007",
            },
            VillageCode {
                name: "麦什扎村委会",
                code: "008",
            },
            VillageCode {
                name: "勒见村委会",
                code: "009",
            },
            VillageCode {
                name: "解放村委会",
                code: "010",
            },
            VillageCode {
                name: "娘毛龙哇村委会",
                code: "011",
            },
            VillageCode {
                name: "回民村委会",
                code: "012",
            },
            VillageCode {
                name: "加让村委会",
                code: "013",
            },
            VillageCode {
                name: "洛科村委会",
                code: "014",
            },
            VillageCode {
                name: "要其村委会",
                code: "015",
            },
            VillageCode {
                name: "如什其村委会",
                code: "016",
            },
            VillageCode {
                name: "科沙唐村委会",
                code: "017",
            },
            VillageCode {
                name: "李加村委会",
                code: "018",
            },
            VillageCode {
                name: "娘毛村委会",
                code: "019",
            },
            VillageCode {
                name: "夏藏滩五村",
                code: "020",
            },
        ],
    },
    TownCode {
        name: "康扬镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "桥头居委会",
                code: "001",
            },
            VillageCode {
                name: "益民社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "城上村委会",
                code: "003",
            },
            VillageCode {
                name: "崖湾村委会",
                code: "004",
            },
            VillageCode {
                name: "巷道村委会",
                code: "005",
            },
            VillageCode {
                name: "宗子拉村委会",
                code: "006",
            },
            VillageCode {
                name: "沙力木村委会",
                code: "007",
            },
            VillageCode {
                name: "尕么堂村委会",
                code: "008",
            },
            VillageCode {
                name: "上庄村委会",
                code: "009",
            },
            VillageCode {
                name: "西麻拉村委会",
                code: "010",
            },
            VillageCode {
                name: "东门村委会",
                code: "011",
            },
            VillageCode {
                name: "寺门村委会",
                code: "012",
            },
            VillageCode {
                name: "河滩村委会",
                code: "013",
            },
            VillageCode {
                name: "格曲村委会",
                code: "014",
            },
            VillageCode {
                name: "夏藏滩六村",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "坎布拉镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "牛滩居委会",
                code: "001",
            },
            VillageCode {
                name: "俄家台居委会",
                code: "002",
            },
            VillageCode {
                name: "吉利村委会",
                code: "003",
            },
            VillageCode {
                name: "完吉合村委会",
                code: "004",
            },
            VillageCode {
                name: "德洪村委会",
                code: "005",
            },
            VillageCode {
                name: "尕吾昂村委会",
                code: "006",
            },
            VillageCode {
                name: "尖藏村委会",
                code: "007",
            },
            VillageCode {
                name: "拉夫旦村委会",
                code: "008",
            },
            VillageCode {
                name: "浪哇村委会",
                code: "009",
            },
            VillageCode {
                name: "坎加村委会",
                code: "010",
            },
            VillageCode {
                name: "满岗村委会",
                code: "011",
            },
            VillageCode {
                name: "茨卡村委会",
                code: "012",
            },
            VillageCode {
                name: "哈玉村委会",
                code: "013",
            },
            VillageCode {
                name: "直岗拉卡村委会",
                code: "014",
            },
            VillageCode {
                name: "上李家村委会",
                code: "015",
            },
            VillageCode {
                name: "下李家村委会",
                code: "016",
            },
            VillageCode {
                name: "尕布村委会",
                code: "017",
            },
            VillageCode {
                name: "仁才村委会",
                code: "018",
            },
            VillageCode {
                name: "俄家村委会",
                code: "019",
            },
            VillageCode {
                name: "香哇东村委会",
                code: "020",
            },
            VillageCode {
                name: "拉群村委会",
                code: "021",
            },
            VillageCode {
                name: "石毛村委会",
                code: "022",
            },
            VillageCode {
                name: "古日羊麻村委会",
                code: "023",
            },
        ],
    },
    TownCode {
        name: "贾加乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "贾加村委会",
                code: "001",
            },
            VillageCode {
                name: "安中村委会",
                code: "002",
            },
            VillageCode {
                name: "南当村委会",
                code: "003",
            },
            VillageCode {
                name: "羊来村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "措周乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "俄什加村委会",
                code: "001",
            },
            VillageCode {
                name: "措干口村委会",
                code: "002",
            },
            VillageCode {
                name: "洛哇村委会",
                code: "003",
            },
            VillageCode {
                name: "措香村委会",
                code: "004",
            },
            VillageCode {
                name: "石乃亥村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "昂拉乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "尖巴昂村委会",
                code: "001",
            },
            VillageCode {
                name: "牙那东村委会",
                code: "002",
            },
            VillageCode {
                name: "措加村委会",
                code: "003",
            },
            VillageCode {
                name: "东加村委会",
                code: "004",
            },
            VillageCode {
                name: "拉毛村委会",
                code: "005",
            },
            VillageCode {
                name: "如什其村委会",
                code: "006",
            },
            VillageCode {
                name: "德吉村委会",
                code: "007",
            },
            VillageCode {
                name: "夏藏滩一村",
                code: "008",
            },
            VillageCode {
                name: "夏藏滩二村",
                code: "009",
            },
            VillageCode {
                name: "夏藏滩三村",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "能科乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "德乾村委会",
                code: "001",
            },
            VillageCode {
                name: "拉萨村委会",
                code: "002",
            },
            VillageCode {
                name: "子哈贡村委会",
                code: "003",
            },
            VillageCode {
                name: "下扎村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "当顺乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "古浪堤村委会",
                code: "001",
            },
            VillageCode {
                name: "香干村委会",
                code: "002",
            },
            VillageCode {
                name: "东当村委会",
                code: "003",
            },
            VillageCode {
                name: "古浪河村委会",
                code: "004",
            },
            VillageCode {
                name: "东果村委会",
                code: "005",
            },
            VillageCode {
                name: "古什当村委会",
                code: "006",
            },
            VillageCode {
                name: "才龙村委会",
                code: "007",
            },
            VillageCode {
                name: "拉德村委会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "尖扎滩乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "萨尕尼哈社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "羊智牧委会",
                code: "002",
            },
            VillageCode {
                name: "岗毛牧委会",
                code: "003",
            },
            VillageCode {
                name: "来玉牧委会",
                code: "004",
            },
            VillageCode {
                name: "石乃亥牧委会",
                code: "005",
            },
            VillageCode {
                name: "洛哇牧委会",
                code: "006",
            },
            VillageCode {
                name: "幸福村牧委会",
                code: "007",
            },
            VillageCode {
                name: "五星村牧委会",
                code: "008",
            },
            VillageCode {
                name: "夏藏滩四村",
                code: "009",
            },
        ],
    },
];

static TOWNS_QH_021: [TownCode; 7] = [
    TownCode {
        name: "泽曲镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "幸福路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "本溪路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "迎宾路社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "民主路社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "泽雄路社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "泽库县恰科日乡金滩社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "东格尔社区居民委员会",
                code: "007",
            },
            VillageCode {
                name: "夏德日村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "东科日村民委员会",
                code: "009",
            },
            VillageCode {
                name: "俄果村民委员会",
                code: "010",
            },
            VillageCode {
                name: "夸日龙村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "巴什则村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "羊玛日村民委员会",
                code: "013",
            },
            VillageCode {
                name: "泽雄村民委员会",
                code: "014",
            },
            VillageCode {
                name: "热旭日村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "措日更村村民委员会",
                code: "016",
            },
            VillageCode {
                name: "而尖村村民委员会",
                code: "017",
            },
            VillageCode {
                name: "果什则村村民委员会",
                code: "018",
            },
            VillageCode {
                name: "角乎村村民委员会",
                code: "019",
            },
            VillageCode {
                name: "雄让村民委员会",
                code: "020",
            },
            VillageCode {
                name: "智合龙村村民委员会",
                code: "021",
            },
            VillageCode {
                name: "尕贡村民委员会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "麦秀镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "多福顿社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "赛龙村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "贡青村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "哈藏村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "龙藏村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "多龙村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "尕让村村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "和日镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "和日社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "巴滩社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "和日村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "东科日村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "环科日村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "吉龙村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "司么村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "唐德村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "夏拉村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "羊旗村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "叶贡村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "直干木村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "智合茂村村民委员会",
                code: "013",
            },
            VillageCode {
                name: "尕叶合村村民委员会",
                code: "014",
            },
            VillageCode {
                name: "秀恰村村民委员会",
                code: "015",
            },
            VillageCode {
                name: "亚日齐村村民委员会",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "宁秀镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "红城村民委会",
                code: "001",
            },
            VillageCode {
                name: "禾角日村民委员会",
                code: "002",
            },
            VillageCode {
                name: "措夫顿村民委员会",
                code: "003",
            },
            VillageCode {
                name: "宁秀村民委员会",
                code: "004",
            },
            VillageCode {
                name: "热旭日村民委员会",
                code: "005",
            },
            VillageCode {
                name: "仁宗村民委员会",
                code: "006",
            },
            VillageCode {
                name: "赛日龙村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "赛日庆村民委员会",
                code: "008",
            },
            VillageCode {
                name: "塞旺村民委员会",
                code: "009",
            },
            VillageCode {
                name: "秀恰村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "智格日村民委员会",
                code: "011",
            },
            VillageCode {
                name: "智赛村村委会",
                code: "012",
            },
            VillageCode {
                name: "尕强村民委员会",
                code: "013",
            },
            VillageCode {
                name: "尕日当村民委员会",
                code: "014",
            },
            VillageCode {
                name: "拉格日村民委员会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "王加乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "团结村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "旗龙村民委员会",
                code: "002",
            },
            VillageCode {
                name: "红旗村民委员会",
                code: "003",
            },
            VillageCode {
                name: "叶金木村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "西卜沙乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "团结村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "红旗村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "跃进村村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "多禾茂乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "多禾日村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "达格日村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "加仓村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "克宁村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "曲玛日村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "塔士乎村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "秀恰村村民委员会",
                code: "007",
            },
        ],
    },
];

static TOWNS_QH_022: [TownCode; 6] = [
    TownCode {
        name: "优干宁镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "赛日旦社区",
                code: "001",
            },
            VillageCode {
                name: "延巴社区",
                code: "002",
            },
            VillageCode {
                name: "江源社区",
                code: "003",
            },
            VillageCode {
                name: "河曲社区",
                code: "004",
            },
            VillageCode {
                name: "智后茂牧委会",
                code: "005",
            },
            VillageCode {
                name: "荷日恒牧委会",
                code: "006",
            },
            VillageCode {
                name: "德日隆牧委会",
                code: "007",
            },
            VillageCode {
                name: "泽雄牧委会",
                code: "008",
            },
            VillageCode {
                name: "秀甲牧委会",
                code: "009",
            },
            VillageCode {
                name: "参美牧委会",
                code: "010",
            },
            VillageCode {
                name: "阿木乎牧委会",
                code: "011",
            },
            VillageCode {
                name: "南旗牧委会",
                code: "012",
            },
            VillageCode {
                name: "吉仁牧委会",
                code: "013",
            },
            VillageCode {
                name: "多特牧委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "宁木特镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "东吾沟社区",
                code: "001",
            },
            VillageCode {
                name: "鲁沙社区",
                code: "002",
            },
            VillageCode {
                name: "浪琴牧委会",
                code: "003",
            },
            VillageCode {
                name: "梧桐牧委会",
                code: "004",
            },
            VillageCode {
                name: "周龙牧委会",
                code: "005",
            },
            VillageCode {
                name: "作毛牧委会",
                code: "006",
            },
            VillageCode {
                name: "苏青牧委会",
                code: "007",
            },
            VillageCode {
                name: "德日旦牧委会",
                code: "008",
            },
            VillageCode {
                name: "夏拉牧委会",
                code: "009",
            },
            VillageCode {
                name: "卫拉牧委会",
                code: "010",
            },
            VillageCode {
                name: "尕群牧委会",
                code: "011",
            },
            VillageCode {
                name: "赛尔永牧委会",
                code: "012",
            },
            VillageCode {
                name: "宁木特牧委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "多松乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "拉让牧委会",
                code: "001",
            },
            VillageCode {
                name: "多松牧委会",
                code: "002",
            },
            VillageCode {
                name: "夏日达哇牧委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "赛尔龙乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "赛尔龙牧委会",
                code: "001",
            },
            VillageCode {
                name: "兰龙牧委会",
                code: "002",
            },
            VillageCode {
                name: "尕克牧委会",
                code: "003",
            },
            VillageCode {
                name: "尖克牧委会",
                code: "004",
            },
            VillageCode {
                name: "尕庆牧委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "柯生乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "尖克牧委会",
                code: "001",
            },
            VillageCode {
                name: "次汉苏牧委会",
                code: "002",
            },
            VillageCode {
                name: "毛曲牧委会",
                code: "003",
            },
            VillageCode {
                name: "柯生牧委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "托叶玛乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "曲海牧委会",
                code: "001",
            },
            VillageCode {
                name: "托叶玛牧委会",
                code: "002",
            },
            VillageCode {
                name: "曲龙牧委会",
                code: "003",
            },
            VillageCode {
                name: "文群牧委会",
                code: "004",
            },
            VillageCode {
                name: "宁赛牧委会",
                code: "005",
            },
            VillageCode {
                name: "夏吾特牧委会",
                code: "006",
            },
        ],
    },
];

static TOWNS_QH_023: [TownCode; 15] = [
    TownCode {
        name: "恰卜恰镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城中居委会",
                code: "001",
            },
            VillageCode {
                name: "城西居委会",
                code: "002",
            },
            VillageCode {
                name: "城北居委会",
                code: "003",
            },
            VillageCode {
                name: "城东居委会",
                code: "004",
            },
            VillageCode {
                name: "金安居委会",
                code: "005",
            },
            VillageCode {
                name: "城南居委会",
                code: "006",
            },
            VillageCode {
                name: "工业园居委会",
                code: "007",
            },
            VillageCode {
                name: "黄河路社区居委会",
                code: "008",
            },
            VillageCode {
                name: "城北新区居委会",
                code: "009",
            },
            VillageCode {
                name: "次汗素村委会",
                code: "010",
            },
            VillageCode {
                name: "东香卡村委会",
                code: "011",
            },
            VillageCode {
                name: "西台村委会",
                code: "012",
            },
            VillageCode {
                name: "尕寺村委会",
                code: "013",
            },
            VillageCode {
                name: "下塔迈村委会",
                code: "014",
            },
            VillageCode {
                name: "索吉亥村委会",
                code: "015",
            },
            VillageCode {
                name: "西香卡村委会",
                code: "016",
            },
            VillageCode {
                name: "上塔迈村委会",
                code: "017",
            },
            VillageCode {
                name: "加拉村委会",
                code: "018",
            },
            VillageCode {
                name: "乙浪堂村委会",
                code: "019",
            },
            VillageCode {
                name: "上梅村委会",
                code: "020",
            },
            VillageCode {
                name: "索尔加村委会",
                code: "021",
            },
            VillageCode {
                name: "东巴村委会",
                code: "022",
            },
            VillageCode {
                name: "加隆台村委会",
                code: "023",
            },
            VillageCode {
                name: "下梅村委会",
                code: "024",
            },
        ],
    },
    TownCode {
        name: "倒淌河镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "倒淌河居委会",
                code: "001",
            },
            VillageCode {
                name: "倒淌河镇第二社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "哈乙亥村委会",
                code: "003",
            },
            VillageCode {
                name: "次汗达哇村委会",
                code: "004",
            },
            VillageCode {
                name: "拉乙亥麻村委会",
                code: "005",
            },
            VillageCode {
                name: "元者村委会",
                code: "006",
            },
            VillageCode {
                name: "东卫村委会",
                code: "007",
            },
            VillageCode {
                name: "蒙古村委会",
                code: "008",
            },
            VillageCode {
                name: "黄科村委会",
                code: "009",
            },
            VillageCode {
                name: "甲乙村委会",
                code: "010",
            },
            VillageCode {
                name: "黑科村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "龙羊峡镇",
        code: "003",
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
                name: "龙才村委会",
                code: "003",
            },
            VillageCode {
                name: "瓦里关村委会",
                code: "004",
            },
            VillageCode {
                name: "龙羊新村委会",
                code: "005",
            },
            VillageCode {
                name: "麻尼磨台村委会",
                code: "006",
            },
            VillageCode {
                name: "阿乙亥村委会",
                code: "007",
            },
            VillageCode {
                name: "次汗土亥村委会",
                code: "008",
            },
            VillageCode {
                name: "后菊花村委会",
                code: "009",
            },
            VillageCode {
                name: "多隆沟村委会",
                code: "010",
            },
            VillageCode {
                name: "克才村委会",
                code: "011",
            },
            VillageCode {
                name: "德胜村委会",
                code: "012",
            },
            VillageCode {
                name: "曹多隆村委会",
                code: "013",
            },
            VillageCode {
                name: "黄河村委会",
                code: "014",
            },
            VillageCode {
                name: "查那村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "塘格木镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "塘格木镇社区居委会",
                code: "001",
            },
            VillageCode {
                name: "曲宗村委会",
                code: "002",
            },
            VillageCode {
                name: "智德村委会",
                code: "003",
            },
            VillageCode {
                name: "华塘村委会",
                code: "004",
            },
            VillageCode {
                name: "达拉村委会",
                code: "005",
            },
            VillageCode {
                name: "吾赫勒村委会",
                code: "006",
            },
            VillageCode {
                name: "哈尔干村委会",
                code: "007",
            },
            VillageCode {
                name: "加什达村委会",
                code: "008",
            },
            VillageCode {
                name: "曲让村委会",
                code: "009",
            },
            VillageCode {
                name: "黄河村委会",
                code: "010",
            },
            VillageCode {
                name: "中果村委会",
                code: "011",
            },
            VillageCode {
                name: "治海村委会",
                code: "012",
            },
            VillageCode {
                name: "更尕村委会",
                code: "013",
            },
            VillageCode {
                name: "浪娘村委会",
                code: "014",
            },
            VillageCode {
                name: "金塘村委会",
                code: "015",
            },
            VillageCode {
                name: "尕当村委会",
                code: "016",
            },
            VillageCode {
                name: "东格村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "黑马河镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "黑马河居委会",
                code: "001",
            },
            VillageCode {
                name: "正却乎村委会",
                code: "002",
            },
            VillageCode {
                name: "加隆村委会",
                code: "003",
            },
            VillageCode {
                name: "然却乎村委会",
                code: "004",
            },
            VillageCode {
                name: "文巴村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "石乃亥镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "石乃亥居委会",
                code: "001",
            },
            VillageCode {
                name: "切吉村委会",
                code: "002",
            },
            VillageCode {
                name: "尕日拉村委会",
                code: "003",
            },
            VillageCode {
                name: "向公村委会",
                code: "004",
            },
            VillageCode {
                name: "肉龙村委会",
                code: "005",
            },
            VillageCode {
                name: "铁卜加村委会",
                code: "006",
            },
            VillageCode {
                name: "鲁色村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "江西沟镇",
        code: "007",
        villages: &[
            VillageCode {
                name: "江西沟乡社区居委会",
                code: "001",
            },
            VillageCode {
                name: "大仓村委会",
                code: "002",
            },
            VillageCode {
                name: "元者村委会",
                code: "003",
            },
            VillageCode {
                name: "莫热村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "沙珠玉乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "珠玉村委会",
                code: "001",
            },
            VillageCode {
                name: "曲沟村委会",
                code: "002",
            },
            VillageCode {
                name: "耐海塔村委会",
                code: "003",
            },
            VillageCode {
                name: "治乃亥村委会",
                code: "004",
            },
            VillageCode {
                name: "下卡力岗村委会",
                code: "005",
            },
            VillageCode {
                name: "达连海村委会",
                code: "006",
            },
            VillageCode {
                name: "种子村委会",
                code: "007",
            },
            VillageCode {
                name: "上卡力岗村委会",
                code: "008",
            },
            VillageCode {
                name: "上村村委会",
                code: "009",
            },
            VillageCode {
                name: "扎布达村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "铁盖乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "马汉台村委会",
                code: "001",
            },
            VillageCode {
                name: "委曲村委会",
                code: "002",
            },
            VillageCode {
                name: "托勒台村委会",
                code: "003",
            },
            VillageCode {
                name: "上合乐寺村委会",
                code: "004",
            },
            VillageCode {
                name: "吾雷村委会",
                code: "005",
            },
            VillageCode {
                name: "七台村委会",
                code: "006",
            },
            VillageCode {
                name: "铁盖村委会",
                code: "007",
            },
            VillageCode {
                name: "下合乐寺村委会",
                code: "008",
            },
            VillageCode {
                name: "拉才村委会",
                code: "009",
            },
            VillageCode {
                name: "哈汉土亥村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "廿地乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "廿地村委会",
                code: "001",
            },
            VillageCode {
                name: "切扎村委会",
                code: "002",
            },
            VillageCode {
                name: "拉龙村委会",
                code: "003",
            },
            VillageCode {
                name: "羊让村委会",
                code: "004",
            },
            VillageCode {
                name: "曲什那村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "切吉乡",
        code: "011",
        villages: &[
            VillageCode {
                name: "社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东科村委会",
                code: "002",
            },
            VillageCode {
                name: "塔秀村委会",
                code: "003",
            },
            VillageCode {
                name: "哇合村委会",
                code: "004",
            },
            VillageCode {
                name: "祁加村委会",
                code: "005",
            },
            VillageCode {
                name: "新村村委会",
                code: "006",
            },
            VillageCode {
                name: "莫合村委会",
                code: "007",
            },
            VillageCode {
                name: "加什科村委会",
                code: "008",
            },
            VillageCode {
                name: "乔夫旦村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "海南州绿色产业发展园区管理区委员会",
        code: "012",
        villages: &[VillageCode {
            name: "海南州绿色产业发展园区管理区委员会虚拟社区",
            code: "001",
        }],
    },
    TownCode {
        name: "巴卡台农场",
        code: "013",
        villages: &[VillageCode {
            name: "巴卡台农场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "安置农场",
        code: "014",
        villages: &[VillageCode {
            name: "安置农场虚拟生活区",
            code: "001",
        }],
    },
    TownCode {
        name: "铁卜加草改站",
        code: "015",
        villages: &[VillageCode {
            name: "铁卜加草改站虚拟生活区",
            code: "001",
        }],
    },
];

static TOWNS_QH_024: [TownCode; 5] = [
    TownCode {
        name: "尕巴松多镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城关第一居委会",
                code: "001",
            },
            VillageCode {
                name: "城关第二居委会",
                code: "002",
            },
            VillageCode {
                name: "北巴滩社区",
                code: "003",
            },
            VillageCode {
                name: "县城新区赛康社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "牧场社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "德什端村委会",
                code: "006",
            },
            VillageCode {
                name: "科加村委会",
                code: "007",
            },
            VillageCode {
                name: "瓜什则村委会",
                code: "008",
            },
            VillageCode {
                name: "贡麻村委会",
                code: "009",
            },
            VillageCode {
                name: "秀麻村委会",
                code: "010",
            },
            VillageCode {
                name: "夏日仓村委会",
                code: "011",
            },
            VillageCode {
                name: "欧沟村委会",
                code: "012",
            },
            VillageCode {
                name: "北扎村委会",
                code: "013",
            },
            VillageCode {
                name: "完科村委会",
                code: "014",
            },
            VillageCode {
                name: "赛加村委会",
                code: "015",
            },
            VillageCode {
                name: "知后迈村委会",
                code: "016",
            },
            VillageCode {
                name: "美日克村委会",
                code: "017",
            },
            VillageCode {
                name: "豆言村委会",
                code: "018",
            },
            VillageCode {
                name: "科日干村委会",
                code: "019",
            },
            VillageCode {
                name: "申吾村委会",
                code: "020",
            },
            VillageCode {
                name: "欧后扎村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "唐谷镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "科加滩社区",
                code: "001",
            },
            VillageCode {
                name: "县城新区日雪社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "托斯村委会",
                code: "003",
            },
            VillageCode {
                name: "阿血村委会",
                code: "004",
            },
            VillageCode {
                name: "美日克村委会",
                code: "005",
            },
            VillageCode {
                name: "尤龙村委会",
                code: "006",
            },
            VillageCode {
                name: "那仁村委会",
                code: "007",
            },
            VillageCode {
                name: "东吾村委会",
                code: "008",
            },
            VillageCode {
                name: "扎血村委会",
                code: "009",
            },
            VillageCode {
                name: "元庄村委会",
                code: "010",
            },
            VillageCode {
                name: "达隆村委会",
                code: "011",
            },
            VillageCode {
                name: "合土乎村委会",
                code: "012",
            },
            VillageCode {
                name: "力伦村委会",
                code: "013",
            },
            VillageCode {
                name: "哈夏村委会",
                code: "014",
            },
            VillageCode {
                name: "加吾村委会",
                code: "015",
            },
            VillageCode {
                name: "东格村委会",
                code: "016",
            },
            VillageCode {
                name: "赛什堂村委会",
                code: "017",
            },
            VillageCode {
                name: "青迈村委会",
                code: "018",
            },
            VillageCode {
                name: "加拉村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "巴沟乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "下巴村委会",
                code: "001",
            },
            VillageCode {
                name: "上巴村委会",
                code: "002",
            },
            VillageCode {
                name: "松多村委会",
                code: "003",
            },
            VillageCode {
                name: "地干村委会",
                code: "004",
            },
            VillageCode {
                name: "火角村委会",
                code: "005",
            },
            VillageCode {
                name: "曲乃亥村委会",
                code: "006",
            },
            VillageCode {
                name: "尕哇麻村委会",
                code: "007",
            },
            VillageCode {
                name: "托头村委会",
                code: "008",
            },
            VillageCode {
                name: "上尕毛其村委会",
                code: "009",
            },
            VillageCode {
                name: "下尕毛其村委会",
                code: "010",
            },
            VillageCode {
                name: "上阿格村委会",
                code: "011",
            },
            VillageCode {
                name: "下阿格村委会",
                code: "012",
            },
            VillageCode {
                name: "班多村委会",
                code: "013",
            },
            VillageCode {
                name: "团结村委会",
                code: "014",
            },
            VillageCode {
                name: "卡力岗村委会",
                code: "015",
            },
            VillageCode {
                name: "上才乃亥村委会",
                code: "016",
            },
            VillageCode {
                name: "下才乃亥村委会",
                code: "017",
            },
            VillageCode {
                name: "新村委会",
                code: "018",
            },
            VillageCode {
                name: "直德村委会",
                code: "019",
            },
            VillageCode {
                name: "本龙村委会",
                code: "020",
            },
            VillageCode {
                name: "加日亥村委会",
                code: "021",
            },
            VillageCode {
                name: "然果村委会",
                code: "022",
            },
        ],
    },
    TownCode {
        name: "秀麻乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "赛隆社区",
                code: "001",
            },
            VillageCode {
                name: "县城新区巴塘社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "三巴村委会",
                code: "003",
            },
            VillageCode {
                name: "豆素村委会",
                code: "004",
            },
            VillageCode {
                name: "木合村委会",
                code: "005",
            },
            VillageCode {
                name: "达哇村委会",
                code: "006",
            },
            VillageCode {
                name: "宁龙村委会",
                code: "007",
            },
            VillageCode {
                name: "豆龙村委会",
                code: "008",
            },
            VillageCode {
                name: "德格村委会",
                code: "009",
            },
            VillageCode {
                name: "老虎村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "河北乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "赛堂社区",
                code: "001",
            },
            VillageCode {
                name: "县城新区斗尔宗社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "赛堂村委会",
                code: "003",
            },
            VillageCode {
                name: "赛若村委会",
                code: "004",
            },
            VillageCode {
                name: "格什格村委会",
                code: "005",
            },
            VillageCode {
                name: "黄河村委会",
                code: "006",
            },
            VillageCode {
                name: "下知迈村委会",
                code: "007",
            },
            VillageCode {
                name: "赛青村委会",
                code: "008",
            },
            VillageCode {
                name: "赛羊村委会",
                code: "009",
            },
            VillageCode {
                name: "上知迈村委会",
                code: "010",
            },
            VillageCode {
                name: "金科村委会",
                code: "011",
            },
            VillageCode {
                name: "赛德村委会",
                code: "012",
            },
        ],
    },
];

static TOWNS_QH_025: [TownCode; 7] = [
    TownCode {
        name: "河阴镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "古城社区居委会",
                code: "001",
            },
            VillageCode {
                name: "城东居委会",
                code: "002",
            },
            VillageCode {
                name: "城西居委会",
                code: "003",
            },
            VillageCode {
                name: "南海社区居委会",
                code: "004",
            },
            VillageCode {
                name: "新东路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "歇春园社区居委会",
                code: "006",
            },
            VillageCode {
                name: "林荫社区居委会",
                code: "007",
            },
            VillageCode {
                name: "城关村村委会",
                code: "008",
            },
            VillageCode {
                name: "城东村村委会",
                code: "009",
            },
            VillageCode {
                name: "城西村村委会",
                code: "010",
            },
            VillageCode {
                name: "城北村村委会",
                code: "011",
            },
            VillageCode {
                name: "郭拉村村委会",
                code: "012",
            },
            VillageCode {
                name: "邓家村村委会",
                code: "013",
            },
            VillageCode {
                name: "童家村村委会",
                code: "014",
            },
            VillageCode {
                name: "西家嘴村村委会",
                code: "015",
            },
            VillageCode {
                name: "大史家村村委会",
                code: "016",
            },
            VillageCode {
                name: "张家沟村村委会",
                code: "017",
            },
            VillageCode {
                name: "红柳滩村村委会",
                code: "018",
            },
            VillageCode {
                name: "杏花村村委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "河西镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "河西镇居民委员会",
                code: "001",
            },
            VillageCode {
                name: "格尔加村村委会",
                code: "002",
            },
            VillageCode {
                name: "上刘屯村村委会",
                code: "003",
            },
            VillageCode {
                name: "下刘屯村村委会",
                code: "004",
            },
            VillageCode {
                name: "下马家村村委会",
                code: "005",
            },
            VillageCode {
                name: "瓦家村村委会",
                code: "006",
            },
            VillageCode {
                name: "多勒仓村村委会",
                code: "007",
            },
            VillageCode {
                name: "加洛苏合村村委会",
                code: "008",
            },
            VillageCode {
                name: "拉及盖村村委会",
                code: "009",
            },
            VillageCode {
                name: "才堂村村委会",
                code: "010",
            },
            VillageCode {
                name: "红岩村村委会",
                code: "011",
            },
            VillageCode {
                name: "下排村村委会",
                code: "012",
            },
            VillageCode {
                name: "温泉村村委会",
                code: "013",
            },
            VillageCode {
                name: "贡拜村村委会",
                code: "014",
            },
            VillageCode {
                name: "大户村村委会",
                code: "015",
            },
            VillageCode {
                name: "多哇村村委会",
                code: "016",
            },
            VillageCode {
                name: "山坪村村委会",
                code: "017",
            },
            VillageCode {
                name: "北房村村委会",
                code: "018",
            },
            VillageCode {
                name: "木干村村委会",
                code: "019",
            },
            VillageCode {
                name: "本科村村委会",
                code: "020",
            },
            VillageCode {
                name: "加莫河滩村村委会",
                code: "021",
            },
            VillageCode {
                name: "加莫台村村委会",
                code: "022",
            },
            VillageCode {
                name: "贺尔加村村委会",
                code: "023",
            },
            VillageCode {
                name: "江仓麻村村委会",
                code: "024",
            },
            VillageCode {
                name: "西山湾村村委会",
                code: "025",
            },
            VillageCode {
                name: "园艺村村委会",
                code: "026",
            },
            VillageCode {
                name: "甘家村村委会",
                code: "027",
            },
            VillageCode {
                name: "团结村村委会",
                code: "028",
            },
            VillageCode {
                name: "幸福村村委会",
                code: "029",
            },
            VillageCode {
                name: "万珠村村委会",
                code: "030",
            },
        ],
    },
    TownCode {
        name: "拉西瓦镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "罗汉堂村村委会",
                code: "001",
            },
            VillageCode {
                name: "尼那村村委会",
                code: "002",
            },
            VillageCode {
                name: "多拉村村委会",
                code: "003",
            },
            VillageCode {
                name: "曲卜藏村村委会",
                code: "004",
            },
            VillageCode {
                name: "昨那村村委会",
                code: "005",
            },
            VillageCode {
                name: "豆后浪村村委会",
                code: "006",
            },
            VillageCode {
                name: "叶后浪村村委会",
                code: "007",
            },
            VillageCode {
                name: "仍果村村委会",
                code: "008",
            },
            VillageCode {
                name: "尼那新村村委会",
                code: "009",
            },
            VillageCode {
                name: "曲乃海村村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "常牧镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "常牧社区居委会",
                code: "001",
            },
            VillageCode {
                name: "周屯村村委会",
                code: "002",
            },
            VillageCode {
                name: "高红崖村村委会",
                code: "003",
            },
            VillageCode {
                name: "新建坪村村委会",
                code: "004",
            },
            VillageCode {
                name: "斜马浪村村委会",
                code: "005",
            },
            VillageCode {
                name: "加卜查村村委会",
                code: "006",
            },
            VillageCode {
                name: "却加村村委会",
                code: "007",
            },
            VillageCode {
                name: "上兰角村村委会",
                code: "008",
            },
            VillageCode {
                name: "下兰角村村委会",
                code: "009",
            },
            VillageCode {
                name: "色尔加村村委会",
                code: "010",
            },
            VillageCode {
                name: "卷木村村委会",
                code: "011",
            },
            VillageCode {
                name: "梅加村村委会",
                code: "012",
            },
            VillageCode {
                name: "浪查村村委会",
                code: "013",
            },
            VillageCode {
                name: "苟后扎村村委会",
                code: "014",
            },
            VillageCode {
                name: "豆后漏村村委会",
                code: "015",
            },
            VillageCode {
                name: "兰角滩村村委会",
                code: "016",
            },
            VillageCode {
                name: "干果羊村村委会",
                code: "017",
            },
            VillageCode {
                name: "曲玛塘村村委会",
                code: "018",
            },
            VillageCode {
                name: "下岗查村村委会",
                code: "019",
            },
            VillageCode {
                name: "都秀村村委会",
                code: "020",
            },
            VillageCode {
                name: "拉德村村委会",
                code: "021",
            },
            VillageCode {
                name: "切扎村村委会",
                code: "022",
            },
            VillageCode {
                name: "吾隆村村委会",
                code: "023",
            },
            VillageCode {
                name: "岗查贡麻村村委会",
                code: "024",
            },
            VillageCode {
                name: "达尕羊村村委会",
                code: "025",
            },
            VillageCode {
                name: "达隆村村委会",
                code: "026",
            },
        ],
    },
    TownCode {
        name: "河东乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "河东社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东河社区居委会",
                code: "002",
            },
            VillageCode {
                name: "马家西村村委会",
                code: "003",
            },
            VillageCode {
                name: "太平村村委会",
                code: "004",
            },
            VillageCode {
                name: "下罗家村村委会",
                code: "005",
            },
            VillageCode {
                name: "周家村村委会",
                code: "006",
            },
            VillageCode {
                name: "杨家村村委会",
                code: "007",
            },
            VillageCode {
                name: "王屯村村委会",
                code: "008",
            },
            VillageCode {
                name: "保宁村村委会",
                code: "009",
            },
            VillageCode {
                name: "麻巴村村委会",
                code: "010",
            },
            VillageCode {
                name: "哇里村村委会",
                code: "011",
            },
            VillageCode {
                name: "贡巴村村委会",
                code: "012",
            },
            VillageCode {
                name: "查达村村委会",
                code: "013",
            },
            VillageCode {
                name: "阿什贡村村委会",
                code: "014",
            },
            VillageCode {
                name: "西北村村委会",
                code: "015",
            },
            VillageCode {
                name: "沙柳湾村村委会",
                code: "016",
            },
            VillageCode {
                name: "边都村村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "新街回族乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "新街村村委会",
                code: "001",
            },
            VillageCode {
                name: "藏盖村村委会",
                code: "002",
            },
            VillageCode {
                name: "鱼山村村委会",
                code: "003",
            },
            VillageCode {
                name: "陆切村村委会",
                code: "004",
            },
            VillageCode {
                name: "麻吾村村委会",
                code: "005",
            },
            VillageCode {
                name: "上卡村村委会",
                code: "006",
            },
            VillageCode {
                name: "下卡村村委会",
                code: "007",
            },
            VillageCode {
                name: "老虎口村村委会",
                code: "008",
            },
            VillageCode {
                name: "尕麻堂村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "尕让乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "尕让村村委会",
                code: "001",
            },
            VillageCode {
                name: "阿什贡村村委会",
                code: "002",
            },
            VillageCode {
                name: "阿言麦村村委会",
                code: "003",
            },
            VillageCode {
                name: "查曲昂村村委会",
                code: "004",
            },
            VillageCode {
                name: "亦什扎村村委会",
                code: "005",
            },
            VillageCode {
                name: "松巴村村委会",
                code: "006",
            },
            VillageCode {
                name: "扎力毛村村委会",
                code: "007",
            },
            VillageCode {
                name: "业隆村村委会",
                code: "008",
            },
            VillageCode {
                name: "关加村村委会",
                code: "009",
            },
            VillageCode {
                name: "大滩村村委会",
                code: "010",
            },
            VillageCode {
                name: "洛乙海村村委会",
                code: "011",
            },
            VillageCode {
                name: "东果堂村村委会",
                code: "012",
            },
            VillageCode {
                name: "蓆芨滩村村委会",
                code: "013",
            },
            VillageCode {
                name: "亦扎石村村委会",
                code: "014",
            },
            VillageCode {
                name: "大磨村村委会",
                code: "015",
            },
            VillageCode {
                name: "者么昂村村委会",
                code: "016",
            },
            VillageCode {
                name: "千户村村委会",
                code: "017",
            },
            VillageCode {
                name: "俄加村村委会",
                code: "018",
            },
            VillageCode {
                name: "二连村村委会",
                code: "019",
            },
            VillageCode {
                name: "黄河滩村村委会",
                code: "020",
            },
            VillageCode {
                name: "希望村村委会",
                code: "021",
            },
            VillageCode {
                name: "江拉新村村委会",
                code: "022",
            },
        ],
    },
];

static TOWNS_QH_026: [TownCode; 7] = [
    TownCode {
        name: "子科滩镇",
        code: "001",
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
                name: "城东社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "城西社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "泉曲村民委员会",
                code: "005",
            },
            VillageCode {
                name: "那洞村牧委会",
                code: "006",
            },
            VillageCode {
                name: "青根河村牧委会",
                code: "007",
            },
            VillageCode {
                name: "黄清村牧委会",
                code: "008",
            },
            VillageCode {
                name: "恰当村牧委会",
                code: "009",
            },
            VillageCode {
                name: "日干村牧委会",
                code: "010",
            },
            VillageCode {
                name: "直亥买村牧委会",
                code: "011",
            },
            VillageCode {
                name: "切卜藏村民委员会",
                code: "012",
            },
        ],
    },
    TownCode {
        name: "河卡镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "河卡镇社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "都台村民委员会",
                code: "002",
            },
            VillageCode {
                name: "幸福村民委员会",
                code: "003",
            },
            VillageCode {
                name: "上游村民委员会",
                code: "004",
            },
            VillageCode {
                name: "灯塔村民委员会",
                code: "005",
            },
            VillageCode {
                name: "白龙村民委员会",
                code: "006",
            },
            VillageCode {
                name: "红旗村民委员会",
                code: "007",
            },
            VillageCode {
                name: "宁曲村民委员会",
                code: "008",
            },
            VillageCode {
                name: "五一村民委员会",
                code: "009",
            },
            VillageCode {
                name: "羊曲村民委员会",
                code: "010",
            },
            VillageCode {
                name: "加拉确什滩村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "曲什安镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "曲什安镇社区",
                code: "001",
            },
            VillageCode {
                name: "大米滩村民委员会",
                code: "002",
            },
            VillageCode {
                name: "莫多村民委员会",
                code: "003",
            },
            VillageCode {
                name: "塔洞村民委员会",
                code: "004",
            },
            VillageCode {
                name: "团结村民委员会",
                code: "005",
            },
            VillageCode {
                name: "才乃亥村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "温泉乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "温泉点社区",
                code: "001",
            },
            VillageCode {
                name: "南木塘村民委员会",
                code: "002",
            },
            VillageCode {
                name: "温泉村牧委会",
                code: "003",
            },
            VillageCode {
                name: "长水村牧委会",
                code: "004",
            },
            VillageCode {
                name: "多巴村牧委会",
                code: "005",
            },
            VillageCode {
                name: "尕科河村牧委会",
                code: "006",
            },
            VillageCode {
                name: "盖什干村民委员会",
                code: "007",
            },
            VillageCode {
                name: "赛什塘村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "龙藏乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "赛日巴村民委员会",
                code: "001",
            },
            VillageCode {
                name: "浪青村民委员会",
                code: "002",
            },
            VillageCode {
                name: "桑什斗村民委员会",
                code: "003",
            },
            VillageCode {
                name: "木果村牧委会",
                code: "004",
            },
            VillageCode {
                name: "麻日毛村牧委会",
                code: "005",
            },
            VillageCode {
                name: "日旭村民委员会",
                code: "006",
            },
            VillageCode {
                name: "那洞村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "中铁乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "杜宗村民委员会",
                code: "001",
            },
            VillageCode {
                name: "吉浪村牧委会",
                code: "002",
            },
            VillageCode {
                name: "恰青村牧委会",
                code: "003",
            },
            VillageCode {
                name: "豆后塘村牧委会",
                code: "004",
            },
            VillageCode {
                name: "然毛村牧委会",
                code: "005",
            },
            VillageCode {
                name: "隆吾龙村牧委会",
                code: "006",
            },
            VillageCode {
                name: "民族村民委员会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "唐乃亥乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "中村村民委员会",
                code: "001",
            },
            VillageCode {
                name: "加吾沟村村民委员会",
                code: "002",
            },
            VillageCode {
                name: "上鹿圈村村民委员会",
                code: "003",
            },
            VillageCode {
                name: "下鹿圈村村民委员会",
                code: "004",
            },
            VillageCode {
                name: "上村村民委员会",
                code: "005",
            },
            VillageCode {
                name: "下村村民委员会",
                code: "006",
            },
            VillageCode {
                name: "沙那村村民委员会",
                code: "007",
            },
            VillageCode {
                name: "民族村村民委员会",
                code: "008",
            },
            VillageCode {
                name: "龙曲村村民委员会",
                code: "009",
            },
            VillageCode {
                name: "桑当村村民委员会",
                code: "010",
            },
            VillageCode {
                name: "明星村村民委员会",
                code: "011",
            },
            VillageCode {
                name: "野马台村村民委员会",
                code: "012",
            },
            VillageCode {
                name: "夏塘村村民委员会",
                code: "013",
            },
        ],
    },
];

static TOWNS_QH_027: [TownCode; 6] = [
    TownCode {
        name: "茫曲镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城北社区居委会",
                code: "001",
            },
            VillageCode {
                name: "城南社区居委会",
                code: "002",
            },
            VillageCode {
                name: "城东社区居委会",
                code: "003",
            },
            VillageCode {
                name: "城西社区居委会",
                code: "004",
            },
            VillageCode {
                name: "加土乎村委会",
                code: "005",
            },
            VillageCode {
                name: "达玉村委会",
                code: "006",
            },
            VillageCode {
                name: "毡匠村委会",
                code: "007",
            },
            VillageCode {
                name: "河州村委会",
                code: "008",
            },
            VillageCode {
                name: "江当村委会",
                code: "009",
            },
            VillageCode {
                name: "那燃村委会",
                code: "010",
            },
            VillageCode {
                name: "昂索村委会",
                code: "011",
            },
            VillageCode {
                name: "托勒村委会",
                code: "012",
            },
            VillageCode {
                name: "沙拉村委会",
                code: "013",
            },
            VillageCode {
                name: "塔哇村委会",
                code: "014",
            },
            VillageCode {
                name: "上达玉村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "过马营镇",
        code: "002",
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
                name: "过茫村委会",
                code: "004",
            },
            VillageCode {
                name: "直亥村委会",
                code: "005",
            },
            VillageCode {
                name: "日安秀麻村委会",
                code: "006",
            },
            VillageCode {
                name: "角色村委会",
                code: "007",
            },
            VillageCode {
                name: "洛加村委会",
                code: "008",
            },
            VillageCode {
                name: "麻什干村委会",
                code: "009",
            },
            VillageCode {
                name: "沙加村委会",
                code: "010",
            },
            VillageCode {
                name: "切扎村委会",
                code: "011",
            },
            VillageCode {
                name: "多拉村委会",
                code: "012",
            },
            VillageCode {
                name: "达拉村委会",
                code: "013",
            },
            VillageCode {
                name: "查乃亥村委会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "森多镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "社区居委会",
                code: "001",
            },
            VillageCode {
                name: "青科羊村委会",
                code: "002",
            },
            VillageCode {
                name: "完秀村委会",
                code: "003",
            },
            VillageCode {
                name: "日茫村委会",
                code: "004",
            },
            VillageCode {
                name: "塔哇村委会",
                code: "005",
            },
            VillageCode {
                name: "鲁秀麻村委会",
                code: "006",
            },
            VillageCode {
                name: "斯肉村委会",
                code: "007",
            },
            VillageCode {
                name: "加尚村委会",
                code: "008",
            },
            VillageCode {
                name: "卡加村委会",
                code: "009",
            },
            VillageCode {
                name: "尼麻龙村委会",
                code: "010",
            },
            VillageCode {
                name: "本龙村委会",
                code: "011",
            },
            VillageCode {
                name: "贡哇村委会",
                code: "012",
            },
            VillageCode {
                name: "加当村委会",
                code: "013",
            },
            VillageCode {
                name: "斗龙村委会",
                code: "014",
            },
            VillageCode {
                name: "扎日格村委会",
                code: "015",
            },
            VillageCode {
                name: "赛羊村委会",
                code: "016",
            },
            VillageCode {
                name: "元义村委会",
                code: "017",
            },
        ],
    },
    TownCode {
        name: "沙沟乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "郭仁多村委会",
                code: "001",
            },
            VillageCode {
                name: "赛尔塘村委会",
                code: "002",
            },
            VillageCode {
                name: "石崖村委会",
                code: "003",
            },
            VillageCode {
                name: "东吾羊村委会",
                code: "004",
            },
            VillageCode {
                name: "尕巴村委会",
                code: "005",
            },
            VillageCode {
                name: "洛合相村委会",
                code: "006",
            },
            VillageCode {
                name: "石乃亥村委会",
                code: "007",
            },
            VillageCode {
                name: "查纳村委会",
                code: "008",
            },
            VillageCode {
                name: "拉扎村委会",
                code: "009",
            },
            VillageCode {
                name: "居乎拉村委会",
                code: "010",
            },
            VillageCode {
                name: "过列村委会",
                code: "011",
            },
            VillageCode {
                name: "东让村委会",
                code: "012",
            },
            VillageCode {
                name: "德茫村委会",
                code: "013",
            },
            VillageCode {
                name: "关塘村委会",
                code: "014",
            },
            VillageCode {
                name: "汪什科村委会",
                code: "015",
            },
        ],
    },
    TownCode {
        name: "茫拉乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "却旦塘村委会",
                code: "001",
            },
            VillageCode {
                name: "克州村委会",
                code: "002",
            },
            VillageCode {
                name: "都兰村委会",
                code: "003",
            },
            VillageCode {
                name: "上洛哇村委会",
                code: "004",
            },
            VillageCode {
                name: "下洛哇村委会",
                code: "005",
            },
            VillageCode {
                name: "活然村委会",
                code: "006",
            },
            VillageCode {
                name: "康吾羊村委会",
                code: "007",
            },
            VillageCode {
                name: "郭拉村委会",
                code: "008",
            },
            VillageCode {
                name: "吐鲁村委会",
                code: "009",
            },
            VillageCode {
                name: "下江当村委会",
                code: "010",
            },
            VillageCode {
                name: "麻格塘村委会",
                code: "011",
            },
            VillageCode {
                name: "郭玉乎村委会",
                code: "012",
            },
            VillageCode {
                name: "拉干村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "塔秀乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "社区居委会",
                code: "001",
            },
            VillageCode {
                name: "达茫村委会",
                code: "002",
            },
            VillageCode {
                name: "塔秀村委会",
                code: "003",
            },
            VillageCode {
                name: "贡哇村委会",
                code: "004",
            },
            VillageCode {
                name: "子哈村委会",
                code: "005",
            },
            VillageCode {
                name: "加斯村委会",
                code: "006",
            },
            VillageCode {
                name: "达龙村委会",
                code: "007",
            },
            VillageCode {
                name: "西格村委会",
                code: "008",
            },
            VillageCode {
                name: "扎日干村委会",
                code: "009",
            },
            VillageCode {
                name: "巴塘新村村委会",
                code: "010",
            },
            VillageCode {
                name: "贵南黑羊场生活区",
                code: "011",
            },
        ],
    },
];

static TOWNS_QH_028: [TownCode; 8] = [
    TownCode {
        name: "大武镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "雪山路北社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "黄河路北社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "黄河路南社区居民委员会",
                code: "003",
            },
            VillageCode {
                name: "雪山路南社区居民委员会",
                code: "004",
            },
            VillageCode {
                name: "团结路北社区居民委员会",
                code: "005",
            },
            VillageCode {
                name: "团结路南社区居民委员会",
                code: "006",
            },
            VillageCode {
                name: "永宝村民委员会",
                code: "007",
            },
            VillageCode {
                name: "吾麻村民委员会",
                code: "008",
            },
            VillageCode {
                name: "尼玛龙村民委员会",
                code: "009",
            },
            VillageCode {
                name: "查仓村民委员会",
                code: "010",
            },
            VillageCode {
                name: "血麻村民委员会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "拉加镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "军功路社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "拉加路社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "曲哇加萨村民委员会",
                code: "003",
            },
            VillageCode {
                name: "赞根村民委员会",
                code: "004",
            },
            VillageCode {
                name: "思肉欠村民委员会",
                code: "005",
            },
            VillageCode {
                name: "哈夏村民委员会",
                code: "006",
            },
            VillageCode {
                name: "台西村民委员会",
                code: "007",
            },
            VillageCode {
                name: "欧科村民委员会",
                code: "008",
            },
            VillageCode {
                name: "拉什德村民委员会",
                code: "009",
            },
            VillageCode {
                name: "加思乎村民委员会",
                code: "010",
            },
            VillageCode {
                name: "叶合恰村民委员会",
                code: "011",
            },
            VillageCode {
                name: "洋玉村民委员会",
                code: "012",
            },
            VillageCode {
                name: "赛什托村民委员会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "大武乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "日进村民委员会",
                code: "001",
            },
            VillageCode {
                name: "江前村民委员会",
                code: "002",
            },
            VillageCode {
                name: "哈龙村民委员会",
                code: "003",
            },
            VillageCode {
                name: "格多村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "东倾沟乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "东柯河村民委员会",
                code: "001",
            },
            VillageCode {
                name: "当前村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "雪山乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "阴柯河村民委员会",
                code: "001",
            },
            VillageCode {
                name: "阳柯河村民委员会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "下大武乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "年扎村民委员会",
                code: "001",
            },
            VillageCode {
                name: "尼青村民委员会",
                code: "002",
            },
            VillageCode {
                name: "清水村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "优云乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "优曲村民委员会",
                code: "001",
            },
            VillageCode {
                name: "德当村民委员会",
                code: "002",
            },
            VillageCode {
                name: "阳桑村民委员会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "当洛乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "加青村民委员会",
                code: "001",
            },
            VillageCode {
                name: "查雀干麻村民委员会",
                code: "002",
            },
            VillageCode {
                name: "查雀贡麻村民委员会",
                code: "003",
            },
            VillageCode {
                name: "贡龙村民委员会",
                code: "004",
            },
            VillageCode {
                name: "格雅村民委员会",
                code: "005",
            },
        ],
    },
];

static TOWNS_QH_029: [TownCode; 9] = [
    TownCode {
        name: "赛来塘镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "莲花居委会",
                code: "001",
            },
            VillageCode {
                name: "雍忠社区居委会",
                code: "002",
            },
            VillageCode {
                name: "班脑河村委会",
                code: "003",
            },
            VillageCode {
                name: "德昂村委会",
                code: "004",
            },
            VillageCode {
                name: "合科村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "多贡麻乡",
        code: "002",
        villages: &[
            VillageCode {
                name: "多贡麻村委会",
                code: "001",
            },
            VillageCode {
                name: "玛当吾村委会",
                code: "002",
            },
            VillageCode {
                name: "满掌村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "马可河乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "则达村委会",
                code: "001",
            },
            VillageCode {
                name: "则多村委会",
                code: "002",
            },
            VillageCode {
                name: "马格勒村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "吉卡乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "当吾村委会",
                code: "001",
            },
            VillageCode {
                name: "贡掌村委会",
                code: "002",
            },
            VillageCode {
                name: "玛尼村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "达卡乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "佐诺村委会",
                code: "001",
            },
            VillageCode {
                name: "多娘村委会",
                code: "002",
            },
            VillageCode {
                name: "董仲村委会",
                code: "003",
            },
            VillageCode {
                name: "兰青村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "知钦乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "赤沟村委会",
                code: "001",
            },
            VillageCode {
                name: "克迈村委会",
                code: "002",
            },
            VillageCode {
                name: "知钦村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "江日堂乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "多日麻村委会",
                code: "001",
            },
            VillageCode {
                name: "尕日麻村委会",
                code: "002",
            },
            VillageCode {
                name: "阿什羌村委会",
                code: "003",
            },
            VillageCode {
                name: "更达村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "亚尔堂乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "王柔村委会",
                code: "001",
            },
            VillageCode {
                name: "果芒村委会",
                code: "002",
            },
            VillageCode {
                name: "日合洞村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "灯塔乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "要什道村委会",
                code: "001",
            },
            VillageCode {
                name: "忠智村委会",
                code: "002",
            },
            VillageCode {
                name: "班前村委会",
                code: "003",
            },
            VillageCode {
                name: "仁青岗村委会",
                code: "004",
            },
            VillageCode {
                name: "科培村委会",
                code: "005",
            },
            VillageCode {
                name: "格日则村委会",
                code: "006",
            },
        ],
    },
];

static TOWNS_QH_030: [TownCode; 7] = [
    TownCode {
        name: "柯曲镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "柯曲居委会",
                code: "001",
            },
            VillageCode {
                name: "达协居委会",
                code: "002",
            },
            VillageCode {
                name: "当成村委会",
                code: "003",
            },
            VillageCode {
                name: "目日村委会",
                code: "004",
            },
            VillageCode {
                name: "阿隆村委会",
                code: "005",
            },
            VillageCode {
                name: "东吾村委会",
                code: "006",
            },
            VillageCode {
                name: "安木掌村委会",
                code: "007",
            },
            VillageCode {
                name: "达协村委会",
                code: "008",
            },
            VillageCode {
                name: "曲纳合村委会",
                code: "009",
            },
            VillageCode {
                name: "德肉村委会",
                code: "010",
            },
            VillageCode {
                name: "德里尖村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "上贡麻乡",
        code: "002",
        villages: &[
            VillageCode {
                name: "哇英村委会",
                code: "001",
            },
            VillageCode {
                name: "旺日呼村委会",
                code: "002",
            },
            VillageCode {
                name: "隆亚村委会",
                code: "003",
            },
            VillageCode {
                name: "扎加隆村委会",
                code: "004",
            },
            VillageCode {
                name: "珠合隆村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "下贡麻乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "折合血村委会",
                code: "001",
            },
            VillageCode {
                name: "俄尔金村委会",
                code: "002",
            },
            VillageCode {
                name: "龙恩村委会",
                code: "003",
            },
            VillageCode {
                name: "索合青村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "岗龙乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "岗龙村委会",
                code: "001",
            },
            VillageCode {
                name: "科拉村委会",
                code: "002",
            },
            VillageCode {
                name: "龙木且村委会",
                code: "003",
            },
            VillageCode {
                name: "恰不将村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "青珍乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "青珍村委会",
                code: "001",
            },
            VillageCode {
                name: "龙尕日村委会",
                code: "002",
            },
            VillageCode {
                name: "直尕日村委会",
                code: "003",
            },
            VillageCode {
                name: "休麻村委会",
                code: "004",
            },
            VillageCode {
                name: "直合麻村委会",
                code: "005",
            },
            VillageCode {
                name: "典哲村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "江千乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "恰曲纳合村委会",
                code: "001",
            },
            VillageCode {
                name: "隆吉村委会",
                code: "002",
            },
            VillageCode {
                name: "叶合青村委会",
                code: "003",
            },
            VillageCode {
                name: "协隆村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "下藏科乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "旦库村委会",
                code: "001",
            },
            VillageCode {
                name: "江千村委会",
                code: "002",
            },
            VillageCode {
                name: "洞索村委会",
                code: "003",
            },
            VillageCode {
                name: "科赛村委会",
                code: "004",
            },
        ],
    },
];

static TOWNS_QH_031: [TownCode; 10] = [
    TownCode {
        name: "吉迈镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "丹玛社区居委会",
                code: "001",
            },
            VillageCode {
                name: "达沃社区居委会",
                code: "002",
            },
            VillageCode {
                name: "岭格社区居委会",
                code: "003",
            },
            VillageCode {
                name: "普忙村委会",
                code: "004",
            },
            VillageCode {
                name: "垮热村委会",
                code: "005",
            },
            VillageCode {
                name: "龙才村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "满掌乡",
        code: "002",
        villages: &[
            VillageCode {
                name: "木热村委会",
                code: "001",
            },
            VillageCode {
                name: "查干村委会",
                code: "002",
            },
            VillageCode {
                name: "布东村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "德昂乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "唐什加村委会",
                code: "001",
            },
            VillageCode {
                name: "莫日合村委会",
                code: "002",
            },
            VillageCode {
                name: "康隆村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "窝赛乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "康巴村委会",
                code: "001",
            },
            VillageCode {
                name: "直却村委会",
                code: "002",
            },
            VillageCode {
                name: "依隆村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "莫坝乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "赛尔钦村委会",
                code: "001",
            },
            VillageCode {
                name: "萨尔根村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "上红科乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "哈青村委会",
                code: "001",
            },
            VillageCode {
                name: "尼勒村委会",
                code: "002",
            },
            VillageCode {
                name: "优根村委会",
                code: "003",
            },
            VillageCode {
                name: "特根村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "下红科乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "达孜村委会",
                code: "001",
            },
            VillageCode {
                name: "色隆村委会",
                code: "002",
            },
            VillageCode {
                name: "哲格村委会",
                code: "003",
            },
            VillageCode {
                name: "那尼村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "建设乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "测日哇村委会",
                code: "001",
            },
            VillageCode {
                name: "沙日纳村委会",
                code: "002",
            },
            VillageCode {
                name: "达日龙村委会",
                code: "003",
            },
            VillageCode {
                name: "长查村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "桑日麻乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "向阳村委会",
                code: "001",
            },
            VillageCode {
                name: "东风村委会",
                code: "002",
            },
            VillageCode {
                name: "红旗村委会",
                code: "003",
            },
            VillageCode {
                name: "前进村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "特合土乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "扣压村委会",
                code: "001",
            },
            VillageCode {
                name: "夏曲牧委会",
                code: "002",
            },
            VillageCode {
                name: "科曲村委会",
                code: "003",
            },
        ],
    },
];

static TOWNS_QH_032: [TownCode; 6] = [
    TownCode {
        name: "智青松多镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "南环路社区",
                code: "001",
            },
            VillageCode {
                name: "黄河路社区",
                code: "002",
            },
            VillageCode {
                name: "滨河路社区",
                code: "003",
            },
            VillageCode {
                name: "德合龙村委会",
                code: "004",
            },
            VillageCode {
                name: "宁友村委会",
                code: "005",
            },
            VillageCode {
                name: "沙科村委会",
                code: "006",
            },
            VillageCode {
                name: "果江村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "门堂乡",
        code: "002",
        villages: &[
            VillageCode {
                name: "门堂村委会",
                code: "001",
            },
            VillageCode {
                name: "果囊村委会",
                code: "002",
            },
        ],
    },
    TownCode {
        name: "哇赛乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "折安村委会",
                code: "001",
            },
            VillageCode {
                name: "国钦村委会",
                code: "002",
            },
            VillageCode {
                name: "富钦村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "索呼日麻乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "索呼日麻村委会",
                code: "001",
            },
            VillageCode {
                name: "尖木村委会",
                code: "002",
            },
            VillageCode {
                name: "扎拉村委会",
                code: "003",
            },
            VillageCode {
                name: "章达村委会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "白玉乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "龙格村委会",
                code: "001",
            },
            VillageCode {
                name: "白玉村委会",
                code: "002",
            },
            VillageCode {
                name: "牧羊村委会",
                code: "003",
            },
            VillageCode {
                name: "科索村委会",
                code: "004",
            },
            VillageCode {
                name: "俄科村委会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "哇尔依乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "满格村委会",
                code: "001",
            },
            VillageCode {
                name: "赛池村委会",
                code: "002",
            },
            VillageCode {
                name: "达尕村委会",
                code: "003",
            },
            VillageCode {
                name: "扎依村委会",
                code: "004",
            },
        ],
    },
];

static TOWNS_QH_033: [TownCode; 4] = [
    TownCode {
        name: "玛查理镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "玛查理居委会",
                code: "001",
            },
            VillageCode {
                name: "江措村委会",
                code: "002",
            },
            VillageCode {
                name: "江多村委会",
                code: "003",
            },
            VillageCode {
                name: "隆埂村委会",
                code: "004",
            },
            VillageCode {
                name: "尕拉村委会",
                code: "005",
            },
            VillageCode {
                name: "赫拉村委会",
                code: "006",
            },
            VillageCode {
                name: "刊木青村委会",
                code: "007",
            },
            VillageCode {
                name: "玛拉驿村委会",
                code: "008",
            },
            VillageCode {
                name: "玛查理新村",
                code: "009",
            },
            VillageCode {
                name: "野牛沟新村",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "花石峡镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "花石峡居委会",
                code: "001",
            },
            VillageCode {
                name: "措柔村委会",
                code: "002",
            },
            VillageCode {
                name: "东泽村委会",
                code: "003",
            },
            VillageCode {
                name: "日谢村委会",
                code: "004",
            },
            VillageCode {
                name: "吉日迈村委会",
                code: "005",
            },
            VillageCode {
                name: "维日埂村委会",
                code: "006",
            },
            VillageCode {
                name: "扎地村委会",
                code: "007",
            },
            VillageCode {
                name: "斗纳村委会",
                code: "008",
            },
            VillageCode {
                name: "加果村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "黄河乡",
        code: "003",
        villages: &[
            VillageCode {
                name: "江旁村委会",
                code: "001",
            },
            VillageCode {
                name: "热曲村委会",
                code: "002",
            },
            VillageCode {
                name: "阿映村委会",
                code: "003",
            },
            VillageCode {
                name: "白玛纳村委会",
                code: "004",
            },
            VillageCode {
                name: "塘格玛村委会",
                code: "005",
            },
            VillageCode {
                name: "斗江村委会",
                code: "006",
            },
            VillageCode {
                name: "果洛新村",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "扎陵湖乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "阿涌村委会",
                code: "001",
            },
            VillageCode {
                name: "尕泽村委会",
                code: "002",
            },
            VillageCode {
                name: "多涌村委会",
                code: "003",
            },
            VillageCode {
                name: "擦泽村委会",
                code: "004",
            },
            VillageCode {
                name: "卓让村委会",
                code: "005",
            },
            VillageCode {
                name: "勒那村委会",
                code: "006",
            },
        ],
    },
];

static TOWNS_QH_034: [TownCode; 7] = [
    TownCode {
        name: "河西街道",
        code: "001",
        villages: &[
            VillageCode {
                name: "建设路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "昆仑路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "朝阳社区居委会",
                code: "003",
            },
            VillageCode {
                name: "渠南社区居委会",
                code: "004",
            },
            VillageCode {
                name: "柴达木路社区居委会",
                code: "005",
            },
            VillageCode {
                name: "尕南庄社区居委会",
                code: "006",
            },
            VillageCode {
                name: "巴音河村委会",
                code: "007",
            },
            VillageCode {
                name: "北山村委会",
                code: "008",
            },
            VillageCode {
                name: "巴音河西村委会",
                code: "009",
            },
            VillageCode {
                name: "白水河村委会",
                code: "010",
            },
            VillageCode {
                name: "甘南村委会",
                code: "011",
            },
        ],
    },
    TownCode {
        name: "河东街道",
        code: "002",
        villages: &[
            VillageCode {
                name: "长江路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "祁连路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "滨河路社区居委会",
                code: "003",
            },
            VillageCode {
                name: "乌兰路社区居委会",
                code: "004",
            },
            VillageCode {
                name: "红光村委会",
                code: "005",
            },
            VillageCode {
                name: "东山村委会",
                code: "006",
            },
            VillageCode {
                name: "阳光村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "火车站街道",
        code: "003",
        villages: &[
            VillageCode {
                name: "黄河路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "都兰路社区居委会",
                code: "002",
            },
            VillageCode {
                name: "工业园区社区居委会",
                code: "003",
            },
            VillageCode {
                name: "城南新区社区居委会",
                code: "004",
            },
            VillageCode {
                name: "民乐村委会",
                code: "005",
            },
            VillageCode {
                name: "平原村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "尕海镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "尕海街道社区居委会",
                code: "001",
            },
            VillageCode {
                name: "尕海村委会",
                code: "002",
            },
            VillageCode {
                name: "郭里木新村村委会",
                code: "003",
            },
            VillageCode {
                name: "新源村委会",
                code: "004",
            },
            VillageCode {
                name: "陶哈村委会",
                code: "005",
            },
            VillageCode {
                name: "努尔村委会",
                code: "006",
            },
            VillageCode {
                name: "富源村委会",
                code: "007",
            },
            VillageCode {
                name: "东升村委会",
                code: "008",
            },
            VillageCode {
                name: "泉水村委会",
                code: "009",
            },
            VillageCode {
                name: "富康村委会",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "怀头他拉镇",
        code: "005",
        villages: &[
            VillageCode {
                name: "小康路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东滩村委会",
                code: "002",
            },
            VillageCode {
                name: "西滩村委会",
                code: "003",
            },
            VillageCode {
                name: "怀图村委会",
                code: "004",
            },
            VillageCode {
                name: "巴力沟村委会",
                code: "005",
            },
            VillageCode {
                name: "卡格图村委会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "柯鲁柯镇",
        code: "006",
        villages: &[
            VillageCode {
                name: "德兴路社区居委会",
                code: "001",
            },
            VillageCode {
                name: "乌兰干沟村委会",
                code: "002",
            },
            VillageCode {
                name: "德令哈村委会",
                code: "003",
            },
            VillageCode {
                name: "茶汉沙村委会",
                code: "004",
            },
            VillageCode {
                name: "克鲁诺尔村委会",
                code: "005",
            },
            VillageCode {
                name: "陶生诺尔村委会",
                code: "006",
            },
            VillageCode {
                name: "新秀村委会",
                code: "007",
            },
            VillageCode {
                name: "花土村委会",
                code: "008",
            },
            VillageCode {
                name: "连湖村委会",
                code: "009",
            },
            VillageCode {
                name: "金原村委会",
                code: "010",
            },
            VillageCode {
                name: "希望村委会",
                code: "011",
            },
            VillageCode {
                name: "安康村委会",
                code: "012",
            },
            VillageCode {
                name: "民兴村委会",
                code: "013",
            },
        ],
    },
    TownCode {
        name: "蓄集乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "野马滩社区居委会",
                code: "001",
            },
            VillageCode {
                name: "浩特茶汉村委会",
                code: "002",
            },
            VillageCode {
                name: "茶汉哈达村委会",
                code: "003",
            },
            VillageCode {
                name: "贡艾里沟村委会",
                code: "004",
            },
            VillageCode {
                name: "陶斯图村委会",
                code: "005",
            },
            VillageCode {
                name: "伊克拉村委会",
                code: "006",
            },
            VillageCode {
                name: "乌察汗村委会",
                code: "007",
            },
        ],
    },
];

static TOWNS_QH_035: [TownCode; 5] = [
    TownCode {
        name: "希里沟镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城东社区居委会",
                code: "001",
            },
            VillageCode {
                name: "城中社区居委会",
                code: "002",
            },
            VillageCode {
                name: "城西社区居委会",
                code: "003",
            },
            VillageCode {
                name: "东庄村村委会",
                code: "004",
            },
            VillageCode {
                name: "西庄村村委会",
                code: "005",
            },
            VillageCode {
                name: "北庄村村委会",
                code: "006",
            },
            VillageCode {
                name: "河东村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "茶卡镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "茶卡社区居委会",
                code: "001",
            },
            VillageCode {
                name: "巴里河滩村村委会",
                code: "002",
            },
            VillageCode {
                name: "夏艾里沟村村委会",
                code: "003",
            },
            VillageCode {
                name: "乌兰哈达村村委会",
                code: "004",
            },
            VillageCode {
                name: "那仁村村委会",
                code: "005",
            },
            VillageCode {
                name: "塔拉村村委会",
                code: "006",
            },
            VillageCode {
                name: "扎布寺村村委会",
                code: "007",
            },
            VillageCode {
                name: "茶卡村村委会",
                code: "008",
            },
            VillageCode {
                name: "巴音村村委会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "柯柯镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "柯柯社区居委会",
                code: "001",
            },
            VillageCode {
                name: "东村村委会",
                code: "002",
            },
            VillageCode {
                name: "中村村委会",
                code: "003",
            },
            VillageCode {
                name: "西村村委会",
                code: "004",
            },
            VillageCode {
                name: "新村村委会",
                code: "005",
            },
            VillageCode {
                name: "赛什克村村委会",
                code: "006",
            },
            VillageCode {
                name: "北柯柯村村委会",
                code: "007",
            },
            VillageCode {
                name: "南柯柯村村委会",
                code: "008",
            },
            VillageCode {
                name: "卜浪沟村村委会",
                code: "009",
            },
            VillageCode {
                name: "东沙沟村村委会",
                code: "010",
            },
            VillageCode {
                name: "西沙沟村村委会",
                code: "011",
            },
            VillageCode {
                name: "南沙沟村村委会",
                code: "012",
            },
            VillageCode {
                name: "纳木哈村村委会",
                code: "013",
            },
            VillageCode {
                name: "怀灿吉村村委会",
                code: "014",
            },
            VillageCode {
                name: "托海村村委会",
                code: "015",
            },
            VillageCode {
                name: "圆山村村委会",
                code: "016",
            },
            VillageCode {
                name: "路顺村村委会",
                code: "017",
            },
            VillageCode {
                name: "赛纳村村委会",
                code: "018",
            },
            VillageCode {
                name: "兴隆村村委会",
                code: "019",
            },
            VillageCode {
                name: "兴乐村村委会",
                code: "020",
            },
            VillageCode {
                name: "兴化村村委会",
                code: "021",
            },
        ],
    },
    TownCode {
        name: "铜普镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "察汗诺社区居委会",
                code: "001",
            },
            VillageCode {
                name: "上尕巴村村委会",
                code: "002",
            },
            VillageCode {
                name: "都兰河村村委会",
                code: "003",
            },
            VillageCode {
                name: "河北村村委会",
                code: "004",
            },
            VillageCode {
                name: "河南村村委会",
                code: "005",
            },
            VillageCode {
                name: "察汗诺村村委会",
                code: "006",
            },
            VillageCode {
                name: "察汗河村村委会",
                code: "007",
            },
        ],
    },
    TownCode {
        name: "海西州莫河畜牧场",
        code: "005",
        villages: &[VillageCode {
            name: "场部虚拟生活区",
            code: "001",
        }],
    },
];

static TOWNS_QH_036: [TownCode; 8] = [
    TownCode {
        name: "察汉乌苏镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "和平街居委会",
                code: "001",
            },
            VillageCode {
                name: "新华街居委会",
                code: "002",
            },
            VillageCode {
                name: "民主街居委会",
                code: "003",
            },
            VillageCode {
                name: "英得尔居委会",
                code: "004",
            },
            VillageCode {
                name: "西河滩中村委会",
                code: "005",
            },
            VillageCode {
                name: "西河滩下村委会",
                code: "006",
            },
            VillageCode {
                name: "西沙滩村委会",
                code: "007",
            },
            VillageCode {
                name: "西建村委会",
                code: "008",
            },
            VillageCode {
                name: "西星村委会",
                code: "009",
            },
            VillageCode {
                name: "北园村委会",
                code: "010",
            },
            VillageCode {
                name: "西园村委会",
                code: "011",
            },
            VillageCode {
                name: "下滩村委会",
                code: "012",
            },
            VillageCode {
                name: "上滩东村委会",
                code: "013",
            },
            VillageCode {
                name: "上滩西村委会",
                code: "014",
            },
            VillageCode {
                name: "东山根西村委会",
                code: "015",
            },
            VillageCode {
                name: "上庄村委会",
                code: "016",
            },
            VillageCode {
                name: "上西台村委会",
                code: "017",
            },
            VillageCode {
                name: "下西台村委会",
                code: "018",
            },
            VillageCode {
                name: "西庄村委会",
                code: "019",
            },
            VillageCode {
                name: "中庄村委会",
                code: "020",
            },
            VillageCode {
                name: "东庄村委会",
                code: "021",
            },
            VillageCode {
                name: "东山根上村委会",
                code: "022",
            },
            VillageCode {
                name: "东山根中村委会",
                code: "023",
            },
            VillageCode {
                name: "东山根下村委会",
                code: "024",
            },
            VillageCode {
                name: "西河滩上村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "香日德镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "南北街居委会",
                code: "001",
            },
            VillageCode {
                name: "香巴社区居委会",
                code: "002",
            },
            VillageCode {
                name: "沱海村委会",
                code: "003",
            },
            VillageCode {
                name: "新源村委会",
                code: "004",
            },
            VillageCode {
                name: "新华村委会",
                code: "005",
            },
            VillageCode {
                name: "下柴源村委会",
                code: "006",
            },
            VillageCode {
                name: "上柴源村委会",
                code: "007",
            },
            VillageCode {
                name: "柴兴村委会",
                code: "008",
            },
            VillageCode {
                name: "下柴开村委会",
                code: "009",
            },
            VillageCode {
                name: "中庄村委会",
                code: "010",
            },
            VillageCode {
                name: "上柴开村委会",
                code: "011",
            },
            VillageCode {
                name: "沙珠玉村委会",
                code: "012",
            },
            VillageCode {
                name: "察汗毛村委会",
                code: "013",
            },
            VillageCode {
                name: "得胜村委会",
                code: "014",
            },
            VillageCode {
                name: "兴盛村委会",
                code: "015",
            },
            VillageCode {
                name: "永盛村委会",
                code: "016",
            },
            VillageCode {
                name: "联盛村委会",
                code: "017",
            },
            VillageCode {
                name: "东盛村村委会",
                code: "018",
            },
            VillageCode {
                name: "乐盛村村委会",
                code: "019",
            },
            VillageCode {
                name: "香乐村村委会",
                code: "020",
            },
            VillageCode {
                name: "东山村村委会",
                code: "021",
            },
            VillageCode {
                name: "香盛村村委会",
                code: "022",
            },
            VillageCode {
                name: "香源村村委会",
                code: "023",
            },
            VillageCode {
                name: "小夏滩村村委会",
                code: "024",
            },
            VillageCode {
                name: "幸福村委会",
                code: "025",
            },
        ],
    },
    TownCode {
        name: "夏日哈镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "夏兴街居委会",
                code: "001",
            },
            VillageCode {
                name: "新乐村委会",
                code: "002",
            },
            VillageCode {
                name: "沙珠玉村委会",
                code: "003",
            },
            VillageCode {
                name: "夏塔拉村委会",
                code: "004",
            },
            VillageCode {
                name: "果米村委会",
                code: "005",
            },
            VillageCode {
                name: "查查村委会",
                code: "006",
            },
            VillageCode {
                name: "河北村委会",
                code: "007",
            },
            VillageCode {
                name: "河南村委会",
                code: "008",
            },
            VillageCode {
                name: "联合村委会",
                code: "009",
            },
            VillageCode {
                name: "青海省柴达木农垦查查香卡生活区",
                code: "010",
            },
        ],
    },
    TownCode {
        name: "宗加镇",
        code: "004",
        villages: &[
            VillageCode {
                name: "诺木洪中心街社区",
                code: "001",
            },
            VillageCode {
                name: "诺木洪村委会",
                code: "002",
            },
            VillageCode {
                name: "西西牧委会",
                code: "003",
            },
            VillageCode {
                name: "阿斯林牧委会",
                code: "004",
            },
            VillageCode {
                name: "努日牧委会",
                code: "005",
            },
            VillageCode {
                name: "那木哈牧委会",
                code: "006",
            },
            VillageCode {
                name: "布拉牧委会",
                code: "007",
            },
            VillageCode {
                name: "沙日牧委会",
                code: "008",
            },
            VillageCode {
                name: "托海牧委会",
                code: "009",
            },
            VillageCode {
                name: "铁奎牧委会",
                code: "010",
            },
            VillageCode {
                name: "农业村委会",
                code: "011",
            },
            VillageCode {
                name: "哈西娃牧委会",
                code: "012",
            },
            VillageCode {
                name: "田格力牧委会",
                code: "013",
            },
            VillageCode {
                name: "艾斯力金牧委会",
                code: "014",
            },
            VillageCode {
                name: "乌图牧委会",
                code: "015",
            },
            VillageCode {
                name: "诺木洪农场生活区",
                code: "016",
            },
        ],
    },
    TownCode {
        name: "热水乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "扎玛日村委会",
                code: "001",
            },
            VillageCode {
                name: "赛什堂村委会",
                code: "002",
            },
            VillageCode {
                name: "智尕日村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "香加乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "立新村委会",
                code: "001",
            },
            VillageCode {
                name: "先锋村委会",
                code: "002",
            },
            VillageCode {
                name: "前进村委会",
                code: "003",
            },
            VillageCode {
                name: "向阳村委会",
                code: "004",
            },
            VillageCode {
                name: "红星村委会",
                code: "005",
            },
            VillageCode {
                name: "团结村委会",
                code: "006",
            },
            VillageCode {
                name: "红旗村委会",
                code: "007",
            },
            VillageCode {
                name: "全杰村委会",
                code: "008",
            },
            VillageCode {
                name: "科尔村委会",
                code: "009",
            },
            VillageCode {
                name: "孟克台牧委会",
                code: "010",
            },
            VillageCode {
                name: "阿木尼科尔牧委会",
                code: "011",
            },
            VillageCode {
                name: "陶生湖牧委会",
                code: "012",
            },
            VillageCode {
                name: "柯克沙牧委会",
                code: "013",
            },
            VillageCode {
                name: "呼力木图牧委会",
                code: "014",
            },
            VillageCode {
                name: "德布生牧委会",
                code: "015",
            },
            VillageCode {
                name: "柯克哈达牧委会",
                code: "016",
            },
            VillageCode {
                name: "傲包图牧委会",
                code: "017",
            },
            VillageCode {
                name: "艾力斯台牧委会",
                code: "018",
            },
            VillageCode {
                name: "科学图牧委会",
                code: "019",
            },
        ],
    },
    TownCode {
        name: "沟里乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "热龙村委会",
                code: "001",
            },
            VillageCode {
                name: "智玉村委会",
                code: "002",
            },
            VillageCode {
                name: "秀毛村委会",
                code: "003",
            },
        ],
    },
    TownCode {
        name: "巴隆乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "托托牧委会",
                code: "001",
            },
            VillageCode {
                name: "乌拉斯台牧委会",
                code: "002",
            },
            VillageCode {
                name: "诺木洪牧委会",
                code: "003",
            },
            VillageCode {
                name: "哈图牧委会",
                code: "004",
            },
            VillageCode {
                name: "夏图牧委会",
                code: "005",
            },
            VillageCode {
                name: "伊克高里牧委会",
                code: "006",
            },
            VillageCode {
                name: "布洛格牧委会",
                code: "007",
            },
            VillageCode {
                name: "科日农业村委会",
                code: "008",
            },
            VillageCode {
                name: "河西村委会",
                code: "009",
            },
            VillageCode {
                name: "清泉村委会",
                code: "010",
            },
            VillageCode {
                name: "三合村委会",
                code: "011",
            },
            VillageCode {
                name: "雅日哈图村委会",
                code: "012",
            },
            VillageCode {
                name: "新隆村委会",
                code: "013",
            },
            VillageCode {
                name: "河东村委会",
                code: "014",
            },
            VillageCode {
                name: "科日牧业牧委会",
                code: "015",
            },
        ],
    },
];

static TOWNS_QH_037: [TownCode; 10] = [
    TownCode {
        name: "新源镇",
        code: "001",
        villages: &[
            VillageCode {
                name: "城东社区居民委员会",
                code: "001",
            },
            VillageCode {
                name: "城西社区居民委员会",
                code: "002",
            },
            VillageCode {
                name: "梅陇村民委员会",
                code: "003",
            },
            VillageCode {
                name: "茶木康村民委员会",
                code: "004",
            },
            VillageCode {
                name: "上日许尔村民委员会",
                code: "005",
            },
            VillageCode {
                name: "天棚村民委员会",
                code: "006",
            },
            VillageCode {
                name: "曲陇村民委员会",
                code: "007",
            },
            VillageCode {
                name: "扎德勒村民委员会",
                code: "008",
            },
            VillageCode {
                name: "拉陇村民委员会",
                code: "009",
            },
            VillageCode {
                name: "夏尔宗村民委员会",
                code: "010",
            },
            VillageCode {
                name: "下日许尔村民委员会",
                code: "011",
            },
            VillageCode {
                name: "拉萨尔村民委员会",
                code: "012",
            },
            VillageCode {
                name: "达尔角合村民委员会",
                code: "013",
            },
            VillageCode {
                name: "赛尔雄村民委员会",
                code: "014",
            },
        ],
    },
    TownCode {
        name: "木里镇",
        code: "002",
        villages: &[
            VillageCode {
                name: "聚乎根村民委员会",
                code: "001",
            },
            VillageCode {
                name: "赛纳合让村民委员会",
                code: "002",
            },
            VillageCode {
                name: "佐陇村民委员会",
                code: "003",
            },
            VillageCode {
                name: "唐莫日村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "江河镇",
        code: "003",
        villages: &[
            VillageCode {
                name: "茶果尔村民委员会",
                code: "001",
            },
            VillageCode {
                name: "莫合拉村民委员会",
                code: "002",
            },
            VillageCode {
                name: "柔丹村民委员会",
                code: "003",
            },
            VillageCode {
                name: "舟东村民委员会",
                code: "004",
            },
            VillageCode {
                name: "结盛村民委员会",
                code: "005",
            },
            VillageCode {
                name: "索德村民委员会",
                code: "006",
            },
            VillageCode {
                name: "织合干木村民委员会",
                code: "007",
            },
            VillageCode {
                name: "赛尔创村民委员会",
                code: "008",
            },
        ],
    },
    TownCode {
        name: "快尔玛乡",
        code: "004",
        villages: &[
            VillageCode {
                name: "赛尔曲村民委员会",
                code: "001",
            },
            VillageCode {
                name: "纳尔宗村民委员会",
                code: "002",
            },
            VillageCode {
                name: "曲尕追村民委员会",
                code: "003",
            },
            VillageCode {
                name: "恰通村民委员会",
                code: "004",
            },
            VillageCode {
                name: "德陇村民委员会",
                code: "005",
            },
            VillageCode {
                name: "多尔则村民委员会",
                code: "006",
            },
            VillageCode {
                name: "阳陇村民委员会",
                code: "007",
            },
            VillageCode {
                name: "莫日通村民委员会",
                code: "008",
            },
            VillageCode {
                name: "参木康村民委员会",
                code: "009",
            },
        ],
    },
    TownCode {
        name: "舟群乡",
        code: "005",
        villages: &[
            VillageCode {
                name: "桑毛村民委员会",
                code: "001",
            },
            VillageCode {
                name: "迪尔恩村民委员会",
                code: "002",
            },
            VillageCode {
                name: "岗东村民委员会",
                code: "003",
            },
            VillageCode {
                name: "吉陇村民委员会",
                code: "004",
            },
            VillageCode {
                name: "茫扎村民委员会",
                code: "005",
            },
            VillageCode {
                name: "浪钦村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "织合玛乡",
        code: "006",
        villages: &[
            VillageCode {
                name: "吉刚村民委员会",
                code: "001",
            },
            VillageCode {
                name: "加吉村民委员会",
                code: "002",
            },
            VillageCode {
                name: "加陇村民委员会",
                code: "003",
            },
            VillageCode {
                name: "多玉村民委员会",
                code: "004",
            },
            VillageCode {
                name: "曲陇村民委员会",
                code: "005",
            },
            VillageCode {
                name: "达尔那村民委员会",
                code: "006",
            },
        ],
    },
    TownCode {
        name: "苏里乡",
        code: "007",
        villages: &[
            VillageCode {
                name: "豆库尔村民委员会",
                code: "001",
            },
            VillageCode {
                name: "措岗村民委员会",
                code: "002",
            },
            VillageCode {
                name: "登陇村民委员会",
                code: "003",
            },
            VillageCode {
                name: "曲尕追村民委员会",
                code: "004",
            },
            VillageCode {
                name: "尕河村民委员会",
                code: "005",
            },
        ],
    },
    TownCode {
        name: "生格乡",
        code: "008",
        villages: &[
            VillageCode {
                name: "阿吾嘎尔村民委员会",
                code: "001",
            },
            VillageCode {
                name: "奥陇村民委员会",
                code: "002",
            },
            VillageCode {
                name: "织合纳合村民委员会",
                code: "003",
            },
            VillageCode {
                name: "秀陇村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "阳康乡",
        code: "009",
        villages: &[
            VillageCode {
                name: "赛尔娘村民委员会",
                code: "001",
            },
            VillageCode {
                name: "曲陇村民委员会",
                code: "002",
            },
            VillageCode {
                name: "恰浩尔村民委员会",
                code: "003",
            },
            VillageCode {
                name: "果当村民委员会",
                code: "004",
            },
        ],
    },
    TownCode {
        name: "龙门乡",
        code: "010",
        villages: &[
            VillageCode {
                name: "龙门尔村民委员会",
                code: "001",
            },
            VillageCode {
                name: "扎玛尔村民委员会",
                code: "002",
            },
            VillageCode {
                name: "措芒村民委员会",
                code: "003",
            },
            VillageCode {
                name: "那尔扎村民委员会",
                code: "004",
            },
        ],
    },
];

pub const CITIES_QH: [CityCode; 38] = [
    CityCode {
        name: "省辖市",
        code: "000",
        towns: &[],
    },
    CityCode {
        name: "西宁市",
        code: "001",
        towns: &TOWNS_QH_001,
    },
    CityCode {
        name: "城东市",
        code: "002",
        towns: &TOWNS_QH_002,
    },
    CityCode {
        name: "城中市",
        code: "003",
        towns: &TOWNS_QH_003,
    },
    CityCode {
        name: "城西市",
        code: "004",
        towns: &TOWNS_QH_004,
    },
    CityCode {
        name: "城北市",
        code: "005",
        towns: &TOWNS_QH_005,
    },
    CityCode {
        name: "湟中市",
        code: "006",
        towns: &TOWNS_QH_006,
    },
    CityCode {
        name: "大通市",
        code: "007",
        towns: &TOWNS_QH_007,
    },
    CityCode {
        name: "湟源市",
        code: "008",
        towns: &TOWNS_QH_008,
    },
    CityCode {
        name: "乐都市",
        code: "009",
        towns: &TOWNS_QH_009,
    },
    CityCode {
        name: "平安市",
        code: "010",
        towns: &TOWNS_QH_010,
    },
    CityCode {
        name: "民和市",
        code: "011",
        towns: &TOWNS_QH_011,
    },
    CityCode {
        name: "互助市",
        code: "012",
        towns: &TOWNS_QH_012,
    },
    CityCode {
        name: "化隆市",
        code: "013",
        towns: &TOWNS_QH_013,
    },
    CityCode {
        name: "循化市",
        code: "014",
        towns: &TOWNS_QH_014,
    },
    CityCode {
        name: "门源市",
        code: "015",
        towns: &TOWNS_QH_015,
    },
    CityCode {
        name: "祁连市",
        code: "016",
        towns: &TOWNS_QH_016,
    },
    CityCode {
        name: "海晏市",
        code: "017",
        towns: &TOWNS_QH_017,
    },
    CityCode {
        name: "刚察市",
        code: "018",
        towns: &TOWNS_QH_018,
    },
    CityCode {
        name: "同仁市",
        code: "019",
        towns: &TOWNS_QH_019,
    },
    CityCode {
        name: "尖扎市",
        code: "020",
        towns: &TOWNS_QH_020,
    },
    CityCode {
        name: "泽库市",
        code: "021",
        towns: &TOWNS_QH_021,
    },
    CityCode {
        name: "河南市",
        code: "022",
        towns: &TOWNS_QH_022,
    },
    CityCode {
        name: "共和市",
        code: "023",
        towns: &TOWNS_QH_023,
    },
    CityCode {
        name: "同德市",
        code: "024",
        towns: &TOWNS_QH_024,
    },
    CityCode {
        name: "贵德市",
        code: "025",
        towns: &TOWNS_QH_025,
    },
    CityCode {
        name: "兴海市",
        code: "026",
        towns: &TOWNS_QH_026,
    },
    CityCode {
        name: "贵南市",
        code: "027",
        towns: &TOWNS_QH_027,
    },
    CityCode {
        name: "玛沁市",
        code: "028",
        towns: &TOWNS_QH_028,
    },
    CityCode {
        name: "班玛市",
        code: "029",
        towns: &TOWNS_QH_029,
    },
    CityCode {
        name: "甘德市",
        code: "030",
        towns: &TOWNS_QH_030,
    },
    CityCode {
        name: "达日市",
        code: "031",
        towns: &TOWNS_QH_031,
    },
    CityCode {
        name: "久治市",
        code: "032",
        towns: &TOWNS_QH_032,
    },
    CityCode {
        name: "玛多市",
        code: "033",
        towns: &TOWNS_QH_033,
    },
    CityCode {
        name: "德令哈市",
        code: "034",
        towns: &TOWNS_QH_034,
    },
    CityCode {
        name: "乌兰市",
        code: "035",
        towns: &TOWNS_QH_035,
    },
    CityCode {
        name: "都兰市",
        code: "036",
        towns: &TOWNS_QH_036,
    },
    CityCode {
        name: "天峻市",
        code: "037",
        towns: &TOWNS_QH_037,
    },
];
