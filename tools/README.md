# 1.工具模块
* 模块简介：工具模块含无私钥地址生成工具、地址生成工具等；

****
# 2.无私钥地址工具/keyless
* 可根据输入“种子”通过blake2_256算法得出无私钥地址；
* // 完整 43 个行政区划种子（已修正陕西拼写并为SHAANXI）
    let provinces = [
        "01_ZHONGSHU", "02_LINGNAN", "03_GUANGDONG", "04_GUANGXI", "05_FUJIAN",
        "06_HAINAN", "07_YUNNAN", "08_GUIZHOU", "09_HUNAN", "10_JIANGXI",
        "11_ZHEJIANG", "12_JIANGSU", "13_SHANDONG", "14_SHANXI", "15_HENAN",
        "16_HEBEI", "17_HUBEI", "18_SHAANXI", "19_CHONGQING", "20_SICHUAN",
        "21_GANSU", "22_BEIPING", "23_HAIBIN", "24_SONGJIANG", "25_LONGJIANG",
        "26_JILIN", "27_LIAONING", "28_NINGXIA", "29_QINGHAI", "30_ANHUI",
        "31_TAIWAN", "32_XIZANG", "33_XINJIANG", "34_XIKANG", "35_ALI",
        "36_CONGLING", "37_TIANSHAN", "38_HEXI", "39_KUNLUN", "40_HETAO",
        "41_REHE", "42_XINGAN", "43_HEJIANG",
    ];
* 以上为43个省储行永久质押地址的“种子”；

****
# 3.地址生成工具/subkey
* 可使用subkey工具生成公民币区块使用的助记词、私钥、公钥；
* 务必断网本地生成。