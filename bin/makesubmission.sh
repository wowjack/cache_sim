#! /usr/bin/env sh

PROJECT_NAME=project2

if [[ ! -f PREFETCHER.md ]]; then
    echo "This script MUST be run from the same directory as README.md" 1>&2
    exit 1
fi
printf "Enter your MultiPass username: "
read username
filename="${username}-${PROJECT_NAME}.tar.gz"
tmpdir=$(mktemp -d)
tar jcf ${tmpdir}/${filename} Makefile src PREFETCHER.md Cargo.toml
mv ${tmpdir}/${filename} .
echo "${filename} created with the following files:"
tar tfv ${filename}