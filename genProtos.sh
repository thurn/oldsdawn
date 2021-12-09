#!/bin/sh
set -e

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd $SCRIPT_DIR

cargo run --bin gen_protos
rm crates/protos/src/google.protobuf.rs

cd proto
rm -f *.cs
rm -r -f ./bin
rm -r -f ./obj
dotnet clean
dotnet build
mv *.cs ../Assets/Spelldawn/Protos
dotnet clean
rm -r bin/
rm -r obj/
