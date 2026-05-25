#!/bin/bash

TARGET_OS=$1
TARGET_ARCH=$2

if [ -z "$TARGET_OS" ] || [ -z "$TARGET_ARCH" ]; then
    echo "错误: 缺少参数"
    echo "用法: $0 <windows|linux|android|macos|ios> <x86_64|x86|aarch64|arm|armeabi-v7a|arm64-v8a|arm64>"
    exit 1
fi

echo "为 $TARGET_OS ($TARGET_ARCH) 生成 FFmpeg 配置..."

OPTIONS=(
    --disable-everything
    --disable-programs
    --disable-network
    --disable-doc
    --enable-avcodec
    --enable-avformat
    --enable-avutil
    --enable-swresample
    --disable-avdevice
    --disable-avfilter
    --disable-swscale
    --enable-protocol=file
    --disable-autodetect
    --disable-asm
    --disable-x86asm
    --disable-inline-asm
)

DEMUXERS="aac,ac3,aiff,amr,ape,asf,au,caf,dsdiff,dsf,dts,dtshd,eac3,flac,iff,m4v,matroska,mov,mp3,mpc,mpc8,ogg,rm,spdif,tak,truehd,tta,w64,wav,wv"
DECODERS="aac,aac_latm,ac3,adpcm_ima_wav,adpcm_ms,adpcm_swf,alac,als,amrnb,amrwb,ape,cook,dca,dsd_lsbf,dsd_lsbf_planar,dsd_msbf,dsd_msbf_planar,eac3,flac,mlp,mp3,mpc7,mpc8,opus,pcm_alaw,pcm_bluray,pcm_dvd,pcm_f32be,pcm_f32le,pcm_f64be,pcm_f64le,pcm_mulaw,pcm_s16be,pcm_s16le,pcm_s24be,pcm_s24le,pcm_s32be,pcm_s32le,pcm_s8,pcm_u16be,pcm_u16le,pcm_u24be,pcm_u24le,pcm_u32be,pcm_u32le,pcm_u8,ra_144,ra_288,shorten,tak,truehd,tta,vorbis,wavpack,wmalossless,wmapro,wmav1,wmav2,wmavoice"
PARSERS="aac,aac_latm,ac3,amr,cook,dca,flac,mlp,mpegaudio,opus,sipr,tak,vorbis,wma"

OPTIONS+=("--enable-demuxer=$DEMUXERS")
OPTIONS+=("--enable-decoder=$DECODERS")
OPTIONS+=("--enable-parser=$PARSERS")

if [ "$TARGET_OS" == "windows" ]; then
    OPTIONS+=(--toolchain=msvc)
    if [ "$TARGET_ARCH" == "x86_64" ]; then
        OPTIONS+=(--target-os=win64 --arch=x86_64)
    elif [ "$TARGET_ARCH" == "x86" ]; then
        OPTIONS+=(--target-os=win32 --arch=i386)
    elif [ "$TARGET_ARCH" == "arm64" ] || [ "$TARGET_ARCH" == "aarch64" ]; then
        OPTIONS+=(--target-os=win32 --arch=aarch64 --enable-cross-compile)
        TARGET_ARCH="arm64"
    else
        echo "不支持的 Windows 架构: $TARGET_ARCH"
        exit 1
    fi
    OPTIONS+=(--extra-cflags=-DHAVE_UNISTD_H=0)

elif [ "$TARGET_OS" == "linux" ]; then
    OPTIONS+=(--target-os=linux)
    if [ "$TARGET_ARCH" == "arm64" ] || [ "$TARGET_ARCH" == "aarch64" ]; then
        OPTIONS+=(--arch=aarch64 --enable-cross-compile --cc=aarch64-linux-gnu-gcc)
        TARGET_ARCH="arm64"
    else
        OPTIONS+=(--arch=$TARGET_ARCH)
    fi

