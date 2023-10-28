import sys
import requests

parser = argparse.ArgumentParser(description='Setup the required Azure services for LocaLink to work.')
parser.add_argument('--search-service-name', help='The service name to use. You must create the search resource on the Azure dashboard, more info here: https://learn.microsoft.com/en-us/azure/search/search-create-service-portal')
parser.add_argument('--cosmosdb-account-name', help='The account name related to the already existing Cosmos DB NoSQL instance.')
parser.add_argument('--cosmosdb-account-key', help='The account key related to the already existing Cosmos DB NoSQL instance.')

args = parser.parse_args()

azsearch_key = args["search-key"]
azsearch_service_name = args["search-service-name"]
cosmosdb_account_name = args["cosmosdb-account-name"]
cosmosdb_account_key = args["cosmosdb-account-key"]

# shared between all requests
headers = {'api-key': azsearch_key}

# Set Cosmos DB as a data source
data_source_payload = {
	"name": "cosmosdb-ds",
    "type": "cosmosdb",
    "credentials": {
      "connectionString": f"AccountEndpoint=https://{cosmosdb_account_name}.documents.azure.com;AccountKey={cosmosdb_account_key};Database=maindb"
    },
    "container": {
      "name": "users",
      "query": null # todo figure out if we need this to project the correct schema (we can probs ignore this)
    },
    "dataChangeDetectionPolicy": {
      "@odata.type": "#Microsoft.Azure.Search.HighWaterMarkChangeDetectionPolicy",
      "highWaterMarkColumnName": "_ts"
    },
    "dataDeletionDetectionPolicy": null,
    "encryptionKey": null,
    "identity": null
}
r = requests.post(f'https://{azsearch_service_name}.search.windows.net/datasources?api-version=2020-06-30', json=data_source_payload)

# Configure the search index
search_index_payload = {
	"name": "main-search-index",
    "fields": [{
        "name": "rid",
        "type": "Edm.String",
        "key": true,
        "searchable": false
    }, 
    { # todo add vector stuff and last position
        "name": "description",
        "type": "Edm.String",
        "filterable": False,
        "searchable": True,
        "sortable": False,
        "facetable": False,
        "suggestions": False
    }
  ]
}
r = requests.post(f'https://{azsearch_service_name}.search.windows.net/indexes?api-version=2020-06-30', json=search_index_payload)

# Link search index and data source
create_indexer_payload = {
	"name" : "main_indexer",
    "dataSourceName" : "cosmosdb-ds",
    "targetIndexName" : "main-search-index",
    "disabled": null,
    "schedule": null,
    "parameters": {
        "batchSize": null,
        "maxFailedItems": 0,
        "maxFailedItemsPerBatch": 0,
        "base64EncodeKeys": false,
        "configuration": {}
        },
    "fieldMappings": [],
    "encryptionKey": null
}
r = requests.post(f'https://{azsearch_service_name}.search.windows.net/indexes?api-version=2020-06-30', json=create_indexer_payload)

# todo monitor indexer