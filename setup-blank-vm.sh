#!/bin/bash

# This script sets up a blank Ubuntu 22.04 VM for Chromium debugging.

wget "https://github.com/google/fetchchromium/releases/download/v0.4.0/fetchchromium_0.4.0-1_amd64.deb"
sudo dpkg -i fetchchromium_0.4.0-1_amd64.deb

# Hard coded because of a bug in some versions
wget "https://deb.rug.nl/ppa/mirror/dl.google.com/linux/chrome-remote-desktop/deb/pool/main/c/chrome-remote-desktop/chrome-remote-desktop_125.0.6422.20_amd64.deb"
sudo dpkg -i chrome-remote-desktop_125.0.6422.20_amd64.deb

sudo DEBIAN_FRONTEND=noninteractive apt --fix-broken --assume-yes install
sudo DEBIAN_FRONTEND=noninteractive apt-get install --assume-yes xfce4 llvm desktop-base dbus-x11 xscreensaver
sudo bash -c 'echo "exec /etc/X11/Xsession /usr/bin/xfce4-session" > /etc/chrome-remote-desktop-session'

echo "Restarting the VM - please SSH in again, then run fetchchromium with whatever options you like (possibly none)"
echo "and run the command given at https://remotedesktop.google.com/headless to set up CRD"
sudo shutdown -r now
