mod helpers;

use helpers::prelude::*;

#[test]
fn test_password_retry_ok() {
    let rooster_file = tempfile();
    assert_eq!(
        0,
        main_with_args(
            &["rooster", "init", "--force-for-tests"],
            &mut CursorInput::new("\nxxxx\n"),
            &mut CursorOutput::new(),
            &rooster_file
        )
    );

    assert_eq!(
        0,
        main_with_args(
            &["rooster", "list"],
            &mut CursorInput::new("nok\nnok\nxxxx\n"),
            &mut CursorOutput::new(),
            &rooster_file
        )
    );
}

#[test]
fn test_password_retry_nok() {
    let rooster_file = tempfile();
    assert_eq!(
        0,
        main_with_args(
            &["rooster", "init", "--force-for-tests"],
            &mut CursorInput::new("\nxxxx\n"),
            &mut CursorOutput::new(),
            &rooster_file
        )
    );

    let mut output = CursorOutput::new();
    assert_eq!(
        1,
        main_with_args(
            &["rooster", "list"],
            &mut CursorInput::new("nok\nnok\nnok\n"),
            &mut output,
            &rooster_file
        )
    );
    let output_as_vecu8 = output.error_cursor.into_inner();
    let output_as_string = String::from_utf8_lossy(output_as_vecu8.as_slice());
    assert!(output_as_string.contains("Decryption of your Rooster file keeps failing"));
}
