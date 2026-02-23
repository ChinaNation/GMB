import json

try:
    from blake3 import blake3
except ModuleNotFoundError as e:
    raise ModuleNotFoundError("missing dependency 'blake3', please install with: pip install blake3") from e

def calc_multisig_hex(public_keys, threshold):
    """
    纯 Python 实现 Substrate 多签 AccountID 计算 (0x 格式)
    """
    # 1. 公钥排序 (必须按照字节序从小到大)
    # 去掉 0x 前缀并转为 bytes 排序
    pk_bytes_list = sorted([bytes.fromhex(pk[2:]) for pk in public_keys])
    
    # 2. 构造计算载荷 (Payload)
    # 前缀: 'modl' + 'py/utilisuba'
    prefix = b'modlpy/utilisuba'
    
    # 拼接: 前缀 + 成员数量(1字节) + 阈值(2字节, 小端) + 排序后的公钥
    num_members = len(pk_bytes_list).to_bytes(1, 'little')
    thresh = threshold.to_bytes(2, 'little')
    
    payload = prefix + num_members + thresh
    for pk in pk_bytes_list:
        payload += pk
        
    # 3. 计算 BLAKE3 哈希 (256位/32字节)
    final_hash = blake3(payload).digest(length=32)
    
    # 4. 返回 0x 开头的十六进制字符串
    return "0x" + final_hash.hex()

if __name__ == "__main__":
    try:
        # 读取 JSON 文件
        with open("vault_without_salt.json", "r") as f:
            vault_data = json.load(f)
        
        # 提取公钥
        all_pks = [item["public_key"] for item in vault_data]
        
        if len(all_pks) < 2:
            print("错误：公钥数量不足")
            exit()

        # 示例：使用前 3 个公钥生成 2/3 多签
        selected_pks = all_pks[:9]
        threshold = 6
        
        multisig_hex = calc_multisig_hex(selected_pks, threshold)
        
        print(f"\n✅ 计算完成")
        print(f"参与成员数量: {len(selected_pks)}")
        print(f"签名阈值: {threshold}")
        print(f"多签 AccountID (Hex): {multisig_hex}")
        
    except FileNotFoundError:
        print("错误：未找到 vault_without_salt.json")
    except Exception as e:
        print(f"发生错误: {e}")
