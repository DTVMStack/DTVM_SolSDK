#!/usr/bin/env bash
set -e

if [ "$1" = "prepare" ]; then
  echo "done prepare docker build context"
  ls
else
  cd docker
  docker build . --platform linux/x86_64 -f Dockerfile -t dtvmdev1/dtvm-sol-dev-x64:latest
  cd ..
fi
