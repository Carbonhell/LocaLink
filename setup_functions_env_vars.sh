#!/bin/bash

# Variable block
resourceGroup="localink-rg"
account="localink-account-cosmos"
searchName='localink-search'

unset -v openaiKey
unset -v googleClientId

usage() { echo "Usage: $0 [-o openaiKey] [-g googleClientId]" 1>&2; exit 1; }

while getopts o:g: opt; do
    case $opt in
        o) openaiKey=$OPTARG ;;
        g) googleClientId=$OPTARG ;;
        *)
            usage
            ;;
    esac
done
shift $((OPTIND-1))

if [ -z "$openaiKey" ]; then
        echo 'Missing OpenAI key (-o)' >&2
        exit 1
fi
if [ -z "$googleClientId" ]; then
        echo 'Missing Google Client ID (-g)' >&2
        exit 1
fi

echo "Fetching required keys from Azure..."
cosmosKey=$(az cosmosdb keys list -g $resourceGroup -n $account --query primaryMasterKey -o tsv)
cosmosKey="${cosmosKey%$'\r'}"

adminKey=$(az search admin-key show -g $resourceGroup --service-name $searchName --query primaryKey --out tsv)
adminKey="${adminKey%$'\r'}"

echo "Configuring local settings for local Azure function execution..."
cp local.settings.template.json local.settings.json

sed -i -e "s/\${{COSMOS_PRIMARY_KEY}}/$cosmosKey/g" local.settings.json
sed -i -e "s/\${{SEARCH_ADMIN_KEY}}/$adminKey/g" local.settings.json
sed -i -e "s/\${{OPENAI_API_KEY}}/$openaiKey/g" local.settings.json
sed -i -e "s/\${{GOOGLE_CLIENT_ID}}/$googleClientId/g" local.settings.json

echo "Copying local settings to each Azure function source directory..."
cp local.settings.json Auth/local.settings.json
cp local.settings.json GenerateEmbeddings/local.settings.json
cp local.settings.json Match/local.settings.json
cp local.settings.json Query/local.settings.json
cp local.settings.json SyncPosition/local.settings.json
cp local.settings.json Meet/local.settings.json
