# Card 05 打包前置(Windows):把 registry 二进制 + 前端产物 + china.sqlite + PostgreSQL
# 官方二进制组装到 node\{binaries,resources}。之后在 node\ 跑 `npm run tauri build` 产安装包。
#
# 用法:
#   $env:CITIZENCHAIN_PG_DIST = "<postgresql.org 官方二进制解压目录(含 bin\lib\share)>"
#   citizenchain\scripts\prepack.ps1
$ErrorActionPreference = "Stop"

$Root = (Resolve-Path "$PSScriptRoot\..").Path          # citizenchain\
$Here = (Join-Path $Root "node")                        # citizenchain\node

Write-Host "[prepack] build registry (release)"
Push-Location $Root; cargo build -p registry --release; Pop-Location

Write-Host "[prepack] build registry frontend"
Push-Location "$Root\registry\frontend"; npm ci; npm run build; Pop-Location

Write-Host "[prepack] assemble node\resources"
New-Item -ItemType Directory -Force -Path "$Here\resources\registry-bin", "$Here\resources\registry-frontend", "$Here\resources\postgres" | Out-Null
# registry 二进制随包(Tauri resources\registry-bin),registry_proc 从资源目录解析。
Copy-Item "$Root\target\release\registry.exe" "$Here\resources\registry-bin\registry.exe" -Force
Copy-Item "$Root\registry\src\china\china.sqlite" "$Here\resources\china.sqlite" -Force
if (Test-Path "$Here\resources\registry-frontend\dist") { Remove-Item -Recurse -Force "$Here\resources\registry-frontend\dist" }
Copy-Item -Recurse "$Root\registry\frontend\dist" "$Here\resources\registry-frontend\dist"

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

Write-Host "[prepack] done. 接着在 node\ 执行: npm run tauri build"
