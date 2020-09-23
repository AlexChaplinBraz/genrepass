#!/usr/bin/env sh

# Build all release targets and package them for publishing.

ProjectName=$(grep -m1 'name' Cargo.toml | grep -o '"[^"]\+"' | sed 's/"//g')
Version=$(grep -m1 'version' Cargo.toml | grep -o '"[^"]\+"' | sed 's/"//g')

mkdir releases 2>/dev/null
printf 'Building all releases...\n\n'

# $1 = zip/tar, $2 = $Target, $3 = $Version, $4 = $ProjectName, $5 = $FileName
build_target() {
    printf 'Building for %s...\n' $2
    if cargo build --release --target $2; then
        printf 'Build finished correctly, packing...\n'
        if [ "$1" = 'zip' ]; then
            zip -j releases/$4-$2-$3.zip target/$2/release/$5 LICENSE README.md
        elif [ "$1" = 'tar' ]; then
            \cp LICENSE README.md target/$2/release
            tar -zcf releases/$4-$2-$3.tar.gz -C target/$2/release $5 LICENSE README.md
        fi
        printf 'Release %s-%s-%s packed.\n\n' $4 $2 $3
    else
        printf 'ERROR: failed to build release for %s.\n' $2
        exit 1
    fi
}

build_target tar x86_64-unknown-linux-gnu $Version $ProjectName $ProjectName
build_target tar i686-unknown-linux-gnu $Version $ProjectName $ProjectName
build_target zip x86_64-pc-windows-gnu $Version $ProjectName ${ProjectName}.exe
build_target zip i686-pc-windows-gnu $Version $ProjectName ${ProjectName}.exe

printf 'All releases packed correctly.\n'
