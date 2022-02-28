#!/bin/zsh
set -e

echo "building the release build.."

cargo build --release

echo "flashing the release build.."
echo "hold boot button and release it when the flashing starts..."

sleep 2


espflash /dev/cu.usbserial-0001 target/xtensa-esp32-espidf/release/sirius-alpha-rust
