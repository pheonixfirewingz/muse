#!/usr/bin/env bash
set -e

echo "🔧 Setting up environment..."

# Directory for user-local .NET installation
DOTNET_ROOT="$HOME/dotnet"
export DOTNET_ROOT
export PATH="$DOTNET_ROOT:$PATH"

# Install .NET 9 if missing
if ! dotnet --list-sdks | grep -q "^9\."; then
    echo ".NET 9 SDK not found. Installing..."
    TMP_DIR=$(mktemp -d)
    cd $TMP_DIR
    wget https://dot.net/v1/dotnet-install.sh -O dotnet-install.sh
    chmod +x dotnet-install.sh
    ./dotnet-install.sh --version 9.0.305 --install-dir "$DOTNET_ROOT"
    cd -
    echo ".NET 9 installed to $DOTNET_ROOT"
fi

echo "🚀 Building fat executables..."

# Platforms to build
targets=(win-x64 linux-x64 osx-x64 linux-arm64 osx-arm64)

for r in "${targets[@]}"; do
    echo "Building for $r..."
    dotnet publish ./YtExplodeCli.csproj \
        -c Release \
        -r "$r" \
        -p:PublishSingleFile=true \
        -p:SelfContained=true \
        -p:PublishTrimmed=false \
        -p:EnableCompressionInSingleFile=true \
        -p:IncludeNativeLibrariesForSelfExtract=true \
        -o "publish/$r"
done

echo "✅ All platform builds complete."

# Optional: compress with UPX if available
if command -v upx &> /dev/null; then
    echo "🪶 Compressing executables with UPX..."
    find publish -type f -perm /111 -exec upx --best --lzma {} \;
fi

echo "🎉 Done!"
