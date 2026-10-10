#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
NOTES_FILE="${2:-}"

if [[ -z "$VERSION" || -z "$NOTES_FILE" ]]; then
  echo "用法: pnpm release <version> <release-notes.md>" >&2
  echo "例如: pnpm release 1.0.6 release-notes.md" >&2
  echo "（release-notes.md 需包含「更新亮点」等内容，用于 GitHub Release 说明）" >&2
  exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "版本号格式无效：$VERSION（应为 x.y.z）" >&2
  exit 1
fi

if [[ ! -f "$NOTES_FILE" ]]; then
  echo "未找到 release notes 文件：$NOTES_FILE" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TAG="v$VERSION"

if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  echo "标签 $TAG 已存在，请更换版本号" >&2
  exit 1
fi

if ! command -v gh >/dev/null; then
  echo "未找到 gh 命令，请先安装 GitHub CLI" >&2
  exit 1
fi

echo "==> 更新版本号至 $VERSION"
sed -i.bak -E "s/(\"version\": \")[^\"]*(\")/\1$VERSION\2/" package.json
sed -i.bak -E "s/(\"version\": \")[^\"]*(\")/\1$VERSION\2/" src-tauri/tauri.conf.json
sed -i.bak -E "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
rm -f package.json.bak src-tauri/tauri.conf.json.bak src-tauri/Cargo.toml.bak

echo "==> 打包（tauri build）"
pnpm tauri build

case "$(uname -m)" in
  arm64 | aarch64) ARCH=aarch64 ;;
  *) ARCH=x64 ;;
esac
DMG_DIR="src-tauri/target/release/bundle/dmg"
DMG="$(find "$DMG_DIR" -maxdepth 1 -type f -name "*_${VERSION}_${ARCH}.dmg" ! -name "rw.*" | head -1)"
if [[ -z "$DMG" || ! -f "$DMG" ]]; then
  echo "未找到打包产物（$DMG_DIR/*_${VERSION}_${ARCH}.dmg）" >&2
  exit 1
fi
echo "==> 打包产物：$DMG"

echo "==> 提交并打标签 $TAG"
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "Release $TAG"
git tag "$TAG"

echo "==> 推送 main 与标签"
git push origin HEAD
git push origin "$TAG"

echo "==> 创建 GitHub Release"
gh release create "$TAG" --title "Todo $TAG" --notes-file "$NOTES_FILE" "$DMG"

echo "==> 完成：已发布 $TAG"
