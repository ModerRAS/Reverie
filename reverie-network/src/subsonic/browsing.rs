//! 浏览相关端点处理器
//!
//! 实现 Subsonic API 的浏览功能（getIndexes, getMusicDirectory, getGenres 等）

use axum::{
    extract::{Query, State},
    response::Response,
};
use reverie_storage::SubsonicStorage;
use std::collections::HashMap;

use super::{error_response, format_response, SubsonicState};
use super::response::*;

/// GET /rest/getIndexes - 获取艺术家索引
pub async fn get_indexes_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let music_folder_id = params.get("musicFolderId").and_then(|s| s.parse().ok());
    let if_modified_since = params.get("ifModifiedSince").and_then(|s| s.parse().ok());

    match state.storage.get_indexes(music_folder_id, if_modified_since).await {
        Ok(indexes) => {
            let data = build_indexes(&indexes, 0);
            let response = SubsonicResponse::ok_with(ResponseData::Indexes(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getMusicDirectory - 获取目录内容
pub async fn get_music_directory_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.get_music_directory(id).await {
        Ok(Some(dir)) => {
            let data = DirectoryData {
                directory: DirectoryInner::from(&dir),
            };
            let response = SubsonicResponse::ok_with(ResponseData::Directory(data));
            format_response(&params, response)
        }
        Ok(None) => error_response(&params, 70, "Directory not found"),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getGenres - 获取流派列表
pub async fn get_genres_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.get_genres().await {
        Ok(genres) => {
            let items: Vec<GenreItem> = genres.iter().map(GenreItem::from).collect();
            let data = GenresData {
                genres: GenresInner { genre: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::Genres(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getAlbumList - 获取专辑列表（基于文件夹）
pub async fn get_album_list_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let list_type = match params.get("type") {
        Some(t) => t.as_str(),
        None => return error_response(&params, 10, "Missing required parameter: type"),
    };

    let size = params.get("size").and_then(|s| s.parse().ok());
    let offset = params.get("offset").and_then(|s| s.parse().ok());
    let from_year = params.get("fromYear").and_then(|s| s.parse().ok());
    let to_year = params.get("toYear").and_then(|s| s.parse().ok());
    let genre = params.get("genre").map(|s| s.as_str());
    let music_folder_id = params.get("musicFolderId").and_then(|s| s.parse().ok());

    match state
        .storage
        .get_album_list(list_type, size, offset, from_year, to_year, genre, music_folder_id)
        .await
    {
        Ok(albums) => {
            // AlbumList 返回 Child 类型，与 AlbumList2 不同
            let items: Vec<Child> = albums.iter().map(|a| Child {
                id: a.id.clone(),
                parent: a.artist_id.clone(),
                is_dir: true,
                title: a.name.clone(),
                album: Some(a.name.clone()),
                artist: a.artist.clone(),
                track: None,
                year: a.year,
                genre: a.genre.clone(),
                cover_art: a.cover_art.clone(),
                size: None,
                content_type: None,
                suffix: None,
                duration: Some(a.duration as i32),
                bit_rate: None,
                path: None,
                play_count: a.play_count,
                disc_number: None,
                created: a.created.map(|d| d.to_rfc3339()),
                album_id: Some(a.id.clone()),
                artist_id: a.artist_id.clone(),
                starred: a.starred.map(|d| d.to_rfc3339()),
                user_rating: a.user_rating,
                media_type: Some("album".to_string()),
                is_video: false,
            }).collect();
            let data = AlbumListData {
                album_list: AlbumListInner { album: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::AlbumList(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getRandomSongs - 获取随机歌曲
pub async fn get_random_songs_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let size = params.get("size").and_then(|s| s.parse().ok());
    let genre = params.get("genre").map(|s| s.as_str());
    let from_year = params.get("fromYear").and_then(|s| s.parse().ok());
    let to_year = params.get("toYear").and_then(|s| s.parse().ok());
    let music_folder_id = params.get("musicFolderId").and_then(|s| s.parse().ok());

    match state
        .storage
        .get_random_songs(size, genre, from_year, to_year, music_folder_id)
        .await
    {
        Ok(songs) => {
            let items: Vec<Child> = songs.iter().map(Child::from).collect();
            let data = RandomSongsData {
                random_songs: RandomSongsInner { song: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::RandomSongs(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getSongsByGenre - 获取指定流派的歌曲
pub async fn get_songs_by_genre_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let genre = match params.get("genre") {
        Some(g) => g.as_str(),
        None => return error_response(&params, 10, "Missing required parameter: genre"),
    };

    let count = params.get("count").and_then(|s| s.parse().ok());
    let offset = params.get("offset").and_then(|s| s.parse().ok());
    let music_folder_id = params.get("musicFolderId").and_then(|s| s.parse().ok());

    match state
        .storage
        .get_songs_by_genre(genre, count, offset, music_folder_id)
        .await
    {
        Ok(songs) => {
            let items: Vec<Child> = songs.iter().map(Child::from).collect();
            let data = SongsByGenreData {
                songs_by_genre: SongsByGenreInner { song: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::SongsByGenre(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getStarred - 获取收藏内容
pub async fn get_starred_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let music_folder_id = params.get("musicFolderId").and_then(|s| s.parse().ok());

    match state.storage.get_starred(music_folder_id).await {
        Ok(starred) => {
            // 转换 artists
            let artists: Vec<ArtistItem> = starred.artists.iter().map(|a| ArtistItem {
                id: a.id.clone(),
                name: a.name.clone(),
                cover_art: a.cover_art.clone(),
                artist_image_url: None,
                starred: a.starred.map(|d| d.to_rfc3339()),
                user_rating: a.user_rating,
            }).collect();
            
            // 转换 albums 为 Child
            let albums: Vec<Child> = starred.albums.iter().map(|a| Child {
                id: a.id.clone(),
                parent: a.artist_id.clone(),
                is_dir: true,
                title: a.name.clone(),
                album: Some(a.name.clone()),
                artist: a.artist.clone(),
                track: None,
                year: a.year,
                genre: a.genre.clone(),
                cover_art: a.cover_art.clone(),
                size: None,
                content_type: None,
                suffix: None,
                duration: Some(a.duration as i32),
                bit_rate: None,
                path: None,
                play_count: a.play_count,
                disc_number: None,
                created: a.created.map(|d| d.to_rfc3339()),
                album_id: Some(a.id.clone()),
                artist_id: a.artist_id.clone(),
                starred: a.starred.map(|d| d.to_rfc3339()),
                user_rating: a.user_rating,
                media_type: Some("album".to_string()),
                is_video: false,
            }).collect();
            
            // 转换 songs
            let songs: Vec<Child> = starred.songs.iter().map(Child::from).collect();
            
            let data = StarredData {
                starred: StarredInner {
                    artist: artists,
                    album: albums,
                    song: songs,
                },
            };
            let response = SubsonicResponse::ok_with(ResponseData::Starred(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getStarred2 - 获取收藏内容（ID3）
pub async fn get_starred2_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let music_folder_id = params.get("musicFolderId").and_then(|s| s.parse().ok());

    match state.storage.get_starred2(music_folder_id).await {
        Ok(starred) => {
            let artists: Vec<ArtistID3Item> = starred.artists.iter().map(ArtistID3Item::from).collect();
            let albums: Vec<AlbumID3Item> = starred.albums.iter().map(AlbumID3Item::from).collect();
            let songs: Vec<Child> = starred.songs.iter().map(Child::from).collect();
            
            let data = Starred2Data {
                starred2: Starred2Inner {
                    artist: artists,
                    album: albums,
                    song: songs,
                },
            };
            let response = SubsonicResponse::ok_with(ResponseData::Starred2(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getNowPlaying - 获取正在播放
pub async fn get_now_playing_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.get_now_playing().await {
        Ok(entries) => {
            let items: Vec<NowPlayingEntry> = entries.iter().map(NowPlayingEntry::from).collect();
            let data = NowPlayingData {
                now_playing: NowPlayingInner { entry: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::NowPlaying(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}
