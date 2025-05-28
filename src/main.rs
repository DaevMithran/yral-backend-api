use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use types::{
    AppState, GetVideosQuery, GetVideosResponse, UploadVideoRequest, Video, VideoResponse,
};
use uuid::Uuid;

mod types;

impl AppState {
    async fn increment_likes(&self, video_id: &str) -> Result<u64, &'static str> {
        // Find video
        let video = {
            let videos = self.videos.read().unwrap();
            videos.get(video_id).cloned()
        };

        match video {
            Some(video) => {
                // Handle race condition
                let new_likes = video.likes.fetch_add(1, Ordering::Relaxed) + 1;

                Ok(new_likes)
            }
            None => Err("Video not found"),
        }
    }

    async fn increment_views(&self, video_id: &str) -> Result<u64, &'static str> {
        let video = {
            let videos = self.videos.read().unwrap();
            videos.get(video_id).cloned()
        };

        match video {
            Some(video) => {
                let new_views = video.views.fetch_add(1, Ordering::Relaxed) + 1;

                Ok(new_views)
            }
            None => Err("Video not found"),
        }
    }
}

// functions
async fn get_videos(
    Query(params): Query<GetVideosQuery>,
    State(state): State<AppState>,
) -> Result<Json<GetVideosResponse>, StatusCode> {
    let page = params.page.unwrap_or(0);
    let limit = params.limit.unwrap_or(10).min(50);
    let sort = params.sort.unwrap_or_else(|| "trending".to_string());

    let videos = state.videos.read().unwrap();
    let mut video_list: Vec<VideoResponse> = videos.values().map(VideoResponse::from).collect();

    // Sort
    match sort.as_str() {
        "recent" => video_list.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        "popular" => video_list.sort_by(|a, b| b.likes.cmp(&a.likes)),
        _ => todo!(),
    }

    // Pagination
    let total_videos = video_list.len();
    let total_pages = (total_videos + limit - 1) / limit;

    let start = page * limit;
    let end = (start + limit).min(total_videos);

    let paginated_videos = if start < total_videos {
        video_list[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(GetVideosResponse {
        videos: paginated_videos,
        page,
        total_pages,
        total_videos,
    }))
}

async fn get_video(
    Path(video_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<VideoResponse>, StatusCode> {
    let _ = state.increment_views(&video_id).await;

    let videos = state.videos.read().unwrap();
    match videos.get(&video_id) {
        Some(video) => Ok(Json(VideoResponse::from(video))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_video(
    State(state): State<AppState>,
    Json(payload): Json<UploadVideoRequest>,
) -> Result<Json<VideoResponse>, StatusCode> {
    let video_id = Uuid::new_v4().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let video = Video {
        id: video_id.clone(),
        title: payload.title,
        creator: payload.creator,
        description: payload.description,
        url: payload.url,
        created_at: now,
        likes: Arc::new(AtomicU64::new(0)),
        views: Arc::new(AtomicU64::new(0)),
    };

    let response = VideoResponse::from(&video);

    // Safe write
    {
        let mut videos = state.videos.write().unwrap();
        videos.insert(video_id.clone(), video);
    }

    Ok(Json(response))
}
async fn like_video(
    Path(video_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.increment_likes(&video_id).await {
        Ok(likes) => Ok(Json(serde_json::json!({
            "video_id": video_id,
            "likes": likes,
            "message": "Video liked successfully"
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[tokio::main]
async fn main() {
    let state = types::AppState::new();

    // router
    let app = Router::new()
        .route("/api/videos", get(get_videos).post(create_video))
        .route("/api/videos/:id", get(get_video))
        .route("/api/videos/:id/like", post(like_video))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
