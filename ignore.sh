#!/bin/sh

mkdir -p Library
mkdir -p Logs
mkdir -p obj
mkdir -p UserSettings
mkdir -p Temp
mkdir -p out

xattr -w com.dropbox.ignored 1 Library/
xattr -w com.dropbox.ignored 1 Logs/
xattr -w com.dropbox.ignored 1 obj/
xattr -w com.dropbox.ignored 1 UserSettings/
xattr -w com.dropbox.ignored 1 Temp/
xattr -w com.dropbox.ignored 1 out/
xattr -w com.dropbox.ignored 1 out_BurstDebugInformation_DoNotShip/
