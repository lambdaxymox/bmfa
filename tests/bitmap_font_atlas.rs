use std::fs;
use std::fs::File;
use std::path::Path;
use bmfa;
use image::png;
use zip;

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
    let zip_file = zip::ZipArchive::new(file).unwrap();

    assert_eq!(zip_file.len(), 2);
}

///
/// Given a valid bmfa font file, the underlying zip archive storage should have
/// exactly two files: a png image and a json file containing all the metadata.
///
#[test]
fn a_valid_bmfa_file_has_exactly_one_metadata_file() {
    let file = File::open(SAMPLE_FILE).unwrap();
    let mut zip_file = zip::ZipArchive::new(file).unwrap();
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
    let mut zip_file = zip::ZipArchive::new(file).unwrap();
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


#[test]
fn bmfa_file_should_write_to_disk_successfully() {
    let font_atlas = bmfa::load(SAMPLE_FILE).unwrap();
    let path = Path::new("atlas.bmfa");
    let result = bmfa::write_to_file(&font_atlas, path);
    fs::remove_file(path).unwrap();

    assert!(result.is_ok());
}


struct ReadWriteTest {
    expected_path: Box<Path>,
    expected_atlas: bmfa::BitmapFontAtlas,
    result_path: Box<Path>,
    result_atlas: bmfa::BitmapFontAtlas,
}

impl ReadWriteTest {
    fn new(
        expected_path: &Path, expected_atlas: bmfa::BitmapFontAtlas,
        result_path: &Path, result_atlas: bmfa::BitmapFontAtlas) -> ReadWriteTest {

        ReadWriteTest {
            expected_path: Box::from(expected_path),
            expected_atlas: expected_atlas,
            result_path: Box::from(result_path),
            result_atlas: result_atlas,
        }
    }
}

fn read_write_test() -> ReadWriteTest {
    let expected_path = Path::new(SAMPLE_FILE);
    let expected_atlas = bmfa::load(expected_path).unwrap();
    let result_path = Path::new("atlas.bmfa");
    bmfa::write_to_file(&expected_atlas, result_path).unwrap();
    let result_atlas = bmfa::load(result_path).unwrap();
    fs::remove_file(result_path).unwrap();

    ReadWriteTest::new(expected_path, expected_atlas, result_path, result_atlas)
}

#[test]
fn bmfa_file_written_and_then_read_should_match_dimensions() {
    let test = read_write_test();

    assert_eq!(test.result_atlas.dimensions, test.expected_atlas.dimensions);
}

#[test]
fn bmfa_file_written_and_then_read_should_match_columns() {
    let test = read_write_test();

    assert_eq!(test.result_atlas.columns, test.expected_atlas.columns);
}

#[test]
fn bmfa_file_written_and_then_read_should_match_rows() {
    let test = read_write_test();

    assert_eq!(test.result_atlas.rows, test.expected_atlas.rows);
}

#[test]
fn bmfa_file_written_and_then_read_should_match_padding() {
    let test = read_write_test();

    assert_eq!(test.result_atlas.padding, test.expected_atlas.padding);
}

#[test]
fn test_file_written_and_then_read_should_match_slot_glyph_size() {
    let test = read_write_test();

    assert_eq!(test.result_atlas.slot_glyph_size, test.expected_atlas.slot_glyph_size);
}

#[test]
fn bmfa_file_written_and_then_read_should_match_glyph_size() {
    let test = read_write_test();

    assert_eq!(test.result_atlas.glyph_size, test.expected_atlas.glyph_size);
}

#[test]
fn bmfa_file_written_and_then_read_should_match_metadata() {
    let test = read_write_test();

    assert_eq!(test.result_atlas.metadata(), test.expected_atlas.metadata());
}

#[test]
fn bmfa_file_written_and_then_read_should_match_glyph_metadata() {
    let test = read_write_test();

    assert_eq!(test.result_atlas.glyph_metadata, test.expected_atlas.glyph_metadata);
}

#[test]
fn bmfa_file_written_and_then_read_should_match_atlases() {
    let test = read_write_test();

    assert_eq!(test.result_atlas.image, test.expected_atlas.image);
}
