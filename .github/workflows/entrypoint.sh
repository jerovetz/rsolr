#!/bin/sh
docker_run="docker run -d -p 8983:8983 solr:8"
sh -c "$docker_run"