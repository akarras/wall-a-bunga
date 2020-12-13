use crate::{WHResult, WallhavenApiClientError};
/// Types used to serialize/deserialize from the http://wallhaven.cc API
/// Derived directly from https://wallhaven.cc/help/api
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::convert::TryFrom;
use std::fmt::Display;
use serde::export::Formatter;
use std::fmt;
use serde::de::Visitor;
use serde_with::*;
use serde_as;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Purity {
    pub clean: bool,
    pub sketchy: bool,
    /// Requires API token
    pub nsfw: bool,
}

impl Default for Purity {
    fn default() -> Self {
        Self {
            clean: true,
            sketchy: false,
            nsfw: false
        }
    }
}

impl Serialize for Purity {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let str: String = self.into();
        serializer.serialize_str(str.as_str())
    }
}

impl Serialize for Categories {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let str: String = self.into();
        serializer.serialize_str(str.as_str())
    }
}

fn explicit_char_bool(character: char) -> WHResult<bool> {
    match character {
        '0' => Ok(false),
        '1' => Ok(true),
        _ => Err(WallhavenApiClientError::InvalidContent),
    }
}

impl TryFrom<&str> for Purity {
    type Error = WallhavenApiClientError;

    fn try_from(value: &str) -> WHResult<Self> {
        let chars = &mut value.chars();
        if value.len() < 3 {
            return Err(WallhavenApiClientError::InvalidContent);
        }
        Ok(Purity {
            clean: explicit_char_bool(chars.nth(0).unwrap())?,
            sketchy: explicit_char_bool(chars.nth(0).unwrap())?,
            nsfw: explicit_char_bool(chars.nth(0).unwrap())?,
        })
    }
}

fn bool_to_bit_char(val: bool) -> char {
    match val {
        false => '0',
        true => '1',
    }
}

impl Into<String> for &Purity {
    fn into(self) -> String {
        let mut string = String::with_capacity(3);
        string.push(bool_to_bit_char(self.clean));
        string.push(bool_to_bit_char(self.sketchy));
        string.push(bool_to_bit_char(self.nsfw));
        string
    }
}

#[derive(Debug, Clone)]
pub struct Categories {
    pub general: bool,
    pub anime: bool,
    pub people: bool,
}

impl Default for Categories {
    fn default() -> Self {
        Categories {
            general: true,
            anime: true,
            people: true
        }
    }
}

impl Into<String> for &Categories {
    fn into(self) -> String {
        let mut str = String::with_capacity(3);
        str.push(bool_to_bit_char(self.general));
        str.push(bool_to_bit_char(self.anime));
        str.push(bool_to_bit_char(self.people));
        str
    }
}

impl TryFrom<&str> for Categories {
    type Error = WallhavenApiClientError;

    fn try_from(value: &str) -> WHResult<Self> {
        let mut chars = value.chars();
        if value.len() < 3 {
            return Err(WallhavenApiClientError::InvalidContent);
        }
        Ok(Categories {
            general: explicit_char_bool(chars.next().unwrap())?,
            anime: explicit_char_bool(chars.next().unwrap())?,
            people: explicit_char_bool(chars.next().unwrap())?,
        })
    }
}

#[derive(Serialize, Debug, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Sorting {
    DateAdded,
    Relevance,
    Random,
    Views,
    Favorites,
    TopList,
}

impl Sorting {
    pub const LIST: [Sorting; 6] = [Sorting::DateAdded, Sorting::TopList, Sorting::Relevance, Sorting::Favorites, Sorting::Views, Sorting::Random];
}

impl Display for Sorting {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Sorting::DateAdded => {
                write!(f, "Date Added")
            }
            Sorting::Relevance => {
                write!(f, "Relevance")
            }
            Sorting::Random => {
                write!(f, "Random")
            }
            Sorting::Views => {
                write!(f, "Views")
            }
            Sorting::Favorites => {
                write!(f, "Favorites")
            }
            Sorting::TopList => {
                write!(f, "Top List")
            }
        }
    }
}

