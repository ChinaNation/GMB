use blake2::{Blake2bVar, digest::{Update, VariableOutput}};
use hex;

fn blake2_256(input: &[u8]) -> [u8; 32] {
    // 明确指定输出为 32 字节 (256位)
    let mut hasher = Blake2bVar::new(32).expect("Blake2bVar weight failed");
    hasher.update(input);
    let mut out = [0u8; 32];
    hasher.finalize_variable(&mut out).expect("Finalize failed");
    out
}

fn main() {
    // 完整 43 个行政区划种子（已修正陕西拼写并为SHAANXI）
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

    println!("// GMB Provincial Keyless Addresses - Generated via Blake2b-256");
    println!("pub const PROVINCIAL_STAKE_ACCOUNTS: [[u8; 32]; 43] = [");
    
    for (i, p) in provinces.iter().enumerate() {
        let seed = format!("GMB_SHENGBANK_STAKE_ADDRESS_{}", p);
        let hash = blake2_256(seed.as_bytes());
        
        // 打印带注释的代码行，方便后续维护核对
        println!(
            "    hex!(\"{}\"), // {:02} {}",
            hex::encode(hash),
            i + 1,
            seed
        );
    }
    println!("];");
}