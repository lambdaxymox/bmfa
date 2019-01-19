use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;
use image::png;
use image::{ColorType, ImageDecoder};


///
/// A `GlyphMetadata` struct stores the parameters necessary to represent
/// the glyph in a bitmap font atlas.
///
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlyphMetadata {
    /// The unicode code point.
    pub code_point: usize,
    /// The row of the atlas the glyph is stored in.
    pub row: usize,
    /// The column og the atlas the glyph is stored in.
    pub column: usize,
    /// The minimum offset of the glyph into the slot from the bounding box.
    pub x_min: f32,
    /// The width of the glyph, stored in [0,1].
    pub width: f32,
    /// The height of the glyph, represented in the interval [0,1].
    pub height: f32,
    /// The maximum depth of the glyph that falls below the baseline for the font.
    pub y_min: f32,
    pub y_offset: f32,
}

impl GlyphMetadata {
    pub fn new(
        code_point: usize, row: usize, column: usize,
        width: f32, height: f32,
        x_min: f32, y_min: f32, y_offset: f32) -> GlyphMetadata {

        GlyphMetadata {
            code_point: code_point,
            row: row,
            column: column,
            width: width,
            height: height,
            x_min: x_min,
            y_min: y_min,
            y_offset: y_offset,
        }
    }
}

///
/// The `Origin` parameter determines which part of the underlying font atlas image is considered
/// the origin of the image. That is, when trying to render the font atlas in a graphics application,
/// this parameter tells the BMFA parser how to format the atlas image for rendering.
///
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Origin {
    /// The atlas image starts in the top left corner of the image, with the x-axis pointing right,
    /// and the y-axis pointing down.
    TopLeft,
    /// The atlas image starts in the bottom right corner of the image, with the x-axis pointing right,
    /// and the y-axis pointing up.
    BottomLeft,
}

///
/// The `BitmapFontAtlasMetadata` struct holds all the information about the image
/// and every glyph in the font atlas, including where each glyph is located in the
/// atlas image for rendering text.
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BitmapFontAtlasMetadata {
    /// The origin of the image. This determines the coordinate system and orientation of the image.
    pub origin: Origin,
    /// The width and height of the image, in pixels.
    pub dimensions: usize,
    /// The number of glyphs per row in the atlas.
    pub columns: usize,
    /// The number of glyphs per column in the atlas.
    pub rows: usize,
    /// The number of pixels of padding from the edges of a glyph slot.
    pub padding: usize,
    /// The size of a glyph slot in the atlas in pixels.
    pub slot_glyph_size: usize,
    /// The size of a glyph inside a glyph slot, in pixels.
    pub glyph_size: usize,
    /// The table containing the metadata for each glyph.
    pub glyph_metadata: HashMap<usize, GlyphMetadata>,
}

///
/// A `BitmapFontAtlasImage` represents the underlying bitmapped image containing the
/// font glyph images.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitmapFontAtlasImage {
    /// The coordinate origin and coordinate basis for the image.
    origin: Origin,
    /// The width of the image, in pixels.
    width: usize,
    /// The height of the image, in pixels.
    height: usize,
    /// The underlying raw image data.
    data: Vec<u8>,
}

impl BitmapFontAtlasImage {
    pub fn new(data: Vec<u8>, width: usize, height: usize, origin: Origin) -> BitmapFontAtlasImage {
        BitmapFontAtlasImage {
            origin: origin,
            width: width,
            height: height,
            data: data,
        }
    }

    ///
    /// Return the width of the image in pixels.
    ///
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    ///
    /// Return the height of the image in pixels.
    ///
    #[inline]
    fn height(&self) -> usize {
        self.height
    }

    ///
    /// Return a pointer to the underlying image data.
    ///
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        &self.data[0]
    }

    ///
    /// The size of the image, in bytes.
    ///
    pub fn len_bytes(&self) -> usize {
        self.data.len()
    }
}

