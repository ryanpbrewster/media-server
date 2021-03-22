use image::{GenericImageView, ImageFormat};

#[derive(Debug)]
pub enum Metadata {
    Image {
        format: ImageFormat,
        blurhash: String,
    },
    Unknown {
        original_media_type: Option<String>,
    },
}

impl Metadata {
    pub fn new(raw: &[u8], media_type: Option<String>) -> Metadata {
        fallible_metadata(raw).unwrap_or(Metadata::Unknown {
            original_media_type: media_type,
        })
    }
}

const X_COMPONENTS: u32 = 5;
const Y_COMPONENTS: u32 = 5;
fn fallible_metadata(raw: &[u8]) -> Option<Metadata> {
    if let Ok(image_format) = image::guess_format(raw) {
        let img = image::load_from_memory_with_format(raw, image_format).ok()?;
        let (width, height) = img.dimensions();
        let hash = blurhash::encode(
            X_COMPONENTS,
            Y_COMPONENTS,
            width,
            height,
            &img.into_rgba().into_vec(),
        );
        return Some(Metadata::Image {
            format: image_format,
            blurhash: hash,
        });
    }
    None
}
