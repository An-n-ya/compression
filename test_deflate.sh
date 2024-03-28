#!/bin/bash

cargo test test_deflate_no_compression

python gzstat.py < no_compression.gz