#!/usr/bin/env bash
if ! command -v git > /dev/null ; then
    sudo apt update && sudo apt install git
fi

git clone https://github.com/tpoechtrager/osxcross
cd osxcross
sudo tools/get_dependencies.sh
wget -nc https://s3.dockerproject.org/darwin/v2/MacOSX10.10.sdk.tar.xz
mv MacOSX10.10.sdk.tar.xz tarballs/
UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
cd ../

export PATH="$(pwd)/osxcross/target/bin:$PATH" 
export LIBZ_SYS_STATIC=1
export CC=o64-clang
export CXX=o64-clang++

cargo build --release --target x86_64-apple-darwin