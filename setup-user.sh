#!/bin/bash

set -e

for key in $(xfconf-query -c xfce4-desktop -l); do
  if [[ "$key" =~ /last-image$ ]]; then
    xfconf-query -c xfce4-desktop -p $key -s /mnt/duck/duck.png
  fi
done