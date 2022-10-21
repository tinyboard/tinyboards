#!/bin/sh

# This script uses a docker file that builds with musl, and runs on linux alpine
# Its a bit slower for development than the volume mount.

set -e

sudo docker-compose down
sudo docker build ../ --file Dockerfile -t porpl-dev:latest
sudo docker build ../../porpl-fe --file Dockerfile -t porpl-fe:latest
#sudo docker build /home/bunchies/gits/porpl/porpl-fe --file Dockerfile -t porpl-fe:latest
sudo docker-compose pull --ignore-pull-failures || true
#sudo docker-compose up -d
sudo docker-compose up
