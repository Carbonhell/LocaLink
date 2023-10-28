#!/bin/bash
search_service_name=''

echo "Configuring required Azure services..."

echo "Defining the datasource..."

curl -X POST -H 'Content-Type: application/json' -d '{"message": "hello"}' https://[service name].search.windows.net/datasources?api-version=2020-06-30
