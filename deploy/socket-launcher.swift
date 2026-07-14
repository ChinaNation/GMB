import Darwin
import Foundation

// 中文注释：从 launchd 取得 41731 的已监听 socket，转交给 Node，控制台无需常驻等待。
@_silgen_name("launch_activate_socket")
private func launchActivateSocket(
  _ name: UnsafePointer<CChar>,
  _ fileDescriptors: UnsafeMutablePointer<UnsafeMutablePointer<Int32>?>,
  _ count: UnsafeMutablePointer<Int>
) -> Int32

guard CommandLine.arguments.count == 3 else {
  fputs("缺少 Node 或 server.mjs 路径\n", stderr)
  exit(2)
}

var descriptors: UnsafeMutablePointer<Int32>?
var descriptorCount = 0
let activationStatus = "Listeners".withCString {
  launchActivateSocket($0, &descriptors, &descriptorCount)
}
guard activationStatus == 0, descriptorCount > 0, let descriptors else {
  fputs("无法取得 launchd 监听 socket：\(activationStatus)\n", stderr)
  exit(1)
}

let inheritedFD: Int32 = STDIN_FILENO
guard dup2(descriptors[0], inheritedFD) >= 0 else {
  perror("dup2")
  exit(1)
}
for index in 0..<descriptorCount where descriptors[index] != inheritedFD {
  close(descriptors[index])
}
free(descriptors)

setenv("GMB_DEPLOY_FD", "0", 1)
setenv("GMB_DEPLOY_LAUNCHD", "1", 1)

let arguments = [CommandLine.arguments[1], CommandLine.arguments[2]]
var cArguments = arguments.map { strdup($0) }
cArguments.append(nil)
defer { cArguments.compactMap { $0 }.forEach { free($0) } }

_ = cArguments.withUnsafeMutableBufferPointer { buffer in
  execv(CommandLine.arguments[1], buffer.baseAddress)
}
perror("execv")
exit(1)
