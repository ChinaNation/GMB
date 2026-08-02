package org.citizenapp

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.fail
import org.junit.Test
import org.junit.runner.RunWith

/// Android Keystore 真实运行态验收：设备子钥删除后，同一 walletIndex 必须生成全新密钥。
///
/// 使用远离产品钱包索引范围的测试专用值，且测试前后都幂等删除别名；不读取钱包数据库、
/// 不触碰用户真实钱包，不访问 Cloudflare 或区块链。
@RunWith(AndroidJUnit4::class)
class DeviceSubkeyBridgeInstrumentedTest {
    private val bridge = DeviceSubkeyBridge()
    private val dataKeyVault = DeviceDataKeyVaultBridge()

    companion object {
        private const val TEST_WALLET_INDEX = 2_147_483_000
    }

    @After
    fun removeTestAlias() {
        bridge.delete(TEST_WALLET_INDEX)
        dataKeyVault.delete(TEST_WALLET_INDEX)
    }

    @Test
    fun deleteRemovesHardwareKeyAndNextReadCreatesDifferentKey() {
        bridge.delete(TEST_WALLET_INDEX)
        val firstPublicKey = bridge.publicKeyHex(TEST_WALLET_INDEX)
        assertEquals(130, firstPublicKey.length)
        assertEquals("04", firstPublicKey.substring(0, 2))

        bridge.delete(TEST_WALLET_INDEX)
        val secondPublicKey = bridge.publicKeyHex(TEST_WALLET_INDEX)
        assertEquals(130, secondPublicKey.length)
        assertEquals("04", secondPublicKey.substring(0, 2))
        assertNotEquals(firstPublicKey, secondPublicKey)
    }

    @Test
    fun deviceDataKeyVaultBindsCiphertextToAadAndDeleteInvalidatesIt() {
        dataKeyVault.delete(TEST_WALLET_INDEX)
        val plaintext = ByteArray(32) { it.toByte() }
        val aad = "genesis|cid|revision|account|chat".toByteArray()
        val blob = dataKeyVault.seal(TEST_WALLET_INDEX, plaintext, aad)

        assertArrayEquals(
            plaintext,
            dataKeyVault.open(TEST_WALLET_INDEX, blob, aad),
        )
        assertFails { dataKeyVault.open(TEST_WALLET_INDEX, blob, "other-binding".toByteArray()) }

        dataKeyVault.delete(TEST_WALLET_INDEX)
        assertFails { dataKeyVault.open(TEST_WALLET_INDEX, blob, aad) }
    }

    private fun assertFails(block: () -> Unit) {
        try {
            block()
            fail("预期设备数据钥操作失败")
        } catch (_: Exception) {
            // AAD 不匹配或硬件钥删除后必须失败关闭。
        }
    }
}