impl Default for Sorting {
    fn default() -> Self {
        Sorting::Random
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct XYCombo {
    pub x: i32,
    pub y: i32
}

impl Display for XYCombo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}


pub static RESOLUTION_POSSIBILITIES: [XYCombo; 22] = [XYCombo {
    x: 2560,
    y: 1080
}, XYCombo {
x: 3440,
y: 1440
},
    XYCombo {
        x: 3840,
        y: 1600
    }, XYCombo {
    x: 1280,
    y: 720
}, XYCombo {
    x: 1600,
    y: 900
}, XYCombo {
    x: 2560,
    y: 1440
},
    XYCombo {
        x: 1920,
        y: 1080
    },
    XYCombo {
    x: 3840,
    y: 2160
}, XYCombo {
    x: 1280,
    y: 800
}, XYCombo {
    x: 1600,
    y: 1000
}, XYCombo {
    x: 1920,
    y: 1200
}, XYCombo {
        x: 2560,
        y: 1600
    }, XYCombo {
        x: 3840,
        y: 2400
    }, XYCombo {
        x: 1280,
        y: 960
    },
    XYCombo {
        x: 1600,
        y: 1200
    }, XYCombo {
        x: 1920,
        y: 1440
    }, XYCombo {
        x: 2560,
        y: 1920
    }, XYCombo {
        x: 3840,
        y: 2880
    }, XYCombo {
        x: 1280,
        y: 1024
    }, XYCombo {
        x: 1600,
        y: 1024
    }, XYCombo {
        x: 1920,
        y: 1280
    }, XYCombo {
        x: 2560,
        y: 2048
    }];

pub static ASPECT_RATIOS: [XYCombo; 12] = [XYCombo {
    x: 16,
    y: 9
}, XYCombo {
    x: 16,
    y: 10
}, XYCombo {
    x: 21,
    y: 9
}, XYCombo {
    x: 32,
    y: 9
}, XYCombo {
    x: 48,
    y: 9
}, XYCombo {
    x: 9,
    y: 16
}, XYCombo {
    x: 10,
    y: 16
}, XYCombo {
    x: 9,
    y: 18
}, XYCombo {
    x: 1,
    y: 1
}, XYCombo {
    x: 3,
    y: 2
}, XYCombo {
    x: 4,
    y: 3
}, XYCombo {
    x: 5,
    y: 4
}];

#[derive(Serialize)]
pub enum TopListTimeFilter {
    #[serde(rename = "1d")]
    LastDay,
    #[serde(rename = "3d")]
    LastThreeDays,
    #[serde(rename = "1w")]
    LastWeek,
    #[serde(rename = "1M")]
    LastMonth,
    #[serde(rename = "3M")]
    LastThreeMonths,
    #[serde(rename = "6M")]
    LastSixMonths,
    #[serde(rename = "1y")]
    LastYear
}

impl Serialize for XYCombo {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_str(&format!("{}x{}", self.x, self.y))
    }
}

#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Default, Clone)]
pub struct SearchOptions {
    #[serde(rename = "q")]
    pub query: Option<String>,
    pub page: Option<i32>,
    pub purity: Option<Purity>,
    pub categories: Option<Categories>,
    pub sorting: Option<Sorting>,
    #[serde(rename = "apikey")]
    pub api_key: Option<String>,
    pub seed: Option<String>,
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, XYCombo>>")]
    pub resolutions: Option<HashSet<XYCombo>>,
    #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, XYCombo>>")]
    pub ratios: Option<HashSet<XYCombo>>,
}

