#!/usr/bin/env bash
set -euo pipefail

cd tools/vscode-arco-kdl
npm run check
npm run package
