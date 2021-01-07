use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn create_claim_works(){
    new_test_ext().execute_with(execute: ||{
        let claim: Vec<i32> = vec![0,1];
        assert_ok!(PoeModule::create_claim(Origin::signed(1),claim.clone()));

        assert_eq!(Proofs::<Test>::get(&claim),(1,frame_system::Module::<Test>::block_number));

    })
}

#[test]
fn create_claim_failed_when_claim_already_exist(){
    new_test_ext().execute_with(execute: ||{
        let claim: Vec<u8> = vec![0,1];
        let _=PoeMoule::create_claim(origin: Origin::signed(1),claim.clone());
    })
}