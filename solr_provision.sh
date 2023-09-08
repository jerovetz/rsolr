#!/bin/bash
docker exec rsc_solr sh -c 'solr delete -c default; solr create -c default'