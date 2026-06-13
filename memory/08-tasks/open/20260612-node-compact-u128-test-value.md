# 任务卡:node 既有测试 compact_u128_big_integer 取值错误

ADR-017 卡3 执行中发现的既有失败(与 ADR-017 无关,git stash 复跑确认):测试断言 1_000_000 走 SCALE big-integer 模式,但 1_000_000 < 2^30,正确编码是 four-byte 模式——**测试取值写错,实现是对的**。修法:把用例取值换成 ≥ 2^30 的数(如 2_000_000_000_000)再断言 big-integer 模式。