impl AsRef<[u8]> for BitmapFontAtlasImage {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

///
/// A `BitmapFontAtlas` is a bitmapped font sheet. It contains the glyph parameters necessary to
/// index into the bitmap image as well as the bitmap image itself.
///
pub struct BitmapFontAtlas {
    /// The origin of the image. This determines the coordinate system and orientation of the image.
    pub origin: Origin,
    /// The width and height of the image, in pixels.
    pub dimensions: usize,
    /// The number of glyphs per row in the atlas.
    pub columns: usize,
    /// The number of glyphs per column in the atlas.
    pub rows: usize,
    /// The number of pixels of padding from the edges of a glyph slot.
    pub padding: usize,
    /// The size of a glyph slot in the atlas in pixels.
    pub slot_glyph_size: usize,
    /// The size of a glyph inside a glyph slot, in pixels.
    pub glyph_size: usize,
    /// The table containing the metadata for each glyph.
    pub glyph_metadata: HashMap<usize, GlyphMetadata>,
    /// The array containing the font atlas image itself.
    pub image: BitmapFontAtlasImage,
}

impl BitmapFontAtlas {
    pub fn new(metadata: BitmapFontAtlasMetadata, image: BitmapFontAtlasImage) -> BitmapFontAtlas {
        BitmapFontAtlas {
            origin: metadata.origin,
            dimensions: metadata.dimensions,
            columns: metadata.columns,
            rows: metadata.rows,
            padding: metadata.padding,
            slot_glyph_size: metadata.slot_glyph_size,
            glyph_size: metadata.glyph_size,
            glyph_metadata: metadata.glyph_metadata,
            image: image,
        }
    }

    ///
    /// Generate the metadata for the font atlas.
    ///
    pub fn metadata(&self) -> BitmapFontAtlasMetadata {
        BitmapFontAtlasMetadata {
            origin: self.origin,
            dimensions: self.dimensions,
            columns: self.columns,
            rows: self.rows,
            padding: self.padding,
            slot_glyph_size: self.slot_glyph_size,
            glyph_size: self.glyph_size,
            glyph_metadata: self.glyph_metadata.clone(),
        }
    }
}

impl AsRef<[u8]> for BitmapFontAtlas {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.image.as_ref()
    }
}

struct BitmapFontAtlasBuilder {
    metadata: BitmapFontAtlasMetadata,
    image: BitmapFontAtlasImage,
}

impl BitmapFontAtlasBuilder {
    fn new(metadata: BitmapFontAtlasMetadata, image: BitmapFontAtlasImage) -> BitmapFontAtlasBuilder {
        BitmapFontAtlasBuilder {
            metadata: metadata,
            image: image,
        }
    }

    ///
    /// Build the font atlas and consume the builder.
    ///
    fn build(mut self) -> BitmapFontAtlas {
        // If the origin is declared as the bottom left, we must flip the image since the
        // PNG image format indexes the image starting from the top left corner
        // going right and downwards.
        if self.metadata.origin == Origin::BottomLeft {
            let height = self.image.height;
            let width_in_bytes = 4 * self.image.width;
            let half_height = self.image.height / 2;
            for row in 0..half_height {
                for col in 0..width_in_bytes {
                    let temp = self.image.data[row * width_in_bytes + col];
                    self.image.data[row * width_in_bytes + col] = self.image.data[((height - row - 1) * width_in_bytes) + col];
                    self.image.data[((height - row - 1) * width_in_bytes) + col] = temp;
                }
            }
        }

        BitmapFontAtlas::new(self.metadata, self.image)
    }
}

///
/// A `BmfaError` is an error typing representing the results of the failure of
/// a bmfa read or write operation.
///
pub struct Error {
    repr: Repr,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt( & self.repr, f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt( & self.repr, f)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    FileNotFound,
    FileExistsButCannotBeOpened,
    FontAtlasImageNotFound,
    CannotLoadAtlasImage,
    FontMetadataNotFound,
    CannotLoadAtlasMetadata,
}

impl ErrorKind {
    fn as_str(&self) -> &str {
        match *self {
            ErrorKind::FileNotFound => "File not found",
            ErrorKind::FileExistsButCannotBeOpened => "The file exists but cannot be opened",
            ErrorKind::FontAtlasImageNotFound => "The font atlas contains no atlas image",
            ErrorKind::CannotLoadAtlasImage => "The font atlas contains an atlas image but it cannot be loaded",
            ErrorKind::FontMetadataNotFound => "The font atlas contains no metadata",
            ErrorKind::CannotLoadAtlasMetadata => "The font atlas metadata is corrupt",
        }
    }
}

#[derive(Debug)]
struct Repr {
    kind: ErrorKind,
    error: Box<dyn error::Error + Send + Sync>,
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind_str = self.kind.as_str();
        write!(f, "{}: {}", kind_str, self.error)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&*self.repr.error)
    }
}

