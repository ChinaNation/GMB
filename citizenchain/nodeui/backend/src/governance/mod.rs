// 治理模块入口：注册 Tauri 命令，聚合机构数据。

mod institution;
pub mod proposal;
pub mod sfid_api;
pub mod signing;
mod storage_keys;
pub mod types;

use crate::home;
use crate::settings::cold_wallets;
use types::{GovernanceOverview, InstitutionDetail, InstitutionListItem, OrgType};

use serde::Serialize;
use tauri::AppHandle;

/// 机构静态注册数据：(名称, shenfenId, OrgType, 多签地址 hex)。
/// 来源于 primitives/china/ 常量，与 wuminapp institution_data.dart 完全一致。
struct InstitutionEntry {
    name: &'static str,
    shenfen_id: &'static str,
    org_type: OrgType,
    duoqian_address: &'static str,
}

// 国储会（1 个）
static NATIONAL_COUNCILS: &[InstitutionEntry] = &[
    InstitutionEntry {
        name: "国家储备委员会",
        shenfen_id: "GFR-LN001-CB0C-617776487-20260222",
        org_type: OrgType::Nrc,
        duoqian_address: "a4dcfcee4629dbd67ebcb271aadf2d79b3b0b72c133156c57f136426b819216e",
    },
];

// 省储会（43 个）
static PROVINCIAL_COUNCILS: &[InstitutionEntry] = &[
    InstitutionEntry { name: "中枢省储备委员会", shenfen_id: "GFR-ZS001-CB0X-464088047-20260222", org_type: OrgType::Prc, duoqian_address: "005860c65dfa43d1efd730560d35fdab296841cfce863039614a690ddd456860" },
    InstitutionEntry { name: "岭南省储备委员会", shenfen_id: "GFR-LN002-CB0Q-850177236-20260222", org_type: OrgType::Prc, duoqian_address: "979ddbbac4c3df93e37b410999ff614265d8c5295faa705e795525405b10b8ea" },
    InstitutionEntry { name: "广东省储备委员会", shenfen_id: "GFR-GD000-CB0O-261883838-20260222", org_type: OrgType::Prc, duoqian_address: "58438c61071a1a52a24b01f414bd5f30c2d01b749f0fc0d7dee628d8a734bf3b" },
    InstitutionEntry { name: "广西省储备委员会", shenfen_id: "GFR-GX000-CB0X-936039238-20260222", org_type: OrgType::Prc, duoqian_address: "bf2d2a5bcfdf09556a8c8bce39831f466a7538372231505bd6426a92a1a6e9b6" },
    InstitutionEntry { name: "福建省储备委员会", shenfen_id: "GFR-FJ000-CB0I-232415560-20260222", org_type: OrgType::Prc, duoqian_address: "27e246c446b60d8503e393e1e49ec554cd48bc3ec68df74a20c0b776a04c8cea" },
    InstitutionEntry { name: "海南省储备委员会", shenfen_id: "GFR-HN000-CB04-832186703-20260222", org_type: OrgType::Prc, duoqian_address: "72142867d115388200dbd0f8d6279b6c96bf6399d7bf09a691d513e49a104689" },
    InstitutionEntry { name: "云南省储备委员会", shenfen_id: "GFR-YN000-CB0G-574048259-20260222", org_type: OrgType::Prc, duoqian_address: "ca96f91555a850e99e0f1f62ec4937d69ef52ebf88dd4b501f9d4298e9104dc6" },
    InstitutionEntry { name: "贵州省储备委员会", shenfen_id: "GFR-GZ000-CB03-700488596-20260222", org_type: OrgType::Prc, duoqian_address: "35b4b1bee060112b348478f77e4075be5ec2d969e313ebfd9b26cf519390d05a" },
    InstitutionEntry { name: "湖南省储备委员会", shenfen_id: "GFR-HU000-CB0V-865805553-20260222", org_type: OrgType::Prc, duoqian_address: "b49be3e53ffc0086f74aa4080d49600a6de3a43229d00811e1ce513624ac96f5" },
    InstitutionEntry { name: "江西省储备委员会", shenfen_id: "GFR-JX000-CB09-183645800-20260222", org_type: OrgType::Prc, duoqian_address: "0950cef8244e929f363946110a75d91e00671cb14e5e67b145d42d4826e0be9b" },
    InstitutionEntry { name: "浙江省储备委员会", shenfen_id: "GFR-ZJ000-CB0Y-452554562-20260222", org_type: OrgType::Prc, duoqian_address: "45b50263c9438e8642932bc23c1c5d86ec72dd42adcb1dea95e8204e6922dde4" },
    InstitutionEntry { name: "江苏省储备委员会", shenfen_id: "GFR-JS000-CB0T-266669398-20260222", org_type: OrgType::Prc, duoqian_address: "fcd48c7f4357b0bc6419cf3be4adbe83f9e2bd59003367ecfa7ae171e422e930" },
    InstitutionEntry { name: "山东省储备委员会", shenfen_id: "GFR-SD000-CB0A-354794960-20260222", org_type: OrgType::Prc, duoqian_address: "979570fa62d1963802150c9ed4c75ebde4f223db00420e624f11a08403a3a6cd" },
    InstitutionEntry { name: "山西省储备委员会", shenfen_id: "GFR-SX000-CB0T-700141630-20260222", org_type: OrgType::Prc, duoqian_address: "0f2a278947e933750b3cc14c9613299c7670b95dfd8ef719f9de56d290495122" },
    InstitutionEntry { name: "河南省储备委员会", shenfen_id: "GFR-HE000-CB0R-527771281-20260222", org_type: OrgType::Prc, duoqian_address: "b0f272d9ac4caeb41f463549732bbeddce3e0bf422450f5ab2627b684cb2e24b" },
    InstitutionEntry { name: "河北省储备委员会", shenfen_id: "GFR-HB000-CB04-025532397-20260222", org_type: OrgType::Prc, duoqian_address: "216ad2c3fd9715de1ae1854fd4216b3fe6f9245767575fbd855b80c87060c664" },
    InstitutionEntry { name: "湖北省储备委员会", shenfen_id: "GFR-HI000-CB0M-247491104-20260222", org_type: OrgType::Prc, duoqian_address: "d43bde789ab9b4fa011ac54fcec77047928de324e812b12be7c3d611f107c637" },
    InstitutionEntry { name: "陕西省储备委员会", shenfen_id: "GFR-SI000-CB0Q-626717092-20260222", org_type: OrgType::Prc, duoqian_address: "037afa9fa24097b480ef7d35c142e874f3ac78139cd9edfd20fca3ab0e483986" },
    InstitutionEntry { name: "重庆省储备委员会", shenfen_id: "GFR-CQ001-CB00-452250444-20260222", org_type: OrgType::Prc, duoqian_address: "fca5f44d8fe158205bb9adb859adf60f4683ec0ac0c122677517914ed220b753" },
    InstitutionEntry { name: "四川省储备委员会", shenfen_id: "GFR-SC000-CB0N-676087668-20260222", org_type: OrgType::Prc, duoqian_address: "7b0e36626b4906b36fe60cbc22376deae4b2b6b25f1dc48447cb1339a63be972" },
    InstitutionEntry { name: "甘肃省储备委员会", shenfen_id: "GFR-GS000-CB02-451145443-20260222", org_type: OrgType::Prc, duoqian_address: "86afdddf3d531f775fd46b5b6aca115bc281d06b16434b188f44a5b6e758796c" },
    InstitutionEntry { name: "北平省储备委员会", shenfen_id: "GFR-BP001-CB0C-164347900-20260222", org_type: OrgType::Prc, duoqian_address: "db80eef695bef0ef0268059a027b4d0641a4d59a11d562f1a53cd2c3587aca59" },
    InstitutionEntry { name: "海滨省储备委员会", shenfen_id: "GFR-HA000-CB02-156526094-20260222", org_type: OrgType::Prc, duoqian_address: "e58770d249bd55f63eb052e93b54557e4d565feebc284f6bb8398b238af30529" },
    InstitutionEntry { name: "松江省储备委员会", shenfen_id: "GFR-SJ000-CB0A-005282342-20260222", org_type: OrgType::Prc, duoqian_address: "d8c2177ef57b4ca651460f233cc39f7af405a5442937026a697cc4852e56e2d8" },
    InstitutionEntry { name: "龙江省储备委员会", shenfen_id: "GFR-LJ000-CB0A-105584375-20260222", org_type: OrgType::Prc, duoqian_address: "4bcca0d178ed251c23391f34d9c72214af1656c5431dbbbf8e191785a9b0d0a0" },
    InstitutionEntry { name: "吉林省储备委员会", shenfen_id: "GFR-JL000-CB0T-855212821-20260222", org_type: OrgType::Prc, duoqian_address: "9c52a4de06b27c9cca3fb4b8f2a1794f2dfdc0ee09a8a0286041218075e9be00" },
    InstitutionEntry { name: "辽宁省储备委员会", shenfen_id: "GFR-LI000-CB03-221473214-20260222", org_type: OrgType::Prc, duoqian_address: "69f2eb3f9f161ef9f469010acec759e40a9e8974fbf43249149472ed68bf43c4" },
    InstitutionEntry { name: "宁夏省储备委员会", shenfen_id: "GFR-NX000-CB0A-240866560-20260222", org_type: OrgType::Prc, duoqian_address: "e8f661615592fe19d33a8424d61b647ccdd7c4244349484d651e4851680caf27" },
    InstitutionEntry { name: "青海省储备委员会", shenfen_id: "GFR-QH000-CB0N-229555853-20260222", org_type: OrgType::Prc, duoqian_address: "5fbca2c6f277e9382747bdbbdfc170c3f83d563d3acd1a4fec3aa7ff81aca71b" },
    InstitutionEntry { name: "安徽省储备委员会", shenfen_id: "GFR-AH000-CB0Q-714959233-20260222", org_type: OrgType::Prc, duoqian_address: "53aa5754796f98f8f6fb74f0302ea381936b6b06d48c17e455bc64725e8af35b" },
    InstitutionEntry { name: "台湾省储备委员会", shenfen_id: "GFR-TW000-CB0U-188063480-20260222", org_type: OrgType::Prc, duoqian_address: "fe6d7dcc07faaae8face0c0fdd66de2933dea83d9bd0df25bd571979bdd55859" },
    InstitutionEntry { name: "西藏省储备委员会", shenfen_id: "GFR-XZ000-CB0R-085197231-20260222", org_type: OrgType::Prc, duoqian_address: "f3e4b26435892b5e0330028690498f309dc5eaec1ba91942cc0902d13c71a4df" },
    InstitutionEntry { name: "新疆省储备委员会", shenfen_id: "GFR-XJ000-CB0I-803866647-20260222", org_type: OrgType::Prc, duoqian_address: "a809b8e77ad103708a77b3be1d2277555eedbf0d433f436f9901d46bdb217c79" },
    InstitutionEntry { name: "西康省储备委员会", shenfen_id: "GFR-XK000-CB0B-810391358-20260222", org_type: OrgType::Prc, duoqian_address: "f4937d7a2c61c57cdf5079d25e0d9ff8e189b668a98b0489ab946e065a6c1c63" },
    InstitutionEntry { name: "阿里省储备委员会", shenfen_id: "GFR-AL000-CB08-769336671-20260222", org_type: OrgType::Prc, duoqian_address: "969316fc4c788f7c9e1b96cd6a33ade8f40acd759b353502f64b3a3427e569c1" },
    InstitutionEntry { name: "葱岭省储备委员会", shenfen_id: "GFR-CL000-CB0Z-914234080-20260222", org_type: OrgType::Prc, duoqian_address: "6e08fcbf5a5c3429b5c408da8b8bc558feb9581ab50b758cd5c89fd7c1db3263" },
    InstitutionEntry { name: "天山省储备委员会", shenfen_id: "GFR-TS000-CB0O-063508625-20260222", org_type: OrgType::Prc, duoqian_address: "6ce2b03f2b129a204f332da81a61b1248f53efbf08848a77a6fa39ddd3c2b8b2" },
    InstitutionEntry { name: "河西省储备委员会", shenfen_id: "GFR-HX000-CB0J-238307168-20260222", org_type: OrgType::Prc, duoqian_address: "584dc4763c2a9998f137b96e55a9984e3ccb4436aefed3667b5ee33ae4f7b9d1" },
    InstitutionEntry { name: "昆仑省储备委员会", shenfen_id: "GFR-KL000-CB00-453003140-20260222", org_type: OrgType::Prc, duoqian_address: "51041527a777faa5df81ea521fd19b1981712c9bff15056fa44fd0de2696c20e" },
    InstitutionEntry { name: "河套省储备委员会", shenfen_id: "GFR-HT000-CB0F-763975330-20260222", org_type: OrgType::Prc, duoqian_address: "44a0d06f571743e1a513d28dad6e6609445451f23c6929387372f0dc9bd761d3" },
    InstitutionEntry { name: "热河省储备委员会", shenfen_id: "GFR-RH000-CB0T-258553387-20260222", org_type: OrgType::Prc, duoqian_address: "7a2703df0624d7d7afab04a169dd04ef9a89991ee76f059c586aaf376437e653" },
    InstitutionEntry { name: "兴安省储备委员会", shenfen_id: "GFR-XA000-CB0D-997757073-20260222", org_type: OrgType::Prc, duoqian_address: "3a4d16f29220b431fd778bba9ff0d0b1e1ee8958e3b36fb22512160d6b4eca0f" },
    InstitutionEntry { name: "合江省储备委员会", shenfen_id: "GFR-HJ000-CB0C-544834501-20260222", org_type: OrgType::Prc, duoqian_address: "8ce152ac8c86e441ebcba60f515d5530492b42d9eb3335d99b526471a76d3495" },
];

