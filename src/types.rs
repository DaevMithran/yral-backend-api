use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub description: String,
    pub creator: String,
    pub url: String,
    pub created_at: u64,

    #[serde(skip)]
    pub likes: Arc<AtomicU64>,
    #[serde(skip)]
    pub views: Arc<AtomicU64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct VideoResponse {
    id: String,
    title: String,
    creator: String,
    video_url: String,
    pub created_at: u64,
    pub likes: u64,
    views: u64,
}

impl From<&Video> for VideoResponse {
    fn from(video: &Video) -> Self {
        let likes = video.likes.load(Ordering::Relaxed);
        let views = video.views.load(Ordering::Relaxed);

        VideoResponse {
            id: video.id.clone(),
            title: video.title.clone(),
            creator: video.creator.clone(),
            video_url: video.url.clone(),
            created_at: video.created_at,
            likes,
            views,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub videos: Arc<RwLock<HashMap<String, Video>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            // Arc  -> shared ownersip across async tasks
            // Rwlock -> Many readers or one writer
            videos: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UploadVideoRequest {
    pub title: String,
    pub creator: String,
    pub description: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct GetVideosResponse {
    pub videos: Vec<VideoResponse>,
    pub page: usize,
    pub total_pages: usize,
    pub total_videos: usize,
}

#[derive(Debug, Deserialize)]
pub struct GetVideosQuery {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub sort: Option<String>,
}

// TODO: implement notifications
enum NotifyMessage {
    Like { video_id: String, likes: u64 },
    View { video_id: String, views: u64 },
    Comment { video_id: String, comments: u64 },
    NewVideo { video: Video },
}
