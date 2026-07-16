# Card 05 打包前置(Windows):把 onchina 二进制 + 前端产物 + china.sqlite + PostgreSQL
# 官方二进制 + 创世链状态包组装到 node\{binaries,resources}。之后在 node\ 跑
# `npm run tauri build` 产安装包。
#
# 用法:
#   $env:CITIZENCHAIN_PG_DIST = "<postgresql.org 官方二进制解压目录(含 bin\lib\share)>"
#   $env:CITIZENCHAIN_GENESIS_STATE_DIR = "<bake-chainspec.sh 生成的 genesis-state 目录>"
#   citizenchain\scripts\prepack.ps1
$ErrorActionPreference = "Stop"

$Root = (Resolve-Path "$PSScriptRoot\..").Path          # citizenchain\
$Here = (Join-Path $Root "node")                        # citizenchain\node

function Assert-GenesisStatePackage([string]$PackageRoot) {
  $manifestPath = Join-Path $PackageRoot "manifest.json"
  $dbPath = Join-Path $PackageRoot "chains\citizenchain\db"
  if (-not (Test-Path $manifestPath -PathType Leaf) -or -not (Test-Path $dbPath -PathType Container)) {
    throw "创世状态包缺少 manifest.json 或 chains\\citizenchain\\db:$PackageRoot"
  }
  $rootPath = (Resolve-Path $PackageRoot).Path
  Get-ChildItem -LiteralPath $rootPath -Force -Recurse | ForEach-Object {
    if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) {
      throw "创世状态包禁止符号链接:$($_.FullName)"
    }
    $relative = $_.FullName.Substring($rootPath.Length).TrimStart('\', '/').Replace('\', '/')
    if ($relative -ne "manifest.json" -and
        $relative -ne "chains" -and
        $relative -ne "chains/citizenchain" -and
        $relative -ne "chains/citizenchain/db" -and
        -not $relative.StartsWith("chains/citizenchain/db/")) {
      throw "创世状态包包含白名单外残留:$relative"
    }
  }
  $manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
  $required = @("artifact_stage", "genesis_hash", "state_root", "chainspec_hash", "runtime_wasm_hash", "runtime_wasm_ci_run_id", "runtime_wasm_ci_head_sha", "light_sync_state_hash", "public_institution_root")
  if ($manifest.package_format -ne "citizenchain-genesis-state-v1" -or $manifest.chain_id -ne "citizenchain") {
    throw "创世状态包 manifest 身份无效"
  }
  if ($manifest.artifact_stage -ne "release") {
    throw "安装包禁止使用 preview 创世状态包"
  }
  foreach ($field in $required) {
    if (-not $manifest.$field) { throw "创世状态包 manifest 缺少字段:$field" }
  }
  if (@($manifest.included_paths).Count -ne 1 -or $manifest.included_paths[0] -ne "chains/citizenchain/db") {
    throw "创世状态包 manifest.included_paths 无效"
  }
  $nodeSpecPath = Join-Path $Root "node\chainspecs\citizenchain.plain.json"
  $nodeSpecHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $nodeSpecPath).Hash.ToLowerInvariant()
  if ($manifest.chainspec_hash -ne $nodeSpecHash) {
    throw "创世状态包与当前冻结 node plain chainspec 不一致"
  }
}

$GenesisStateSource = if ($env:CITIZENCHAIN_GENESIS_STATE_DIR) { $env:CITIZENCHAIN_GENESIS_STATE_DIR } else { "$Root\target\chainspec\genesis-state" }
try {
  # 先失败关闭，再开始耗时构建；preview 包和不匹配的 node spec 都不得进入安装资源。
  Assert-GenesisStatePackage $GenesisStateSource
} catch {
  Write-Error "创世状态包缺失、不是 release、与冻结 spec 不一致或包含白名单外残留:$GenesisStateSource。$($_.Exception.Message)"
  exit 1
}

Write-Host "[prepack] build onchina (release)"
Push-Location $Root; cargo build -p onchina --release; Pop-Location

Write-Host "[prepack] build onchina frontend"
Push-Location "$Root\onchina\frontend"; npm ci; npm run build; Pop-Location

Write-Host "[prepack] assemble node\resources"
New-Item -ItemType Directory -Force -Path "$Here\resources\onchina-bin", "$Here\resources\onchina-frontend", "$Here\resources\postgres", "$Here\resources\genesis-state" | Out-Null
# onchina 二进制随包(Tauri resources\onchina-bin),onchina_proc 从资源目录解析。
Copy-Item "$Root\target\release\onchina.exe" "$Here\resources\onchina-bin\onchina.exe" -Force
if (Test-Path "$Here\resources\onchina-frontend\dist") { Remove-Item -Recurse -Force "$Here\resources\onchina-frontend\dist" }
Copy-Item -Recurse "$Root\onchina\frontend\dist" "$Here\resources\onchina-frontend\dist"

# PostgreSQL 官方二进制(postgresql.org):CITIZENCHAIN_PG_DIST 指向已解压目录(含 bin\lib\share)。
if ($env:CITIZENCHAIN_PG_DIST -and (Test-Path "$($env:CITIZENCHAIN_PG_DIST)\bin")) {
  $dst = "$Here\resources\postgres\windows"
  if (Test-Path $dst) { Remove-Item -Recurse -Force $dst }
  New-Item -ItemType Directory -Force -Path $dst | Out-Null
  Copy-Item -Recurse "$($env:CITIZENCHAIN_PG_DIST)\*" $dst
  Write-Host "[prepack] PostgreSQL 已组装(windows)"
} else {
  Write-Host "[prepack][warn] 未提供 CITIZENCHAIN_PG_DIST。请从 https://www.postgresql.org/download/windows/"
  Write-Host "                取官方二进制(含 bin\lib\share),解压后设 CITIZENCHAIN_PG_DIST 再重跑;否则安装包不含内嵌 PG。"
}

$dst = "$Here\resources\genesis-state"
if (Test-Path $dst) { Remove-Item -Recurse -Force $dst }
New-Item -ItemType Directory -Force -Path "$dst\chains\citizenchain" | Out-Null
Copy-Item -LiteralPath "$GenesisStateSource\manifest.json" -Destination "$dst\manifest.json"
Copy-Item -Recurse -LiteralPath "$GenesisStateSource\chains\citizenchain\db" -Destination "$dst\chains\citizenchain\db"
Write-Host "[prepack] 创世链状态包已组装:$GenesisStateSource"

Write-Host "[prepack] done. 接着在 node\ 执行: npm run tauri build"
