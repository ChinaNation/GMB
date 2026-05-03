//! 中文注释:省管理员 3-tier 名册的 SFID 后端本地基线。
//!
//! `sfid::province` 只负责 SFID 号码所需的省市代码;本文件负责省管理员
//! main 公钥、main/backup_1/backup_2 三槽模型和登录归属判断。

#[derive(Debug, Clone, Copy)]
pub(crate) struct ShengAdminMain {
    pub(crate) province: &'static str,
    pub(crate) pubkey: &'static str,
}

#[rustfmt::skip]
pub(crate) const SHENG_ADMIN_MAINS: [ShengAdminMain; 43] = [
    ShengAdminMain { province: "中枢省", pubkey: "0xd641dbfe17fa3fb2427b974212a0fe821b12576e0eade088309d4f05f2cc9930" },
    ShengAdminMain { province: "岭南省", pubkey: "0xe28a39b8f9f9bdc7d0d5c2f6bf290f892a25aeeb34c57002cdb978d13c4efa26" },
    ShengAdminMain { province: "广东省", pubkey: "0x5cdd16e9a9b63f2660ad7829c6d2004ddb713ea46ee5086e53edbda3dd175b42" },
    ShengAdminMain { province: "广西省", pubkey: "0x1cb60c7ae7236b61ab6d678ee240978ba7653174f725cebe50db02642f2e9129" },
    ShengAdminMain { province: "福建省", pubkey: "0x02d25858d77d87bf0637bdf37e0ae45819bed00b06ed41dc3c2e4888512a7003" },
    ShengAdminMain { province: "海南省", pubkey: "0x94c8853d6090b02581659cae1ce33ce0b3c84078b606e53e052d8439e73fec1e" },
    ShengAdminMain { province: "云南省", pubkey: "0xe658db8112f1ea0a7d2e63b7622e2514c5c65a89db441e3df507272ab2d6231e" },
    ShengAdminMain { province: "贵州省", pubkey: "0xfe7176d115b207356914f92e2da1391db92bc5a463be7f89f2b37d65e367895e" },
    ShengAdminMain { province: "湖南省", pubkey: "0x8aaa255eb6fc0ae304b89a55e93809092f897641917f78d0d1e360c198599105" },
    ShengAdminMain { province: "江西省", pubkey: "0x6c11e617a58e56ba71a2d92b7e989de1a649e4103776dbd8465a3f729b66ca31" },
    ShengAdminMain { province: "浙江省", pubkey: "0xf47373164ca9f7167e1da17955761b17e38823348c8aeecb5f259a25d3ad6d2f" },
    ShengAdminMain { province: "江苏省", pubkey: "0x78bc0525055f37f2c7245e94dc95baa3dafc1dc051631f0333bd9dbf9818fb0e" },
    ShengAdminMain { province: "山东省", pubkey: "0x9edf2e0e022b9ff892175528d4a87ef466c0896cc2586b705523932cfd5a1777" },
    ShengAdminMain { province: "山西省", pubkey: "0xac2d0d1ffed7aa373adefa5ddfbc4f377edc91b825b2b13464932bbbb264b40f" },
    ShengAdminMain { province: "河南省", pubkey: "0xdc95de49abd2d371b368256939d15370d0f9915d738d52434431b0c763004b50" },
    ShengAdminMain { province: "河北省", pubkey: "0x604925f9cb49555816b880542cb8045ad4e50165351f5b14d1fd111171bb8617" },
    ShengAdminMain { province: "湖北省", pubkey: "0x1ec98129b379e9f60bad6f0d0bc73e327c20424ac5392192518b71627f752e24" },
    ShengAdminMain { province: "陕西省", pubkey: "0xf6c3e174783aeeea0afc736a42e52ebd2029b4a56de04e9a5301d98094332f45" },
    ShengAdminMain { province: "重庆省", pubkey: "0x1c6f70806461448e7e2621cf29b0924aee483300f4554bea393c1b9c54e78442" },
    ShengAdminMain { province: "四川省", pubkey: "0x7ed7d3bd8ae09960884ff1a98db4493fc5f6818e900f45f66b6b7e76e11e8274" },
    ShengAdminMain { province: "甘肃省", pubkey: "0x52be4ed7bf042b94a4f54ea74369133f5e6ced79e03e84020093c8ec73114c78" },
    ShengAdminMain { province: "北平省", pubkey: "0x940e9a759ce49bee1a49eb8a32dbd03a8813e52f4632534d4cc5c4b7a4cea746" },
    ShengAdminMain { province: "海滨省", pubkey: "0xfccb22b76f7fff0f05dbbab53cba7bbe1bbe0edfece43b139321bec88cb7aa1f" },
    ShengAdminMain { province: "松江省", pubkey: "0x1a1c763345d8bb2e08b30e18788c1bc8e977fd54ba61aa936a8c5db13cf09c03" },
    ShengAdminMain { province: "龙江省", pubkey: "0x4a74ce94de45a80b73850750fd2b08c1782f8e6f4a2301fc2a72af7938a92436" },
    ShengAdminMain { province: "吉林省", pubkey: "0x9a2c2b408a0773c19cfc7207780571ab321dd285f11b7a1bb09e013fed73e737" },
    ShengAdminMain { province: "辽宁省", pubkey: "0xdc3295a5e874ea91d6dcde444b698c5ecf183b16f11954c9fc71e91bfe87b377" },
    ShengAdminMain { province: "宁夏省", pubkey: "0xf05e4afa76f9d883151a6ef656013efef42a6821feef45b42b43f67eca6d6328" },
    ShengAdminMain { province: "青海省", pubkey: "0x1af800fa82965b12fa04f7a87245cc9be5d3fb8cf88a1026e3dc45eacfec405d" },
    ShengAdminMain { province: "安徽省", pubkey: "0x5498141113bf85eca686955162ee2912ac6c3b050ba9ffa102ac923ab0bb350b" },
    ShengAdminMain { province: "台湾省", pubkey: "0xd81866ce95bc72bc7f66e67262e829dcde04b069df3f816faa2865a9382fbf25" },
    ShengAdminMain { province: "西藏省", pubkey: "0x506bb4c300584f13b4307e8cdc251e7756f212c2ee7c302bdd778688c47b201b" },
    ShengAdminMain { province: "新疆省", pubkey: "0x9281ec501bb174b6a608e23fe74770643bdb14e9f26f1aee45f740e3e1d80657" },
    ShengAdminMain { province: "西康省", pubkey: "0xbc6215cb2b86840fb27864f72f08ba09a552e2dfcb38fe8ec010664c37e6b748" },
    ShengAdminMain { province: "阿里省", pubkey: "0xb217302c1c6d099df4a440126df288b74c17ec6b59cd02952b772f47e8154c6d" },
    ShengAdminMain { province: "葱岭省", pubkey: "0x98db54a14cdb9015525467d129668eb58573103013ee9ec8ba380384e2b54b41" },
    ShengAdminMain { province: "天山省", pubkey: "0x463d76ac7e1d3c4cb3355128395189d17bbafb6552a9fdacf075b1fe1f13c32c" },
    ShengAdminMain { province: "河西省", pubkey: "0x2608cab4ded7bee2ac75d55d46d76904f1907b90a4ef768e03cc1663a04de062" },
    ShengAdminMain { province: "昆仑省", pubkey: "0xc645ea0c6e3adb4809268d13cd9820fd759056b2382a5531406873638ce7044a" },
    ShengAdminMain { province: "河套省", pubkey: "0x10972b4b6b227da8cb90cac066502d7210a50955256c83ec083f6b87e3abd71e" },
    ShengAdminMain { province: "热河省", pubkey: "0x1e312af5890084151339ec37b9e7145211366c7ac3163a5ca3d7e8ccb809d674" },
    ShengAdminMain { province: "兴安省", pubkey: "0x10e74326066fceebb3eb103182f36825dee56b077722900c4f718a1fe754823b" },
    ShengAdminMain { province: "合江省", pubkey: "0x8c72490d8774dc1c4305825d82788ad1bd1dc53b06360c2301974e6bc12df638" },
];

