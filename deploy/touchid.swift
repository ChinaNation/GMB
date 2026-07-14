import Foundation
import LocalAuthentication

// 中文注释：生产操作只允许生物识别，不使用设备密码降级。
let context = LAContext()
context.localizedFallbackTitle = ""
var error: NSError?
guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) else {
    fputs("当前设备无法使用 Touch ID\n", stderr)
    exit(2)
}

let semaphore = DispatchSemaphore(value: 0)
var authenticated = false
context.evaluatePolicy(
    .deviceOwnerAuthenticationWithBiometrics,
    localizedReason: "授权执行 GMB 生产部署"
) { success, _ in
    authenticated = success
    semaphore.signal()
}
semaphore.wait()
exit(authenticated ? 0 : 1)
