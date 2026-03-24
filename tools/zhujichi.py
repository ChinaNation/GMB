import subprocess
import json

def format_item_as_text(item):
    return "\n".join([
        f"第 {item['order']} 组",
        f"助记词: {item['mnemonic']}",
        f"公钥: {item['public_key']}",
    ])

def generate_ultra_safe_batch(count, output_file="vault_without_salt.txt"):
    all_keys = []
    
    print(f"\n🚀 开始执行高安全随机生成流程 (无盐值)...")
    
    for i in range(1, count + 1):
        print(f"\n--- 正在生成第 {i}/{count} 组 ---")
        
        # 1. 构造命令，不使用 --password
        cmd = [
            "subkey", "generate", 
            "--scheme", "sr25519", 
            "--words", "24",
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
            
        except subprocess.CalledProcessError as e:
            stderr = (e.stderr or "").strip()
            print(f"[-] 第 {i} 组生成失败: {stderr or e}")
        except Exception as e:
            print(f"[-] 第 {i} 组生成失败: {e}")

    # 4. 写入 TXT 文件，方便离线查看和传递
    with open(output_file, "w", encoding="utf-8") as f:
        if all_keys:
            content = "\n\n".join(format_item_as_text(item) for item in all_keys)
            f.write(content + "\n")
        else:
            f.write("未生成任何数据。\n")
    
    print(f"\n✨ 完成！结果已存入：{output_file}")
    print("⚠️ 提示：此 TXT 文件不包含任何盐值，每组密钥都是随机生成的助记词。")

if __name__ == "__main__":
    try:
        num_accounts = int(input("请输入总套数: "))
        generate_ultra_safe_batch(num_accounts)
    except ValueError:
        print("错误：请输入数字")
