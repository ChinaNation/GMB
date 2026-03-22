import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  private var blurView: UIVisualEffectView?
  private var screenshotProtectionEnabled = false

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    let result = super.application(application, didFinishLaunchingWithOptions: launchOptions)

    if let controller = window?.rootViewController as? FlutterViewController {
      let channel = FlutterMethodChannel(
        name: "com.wuminapp.wumin/security",
        binaryMessenger: controller.binaryMessenger
      )
      channel.setMethodCallHandler { [weak self] call, result in
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
    }

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
    // 尝试写入受保护路径
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
