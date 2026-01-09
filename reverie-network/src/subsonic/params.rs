use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct CommonParams {
    pub id: Option<String>,
    pub if_modified_since: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumListParams {
    pub list_type: String,
    pub size: Option<i32>,
    pub offset: Option<i32>,
    pub from_year: Option<i32>,
    pub to_year: Option<i32>,
    pub genre: Option<String>,
    pub music_folder_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RandomSongsParams {
    pub size: Option<i32>,
    pub genre: Option<String>,
    pub from_year: Option<i32>,
    pub to_year: Option<i32>,
    pub music_folder_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SongsByGenreParams {
    pub genre: String,
    pub count: Option<i32>,
    pub offset: Option<i32>,
    pub music_folder_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub artist_count: Option<i32>,
    pub artist_offset: Option<i32>,
    pub album_count: Option<i32>,
    pub album_offset: Option<i32>,
    pub song_count: Option<i32>,
    pub song_offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistParams {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub public: Option<bool>,
    pub comment: Option<String>,
    pub playlist_id: Option<String>,
    pub song_ids: Option<String>,
    pub song_ids_to_add: Option<String>,
    pub song_indexes_to_remove: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StreamParams {
    pub id: String,
    pub format: Option<String>,
    pub bitrate: Option<i32>,
    pub max_bit_rate: Option<i32>,
    pub time_offset: Option<i32>,
    pub size: Option<i32>,
    pub estimate_content_length: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CoverArtParams {
    pub id: String,
    pub size: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct LyricsParams {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AnnotationParams {
    pub id: Option<String>,
    pub album_id: Option<String>,
    pub artist_id: Option<String>,
    pub rating: Option<i32>,
    pub time: Option<i64>,
    pub submission: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkParams {
    pub id: String,
    pub position: i64,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlayQueueParams {
    pub ids: Option<String>,
    pub current: Option<String>,
    pub position: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ShareParams {
    pub id: Option<String>,
    pub ids: Option<String>,
    pub description: Option<String>,
    pub expires: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RadioParams {
    pub id: Option<String>,
    pub stream_url: Option<String>,
    pub name: Option<String>,
    pub homepage_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScanParams {
    pub scan_count: Option<i32>,
}

pub fn parse_query_params(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].to_string();
            let value = urlencoding::decode(parts[1])
                .unwrap_or_else(|_| parts[1].to_string())
                .into_owned();
            params.insert(key, value);
        }
    }

    params
}

pub fn get_id(params: &HashMap<String, String>) -> Option<&String> {
    params.get("id")
}

pub fn get_bool(param: Option<&String>) -> bool {
    param.map(|s| s == "true" || s == "1").unwrap_or(false)
}

pub fn split_ids(param: Option<&String>) -> Vec<&str> {
    param.map(|s| s.split(',').collect()).unwrap_or_default()
}

pub fn parse_i32(param: Option<&String>) -> Option<i32> {
    param.and_then(|s| s.parse().ok())
}

pub fn parse_i64(param: Option<&String>) -> Option<i64> {
    param.and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_params() {
        let query = "u=admin&p=password&v=1.16.1&c=test";
        let params = parse_query_params(query);

        assert_eq!(params.get("u"), Some(&"admin".to_string()));
        assert_eq!(params.get("p"), Some(&"password".to_string()));
        assert_eq!(params.get("v"), Some(&"1.16.1".to_string()));
        assert_eq!(params.get("c"), Some(&"test".to_string()));
    }

    #[test]
    fn test_parse_query_params_with_encoded_values() {
        let query = "query=hello%20world&artist=AC%2FDC";
        let params = parse_query_params(query);

        assert_eq!(params.get("query"), Some(&"hello world".to_string()));
        assert_eq!(params.get("artist"), Some(&"AC/DC".to_string()));
    }

    #[test]
    fn test_parse_query_params_empty_query() {
        let params = parse_query_params("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_query_params_single_param() {
        let params = parse_query_params("id=123");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_parse_query_params_with_equals_in_value() {
        let params = parse_query_params("desc=test=value&other=foo");
        assert_eq!(params.get("desc"), Some(&"test=value".to_string()));
        assert_eq!(params.get("other"), Some(&"foo".to_string()));
    }

    #[test]
    fn test_split_ids() {
        let ids = Some("id1,id2,id3".to_string());
        let result = split_ids(ids.as_ref());
        assert_eq!(result, vec!["id1", "id2", "id3"]);
    }

    #[test]
    fn test_split_ids_empty() {
        let ids = Some("".to_string());
        let result = split_ids(ids.as_ref());
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_ids_single() {
        let ids = Some("single".to_string());
        let result = split_ids(ids.as_ref());
        assert_eq!(result, vec!["single"]);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(get_bool(Some(&"true".to_string())), true);
        assert_eq!(get_bool(Some(&"1".to_string())), true);
        assert_eq!(get_bool(Some(&"false".to_string())), false);
        assert_eq!(get_bool(Some(&"0".to_string())), false);
        assert_eq!(get_bool(None), false);
    }

    #[test]
    fn test_parse_i32() {
        assert_eq!(parse_i32(Some(&"42".to_string())), Some(42));
        assert_eq!(parse_i32(Some(&"-10".to_string())), Some(-10));
        assert_eq!(parse_i32(Some(&"abc".to_string())), None);
        assert_eq!(parse_i32(None), None);
    }

    #[test]
    fn test_parse_i32_overflow() {
        let large = "99999999999999999999";
        assert_eq!(parse_i32(Some(&large.to_string())), None);
    }

    #[test]
    fn test_parse_i64() {
        assert_eq!(parse_i64(Some(&"42".to_string())), Some(42));
        assert_eq!(parse_i64(Some(&"-10".to_string())), Some(-10));
        assert_eq!(parse_i64(Some(&"abc".to_string())), None);
        assert_eq!(parse_i64(None), None);
    }

    #[test]
    fn test_parse_i64_large_value() {
        let large = "9999999999999999999";
        assert_eq!(parse_i64(Some(&large.to_string())), Some(9999999999999999999));
    }

    #[test]
    fn test_get_id() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "test123".to_string());
        assert_eq!(get_id(&params), Some(&"test123".to_string()));
    }

    #[test]
    fn test_get_id_missing() {
        let params = HashMap::new();
        assert_eq!(get_id(&params), None);
    }
}
