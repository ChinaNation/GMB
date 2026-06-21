package org.citizenapp

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import android.view.WindowManager
import androidx.core.app.ActivityCompat
import androidx.core.content.FileProvider
import androidx.core.content.ContextCompat
import io.flutter.embedding.android.FlutterFragmentActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import java.io.File

class MainActivity : FlutterFragmentActivity() {
    private val securityChannelName = "org.citizenapp/security"
    private val updateChannelName = "org.citizenapp/update"
    private val permissionsChannelName = "org.citizenapp/permissions"
    private val notificationPermissionRequestCode = 170517
    private var pendingNotificationPermissionResult: MethodChannel.Result? = null

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, securityChannelName)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "enableScreenshotProtection" -> {
                        window.setFlags(
                            WindowManager.LayoutParams.FLAG_SECURE,
                            WindowManager.LayoutParams.FLAG_SECURE
                        )
                        result.success(null)
                    }
                    "disableScreenshotProtection" -> {
                        window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
                        result.success(null)
                    }
                    "isDeviceRooted" -> {
                        result.success(checkRoot())
                    }
                    else -> result.notImplemented()
                }
            }

        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, permissionsChannelName)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "requestNotificationPermission" -> requestNotificationPermission(result)
                    "getNotificationPermissionStatus" ->
                        result.success(isNotificationPermissionGranted())
                    else -> result.notImplemented()
                }
            }

        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, updateChannelName)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "getPackageInfo" -> {
                        val packageInfo = packageManager.getPackageInfo(packageName, 0)
                        val versionCode = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                            packageInfo.longVersionCode
                        } else {
                            @Suppress("DEPRECATION")
                            packageInfo.versionCode.toLong()
                        }
                        result.success(
                            mapOf(
                                "packageName" to packageName,
                                "versionName" to (packageInfo.versionName ?: ""),
                                "versionCode" to versionCode,
                            )
                        )
                    }
                    "installApk" -> {
                        val apkPath = call.argument<String>("apkPath")
                        if (apkPath.isNullOrBlank()) {
                            result.error("INVALID_APK_PATH", "APK 路径为空", null)
                            return@setMethodCallHandler
                        }
                        try {
                            result.success(installApk(File(apkPath)))
                        } catch (error: Exception) {
                            result.error(
                                "INSTALL_APK_FAILED",
                                error.message ?: "拉起系统安装器失败",
                                null
                            )
                        }
                    }
                    else -> result.notImplemented()
                }
            }
    }

    private fun isNotificationPermissionGranted(): Boolean {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            return true
        }
        return ContextCompat.checkSelfPermission(
            this,
            Manifest.permission.POST_NOTIFICATIONS
        ) == PackageManager.PERMISSION_GRANTED
    }

    private fun requestNotificationPermission(result: MethodChannel.Result) {
        if (isNotificationPermissionGranted()) {
            result.success(true)
            return
        }
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            result.success(true)
            return
        }
        if (pendingNotificationPermissionResult != null) {
            result.error("REQUEST_IN_PROGRESS", "通知权限申请正在进行中", null)
            return
        }

        // 中文注释：通知权限只在用户确认首启说明后申请；拒绝不会阻塞 App 使用。
        pendingNotificationPermissionResult = result
        ActivityCompat.requestPermissions(
            this,
            arrayOf(Manifest.permission.POST_NOTIFICATIONS),
            notificationPermissionRequestCode
        )
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        if (requestCode == notificationPermissionRequestCode) {
            val granted = grantResults.isNotEmpty() &&
                grantResults[0] == PackageManager.PERMISSION_GRANTED
            pendingNotificationPermissionResult?.success(granted)
            pendingNotificationPermissionResult = null
            return
        }
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
    }

    private fun installApk(apkFile: File): Boolean {
        if (!apkFile.exists()) {
            throw IllegalArgumentException("APK 文件不存在: ${apkFile.absolutePath}")
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O &&
            !packageManager.canRequestPackageInstalls()
        ) {
            // 中文注释：Android 8+ 必须由用户授权“允许安装未知应用”，App 不能绕过系统确认。
            val intent = Intent(
                Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,
                Uri.parse("package:$packageName")
            )
            startActivity(intent)
            return false
        }

        val apkUri = FileProvider.getUriForFile(
            this,
            "$packageName.update_file_provider",
            apkFile
        )
        val intent = Intent(Intent.ACTION_VIEW).apply {
            setDataAndType(apkUri, "application/vnd.android.package-archive")
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        startActivity(intent)
        return true
    }

    private fun checkRoot(): Boolean {
        val suPaths = arrayOf(
            "/system/bin/su", "/system/xbin/su", "/sbin/su",
            "/data/local/xbin/su", "/data/local/bin/su",
            "/system/sd/xbin/su", "/system/bin/failsafe/su",
            "/data/local/su", "/su/bin/su",
            "/system/app/Superuser.apk",
            "/system/app/SuperSU.apk",
        )
        for (path in suPaths) {
            if (File(path).exists()) return true
        }
        val buildTags = android.os.Build.TAGS
        if (buildTags != null && buildTags.contains("test-keys")) return true
        if (File("/sbin/.magisk").exists()) return true
        if (File("/data/adb/magisk").exists()) return true
        return false
    }
}
