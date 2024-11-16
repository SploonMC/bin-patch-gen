#!/bin/bash

# update repo and discard changes
git stash
git checkout release
git branch --set-upstream-to=origin/release release
git pull

if [[ ! -d "work/.git" ]]; then
  mkdir -p work
  cd work
  chown -R radsteve .
  git init .
  git remote add origin https://github.com/SploonMC/patches.git
  git pull
fi

# check if docker container exists
if docker container inspect sploon-bin-patch-gen > /dev/null 2>&1; then
  # if it does, check if it's running
  if [ "$(docker container inspect -f '{{.State.Status}}' sploon-bin-patch-gen)" != "running" ]; then
    docker compose down
    docker compose up

    cd work || exit 1
    git add .
    git commit -am "automated update"
    git push -u origin master
  fi
else
  docker compose up

  cd work || exit 1
  git add .
  git commit -am "first-time setup"
  git push -u origin master
fi