// 省储行（43 个）
static PROVINCIAL_BANKS: &[InstitutionEntry] = &[
    InstitutionEntry { name: "中枢省公民储备银行", shenfen_id: "SFR-ZS001-CH1Z-572590896-20260222", org_type: OrgType::Prb, duoqian_address: "fe45d3e78fd7dce6e13715a3e30ffc52ee80551d5f40e68ef4c501c3c2985ab1" },
    InstitutionEntry { name: "岭南省公民储备银行", shenfen_id: "SFR-LN001-CH1D-067241191-20260222", org_type: OrgType::Prb, duoqian_address: "6f26889bc70faa896c2fc464c0c4a4da1cd3f3df1f4347c0d56edf9e3883dc71" },
    InstitutionEntry { name: "广东省公民储备银行", shenfen_id: "SFR-GD000-CH1S-539766913-20260222", org_type: OrgType::Prb, duoqian_address: "cffd5c331e9323b1fd5b3724a3b35804bba9492e60b63a2353c857c585e2fd63" },
    InstitutionEntry { name: "广西省公民储备银行", shenfen_id: "SFR-GX000-CH17-770836097-20260222", org_type: OrgType::Prb, duoqian_address: "df01f593daed649ebaaa8b658dd127c792c02b41df515b18df05cccb483787ee" },
    InstitutionEntry { name: "福建省公民储备银行", shenfen_id: "SFR-FJ000-CH1Y-285514007-20260222", org_type: OrgType::Prb, duoqian_address: "bec1ed0746ea6e6e24db89750fb44a76a289556ca65c84e425c0b448205e18e8" },
    InstitutionEntry { name: "海南省公民储备银行", shenfen_id: "SFR-HN000-CH1W-701494632-20260222", org_type: OrgType::Prb, duoqian_address: "da92404c22e9f2d52253e737ced41bd1cdbe83c18df0ffaed5408fd1221cae53" },
    InstitutionEntry { name: "云南省公民储备银行", shenfen_id: "SFR-YN000-CH1M-088552001-20260222", org_type: OrgType::Prb, duoqian_address: "2dbe1db434c63c032aac0772681f457506c1c022e8f43ab0d656a5f0d9e611d2" },
    InstitutionEntry { name: "贵州省公民储备银行", shenfen_id: "SFR-GZ000-CH17-073795499-20260222", org_type: OrgType::Prb, duoqian_address: "e743674d50fd8cac955958b9dd1f46b0fd92bf18be5f709de6e75c9c9b13b681" },
    InstitutionEntry { name: "湖南省公民储备银行", shenfen_id: "SFR-HU000-CH1P-721228492-20260222", org_type: OrgType::Prb, duoqian_address: "54e7e17e7b493ba360e8035f86976a5e7deef2833738fd41ba955b8794022c73" },
    InstitutionEntry { name: "江西省公民储备银行", shenfen_id: "SFR-JX000-CH1T-532829662-20260222", org_type: OrgType::Prb, duoqian_address: "c5f77d6ecc1bc1e2bfe144754355ae24b7f5b0909f15705914de87d7e6382e6b" },
    InstitutionEntry { name: "浙江省公民储备银行", shenfen_id: "SFR-ZJ000-CH19-249528657-20260222", org_type: OrgType::Prb, duoqian_address: "a97dfa62d5eca6d2f1bded65fa6528c6372e2bf34f740181beb7b5c8e5e4cc77" },
    InstitutionEntry { name: "江苏省公民储备银行", shenfen_id: "SFR-JS000-CH1C-191178842-20260222", org_type: OrgType::Prb, duoqian_address: "c7fc95907a57f04c07869d4e181d17f46393e7c1224f6d2ebf16ddfec348310d" },
    InstitutionEntry { name: "山东省公民储备银行", shenfen_id: "SFR-SD000-CH1V-887886640-20260222", org_type: OrgType::Prb, duoqian_address: "98d016ea45313719d30d171932500168ed9e3de37fa07ee9f9f6f977fdba0f79" },
    InstitutionEntry { name: "山西省公民储备银行", shenfen_id: "SFR-SX000-CH1F-755750488-20260222", org_type: OrgType::Prb, duoqian_address: "735599f633072eff9cc2074520a5db9e5aa4afdfda5d0ec2dd925b0c0c14b2a1" },
    InstitutionEntry { name: "河南省公民储备银行", shenfen_id: "SFR-HE000-CH1T-357503840-20260222", org_type: OrgType::Prb, duoqian_address: "736b13ab5bd7242d880e95507a2068d05a5ae6cd78dc72bc5d44c3f474e724d6" },
    InstitutionEntry { name: "河北省公民储备银行", shenfen_id: "SFR-HB000-CH12-172598053-20260222", org_type: OrgType::Prb, duoqian_address: "e08397c483d8962e6aea1d2ebf18ae39f7291f8918fd918eba32de54ad50c394" },
    InstitutionEntry { name: "湖北省公民储备银行", shenfen_id: "SFR-HI000-CH1W-584177104-20260222", org_type: OrgType::Prb, duoqian_address: "98d151fde59630b63b99ba5c9aa56389247ece26689b432d9ebe7baddd7d8191" },
    InstitutionEntry { name: "陕西省公民储备银行", shenfen_id: "SFR-SI000-CH1G-814942227-20260222", org_type: OrgType::Prb, duoqian_address: "58c0b0ea8fb4fa430de47c4d70030645ac3a4f464728ab1e7ab304669403a732" },
    InstitutionEntry { name: "重庆省公民储备银行", shenfen_id: "SFR-CQ001-CH1A-811483361-20260222", org_type: OrgType::Prb, duoqian_address: "072abcf96cb315ab1c654a482172429314f9f15b126c1f51d2bf1ef233e03d1f" },
    InstitutionEntry { name: "四川省公民储备银行", shenfen_id: "SFR-SC000-CH19-320507619-20260222", org_type: OrgType::Prb, duoqian_address: "e104ec87a747420fc31702551d8153f0edf0a7ac2a77a5bfe8910adc3f8b0ae9" },
    InstitutionEntry { name: "甘肃省公民储备银行", shenfen_id: "SFR-GS000-CH1U-319639307-20260222", org_type: OrgType::Prb, duoqian_address: "ea360306c0190de49513faede894fc44f827960fa8f45b33be9093800d104791" },
    InstitutionEntry { name: "北平省公民储备银行", shenfen_id: "SFR-BP001-CH19-330141933-20260222", org_type: OrgType::Prb, duoqian_address: "5b9005b8abfb70803e2b0fdbd31e494044f09e8f3bd369abbafdeb481c0e148a" },
    InstitutionEntry { name: "滨海省公民储备银行", shenfen_id: "SFR-HA000-CH1N-832919801-20260222", org_type: OrgType::Prb, duoqian_address: "3670a84c5f8a3d0e710e881d59113df7f3d8694532be797c9415e0bdd5d25a3a" },
    InstitutionEntry { name: "松江省公民储备银行", shenfen_id: "SFR-SJ000-CH17-991726244-20260222", org_type: OrgType::Prb, duoqian_address: "6842bab4d4c88d0508255d1f6e768262c1dffe5b6f31470757bebbaab37990bb" },
    InstitutionEntry { name: "龙江省公民储备银行", shenfen_id: "SFR-LJ000-CH1U-321069400-20260222", org_type: OrgType::Prb, duoqian_address: "b30f35b0013af60c12cda5c17b997957412a090e2b49987cfba291778774bd92" },
    InstitutionEntry { name: "吉林省公民储备银行", shenfen_id: "SFR-JL000-CH1Z-114671562-20260222", org_type: OrgType::Prb, duoqian_address: "9ee95711f6dc002676e3da8dc1cb9bf88669b9e12e5349636e0f2700560c0c21" },
    InstitutionEntry { name: "辽宁省公民储备银行", shenfen_id: "SFR-LI000-CH1O-060821950-20260222", org_type: OrgType::Prb, duoqian_address: "b53e53d962f192c2f081f88bbce75267ff5dd344ed94ea8220b1dfd6e4467882" },
    InstitutionEntry { name: "宁夏省公民储备银行", shenfen_id: "SFR-NX000-CH1W-927112322-20260222", org_type: OrgType::Prb, duoqian_address: "5fae794dac4b836be2dd0827f47a84f11531b9447cf6f9ffd1ac770abfda9243" },
    InstitutionEntry { name: "青海省公民储备银行", shenfen_id: "SFR-QH000-CH15-480036803-20260222", org_type: OrgType::Prb, duoqian_address: "b656dbc26f915cc6d5b872f57aaa9a6a4cb80fb899bdbe8e2e60d1a3e18a3f21" },
    InstitutionEntry { name: "安徽省公民储备银行", shenfen_id: "SFR-AH000-CH14-243470490-20260222", org_type: OrgType::Prb, duoqian_address: "efc6292e20288623f6cfe838abde86b9fe132018393717e5af3ad2f46b17b895" },
    InstitutionEntry { name: "台湾省公民储备银行", shenfen_id: "SFR-TW000-CH1O-339827620-20260222", org_type: OrgType::Prb, duoqian_address: "b2c055b85357313c990832ef61ac3d9fd1b52476a8671c23a14ca7edd6302e1b" },
    InstitutionEntry { name: "西藏省公民储备银行", shenfen_id: "SFR-XZ000-CH1A-076183922-20260222", org_type: OrgType::Prb, duoqian_address: "83d7ecc0558c66037fb4bf0b32e03ac152c44ad19c40f22f2271bcb5c5b441db" },
    InstitutionEntry { name: "新疆省公民储备银行", shenfen_id: "SFR-XJ000-CH1T-624864385-20260222", org_type: OrgType::Prb, duoqian_address: "1024ef3049018c3e045d7025bcb1db301b50dee6c3c6a42259191351b988cb3a" },
    InstitutionEntry { name: "西康省公民储备银行", shenfen_id: "SFR-XK000-CH19-727906387-20260222", org_type: OrgType::Prb, duoqian_address: "817ecdb4588004991fb7ee6cdc27c212641b278fc0b2b47cccd0ce47ba0c12ca" },
    InstitutionEntry { name: "阿里省公民储备银行", shenfen_id: "SFR-AL000-CH1Z-823361903-20260222", org_type: OrgType::Prb, duoqian_address: "30990485af39af3e37e3d802319e929091adf4c75b5848dea5de19fee495393e" },
    InstitutionEntry { name: "葱岭省公民储备银行", shenfen_id: "SFR-CL000-CH1I-930688147-20260222", org_type: OrgType::Prb, duoqian_address: "00363eb57b4ed7e22ae0f1b11f58f7d8eb17d5d9899967b160682133ca88af1c" },
    InstitutionEntry { name: "天山省公民储备银行", shenfen_id: "SFR-TS000-CH1S-351739678-20260222", org_type: OrgType::Prb, duoqian_address: "72df5a28d36d27568996779bc43b043d4fc91c31d9000e8affa2b15475aa0448" },
    InstitutionEntry { name: "河西省公民储备银行", shenfen_id: "SFR-HX000-CH1X-115163356-20260222", org_type: OrgType::Prb, duoqian_address: "043cfa9fabcd16c21b55bedd2dd88fae917f25224ba12f4ea0837fae1e4407d4" },
    InstitutionEntry { name: "昆仑省公民储备银行", shenfen_id: "SFR-KL000-CH1F-853206078-20260222", org_type: OrgType::Prb, duoqian_address: "bec71276f83ca65b5fe38748f93540e3b8c935b4c0c219813f40b4a524e87380" },
    InstitutionEntry { name: "河套省公民储备银行", shenfen_id: "SFR-HT000-CH1H-294801127-20260222", org_type: OrgType::Prb, duoqian_address: "3f9f61de83c84bdd9cdc723c878135316fbd88eb9b9a98d91ff389ddb887c4b0" },
    InstitutionEntry { name: "热河省公民储备银行", shenfen_id: "SFR-RH000-CH14-762808938-20260222", org_type: OrgType::Prb, duoqian_address: "e8fc4c4266531ac8e056f16458edcccf56e74a5e766e068a23ed95a60a832af8" },
    InstitutionEntry { name: "兴安省公民储备银行", shenfen_id: "SFR-XA000-CH1P-285320269-20260222", org_type: OrgType::Prb, duoqian_address: "e0a70ce7e5ae81e8f95f1510ebfa72da10d73116ed49249ea1cc6c96b4773e3c" },
    InstitutionEntry { name: "合江省公民储备银行", shenfen_id: "SFR-HJ000-CH1C-538936570-20260222", org_type: OrgType::Prb, duoqian_address: "8907191cf2c30e055072de592c2d29ee5539d13260e23f41f0081c50f845464d" },
];