elif [ "$TARGET_OS" == "android" ]; then
    OPTIONS+=(--target-os=android --enable-cross-compile)

    NDK_PATH="${ANDROID_NDK_LATEST_HOME:-$ANDROID_NDK_HOME}"
    if [ -z "$NDK_PATH" ]; then
        echo "错误: 找不到环境变量 ANDROID_NDK_HOME"
        exit 1
    fi

    TOOLCHAIN="$NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/bin"
    API=26

    if [ "$TARGET_ARCH" == "arm" ] || [ "$TARGET_ARCH" == "armeabi-v7a" ]; then
        OPTIONS+=(--arch=arm --cpu=armv7-a)
        OPTIONS+=(--cc="$TOOLCHAIN/armv7a-linux-androideabi$API-clang")
        TARGET_ARCH="armeabi-v7a"
    elif [ "$TARGET_ARCH" == "aarch64" ] || [ "$TARGET_ARCH" == "arm64-v8a" ]; then
        OPTIONS+=(--arch=aarch64)
        OPTIONS+=(--cc="$TOOLCHAIN/aarch64-linux-android$API-clang")
        TARGET_ARCH="arm64-v8a"
    elif [ "$TARGET_ARCH" == "x86" ]; then
        OPTIONS+=(--arch=x86 --cpu=i686)
        OPTIONS+=(--cc="$TOOLCHAIN/i686-linux-android$API-clang")
        TARGET_ARCH="x86"
    elif [ "$TARGET_ARCH" == "x86_64" ]; then
        OPTIONS+=(--arch=x86_64)
        OPTIONS+=(--cc="$TOOLCHAIN/x86_64-linux-android$API-clang")
        TARGET_ARCH="x86_64"
    else
        echo "不支持的 Android 架构: $TARGET_ARCH"
        exit 1
    fi

elif [ "$TARGET_OS" == "macos" ]; then
    OPTIONS+=(--target-os=darwin)
    if [ "$TARGET_ARCH" == "x86_64" ]; then
        OPTIONS+=(--arch=x86_64 --enable-cross-compile)
        OPTIONS+=(--extra-cflags="-arch x86_64" --extra-ldflags="-arch x86_64")
        TARGET_ARCH="x86_64"
    elif [ "$TARGET_ARCH" == "arm64" ] || [ "$TARGET_ARCH" == "aarch64" ]; then
        OPTIONS+=(--arch=aarch64)
        TARGET_ARCH="arm64"
    else
        echo "不支持的 macOS 架构: $TARGET_ARCH"
        exit 1
    fi

elif [ "$TARGET_OS" == "ios" ]; then
    OPTIONS+=(--target-os=darwin --enable-cross-compile)
    if [ "$TARGET_ARCH" == "arm64" ] || [ "$TARGET_ARCH" == "aarch64" ]; then
        OPTIONS+=(--arch=aarch64 --cc=clang)

        SYSROOT=$(xcrun --sdk iphoneos --show-sdk-path)
        OPTIONS+=(--extra-cflags="-isysroot $SYSROOT")
        OPTIONS+=(--extra-cflags="-target aarch64-apple-ios12.0")
        OPTIONS+=(--extra-cflags="-miphoneos-version-min=12.0")
        OPTIONS+=(--extra-ldflags="-isysroot $SYSROOT")
        OPTIONS+=(--extra-ldflags="-target aarch64-apple-ios12.0")

        TARGET_ARCH="arm64"
    else
        echo "不支持的 iOS 架构: $TARGET_ARCH"
        exit 1
    fi
else
    echo "不支持的 OS: $TARGET_OS"
    exit 1
fi

BUILD_DIR="build_out_${TARGET_OS}_${TARGET_ARCH}"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

echo "运行 Configure 命令:"
echo "../configure ${OPTIONS[*]}"

../configure "${OPTIONS[@]}"

if [ $? -eq 0 ]; then
    echo "Configure 成功，生成编译日志..."
    make V=1 -n > make_dryrun.log
    echo "配置已输出至 : ${BUILD_DIR}/config.h"
    echo "编译日志输出至: ${BUILD_DIR}/make_dryrun.log"
else
    echo "Configure 失败！请检查 ffbuild/config.log 报错"

    if [ -f "ffbuild/config.log" ]; then
        echo "-------------------------------------------------"
        tail -n 500 ffbuild/config.log
        echo "-------------------------------------------------"
    else
        echo "未找到 ffbuild/config.log，无法打印错误信息"
    fi

    exit 1
fi
