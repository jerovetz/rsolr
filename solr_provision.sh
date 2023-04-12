#!/bin/bash
docker exec -it rsc_solr sh -c 'solr delete -c default; solr create -c default'
curl --location --request POST 'http://localhost:8983/api/cores/default/config' \
--header 'Content-Type: application/json' \
--data-raw '{
    "set-property": [{
            "requestDispatcher.requestParsers.enableRemoteStreaming": true
        },
        {
            "requestDispatcher.requestParsers.enableStreamBody": true
        }
    ]
}'