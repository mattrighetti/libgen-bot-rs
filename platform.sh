#!/bin/sh

# Used in Docker build to set platform dependent variables

case $TARGETARCH in
    "amd64")
	echo "x86_64-unknown-linux-musl" > /.platform
	;;
    "arm64")
	echo "aarch64-unknown-linux-musl" > /.platform
	;;
esac