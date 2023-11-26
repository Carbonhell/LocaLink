#!/bin/bash

# Variable block
location="West Europe"
shortLocation="westeurope"
resourceGroup="localink-rg"
tag="localink"
account="localink-account-cosmos" #needs to be lower case
database="main"
user_container="users"
partitionKey="/id"

searchName='localink-search'
searchDataSourceName='localink-datasource'
searchIndexName='localink-search-index'
searchIndexerName='localink-indexer'

storageAccountName="localinkstorage"
functionAuthName="localink-auth"
functionGenerateEmbeddingsName="localink-generate-embeddings"
functionMatchName="localink-match"
functionMeetName="localink-meet"
functionQueryName="localink-query"
functionSyncPositionName="localink-sync-position"


usage() { echo "Usage: $0 [-u] [-f]" 1>&2; exit 1; }

while getopts "uf" o; do
    case "${o}" in
        u)
            echo "Deleting resources..."
            az group delete --name $resourceGroup
            exit 0
            ;;
        f)
          echo "Creating function auth"
          az storage account create --name "$storageAccountName" --location "$location" --resource-group "$resourceGroup" --sku Standard_LRS --allow-blob-public-access false
          az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionAuthName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
          echo "Creating function generateEmbeddings"
          az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionGenerateEmbeddingsName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
          echo "Creating function match"
          az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionMatchName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
          echo "Creating function meet"
          az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionMeetName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
          echo "Creating function query"
          az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionQueryName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
          echo "Creating function syncPosition"
          az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionSyncPositionName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
          exit 0
          ;;
        *)
            usage
            ;;
    esac
done
shift $((OPTIND-1))


echo "Configuring required Azure services..."

# Create a resource group
echo "Creating $resourceGroup in $location..."
az group create --name $resourceGroup --location "$location" --tags $tag


# Cosmos DB
echo "Configuring Cosmos DB resources..."

# Create a Cosmos account for SQL API
echo "Creating Cosmos DB account"
az cosmosdb create --name $account --resource-group $resourceGroup --default-consistency-level Eventual --locations regionName="$location" failoverPriority=0 isZoneRedundant=False --capabilities EnableServerless

# Create a SQL API database
echo "Creating $database database"
az cosmosdb sql database create --account-name $account --resource-group $resourceGroup --name $database

# Create a SQL API user_container
echo "Creating $user_container with $partitionKey"
az cosmosdb sql container create --account-name $account --resource-group $resourceGroup --database-name $database --name $user_container --partition-key-path $partitionKey

# Azure Cognitive Search
echo "Creating search service"
az search service create --name $searchName --resource-group $resourceGroup --sku Free

# Fetch generated primary admin key
echo "Fetching the admin key for subsequent data calls"
adminKey=$(az search admin-key show --resource-group $resourceGroup --service-name $searchName --query primaryKey --out tsv)
adminKey="${adminKey%$'\r'}"

echo "Creating search index"
jsonBody=$(cat <<EOF
{
    "name": "$searchIndexName",
    "fields": [{
        "name": "id",
        "type": "Edm.String",
        "key": true,
        "searchable": false
    }, 
    {
        "name": "description",
        "type": "Edm.String",
        "filterable": false,
        "searchable": true,
        "sortable": false,
        "facetable": false,
        "retrievable": true
    },
    {
        "name": "name",
        "type": "Edm.String",
        "filterable": false,
        "searchable": true,
        "sortable": false,
        "facetable": false,
        "retrievable": true
    },
    {
      "name": "description_embeddings",
      "type": "Collection(Edm.Single)",
      "searchable": true,
      "filterable": false,
      "retrievable": false,
      "sortable": false,
      "facetable": false,
      "key": false,
      "indexAnalyzer": null,
      "searchAnalyzer": null,
      "analyzer": null,
      "synonymMaps": [],
      "dimensions": 1536,
      "vectorSearchProfile": "default-vector-profile"
    },
    {
      "name": "location",
      "type": "Edm.GeographyPoint",
      "searchable": false,
      "filterable": true,
      "sortable": true,
      "facetable": false,
      "retrievable": true
    }
  ],
  "vectorSearch": {
     "algorithms": [
         {
             "name": "hnsw-main",
             "kind": "hnsw",
             "hnswParameters": {
                 "m": 4,
                 "efConstruction": 400,
                 "efSearch": 500,
                 "metric": "cosine"
             }
         }
     ],
     "profiles": [
       {
         "name": "default-vector-profile",
         "algorithm": "hnsw-main"
       }
     ]
 }
}
EOF
)
curl -H "Content-Type: application/json" -H "api-key: $adminKey" --request PUT --data "$jsonBody" "https://$searchName.search.windows.net/indexes('$searchIndexName')?allowIndexDowntime=False&api-version=2023-10-01-Preview"

echo "Creating function auth"
az storage account create --name "$storageAccountName" --location "$location" --resource-group "$resourceGroup" --sku Standard_LRS --allow-blob-public-access false
az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionAuthName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
echo "Creating function generateEmbeddings"
az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionGenerateEmbeddingsName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
echo "Creating function match"
az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionMatchName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
echo "Creating function meet"
az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionMeetName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
echo "Creating function query"
az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionQueryName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights
echo "Creating function syncPosition"
az functionapp create --resource-group "$resourceGroup" --consumption-plan-location "$shortLocation" --functions-version 4 --name "$functionSyncPositionName" --storage-account "$storageAccountName" --os-type linux --runtime custom --disable-app-insights


# Currently disabled - a push strategy seems better for our usecase (otherwise the delay would be minimum 3 minutes), but in future batching might be better
#echo "Creating data source"
#cosmosKey=$(az cosmosdb keys list --name $account --resource-group $resourceGroup --type keys --query primaryMasterKey --out tsv)
#cosmosKey="${cosmosKey%$'\r'}"
# todo filter data
#jsonBody=$(cat <<EOF
#{
#    "name": "$searchDataSourceName",
#    "type": "cosmosdb",
#    "credentials": {
#      "connectionString": "AccountEndpoint=https://$account.documents.azure.com;AccountKey=$cosmosKey;Database=$database"
#    },
#    "user_container": {
#      "name": "$user_container",
#      "query": null
#    },
#    "dataChangeDetectionPolicy": {
#      "@odata.type": "#Microsoft.Azure.Search.HighWaterMarkChangeDetectionPolicy",
#      "highWaterMarkColumnName": "_ts"
#    }
#}
#EOF
#)
#
#curl -H "Content-Type: application/json" -H "api-key: $adminKey" --request POST --data "$jsonBody" https://$searchName.search.windows.net/datasources?api-version=2020-06-30
#
#
#echo "Creating indexer"
#jsonBody=$(cat <<EOF
#{
#    "name" : "$searchIndexerName",
#    "dataSourceName" : "$searchDataSourceName",
#    "targetIndexName" : "$searchIndexName",
#    "disabled": null,
#    "schedule": null,
#    "parameters": {
#        "batchSize": null,
#        "maxFailedItems": 0,
#        "maxFailedItemsPerBatch": 0,
#        "base64EncodeKeys": false,
#        "configuration": {}
#        },
#    "fieldMappings": [],
#    "encryptionKey": null
#}
#EOF
#)
#
#curl -H "Content-Type: application/json" -H "api-key: $adminKey" --request POST --data "$jsonBody" https://$searchName.search.windows.net/indexers?api-version=2020-06-30
#
# todo check indexer status
