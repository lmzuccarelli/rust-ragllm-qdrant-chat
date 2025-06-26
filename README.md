# Overview

A RAG (retrieval augmented genertion) agent using llama3 LLM via the llama.cpp api service with a qdrant
vector database that updates vectors based on reading markdown files (bash scripts) in chunks. 

This version includes a simple chat client for user interaction

## Requirements

This service uses llama.cpp for both model serving and embeddings, as well as qdrant vectordb

### llama.cpp

As this project relies heavily on llama.cpp - please reference the repo (how build, install and server both embeddings and model)

https://github.com/ggml-org/llama.cpp

Start 2 server instances one for the embeddings and another to serve the model to infer against.

### qdrant

Create a directorty called qdrant-data

Download and install qdrant  (simple quick start is to use podman)

```
podman run -p 6333:6333 -p 6334:6334 -e QDRANT_SERVICE_GRPC_PORT="6334"  -v $(pwd)/qdrant-data:/qdrant/storage:z     qdrant/qdrant
```

## Usage

Clone this repo

cd rust-ragllm-qdrant-chat

```
make build
```

Create your relevant markdown documents

Some hints

- create folders under the kbDocsPath folder
- each folder could represent a specific category (or for example just be generic and include a variety of markdown files)
- ensure the folder name matches the category name (in the config.json)
- have descriptive headings (these are used to generate the embeddings) 
- keep the contents size small and ensure the contents describes what you have stated in the heading
- use multiple small files, each with specific headings

Update the config.json file (in this repo)

Launch the embedding service

```
./target/release/rust-ragllm-qdrant-chat --config config.json --loglevel info 
```

Launch normal chat client workflow

```
./target/release/rust-ragllm-qdrant-chat --config config.json --loglevel info -chat-client 
```







