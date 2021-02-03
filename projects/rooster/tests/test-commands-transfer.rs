mod helpers;

use crate::helpers::prelude::*;

#[test]
fn test_command_transfer() {
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

    assert_eq!(
        0,
        main_with_args(
            &["rooster", "transfer", "youtube", "videos@example.com"],
            &mut CursorInput::new("xxxx\n"),
            &mut CursorOutput::new(),
            &rooster_file
        )
    );

    let mut output = CursorOutput::new();
    assert_eq!(
        0,
        main_with_args(
            &["rooster", "list"],
            &mut CursorInput::new("xxxx\n"),
            &mut output,
            &rooster_file
        )
    );
    let output_as_vecu8 = output.standard_cursor.into_inner();
    let output_as_string = String::from_utf8_lossy(output_as_vecu8.as_slice());
    assert!(output_as_string.contains("Youtube"));
    assert!(!output_as_string.contains("yt@example.com"));
    assert!(output_as_string.contains("videos@example.com"));
}
