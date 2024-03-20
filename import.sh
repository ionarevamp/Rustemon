#!/bin/bash

rustc import.rs -o import && \
./import && \
cat mons_enum.txt mons_impl.txt > src/mons.rs
