# LLM-made WAVS components

This repo contains examples of one-shot WAVS components made using Claude and Cursor rulefiles. 

Prompts:
```
let's make a component that takes the input of a zip code, queries the openbrewerydb, and returns the breweries in the area.  @https://api.openbrewerydb.org/v1/breweries?by_postal=92101&per_page=3
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
 My api key is WAVS_ENV_OPENAI_KEY in my .env file.
```