fn entry_to_list_item(e: &InstitutionEntry) -> InstitutionListItem {
    InstitutionListItem {
        name: e.name.to_string(),
        shenfen_id: e.shenfen_id.to_string(),
        org_type: e.org_type as u8,
        org_type_label: e.org_type.label().to_string(),
        duoqian_address: e.duoqian_address.to_string(),
    }
}

fn find_entry(shenfen_id: &str) -> Option<&'static InstitutionEntry> {
    NATIONAL_COUNCILS
        .iter()
        .chain(PROVINCIAL_COUNCILS.iter())
        .chain(PROVINCIAL_BANKS.iter())
        .find(|e| e.shenfen_id == shenfen_id)
}

fn internal_threshold(org_type: OrgType) -> u32 {
    match org_type {
        OrgType::Nrc => 13,
        OrgType::Prc | OrgType::Prb => 6,
    }
}

fn joint_vote_weight(org_type: OrgType) -> u32 {
    match org_type {
        OrgType::Nrc => 19,
        OrgType::Prc | OrgType::Prb => 1,
    }
}

/// 获取治理首页机构分类列表（纯静态数据，不需要节点运行）。
#[tauri::command]
pub async fn get_governance_overview() -> Result<GovernanceOverview, String> {
    Ok(GovernanceOverview {
        national_councils: NATIONAL_COUNCILS.iter().map(entry_to_list_item).collect(),
        provincial_councils: PROVINCIAL_COUNCILS.iter().map(entry_to_list_item).collect(),
        provincial_banks: PROVINCIAL_BANKS.iter().map(entry_to_list_item).collect(),
        warning: None,
    })
}

