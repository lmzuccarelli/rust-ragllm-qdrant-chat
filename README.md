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

- have descriptive headings (these are used to generate the embeddings) 
- keep the contents size small and ensure the contents descibes what you have stated in the heading
- use multiple small files, each with specific headings

Update the config.json file (in this repo)

Launch the embedding service

```
./target/release/rust-ragllm-qdrant --config config.json --loglevel info --user-prompt "tell me about enclave support"
```

Launch normal prompt workflow

```
./target/release/rust-ragllm-qdrant --config config.json --loglevel info --skip-embedding --user-prompt "tell me about enclave support"
```







