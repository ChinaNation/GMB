import Flutter
import UIKit
import UserNotifications

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  private var blurView: UIVisualEffectView?
  private var screenshotProtectionEnabled = false
  // 原生通道由 AppDelegate 强引用，确保 Flutter engine 生命周期内处理器始终有效。
  private var hardwareBoundSeedVaultChannel: HardwareBoundSeedVaultChannel?
  private var deviceSubkeyChannel: DeviceSubkeyChannel?
  private var securityChannel: FlutterMethodChannel?
  private var permissionsChannel: FlutterMethodChannel?

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    let result = super.application(application, didFinishLaunchingWithOptions: launchOptions)

    NotificationCenter.default.addObserver(
      self,
      selector: #selector(appWillResignActive),
      name: UIApplication.willResignActiveNotification,
      object: nil
    )
    NotificationCenter.default.addObserver(
      self,
      selector: #selector(appDidBecomeActive),
      name: UIApplication.didBecomeActiveNotification,
      object: nil
    )

    return result
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
    // UIScene 模式下 AppDelegate.window 在启动回调中尚未建立；必须使用 engine
    // 提供的 application registrar 注册通道，确保 iOS 16+ 首次启动也必然可用。
    registerApplicationChannels(
      binaryMessenger: engineBridge.applicationRegistrar.messenger()
    )
  }

  private func registerApplicationChannels(binaryMessenger: FlutterBinaryMessenger) {
    hardwareBoundSeedVaultChannel = HardwareBoundSeedVaultChannel(
      binaryMessenger: binaryMessenger
    )
    deviceSubkeyChannel = DeviceSubkeyChannel(binaryMessenger: binaryMessenger)

    let securityChannel = FlutterMethodChannel(
      name: "org.citizenapp/security",
      binaryMessenger: binaryMessenger
    )
    securityChannel.setMethodCallHandler { [weak self] call, result in
      switch call.method {
      case "enableScreenshotProtection":
        self?.screenshotProtectionEnabled = true
        result(nil)
      case "disableScreenshotProtection":
        self?.screenshotProtectionEnabled = false
        self?.removeBlur()
        result(nil)
      case "isDeviceRooted":
        result(AppDelegate.checkJailbreak())
      default:
        result(FlutterMethodNotImplemented)
      }
    }
    self.securityChannel = securityChannel

    let permissionsChannel = FlutterMethodChannel(
      name: "org.citizenapp/permissions",
      binaryMessenger: binaryMessenger
    )
    permissionsChannel.setMethodCallHandler { call, result in
      switch call.method {
      case "requestNotificationPermission":
        // iOS 通知授权必须由 App 主动发起，拒绝后不阻塞进入主界面。
        UNUserNotificationCenter.current().requestAuthorization(
          options: [.alert, .badge, .sound]
        ) { granted, error in
          DispatchQueue.main.async {
            if let error = error {
              result(
                FlutterError(
                  code: "NOTIFICATION_PERMISSION_FAILED",
                  message: error.localizedDescription,
                  details: nil
                )
              )
            } else {
              result(granted)
            }
          }
        }
      case "getNotificationPermissionStatus":
        UNUserNotificationCenter.current().getNotificationSettings { settings in
          let granted = settings.authorizationStatus == .authorized ||
            settings.authorizationStatus == .provisional
          DispatchQueue.main.async {
            result(granted)
          }
        }
      default:
        result(FlutterMethodNotImplemented)
      }
    }
    self.permissionsChannel = permissionsChannel
  }

  @objc private func appWillResignActive() {
    guard screenshotProtectionEnabled else { return }
    addBlur()
  }

  @objc private func appDidBecomeActive() {
    removeBlur()
  }

  private func addBlur() {
    guard blurView == nil, let keyWindow = window else { return }
    let blur = UIVisualEffectView(effect: UIBlurEffect(style: .light))
    blur.frame = keyWindow.bounds
    blur.tag = 999
    keyWindow.addSubview(blur)
    blurView = blur
  }

  private func removeBlur() {
    blurView?.removeFromSuperview()
    blurView = nil
  }

  private static func checkJailbreak() -> Bool {
    #if targetEnvironment(simulator)
    return false
    #else
    let paths = [
      "/Applications/Cydia.app",
      "/Library/MobileSubstrate/MobileSubstrate.dylib",
      "/bin/bash", "/usr/sbin/sshd", "/etc/apt",
      "/private/var/lib/apt/",
      "/usr/bin/ssh",
      "/var/lib/cydia",
      "/var/cache/apt",
      "/var/jb",
    ]
    for path in paths {
      if FileManager.default.fileExists(atPath: path) { return true }
    }
    let testPath = "/private/jailbreak_test_\(UUID().uuidString)"
    do {
      try "test".write(toFile: testPath, atomically: true, encoding: .utf8)
      try FileManager.default.removeItem(atPath: testPath)
      return true
    } catch {
      return false
    }
    #endif
  }
}