/// 获取指定机构的详细信息（管理员列表、余额等需要节点运行）。
#[tauri::command]
pub async fn get_institution_detail(
    app: AppHandle,
    shenfen_id: String,
) -> Result<InstitutionDetail, String> {
    let entry = find_entry(&shenfen_id)
        .ok_or_else(|| format!("未知的机构 shenfenId: {shenfen_id}"))?;

    // 在阻塞线程中执行 RPC 查询
    let shenfen_id_clone = shenfen_id.clone();
    let duoqian_address = entry.duoqian_address.to_string();
    let org_type = entry.org_type;
    let name = entry.name.to_string();

    tauri::async_runtime::spawn_blocking(move || {
        let mut warnings: Vec<String> = Vec::new();

        // 检查节点是否运行
        let status = home::current_status(&app)?;
        let (admins, balance_fen) = if status.running {
            let admins = match institution::fetch_admins(&shenfen_id_clone) {
                Ok(a) => a,
                Err(e) => {
                    warnings.push(format!("查询管理员失败: {e}"));
                    Vec::new()
                }
            };
            let balance = match institution::fetch_balance(&duoqian_address) {
                Ok(b) => b.map(|v| v.to_string()),
                Err(e) => {
                    warnings.push(format!("查询余额失败: {e}"));
                    None
                }
            };
            (admins, balance)
        } else {
            warnings.push("节点未运行，无法查询链上数据".to_string());
            (Vec::new(), None)
        };

        Ok(InstitutionDetail {
            name,
            shenfen_id: shenfen_id_clone,
            org_type: org_type as u8,
            org_type_label: org_type.label().to_string(),
            duoqian_address,
            balance_fen,
            admins,
            internal_threshold: internal_threshold(org_type),
            joint_vote_weight: joint_vote_weight(org_type),
            warning: if warnings.is_empty() {
                None
            } else {
                Some(warnings.join("；"))
            },
        })
    })
    .await
    .map_err(|e| format!("institution detail task failed: {e}"))?
}

