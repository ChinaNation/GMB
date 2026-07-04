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

$GenesisStateSource = if ($env:CITIZENCHAIN_GENESIS_STATE_DIR) { $env:CITIZENCHAIN_GENESIS_STATE_DIR } else { "$Root\target\chainspec\genesis-state" }
if ((Test-Path "$GenesisStateSource\manifest.json") -and (Test-Path "$GenesisStateSource\chains\citizenchain\db")) {
  $dst = "$Here\resources\genesis-state"
  if (Test-Path $dst) { Remove-Item -Recurse -Force $dst }
  New-Item -ItemType Directory -Force -Path $dst | Out-Null
  Copy-Item -Recurse "$GenesisStateSource\*" $dst
  Write-Host "[prepack] 创世链状态包已组装:$GenesisStateSource"
} else {
  Write-Host "[prepack][warn] 未找到创世链状态包:$GenesisStateSource"
  Write-Host "                正式安装包必须先执行 bake-chainspec.sh --finalize --wasm <CI_WASM>,"
  Write-Host "                并让 CITIZENCHAIN_GENESIS_STATE_DIR 指向生成的 genesis-state 目录。"
}

Write-Host "[prepack] done. 接着在 node\ 执行: npm run tauri build"
