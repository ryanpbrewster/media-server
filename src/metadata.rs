use image::ImageFormat;

pub enum Metadata {
    Image {
        format: ImageFormat,
        blurhash: String,
    },
    Unknown {
        original_media_type: String,
    },
}
