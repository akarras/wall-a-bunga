use crate::types::{GenericResponse, ListingData, SearchOptions};
use log::{debug, info};
use thiserror::Error;

pub mod types;

#[derive(Error, Debug)]
pub enum WallhavenApiClientError {
    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("Invalid content supplied")]
    InvalidContent,
}

pub type WHResult<T> = Result<T, WallhavenApiClientError>;

/// Provides a client that provides async access to the Wallhaven api
/// No blocking client is provided, because I don't want to :)
#[derive(Default, Debug, Clone)]
pub struct WallhavenClient {}

impl WallhavenClient {
    /// Searches wallhaven.cc using the given search options
    ///
    /// # Arguments
    /// * `options` - Provides a top down struct of all available search options
    ///
    /// # Example Usage
    /// ```
    /// use wallhaven_api::{WallhavenClient, types::SearchOptions};
    ///
    /// async fn search_example() {
    ///     let results = WallhavenClient::search(&SearchOptions {
    ///         query: Some("Cats".to_string()),
    ///         ..Default::default()
    ///     }).await;
    ///     // Just print out the results as an example
    ///     println!("received wallpapers: {:?}", results);
    /// }
    /// ```
    pub async fn search(options: &SearchOptions) -> WHResult<GenericResponse<Vec<ListingData>>> {
        let search_url_base = "https://wallhaven.cc/api/v1/search";
        let client = reqwest::Client::builder().build()?;
        let request = client.get(search_url_base).query(&options).build()?;
        info!("Requesting from url: {:?}", &request);
        let response = client.execute(request).await?;
        let content = response.json().await?;
        debug!("Received content {:?}", content);
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use crate::{SearchOptions, WallhavenClient};

    #[tokio::test]
    async fn search_test() {
        let results = WallhavenClient::search(&SearchOptions::new())
            .await
            .expect("No failure");
        let values = results.data.unwrap();

        assert_eq!(values.len() > 0, true);
    }
}
