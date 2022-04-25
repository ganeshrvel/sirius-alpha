#!/bin/zsh
set -e

source ./scripts/setup_envs.sh

echo "building the debug build.."

cargo build

echo "flashing the debug build.."
echo "hold boot button and release it when the flashing starts..."

sleep 2


espflash /dev/cu.usbserial-0001 target/xtensa-esp32-espidf/debug/sirius-alpha-rust


