import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  private var blurView: UIVisualEffectView?
  private var screenshotProtectionEnabled = false
  private var eventSink: FlutterEventSink?

  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    let result = super.application(application, didFinishLaunchingWithOptions: launchOptions)

    if let controller = window?.rootViewController as? FlutterViewController {
      // MethodChannel: 开关截屏保护、检测越狱
      let methodChannel = FlutterMethodChannel(
        name: "com.wuminapp.wumin/security",
        binaryMessenger: controller.binaryMessenger
      )
      methodChannel.setMethodCallHandler { [weak self] call, result in
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

      // EventChannel: 截屏/录屏事件推送给 Flutter
      let eventChannel = FlutterEventChannel(
        name: "com.wuminapp.wumin/security_events",
        binaryMessenger: controller.binaryMessenger
      )
      eventChannel.setStreamHandler(SecurityEventStreamHandler(delegate: self))
    }

    // 后台模糊
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

    // 截屏监听
    NotificationCenter.default.addObserver(
      self,
      selector: #selector(userDidTakeScreenshot),
      name: UIApplication.userDidTakeScreenshotNotification,
      object: nil
    )

    // 录屏监听
    NotificationCenter.default.addObserver(
      self,
      selector: #selector(screenCaptureDidChange),
      name: UIScreen.capturedDidChangeNotification,
      object: nil
    )

    return result
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
  }

  // MARK: - 后台模糊

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

  // MARK: - 截屏/录屏事件

  @objc private func userDidTakeScreenshot() {
    guard screenshotProtectionEnabled else { return }
    eventSink?("screenshot_taken")
  }

  @objc private func screenCaptureDidChange() {
    guard screenshotProtectionEnabled else { return }
    if UIScreen.main.isCaptured {
      eventSink?("screen_recording_started")
    } else {
      eventSink?("screen_recording_stopped")
    }
  }

  // MARK: - 越狱检测

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

// MARK: - EventChannel StreamHandler

private class SecurityEventStreamHandler: NSObject, FlutterStreamHandler {
  weak var delegate: AppDelegate?

  init(delegate: AppDelegate) {
    self.delegate = delegate
  }

  func onListen(withArguments arguments: Any?, eventSink events: @escaping FlutterEventSink) -> FlutterError? {
    delegate?.eventSink = events
    // 如果当前正在录屏，立即推送
    if UIScreen.main.isCaptured {
      events("screen_recording_started")
    }
    return nil
  }

  func onCancel(withArguments arguments: Any?) -> FlutterError? {
    delegate?.eventSink = nil
    return nil
  }
}
