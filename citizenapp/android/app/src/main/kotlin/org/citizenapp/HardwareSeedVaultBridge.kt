package org.citizenapp

// 硬件绑定 seed 金库的 Android 原生桥（信封加密 + auth-bound KEK）。
//
// 设计（详见任务卡 20260709-citizenapp-hardware-cryptoobject-seed-vault）：
// - 硬件 KEK = Keystore RSA-2048/OAEP，公钥加密（写静默）、私钥经 BiometricPrompt.
//   CryptoObject 解密（读弹生物识别，每次一验，令牌原子绑定 doFinal）。
// - 混合信封：随机 AES-256-GCM DEK 加密明文，KEK 公钥 wrap DEK；规避 24 词助记词
//   超 RSA 块，且加解密体积无关。
// - 两档（tier）访问控制不同：
//   · strict（seed）：仅强生物识别，增删指纹即永久失效（invalidatedByEnrollment=true）。
//   · recovery（助记词）：强生物识别 或 设备凭证（PIN/图案/密码），扛换指纹（false）。
// - OAEP 铁律：MGF1 掩码摘要必须 SHA-1（主摘要 SHA-256），否则 keystore2 抛
//   INCOMPATIBLE_MGF_DIGEST。加解密两端逐字节共用 oaepSpec()。
//
// 本桥只管 KEK 生命周期 + 信封加解密；密文 blob 由 Dart 侧持久化（flutter_secure_storage）。

import android.app.KeyguardManager
import android.content.Context
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.KeyProperties
import android.util.Base64
import android.util.Log
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import io.flutter.plugin.common.MethodChannel
import java.io.ByteArrayOutputStream
import java.security.KeyFactory
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PrivateKey
import java.security.SecureRandom
import java.security.spec.MGF1ParameterSpec
import java.security.spec.X509EncodedKeySpec
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.OAEPParameterSpec
import javax.crypto.spec.PSource
import javax.crypto.spec.SecretKeySpec

class HardwareSeedVaultBridge(private val activity: FragmentActivity) {
    companion object {
        private const val TAG = "HW_SEED_VAULT"
        private const val KEYSTORE = "AndroidKeyStore"
        private const val RSA_TRANSFORM = "RSA/ECB/OAEPPadding"
        private const val AES_TRANSFORM = "AES/GCM/NoPadding"
        private const val DEK_BITS = 256
        private const val GCM_IV_BYTES = 12
        private const val GCM_TAG_BITS = 128
        private const val BLOB_VERSION = 1

        const val TIER_STRICT = "strict"
        const val TIER_RECOVERY = "recovery"

        private fun aliasFor(tier: String, walletIndex: Int) =
            "gmb_${tier}_kek_v1_$walletIndex"
    }

    // 主摘要 SHA-256、MGF1 掩码摘要必须 SHA-1（AndroidKeyStore OAEP 变换内部恒用 SHA-1）。
    private fun oaepSpec() = OAEPParameterSpec(
        "SHA-256", "MGF1", MGF1ParameterSpec.SHA1, PSource.PSpecified.DEFAULT
    )

    private fun keyStore() = KeyStore.getInstance(KEYSTORE).apply { load(null) }

    // ---- 咨询用能力查询 ----

