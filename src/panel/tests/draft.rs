use crate::definitions::Definitions;
use crate::panel::Draft;

#[test]
fn the_draft_knows_when_it_differs_from_the_simulation() {
    let text = ron::ser::to_string(&*Definitions::builtin()).unwrap();
    let mut draft = Draft::from_simulation(text.clone()).unwrap();
    assert!(!draft.differs());
    draft.definitions.species_mut().parts_mut().pop();
    assert!(draft.differs());
    draft.synchronize(draft.text());
    assert!(!draft.differs());
    assert!(Draft::from_simulation("(pas du ron".into()).is_err());
}
