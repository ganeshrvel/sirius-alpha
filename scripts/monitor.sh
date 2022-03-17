#!/bin/zsh
set -e

espmonitor /dev/cu.usbserial-0001 --bin target/xtensa-esp32-espidf/debug/sirius-alpha-rust