impl Error {
    pub fn new(kind: ErrorKind, error: Box<dyn error::Error + Send + Sync>) -> Error {
        Error {
            repr: Repr {
                kind: kind,
                error: error,
            }
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.repr.kind
    }
}

///
/// Read in a bitmap font atlas from an external source.
///
pub fn from_reader<R: io::Read + io::Seek>(reader: R) -> Result<BitmapFontAtlas, Error> {
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| {
        Error::new(ErrorKind::FileExistsButCannotBeOpened, Box::new(e))
    })?;
    let metadata_file = zip.by_name("metadata.json").map_err(|e| {
        Error::new(ErrorKind::FontMetadataNotFound, Box::new(e))
    })?;
    let metadata: BitmapFontAtlasMetadata = serde_json::from_reader(metadata_file).map_err(|e| {
        Error::new(ErrorKind::CannotLoadAtlasMetadata, Box::new(e))
    })?;
    let atlas_file = zip.by_name("atlas.png").map_err(|e| {
        Error::new(ErrorKind::FontAtlasImageNotFound, Box::new(e))
    })?;
    let png_reader = png::PNGDecoder::new(atlas_file).map_err(|e| {
        Error::new(ErrorKind::CannotLoadAtlasImage, Box::new(e))
    })?;
    let (width, height) = png_reader.dimensions();
    let image = png_reader.read_image().map_err(|e| {
        Error::new(ErrorKind::CannotLoadAtlasImage, Box::new(e))
    })?;
    let atlas_image = BitmapFontAtlasImage::new(
        image, width as usize, height as usize, metadata.origin
    );
    let builder = BitmapFontAtlasBuilder::new(metadata, atlas_image);

    Ok(builder.build())
}

///
/// Load a bitmap font atlas directly from a file.
///
pub fn load<P: AsRef<Path>>(path: P) -> Result<BitmapFontAtlas, Error> {
    let reader = File::open(&path).map_err(|e| {
        Error::new(ErrorKind::FileNotFound, Box::new(e))
    })?;

    from_reader(reader)
}

///
/// Write out of bitmap font atlas to a writer or buffer.
///
pub fn to_writer<W: io::Write + io::Seek>(writer: W, atlas: &BitmapFontAtlas) -> io::Result<()> {
    let mut zip_file = zip::ZipWriter::new(writer);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Write out the metadata.
    zip_file.start_file("metadata.json", options)?;
    serde_json::to_writer_pretty(&mut zip_file, &atlas.metadata())?;

    // if the origin is the bottom left of the image, we need to flip the image back over
    // before writing it out.
    let mut image = atlas.image.clone();
    if image.origin == Origin::BottomLeft {
        let height = image.height;
        let width_in_bytes = 4 * image.width;
        let half_height = image.height / 2;
        for row in 0..half_height {
            for col in 0..width_in_bytes {
                let temp = image.data[row * width_in_bytes + col];
                image.data[row * width_in_bytes + col] = image.data[((height - row - 1) * width_in_bytes) + col];
                image.data[((height - row - 1) * width_in_bytes) + col] = temp;
            }
        }
    }

    // Write out the atlas image.
    zip_file.start_file("atlas.png", options)?;
    let png_writer = png::PNGEncoder::new(&mut zip_file);
    png_writer.encode(
        image.as_ref(), atlas.dimensions as u32, atlas.dimensions as u32, ColorType::RGBA(8)
    )?;

    zip_file.finish()?;

    Ok(())
}

///
/// Write the bitmap font atlas to the disk.
///
pub fn write_to_file<P: AsRef<Path>>(path: P, atlas: &BitmapFontAtlas) -> io::Result<()> {
    // Set up the image zip archive.
    let mut file_path = path.as_ref().to_path_buf();
    file_path.set_extension("bmfa");
    let file = File::create(&file_path)?;

    // Write out the atlas contents.
    to_writer(file, atlas)
}
