use crate::gui::WallpaperMessage;
use iced::futures::stream::BoxStream;
use iced::{Text, Length, Row};
use indexmap::IndexMap;
use log::{debug, error, info};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use reqwest::Response;
use font_awesome_as_a_crate::Type;
use iced::svg::Handle;

#[derive(Debug, Clone)]
pub(crate) struct DownloadManager {
    downloads: IndexMap<String, ImageDownload>,
    finished_downloads: usize,
    concurrent_downloads: usize,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self {
            downloads: Default::default(),
            concurrent_downloads: 5,
            finished_downloads: 0
        }
    }
}

impl DownloadManager {
    pub fn queue_download<T: ToString>(&mut self, url: T, id: T, save_path: PathBuf) {
        self.downloads.insert(
            id.to_string(),
            ImageDownload {
                url: url.to_string(),
                id: id.to_string(),
                save_path,
            },
        );
        debug!("Download queue updated {:?}", self.downloads);
    }

    pub fn remove_download(&mut self, id: &str) {
        self.downloads.remove(id);
        self.finished_downloads += 1;
    }

    pub fn get_subscriptions(&self) -> Vec<iced::Subscription<DownloadStatus>> {
        self.downloads
            .iter()
            .take(self.concurrent_downloads) // limit downloads at the same time
            .map(|(_, d)| iced::Subscription::from_recipe(d.clone()))
            .collect()
    }

    pub fn view(&self) -> Row<WallpaperMessage> {
        let download_svg = font_awesome_as_a_crate::svg(Type::Solid, "download").unwrap();
        let complete_svg = font_awesome_as_a_crate::svg(Type::Solid, "check").unwrap();
        let download_icon = iced::svg::Svg::new(Handle::from_memory(download_svg.as_bytes().to_vec()));
        let complete_icon = iced::svg::Svg::new(Handle::from_memory(complete_svg.as_bytes().to_vec()));
        if self.downloads.len() > 0 || self.finished_downloads > 0 {
            Row::new().push(download_icon.height(Length::Units(15)))
                .push(Text::new(format!("{}", self.downloads.len())))
                .push(complete_icon.height(Length::Units(15)))
                .push(Text::new(format!("{}", self.finished_downloads)))
        } else {
            Row::new().push(download_icon.height(Length::Units(15)))
                .push(Text::new("0"))
        }
    }

    pub fn set_concurrent_downloads(&mut self, concurrent_downloads: usize) {
        self.concurrent_downloads = concurrent_downloads;
    }
}

/// Provides a subscriber for Iced to return messages
#[derive(Debug, Clone)]
struct ImageDownload {
    /// URL of the image we're downloading
    url: String,
    /// ID of the message
    id: String,
    /// Location to store the image
    save_path: PathBuf,
}

#[derive(Debug)]
enum DownloadState {
    Started {
        url: String,
        id: String,
        save_path: PathBuf,
    },
    Downloading {
        response: Box<Response>,
        file: Box<File>,
        total: u64,
        downloaded: u64,
        id: String,
        save_path: PathBuf,
    },
    Completed,
}

#[derive(Clone, Debug)]
pub(crate) enum DownloadStatus {
    Progress(String, f32),
    Failed(String),
    Finished(String),
}

impl<H, I> iced_native::subscription::Recipe<H, I> for ImageDownload
where
    H: std::hash::Hasher,
{
    type Output = DownloadStatus;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        self.url.hash(state);
    }

    fn stream(self: Box<Self>, _: BoxStream<I>) -> BoxStream<Self::Output> {
        Box::pin(futures::stream::unfold(
            DownloadState::Started {
                url: self.url,
                id: self.id,
                save_path: self.save_path,
            },
            |state| async move {
                match state {
                    DownloadState::Started { url, id, save_path } => {
                        info!("Downloading url: {}", &url);
                        let response = reqwest::get(&url).await;
                        match response {
                            Ok(response) => {
                                if let Some(total) = response.content_length() {
                                    if let Ok(file) = File::create(&save_path).await {
                                        Some((
                                            DownloadStatus::Progress(id.clone(), 0.0),
                                            DownloadState::Downloading {
                                                response: Box::new(response),
                                                file: Box::new(file),
                                                total,
                                                downloaded: 0,
                                                id,
                                                save_path,
                                            },
                                        ))
                                    } else {
                                        Some((DownloadStatus::Failed(id), DownloadState::Completed))
                                    }
                                } else {
                                    error!("Failed to create file {:?}", &save_path);
                                    Some((DownloadStatus::Failed(id), DownloadState::Completed))
                                }
                            }
                            Err(_) => Some((DownloadStatus::Failed(id), DownloadState::Completed)),
                        }
                    }
                    DownloadState::Downloading {
                        mut response,
                        mut file,
                        total,
                        downloaded,
                        id,
                        save_path,
                    } => match response.chunk().await {
                        Ok(Some(chunk)) => {
                            debug!("Downloaded chunk {} bytes {}", &id, chunk.len());
                            let downloaded = downloaded + chunk.len() as u64;
                            let percentage = (downloaded as f32 / total as f32) * 100.0;
                            if file.write(&chunk).await.is_ok() {
                                Some((
                                    DownloadStatus::Progress(id.clone(), percentage),
                                    DownloadState::Downloading {
                                        response,
                                        file,
                                        total,
                                        downloaded,
                                        id,
                                        save_path,
                                    },
                                ))
                            } else {
                                error!("Failed to write file! {:?}", &save_path);
                                tokio::fs::remove_file(&save_path)
                                    .await
                                    .expect("Failed to delete file");
                                Some((DownloadStatus::Failed(id), DownloadState::Completed))
                            }
                        }
                        Ok(None) => Some((DownloadStatus::Finished(id), DownloadState::Completed)),
                        Err(_) => Some((DownloadStatus::Failed(id), DownloadState::Completed)),
                    },
                    DownloadState::Completed => {
                        debug!("Closing download");
                        None
                    }
                }
            },
        ))
    }
}
