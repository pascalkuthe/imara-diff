#!/bin/bash

set -e
num_revs=20

for repo_file in corpus/*.info; do
  repo_url=$(cat "${repo_file}")
  git clone "${repo_url}"
  repo_name=$(basename -- "$repo_file")
  repo_name="${repo_file_name%.*}"
  cd "${repo_name}"
  git revl
  git diff --n $num_revs
  cd ..
done
