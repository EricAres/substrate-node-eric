#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode,Decode};
use frame_support::{
	decl_module,
	decl_storage,
	decl_event,
    decl_error,
    ensure,
    StorageValue,
    StorageMap,
    traits::Randomness,
	dispatch::{DispatchResult},
};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::DispatchError;

type KittyIndex =u32;
#[derive(Encode,Decode)]
pub struct Kitty(pub [u8;16]);

pub trait Trait: system::Trait {
	/// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Randomness:Randomness<Self::Hash>;
}

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
        pub Kitties get(fn kitties):map hasher(blake2_128_concat) KittyIndex=>Option<Kitty>;
        pub KittiesCount get(fn kitties_count):KittyIndex;
        pub KittyOwners get(fn kitty_owner):map hasher(blake2_128_concat) KittyIndex=>Option<T::AcountId>;
    }
	
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		Created(AccountId,KittyIndex),
		Transferred(AccountId,AccountId,KittyIndex),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
	    KittysCountOverflow,
        InvalidKittyId,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        #[weight=0]
        pub fn create(origin){
            let sender=ensure_signed(origin)?;
            let kitty_id=Self::next_kitty_id()?;
            let dna=Self::random_value(&sender);
            let kitty = Kitty(dna);
            Self::insert_kitty(&sender,kitty_id,kitty);
            Self::deposit_event(RawEvent::Created(sender,kitty_id));
        }

        #[weight=0]
        pub fn transfer(origin,to:T::AccountId,kitty_id:KittyIndex){
            let sender=ensure_signed(origin)?;
            <KittyOwners<T>>::insert(kitty_id,to.clone());
            Self::desposit_event(RawEvent::Transferred(sender,to,kitty_id));
        }
        #[weight=0]
        pub fn breed(origin,kitty_id_1:KittyIndex,kitty_id_2:KittyIndex){
            let sender = ensure_signed(origin)?;
            let new_kitty_id=Self::do_breed(&sender,new_kitty_id);

        }
    }
}

impl<T:Trait>Module<T>{
    fn next_kitty_id() ->sp_std::result::Result<KittyIndex,DispatchError>{
        let kitty_id=Self::kittys_count();
        if kitty_id==KittyIndex::max_value(){
            return Err(Error::<T>::KittysCountOverflow.info());
        }
        Ok(kitty_id)
    }
    #[weitht=0]
    fn radom_value(sender:&T::AccountId)->[u8;16]{
        let payload= {
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        };
        payload.using_encoded(blake2_128)

    }
    fn insert_kitty(owner:&T::AccountId,kitty_id:KittyIndex,kitty:Kitty)
    {
        Kitties::insert(kitty_id,kitty);
        KittiesCount::put(kitty_id+1);
        <KittyOwners<T>>::insert(kitty_id,owner);
    }
    fn do_breed(sender:&T::AccountId,kitty_id_1:KittyIndex,kitty_id_2:KittyIndex)-> sp_std::result::Result<Kitty> {
        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;
        ensure!(kitty_id_1 != kitty_id_2,Error::<T>::RequireDifferentParent);

        let kitty_id = Self::next_kitty_id()?;
        let kitty1_dna = kitty1.0;
        let kitty2_dna = kitty2.0;
        let selector = Self::randome_value(&sender);
        let mut new_dna= [0u8, 16];

        for i in 0..kitty1_dna.len{
            new_dna[i] = combine.dna(kitty1_dna[i], selector[i]);
        }
        Self::insert_kitty(sender,kitty_id,Kitty(new_dna));
        Ok(kitty_id)
    }
    fn combine_dna(dna1:u8,dna2:u8,Seletor:u8)->u8{
        (Seletor & dna1)|(Seletor & dna2)
    }
}
#[cfg(test)]
mod tests{
    use super::*;
}