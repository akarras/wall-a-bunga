use crate::download_manager::{DownloadManager, DownloadStatus};
use crate::font_awesome::FAIcon;
use crate::settings::SavedSettings;
use crate::style::{button_style, inactive_style};
use crate::style::{make_button, make_button_fa};
use crate::submenus::ratio_menu::RatioMenu;
use crate::submenus::resolution_menu::ResolutionOptionsMenu;
use crate::utils::trendy_number_format;
use anyhow::Result;
use font_awesome_as_a_crate::Type;
use iced::widget::image::Viewer;
use iced::widget::scrollable::Viewport;
use iced::widget::{
    image, Button, Checkbox, Column, Container, Image, PickList, ProgressBar,
    Row, Scrollable, Space, Text, TextInput,
};
use iced::{
    alignment, executor, Alignment, Application, Command, Element, Length,
    Subscription,
};
use log::{debug, error, info, warn};
use native_dialog::FileDialog;
use rand::{thread_rng, RngCore};
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use thiserror::Error;
use tokio::fs::metadata;
use tokio::task::spawn_blocking;
use wallapi::types::{
    Categories, Category, GenericResponse, ListingData, Purity, SearchMetaData, SearchOptions,
    Sorting, XYCombo,
};
use wallapi::{WallhavenApiClientError, WallhavenClient};

#[derive(Debug, Default)]
pub(crate) struct WallpaperUi {
    controls: SearchControls,
    search_value: String,
    search_results: Vec<(ListingData, ImageView)>,
    search_meta: Option<SearchMetaData>,
    search_options: SearchOptions,
    error_message: String,
    settings: SavedSettings,
    api_key: String,
    resolution_menu: ResolutionOptionsMenu,
    aspect_menu: RatioMenu,
    download_manager: DownloadManager,
    concurrent_download_control: IncrementControl,
    preview_mode: PreviewMode,
}

#[derive(Debug, Default)]
struct IncrementControl {
    value: i32,
}

