use crate::api::schema::*;
use futures::StreamExt;
use ollama_rs::generation::completion::request::GenerationRequest;
//use ollama_rs::generation::completion::GenerationResponse;
use ollama_rs::Ollama;
use tokio::io::AsyncWriteExt;

pub async fn gen_stream_print(ollama: &Ollama, gen_req: GenerationRequest) -> Result<()> {
    // get response as stream
    let mut stream = ollama.generate_stream(gen_req).await?;

    let mut stdout = tokio::io::stdout();
    let mut char_count = 0;

    while let Some(res) = stream.next().await {
        let res_list = res?;

        for res in res_list {
            let bytes = res.response.as_bytes();

            // Poor man's wrapping
            char_count += bytes.len();
            if char_count > 80 {
                stdout.write_all(b"\n").await?;
                char_count = 0;
            }

            // Write output
            stdout.write_all(bytes).await?;
            stdout.flush().await?;
        }
    }

    stdout.write_all(b"\n").await?;
    stdout.flush().await?;

    Ok(())
}
