//! GPU PoW miner using OpenCL.
//!
//! This module is only compiled when the `gpu-mining` feature is enabled.
//! It provides a GPU-accelerated blake2b-256 hash search that runs alongside
//! the CPU miner, using the upper half of the nonce space (bit 63 = 1).

use crate::service::SimplePow;
use citizenchain::opaque::Block;
use codec::Encode;
use ocl::{Buffer, Kernel, MemFlags, ProQue, Queue};
use sc_consensus_pow::MiningHandle;
use sp_core::U256;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

/// GPU 哈希率（hashes/sec），以 f64 的 bits 存储在 AtomicU64 中。
/// 全局变量，供 RPC 接口读取。
static GPU_HASHRATE: AtomicU64 = AtomicU64::new(0);

/// 获取当前 GPU 哈希率（hashes/sec）。
pub fn gpu_hashrate() -> f64 {
    f64::from_bits(GPU_HASHRATE.load(Ordering::Relaxed))
}

/// OpenCL kernel source embedded at compile time.
const KERNEL_SRC: &str = include_str!("../kernels/blake2b_pow.cl");

/// Number of nonces to test per GPU batch dispatch.
/// 2^24 = ~16 million — good balance between GPU utilization and responsiveness.
const DEFAULT_BATCH_SIZE: u32 = 1 << 24;

/// GPU miner state holding OpenCL resources.
struct GpuMiner {
    pro_que: ProQue,
    // Persistent GPU buffers (reused across batches).
    buf_pre_hash: Buffer<u8>,
    buf_target: Buffer<u64>,
    buf_result_nonce: Buffer<u64>,
    buf_found: Buffer<u32>,
    batch_size: u32,
}

impl GpuMiner {
    /// Try to initialize the GPU miner on the given device.
    /// Returns Err if no GPU is available or OpenCL initialization fails.
    fn try_init(device_index: usize) -> Result<Self, String> {
        let platform = ocl::Platform::default();
        let devices = ocl::Device::list(platform, Some(ocl::flags::DeviceType::GPU))
            .map_err(|e| format!("failed to list GPU devices: {e}"))?;

        if devices.is_empty() {
            return Err("no GPU devices found".into());
        }

        let device = devices
            .get(device_index)
            .ok_or_else(|| {
                format!(
                    "GPU device index {} out of range (found {} devices)",
                    device_index,
                    devices.len()
                )
            })?
            .clone();

        let device_name = device.name().unwrap_or_else(|_| "unknown".into());
        log::info!(
            "Initializing GPU miner on device {}: {}",
            device_index,
            device_name
        );

        let batch_size = DEFAULT_BATCH_SIZE;

        let pro_que = ProQue::builder()
            .platform(platform)
            .device(device)
            .src(KERNEL_SRC)
            .dims(batch_size as usize)
            .build()
            .map_err(|e| format!("failed to build OpenCL program: {e}"))?;

        let buf_pre_hash = Buffer::<u8>::builder()
            .queue(pro_que.queue().clone())
            .flags(MemFlags::new().read_only())
            .len(32)
            .build()
            .map_err(|e| format!("failed to create pre_hash buffer: {e}"))?;

        let buf_target = Buffer::<u64>::builder()
            .queue(pro_que.queue().clone())
            .flags(MemFlags::new().read_only())
            .len(4)
            .build()
            .map_err(|e| format!("failed to create target buffer: {e}"))?;

        let buf_result_nonce = Buffer::<u64>::builder()
            .queue(pro_que.queue().clone())
            .flags(MemFlags::new().write_only())
            .len(1)
            .build()
            .map_err(|e| format!("failed to create result_nonce buffer: {e}"))?;

        let buf_found = Buffer::<u32>::builder()
            .queue(pro_que.queue().clone())
            .flags(MemFlags::new().read_write())
            .len(1)
            .build()
            .map_err(|e| format!("failed to create found buffer: {e}"))?;

        Ok(GpuMiner {
            pro_que,
            buf_pre_hash,
            buf_target,
            buf_result_nonce,
            buf_found,
            batch_size,
        })
    }

    /// Run a single batch of nonce searches on the GPU.
    /// Returns Some(nonce) if a valid nonce is found, None otherwise.
    fn search_batch(
        &self,
        pre_hash: &[u8],
        target_be: &[u64; 4],
        nonce_base: u64,
    ) -> Result<Option<u64>, String> {
        // Upload pre_hash and target to GPU.
        self.buf_pre_hash
            .write(pre_hash)
            .enq()
            .map_err(|e| format!("write pre_hash: {e}"))?;
        self.buf_target
            .write(target_be.as_slice())
            .enq()
            .map_err(|e| format!("write target: {e}"))?;

        // Reset found flag to 0.
        self.buf_found
            .write(&[0u32])
            .enq()
            .map_err(|e| format!("reset found: {e}"))?;

        // Build and enqueue the kernel.
        let kernel = self
            .pro_que
            .kernel_builder("blake2b_pow_mine")
            .arg(&self.buf_pre_hash)
            .arg(nonce_base)
            .arg(&self.buf_target)
            .arg(&self.buf_result_nonce)
            .arg(&self.buf_found)
            .build()
            .map_err(|e| format!("build kernel: {e}"))?;

        unsafe {
            kernel
                .enq()
                .map_err(|e| format!("enqueue kernel: {e}"))?;
        }

        // Read back results.
        let mut found = [0u32; 1];
        self.buf_found
            .read(&mut found)
            .enq()
            .map_err(|e| format!("read found: {e}"))?;

        if found[0] != 0 {
            let mut result_nonce = [0u64; 1];
            self.buf_result_nonce
                .read(&mut result_nonce)
                .enq()
                .map_err(|e| format!("read result_nonce: {e}"))?;
            Ok(Some(result_nonce[0]))
        } else {
            Ok(None)
        }
    }
}

