import base58
import json
from cryptography.hazmat.primitives.asymmetric import ed25519

def generate_identity(domain):
    # 1. 生成符合 Substrate 要求的 Ed25519 私钥
    priv_key = ed25519.Ed25519PrivateKey.generate()
    node_key_hex = priv_key.private_bytes_raw().hex()
    pub_bytes = priv_key.public_key().public_bytes_raw()
    
    # 2. 构造 Identity 模式的 Peer ID (Substrate 官方标准)
    # \x00\x24\x08\x01\x12\x20 是 Ed25519 Identity 的固定前缀
    prefix = b'\x00\x24\x08\x01\x12\x20'
    full_identity_msg = prefix + pub_bytes
    
    # 3. 生成 12D3 开头的 Peer ID
    peer_id = base58.b58encode(full_identity_msg).decode('utf-8')
    
    return {
        "domain": domain,
        "node_key": node_key_hex,
        "peer_id": peer_id,
        "multiaddr": f"/dns4/{domain}/tcp/30333/p2p/{peer_id}"
    }

def batch_process():
    # 你提供的 44 个真实原始地址模板（仅提取其中的域名部分）
    raw_addresses = [
        "/dns4/nrcgch01.wuminbi.com/tcp/30333/p2p/2DeusbNnisuEiDuDEiQ1JGAzZVruPDfTaW7wy26NwU7GTc",
        "/dns4/prczss01.wuminbi.com/tcp/30333/p2p/2DhbTPiF4kyhHK462KFQVxKsh1fQqfEnC6LP7hpLwScAER",
        "/dns4/prclns02.wuminbi.com/tcp/30333/p2p/2DbZ3k5FRM3qSRCKVUmPjufzXN87w5sPWPjgaZ9xJgAK6s",
        "/dns4/prcgds03.wuminbi.com/tcp/30333/p2p/2DSWjEc1xdG4w7NFwyL7HewKhtxKL54uFtrvzRFWAvmvgE",
        "/dns4/prcgxs04.wuminbi.com/tcp/30333/p2p/2DZZgKgBGZ2YLYqzZE3QF3g6WwRB1XMUXepK6XLinY36cg",
        "/dns4/prcfjs05.wuminbi.com/tcp/30333/p2p/2DeZoisMBsCRDsdcmFadpKhW9PU2zVavQ3aq7VpRe2WnWd",
        "/dns4/prchns06.wuminbi.com/tcp/30333/p2p/2DgrnXTNUB48z6gAtPL4GvCKsjvfGB3afth2rjT2HBMmaE",
        "/dns4/prcyns07.wuminbi.com/tcp/30333/p2p/2DftrpbqDdNaic14yHuJsen3KDd7BMZErHhjikLPm5XVmh",
        "/dns4/prcgzs08.wuminbi.com/tcp/30333/p2p/2DfGHXNPZGctUGxAJx9sQ3nMYeqegEQRGRjy3ngMWaKnMh",
        "/dns4/prchns09.wuminbi.com/tcp/30333/p2p/2Dg5bNEjBL5ZAdzNXP8rdBMvziufMB5wP4uWXyHrCKaSCb",
        "/dns4/prcjxs10.wuminbi.com/tcp/30333/p2p/2DRF4KHFo2Ry5BDm8JXFMaRPnootUJReGxYxy3d3faX8X2",
        "/dns4/prczjs11.wuminbi.com/tcp/30333/p2p/2DY5qMJsVB6CKTBC7hDeiR9225YoShgSffihQUDvh8JsNM",
        "/dns4/prcjss12.wuminbi.com/tcp/30333/p2p/2Dcua5EgpN9LEbgHBLkUEPVe6pdsGYSK6rwgHxDL88sJL8",
        "/dns4/prcsds13.wuminbi.com/tcp/30333/p2p/2DSsLbhGuU4GW8QzYvGRpah2YQvenVkdH13WiUDPTWZivT",
        "/dns4/prcsxs14.wuminbi.com/tcp/30333/p2p/2Dh31mB3SgKMxWAUSQoVDWcKLmyN7kfFYfaMoQXwgEL38x",
        "/dns4/prchns15.wuminbi.com/tcp/30333/p2p/2DTFSWjwWyjZb2YeHXEbDJATL6PcNfjeVYvgYUyKEiMjRY",
        "/dns4/prchbs16.wuminbi.com/tcp/30333/p2p/2DStMvrU1FaXst1zJk3KdF7cPMHYufEFANdWEyroePbEDB",
        "/dns4/prchbs17.wuminbi.com/tcp/30333/p2p/2DVHibYUSQ7U89eNGNPNjbZbYKnGtk2gVZ5DZWLD96Je5B",
        "/dns4/prcsxs18.wuminbi.com/tcp/30333/p2p/2DQuf9qo2ryikyApvgfKnfAanDSVp19ougaWyGNfiqcFTF",
        "/dns4/prccqs19.wuminbi.com/tcp/30333/p2p/2Dfa7YbZDZNwZjBDsgzzEvtpkxdSrUuvD83Uad4xPjp1np",
        "/dns4/prcscs20.wuminbi.com/tcp/30333/p2p/2DYRkPk9zcEy4gvAxw5yuzzXcbEV7KxPgBNyHHJtgsLQ9s",
        "/dns4/prcscs20.wuminbi.com/tcp/30333/p2p/2DYRkPk9zcEy4gvAxw5yuzzXcbEV7KxPgBNyHHJtgsLQ9s",
        "/dns4/prcgss21.wuminbi.com/tcp/30333/p2p/2Dg3QEjVZ1qMhqZGiW6hyoREoLcWChJnyjN3Pnh3t4DhE2",
        "/dns4/prcbps22.wuminbi.com/tcp/30333/p2p/2DgzjAcKQddi8TfuvRNYn18unhXoYUjR16np2BMPB31CZK",
        "/dns4/prcbhs23.wuminbi.com/tcp/30333/p2p/2DRtqpf2yNWeYBejQs6urwS8irXVkuxWYFD3coaeRxxmah",
        "/dns4/prcsjs24.wuminbi.com/tcp/30333/p2p/2DZ1g8But8K1NuRADUMKpmyWfaLL8ys64jwTGsngpbeVtL",
        "/dns4/prcljs25.wuminbi.com/tcp/30333/p2p/2DZqiyjEF8JZ6ow9yBzfet2LmNrmNMdYWKATk358MWbXLf",
        "/dns4/prcjls26.wuminbi.com/tcp/30333/p2p/2DgPstcCVkX43g6iK8UwkJixU62WSRvtEdUpht59gqp1LB",
        "/dns4/prclns27.wuminbi.com/tcp/30333/p2p/2DVtmYUPmmWKxSeWHDiLDMGf8EVSGRVNPiMRpRSXqDf1xp",
        "/dns4/prcnxs28.wuminbi.com/tcp/30333/p2p/2Da2bg5VVbzEzhjcfLUZK5NA3Bs5Fzpq9hE81g71e7MFqm",
        "/dns4/prcqhs29.wuminbi.com/tcp/30333/p2p/2DRVU83CTXBPeDCyQtjCBsLBWjgDjXsq1HjJn4tqCnNK9Z",
        "/dns4/prcahs30.wuminbi.com/tcp/30333/p2p/2DhLtABDFzDeUJStWrVPZM2R8RzdFTPmNMsGZg4gWN94X9",
        "/dns4/prctws31.wuminbi.com/tcp/30333/p2p/2DdHcewjawdRNpHXytzTHTDEPMqKisUSept578bC6iBFsg",
        "/dns4/prcxzs32.wuminbi.com/tcp/30333/p2p/2DhPCPJDL1rquDcXL3wSQ4vksjztyrMrZDd7naSGWoeVJy",
        "/dns4/prcxjs33.wuminbi.com/tcp/30333/p2p/2DXYA689D5BgE8XWRV6pzn6SDCTPH1AXgzbhHDi7Ftg8L4",
        "/dns4/prcxks34.wuminbi.com/tcp/30333/p2p/2DRqux2bjwQvZaXDtscZUiTmTJ2mQxstdFUf88vxtUJg2Q",
        "/dns4/prcals35.wuminbi.com/tcp/30333/p2p/2DfVXRpoL3zBzPNcznLUEctHPn8ZwEjCMo14UFdKkxPxxM",
        "/dns4/prccls36.wuminbi.com/tcp/30333/p2p/2DQxSp2GvrDP7yL7LjvQ61CtjYbVepBgpvVnVTcJmKFvji",
        "/dns4/prctss37.wuminbi.com/tcp/30333/p2p/2DW9knzEL4DqE3hhtad3kArw3uJMrEYGp4XwCGaDatQhNS",
        "/dns4/prchxs38.wuminbi.com/tcp/30333/p2p/2DgnFqaodsYm9zzQ3Jv7ZSJVt8r1ncwJRqsVcgk5jLVeNQ",
        "/dns4/prckls39.wuminbi.com/tcp/30333/p2p/2DXfThv4NhD4kQXhYL5rXA9v3uvk25kiMv8akBByPXzpMs",
        "/dns4/prchts40.wuminbi.com/tcp/30333/p2p/2DgP8AFNThsRUtReAf3eNm3qMsmbDZ7fhyExUjLM8qrj6c",
        "/dns4/prcrhs41.wuminbi.com/tcp/30333/p2p/2DYRFG6JiihR3WVjV7ByDRwKmWdcWW7RwLChPXLdSBNWNg",
        "/dns4/prcxas42.wuminbi.com/tcp/30333/p2p/2DVfbKxxxNFZ3EPK97Z6YHdhUqvzgyV69KJUSQVEVhhZSu",
        "/dns4/prchjs43.wuminbi.com/tcp/30333/p2p/2DSto6ToKQx39CZAg7FNTM2A4JHhfHuBUMtprLma52piiN",
    ]

    all_data = []
    print(f"正在基于提供的 {len(raw_addresses)} 个域名生成 Substrate 标准身份...")

    for addr in raw_addresses:
        # 解析域名：从 "/dns4/域名/..." 中提取
        parts = addr.split('/')
        if len(parts) >= 3:
            domain = parts[2]
            all_data.append(generate_identity(domain))

    # 1. 导出详细的部署资产文件
    with open("deployment_assets.txt", "w", encoding="utf-8") as f:
        f.write("GMB 44节点官方部署清单\n" + "="*50 + "\n")
        for item in all_data:
            f.write(f"\n域名: {item['domain']}\n")
            f.write(f"私钥 (Node Key): {item['node_key']}\n")
            f.write(f"Peer ID: {item['peer_id']}\n")
            f.write(f"完整 Multiaddr: {item['multiaddr']}\n")
            f.write("-" * 30 + "\n")

    # 2. 导出为 bootnodes 列表 (用于粘贴到 JSON 配置文件)
    boot_list = [item['multiaddr'] for item in all_data]
    with open("bootnodes_config.json", "w", encoding="utf-8") as f:
        json.dump(boot_list, f, indent=4)

    print(f"\n✅ 成功！已生成所有节点资产。")
    print(f"👉 deployment_assets.txt：查看每个域名的私钥。")
    print(f"👉 bootnodes_config.json：获取全新的 bootnodes 列表。")

if __name__ == "__main__":
    batch_process()