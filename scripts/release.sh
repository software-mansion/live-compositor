#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )"/.. && pwd )"

set +u
if [[ -z "$WORKFLOW_RUN_ID" ]]; then
  echo "WORKFLOW_RUN_ID env variable is required. You can list recent runs using gh run list --workflow \"package for release\" command."
  echo ""
  echo "Recent workflow runs:"
  gh run list --workflow "package for release" | cat
  exit 1
fi

if [[ -z "$RELEASE_TAG" ]]; then
  echo "RELEASE_TAG env variable is required."
  exit 1
fi
set -u

mkdir -p "$ROOT_DIR/release_tmp"
cd "$ROOT_DIR/release_tmp"

gh run download "$WORKFLOW_RUN_ID" -n live_compositor_linux_x86_64.tar.gz
gh run download "$WORKFLOW_RUN_ID" -n live_compositor_linux_aarch64.tar.gz
gh run download "$WORKFLOW_RUN_ID" -n live_compositor_darwin_x86_64.tar.gz
gh run download "$WORKFLOW_RUN_ID" -n live_compositor_darwin_aarch64.tar.gz
gh run download "$WORKFLOW_RUN_ID" -n live_compositor_with_web_renderer_linux_x86_64.tar.gz
gh run download "$WORKFLOW_RUN_ID" -n live_compositor_with_web_renderer_darwin_x86_64.tar.gz
gh run download "$WORKFLOW_RUN_ID" -n live_compositor_with_web_renderer_darwin_aarch64.tar.gz

gh release create "$RELEASE_TAG"
gh release upload "$RELEASE_TAG" live_compositor_linux_x86_64.tar.gz
gh release upload "$RELEASE_TAG" live_compositor_linux_aarch64.tar.gz
gh release upload "$RELEASE_TAG" live_compositor_darwin_x86_64.tar.gz
gh release upload "$RELEASE_TAG" live_compositor_darwin_aarch64.tar.gz
gh release upload "$RELEASE_TAG" live_compositor_with_web_renderer_linux_x86_64.tar.gz
gh release upload "$RELEASE_TAG" live_compositor_with_web_renderer_darwin_x86_64.tar.gz
gh release upload "$RELEASE_TAG" live_compositor_with_web_renderer_darwin_aarch64.tar.gz

rm -rf "$ROOT_DIR/release_tmp"
