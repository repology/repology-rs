#!/bin/sh

echo -n Success | gzip -9 > data.gz
echo -n Success | xz -9 > data.xz
echo -n Success | bzip2 -9 > data.bz2
echo -n Success | zstd -19 > data.zst
