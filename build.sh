#!/usr/bin/env bash

cargo build --release
cargo build --release --target x86_64-pc-windows-gnu

rm dd2-height-alarm-win.zip || true
rm dd2-height-alarm-linux.zip || true

7z a dd2-height-alarm-win.zip target/x86_64-pc-windows-gnu/release/dd2-height-alarm.exe
7z a dd2-height-alarm-linux.zip target/release/dd2-height-alarm
