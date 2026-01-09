//! Unit tests for reverie-network

#[cfg(test)]
mod error_tests {
    use super::super::*;

    #[test]
    fn test_network_error_server_error() {
        let error = NetworkError::ServerError("Internal server error".to_string());
        assert_eq!(format!("{}", error), "Server error: Internal server error");
    }

    #[test]
    fn test_network_error_connection_error() {
        let error = NetworkError::ConnectionError("Connection refused".to_string());
        assert_eq!(format!("{}", error), "Connection error: Connection refused");
    }

    #[test]
    fn test_network_error_invalid_request() {
        let error = NetworkError::InvalidRequest("Invalid parameter".to_string());
        assert_eq!(format!("{}", error), "Invalid request: Invalid parameter");
    }

    #[test]
    fn test_network_error_authentication_failed() {
        let error = NetworkError::AuthenticationFailed("Invalid credentials".to_string());
        assert_eq!(
            format!("{}", error),
            "Authentication failed: Invalid credentials"
        );
    }

    #[test]
    fn test_network_error_not_found() {
        let error = NetworkError::NotFound("Resource not found".to_string());
        assert_eq!(format!("{}", error), "Not found: Resource not found");
    }

    #[test]
    fn test_network_error_internal() {
        let error = NetworkError::Internal("Unexpected error".to_string());
        assert_eq!(format!("{}", error), "Internal error: Unexpected error");
    }

    #[test]
    fn test_network_error_serialization() {
        let error = NetworkError::SerializationError("JSON error".to_string());
        assert_eq!(format!("{}", error), "Serialization error: JSON error");
    }

    #[test]
    fn test_network_error_debug() {
        let error = NetworkError::ServerError("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ServerError"));
    }
}

#[cfg(test)]
mod dto_tests {
    use super::super::*;
    use uuid::Uuid;

    #[test]
    fn test_track_response_creation() {
        let id = Uuid::new_v4();
        let response = TrackResponse {
            id,
            title: "Test Track".to_string(),
            album_id: Some(Uuid::new_v4()),
            artist_id: Some(Uuid::new_v4()),
            duration: 180,
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Rock".to_string()),
        };

        assert_eq!(response.title, "Test Track");
        assert_eq!(response.duration, 180);
        assert!(response.album_id.is_some());
    }

    #[test]
    fn test_album_response_creation() {
        let id = Uuid::new_v4();
        let response = AlbumResponse {
            id,
            name: "Test Album".to_string(),
            artist_id: Some(Uuid::new_v4()),
            year: Some(2024),
            genre: Some("Pop".to_string()),
        };

        assert_eq!(response.name, "Test Album");
        assert_eq!(response.year, Some(2024));
    }

    #[test]
    fn test_artist_response_creation() {
        let id = Uuid::new_v4();
        let response = ArtistResponse {
            id,
            name: "Test Artist".to_string(),
            bio: Some("A great artist".to_string()),
        };

        assert_eq!(response.name, "Test Artist");
        assert_eq!(response.bio, Some("A great artist".to_string()));
    }

    #[test]
    fn test_playlist_response_creation() {
        let id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let response = PlaylistResponse {
            id,
            name: "My Playlist".to_string(),
            description: Some("A cool playlist".to_string()),
            user_id,
            is_public: true,
        };

        assert_eq!(response.name, "My Playlist");
        assert!(response.is_public);
        assert_eq!(response.user_id, user_id);
    }

    #[test]
    fn test_create_playlist_request_creation() {
        let request = CreatePlaylistRequest {
            name: "New Playlist".to_string(),
            description: Some("Description".to_string()),
            is_public: false,
        };

        assert_eq!(request.name, "New Playlist");
        assert!(!request.is_public);
    }

    #[test]
    fn test_add_track_to_playlist_request_creation() {
        let track_id = Uuid::new_v4();
        let request = AddTrackToPlaylistRequest { track_id };

        assert_eq!(request.track_id, track_id);
    }

    #[test]
    fn test_list_response_creation() {
        let response = ListResponse::<TrackResponse> {
            items: vec![],
            total: 0,
            limit: 10,
            offset: 0,
        };

        assert_eq!(response.total, 0);
        assert_eq!(response.limit, 10);
        assert!(response.items.is_empty());
    }

    #[test]
    fn test_list_response_with_items() {
        let items = vec![
            TrackResponse {
                id: Uuid::new_v4(),
                title: "Track 1".to_string(),
                album_id: None,
                artist_id: None,
                duration: 180,
                track_number: None,
                disc_number: None,
                year: None,
                genre: None,
            },
            TrackResponse {
                id: Uuid::new_v4(),
                title: "Track 2".to_string(),
                album_id: None,
                artist_id: None,
                duration: 200,
                track_number: None,
                disc_number: None,
                year: None,
                genre: None,
            },
        ];

        let response = ListResponse {
            items: items.clone(),
            total: 2,
            limit: 10,
            offset: 0,
        };

        assert_eq!(response.items.len(), 2);
        assert_eq!(response.total, 2);
    }

    #[test]
    fn test_error_response_creation() {
        let response = ErrorResponse {
            error: "NOT_FOUND".to_string(),
            message: "Resource not found".to_string(),
        };

        assert_eq!(response.error, "NOT_FOUND");
        assert_eq!(response.message, "Resource not found");
    }

    #[test]
    fn test_health_response_creation() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
        };

        assert_eq!(response.status, "healthy");
        assert_eq!(response.version, "1.0.0");
    }

    #[test]
    fn test_dto_serialization() {
        let response = TrackResponse {
            id: Uuid::new_v4(),
            title: "Test Track".to_string(),
            album_id: None,
            artist_id: None,
            duration: 180,
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Rock".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Test Track"));
        assert!(json.contains("180"));
    }

    #[test]
    fn test_dto_deserialization() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "title": "Test Track",
            "album_id": null,
            "artist_id": null,
            "duration": 180,
            "track_number": 1,
            "disc_number": 1,
            "year": 2024,
            "genre": "Rock"
        }"#;

        let response: TrackResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.title, "Test Track");
        assert_eq!(response.duration, 180);
    }
}
