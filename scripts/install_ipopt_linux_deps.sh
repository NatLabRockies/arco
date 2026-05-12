#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends liblapack-dev libopenblas-dev gfortran
