#!/bin/sh

find . -name '*conflicted*' -delete

mkdir -p Library
mkdir -p Logs
mkdir -p obj
mkdir -p UserSettings
mkdir -p Temp
mkdir -p out
mkdir -p out_BurstDebugInformation_DoNotShip/

xattr -w com.dropbox.ignored 1 Library/
xattr -w com.dropbox.ignored 1 Logs/
xattr -w com.dropbox.ignored 1 obj/
xattr -w com.dropbox.ignored 1 UserSettings/
xattr -w com.dropbox.ignored 1 Temp/
xattr -w com.dropbox.ignored 1 out/
xattr -w com.dropbox.ignored 1 out_BurstDebugInformation_DoNotShip/
xattr -w com.dropbox.ignored 1 proto/bin
xattr -w com.dropbox.ignored 1 proto/obj
xattr -w com.dropbox.ignored 1 target/
xattr -w com.dropbox.ignored 1 bin/

rm -r 'Temp (Ignored Item Conflict 1)'
rm -r 'Temp (Ignored Item Conflict)'
rm -r 'out (Ignored Item Conflict)'
rm -r proto/obj
rm -r proto/bin
rm proto/*.cs
