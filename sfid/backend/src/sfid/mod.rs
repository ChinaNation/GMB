use chrono::Utc;
pub(crate) mod admin;
pub mod province;
use province::{city_code_by_name, province_code_by_name};

const ALPHABET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub struct GenerateSfidInput<'a> {
    pub account_pubkey: &'a str,
    pub a3: &'a str,
    pub p1: &'a str,
    pub province: &'a str,
    pub city: &'a str,
    pub institution: &'a str,
}

fn hash_text(input: &str) -> u32 {
    let digest = blake3::hash(input.as_bytes());
    let mut out = [0_u8; 4];
    out.copy_from_slice(&digest.as_bytes()[..4]);
    u32::from_le_bytes(out)
}

fn checksum(payload: &str) -> char {
    let mut total: usize = 0;
    for (idx, ch) in payload.chars().enumerate() {
        let pos = ALPHABET.find(ch).unwrap_or(0);
        total = (total + (idx + 1) * pos) % 36;
    }
    ALPHABET.as_bytes()[total] as char
}

fn resolve_org_type(institution: &str) -> Result<&'static str, &'static str> {
    let v = institution.trim();
    match v {
        "ZG" | "中国" => Ok("ZG"),
        "ZF" | "政府" => Ok("ZF"),
        "LF" | "立法院" => Ok("LF"),
        "SF" | "司法院" => Ok("SF"),
        "JC" | "监察院" => Ok("JC"),
        "JY" | "教育委员会" | "公民教育委员会" => Ok("JY"),
        "CB" | "储备委员会" | "公民储备委员会" => Ok("CB"),
        "CH" | "储备银行" | "公民储备银行" => Ok("CH"),
        "TG" | "他国" => Ok("TG"),
        _ => Err("institution must be one of ZG/ZF/LF/SF/JC/JY/CB/CH/TG"),
    }
}

fn resolve_a3(a3: &str) -> Result<&'static str, &'static str> {
    let v = a3.trim();
    match v {
        "GMR" | "公民人" => Ok("GMR"),
        "ZRR" | "自然人" => Ok("ZRR"),
        "ZNR" | "智能人" => Ok("ZNR"),
        "GFR" | "公法人" => Ok("GFR"),
        "SFR" | "私法人" => Ok("SFR"),
        "FFR" | "非法人" => Ok("FFR"),
        _ => Err("a3 must be one of GMR/ZRR/ZNR/GFR/SFR/FFR"),
    }
}

fn resolve_p1(p1: &str) -> Result<&'static str, &'static str> {
    let v = p1.trim();
    match v {
        "0" | "非盈利" => Ok("0"),
        "1" | "盈利" => Ok("1"),
        _ => Err("p1 must be 0/1"),
    }
}

pub fn generate_sfid_code(input: GenerateSfidInput<'_>) -> Result<String, &'static str> {
    if input.account_pubkey.trim().is_empty()
        || input.a3.trim().is_empty()
        || input.province.trim().is_empty()
        || input.city.trim().is_empty()
        || input.institution.trim().is_empty()
    {
        return Err("account_pubkey, a3, province, city, institution are required");
    }

    let a3 = resolve_a3(input.a3)?;
    let t2 = resolve_org_type(input.institution)?;
    let p1 = match a3 {
        "GMR" | "ZRR" => "1",
        "GFR" => "0",
        "ZNR" | "SFR" | "FFR" => resolve_p1(input.p1)?,
        _ => return Err("a3 not supported"),
    };
    if a3 == "GFR" && !matches!(t2, "ZF" | "LF" | "SF" | "JC" | "JY" | "CB") {
        return Err("GFR requires institution in ZF/LF/SF/JC/JY/CB");
    }
    if matches!(a3, "GMR" | "ZNR") && t2 != "ZG" {
        return Err("GMR/ZNR requires institution ZG");
    }
    if a3 == "ZRR" && t2 != "TG" {
        return Err("ZRR requires institution TG");
    }
    if a3 == "SFR" && !matches!(t2, "ZG" | "CH" | "TG") {
        return Err("SFR requires institution in ZG/CH/TG");
    }
    if a3 == "FFR" && !matches!(t2, "ZG" | "TG") {
        return Err("FFR requires institution in ZG/TG");
    }
    let d = Utc::now().format("%Y%m%d").to_string();
    let province_code = province_code_by_name(input.province)
        .ok_or("province not found in code table")?
        .to_string();
    let city_code = city_code_by_name(input.province, input.city)
        .ok_or("city not found in province code table")?
        .to_string();
    let r5 = format!("{province_code}{city_code}");
    let n9 = format!(
        "{:09}",
        (hash_text(&format!(
            "{}|{}|{}|{}|{}|{}",
            input.account_pubkey, a3, input.province, input.city, input.institution, d
        )) as usize)
            % 1_000_000_000
    );
    let payload = format!("{a3}{r5}{t2}{p1}{n9}{d}");
    let c1 = checksum(&payload);
    Ok(format!("{a3}-{r5}-{t2}{p1}{c1}-{n9}-{d}"))
}
