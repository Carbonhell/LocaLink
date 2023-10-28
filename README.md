The project is still under development and not testable yet.

# Idea
LocaLink helps the user in building a friendship network. For example, someone who's just transferring to a new city, joining a new social circle (university, work...) or simply has a hard time meeting new people for any reason could use LocaLink as an enabler to find people who match their interest. The application works by allowing the user to input any sort of media characterising his own person (initially just text, but the system is supposed to work with image, audio and video as well). The system uses vectorization to normalize the data and allow finding similarities. The users are then matched firstly based on their position (the objective, after all, is to meet each other in real life!) and subsequently based on the similarity between the inputted data.

---

https://medium.com/geekculture/easyauth-in-functions-app-with-azure-active-directory-29c01cad8477
https://adrianhall.github.io/develop-mobile-apps-with-csharp-and-azure/chapter2/social/#google-configuration
https://adrianhall.github.io/develop-mobile-apps-with-csharp-and-azure/chapter2/backend/
https://learn.microsoft.com/en-us/azure/developer/mobile-apps/authentication
https://learn.microsoft.com/en-us/azure/active-directory-b2c/identity-provider-google?pivots=b2c-user-flow
 
B2C: overkill per un semplice sistema di auth. Vendor lock-in esagerato. https://learn.microsoft.com/en-us/azure/active-directory-b2c/add-sign-up-and-sign-in-policy?pivots=b2c-user-flow https://www.reddit.com/r/dotnet/comments/12glq73/comment/jfneyda/?utm_source=reddit&utm_medium=web2x&context=3
https://blog.bredvid.no/patterns-for-securing-your-azure-functions-2fef634f4020



# Functions
Rust: https://learn.microsoft.com/en-us/azure/azure-functions/create-first-function-vs-code-other?tabs=rust%2Cwindows
https://github.com/caojen/google-oauth

# Storage
https://docs.rs/azure_data_cosmos/latest/azure_data_cosmos/
https://learn.microsoft.com/it-it/azure/cosmos-db/how-to-develop-emulator?tabs=docker-windows%2Ccsharp&pivots=api-nosql

# Search
https://devblogs.microsoft.com/azure-sql/vector-similarity-search-with-azure-sql-database-and-openai/
https://stackoverflow.com/questions/40101159/can-azure-cognitive-search-be-used-as-a-primary-database-for-some-data
https://learn.microsoft.com/en-us/training/modules/search-azure-cosmos-db-sql-api-data-azure-cognitive-search/