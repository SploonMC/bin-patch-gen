#!/bin/bash

cd /srv/sploon-bpg

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
  cd ..
fi

cd work
git pull origin master
cd ..

# check if docker container exists
if docker container inspect sploon-bin-patch-gen > /dev/null 2>&1; then
  # if it does, check if it's running
  if [ "$(docker container inspect -f '{{.State.Status}}' sploon-bin-patch-gen)" != "running" ]; then
    docker compose down
    docker compose up gen

    cd work || exit 1
    git add .
    git commit -am "automated update"
    git push -u origin master
  fi
else
  docker compose up gen

  cd work || exit 1
  git add .
  git commit -am "first-time setup"
  git push -u origin master
fi

