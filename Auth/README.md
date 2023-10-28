# Requirements

You can follow this [Azure guide](https://learn.microsoft.com/en-us/azure/azure-functions/create-first-function-vs-code-other?tabs=rust%2Cwindows) to configure your local dev environment. In particular, the Azure function core tools will allow you to run the Azure function locally for testing purposes.
TODO env stuff to set

You can start an emulated Cosmos DB environment as specified [here]() with the following docker command:
```sh
docker run --publish 8081:8081 --publish 10250-10255:10250-10255 --interactive --tty mcr.microsoft.com/cosmosdb/linux/azure-cosmos-emulator:latest
```

# Building

Build the function in release mode:

```sh
cargo build --release
```

And start the function

```sh
func start
```

# Testing

You can fetch a token ID from the [Google Oauth Playground](https://developers.google.com/oauthplayground/). The scopes required are:
- `https://www.googleapis.com/auth/userinfo.email`
- `openid`
and you can find them under the **Google OAuth2 API v2** group. Verify that in the playground settings you're using a server-side OAuth flow, with your own OAuth credentials (client ID and secret, or else the AUD check will fail).
Once you've exchanged your authorization code for tokens, in the Google response you should see the id_token required to test this function.