    fun authStatus(): Map<String, Any> {
        val strongBiometric =
            BiometricManager.from(activity)
                .canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG) ==
                BiometricManager.BIOMETRIC_SUCCESS
        val keyguard = activity.getSystemService(Context.KEYGUARD_SERVICE) as KeyguardManager
        val deviceSecure = keyguard.isDeviceSecure
        return mapOf(
            "sdk" to Build.VERSION.SDK_INT,
            "strongBiometricEnrolled" to strongBiometric,
            "deviceSecure" to deviceSecure,
        )
    }

    // ---- KEK 生命周期 ----

    private fun ensureKey(tier: String, walletIndex: Int) {
        val ks = keyStore()
        val alias = aliasFor(tier, walletIndex)
        if (ks.containsAlias(alias)) return

        val authTypes = when (tier) {
            TIER_STRICT -> KeyProperties.AUTH_BIOMETRIC_STRONG
            TIER_RECOVERY ->
                KeyProperties.AUTH_BIOMETRIC_STRONG or KeyProperties.AUTH_DEVICE_CREDENTIAL
            else -> throw IllegalArgumentException("unknownTier:$tier")
        }
        val builder = KeyGenParameterSpec.Builder(
            alias,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
        )
            .setDigests(KeyProperties.DIGEST_SHA256)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_RSA_OAEP)
            .setKeySize(2048)
            .setUserAuthenticationRequired(true)
            // strict：增删指纹即永久失效 → 走助记词自愈；recovery：扛换指纹（锚定设备凭证）。
            .setInvalidatedByBiometricEnrollment(tier == TIER_STRICT)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            // 0 = 每次使用都需认证；靠 CryptoObject 携令牌原子解锁。
            builder.setUserAuthenticationParameters(0, authTypes)
        } else {
            @Suppress("DEPRECATION")
            builder.setUserAuthenticationValidityDurationSeconds(-1)
        }
        val kpg = KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_RSA, KEYSTORE)
        kpg.initialize(builder.build())
        kpg.generateKeyPair()
        Log.i(TAG, "ensureKey: generated $alias tier=$tier")
    }

    fun deleteKey(tier: String, walletIndex: Int) {
        val ks = keyStore()
        val alias = aliasFor(tier, walletIndex)
        if (ks.containsAlias(alias)) ks.deleteEntry(alias)
        Log.i(TAG, "deleteKey: $alias")
    }

    // ---- 写：公钥静默加密（混合信封）----

    fun encrypt(tier: String, walletIndex: Int, plaintext: String): String {
        ensureKey(tier, walletIndex)
        val ks = keyStore()
        val alias = aliasFor(tier, walletIndex)

        // 随机 AES-256 DEK 加密明文（软件，静默）。
        val dek = KeyGenerator.getInstance("AES").apply { init(DEK_BITS) }.generateKey()
        val iv = ByteArray(GCM_IV_BYTES).also { SecureRandom().nextBytes(it) }
        val aes = Cipher.getInstance(AES_TRANSFORM)
        aes.init(Cipher.ENCRYPT_MODE, dek, GCMParameterSpec(GCM_TAG_BITS, iv))
        val body = aes.doFinal(plaintext.toByteArray(Charsets.UTF_8))

        // KEK 公钥 wrap DEK（公钥操作不需认证，静默）。剥离 keystore 授权元数据。
        val pub = ks.getCertificate(alias).publicKey
        val unrestricted = KeyFactory.getInstance(pub.algorithm)
            .generatePublic(X509EncodedKeySpec(pub.encoded))
        val rsa = Cipher.getInstance(RSA_TRANSFORM)
        rsa.init(Cipher.ENCRYPT_MODE, unrestricted, oaepSpec())
        val wrappedDek = rsa.doFinal(dek.encoded)

        val blob = ByteArrayOutputStream().apply {
            write(BLOB_VERSION)
            write((wrappedDek.size ushr 8) and 0xff)
            write(wrappedDek.size and 0xff)
            write(wrappedDek)
            write(iv)
            write(body)
        }.toByteArray()
        Log.i(TAG, "encrypt: OK (silent) tier=$tier idx=$walletIndex blobLen=${blob.size}")
        return Base64.encodeToString(blob, Base64.NO_WRAP)
    }

    // ---- 读：私钥经 BiometricPrompt.CryptoObject 解密 → 软件 AES 解密 ----

    fun decrypt(tier: String, walletIndex: Int, blobB64: String, result: MethodChannel.Result) {
        val raw = Base64.decode(blobB64, Base64.NO_WRAP)
        if (raw.isEmpty() || raw[0].toInt() != BLOB_VERSION) {
            result.error("badBlob", "unsupported blob version", null)
            return
        }
        val wlen = ((raw[1].toInt() and 0xff) shl 8) or (raw[2].toInt() and 0xff)
        val wrappedDek = raw.copyOfRange(3, 3 + wlen)
        val iv = raw.copyOfRange(3 + wlen, 3 + wlen + GCM_IV_BYTES)
        val body = raw.copyOfRange(3 + wlen + GCM_IV_BYTES, raw.size)

        val ks = keyStore()
        val alias = aliasFor(tier, walletIndex)
        val rsa = Cipher.getInstance(RSA_TRANSFORM)
        try {
            val priv = ks.getKey(alias, null) as PrivateKey
            rsa.init(Cipher.DECRYPT_MODE, priv, oaepSpec())
        } catch (e: KeyPermanentlyInvalidatedException) {
            Log.w(TAG, "decrypt: KEY_PERMANENTLY_INVALIDATED $alias", e)
            result.error("keyPermanentlyInvalidated", e.message, null)
            return
        } catch (e: Exception) {
            Log.e(TAG, "decrypt: init failed $alias", e)
            result.error("initFailed", e.message, null)
            return
        }

        val callback = object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationError(code: Int, msg: CharSequence) {
                Log.w(TAG, "decrypt: auth error code=$code msg=$msg")
                val mapped = when (code) {
                    BiometricPrompt.ERROR_USER_CANCELED,
                    BiometricPrompt.ERROR_NEGATIVE_BUTTON,
                    BiometricPrompt.ERROR_CANCELED -> "userCancelled"
                    BiometricPrompt.ERROR_LOCKOUT,
                    BiometricPrompt.ERROR_LOCKOUT_PERMANENT -> "lockout"
                    BiometricPrompt.ERROR_NO_BIOMETRICS,
                    BiometricPrompt.ERROR_NO_DEVICE_CREDENTIAL -> "notEnrolled"
                    BiometricPrompt.ERROR_HW_NOT_PRESENT,
                    BiometricPrompt.ERROR_HW_UNAVAILABLE -> "unavailable"
                    else -> "authError"
                }
                result.error(mapped, msg.toString(), code)
            }

            override fun onAuthenticationSucceeded(auth: BiometricPrompt.AuthenticationResult) {
                try {
                    val authed = auth.cryptoObject?.cipher
                        ?: throw IllegalStateException("cryptoObject.cipher == null")
                    val dekBytes = authed.doFinal(wrappedDek)
                    val aes = Cipher.getInstance(AES_TRANSFORM)
                    aes.init(
                        Cipher.DECRYPT_MODE,
                        SecretKeySpec(dekBytes, "AES"),
                        GCMParameterSpec(GCM_TAG_BITS, iv),
                    )
                    val plain = aes.doFinal(body)
                    val text = String(plain, Charsets.UTF_8)
                    dekBytes.fill(0)
                    plain.fill(0)
                    Log.i(TAG, "decrypt: SUCCESS tier=$tier idx=$walletIndex")
                    result.success(text)
                } catch (e: KeyPermanentlyInvalidatedException) {
                    Log.w(TAG, "decrypt: doFinal invalidated", e)
                    result.error("keyPermanentlyInvalidated", e.message, null)
                } catch (e: Exception) {
                    Log.e(TAG, "decrypt: doFinal failed", e)
                    result.error("doFinalFailed", e.message, null)
                }
            }

            override fun onAuthenticationFailed() {
                Log.i(TAG, "decrypt: one attempt failed (not fatal)")
            }
        }

        val prompt = BiometricPrompt(activity, ContextCompat.getMainExecutor(activity), callback)
        val allowed = when (tier) {
            TIER_STRICT -> BiometricManager.Authenticators.BIOMETRIC_STRONG
            else -> BiometricManager.Authenticators.BIOMETRIC_STRONG or
                BiometricManager.Authenticators.DEVICE_CREDENTIAL
        }
        val infoBuilder = BiometricPrompt.PromptInfo.Builder()
            .setTitle("验证身份")
            .setSubtitle("解锁钱包密钥以继续")
            .setAllowedAuthenticators(allowed)
        // 允许设备凭证时不能设 negative button（互斥）；纯生物识别时必须设。
        if (tier == TIER_STRICT) {
            infoBuilder.setNegativeButtonText("取消")
        }
        Log.i(TAG, "decrypt: showing BiometricPrompt tier=$tier idx=$walletIndex")
        prompt.authenticate(infoBuilder.build(), BiometricPrompt.CryptoObject(rsa))
    }
}
