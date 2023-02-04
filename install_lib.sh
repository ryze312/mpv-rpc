#!/bin/bash
TARGET=$1

if [[ -z "$TARGET" ]] 
then
    TARGET="debug"
fi

echo "Copying library to mpv scripts directory..."
cp "target/$TARGET/libmpv_rpc.so" "$HOME/.config/mpv/scripts/"