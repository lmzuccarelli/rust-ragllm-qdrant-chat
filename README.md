# Overview

A RAG (retrieval augmented genertion) agent using llama3 LLM via the ollama api service with a qdrant
vector database that updates vectors based on reading markdown files in chunks 

## Requirements

This service uses ollama3 (and the all-minilm embedding model), as well as qdrant vectordb

### ollama3

Download and launch olloma3 services https://ollama.com/

### qdrant

Create a directorty called qdrant-data

Download and install qdrant  (simple quick start is to use podman)

```
podman run -p 6333:6333 -p 6334:6334 -e QDRANT_SERVICE_GRPC_PORT="6334"  -v $(pwd)/qdrant-data:/qdrant/storage:z     qdrant/qdrant
```

## Usage

Clone this repo

cd rust-ragllm-qdrant

```
make build
```

Create your relevant markdown documents

Some hints

- create folders under the kbDocsPath folder
- each folder could represent a specific category (or for example just be generic and include a variety of markdown files)
- ensure the folder name matches the category name (in the config.json)
- have descriptive headings (these are used to generate the embeddings) 
- keep the contents size small and ensure the contents descibes what you have stated in the heading
- use multiple small files, each with specific headings

Update the config.json file (in this repo)

Launch the embedding service

```
./target/release/rust-ragllm-qdrant --config config.json --loglevel info 
```

Launch normal prompt workflow (starts a service omn port descibed in serverPort field of the config)

```
./target/release/rust-ragllm-qdrant --config config.json --loglevel info --skip-embedding 
```