/// 通过 shenfenId 查找机构名称（供 proposal 模块反查用）。
pub(crate) fn find_institution_name(shenfen_id: &str) -> Option<String> {
    find_entry(shenfen_id).map(|e| e.name.to_string())
}

/// 获取提案分页列表（需要节点运行）。
#[tauri::command]
pub async fn get_proposal_page(
    app: AppHandle,
    start_id: u64,
    count: u32,
) -> Result<proposal::ProposalPageResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || proposal::fetch_proposal_page(start_id, count))
        .await
        .map_err(|e| format!("proposal page task failed: {e}"))?
}

/// 获取单个提案完整信息（需要节点运行）。
#[tauri::command]
pub async fn get_proposal_detail(
    app: AppHandle,
    proposal_id: u64,
) -> Result<proposal::ProposalFullInfo, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || proposal::fetch_proposal_full(proposal_id))
        .await
        .map_err(|e| format!("proposal detail task failed: {e}"))?
}

/// 获取 NextProposalId（需要节点运行）。
#[tauri::command]
pub async fn get_next_proposal_id(app: AppHandle) -> Result<u64, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询提案 ID".to_string());
    }
    tauri::async_runtime::spawn_blocking(proposal::fetch_next_proposal_id)
        .await
        .map_err(|e| format!("next proposal id task failed: {e}"))?
}