pub(crate) fn sheng_admin_mains() -> &'static [ShengAdminMain] {
    &SHENG_ADMIN_MAINS
}

pub(crate) fn sheng_admin_province(pubkey: &str) -> Option<&'static str> {
    SHENG_ADMIN_MAINS
        .iter()
        .find(|p| p.pubkey.eq_ignore_ascii_case(pubkey))
        .map(|p| p.province)
}

pub(crate) fn sheng_admin_display_name(pubkey: &str) -> Option<String> {
    let province_name = sheng_admin_province(pubkey)?;
    Some(format!("{province_name}省级管理员"))
}

/// 省管理员槽位。链上 storage `ShengAdmins[Province][Slot]` 同语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Slot {
    Main,
    Backup1,
    Backup2,
}

/// 某省当前生效的三槽管理员公钥。
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProvinceAdmins {
    pub(crate) main: [u8; 32],
    pub(crate) backup_1: Option<[u8; 32]>,
    pub(crate) backup_2: Option<[u8; 32]>,
}

#[allow(dead_code)]
impl ProvinceAdmins {
    pub(crate) fn from_main(main: [u8; 32]) -> Self {
        Self {
            main,
            backup_1: None,
            backup_2: None,
        }
    }

    pub(crate) fn slot_of(&self, pubkey: &[u8; 32]) -> Option<Slot> {
        if &self.main == pubkey {
            return Some(Slot::Main);
        }
        if let Some(b) = self.backup_1.as_ref() {
            if b == pubkey {
                return Some(Slot::Backup1);
            }
        }
        if let Some(b) = self.backup_2.as_ref() {
            if b == pubkey {
                return Some(Slot::Backup2);
            }
        }
        None
    }
}

/// 把 0x 小写 hex 字符串解析为 32 字节 pubkey。失败返回 None。
pub(crate) fn pubkey_from_hex(hex: &str) -> Option<[u8; 32]> {
    let trimmed = hex
        .trim()
        .strip_prefix("0x")
        .or_else(|| hex.trim().strip_prefix("0X"))
        .unwrap_or_else(|| hex.trim());
    let raw = ::hex::decode(trimmed).ok()?;
    if raw.len() != 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&raw);
    Some(out)
}

/// 中文注释:链上 backup 公钥 pull 尚未切真,当前固定返回空槽。
#[allow(dead_code)]
pub(crate) fn fetch_backup_admins(_province: &str) -> [Option<[u8; 32]>; 2] {
    tracing::warn!("fetch_backup_admins mocked, awaiting chain pull");
    [None, None]
}

#[allow(dead_code)]
pub(crate) fn province_admins_for(province_name: &str) -> Option<ProvinceAdmins> {
    let p = SHENG_ADMIN_MAINS
        .iter()
        .find(|p| p.province == province_name)?;
    let main = pubkey_from_hex(p.pubkey)?;
    let [b1, b2] = fetch_backup_admins(province_name);
    Some(ProvinceAdmins {
        main,
        backup_1: b1,
        backup_2: b2,
    })
}
