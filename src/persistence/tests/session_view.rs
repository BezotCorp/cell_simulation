use crate::persistence::SessionView;

#[test]
fn non_finite_saved_speed_is_rejected() {
    let view = SessionView::default();
    let text = ron::ser::to_string(&view).unwrap();
    assert!(text.contains("speed:1.0"));
    let invalid: SessionView = ron::from_str(&text.replace("speed:1.0", "speed:NaN")).unwrap();
    assert!(invalid.validate().is_err());
}

#[test]
fn speed_commands_preserve_the_supported_range() {
    let mut view = SessionView::default();
    for _ in 0..20 {
        view.faster();
    }
    assert_eq!(view.speed(), 64.0);
    for _ in 0..20 {
        view.slower();
    }
    assert_eq!(view.speed(), 0.25);
    assert!(view.validate().is_ok());
}
