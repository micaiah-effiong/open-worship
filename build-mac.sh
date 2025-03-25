#!/bin/bash

BUILD_DIR="build/macos"
CURRENT_DIR=$(pwd)

# build bundle
# ensure cargo-bundle is install
cargo bundle --release --format osx

# clean build dir
rm -r $BUILD_DIR
mkdir -p $BUILD_DIR
cp -r target/release/bundle/osx/Openworship.app $BUILD_DIR
cd $BUILD_DIR

# link dylib
dylibbundler -od -b -x Openworship.app/Contents/MacOS/open-worship -d Openworship.app/Contents/Resources/libs -p @executable_path/../Resources/libs

# log 
otool -L  Openworship.app/Contents/MacOS/open-worship

# reset dir
cd $CURRENT_DIR
