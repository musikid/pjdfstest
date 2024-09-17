#!/bin/sh

# This script is used to build the book and merge the crate documentation
# into the book.

set -e

# Build the book
mdbook build

# Merge the crate documentation into the book
(cd ../rust && cargo doc --no-deps && mkdir ../build/doc && mv target/doc/ ../build/)