impl SearchOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_query(&mut self, query: String) -> &mut SearchOptions {
        self.query = Some(query);
        self
    }

    pub fn set_page(&mut self, page: i32) -> &mut SearchOptions {
        self.page = Some(page);
        self
    }

    pub fn set_purity(&mut self, purity: Purity) -> &mut SearchOptions {
        self.purity = Some(purity);
        self
    }

    pub fn set_type(&mut self, t: Categories) -> &mut SearchOptions {
        self.categories = Some(t);
        self
    }

    pub fn get_resolution_possibilities() -> Vec<XYCombo> {
        RESOLUTION_POSSIBILITIES.to_vec()
    }

    pub fn get_aspect_ratio_possibilities() -> Vec<XYCombo> {
        ASPECT_RATIOS.to_vec()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct GenericResponse<T> {
    /// Returned data
    pub data: Option<T>,
    /// Error message with a response body
    pub error: Option<String>,
    /// Meta data about the message
    pub meta: Option<SearchMetaData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListingData {
    pub id: String,
    pub url: String,
    pub short_url: String,
    pub views: i64,
    pub favorites: i64,
    pub source: String,
    pub purity: String,
    pub category: String,
    pub dimension_x: i64,
    pub dimension_y: i64,
    pub resolution: String,
    pub ratio: String,
    pub file_size: i64,
    pub file_type: String,
    pub created_at: String,
    pub colors: Vec<String>,
    pub path: String,
    pub thumbs: Thumbs,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Thumbs {
    pub large: String,
    pub original: String,
    pub small: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SerializeableInt {
    val: i64
}

pub struct StringOrIntVisitor;

impl<'de> Visitor<'de> for StringOrIntVisitor {
    type Value = i64;

    fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("integer or string")
    }

    fn visit_i64<E>(self, val: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(val)
    }

    fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(val as i64)
    }

    fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        match val.parse::<i64>() {
            Ok(val) => self.visit_i64(val),
            Err(_) => Err(E::custom("failed to parse integer")),
        }
    }
}


fn deserialize_string_or_int<'de, D>(deserialize: D)-> Result<i64, D::Error> where D: Deserializer<'de> {
    deserialize.deserialize_any(StringOrIntVisitor)
}

/*impl<'de> Deserialize<'de> for StringOrInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        if let Ok(str) = deserializer.deserialize_str() {
            if let Ok(val) = str.parse() {
                Ok(val)
            }
        }
        deserializer.deserialize_i64()


    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de>, {
        if let Ok(str) = deserializer.deserialize_str(place) {
            if let Ok(val) = str.parse::<i64>() {
                Ok(val)
            }
        }
        deserializer.deserialize_i64(place)
    }
}*/

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchMetaData {
    pub current_page: i64,
    pub last_page: i64,
    #[serde(deserialize_with="deserialize_string_or_int")]
    pub per_page: i64,
    pub total: i64,
    pub query: Option<String>,
    pub seed: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::SearchOptions;
    use crate::types::{Purity, Categories, Sorting, XYCombo};

    // ensure that the search options query string serializes properly
    #[test]
    fn query_serialize_full_options() {
        let client = reqwest::Client::new();
        let query_options = SearchOptions {
            query: Some("Zero Two".to_string()),
            page: Some(2),
            purity: Some(Purity { // 0 1 1
                clean: false,
                sketchy: true,
                nsfw: true
            }),
            categories: Some(Categories { // expected 0 1 0
                general: false,
                anime: true,
                people: false
            }),
            sorting: Some(Sorting::Views),
            api_key: Some("supersecretapikey".to_string()),
            seed: Some("seedyroots".to_string()),
            resolutions: Some(vec![XYCombo {
                x: 1920,
                y: 1280
            }].into_iter().collect()),
            ratios: Some(vec![XYCombo {
                x: 16,
                y: 9
            }].into_iter().collect())
        };
        let request = client.get("http://test.test/")
            .query(&query_options).build().unwrap();
        assert_eq!(&request.url().to_string(), "http://test.test/?q=Zero+Two&page=2&purity=011&categories=010&sorting=views&apikey=supersecretapikey&seed=seedyroots&resolutions=1920x1280&ratios=16x9");
    }

    #[test]
    fn query_serialize_resolutions() {
        let client = reqwest::Client::new();
        let query_options = SearchOptions {
            query: Some("Zero Two".to_string()),
            resolutions: Some(vec![XYCombo {
                x: 1920,
                y: 1280
            }, XYCombo {
                x: 2550,
                y: 1440
            }].into_iter().collect()),
            ..Default::default()
        };
        let request = client.get("http://test.test/")
            .query(&query_options).build().unwrap();
        assert_eq!(&request.url().to_string(), "http://test.test/?q=Zero+Two&resolutions=1920x1280%2C2550x1440");
    }

    #[test]
    fn query_text_only() {
        let query_options = SearchOptions {
            query: Some("Zero Two".to_string()),
            ..Default::default()
        };

        let client = reqwest::Client::new();
        let request = client.get("http://test.test/")
            .query(&query_options).build().unwrap();
        assert_eq!(&request.url().to_string(), "http://test.test/?q=Zero+Two");
    }
}

