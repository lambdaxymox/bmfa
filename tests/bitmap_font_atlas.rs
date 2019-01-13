use std::fs::File;
use std::path::Path;
use bmfa;
use zip::ZipArchive;

const SAMPLE_FILE: &str = "samples/freemono.bmfa";


///
/// Loading a bmfa file that does not exist should fail.
///
#[test]
fn loading_a_nonexistent_bmfa_file_should_fail() {
    let path = Path::new("DoesNotExist.bmfa");
    assert!(!path.exists());

    let maybe_atlas = bmfa::load(path);
    assert!(maybe_atlas.is_err());
}

///
/// Given a valid bmfa font file, the underlying zip archive storage should have
/// exactly two files: a png image containing all the glyphs and a json file containing
/// all the metadata.
///
#[test]
fn a_valid_bmfa_font_file_has_exactly_two_files() {
    let file = File::open(SAMPLE_FILE).unwrap();
    let zip_file = ZipArchive::new(file).unwrap();

    assert_eq!(zip_file.len(), 2);
}

///
/// Given a valid bmfa font file, the underlying zip archive storage should have
/// exactly two files: a png image and a json file containing all the metadata.
///
#[test]
fn a_valid_bmfa_file_has_exactly_one_metadata_file() {
    let file = File::open(SAMPLE_FILE).unwrap();
    let mut zip_file = ZipArchive::new(file).unwrap();
    let metadata_file = zip_file.by_name("metadata.json");

    assert!(metadata_file.is_ok());
}

///
/// Given a valid bmfa font file, the underlying zip archive storage should have
/// exactly two files: a png image and a json file containing all the metadata.
///
#[test]
fn a_valid_bmfa_file_has_exactly_one_image_file() {
    let file = File::open(SAMPLE_FILE).unwrap();
    let mut zip_file = ZipArchive::new(file).unwrap();
    let atlas_file = zip_file.by_name("atlas.png");

    assert!(atlas_file.is_ok());
}

///
/// Given a valid bmfa font file, the loader should succeed in loading it.
///
#[test]
fn bmfa_loader_should_load_valid_bmfa_file() {
    let font_atlas = bmfa::load(SAMPLE_FILE);

    assert!(font_atlas.is_ok());
}

///
/// A valid bmfa file's dimensions should match the length of the underlying buffer.
/// That is, the width and height of the image in the metadata should satisfy the relation
/// ```
/// 4 * height * width == buffer length
/// ```
///
#[test]
fn bmfa_file_dimensions_should_match_buffer_length() {
    let font_atlas = bmfa::load(SAMPLE_FILE).unwrap();
    let expected = font_atlas.image.len();
    let result = 4 * font_atlas.dimensions * font_atlas.dimensions;

    assert_eq!(result, expected);
}

///
/// A valid bmfa file's dimensions, in units of pixels, should match satisfy the following
/// relation
/// ```
/// width == columns * slot glyph size
/// ```
/// That is, the width of the image should align with the column could and the slot glyph size.
///
#[test]
fn bmfa_file_dimensions_should_match_width() {
    let font_atlas = bmfa::load(SAMPLE_FILE).unwrap();
    let expected = font_atlas.columns * font_atlas.slot_glyph_size;
    let result = font_atlas.dimensions;

    assert_eq!(result, expected);
}

///
/// A valid bmfa file's dimensions, in units of pixels, should match satisfy the following
/// relation
/// ```
/// height == rows * slot glyph size
/// ```
/// That is, the height of the image should align with the row count and the slot glyph size.
///
#[test]
fn bmfa_file_dimensions_should_match_height() {
    let font_atlas = bmfa::load(SAMPLE_FILE).unwrap();
    let expected = font_atlas.rows * font_atlas.slot_glyph_size;
    let result = font_atlas.dimensions;

    assert_eq!(result, expected);
}

///
/// The slot glyph size in the font atlas metadata should satisfy the following relation.
/// ```
/// slot glyph size == padding + glyph size
/// ```
/// Here, the slot glyph size is the size of the slot that a glyph is stored in. The padding is the
/// offset from the edges of the boundary box inside of which the glyph is stored, and the glyph size
/// is the size of the glyph image in pixels.
///
#[test]
fn bmfa_file_slot_glyph_size_should_be_sum_of_padding_and_glyph_size() {
    let font_atlas = bmfa::load(SAMPLE_FILE).unwrap();
    let expected = font_atlas.padding + font_atlas.glyph_size;
    let result = font_atlas.slot_glyph_size;

    assert_eq!(result, expected);
}

