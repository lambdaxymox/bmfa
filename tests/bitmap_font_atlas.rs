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
