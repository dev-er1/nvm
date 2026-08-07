// Тесты на парсинг и валидацию версии `.nb`-файлов.
use nvm_core::loader::err::LoaderErrorKind;

use super::*;

#[test]
fn current_version_parses_ok() {
    let result = run_loader(make_nb_with_version(0, 1, 0, &nop_bytes()));
    assert!(result.is_ok());
}

#[test]
fn older_version_works() {
    let result = run_loader(make_nb_with_version(0, 0, 0, &nop_bytes()));
    assert!(result.is_ok());
}

#[test]
fn older_patch_version_works() {
    let result = run_loader(make_nb_with_version(0, 0, 5, &nop_bytes()));
    assert!(result.is_ok());
}

#[test]
fn newer_major_version_fails() {
    let err = match run_loader(make_nb_with_version(1, 0, 0, &nop_bytes())) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnsupportedVersion { .. }
    ));
}

#[test]
fn newer_minor_version_fails() {
    let err = match run_loader(make_nb_with_version(0, 2, 0, &nop_bytes())) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnsupportedVersion { .. }
    ));
}

#[test]
fn newer_patch_version_fails() {
    let err = match run_loader(make_nb_with_version(0, 1, 3, &nop_bytes())) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnsupportedVersion { .. }
    ));
}


#[test]
fn major_version_zero_with_high_minor_patch_works() {
    let result = run_loader(make_nb_with_version(0, 0, 0, &nop_bytes()));
    assert!(result.is_ok());
}

#[test]
fn version_string_format_is_correct() {
    let result = run_loader(make_nb_with_version(0, 0, 0, &[]));
    assert!(result.is_ok());
}

#[test]
fn version_with_large_numbers_in_file_fails() {
    // "99.99.99" > "0.1.0" как строки -> ошибка версии.
    let err = match run_loader(make_nb_with_version(99, 99, 99, &nop_bytes())) {
        Err(e) => e,
        Ok(_) => panic!("expected loader error"),
    };

    assert!(matches!(
        err.kind,
        LoaderErrorKind::UnsupportedVersion { .. }
    ));
}

#[test]
fn version_parsed_correctly_from_bytes() {
    let data = make_nb_with_version(0, 0, 0, &[]);
    // major = 0x00 0x00, minor = 0x00 0x00, patch = 0x00 0x00
    assert_eq!(&data[5..7], &[0x00, 0x00]);
    assert_eq!(&data[7..9], &[0x00, 0x00]);
    assert_eq!(&data[9..11], &[0x00, 0x00]);
}
