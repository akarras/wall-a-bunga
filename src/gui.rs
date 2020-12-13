use crate::ratio_menu::RatioMenu;
use crate::resolution_menu::ResolutionOptionsMenu;
use crate::settings::SavedSettings;
use crate::style::make_button;
use crate::style::{button_style, inactive_style};
use anyhow::Result;
use iced::{
    button, executor, image, pick_list, scrollable, text_input, Align, Application, Button,
    Checkbox, Column, Command, Container, Element, Image, Length, PickList, Row, Scrollable, Space,
    Text, TextInput,
};
use log::{debug, error, info};
use native_dialog::Dialog;
use rand::{thread_rng, RngCore};
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs::metadata;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use wallapi::types::{
    Categories, GenericResponse, ListingData, Purity, SearchOptions, Sorting, XYCombo,
};
use wallapi::{WallhavenApiClientError, WallhavenClient};

#[derive(Debug, Default, Clone)]
pub(crate) struct WallpaperUi {
    controls: SearchControls,
    search_state: text_input::State,
    search_value: String,
    search_button: button::State,
    current_page: u32,
    max_pages: Option<u32>,
    search_results: Vec<(ListingData, ImageView)>,
    client: WallhavenClient,
    search_options: SearchOptions,
    error_message: String,
    scroll_state: scrollable::State,
    api_text_input: text_input::State,
    settings: SavedSettings,
    api_key: String,
    resolutions_menu_button: button::State,
    aspect_menu_button: button::State,
    resolution_menu: ResolutionOptionsMenu,
    aspect_menu: RatioMenu,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ImageState {
    Unselected,
    Selected,
    Downloading,
    Downloaded,
}

impl Default for ImageState {
    fn default() -> Self {
        ImageState::Unselected
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ImageView {
    state: ImageState,
    image_handle: image::Handle,
    button_state: button::State,
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
pub(crate) enum WallpaperMessage {
    Search(),
    NextPage(),
    SearchUpdated(String),
    SearchReceived(GenericResponse<Vec<(ListingData, ImageView)>>),
    /// Where String == image.id
    ImageChecked(String),
    DownloadImages(),
    ImageDownloaded(()),
    SortingTypeChanged(Sorting),
    TogglePurity(PurityOptions),
    ToggleContentType(ContentTypes),
    ApiTokenSet(String),
    ChangeSubmenu(Submenu),
    ChooseDirectory(),
    DirectoryChosen(Option<PathBuf>),
    ResolutionSelected(XYCombo),
    AspectRatioSelected(XYCombo),
    SaveSettings(),
    SaveCompleted(()),
    SetIgnoreDownloaded(bool),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Submenu {
    None,
    Settings,
    Resolution,
    AspectRatio,
}

impl Default for Submenu {
    fn default() -> Self {
        Submenu::None
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct SearchControls {
    next_page_button: button::State,
    prev_page_button: button::State,
    sorting_picker: pick_list::State<Sorting>,
    download_button: button::State,
    nsfw_button: button::State,
    sketchy_button: button::State,
    sfw_button: button::State,
    general_button: button::State,
    anime_button: button::State,
    people_button: button::State,
    settings: button::State,
    submenu: Submenu,
    settings_button: button::State,
    save_settings_button: button::State,
    choose_directory_button: button::State,
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
            button_state: Default::default(),
        };
        Ok((data, result))
    }

    async fn download_image(mut save_path: PathBuf, url: &str) -> GuiResult<()> {
        debug!("Downloading from url: {}", url);
        let response = reqwest::get(url).await?;
        if !response.status().is_success() {
            return Err(WallGuiError::BadResponse(response.status().to_string()).into());
        }
        if let Some(file_name) = url.split('/').into_iter().last() {
            // TODO save directory location
            debug!(
                "writing {} bytes to {}",
                &response.content_length().unwrap_or_default(),
                &file_name
            );
            save_path.push(file_name);
            let mut file = File::create(save_path).await?;
            file.write_all(&response.bytes().await?).await?;
        } else {
            error!("Error getting filename of url: {}", url);
        }

        Ok(())
    }

    async fn download_images(save_path: PathBuf, urls: Vec<String>) -> () {
        for url in urls {
            if let Err(e) = WallpaperUi::download_image(save_path.clone(), &url).await {
                error!("{:3?}", e);
            }
        }
    }

    async fn search_command(
        options: SearchOptions,
        directory: PathBuf,
    ) -> GenericResponse<Vec<(ListingData, ImageView)>> {
        match WallpaperUi::do_search(options, directory).await {
            Ok(search) => GenericResponse {
                data: Some(search),
                ..Default::default()
            },
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
    ) -> GuiResult<Vec<(ListingData, ImageView)>> {
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
            return Ok(map);
        }

        Err(
            WallGuiError::BadResponse(response.error.unwrap_or("No error message".to_string()))
                .into(),
        )
    }

    async fn choose_directory() -> Option<PathBuf> {
        let dir_picker = native_dialog::OpenSingleDir { dir: None };

        dir_picker.show().ok().flatten()
    }
}

impl Application for WallpaperUi {
    type Executor = executor::Default;
    type Message = WallpaperMessage;
    type Flags = Option<SavedSettings>;

    fn new(flags: Self::Flags) -> (Self, Command<WallpaperMessage>) {
        let key = flags.clone().unwrap_or_default().api_key.clone();
        (
            Self {
                settings: flags.clone().unwrap_or_default(),
                search_options: SearchOptions {
                    api_key: key.clone(),
                    ..Default::default()
                },
                api_key: key.clone().unwrap_or_default(),
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
            }
            WallpaperMessage::NextPage() => {
                self.search_options.page = Some(self.search_options.page.unwrap_or(1) + 1);
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
            WallpaperMessage::ImageChecked(id) => {
                let image = self.search_results.iter_mut().find(|(l, _)| l.id == id);
                if let Some((_, result_data)) = image {
                    // toggle checked
                    result_data.state = match result_data.state {
                        ImageState::Unselected => ImageState::Selected,
                        ImageState::Selected => ImageState::Unselected,
                        // default return same state
                        _ => result_data.state,
                    }
                }
            }
            WallpaperMessage::DownloadImages() => {
                let image_urls: Vec<_> = self
                    .search_results
                    .iter_mut()
                    .filter(|(_, image)| image.state.eq(&ImageState::Selected))
                    .map(|(listing, image)| {
                        image.state = ImageState::Downloading;
                        listing.path.clone()
                    })
                    .collect();

                return Command::perform(
                    WallpaperUi::download_images(
                        self.settings
                            .save_directory
                            .clone()
                            .unwrap_or_default()
                            .into(),
                        image_urls,
                    ),
                    WallpaperMessage::ImageDownloaded,
                );
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
                self.search_results.clear();
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
                let res_map = self
                    .search_options
                    .resolutions
                    .get_or_insert(HashSet::new());
                if res_map.contains(&resolution) {
                    res_map.remove(&resolution);
                } else {
                    res_map.insert(resolution);
                }
            }
            WallpaperMessage::AspectRatioSelected(aspect_ratio) => {
                let ratio_map = self.search_options.ratios.get_or_insert(HashSet::new());
                if ratio_map.contains(&aspect_ratio) {
                    ratio_map.remove(&aspect_ratio);
                } else {
                    ratio_map.insert(aspect_ratio);
                }
            }
            WallpaperMessage::ImageDownloaded(()) => {
                // TODO implement
            }
            WallpaperMessage::SaveSettings() => {
                self.settings.api_key = self.search_options.api_key.clone();
                return Command::perform(
                    SavedSettings::save_settings(self.settings.clone()),
                    WallpaperMessage::SaveCompleted,
                );
            }
            WallpaperMessage::SaveCompleted(()) => {}
            WallpaperMessage::SetIgnoreDownloaded(value) => {
                self.settings.ignore_downloaded = value;
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        // Build columns of 5 with our images
        let ignore_downloaded = self.settings.ignore_downloaded;

        let images = self
            .search_results
            .as_mut_slice()
            .chunks_mut(5)
            .into_iter()
            .map(|chunk| {
                chunk
                    .iter_mut()
                    .filter(|(_, image)| -> bool {
                        match ignore_downloaded {
                            true => match image.state {
                                ImageState::Downloaded => false,
                                _ => true,
                            },
                            false => true,
                        }
                    })
                    .map(|(listing, image)| {
                        Button::new(
                            &mut image.button_state,
                            Image::new(image.image_handle.clone()),
                        )
                        .style(match image.state {
                            ImageState::Selected => button_style::Button::Select,
                            ImageState::Unselected => button_style::Button::Inactive,
                            ImageState::Downloading => button_style::Button::Downloading,
                            ImageState::Downloaded => button_style::Button::Primary,
                        })
                        .on_press(WallpaperMessage::ImageChecked(listing.id.clone()))
                    })
                    .fold(Row::new().spacing(5), |row, item| row.push(item))
            })
            .fold(
                Column::new()
                    .spacing(5)
                    .push(Text::new("Search results").size(16)),
                |column, row| column.push(row),
            )
            .push(
                make_button(&mut self.controls.next_page_button, "Load More")
                    .on_press(WallpaperMessage::NextPage()),
            )
            .align_items(Align::Center);

        let text_input = Row::new()
            .height(Length::Shrink)
            .width(Length::Fill)
            .push(
                TextInput::new(
                    &mut self.search_state,
                    "Search",
                    &self.search_value,
                    WallpaperMessage::SearchUpdated,
                )
                .size(16)
                .padding(15)
                .on_submit(WallpaperMessage::Search()),
            )
            .push(
                make_button(&mut self.search_button, "search")
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

        let mut nsfw_button =
            make_button(&mut self.controls.nsfw_button, "nsfw").style(inactive_style(purity.nsfw));
        if !self.api_key.is_empty() {
            nsfw_button = nsfw_button.on_press(WallpaperMessage::TogglePurity(PurityOptions::Nsfw));
        }

        let filter_row = Row::new()
            .height(Length::Shrink)
            .width(Length::Shrink)
            //.align_items(Align::Center)
            .push(
                make_button(&mut self.controls.general_button, "general")
                    .on_press(WallpaperMessage::ToggleContentType(ContentTypes::General))
                    .style(inactive_style(search_type.general)),
            )
            .push(
                make_button(&mut self.controls.anime_button, "anime")
                    .on_press(WallpaperMessage::ToggleContentType(ContentTypes::Anime))
                    .style(inactive_style(search_type.anime)),
            )
            .push(
                make_button(&mut self.controls.people_button, "people")
                    .on_press(WallpaperMessage::ToggleContentType(ContentTypes::People))
                    .style(inactive_style(search_type.people)),
            )
            .push(Space::new(Length::FillPortion(5), Length::Shrink))
            .push(
                make_button(&mut self.controls.sfw_button, "clean")
                    .on_press(WallpaperMessage::TogglePurity(PurityOptions::Sfw))
                    .style(inactive_style(purity.clean)),
            )
            .push(
                make_button(&mut self.controls.sketchy_button, "sketchy")
                    .on_press(WallpaperMessage::TogglePurity(PurityOptions::Sketchy))
                    .style(inactive_style(purity.sketchy)),
            )
            .push(nsfw_button)
            .push(
                PickList::new(
                    &mut self.controls.sorting_picker,
                    &Sorting::LIST[..],
                    self.search_options.sorting.clone(),
                    WallpaperMessage::SortingTypeChanged,
                )
                .text_size(26)
                .width(Length::Shrink)
                .padding(5),
            )
            .push(
                make_button(&mut self.resolutions_menu_button, "resolutions")
                    .on_press(WallpaperMessage::ChangeSubmenu(Submenu::Resolution)),
            )
            .push(
                make_button(&mut self.aspect_menu_button, "aspect ratio")
                    .on_press(WallpaperMessage::ChangeSubmenu(Submenu::AspectRatio)),
            )
            .push(Space::new(Length::FillPortion(5), Length::Shrink))
            .push(
                make_button(&mut self.controls.settings_button, "settings")
                    .on_press(WallpaperMessage::ChangeSubmenu(Submenu::Settings)),
            )
            .push(
                make_button(&mut self.controls.download_button, "download")
                    .on_press(WallpaperMessage::DownloadImages()),
            );

        let settings_row = Row::new()
            .align_items(Align::Center)
            .push(
                Column::new()
                    .width(Length::FillPortion(4))
                    .push(Text::new("wallhaven.cc api token (required for nsfw):"))
                    .push(TextInput::new(
                        &mut self.api_text_input,
                        "api key",
                        &*self.api_key,
                        WallpaperMessage::ApiTokenSet,
                    )),
            )
            .push(
                Row::new()
                    .width(Length::FillPortion(4))
                    .push(
                        Column::new()
                            .push(Text::new("save directory:"))
                            .push(Text::new(
                                self.settings
                                    .save_directory
                                    .as_ref()
                                    .unwrap_or(&"./".to_string()),
                            )),
                    )
                    .push(
                        make_button(
                            &mut self.controls.choose_directory_button,
                            "Choose Directory",
                        )
                        .on_press(WallpaperMessage::ChooseDirectory()),
                    ),
            )
            .push(Checkbox::new(
                self.settings.ignore_downloaded,
                "Ignore downloaded",
                WallpaperMessage::SetIgnoreDownloaded,
            ))
            .push(
                make_button(&mut self.controls.save_settings_button, "save settings")
                    .on_press(WallpaperMessage::SaveSettings())
                    .width(Length::FillPortion(2)),
            );

        let column = Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .align_items(Align::Center)
            .spacing(10)
            .push(filter_row)
            .push(match self.controls.submenu {
                Submenu::Settings => settings_row,
                Submenu::Resolution => self
                    .resolution_menu
                    .build_resolution_row(&self.search_options.resolutions),
                Submenu::AspectRatio => self
                    .aspect_menu
                    .build_ratio_row(&self.search_options.ratios), // todo implement
                Submenu::None => Row::new(),
            })
            .push(text_input)
            .push(
                Scrollable::new(&mut self.scroll_state)
                    .push(images)
                    .height(Length::Fill),
            );
        //.push(page_button_row);

        Container::new(column)
            .padding(15)
            .align_y(Align::Start)
            .center_x()
            .into()
    }
}
