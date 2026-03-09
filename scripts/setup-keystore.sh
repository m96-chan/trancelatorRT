#!/usr/bin/env bash
set -euo pipefail

# Generate Android signing keystore for trancelatorRT
# Run this once, then add secrets to GitHub repository settings.
#
# Usage: ./scripts/setup-keystore.sh

KEYSTORE_FILE="trancelatorrt-release.jks"
KEY_ALIAS="trancelatorrt"

echo "=== Android Keystore Setup ==="
echo ""

if [ -f "$KEYSTORE_FILE" ]; then
    echo "Keystore already exists: ${KEYSTORE_FILE}"
    echo "Delete it first if you want to regenerate."
    exit 1
fi

echo "Generating keystore: ${KEYSTORE_FILE}"
echo "Key alias: ${KEY_ALIAS}"
echo ""

keytool -genkeypair \
    -v \
    -keystore "$KEYSTORE_FILE" \
    -keyalg RSA \
    -keysize 2048 \
    -validity 10000 \
    -alias "$KEY_ALIAS" \
    -dname "CN=trancelatorRT, O=m96-chan"

echo ""
echo "=== Keystore generated: ${KEYSTORE_FILE} ==="
echo ""
echo "Add these GitHub repository secrets:"
echo ""
echo "  ANDROID_KEYSTORE_BASE64:"
echo "    base64 -w0 ${KEYSTORE_FILE}"
echo ""
echo "  ANDROID_KEYSTORE_PASSWORD: (the password you entered)"
echo "  ANDROID_KEY_ALIAS: ${KEY_ALIAS}"
echo "  ANDROID_KEY_PASSWORD: (the password you entered)"
echo ""
echo "IMPORTANT: Do NOT commit the .jks file to git!"
echo ""
