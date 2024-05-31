#!/bin/bash

# Upgrade dependencies but exlude the once that are patched by risc0
#
# More info: https://github.com/eigerco/lumina/pull/171
exec cargo upgrade \
    --exclude sha2 \
    --exclude crypto-bigint \
    --exclude curve25519-dalek \
    --exclude ed25519-dalek \
    "$@"
