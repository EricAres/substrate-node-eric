//编译条件 只有满足条件时才会编译
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[warn(unused_imports)]
//#![no_std]
use codec::{Encode, Decode};
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, ensure, StorageValue, dispatch, StorageMap, traits::Randomness, Parameter,
};
use sp_io::hashing::blake2_128;

use frame_system::ensure_signed;
use sp_std::vec::Vec;
//use pallet_balances as balances;
use sp_runtime::{DispatchError, traits::{AtLeast32Bit, Bounded}};
use frame_support::traits::{Currency, ReservableCurrency, Get};
//use pallet_randomness_collective_flip;
//use parity_scale_codec::Encode;
//use sp_core::blake2_128;

//id
//type KittyIndex = u32;

//DNA u8类型 长度为16的数组
#[derive(Encode, Decode)]
pub struct Kitty(
    pub [u8; 16]
);

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;


// 2. Configuration
// Configure the pallet by specifying the parameters and types on which it depends.
// 定义一个trait,trait的名字叫做Trait,这个trait继承自frame_system::Trait
pub trait Trait: frame_system::Trait {
    // Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;

    // 定义 KittyIndex 类型，要求实现指定的 trait
    // Parameter 表示可以用于函数参数传递
    // AtLeast32Bit 表示转换为 u32 不会造成数据丢失
    // Bounded 表示包含上界和下界
    // Default 表示有默认值
    // Copy 表示实现 Copy这个trait
    // 类型KiityIndex需要同时实现以下5种trait
    type KittyIndex: Parameter + AtLeast32Bit + Bounded + Default + Copy;

    //某一种货币，具体是什么货币则有runtime实例化设置
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    // 创建 Kitty 的时候，需要质押的代币数量
    type LockAmount: Get<BalanceOf<Self>>;
}

// 3. Storage
// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {

	//这个写法上是substrte特有的，不是rust语法，所以参照写即可
    trait Store for Module<T: Trait> as TemplateModule {

        // key是kitty每次新增的序列id, vulue 是DNA
    	pub Kitties get(fn kitties):map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;

		// KittiesCount这个存储单元用于存放一个计数器的值,每新增一个kitty计数器+1
    	pub KittiesCount get(fn kitties_count):T::KittyIndex;

		// key是序列，value是所有者的账号id， Option<T::AccountId>语法含义是什么
    	pub KittyOwners get(fn kitty_owner):map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;

		// 记录某个账号拥有的猫  双键映射map key1是拥有者账号id  key2是猫的序列id  value是猫的序列id
    	pub OwnerKitties get(fn owner_kitties):double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::KittyIndex => Option<T::KittyIndex>;

		// 记录某只猫的父母  单键映射map， key是猫的序列id  value是元组（父,母）
		pub KittyParents get(fn kitty_parents):map hasher(blake2_128_concat) T::KittyIndex => (T::KittyIndex, T::KittyIndex);

		// 记录某只猫的孩子们，双键映射map key1是主猫的id  key2是孩子，value也是孩子
		pub KittyChildren get(fn kitty_children):double_map hasher(blake2_128_concat) T::KittyIndex, hasher(blake2_128_concat) T::KittyIndex => Option<T::KittyIndex>;

		// 记录某只猫的伴侣，双键映射map key1是主猫，key2是伴侣猫，value是伴侣猫
		pub KittyPartners get(fn kitty_partners):double_map hasher(blake2_128_concat) T::KittyIndex, hasher(blake2_128_concat) T::KittyIndex => Option<T::KittyIndex>;

    }
}

// 4. Events
// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event! {

    // 这里的KittyIndex = <T as Trait>::KittyIndex的语法是什么写法？
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId,KittyIndex = <T as Trait>::KittyIndex {
		//
    	Created(AccountId, KittyIndex),

    	Transferred(AccountId, AccountId,KittyIndex),

    }
}

// 5. Errors
// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
		KittiesCountOverflow,
		InvalidKittyId,
		RequireDifferentParent,
		KittyNotExists,
		NotKittyOwner,
		MoneyNotEnough,
	    }
}

