# LLM-made components

This repo contains examples of one-shot WAVS components made using Claude and Cursor rulefiles. 

Prompts: 
```
let's make a component that takes the input of a zip code, queries the openbrewerydb, and returns the breweries in the area. View the docs for the api: https://github.com/openbrewerydb/openbrewerydb-gatsby/blob/e2aa72e3fe32455f54cb22bb30f15c90b18f9a3f/content/documentation/03-search.md?plain=1#L27
```
```
I want to build a new component that takes the input of a wallet address, queries the usdt contract, and returns the balance of that address.
```
```
Please make a component that takes a prompt as input, sends an api request to OpenAI, and returns the response.

  Use this api structure:
  {
    "seed": $SEED,
    "model": "gpt-4o",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "<PROMPT>"}
    ]
  }
```
