use crate::error::handler::*;
use crate::MarkdownFile;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::with_payload_selector::SelectorOptions;
use qdrant_client::qdrant::{
    CreateCollection, Distance, PointStruct, ScoredPoint, SearchPoints, UpsertPointsBuilder,
    VectorParams, VectorsConfig, WithPayloadSelector,
};
use qdrant_client::Payload;
use qdrant_client::Qdrant;
use serde_json::json;

pub struct VectorDB {
    id: u64,
    client: Qdrant,
}

impl VectorDB {
    pub fn new(client: Qdrant) -> Self {
        Self { id: 0, client }
    }

    pub async fn reset_collection(
        &self,
        collection: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client.delete_collection(collection.clone()).await?;

        self.client
            .create_collection(CreateCollection {
                collection_name: collection,
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: 384,
                        distance: Distance::Cosine.into(),
                        hnsw_config: None,
                        quantization_config: None,
                        on_disk: None,
                        datatype: None,
                        multivector_config: None,
                    })),
                }),
                ..Default::default()
            })
            .await?;

        Ok(())
    }

    pub async fn upsert_embedding(
        &mut self,
        collection: String,
        embedding: Vec<f32>,
        mkd_file: &MarkdownFile,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload: Payload = json!({
            "id": mkd_file.path.clone(),
            "contents": mkd_file.contents.clone(),
        })
        .try_into()
        .map_err(|_| EmbeddingsError {
            details: "".to_string(),
        })?;

        let points = vec![PointStruct::new(self.id, embedding.clone(), payload)];
        self.client
            .upsert_points(UpsertPointsBuilder::new(collection, points))
            .await?;
        self.id += 1;

        Ok(())
    }

    pub async fn search(
        &self,
        collection: String,
        embedding: Vec<f32>,
        search_limit: u64,
    ) -> Result<Vec<ScoredPoint>, Box<dyn std::error::Error>> {
        let payload_selector = WithPayloadSelector {
            selector_options: Some(SelectorOptions::Enable(true)),
        };

        let search_points = SearchPoints {
            collection_name: collection,
            vector: embedding.clone(),
            limit: search_limit,
            with_payload: Some(payload_selector),
            ..Default::default()
        };

        let search_result = self.client.search_points(search_points).await?;
        let result = search_result.result.clone();
        Ok(result)
    }
}