/// 获取机构活跃提案 ID 列表（需要节点运行）。
#[tauri::command]
pub async fn get_institution_proposals(
    app: AppHandle,
    shenfen_id: String,
) -> Result<Vec<proposal::ProposalListItem>, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let ids = proposal::fetch_active_proposal_ids(&shenfen_id)?;
        let mut items = Vec::new();
        for id in ids.iter().rev() {
            match proposal::fetch_proposal_page(*id, 1) {
                Ok(page) => items.extend(page.items),
                Err(_) => {}
            }
        }
        Ok(items)
    })
    .await
    .map_err(|e| format!("institution proposals task failed: {e}"))?
}

/// 匹配到的管理员钱包信息。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminWalletMatch {
    pub address: String,
    pub pubkey_hex: String,
    pub name: String,
}

/// 检查已导入冷钱包中哪些是指定机构的管理员（需要节点运行）。
#[tauri::command]
pub async fn check_admin_wallets(
    app: AppHandle,
    shenfen_id: String,
) -> Result<Vec<AdminWalletMatch>, String> {
    // 读取已导入的冷钱包
    let wallet_list = cold_wallets::get_cold_wallets(app.clone())?;
    if wallet_list.wallets.is_empty() {
        return Ok(Vec::new());
    }

    // 检查节点状态
    let status = home::current_status(&app)?;
    if !status.running {
        return Ok(Vec::new());
    }

    let wallets = wallet_list.wallets;
    tauri::async_runtime::spawn_blocking(move || {
        // 查询链上该机构的管理员列表
        let admins = match institution::fetch_admins(&shenfen_id) {
            Ok(a) => a,
            Err(_) => return Ok(Vec::new()),
        };
        if admins.is_empty() {
            return Ok(Vec::new());
        }

        // 匹配冷钱包公钥与管理员列表
        let mut matches = Vec::new();
        for wallet in &wallets {
            let normalized = wallet.pubkey_hex.to_ascii_lowercase();
            if admins.iter().any(|a| *a == normalized) {
                matches.push(AdminWalletMatch {
                    address: wallet.address.clone(),
                    pubkey_hex: wallet.pubkey_hex.clone(),
                    name: wallet.name.clone(),
                });
            }
        }
        Ok(matches)
    })
    .await
    .map_err(|e| format!("check admin wallets task failed: {e}"))?
}

