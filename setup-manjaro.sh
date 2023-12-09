#!/bin/bash

set -e

# allow my user to access the shared folder
usermod -aG vboxsf duck

# system upgrade
pacman -Syu --noconfirm

# virtual box
pacman -S --needed --noconfirm virtualbox-guest-utils

# rust
pacman -S --needed --noconfirm rustup
rustup toolchain install stable
rustup update

# build tools
pacman -S --needed --noconfirm pkg-config neovim

# ssh access for IDE outside VM
systemctl enable sshd --now