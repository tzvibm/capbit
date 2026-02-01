#!/bin/bash
export TMPDIR=$HOME/tmp
cd /data/data/com.termux/files/home/capbit
cargo build --release --features server
