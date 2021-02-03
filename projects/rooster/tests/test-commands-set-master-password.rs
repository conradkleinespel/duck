mod helpers;

use crate::helpers::prelude::*;

#[test]
fn test_command_set_master_password() {
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
            &["rooster", "set-master-password"],
            &mut CursorInput::new("xxxx\nabcd\nabcd\n"),
            &mut CursorOutput::new(),
            &rooster_file
        )
    );

    assert_eq!(
        1,
        main_with_args(
            &["rooster", "list"],
            &mut CursorInput::new("xxxx\n"),
            &mut CursorOutput::new(),
            &rooster_file
        )
    );
    assert_eq!(
        0,
        main_with_args(
            &["rooster", "list"],
            &mut CursorInput::new("abcd\n"),
            &mut CursorOutput::new(),
            &rooster_file
        )
    );
}
