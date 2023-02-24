#!/bin/bash

key='D'
script_binding='script-binding "libmpv_rpc/toggle-rpc"'
mpv_home="${MPV_HOME:-${XDG_CONFIG_HOME:-${HOME}/.config}/mpv}"
scripts_dir="$mpv_home/scripts"

if [ -d $scripts_dir ]; then
    mkdir -p "$scripts_dir"
fi

echo -n "Copying script..."
cp ./bin/libmpv_rpc.so "$scripts_dir"
echo "Done!"

if [ ! -f "$mpv_home/rpc.json" ]; then
    echo -n "Copying default config..."
    cp ./config/rpc.json "$mpv_home"
    echo "Done!"
fi

if ! grep -q "$script_binding" "$mpv_home/input.conf"; then
    echo -n "Adding keybinding entry to input.conf..."
    echo "$key $script_binding" >> "$mpv_home/input.conf"
    echo "Done!"
fi