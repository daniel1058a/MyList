use std::{fs, io::Cursor, path::Path};

use base64::{Engine, engine::general_purpose::STANDARD};
use image::{DynamicImage, ImageFormat, ImageReader, Limits};
use serde::Deserialize;
use uuid::Uuid;

const MAX_BYTES: usize = 12 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum CoverError {
    Invalid,
    TooLarge,
    Unavailable,
}

pub struct Cover {
    original: Vec<u8>,
    thumbnail: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum CoverChange {
    Keep,
    Remove,
    Replace { data: String },
}

pub enum PreparedCover {
    Keep,
    Remove,
    Replace(Cover),
}

impl CoverChange {
    pub fn prepare(self) -> Result<PreparedCover, CoverError> {
        self.prepare_with_size(360, 540)
    }

    pub fn prepare_panorama(self) -> Result<PreparedCover, CoverError> {
        self.prepare_with_size(1280, 720)
    }

    fn prepare_with_size(self, width: u32, height: u32) -> Result<PreparedCover, CoverError> {
        Ok(match self {
            Self::Keep => PreparedCover::Keep,
            Self::Remove => PreparedCover::Remove,
            Self::Replace { data } => PreparedCover::Replace(Cover::decode(&data, width, height)?),
        })
    }
}

impl PreparedCover {
    pub fn persist<T, E: From<CoverError>>(
        self,
        data_dir: &Path,
        previous: Option<&str>,
        write: impl FnOnce(Option<&str>) -> Result<T, E>,
    ) -> Result<T, E> {
        let new_key = match &self {
            Self::Replace(cover) => Some(cover.store(data_dir)?),
            _ => None,
        };
        let chosen = match self {
            Self::Keep => previous,
            Self::Remove => None,
            Self::Replace(_) => new_key.as_deref(),
        };
        let result = write(chosen);
        if result.is_ok() {
            if let Some(key) = previous.filter(|_| previous != chosen) {
                remove(data_dir, key);
            }
        } else if let Some(key) = new_key {
            remove(data_dir, &key);
        }
        result
    }
}

impl Cover {
    fn decode(encoded: &str, preview_width: u32, preview_height: u32) -> Result<Self, CoverError> {
        use CoverError::*;
        if encoded.len() > MAX_BYTES.div_ceil(3) * 4 {
            return Err(TooLarge);
        }
        let original = STANDARD.decode(encoded).map_err(|_| Invalid)?;
        if original.len() > MAX_BYTES {
            return Err(TooLarge);
        }
        let format = image::guess_format(&original).map_err(|_| Invalid)?;
        if !matches!(
            format,
            ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP
        ) {
            return Err(Invalid);
        }
        let reader = || ImageReader::with_format(Cursor::new(&original), format);
        let (width, height) = reader().into_dimensions().map_err(|_| Invalid)?;
        if width > 6000 || height > 6000 || u64::from(width) * u64::from(height) > 24_000_000 {
            return Err(TooLarge);
        }
        let mut reader = reader();
        let mut limits = Limits::default();
        limits.max_image_width = Some(6000);
        limits.max_image_height = Some(6000);
        limits.max_alloc = Some(192 * 1024 * 1024);
        reader.limits(limits);
        let image = reader.decode().map_err(|_| Invalid)?;
        let preview = image.thumbnail(preview_width, preview_height).into_rgba8();
        let mut thumbnail = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(preview)
            .write_to(&mut thumbnail, ImageFormat::WebP)
            .map_err(|_| Unavailable)?;
        Ok(Self {
            original,
            thumbnail: thumbnail.into_inner(),
        })
    }

    pub fn store(&self, data_dir: &Path) -> Result<String, CoverError> {
        let key = Uuid::new_v4().to_string();
        let root = data_dir.join("covers");
        let folder = root.join(&key);
        fs::create_dir_all(&root).map_err(|_| CoverError::Unavailable)?;
        managed_root(data_dir).ok_or(CoverError::Unavailable)?;
        fs::create_dir(&folder).map_err(|_| CoverError::Unavailable)?;
        let result = fs::write(folder.join("original"), &self.original)
            .and_then(|()| fs::write(folder.join("thumbnail.webp"), &self.thumbnail));
        if result.is_err() {
            remove(data_dir, &key);
            return Err(CoverError::Unavailable);
        }
        Ok(key)
    }
}

pub(crate) fn valid_key(key: &str) -> bool {
    Uuid::parse_str(key).is_ok_and(|id| id.get_version_num() == 4 && id.to_string() == key)
}

pub fn remove(data_dir: &Path, key: &str) {
    if valid_key(key)
        && let Some(root) = managed_root(data_dir)
    {
        let _ = fs::remove_dir_all(root.join(key));
    }
}

pub(crate) fn managed_root(data_dir: &Path) -> Option<std::path::PathBuf> {
    let root = data_dir.canonicalize().ok()?.join("covers");
    (root.canonicalize().ok()? == root).then_some(root)
}

pub fn thumbnail(data_dir: &Path, key: &str) -> Option<Vec<u8>> {
    if !valid_key(key) {
        return None;
    }
    let path = managed_root(data_dir)?.join(key).join("thumbnail.webp");
    let resolved = path.canonicalize().ok()?;
    if resolved != path || !resolved.is_file() {
        return None;
    }
    fs::read(resolved).ok()
}
