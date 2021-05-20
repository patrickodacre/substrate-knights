use crate::{mock::*, Error};
use frame_support::{assert_err, assert_noop, assert_ok};

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

        let knight_ids = KnightModule::owner_to_knights(&1).unwrap();
        assert_eq!(knight_ids.len(), 1);

        let owner_knight_count = KnightModule::owner_to_knight_count(&1);
        assert_eq!(owner_knight_count, 1);
    });
}

#[test]
fn can_set_price_for_knight() {
    new_test_ext().execute_with(|| {
        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Sir Rowan".as_bytes().to_vec()
        ));

        KnightModule::set_price(Origin::signed(1), 1, 100).expect("cannot set price");
        let sir_rowan = KnightModule::knights(1).unwrap();
        assert_eq!(sir_rowan.price, 100);

        KnightModule::set_price(Origin::signed(1), 1, 0).expect("cannot set price");
        let sir_rowan = KnightModule::knights(1).unwrap();
        assert_eq!(sir_rowan.price, 0);
    });
}

#[test]
fn non_owner_cannot_set_price_for_knight() {
    new_test_ext().execute_with(|| {
        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Sir Cedric".as_bytes().to_vec(),
        ));

        let check = Origin::signed(1000);

        assert_err!(
            KnightModule::set_price(Origin::signed(10000), 1, 200),
            Error::<Test>::NotRightfulOwner
        );
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
fn can_get_knights_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Evan the Bold".as_bytes().to_vec()
        ));

        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Daniel the Courageous".as_bytes().to_vec()
        ));

        let owner = KnightModule::knight_to_owner(&1).unwrap();
        assert_eq!(owner, 1);

        let owner = KnightModule::knight_to_owner(&2).unwrap();
        assert_eq!(owner, 1);
    });
}

#[test]
fn can_get_owners_knights() {
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

        let knights = KnightModule::owner_to_knights(&1).unwrap();

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

#[test]
fn knight_has_unique_id_even_with_identical_names_and_owners() {
    new_test_ext().execute_with(|| {
        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Sir Evan".as_bytes().to_vec()
        ));

        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Sir Evan".as_bytes().to_vec()
        ));

        let sir_evan1 = KnightModule::knights(&1).unwrap();
        let sir_evan2 = KnightModule::knights(&2).unwrap();

        assert_ne!(sir_evan1.id, sir_evan2.id);
        assert_ne!(sir_evan1.dna, sir_evan2.dna);
    });
}

#[test]
fn can_transfer_knight() {
    new_test_ext().execute_with(|| {
        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Beric the Briton".as_bytes().to_vec()
        ));

        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Sir Rowan of Chessington".as_bytes().to_vec()
        ));

        let owner = KnightModule::knight_to_owner(&1).unwrap();
        assert_eq!(owner, 1);
        let knights = KnightModule::owner_to_knights(&1).unwrap();
        assert_eq!(knights.len(), 2);
        let knights = KnightModule::owner_to_knights(&2).unwrap_or(Vec::<u64>::new());
        assert_eq!(knights.len(), 0);

        KnightModule::transfer_knight(Origin::signed(1), 1, 2).unwrap();

        let owner = KnightModule::knight_to_owner(&1).unwrap();
        assert_eq!(owner, 2);
        let knights = KnightModule::owner_to_knights(&1).unwrap();
        assert_eq!(knights.len(), 1);
        let knights = KnightModule::owner_to_knights(&2).unwrap();
        assert_eq!(knights.len(), 1);
    });
}

#[test]
fn non_owner_cannot_transfer_knight() {
    new_test_ext().execute_with(|| {
        assert_ok!(KnightModule::create_knight(
            Origin::signed(1),
            "Alfred the Great".as_bytes().to_vec()
        ));

        assert_err!(
            KnightModule::transfer_knight(Origin::signed(2), 1, 2),
            Error::<Test>::NotRightfulOwner
        );
    });
}

#[test]
fn cannot_transfer_non_existant_knight() {
    new_test_ext().execute_with(|| {
        assert_err!(
            KnightModule::transfer_knight(Origin::signed(1), 1, 2),
            Error::<Test>::KnightNotFound
        );
    });
}