/// Convert U256 difficulty to big-endian target bytes for the GPU kernel.
/// target = U256::MAX / difficulty, stored as 4 x u64 in big-endian word order.
fn difficulty_to_target_be(difficulty: U256) -> [u64; 4] {
    if difficulty.is_zero() {
        return [0u64; 4];
    }
    let target = U256::MAX / difficulty;
    let mut bytes = [0u8; 32];
    target.to_big_endian(&mut bytes);
    // Convert 32 big-endian bytes to 4 big-endian u64 words.
    let mut words = [0u64; 4];
    for i in 0..4 {
        let offset = i * 8;
        words[i] = u64::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
    }
    words
}

/// CPU 矿工的提交门控时刻（纳秒），GPU 矿工共享同一个门控。
/// 在 service.rs 中由 start_cpu_miner 定义和更新。
use crate::service::LAST_SUBMIT_NS;

/// Try to start the GPU miner. Spawns a background thread.
/// Returns Ok(()) if GPU initialization succeeded, Err otherwise.
pub fn try_start<Proof: Send + 'static>(
    worker: MiningHandle<Block, SimplePow, (), Proof>,
    device_index: usize,
    epoch: Instant,
    pool_ready: std::sync::Arc<dyn Fn() -> usize + Send + Sync>,
    target_block_time_ms: u64,
) -> Result<(), String> {
    let miner = GpuMiner::try_init(device_index)?;

    thread::spawn(move || {
        // 中文注释：出块目标时间从 chain-phase-control Runtime API 读取，替代编译期常量。
        let min_submit_interval = Duration::from_millis(target_block_time_ms);
        let batch_size = miner.batch_size;

        loop {
            let Some(metadata) = worker.metadata() else {
                thread::sleep(Duration::from_millis(200));
                continue;
            };

            // 空块不提交：交易池无待打包交易时不挖矿，避免产生空块。
            if pool_ready() == 0 {
                thread::sleep(Duration::from_millis(500));
                continue;
            }

            let build_version = worker.version();

            // GPU uses upper nonce space (bit 63 = 1).
            let random_base = {
                let seed_bytes = metadata.pre_hash.as_ref();
                let seed =
                    u64::from_le_bytes(seed_bytes[..8].try_into().unwrap_or([0u8; 8]));
                seed | 0x8000000000000000
            };
            let mut nonce_base = random_base;

            let target_be = difficulty_to_target_be(metadata.difficulty);

            loop {
                if worker.version() != build_version {
                    break;
                }

                let batch_start = Instant::now();
                match miner.search_batch(metadata.pre_hash.as_ref(), &target_be, nonce_base) {
                    Ok(Some(nonce)) => {
                        let elapsed = batch_start.elapsed();
                        if elapsed.as_nanos() > 0 {
                            let hr = batch_size as f64 / elapsed.as_secs_f64();
                            GPU_HASHRATE.store(hr.to_bits(), Ordering::Relaxed);
                        }
                        // 无锁提交门控，与 CPU 矿工共享 LAST_SUBMIT_NS。
                        // u64::MAX 表示"从未提交过"，首次直接放行。
                        let last_ns = LAST_SUBMIT_NS.load(Ordering::Acquire);
                        if last_ns != u64::MAX {
                            let now_ns = epoch.elapsed().as_nanos() as u64;
                            let interval_ns = min_submit_interval.as_nanos() as u64;
                            let deadline_ns = last_ns.saturating_add(interval_ns);
                            if now_ns < deadline_ns {
                                let wait = Duration::from_nanos(deadline_ns - now_ns);
                                thread::sleep(wait);
                            }
                        }

                        // sleep 后检查 build 是否仍有效。
                        if worker.version() != build_version {
                            break;
                        }

                        let submitted =
                            futures::executor::block_on(worker.submit(nonce.encode()));
                        if submitted {
                            let submit_ns = epoch.elapsed().as_nanos() as u64;
                            if pool_ready() > 0 {
                                // 与 CPU 矿工同理：空块后缩短门控为 MinPeriod。
                                let half_ns = min_submit_interval.as_nanos() as u64 / 2;
                                LAST_SUBMIT_NS.store(
                                    submit_ns.saturating_sub(half_ns),
                                    Ordering::Release,
                                );
                            } else {
                                LAST_SUBMIT_NS.store(submit_ns, Ordering::Release);
                            }
                        }
                        break;
                    }
                    Ok(None) => {
                        // 更新哈希率统计。
                        let elapsed = batch_start.elapsed();
                        if elapsed.as_nanos() > 0 {
                            let hr = batch_size as f64 / elapsed.as_secs_f64();
                            GPU_HASHRATE.store(hr.to_bits(), Ordering::Relaxed);
                        }
                        // No solution in this batch, advance nonce_base.
                        nonce_base = nonce_base.wrapping_add(batch_size as u64);
                    }
                    Err(e) => {
                        log::error!("GPU mining error: {e}");
                        // Back off before retrying to avoid spamming logs.
                        thread::sleep(Duration::from_secs(5));
                        break;
                    }
                }
            }
        }
    });

    Ok(())
}
