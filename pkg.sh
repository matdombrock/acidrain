#!/bin/bash

# NOTE: Requires `wasm-strip` from `wabt` package.
# https://github.com/WebAssembly/wabt/releases

set -e
title="ACIDRAIN"

cargo build --release
wasm-strip target/wasm32-unknown-unknown/release/cart.wasm
cd target/wasm32-unknown-unknown/release
w4 bundle cart.wasm --title "$title" --html index.html \
  --windows $title-windows.exe \
  --mac $title-macos \
  --linux $title-linux 
cd -

out_dir=dist
mkdir -p $out_dir
rm -rf $out_dir/*
out_dir_web="$out_dir/$title-web"
mkdir -p $out_dir_web

target=target/wasm32-unknown-unknown/release

cp $target/cart.wasm $out_dir_web
cp $target/index.html $out_dir_web 
zip -j "$out_dir_web.zip" $out_dir_web/*

cp $target/"$title"-windows.exe $out_dir
cp $target/"$title"-macos $out_dir
cp $target/"$title"-linux $out_dir 

echo "Packaged builds are in the '$out_dir' directory."
