use std::fs::File;
use std::path::Path;
use bmfa;
use zip::ZipArchive;

const SAMPLE_FILE: &str = "samples/freemono.bmfa";


#[test]
fn loading_a_nonexistent_bmfa_file_should_fail() {
    let path = Path::new("DoesNotExist.bmfa");
    assert!(!path.exists());

    let maybe_atlas = bmfa::load(path);
    assert!(maybe_atlas.is_err());
}

#[test]
fn a_valid_bmfa_font_file_has_exactly_two_files() {
    let file = File::open(SAMPLE_FILE).unwrap();
    let zip_file = ZipArchive::new(file).unwrap();

    assert_eq!(zip_file.len(), 2);
}

#[test]
fn a_valid_bmfa_file_has_exactly_one_metadata_file() {
    let file = File::open(SAMPLE_FILE).unwrap();
    let mut zip_file = ZipArchive::new(file).unwrap();
    let metadata_file = zip_file.by_name("metadata.json");

    assert!(metadata_file.is_ok());
}

#[test]
fn a_valid_bmfa_file_has_exactly_one_image_file() {
    let file = File::open(SAMPLE_FILE).unwrap();
    let mut zip_file = ZipArchive::new(file).unwrap();
    let atlas_file = zip_file.by_name("atlas.png");

    assert!(atlas_file.is_ok());
}

#[test]
fn bmfa_loader_should_load_valid_bmfa_file() {
    let font_atlas = bmfa::load(SAMPLE_FILE);

    assert!(font_atlas.is_ok());
}

#[test]
fn bmfa_file_dimensions_should_match_buffer_length() {
    let font_atlas = bmfa::load(SAMPLE_FILE).unwrap();
    let expected = font_atlas.buffer.len();
    let result = 4 * font_atlas.metadata.dimensions * font_atlas.metadata.dimensions;

    assert_eq!(result, expected);
}

#[test]
fn bmfa_file_dimensions_should_match_width() {
    let font_atlas = bmfa::load(SAMPLE_FILE).unwrap();
    let expected = font_atlas.metadata.columns * font_atlas.metadata.slot_glyph_size;
    let result = font_atlas.metadata.dimensions;

    assert_eq!(result, expected);
}

#[test]
fn bmfa_file_dimensions_should_match_height() {
    let font_atlas = bmfa::load(SAMPLE_FILE).unwrap();
    let expected = font_atlas.metadata.rows * font_atlas.metadata.slot_glyph_size;
    let result = font_atlas.metadata.dimensions;

    assert_eq!(result, expected);
}

#[test]
fn bmfa_file_slot_glyph_size_should_be_sum_of_padding_and_glyph_size() {
    let font_atlas = bmfa::load(SAMPLE_FILE).unwrap();
    let expected = font_atlas.metadata.padding + font_atlas.metadata.glyph_size;
    let result = font_atlas.metadata.slot_glyph_size;

    assert_eq!(result, expected);
}

