import subprocess
import json

def generate_ultra_safe_batch(count, output_file="vault_without_salt.json"):
    all_keys = []
    
    print(f"\n🚀 开始执行高安全随机生成流程 (无盐值)...")
    
    for i in range(1, count + 1):
        print(f"\n--- 正在生成第 {i}/{count} 组 ---")
        
        # 1. 构造命令，不使用 --password
        cmd = [
            "subkey", "generate", 
            "--scheme", "sr25519", 
            "--words", "24",
            "--network", "2027",
            "--output-type", "json"
        ]
        
        try:
            # 2. 调用 subkey 生成助记词
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            data = json.loads(result.stdout)
            
            mnemonic = data.get("phrase") or data.get("secretPhrase")
            
            # 3. 保存助记词和公钥，不存任何盐值
            item = {
                "order": i,
                "mnemonic": mnemonic,
                "public_key": data.get("publicKey")
            }
            all_keys.append(item)
            
            print(f"[+] 第 {i} 组生成成功")
            
        except Exception as e:
            print(f"[-] 第 {i} 组生成失败: {e}")

    # 4. 写入 JSON 文件
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(all_keys, f, indent=4, ensure_ascii=False)
    
    print(f"\n✨ 完成！结果已存入：{output_file}")
    print("⚠️ 提示：此 JSON 文件不包含任何盐值，每组密钥都是随机生成的助记词。")

if __name__ == "__main__":
    try:
        num_accounts = int(input("请输入总套数: "))
        generate_ultra_safe_batch(num_accounts)
    except ValueError:
        print("错误：请输入数字")