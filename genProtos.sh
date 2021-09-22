#!/bin/sh
set -e

cd proto
dotnet clean
dotnet build
mv *.cs ../Assets/Spelldawn/Protos
