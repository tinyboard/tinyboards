#!/bin/sh

# This script uses a docker file that builds with musl, and runs on linux alpine
# Its a bit slower for development than the volume mount.

set -e

sudo docker-compose down
sudo docker-compose build --no-cache
sudo docker-compose up

#sudo docker build ../ --file Dockerfile -t tinyboards-dev:latest
#sudo docker build ../../tinyboards-fe --file Dockerfile -t tinyboards-fe:latest
#sudo docker build /home/bunchies/gits/tinyboards/tinyboards-fe --file Dockerfile -t tinyboards-fe:latest
#sudo docker-compose pull --ignore-pull-failures || true
#sudo docker-compose up -d
