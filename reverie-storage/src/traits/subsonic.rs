//! Subsonic API 存储 trait
//!
//! 完整的 Subsonic API 存储 trait，实现 navidrome 兼容的 Subsonic API 所需的所有方法。

use crate::error::Result;
use async_trait::async_trait;
use reverie_core::{
    MediaFile, SubsonicAlbum, SubsonicAlbumInfo, SubsonicArtist, SubsonicArtistIndexes,
    SubsonicArtistInfo, SubsonicBookmark, SubsonicDirectory, SubsonicGenre,
    SubsonicInternetRadioStation, SubsonicLyrics, SubsonicMusicFolder, SubsonicNowPlaying,
    SubsonicOpenSubsonicExtension, SubsonicPlayQueue, SubsonicPlaylist, SubsonicPlaylistWithSongs,
    SubsonicScanStatus, SubsonicSearchResult2, SubsonicSearchResult3, SubsonicShare,
    SubsonicStarred, SubsonicStructuredLyrics, SubsonicTopSongs, SubsonicUser,
};

/// 完整的 Subsonic API 存储 trait
/// 实现 navidrome 兼容的 Subsonic API 所需的所有方法
#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait SubsonicStorage: Send + Sync {
    // === 系统 ===
    /// 获取服务器许可证信息（自托管始终有效）
    async fn get_license(&self) -> Result<bool> {
        Ok(true)
    }

    // === 浏览 ===
    /// 获取所有配置的音乐文件夹
    async fn get_music_folders(&self) -> Result<Vec<SubsonicMusicFolder>>;

    /// 获取艺术家索引（A-Z 分组的艺术家）
    async fn get_indexes(
        &self,
        music_folder_id: Option<i32>,
        if_modified_since: Option<i64>,
    ) -> Result<SubsonicArtistIndexes>;

    /// 获取所有流派及歌曲/专辑数量
    async fn get_genres(&self) -> Result<Vec<SubsonicGenre>>;

    /// 获取目录内容（用于基于文件夹的浏览）
    async fn get_music_directory(&self, id: &str) -> Result<Option<SubsonicDirectory>>;

    /// 获取艺术家（基于 ID3 标签）
    async fn get_artists(&self, music_folder_id: Option<i32>) -> Result<SubsonicArtistIndexes>;

    /// 通过 ID 获取单个艺术家
    async fn get_artist(&self, id: &str) -> Result<Option<SubsonicArtist>>;

    /// 通过 ID 获取单个专辑
    async fn get_album(&self, id: &str) -> Result<Option<SubsonicAlbum>>;

    /// 通过 ID 获取单个歌曲
    async fn get_song(&self, id: &str) -> Result<Option<MediaFile>>;

    /// 获取视频（未实现，返回空）
    async fn get_videos(&self) -> Result<Vec<MediaFile>> {
        Ok(vec![])
    }

    /// 获取艺术家信息（简介、图片、相似艺术家）
    async fn get_artist_info(
        &self,
        id: &str,
        count: Option<i32>,
        include_not_present: Option<bool>,
    ) -> Result<SubsonicArtistInfo>;

    /// 获取艺术家信息（ID3 版本）
    async fn get_artist_info2(
        &self,
        id: &str,
        count: Option<i32>,
        include_not_present: Option<bool>,
    ) -> Result<SubsonicArtistInfo>;

    /// 获取专辑信息（备注、图片）
    async fn get_album_info(&self, id: &str) -> Result<SubsonicAlbumInfo>;

    /// 获取专辑信息（ID3 版本）
    async fn get_album_info2(&self, id: &str) -> Result<SubsonicAlbumInfo>;

    /// 获取相似歌曲
    async fn get_similar_songs(&self, id: &str, count: Option<i32>) -> Result<Vec<MediaFile>>;

    /// 获取相似歌曲（ID3 版本）
    async fn get_similar_songs2(&self, id: &str, count: Option<i32>) -> Result<Vec<MediaFile>>;

    /// 获取艺术家的热门歌曲
    async fn get_top_songs(&self, artist: &str, count: Option<i32>) -> Result<SubsonicTopSongs>;

    // === 专辑/歌曲列表 ===
    /// 获取专辑列表（多种排序类型）
    async fn get_album_list(
        &self,
        list_type: &str,
        size: Option<i32>,
        offset: Option<i32>,
        from_year: Option<i32>,
        to_year: Option<i32>,
        genre: Option<&str>,
        music_folder_id: Option<i32>,
    ) -> Result<Vec<SubsonicAlbum>>;

    /// 获取专辑列表（ID3 版本）
    async fn get_album_list2(
        &self,
        list_type: &str,
        size: Option<i32>,
        offset: Option<i32>,
        from_year: Option<i32>,
        to_year: Option<i32>,
        genre: Option<&str>,
        music_folder_id: Option<i32>,
    ) -> Result<Vec<SubsonicAlbum>>;

    /// 获取随机歌曲
    async fn get_random_songs(
        &self,
        size: Option<i32>,
        genre: Option<&str>,
        from_year: Option<i32>,
        to_year: Option<i32>,
        music_folder_id: Option<i32>,
    ) -> Result<Vec<MediaFile>>;

    /// 获取按流派的歌曲
    async fn get_songs_by_genre(
        &self,
        genre: &str,
        count: Option<i32>,
        offset: Option<i32>,
        music_folder_id: Option<i32>,
    ) -> Result<Vec<MediaFile>>;

    /// 获取正在播放的条目
    async fn get_now_playing(&self) -> Result<Vec<SubsonicNowPlaying>>;

    /// 获取收藏的项目
    async fn get_starred(&self, music_folder_id: Option<i32>) -> Result<SubsonicStarred>;

    /// 获取收藏的项目（ID3 版本）
    async fn get_starred2(&self, music_folder_id: Option<i32>) -> Result<SubsonicStarred>;

    // === 搜索 ===
    /// 搜索（已废弃，使用 search2/search3）
    async fn search(
        &self,
        artist: Option<&str>,
        album: Option<&str>,
        title: Option<&str>,
        any: Option<&str>,
        _count: Option<i32>,
        _offset: Option<i32>,
        _newer_than: Option<i64>,
    ) -> Result<SubsonicSearchResult2> {
        // 使用 search2 的默认实现
        let query = any.or(title).or(album).or(artist).unwrap_or("");
        self.search2(query, None, None, None, None, None, None)
            .await
    }

    /// Search2（基于文件夹）
    async fn search2(
        &self,
        query: &str,
        artist_count: Option<i32>,
        artist_offset: Option<i32>,
        album_count: Option<i32>,
        album_offset: Option<i32>,
        song_count: Option<i32>,
        song_offset: Option<i32>,
    ) -> Result<SubsonicSearchResult2>;

    /// Search3（基于 ID3）
    async fn search3(
        &self,
        query: &str,
        artist_count: Option<i32>,
        artist_offset: Option<i32>,
        album_count: Option<i32>,
        album_offset: Option<i32>,
        song_count: Option<i32>,
        song_offset: Option<i32>,
    ) -> Result<SubsonicSearchResult3>;

    // === 播放列表 ===
    /// 获取所有播放列表
    async fn get_playlists(&self, username: Option<&str>) -> Result<Vec<SubsonicPlaylist>>;

    /// 获取包含歌曲的单个播放列表
    async fn get_playlist(&self, id: &str) -> Result<Option<SubsonicPlaylistWithSongs>>;

    /// 创建播放列表
    async fn create_playlist(
        &self,
        name: Option<&str>,
        playlist_id: Option<&str>,
        song_ids: &[&str],
    ) -> Result<SubsonicPlaylistWithSongs>;

    /// 更新播放列表
    async fn update_playlist(
        &self,
        playlist_id: &str,
        name: Option<&str>,
        comment: Option<&str>,
        public: Option<bool>,
        song_ids_to_add: &[&str],
        song_indexes_to_remove: &[i32],
    ) -> Result<()>;

    /// 删除播放列表
    async fn delete_playlist(&self, id: &str) -> Result<()>;

    // === 媒体检索（仅路径，实际流媒体由网络层处理） ===
    /// 获取流媒体文件路径
    async fn get_stream_path(&self, id: &str) -> Result<Option<String>>;

    /// 获取封面图片路径
    async fn get_cover_art_path(&self, id: &str) -> Result<Option<String>>;

    /// 获取歌词
    async fn get_lyrics(
        &self,
        artist: Option<&str>,
        title: Option<&str>,
    ) -> Result<Option<SubsonicLyrics>>;

    /// 通过歌曲 ID 获取歌词（OpenSubsonic）
    async fn get_lyrics_by_song_id(&self, id: &str) -> Result<Vec<SubsonicStructuredLyrics>>;

    /// 获取用户头像路径
    async fn get_avatar_path(&self, username: &str) -> Result<Option<String>>;

    // === 媒体标注 ===
    /// 收藏项目（添加到收藏夹）
    async fn star(&self, ids: &[&str], album_ids: &[&str], artist_ids: &[&str]) -> Result<()>;

    /// 取消收藏（从收藏夹移除）
    async fn unstar(&self, ids: &[&str], album_ids: &[&str], artist_ids: &[&str]) -> Result<()>;

    /// 设置评分（0-5）
    async fn set_rating(&self, id: &str, rating: i32) -> Result<()>;

    /// 记录播放（Scrobble）
    async fn scrobble(&self, id: &str, time: Option<i64>, submission: bool) -> Result<()>;

    // === 书签 ===
    /// 获取用户的所有书签
    async fn get_bookmarks(&self) -> Result<Vec<SubsonicBookmark>>;

    /// 创建/更新书签
    async fn create_bookmark(&self, id: &str, position: i64, comment: Option<&str>) -> Result<()>;

    /// 删除书签
    async fn delete_bookmark(&self, id: &str) -> Result<()>;

    /// 获取播放队列
    async fn get_play_queue(&self) -> Result<Option<SubsonicPlayQueue>>;

    /// 保存播放队列
    async fn save_play_queue(
        &self,
        ids: &[&str],
        current: Option<&str>,
        position: Option<i64>,
    ) -> Result<()>;

    // === 分享 ===
    /// 获取所有分享
    async fn get_shares(&self) -> Result<Vec<SubsonicShare>>;

    /// 创建分享
    async fn create_share(
        &self,
        ids: &[&str],
        description: Option<&str>,
        expires: Option<i64>,
    ) -> Result<SubsonicShare>;

    /// 更新分享
    async fn update_share(
        &self,
        id: &str,
        description: Option<&str>,
        expires: Option<i64>,
    ) -> Result<()>;

    /// 删除分享
    async fn delete_share(&self, id: &str) -> Result<()>;

    // === 网络电台 ===
    /// 获取所有网络电台
    async fn get_internet_radio_stations(&self) -> Result<Vec<SubsonicInternetRadioStation>>;

    /// 创建网络电台
    async fn create_internet_radio_station(
        &self,
        stream_url: &str,
        name: &str,
        homepage_url: Option<&str>,
    ) -> Result<()>;

    /// 更新网络电台
    async fn update_internet_radio_station(
        &self,
        id: &str,
        stream_url: &str,
        name: &str,
        homepage_url: Option<&str>,
    ) -> Result<()>;

    /// 删除网络电台
    async fn delete_internet_radio_station(&self, id: &str) -> Result<()>;

    // === 用户管理 ===
    /// 通过用户名获取用户
    async fn get_user(&self, username: &str) -> Result<Option<SubsonicUser>>;

    /// 获取所有用户
    async fn get_users(&self) -> Result<Vec<SubsonicUser>>;

    /// 创建用户
    async fn create_user(
        &self,
        username: &str,
        password: &str,
        email: Option<&str>,
        admin_role: bool,
        settings_role: bool,
        stream_role: bool,
        jukebox_role: bool,
        download_role: bool,
        upload_role: bool,
        playlist_role: bool,
        cover_art_role: bool,
        comment_role: bool,
        podcast_role: bool,
        share_role: bool,
        video_conversion_role: bool,
        music_folder_ids: &[i32],
    ) -> Result<()>;

    /// 更新用户
    async fn update_user(
        &self,
        username: &str,
        password: Option<&str>,
        email: Option<&str>,
        admin_role: Option<bool>,
        settings_role: Option<bool>,
        stream_role: Option<bool>,
        jukebox_role: Option<bool>,
        download_role: Option<bool>,
        upload_role: Option<bool>,
        playlist_role: Option<bool>,
        cover_art_role: Option<bool>,
        comment_role: Option<bool>,
        podcast_role: Option<bool>,
        share_role: Option<bool>,
        video_conversion_role: Option<bool>,
        music_folder_ids: Option<&[i32]>,
        max_bit_rate: Option<i32>,
    ) -> Result<()>;

    /// 删除用户
    async fn delete_user(&self, username: &str) -> Result<()>;

    /// 更改密码
    async fn change_password(&self, username: &str, password: &str) -> Result<()>;

    // === 库扫描 ===
    /// 获取扫描状态
    async fn get_scan_status(&self) -> Result<SubsonicScanStatus>;

    /// 开始扫描库
    async fn start_scan(&self) -> Result<SubsonicScanStatus>;

    // === OpenSubsonic 扩展 ===
    /// 获取支持的 OpenSubsonic 扩展
    async fn get_open_subsonic_extensions(&self) -> Result<Vec<SubsonicOpenSubsonicExtension>> {
        Ok(vec![
            SubsonicOpenSubsonicExtension {
                name: "transcodeOffset".to_string(),
                versions: vec![1],
            },
            SubsonicOpenSubsonicExtension {
                name: "formPost".to_string(),
                versions: vec![1],
            },
            SubsonicOpenSubsonicExtension {
                name: "songLyrics".to_string(),
                versions: vec![1],
            },
        ])
    }
}
