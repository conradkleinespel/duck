mod helpers;

use helpers::prelude::*;

#[test]
fn test_command_regenerate() {
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
            &["rooster", "generate", "-s", "Youtube", "yt@example.com"],
            &mut CursorInput::new("xxxx\n"),
            &mut CursorOutput::new(),
            &rooster_file
        )
    );

    // Password exists
    assert_eq!(
        1,
        main_with_args(
            &["rooster", "generate", "-s", "Youtube", "yt@example.com"],
            &mut CursorInput::new("xxxx\n"),
            &mut CursorOutput::new(),
            &rooster_file
        )
    );

    let mut output_1 = CursorOutput::new();
    assert_eq!(
        0,
        main_with_args(
            &["rooster", "get", "-s", "youtube"],
            &mut CursorInput::new("xxxx\n"),
            &mut output_1,
            &rooster_file
        )
    );
    let output_1_as_vecu8 = output_1.standard_cursor.into_inner();
    let output_1_as_string = String::from_utf8_lossy(output_1_as_vecu8.as_slice());

    assert_eq!(
        0,
        main_with_args(
            &["rooster", "regenerate", "-s", "Youtube"],
            &mut CursorInput::new("xxxx\n"),
            &mut CursorOutput::new(),
            &rooster_file
        )
    );

    let mut output_2 = CursorOutput::new();
    assert_eq!(
        0,
        main_with_args(
            &["rooster", "get", "-s", "youtube"],
            &mut CursorInput::new("xxxx\n"),
            &mut output_2,
            &rooster_file
        )
    );
    let output_2_as_vecu8 = output_2.standard_cursor.into_inner();
    let output_2_as_string = String::from_utf8_lossy(output_2_as_vecu8.as_slice());

    assert_ne!(output_1_as_string, output_2_as_string);
}
