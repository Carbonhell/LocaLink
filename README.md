# Idea
LocaLink helps the user in building a friendship network. For example, someone who's just transferring to a new city, joining a new social circle (university, work...) or simply has a hard time meeting new people for any reason could use LocaLink as an enabler to find people who match their interest. The application works by allowing the user to input any sort of media characterising his own person (initially just text, but the system is supposed to work with image, audio and video as well). The system uses vectorization to normalize the data and allow finding similarities. The users are then matched firstly based on their position (the objective, after all, is to meet each other in real life!) and subsequently based on the similarity between the inputted data.

# How to build
The project is composed of six Azure Functions and a Flutter app.

## Requirements
- [Azure CLI](https://learn.microsoft.com/en-us/cli/azure/install-azure-cli) to provision the infrastructure
- (optional) [Azure Functions Core Tools](https://learn.microsoft.com/en-us/azure/azure-functions/functions-run-local) for local execution of Azure functions
- Zip (sudo apt install zip) for Azure Functions Zip deployment
- [Rust](https://www.rust-lang.org/) (rustc 1.73.0 was used, but it is not a hard dependency) to build the functions' source code
- [Flutter](https://docs.flutter.dev/get-started/install) for the mobile app


## Setting up the Azure infrastructure
The Azure services used for this project are Cosmos DB (NoSQL), Cognitive Search (vector search) and obviously Azure Functions.
A helper script is provided to quickly set up the infrastructure with the use of the [Azure CLI](https://learn.microsoft.com/en-us/cli/azure/install-azure-cli).
You can provision all the required services by running the following command:
```sh
./setup_azure.sh
```
All services will be provisioned under the `localink-rg` resource group. You can quickly delete it by running the same command with the `-u` flag.

Once done, you can set up the required environments of all the Azure Functions with the `setup_functions_env_vars.sh` script, which uses the `local.settings.template.json` template by filling it with the required Azure keys obtained through the Azure CLI.
```sh
./setup_functions_env_vars.sh -o OPENAI_KEY -g GOOGLE_ID
```
You will need to pass the external service keys yourself (OpenAI for the Ada model, for embeddings, and the Google ID for Google Auth)

## Deploying the functions on Azure
You'll need pkg-config, openssl, libssl-dev, and musl-tools to build the function binaries.
Be sure the local properties of each function is up to date (you can use the `setup_functions_env_vars.sh` script to refresh it).

Then, for each function, run the following command in the function root dir:
```sh
cargo build --release --target=x86_64-unknown-linux-musl
```

Copy the newly generated binary to the function root dir by replacing {FUNCTION_NAME} with the crate name (e.g. .../auth and auth_handler):
```sh
cp target/x86_64-unknown-linux-musl/release/{FUNCTION_NAME} {FUNCTION_NAME}_handler
```

Finally, perform the deploy (by replacing {FUNCTION_NAME} as stated previously):
```sh
func azure functionapp publish localink-{FUNCTION_NAME} --no-build --publish-local-settings
```

## (optional) Running the functions locally

If you want to debug functions locally, you simply need to build each with the following command:
```sh
cargo build --release
```

Remember to change the function's `host.json` file to point at the correct generated executable.

Once done, you can run a function with the following command.
```sh
func start --port PORT
```
Be sure to change the port to an available one, considering you will need six different ports, one per function.

If you plan to run the Flutter app on a physical device, the easiest way for the device to be able to access the functions is to use a tunnel service like [ngrok](https://ngrok.com) with a configuration file to be able to serve multiple services (up to 3 on the free version) with a single tunnel.

## Running the Flutter app
You'll need to set up an .env file in the root folder of the flutter project. You can use the example env as a base and simply fill all the fields. This is where you should put the urls of your generated Azure Functions.

You'll also need to create a local.properties file under the android folder. IDEs or Flutter itself will probably create it for you, but you will need to insert the `flutter.mapsAPIKey` key with the value of your Google Maps key.

As of 2023/11/22, only the Android version of the app is tested and maintained, but supporting other platforms should be easy.

Once done, you can simply run the Flutter app using the main.dart file as entrypoint.
