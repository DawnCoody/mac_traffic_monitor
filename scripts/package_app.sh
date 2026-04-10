#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="mac_traffic_monitor"
BUNDLE_NAME="${APP_NAME}.app"
BUNDLE_DIR="${PROJECT_ROOT}/dist/${BUNDLE_NAME}"
CONTENTS_DIR="${BUNDLE_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"
BINARY_PATH="${PROJECT_ROOT}/target/release/${APP_NAME}"
ICON_PATH="${PROJECT_ROOT}/assets/icons/rocket.icns"
PLIST_PATH="${CONTENTS_DIR}/Info.plist"
ZIP_PATH="${PROJECT_ROOT}/dist/${APP_NAME}-macos-app.zip"
DMG_STAGING_DIR="${PROJECT_ROOT}/dist/dmg-staging"
DMG_PATH="${PROJECT_ROOT}/dist/${APP_NAME}.dmg"
VOLUME_NAME="mac_traffic_monitor"
TMP_DMG_PATH="${PROJECT_ROOT}/dist/${APP_NAME}-temp.dmg"

cargo build --release --manifest-path "${PROJECT_ROOT}/Cargo.toml"

rm -rf "${BUNDLE_DIR}"
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

cp "${BINARY_PATH}" "${MACOS_DIR}/${APP_NAME}"
cp "${ICON_PATH}" "${RESOURCES_DIR}/rocket.icns"
chmod +x "${MACOS_DIR}/${APP_NAME}"

cat > "${PLIST_PATH}" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>zh_CN</string>
    <key>CFBundleExecutable</key>
    <string>mac_traffic_monitor</string>
    <key>CFBundleIconFile</key>
    <string>rocket</string>
    <key>CFBundleIdentifier</key>
    <string>com.mactrafficmonitor.app</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>mac_traffic_monitor</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>LSUIElement</key>
    <false/>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

rm -f "${ZIP_PATH}" "${DMG_PATH}" "${TMP_DMG_PATH}"
/usr/bin/ditto -c -k --sequesterRsrc --keepParent "${BUNDLE_DIR}" "${ZIP_PATH}"

rm -rf "${DMG_STAGING_DIR}"
mkdir -p "${DMG_STAGING_DIR}"
cp -R "${BUNDLE_DIR}" "${DMG_STAGING_DIR}/${BUNDLE_NAME}"
ln -s /Applications "${DMG_STAGING_DIR}/Applications"

hdiutil create -volname "${VOLUME_NAME}" -srcfolder "${DMG_STAGING_DIR}" -ov -format UDZO "${DMG_PATH}"
rm -rf "${DMG_STAGING_DIR}"

printf 'Created app bundle: %s\n' "${BUNDLE_DIR}"
printf 'Created release zip: %s\n' "${ZIP_PATH}"
printf 'Created release dmg: %s\n' "${DMG_PATH}"
