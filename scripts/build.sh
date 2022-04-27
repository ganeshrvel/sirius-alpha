#!/bin/zsh
set -e
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

echo "\n\nMinimum nodejs >=16"
echo "Make sure to check the nodejs version first"
echo "run: nvm use 16 if you haven't yet\n\n"

source ./setup_envs.sh
./build-setup.mjs
