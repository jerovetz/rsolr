#!/bin/bash
docker exec rsc_solr sh -c 'solr delete -c default; solr create -c default;solr delete -c techproducts; solr create -c techproducts -n techproducts'