// 6. Callable Functions
// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {

	// 定义了一个结构体,名字为Module,这是一个带有泛型的结构体,泛型的要求是必须实现了Trait
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn create(orign){

			let sender = ensure_signed(orign)?;

			//质押币
			T::Currency::reserve(&sender, T::LockAmount::get())
					.map_err(|_| "locker can't afford to lock the amount requested")?;

			// Self是指调用方这个对象，这里因为当前这个方法create所在的struct同时是next_kitty_id所在impl实现的结构体Module
			let kitty_id = Self::next_kitty_id()?;

			// 获取一个DNA数组
			let dna = Self::random_value(&sender);

			// 实例化结构体Kitty
			let kitty = Kitty(dna);

			Self::insert_kitty(&sender, kitty_id, kitty);

			Self::deposit_event(RawEvent::Created(sender,kitty_id));

		}

		#[weight = 0]
		pub fn transfer(orign, to:T::AccountId, kitty_id:T::KittyIndex)
		{
			let sender = ensure_signed(orign)?;
			//如果这个kitty存在owner则返回一个Option,否则抛出KittyNotExists异常
			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::KittyNotExists)?;

			//option类型和sender可以直接对比？
			ensure!(sender == owner, Error::<T>::NotKittyOwner);

			// 质押被转让人的代币
			T::Currency::reserve(&to, T::LockAmount::get()).map_err(|_| Error::<T>::MoneyNotEnough )?;
			T::Currency::unreserve(&sender, T::LockAmount::get());

			//删除kitty-原owner的关系
			<KittyOwners<T>>::remove(kitty_id);
			//插入kitty-新owner的关系
			<KittyOwners<T>>::insert(kitty_id,&to);


			// OwnerKitties记录某个账号拥有的猫  双键映射map key1是拥有者账号id  key2是猫的序列id  value是猫的序列id
			//删除原来owner包含的kitty的数据
			//sender的类型是accountId,这个类型是继承自frame_system::Trait，由于这个类型不是基础类型，所以需要加&防止所有权转移
			<OwnerKitties<T>>::remove(&sender, kitty_id);
			//OwnedKitties::<T>::remove(&sender, kitty_id);

			<OwnerKitties<T>>::insert(&to, kitty_id, kitty_id);

			Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
		}

		#[weight = 0]
		pub fn breed(orign, kitty_id_1:T::KittyIndex, kitty_id_2:T::KittyIndex)
		{
			let sender = ensure_signed(orign)?;
			let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
			Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
		}

}


}

/// 这里的Module是应该是90行中decl_module中定义的的结构体module,这里是对结构体Module的方法实现，rust中结构体可以有多个impl
/// 结构体的方法实现并不需要像trait那样先进行方法定义
/// 这个结构体 有一个泛型T，这个范型要求实现trait,如果这个结构体的实例化对象没有实现Trait,则当前这个impl对这个结构体的实现无效,这就是有条件地实现结构体的方法
impl<T: Trait> Module<T> {
    //T::KittyIndex是指实现Trait这个trait的对象中的成员变量KittyIndex
    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {

        // Self代表的时候这个方法next_kitty_id()的调用方，KittiesCount这个存储单元在使用时有一个可选的方法kitties_count()
        let kitty_id = Self::kitties_count();

        // 计数器达到了i32的最大值则抛出异常
        if kitty_id == T::KittyIndex::max_value()
        {
            return Err(Error::<T>::KittiesCountOverflow.into());
        }

        Ok(kitty_id)
    }

    //生成一个u8类型长度为16的数组，算是一个伪随机的数
    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let paylaod = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        paylaod.using_encoded(blake2_128)
    }

    /// 插入kitty的值到存储单元
    /// @owner：调用者sender
    /// @kitty_id：是一个每次新增kitty都+1的计数器
    /// @kitty: kitty数据是一个包含一个数组的结构体，数据内容是伪随机数据DNA
    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty)
    {
        // 这里的存储单元为什么要这么写? into是什么方法
        <KittiesCount::<T>>::put(kitty_id + 1.into());
        <KittyOwners::<T>>::insert(kitty_id, owner);
        <Kitties::<T>>::insert(kitty_id, kitty);
        //拥有者 有哪些猫的存储单元
        <OwnerKitties::<T>>::insert(owner, kitty_id, kitty_id);
        //Kitties::insert(kitty_id, kitty);
        //KittiesCount::put(kitty_id + 1.into());
        //<KittyOwners<T>>::insert(kitty_id, owner);
    }


    fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
        (selector & dna1) | (!selector & dna2)
    }

    fn do_breed(sender: &T::AccountId, father_id: T::KittyIndex, mother_id: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError>
    {
        let father_dna = Self::kitties(father_id).ok_or(Error::<T>::InvalidKittyId)?;
        let mother_dna = Self::kitties(mother_id).ok_or(Error::<T>::InvalidKittyId)?;
        ensure!(father_id != mother_id, Error::<T>::RequireDifferentParent);

        let new_kitty_index = Self::next_kitty_id()?;
        let father_dna = father_dna.0;
        let mother_dna = mother_dna.0;
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8; 16];

        for i in 0..father_dna.len() {
            new_dna[i] = Self::combine_dna(father_dna[i], mother_dna[i], selector[i]);
        }

        Self::insert_kitty(sender, new_kitty_index, Kitty(new_dna));
        // 设置父母
        <KittyParents::<T>>::insert(new_kitty_index, (father_id, mother_id));

        // 记录某只猫的孩子们，双键映射map key1是主猫的id  key2是孩子，value也是孩子
        <KittyChildren::<T>>::insert(father_id, new_kitty_index, new_kitty_index);
        <KittyChildren::<T>>::insert(mother_id, new_kitty_index, new_kitty_index);

        // 记录某只猫的伴侣，双键映射map key1是主猫，key2是伴侣猫，value是伴侣猫
        <KittyPartners::<T>>::insert(father_id, mother_id, mother_id);
        <KittyPartners::<T>>::insert(mother_id, father_id, father_id);

        Ok(new_kitty_index)
    }
}