impl IncrementControl {
    fn view(&self) -> Row<WallpaperMessage> {
        Row::new()
            .push(
                make_button("-")
                    .on_press(WallpaperMessage::ChangeConcurrentDownloads(self.value - 1))
                    .padding([5, 5]),
            )
            .push(Text::new(format!("{}", self.value)).size(26))
            .push(
                make_button("+")
                    .on_press(WallpaperMessage::ChangeConcurrentDownloads(self.value + 1))
                    .padding([5, 5]),
            )
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
enum ImageState {
    #[default]
    Unselected,
    Selected,
    Queued,
    // f32 measures progress
    Downloading(f32),
    Downloaded,
    Failed,
}
#[derive(Debug, Clone)]
pub(crate) struct ImageView {
    state: ImageState,
    image_handle: image::Handle,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum PurityOptions {
    Sfw,
    Sketchy,
    Nsfw,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum ContentTypes {
    Anime,
    General,
    People,
}

#[derive(Debug, Clone)]
pub(crate) enum SelectionUpdateType {
    Single(String),
    SelectAll,
    DeselectAll,
}

#[derive(Debug, Clone)]
pub(crate) enum WallpaperMessage {
    Search(),
    SearchUpdated(String),
    SearchReceived(GenericResponse<Vec<(ListingData, ImageView)>>),
    /// Where String == image.id
    SelectionUpdate(SelectionUpdateType),
    DownloadImages(),
    SortingTypeChanged(Sorting),
    TogglePurity(PurityOptions),
    ToggleContentType(ContentTypes),
    ApiTokenSet(String),
    ChangeSubmenu(Submenu),
    ChooseDirectory(),
    DirectoryChosen(Option<PathBuf>),
    ResolutionSelected(XYCombo),
    ResolutionIsSingleTargetChanged(bool),
    AspectRatioSelected(XYCombo),
    SaveSettings(),
    SaveCompleted(()),
    SetIgnoreDownloaded(bool),
    DownloadUpdated(DownloadStatus),
    SetMinimumResolution(XYCombo),
    ChangeConcurrentDownloads(i32),
    Scroll(Viewport),
    NextPage(),
    /// Downloads the preview, usize is an index into the currently downloaded results.
    DownloadPreview(usize),
    CancelPreview(),
    UpdatePreviewMode(PreviewMode),
}

#[derive(Default, Debug, Clone)]
pub(crate) enum PreviewMode {
    #[default]
    Disable,
    /// User has requested a full screen preview, but we don't have the full size downloaded
    PreviewRequestDownloading {
        /// Image handle to the small thumbnail
        preview_handle: image::Handle,
        cancel_mechanism: tokio::sync::mpsc::Sender<()>,
    },
    /// Handle to the downloaded image
    PreviewView(image::Handle),
    PreviewFailed,
}

#[derive(Default, Debug, Eq, PartialEq, Clone)]
pub(crate) enum Submenu {
    #[default]
    None,
    Settings,
    Resolution,
    AspectRatio,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct SearchControls {
    submenu: Submenu,
}

#[derive(Error, Debug)]
pub enum WallGuiError {
    #[error("Networking error")]
    Request(#[from] reqwest::Error),
    #[error("Api Client Error")]
    WHClientResult(#[from] WallhavenApiClientError),
    #[error("Bad response")]
    BadResponse(String),
    #[error("File error is invalid")]
    FileError(#[from] std::io::Error),
}

pub type GuiResult<T> = Result<T>;

impl WallpaperUi {
    async fn fetch_image(
        data: ListingData,
        mut storage_directory: PathBuf,
    ) -> Result<(ListingData, ImageView), reqwest::Error> {
        let bytes = reqwest::get(&data.thumbs.small).await?.bytes().await?;
        storage_directory.push(data.path.split('/').last().unwrap_or(""));
        let state = match metadata(storage_directory).await {
            Ok(_) => ImageState::Downloaded,
            Err(_) => ImageState::Unselected,
        };

        let result = ImageView {
            state,
            image_handle: image::Handle::from_memory(bytes.as_ref().to_vec()),
        };
        Ok((data, result))
    }

    async fn fetch_full_image(url: String) -> Result<image::Handle, anyhow::Error> {
        let bytes = reqwest::get(url).await?.bytes().await?;
        Ok(spawn_blocking(move || {
            if let Ok(image) = image_rs::load_from_memory(&bytes) {
                let pixels: Vec<_> = image
                    .to_rgba8()
                    .pixels()
                    .flat_map(|m| m.0)
                    .collect();
                image::Handle::from_pixels(image.width(), image.height(), pixels)
            } else {
                warn!("Failed to convert image ourselves, letting Iced try.");
                image::Handle::from_memory(bytes.to_vec())
            }
        })
        .await?)
    }

    async fn search_command(
        options: SearchOptions,
        directory: PathBuf,
    ) -> GenericResponse<Vec<(ListingData, ImageView)>> {
        match WallpaperUi::do_search(options, directory).await {
            Ok(search) => search,
            Err(e) => {
                error!("{:3?}", e);
                GenericResponse {
                    error: Some(e.to_string()),
                    ..Default::default()
                }
            }
        }
    }

    async fn do_search(
        options: SearchOptions,
        directory: PathBuf,
    ) -> GuiResult<GenericResponse<Vec<(ListingData, ImageView)>>> {
        let response = WallhavenClient::search(&options).await?;
        if let Some(data) = response.data {
            info!("Received {} search results", &data.len());
            let images: Vec<_> = data
                .into_iter()
                .map(|listing| WallpaperUi::fetch_image(listing, directory.clone()))
                .collect();
            let joined = futures::future::join_all(images).await;
            let map: Vec<_> = joined.into_iter().filter_map(|m| m.ok()).collect();
            info!("Downloaded {} images", &map.len());
            return Ok(GenericResponse {
                data: Some(map),
                error: response.error,
                meta: response.meta,
            });
        }

        Err(WallGuiError::BadResponse(
            response
                .error
                .unwrap_or_else(|| "No error message".to_string()),
        )
        .into())
    }

    async fn choose_directory() -> Option<PathBuf> {
        FileDialog::new().show_open_single_dir().ok().flatten()
    }

    /// guesstimate our loading status based on our page
    fn get_loading_status(&self) -> Text {
        let page = self.search_options.page.unwrap_or(1) as i64;
        let is_loading = match &self.search_meta {
            Some(meta) => meta.current_page != page,
            None => true, // if this is none, we haven't received anything yet
        };
        debug!(
            "calculated loading status {:?} page {:?}",
            self.search_meta, self.search_options.page
        );
        let loading_text = if is_loading { "Loading..." } else { "" };
        Text::new(loading_text).size(42)
    }
}

impl Application for WallpaperUi {
    type Executor = executor::Default;
    type Message = WallpaperMessage;
    type Flags = Option<SavedSettings>;

    fn new(flags: Self::Flags) -> (Self, Command<WallpaperMessage>) {
        let key = flags.clone().unwrap_or_default().api_key;
        (
            Self {
                settings: flags.unwrap_or_default(),
                search_options: SearchOptions {
                    api_key: key.clone(),
                    ..Default::default()
                },
                api_key: key.unwrap_or_default(),
                concurrent_download_control: IncrementControl {
                    value: 5,
                },
                ..Self::default()
            },
            Command::perform(
                WallpaperUi::search_command(SearchOptions::default(), "./".into()),
                WallpaperMessage::SearchReceived,
            ),
        )
    }

    fn title(&self) -> String {
        "wall-a-bunga".to_string()
    }

    fn update(&mut self, message: WallpaperMessage) -> Command<WallpaperMessage> {
        match message {
            WallpaperMessage::Search() => {
                self.search_options.set_query(self.search_value.clone());
                self.search_options.page = None;
                let mut rng = thread_rng();
                self.search_options.seed = Some(rng.next_u64().to_string());
                self.search_results.clear();
                self.preview_mode = PreviewMode::Disable;
                return Command::perform(
                    WallpaperUi::search_command(
                        self.search_options.clone(),
                        self.settings
                            .save_directory
                            .as_ref()
                            .unwrap_or(&"./".to_string())
                            .into(),
                    ),
                    WallpaperMessage::SearchReceived,
                );
            }
            WallpaperMessage::SearchUpdated(msg) => {
                self.search_value = msg;
            }
            WallpaperMessage::SearchReceived(mut values) => {
                if let Some(data) = &mut values.data {
                    info!("Updated search results");
                    self.search_results.append(data);
                } else if let Some(error) = values.error {
                    self.error_message = error;
                }
                debug!("Updating search meta: {:?}", values.meta);
                self.search_meta = values.meta;
            }
            WallpaperMessage::SelectionUpdate(option) => {
                match option {
                    SelectionUpdateType::Single(id) => {
                        let image = self.search_results.iter_mut().find(|(l, _)| l.id == id);
                        if let Some((_, result_data)) = image {
                            // toggle checked
                            result_data.state = match result_data.state {
                                ImageState::Unselected => ImageState::Selected,
                                ImageState::Selected => ImageState::Unselected,
                                ImageState::Failed => ImageState::Selected,
                                // default return same state
                                _ => result_data.state,
                            }
                        }
                    }
                    SelectionUpdateType::SelectAll => {
                        for (_, r) in &mut self.search_results {
                            r.state = match r.state {
                                ImageState::Unselected => ImageState::Selected,
                                _ => r.state,
                            }
                        }
                    }
                    SelectionUpdateType::DeselectAll => {
                        for (_, r) in &mut self.search_results {
                            r.state = match r.state {
                                ImageState::Selected => ImageState::Unselected,
                                _ => r.state,
                            }
                        }
                    }
                }
            }
            WallpaperMessage::DownloadImages() => {
                let image_urls = self
                    .search_results
                    .iter_mut()
                    .rev() // reverse the order so that when we queue these, the first are inserted last
                    .filter(|(_, image)| {
                        image.state == ImageState::Selected || image.state == ImageState::Failed
                    })
                    .map(|(listing, image)| {
                        image.state = ImageState::Queued;
                        (&listing.path, &listing.id)
                    });

                for (url, id) in image_urls {
                    let file_name = match url.split('/').last() {
                        Some(name) => name,
                        None => {
                            error!("Error getting filename of url: {}", url);
                            continue;
                        }
                    };
                    let save_path = PathBuf::from(
                        &self
                            .settings
                            .save_directory
                            .clone()
                            .unwrap_or_else(|| "./".to_string()),
                    )
                    .join(file_name);
                    self.download_manager.queue_download(url, id, save_path);
                }
            }
            WallpaperMessage::SortingTypeChanged(sort) => {
                self.search_options.sorting = Some(sort);
            }
            WallpaperMessage::TogglePurity(purity_toggle) => {
                let purity = self.search_options.purity.get_or_insert(Purity::default());
                match purity_toggle {
                    PurityOptions::Sfw => {
                        purity.clean = !purity.clean;
                    }
                    PurityOptions::Sketchy => {
                        purity.sketchy = !purity.sketchy;
                    }
                    PurityOptions::Nsfw => {
                        purity.nsfw = !purity.nsfw;
                    }
                }
            }
            WallpaperMessage::ToggleContentType(content_toggle) => {
                let content = self
                    .search_options
                    .categories
                    .get_or_insert(Categories::default());
                match content_toggle {
                    ContentTypes::Anime => {
                        content.anime = !content.anime;
                    }
                    ContentTypes::General => {
                        content.general = !content.general;
                    }
                    ContentTypes::People => {
                        content.people = !content.people;
                    }
                }
            }
            WallpaperMessage::ApiTokenSet(token) => {
                self.api_key = token;
                if !self.api_key.is_empty() {
                    self.search_options.api_key = Some(self.api_key.clone());
                } else {
                    self.search_options.api_key = None;
                }
            }
            WallpaperMessage::ChangeSubmenu(menu) => {
                // Toggle the submenu to none if already set, otherwise set value
                if self.controls.submenu == menu {
                    self.controls.submenu = Submenu::None;
                } else {
                    self.controls.submenu = menu;
                }
            }
            WallpaperMessage::ChooseDirectory() => {
                return Command::perform(
                    WallpaperUi::choose_directory(),
                    WallpaperMessage::DirectoryChosen,
                );
            }
            WallpaperMessage::DirectoryChosen(path) => {
                if let Some(p) = path {
                    if let Some(s) = p.to_str() {
                        self.settings.save_directory = Some(s.to_string());
                        return Command::none();
                    }
                }
                self.settings.save_directory = None;
            }
            WallpaperMessage::ResolutionSelected(resolution) => {
                // Clear out the minimum resolution option
                self.search_options.minimum_resolution = None;
                debug!("Resolution selected {}", resolution);
                let res_map = self
                    .search_options
                    .resolutions
                    .get_or_insert(HashSet::new());
                if res_map.contains(&resolution) {
                    res_map.remove(&resolution);
                    if res_map.is_empty() {
                        self.search_options.resolutions = None;
                    }
                } else {
                    res_map.insert(resolution);
                }
            }
            WallpaperMessage::AspectRatioSelected(aspect_ratio) => {
                info!("Selected aspect ratio {}", aspect_ratio);
                let ratio_map = self.search_options.ratios.get_or_insert(HashSet::new());
                if ratio_map.contains(&aspect_ratio) {
                    ratio_map.remove(&aspect_ratio);
                } else {
                    ratio_map.insert(aspect_ratio);
                }
            }
            WallpaperMessage::SaveSettings() => {
                self.settings.api_key = self.search_options.api_key.clone();
                return Command::perform(
                    SavedSettings::save_settings(self.settings.clone()),
                    WallpaperMessage::SaveCompleted,
                );
            }
            WallpaperMessage::SaveCompleted(()) => {
                info!("Save complete!");
            }
            WallpaperMessage::SetIgnoreDownloaded(value) => {
                self.settings.ignore_downloaded = value;
            }
            WallpaperMessage::DownloadUpdated(u) => match u {
                DownloadStatus::Progress(id, progress) => {
                    if let Some((_, i)) = self
                        .search_results
                        .iter_mut()
                        .find(|(val, _)| val.id.eq(&id))
                    {
                        i.state = ImageState::Downloading(progress);
                    }
                }
                DownloadStatus::Failed(image) => {
                    error!("Image {} failed", image);
                    if let Some((_, l)) = self
                        .search_results
                        .iter_mut()
                        .find(|(l, _)| l.id.eq(&image))
                    {
                        l.state = ImageState::Failed
                    };
                    self.download_manager.remove_download(&image);
                }
                DownloadStatus::Finished(id) => {
                    info!("Image {} complete", id);
                    if let Some((_, l)) = self.search_results.iter_mut().find(|(l, _)| l.id.eq(&id))
                    {
                        l.state = ImageState::Downloaded
                    };
                    self.download_manager.remove_download(&id);
                }
            },
            WallpaperMessage::ResolutionIsSingleTargetChanged(res_mode) => {
                self.resolution_menu.is_minimum_set = res_mode;
            }
            WallpaperMessage::SetMinimumResolution(resolution) => {
                // clear out other resolutions options in preference of min resolution
                info!("Minimum resolution set to {}", resolution);
                self.search_options.resolutions = None;
                self.search_options.minimum_resolution = Some(resolution);
            }
            WallpaperMessage::ChangeConcurrentDownloads(c) => {
                let value = match c > 0 && c < 10 {
                    true => c,
                    false => self.concurrent_download_control.value,
                };
                self.concurrent_download_control.value = value;
                self.download_manager
                    .set_concurrent_downloads(value as usize)
            }
            WallpaperMessage::Scroll(scroll) => {
                if let PreviewMode::Disable = &self.preview_mode {
                    // currently we only want to respond to scroll events when the user can see the image list
                    debug!("scroll {:?}", scroll);
                    // scroll ranges from 0 to 1. if 1, try to load more wallpapers
                    let search_meta = if let Some(search_meta) = &self.search_meta {
                        search_meta
                    } else {
                        return Command::none();
                    };
                    let page = self.search_options.page.unwrap_or(1);
                    if scroll.relative_offset().y >= 1.0
                        && page < search_meta.last_page as i32
                        && page == search_meta.current_page as i32
                    {
                        self.search_options.page = Some(page + 1);
                        return Command::perform(
                            WallpaperUi::search_command(
                                self.search_options.clone(),
                                self.settings
                                    .save_directory
                                    .as_ref()
                                    .unwrap_or(&"./".to_string())
                                    .into(),
                            ),
                            WallpaperMessage::SearchReceived,
                        );
                    }
                }
            }
            WallpaperMessage::NextPage() => {
                let mut page = self.search_options.page.unwrap_or(1);
                if let Some(max_page) = self.search_meta.as_ref().map(|m| m.last_page) {
                    page += 1;
                    if page > max_page as i32 {
                        page = max_page as i32;
                    }
                    self.search_options.set_page(page);
                    return Command::perform(
                        WallpaperUi::search_command(
                            self.search_options.clone(),
                            self.settings
                                .save_directory
                                .as_ref()
                                .unwrap_or(&"./".to_string())
                                .into(),
                        ),
                        WallpaperMessage::SearchReceived,
                    );
                }
            }
            WallpaperMessage::UpdatePreviewMode(preview) => {
                self.preview_mode = preview;
            }
            WallpaperMessage::DownloadPreview(index) => {
                if let Some((value, image_view)) = self.search_results.get(index) {
                    let url = value.path.clone();
                    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
                    let future = async move {
                        tokio::select! {
                            img = WallpaperUi::fetch_full_image(url) => Some(img),
                            _ = receiver.recv() => None,
                        }
                    };

                    self.preview_mode = PreviewMode::PreviewRequestDownloading {
                        preview_handle: image_view.image_handle.clone(),
                        cancel_mechanism: sender,
                    };
                    return Command::perform(future, |selection| match selection {
                        Some(wall) => {
                            if let Ok(handle) = wall {
                                info!("preview loaded!");
                                WallpaperMessage::UpdatePreviewMode(PreviewMode::PreviewView(
                                    handle,
                                ))
                            } else {
                                error!("failed to load preview");
                                WallpaperMessage::UpdatePreviewMode(PreviewMode::PreviewFailed)
                            }
                        }
                        None => {
                            info!("User cancelled task");
                            WallpaperMessage::UpdatePreviewMode(PreviewMode::Disable)
                        }
                    });
                }
            }
            WallpaperMessage::CancelPreview() => match &self.preview_mode {
                PreviewMode::PreviewRequestDownloading {
                    cancel_mechanism, ..
                } => {
                    let cancel_mechanism = cancel_mechanism.clone();
                    return Command::perform(
                        async move {
                            cancel_mechanism.send(()).await.unwrap();
                        },
                        |_| {
                            info!("cancel sent!");
                            WallpaperMessage::UpdatePreviewMode(PreviewMode::Disable)
                        },
                    );
                }
                _ => self.preview_mode = PreviewMode::Disable,
            },
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(self.download_manager.get_subscriptions())
            .map(WallpaperMessage::DownloadUpdated)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let loading_status = self.get_loading_status();
        let selected_count = self
            .search_results
            .iter()
            .filter(|(_, l)| l.state == ImageState::Selected)
            .count();

        // Build columns of 5 with our images
        let ignore_downloaded = self.settings.ignore_downloaded;

        let results = match self.settings.ignore_downloaded {
            true => {
                let num_hidden = self
                    .search_results
                    .iter()
                    .filter(|(_, v)| v.state.eq(&ImageState::Downloaded))
                    .count();
                format!(
                    "{} results ({} hidden)",
                    self.search_results.len(),
                    num_hidden
                )
            }
            false => {
                format!("{} results", self.search_results.len())
            }
        };

        // create a next button based on whether or we have another page
        let next_button = if self
            .search_meta
            .as_ref()
            .map(|m| (self.search_options.page.unwrap_or(1) as i64).ne(&m.last_page))
            .unwrap_or(true)
        {
            Column::new().push(
                make_button_fa("next page", "arrow-right").on_press(WallpaperMessage::NextPage()),
            )
        } else {
            Column::new()
        };
        let is_preview_disabled = matches!(&self.preview_mode, PreviewMode::Disable);

        let main_content = match &self.preview_mode {
            PreviewMode::Disable => {
                let mut row = Row::new();
                let mut column = Column::new().spacing(5).push(Text::new("Search results"));

                for (index, (listing, image)) in self
                    .search_results
                    .iter()
                    .filter(|(_, image)| {
                        !ignore_downloaded || matches!(image.state, ImageState::Downloaded)
                    })
                    .enumerate()
                {
                    let mut wallpaper_column = Column::new()
                        // .width(Length::Fixed(250.0))
                        .push(
                            Button::new(Image::new(image.image_handle.clone()))
                                .style(iced::theme::Button::Custom(Box::new(match image.state {
                                    ImageState::Selected => button_style::Button::Primary,
                                    ImageState::Unselected => button_style::Button::Inactive,
                                    ImageState::Queued => button_style::Button::Downloading,
                                    ImageState::Downloading(_) => button_style::Button::Downloading,
                                    ImageState::Downloaded => button_style::Button::Downloaded,
                                    ImageState::Failed => button_style::Button::Failed,
                                })))
                                .on_press(WallpaperMessage::SelectionUpdate(
                                    SelectionUpdateType::Single(listing.id.clone()),
                                )),
                        )
                        .push(
                            Row::new()
                                .push(
                                    Column::new()
                                        .push(Text::new(format!(
                                            "w:{}px h:{}px",
                                            listing.dimension_x, listing.dimension_y
                                        )))
                                        .width(Length::Shrink)
                                        .push(
                                            Row::new()
                                                .width(Length::Shrink)
                                                .push(
                                                    FAIcon::new(Type::Solid, "heart")
                                                        .svg()
                                                        .height(Length::Fixed(20.0)),
                                                )
                                                .push(Text::new(trendy_number_format(
                                                    listing.favorites as f64,
                                                )))
                                                .push(Space::new(
                                                    Length::Fixed(5.0),
                                                    Length::Shrink,
                                                ))
                                                .push(
                                                    FAIcon::new(Type::Solid, "eye")
                                                        .svg()
                                                        .height(Length::Fixed(20.0)),
                                                )
                                                .push(Text::new(trendy_number_format(
                                                    listing.views as f64,
                                                )))
                                                .push(Space::new(
                                                    Length::Fixed(5.0),
                                                    Length::Shrink,
                                                ))
                                                .push(Text::new(match &listing.category {
                                                    Category::Anime => "Anime",
                                                    Category::People => "People",
                                                    Category::General => "General",
                                                })),
                                        ),
                                )
                                .push(Space::new(Length::Fixed(10.0), Length::Shrink))
                                .push(
                                    make_button_fa("preview", "image")
                                        .on_press(WallpaperMessage::DownloadPreview(index)),
                                )
                                .width(Length::Shrink),
                        );
                    wallpaper_column = match image.state {
                        ImageState::Downloading(progress) => wallpaper_column.push(
                            ProgressBar::new(0.0..=100.0, progress).width(Length::Fixed(256.0)),
                        ),
                        _ => wallpaper_column,
                    };
                    row = row.push(wallpaper_column);
                    // grid wrapping
                    if index % 5 == 4 {
                        let element: Element<'_, WallpaperMessage> = row.into();
                        // let element = element.explain(Color::WHITE);
                        column = column.push(element);
                        row = Row::new();
                    }
                }
                column
                    .push(row)
                    .push(loading_status)
                    .push(next_button)
                    .width(Length::Fill)
                    .align_items(Alignment::Center)
            }
            PreviewMode::PreviewRequestDownloading { preview_handle, .. } => Column::new()
                .push(Text::new("Downloading full-size image preview").size(26))
                .push(make_button_fa("cancel", "ban").on_press(WallpaperMessage::CancelPreview()))
                .push(Image::new(preview_handle.clone())),
            PreviewMode::PreviewView(image) => Column::new()
                .push(
                    make_button_fa("back", "arrow-left")
                        .on_press(WallpaperMessage::UpdatePreviewMode(PreviewMode::Disable)),
                )
                .push(Viewer::new(image.clone()).width(Length::Fill))
                .align_items(Alignment::Center),
            PreviewMode::PreviewFailed => Column::new()
                .push(
                    make_button_fa("back", "arrow-left")
                        .on_press(WallpaperMessage::UpdatePreviewMode(PreviewMode::Disable)),
                )
                .push(Text::new("Failed to load preview").size(26))
                .align_items(Alignment::Center),
        };

        let text_input = Row::new()
            .height(Length::Shrink)
            .width(Length::Fill)
            .push(
                TextInput::new("Search", &self.search_value)
                    .size(16)
                    .padding(15)
                    .on_input(WallpaperMessage::SearchUpdated)
                    .on_submit(WallpaperMessage::Search()),
            )
            .push(
                make_button_fa("search", "search")
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .on_press(WallpaperMessage::Search()),
            );

        let default_t = Categories::default();
        let default_p = Purity::default();
        let search_type = self
            .search_options
            .categories
            .as_ref()
            .unwrap_or(&default_t);
        let purity = self.search_options.purity.as_ref().unwrap_or(&default_p);

        let mut nsfw_button = make_button("nsfw").style(inactive_style(purity.nsfw));
        if !self.api_key.is_empty() {
            nsfw_button = nsfw_button.on_press(WallpaperMessage::TogglePurity(PurityOptions::Nsfw));
        }

        let filter_row = Row::new()
            .height(Length::Shrink)
            .width(Length::Shrink)
            //.align_items(Align::Center)
            .push(
                make_button("general")
                    .on_press(WallpaperMessage::ToggleContentType(ContentTypes::General))
                    .style(inactive_style(search_type.general)),
            )
            .push(
                make_button("anime")
                    .on_press(WallpaperMessage::ToggleContentType(ContentTypes::Anime))
                    .style(inactive_style(search_type.anime)),
            )
            .push(
                make_button("people")
                    .on_press(WallpaperMessage::ToggleContentType(ContentTypes::People))
                    .style(inactive_style(search_type.people)),
            )
            .push(Space::new(Length::FillPortion(5), Length::Shrink))
            .push(
                make_button("clean")
                    .on_press(WallpaperMessage::TogglePurity(PurityOptions::Sfw))
                    .style(inactive_style(purity.clean)),
            )
            .push(
                make_button("sketchy")
                    .on_press(WallpaperMessage::TogglePurity(PurityOptions::Sketchy))
                    .style(inactive_style(purity.sketchy)),
            )
            .push(nsfw_button)
            .push(
                PickList::new(
                    &Sorting::LIST[..],
                    self.search_options.sorting,
                    WallpaperMessage::SortingTypeChanged,
                )
                .style(iced::theme::PickList::Custom(
                    Rc::new(crate::style::pick_style::PickList),
                    Rc::new(crate::style::pick_style::PickList),
                ))
                .text_size(26)
                .width(Length::Shrink)
                .padding(5),
            )
            .push(
                make_button("resolutions")
                    .on_press(WallpaperMessage::ChangeSubmenu(Submenu::Resolution)),
            )
            .push(
                make_button("aspect ratio")
                    .on_press(WallpaperMessage::ChangeSubmenu(Submenu::AspectRatio)),
            )
            .push(Space::new(Length::FillPortion(5), Length::Shrink))
            .push(
                make_button("select all").on_press(WallpaperMessage::SelectionUpdate(
                    SelectionUpdateType::SelectAll,
                )),
            )
            .push(
                make_button("deselect all").on_press(WallpaperMessage::SelectionUpdate(
                    SelectionUpdateType::DeselectAll,
                )),
            )
            .push(
                make_button("settings")
                    .on_press(WallpaperMessage::ChangeSubmenu(Submenu::Settings)),
            )
            .push(
                make_button_fa("download", "download").on_press(WallpaperMessage::DownloadImages()),
            );

        let (current_page, last_page) = self
            .search_meta
            .as_ref()
            .map_or((0, 0), |f| (f.current_page, f.last_page));

        let selection_info = Column::new().push(
            Text::new(format!(
                "selected: {}  page: {}/{} {}",
                selected_count, current_page, last_page, results
            ))
            // .color(Color::WHITE)
            .size(26),
        );

        let status_row = Row::new()
            .align_items(Alignment::Center)
            .push(Space::new(Length::Fill, Length::Fixed(10.0)))
            .push(self.download_manager.view())
            .spacing(5);

        let submenu = match self.controls.submenu {
            Submenu::Settings => Column::new()
                .align_items(Alignment::Start)
                .push(Text::new("Settings").size(26))
                .push(
                    Column::new()
                        .padding([10, 5])
                        .push(Text::new("Concurrent Downloads"))
                        .push(self.concurrent_download_control.view()),
                )
                .push(
                    Column::new()
                        .padding([10, 5])
                        .width(Length::Fill)
                        .push(Text::new("wallhaven.cc api token (required for nsfw):"))
                        .push(
                            TextInput::new("api key", &self.api_key)
                                .on_input(WallpaperMessage::ApiTokenSet)
                                .width(Length::Fixed(600.0)),
                        ),
                )
                .push(
                    Row::new()
                        .width(Length::FillPortion(4))
                        .push(
                            Column::new()
                                .padding([10, 5])
                                .push(Text::new("save directory:"))
                                .push(Text::new(
                                    self.settings
                                        .save_directory
                                        .clone()
                                        .map(|s| s.into())
                                        .unwrap_or(Cow::Borrowed("./")),
                                )),
                        )
                        .push(
                            make_button("Choose Directory")
                                .on_press(WallpaperMessage::ChooseDirectory())
                                .padding([10, 5]),
                        ),
                )
                .push(Checkbox::new(
                    "Ignore downloaded",
                    self.settings.ignore_downloaded,
                    WallpaperMessage::SetIgnoreDownloaded,
                ))
                .push(
                    make_button("save settings")
                        .on_press(WallpaperMessage::SaveSettings())
                        .width(Length::Shrink),
                ),
            Submenu::Resolution => Column::new().push(self.resolution_menu.build_resolution_row(
                &self.search_options.resolutions,
                &self.search_options.minimum_resolution,
            )),
            Submenu::AspectRatio => Column::new().push(
                self.aspect_menu
                    .build_ratio_row(&self.search_options.ratios),
            ), // todo implement
            Submenu::None => Column::new(),
        };

        let mut column = Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .align_items(Alignment::Center)
            .spacing(10)
            .push(status_row)
            .push(filter_row)
            .push(submenu)
            .push(text_input);
        // this horrible hack lets me disable the scroll for preview mode.
        // is there a better way to do this?
        // yes.
        // am i going to do it right now?
        // no.
        // maybe one day.
        if is_preview_disabled {
            column = column
                .push(
                    Scrollable::new(main_content)
                        .on_scroll(WallpaperMessage::Scroll)
                        .width(Length::Fill)
                        .height(Length::Fill), // .align_items(Alignment::Center),
                )
                .push(selection_info);
        } else {
            column = column.push(main_content);
        }
        Container::new(column)
            .padding(15)
            .align_y(alignment::Vertical::Top)
            .center_x()
            .into()
    }

    type Theme = iced::Theme;

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    // type Theme = WallabungaTheme;
}

// #[derive(Default)]
// pub struct WallabungaTheme;

// pub enum StyleMode {
//     Dark,
//     Light,
// }

// impl Default for StyleMode {
//     fn default() -> Self {
//         Self::Dark
//     }
// }

// impl StyleSheet for WallabungaTheme {
//     type Style = StyleMode;

//     fn appearance(&self, style: &Self::Style) -> iced::application::Appearance {
//         match style {
//             StyleMode::Dark => iced::application::Appearance {
//                 background_color: Color::from_rgb(0.1, 0.1, 0.11),
//                 text_color: Color::from_rgb(0.98, 0.97, 0.95),
//             },
//             StyleMode::Light => iced::application::Appearance {
//                 background_color: Color::from_rgb(0.98, 0.97, 0.95),
//                 text_color: Color::from_rgb(0.1, 0.1, 0.11),
//             },
//         }
//     }
// }
