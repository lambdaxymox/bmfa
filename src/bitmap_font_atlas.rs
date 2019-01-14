use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
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
    ///
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
        code_point: usize,
        width: f32, height: f32,
        x_min: f32, y_min: f32, y_offset: f32) -> GlyphMetadata {

        GlyphMetadata {
            code_point: code_point,
            width: width,
            height: height,
            x_min: x_min,
            y_min: y_min,
            y_offset: y_offset,
        }
    }
}

///
/// The `BitmapFontAtlasMetadata` struct holds all the information about the image
/// and every glyph in the font atlas, including where each glyph is located in the
/// atlas image for rendering text.
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BitmapFontAtlasMetadata {
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
/// A `BitmapFontAtlas` is a bitmapped font sheet. It contains the glyph parameters necessary to
/// index into the bitmap image as well as the bitmap image itself.
///
pub struct BitmapFontAtlas {
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
    pub image: Vec<u8>,
}

impl BitmapFontAtlas {
    pub fn new(metadata: BitmapFontAtlasMetadata, image: Vec<u8>) -> BitmapFontAtlas {
        BitmapFontAtlas {
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

    pub fn metadata(&self) -> BitmapFontAtlasMetadata {
        BitmapFontAtlasMetadata {
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
    pub fn new(kind: ErrorKind, error: Box<dyn error::Error+Send+Sync>) -> Error {
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
    let metadata = serde_json::from_reader(metadata_file).map_err(|e| {
        Error::new(ErrorKind::CannotLoadAtlasMetadata, Box::new(e))
    })?;
    let atlas_file = zip.by_name("atlas.png").map_err(|e| {
        Error::new(ErrorKind::FontAtlasImageNotFound, Box::new(e))
    })?;
    let png_reader = png::PNGDecoder::new(atlas_file).map_err(|e| {
        Error::new(ErrorKind::CannotLoadAtlasImage, Box::new(e))
    })?;
    let atlas_image = png_reader.read_image().map_err(|e| {
        Error::new(ErrorKind::CannotLoadAtlasImage, Box::new(e))
    })?;

    Ok(BitmapFontAtlas::new(metadata, atlas_image))
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

    // Write out the atlas image.
    zip_file.start_file("atlas.png", options)?;
    let png_writer = png::PNGEncoder::new(&mut zip_file);
    png_writer.encode(
        &atlas.image, atlas.dimensions as u32, atlas.dimensions as u32, ColorType::RGBA(8)
    )?;

    zip_file.finish()?;

    Ok(())
}

///
/// Write the bitmap font atlas to the disk.
///
pub fn write_to_file<P: AsRef<Path>>(atlas: &BitmapFontAtlas, path: P) -> io::Result<()> {
    // Set up the image archive.
    let mut file_path = path.as_ref().to_path_buf();
    file_path.set_extension("bmfa");
    let file = File::create(&file_path)?;

    to_writer(file, atlas)
}