/// 构建投票签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_vote_request(
    app: AppHandle,
    proposal_id: u64,
    pubkey_hex: String,
    approve: bool,
) -> Result<signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_vote_sign_request(proposal_id, &pubkey_hex, approve)
    })
    .await
    .map_err(|e| format!("build vote request task failed: {e}"))?
}

/// 构建联合投票签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_joint_vote_request(
    app: AppHandle,
    proposal_id: u64,
    pubkey_hex: String,
    shenfen_id: String,
    approve: bool,
) -> Result<signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_joint_vote_sign_request(proposal_id, &pubkey_hex, &shenfen_id, approve)
    })
    .await
    .map_err(|e| format!("build joint vote request task failed: {e}"))?
}

/// 验证签名响应并提交投票（通用，支持内部和联合投票）。
///
/// call_data_hex 为完整的 SCALE call data hex（不含 0x 前缀）。
#[tauri::command]
pub async fn submit_vote(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    call_data_hex: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交投票".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data = hex::decode(&call_data_hex)
            .map_err(|e| format!("call_data 解码失败: {e}"))?;
        signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit vote task failed: {e}"))?
}

/// 查询用户投票状态（需要节点运行）。
#[tauri::command]
pub async fn check_vote_status(
    app: AppHandle,
    proposal_id: u64,
    pubkey_hex: String,
    shenfen_id: Option<String>,
) -> Result<proposal::UserVoteStatus, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法查询投票状态".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        proposal::fetch_user_vote_status(
            proposal_id,
            &pubkey_hex,
            shenfen_id.as_deref(),
        )
    })
    .await
    .map_err(|e| format!("check vote status task failed: {e}"))?
}

