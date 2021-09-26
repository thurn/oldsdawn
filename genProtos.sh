#!/bin/sh
set -e

cd proto
rm -f *.cs
rm -r -f ./bin
rm -r -f ./obj
dotnet clean
dotnet build
mv *.cs ../Assets/Spelldawn/Protos
