#!/bin/bash

rustc import.rs -o import && \
./import && \
cat mons_enum.part mons_impl.part mons_funcs.part > src/mons.rs && \
echo "Import successful."
