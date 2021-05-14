use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn can_create_knight() {
    new_test_ext().execute_with(|| {
        let name = "Danny the Brave";
        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            name.as_bytes().to_vec()
        ));

        let k = KnightModule::knights(&1).unwrap();

        assert_eq!(k.name, name.as_bytes().to_vec());
    });
}

#[test]
fn can_get_knight_count() {
    new_test_ext().execute_with(|| {
        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Evan the Great".as_bytes().to_vec()
        ));

        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Christian the Fearless".as_bytes().to_vec()
        ));

        let count = KnightModule::knight_count().unwrap();

        assert_eq!(count, 2);
    });
}

#[test]
fn can_get_knights_creator() {
    new_test_ext().execute_with(|| {
        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Evan the Bold".as_bytes().to_vec()
        ));

        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Daniel the Courageous".as_bytes().to_vec()
        ));

        let creator = KnightModule::knight_to_creator(&1).unwrap();
        assert_eq!(creator, 1);

        let creator = KnightModule::knight_to_creator(&2).unwrap();
        assert_eq!(creator, 1);
    });
}

#[test]
fn can_get_creators_knights() {
    new_test_ext().execute_with(|| {
        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Sir Christian".as_bytes().to_vec()
        ));

        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Sir Daniel".as_bytes().to_vec()
        ));

        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Sir Evan".as_bytes().to_vec()
        ));

        let knights = KnightModule::creator_to_knights(&1).unwrap();

        assert_eq!(knights.len(), 3);

        let sir_christian_id = knights.get(0).unwrap();
        assert_eq!(sir_christian_id, &1);
        let sir_christian = KnightModule::knights(sir_christian_id).unwrap();
        assert_eq!(sir_christian.name, "Sir Christian".as_bytes().to_vec());

        let sir_daniel_id = knights.get(1).unwrap();
        assert_eq!(sir_daniel_id, &2);
        let sir_daniel = KnightModule::knights(sir_daniel_id).unwrap();
        assert_eq!(sir_daniel.name, "Sir Daniel".as_bytes().to_vec());

        let sir_evan_id = knights.get(2).unwrap();
        assert_eq!(sir_evan_id, &3);
        let sir_evan = KnightModule::knights(sir_evan_id).unwrap();
        assert_eq!(sir_evan.name, "Sir Evan".as_bytes().to_vec());
    });
}
