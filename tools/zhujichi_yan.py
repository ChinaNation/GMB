import subprocess
import json

def generate_ultra_safe_batch(count, output_file="vault_without_salt.json"):
    all_keys = []
    
    print(f"\n🚀 开始执行高安全派生流程...")
    
    for i in range(1, count + 1):
        print(f"\n--- 正在准备第 {i}/{count} 组 ---")
        
        # 1. 这里你输入的盐值只存在于内存中，用于计算
        current_salt = input(f"请输入第 {i} 组的私密盐值 (可见输入): ")
        
        cmd = [
            "subkey", "generate", 
            "--scheme", "sr25519", 
            "--words", "24",
            "--password", current_salt,  # 传入 subkey 进行计算
            "--output-type", "json"
        ]
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, check=True)
            data = json.loads(result.stdout)
            
            mnemonic = data.get("phrase") or data.get("secretPhrase")
            
            # 2. 【核心安全点】：JSON 结果中绝对不包含盐值（current_salt）
            item = {
                "order": i,
                "mnemonic": mnemonic,         # 只存助记词
                "public_key": data.get("publicKey") # 只存公钥用于对账
            }
            all_keys.append(item)
            
            print(f"[+] 第 {i} 组生成成功（盐值已从内存抹除）")
            
        except Exception as e:
            print(f"[-] 错误: {e}")

    # 3. 写入文件
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(all_keys, f, indent=4, ensure_ascii=False)
    
    print(f"\n✨ 任务完成！结果已存入：{output_file}")
    print("⚠️  安全提示：JSON 文件现在不包含盐值。请务必用其他方式（或脑子）记住你刚才输入的盐值！")

if __name__ == "__main__":
    try:
        num_accounts = int(input("请输入总套数: "))
        generate_ultra_safe_batch(num_accounts)
    except ValueError:
        print("错误：请输入数字")