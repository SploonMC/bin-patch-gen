#!/bin/bash

# update repo and discard changes
git stash
git checkout release
git branch --set-upstream-to=origin/release release
git pull

# check if docker container exists
if docker container inspect sploon-bin-patch-gen > /dev/null 2>&1; then
  # if it does, check if it's running
  if [ "$(docker container inspect -f '{{.State.Status}}' sploon-bin-patch-gen)" != "running" ]; then
    # restart docker container, this will block until finished
    docker compose down
    docker compose up

    # git shit
    cd work || exit 1
    [[ ! -d ".git" ]] && git init .
    git add .
    git commit -am "automated update"
    git push -u origin master
  fi
else
  # If the container doesn't exist, assume first time setup
  echo "Container does not exist, assuming this is the first time."
  docker compose up

  # git shit for first-time setup
  cd work || exit 1
  [[ ! -d ".git" ]] && git init .
  git remote remove origin
  git remote add origin https://github.com/SploonMC/patches.git
  git add .
  git commit -am "first-time setup"
  git push -u origin master
fi