/// 构建创建转账提案签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_propose_transfer_request(
    app: AppHandle,
    pubkey_hex: String,
    shenfen_id: String,
    org_type: u8,
    beneficiary_address: String,
    amount_yuan: f64,
    remark: String,
) -> Result<signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_propose_transfer_sign_request(
            &pubkey_hex,
            &shenfen_id,
            org_type,
            &beneficiary_address,
            amount_yuan,
            &remark,
        )
    })
    .await
    .map_err(|e| format!("build propose transfer request task failed: {e}"))?
}

/// 构建开发期 runtime 直接升级签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_developer_upgrade_request(
    app: AppHandle,
    pubkey_hex: String,
    wasm_path: String,
) -> Result<signing::VoteSignRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        signing::build_developer_upgrade_sign_request(&pubkey_hex, &wasm_path)
    })
    .await
    .map_err(|e| format!("build developer upgrade request task failed: {e}"))?
}

/// 验证签名响应并提交开发期 runtime 直接升级。
#[tauri::command]
pub async fn submit_developer_upgrade(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    wasm_path: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交升级".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data = signing::build_developer_upgrade_call_data(&wasm_path)?;
        signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit developer upgrade task failed: {e}"))?
}

/// 构建 propose_runtime_upgrade 签名请求的返回值（包含 SFID 快照数据）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposeUpgradeRequestResult {
    pub request_json: String,
    pub request_id: String,
    pub expected_payload_hash: String,
    pub sign_nonce: u32,
    pub sign_block_number: u64,
    pub eligible_total: u64,
    pub snapshot_nonce: String,
    pub snapshot_signature: String,
}

/// 构建 Runtime 升级提案签名请求 QR JSON（需要节点运行）。
#[tauri::command]
pub async fn build_propose_upgrade_request(
    app: AppHandle,
    pubkey_hex: String,
    wasm_path: String,
    reason: String,
) -> Result<ProposeUpgradeRequestResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法构建签名请求".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let (sign_result, snapshot) =
            signing::build_propose_runtime_upgrade_sign_request(&pubkey_hex, &wasm_path, &reason)?;
        Ok(ProposeUpgradeRequestResult {
            request_json: sign_result.request_json,
            request_id: sign_result.request_id,
            expected_payload_hash: sign_result.expected_payload_hash,
            sign_nonce: sign_result.sign_nonce,
            sign_block_number: sign_result.sign_block_number,
            eligible_total: snapshot.eligible_total,
            snapshot_nonce: snapshot.snapshot_nonce,
            snapshot_signature: snapshot.signature,
        })
    })
    .await
    .map_err(|e| format!("build propose upgrade request task failed: {e}"))?
}

/// 验证签名响应并提交 Runtime 升级提案。
#[tauri::command]
pub async fn submit_propose_upgrade(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    wasm_path: String,
    reason: String,
    eligible_total: u64,
    snapshot_nonce: String,
    snapshot_signature: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let call_data = signing::build_propose_runtime_upgrade_call_data(
            &wasm_path,
            &reason,
            eligible_total,
            &snapshot_nonce,
            &snapshot_signature,
        )?;
        signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit propose upgrade task failed: {e}"))?
}

/// 验证签名响应并提交转账提案（专用命令，后端构建 call data）。
#[tauri::command]
pub async fn submit_propose_transfer(
    app: AppHandle,
    request_id: String,
    expected_pubkey_hex: String,
    expected_payload_hash: String,
    shenfen_id: String,
    org_type: u8,
    beneficiary_address: String,
    amount_yuan: f64,
    remark: String,
    sign_nonce: u32,
    sign_block_number: u64,
    response_json: String,
) -> Result<signing::VoteSubmitResult, String> {
    let status = home::current_status(&app)?;
    if !status.running {
        return Err("节点未运行，无法提交提案".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let amount_fen = (amount_yuan * 100.0).round() as u128;
        let institution_id = storage_keys::shenfen_id_to_fixed48(&shenfen_id);
        let beneficiary_bytes = signing::decode_ss58_to_pubkey(&beneficiary_address)?;
        let remark_bytes = remark.as_bytes();
        let remark_compact = signing::encode_compact_u32_pub(remark_bytes.len() as u32);

        let mut call_data = Vec::new();
        call_data.push(19u8);
        call_data.push(0u8);
        call_data.push(org_type);
        call_data.extend_from_slice(&institution_id);
        call_data.extend_from_slice(&beneficiary_bytes);
        call_data.extend_from_slice(&amount_fen.to_le_bytes());
        call_data.extend_from_slice(&remark_compact);
        call_data.extend_from_slice(remark_bytes);

        signing::verify_and_submit(
            &request_id,
            &expected_pubkey_hex,
            &expected_payload_hash,
            &call_data,
            sign_nonce,
            sign_block_number,
            &response_json,
        )
    })
    .await
    .map_err(|e| format!("submit propose transfer task failed: {e}"))?
}
