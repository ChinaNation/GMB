//! 国储会 + 43 个初始省储会节点的常量=reserve_nodes_const.rs
//! 每一个节点包括：储会ID（pallet_id）、节点名称（node_name）、交易账户（pallet_address）、节点地址（p2p_bootnodes）、管理员列表（admins）；
//! 其中 admins 为该节点的创世管理员公钥数组，用于多签权限控制，可通过内部投票更换。

use sp_core::H256;

/// 单个储委会的常量结构
#[derive(Debug, Clone)]
pub struct ReserveNodeConst {
    pub pallet_id: &'static str,
    pub node_name: &'static str,
    pub pallet_address: [u8; 32],
    pub p2p_bootnodes: &'static [&'static str],
    pub admins: &'static [[u8; 32]],
}

/// 所有国储会+省储会节点数组
pub const RESERVE_NODES: &[ReserveNodeConst] = &[
    // ======国储会=======
    ReserveNodeConst {
        pallet_id: "nrcgch01",
        node_name: "国家储备委员会权威节点",
        pallet_address: hex!("6d6f646c6e726367636830310000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/nrcgch01.wuminbi.com/tcp/30333/p2p/12D3KooWHepcMGD3h9VC1XNWmrac3pXo63RimV5jhTU2nC2TLAyS"],
        admins: &[
            hex!("9aa1e0672efcf2e186a6237da9fa706279e2c1d785212c48334bde7cae400215"),hex!("04129fc398266d11a12e6f9d673a94ca7286f78ab1539d4a16842977b2fdbc0f"),hex!("9ecdc2bbf31054b4f71d36255f0befba001934ec372ac7042353dfdba63ad04a"),hex!("dc9a9c9a914f5cd090d8d2ac03c6e1e3ab34803c4f2c9ddda6689404261ee736"),
            hex!("541ba27c96c9f49565e5205baa8cbd62df9409b076fdd5e49983165b81e26110"),hex!("828a0880017322a983585bfd5b8132207375b016a87ceee8a7eab5fe86ee515f"),hex!("de234d6832cd999d3d57c289dfe99c9c14e4477e5f4f91f330e518e473028b11"),hex!("60de639ac26697bd2a3ede622ffc333ba5ff4d450aad4f4c0ef4e24051c6b705"),
            hex!("aa3efa525e5b28c215f185cb6e978c1daef37a429a8c7b0016b5b6ca8acecd20"),hex!("a618a20a4367ee2f8a810ecf63817d7bc40b6ee9b5dbdd0d934cfce967865242"),hex!("90a611abaaeaabc694a61aeb8dcd76945d6cc79d677651f05699a3557cd4e735"),hex!("cc8954ef347a2fb4b2e569a69a31bf01caad2084ec67a4788d58fe1152efa53b"),
            hex!("1e833d814e9a75e93de4bcf9319f59afd1f7c342b2e8a505bb2f9b569d39986c"),hex!("9a05884f3d28823a8afbb7d91d3c76ece864d32c3175eb0635af0481f40a422b"),hex!("5c967c2bc023dd2cad6734febd672f057c7fa2f4e4da7ec56f269bc1f979ba5c"),hex!("101082fe5e5e2ff7d9044d622c8eff7ccd1311a42b429b58a2eddd3edcbbe359"),
            hex!("aae25e9ac45fd3988db347a2e0893b719e557fdc0482cb5447d00112ba5bdf6a"),hex!("241c8760ab817c243feb98359529898b295a726e71c10dac82fe14b5f387ea67"),hex!("b63f7c64a415525c34d8484536fceec1f155d24b67b43393d0cce37911df622e"),
        ],
    },

    // ======中枢省储委会======
    ReserveNodeConst {
        pallet_id: "prczss01",
        node_name: "中枢省储备委员会权威节点",
        pallet_address: hex!("6d6f646c7072637a737330310000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prczss01.wuminbi.com/tcp/30333/p2p/12D3KooWPjWNXvCzPv6PPuiGnF3J5uToW3ySfaB7rKkwUrN2CALv"],
        admins: &[
            hex!("36e64c89c71651e470b897c0014d9c64edfdc5a27d70acd80020d1cb1cdfc614"),hex!("de7aaa6ddcbd4f57b553040857a67f29345e22dfd3846cdefaa01e4b3a14346b"),hex!("aedee0cf57ec6e2013f64a70c5f8f6b2e40ebfe679733ffa0fe13022c0a86461"),
            hex!("84362f6e665f1d2e6ec841ebd0a9acdffd3fa2b52ccfd281a001ef0800fbf941"),hex!("8266a798437cd55268dd09664ef11714fe37f5b59068e65f4923b1dde6f3826e"),hex!("886886be3990a175997db56efaf23579569b133c2ac8b6eafcded90440a6b700"),
            hex!("a4d346bc4191a532f694e80a2bbf736c8ee98eaeee0c0e651bebbea14ac5b367"),hex!("fefa8b9ffbce4f5579c7cc4a0dcddf882d14a6b9f81d5cde104a69ace6df8b00"),hex!("04f5a8b7bcba353972d608021c14761aa5732ea96514afb7c3e1786f3e8ee67f"),
        ],
    },

    // ======岭南省储委会======
    ReserveNodeConst {
        pallet_id: "prclns02",
        node_name: "岭南省储备委员会权威节点",
        pallet_address: hex!("6d6f646c7072636c6e7330320000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prclns02.wuminbi.com/tcp/30333/p2p/12D3KooWD9EpWCRceAQBc5rxq8pMS75ke9ovDyqAF8ZjoVQVD3tt"],
        admins: &[
            hex!("c24043655f86afe9e38f811eb9a5ccd3fadb0c01461ba5c50c4650c91e1cbd6c"),hex!("b2b9d0eea8f6c52e02fd110fa117fbec2c54d622c5075407be6bc4fc71f8bd50"),hex!("8e0eca16c4220dcfef2f08fb69941328b8e5c9b3d71296b3c5efd6bb8c9a495a"),
            hex!("747caff30a59ea05c625670dff110a6be936860fd5f3aede57db9d660eb7bd69"),hex!("4a3891f709995c8ea09f4d7f1eb8c3f371d12a033fda1dff99b6f71e23491c44"),hex!("6aa7e93e4154e8922eb98d6e7e9c4404058b0478ea6baed74aef8b41fb243c4d"),
            hex!("f096c7cba81e3be77a96e56d1c7df28c81d5009c758ff0566116d4e83bb73613"),hex!("e4bb764099f969485273c6689dcfd37618269ed804a26d4c1c733c41dd6ffb0d"),hex!("e8c9352f83e9212909116979f2ee79bb90196b04c49e1cda64e65f8d3bd3427c"),
        ],
    },

    // ======广东省储委会======
    ReserveNodeConst {
        pallet_id: "prcgds03",
        node_name: "广东省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726367647330330000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcgds03.wuminbi.com/tcp/30333/p2p/12D3KooWJKT8iE9guv4wfem1L9Xd91bNC9CTcLmZyRgUuWkpmEqf"],
        admins: &[
            hex!("9ef3f954efcadd7019c09d1648f9f00db94d31773281a764493a602107fab653"),hex!("e44c7e0f27c0ac64b00126b7091bd8d35fe0054100147f197bea17fc48bb5828"),hex!("da5fd44b006fef980c014a5334f81be7174bee7d515e1321d66cbd368f2d0167"),
            hex!("6ad0baf701b8c9eb206470793ca6ad0272b2db76a4a9edc15c8d947ca84b6416"),hex!("0e34353ea0ee0a3b1b6f4ddfcda556fd8014e2f87aa8a7ae3dace9ed246bf42d"),hex!("b08fe66125c71531f18d9986ec84458c990dc940b08d22c88c617bcea606624a"),
            hex!("9e1a986cf5c984b2fce1c83ab31e85884fc34ce82f6a442c9a1c2975d0a4d71f"),hex!("463703eb3aea7dcb5dc10b0e39b9ce19f0b7fddf8f0cef1e672170dbd7db3b37"),hex!("b44529f976d5b5d7f553b668863d1b7488ff6d9dade6b8ae892c45ca0893df55"),
        ],
    },

    // ======广西省储委会======
    ReserveNodeConst {
        pallet_id: "prcgxs04",
        node_name: "广西省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726367787330340000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcgxs04.wuminbi.com/tcp/30333/p2p/12D3KooWAxCE4TpEkDKibtQBzFtEuTAvxrDp1JXabhXPY7tAp9qx"],
        admins: &[
            hex!("2ccc2bcf757337b58afcc51a0bf97d49884e7e6d43f3adfe812299f6a1374820"),hex!("76771cdae903f483004a5dc5583ddce579068b6eea7c4cafe4592a0303e84c72"),hex!("9e86d42aad2fc479d980fc2df04e4c09f2294208b2af6b95762d5d0de6681352"),
            hex!("e6d413e1ce667b803336548a0c757f1a76566f83846a46fd62030ba21f09dc72"),hex!("5483c5e3f9f2263c844a1a1a2f44cc5cdd2ba25c80408fbb34a6d0be87a03473"),hex!("6ad2805dc755a46cba50255a11c32f374d756695afefceaed816b21b002c0c00"),
            hex!("22522e0f0ef5c685993dded9e207655656f4adff0f89ae76f680d0a243ed3f43"),hex!("f4dfa707f19058f0256d1f5dff92a2d135e064e72125584129e10dbccc25105a"),hex!("22a186a66e31e38a0c359edd591a562af113dea51898ed2543d542c7c3b9001e"),
        ],
    },

    // ======福建省储委会======
    ReserveNodeConst {
        pallet_id: "prcfjs05",
        node_name: "福建省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263666a7330350000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcfjs05.wuminbi.com/tcp/30333/p2p/12D3KooWJdGUANuEpVCmarfH2gi23GodbbbBBabuw9Eb4raBabt8"],
        admins: &[
            hex!("4ab76c9f6d49a57b1e869265716911f08195f5a744cbf4d103a336b7c38bbb4e"),hex!("ca3d90624d3b9cad85c8041fab2301be4c87cedecba181ff68802d41c1d5f50a"),hex!("5a6fae4ed279becd917947366f09c02c1073441d281baeb7099c96fe7bac9457"),
            hex!("8211f76bfc89a30b4ff66a944404a43bf97a3665732c8e8eb066bc83eb2c6f24"),hex!("8c48b79487bc93d50dfbc4e2b67f0792ee94ee46ba795507e2a5d7ef5af8d42b"),hex!("f87b77ded40cf9564834018d22f24de8ea19eba8852ae355fd8062b44e44a613"),
            hex!("44867f617150a2d4913b1baa96dc7263c5f057d596c8fc6639734cdcbddb7024"),hex!("b4162e777ab66747dab3114ec9b94c81c5b99619d75b3cab14b78d3c69101d5d"),hex!("30ada73540234174af81dd83708858c4810bc585932927a9dc6c4999acb9e958"),
        ],
    },

    // ======海南省储委会======
    ReserveNodeConst {
        pallet_id: "prchns06",
        node_name: "海南省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263686e7330360000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prchns06.wuminbi.com/tcp/30333/p2p/12D3KooWEhovD6QmFbZZGBS7pkwKZinfGZPCAKvyEGGDqkja8HDa"],
        admins: &[
            hex!("f433269884a6d155667b13eccddf666f0d47cb319f2db7028d13243d3a98cd04"),hex!("38bbd516c9594e8f15bc58ddd5ce6744deb228770cbbe5d8a50eced8170b3537"),hex!("b6a1c4aa905ab8302df0f216784358293e2ba27e4f17e90932df7cb2c289d71f"),
            hex!("26de194b359b61124d3e9cfc0b50a8782e0360f97ee28a92cb29a59e89c6fa27"),hex!("92c01f7d993e04c95617d91303800a732fe7a1d0a81f000581b3a517b35e5d71"),hex!("101f7c4ae2ab22b5cff99056589a2064e991b686d437f188042a981e8b9b1c6b"),
            hex!("82ac8e979ce61d8910fb242aa9a96f0cf6e1f391bcc920179c102da2279b6773"),hex!("4009fafccfa7116cb6d77fe00019318813518675cd026c2ca5d155e3dae7eb41"),hex!("d81c2c80dc70998ff6cc62ec22af159315f4928a0b1ee075e46325dbdcc57067"),
        ],
    },

    // ======云南省储委会======
    ReserveNodeConst {
        pallet_id: "prcyns07",
        node_name: "云南省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263796e7330370000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcyns07.wuminbi.com/tcp/30333/p2p/12D3KooWB7kZKwKEPFDo7DToUeFHeyZCJWXUR1wUN1t6uW7mFr2Z"],
        admins: &[
            hex!("b8fc3fa57be8aa1fc4a520aa1d5432a947845b77a08fe169f4f7a03511eb526c"),hex!("f097b4960e84d71c0a4fadd91f9d113a8ee8c51cd0286fcc00e55c18e188eb73"),hex!("2e5829df2acf77b37c414159c5828a295e4a8add871d9563a5757f83e345056d"),
            hex!("3a6db862069d8b2590cd6dc87f46e1e4943d4d50872d002b2eb70ec77b01446d"),hex!("b44f74d8b3f3e1145abfcb141b038edaf5a1397540e0084ba96cdc456a586503"),hex!("2ae0541bb04b58c83bf0d266ee2305e124a22162c96852fedeaa8041d6935434"),
            hex!("e6e68601ba10559f69513122ae8da3092546b0ab2f5bd84d3fb7bf7ccdf4fb57"),hex!("c40fe8e9d568feed2be4b1301e3e874ed7905ce7aaf31785fdf38bd66ed06115"),hex!("8e4dd3a52f884660231f35b7bd430bc1c0a0ce4be00e9b22fb7f67d0994d0704"),
        ],
    },

    // ======贵州省储委会======
    ReserveNodeConst {
        pallet_id: "prcgzs08",
        node_name: "贵州省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263677a7330380000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcgzs08.wuminbi.com/tcp/30333/p2p/12D3KooWC7t4V1Z2aQWS9HikBdXQgXEaTqeZ5YD78cnxtYBDn31M"],
        admins: &[
            hex!("e0fb43daac7243a64e90b95250e4ffac3d47549c72b53b086785c902365ed148"),hex!("28e876572c7db085cc95436a3a4f365f601836ebac52a1c191a4a44eb50de841"),hex!("3216ccf91dfc6db934068b4c08eda97b64ecab09970116706e190b7bb0ec0e77"),
            hex!("bac3f9f08d383b651c5e210a0ef5bacada88224522cb09eda4b29b6b1406bb3c"),hex!("f09813b5b8c0e001083755ba2947d8b6beaf630cc9ffa03a4c51855e13e1c708"),hex!("9c95388b5d4c14b86a8597262a6d4a14189e3b695280a29cde0ae4fa7670a254"),
            hex!("3c52537db96f06b4869d5abfd5ccfd6e10960aca40718a84631a6cbce101d11a"),hex!("f6a5fa84c98a479d4c00918c9e021c777ecde8cdcff02333cecdf8e2856cfd4a"),hex!("c8479fd45f2ed2834b0dc745c8963876d45d235ebfd7c45585a81e67f8ca7d00"),
        ],
    },

    // ======湖南省储委会======
    ReserveNodeConst {
        pallet_id: "prchns09",
        node_name: "湖南省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263686e7330390000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prchns09.wuminbi.com/tcp/30333/p2p/12D3KooWHS6G18ZtqiCGFYxb3CdvXT3Hb3zds8zknuWPCsdkFPPL"],
        admins: &[
            hex!("7ca56451d3543ee5740951272dcb89e40891ed4f0fad3a4bb097de86705e847c"),hex!("c8d10b00136eaadf749e56a99a9016c283adcb60eb36b742077b845128cfc517"),hex!("4293d2ce86b81783e76ef44404fcb2eab3b2f9286d4ac8d51cc61a991ceeb964"),
            hex!("4c2dfdb6f6e493429a427f47f157a22639741a063be3bb1d770625800686df47"),hex!("94ac05bb0fee7acf36f1b26901bc2ac7bb99f4e67b53af770558c309ac20d430"),hex!("f07e21609e2d10dd0528b5e2b6aef997389bfaad306303594b6247ec2cf6bf43"),
            hex!("7ade4cafc4f15c6e0eea7401f66972f04330ecbab86b203588794b9b00688e2d"),hex!("96730343012143a2d29d0c551a28751fc49960bc7437e0161c8b6ee32267d34c"),hex!("d62723624c8adb06189258b2cfbfc163baa8aa8f6b47c42cba7ecc38b61fa261"),
        ],
    },

    // ======江西省储委会======
    ReserveNodeConst {
        pallet_id: "prcjxs10",
        node_name: "江西省储备委员会权威节点",
        pallet_address: hex!("6d6f646c7072636a787331300000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcjxs10.wuminbi.com/tcp/30333/p2p/12D3KooWNpANUi6qmJCJXkMzyAMzjf4nY9wUdkAbwcGRJgikSY13"],
        admins: &[
            hex!("443e0e4a4a215622010f4410f77d5f154721c1a10e9f951591e0481023447322"),hex!("a00c9599a98af6ecc0eeb88875d6b1e4cde4d40e7120c2742d4cd560fcfbf736"),hex!("9c6becc31ad6592cd526a653e5f5baadae85fbfc02ef319ead983df19d39d174"),
            hex!("5ae4fd458886b6b5dd1bfd364e345889a3a8272e99fffb21b412b20cab4c4c76"),hex!("0264842cd3986cf76bdb3a2d7fdcbf966a820991ff31f3e5db47bc1538e63959"),hex!("d69357842e50dd4ca1300be0f5bf302548b0afeaf3ce66dece987c5a43b63a20"),
            hex!("6409493eb287f2f7e6baab6eae4db658233f28f25d31c704e8ffceea0f356c5b"),hex!("945df5590191c63e7d38ee0be1f0cfa23e936b2cc6320d7f52d1373a962dca53"),hex!("2ea1e91d61bcfe383cef6a18385771e9960f40568f99dd872530d343ceb77f72"),
        ],
    },

    // ======浙江省储委会======
    ReserveNodeConst {
        pallet_id: "prczjs11",
        node_name: "浙江省储备委员会权威节点",
        pallet_address: hex!("6d6f646c7072637a6a7331310000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prczjs11.wuminbi.com/tcp/30333/p2p/12D3KooWKLAEv8qEicjGX3MF667gqGF8Lf1iEATskv61pRdGaxS4"],
        admins: &[
            hex!("962634d0a5d6f037d3571742d5166ceb496249667b4b767ca9f468e23d03ed24"),hex!("aa62b7f79b26f549f996a7d781dd3f8001e2bef9b5542d8816436aeda32ab704"),hex!("84a82b7a35c126a70eda469d46e16f965e9f3bf2eb916e4a9ffcd9cf2c7f7728"),
            hex!("2e9010e9c6d344ca3eefaec7eb084b2a89b4712b7e114ff0fec1fca56c75777c"),hex!("c21d9feb2e79597ee74d7bc1450ff139fef7948061f3c8e4c0cc34460cca3e7a"),hex!("0cf4018567e3be84ff3f6048efcc18ddc7cdedd8ae1a6b6d872b8fe1ab75555c"),
            hex!("702f2f1348a54d58ca42e67e37bc20048bfc51cfe52bf17cbcaada07280ad45c"),hex!("12d801baa21e0b568d8e8dac9fd696bb6c6389a6bb59e2da1daf07699600234c"),hex!("0c1528d3146f26f8edb078ec3d6db9a03c8f46c653419ca73a52750c363ab578"),
        ],
    },

    // ======江苏省储委会======
    ReserveNodeConst {
        pallet_id: "prcjss12",
        node_name: "江苏省储备委员会权威节点",
        pallet_address: hex!("6d6f646c7072636a737331320000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcjss12.wuminbi.com/tcp/30333/p2p/12D3KooWQqjnQ8wLx6qNX94PoJGZgEJkgyCA3G5ck3zetcpuQp7f"],
        admins: &[
            hex!("beab132b103ef629f7e4015df7510180cee2e61b034cd0e91f888201828ce34f"),hex!("905af596cfc37f099038c89110b02157d1f76c17bb959dcb7e7cfdc31e4d570e"),hex!("86b5bc1043de6391ab23178662025d178defa1c292c650f70d94d8ea0984273a"),
            hex!("bcd72bd3d559cf3e8bf5482a3bbccd014a7897c646f6f23a586857970b31c250"),hex!("12e65b405b02bfb5e758a07f6ca86152ec6ce4daebe137dfc4aae486571cda10"),hex!("300d0ea667ce2d6321c540ac1c1c1d8e4c0bd19c36dd7ca1236ea8903d721f2b"),
            hex!("685b76564443e28dafb4b24945cd526bfe2acc7752749398d9133edd2b30ad79"),hex!("229aee961eeacf5b47febaa547344fad83540e94f17bf93f64dae7da80273f1a"),hex!("269624697ebb13975adc141321d6064f181eeafbe0c2fc04215f1bd0f1ad9328"),
        ],
    },

    // ======山东省储委会======
    ReserveNodeConst {
        pallet_id: "prcsds13",
        node_name: "山东省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726373647331330000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcsds13.wuminbi.com/tcp/30333/p2p/12D3KooWFgD8cFDqherjpiuRkHwHfAcCwaqXcBjTS2G3LkwUBTsq"],
        admins: &[
            hex!("34917d22a17b566faf9af6787385a689b1f64e8acd1da183132dd28b0dc85c7c"),hex!("965b89310f32bba16311632d62814c06bf1b516410c7647e4eb1d935eac54336"),hex!("e0e0c9fb6627ffcb22cedd547dd969a83047f2e4a7d4c9218968a9178ee3ae78"),
            hex!("d8b3e6f26d7bb9c9e0df74a001cd62a4f98552ba90ad3f8163e54ca68c574b02"),hex!("003c6ace0ee2eea042863608b4df8f67af887292db10e37663c7341cc01efa5e"),hex!("40dff4ecc577ffd7bf979ad6df4fca4ff4a1aac9a057eacc1ee6ef478c622744"),
            hex!("4a642264cebd402b710e101097266756f9fac429aaf465d7990572ca5745cb3b"),hex!("96a30f7e19a3100b2d942a1e42d5f6be94522d9bf0a191fef1ef63ef618c7918"),hex!("0ef104ec017a3b287b4d93be05e467a190b8c5d5f4283b09adaf7c5698809705"),
        ],
    },

    // ======山西省储委会======
    ReserveNodeConst {
        pallet_id: "prcsxs14",
        node_name: "山西省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726373787331340000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcsxs14.wuminbi.com/tcp/30333/p2p/12D3KooWQY3DEaJy9wEBE2bQ9gG1B8XByfVaz839jf1ov75kRmD9"],
        admins: &[
            hex!("0420f71558255ad2eb00e00aeb82e4c31e4d0618f71e7fd2480ae5b7acebc071"),hex!("929eb08e413356702c791556d154e3df085d04d536bf2133dfaebaca4d960676"),hex!("72042d75896d2f9eeb161c02e313a53850a18f18d329d37c2c7adcf9f7289128"),
            hex!("6a65ae442aea8394c7bab01a7424df8a453f031617d3fda076e26044f938a702"),hex!("26e5e68b3a97ca43a356de4336747c64b5e564836c1fc36c61d3a6d9c02df259"),hex!("6871c011b20d4010fca2c9dc0c2449655b65df4a4bea5e46b8f501c21d8e1d27"),
            hex!("d27c35bc267be3c0fa67becddca2381ac48ba43f89efbab3620bd25fd1504a1d"),hex!("f4d2277b323e1fd313b7a02f59ef11f9b431c036014ee27bbff52bd5e925cf15"),hex!("868a05cd196dbaad224c45c77c557fb3b237163d2df369fab3080075aca60008"),
        ],
    },

    // ======河南省储委会======
    ReserveNodeConst {
        pallet_id: "prchns15",
        node_name: "河南省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263686e7331350000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prchns15.wuminbi.com/tcp/30333/p2p/12D3KooWSkKBEJ2KZXckFhzLvrqqbhpq4PVKeFuWsxdTF7hfzoGc"],
        admins: &[
            hex!("ce08dab991bd8b5cd37bc7a11d360ca5b5157bc021606088901d6ff685971a24"),hex!("82f7cf995d842c30baed04d0c05984a65b52eaea87e0ecc7fa18c8bd3dd5d336"),hex!("e87b795d3bfe235d706913e453949538012d2f5fe819aa7fbd8974991432db72"),
            hex!("4c0296f4c16354e292d66b26f68b84a3f46a1e6ebb58080989226a98ce5cc450"),hex!("7e560db5dda74021cd0ef40e807230dd8085df1a9e3109766e12cf9ca91a6604"),hex!("8cc23606e813b66f315cb243769a9e7762ad801873abe94b85fe8a44791a6312"),
            hex!("4ec7e5499685ce60cc8be35f8e21c79a9a334b04e73f73e911bfa2a3a990dd5a"),hex!("826e38c7728c95334b37f2f18a1e7da3886956f1788d9a4e37d40fbb4741823c"),hex!("f407b3f53e3f62e30e0df8e4299b738d363e33fe0d1b95d91ee70363bd3e6e3d"),
        ],
    },

    // ======河北省储委会======
    ReserveNodeConst {
        pallet_id: "prchbs16",
        node_name: "河北省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726368627331360000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prchbs16.wuminbi.com/tcp/30333/p2p/12D3KooWMXQoZ9F6nxMuoC2ZnzxEKAn4z2qPKAugP2CZFEcXDqkT"],
        admins: &[
            hex!("4e3df6bdac65b48ea340a09463d64bd071aede8b7837bb951c20836c6925693c"),hex!("1a5e281c02a62c179e3bf6044f9321a76db4dbb446b74d78a515f74b6616934d"),hex!("842b630532ab7a2dd99765dad1b71e3dae11f7c0e3f53d73402a85ea733ae25f"),
            hex!("5ef82240ca85f2b0a747b7956b52510c37ceccd32d49afe10b7cec703f06612d"),hex!("8a265838a4eb4564251ed6dc77af40587b2ac82c079b3922e8925c3d00787c3c"),hex!("78724bf397ca64b458a648e233434c1695ca9cb8e4e1b1a008ddb68c04552a32"),
            hex!("9a3b6ab3204f60e0d87816f381b47377a3cc362d54c0a486a0e0b1c9f885d80e"),hex!("dee125b88d22d8a674fded7b3754ae168ff4569d65b297c1ee9d3cfe41fad217"),hex!("921a4d50c633d960bf26de00761a2ed856e60ed90730c8190451f5c48283fc0d"),
        ],
    },

    // ======湖北省储委会======
    ReserveNodeConst {
        pallet_id: "prchbs17",
        node_name: "湖北省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726368627331370000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prchbs17.wuminbi.com/tcp/30333/p2p/12D3KooWS2WYJ9AQ6Y1AKZcKjaHbmCFNkozV7XBBqqDG8kvwsH22"],
        admins: &[
            hex!("0ccde5b418f9fc572d28afa64d3a4b474d4590438e1edcea4031959d6f01a946"),hex!("0a8130175c6563770cb7a25caa3710f7077137da913b9f463da39b009d501b5a"),hex!("0670ecd2c3bfd67d70a66bb125d438c50113d800acf8c398c794ebf212217130"),
            hex!("d696333821a11115f064d4f3e1cf05e19e7ad2477675fde3d459b92dd50dc37f"),hex!("9a649d423961d80133acd47f7f4b0bd68f1af63d2fee8377c5314ae07df0190a"),hex!("66de8bee9f771dacb10c92dbf98e1cd24f5c257ca534b1e3cfdedf1992b04821"),
            hex!("a68bf9054358b1526ee30fdb6ca63513056eb03b4449f34e06bdf027df17c017"),hex!("ac4ebf8354b24bdcea96db337d90372e3f5925c8788d87cdd5e54aa70fc4e16e"),hex!("02d1a132c2ab2bccaf666aecfe7856b25640631b85405beae262780ba3d60f1a"),
        ],
    },

    // ======陕西省储委会======
    ReserveNodeConst {
        pallet_id: "prcsxs18",
        node_name: "陕西省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726373787331380000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcsxs18.wuminbi.com/tcp/30333/p2p/12D3KooWNr4EWB1PwBANoU9h2FzZXfS78vxDQynLtft3TDWMQ42p"],
        admins: &[
            hex!("5c652aa8012ce7fba7665b6836116651cdc290e1c4ce17b926944351389af75e"),hex!("dcea950deb017614ab6d2d70a69472f5697d3921b4c32145ba359e663a4bab3e"),hex!("3a256a29d5e8751d5ce2775c9a4f94a6d057e1488aa9fb247f032533f07d9256"),
            hex!("0225ff7444352f830d2152f41552ac11be732f4e44757fb3b0be381861f6ad04"),hex!("54f8be6f17189394348cbed7ec41aecdfee2952a6f30d64ca51c80825f497702"),hex!("ce73a6f40b8998f4d41581870751f3b1e1527706aafa6e8bdbcb55316aeb9256"),
            hex!("dc3d8a9102b5f65498f47666bb9d8ca0ad9eda79de36d665e3354ed33983f368"),hex!("6a755a8852e72aceedad0adf71affa1271a745b8662ac4a9e43bf8baaab2ba2f"),hex!("ca5077e5250f6e6f8c2239070ce43fb4b214b6ccb8be72a866635b88eb61df38"),
        ],
    },

    // ======重庆省储委会======
    ReserveNodeConst {
        pallet_id: "prccqs19",
        node_name: "重庆省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726363717331390000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prccqs19.wuminbi.com/tcp/30333/p2p/12D3KooWD8qAmRfVPyDn65j8aNLUZ3xKpc4jVVJ2Jdro3LZKJhrY"],
        admins: &[
            hex!("2ef4db13e343f5395c9dda46c323d7ffb592d96f4af55d06f9088a54c7c2ae48"),hex!("6c974aa78b7ef6a9f1ce0adf6632bafdaadab49d05b06624eba7f1c7f5ce825b"),hex!("faf86edf3aa251e36ec809503359ec35bb783b57fb753b0c61b85da98b01e159"),
            hex!("0efb493a48b30ce645cb169cbc800bd9b579d5457039daa267268f797db81930"),hex!("6e3dbdf2a9e9a22bad74f45f9f1fb2f0be9fcddfefb7bee0133a640045fa0553"),hex!("e2ce1a1a95d21b4aa93a62deb630a262f0d9ed7b700c4bb5a07298876784a047"),
            hex!("d07c54c5bca242360ee3dbcacb056dd5776cd259139d07fbac99a5d9dc4f0325"),hex!("6caf2aa045b35d7512354177449a7b96406091acce75f3e21a8e882c4878114c"),hex!("62d9758bf4a332d14e1fdbe6456aad74cabfd7df94eb7ca11303903c13195c6a"),
        ],
    },

    // ======四川省储委会======
    ReserveNodeConst {
        pallet_id: "prcscs20",
        node_name: "四川省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726373637332300000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcscs20.wuminbi.com/tcp/30333/p2p/12D3KooWR831Zp5wr6AXtwo5f6uoLzig1vTq8GtN8PK7AL3A4t1m"],
        admins: &[
            hex!("16212ee287e95f381f1e85e036af61c79e361411b9fb8b7f0ab9b7dcf4347230"),hex!("9a3208b10edb6dd02fcfe1d29aa1d181d9d1b12dd227686684de9515f3fda025"),hex!("8ec314048205ba56fbe1804c858431f47318d3fbfeeeb4300c6bdcd1e2d7df2c"),
            hex!("70a15050e04277dce6c63a26470a9a89d26839246137be62d433fd2fcfadbc66"),hex!("ccf73b9b6a779f486540494fcf64fb1f420df76bdc1379be14c44efe1c75fe4e"),hex!("6230f19b487bd2a1e52f4433d9ee6cd475289267b4564a4e45fdcba43383b278"),
            hex!("16c1f1117347cc339ebf633281c6047ad71d698b013f0ffe0e3b676ba00d1b10"),hex!("4aca71d0e4a6dc3908295c84ee87d65c8dfd715812e1880e4651c351cc9b4e22"),hex!("0814cd4cc90979edfb6bac2b528063e8dac9940b670cfa0f50187ffd5e30f908"),
        ],
    },

    // ======甘肃省储委会======
    ReserveNodeConst {
        pallet_id: "prcgss21",
        node_name: "甘肃省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726367737332310000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcgss21.wuminbi.com/tcp/30333/p2p/12D3KooWRKEFiEJGBdK6AdkJb6ei5FJiqSAvEkk4NxGnoT9p5MUS"],
        admins: &[
            hex!("869bb482411a81f4e3b6264cb0ee292b684c003446dc434164d987bdb831c30b"),hex!("cca18ed750841384f46e7c017babf3ab1b30176ad12bc4fdffa88a939059c51a"),hex!("fe6a75310a2dbc95e24b06483d30108c5303596f86775f17a7810a2fbb451609"),
            hex!("c08bd16fb31598869f577c700d6be149cb91193460034e95a8ce878f1ee3ed71"),hex!("b0b26bc35d9f4ab658d88fee8aed8647e7255d2d99deeeea94595b2aa2446c2f"),hex!("5c45e20af9b628fb57d1002ef73fcc4cabe1d477b8d4a9c4eb65fc02be14d418"),
            hex!("44430ab725515591540e8e776a9c1d6fc561c440f2e6296326af98d27b69272d"),hex!("e66f89f5b4bb1b6900c5d6602c02a939d2a464a1b92528165ba9155b1e14645d"),hex!("9024d5c388ee005713c3842cd0cb82fdd6372aea8d5ab5670cfcdebbda6b994a"),
        ],
    },

    // ======北平省储委会======
    ReserveNodeConst {
        pallet_id: "prcbps22",
        node_name: "北平省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726362707332320000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcbps22.wuminbi.com/tcp/30333/p2p/12D3KooWQZF44Z2U9mT6Q371ULaRLHK9ucTuxPVV8WpaUnw9Q4Ug"],
        admins: &[
            hex!("721b9533444e208d357ee90bfcf0e2eee6f27e0f90df6c4a5e17e3d5a5105233"),hex!("cabb2777b049e3e4f7833c65b8cd99596fc7d202946f49a588181d18fa94204d"),hex!("8a9d559f5fd16ef10b7923f993d0acc8cbaafa59875a0a0bc537acaa0697a01a"),
            hex!("46cfbcd8a875ad29fc039edf2fca2b734d534b42d429100da04448675a90c653"),hex!("2c7a7f34f2c3b5821ee758dadf353786a69f6f20459c8668eecd02c67e9b2658"),hex!("7416d95100f02d3ac78102a1f9c515eda1cbcbfb23aebe885b165691acea7943"),
            hex!("46156d787302d0bf7edc663f942de3a1adcfc80f0c0aaa7ffd351a44966f3b56"),hex!("101e875093d7e67b7e52193960f6a75279e13851fe4d89e5b396a77ebc054c29"),hex!("3e9a41ed62cb8be800a16587cdb5290c0c4b2602d54469baa9fb84f067541a04"),
        ],
    },

    // ======滨海省储委会======
    ReserveNodeConst {
        pallet_id: "prcbhs23",
        node_name: "滨海省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726362687332330000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcbhs23.wuminbi.com/tcp/30333/p2p/12D3KooWE69n2vS9KqPuXvZPAVRAXwfLcnAfHLz6EDBCD6G8Zqdk"],
        admins: &[
            hex!("aae591c8f6a4ab4dd02a9c6e07685da23b2e678df559e0444bc84b1897a58478"),hex!("56eed859f11c6b7ca8d5149c650d24f4ceab6a9eb7ffefd852dbb4899efd4544"),hex!("5cd2b74bbdc0b3af323de618cad0527699111eb36b3f65a84f0bee7b27e2b75f"),
            hex!("80dee603e439bcd91cc4b8abd95f1585cdebaf2921b11723478ba8cff571c703"),hex!("fefd342e384c7f9ae9a8ccdeeaf2f5aae659a57fb531d0597280d7542dfb7b58"),hex!("a659ae743a347027e9191e870df82a456c67d4c83b5a19f491642660d4e9ab3a"),
            hex!("ce31b21205fe511e998241b2f1ff7482678dddb0c1b83f67e70d336ecb8e0325"),hex!("acbb3b10b5f7a0fc0bc351845e86cefe9d40be360826e50030a7ef224276e97b"),hex!("0059656a3f16dfd726bf41b2079df6c0f3e5a3f3f075e612b8cbea06e78c166c"),
        ],
    },

    // ======松江省储委会======
    ReserveNodeConst {
        pallet_id: "prcsjs24",
        node_name: "松江省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263736a7332340000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcsjs24.wuminbi.com/tcp/30333/p2p/12D3KooWRQt9MWd8v1F5b8nNksRgvCk7XmMgntxiv6RX12gkY5Dx"],
        admins: &[
            hex!("f209b25849cf6a8b4067b853c969def73a60bb29cd11804b1a121ba98508653b"),hex!("9859508da9b3d0ae9265fed1303fee110b4f4a38ea5336ee26a4ee94f8e19d5d"),hex!("f23d8d6b3063fcce72a65c0c1ea6b7fa3c3f4b38adec7a341918254b124ed046"),
            hex!("1a08830eba0532e1976cb6fc5ed7cd8cb14eec2b1544eb418d64efaa455e907b"),hex!("f211c8d3cdebb7b6da346acdd9b01176ac49200b7a7f9b6eb64a84a3648f0b09"),hex!("d23b9b9b3d819c55be55329181047f0c638bb09702ba0dd7827699082f489e58"),
            hex!("76cacf3964738298b193c57b76642a694694922fa82583daa78906231988ff09"),hex!("ce74a941f896ff1c139e71f11de3461600d481b1b23ad20c9dd102e104743e33"),hex!("22376b940bcfedee8fd595c9de53ef46ceee595fdfb1b79da73e2772476eea78"),
        ],
    },

    // ======龙江省储委会======
    ReserveNodeConst {
        pallet_id: "prcljs25",
        node_name: "龙江省储备委员会权威节点",
        pallet_address: hex!("6d6f646c7072636c6a7332350000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcljs25.wuminbi.com/tcp/30333/p2p/12D3KooWGdzag2ekE4JBbcNYNNg3bAJJrqfrZQnsC4uaVavNpmtX"],
        admins: &[
            hex!("da4c7a7d4d3a2bf0ad33d0fce6f0a48aceb60a29f3de3d1fbce7034d4e04c620"),hex!("aab062939efedeedd4f8e753ab067feb13b527d51369fef66a550106c5a39166"),hex!("829740f3a8b2a75fcad7a8c59e5738bb698c4d43fb1eaaafc80dc566446d105d"),
            hex!("aa8fdc2be2ae150d74a03ffa21096a99be0fcc4cbd06c6a567680af244ebdc37"),hex!("8c369816c4f620216ebc4753b0742e8367acab65c424bc69cf7d4f22ab929b2c"),hex!("aa9a365b55ffd901693b0d85e22d53e3f6a6939c0c55b195c85d416a05d18b38"),
            hex!("0c384af439126349924d843d00b93bee219ba49a88f31a90ac53e73e0a141e58"),hex!("a490446d8a82a4d1947d5e163b1d21d912f1189bc70f8510705d5b07cd719544"),hex!("986e1b2ac396693592d7501038a7f193c81156fba638219828747f670e324d6c"),
        ],
    },

    // ======吉林省储委会======
    ReserveNodeConst {
        pallet_id: "prcjls26",
        node_name: "吉林省储备委员会权威节点",
        pallet_address: hex!("6d6f646c7072636a6c7332360000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcjls26.wuminbi.com/tcp/30333/p2p/12D3KooWHbuz7D91uDpbEPKLpSSKE9ZVqPSsTXFMewBbYAAxJYc2"],
        admins: &[
            hex!("f4845565b8c154dcbfd80c9ce7519058afe55f143433b06fa0d537ad7360cd1a"),hex!("02c548a209c4051c3ca2fdf74cd6c68234d74fe6374b74d95411227e04d45265"),hex!("745d18680cd4fcf3f21b6146e40e35c5275f5bd1bc1d209ccdecc00dcb5ea750"),
            hex!("24883ccafd903ba84d276e182aff2507d9cadac157b6a12718e0f398b1ac9378"),hex!("86b0de7de2db7650720956e21e895f153b562df9ce0a6bcb3ee652838b584c6f"),hex!("dc9b35a9a285cbb2b4287298aa37f2f2281bc1f4b971d8f158f17aeab9440829"),
            hex!("269d117adc1685983294f982f84eae814ef2d13becc1a46490e62b0fadaf7e62"),hex!("b0f3c14cc9a56043906817e0bf8a6e437c229ac2e8d7b3b994864e22fcf50048"),hex!("f8784d1fe83130e720716c7db6f3550e35d731d4f0d7c8e07fa9d4c4db8c844c"),
        ],
    },

    // ======辽宁省储委会======
    ReserveNodeConst {
        pallet_id: "prclns27",
        node_name: "辽宁省储备委员会权威节点",
        pallet_address: hex!("6d6f646c7072636c6e7332370000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prclns27.wuminbi.com/tcp/30333/p2p/12D3KooWE8RugcDKrBwxobPzGkVxke4WnGJhi74No53EH7zhaziB"],
        admins: &[
            hex!("5ce10023b13c3820b38b2eaa2b25f38d423c8ebff8fa9987da6043d2301bb41f"),hex!("56786015541de3069ace6e483b1f90af157b52b0caeca1f14f0783d7ce1a6878"),hex!("da198ab6b20dbea0a83ffc2fb52f7467414af14822bf748e8fff506cf0376b08"),
            hex!("82191bc36a20af00633e5ab68468f6221160de6082256b1d6f9e2314c3867632"),hex!("60b95b451ccc9ebf1237976b12d08c2d50ae81066a2bcee5881cad142bcd466d"),hex!("b2fab3ed222bbd224205ce87057a1ac7c817e644ac7de6c148506c8deefd4430"),
            hex!("5ec46177838b7fc1398e2666b1224a91e768bc63bc91cfe64f78fa3d2674b253"),hex!("0cbac8ba27c1b2472e350234f5d36d3103f8b6ed3429222eab9c16fee0944274"),hex!("9656b6e1b382e2dce50043df562f287052737af456e06a073eac106a12ac4439"),
        ],
    },

    // ======宁夏省储委会======
    ReserveNodeConst {
        pallet_id: "prcnxs28",
        node_name: "宁夏省储备委员会权威节点",
        pallet_address: hex!("6d6f646c7072636e787332380000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcnxs28.wuminbi.com/tcp/30333/p2p/12D3KooWGdFwKQQoZTyGbKHtq6FcEjmXSWJ4MfdebuM37MXXNV1T"],
        admins: &[
            hex!("04379eb91f117b2783a9c768025eb37f77107f4b8a8642bf0beb96be544ea34b"),hex!("4cb11155c0eadb7d1f93324171f080f5661a62259f741b542fb069724c08c840"),hex!("7236ae7d43e3d62838a4c43b23cbd56a015660f0668b6056ce108eefa68ea46b"),
            hex!("4eaf1b9bf497f01470a66046611c48c50598fabea24893b0a13fe423a960e639"),hex!("22525d2dafe9603e80529abd458da7305f80f792e5fbb96f9eb1a40f36b54854"),hex!("b4c6b3859f688d7bcb07f544735a3549cbb3783b350d5c437ccd9e47575acd52"),
            hex!("ea65c294cdc5124a6c05aed9887ce4cf3ed5984df73a508b7708a7fe369ab203"),hex!("645ffe0b96876f29191e441d4b1fba5bf1b59026181d7fb64a94b4a54ab99826"),hex!("e24f79123afbf4daf7bdf24832c1679668ea019b3157ebef1ba2abf1db17aa25"),
        ],
    },

    // ======青海省储委会======
    ReserveNodeConst {
        pallet_id: "prcqhs29",
        node_name: "青海省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726371687332390000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcqhs29.wuminbi.com/tcp/30333/p2p/12D3KooWEL5PTHVD4HEGRcsTxQKWanzW31qSzAGapvwnBsfdTWWS"],
        admins: &[
            hex!("1cb7f16cda56e42c93f76460247a84de444a6f24164442c782e5e96535011778"),hex!("22ac04c138217754099d4a414df855f8d400bbe14309a6d56e8f6f0115200866"),hex!("f88db68796417f9dc971ecf6bc191fde31d07626c6c4bb29c7d2ec6505e2382c"),
            hex!("c627323ed47c2d4315a115eb0fc4a0378251db27b4659520233e88960765ce7c"),hex!("9e7b66911c6e8a9a8ec81016358fe17df77f7702177dfd55a387fd14938b0914"),hex!("7047bf536b9144b616a2ed784835e2c307676f0901558946e09c6a2d1910b049"),
            hex!("d258bc272c331c120caae5299b166a01529fdfb10721a080fdeb7d73c585fe7f"),hex!("14cdd489fd3bfeab48696bb0f47b624aa4aad70580780338e5db91f5e4f2a07d"),hex!("6c4ed94ef08a1a9e51d117297a477f393652a04794a8b30b3b6748fe528f8148"),
        ],
    },

    // ======安徽省储委会======
    ReserveNodeConst {
        pallet_id: "prcahs30",
        node_name: "安徽省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726361687333300000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcahs30.wuminbi.com/tcp/30333/p2p/12D3KooWPC96XCXpuuErd8G7bteNhmvkk6NTPjLtccPCiRwLRGSw"],
        admins: &[
            hex!("92a4067c099c83db7853da7e8dc923ee6aceb09bd30feb547f3ed77a4e3bd74f"),hex!("dc610ead08eee19c0b2ac5dbefa6058b2fd392170e0e250188d141241c69e26b"),hex!("8a38abf3c492cf4946d7cc56485b8c101f60b51c535386e985cb7f9bfa33a54c"),
            hex!("84f6d8149f5dc06625c581fc0c44cccabd46b4e214037957325407fa1d7dbc3b"),hex!("8ab863c1ab63651c35a0187e703bee26fb62c8972e92b6e22a072d8662931673"),hex!("2ecc5d765898e1151a83e883a2981268ff4a2d76ef944bff6bb1c9a807135669"),
            hex!("bccebfc9fed2af7a516f08863eb1343e7f30adc9c8d294947aaf2a679f407a4a"),hex!("1e753b7d564c14d1ef033e06b00349baa01232b09bd2345902f2c05db17d832a"),hex!("20af27119abb66eb01e2dc58592a236906505f7c0f3130f67cc2d53f10a2e45e"),
        ],
    },

    // ======台湾省储委会======
    ReserveNodeConst {
        pallet_id: "prctws31",
        node_name: "台湾省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726374777333310000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prctws31.wuminbi.com/tcp/30333/p2p/12D3KooWQYc1jQZQyaUQC1snk9DHGmydhMdgtJ9LZZ5pbzTciG2J"],
        admins: &[
            hex!("14dd7aa11f3c8d7745ac0bce6d160701ebe1f382a3b9f503e258ca34c5730835"),hex!("700da7732d20b2c1d668381a04396e826834e326b318228443ebdf814c72da67"),hex!("70d00e52a19908624c6db321a1f55b816264bd257cb636bbed584af8366c3118"),
            hex!("104f6bf1b3826deb7fb67983187b53c493ddd521315bcdccee76114368d4670f"),hex!("fce5850d7853e167bc0f5c925aa59232c3d41358518e8c2dd8d9b16e4e636522"),hex!("300861c2315ccd0f30a2d6ffefcbd23789315c4a4038007d139ee193675d2a02"),
            hex!("869a65def49d615ed215a30e72ddbdbdbaeac55db980f42122fd253b427bd51e"),hex!("a290116221fce8f02bed080f0a0b9082e8db4a337f95a76022f03a005c3c9137"),hex!("20cf1a03ba6f9432e1ad25d4bbee6f885050e1ded68ca1df547df6aa4fd59f4f"),
        ],
    },

    // ======西藏省储委会======
    ReserveNodeConst {
        pallet_id: "prcxzs32",
        node_name: "西藏省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263787a7333320000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcxzs32.wuminbi.com/tcp/30333/p2p/12D3KooWNhQUZN2zvX8WTa5SvbyziGvr18qjVNnhygstb8KHQ7Ro"],
        admins: &[
            hex!("6c3b884d685780ebd05603dd24d467033316b1ccc91dd9b85714978de4c7c632"),hex!("f0ee0d300c114090fbba602a484558f31cb110b34546257e31e3463891b89f42"),hex!("36c856452191f55d0214ba53e685f965b0ef5ef21b8cc007b72e52f5aa6da17b"),
            hex!("5aef650b8169203d5eabf7ae5bfb60c21ebdef21b7ab7e5d9b2c80d954317024"),hex!("34adbbca95427cd9c3fa6b92b7020cd50e14b08c2be45250dd7003311235941a"),hex!("a6d4400c3e69f953f8806afa07d731dfada676b15c41c0f52bae083159123a6f"),
            hex!("3ae16fec56ad3d72b64cae2dc1ce9ab955dca199c9358c4f2cca61a7bef28b2f"),hex!("9a714778bc7e9afee83e0e0da94e4a3b44c9730831ccc126a06ef9b7f3dde810"),hex!("ae7f26a79cc355cb21d9ad4b8526930de67fd4ee77fccc11d696f774c6fb7a46"),
        ],
    },

    // ======新疆省储委会======
    ReserveNodeConst {
        pallet_id: "prcxjs33",
        node_name: "新疆省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263786a7333330000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcxjs33.wuminbi.com/tcp/30333/p2p/12D3KooWMbsFaTXiGKXqjEFZjuP5Tp7iU4FFvf3MJoSmGRXDVc69"],
        admins: &[
            hex!("d0a03917c1501fd6e37bfa574a9f9c7415af43982e44d8bd2f2262a7241c9366"),hex!("34b8fb4e24fb4a45960b8f842ef55150988633f607dd745583726a35c6b52c64"),hex!("06f061f330578f10d9ac0c30f684ecd93f86b00a79189b5255e2f11b34bd8364"),
            hex!("34a028e6a82d4a724f5de65856ebb8fdafd7dd6acd2fa20f673005b927820c03"),hex!("ee64d7d1c345f1aabe5f73693c98fb0da099350071b243f143d1942c8b36ae7d"),hex!("a004b4c8afe340aeb78ea2e10d8d2963b8bd3a6a756dda6f3863b83b25d24939"),
            hex!("4c370a5f20aec23e2f502e6b1e86b09457bb8c03899e2b264328c4c0666d2213"),hex!("6ccd63431eed80330251eb7a4c6fd000ce7e4f0d249034273758723cb72b9f20"),hex!("e800e7b88a4e9ad208d33b9824f11b7adb6f529ce26335a527607d459d3b381f"),
        ],
    },

    // ======西康省储委会======
    ReserveNodeConst {
        pallet_id: "prcxks34",
        node_name: "西康省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263786b7333340000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcxks34.wuminbi.com/tcp/30333/p2p/12D3KooWBczZmptJkbQkX4yx4XP7QXwtJXxZn1We8R4GtbRExUox"],
        admins: &[
            hex!("0881de91b1a1ab4c57b89be8f7e630af9aada874fdb8acfd45f5975db173d467"),hex!("d4f2c0eaa56597aa19567734c7387d507fb2bddfc33c63aaee1cc0e132f9eb6f"),hex!("28586edd920320c323e2390108cabacf62972bf35f7cae7af0b541a8b7b5be42"),
            hex!("da63409c3b9fb4eddaa47563d92ea7965d7269e0663e1917b1ffb3a9a94e6b7b"),hex!("f26eab1c542525b4fe778abfe33c622572e3311c316705dfa6ba781a1ec79a58"),hex!("c2f437b75d48d34df2ab3c6f2c5cfa4a9bdaad1bb79228667fe5edd4b1469856"),
            hex!("5290d3f7676ac37c84715d3ecc786c896fc37eaa90845fb95539b3e948f7aa46"),hex!("e24b7c0117026cad23fbcdaaf51386268e1ea38a85a4827a95d77d93853bf87a"),hex!("645c319a18abeea8aa5c7630b385187e6427c227d5350da20c35a29f015aba27"),
        ],
    },

    // ======阿里省储委会======
    ReserveNodeConst {
        pallet_id: "prcals35",
        node_name: "阿里省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263616c7333350000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcals35.wuminbi.com/tcp/30333/p2p/12D3KooWJKCXsrzLVWLuZVTENBLeLG5F9KcLoeGhdp1tjs8qtk2y"],
        admins: &[
            hex!("e8e3c20ca22c65654c68335a7a140287c6df906e971065485794a839aa4c766c"),hex!("ea6abd091bb012beb14a7322ebcc8d65407379ddf9de0eb5d0f6883f2783220c"),hex!("e657f8f30589169abd250e2d5b10f7ae33629a0e8ef5108f035d23303be84e31"),
            hex!("6e51926aba8b2d66cd521c56ece41b35a7776f867bf82668a092b8838aec5a60"),hex!("cef9959fd5209c86c7328779ccdd19ab6771e61f0a7914d2fd7a1ee0dc8fdc4a"),hex!("6c24bdfd475c8e1fff5c1670dabd667053c8be6a8064031f1fc5b81f4a6a5237"),
            hex!("2010c4b43a6241285126a7974e5f53c072264d07f40df3be9cb9e92ba91a3822"),hex!("72609a42d97c09dcf823d5516ed66098bf4e9fc69f44eeb4ef4ef62894265f5d"),hex!("20e67bb8e0e4972fc553145161c9963136b9786cb26abb5af6340bf6962ee262"),
        ],
    },

    // ======葱岭省储委会======
    ReserveNodeConst {
        pallet_id: "prccls36",
        node_name: "葱岭省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263636c7333360000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prccls36.wuminbi.com/tcp/30333/p2p/12D3KooWMU7y4HSkWdKQYQ15xQC9L33TUkfcMfBgYQtkxMDcos9v"],
        admins: &[
            hex!("a87c4b41c8d1494e2e05fb30f7c263b393c8f77701952f6a18e8561238749571"),hex!("4209862188320e1bcfddda367da543dd62bfd1ff187bae7fb50bf74e8e38331b"),hex!("d07db0a48948480f917c7652daba071f581ab2c90275ca11257a902f9bfa6e74"),
            hex!("b4f214db8c8e17511b890590c2de9b6bdea0533637f6d19a47376dbd1d4d297c"),hex!("02d80e629a8db84fc9283f35b7f362e3f092702c83f3e0cc8aeeb8d8b80e0521"),hex!("c061f31265dc62707446a7ab08f8bba24c3412467aa5d66c673925b9faff8703"),
            hex!("4673ae216f91e62a11169721d3b3219dd34df0e4342f8d882d0e48a5c676a92b"),hex!("a86c79964a2f0d14d42d46dcf4d39aa6b09f55134868394fd7d05ca35b906626"),hex!("46926d45559ee41ee7355f439a9c1e23c61f1a4ac39821c7f85e381239041f23"),
        ],
    },

    // ======天山省储委会======
    ReserveNodeConst {
        pallet_id: "prctss37",
        node_name: "天山省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726374737333370000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prctss37.wuminbi.com/tcp/30333/p2p/12D3KooWG8ZyfEQo7MkkcKqUczQkY1eKVZFvvAeUpz4EPFi8vEoN"],
        admins: &[
            hex!("24acc05f6487a97c7ffc6a319cc532cca81e05f26935e56f3867bd54ddf57274"),hex!("568f7c9301c1a56fd66e4e6b1d4e80f19f92fc8c1f25388f7f4a432b08fb111d"),hex!("24c44b770b9be1d1bd7c34c8f39b718511ddb87576506d2954d073f2256ad608"),
            hex!("8e6d4905ad8cceb3c416e1eb65660d3a49642202e43fb25200744a981e2b202e"),hex!("982c6e6cdf2b2a45bde79b60c66199100a873ec6fa5f1db9c2770a2d1c711d2d"),hex!("56f74bb044ab5d559228c063337d4c0bc83b1cc80105b6e3ff0e0be47d5fa65f"),
            hex!("28b6432940a6f1b8b4c817a4935f7a14e3dd42234397b67511c38453689e4e37"),hex!("30afa841d3e94a46228316664c6bb88b858e4e3368ac5db91effd98f44d06e5e"),hex!("3e568f9d9ff2b96380eed8e2795a6eeaf47e2ea6222bdfeb67e41d0162491354"),
        ],
    },

    // ======河西省储委会======
    ReserveNodeConst {
        pallet_id: "prchxs38",
        node_name: "河西省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726368787333380000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prchxs38.wuminbi.com/tcp/30333/p2p/12D3KooWBjDquSWFYAjTy5LWxBqYKC453WT8FpoJTVKK6qGk2G4y"],
        admins: &[
            hex!("fa350c6790bdce5f2f39b8ff4dc7994200e6e7b79a77ab0e7707190472e61d07"),hex!("722ac6c1f12fad76210455629115e9f4aadb9bfa1bd61676b366c807fabf8d63"),hex!("22941c3a0375aeb90c08fe9af7ea32c7937810b6f0bcd153745da19704eb1219"),
            hex!("b4729732d9f0859cf88a97631acf148f858f8e29a88526e39c31a807b6ba853f"),hex!("68a741a5e34f3efe44e5d44a38945684dbdb83d53814dd65690a7906be479d37"),hex!("acaa713f2328e540ef7b172ae0a935d5c2a96be0732a77f0745a59096c62e10e"),
            hex!("2eb9ba1623e956687f0141c91aaf4d2b4315aee7cad2d8f783ea40abdb975a34"),hex!("427e53a189f2f9d25fae6603cf1eea77a9afe6bfafb2c23d7168563e228cb64a"),hex!("9e6fd64b1547942561a6d4f239e52cd008282837d939aa2b87c0f3904691c25c"),
        ],
    },

    // ======昆仑省储委会======
    ReserveNodeConst {
        pallet_id: "prckls39",
        node_name: "昆仑省储备委员会权威节点",
        pallet_address: hex!("6d6f646c7072636b6c7333390000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prckls39.wuminbi.com/tcp/30333/p2p/12D3KooWSeAo5RUTjTX53NmD8Ncv6fXfnkqd461a6FBEGv8szB8N"],
        admins: &[
            hex!("32320497967509f7db42bd389f6087e30235557abc0cc1c29142446f5d8b9b3f"),hex!("480cba2bc9ac30646fc03497ac8d154cbeb5a9b72faded8ad3f9111c91eeaa20"),hex!("522116712bd131d4875e3d860c410151d70a5a504c746237912a91f639c15023"),
            hex!("8ec7f1baede977acee4c052f1123bbabf6958ef16c59246a7aabd9e592496804"),hex!("ec61ab0c4f6204dc09c95a58a665163c43b04274fb3ec0ccfc8d69c9c95ec079"),hex!("86ff205da7955c9cf0e71ba5a69383907a4b397ad92a6e01467d30e290fa7f68"),
            hex!("0e9de2455112b7adcbbc8bf09adc3efdf41fe842aa52dcdac2e86977902e2d5d"),hex!("0ce9934ba0396629a475d18992591e5b618fafe5d9c2824cd81829768085d817"),hex!("004340761ce099395db121851704d6a6fed79788a1fb98744d4f572dc9cef846"),
        ],
    },

    // ======河套省储委会======
    ReserveNodeConst {
        pallet_id: "prchts40",
        node_name: "河套省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726368747334300000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prchts40.wuminbi.com/tcp/30333/p2p/12D3KooWFrXygQG5HZ1buBcrGwe7KYQagNu29ippkUAbLUndxt9v"],
        admins: &[
            hex!("1a3356518d94c59d2a2f182955d5d05806cfe80c927042e6f6928555bf4f657c"),hex!("90f6deed86cce12a9766a55a8db95b08c4061ba88151e3749ec764c28eb7d349"),hex!("282b05d98c99c3637de456f6aa51d073d29066f1e89f0da91619c70f3fe6964f"),
            hex!("90f20f84b01c478c4c69192795bcbda6038d28c9c631a8d3465817586261020c"),hex!("70269f2d803d8a1b7feee8279184c4e41cd2f055576768af14aa2bb6419d0842"),hex!("a66f52c3516d0dc8f77fe5e50282cdf3000914b3a66a60787fe35b4c7839c461"),
            hex!("a8ffc235291612e49f980385db922affe4deaf903ad37d4f9f9ec7eb80dd8466"),hex!("14e4b29287ea7c9c2283bacf48cee46b3832570705349af9b6ef35b1b8b14b15"),hex!("b48ee0978391532d44739462e98162d80c03b3aeee4a161a61b5e56569a66b50"),
        ],
    },

    // ======热河省储委会======
    ReserveNodeConst {
        pallet_id: "prcrhs41",
        node_name: "热河省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726372687334310000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcrhs41.wuminbi.com/tcp/30333/p2p/12D3KooWBUyRBBAb6QFkJ3obK1bniWNFu4Gk7VoZAAQo7jrQfNCf"],
        admins: &[
            hex!("045a9f06c01d50bd09ab61a58e35889263df42399069754ed316504f151c3f74"),hex!("c8fb7f296f0c860e8c5f3edca93d6632985e61b89b0c2ab9e6dc00736e615204"),hex!("06a3cae0b46e3d3711e37942c7438112ef83f8b4f738bdbad698f2949d72440b"),
            hex!("76c3b3b705d6c60e8413539bab6a9312dcc36a264f4e0520018833fe3bb39a69"),hex!("6c15e31b79b225a0c6e811945d1514eb60aaeae7910701d9acc78684da470417"),hex!("e404f2855b43b5c4e0818dc0de24d24345861f379162a93529f281de4c4af72e"),
            hex!("54f0115e688a5535c6b757658e91ef544aa77f6e235d0985cb39900b8882526e"),hex!("369e345a95c88a9c01f2e0f6b6df9b327471c11f9a69c2e74fe1b49d5e30c226"),hex!("e0e890e1f10a82a1f91213ccf7fc49ddc7ad3310a90a96cc3de30bc8d98bbc41"),
        ],
    },

    // ======兴安省储委会======
    ReserveNodeConst {
        pallet_id: "prcxas42",
        node_name: "兴安省储备委员会权威节点",
        pallet_address: hex!("6d6f646c70726378617334320000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prcxas42.wuminbi.com/tcp/30333/p2p/12D3KooWC4errbqKaeyDZVjpNmpryUAfbLM8h6CjvyAbkYjzgnne"],
        admins: &[
            hex!("f8fe916bd56ee3af0267fcd5ab1de6f09e2efe4efd8c2bb3cc4d90b91a50211c"),hex!("7c70bee976cacf0fbf9e08e781a100fae11e0574cf5b13bfb067878758e27345"),hex!("5800fac49cc52072463f85e12e15e1fc931631cccc23961ce9e7b69dea9f8009"),
            hex!("f241133d9f5a7f372d2daef945191bbad4c8ef245f3c344a039b7920d9875d07"),hex!("c264a39276fd233dcbb85ab9a63091004b42cbb5892028581d1d316ec4c0ac4b"),hex!("929fb816a44fd71842be61712501689f42a75ccc760beb36af9bec8b29f47041"),
            hex!("1c051e137c7e3ce9f1014da4593063e1939ca04a07fa15c682413913403b757a"),hex!("5654500752c59bd8b1e4f477406b42e42f034edf76781b8da47f136a1500a33d"),hex!("bcab013a8807660c1d3901618f827146adf2861464f71be15e3b6989072abd4c"),
        ],
    },

    // ======合江省储委会======
    ReserveNodeConst {
        pallet_id: "prchjs43",
        node_name: "合江省储备委员会权威节点",
        pallet_address: hex!("6d6f646c707263686a7334330000000000000000000000000000000000000000"),
        p2p_bootnodes: &["/dns4/prchjs43.wuminbi.com/tcp/30333/p2p/12D3KooWPciaAo15DT24rXPZK5EUtBdEyotFBhvEdw6d3zBmzVHH"],
        admins: &[
            hex!("e2df7a8927683de5554e541d4e1d028fb159cf42d5663cdef310f16a177f2369"),hex!("4455c00ff6b4918ff744038c61cb92d194cba08ed156b111c9e9ac0643074a04"),hex!("e09e8a9388265d592beb486b5fd806db5941e5c162d25b969438ab2291886a17"),
            hex!("12cef2bd9cbed363a4264fd6d2159cb7e89ac058e281f1879e60d6e7742e2647"),hex!("5865a92ad28589e12e232c54b337b02b0e8f8b745f3306f051ee03b932680b64"),hex!("c8a111f5582d53ba4c717aacf2580cba3b3fa0c562db8094cf9459f38a13522f"),
            hex!("58f461f6a65bf6aa72f3fa16392646876a79aa1973e41abf12eb9082fcd13c06"),hex!("eee5c120839ae03a0218c3399897fbf0f34179af6d28a1d0731aed24d8dceb10"),hex!("2c5a156c83d3f8b33ba7753a068cf19369eec2079501c2975abdbf4f73a27963"),
        ],
    },
];